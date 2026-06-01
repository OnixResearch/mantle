//! Pure source-built toolchain closure manifest validation.
//!
//! This module intentionally performs no I/O. Shell code is responsible for
//! reading manifests and checking file contents; this core validates the
//! receipt-bound shape and computes the deterministic policy digest.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA: &str = "mantle-source-built-toolchain-closure-v1";
const ABSENT_CLOSURE_STATUS: &str = "not-provided";
pub(crate) const SOURCE_BUILT_NON_CLAIM: &str = "not-source-built-toolchain-closure";
const VALIDATED_NOT_ENFORCED_STATUS: &str = "validated-not-enforced";
const VALIDATED_ENFORCED_STATUS: &str = "validated-enforced";
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
const DISALLOWED_SEED_EXCEPTION_MARKERS: [&str; 3] = ["placeholder", "todo", "unverified"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct SourceBuiltToolchainClosureStatus {
    pub(crate) schema: &'static str,
    pub(crate) status: &'static str,
    pub(crate) claim: bool,
    pub(crate) non_claim: &'static str,
    pub(crate) manifest_path: Option<PathBuf>,
    pub(crate) policy_digest_blake3: Option<String>,
    pub(crate) member_count: Option<usize>,
    pub(crate) source_built_member_count: Option<usize>,
    pub(crate) seed_exception_count: Option<usize>,
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
pub(crate) struct ToolchainObservedInput {
    pub(crate) role: ToolchainRole,
    pub(crate) execution_path: String,
    pub(crate) content_digest_blake3: Option<String>,
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
    PlaceholderSeedException,
    HostToolLeakage,
    Serialization,
}

pub(crate) fn absent_source_built_toolchain_closure() -> SourceBuiltToolchainClosureStatus {
    SourceBuiltToolchainClosureStatus {
        schema: SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA,
        status: ABSENT_CLOSURE_STATUS,
        claim: false,
        non_claim: SOURCE_BUILT_NON_CLAIM,
        manifest_path: None,
        policy_digest_blake3: None,
        member_count: None,
        source_built_member_count: None,
        seed_exception_count: None,
    }
}

pub(crate) fn validated_source_built_toolchain_closure(
    manifest_path: PathBuf,
    validation: &ToolchainClosureValidation,
) -> SourceBuiltToolchainClosureStatus {
    source_built_toolchain_closure_status(manifest_path, validation, VALIDATED_NOT_ENFORCED_STATUS)
}

pub(crate) fn enforced_source_built_toolchain_closure(
    manifest_path: PathBuf,
    validation: &ToolchainClosureValidation,
) -> SourceBuiltToolchainClosureStatus {
    source_built_toolchain_closure_status(manifest_path, validation, VALIDATED_ENFORCED_STATUS)
}

fn source_built_toolchain_closure_status(
    manifest_path: PathBuf,
    validation: &ToolchainClosureValidation,
    status: &'static str,
) -> SourceBuiltToolchainClosureStatus {
    SourceBuiltToolchainClosureStatus {
        schema: SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA,
        status,
        claim: false,
        non_claim: SOURCE_BUILT_NON_CLAIM,
        manifest_path: Some(manifest_path),
        policy_digest_blake3: Some(validation.policy_digest_blake3.clone()),
        member_count: Some(validation.member_count),
        source_built_member_count: Some(validation.source_built_member_count),
        seed_exception_count: Some(validation.seed_exception_count),
    }
}

pub(crate) fn validate_toolchain_closure_manifest(
    manifest: &ToolchainClosureManifest,
) -> Result<ToolchainClosureValidation, ToolchainClosureError> {
    let normalized = normalize_manifest(manifest)?;
    validation_from_normalized_manifest(&normalized)
}

pub(crate) fn enforce_observed_toolchain_inputs(
    manifest: &ToolchainClosureManifest,
    observed: &[ToolchainObservedInput],
) -> Result<ToolchainClosureValidation, ToolchainClosureError> {
    let normalized = normalize_manifest(manifest)?;
    validate_observed_inputs(observed)?;
    for input in observed {
        require_observed_input_declared(&normalized, input)?;
    }
    validation_from_normalized_manifest(&normalized)
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

fn validation_from_normalized_manifest(
    manifest: &ToolchainClosureManifest,
) -> Result<ToolchainClosureValidation, ToolchainClosureError> {
    let policy_digest_blake3 = digest_normalized_manifest(manifest)?;
    let source_built_member_count =
        manifest.members.iter().filter(|member| member.trust == ToolchainTrust::SourceBuilt).count();
    Ok(ToolchainClosureValidation {
        policy_digest_blake3,
        member_count: manifest.members.len(),
        source_built_member_count,
        seed_exception_count: manifest.seed_exceptions.len(),
    })
}

fn validate_observed_inputs(observed: &[ToolchainObservedInput]) -> Result<(), ToolchainClosureError> {
    for input in observed {
        validate_absolute_path("observed execution_path", &input.execution_path)?;
        if let Some(digest) = &input.content_digest_blake3 {
            validate_blake3_hex("observed content_digest_blake3", digest)?;
        }
    }
    Ok(())
}

fn require_observed_input_declared(
    manifest: &ToolchainClosureManifest,
    input: &ToolchainObservedInput,
) -> Result<(), ToolchainClosureError> {
    let role_path_match = manifest
        .members
        .iter()
        .find(|member| member.role == input.role && member.execution_path == input.execution_path);
    let Some(member) = role_path_match else {
        return Err(error(
            ToolchainClosureErrorKind::HostToolLeakage,
            format!("host-tool-leakage: {:?} uses undeclared path {}", input.role, input.execution_path),
        ));
    };
    if let Some(digest) = &input.content_digest_blake3 {
        if member.content_digest_blake3 != *digest {
            return Err(error(
                ToolchainClosureErrorKind::HostToolLeakage,
                format!("host-tool-leakage: {:?} digest mismatch for {}", input.role, input.execution_path),
            ));
        }
    }
    Ok(())
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
        validate_seed_exception_text("seed name", &seed.name)?;
        validate_seed_exception_text("seed reason", &seed.reason)?;
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

fn validate_seed_exception_text(label: &str, value: &str) -> Result<(), ToolchainClosureError> {
    let normalized = value.to_ascii_lowercase();
    for marker in DISALLOWED_SEED_EXCEPTION_MARKERS {
        if contains_disallowed_seed_exception_marker(&normalized, marker) {
            return Err(error(
                ToolchainClosureErrorKind::PlaceholderSeedException,
                format!("{label} contains disallowed seed marker '{marker}'"),
            ));
        }
    }
    Ok(())
}

fn contains_disallowed_seed_exception_marker(value: &str, marker: &str) -> bool {
    debug_assert!(!marker.is_empty());
    let mut search_start = 0;
    while let Some(relative_start) = value[search_start..].find(marker) {
        let marker_start = search_start + relative_start;
        let marker_end = marker_start + marker.len();
        if has_marker_boundaries(value, marker_start, marker_end) {
            return true;
        }
        search_start = marker_end;
    }
    false
}

fn has_marker_boundaries(value: &str, marker_start: usize, marker_end: usize) -> bool {
    debug_assert!(marker_start <= marker_end);
    debug_assert!(marker_end <= value.len());
    let before = value[..marker_start].chars().next_back();
    let after = value[marker_end..].chars().next();
    !is_seed_marker_word_char(before) && !is_seed_marker_word_char(after)
}

fn is_seed_marker_word_char(value: Option<char>) -> bool {
    value.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
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
    const TWO_SEED_EXCEPTION_COUNT: usize = 2;

    #[test]
    fn absent_closure_status_preserves_current_non_claim() {
        let status = absent_source_built_toolchain_closure();

        assert_eq!(status.schema, SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA);
        assert_eq!(status.status, ABSENT_CLOSURE_STATUS);
        assert!(!status.claim);
        assert_eq!(status.non_claim, SOURCE_BUILT_NON_CLAIM);
        assert!(status.manifest_path.is_none());
        assert!(status.policy_digest_blake3.is_none());
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
    fn json_fixture_parses_seed_exception_and_counts_source_built_members() {
        let manifest = serde_json::from_value::<ToolchainClosureManifest>(serde_json::json!({
            "schema": SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA,
            "members": [
                member_json("rustc", "rustc", "/toolchain/rustc", DIGEST_A, DIGEST_B, "source-built"),
                member_json("linker", "ld", "/toolchain/ld", DIGEST_B, DIGEST_C, "source-built"),
                member_json("c-compiler", "cc", "/toolchain/cc", DIGEST_C, DIGEST_D, "source-built"),
                seed_member_json("sysroot", "stage0-sysroot", "/toolchain/sysroot", DIGEST_D),
            ],
            "seed_exceptions": [{
                "name": "stage0-sysroot",
                "reason": "documented bootstrap trust root",
                "content_digest_blake3": DIGEST_D,
            }]
        }))
        .unwrap();

        let validation = validate_toolchain_closure_manifest(&manifest).unwrap();

        assert_eq!(validation.member_count, REQUIRED_TOOLCHAIN_ROLES.len());
        assert_eq!(validation.source_built_member_count, REQUIRED_TOOLCHAIN_ROLES.len() - 1);
        assert_eq!(validation.seed_exception_count, 1);
        assert_eq!(validation.policy_digest_blake3.len(), BLAKE3_HEX_CHAR_COUNT);
    }

    #[test]
    fn seed_exception_policy_digest_is_order_independent_and_accounted() {
        let mut manifest = valid_manifest();
        make_seed_member(&mut manifest, 0);
        make_seed_member(&mut manifest, 1);
        let mut reordered = manifest.clone();
        reordered.members.reverse();
        reordered.seed_exceptions.reverse();

        let original = validate_toolchain_closure_manifest(&manifest).unwrap();
        let sorted = validate_toolchain_closure_manifest(&reordered).unwrap();

        assert_eq!(original.policy_digest_blake3, sorted.policy_digest_blake3);
        assert_eq!(original.member_count, REQUIRED_TOOLCHAIN_ROLES.len());
        assert_eq!(original.source_built_member_count, REQUIRED_TOOLCHAIN_ROLES.len() - TWO_SEED_EXCEPTION_COUNT);
        assert_eq!(original.seed_exception_count, TWO_SEED_EXCEPTION_COUNT);
    }

    #[test]
    fn policy_digest_changes_when_seed_exception_reason_changes() {
        let mut manifest = valid_manifest();
        make_seed_member(&mut manifest, 0);
        let mut changed = manifest.clone();
        changed.seed_exceptions[0].reason = "documented different trust root".to_string();

        let original = validate_toolchain_closure_manifest(&manifest).unwrap();
        let changed = validate_toolchain_closure_manifest(&changed).unwrap();

        assert_ne!(original.policy_digest_blake3, changed.policy_digest_blake3);
        assert_eq!(original.seed_exception_count, changed.seed_exception_count);
        assert_eq!(original.member_count, changed.member_count);
    }

    #[test]
    fn validated_closure_status_keeps_claim_disabled_until_enforcement_lands() {
        let manifest = valid_manifest();
        let validation = validate_toolchain_closure_manifest(&manifest).unwrap();

        let status = validated_source_built_toolchain_closure(PathBuf::from("/tmp/toolchain.json"), &validation);

        assert_eq!(status.schema, SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA);
        assert_eq!(status.status, VALIDATED_NOT_ENFORCED_STATUS);
        assert!(!status.claim);
        assert_eq!(status.non_claim, SOURCE_BUILT_NON_CLAIM);
        assert_eq!(status.manifest_path, Some(PathBuf::from("/tmp/toolchain.json")));
        assert_eq!(status.policy_digest_blake3, Some(validation.policy_digest_blake3));
        assert_eq!(status.member_count, Some(REQUIRED_TOOLCHAIN_ROLES.len()));
    }

    #[test]
    fn enforced_closure_status_still_keeps_claim_disabled_until_real_proof_lands() {
        let manifest = valid_manifest();
        let validation = validate_toolchain_closure_manifest(&manifest).unwrap();

        let status = enforced_source_built_toolchain_closure(PathBuf::from("/tmp/toolchain.json"), &validation);

        assert_eq!(status.schema, SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA);
        assert_eq!(status.status, VALIDATED_ENFORCED_STATUS);
        assert!(!status.claim);
        assert_eq!(status.non_claim, SOURCE_BUILT_NON_CLAIM);
        assert_eq!(status.policy_digest_blake3, Some(validation.policy_digest_blake3));
    }

    #[test]
    fn enforcement_accepts_declared_observed_toolchain_inputs() {
        let manifest = valid_manifest();
        let observed = vec![
            observed(ToolchainRole::Rustc, "/toolchain/rustc", DIGEST_A),
            observed(ToolchainRole::CCompiler, "/toolchain/cc", DIGEST_C),
            ToolchainObservedInput {
                role: ToolchainRole::Sysroot,
                execution_path: "/toolchain/sysroot".to_string(),
                content_digest_blake3: None,
            },
        ];

        let validation = enforce_observed_toolchain_inputs(&manifest, &observed).unwrap();

        assert_eq!(validation.member_count, REQUIRED_TOOLCHAIN_ROLES.len());
        assert_eq!(validation.source_built_member_count, REQUIRED_TOOLCHAIN_ROLES.len());
    }

    #[test]
    fn enforcement_rejects_undeclared_host_rustc_path() {
        let manifest = valid_manifest();
        let observed = vec![observed(
            ToolchainRole::Rustc,
            "/home/user/.rustup/toolchains/nightly/bin/rustc",
            DIGEST_A,
        )];

        let err = enforce_observed_toolchain_inputs(&manifest, &observed).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::HostToolLeakage);
        assert!(err.message().contains("host-tool-leakage"));
        assert!(err.message().contains("Rustc"));
    }

    #[test]
    fn enforcement_rejects_undeclared_host_linker_path() {
        let manifest = valid_manifest();
        let observed = vec![observed(ToolchainRole::Linker, "/usr/bin/ld", DIGEST_B)];

        let err = enforce_observed_toolchain_inputs(&manifest, &observed).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::HostToolLeakage);
        assert!(err.message().contains("host-tool-leakage"));
        assert!(err.message().contains("Linker"));
    }

    #[test]
    fn enforcement_rejects_undeclared_host_pkg_config_path() {
        let manifest = valid_manifest();
        let observed = vec![observed(ToolchainRole::PkgConfig, "/usr/bin/pkg-config", DIGEST_E)];

        let err = enforce_observed_toolchain_inputs(&manifest, &observed).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::HostToolLeakage);
        assert!(err.message().contains("host-tool-leakage"));
        assert!(err.message().contains("PkgConfig"));
    }

    #[test]
    fn enforcement_rejects_declared_path_with_digest_mismatch() {
        let manifest = valid_manifest();
        let observed = vec![observed(ToolchainRole::Rustc, "/toolchain/rustc", DIGEST_B)];

        let err = enforce_observed_toolchain_inputs(&manifest, &observed).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::HostToolLeakage);
        assert!(err.message().contains("digest mismatch"));
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
    fn validator_allows_seed_reason_with_marker_letters_inside_larger_word() {
        let mut manifest = valid_manifest();
        make_seed_member(&mut manifest, 0);
        manifest.seed_exceptions[0].reason = "autodoc bootstrap trust root".to_string();

        let validation = validate_toolchain_closure_manifest(&manifest).unwrap();

        assert_eq!(validation.member_count, REQUIRED_TOOLCHAIN_ROLES.len());
        assert_eq!(validation.source_built_member_count, REQUIRED_TOOLCHAIN_ROLES.len() - 1);
        assert_eq!(validation.seed_exception_count, 1);
    }

    #[test]
    fn validator_rejects_placeholder_seed_exception_reason() {
        let mut manifest = valid_manifest();
        make_seed_member(&mut manifest, 0);
        manifest.seed_exceptions[0].reason = "TODO placeholder source-root contract".to_string();

        let err = validate_toolchain_closure_manifest(&manifest).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::PlaceholderSeedException);
        assert!(err.message().contains("seed reason"));
        assert!(err.message().contains("placeholder"));
    }

    #[test]
    fn validator_rejects_unverified_seed_exception_name() {
        let mut manifest = valid_manifest();
        make_seed_member(&mut manifest, 0);
        manifest.seed_exceptions[0].name = "unverified-stage0-seed".to_string();
        manifest.members[0].name = "unverified-stage0-seed".to_string();

        let err = validate_toolchain_closure_manifest(&manifest).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::PlaceholderSeedException);
        assert!(err.message().contains("seed name"));
        assert!(err.message().contains("unverified"));
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

    fn member_json(
        role: &str,
        name: &str,
        execution_path: &str,
        content_digest_blake3: &str,
        receipt_digest_blake3: &str,
        trust: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "role": role,
            "name": name,
            "execution_path": execution_path,
            "content_digest_blake3": content_digest_blake3,
            "trust": trust,
            "source": {
                "kind": "tarball",
                "name": format!("{name}-source"),
                "digest_blake3": DIGEST_F,
            },
            "build_receipt": {
                "kind": "mantle-rust-topology",
                "name": format!("{name}-receipt"),
                "digest_blake3": receipt_digest_blake3,
            },
        })
    }

    fn seed_member_json(
        role: &str,
        name: &str,
        execution_path: &str,
        content_digest_blake3: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "role": role,
            "name": name,
            "execution_path": execution_path,
            "content_digest_blake3": content_digest_blake3,
            "trust": "seed-exception",
        })
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

    fn observed(role: ToolchainRole, execution_path: &str, digest: &str) -> ToolchainObservedInput {
        ToolchainObservedInput {
            role,
            execution_path: execution_path.to_string(),
            content_digest_blake3: Some(digest.to_string()),
        }
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
