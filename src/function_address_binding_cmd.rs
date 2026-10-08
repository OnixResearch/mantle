use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crunch_release_core::FUNCTION_ADDRESS_KAMACITE_RECEIPT_IDENTITY_SCHEMA;
use crunch_release_core::FUNCTION_ADDRESS_PRESERVES_PROFILE_VERSION;
use crunch_release_core::FUNCTION_ADDRESS_VALENCE_RECEIPT_IDENTITY_SCHEMA;
use crunch_release_core::FunctionAddressBindingReceipt;
use crunch_release_core::FunctionAddressBindingSelection;
use crunch_release_core::FunctionAddressKamaciteReceiptIdentity;
use crunch_release_core::FunctionAddressValenceReceiptIdentity;
use crunch_release_core::MAX_FUNCTION_ADDRESS_PRESERVES_SIDECAR_BYTES;
use crunch_release_core::OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS;
use crunch_release_core::OpaqueEvidenceSidecarBindingReceipt;
use crunch_release_core::ReleaseEvidenceManifest;
use crunch_release_core::function_address_binding_receipt_canonical_bytes;
use crunch_release_core::render_function_address_binding_from_manifest;
use crunch_release_core::render_function_address_binding_from_preserves_manifest;
use serde::Deserialize;
use tempfile::NamedTempFile;

use crate::errors::RunError;
use crate::release_capability::ReleaseCapabilityRoot;
use crate::release_capability::ReleasePathRequest;
use crate::release_capability::ReleaseRootKind;
use crate::release_capability::authorize_release_path;
use crate::release_evidence::verify_release_evidence_bundle;

const MAX_FUNCTION_ADDRESS_UPSTREAM_RECEIPT_BYTES: usize = 1_048_576;
const MAX_FUNCTION_ADDRESS_UPSTREAM_RECEIPT_READ_BYTES: u64 =
    MAX_FUNCTION_ADDRESS_PRESERVES_SIDECAR_BYTES.saturating_add(1);

const _: () = {
    assert!(MAX_FUNCTION_ADDRESS_UPSTREAM_RECEIPT_BYTES > 0);
    assert!(MAX_FUNCTION_ADDRESS_UPSTREAM_RECEIPT_READ_BYTES > MAX_FUNCTION_ADDRESS_PRESERVES_SIDECAR_BYTES);
};

#[derive(Debug)]
pub(crate) struct FunctionAddressBindingCommand {
    pub bundle_dir: PathBuf,
    pub mode: String,
    pub from_preserves_binding: bool,
    pub sidecar_relative_path: Option<String>,
    pub valence_receipt_relative_path: Option<String>,
    pub kamacite_receipt_relative_path: Option<String>,
    pub release_binary_relative_path: Option<String>,
    pub receipt_out: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ValenceReceiptIdentityEnvelope {
    schema_version: String,
    receipt_hash: String,
    #[serde(default = "no_kamacite_receipt_hash")]
    kamacite_receipt_hash: Option<String>,
}

fn no_kamacite_receipt_hash() -> Option<String> {
    None
}

#[derive(Debug, Deserialize)]
struct KamaciteReceiptIdentityEnvelope {
    schema_version: String,
    receipt_hash: String,
}

struct DeclaredExternalPath<'a> {
    manifest: &'a ReleaseEvidenceManifest,
    relative_path: &'a str,
    label: &'static str,
}

struct ReceiptReadRequest<'a> {
    root: &'a ReleaseCapabilityRoot,
    relative_path: &'a str,
    expected_digest_blake3: &'a str,
    label: &'static str,
}

struct DeclaredReceiptDigests {
    valence: String,
    kamacite: Option<String>,
}

struct ExplicitBindingPaths<'a> {
    sidecar: &'a str,
    valence: &'a str,
}

struct IdentityEquality<'a> {
    actual: &'a str,
    expected: &'a str,
    label: &'static str,
}

/// Written receipt and the exact bytes the CLI presents after classification.
pub(crate) struct FunctionAddressBindingOutput {
    pub receipt_path: PathBuf,
    pub receipt: FunctionAddressBindingReceipt,
    pub canonical_bytes: Vec<u8>,
}

// r[impl mantle.release_provenance.function_address_binding_cli.command]
// r[impl mantle.release_provenance.function_address_binding_cli.shell]
pub(crate) fn execute_function_address_binding(
    current_dir: &Path,
    command: FunctionAddressBindingCommand,
) -> Result<FunctionAddressBindingOutput, RunError> {
    debug_assert!(!command.mode.is_empty());
    debug_assert!(!command.receipt_out.as_os_str().is_empty());
    let bundle_dir = resolve_operator_path(current_dir, &command.bundle_dir);
    let receipt_out = resolve_operator_path(current_dir, &command.receipt_out);
    let manifest = verify_release_evidence_bundle(&bundle_dir)?;
    let root = ReleaseCapabilityRoot::open_ambient_nofollow(ReleaseRootKind::ReleaseEvidence, &bundle_dir).map_err(
        |error| RunError::Internal(format!("opening release evidence root {}: {error}", bundle_dir.display())),
    )?;
    let receipt = if command.from_preserves_binding {
        validate_preserves_manifest_identities(&manifest, &root)?;
        render_function_address_binding_from_preserves_manifest(manifest, command.mode.clone())
            .map_err(|error| RunError::Internal(error.to_string()))?
    } else {
        let paths = explicit_binding_paths(&command)?;
        let declared_digests = preflight_declared_receipt_paths(&manifest, &command, &paths)?;
        let selection = binding_selection(&root, &command, &paths, &declared_digests)?;
        render_function_address_binding_from_manifest(manifest, selection)
            .map_err(|error| RunError::Internal(error.to_string()))?
    };
    let bytes = function_address_binding_receipt_canonical_bytes(receipt.clone())
        .map_err(|error| RunError::Internal(error.to_string()))?;
    write_receipt_noclobber(&receipt_out, &bytes)?;
    Ok(FunctionAddressBindingOutput {
        receipt_path: receipt_out,
        receipt,
        canonical_bytes: bytes,
    })
}

pub(crate) fn render_function_address_binding(
    json: bool,
    output: &FunctionAddressBindingOutput,
) -> Result<(), RunError> {
    emit_binding_receipt(json, &output.receipt_path, &output.receipt, &output.canonical_bytes)?;
    reject_invalid_binding(&output.receipt)
}

fn explicit_binding_paths(command: &FunctionAddressBindingCommand) -> Result<ExplicitBindingPaths<'_>, RunError> {
    let Some(sidecar) = command.sidecar_relative_path.as_deref() else {
        return Err(RunError::Internal("explicit function-address binding requires --sidecar".to_string()));
    };
    let Some(valence) = command.valence_receipt_relative_path.as_deref() else {
        return Err(RunError::Internal("explicit function-address binding requires --valence-receipt".to_string()));
    };
    Ok(ExplicitBindingPaths { sidecar, valence })
}

fn preflight_declared_receipt_paths(
    manifest: &ReleaseEvidenceManifest,
    command: &FunctionAddressBindingCommand,
    paths: &ExplicitBindingPaths<'_>,
) -> Result<DeclaredReceiptDigests, RunError> {
    let valence = declared_external_digest(DeclaredExternalPath {
        manifest,
        relative_path: paths.valence,
        label: "Valence receipt",
    })?;
    let kamacite = command
        .kamacite_receipt_relative_path
        .as_deref()
        .map(|path| {
            declared_external_digest(DeclaredExternalPath {
                manifest,
                relative_path: path,
                label: "Kamacite receipt",
            })
        })
        .transpose()?;
    Ok(DeclaredReceiptDigests { valence, kamacite })
}

fn declared_external_digest(request: DeclaredExternalPath<'_>) -> Result<String, RunError> {
    let Some(evidence) = request
        .manifest
        .external_evidence
        .iter()
        .find(|evidence| evidence.relative_path == request.relative_path)
    else {
        return Err(RunError::Internal(format!(
            "function-address {} is not declared in release external evidence: {}",
            request.label, request.relative_path
        )));
    };
    Ok(evidence.digest_blake3.clone())
}

// r[impl mantle.release_provenance.function_address_preserves_sidecars.opaque]
// r[impl mantle.release_provenance.function_address_preserves_sidecars.json_projection]
fn validate_preserves_manifest_identities(
    manifest: &ReleaseEvidenceManifest,
    root: &ReleaseCapabilityRoot,
) -> Result<(), RunError> {
    debug_assert_eq!(
        u64::try_from(MAX_FUNCTION_ADDRESS_UPSTREAM_RECEIPT_BYTES).ok(),
        Some(MAX_FUNCTION_ADDRESS_PRESERVES_SIDECAR_BYTES)
    );
    let receipt = preserves_binding_receipt(manifest)?;
    let binding = &receipt.binding;
    let preserves_bytes = read_bounded_receipt(ReceiptReadRequest {
        root,
        relative_path: &binding.canonical_envelope.relative_path,
        expected_digest_blake3: &binding.canonical_envelope.digest_blake3,
        label: "Kamacite Preserves sidecar",
    })?;
    let actual_size_bytes = u64::try_from(preserves_bytes.len())
        .map_err(|_| RunError::Internal("Kamacite Preserves sidecar size does not fit u64".to_string()))?;
    let expected_size_bytes = binding
        .canonical_envelope
        .size_bytes
        .ok_or_else(|| RunError::Internal("Kamacite Preserves sidecar metadata is missing size_bytes".to_string()))?;
    if actual_size_bytes != expected_size_bytes {
        return Err(RunError::Internal(format!(
            "Kamacite Preserves sidecar size changed after release bundle verification: expected {expected_size_bytes}, got {actual_size_bytes}"
        )));
    }
    let valence = read_valence_identity(ReceiptReadRequest {
        root,
        relative_path: &binding.upstream_validation.relative_path,
        expected_digest_blake3: &binding.upstream_validation.digest_blake3,
        label: "Valence receipt",
    })?;
    require_identity_match(IdentityEquality {
        actual: &valence.schema_version,
        expected: FUNCTION_ADDRESS_VALENCE_RECEIPT_IDENTITY_SCHEMA,
        label: "Valence receipt schema",
    })?;
    let expected_valence_hash = binding.upstream_validation.receipt_hash_blake3.as_deref().ok_or_else(|| {
        RunError::Internal("Preserves binding is missing the Valence logical receipt hash".to_string())
    })?;
    require_identity_match(IdentityEquality {
        actual: &valence.receipt_hash_blake3,
        expected: expected_valence_hash,
        label: "Valence logical receipt hash",
    })?;
    let linked_preserves_hash = valence.kamacite_receipt_hash_blake3.as_deref().ok_or_else(|| {
        RunError::Internal("Valence receipt is missing the canonical Preserves receipt hash link".to_string())
    })?;
    require_identity_match(IdentityEquality {
        actual: linked_preserves_hash,
        expected: &binding.canonical_envelope.digest_blake3,
        label: "Valence-to-Preserves receipt hash",
    })?;
    validate_preserves_json_projection(root, receipt)
}

fn preserves_binding_receipt(
    manifest: &ReleaseEvidenceManifest,
) -> Result<&OpaqueEvidenceSidecarBindingReceipt, RunError> {
    let mut candidates = manifest.opaque_evidence_sidecar_bindings.iter().filter(|receipt| {
        receipt.binding.evidence_kind == OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS
            && receipt.binding.profile_version == FUNCTION_ADDRESS_PRESERVES_PROFILE_VERSION
    });
    let Some(receipt) = candidates.next() else {
        return Err(RunError::Internal("release manifest has no Preserves function-address binding".to_string()));
    };
    if candidates.next().is_some() {
        return Err(RunError::Internal(
            "release manifest has multiple Preserves function-address bindings".to_string(),
        ));
    }
    Ok(receipt)
}

fn validate_preserves_json_projection(
    root: &ReleaseCapabilityRoot,
    receipt: &OpaqueEvidenceSidecarBindingReceipt,
) -> Result<(), RunError> {
    let canonical_digest = &receipt.binding.canonical_envelope.digest_blake3;
    if canonical_digest.is_empty() {
        return Err(RunError::Internal("Kamacite canonical Preserves receipt digest is empty".to_string()));
    }
    if receipt
        .binding
        .compatibility_projections
        .iter()
        .any(|projection| projection.relative_path.is_empty())
    {
        return Err(RunError::Internal("Kamacite JSON projection path is empty".to_string()));
    }
    debug_assert!(!canonical_digest.is_empty());
    debug_assert!(
        receipt
            .binding
            .compatibility_projections
            .iter()
            .all(|projection| !projection.relative_path.is_empty())
    );
    for projection in &receipt.binding.compatibility_projections {
        let identity = read_kamacite_identity(ReceiptReadRequest {
            root,
            relative_path: &projection.relative_path,
            expected_digest_blake3: &projection.digest_blake3,
            label: "Kamacite receipt",
        })?;
        require_identity_match(IdentityEquality {
            actual: &identity.schema_version,
            expected: FUNCTION_ADDRESS_KAMACITE_RECEIPT_IDENTITY_SCHEMA,
            label: "Kamacite JSON projection schema",
        })?;
        require_identity_match(IdentityEquality {
            actual: &identity.receipt_hash_blake3,
            expected: canonical_digest,
            label: "Kamacite JSON projection canonical receipt hash",
        })?;
    }
    Ok(())
}

fn require_identity_match(fields: IdentityEquality<'_>) -> Result<(), RunError> {
    if fields.actual == fields.expected {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "function-address {} mismatch: expected {}, got {}",
        fields.label, fields.expected, fields.actual
    )))
}

fn binding_selection(
    root: &ReleaseCapabilityRoot,
    command: &FunctionAddressBindingCommand,
    paths: &ExplicitBindingPaths<'_>,
    declared_digests: &DeclaredReceiptDigests,
) -> Result<FunctionAddressBindingSelection, RunError> {
    debug_assert_eq!(command.kamacite_receipt_relative_path.is_some(), declared_digests.kamacite.is_some());
    let valence = read_valence_identity(ReceiptReadRequest {
        root,
        relative_path: paths.valence,
        expected_digest_blake3: &declared_digests.valence,
        label: "Valence receipt",
    })?;
    let kamacite = command
        .kamacite_receipt_relative_path
        .as_deref()
        .zip(declared_digests.kamacite.as_deref())
        .map(|(path, digest)| {
            read_kamacite_identity(ReceiptReadRequest {
                root,
                relative_path: path,
                expected_digest_blake3: digest,
                label: "Kamacite receipt",
            })
        })
        .transpose()?;
    Ok(FunctionAddressBindingSelection {
        mode: command.mode.clone(),
        sidecar_relative_path: paths.sidecar.to_string(),
        valence_receipt_relative_path: paths.valence.to_string(),
        kamacite_receipt_relative_path: command.kamacite_receipt_relative_path.clone(),
        release_binary_relative_path: command.release_binary_relative_path.clone(),
        valence_receipt_identity: valence,
        kamacite_receipt_identity: kamacite,
    })
}

fn read_valence_identity(request: ReceiptReadRequest<'_>) -> Result<FunctionAddressValenceReceiptIdentity, RunError> {
    let bytes = read_bounded_receipt(request)?;
    let envelope: ValenceReceiptIdentityEnvelope = serde_json::from_slice(&bytes)
        .map_err(|error| RunError::Internal(format!("parsing Valence function-address receipt identity: {error}")))?;
    Ok(FunctionAddressValenceReceiptIdentity {
        schema_version: envelope.schema_version,
        receipt_hash_blake3: envelope.receipt_hash,
        kamacite_receipt_hash_blake3: envelope.kamacite_receipt_hash,
    })
}

fn read_kamacite_identity(request: ReceiptReadRequest<'_>) -> Result<FunctionAddressKamaciteReceiptIdentity, RunError> {
    let bytes = read_bounded_receipt(request)?;
    let envelope: KamaciteReceiptIdentityEnvelope = serde_json::from_slice(&bytes)
        .map_err(|error| RunError::Internal(format!("parsing Kamacite function-address receipt identity: {error}")))?;
    Ok(FunctionAddressKamaciteReceiptIdentity {
        schema_version: envelope.schema_version,
        receipt_hash_blake3: envelope.receipt_hash,
    })
}

fn read_bounded_receipt(request: ReceiptReadRequest<'_>) -> Result<Vec<u8>, RunError> {
    let validated = authorize_release_path(&ReleasePathRequest {
        required_root: ReleaseRootKind::ReleaseEvidence,
        available_root: Some(request.root.kind()),
        relative_path: request.relative_path.to_string(),
    })
    .map_err(|error| {
        RunError::Internal(format!("authorizing {} path {}: {error:?}", request.label, request.relative_path))
    })?;
    let file = request
        .root
        .open_file_read_nofollow(&validated)
        .map_err(|error| RunError::Internal(format!("opening {} {}: {error}", request.label, request.relative_path)))?;
    let mut bytes = Vec::with_capacity(MAX_FUNCTION_ADDRESS_UPSTREAM_RECEIPT_BYTES);
    file.take(MAX_FUNCTION_ADDRESS_UPSTREAM_RECEIPT_READ_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|error| RunError::Internal(format!("reading {} {}: {error}", request.label, request.relative_path)))?;
    if bytes.len() > MAX_FUNCTION_ADDRESS_UPSTREAM_RECEIPT_BYTES {
        return Err(RunError::Internal(format!(
            "{} exceeds {MAX_FUNCTION_ADDRESS_UPSTREAM_RECEIPT_BYTES} bytes: {}",
            request.label, request.relative_path
        )));
    }
    require_current_receipt_digest(&bytes, &request)?;
    debug_assert!(bytes.len() <= MAX_FUNCTION_ADDRESS_UPSTREAM_RECEIPT_BYTES);
    debug_assert!(receipt_digest_matches(&bytes, request.expected_digest_blake3));
    Ok(bytes)
}

// r[impl mantle.release_provenance.function_address_binding_cli.shell.replacement]
fn require_current_receipt_digest(bytes: &[u8], request: &ReceiptReadRequest<'_>) -> Result<(), RunError> {
    if receipt_digest_matches(bytes, request.expected_digest_blake3) {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "{} changed after release bundle verification: {}",
        request.label, request.relative_path
    )))
}

fn write_receipt_noclobber(path: &Path, bytes: &[u8]) -> Result<(), RunError> {
    debug_assert!(!FUNCTION_ADDRESS_PRESERVES_PROFILE_VERSION.is_empty());
    let parent = output_parent(path);
    prepare_output_parent(parent)?;
    let mut temporary = NamedTempFile::new_in(parent)
        .map_err(|error| RunError::Internal(format!("creating function-address receipt temporary file: {error}")))?;
    temporary
        .write_all(bytes)
        .map_err(|error| RunError::Internal(format!("writing function-address receipt temporary file: {error}")))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| RunError::Internal(format!("syncing function-address receipt temporary file: {error}")))?;
    temporary.persist_noclobber(path).map_err(|error| {
        RunError::Internal(format!(
            "committing function-address receipt {} without overwrite: {}",
            path.display(),
            error.error
        ))
    })?;
    debug_assert!(path.is_file());
    debug_assert!(!path.is_symlink());
    Ok(())
}

fn prepare_output_parent(parent: &Path) -> Result<(), RunError> {
    debug_assert!(!FUNCTION_ADDRESS_PRESERVES_PROFILE_VERSION.is_empty());
    match std::fs::symlink_metadata(parent) {
        Ok(metadata) if metadata.file_type().is_dir() => return Ok(()),
        Ok(_) => {
            return Err(RunError::Internal(format!(
                "function-address receipt parent is not a real directory: {}",
                parent.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(RunError::Internal(format!(
                "inspecting function-address receipt directory {}: {error}",
                parent.display()
            )));
        }
    }
    std::fs::create_dir_all(parent).map_err(|error| {
        RunError::Internal(format!("creating function-address receipt directory {}: {error}", parent.display()))
    })
}

fn output_parent(path: &Path) -> &Path {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    }
}

fn emit_binding_receipt(
    json: bool,
    receipt_path: &Path,
    receipt: &FunctionAddressBindingReceipt,
    canonical_bytes: &[u8],
) -> Result<(), RunError> {
    if json {
        let text = core::str::from_utf8(canonical_bytes)
            .map_err(|error| RunError::Internal(format!("rendering function-address receipt as UTF-8: {error}")))?;
        println!("{text}");
        return Ok(());
    }
    println!("function-address binding receipt: {}", receipt_path.display());
    println!("receipt hash: {}", receipt.receipt_hash);
    println!("verdict: {}", receipt.verdict);
    Ok(())
}

fn reject_invalid_binding(receipt: &FunctionAddressBindingReceipt) -> Result<(), RunError> {
    if receipt.valid {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "function-address binding rejected: {}",
        receipt.verification_summary.diagnostics.join("; ")
    )))
}

fn receipt_digest_matches(bytes: &[u8], expected_digest_blake3: &str) -> bool {
    let actual_digest_blake3 = blake3::hash(bytes).to_hex();
    actual_digest_blake3.as_str() == expected_digest_blake3
}

fn resolve_operator_path(current_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURRENT_RECEIPT_BYTES: &[u8] = br#"{"receipt_hash":"current"}"#;
    const STALE_RECEIPT_BYTES: &[u8] = br#"{"receipt_hash":"stale"}"#;
    const RECEIPT_RELATIVE_PATH: &str = "valence-receipt.json";

    #[test]
    fn current_receipt_bytes_match_manifest_digest() {
        let expected = blake3::hash(CURRENT_RECEIPT_BYTES).to_hex();
        assert!(receipt_digest_matches(CURRENT_RECEIPT_BYTES, expected.as_str()));
        assert!(!expected.as_str().is_empty());
    }

    #[test]
    fn changed_receipt_bytes_do_not_match_manifest_digest() {
        let expected = blake3::hash(CURRENT_RECEIPT_BYTES).to_hex();
        assert!(!receipt_digest_matches(STALE_RECEIPT_BYTES, expected.as_str()));
        assert_ne!(CURRENT_RECEIPT_BYTES, STALE_RECEIPT_BYTES);
    }

    #[cfg(unix)]
    #[test]
    fn bounded_receipt_read_accepts_current_verified_bytes() {
        let temp = tempfile::tempdir().expect("temporary release root");
        std::fs::write(temp.path().join(RECEIPT_RELATIVE_PATH), CURRENT_RECEIPT_BYTES).expect("write current receipt");
        let root = ReleaseCapabilityRoot::open_ambient_nofollow(ReleaseRootKind::ReleaseEvidence, temp.path())
            .expect("open release root");
        let expected = blake3::hash(CURRENT_RECEIPT_BYTES).to_hex();

        let bytes = read_bounded_receipt(ReceiptReadRequest {
            root: &root,
            relative_path: RECEIPT_RELATIVE_PATH,
            expected_digest_blake3: expected.as_str(),
            label: "Valence receipt",
        })
        .expect("current bytes remain bound to manifest digest");

        assert_eq!(bytes, CURRENT_RECEIPT_BYTES);
        assert!(!bytes.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn bounded_receipt_read_rejects_post_verification_replacement() {
        // r[verify mantle.release_provenance.function_address_binding_cli.shell.replacement]
        let temp = tempfile::tempdir().expect("temporary release root");
        std::fs::write(temp.path().join(RECEIPT_RELATIVE_PATH), STALE_RECEIPT_BYTES).expect("write replaced receipt");
        let root = ReleaseCapabilityRoot::open_ambient_nofollow(ReleaseRootKind::ReleaseEvidence, temp.path())
            .expect("open release root");
        let expected = blake3::hash(CURRENT_RECEIPT_BYTES).to_hex();

        let error = read_bounded_receipt(ReceiptReadRequest {
            root: &root,
            relative_path: RECEIPT_RELATIVE_PATH,
            expected_digest_blake3: expected.as_str(),
            label: "Valence receipt",
        })
        .expect_err("replacement bytes must fail before parsing");

        assert!(error.to_string().contains("changed after release bundle verification"));
        assert!(!error.to_string().contains("accepted"));
    }
}
