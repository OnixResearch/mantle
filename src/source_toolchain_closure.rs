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
const ENFORCED_SOURCE_BUILT_STATUS: &str = "enforced-source-built";
const PROVIDED_CLOSURE_STATUS: &str = "provided";
const POLICY_DIGEST_CONTEXT: &str = "mantle-source-built-toolchain-policy-digest-v1";
const MAX_TOOLCHAIN_MEMBERS: u32 = 128;
const MAX_SEED_EXCEPTIONS: u32 = 32;
const BLAKE3_HEX_CHAR_COUNT: usize = 64;
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
const RUST_PROVIDER_BOOTSTRAP_PLAN_SCHEMA: &str = "mantle-rust-source-provider-bootstrap-plan-v1";
const RUST_PROVIDER_BOOTSTRAP_PLAN_DIGEST_CONTEXT: &str = "mantle-rust-source-provider-bootstrap-plan-digest-v1";
const MAX_RUST_PROVIDER_ARTIFACTS: u32 = 256;
const MAX_RUST_PROVIDER_SOURCES: u32 = 64;
const MAX_RUST_PROVIDER_RECEIPTS: u32 = 64;
const MAX_RUST_PROVIDER_RECEIPT_STEPS: u32 = 128;
const MAX_RUST_PROVIDER_RECEIPT_ARGUMENTS: u32 = 64;
const MAX_RUST_PROVIDER_BOOTSTRAP_PLAN_SOURCES: u32 = 16;
const MAX_RUST_PROVIDER_BOOTSTRAP_PLAN_STAGES: u32 = 16;
const MAX_RUST_PROVIDER_BOOTSTRAP_PLAN_OUTPUTS: u32 = 16;
const MAX_RUST_PROVIDER_BOOTSTRAP_STAGE_NOTES: u32 = 16;
const SHA256_HEX_CHAR_COUNT: usize = 64;
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
pub(crate) const NATIVE_RUSTC_NAME: &str = "rustc";
pub(crate) const NATIVE_HOST_CC_NAME: &str = "cc";
pub(crate) const NATIVE_HOST_LINKER_NAME: &str = "ld";
pub(crate) const NATIVE_HOST_SYSROOT_NAME: &str = "host-sysroot";
pub(crate) const NATIVE_HOST_CRT1_NAME: &str = "host-crt1.o";
pub(crate) const NATIVE_HOST_LIBGCC_NAME: &str = "host-libgcc_s.so.1";
pub(crate) const NATIVE_HOST_LIBUNWIND_NAME: &str = "host-libunwind.a";
pub(crate) const NATIVE_HOST_LIBC_NAME: &str = "host-libc.so";
pub(crate) const NATIVE_TARGET_GCC_NAME: &str = "x86_64-linux-musl-gcc";
pub(crate) const NATIVE_TARGET_GXX_NAME: &str = "x86_64-linux-musl-g++";
pub(crate) const NATIVE_TARGET_LD_NAME: &str = "x86_64-linux-musl-ld";
pub(crate) const NATIVE_TARGET_AR_NAME: &str = "x86_64-linux-musl-ar";
pub(crate) const NATIVE_TARGET_RANLIB_NAME: &str = "x86_64-linux-musl-ranlib";
pub(crate) const NATIVE_TARGET_CRT1_NAME: &str = "x86_64-linux-musl-crt1.o";
pub(crate) const NATIVE_TARGET_LIBGCC_NAME: &str = "x86_64-linux-musl-libgcc_s.so.1";
pub(crate) const NATIVE_TARGET_LIBUNWIND_NAME: &str = "x86_64-linux-musl-libunwind.a";
pub(crate) const NATIVE_TARGET_LIBC_NAME: &str = "x86_64-linux-musl-libc.so";
pub(crate) const SOURCE_BUILT_C_COMPILER_ROUTE_ENV: &str = "MANTLE_SOURCE_BUILT_C_COMPILER_ROUTE";
const C_COMPILER_FAMILY_CLANG: &str = "clang";
const C_COMPILER_FAMILY_GCC: &str = "gcc";
const C_COMPILER_FAMILY_UNKNOWN: &str = "unknown";
const REQUIRED_NATIVE_CLOSURE_MEMBERS: &[&str] = &[
    NATIVE_RUSTC_NAME,
    NATIVE_HOST_CC_NAME,
    NATIVE_HOST_LINKER_NAME,
    NATIVE_HOST_SYSROOT_NAME,
    NATIVE_HOST_CRT1_NAME,
    NATIVE_HOST_LIBGCC_NAME,
    NATIVE_HOST_LIBUNWIND_NAME,
    NATIVE_HOST_LIBC_NAME,
    NATIVE_TARGET_GCC_NAME,
    NATIVE_TARGET_GXX_NAME,
    NATIVE_TARGET_LD_NAME,
    NATIVE_TARGET_AR_NAME,
    NATIVE_TARGET_RANLIB_NAME,
    NATIVE_TARGET_CRT1_NAME,
    NATIVE_TARGET_LIBGCC_NAME,
    NATIVE_TARGET_LIBUNWIND_NAME,
    NATIVE_TARGET_LIBC_NAME,
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct SourceBuiltToolchainClosureStatus {
    pub(crate) schema: &'static str,
    pub(crate) status: &'static str,
    pub(crate) claim: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) non_claim: Option<&'static str>,
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
    #[serde(rename = "c-compiler")]
    Ccompiler,
    CxxCompiler,
    PkgConfig,
    Sysroot,
    CrtObject,
    RuntimeLibrary,
    NativeHelper,
}

impl ToolchainRole {
    // Compatibility name retained because sibling root-package modules still pattern-match this role.
    #[allow(non_upper_case_globals)]
    pub(crate) const CCompiler: Self = Self::Ccompiler;

    fn diagnostic_name(self) -> &'static str {
        match self {
            Self::Rustc => "Rustc",
            Self::Linker => "Linker",
            Self::Ccompiler => "CCompiler",
            Self::CxxCompiler => "CxxCompiler",
            Self::PkgConfig => "PkgConfig",
            Self::Sysroot => "Sysroot",
            Self::CrtObject => "CrtObject",
            Self::RuntimeLibrary => "RuntimeLibrary",
            Self::NativeHelper => "NativeHelper",
        }
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustSourceProviderBootstrapPlan {
    pub(crate) schema: String,
    pub(crate) provider_id: String,
    pub(crate) route: String,
    pub(crate) host_triple: String,
    pub(crate) target_triple: String,
    pub(crate) final_version: String,
    pub(crate) policy: RustSourceProviderBootstrapPolicy,
    pub(crate) sources: Vec<RustSourceProviderBootstrapSource>,
    pub(crate) stages: Vec<RustSourceProviderBootstrapStage>,
    pub(crate) final_outputs: Vec<RustSourceProviderBootstrapOutput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustSourceProviderBootstrapPolicy {
    pub(crate) source_built: bool,
    pub(crate) uses_prebuilt_rust: bool,
    pub(crate) forbids_prebuilt_rust: bool,
    pub(crate) reference: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustSourceProviderBootstrapSource {
    pub(crate) id: String,
    pub(crate) kind: ToolchainSourceKind,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) url: String,
    pub(crate) sha256_hex: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustSourceProviderBootstrapStage {
    pub(crate) id: String,
    pub(crate) kind: RustSourceProviderBootstrapStageKind,
    pub(crate) source_ids: Vec<String>,
    pub(crate) bootstrap_stage_id: String,
    pub(crate) rust_version: String,
    pub(crate) outputs: Vec<RustProviderRole>,
    pub(crate) notes: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RustSourceProviderBootstrapStageKind {
    MrustcSeed,
    RustcStage1,
    RustcFinal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustSourceProviderBootstrapOutput {
    pub(crate) role: RustProviderRole,
    pub(crate) path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct RustSourceProviderBootstrapPlanValidation {
    pub(crate) policy_digest_blake3: String,
    pub(crate) source_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) final_output_count: usize,
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
pub(crate) struct NativeClosureCandidateMember {
    pub(crate) role: ToolchainRole,
    pub(crate) name: String,
    pub(crate) execution_path: String,
    pub(crate) content_digest_blake3: String,
    pub(crate) source: ToolchainSourceIdentity,
    pub(crate) build_receipt: ToolchainBuildReceiptIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ReceiptBoundCcompilerRoute {
    pub(crate) role: ToolchainRole,
    pub(crate) name: String,
    pub(crate) execution_path: String,
    pub(crate) content_digest_blake3: String,
    pub(crate) source: ToolchainSourceIdentity,
    pub(crate) build_receipt: ToolchainBuildReceiptIdentity,
    pub(crate) compiler_family: String,
}

pub(crate) use ReceiptBoundCcompilerRoute as ReceiptBoundCCompilerRoute;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeClosureMaterialization {
    pub(crate) manifest: ToolchainClosureManifest,
    pub(crate) validation: ToolchainClosureValidation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeClosureMaterializationError {
    missing_members: Vec<String>,
    invalid_manifest: Option<ToolchainClosureError>,
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
    AmbiguousCcompilerRoute,
    PlaceholderSeedException,
    HostToolLeakage,
    InvalidRustProvider,
    PrebuiltRustProvider,
    MissingRustProviderRole,
    DuplicateRustProviderItem,
    DigestMismatch,
    Serialization,
}

impl ToolchainClosureErrorKind {
    // Compatibility name retained for existing error classification checks in sibling modules.
    #[allow(non_upper_case_globals)]
    pub(crate) const AmbiguousCCompilerRoute: Self = Self::AmbiguousCcompilerRoute;
}

pub(crate) fn absent_source_built_toolchain_closure() -> SourceBuiltToolchainClosureStatus {
    SourceBuiltToolchainClosureStatus {
        schema: SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA,
        status: ABSENT_CLOSURE_STATUS,
        claim: false,
        non_claim: Some(SOURCE_BUILT_NON_CLAIM),
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
    source_built_toolchain_closure_status(manifest_path, validation, VALIDATED_NOT_ENFORCED_STATUS, false)
}

pub(crate) fn enforced_source_built_toolchain_closure(
    manifest_path: PathBuf,
    validation: &ToolchainClosureValidation,
) -> SourceBuiltToolchainClosureStatus {
    let (status, claim) = enforced_closure_claim(validation);
    source_built_toolchain_closure_status(manifest_path, validation, status, claim)
}

pub(crate) fn provided_source_built_rust_provider_closure(
    metadata_path: PathBuf,
    validation: &RustSourceProviderValidation,
) -> SourceBuiltToolchainClosureStatus {
    SourceBuiltToolchainClosureStatus {
        schema: SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA,
        status: PROVIDED_CLOSURE_STATUS,
        claim: true,
        non_claim: None,
        manifest_path: Some(metadata_path),
        policy_digest_blake3: Some(validation.policy_digest_blake3.clone()),
        member_count: Some(validation.artifact_count),
        source_built_member_count: Some(validation.artifact_count),
        seed_exception_count: Some(0),
    }
}

fn source_built_toolchain_closure_status(
    manifest_path: PathBuf,
    validation: &ToolchainClosureValidation,
    status: &'static str,
    claim: bool,
) -> SourceBuiltToolchainClosureStatus {
    SourceBuiltToolchainClosureStatus {
        schema: SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA,
        status,
        claim,
        non_claim: non_claim_for_source_built_status(claim),
        manifest_path: Some(manifest_path),
        policy_digest_blake3: Some(validation.policy_digest_blake3.clone()),
        member_count: Some(validation.member_count),
        source_built_member_count: Some(validation.source_built_member_count),
        seed_exception_count: Some(validation.seed_exception_count),
    }
}

fn enforced_closure_claim(validation: &ToolchainClosureValidation) -> (&'static str, bool) {
    if validation.seed_exception_count != 0 {
        return (VALIDATED_ENFORCED_STATUS, false);
    }
    if validation.source_built_member_count != validation.member_count {
        return (VALIDATED_ENFORCED_STATUS, false);
    }
    (ENFORCED_SOURCE_BUILT_STATUS, true)
}

fn non_claim_for_source_built_status(claim: bool) -> Option<&'static str> {
    if claim {
        return None;
    }
    Some(SOURCE_BUILT_NON_CLAIM)
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

pub(crate) fn select_receipt_bound_c_compiler_route(
    manifest: &ToolchainClosureManifest,
) -> Result<ReceiptBoundCCompilerRoute, ToolchainClosureError> {
    let normalized = normalize_manifest(manifest)?;
    let routes = normalized
        .members
        .iter()
        .filter(|member| member.role == ToolchainRole::CCompiler)
        .map(receipt_bound_c_compiler_route_from_member)
        .collect::<Result<Vec<_>, _>>()?;
    select_c_compiler_route_from_routes(routes)
}

pub(crate) fn validate_receipt_bound_c_compiler_route(
    route: &ReceiptBoundCCompilerRoute,
) -> Result<(), ToolchainClosureError> {
    if route.role != ToolchainRole::CCompiler {
        return Err(error(
            ToolchainClosureErrorKind::MissingRequiredRole,
            format!("receipt-bound C compiler route role must be CCompiler, got {:?}", route.role),
        ));
    }
    validate_non_empty("C compiler route name", &route.name)?;
    validate_absolute_path("C compiler route execution_path", &route.execution_path)?;
    validate_blake3_hex("C compiler route content_digest_blake3", &route.content_digest_blake3)?;
    validate_source_identity(&route.source)?;
    validate_receipt_identity(&route.build_receipt)?;
    validate_non_empty("C compiler route compiler_family", &route.compiler_family)
}

pub(crate) fn materialize_source_built_native_closure(
    candidates: &[NativeClosureCandidateMember],
) -> Result<NativeClosureMaterialization, NativeClosureMaterializationError> {
    assert!(!REQUIRED_NATIVE_CLOSURE_MEMBERS.is_empty());
    let missing_members = missing_native_closure_members(candidates);
    if !missing_members.is_empty() {
        return Err(NativeClosureMaterializationError::missing(missing_members));
    }
    let manifest = ToolchainClosureManifest {
        schema: SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA.to_string(),
        members: candidates.iter().map(native_candidate_to_member).collect(),
        seed_exceptions: Vec::new(),
    };
    let validation =
        validate_toolchain_closure_manifest(&manifest).map_err(NativeClosureMaterializationError::invalid)?;
    debug_assert_eq!(validation.seed_exception_count, 0);
    debug_assert_eq!(validation.source_built_member_count, validation.member_count);
    Ok(NativeClosureMaterialization { manifest, validation })
}

pub(crate) fn required_native_closure_member_names() -> &'static [&'static str] {
    REQUIRED_NATIVE_CLOSURE_MEMBERS
}

pub(crate) fn validate_rust_source_provider_metadata(
    metadata: &RustSourceProviderMetadata,
) -> Result<RustSourceProviderValidation, ToolchainClosureError> {
    let normalized = normalize_rust_source_provider_metadata(metadata)?;
    validation_from_normalized_rust_provider(&normalized)
}

pub(crate) fn validate_rust_source_provider_bootstrap_plan(
    plan: &RustSourceProviderBootstrapPlan,
) -> Result<RustSourceProviderBootstrapPlanValidation, ToolchainClosureError> {
    let normalized = normalize_rust_source_provider_bootstrap_plan(plan)?;
    validation_from_normalized_rust_provider_bootstrap_plan(&normalized)
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

impl NativeClosureMaterializationError {
    fn missing(missing_members: Vec<String>) -> Self {
        debug_assert!(!missing_members.is_empty());
        Self {
            missing_members,
            invalid_manifest: None,
        }
    }

    fn invalid(error: ToolchainClosureError) -> Self {
        Self {
            missing_members: Vec::new(),
            invalid_manifest: Some(error),
        }
    }

    pub(crate) fn message(&self) -> String {
        if !self.missing_members.is_empty() {
            return format!("missing native closure members: {}", self.missing_members.join(", "));
        }
        if let Some(error) = &self.invalid_manifest {
            return format!("invalid native closure manifest: {}", error.message());
        }
        "invalid native closure materialization".to_string()
    }

    pub(crate) fn missing_members(&self) -> &[String] {
        &self.missing_members
    }
}

impl fmt::Display for NativeClosureMaterializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message())
    }
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

fn missing_native_closure_members(candidates: &[NativeClosureCandidateMember]) -> Vec<String> {
    let present = candidates.iter().map(|candidate| candidate.name.as_str()).collect::<BTreeSet<_>>();
    REQUIRED_NATIVE_CLOSURE_MEMBERS
        .iter()
        .copied()
        .filter(|required| !present.contains(required))
        .map(ToOwned::to_owned)
        .collect()
}

fn receipt_bound_c_compiler_route_from_member(
    member: &ToolchainClosureMember,
) -> Result<ReceiptBoundCCompilerRoute, ToolchainClosureError> {
    debug_assert_eq!(member.role, ToolchainRole::CCompiler);
    let source = member.source.clone().ok_or_else(|| {
        error(
            ToolchainClosureErrorKind::MissingSource,
            format!("source-built C compiler member '{}' lacks source", member.name),
        )
    })?;
    let build_receipt = member.build_receipt.clone().ok_or_else(|| {
        error(
            ToolchainClosureErrorKind::MissingBuildReceipt,
            format!("source-built C compiler member '{}' lacks build receipt", member.name),
        )
    })?;
    Ok(ReceiptBoundCCompilerRoute {
        role: member.role,
        name: member.name.clone(),
        execution_path: member.execution_path.clone(),
        content_digest_blake3: member.content_digest_blake3.clone(),
        source,
        build_receipt,
        compiler_family: c_compiler_family(member).to_string(),
    })
}

fn select_c_compiler_route_from_routes(
    routes: Vec<ReceiptBoundCCompilerRoute>,
) -> Result<ReceiptBoundCCompilerRoute, ToolchainClosureError> {
    match routes.as_slice() {
        [] => Err(error(ToolchainClosureErrorKind::MissingRequiredRole, "missing C compiler route")),
        [route] => Ok(route.clone()),
        _many => select_single_clang_c_compiler_route(&routes),
    }
}

fn select_single_clang_c_compiler_route(
    routes: &[ReceiptBoundCCompilerRoute],
) -> Result<ReceiptBoundCCompilerRoute, ToolchainClosureError> {
    let clang_routes = routes
        .iter()
        .filter(|route| route.compiler_family == C_COMPILER_FAMILY_CLANG)
        .cloned()
        .collect::<Vec<_>>();
    match clang_routes.as_slice() {
        [route] => Ok(route.clone()),
        [] => Err(ambiguous_c_compiler_route_error(routes, "no receipt-bound clang route was available")),
        _many => Err(ambiguous_c_compiler_route_error(routes, "multiple receipt-bound clang routes were available")),
    }
}

fn ambiguous_c_compiler_route_error(routes: &[ReceiptBoundCCompilerRoute], reason: &str) -> ToolchainClosureError {
    let route_names = routes.iter().map(|route| route.name.as_str()).collect::<Vec<_>>().join(", ");
    error(
        ToolchainClosureErrorKind::AmbiguousCCompilerRoute,
        format!("ambiguous C compiler route selection: {reason}; candidates: {route_names}"),
    )
}

fn c_compiler_family(member: &ToolchainClosureMember) -> &'static str {
    let lower_name = member.name.to_ascii_lowercase();
    let lower_path = member.execution_path.to_ascii_lowercase();
    if lower_name.contains(C_COMPILER_FAMILY_CLANG) || lower_path.contains(C_COMPILER_FAMILY_CLANG) {
        return C_COMPILER_FAMILY_CLANG;
    }
    if lower_name.contains(C_COMPILER_FAMILY_GCC) || lower_path.contains(C_COMPILER_FAMILY_GCC) {
        return C_COMPILER_FAMILY_GCC;
    }
    C_COMPILER_FAMILY_UNKNOWN
}

fn native_candidate_to_member(candidate: &NativeClosureCandidateMember) -> ToolchainClosureMember {
    ToolchainClosureMember {
        role: candidate.role,
        name: candidate.name.clone(),
        execution_path: candidate.execution_path.clone(),
        content_digest_blake3: candidate.content_digest_blake3.clone(),
        trust: ToolchainTrust::SourceBuilt,
        source: Some(candidate.source.clone()),
        build_receipt: Some(candidate.build_receipt.clone()),
    }
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
            format!(
                "host-tool-leakage: {} uses undeclared path {}",
                input.role.diagnostic_name(),
                input.execution_path
            ),
        ));
    };
    debug_assert_eq!(member.role, input.role);
    debug_assert_eq!(member.execution_path, input.execution_path);
    if let Some(digest) = &input.content_digest_blake3
        && member.content_digest_blake3 != *digest
    {
        return Err(error(
            ToolchainClosureErrorKind::HostToolLeakage,
            format!("host-tool-leakage: {} digest mismatch for {}", input.role.diagnostic_name(), input.execution_path),
        ));
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
    let member_count = checked_collection_len("toolchain closure members", manifest.members.len())?;
    let seed_count = checked_collection_len("toolchain closure seed exceptions", manifest.seed_exceptions.len())?;
    if member_count > MAX_TOOLCHAIN_MEMBERS {
        return Err(error(ToolchainClosureErrorKind::TooManyMembers, "too many toolchain closure members"));
    }
    if seed_count > MAX_SEED_EXCEPTIONS {
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

fn normalize_rust_source_provider_bootstrap_plan(
    plan: &RustSourceProviderBootstrapPlan,
) -> Result<RustSourceProviderBootstrapPlan, ToolchainClosureError> {
    validate_rust_provider_bootstrap_plan_schema(&plan.schema)?;
    validate_rust_provider_bootstrap_plan_identity(plan)?;
    validate_rust_provider_bootstrap_plan_policy(&plan.policy)?;
    let source_ids = validate_rust_provider_bootstrap_sources(&plan.sources)?;
    validate_rust_provider_bootstrap_stages(plan, &source_ids)?;
    validate_rust_provider_bootstrap_outputs(plan)?;
    let mut normalized = plan.clone();
    normalized.sources.sort_by_key(rust_provider_bootstrap_source_sort_key);
    normalized.final_outputs.sort_by_key(rust_provider_bootstrap_output_sort_key);
    Ok(normalized)
}

fn validation_from_normalized_rust_provider_bootstrap_plan(
    plan: &RustSourceProviderBootstrapPlan,
) -> Result<RustSourceProviderBootstrapPlanValidation, ToolchainClosureError> {
    Ok(RustSourceProviderBootstrapPlanValidation {
        policy_digest_blake3: digest_normalized_rust_provider_bootstrap_plan(plan)?,
        source_count: plan.sources.len(),
        stage_count: plan.stages.len(),
        final_output_count: plan.final_outputs.len(),
    })
}

fn validate_rust_provider_bootstrap_plan_schema(schema: &str) -> Result<(), ToolchainClosureError> {
    if schema == RUST_PROVIDER_BOOTSTRAP_PLAN_SCHEMA {
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::InvalidRustProvider,
        format!("expected schema {RUST_PROVIDER_BOOTSTRAP_PLAN_SCHEMA}, got {schema}"),
    ))
}

fn validate_rust_provider_bootstrap_plan_identity(
    plan: &RustSourceProviderBootstrapPlan,
) -> Result<(), ToolchainClosureError> {
    require_equal("rust provider bootstrap provider_id", &plan.provider_id, RUST_SOURCE_PROVIDER_ID)?;
    validate_non_empty("rust provider bootstrap route", &plan.route)?;
    validate_no_disallowed_rust_provider_text("rust provider bootstrap route", &plan.route)?;
    validate_non_empty("rust provider bootstrap host_triple", &plan.host_triple)?;
    validate_non_empty("rust provider bootstrap target_triple", &plan.target_triple)?;
    validate_non_empty("rust provider bootstrap final_version", &plan.final_version)
}

fn validate_rust_provider_bootstrap_plan_policy(
    policy: &RustSourceProviderBootstrapPolicy,
) -> Result<(), ToolchainClosureError> {
    validate_non_empty("rust provider bootstrap reference", &policy.reference)?;
    debug_assert!(!policy.reference.trim().is_empty());
    if !policy.source_built {
        return Err(error(
            ToolchainClosureErrorKind::PrebuiltRustProvider,
            "rust provider bootstrap plan is not source-built",
        ));
    }
    if policy.uses_prebuilt_rust {
        return Err(error(
            ToolchainClosureErrorKind::PrebuiltRustProvider,
            "rust provider bootstrap plan uses prebuilt Rust",
        ));
    }
    if policy.forbids_prebuilt_rust {
        debug_assert!(policy.source_built);
        debug_assert!(!policy.uses_prebuilt_rust);
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::PrebuiltRustProvider,
        "rust provider bootstrap plan does not forbid prebuilt Rust",
    ))
}

fn validate_rust_provider_bootstrap_sources(
    sources: &[RustSourceProviderBootstrapSource],
) -> Result<BTreeSet<String>, ToolchainClosureError> {
    validate_collection_count(
        "rust provider bootstrap sources",
        sources.len(),
        MAX_RUST_PROVIDER_BOOTSTRAP_PLAN_SOURCES,
    )?;
    let mut ids = BTreeSet::new();
    for source in sources {
        validate_rust_provider_bootstrap_source(source)?;
        if !ids.insert(source.id.clone()) {
            return Err(error(
                ToolchainClosureErrorKind::DuplicateRustProviderItem,
                format!("duplicate rust provider bootstrap source id '{}'", source.id),
            ));
        }
    }
    Ok(ids)
}

fn validate_rust_provider_bootstrap_source(
    source: &RustSourceProviderBootstrapSource,
) -> Result<(), ToolchainClosureError> {
    validate_non_empty("rust provider bootstrap source id", &source.id)?;
    validate_non_empty("rust provider bootstrap source name", &source.name)?;
    validate_non_empty("rust provider bootstrap source version", &source.version)?;
    validate_non_empty("rust provider bootstrap source url", &source.url)?;
    validate_no_disallowed_rust_provider_text("rust provider bootstrap source id", &source.id)?;
    validate_no_disallowed_rust_provider_text("rust provider bootstrap source name", &source.name)?;
    if !matches!(source.kind, ToolchainSourceKind::Tarball) {
        return Err(error(
            ToolchainClosureErrorKind::InvalidRustProvider,
            format!("rust provider bootstrap source '{}' must be a tarball", source.id),
        ));
    }
    validate_sha256_hex("rust provider bootstrap source sha256_hex", &source.sha256_hex)
}

fn validate_rust_provider_bootstrap_stages(
    plan: &RustSourceProviderBootstrapPlan,
    source_ids: &BTreeSet<String>,
) -> Result<(), ToolchainClosureError> {
    validate_collection_count(
        "rust provider bootstrap stages",
        plan.stages.len(),
        MAX_RUST_PROVIDER_BOOTSTRAP_PLAN_STAGES,
    )?;
    debug_assert!(!plan.stages.is_empty());
    debug_assert!(u32::try_from(plan.stages.len()).is_ok_and(|count| count <= MAX_RUST_PROVIDER_BOOTSTRAP_PLAN_STAGES));
    let mut seen_stage_ids = BTreeSet::new();
    let mut final_stage_count = 0usize;
    for stage in &plan.stages {
        validate_rust_provider_bootstrap_stage(plan, source_ids, &seen_stage_ids, stage)?;
        if matches!(stage.kind, RustSourceProviderBootstrapStageKind::RustcFinal) {
            final_stage_count = final_stage_count.checked_add(1).ok_or_else(|| {
                error(ToolchainClosureErrorKind::InvalidRustProvider, "rust provider final stage count overflowed")
            })?;
        }
        if !seen_stage_ids.insert(stage.id.clone()) {
            return Err(error(
                ToolchainClosureErrorKind::DuplicateRustProviderItem,
                format!("duplicate rust provider bootstrap stage id '{}'", stage.id),
            ));
        }
    }
    if final_stage_count == 1 {
        debug_assert_eq!(seen_stage_ids.len(), plan.stages.len());
        debug_assert_eq!(final_stage_count, 1);
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::InvalidRustProvider,
        format!("rust provider bootstrap plan has {final_stage_count} final stages"),
    ))
}

fn validate_rust_provider_bootstrap_stage(
    plan: &RustSourceProviderBootstrapPlan,
    source_ids: &BTreeSet<String>,
    seen_stage_ids: &BTreeSet<String>,
    stage: &RustSourceProviderBootstrapStage,
) -> Result<(), ToolchainClosureError> {
    validate_non_empty("rust provider bootstrap stage id", &stage.id)?;
    validate_non_empty("rust provider bootstrap stage rust_version", &stage.rust_version)?;
    validate_no_disallowed_rust_provider_text("rust provider bootstrap stage id", &stage.id)?;
    validate_rust_provider_bootstrap_stage_sources(source_ids, stage)?;
    validate_rust_provider_bootstrap_stage_link(plan, seen_stage_ids, stage)?;
    validate_rust_provider_bootstrap_stage_outputs(stage)?;
    validate_rust_provider_bootstrap_stage_notes(stage)
}

fn validate_rust_provider_bootstrap_stage_sources(
    source_ids: &BTreeSet<String>,
    stage: &RustSourceProviderBootstrapStage,
) -> Result<(), ToolchainClosureError> {
    validate_collection_count(
        "rust provider bootstrap stage source_ids",
        stage.source_ids.len(),
        MAX_RUST_PROVIDER_BOOTSTRAP_PLAN_SOURCES,
    )?;
    debug_assert!(!stage.source_ids.is_empty());
    debug_assert!(
        u32::try_from(stage.source_ids.len()).is_ok_and(|count| count <= MAX_RUST_PROVIDER_BOOTSTRAP_PLAN_SOURCES)
    );
    let mut stage_source_ids = BTreeSet::new();
    for source_id in &stage.source_ids {
        validate_non_empty("rust provider bootstrap stage source_id", source_id)?;
        validate_no_disallowed_rust_provider_text("rust provider bootstrap stage source_id", source_id)?;
        if !source_ids.contains(source_id) {
            return Err(error(
                ToolchainClosureErrorKind::InvalidRustProvider,
                format!("rust provider bootstrap stage '{}' references unknown source_id '{source_id}'", stage.id),
            ));
        }
        if !stage_source_ids.insert(source_id.clone()) {
            return Err(error(
                ToolchainClosureErrorKind::DuplicateRustProviderItem,
                format!("duplicate rust provider bootstrap stage source_id '{source_id}'"),
            ));
        }
    }
    Ok(())
}

fn validate_rust_provider_bootstrap_stage_link(
    plan: &RustSourceProviderBootstrapPlan,
    seen_stage_ids: &BTreeSet<String>,
    stage: &RustSourceProviderBootstrapStage,
) -> Result<(), ToolchainClosureError> {
    match stage.kind {
        RustSourceProviderBootstrapStageKind::MrustcSeed => {
            if stage.bootstrap_stage_id.is_empty() {
                return Ok(());
            }
            Err(error(
                ToolchainClosureErrorKind::InvalidRustProvider,
                format!("mrustc bootstrap stage '{}' must not name a predecessor", stage.id),
            ))
        }
        RustSourceProviderBootstrapStageKind::RustcStage1 => require_prior_bootstrap_stage(seen_stage_ids, stage),
        RustSourceProviderBootstrapStageKind::RustcFinal => {
            require_equal("rust provider final stage version", &stage.rust_version, &plan.final_version)?;
            require_prior_bootstrap_stage(seen_stage_ids, stage)
        }
    }
}

fn require_prior_bootstrap_stage(
    seen_stage_ids: &BTreeSet<String>,
    stage: &RustSourceProviderBootstrapStage,
) -> Result<(), ToolchainClosureError> {
    validate_non_empty("rust provider bootstrap_stage_id", &stage.bootstrap_stage_id)?;
    if seen_stage_ids.contains(&stage.bootstrap_stage_id) {
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::InvalidRustProvider,
        format!(
            "rust provider bootstrap stage '{}' references unknown predecessor '{}'",
            stage.id, stage.bootstrap_stage_id
        ),
    ))
}

fn validate_rust_provider_bootstrap_stage_outputs(
    stage: &RustSourceProviderBootstrapStage,
) -> Result<(), ToolchainClosureError> {
    validate_collection_count(
        "rust provider bootstrap stage outputs",
        stage.outputs.len(),
        MAX_RUST_PROVIDER_BOOTSTRAP_PLAN_OUTPUTS,
    )?;
    let mut roles = BTreeSet::new();
    for role in &stage.outputs {
        if !roles.insert(*role) {
            return Err(error(
                ToolchainClosureErrorKind::DuplicateRustProviderItem,
                format!("duplicate rust provider bootstrap stage output role {role:?}"),
            ));
        }
    }
    Ok(())
}

fn validate_rust_provider_bootstrap_stage_notes(
    stage: &RustSourceProviderBootstrapStage,
) -> Result<(), ToolchainClosureError> {
    validate_max_collection_count(
        "rust provider bootstrap stage notes",
        stage.notes.len(),
        MAX_RUST_PROVIDER_BOOTSTRAP_STAGE_NOTES,
    )?;
    for note in &stage.notes {
        validate_no_disallowed_rust_provider_text("rust provider bootstrap stage note", note)?;
    }
    Ok(())
}

fn validate_rust_provider_bootstrap_outputs(
    plan: &RustSourceProviderBootstrapPlan,
) -> Result<(), ToolchainClosureError> {
    validate_collection_count(
        "rust provider bootstrap final outputs",
        plan.final_outputs.len(),
        MAX_RUST_PROVIDER_BOOTSTRAP_PLAN_OUTPUTS,
    )?;
    let mut roles = BTreeSet::new();
    for output in &plan.final_outputs {
        validate_provider_relative_path("rust provider bootstrap output path", &output.path)?;
        validate_rust_provider_bootstrap_role_path(plan, output)?;
        if !roles.insert(output.role) {
            return Err(error(
                ToolchainClosureErrorKind::DuplicateRustProviderItem,
                format!("duplicate rust provider bootstrap output role {:?}", output.role),
            ));
        }
    }
    require_rust_provider_roles(&roles)
}

fn validate_rust_provider_bootstrap_role_path(
    plan: &RustSourceProviderBootstrapPlan,
    output: &RustSourceProviderBootstrapOutput,
) -> Result<(), ToolchainClosureError> {
    match output.role {
        RustProviderRole::Rustc => require_provider_path(&output.path, RUSTC_PROVIDER_PATH, output.role),
        RustProviderRole::Cargo => require_provider_path(&output.path, CARGO_PROVIDER_PATH, output.role),
        RustProviderRole::Rustdoc => require_provider_path(&output.path, RUSTDOC_PROVIDER_PATH, output.role),
        RustProviderRole::HostRustlib => {
            require_provider_path_prefix(&output.path, rustlib_lib_prefix(&plan.host_triple), output.role)
        }
        RustProviderRole::TargetRustlib => {
            require_provider_path_prefix(&output.path, rustlib_lib_prefix(&plan.target_triple), output.role)
        }
        RustProviderRole::ProviderReceipt => validate_provider_receipt_path(&output.path),
    }
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

fn validate_collection_count(label: &str, len: usize, max: u32) -> Result<(), ToolchainClosureError> {
    let count = checked_collection_len(label, len)?;
    if count == 0 {
        return Err(error(ToolchainClosureErrorKind::InvalidRustProvider, format!("{label} is empty")));
    }
    validate_max_collection_count(label, len, max)
}

fn validate_max_collection_count(label: &str, len: usize, max: u32) -> Result<(), ToolchainClosureError> {
    let count = checked_collection_len(label, len)?;
    if count > max {
        return Err(error(
            ToolchainClosureErrorKind::InvalidRustProvider,
            format!("{label} has more than {max} entries"),
        ));
    }
    Ok(())
}

fn checked_collection_len(label: &str, len: usize) -> Result<u32, ToolchainClosureError> {
    u32::try_from(len).map_err(|_| {
        error(
            ToolchainClosureErrorKind::InvalidRustProvider,
            format!("{label} count exceeds the supported u32 range"),
        )
    })
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
    debug_assert!(!metadata.provider_id.is_empty());
    debug_assert!(!artifact.name.is_empty());
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
    debug_assert!(metadata.build_receipts.len() <= metadata.artifacts.len());
    debug_assert!(!metadata.provider_id.is_empty());
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
            require_provider_path_prefix(&artifact.path, rustlib_lib_prefix(&metadata.host_triple), artifact.role)
        }
        RustProviderRole::TargetRustlib => {
            require_provider_path_prefix(&artifact.path, rustlib_lib_prefix(&metadata.target_triple), artifact.role)
        }
        RustProviderRole::ProviderReceipt => validate_provider_receipt_path(&artifact.path),
    }
}

fn require_provider_path<P: AsRef<str>, E: AsRef<str>>(
    path: P,
    expected: E,
    role: RustProviderRole,
) -> Result<(), ToolchainClosureError> {
    let path = path.as_ref();
    let expected = expected.as_ref();
    if path == expected {
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::InvalidRustProvider,
        format!("rust provider role {role:?} must use path {expected}, got {path}"),
    ))
}

fn require_provider_path_prefix<P: AsRef<str>, E: AsRef<str>>(
    path: P,
    prefix: E,
    role: RustProviderRole,
) -> Result<(), ToolchainClosureError> {
    let path = path.as_ref();
    let prefix = prefix.as_ref();
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

fn validate_provider_relative_path<L: AsRef<str>, V: AsRef<str>>(
    label: L,
    value: V,
) -> Result<(), ToolchainClosureError> {
    let label = label.as_ref();
    let value = value.as_ref();
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
    debug_assert!(!metadata.sources.is_empty());
    debug_assert!(!receipt.source_ids.is_empty());
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
    debug_assert!(!receipt.output_artifacts.is_empty());
    let expected = metadata_artifacts_for_receipt(metadata, &receipt.receipt_id);
    debug_assert!(!expected.is_empty());
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
    debug_assert!(!receipt.build_steps.is_empty());
    debug_assert!(u32::try_from(receipt.build_steps.len()).is_ok_and(|count| count <= MAX_RUST_PROVIDER_RECEIPT_STEPS));
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

fn require_equal<L: AsRef<str>, A: AsRef<str>, E: AsRef<str>>(
    label: L,
    actual: A,
    expected: E,
) -> Result<(), ToolchainClosureError> {
    let label = label.as_ref();
    let actual = actual.as_ref();
    let expected = expected.as_ref();
    if actual == expected {
        return Ok(());
    }
    Err(error(
        ToolchainClosureErrorKind::InvalidRustProvider,
        format!("{label} expected {expected}, got {actual}"),
    ))
}

fn validate_non_empty<L: AsRef<str>, V: AsRef<str>>(label: L, value: V) -> Result<(), ToolchainClosureError> {
    let label = label.as_ref();
    let value = value.as_ref();
    if !value.trim().is_empty() {
        return Ok(());
    }
    Err(error(ToolchainClosureErrorKind::EmptyField, format!("{label} is empty")))
}

fn validate_seed_exception_text<L: AsRef<str>, V: AsRef<str>>(label: L, value: V) -> Result<(), ToolchainClosureError> {
    let label = label.as_ref();
    let normalized = value.as_ref().to_ascii_lowercase();
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

fn validate_no_disallowed_rust_provider_text<L: AsRef<str>, V: AsRef<str>>(
    label: L,
    value: V,
) -> Result<(), ToolchainClosureError> {
    let label = label.as_ref();
    if let Some(marker) = disallowed_rust_provider_marker(value.as_ref()) {
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

fn contains_disallowed_rust_provider_marker<V: AsRef<str>, M: AsRef<str>>(value: V, marker: M) -> bool {
    let value = value.as_ref();
    let marker = marker.as_ref();
    debug_assert!(!marker.is_empty());
    if marker.bytes().any(|byte| !byte.is_ascii_alphanumeric() && byte != b'_') {
        return value.contains(marker);
    }
    contains_disallowed_seed_exception_marker(value, marker)
}

fn contains_disallowed_seed_exception_marker<V: AsRef<str>, M: AsRef<str>>(value: V, marker: M) -> bool {
    let value = value.as_ref();
    let marker = marker.as_ref();
    debug_assert!(!marker.is_empty());
    let mut search_start = 0;
    while let Some(relative_start) = value[search_start..].find(marker) {
        let Some(marker_start) = search_start.checked_add(relative_start) else {
            return false;
        };
        let Some(marker_end) = marker_start.checked_add(marker.len()) else {
            return false;
        };
        if has_marker_boundaries(value, marker_start..marker_end) {
            return true;
        }
        search_start = marker_end;
    }
    false
}

fn has_marker_boundaries(value: &str, marker_range: std::ops::Range<usize>) -> bool {
    debug_assert!(marker_range.start <= marker_range.end);
    debug_assert!(marker_range.end <= value.len());
    let before = value[..marker_range.start].chars().next_back();
    let after = value[marker_range.end..].chars().next();
    !is_seed_marker_word_char(before) && !is_seed_marker_word_char(after)
}

fn is_seed_marker_word_char(value: Option<char>) -> bool {
    value.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn validate_absolute_path<L: AsRef<str>, V: AsRef<str>>(label: L, value: V) -> Result<(), ToolchainClosureError> {
    let label = label.as_ref();
    let value = value.as_ref();
    validate_non_empty(label, value)?;
    if Path::new(value).is_absolute() {
        return Ok(());
    }
    Err(error(ToolchainClosureErrorKind::InvalidExecutionPath, format!("{label} must be absolute: {value}")))
}

fn validate_blake3_hex<L: AsRef<str>, V: AsRef<str>>(label: L, value: V) -> Result<(), ToolchainClosureError> {
    let label = label.as_ref();
    let value = value.as_ref();
    if value.len() != BLAKE3_HEX_CHAR_COUNT {
        return Err(error(ToolchainClosureErrorKind::InvalidDigest, format!("{label} has invalid length")));
    }
    validate_lowercase_hex(label, value)
}

fn validate_sha256_hex<L: AsRef<str>, V: AsRef<str>>(label: L, value: V) -> Result<(), ToolchainClosureError> {
    let label = label.as_ref();
    let value = value.as_ref();
    if value.len() != SHA256_HEX_CHAR_COUNT {
        return Err(error(ToolchainClosureErrorKind::InvalidDigest, format!("{label} has invalid length")));
    }
    validate_lowercase_hex(label, value)
}

fn validate_lowercase_hex<L: AsRef<str>, V: AsRef<str>>(label: L, value: V) -> Result<(), ToolchainClosureError> {
    let label = label.as_ref();
    let value = value.as_ref();
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

fn digest_normalized_rust_provider_bootstrap_plan(
    plan: &RustSourceProviderBootstrapPlan,
) -> Result<String, ToolchainClosureError> {
    let payload = serde_json::json!({
        "context": RUST_PROVIDER_BOOTSTRAP_PLAN_DIGEST_CONTEXT,
        "plan": plan,
    });
    let bytes = serde_json::to_vec(&payload).map_err(|err| {
        error(ToolchainClosureErrorKind::Serialization, format!("serialize rust provider bootstrap plan: {err}"))
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

fn rust_provider_bootstrap_source_sort_key(source: &RustSourceProviderBootstrapSource) -> (String, String) {
    (source.id.clone(), source.sha256_hex.clone())
}

fn rust_provider_bootstrap_output_sort_key(output: &RustSourceProviderBootstrapOutput) -> (RustProviderRole, String) {
    (output.role, output.path.clone())
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
    const EXPECTED_BOOTSTRAP_PLAN_SOURCE_COUNT: usize = 6;
    const EXPECTED_BOOTSTRAP_PLAN_STAGE_COUNT: usize = 5;
    const EXPECTED_BOOTSTRAP_PLAN_FINAL_OUTPUT_COUNT: usize = 6;
    const EXPECTED_PROVIDER_STATUS_ARTIFACT_COUNT: usize = 6;
    const EXPECTED_PROVIDER_STATUS_SOURCE_COUNT: usize = 6;
    const EXPECTED_PROVIDER_STATUS_RECEIPT_COUNT: usize = 1;
    const MRUSTC_SOURCE_SHA256_HEX: &str = "c1ba35f5fc5c4ca2952d9f5526e900dcb6632ea7fd4d71fa58029b3bb563ae56";
    const FINAL_RUST_VERSION: &str = "1.94.1";

    #[test]
    fn absent_closure_status_preserves_current_non_claim() {
        let status = absent_source_built_toolchain_closure();

        assert_eq!(status.schema, SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA);
        assert_eq!(status.status, ABSENT_CLOSURE_STATUS);
        assert!(!status.claim);
        assert_eq!(status.non_claim, Some(SOURCE_BUILT_NON_CLAIM));
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
        assert_eq!(status.non_claim, Some(SOURCE_BUILT_NON_CLAIM));
        assert_eq!(status.manifest_path, Some(PathBuf::from("/tmp/toolchain.json")));
        assert_eq!(status.policy_digest_blake3, Some(validation.policy_digest_blake3));
        assert_eq!(status.member_count, Some(REQUIRED_TOOLCHAIN_ROLES.len()));
    }

    #[test]
    fn enforced_complete_closure_status_promotes_source_built_claim() {
        let manifest = valid_manifest();
        let validation = validate_toolchain_closure_manifest(&manifest).unwrap();

        let status = enforced_source_built_toolchain_closure(PathBuf::from("/tmp/toolchain.json"), &validation);

        assert_eq!(status.schema, SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA);
        assert_eq!(status.status, ENFORCED_SOURCE_BUILT_STATUS);
        assert!(status.claim);
        assert_eq!(status.non_claim, None);
        assert_eq!(status.policy_digest_blake3, Some(validation.policy_digest_blake3));
        assert_eq!(status.seed_exception_count, Some(0));
    }

    #[test]
    fn enforced_seed_exception_closure_status_keeps_non_claim() {
        let mut manifest = valid_manifest();
        make_seed_member(&mut manifest, 0);
        let validation = validate_toolchain_closure_manifest(&manifest).unwrap();

        let status = enforced_source_built_toolchain_closure(PathBuf::from("/tmp/toolchain.json"), &validation);

        assert_eq!(status.schema, SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA);
        assert_eq!(status.status, VALIDATED_ENFORCED_STATUS);
        assert!(!status.claim);
        assert_eq!(status.non_claim, Some(SOURCE_BUILT_NON_CLAIM));
        assert_eq!(status.source_built_member_count, Some(REQUIRED_TOOLCHAIN_ROLES.len() - 1));
        assert_eq!(status.seed_exception_count, Some(1));
    }

    #[test]
    fn provided_rust_provider_status_promotes_source_built_claim() {
        let validation = RustSourceProviderValidation {
            policy_digest_blake3: DIGEST_A.to_string(),
            artifact_count: EXPECTED_PROVIDER_STATUS_ARTIFACT_COUNT,
            source_count: EXPECTED_PROVIDER_STATUS_SOURCE_COUNT,
            receipt_count: EXPECTED_PROVIDER_STATUS_RECEIPT_COUNT,
        };

        let status = provided_source_built_rust_provider_closure(PathBuf::from("/tmp/provider.json"), &validation);

        assert_eq!(status.schema, SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA);
        assert_eq!(status.status, PROVIDED_CLOSURE_STATUS);
        assert!(status.claim);
        assert_eq!(status.non_claim, None);
        assert_eq!(status.manifest_path, Some(PathBuf::from("/tmp/provider.json")));
        assert_eq!(status.policy_digest_blake3, Some(DIGEST_A.to_string()));
        assert_eq!(status.member_count, Some(EXPECTED_PROVIDER_STATUS_ARTIFACT_COUNT));
        assert_eq!(status.source_built_member_count, Some(EXPECTED_PROVIDER_STATUS_ARTIFACT_COUNT));
        assert_eq!(status.seed_exception_count, Some(0));
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
        assert_eq!(metadata.host_triple, "x86_64-unknown-linux-gnu");
        assert_eq!(metadata.target_triple, "x86_64-unknown-linux-musl");
        assert!(metadata.artifacts.iter().any(|artifact| {
            artifact.role == RustProviderRole::HostRustlib
                && artifact.path == "lib/rustlib/x86_64-unknown-linux-gnu/lib"
        }));
        assert!(metadata.artifacts.iter().any(|artifact| {
            artifact.role == RustProviderRole::TargetRustlib
                && artifact.path == "lib/rustlib/x86_64-unknown-linux-musl/lib"
        }));
        assert_eq!(validation.policy_digest_blake3, validation_reordered.policy_digest_blake3);
    }

    #[test]
    fn rust_source_provider_rejects_host_rustlib_satisfied_by_target_path() {
        let mut metadata = valid_rust_provider_metadata();
        let host_rustlib = metadata
            .artifacts
            .iter_mut()
            .find(|artifact| artifact.role == RustProviderRole::HostRustlib)
            .unwrap();
        host_rustlib.path = "lib/rustlib/x86_64-unknown-linux-musl/lib".to_string();

        let err = validate_rust_source_provider_metadata(&metadata).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::InvalidRustProvider);
        assert!(err.message().contains("HostRustlib"));
        assert!(err.message().contains("x86_64-unknown-linux-gnu"));
    }

    #[test]
    fn rust_source_provider_rejects_target_rustlib_satisfied_by_host_path() {
        let mut metadata = valid_rust_provider_metadata();
        let target_rustlib = metadata
            .artifacts
            .iter_mut()
            .find(|artifact| artifact.role == RustProviderRole::TargetRustlib)
            .unwrap();
        target_rustlib.path = "lib/rustlib/x86_64-unknown-linux-gnu/lib".to_string();

        let err = validate_rust_source_provider_metadata(&metadata).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::InvalidRustProvider);
        assert!(err.message().contains("TargetRustlib"));
        assert!(err.message().contains("x86_64-unknown-linux-musl"));
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

    #[test]
    fn valid_rust_source_provider_bootstrap_plan_yields_stable_digest() {
        let plan = valid_rust_provider_bootstrap_plan();
        let mut reordered = plan.clone();
        reordered.sources.reverse();
        reordered.final_outputs.reverse();

        let validation = validate_rust_source_provider_bootstrap_plan(&plan).unwrap();
        let validation_reordered = validate_rust_source_provider_bootstrap_plan(&reordered).unwrap();

        assert_eq!(validation.source_count, EXPECTED_BOOTSTRAP_PLAN_SOURCE_COUNT);
        assert_eq!(validation.stage_count, EXPECTED_BOOTSTRAP_PLAN_STAGE_COUNT);
        assert_eq!(validation.final_output_count, EXPECTED_BOOTSTRAP_PLAN_FINAL_OUTPUT_COUNT);
        assert_eq!(plan.host_triple, "x86_64-unknown-linux-gnu");
        assert_eq!(plan.target_triple, "x86_64-unknown-linux-musl");
        assert!(plan.final_outputs.iter().any(|output| {
            output.role == RustProviderRole::HostRustlib && output.path == "lib/rustlib/x86_64-unknown-linux-gnu/lib"
        }));
        assert!(plan.final_outputs.iter().any(|output| {
            output.role == RustProviderRole::TargetRustlib && output.path == "lib/rustlib/x86_64-unknown-linux-musl/lib"
        }));
        assert_eq!(validation.policy_digest_blake3, validation_reordered.policy_digest_blake3);
    }

    #[test]
    fn rust_source_provider_bootstrap_plan_rejects_host_target_rustlib_aliasing() {
        let mut host_alias_plan = valid_rust_provider_bootstrap_plan();
        let host_output = host_alias_plan
            .final_outputs
            .iter_mut()
            .find(|output| output.role == RustProviderRole::HostRustlib)
            .unwrap();
        host_output.path = "lib/rustlib/x86_64-unknown-linux-musl/lib".to_string();

        let host_err = validate_rust_source_provider_bootstrap_plan(&host_alias_plan).unwrap_err();

        assert_eq!(host_err.kind(), ToolchainClosureErrorKind::InvalidRustProvider);
        assert!(host_err.message().contains("HostRustlib"));
        assert!(host_err.message().contains("x86_64-unknown-linux-gnu"));

        let mut target_alias_plan = valid_rust_provider_bootstrap_plan();
        let target_output = target_alias_plan
            .final_outputs
            .iter_mut()
            .find(|output| output.role == RustProviderRole::TargetRustlib)
            .unwrap();
        target_output.path = "lib/rustlib/x86_64-unknown-linux-gnu/lib".to_string();

        let target_err = validate_rust_source_provider_bootstrap_plan(&target_alias_plan).unwrap_err();

        assert_eq!(target_err.kind(), ToolchainClosureErrorKind::InvalidRustProvider);
        assert!(target_err.message().contains("TargetRustlib"));
        assert!(target_err.message().contains("x86_64-unknown-linux-musl"));
    }

    #[test]
    fn checked_in_rust_source_plan_loads_stagex_mrustc_route() {
        let plan_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap/rust-source-plan.ncl");
        let import_paths: Vec<std::ffi::OsString> = Vec::new();
        let plan: RustSourceProviderBootstrapPlan = crunch_eval::evaluate_and_deserialize(&plan_path, &import_paths)
            .unwrap_or_else(|err| panic!("loading {}: {err}", plan_path.display()));

        let validation = validate_rust_source_provider_bootstrap_plan(&plan).unwrap();

        assert_eq!(plan.final_version, FINAL_RUST_VERSION);
        assert_eq!(plan.host_triple, "x86_64-unknown-linux-gnu");
        assert_eq!(plan.target_triple, "x86_64-unknown-linux-musl");
        assert!(plan.final_outputs.iter().any(|output| {
            output.role == RustProviderRole::HostRustlib && output.path == "lib/rustlib/x86_64-unknown-linux-gnu/lib"
        }));
        assert!(plan.final_outputs.iter().any(|output| {
            output.role == RustProviderRole::TargetRustlib && output.path == "lib/rustlib/x86_64-unknown-linux-musl/lib"
        }));
        assert_eq!(validation.source_count, EXPECTED_BOOTSTRAP_PLAN_SOURCE_COUNT);
        assert_eq!(validation.stage_count, EXPECTED_BOOTSTRAP_PLAN_STAGE_COUNT);
        assert!(
            plan.sources
                .iter()
                .any(|source| { source.id == "mrustc-0.12.0" && source.sha256_hex == MRUSTC_SOURCE_SHA256_HEX })
        );
        assert!(plan.stages.iter().any(|stage| {
            stage.kind == RustSourceProviderBootstrapStageKind::MrustcSeed
                && stage.source_ids.iter().any(|source_id| source_id == "mrustc-0.12.0")
        }));
        assert!(plan.stages.iter().any(|stage| {
            stage.kind == RustSourceProviderBootstrapStageKind::RustcFinal
                && stage.bootstrap_stage_id == "rust-1.93.1-stage1"
        }));
    }

    #[test]
    fn checked_in_musl_host_rust_source_plan_loads_stagex_mrustc_route() {
        let plan_path =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap/rust-source-musl-host-plan.ncl");
        let import_paths: Vec<std::ffi::OsString> = Vec::new();
        let plan: RustSourceProviderBootstrapPlan = crunch_eval::evaluate_and_deserialize(&plan_path, &import_paths)
            .unwrap_or_else(|err| panic!("loading {}: {err}", plan_path.display()));

        let validation = validate_rust_source_provider_bootstrap_plan(&plan).unwrap();

        assert_eq!(plan.final_version, FINAL_RUST_VERSION);
        assert_eq!(plan.host_triple, "x86_64-unknown-linux-musl");
        assert_eq!(plan.target_triple, "x86_64-unknown-linux-musl");
        assert!(plan.policy.source_built);
        assert!(!plan.policy.uses_prebuilt_rust);
        assert!(plan.policy.forbids_prebuilt_rust);
        assert!(plan.final_outputs.iter().any(|output| {
            output.role == RustProviderRole::HostRustlib && output.path == "lib/rustlib/x86_64-unknown-linux-musl/lib"
        }));
        assert!(plan.final_outputs.iter().any(|output| {
            output.role == RustProviderRole::TargetRustlib && output.path == "lib/rustlib/x86_64-unknown-linux-musl/lib"
        }));
        assert_eq!(validation.source_count, EXPECTED_BOOTSTRAP_PLAN_SOURCE_COUNT);
        assert_eq!(validation.stage_count, EXPECTED_BOOTSTRAP_PLAN_STAGE_COUNT);
    }

    #[test]
    fn rust_source_provider_bootstrap_plan_rejects_prebuilt_policy() {
        let mut plan = valid_rust_provider_bootstrap_plan();
        plan.policy.uses_prebuilt_rust = true;

        let err = validate_rust_source_provider_bootstrap_plan(&plan).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::PrebuiltRustProvider);
        assert!(err.message().contains("uses prebuilt Rust"));
    }

    #[test]
    fn rust_source_provider_bootstrap_plan_requires_final_roles() {
        let mut plan = valid_rust_provider_bootstrap_plan();
        plan.final_outputs.retain(|output| output.role != RustProviderRole::TargetRustlib);

        let err = validate_rust_source_provider_bootstrap_plan(&plan).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::MissingRustProviderRole);
        assert!(err.message().contains("TargetRustlib"));
    }

    #[test]
    fn rust_source_provider_bootstrap_plan_requires_ordered_stage_predecessors() {
        let mut plan = valid_rust_provider_bootstrap_plan();
        plan.stages[1].bootstrap_stage_id = "future-stage".to_string();

        let err = validate_rust_source_provider_bootstrap_plan(&plan).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::InvalidRustProvider);
        assert!(err.message().contains("unknown predecessor"));
    }

    #[test]
    fn rust_source_provider_bootstrap_plan_rejects_unknown_stage_source() {
        let mut plan = valid_rust_provider_bootstrap_plan();
        plan.stages[0].source_ids[0] = "missing-source".to_string();

        let err = validate_rust_source_provider_bootstrap_plan(&plan).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::InvalidRustProvider);
        assert!(err.message().contains("unknown source_id"));
    }

    #[test]
    fn native_materialization_builds_zero_seed_manifest_from_complete_candidates() {
        let candidates = native_closure_candidates();

        let materialized = materialize_source_built_native_closure(&candidates).unwrap();

        assert_eq!(materialized.manifest.schema, SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA);
        assert_eq!(materialized.manifest.members.len(), required_native_closure_member_names().len());
        assert!(materialized.manifest.seed_exceptions.is_empty());
        assert_eq!(materialized.validation.seed_exception_count, 0);
        assert_eq!(materialized.validation.source_built_member_count, materialized.validation.member_count);
        assert!(materialized.manifest.members.iter().all(|member| member.trust == ToolchainTrust::SourceBuilt));
    }

    #[test]
    fn native_materialization_rejects_missing_host_runtime_members() {
        let mut candidates = native_closure_candidates();
        candidates.retain(|candidate| candidate.name != NATIVE_HOST_LIBC_NAME);

        let err = materialize_source_built_native_closure(&candidates).unwrap_err();

        assert_eq!(err.missing_members(), &[NATIVE_HOST_LIBC_NAME.to_string()]);
        assert!(err.message().contains("missing native closure members"));
        assert!(err.message().contains(NATIVE_HOST_LIBC_NAME));
    }

    #[test]
    fn native_materialization_rejects_missing_target_helper_members() {
        let mut candidates = native_closure_candidates();
        candidates.retain(|candidate| candidate.name != NATIVE_TARGET_RANLIB_NAME);

        let err = materialize_source_built_native_closure(&candidates).unwrap_err();

        assert_eq!(err.missing_members(), &[NATIVE_TARGET_RANLIB_NAME.to_string()]);
        assert!(err.message().contains("missing native closure members"));
        assert!(err.message().contains(NATIVE_TARGET_RANLIB_NAME));
    }

    #[test]
    fn c_compiler_route_selection_records_receipt_bound_identity() {
        let mut manifest = valid_manifest();
        manifest.members[2].execution_path = "/toolchain/bin/x86_64-linux-musl-gcc".to_string();

        let route = select_receipt_bound_c_compiler_route(&manifest).unwrap();

        assert_eq!(route.role, ToolchainRole::CCompiler);
        assert_eq!(route.name, "cc");
        assert_eq!(route.execution_path, "/toolchain/bin/x86_64-linux-musl-gcc");
        assert_eq!(route.content_digest_blake3, DIGEST_C);
        assert_eq!(route.source.name, "cc-source");
        assert_eq!(route.build_receipt.name, "cc-receipt");
        assert_eq!(route.compiler_family, C_COMPILER_FAMILY_GCC);
    }

    #[test]
    fn c_compiler_route_selection_prefers_single_clang_route() {
        let mut manifest = valid_manifest();
        manifest.members[2].execution_path = "/toolchain/bin/x86_64-linux-musl-gcc".to_string();
        manifest.members.push(member(ToolchainRole::CCompiler, "clang", DIGEST_E, DIGEST_F));
        let last = manifest.members.last_mut().unwrap();
        last.execution_path = "/toolchain/bin/clang".to_string();

        let route = select_receipt_bound_c_compiler_route(&manifest).unwrap();

        assert_eq!(route.name, "clang");
        assert_eq!(route.execution_path, "/toolchain/bin/clang");
        assert_eq!(route.compiler_family, C_COMPILER_FAMILY_CLANG);
    }

    #[test]
    fn c_compiler_route_selection_rejects_ambiguous_non_clang_routes() {
        let mut manifest = valid_manifest();
        manifest.members[2].execution_path = "/toolchain/bin/x86_64-linux-musl-gcc".to_string();
        manifest.members.push(member(ToolchainRole::CCompiler, "alt-gcc", DIGEST_E, DIGEST_F));
        let last = manifest.members.last_mut().unwrap();
        last.execution_path = "/toolchain/bin/alt-gcc".to_string();

        let err = select_receipt_bound_c_compiler_route(&manifest).unwrap_err();

        assert_eq!(err.kind(), ToolchainClosureErrorKind::AmbiguousCCompilerRoute);
        assert!(err.message().contains("no receipt-bound clang route"));
        assert!(err.message().contains("alt-gcc"));
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

    fn native_closure_candidates() -> Vec<NativeClosureCandidateMember> {
        vec![
            native_candidate(ToolchainRole::Rustc, NATIVE_RUSTC_NAME, DIGEST_A),
            native_candidate(ToolchainRole::CCompiler, NATIVE_HOST_CC_NAME, DIGEST_B),
            native_candidate(ToolchainRole::Linker, NATIVE_HOST_LINKER_NAME, DIGEST_C),
            native_candidate(ToolchainRole::Sysroot, NATIVE_HOST_SYSROOT_NAME, DIGEST_D),
            native_candidate(ToolchainRole::CrtObject, NATIVE_HOST_CRT1_NAME, DIGEST_E),
            native_candidate(ToolchainRole::RuntimeLibrary, NATIVE_HOST_LIBGCC_NAME, DIGEST_F),
            native_candidate(ToolchainRole::RuntimeLibrary, NATIVE_HOST_LIBUNWIND_NAME, DIGEST_B),
            native_candidate(ToolchainRole::RuntimeLibrary, NATIVE_HOST_LIBC_NAME, DIGEST_A),
            native_candidate(ToolchainRole::NativeHelper, NATIVE_TARGET_GCC_NAME, DIGEST_B),
            native_candidate(ToolchainRole::NativeHelper, NATIVE_TARGET_GXX_NAME, DIGEST_C),
            native_candidate(ToolchainRole::NativeHelper, NATIVE_TARGET_LD_NAME, DIGEST_D),
            native_candidate(ToolchainRole::NativeHelper, NATIVE_TARGET_AR_NAME, DIGEST_E),
            native_candidate(ToolchainRole::NativeHelper, NATIVE_TARGET_RANLIB_NAME, DIGEST_F),
            native_candidate(ToolchainRole::CrtObject, NATIVE_TARGET_CRT1_NAME, DIGEST_A),
            native_candidate(ToolchainRole::RuntimeLibrary, NATIVE_TARGET_LIBGCC_NAME, DIGEST_B),
            native_candidate(ToolchainRole::RuntimeLibrary, NATIVE_TARGET_LIBUNWIND_NAME, DIGEST_D),
            native_candidate(ToolchainRole::RuntimeLibrary, NATIVE_TARGET_LIBC_NAME, DIGEST_C),
        ]
    }

    fn native_candidate(role: ToolchainRole, name: &str, content_digest_blake3: &str) -> NativeClosureCandidateMember {
        NativeClosureCandidateMember {
            role,
            name: name.to_string(),
            execution_path: format!("/native-closure/{name}"),
            content_digest_blake3: content_digest_blake3.to_string(),
            source: ToolchainSourceIdentity {
                kind: ToolchainSourceKind::LocalTree,
                name: "mantle-source-built-native-closure".to_string(),
                digest_blake3: DIGEST_D.to_string(),
            },
            build_receipt: ToolchainBuildReceiptIdentity {
                kind: ToolchainBuildReceiptKind::ExternalAttestedBuild,
                name: "mantle-source-built-native-closure-receipt".to_string(),
                digest_blake3: DIGEST_E.to_string(),
            },
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

    fn valid_rust_provider_bootstrap_plan() -> RustSourceProviderBootstrapPlan {
        RustSourceProviderBootstrapPlan {
            schema: RUST_PROVIDER_BOOTSTRAP_PLAN_SCHEMA.to_string(),
            provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
            route: "mrustc-to-rust-current-source-route".to_string(),
            host_triple: "x86_64-unknown-linux-gnu".to_string(),
            target_triple: "x86_64-unknown-linux-musl".to_string(),
            final_version: FINAL_RUST_VERSION.to_string(),
            policy: RustSourceProviderBootstrapPolicy {
                source_built: true,
                uses_prebuilt_rust: false,
                forbids_prebuilt_rust: true,
                reference: "stagex core rust package route".to_string(),
            },
            sources: vec![
                rust_provider_bootstrap_source("mrustc-0.12.0", "0.12.0", MRUSTC_SOURCE_SHA256_HEX),
                rust_provider_bootstrap_source(
                    "rust-1.90.0",
                    "1.90.0",
                    "799a9f9cba4ed5351e071048bcf6b5560755d9009648def33a407dd4961f9b7e",
                ),
                rust_provider_bootstrap_source(
                    "rust-1.91.1",
                    "1.91.1",
                    "38dce205d39f61571261f0444237a1ce9efecb970e760d8ec4d957af5b445723",
                ),
                rust_provider_bootstrap_source(
                    "rust-1.92.0",
                    "1.92.0",
                    "9e0d2ca75c7e275fdc758255bf4b03afb3d65d1543602746907c933b6901c3b8",
                ),
                rust_provider_bootstrap_source(
                    "rust-1.93.1",
                    "1.93.1",
                    "4c230a44b3d9c9f3cef950943719f8380058d27c91fda5e36a9a947ef013e01f",
                ),
                rust_provider_bootstrap_source(
                    "rust-1.94.1",
                    FINAL_RUST_VERSION,
                    "4c142a625f12e3cdf716c68ae19f4f60d98ad1482627b08579b15838e95ad514",
                ),
            ],
            stages: vec![
                rust_provider_bootstrap_stage(
                    "mrustc-to-rust-1.90.0",
                    RustSourceProviderBootstrapStageKind::MrustcSeed,
                    &["mrustc-0.12.0", "rust-1.90.0"],
                    "",
                    "1.90.0",
                ),
                rust_provider_bootstrap_stage(
                    "rust-1.91.1-stage1",
                    RustSourceProviderBootstrapStageKind::RustcStage1,
                    &["rust-1.91.1"],
                    "mrustc-to-rust-1.90.0",
                    "1.91.1",
                ),
                rust_provider_bootstrap_stage(
                    "rust-1.92.0-stage1",
                    RustSourceProviderBootstrapStageKind::RustcStage1,
                    &["rust-1.92.0"],
                    "rust-1.91.1-stage1",
                    "1.92.0",
                ),
                rust_provider_bootstrap_stage(
                    "rust-1.93.1-stage1",
                    RustSourceProviderBootstrapStageKind::RustcStage1,
                    &["rust-1.93.1"],
                    "rust-1.92.0-stage1",
                    "1.93.1",
                ),
                rust_provider_bootstrap_stage(
                    "rust-1.94.1-final",
                    RustSourceProviderBootstrapStageKind::RustcFinal,
                    &["rust-1.94.1"],
                    "rust-1.93.1-stage1",
                    FINAL_RUST_VERSION,
                ),
            ],
            final_outputs: vec![
                rust_provider_bootstrap_output(RustProviderRole::Rustc, RUSTC_PROVIDER_PATH),
                rust_provider_bootstrap_output(RustProviderRole::Cargo, CARGO_PROVIDER_PATH),
                rust_provider_bootstrap_output(RustProviderRole::Rustdoc, RUSTDOC_PROVIDER_PATH),
                rust_provider_bootstrap_output(
                    RustProviderRole::HostRustlib,
                    "lib/rustlib/x86_64-unknown-linux-gnu/lib",
                ),
                rust_provider_bootstrap_output(
                    RustProviderRole::TargetRustlib,
                    "lib/rustlib/x86_64-unknown-linux-musl/lib",
                ),
                rust_provider_bootstrap_output(
                    RustProviderRole::ProviderReceipt,
                    "share/mantle-rust-provider/receipts/build.json",
                ),
            ],
        }
    }

    fn rust_provider_bootstrap_source(id: &str, version: &str, sha256_hex: &str) -> RustSourceProviderBootstrapSource {
        let url = if id == "mrustc-0.12.0" {
            "https://github.com/thepowersgang/mrustc/archive/refs/tags/v0.12.0.tar.gz".to_string()
        } else {
            format!("https://static.rust-lang.org/dist/rustc-{version}-src.tar.gz")
        };
        RustSourceProviderBootstrapSource {
            id: id.to_string(),
            kind: ToolchainSourceKind::Tarball,
            name: "rust-compiler-source".to_string(),
            version: version.to_string(),
            url,
            sha256_hex: sha256_hex.to_string(),
        }
    }

    fn rust_provider_bootstrap_stage(
        id: &str,
        kind: RustSourceProviderBootstrapStageKind,
        source_ids: &[&str],
        bootstrap_stage_id: &str,
        rust_version: &str,
    ) -> RustSourceProviderBootstrapStage {
        let outputs = match kind {
            RustSourceProviderBootstrapStageKind::RustcFinal => vec![
                RustProviderRole::Rustc,
                RustProviderRole::Cargo,
                RustProviderRole::Rustdoc,
                RustProviderRole::HostRustlib,
                RustProviderRole::TargetRustlib,
                RustProviderRole::ProviderReceipt,
            ],
            RustSourceProviderBootstrapStageKind::MrustcSeed | RustSourceProviderBootstrapStageKind::RustcStage1 => {
                vec![
                    RustProviderRole::Rustc,
                    RustProviderRole::Cargo,
                    RustProviderRole::HostRustlib,
                    RustProviderRole::TargetRustlib,
                ]
            }
        };
        RustSourceProviderBootstrapStage {
            id: id.to_string(),
            kind,
            source_ids: source_ids.iter().map(|source_id| (*source_id).to_string()).collect(),
            bootstrap_stage_id: bootstrap_stage_id.to_string(),
            rust_version: rust_version.to_string(),
            outputs,
            notes: vec!["compile source stage".to_string()],
        }
    }

    fn rust_provider_bootstrap_output(role: RustProviderRole, path: &str) -> RustSourceProviderBootstrapOutput {
        RustSourceProviderBootstrapOutput {
            role,
            path: path.to_string(),
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
