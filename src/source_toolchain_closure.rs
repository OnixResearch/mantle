//! Pure source-built toolchain closure manifest validation.
//!
//! This module intentionally performs no I/O. Shell code is responsible for
//! reading manifests and checking file contents; this core validates the
//! receipt-bound shape and computes the deterministic policy digest.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::fmt;
use std::path::Component;
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
pub(crate) const RUST_SOURCE_PROVIDER_SCHEMA: &str = "mantle-rust-source-provider-v1";
pub(crate) const RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA: &str = "mantle-rust-source-provider-receipt-v1";
pub(crate) const RUST_SOURCE_PROVIDER_ID: &str = "mantle-rust-source-provider";
pub(crate) const RUST_SOURCE_PROVIDER_METADATA_PATH: &str = "share/mantle-rust-provider/provider.json";
pub(crate) const RUST_SOURCE_PROVIDER_RECEIPTS_DIR: &str = "share/mantle-rust-provider/receipts";
const RUST_PROVIDER_POLICY_DIGEST_CONTEXT: &str = "mantle-rust-source-provider-policy-digest-v1";
const MAX_RUST_PROVIDER_ARTIFACTS: usize = 256;
const MAX_RUST_PROVIDER_SOURCES: usize = 64;
const MAX_RUST_PROVIDER_RECEIPTS: usize = 64;
const MAX_RUST_PROVIDER_RECEIPT_STEPS: usize = 128;
const MAX_RUST_PROVIDER_RECEIPT_ARGUMENTS: usize = 64;
const REQUIRED_RUST_PROVIDER_ROLES: [RustProviderRole; 5] = [
    RustProviderRole::Rustc,
    RustProviderRole::Cargo,
    RustProviderRole::HostRustlib,
    RustProviderRole::TargetRustlib,
    RustProviderRole::ProviderReceipt,
];
const DISALLOWED_SEED_EXCEPTION_MARKERS: [&str; 3] = ["placeholder", "todo", "unverified"];
const DISALLOWED_RUST_PROVIDER_MARKERS: [&str; 8] = [
    "placeholder",
    "todo",
    "unverified",
    "prebuilt",
    "rustup",
    "nix",
    "/nix/store",
    "wrapper",
];
const RUSTC_PROVIDER_PATH: &str = "bin/rustc";
const CARGO_PROVIDER_PATH: &str = "bin/cargo";
const RUSTDOC_PROVIDER_PATH: &str = "bin/rustdoc";
const RUSTLIB_PROVIDER_PREFIX: &str = "lib/rustlib";
const PROVIDER_RECEIPT_EXTENSION: &str = ".json";

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustSourceProviderMetadata {
    pub(crate) schema: String,
    pub(crate) provider_id: String,
    pub(crate) host_triple: String,
    pub(crate) target_triple: String,
    pub(crate) provenance: RustSourceProviderProvenance,
    pub(crate) sources: Vec<RustProviderSourceIdentity>,
    pub(crate) build_receipts: Vec<RustProviderBuildReceiptIdentity>,
    pub(crate) artifacts: Vec<RustProviderArtifact>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustSourceProviderBuildReceipt {
    pub(crate) schema: String,
    pub(crate) receipt_id: String,
    pub(crate) provider_id: String,
    pub(crate) host_triple: String,
    pub(crate) target_triple: String,
    pub(crate) source_ids: Vec<String>,
    pub(crate) output_artifacts: Vec<RustProviderReceiptArtifact>,
    pub(crate) build_steps: Vec<RustProviderReceiptStep>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustProviderReceiptArtifact {
    pub(crate) role: RustProviderRole,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) content_digest_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustProviderReceiptStep {
    pub(crate) name: String,
    pub(crate) program: String,
    pub(crate) arguments: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustSourceProviderProvenance {
    pub(crate) source_built: bool,
    pub(crate) uses_prebuilt_rust: bool,
    pub(crate) build_recipe: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustProviderSourceIdentity {
    pub(crate) id: String,
    pub(crate) kind: ToolchainSourceKind,
    pub(crate) name: String,
    pub(crate) digest_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustProviderBuildReceiptIdentity {
    pub(crate) id: String,
    pub(crate) kind: ToolchainBuildReceiptKind,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) digest_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustProviderArtifact {
    pub(crate) role: RustProviderRole,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) content_digest_blake3: String,
    pub(crate) source_id: String,
    pub(crate) build_receipt_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RustProviderRole {
    Rustc,
    Cargo,
    Rustdoc,
    HostRustlib,
    TargetRustlib,
    ProviderReceipt,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RustProviderObservedArtifact {
    pub(crate) role: RustProviderRole,
    pub(crate) path: String,
    pub(crate) content_digest_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RustProviderObservedBuildReceipt {
    pub(crate) path: String,
    pub(crate) content_digest_blake3: String,
    pub(crate) receipt: RustSourceProviderBuildReceipt,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct RustSourceProviderValidation {
    pub(crate) policy_digest_blake3: String,
    pub(crate) artifact_count: usize,
    pub(crate) source_count: usize,
    pub(crate) receipt_count: usize,
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
    InvalidRustProvider,
    PrebuiltRustProvider,
    MissingRustProviderRole,
    DuplicateRustProviderItem,
    DigestMismatch,
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

pub(crate) fn validate_rust_source_provider_metadata(
    metadata: &RustSourceProviderMetadata,
) -> Result<RustSourceProviderValidation, ToolchainClosureError> {
    let normalized = normalize_rust_source_provider_metadata(metadata)?;
    validation_from_normalized_rust_provider(&normalized)
}

pub(crate) fn enforce_observed_rust_source_provider_artifacts(
    metadata: &RustSourceProviderMetadata,
    observed: &[RustProviderObservedArtifact],
) -> Result<RustSourceProviderValidation, ToolchainClosureError> {
    let normalized = normalize_rust_source_provider_metadata(metadata)?;
    validate_observed_rust_provider_artifacts(observed)?;
    for artifact in observed {
        require_observed_rust_provider_artifact_declared(&normalized, artifact)?;
    }
    validation_from_normalized_rust_provider(&normalized)
}

pub(crate) fn enforce_observed_rust_source_provider_receipts(
    metadata: &RustSourceProviderMetadata,
    observed: &[RustProviderObservedBuildReceipt],
) -> Result<RustSourceProviderValidation, ToolchainClosureError> {
    let normalized = normalize_rust_source_provider_metadata(metadata)?;
    validate_observed_rust_provider_receipts(&normalized, observed)?;
    validation_from_normalized_rust_provider(&normalized)
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
    validate_source_built_rust_member_claim(member, source, receipt)
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

fn validate_source_built_rust_member_claim(
    member: &ToolchainClosureMember,
    source: &ToolchainSourceIdentity,
    receipt: &ToolchainBuildReceiptIdentity,
) -> Result<(), ToolchainClosureError> {
    if !matches!(member.role, ToolchainRole::Rustc | ToolchainRole::Sysroot) {
        return Ok(());
    }
    validate_no_disallowed_rust_provider_text("source-built Rust member source name", &source.name)?;
    validate_no_disallowed_rust_provider_text("source-built Rust member build receipt name", &receipt.name)
}

fn normalize_rust_source_provider_metadata(
    metadata: &RustSourceProviderMetadata,
) -> Result<RustSourceProviderMetadata, ToolchainClosureError> {
    validate_rust_provider_schema(&metadata.schema)?;
    validate_rust_provider_identity(metadata)?;
    let source_ids = validate_rust_provider_sources(&metadata.sources)?;
    let receipt_ids = validate_rust_provider_receipts(&metadata.build_receipts)?;
    validate_rust_provider_artifacts(metadata, &source_ids, &receipt_ids)?;
    validate_rust_provider_receipt_artifact_links(metadata)?;
    let mut normalized = metadata.clone();
    normalized.sources.sort_by_key(rust_provider_source_sort_key);
    normalized.build_receipts.sort_by_key(rust_provider_receipt_sort_key);
    normalized.artifacts.sort_by_key(rust_provider_artifact_sort_key);
    Ok(normalized)
}

fn validation_from_normalized_rust_provider(
    metadata: &RustSourceProviderMetadata,
) -> Result<RustSourceProviderValidation, ToolchainClosureError> {
    Ok(RustSourceProviderValidation {
        policy_digest_blake3: digest_normalized_rust_provider(metadata)?,
        artifact_count: metadata.artifacts.len(),
        source_count: metadata.sources.len(),
        receipt_count: metadata.build_receipts.len(),
    })
}

fn validate_rust_provider_schema(schema: &str) -> Result<(), ToolchainClosureError> {
    if schema == RUST_SOURCE_PROVIDER_SCHEMA {
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::InvalidRustProvider,
        format!("expected schema {RUST_SOURCE_PROVIDER_SCHEMA}, got {schema}"),
    ))
}

fn validate_rust_provider_identity(metadata: &RustSourceProviderMetadata) -> Result<(), ToolchainClosureError> {
    if metadata.provider_id != RUST_SOURCE_PROVIDER_ID {
        return Err(error(
            ToolchainClosureErrorKind::InvalidRustProvider,
            format!("expected provider_id {RUST_SOURCE_PROVIDER_ID}, got {}", metadata.provider_id),
        ));
    }
    validate_non_empty("rust provider host_triple", &metadata.host_triple)?;
    validate_non_empty("rust provider target_triple", &metadata.target_triple)?;
    validate_rust_provider_provenance(&metadata.provenance)?;
    validate_collection_count("rust provider sources", metadata.sources.len(), MAX_RUST_PROVIDER_SOURCES)?;
    validate_collection_count(
        "rust provider build_receipts",
        metadata.build_receipts.len(),
        MAX_RUST_PROVIDER_RECEIPTS,
    )?;
    validate_collection_count("rust provider artifacts", metadata.artifacts.len(), MAX_RUST_PROVIDER_ARTIFACTS)
}

fn validate_rust_provider_provenance(provenance: &RustSourceProviderProvenance) -> Result<(), ToolchainClosureError> {
    validate_non_empty("rust provider build_recipe", &provenance.build_recipe)?;
    validate_no_disallowed_rust_provider_text("rust provider build_recipe", &provenance.build_recipe)?;
    if !provenance.source_built {
        return Err(error(
            ToolchainClosureErrorKind::PrebuiltRustProvider,
            "rust provider provenance is not source-built",
        ));
    }
    if provenance.uses_prebuilt_rust {
        return Err(error(
            ToolchainClosureErrorKind::PrebuiltRustProvider,
            "rust provider provenance uses prebuilt Rust",
        ));
    }
    Ok(())
}

fn validate_collection_count(label: &str, len: usize, max: usize) -> Result<(), ToolchainClosureError> {
    if len == 0 {
        return Err(error(ToolchainClosureErrorKind::InvalidRustProvider, format!("{label} is empty")));
    }
    validate_max_collection_count(label, len, max)
}

fn validate_max_collection_count(label: &str, len: usize, max: usize) -> Result<(), ToolchainClosureError> {
    if len > max {
        return Err(error(
            ToolchainClosureErrorKind::InvalidRustProvider,
            format!("{label} has more than {max} entries"),
        ));
    }
    Ok(())
}

fn validate_rust_provider_sources(
    sources: &[RustProviderSourceIdentity],
) -> Result<BTreeSet<String>, ToolchainClosureError> {
    let mut ids = BTreeSet::new();
    for source in sources {
        validate_non_empty("rust provider source id", &source.id)?;
        validate_non_empty("rust provider source name", &source.name)?;
        validate_no_disallowed_rust_provider_text("rust provider source id", &source.id)?;
        validate_no_disallowed_rust_provider_text("rust provider source name", &source.name)?;
        validate_blake3_hex("rust provider source digest_blake3", &source.digest_blake3)?;
        if !ids.insert(source.id.clone()) {
            return Err(error(
                ToolchainClosureErrorKind::DuplicateRustProviderItem,
                format!("duplicate rust provider source id '{}'", source.id),
            ));
        }
    }
    Ok(ids)
}

fn validate_rust_provider_receipts(
    receipts: &[RustProviderBuildReceiptIdentity],
) -> Result<BTreeSet<String>, ToolchainClosureError> {
    let mut ids = BTreeSet::new();
    for receipt in receipts {
        validate_non_empty("rust provider receipt id", &receipt.id)?;
        validate_non_empty("rust provider receipt name", &receipt.name)?;
        validate_provider_relative_path("rust provider receipt path", &receipt.path)?;
        validate_receipt_path_prefix(&receipt.path)?;
        validate_no_disallowed_rust_provider_text("rust provider receipt id", &receipt.id)?;
        validate_no_disallowed_rust_provider_text("rust provider receipt name", &receipt.name)?;
        validate_blake3_hex("rust provider receipt digest_blake3", &receipt.digest_blake3)?;
        if !ids.insert(receipt.id.clone()) {
            return Err(error(
                ToolchainClosureErrorKind::DuplicateRustProviderItem,
                format!("duplicate rust provider receipt id '{}'", receipt.id),
            ));
        }
    }
    Ok(ids)
}

fn validate_rust_provider_artifacts(
    metadata: &RustSourceProviderMetadata,
    source_ids: &BTreeSet<String>,
    receipt_ids: &BTreeSet<String>,
) -> Result<(), ToolchainClosureError> {
    let mut artifact_keys = BTreeSet::new();
    let mut roles = BTreeSet::new();
    for artifact in &metadata.artifacts {
        validate_rust_provider_artifact(metadata, artifact, source_ids, receipt_ids)?;
        roles.insert(artifact.role);
        if !artifact_keys.insert((artifact.role, artifact.name.clone(), artifact.path.clone())) {
            return Err(error(
                ToolchainClosureErrorKind::DuplicateRustProviderItem,
                format!("duplicate rust provider artifact '{}'", artifact.name),
            ));
        }
    }
    require_rust_provider_roles(&roles)
}

fn validate_rust_provider_artifact(
    metadata: &RustSourceProviderMetadata,
    artifact: &RustProviderArtifact,
    source_ids: &BTreeSet<String>,
    receipt_ids: &BTreeSet<String>,
) -> Result<(), ToolchainClosureError> {
    validate_non_empty("rust provider artifact name", &artifact.name)?;
    validate_no_disallowed_rust_provider_text("rust provider artifact name", &artifact.name)?;
    validate_provider_relative_path("rust provider artifact path", &artifact.path)?;
    validate_rust_provider_role_path(metadata, artifact)?;
    validate_blake3_hex("rust provider artifact content_digest_blake3", &artifact.content_digest_blake3)?;
    if !source_ids.contains(&artifact.source_id) {
        return Err(error(
            ToolchainClosureErrorKind::InvalidRustProvider,
            format!("rust provider artifact '{}' references unknown source_id '{}'", artifact.name, artifact.source_id),
        ));
    }
    if !receipt_ids.contains(&artifact.build_receipt_id) {
        return Err(error(
            ToolchainClosureErrorKind::InvalidRustProvider,
            format!(
                "rust provider artifact '{}' references unknown build_receipt_id '{}'",
                artifact.name, artifact.build_receipt_id
            ),
        ));
    }
    Ok(())
}

fn validate_rust_provider_receipt_artifact_links(
    metadata: &RustSourceProviderMetadata,
) -> Result<(), ToolchainClosureError> {
    for receipt in &metadata.build_receipts {
        let Some(artifact) = metadata
            .artifacts
            .iter()
            .find(|artifact| artifact.role == RustProviderRole::ProviderReceipt && artifact.path == receipt.path)
        else {
            return Err(error(
                ToolchainClosureErrorKind::MissingRustProviderRole,
                format!("rust provider receipt '{}' has no provider-receipt artifact", receipt.id),
            ));
        };
        if artifact.build_receipt_id != receipt.id {
            return Err(error(
                ToolchainClosureErrorKind::InvalidRustProvider,
                format!(
                    "rust provider receipt '{}' artifact references build_receipt_id '{}'",
                    receipt.id, artifact.build_receipt_id
                ),
            ));
        }
        if artifact.content_digest_blake3 != receipt.digest_blake3 {
            return Err(error(
                ToolchainClosureErrorKind::DigestMismatch,
                format!("rust provider receipt '{}' digest does not match provider-receipt artifact", receipt.id),
            ));
        }
    }
    Ok(())
}

fn require_rust_provider_roles(roles: &BTreeSet<RustProviderRole>) -> Result<(), ToolchainClosureError> {
    for role in REQUIRED_RUST_PROVIDER_ROLES {
        if !roles.contains(&role) {
            return Err(error(
                ToolchainClosureErrorKind::MissingRustProviderRole,
                format!("missing required rust provider role {role:?}"),
            ));
        }
    }
    Ok(())
}

fn validate_rust_provider_role_path(
    metadata: &RustSourceProviderMetadata,
    artifact: &RustProviderArtifact,
) -> Result<(), ToolchainClosureError> {
    match artifact.role {
        RustProviderRole::Rustc => require_provider_path(&artifact.path, RUSTC_PROVIDER_PATH, artifact.role),
        RustProviderRole::Cargo => require_provider_path(&artifact.path, CARGO_PROVIDER_PATH, artifact.role),
        RustProviderRole::Rustdoc => require_provider_path(&artifact.path, RUSTDOC_PROVIDER_PATH, artifact.role),
        RustProviderRole::HostRustlib => {
            require_provider_path_prefix(&artifact.path, &rustlib_lib_prefix(&metadata.host_triple), artifact.role)
        }
        RustProviderRole::TargetRustlib => {
            require_provider_path_prefix(&artifact.path, &rustlib_lib_prefix(&metadata.target_triple), artifact.role)
        }
        RustProviderRole::ProviderReceipt => validate_provider_receipt_path(&artifact.path),
    }
}

fn require_provider_path(path: &str, expected: &str, role: RustProviderRole) -> Result<(), ToolchainClosureError> {
    if path == expected {
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::InvalidRustProvider,
        format!("rust provider role {role:?} must use path {expected}, got {path}"),
    ))
}

fn require_provider_path_prefix(path: &str, prefix: &str, role: RustProviderRole) -> Result<(), ToolchainClosureError> {
    if path == prefix || path.starts_with(&format!("{prefix}/")) {
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::InvalidRustProvider,
        format!("rust provider role {role:?} must live under {prefix}, got {path}"),
    ))
}

fn rustlib_lib_prefix(triple: &str) -> String {
    format!("{RUSTLIB_PROVIDER_PREFIX}/{triple}/lib")
}

fn validate_receipt_path_prefix(path: &str) -> Result<(), ToolchainClosureError> {
    validate_provider_receipt_path(path)
}

fn validate_provider_receipt_path(path: &str) -> Result<(), ToolchainClosureError> {
    let receipt_prefix = format!("{RUST_SOURCE_PROVIDER_RECEIPTS_DIR}/");
    if path.starts_with(&receipt_prefix) && path.ends_with(PROVIDER_RECEIPT_EXTENSION) {
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::InvalidRustProvider,
        format!(
            "rust provider receipt path must live under {RUST_SOURCE_PROVIDER_RECEIPTS_DIR} and end with {PROVIDER_RECEIPT_EXTENSION}, got {path}"
        ),
    ))
}

fn validate_provider_relative_path(label: &str, value: &str) -> Result<(), ToolchainClosureError> {
    validate_non_empty(label, value)?;
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(error(ToolchainClosureErrorKind::InvalidRustProvider, format!("{label} must be relative")));
    }
    if path.components().any(|component| {
        matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_) | Component::CurDir)
    }) {
        return Err(error(
            ToolchainClosureErrorKind::InvalidRustProvider,
            format!("{label} must not contain parent/current/root components"),
        ));
    }
    Ok(())
}

fn validate_observed_rust_provider_artifacts(
    observed: &[RustProviderObservedArtifact],
) -> Result<(), ToolchainClosureError> {
    for artifact in observed {
        validate_provider_relative_path("observed rust provider artifact path", &artifact.path)?;
        validate_blake3_hex("observed rust provider artifact content_digest_blake3", &artifact.content_digest_blake3)?;
    }
    Ok(())
}

fn require_observed_rust_provider_artifact_declared(
    metadata: &RustSourceProviderMetadata,
    observed: &RustProviderObservedArtifact,
) -> Result<(), ToolchainClosureError> {
    let Some(artifact) = metadata
        .artifacts
        .iter()
        .find(|artifact| artifact.role == observed.role && artifact.path == observed.path)
    else {
        return Err(error(
            ToolchainClosureErrorKind::MissingRustProviderRole,
            format!("observed rust provider artifact {:?} at {} is undeclared", observed.role, observed.path),
        ));
    };
    if artifact.content_digest_blake3 == observed.content_digest_blake3 {
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::DigestMismatch,
        format!("rust provider artifact {:?} digest mismatch at {}", observed.role, observed.path),
    ))
}

fn validate_observed_rust_provider_receipts(
    metadata: &RustSourceProviderMetadata,
    observed: &[RustProviderObservedBuildReceipt],
) -> Result<(), ToolchainClosureError> {
    validate_collection_count("observed rust provider receipts", observed.len(), MAX_RUST_PROVIDER_RECEIPTS)?;
    let mut observed_paths = BTreeSet::new();
    for receipt in observed {
        validate_provider_relative_path("observed rust provider receipt path", &receipt.path)?;
        validate_receipt_path_prefix(&receipt.path)?;
        validate_blake3_hex("observed rust provider receipt content_digest_blake3", &receipt.content_digest_blake3)?;
        if !observed_paths.insert(receipt.path.clone()) {
            return Err(error(
                ToolchainClosureErrorKind::DuplicateRustProviderItem,
                format!("duplicate observed rust provider receipt '{}'", receipt.path),
            ));
        }
        validate_observed_rust_provider_receipt(metadata, receipt)?;
    }
    require_declared_rust_provider_receipts_observed(metadata, &observed_paths)
}

fn validate_observed_rust_provider_receipt(
    metadata: &RustSourceProviderMetadata,
    observed: &RustProviderObservedBuildReceipt,
) -> Result<(), ToolchainClosureError> {
    let Some(declared) = metadata.build_receipts.iter().find(|receipt| receipt.path == observed.path) else {
        return Err(error(
            ToolchainClosureErrorKind::MissingRustProviderRole,
            format!("observed rust provider receipt {} is undeclared", observed.path),
        ));
    };
    if declared.digest_blake3 != observed.content_digest_blake3 {
        return Err(error(
            ToolchainClosureErrorKind::DigestMismatch,
            format!("rust provider receipt '{}' digest mismatch", declared.id),
        ));
    }
    validate_rust_provider_receipt_payload(metadata, declared, &observed.receipt)
}

fn require_declared_rust_provider_receipts_observed(
    metadata: &RustSourceProviderMetadata,
    observed_paths: &BTreeSet<String>,
) -> Result<(), ToolchainClosureError> {
    for receipt in &metadata.build_receipts {
        if !observed_paths.contains(&receipt.path) {
            return Err(error(
                ToolchainClosureErrorKind::MissingRustProviderRole,
                format!("declared rust provider receipt '{}' was not observed", receipt.path),
            ));
        }
    }
    Ok(())
}

fn validate_rust_provider_receipt_payload(
    metadata: &RustSourceProviderMetadata,
    declared: &RustProviderBuildReceiptIdentity,
    receipt: &RustSourceProviderBuildReceipt,
) -> Result<(), ToolchainClosureError> {
    if receipt.schema != RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA {
        return Err(error(
            ToolchainClosureErrorKind::InvalidRustProvider,
            format!("expected receipt schema {RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA}, got {}", receipt.schema),
        ));
    }
    require_equal("rust provider receipt_id", &receipt.receipt_id, &declared.id)?;
    require_equal("rust provider receipt provider_id", &receipt.provider_id, &metadata.provider_id)?;
    require_equal("rust provider receipt host_triple", &receipt.host_triple, &metadata.host_triple)?;
    require_equal("rust provider receipt target_triple", &receipt.target_triple, &metadata.target_triple)?;
    validate_rust_provider_receipt_sources(metadata, receipt)?;
    validate_rust_provider_receipt_artifacts(metadata, receipt)?;
    validate_rust_provider_receipt_steps(receipt)
}

fn validate_rust_provider_receipt_sources(
    metadata: &RustSourceProviderMetadata,
    receipt: &RustSourceProviderBuildReceipt,
) -> Result<(), ToolchainClosureError> {
    validate_collection_count("rust provider receipt source_ids", receipt.source_ids.len(), MAX_RUST_PROVIDER_SOURCES)?;
    let declared_sources: BTreeSet<String> = metadata.sources.iter().map(|source| source.id.clone()).collect();
    let mut receipt_sources = BTreeSet::new();
    for source_id in &receipt.source_ids {
        validate_non_empty("rust provider receipt source_id", source_id)?;
        validate_no_disallowed_rust_provider_text("rust provider receipt source_id", source_id)?;
        if !declared_sources.contains(source_id) {
            return Err(error(
                ToolchainClosureErrorKind::InvalidRustProvider,
                format!("rust provider receipt references unknown source_id '{source_id}'"),
            ));
        }
        if !receipt_sources.insert(source_id.clone()) {
            return Err(error(
                ToolchainClosureErrorKind::DuplicateRustProviderItem,
                format!("duplicate rust provider receipt source_id '{source_id}'"),
            ));
        }
    }
    Ok(())
}

fn validate_rust_provider_receipt_artifacts(
    metadata: &RustSourceProviderMetadata,
    receipt: &RustSourceProviderBuildReceipt,
) -> Result<(), ToolchainClosureError> {
    validate_collection_count(
        "rust provider receipt output_artifacts",
        receipt.output_artifacts.len(),
        MAX_RUST_PROVIDER_ARTIFACTS,
    )?;
    let expected = metadata_artifacts_for_receipt(metadata, &receipt.receipt_id);
    let mut actual = BTreeSet::new();
    for artifact in &receipt.output_artifacts {
        validate_non_empty("rust provider receipt artifact name", &artifact.name)?;
        validate_no_disallowed_rust_provider_text("rust provider receipt artifact name", &artifact.name)?;
        validate_provider_relative_path("rust provider receipt artifact path", &artifact.path)?;
        validate_blake3_hex("rust provider receipt artifact content_digest_blake3", &artifact.content_digest_blake3)?;
        if !actual.insert(receipt_artifact_key(artifact)) {
            return Err(error(
                ToolchainClosureErrorKind::DuplicateRustProviderItem,
                format!("duplicate rust provider receipt artifact '{}'", artifact.name),
            ));
        }
    }
    if actual == expected {
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::InvalidRustProvider,
        format!("rust provider receipt '{}' output artifacts do not match metadata", receipt.receipt_id),
    ))
}

fn validate_rust_provider_receipt_steps(receipt: &RustSourceProviderBuildReceipt) -> Result<(), ToolchainClosureError> {
    validate_collection_count(
        "rust provider receipt build_steps",
        receipt.build_steps.len(),
        MAX_RUST_PROVIDER_RECEIPT_STEPS,
    )?;
    for step in &receipt.build_steps {
        validate_non_empty("rust provider receipt step name", &step.name)?;
        validate_non_empty("rust provider receipt step program", &step.program)?;
        validate_no_disallowed_rust_provider_text("rust provider receipt step name", &step.name)?;
        validate_no_disallowed_rust_provider_text("rust provider receipt step program", &step.program)?;
        validate_max_collection_count(
            "rust provider receipt step arguments",
            step.arguments.len(),
            MAX_RUST_PROVIDER_RECEIPT_ARGUMENTS,
        )?;
        for argument in &step.arguments {
            validate_no_disallowed_rust_provider_text("rust provider receipt step argument", argument)?;
        }
    }
    Ok(())
}

fn metadata_artifacts_for_receipt(
    metadata: &RustSourceProviderMetadata,
    receipt_id: &str,
) -> BTreeSet<(RustProviderRole, String, String, String)> {
    metadata
        .artifacts
        .iter()
        .filter(|artifact| {
            artifact.build_receipt_id == receipt_id && artifact.role != RustProviderRole::ProviderReceipt
        })
        .map(|artifact| {
            (artifact.role, artifact.name.clone(), artifact.path.clone(), artifact.content_digest_blake3.clone())
        })
        .collect()
}

fn receipt_artifact_key(artifact: &RustProviderReceiptArtifact) -> (RustProviderRole, String, String, String) {
    (artifact.role, artifact.name.clone(), artifact.path.clone(), artifact.content_digest_blake3.clone())
}

fn require_equal(label: &str, actual: &str, expected: &str) -> Result<(), ToolchainClosureError> {
    if actual == expected {
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::InvalidRustProvider,
        format!("{label} expected {expected}, got {actual}"),
    ))
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

fn validate_no_disallowed_rust_provider_text(label: &str, value: &str) -> Result<(), ToolchainClosureError> {
    if let Some(marker) = disallowed_rust_provider_marker(value) {
        return Err(error(
            ToolchainClosureErrorKind::PrebuiltRustProvider,
            format!("{label} contains disallowed Rust provider marker '{marker}'"),
        ));
    }
    Ok(())
}

pub(crate) fn disallowed_rust_provider_marker(value: &str) -> Option<&'static str> {
    let normalized = value.to_ascii_lowercase();
    DISALLOWED_RUST_PROVIDER_MARKERS
        .iter()
        .copied()
        .find(|marker| contains_disallowed_rust_provider_marker(&normalized, marker))
}

fn contains_disallowed_rust_provider_marker(value: &str, marker: &str) -> bool {
    debug_assert!(!marker.is_empty());
    if marker.bytes().any(|byte| !byte.is_ascii_alphanumeric() && byte != b'_') {
        return value.contains(marker);
    }
    contains_disallowed_seed_exception_marker(value, marker)
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

fn digest_normalized_rust_provider(metadata: &RustSourceProviderMetadata) -> Result<String, ToolchainClosureError> {
    let payload = serde_json::json!({
        "context": RUST_PROVIDER_POLICY_DIGEST_CONTEXT,
        "metadata": metadata,
    });
    let bytes = serde_json::to_vec(&payload).map_err(|err| {
        error(ToolchainClosureErrorKind::Serialization, format!("serialize rust provider policy: {err}"))
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

fn rust_provider_source_sort_key(source: &RustProviderSourceIdentity) -> (String, String) {
    (source.id.clone(), source.digest_blake3.clone())
}

fn rust_provider_receipt_sort_key(receipt: &RustProviderBuildReceiptIdentity) -> (String, String) {
    (receipt.id.clone(), receipt.digest_blake3.clone())
}

fn rust_provider_artifact_sort_key(artifact: &RustProviderArtifact) -> (RustProviderRole, String, String) {
    (artifact.role, artifact.name.clone(), artifact.path.clone())
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

    #[test]
    fn validator_rejects_source_built_rustc_with_prebuilt_source_marker() {
        let mut manifest = valid_manifest();
        manifest.members[0].source.as_mut().unwrap().name = "nix rustup prebuilt rustc".to_string();

        let err = validate_toolchain_closure_manifest(&manifest).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::PrebuiltRustProvider);
        assert!(err.message().contains("source-built Rust member source name"));
        assert!(err.message().contains("prebuilt"));
    }

    #[test]
    fn valid_rust_source_provider_metadata_yields_stable_digest() {
        let metadata = valid_rust_provider_metadata();
        let mut reordered = metadata.clone();
        reordered.artifacts.reverse();
        reordered.sources.reverse();
        reordered.build_receipts.reverse();

        let validation = validate_rust_source_provider_metadata(&metadata).unwrap();
        let validation_reordered = validate_rust_source_provider_metadata(&reordered).unwrap();

        assert_eq!(validation.artifact_count, REQUIRED_RUST_PROVIDER_ROLES.len());
        assert_eq!(validation.source_count, 1);
        assert_eq!(validation.receipt_count, 1);
        assert_eq!(validation.policy_digest_blake3, validation_reordered.policy_digest_blake3);
    }

    #[test]
    fn rust_source_provider_rejects_prebuilt_provenance() {
        let mut metadata = valid_rust_provider_metadata();
        metadata.provenance.uses_prebuilt_rust = true;

        let err = validate_rust_source_provider_metadata(&metadata).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::PrebuiltRustProvider);
        assert!(err.message().contains("uses prebuilt Rust"));
    }

    #[test]
    fn rust_source_provider_rejects_missing_target_rustlib_role() {
        let mut metadata = valid_rust_provider_metadata();
        metadata.artifacts.retain(|artifact| artifact.role != RustProviderRole::TargetRustlib);

        let err = validate_rust_source_provider_metadata(&metadata).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::MissingRustProviderRole);
        assert!(err.message().contains("TargetRustlib"));
    }

    #[test]
    fn rust_source_provider_rejects_digest_mismatch_from_shell_observation() {
        let metadata = valid_rust_provider_metadata();
        let mut observed = observed_rust_provider_artifacts(&metadata);
        observed[0].content_digest_blake3 = DIGEST_F.to_string();

        let err = enforce_observed_rust_source_provider_artifacts(&metadata, &observed).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::DigestMismatch);
        assert!(err.message().contains("digest mismatch"));
    }

    #[test]
    fn rust_source_provider_accepts_matching_receipt_payload() {
        let metadata = valid_rust_provider_metadata();
        let observed = observed_rust_provider_receipts(&metadata);

        let validation = enforce_observed_rust_source_provider_receipts(&metadata, &observed).unwrap();

        assert_eq!(validation.receipt_count, 1);
        assert_eq!(validation.artifact_count, REQUIRED_RUST_PROVIDER_ROLES.len());
    }

    #[test]
    fn rust_source_provider_rejects_bad_receipt_schema() {
        let metadata = valid_rust_provider_metadata();
        let mut observed = observed_rust_provider_receipts(&metadata);
        observed[0].receipt.schema = "mantle-rust-provider-receipt-v0".to_string();

        let err = enforce_observed_rust_source_provider_receipts(&metadata, &observed).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::InvalidRustProvider);
        assert!(err.message().contains(RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA));
    }

    #[test]
    fn rust_source_provider_rejects_receipt_with_prebuilt_step() {
        let metadata = valid_rust_provider_metadata();
        let mut observed = observed_rust_provider_receipts(&metadata);
        observed[0].receipt.build_steps[0].program = "nix-build".to_string();

        let err = enforce_observed_rust_source_provider_receipts(&metadata, &observed).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::PrebuiltRustProvider);
        assert!(err.message().contains("rust provider receipt step program"));
        assert!(err.message().contains("nix"));
    }

    #[test]
    fn rust_source_provider_rejects_receipt_artifact_mismatch() {
        let metadata = valid_rust_provider_metadata();
        let mut observed = observed_rust_provider_receipts(&metadata);
        observed[0].receipt.output_artifacts.pop();

        let err = enforce_observed_rust_source_provider_receipts(&metadata, &observed).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::InvalidRustProvider);
        assert!(err.message().contains("output artifacts do not match metadata"));
    }

    #[test]
    fn rust_source_provider_rejects_receipt_digest_mismatch() {
        let metadata = valid_rust_provider_metadata();
        let mut observed = observed_rust_provider_receipts(&metadata);
        observed[0].content_digest_blake3 = DIGEST_F.to_string();

        let err = enforce_observed_rust_source_provider_receipts(&metadata, &observed).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::DigestMismatch);
        assert!(err.message().contains("receipt 'build-receipt' digest mismatch"));
    }

    #[test]
    fn rust_source_provider_rejects_rustlib_outside_lib_dir() {
        let mut metadata = valid_rust_provider_metadata();
        let host_rustlib = metadata
            .artifacts
            .iter_mut()
            .find(|artifact| artifact.role == RustProviderRole::HostRustlib)
            .unwrap();
        host_rustlib.path = "lib/rustlib/x86_64-unknown-linux-gnu".to_string();

        let err = validate_rust_source_provider_metadata(&metadata).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::InvalidRustProvider);
        assert!(err.message().contains("HostRustlib"));
        assert!(err.message().contains("/lib"));
    }

    #[test]
    fn rust_source_provider_rejects_provider_receipt_digest_link_mismatch() {
        let mut metadata = valid_rust_provider_metadata();
        let provider_receipt = metadata
            .artifacts
            .iter_mut()
            .find(|artifact| artifact.role == RustProviderRole::ProviderReceipt)
            .unwrap();
        provider_receipt.content_digest_blake3 = DIGEST_A.to_string();

        let err = validate_rust_source_provider_metadata(&metadata).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::DigestMismatch);
        assert!(err.message().contains("provider-receipt artifact"));
    }

    #[test]
    fn rust_source_provider_rejects_prebuilt_source_name() {
        let mut metadata = valid_rust_provider_metadata();
        metadata.sources[0].name = "rustup wrapped compiler".to_string();

        let err = validate_rust_source_provider_metadata(&metadata).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::PrebuiltRustProvider);
        assert!(err.message().contains("rust provider source name"));
        assert!(err.message().contains("rustup"));
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

    fn valid_rust_provider_metadata() -> RustSourceProviderMetadata {
        RustSourceProviderMetadata {
            schema: RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
            provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
            host_triple: "x86_64-unknown-linux-gnu".to_string(),
            target_triple: "x86_64-unknown-linux-musl".to_string(),
            provenance: RustSourceProviderProvenance {
                source_built: true,
                uses_prebuilt_rust: false,
                build_recipe: "bootstrap/rust-source.ncl".to_string(),
            },
            sources: vec![RustProviderSourceIdentity {
                id: "rust-src".to_string(),
                kind: ToolchainSourceKind::Tarball,
                name: "rust-compiler-source".to_string(),
                digest_blake3: DIGEST_A.to_string(),
            }],
            build_receipts: vec![RustProviderBuildReceiptIdentity {
                id: "build-receipt".to_string(),
                kind: ToolchainBuildReceiptKind::MantleDerivation,
                name: "rust-source-build-receipt".to_string(),
                path: "share/mantle-rust-provider/receipts/build.json".to_string(),
                digest_blake3: DIGEST_B.to_string(),
            }],
            artifacts: vec![
                rust_provider_artifact(RustProviderRole::Rustc, "rustc", RUSTC_PROVIDER_PATH, DIGEST_C),
                rust_provider_artifact(RustProviderRole::Cargo, "cargo", CARGO_PROVIDER_PATH, DIGEST_D),
                rust_provider_artifact(
                    RustProviderRole::HostRustlib,
                    "host-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-gnu/lib",
                    DIGEST_E,
                ),
                rust_provider_artifact(
                    RustProviderRole::TargetRustlib,
                    "target-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-musl/lib",
                    DIGEST_F,
                ),
                rust_provider_artifact(
                    RustProviderRole::ProviderReceipt,
                    "build-receipt",
                    "share/mantle-rust-provider/receipts/build.json",
                    DIGEST_B,
                ),
            ],
        }
    }

    fn rust_provider_artifact(
        role: RustProviderRole,
        name: &str,
        path: &str,
        content_digest_blake3: &str,
    ) -> RustProviderArtifact {
        RustProviderArtifact {
            role,
            name: name.to_string(),
            path: path.to_string(),
            content_digest_blake3: content_digest_blake3.to_string(),
            source_id: "rust-src".to_string(),
            build_receipt_id: "build-receipt".to_string(),
        }
    }

    fn observed_rust_provider_artifacts(metadata: &RustSourceProviderMetadata) -> Vec<RustProviderObservedArtifact> {
        metadata
            .artifacts
            .iter()
            .map(|artifact| RustProviderObservedArtifact {
                role: artifact.role,
                path: artifact.path.clone(),
                content_digest_blake3: artifact.content_digest_blake3.clone(),
            })
            .collect()
    }

    fn observed_rust_provider_receipts(metadata: &RustSourceProviderMetadata) -> Vec<RustProviderObservedBuildReceipt> {
        metadata
            .build_receipts
            .iter()
            .map(|receipt| RustProviderObservedBuildReceipt {
                path: receipt.path.clone(),
                content_digest_blake3: receipt.digest_blake3.clone(),
                receipt: valid_rust_provider_receipt(metadata, &receipt.id),
            })
            .collect()
    }

    fn valid_rust_provider_receipt(
        metadata: &RustSourceProviderMetadata,
        receipt_id: &str,
    ) -> RustSourceProviderBuildReceipt {
        RustSourceProviderBuildReceipt {
            schema: RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA.to_string(),
            receipt_id: receipt_id.to_string(),
            provider_id: metadata.provider_id.clone(),
            host_triple: metadata.host_triple.clone(),
            target_triple: metadata.target_triple.clone(),
            source_ids: metadata.sources.iter().map(|source| source.id.clone()).collect(),
            output_artifacts: metadata
                .artifacts
                .iter()
                .filter(|artifact| {
                    artifact.build_receipt_id == receipt_id && artifact.role != RustProviderRole::ProviderReceipt
                })
                .map(|artifact| RustProviderReceiptArtifact {
                    role: artifact.role,
                    name: artifact.name.clone(),
                    path: artifact.path.clone(),
                    content_digest_blake3: artifact.content_digest_blake3.clone(),
                })
                .collect(),
            build_steps: vec![RustProviderReceiptStep {
                name: "compile-rust-from-source".to_string(),
                program: "mantle-rust-source-stage".to_string(),
                arguments: vec!["bootstrap/rust-source.ncl".to_string()],
            }],
        }
    }
}
