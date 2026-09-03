use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

// machine-artifact-public: source.source-observation
pub const SOURCE_OBSERVATION_SCHEMA: &str = "mantle-source-observation-v1";
pub const SOURCE_OBSERVATION_NON_CLAIM: &str = "source-observation-binds-supplied-origin-projection-profile-and-content-facts-without-proving-ownership-trust-review-or-correctness";
pub const BLAKE3_HEX_CHARS: usize = 64;
pub const GIT_SHA1_HEX_CHARS: usize = 40;
pub const GIT_SHA256_HEX_CHARS: usize = 64;
pub const SOURCE_HINT_BYTES_MIN: u32 = 1;
pub const SOURCE_PROJECTION_BYTES_MIN: u32 = 1;
pub const LOCATOR_HINT_BYTES_MAX: u32 = 4_096;
pub const MUTABLE_REFERENCE_HINT_BYTES_MAX: u32 = 1_024;
pub const PROJECTION_BYTES_MAX: u32 = 4_096;
const SOURCE_OBSERVATION_DOMAIN: &[u8] = b"mantle.source-observation.v1";
const DOMAIN_SEPARATOR: u8 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    FixedUrl,
    VcsSnapshot,
    LocalLogical,
    PackageMirror,
    OpaqueAdapter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocatorClass {
    Url,
    GitRemote,
    LogicalPath,
    Mirror,
    Opaque,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GitObjectFormat {
    Sha1,
    Sha256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SnapshotProfile {
    FlatFileV1,
    CanonicalTreeV1,
    ArchiveTreeV1,
    OpaqueV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProvenanceDisposition {
    Complete,
    UnavailableLegacyV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImmutableRevisionWire {
    pub object_format: GitObjectFormat,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceObservationWire {
    pub schema: String,
    pub source_kind: SourceKind,
    pub locator_class: LocatorClass,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub immutable_revision: Option<ImmutableRevisionWire>,
    pub normalized_projection: String,
    pub snapshot_profile: SnapshotProfile,
    pub content_blake3: String,
    pub observation_blake3: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutable_reference_hint: Option<String>,
    pub provenance: ProvenanceDisposition,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceObservationSubject {
    pub schema: String,
    pub source_kind: SourceKind,
    pub locator_class: LocatorClass,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub immutable_revision: Option<ImmutableRevisionWire>,
    pub normalized_projection: String,
    pub snapshot_profile: SnapshotProfile,
    pub content_blake3: String,
    pub observation_blake3: String,
    pub provenance: ProvenanceDisposition,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceObservationDraft {
    pub source_kind: SourceKind,
    pub locator_class: LocatorClass,
    pub locator_hint: Option<String>,
    pub immutable_revision: Option<ImmutableRevisionWire>,
    pub normalized_projection: String,
    pub snapshot_profile: SnapshotProfile,
    pub content_blake3: String,
    pub mutable_reference_hint: Option<String>,
    pub provenance: ProvenanceDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceObservation {
    wire: SourceObservationWire,
}

impl SourceObservation {
    #[must_use]
    pub fn as_wire(&self) -> &SourceObservationWire {
        debug_assert_eq!(self.wire.schema, SOURCE_OBSERVATION_SCHEMA);
        debug_assert_eq!(self.wire.observation_blake3.len(), BLAKE3_HEX_CHARS);
        &self.wire
    }

    #[must_use]
    pub fn into_wire(self) -> SourceObservationWire {
        debug_assert_eq!(self.wire.non_claim, SOURCE_OBSERVATION_NON_CLAIM);
        debug_assert!(valid_blake3(&self.wire.content_blake3));
        self.wire
    }

    #[must_use]
    pub fn observation_blake3(&self) -> &str {
        debug_assert!(valid_blake3(&self.wire.observation_blake3));
        debug_assert_eq!(self.wire.schema, SOURCE_OBSERVATION_SCHEMA);
        &self.wire.observation_blake3
    }

    #[must_use]
    pub fn content_blake3(&self) -> &str {
        debug_assert!(valid_blake3(&self.wire.content_blake3));
        debug_assert!(!self.wire.normalized_projection.is_empty());
        &self.wire.content_blake3
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceObservationError {
    Schema,
    SourceLocatorMismatch,
    RevisionMissing,
    RevisionUnexpected,
    RevisionFormat,
    Projection,
    SnapshotProfile,
    ContentDigest,
    ObservationDigest,
    LocatorHint,
    SecretBearingLocator,
    MutableReferenceHint,
    Provenance,
    NonClaim,
    Canonicalization,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacySourceObservationFacts {
    pub source_kind: SourceKind,
    pub locator_class: Option<LocatorClass>,
    pub locator_hint: Option<String>,
    pub immutable_revision: Option<ImmutableRevisionWire>,
    pub normalized_projection: Option<String>,
    pub snapshot_profile: Option<SnapshotProfile>,
    pub content_blake3: String,
    pub mutable_reference_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyProvenanceUnavailable {
    pub source_kind: SourceKind,
    pub content_blake3: String,
    pub missing_fields: Vec<String>,
    pub disposition: ProvenanceDisposition,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacySourceProjection {
    Complete(SourceObservationWire),
    ProvenanceUnavailable(LegacyProvenanceUnavailable),
}

#[derive(Serialize)]
struct CanonicalSourceObservation<'a> {
    schema: &'static str,
    source_kind: SourceKind,
    locator_class: LocatorClass,
    immutable_revision: &'a Option<ImmutableRevisionWire>,
    normalized_projection: &'a str,
    snapshot_profile: SnapshotProfile,
    content_blake3: &'a str,
    provenance: ProvenanceDisposition,
    non_claim: &'static str,
}

pub fn build_source_observation(draft: SourceObservationDraft) -> Result<SourceObservation, SourceObservationError> {
    validate_draft(&draft)?;
    let observation_blake3 = draft_identity(&draft)?;
    let wire = SourceObservationWire {
        schema: SOURCE_OBSERVATION_SCHEMA.to_string(),
        source_kind: draft.source_kind,
        locator_class: draft.locator_class,
        locator_hint: draft.locator_hint,
        immutable_revision: draft.immutable_revision,
        normalized_projection: draft.normalized_projection,
        snapshot_profile: draft.snapshot_profile,
        content_blake3: draft.content_blake3,
        observation_blake3,
        mutable_reference_hint: draft.mutable_reference_hint,
        provenance: draft.provenance,
        non_claim: SOURCE_OBSERVATION_NON_CLAIM.to_string(),
    };
    admit_source_observation(wire)
}

#[must_use]
pub fn source_observation_subject(observation: &SourceObservationWire) -> SourceObservationSubject {
    SourceObservationSubject {
        schema: observation.schema.clone(),
        source_kind: observation.source_kind,
        locator_class: observation.locator_class,
        immutable_revision: observation.immutable_revision.clone(),
        normalized_projection: observation.normalized_projection.clone(),
        snapshot_profile: observation.snapshot_profile,
        content_blake3: observation.content_blake3.clone(),
        observation_blake3: observation.observation_blake3.clone(),
        provenance: observation.provenance,
        non_claim: observation.non_claim.clone(),
    }
}

pub fn admit_source_observation_subject(
    subject: SourceObservationSubject,
) -> Result<SourceObservation, SourceObservationError> {
    admit_source_observation(SourceObservationWire {
        schema: subject.schema,
        source_kind: subject.source_kind,
        locator_class: subject.locator_class,
        locator_hint: None,
        immutable_revision: subject.immutable_revision,
        normalized_projection: subject.normalized_projection,
        snapshot_profile: subject.snapshot_profile,
        content_blake3: subject.content_blake3,
        observation_blake3: subject.observation_blake3,
        mutable_reference_hint: None,
        provenance: subject.provenance,
        non_claim: subject.non_claim,
    })
}

pub fn admit_source_observation(wire: SourceObservationWire) -> Result<SourceObservation, SourceObservationError> {
    if wire.schema != SOURCE_OBSERVATION_SCHEMA {
        return Err(SourceObservationError::Schema);
    }
    if wire.non_claim != SOURCE_OBSERVATION_NON_CLAIM {
        return Err(SourceObservationError::NonClaim);
    }
    let draft = draft_from_wire(&wire);
    validate_draft(&draft)?;
    let expected_blake3 = draft_identity(&draft)?;
    if wire.observation_blake3 != expected_blake3 {
        return Err(SourceObservationError::ObservationDigest);
    }
    debug_assert!(valid_blake3(&wire.observation_blake3));
    debug_assert_eq!(wire.normalized_projection, draft.normalized_projection);
    Ok(SourceObservation { wire })
}

pub fn project_legacy_source_observation(
    facts: LegacySourceObservationFacts,
) -> Result<LegacySourceProjection, SourceObservationError> {
    debug_assert!(facts.content_blake3.is_char_boundary(facts.content_blake3.len()));
    debug_assert!(facts.normalized_projection.as_ref().is_none_or(|value| value.is_char_boundary(value.len())));
    if !valid_blake3(&facts.content_blake3) {
        return Err(SourceObservationError::ContentDigest);
    }
    let missing_fields = legacy_missing_fields(&facts);
    if !missing_fields.is_empty() {
        return Ok(LegacySourceProjection::ProvenanceUnavailable(LegacyProvenanceUnavailable {
            source_kind: facts.source_kind,
            content_blake3: facts.content_blake3,
            missing_fields,
            disposition: ProvenanceDisposition::UnavailableLegacyV1,
            non_claim: SOURCE_OBSERVATION_NON_CLAIM.to_string(),
        }));
    }
    let draft = SourceObservationDraft {
        source_kind: facts.source_kind,
        locator_class: facts.locator_class.ok_or(SourceObservationError::SourceLocatorMismatch)?,
        locator_hint: facts.locator_hint,
        immutable_revision: facts.immutable_revision,
        normalized_projection: facts.normalized_projection.ok_or(SourceObservationError::Projection)?,
        snapshot_profile: facts.snapshot_profile.ok_or(SourceObservationError::SnapshotProfile)?,
        content_blake3: facts.content_blake3,
        mutable_reference_hint: facts.mutable_reference_hint,
        provenance: ProvenanceDisposition::Complete,
    };
    Ok(LegacySourceProjection::Complete(build_source_observation(draft)?.into_wire()))
}

fn draft_from_wire(wire: &SourceObservationWire) -> SourceObservationDraft {
    SourceObservationDraft {
        source_kind: wire.source_kind,
        locator_class: wire.locator_class,
        locator_hint: wire.locator_hint.clone(),
        immutable_revision: wire.immutable_revision.clone(),
        normalized_projection: wire.normalized_projection.clone(),
        snapshot_profile: wire.snapshot_profile,
        content_blake3: wire.content_blake3.clone(),
        mutable_reference_hint: wire.mutable_reference_hint.clone(),
        provenance: wire.provenance,
    }
}

fn validate_draft(draft: &SourceObservationDraft) -> Result<(), SourceObservationError> {
    validate_source_locator(draft.source_kind, draft.locator_class)?;
    validate_revision(draft.source_kind, draft.immutable_revision.as_ref())?;
    validate_projection(&draft.normalized_projection)?;
    validate_profile(draft.source_kind, draft.snapshot_profile)?;
    if !valid_blake3(&draft.content_blake3) {
        return Err(SourceObservationError::ContentDigest);
    }
    validate_locator_hint(draft.locator_class, draft.locator_hint.as_deref())?;
    validate_mutable_reference_hint(draft.mutable_reference_hint.as_deref())?;
    if draft.provenance != ProvenanceDisposition::Complete {
        return Err(SourceObservationError::Provenance);
    }
    Ok(())
}

fn validate_source_locator(kind: SourceKind, locator: LocatorClass) -> Result<(), SourceObservationError> {
    let is_source_locator_pair = matches!(
        (kind, locator),
        (SourceKind::FixedUrl, LocatorClass::Url)
            | (SourceKind::VcsSnapshot, LocatorClass::GitRemote)
            | (SourceKind::LocalLogical, LocatorClass::LogicalPath)
            | (SourceKind::PackageMirror, LocatorClass::Mirror)
            | (SourceKind::OpaqueAdapter, LocatorClass::Opaque)
    );
    if is_source_locator_pair {
        return Ok(());
    }
    Err(SourceObservationError::SourceLocatorMismatch)
}

fn validate_revision(kind: SourceKind, revision: Option<&ImmutableRevisionWire>) -> Result<(), SourceObservationError> {
    if kind != SourceKind::VcsSnapshot {
        return if revision.is_none() {
            Ok(())
        } else {
            Err(SourceObservationError::RevisionUnexpected)
        };
    }
    let revision = revision.ok_or(SourceObservationError::RevisionMissing)?;
    let expected_chars = match revision.object_format {
        GitObjectFormat::Sha1 => GIT_SHA1_HEX_CHARS,
        GitObjectFormat::Sha256 => GIT_SHA256_HEX_CHARS,
    };
    if revision.value.len() != expected_chars || !lower_hex(&revision.value) {
        return Err(SourceObservationError::RevisionFormat);
    }
    Ok(())
}

fn validate_projection(projection: &str) -> Result<(), SourceObservationError> {
    let bytes = u32::try_from(projection.len()).map_err(|_| SourceObservationError::Projection)?;
    let is_projection_bounded = (SOURCE_PROJECTION_BYTES_MIN..=PROJECTION_BYTES_MAX).contains(&bytes);
    if !is_projection_bounded {
        return Err(SourceObservationError::Projection);
    }
    if projection.starts_with('/') || projection.contains('\\') {
        return Err(SourceObservationError::Projection);
    }
    if projection == "." {
        return Ok(());
    }
    for component in projection.split('/') {
        let is_reserved_component = component == "." || component == "..";
        let has_control_character = component.chars().any(char::is_control);
        if component.is_empty() {
            return Err(SourceObservationError::Projection);
        }
        if is_reserved_component || has_control_character {
            return Err(SourceObservationError::Projection);
        }
    }
    debug_assert!(is_projection_bounded);
    debug_assert!(!projection.starts_with('/'));
    Ok(())
}

fn validate_profile(kind: SourceKind, profile: SnapshotProfile) -> Result<(), SourceObservationError> {
    let is_profile_allowed = match kind {
        SourceKind::FixedUrl => matches!(profile, SnapshotProfile::FlatFileV1 | SnapshotProfile::ArchiveTreeV1),
        SourceKind::VcsSnapshot => {
            matches!(profile, SnapshotProfile::CanonicalTreeV1 | SnapshotProfile::ArchiveTreeV1)
        }
        SourceKind::LocalLogical => matches!(profile, SnapshotProfile::FlatFileV1 | SnapshotProfile::CanonicalTreeV1),
        SourceKind::PackageMirror => {
            matches!(profile, SnapshotProfile::ArchiveTreeV1 | SnapshotProfile::CanonicalTreeV1)
        }
        SourceKind::OpaqueAdapter => profile == SnapshotProfile::OpaqueV1,
    };
    if is_profile_allowed {
        return Ok(());
    }
    Err(SourceObservationError::SnapshotProfile)
}

fn validate_locator_hint(locator: LocatorClass, hint: Option<&str>) -> Result<(), SourceObservationError> {
    let Some(hint) = hint else {
        return Ok(());
    };
    validate_bounded_hint(hint, LOCATOR_HINT_BYTES_MAX, SourceObservationError::LocatorHint)?;
    if locator == LocatorClass::LogicalPath && (hint.starts_with('/') || hint.contains("..")) {
        return Err(SourceObservationError::LocatorHint);
    }
    if locator_secret_bearing(hint) {
        return Err(SourceObservationError::SecretBearingLocator);
    }
    Ok(())
}

fn validate_mutable_reference_hint(hint: Option<&str>) -> Result<(), SourceObservationError> {
    let Some(hint) = hint else {
        return Ok(());
    };
    validate_bounded_hint(hint, MUTABLE_REFERENCE_HINT_BYTES_MAX, SourceObservationError::MutableReferenceHint)?;
    if locator_secret_bearing(hint) {
        return Err(SourceObservationError::SecretBearingLocator);
    }
    Ok(())
}

fn validate_bounded_hint(
    value: &str,
    maximum_bytes: u32,
    error: SourceObservationError,
) -> Result<(), SourceObservationError> {
    let bytes = u32::try_from(value.len()).map_err(|_| error)?;
    let is_hint_bounded = (SOURCE_HINT_BYTES_MIN..=maximum_bytes).contains(&bytes);
    if !is_hint_bounded {
        return Err(error);
    }
    if value.chars().any(char::is_control) {
        return Err(error);
    }
    Ok(())
}

fn locator_secret_bearing(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let has_secret_query_field = [
        "token=",
        "access_token=",
        "password=",
        "secret=",
        "x-amz-signature=",
        "sig=",
    ]
    .iter()
    .any(|marker| lower.contains(marker));
    let has_userinfo = lower.split_once("://").is_some_and(|(_, authority)| {
        let authority = authority.split('/').next().unwrap_or(authority);
        authority.contains('@')
    });
    has_secret_query_field || has_userinfo
}

fn legacy_missing_fields(facts: &LegacySourceObservationFacts) -> Vec<String> {
    let mut missing = Vec::new();
    if facts.locator_class.is_none() {
        missing.push("locator_class".to_string());
    }
    if facts.normalized_projection.is_none() {
        missing.push("normalized_projection".to_string());
    }
    if facts.snapshot_profile.is_none() {
        missing.push("snapshot_profile".to_string());
    }
    if facts.source_kind == SourceKind::VcsSnapshot && facts.immutable_revision.is_none() {
        missing.push("immutable_revision".to_string());
    }
    missing
}

fn draft_identity(draft: &SourceObservationDraft) -> Result<String, SourceObservationError> {
    let canonical = CanonicalSourceObservation {
        schema: SOURCE_OBSERVATION_SCHEMA,
        source_kind: draft.source_kind,
        locator_class: draft.locator_class,
        immutable_revision: &draft.immutable_revision,
        normalized_projection: &draft.normalized_projection,
        snapshot_profile: draft.snapshot_profile,
        content_blake3: &draft.content_blake3,
        provenance: draft.provenance,
        non_claim: SOURCE_OBSERVATION_NON_CLAIM,
    };
    canonical_blake3(SOURCE_OBSERVATION_DOMAIN, &canonical)
}

pub(crate) fn canonical_blake3<T: Serialize>(domain: &[u8], value: &T) -> Result<String, SourceObservationError> {
    let bytes = serde_json::to_vec(value).map_err(|_| SourceObservationError::Canonicalization)?;
    let domain_bytes = u64::try_from(domain.len()).map_err(|_| SourceObservationError::Canonicalization)?;
    let value_bytes = u64::try_from(bytes.len()).map_err(|_| SourceObservationError::Canonicalization)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&domain_bytes.to_le_bytes());
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(domain);
    hasher.update(&value_bytes.to_le_bytes());
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(&bytes);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    debug_assert!(!domain.is_empty());
    Ok(digest)
}

pub(crate) fn valid_blake3(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS && lower_hex(value)
}

fn lower_hex(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
