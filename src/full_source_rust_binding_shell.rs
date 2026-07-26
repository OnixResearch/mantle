//! Imperative shell for constructing the full-source Rust binding receipt.

use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crate::full_source_provider::FullSourceProviderAdmissionReport;
use crate::full_source_provider::admit_full_source_provider;
use crate::full_source_rust_binding::FULL_SOURCE_RUST_BINDING_SCHEMA;
use crate::full_source_rust_binding::FullSourceNativeArtifactBinding;
use crate::full_source_rust_binding::FullSourceNativeProviderAdmissionIdentity;
use crate::full_source_rust_binding::FullSourceRustArtifactBinding;
use crate::full_source_rust_binding::FullSourceRustProviderBindingReceipt;
use crate::full_source_rust_binding::FullSourceRustReceiptBinding;
use crate::full_source_rust_binding::canonical_full_source_rust_binding_bytes;
use crate::full_source_rust_binding::required_full_source_native_artifacts;
use crate::full_source_rust_binding::validate_full_source_rust_provider_binding;
use crate::rust_source_provider::RustSourceProviderError;
use crate::rust_source_provider::observed_provider_receipts;
use crate::rust_source_provider::validate_materialized_rust_source_provider;
use crate::source_toolchain_closure::RustSourceProviderMetadata;

pub(crate) const FULL_SOURCE_RUST_BINDING_RELATIVE_PATH: &str =
    "share/mantle-rust-provider/receipts/full-source-binding.json";
const FULL_SOURCE_BINDING_RECEIPT_ID: &str = "full-source-rust-provider-construction";
const FULL_SOURCE_PROVIDER_TARGET: &str = "x86_64-linux-musl";
const FULL_SOURCE_COMPILER_TARGET: &str = "x86_64-unknown-linux-musl";
const FULL_SOURCE_SOURCE_POLICY: &str = "authenticated-offline-only";
const CURRENT_ROUTE_USES_AMBIENT_TOOL_DISCOVERY: bool = true;
const FULL_SOURCE_ROUTE_BLOCKER: &str = "FULL_SOURCE_RUST_SOURCE_BUILT_CMAKE_NOT_MATERIALIZED: the current Rust route discovers ambient CMake and orchestration tools";
const BLAKE3_HEX_LENGTH: usize = 64;
const NATIVE_ARTIFACT_BYTES_MAX: u64 = 536_870_912;
const HASH_BUFFER_BYTES: usize = 65_536;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FullSourceRustBindingPublication {
    pub(crate) path: PathBuf,
    pub(crate) content_digest_blake3: String,
    pub(crate) native_artifact_count: usize,
}

pub(crate) fn ensure_full_source_binding_route_ready() -> Result<(), RustSourceProviderError> {
    if CURRENT_ROUTE_USES_AMBIENT_TOOL_DISCOVERY {
        return Err(RustSourceProviderError::Validate(FULL_SOURCE_ROUTE_BLOCKER.to_string()));
    }
    Ok(())
}

pub(crate) fn bind_full_source_rust_provider_candidate(
    provider_candidate_dir: &Path,
    admission_report_path: &Path,
) -> Result<FullSourceRustBindingPublication, RustSourceProviderError> {
    let report_bytes = read_nonempty(admission_report_path, "full-source admission report")?;
    let parsed_report =
        serde_json::from_slice::<FullSourceProviderAdmissionReport>(&report_bytes).map_err(|error| {
            RustSourceProviderError::Parse(format!(
                "full-source admission report {}: {error}",
                admission_report_path.display()
            ))
        })?;
    let revalidated_report = revalidate_admission_report(&parsed_report)?;
    validate_report_matches_revalidation(&parsed_report, &revalidated_report)?;
    let admission = admission_identity(&revalidated_report, &report_bytes)?;
    let native_artifacts = observe_native_artifacts(&revalidated_report.provider_path)?;
    let provider = validate_materialized_rust_source_provider(provider_candidate_dir)?;
    let observed_receipts = observed_provider_receipts(provider_candidate_dir, &provider.metadata)?;
    let binding = binding_receipt(
        &provider.metadata,
        &provider.validation.policy_digest_blake3,
        admission.clone(),
        native_artifacts.clone(),
    );
    let validation = validate_full_source_rust_provider_binding(
        &provider.metadata,
        &observed_receipts,
        &binding,
        &admission,
        &native_artifacts,
    )
    .map_err(|error| RustSourceProviderError::Validate(error.to_string()))?;
    let canonical_bytes = canonical_full_source_rust_binding_bytes(&binding)
        .map_err(|error| RustSourceProviderError::Validate(error.to_string()))?;
    let content_digest_blake3 = blake3::hash(&canonical_bytes).to_hex().to_string();
    if content_digest_blake3 != validation.binding_receipt_digest_blake3 {
        return Err(RustSourceProviderError::Digest(
            "canonical full-source binding digest changed between validation and publication".to_string(),
        ));
    }
    let path = provider_candidate_dir.join(FULL_SOURCE_RUST_BINDING_RELATIVE_PATH);
    write_create_new(&path, &canonical_bytes)?;
    Ok(FullSourceRustBindingPublication {
        path,
        content_digest_blake3,
        native_artifact_count: validation.native_artifact_count,
    })
}

fn revalidate_admission_report(
    report: &FullSourceProviderAdmissionReport,
) -> Result<FullSourceProviderAdmissionReport, RustSourceProviderError> {
    admit_full_source_provider(
        &report.provider_path,
        &report.expected_output_digest_blake3,
        &report.source_closure_path,
        &report.expected_source_closure_manifest_blake3,
    )
    .map_err(|error| RustSourceProviderError::Validate(format!("revalidating full-source admission: {error}")))
}

fn validate_report_matches_revalidation(
    parsed: &FullSourceProviderAdmissionReport,
    revalidated: &FullSourceProviderAdmissionReport,
) -> Result<(), RustSourceProviderError> {
    let parsed_facts = admission_report_facts(parsed);
    let revalidated_facts = admission_report_facts(revalidated);
    if parsed_facts == revalidated_facts {
        return Ok(());
    }
    Err(RustSourceProviderError::Validate(
        "full-source admission report does not match independent revalidation".to_string(),
    ))
}

fn admission_report_facts(
    report: &FullSourceProviderAdmissionReport,
) -> (&str, &str, &str, &Path, &str, &str, &str, &Path, &str, &str, usize) {
    (
        &report.schema,
        &report.status,
        &report.provider_id,
        &report.provider_path,
        &report.metadata_digest_blake3,
        &report.output_digest_blake3,
        &report.expected_output_digest_blake3,
        &report.source_closure_path,
        &report.source_closure_manifest_blake3,
        &report.expected_source_closure_manifest_blake3,
        report.source_closure_record_count,
    )
}

fn admission_identity(
    report: &FullSourceProviderAdmissionReport,
    report_bytes: &[u8],
) -> Result<FullSourceNativeProviderAdmissionIdentity, RustSourceProviderError> {
    let source_closure_record_count = u32::try_from(report.source_closure_record_count)
        .map_err(|_| RustSourceProviderError::Validate("full-source admission record count exceeds u32".to_string()))?;
    Ok(FullSourceNativeProviderAdmissionIdentity {
        schema: report.schema.clone(),
        status: report.status.clone(),
        provider_id: report.provider_id.clone(),
        provider_target: FULL_SOURCE_PROVIDER_TARGET.to_string(),
        compiler_target: FULL_SOURCE_COMPILER_TARGET.to_string(),
        admission_report_digest_blake3: blake3::hash(report_bytes).to_hex().to_string(),
        metadata_digest_blake3: report.metadata_digest_blake3.clone(),
        output_digest_blake3: report.output_digest_blake3.clone(),
        expected_output_digest_blake3: report.expected_output_digest_blake3.clone(),
        source_closure_manifest_blake3: report.source_closure_manifest_blake3.clone(),
        expected_source_closure_manifest_blake3: report.expected_source_closure_manifest_blake3.clone(),
        source_closure_record_count,
    })
}

fn observe_native_artifacts(
    provider_dir: &Path,
) -> Result<Vec<FullSourceNativeArtifactBinding>, RustSourceProviderError> {
    let required = required_full_source_native_artifacts();
    let mut observed = Vec::with_capacity(required.len());
    for (relative_path, role) in required {
        let path = provider_dir.join(relative_path);
        observed.push(FullSourceNativeArtifactBinding {
            role: *role,
            path: (*relative_path).to_string(),
            content_digest_blake3: hash_bounded_file(&path)?,
        });
    }
    if observed.len() != required.len() {
        return Err(RustSourceProviderError::Validate(
            "native artifact observation count changed unexpectedly".to_string(),
        ));
    }
    Ok(observed)
}

fn binding_receipt(
    metadata: &RustSourceProviderMetadata,
    policy_digest_blake3: &str,
    native_provider: FullSourceNativeProviderAdmissionIdentity,
    native_artifacts: Vec<FullSourceNativeArtifactBinding>,
) -> FullSourceRustProviderBindingReceipt {
    FullSourceRustProviderBindingReceipt {
        schema: FULL_SOURCE_RUST_BINDING_SCHEMA.to_string(),
        receipt_id: FULL_SOURCE_BINDING_RECEIPT_ID.to_string(),
        rust_provider_id: metadata.provider_id.clone(),
        host_triple: metadata.host_triple.clone(),
        target_triple: metadata.target_triple.clone(),
        source_policy: FULL_SOURCE_SOURCE_POLICY.to_string(),
        ambient_tool_discovery: CURRENT_ROUTE_USES_AMBIENT_TOOL_DISCOVERY,
        rust_provider_policy_digest_blake3: policy_digest_blake3.to_string(),
        native_provider,
        native_artifacts,
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

fn hash_bounded_file(path: &Path) -> Result<String, RustSourceProviderError> {
    let file = File::open(path)
        .map_err(|error| RustSourceProviderError::Read(format!("native artifact {}: {error}", path.display())))?;
    let metadata = file.metadata().map_err(|error| {
        RustSourceProviderError::Read(format!("native artifact metadata {}: {error}", path.display()))
    })?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > NATIVE_ARTIFACT_BYTES_MAX {
        return Err(RustSourceProviderError::MissingArtifact(format!(
            "native artifact is empty, non-file, or over limit: {}",
            path.display()
        )));
    }
    let mut reader = BufReader::with_capacity(HASH_BUFFER_BYTES, file);
    let mut buffer = [0u8; HASH_BUFFER_BYTES];
    let mut total_bytes = 0u64;
    let mut hasher = blake3::Hasher::new();
    loop {
        let read_bytes = reader
            .read(&mut buffer)
            .map_err(|error| RustSourceProviderError::Read(format!("native artifact {}: {error}", path.display())))?;
        if read_bytes == 0 {
            break;
        }
        total_bytes =
            total_bytes
                .checked_add(u64::try_from(read_bytes).map_err(|_| {
                    RustSourceProviderError::Digest("native artifact read length exceeds u64".to_string())
                })?)
                .ok_or_else(|| RustSourceProviderError::Digest("native artifact size overflow".to_string()))?;
        if total_bytes > NATIVE_ARTIFACT_BYTES_MAX {
            return Err(RustSourceProviderError::Digest(format!(
                "native artifact exceeded byte limit while hashing: {}",
                path.display()
            )));
        }
        hasher.update(&buffer[..read_bytes]);
    }
    if total_bytes != metadata.len() {
        return Err(RustSourceProviderError::Digest(format!(
            "native artifact size changed while hashing: {}",
            path.display()
        )));
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn read_nonempty(path: &Path, label: &str) -> Result<Vec<u8>, RustSourceProviderError> {
    let bytes = fs::read(path)
        .map_err(|error| RustSourceProviderError::Read(format!("{label} {}: {error}", path.display())))?;
    if bytes.is_empty() {
        return Err(RustSourceProviderError::Read(format!("{label} {} is empty", path.display())));
    }
    Ok(bytes)
}

fn write_create_new(path: &Path, bytes: &[u8]) -> Result<(), RustSourceProviderError> {
    if bytes.is_empty() {
        return Err(RustSourceProviderError::Copy(
            "refusing to publish an empty full-source binding receipt".to_string(),
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| RustSourceProviderError::Copy(format!("binding path {} has no parent", path.display())))?;
    fs::create_dir_all(parent)
        .map_err(|error| RustSourceProviderError::Copy(format!("create {}: {error}", parent.display())))?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| RustSourceProviderError::Copy(format!("create-new {}: {error}", path.display())))?;
    file.write_all(bytes)
        .map_err(|error| RustSourceProviderError::Copy(format!("write {}: {error}", path.display())))?;
    file.sync_all()
        .map_err(|error| RustSourceProviderError::Copy(format!("sync {}: {error}", path.display())))?;
    let digest = blake3::hash(bytes).to_hex().to_string();
    if digest.len() != BLAKE3_HEX_LENGTH {
        return Err(RustSourceProviderError::Digest(
            "published full-source binding digest has invalid length".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_BYTES: &[u8] = b"full-source-native-artifact";

    #[test]
    fn bounded_file_hash_accepts_nonempty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("artifact");
        fs::write(&path, TEST_BYTES).unwrap();

        let digest = hash_bounded_file(&path).unwrap();

        assert_eq!(digest, blake3::hash(TEST_BYTES).to_hex().to_string());
        assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    }

    #[test]
    fn bounded_file_hash_rejects_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("artifact");
        fs::write(&path, []).unwrap();

        let error = hash_bounded_file(&path).unwrap_err();

        assert!(error.to_string().contains("empty"));
        assert!(!error.to_string().contains("digest mismatch"));
    }

    #[test]
    fn create_new_publication_refuses_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("binding.json");
        write_create_new(&path, TEST_BYTES).unwrap();

        let error = write_create_new(&path, b"replacement").unwrap_err();

        assert_eq!(fs::read(&path).unwrap(), TEST_BYTES);
        assert!(error.to_string().contains("create-new"));
    }

    #[test]
    fn selected_full_source_route_fails_before_ambient_tool_discovery() {
        let error = ensure_full_source_binding_route_ready().unwrap_err();
        let rendered = error.to_string();

        assert!(rendered.contains("FULL_SOURCE_RUST_SOURCE_BUILT_CMAKE_NOT_MATERIALIZED"));
        assert!(rendered.contains("ambient CMake"));
    }
}
