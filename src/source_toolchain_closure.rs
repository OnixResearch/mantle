//! Pure source-built toolchain closure manifest validation.
//!
//! This module intentionally performs no I/O. Shell code is responsible for
//! reading manifests and checking file contents; this core validates the
//! receipt-bound shape and computes the deterministic policy digest.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA: &str = "mantle-source-built-toolchain-closure-v1";
const ABSENT_CLOSURE_STATUS: &str = "not-provided";
const SOURCE_BUILT_NON_CLAIM: &str = "not-source-built-toolchain-closure";
const POLICY_DIGEST_CONTEXT: &str = "mantle-source-built-toolchain-policy-digest-v1";
const MAX_TOOLCHAIN_MEMBERS: usize = 128;
const MAX_SEED_EXCEPTIONS: usize = 32;
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_CHAR_COUNT: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
const REQUIRED_TOOLCHAIN_ROLES: [ToolchainRole; 4] = [
    ToolchainRole::Rustc,
    ToolchainRole::Linker,
    ToolchainRole::CCompiler,
    ToolchainRole::Sysroot,
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct SourceBuiltToolchainClosureStatus {
    pub(crate) schema: &'static str,
    pub(crate) status: &'static str,
    pub(crate) claim: bool,
    pub(crate) non_claim: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ToolchainClosureManifest {
    pub(crate) schema: String,
    pub(crate) members: Vec<ToolchainClosureMember>,
    pub(crate) seed_exceptions: Vec<ToolchainSeedException>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ToolchainClosureMember {
    pub(crate) role: ToolchainRole,
    pub(crate) name: String,
    pub(crate) execution_path: String,
    pub(crate) content_digest_blake3: String,
    pub(crate) trust: ToolchainTrust,
    pub(crate) source: Option<ToolchainSourceIdentity>,
    pub(crate) build_receipt: Option<ToolchainBuildReceiptIdentity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ToolchainSourceIdentity {
    pub(crate) kind: ToolchainSourceKind,
    pub(crate) name: String,
    pub(crate) digest_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ToolchainBuildReceiptIdentity {
    pub(crate) kind: ToolchainBuildReceiptKind,
    pub(crate) name: String,
    pub(crate) digest_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ToolchainSeedException {
    pub(crate) name: String,
    pub(crate) reason: String,
    pub(crate) content_digest_blake3: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ToolchainRole {
    Rustc,
    Linker,
    CCompiler,
    CxxCompiler,
    PkgConfig,
    Sysroot,
    CrtObject,
    RuntimeLibrary,
    NativeHelper,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ToolchainTrust {
    SourceBuilt,
    SeedException,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ToolchainSourceKind {
    Tarball,
    Git,
    LocalTree,
    Generated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ToolchainBuildReceiptKind {
    MantleRustTopology,
    MantleDerivation,
    ExternalAttestedBuild,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ToolchainClosureValidation {
    pub(crate) policy_digest_blake3: String,
    pub(crate) member_count: usize,
    pub(crate) source_built_member_count: usize,
    pub(crate) seed_exception_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ToolchainClosureError {
    kind: ToolchainClosureErrorKind,
    message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToolchainClosureErrorKind {
    InvalidSchema,
    EmptyMembers,
    TooManyMembers,
    TooManySeeds,
    DuplicateMember,
    DuplicateSeed,
    MissingRequiredRole,
    EmptyField,
    InvalidDigest,
    InvalidExecutionPath,
    MissingSource,
    MissingBuildReceipt,
    MissingSeedException,
    Serialization,
}

pub(crate) fn absent_source_built_toolchain_closure() -> SourceBuiltToolchainClosureStatus {
    SourceBuiltToolchainClosureStatus {
        schema: SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA,
        status: ABSENT_CLOSURE_STATUS,
        claim: false,
        non_claim: SOURCE_BUILT_NON_CLAIM,
    }
}

pub(crate) fn validate_toolchain_closure_manifest(
    manifest: &ToolchainClosureManifest,
) -> Result<ToolchainClosureValidation, ToolchainClosureError> {
    let normalized = normalize_manifest(manifest)?;
    let policy_digest_blake3 = digest_normalized_manifest(&normalized)?;
    let source_built_member_count =
        normalized.members.iter().filter(|member| member.trust == ToolchainTrust::SourceBuilt).count();
    Ok(ToolchainClosureValidation {
        policy_digest_blake3,
        member_count: normalized.members.len(),
        source_built_member_count,
        seed_exception_count: normalized.seed_exceptions.len(),
    })
}

impl ToolchainClosureError {
    pub(crate) fn kind(&self) -> ToolchainClosureErrorKind {
        self.kind
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ToolchainClosureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ToolchainClosureError {}

fn normalize_manifest(manifest: &ToolchainClosureManifest) -> Result<ToolchainClosureManifest, ToolchainClosureError> {
    validate_schema(&manifest.schema)?;
    validate_collection_limits(manifest)?;
    let seed_keys = validate_seeds(&manifest.seed_exceptions)?;
    validate_members(&manifest.members, &seed_keys)?;
    validate_required_roles(&manifest.members)?;
    let mut normalized = manifest.clone();
    normalized.members.sort_by_key(member_sort_key);
    normalized.seed_exceptions.sort_by_key(seed_sort_key);
    Ok(normalized)
}

fn validate_schema(schema: &str) -> Result<(), ToolchainClosureError> {
    if schema == SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA {
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::InvalidSchema,
        format!("expected schema {SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA}, got {schema}"),
    ))
}

fn validate_collection_limits(manifest: &ToolchainClosureManifest) -> Result<(), ToolchainClosureError> {
    if manifest.members.is_empty() {
        return Err(error(ToolchainClosureErrorKind::EmptyMembers, "toolchain closure has no members"));
    }
    if manifest.members.len() > MAX_TOOLCHAIN_MEMBERS {
        return Err(error(ToolchainClosureErrorKind::TooManyMembers, "too many toolchain closure members"));
    }
    if manifest.seed_exceptions.len() > MAX_SEED_EXCEPTIONS {
        return Err(error(ToolchainClosureErrorKind::TooManySeeds, "too many toolchain seed exceptions"));
    }
    Ok(())
}

fn validate_seeds(seeds: &[ToolchainSeedException]) -> Result<BTreeSet<(String, String)>, ToolchainClosureError> {
    let mut keys = BTreeSet::new();
    for seed in seeds {
        validate_non_empty("seed name", &seed.name)?;
        validate_non_empty("seed reason", &seed.reason)?;
        validate_blake3_hex("seed content_digest_blake3", &seed.content_digest_blake3)?;
        if !keys.insert((seed.name.clone(), seed.content_digest_blake3.clone())) {
            return Err(error(ToolchainClosureErrorKind::DuplicateSeed, format!("duplicate seed '{}'", seed.name)));
        }
    }
    Ok(keys)
}

fn validate_members(
    members: &[ToolchainClosureMember],
    seed_keys: &BTreeSet<(String, String)>,
) -> Result<(), ToolchainClosureError> {
    let mut member_keys = BTreeSet::new();
    for member in members {
        validate_member(member, seed_keys)?;
        if !member_keys.insert(member_key(member)) {
            return Err(error(
                ToolchainClosureErrorKind::DuplicateMember,
                format!("duplicate member '{}'", member.name),
            ));
        }
    }
    Ok(())
}

fn validate_member(
    member: &ToolchainClosureMember,
    seed_keys: &BTreeSet<(String, String)>,
) -> Result<(), ToolchainClosureError> {
    validate_non_empty("member name", &member.name)?;
    validate_absolute_path("member execution_path", &member.execution_path)?;
    validate_blake3_hex("member content_digest_blake3", &member.content_digest_blake3)?;
    validate_optional_member_identities(member)?;
    match member.trust {
        ToolchainTrust::SourceBuilt => validate_source_built_member(member),
        ToolchainTrust::SeedException => validate_seed_member(member, seed_keys),
    }
}

fn validate_optional_member_identities(member: &ToolchainClosureMember) -> Result<(), ToolchainClosureError> {
    if let Some(source) = &member.source {
        validate_source_identity(source)?;
    }
    if let Some(receipt) = &member.build_receipt {
        validate_receipt_identity(receipt)?;
    }
    Ok(())
}

fn validate_source_built_member(member: &ToolchainClosureMember) -> Result<(), ToolchainClosureError> {
    let source = member.source.as_ref().ok_or_else(|| {
        error(
            ToolchainClosureErrorKind::MissingSource,
            format!("source-built member '{}' lacks source", member.name),
        )
    })?;
    let receipt = member.build_receipt.as_ref().ok_or_else(|| {
        error(
            ToolchainClosureErrorKind::MissingBuildReceipt,
            format!("source-built member '{}' lacks build receipt", member.name),
        )
    })?;
    debug_assert!(validate_source_identity(source).is_ok());
    debug_assert!(validate_receipt_identity(receipt).is_ok());
    Ok(())
}

fn validate_seed_member(
    member: &ToolchainClosureMember,
    seed_keys: &BTreeSet<(String, String)>,
) -> Result<(), ToolchainClosureError> {
    let key = (member.name.clone(), member.content_digest_blake3.clone());
    if seed_keys.contains(&key) {
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::MissingSeedException,
        format!("seed member '{}' lacks matching seed exception", member.name),
    ))
}

fn validate_source_identity(source: &ToolchainSourceIdentity) -> Result<(), ToolchainClosureError> {
    validate_non_empty("source name", &source.name)?;
    validate_blake3_hex("source digest_blake3", &source.digest_blake3)
}

fn validate_receipt_identity(receipt: &ToolchainBuildReceiptIdentity) -> Result<(), ToolchainClosureError> {
    validate_non_empty("build receipt name", &receipt.name)?;
    validate_blake3_hex("build receipt digest_blake3", &receipt.digest_blake3)
}

fn validate_required_roles(members: &[ToolchainClosureMember]) -> Result<(), ToolchainClosureError> {
    let present_roles: BTreeSet<ToolchainRole> = members.iter().map(|member| member.role).collect();
    for role in REQUIRED_TOOLCHAIN_ROLES {
        if !present_roles.contains(&role) {
            return Err(error(
                ToolchainClosureErrorKind::MissingRequiredRole,
                format!("missing required toolchain role {role:?}"),
            ));
        }
    }
    Ok(())
}

fn validate_non_empty(label: &str, value: &str) -> Result<(), ToolchainClosureError> {
    if !value.trim().is_empty() {
        return Ok(());
    }
    Err(error(ToolchainClosureErrorKind::EmptyField, format!("{label} is empty")))
}

fn validate_absolute_path(label: &str, value: &str) -> Result<(), ToolchainClosureError> {
    validate_non_empty(label, value)?;
    if Path::new(value).is_absolute() {
        return Ok(());
    }
    Err(error(ToolchainClosureErrorKind::InvalidExecutionPath, format!("{label} must be absolute: {value}")))
}

fn validate_blake3_hex(label: &str, value: &str) -> Result<(), ToolchainClosureError> {
    if value.len() != BLAKE3_HEX_CHAR_COUNT {
        return Err(error(ToolchainClosureErrorKind::InvalidDigest, format!("{label} has invalid length")));
    }
    if value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Ok(());
    }
    Err(error(ToolchainClosureErrorKind::InvalidDigest, format!("{label} must be lowercase hex")))
}

fn digest_normalized_manifest(manifest: &ToolchainClosureManifest) -> Result<String, ToolchainClosureError> {
    let payload = serde_json::json!({
        "context": POLICY_DIGEST_CONTEXT,
        "manifest": manifest,
    });
    let bytes = serde_json::to_vec(&payload).map_err(|err| {
        error(ToolchainClosureErrorKind::Serialization, format!("serialize toolchain closure policy: {err}"))
    })?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn member_key(member: &ToolchainClosureMember) -> (ToolchainRole, String) {
    (member.role, member.name.clone())
}

fn member_sort_key(member: &ToolchainClosureMember) -> (ToolchainRole, String, String) {
    (member.role, member.name.clone(), member.content_digest_blake3.clone())
}

fn seed_sort_key(seed: &ToolchainSeedException) -> (String, String) {
    (seed.name.clone(), seed.content_digest_blake3.clone())
}

fn error(kind: ToolchainClosureErrorKind, message: impl Into<String>) -> ToolchainClosureError {
    ToolchainClosureError {
        kind,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const DIGEST_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const DIGEST_E: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    const DIGEST_F: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

    #[test]
    fn absent_closure_status_preserves_current_non_claim() {
        let status = absent_source_built_toolchain_closure();

        assert_eq!(status.schema, SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA);
        assert_eq!(status.status, ABSENT_CLOSURE_STATUS);
        assert!(!status.claim);
        assert_eq!(status.non_claim, SOURCE_BUILT_NON_CLAIM);
    }

    #[test]
    fn valid_manifest_yields_stable_order_independent_policy_digest() {
        let manifest = valid_manifest();
        let mut reversed = manifest.clone();
        reversed.members.reverse();

        let original = validate_toolchain_closure_manifest(&manifest).unwrap();
        let reordered = validate_toolchain_closure_manifest(&reversed).unwrap();

        assert_eq!(original.member_count, REQUIRED_TOOLCHAIN_ROLES.len());
        assert_eq!(original.source_built_member_count, REQUIRED_TOOLCHAIN_ROLES.len());
        assert_eq!(original.seed_exception_count, 0);
        assert_eq!(original.policy_digest_blake3, reordered.policy_digest_blake3);
    }

    #[test]
    fn validator_rejects_source_built_member_without_receipt() {
        let mut manifest = valid_manifest();
        manifest.members[0].build_receipt = None;

        let err = validate_toolchain_closure_manifest(&manifest).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::MissingBuildReceipt);
        assert!(err.message().contains("lacks build receipt"));
    }

    #[test]
    fn validator_rejects_seed_member_without_seed_exception() {
        let mut manifest = valid_manifest();
        manifest.members[0].trust = ToolchainTrust::SeedException;
        manifest.members[0].source = None;
        manifest.members[0].build_receipt = None;

        let err = validate_toolchain_closure_manifest(&manifest).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::MissingSeedException);
        assert!(err.message().contains("lacks matching seed exception"));
    }

    #[test]
    fn validator_accepts_explicit_seed_exception() {
        let mut manifest = valid_manifest();
        make_seed_member(&mut manifest, 0);

        let validation = validate_toolchain_closure_manifest(&manifest).unwrap();

        assert_eq!(validation.member_count, REQUIRED_TOOLCHAIN_ROLES.len());
        assert_eq!(validation.source_built_member_count, REQUIRED_TOOLCHAIN_ROLES.len() - 1);
        assert_eq!(validation.seed_exception_count, 1);
    }

    #[test]
    fn validator_rejects_invalid_optional_source_on_seed_member() {
        let mut manifest = valid_manifest();
        make_seed_member(&mut manifest, 0);
        manifest.members[0].source = Some(ToolchainSourceIdentity {
            kind: ToolchainSourceKind::Tarball,
            name: "seed-source".to_string(),
            digest_blake3: "FFFF".to_string(),
        });

        let err = validate_toolchain_closure_manifest(&manifest).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::InvalidDigest);
        assert!(err.message().contains("source digest_blake3"));
    }

    #[test]
    fn validator_rejects_invalid_optional_receipt_on_seed_member() {
        let mut manifest = valid_manifest();
        make_seed_member(&mut manifest, 0);
        manifest.members[0].build_receipt = Some(ToolchainBuildReceiptIdentity {
            kind: ToolchainBuildReceiptKind::ExternalAttestedBuild,
            name: "seed-receipt".to_string(),
            digest_blake3: "GGGG".to_string(),
        });

        let err = validate_toolchain_closure_manifest(&manifest).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::InvalidDigest);
        assert!(err.message().contains("build receipt digest_blake3"));
    }

    #[test]
    fn validator_rejects_invalid_digest_shape() {
        let mut manifest = valid_manifest();
        manifest.members[0].content_digest_blake3 = "AAAA".to_string();

        let err = validate_toolchain_closure_manifest(&manifest).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::InvalidDigest);
        assert!(err.message().contains("invalid length"));
    }

    #[test]
    fn validator_rejects_missing_required_role() {
        let mut manifest = valid_manifest();
        manifest.members.retain(|member| member.role != ToolchainRole::Sysroot);

        let err = validate_toolchain_closure_manifest(&manifest).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::MissingRequiredRole);
        assert!(err.message().contains("Sysroot"));
    }

    #[test]
    fn validator_rejects_duplicate_member_identity() {
        let mut manifest = valid_manifest();
        manifest.members.push(manifest.members[0].clone());

        let err = validate_toolchain_closure_manifest(&manifest).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::DuplicateMember);
        assert!(err.message().contains("duplicate member"));
    }

    fn valid_manifest() -> ToolchainClosureManifest {
        ToolchainClosureManifest {
            schema: SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA.to_string(),
            members: vec![
                member(ToolchainRole::Rustc, "rustc", DIGEST_A, DIGEST_B),
                member(ToolchainRole::Linker, "ld", DIGEST_B, DIGEST_C),
                member(ToolchainRole::CCompiler, "cc", DIGEST_C, DIGEST_D),
                member(ToolchainRole::Sysroot, "sysroot", DIGEST_D, DIGEST_E),
            ],
            seed_exceptions: Vec::new(),
        }
    }

    fn make_seed_member(manifest: &mut ToolchainClosureManifest, member_index: usize) {
        let seed_name = manifest.members[member_index].name.clone();
        let seed_digest = manifest.members[member_index].content_digest_blake3.clone();
        manifest.members[member_index].trust = ToolchainTrust::SeedException;
        manifest.members[member_index].source = None;
        manifest.members[member_index].build_receipt = None;
        manifest.seed_exceptions.push(ToolchainSeedException {
            name: seed_name,
            reason: "stage0 bootstrap seed".to_string(),
            content_digest_blake3: seed_digest,
        });
    }

    fn member(
        role: ToolchainRole,
        name: &str,
        content_digest_blake3: &str,
        receipt_digest_blake3: &str,
    ) -> ToolchainClosureMember {
        ToolchainClosureMember {
            role,
            name: name.to_string(),
            execution_path: format!("/toolchain/{name}"),
            content_digest_blake3: content_digest_blake3.to_string(),
            trust: ToolchainTrust::SourceBuilt,
            source: Some(ToolchainSourceIdentity {
                kind: ToolchainSourceKind::Tarball,
                name: format!("{name}-source"),
                digest_blake3: DIGEST_F.to_string(),
            }),
            build_receipt: Some(ToolchainBuildReceiptIdentity {
                kind: ToolchainBuildReceiptKind::MantleRustTopology,
                name: format!("{name}-receipt"),
                digest_blake3: receipt_digest_blake3.to_string(),
            }),
        }
    }
}
