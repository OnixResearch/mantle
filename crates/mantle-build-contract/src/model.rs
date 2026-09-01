#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the closed wire model keeps owned interchange types in one reviewable module"
)]

use alloc::string::String;
use alloc::vec::Vec;

pub const REQUEST_SCHEMA: &str = "mantle.build-request.v1";
pub const OBSERVATION_SCHEMA: &str = "mantle.build-observation.v1";
pub const RECEIPT_SCHEMA: &str = "mantle.build-receipt.v1";
pub const NATIVE_REPORT_SCHEMA: &str = "crunch-build-report-v1";
pub const BLAKE3_PREFIX: &str = "b3:";
pub const BLAKE3_HEX_LENGTH: usize = 64;
pub const MAXIMUM_LABEL_BYTES: usize = 128;
pub const MAXIMUM_REQUIRED_PRODUCTS: usize = 256;
pub const MAXIMUM_PRODUCTS: usize = 256;
pub const MAXIMUM_LOGS: usize = 256;
pub const MAXIMUM_METRICS: usize = 256;

pub const REQUIRED_NON_CLAIMS: [&str; 6] = [
    "build observation does not prove build correctness",
    "build observation does not prove sandbox completeness",
    "cache observation does not prove cache truth",
    "product admission does not prove product semantics",
    "build observation does not prove reproducibility",
    "build observation does not establish release readiness",
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity(String);

impl Identity {
    pub fn new(value: String) -> Result<Self, ValueError> {
        let digest = value.strip_prefix(BLAKE3_PREFIX);
        let expected_identity_bytes =
            BLAKE3_PREFIX.len().checked_add(BLAKE3_HEX_LENGTH).ok_or(ValueError::InvalidIdentity)?;
        if value.len() == expected_identity_bytes
            && digest
                .is_some_and(|digest| digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
        {
            Ok(Self(value))
        } else {
            Err(ValueError::InvalidIdentity)
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn from_digest(value: blake3::Hash) -> Self {
        let mut identity = String::from(BLAKE3_PREFIX);
        identity.push_str(value.to_hex().as_str());
        Self(identity)
    }
}

impl serde::Serialize for Identity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Identity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(String);

impl Label {
    pub fn new(value: String) -> Result<Self, ValueError> {
        if value.is_empty() || value.len() > MAXIMUM_LABEL_BYTES {
            return Err(ValueError::InvalidLabel);
        }
        let has_valid_characters = value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b':' | b'-')
        });
        if !has_valid_characters {
            return Err(ValueError::InvalidLabel);
        }
        if !value.as_bytes().first().is_some_and(u8::is_ascii_lowercase) {
            return Err(ValueError::InvalidLabel);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl serde::Serialize for Label {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Label {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueError {
    InvalidIdentity,
    InvalidLabel,
}

impl core::fmt::Display for ValueError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidIdentity => "value is not a canonical BLAKE3 identity",
            Self::InvalidLabel => "value is not a canonical bounded label",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildOutcome {
    Success,
    Failed,
    Cancelled,
    TimedOut,
    Unknown,
}

impl BuildOutcome {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheKind {
    None,
    Local,
    Substitution,
}

impl CacheKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Local => "local",
            Self::Substitution => "substitution",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildRequest {
    pub schema: String,
    pub identity: Identity,
    pub idempotency_key: Identity,
    pub effect_identity: Identity,
    pub attempt_identity: Identity,
    pub candidate_identity: Identity,
    pub pipeline_revision_identity: Identity,
    pub plan_identity: Identity,
    pub policy_identity: Identity,
    pub platform: Label,
    pub required_products: Vec<Label>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductObservation {
    pub name: Label,
    pub artifact_identity: Identity,
    pub store_identity: Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricObservation {
    pub name: Label,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CacheObservation {
    pub kind: CacheKind,
    pub source_identity: Option<Identity>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildObservation {
    pub schema: String,
    pub identity: Identity,
    pub request_identity: Identity,
    pub outcome: BuildOutcome,
    pub products: Vec<ProductObservation>,
    pub builder_identity: Identity,
    pub worker_identity: Identity,
    pub store_identity: Identity,
    pub cache: CacheObservation,
    pub logs: Vec<Identity>,
    pub metrics: Vec<MetricObservation>,
    pub receipt_identity: Option<Identity>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractError {
    UnsupportedSchema,
    RequestIdentityMismatch,
    ObservationIdentityMismatch,
    RequiredProductsNotCanonical,
    CollectionLimitExceeded,
    RequestMismatch,
    BuilderMismatch,
    ProductMismatch,
    OutcomeContradiction,
    CacheContradiction,
    NonClaimsMismatch,
}

impl core::fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedSchema => "unsupported Mantle build interchange schema",
            Self::RequestIdentityMismatch => "build request identity mismatch",
            Self::ObservationIdentityMismatch => "build observation identity mismatch",
            Self::RequiredProductsNotCanonical => "required products are not sorted and unique",
            Self::CollectionLimitExceeded => "build interchange collection limit exceeded",
            Self::RequestMismatch => "build observation names another request",
            Self::BuilderMismatch => "build observation names another builder",
            Self::ProductMismatch => "build products do not match the request",
            Self::OutcomeContradiction => "build outcome contradicts its receipt or products",
            Self::CacheContradiction => "cache observation is contradictory",
            Self::NonClaimsMismatch => "build observation non-claims differ from the contract",
        })
    }
}
