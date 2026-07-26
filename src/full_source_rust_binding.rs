//! Pure binding contracts for full-source Rust and native providers.
//!
//! The imperative shell reads admission reports, hashes files, and publishes
//! receipts. This module only validates supplied facts and computes canonical
//! BLAKE3 identities.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::path::Component;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::source_toolchain_closure::RustProviderArtifact;
use crate::source_toolchain_closure::RustProviderBuildReceiptIdentity;
use crate::source_toolchain_closure::RustProviderObservedBuildReceipt;
use crate::source_toolchain_closure::RustProviderRole;
use crate::source_toolchain_closure::RustSourceProviderMetadata;
use crate::source_toolchain_closure::ToolchainBuildReceiptKind;
use crate::source_toolchain_closure::ToolchainClosureManifest;
use crate::source_toolchain_closure::enforce_observed_rust_source_provider_receipts;
use crate::source_toolchain_closure::validate_rust_source_provider_metadata;
use crate::source_toolchain_closure::validate_toolchain_closure_manifest;

pub(crate) const FULL_SOURCE_RUST_BINDING_SCHEMA: &str = "mantle-full-source-rust-provider-binding-v1";
pub(crate) const FULL_SOURCE_CLOSURE_BINDING_SCHEMA: &str = "mantle-full-source-toolchain-closure-binding-v1";
const FULL_SOURCE_ADMISSION_SCHEMA: &str = "mantle-full-source-provider-admission-v2";
const FULL_SOURCE_ADMISSION_STATUS: &str = "admitted";
const FULL_SOURCE_PROVIDER_ID: &str = "full-source-v1";
const FULL_SOURCE_PROVIDER_TARGET: &str = "x86_64-linux-musl";
const FULL_SOURCE_COMPILER_TARGET: &str = "x86_64-unknown-linux-musl";
const FULL_SOURCE_RUST_TRIPLE: &str = "x86_64-unknown-linux-musl";
const FULL_SOURCE_POLICY: &str = "authenticated-offline-only";
const FULL_SOURCE_CLOSURE_DIGEST_CONTEXT: &str = "mantle-full-source-toolchain-closure-binding-digest-v1";
const BLAKE3_HEX_LENGTH: usize = 64;
const SOURCE_CLOSURE_RECORD_COUNT_MAX: u32 = 65_536;
const RUST_BINDING_SOURCE_COUNT_MAX: u32 = 64;
const RUST_BINDING_RECEIPT_COUNT_MAX: u32 = 64;
const RUST_BINDING_ARTIFACT_COUNT_MAX: u32 = 256;

const REQUIRED_NATIVE_ARTIFACTS: &[(&str, FullSourceNativeArtifactRole)] = &[
    ("bin/x86_64-linux-musl-gcc", FullSourceNativeArtifactRole::CCompiler),
    ("bin/x86_64-linux-musl-g++", FullSourceNativeArtifactRole::CxxCompiler),
    ("bin/x86_64-linux-musl-c++", FullSourceNativeArtifactRole::CxxCompiler),
    ("bin/x86_64-linux-musl-cpp", FullSourceNativeArtifactRole::Preprocessor),
    ("bin/x86_64-linux-musl-gcc-ar", FullSourceNativeArtifactRole::ArchiveTool),
    ("bin/x86_64-linux-musl-gcc-nm", FullSourceNativeArtifactRole::SymbolTool),
    ("bin/x86_64-linux-musl-gcc-ranlib", FullSourceNativeArtifactRole::Ranlib),
    ("bin/x86_64-linux-musl-ar", FullSourceNativeArtifactRole::ArchiveTool),
    ("bin/x86_64-linux-musl-as", FullSourceNativeArtifactRole::Assembler),
    ("bin/x86_64-linux-musl-ld", FullSourceNativeArtifactRole::Linker),
    ("bin/x86_64-linux-musl-nm", FullSourceNativeArtifactRole::SymbolTool),
    ("bin/x86_64-linux-musl-objcopy", FullSourceNativeArtifactRole::ObjectCopy),
    ("bin/x86_64-linux-musl-objdump", FullSourceNativeArtifactRole::ObjectDump),
    ("bin/x86_64-linux-musl-ranlib", FullSourceNativeArtifactRole::Ranlib),
    ("bin/x86_64-linux-musl-readelf", FullSourceNativeArtifactRole::ObjectFormat),
    ("bin/x86_64-linux-musl-size", FullSourceNativeArtifactRole::ObjectFormat),
    ("bin/x86_64-linux-musl-strings", FullSourceNativeArtifactRole::ObjectFormat),
    ("bin/x86_64-linux-musl-strip", FullSourceNativeArtifactRole::ObjectCopy),
    ("libexec/gcc/x86_64-unknown-linux-musl/10.5.0/cc1", FullSourceNativeArtifactRole::CompilerInternal),
    (
        "libexec/gcc/x86_64-unknown-linux-musl/10.5.0/cc1plus",
        FullSourceNativeArtifactRole::CompilerInternal,
    ),
    ("x86_64-linux-musl/lib/crt1.o", FullSourceNativeArtifactRole::CrtObject),
    ("x86_64-linux-musl/lib/crti.o", FullSourceNativeArtifactRole::CrtObject),
    ("x86_64-linux-musl/lib/crtn.o", FullSourceNativeArtifactRole::CrtObject),
    ("x86_64-linux-musl/lib/libc.a", FullSourceNativeArtifactRole::Libc),
    ("x86_64-linux-musl/lib/libc.so", FullSourceNativeArtifactRole::Libc),
    ("x86_64-linux-musl/lib/libgcc.a", FullSourceNativeArtifactRole::Libgcc),
    ("x86_64-linux-musl/lib/libgcc_eh.a", FullSourceNativeArtifactRole::Libgcc),
    ("x86_64-linux-musl/lib/libgcc_s.so.1", FullSourceNativeArtifactRole::Libgcc),
    ("x86_64-linux-musl/lib/libstdc++.a", FullSourceNativeArtifactRole::Libstdcxx),
    ("x86_64-linux-musl/lib/libstdc++.so.6.0.28", FullSourceNativeArtifactRole::Libstdcxx),
    ("x86_64-linux-musl/lib/ld-musl-x86_64.so.1", FullSourceNativeArtifactRole::DynamicLinker),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum FullSourceNativeArtifactRole {
    CCompiler,
    CxxCompiler,
    Preprocessor,
    CompilerInternal,
    Assembler,
    Linker,
    ArchiveTool,
    Ranlib,
    SymbolTool,
    ObjectCopy,
    ObjectDump,
    ObjectFormat,
    CrtObject,
    Libc,
    Libgcc,
    Libstdcxx,
    DynamicLinker,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FullSourceNativeArtifactBinding {
    pub(crate) role: FullSourceNativeArtifactRole,
    pub(crate) path: String,
    pub(crate) content_digest_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FullSourceNativeProviderAdmissionIdentity {
    pub(crate) schema: String,
    pub(crate) status: String,
    pub(crate) provider_id: String,
    pub(crate) provider_target: String,
    pub(crate) compiler_target: String,
    pub(crate) admission_report_digest_blake3: String,
    pub(crate) metadata_digest_blake3: String,
    pub(crate) output_digest_blake3: String,
    pub(crate) expected_output_digest_blake3: String,
    pub(crate) source_closure_manifest_blake3: String,
    pub(crate) expected_source_closure_manifest_blake3: String,
    pub(crate) source_closure_record_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FullSourceRustArtifactBinding {
    pub(crate) role: RustProviderRole,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) content_digest_blake3: String,
    pub(crate) source_id: String,
    pub(crate) build_receipt_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FullSourceRustReceiptBinding {
    pub(crate) id: String,
    pub(crate) kind: ToolchainBuildReceiptKind,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) digest_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FullSourceRustProviderBindingReceipt {
    pub(crate) schema: String,
    pub(crate) receipt_id: String,
    pub(crate) rust_provider_id: String,
    pub(crate) host_triple: String,
    pub(crate) target_triple: String,
    pub(crate) source_policy: String,
    pub(crate) ambient_tool_discovery: bool,
    pub(crate) rust_provider_policy_digest_blake3: String,
    pub(crate) native_provider: FullSourceNativeProviderAdmissionIdentity,
    pub(crate) native_artifacts: Vec<FullSourceNativeArtifactBinding>,
    pub(crate) rust_source_ids: Vec<String>,
    pub(crate) rust_build_receipts: Vec<FullSourceRustReceiptBinding>,
    pub(crate) rust_artifacts: Vec<FullSourceRustArtifactBinding>,
    pub(crate) fallback_events: Vec<String>,
    pub(crate) seed_exceptions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FullSourceToolchainClosureBinding {
    pub(crate) schema: String,
    pub(crate) native_provider_output_digest_blake3: String,
    pub(crate) native_provider_admission_report_digest_blake3: String,
    pub(crate) rust_provider_binding_receipt_digest_blake3: String,
    pub(crate) rust_provider_policy_digest_blake3: String,
    pub(crate) native_closure_policy_digest_blake3: String,
    pub(crate) seed_exceptions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct FullSourceRustProviderBindingValidation {
    pub(crate) binding_receipt_digest_blake3: String,
    pub(crate) rust_provider_policy_digest_blake3: String,
    pub(crate) native_artifact_count: usize,
    pub(crate) rust_source_count: usize,
    pub(crate) rust_receipt_count: usize,
    pub(crate) rust_artifact_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct FullSourceToolchainClosureBindingValidation {
    pub(crate) binding_digest_blake3: String,
    pub(crate) native_closure_policy_digest_blake3: String,
    pub(crate) rust_provider_binding_receipt_digest_blake3: String,
    pub(crate) member_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FullSourceRustBindingErrorKind {
    InvalidSchema,
    InvalidProviderIdentity,
    SourceMismatch,
    ReceiptMismatch,
    ArtifactMismatch,
    NativeProviderMismatch,
    RoleSubstitution,
    SeedException,
    AmbientDiscovery,
    DigestMismatch,
    InvalidPath,
    Serialization,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FullSourceRustBindingError {
    kind: FullSourceRustBindingErrorKind,
    message: String,
}

pub(crate) fn validate_full_source_rust_provider_binding(
    metadata: &RustSourceProviderMetadata,
    observed_receipts: &[RustProviderObservedBuildReceipt],
    binding: &FullSourceRustProviderBindingReceipt,
    observed_admission: &FullSourceNativeProviderAdmissionIdentity,
    observed_native_artifacts: &[FullSourceNativeArtifactBinding],
) -> Result<FullSourceRustProviderBindingValidation, FullSourceRustBindingError> {
    let provider = validate_generic_rust_provider(metadata, observed_receipts)?;
    validate_binding_header(metadata, binding, &provider.policy_digest_blake3)?;
    validate_native_admission(&binding.native_provider)?;
    validate_native_admission(observed_admission)?;
    require_equal("native provider admission", &binding.native_provider, observed_admission)?;
    validate_native_artifacts(&binding.native_artifacts)?;
    validate_native_artifacts(observed_native_artifacts)?;
    let bound_native_artifacts = normalize_native_artifacts(&binding.native_artifacts);
    let observed_native_artifacts = normalize_native_artifacts(observed_native_artifacts);
    require_equal("native artifact identities", &bound_native_artifacts, &observed_native_artifacts)?;
    validate_bound_rust_sources(metadata, &binding.rust_source_ids)?;
    validate_bound_rust_receipts(metadata, &binding.rust_build_receipts)?;
    validate_bound_rust_artifacts(metadata, &binding.rust_artifacts)?;
    validate_zero_exceptions(binding)?;
    let normalized = normalize_binding(binding);
    let binding_receipt_digest_blake3 = digest_binding(&normalized)?;
    Ok(FullSourceRustProviderBindingValidation {
        binding_receipt_digest_blake3,
        rust_provider_policy_digest_blake3: provider.policy_digest_blake3,
        native_artifact_count: binding.native_artifacts.len(),
        rust_source_count: binding.rust_source_ids.len(),
        rust_receipt_count: binding.rust_build_receipts.len(),
        rust_artifact_count: binding.rust_artifacts.len(),
    })
}

pub(crate) fn validate_full_source_toolchain_closure_binding(
    closure: &ToolchainClosureManifest,
    binding: &FullSourceToolchainClosureBinding,
    rust_validation: &FullSourceRustProviderBindingValidation,
    native_admission: &FullSourceNativeProviderAdmissionIdentity,
) -> Result<FullSourceToolchainClosureBindingValidation, FullSourceRustBindingError> {
    validate_closure_binding_header(binding)?;
    require_empty("closure seed_exceptions", &closure.seed_exceptions)?;
    require_empty("closure binding seed_exceptions", &binding.seed_exceptions)?;
    let closure_validation = validate_toolchain_closure_manifest(closure)
        .map_err(|error| binding_error(FullSourceRustBindingErrorKind::InvalidProviderIdentity, error.to_string()))?;
    require_digest_equal(
        "native provider output",
        &binding.native_provider_output_digest_blake3,
        &native_admission.output_digest_blake3,
    )?;
    require_digest_equal(
        "native provider admission report",
        &binding.native_provider_admission_report_digest_blake3,
        &native_admission.admission_report_digest_blake3,
    )?;
    require_digest_equal(
        "Rust provider binding receipt",
        &binding.rust_provider_binding_receipt_digest_blake3,
        &rust_validation.binding_receipt_digest_blake3,
    )?;
    require_digest_equal(
        "Rust provider policy",
        &binding.rust_provider_policy_digest_blake3,
        &rust_validation.rust_provider_policy_digest_blake3,
    )?;
    require_digest_equal(
        "native closure policy",
        &binding.native_closure_policy_digest_blake3,
        &closure_validation.policy_digest_blake3,
    )?;
    let binding_digest_blake3 = digest_closure_binding(binding)?;
    Ok(FullSourceToolchainClosureBindingValidation {
        binding_digest_blake3,
        native_closure_policy_digest_blake3: closure_validation.policy_digest_blake3,
        rust_provider_binding_receipt_digest_blake3: rust_validation.binding_receipt_digest_blake3.clone(),
        member_count: closure_validation.member_count,
    })
}

pub(crate) fn required_full_source_native_artifacts() -> &'static [(&'static str, FullSourceNativeArtifactRole)] {
    REQUIRED_NATIVE_ARTIFACTS
}

pub(crate) fn canonical_full_source_rust_binding_bytes(
    binding: &FullSourceRustProviderBindingReceipt,
) -> Result<Vec<u8>, FullSourceRustBindingError> {
    let normalized = normalize_binding(binding);
    serde_json::to_vec(&normalized).map_err(|error| {
        binding_error(
            FullSourceRustBindingErrorKind::Serialization,
            format!("serializing canonical full-source Rust binding: {error}"),
        )
    })
}

impl FullSourceRustBindingError {
    pub(crate) fn kind(&self) -> FullSourceRustBindingErrorKind {
        self.kind
    }
}

impl fmt::Display for FullSourceRustBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for FullSourceRustBindingError {}

fn validate_generic_rust_provider(
    metadata: &RustSourceProviderMetadata,
    observed_receipts: &[RustProviderObservedBuildReceipt],
) -> Result<crate::source_toolchain_closure::RustSourceProviderValidation, FullSourceRustBindingError> {
    validate_rust_source_provider_metadata(metadata)
        .map_err(|error| binding_error(FullSourceRustBindingErrorKind::InvalidProviderIdentity, error.to_string()))?;
    enforce_observed_rust_source_provider_receipts(metadata, observed_receipts)
        .map_err(|error| binding_error(FullSourceRustBindingErrorKind::ReceiptMismatch, error.to_string()))
}

fn validate_binding_header(
    metadata: &RustSourceProviderMetadata,
    binding: &FullSourceRustProviderBindingReceipt,
    policy_digest_blake3: &str,
) -> Result<(), FullSourceRustBindingError> {
    require_text("binding schema", &binding.schema, FULL_SOURCE_RUST_BINDING_SCHEMA)?;
    validate_nonempty("binding receipt_id", &binding.receipt_id)?;
    require_text("binding rust_provider_id", &binding.rust_provider_id, &metadata.provider_id)?;
    require_text("binding host_triple", &binding.host_triple, &metadata.host_triple)?;
    require_text("binding target_triple", &binding.target_triple, &metadata.target_triple)?;
    require_text("full-source host triple", &binding.host_triple, FULL_SOURCE_RUST_TRIPLE)?;
    require_text("full-source target triple", &binding.target_triple, FULL_SOURCE_RUST_TRIPLE)?;
    require_text("binding source_policy", &binding.source_policy, FULL_SOURCE_POLICY)?;
    if binding.ambient_tool_discovery {
        return Err(binding_error(
            FullSourceRustBindingErrorKind::AmbientDiscovery,
            "full-source Rust binding records ambient tool discovery",
        ));
    }
    require_digest_equal("Rust provider policy", &binding.rust_provider_policy_digest_blake3, policy_digest_blake3)
}

fn validate_native_admission(
    admission: &FullSourceNativeProviderAdmissionIdentity,
) -> Result<(), FullSourceRustBindingError> {
    require_text("native admission schema", &admission.schema, FULL_SOURCE_ADMISSION_SCHEMA)?;
    require_text("native admission status", &admission.status, FULL_SOURCE_ADMISSION_STATUS)?;
    require_text("native admission provider_id", &admission.provider_id, FULL_SOURCE_PROVIDER_ID)?;
    require_text("native admission provider_target", &admission.provider_target, FULL_SOURCE_PROVIDER_TARGET)?;
    require_text("native admission compiler_target", &admission.compiler_target, FULL_SOURCE_COMPILER_TARGET)?;
    validate_digest("native admission report digest", &admission.admission_report_digest_blake3)?;
    validate_digest("native metadata digest", &admission.metadata_digest_blake3)?;
    require_digest_equal("native output", &admission.output_digest_blake3, &admission.expected_output_digest_blake3)?;
    require_digest_equal(
        "native source closure",
        &admission.source_closure_manifest_blake3,
        &admission.expected_source_closure_manifest_blake3,
    )?;
    if admission.source_closure_record_count == 0
        || admission.source_closure_record_count > SOURCE_CLOSURE_RECORD_COUNT_MAX
    {
        return Err(binding_error(
            FullSourceRustBindingErrorKind::NativeProviderMismatch,
            "native admission source closure record count is out of bounds",
        ));
    }
    Ok(())
}

fn validate_native_artifacts(artifacts: &[FullSourceNativeArtifactBinding]) -> Result<(), FullSourceRustBindingError> {
    if artifacts.len() != REQUIRED_NATIVE_ARTIFACTS.len() {
        return Err(binding_error(
            FullSourceRustBindingErrorKind::ArtifactMismatch,
            format!(
                "full-source native artifact count must be {}, got {}",
                REQUIRED_NATIVE_ARTIFACTS.len(),
                artifacts.len()
            ),
        ));
    }
    let mut observed = BTreeMap::new();
    for artifact in artifacts {
        validate_relative_path("native artifact path", &artifact.path)?;
        validate_digest("native artifact digest", &artifact.content_digest_blake3)?;
        if observed.insert(artifact.path.as_str(), artifact.role).is_some() {
            return Err(binding_error(
                FullSourceRustBindingErrorKind::ArtifactMismatch,
                format!("duplicate native artifact path '{}'", artifact.path),
            ));
        }
    }
    for (path, role) in REQUIRED_NATIVE_ARTIFACTS {
        if observed.get(path) != Some(role) {
            return Err(binding_error(
                FullSourceRustBindingErrorKind::RoleSubstitution,
                format!("native artifact '{path}' is missing or has the wrong role"),
            ));
        }
    }
    Ok(())
}

fn validate_bound_rust_sources(
    metadata: &RustSourceProviderMetadata,
    bound_source_ids: &[String],
) -> Result<(), FullSourceRustBindingError> {
    validate_count("bound Rust sources", bound_source_ids.len(), RUST_BINDING_SOURCE_COUNT_MAX)?;
    let expected = metadata.sources.iter().map(|source| source.id.as_str()).collect::<BTreeSet<_>>();
    let observed = unique_text_set("bound Rust source", bound_source_ids)?;
    if observed == expected {
        return Ok(());
    }
    Err(binding_error(
        FullSourceRustBindingErrorKind::SourceMismatch,
        "full-source binding Rust source IDs do not match provider metadata",
    ))
}

fn validate_bound_rust_receipts(
    metadata: &RustSourceProviderMetadata,
    bound_receipts: &[FullSourceRustReceiptBinding],
) -> Result<(), FullSourceRustBindingError> {
    validate_count("bound Rust receipts", bound_receipts.len(), RUST_BINDING_RECEIPT_COUNT_MAX)?;
    let expected = metadata.build_receipts.iter().map(rust_receipt_key).collect::<BTreeSet<_>>();
    let mut observed = BTreeSet::new();
    for receipt in bound_receipts {
        validate_relative_path("bound Rust receipt path", &receipt.path)?;
        validate_digest("bound Rust receipt digest", &receipt.digest_blake3)?;
        if !observed.insert(full_source_rust_receipt_key(receipt)) {
            return Err(binding_error(
                FullSourceRustBindingErrorKind::ReceiptMismatch,
                format!("duplicate bound Rust receipt '{}'", receipt.id),
            ));
        }
    }
    if observed == expected {
        return Ok(());
    }
    Err(binding_error(
        FullSourceRustBindingErrorKind::ReceiptMismatch,
        "full-source binding Rust receipt identities do not match provider metadata",
    ))
}

fn validate_bound_rust_artifacts(
    metadata: &RustSourceProviderMetadata,
    bound_artifacts: &[FullSourceRustArtifactBinding],
) -> Result<(), FullSourceRustBindingError> {
    validate_count("bound Rust artifacts", bound_artifacts.len(), RUST_BINDING_ARTIFACT_COUNT_MAX)?;
    let expected = metadata.artifacts.iter().map(rust_artifact_key).collect::<BTreeSet<_>>();
    let mut observed = BTreeSet::new();
    for artifact in bound_artifacts {
        validate_relative_path("bound Rust artifact path", &artifact.path)?;
        validate_digest("bound Rust artifact digest", &artifact.content_digest_blake3)?;
        if !observed.insert(full_source_rust_artifact_key(artifact)) {
            return Err(binding_error(
                FullSourceRustBindingErrorKind::ArtifactMismatch,
                format!("duplicate bound Rust artifact '{}'", artifact.name),
            ));
        }
    }
    if observed == expected {
        return Ok(());
    }
    Err(binding_error(
        FullSourceRustBindingErrorKind::ArtifactMismatch,
        "full-source binding Rust artifact identities do not match provider metadata",
    ))
}

fn validate_zero_exceptions(binding: &FullSourceRustProviderBindingReceipt) -> Result<(), FullSourceRustBindingError> {
    require_empty("fallback_events", &binding.fallback_events)?;
    require_empty("seed_exceptions", &binding.seed_exceptions)
}

fn validate_closure_binding_header(
    binding: &FullSourceToolchainClosureBinding,
) -> Result<(), FullSourceRustBindingError> {
    require_text("closure binding schema", &binding.schema, FULL_SOURCE_CLOSURE_BINDING_SCHEMA)?;
    validate_digest("native provider output digest", &binding.native_provider_output_digest_blake3)?;
    validate_digest(
        "native provider admission report digest",
        &binding.native_provider_admission_report_digest_blake3,
    )?;
    validate_digest("Rust provider binding receipt digest", &binding.rust_provider_binding_receipt_digest_blake3)?;
    validate_digest("Rust provider policy digest", &binding.rust_provider_policy_digest_blake3)?;
    validate_digest("native closure policy digest", &binding.native_closure_policy_digest_blake3)
}

fn normalize_native_artifacts(artifacts: &[FullSourceNativeArtifactBinding]) -> Vec<FullSourceNativeArtifactBinding> {
    let mut normalized = artifacts.to_vec();
    normalized.sort_by_key(|artifact| (artifact.path.clone(), artifact.role));
    normalized
}

fn normalize_binding(binding: &FullSourceRustProviderBindingReceipt) -> FullSourceRustProviderBindingReceipt {
    let mut normalized = binding.clone();
    normalized.native_artifacts = normalize_native_artifacts(&binding.native_artifacts);
    normalized.rust_source_ids.sort();
    normalized.rust_build_receipts.sort_by_key(full_source_rust_receipt_key);
    normalized.rust_artifacts.sort_by_key(full_source_rust_artifact_key);
    normalized.fallback_events.sort();
    normalized.seed_exceptions.sort();
    normalized
}

fn digest_binding(binding: &FullSourceRustProviderBindingReceipt) -> Result<String, FullSourceRustBindingError> {
    let bytes = canonical_full_source_rust_binding_bytes(binding)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn digest_closure_binding(binding: &FullSourceToolchainClosureBinding) -> Result<String, FullSourceRustBindingError> {
    digest_payload(FULL_SOURCE_CLOSURE_DIGEST_CONTEXT, binding)
}

fn digest_payload<T: Serialize>(context: &str, value: &T) -> Result<String, FullSourceRustBindingError> {
    let payload = serde_json::json!({ "context": context, "value": value });
    let bytes = serde_json::to_vec(&payload).map_err(|error| {
        binding_error(
            FullSourceRustBindingErrorKind::Serialization,
            format!("serializing full-source binding: {error}"),
        )
    })?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn rust_receipt_key(
    receipt: &RustProviderBuildReceiptIdentity,
) -> (String, ToolchainBuildReceiptKind, String, String, String) {
    (
        receipt.id.clone(),
        receipt.kind,
        receipt.name.clone(),
        receipt.path.clone(),
        receipt.digest_blake3.clone(),
    )
}

fn full_source_rust_receipt_key(
    receipt: &FullSourceRustReceiptBinding,
) -> (String, ToolchainBuildReceiptKind, String, String, String) {
    (
        receipt.id.clone(),
        receipt.kind,
        receipt.name.clone(),
        receipt.path.clone(),
        receipt.digest_blake3.clone(),
    )
}

fn rust_artifact_key(artifact: &RustProviderArtifact) -> (RustProviderRole, String, String, String, String, String) {
    (
        artifact.role,
        artifact.name.clone(),
        artifact.path.clone(),
        artifact.content_digest_blake3.clone(),
        artifact.source_id.clone(),
        artifact.build_receipt_id.clone(),
    )
}

fn full_source_rust_artifact_key(
    artifact: &FullSourceRustArtifactBinding,
) -> (RustProviderRole, String, String, String, String, String) {
    (
        artifact.role,
        artifact.name.clone(),
        artifact.path.clone(),
        artifact.content_digest_blake3.clone(),
        artifact.source_id.clone(),
        artifact.build_receipt_id.clone(),
    )
}

fn unique_text_set<'a>(label: &str, values: &'a [String]) -> Result<BTreeSet<&'a str>, FullSourceRustBindingError> {
    let mut unique = BTreeSet::new();
    for value in values {
        validate_nonempty(label, value)?;
        if !unique.insert(value.as_str()) {
            return Err(binding_error(
                FullSourceRustBindingErrorKind::SourceMismatch,
                format!("duplicate {label} '{value}'"),
            ));
        }
    }
    Ok(unique)
}

fn validate_count(label: &str, len: usize, max: u32) -> Result<(), FullSourceRustBindingError> {
    let count = u32::try_from(len).map_err(|_| {
        binding_error(
            FullSourceRustBindingErrorKind::InvalidProviderIdentity,
            format!("{label} count exceeds the supported range"),
        )
    })?;
    if count == 0 || count > max {
        return Err(binding_error(
            FullSourceRustBindingErrorKind::InvalidProviderIdentity,
            format!("{label} count is out of bounds"),
        ));
    }
    Ok(())
}

fn validate_relative_path(label: &str, value: &str) -> Result<(), FullSourceRustBindingError> {
    validate_nonempty(label, value)?;
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(binding_error(
            FullSourceRustBindingErrorKind::InvalidPath,
            format!("{label} must be provider-relative: {value}"),
        ));
    }
    for component in path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(binding_error(
                FullSourceRustBindingErrorKind::InvalidPath,
                format!("{label} contains a non-normal component: {value}"),
            ));
        }
    }
    Ok(())
}

fn validate_digest(label: &str, digest: &str) -> Result<(), FullSourceRustBindingError> {
    let valid_length = digest.len() == BLAKE3_HEX_LENGTH;
    let valid_alphabet = digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if valid_length && valid_alphabet {
        return Ok(());
    }
    Err(binding_error(
        FullSourceRustBindingErrorKind::DigestMismatch,
        format!("{label} must be lowercase BLAKE3 hex"),
    ))
}

fn validate_nonempty(label: &str, value: &str) -> Result<(), FullSourceRustBindingError> {
    if !value.trim().is_empty() && !value.contains('\0') {
        return Ok(());
    }
    Err(binding_error(
        FullSourceRustBindingErrorKind::InvalidProviderIdentity,
        format!("{label} is empty or contains NUL"),
    ))
}

fn require_text(label: &str, actual: &str, expected: &str) -> Result<(), FullSourceRustBindingError> {
    if actual == expected {
        return Ok(());
    }
    Err(binding_error(
        FullSourceRustBindingErrorKind::InvalidProviderIdentity,
        format!("{label} mismatch: expected '{expected}', got '{actual}'"),
    ))
}

fn require_digest_equal(label: &str, actual: &str, expected: &str) -> Result<(), FullSourceRustBindingError> {
    validate_digest(label, actual)?;
    validate_digest(label, expected)?;
    if actual == expected {
        return Ok(());
    }
    Err(binding_error(FullSourceRustBindingErrorKind::DigestMismatch, format!("{label} digest mismatch")))
}

fn require_equal<T: PartialEq + fmt::Debug>(
    label: &str,
    actual: &T,
    expected: &T,
) -> Result<(), FullSourceRustBindingError> {
    if actual == expected {
        return Ok(());
    }
    Err(binding_error(
        FullSourceRustBindingErrorKind::NativeProviderMismatch,
        format!("{label} does not match independently observed evidence"),
    ))
}

fn require_empty<T>(label: &str, values: &[T]) -> Result<(), FullSourceRustBindingError> {
    if values.is_empty() {
        return Ok(());
    }
    Err(binding_error(FullSourceRustBindingErrorKind::SeedException, format!("{label} must be empty")))
}

fn binding_error(kind: FullSourceRustBindingErrorKind, message: impl Into<String>) -> FullSourceRustBindingError {
    FullSourceRustBindingError {
        kind,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_ID;
    use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA;
    use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_SCHEMA;
    use crate::source_toolchain_closure::RustProviderObservedBuildReceipt;
    use crate::source_toolchain_closure::RustProviderReceiptArtifact;
    use crate::source_toolchain_closure::RustProviderReceiptStep;
    use crate::source_toolchain_closure::RustProviderSourceIdentity;
    use crate::source_toolchain_closure::RustSourceProviderBuildReceipt;
    use crate::source_toolchain_closure::RustSourceProviderProvenance;
    use crate::source_toolchain_closure::SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA;
    use crate::source_toolchain_closure::ToolchainBuildReceiptIdentity;
    use crate::source_toolchain_closure::ToolchainClosureMember;
    use crate::source_toolchain_closure::ToolchainRole;
    use crate::source_toolchain_closure::ToolchainSourceIdentity;
    use crate::source_toolchain_closure::ToolchainSourceKind;
    use crate::source_toolchain_closure::ToolchainTrust;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const DIGEST_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const DIGEST_E: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    const SOURCE_RECORD_COUNT: u32 = 7;

    fn validate_full_source_rust_provider_binding(
        metadata: &RustSourceProviderMetadata,
        observed_receipts: &[RustProviderObservedBuildReceipt],
        binding: &FullSourceRustProviderBindingReceipt,
        observed_admission: &FullSourceNativeProviderAdmissionIdentity,
    ) -> Result<FullSourceRustProviderBindingValidation, FullSourceRustBindingError> {
        super::validate_full_source_rust_provider_binding(
            metadata,
            observed_receipts,
            binding,
            observed_admission,
            &valid_native_artifacts(),
        )
    }

    #[test]
    fn full_source_binding_accepts_complete_native_and_rust_evidence() {
        let metadata = valid_metadata();
        let observed_receipts = valid_observed_receipts(&metadata);
        let provider_validation = validate_rust_source_provider_metadata(&metadata).unwrap();
        let admission = valid_admission();
        let binding = valid_binding(&metadata, &provider_validation.policy_digest_blake3, &admission);

        let validation =
            validate_full_source_rust_provider_binding(&metadata, &observed_receipts, &binding, &admission).unwrap();
        let closure = valid_closure();
        let closure_validation = validate_toolchain_closure_manifest(&closure).unwrap();
        let closure_binding = FullSourceToolchainClosureBinding {
            schema: FULL_SOURCE_CLOSURE_BINDING_SCHEMA.to_string(),
            native_provider_output_digest_blake3: admission.output_digest_blake3.clone(),
            native_provider_admission_report_digest_blake3: admission.admission_report_digest_blake3.clone(),
            rust_provider_binding_receipt_digest_blake3: validation.binding_receipt_digest_blake3.clone(),
            rust_provider_policy_digest_blake3: validation.rust_provider_policy_digest_blake3.clone(),
            native_closure_policy_digest_blake3: closure_validation.policy_digest_blake3,
            seed_exceptions: Vec::new(),
        };

        let closure_result =
            validate_full_source_toolchain_closure_binding(&closure, &closure_binding, &validation, &admission)
                .unwrap();
        assert_eq!(validation.native_artifact_count, REQUIRED_NATIVE_ARTIFACTS.len());
        assert_eq!(closure_result.member_count, closure.members.len());
        assert_eq!(
            closure_result.rust_provider_binding_receipt_digest_blake3,
            validation.binding_receipt_digest_blake3
        );
    }

    #[test]
    fn full_source_binding_rejects_wrong_native_provider_identity() {
        let metadata = valid_metadata();
        let observed_receipts = valid_observed_receipts(&metadata);
        let provider_validation = validate_rust_source_provider_metadata(&metadata).unwrap();
        let admission = valid_admission();
        let mut binding = valid_binding(&metadata, &provider_validation.policy_digest_blake3, &admission);
        binding.native_provider.output_digest_blake3 = DIGEST_E.to_string();
        binding.native_provider.expected_output_digest_blake3 = DIGEST_E.to_string();

        let error = validate_full_source_rust_provider_binding(&metadata, &observed_receipts, &binding, &admission)
            .unwrap_err();
        assert_eq!(error.kind(), FullSourceRustBindingErrorKind::NativeProviderMismatch);
        assert!(error.to_string().contains("independently observed"));
    }

    #[test]
    fn full_source_binding_rejects_host_target_role_substitution() {
        let metadata = valid_metadata();
        let observed_receipts = valid_observed_receipts(&metadata);
        let provider_validation = validate_rust_source_provider_metadata(&metadata).unwrap();
        let admission = valid_admission();
        let mut binding = valid_binding(&metadata, &provider_validation.policy_digest_blake3, &admission);
        binding.host_triple = "x86_64-unknown-linux-gnu".to_string();

        let error = validate_full_source_rust_provider_binding(&metadata, &observed_receipts, &binding, &admission)
            .unwrap_err();
        assert_eq!(error.kind(), FullSourceRustBindingErrorKind::InvalidProviderIdentity);
        assert!(error.to_string().contains("host_triple"));
    }

    #[test]
    fn full_source_binding_rejects_missing_stage_receipt() {
        let metadata = valid_metadata();
        let observed_receipts = valid_observed_receipts(&metadata);
        let provider_validation = validate_rust_source_provider_metadata(&metadata).unwrap();
        let admission = valid_admission();
        let mut binding = valid_binding(&metadata, &provider_validation.policy_digest_blake3, &admission);
        binding.rust_build_receipts.clear();

        let error = validate_full_source_rust_provider_binding(&metadata, &observed_receipts, &binding, &admission)
            .unwrap_err();
        assert_eq!(error.kind(), FullSourceRustBindingErrorKind::InvalidProviderIdentity);
        assert!(error.to_string().contains("receipt"));
    }

    #[test]
    fn full_source_binding_rejects_ambient_tool_discovery() {
        let metadata = valid_metadata();
        let observed_receipts = valid_observed_receipts(&metadata);
        let provider_validation = validate_rust_source_provider_metadata(&metadata).unwrap();
        let admission = valid_admission();
        let mut binding = valid_binding(&metadata, &provider_validation.policy_digest_blake3, &admission);
        binding.ambient_tool_discovery = true;

        let error = validate_full_source_rust_provider_binding(&metadata, &observed_receipts, &binding, &admission)
            .unwrap_err();

        assert_eq!(error.kind(), FullSourceRustBindingErrorKind::AmbientDiscovery);
        assert!(error.to_string().contains("ambient tool discovery"));
    }

    #[test]
    fn full_source_binding_rejects_source_closure_record_count_mismatch() {
        let metadata = valid_metadata();
        let observed_receipts = valid_observed_receipts(&metadata);
        let provider_validation = validate_rust_source_provider_metadata(&metadata).unwrap();
        let admission = valid_admission();
        let mut binding = valid_binding(&metadata, &provider_validation.policy_digest_blake3, &admission);
        binding.native_provider.source_closure_record_count = SOURCE_RECORD_COUNT + 1;

        let error = validate_full_source_rust_provider_binding(&metadata, &observed_receipts, &binding, &admission)
            .unwrap_err();

        assert_eq!(error.kind(), FullSourceRustBindingErrorKind::NativeProviderMismatch);
        assert!(error.to_string().contains("independently observed"));
    }

    #[test]
    fn full_source_binding_rejects_native_artifact_role_substitution() {
        let metadata = valid_metadata();
        let observed_receipts = valid_observed_receipts(&metadata);
        let provider_validation = validate_rust_source_provider_metadata(&metadata).unwrap();
        let admission = valid_admission();
        let mut binding = valid_binding(&metadata, &provider_validation.policy_digest_blake3, &admission);
        binding.native_artifacts[0].role = FullSourceNativeArtifactRole::Linker;

        let error = validate_full_source_rust_provider_binding(&metadata, &observed_receipts, &binding, &admission)
            .unwrap_err();
        assert_eq!(error.kind(), FullSourceRustBindingErrorKind::RoleSubstitution);
        assert!(error.to_string().contains("wrong role"));
    }

    #[test]
    fn full_source_binding_rejects_native_artifact_digest_mismatch() {
        let metadata = valid_metadata();
        let observed_receipts = valid_observed_receipts(&metadata);
        let provider_validation = validate_rust_source_provider_metadata(&metadata).unwrap();
        let admission = valid_admission();
        let mut binding = valid_binding(&metadata, &provider_validation.policy_digest_blake3, &admission);
        binding.native_artifacts[0].content_digest_blake3 = DIGEST_A.to_string();

        let error = validate_full_source_rust_provider_binding(&metadata, &observed_receipts, &binding, &admission)
            .unwrap_err();

        assert_eq!(error.kind(), FullSourceRustBindingErrorKind::NativeProviderMismatch);
        assert!(error.to_string().contains("native artifact identities"));
    }

    #[test]
    fn full_source_closure_binding_rejects_seed_exception() {
        let metadata = valid_metadata();
        let observed_receipts = valid_observed_receipts(&metadata);
        let provider_validation = validate_rust_source_provider_metadata(&metadata).unwrap();
        let admission = valid_admission();
        let binding = valid_binding(&metadata, &provider_validation.policy_digest_blake3, &admission);
        let rust_validation =
            validate_full_source_rust_provider_binding(&metadata, &observed_receipts, &binding, &admission).unwrap();
        let mut closure = valid_closure();
        closure.seed_exceptions.push(crate::source_toolchain_closure::ToolchainSeedException {
            name: "forbidden-seed".to_string(),
            reason: "compatibility".to_string(),
            content_digest_blake3: DIGEST_A.to_string(),
        });
        let closure_binding = FullSourceToolchainClosureBinding {
            schema: FULL_SOURCE_CLOSURE_BINDING_SCHEMA.to_string(),
            native_provider_output_digest_blake3: admission.output_digest_blake3.clone(),
            native_provider_admission_report_digest_blake3: admission.admission_report_digest_blake3.clone(),
            rust_provider_binding_receipt_digest_blake3: rust_validation.binding_receipt_digest_blake3.clone(),
            rust_provider_policy_digest_blake3: rust_validation.rust_provider_policy_digest_blake3.clone(),
            native_closure_policy_digest_blake3: DIGEST_A.to_string(),
            seed_exceptions: Vec::new(),
        };

        let error =
            validate_full_source_toolchain_closure_binding(&closure, &closure_binding, &rust_validation, &admission)
                .unwrap_err();
        assert_eq!(error.kind(), FullSourceRustBindingErrorKind::SeedException);
        assert!(error.to_string().contains("seed"));
    }

    fn valid_admission() -> FullSourceNativeProviderAdmissionIdentity {
        FullSourceNativeProviderAdmissionIdentity {
            schema: FULL_SOURCE_ADMISSION_SCHEMA.to_string(),
            status: FULL_SOURCE_ADMISSION_STATUS.to_string(),
            provider_id: FULL_SOURCE_PROVIDER_ID.to_string(),
            provider_target: FULL_SOURCE_PROVIDER_TARGET.to_string(),
            compiler_target: FULL_SOURCE_COMPILER_TARGET.to_string(),
            admission_report_digest_blake3: DIGEST_A.to_string(),
            metadata_digest_blake3: DIGEST_B.to_string(),
            output_digest_blake3: DIGEST_C.to_string(),
            expected_output_digest_blake3: DIGEST_C.to_string(),
            source_closure_manifest_blake3: DIGEST_D.to_string(),
            expected_source_closure_manifest_blake3: DIGEST_D.to_string(),
            source_closure_record_count: SOURCE_RECORD_COUNT,
        }
    }

    fn valid_binding(
        metadata: &RustSourceProviderMetadata,
        policy_digest_blake3: &str,
        admission: &FullSourceNativeProviderAdmissionIdentity,
    ) -> FullSourceRustProviderBindingReceipt {
        FullSourceRustProviderBindingReceipt {
            schema: FULL_SOURCE_RUST_BINDING_SCHEMA.to_string(),
            receipt_id: "full-source-rust-provider-construction".to_string(),
            rust_provider_id: metadata.provider_id.clone(),
            host_triple: metadata.host_triple.clone(),
            target_triple: metadata.target_triple.clone(),
            source_policy: FULL_SOURCE_POLICY.to_string(),
            ambient_tool_discovery: false,
            rust_provider_policy_digest_blake3: policy_digest_blake3.to_string(),
            native_provider: admission.clone(),
            native_artifacts: valid_native_artifacts(),
            rust_source_ids: metadata.sources.iter().map(|source| source.id.clone()).collect(),
            rust_build_receipts: metadata
                .build_receipts
                .iter()
                .map(|receipt| FullSourceRustReceiptBinding {
                    id: receipt.id.clone(),
                    kind: receipt.kind,
                    name: receipt.name.clone(),
                    path: receipt.path.clone(),
                    digest_blake3: receipt.digest_blake3.clone(),
                })
                .collect(),
            rust_artifacts: metadata
                .artifacts
                .iter()
                .map(|artifact| FullSourceRustArtifactBinding {
                    role: artifact.role,
                    name: artifact.name.clone(),
                    path: artifact.path.clone(),
                    content_digest_blake3: artifact.content_digest_blake3.clone(),
                    source_id: artifact.source_id.clone(),
                    build_receipt_id: artifact.build_receipt_id.clone(),
                })
                .collect(),
            fallback_events: Vec::new(),
            seed_exceptions: Vec::new(),
        }
    }

    fn valid_native_artifacts() -> Vec<FullSourceNativeArtifactBinding> {
        REQUIRED_NATIVE_ARTIFACTS
            .iter()
            .map(|(path, role)| FullSourceNativeArtifactBinding {
                role: *role,
                path: (*path).to_string(),
                content_digest_blake3: DIGEST_E.to_string(),
            })
            .collect()
    }

    fn valid_metadata() -> RustSourceProviderMetadata {
        RustSourceProviderMetadata {
            schema: RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
            provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
            host_triple: FULL_SOURCE_RUST_TRIPLE.to_string(),
            target_triple: FULL_SOURCE_RUST_TRIPLE.to_string(),
            provenance: RustSourceProviderProvenance {
                source_built: true,
                uses_prebuilt_rust: false,
                build_recipe: "mrustc-source-route-musl-host".to_string(),
            },
            sources: vec![RustProviderSourceIdentity {
                id: "rust-stage-source".to_string(),
                kind: ToolchainSourceKind::Tarball,
                name: "rust-compiler-source".to_string(),
                digest_blake3: DIGEST_A.to_string(),
            }],
            build_receipts: vec![RustProviderBuildReceiptIdentity {
                id: "stage-receipt".to_string(),
                kind: ToolchainBuildReceiptKind::MantleDerivation,
                name: "rust-stage-source-build".to_string(),
                path: "share/mantle-rust-provider/receipts/stage.json".to_string(),
                digest_blake3: DIGEST_B.to_string(),
            }],
            artifacts: vec![
                rust_artifact(RustProviderRole::Rustc, "rustc", "bin/rustc", DIGEST_C),
                rust_artifact(RustProviderRole::Cargo, "cargo", "bin/cargo", DIGEST_D),
                rust_artifact(
                    RustProviderRole::HostRustlib,
                    "host-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-musl/lib",
                    DIGEST_E,
                ),
                rust_artifact(
                    RustProviderRole::TargetRustlib,
                    "target-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-musl/lib/target",
                    DIGEST_A,
                ),
                rust_artifact(
                    RustProviderRole::ProviderReceipt,
                    "stage-receipt",
                    "share/mantle-rust-provider/receipts/stage.json",
                    DIGEST_B,
                ),
            ],
        }
    }

    fn rust_artifact(role: RustProviderRole, name: &str, path: &str, digest: &str) -> RustProviderArtifact {
        RustProviderArtifact {
            role,
            name: name.to_string(),
            path: path.to_string(),
            content_digest_blake3: digest.to_string(),
            source_id: "rust-stage-source".to_string(),
            build_receipt_id: "stage-receipt".to_string(),
        }
    }

    fn valid_observed_receipts(metadata: &RustSourceProviderMetadata) -> Vec<RustProviderObservedBuildReceipt> {
        let output_artifacts = metadata
            .artifacts
            .iter()
            .filter(|artifact| artifact.role != RustProviderRole::ProviderReceipt)
            .map(|artifact| RustProviderReceiptArtifact {
                role: artifact.role,
                name: artifact.name.clone(),
                path: artifact.path.clone(),
                content_digest_blake3: artifact.content_digest_blake3.clone(),
            })
            .collect();
        vec![RustProviderObservedBuildReceipt {
            path: "share/mantle-rust-provider/receipts/stage.json".to_string(),
            content_digest_blake3: DIGEST_B.to_string(),
            receipt: RustSourceProviderBuildReceipt {
                schema: RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA.to_string(),
                receipt_id: "stage-receipt".to_string(),
                provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
                host_triple: FULL_SOURCE_RUST_TRIPLE.to_string(),
                target_triple: FULL_SOURCE_RUST_TRIPLE.to_string(),
                source_ids: vec!["rust-stage-source".to_string()],
                output_artifacts,
                build_steps: vec![RustProviderReceiptStep {
                    name: "compile-rust-source".to_string(),
                    program: "/source-provider/bin/rustc".to_string(),
                    arguments: vec!["--crate-name".to_string(), "rustc-main".to_string()],
                }],
            },
        }]
    }

    fn valid_closure() -> ToolchainClosureManifest {
        ToolchainClosureManifest {
            schema: SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA.to_string(),
            members: vec![
                closure_member(ToolchainRole::Rustc, "rustc", DIGEST_A),
                closure_member(ToolchainRole::Linker, "ld", DIGEST_B),
                closure_member(ToolchainRole::CCompiler, "cc", DIGEST_C),
                closure_member(ToolchainRole::Sysroot, "sysroot", DIGEST_D),
            ],
            seed_exceptions: Vec::new(),
        }
    }

    fn closure_member(role: ToolchainRole, name: &str, digest: &str) -> ToolchainClosureMember {
        ToolchainClosureMember {
            role,
            name: name.to_string(),
            execution_path: format!("/full-source/{name}"),
            content_digest_blake3: digest.to_string(),
            trust: ToolchainTrust::SourceBuilt,
            source: Some(ToolchainSourceIdentity {
                kind: ToolchainSourceKind::Tarball,
                name: format!("{name}-source"),
                digest_blake3: DIGEST_E.to_string(),
            }),
            build_receipt: Some(ToolchainBuildReceiptIdentity {
                kind: ToolchainBuildReceiptKind::MantleDerivation,
                name: format!("{name}-receipt"),
                digest_blake3: DIGEST_A.to_string(),
            }),
        }
    }
}
