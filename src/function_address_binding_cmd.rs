use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crunch_release_core::FunctionAddressBindingReceipt;
use crunch_release_core::FunctionAddressBindingSelection;
use crunch_release_core::FunctionAddressKamaciteReceiptIdentity;
use crunch_release_core::FunctionAddressValenceReceiptIdentity;
use crunch_release_core::ReleaseEvidenceManifest;
use crunch_release_core::function_address_binding_receipt_canonical_bytes;
use crunch_release_core::render_function_address_binding_from_manifest;
use serde::Deserialize;
use tempfile::NamedTempFile;

use crate::errors::RunError;
use crate::release_capability::ReleaseCapabilityRoot;
use crate::release_capability::ReleasePathRequest;
use crate::release_capability::ReleaseRootKind;
use crate::release_capability::authorize_release_path;
use crate::release_evidence::verify_release_evidence_bundle;

const MAX_FUNCTION_ADDRESS_UPSTREAM_RECEIPT_BYTES: usize = 1_048_576;
const MAX_FUNCTION_ADDRESS_UPSTREAM_RECEIPT_READ_BYTES: u64 = 1_048_577;

#[derive(Debug)]
pub(crate) struct FunctionAddressBindingCommand {
    pub bundle_dir: PathBuf,
    pub mode: String,
    pub sidecar_relative_path: String,
    pub valence_receipt_relative_path: String,
    pub kamacite_receipt_relative_path: Option<String>,
    pub release_binary_relative_path: Option<String>,
    pub receipt_out: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ValenceReceiptIdentityEnvelope {
    schema_version: String,
    receipt_hash: String,
    #[serde(default)]
    kamacite_receipt_hash: Option<String>,
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

// r[impl mantle.release_provenance.function_address_binding_cli.command]
// r[impl mantle.release_provenance.function_address_binding_cli.shell]
pub(crate) fn cmd_function_address_binding(
    current_dir: &Path,
    json: bool,
    command: FunctionAddressBindingCommand,
) -> Result<(), RunError> {
    debug_assert!(!command.mode.is_empty());
    debug_assert!(!command.receipt_out.as_os_str().is_empty());
    let bundle_dir = resolve_operator_path(current_dir, &command.bundle_dir);
    let receipt_out = resolve_operator_path(current_dir, &command.receipt_out);
    let manifest = verify_release_evidence_bundle(&bundle_dir)?;
    let declared_digests = preflight_declared_receipt_paths(&manifest, &command)?;
    let root = ReleaseCapabilityRoot::open_ambient_nofollow(ReleaseRootKind::ReleaseEvidence, &bundle_dir).map_err(
        |error| RunError::Internal(format!("opening release evidence root {}: {error}", bundle_dir.display())),
    )?;
    let selection = binding_selection(&root, command, &declared_digests)?;
    let receipt = render_function_address_binding_from_manifest(manifest, selection)
        .map_err(|error| RunError::Internal(error.to_string()))?;
    let bytes = function_address_binding_receipt_canonical_bytes(receipt.clone())
        .map_err(|error| RunError::Internal(error.to_string()))?;
    write_receipt_noclobber(&receipt_out, &bytes)?;
    emit_binding_receipt(json, &receipt_out, &receipt, &bytes)?;
    reject_invalid_binding(&receipt)
}

fn preflight_declared_receipt_paths(
    manifest: &ReleaseEvidenceManifest,
    command: &FunctionAddressBindingCommand,
) -> Result<DeclaredReceiptDigests, RunError> {
    let valence = declared_external_digest(DeclaredExternalPath {
        manifest,
        relative_path: &command.valence_receipt_relative_path,
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

fn binding_selection(
    root: &ReleaseCapabilityRoot,
    command: FunctionAddressBindingCommand,
    declared_digests: &DeclaredReceiptDigests,
) -> Result<FunctionAddressBindingSelection, RunError> {
    debug_assert_eq!(command.kamacite_receipt_relative_path.is_some(), declared_digests.kamacite.is_some());
    let valence = read_valence_identity(root, &command.valence_receipt_relative_path, &declared_digests.valence)?;
    let kamacite = command
        .kamacite_receipt_relative_path
        .as_deref()
        .zip(declared_digests.kamacite.as_deref())
        .map(|(path, digest)| read_kamacite_identity(root, path, digest))
        .transpose()?;
    Ok(FunctionAddressBindingSelection {
        mode: command.mode,
        sidecar_relative_path: command.sidecar_relative_path,
        valence_receipt_relative_path: command.valence_receipt_relative_path,
        kamacite_receipt_relative_path: command.kamacite_receipt_relative_path,
        release_binary_relative_path: command.release_binary_relative_path,
        valence_receipt_identity: valence,
        kamacite_receipt_identity: kamacite,
    })
}

fn read_valence_identity(
    root: &ReleaseCapabilityRoot,
    relative_path: &str,
    expected_digest_blake3: &str,
) -> Result<FunctionAddressValenceReceiptIdentity, RunError> {
    let bytes = read_bounded_receipt(ReceiptReadRequest {
        root,
        relative_path,
        expected_digest_blake3,
        label: "Valence receipt",
    })?;
    let envelope: ValenceReceiptIdentityEnvelope = serde_json::from_slice(&bytes)
        .map_err(|error| RunError::Internal(format!("parsing Valence function-address receipt identity: {error}")))?;
    Ok(FunctionAddressValenceReceiptIdentity {
        schema_version: envelope.schema_version,
        receipt_hash_blake3: envelope.receipt_hash,
        kamacite_receipt_hash_blake3: envelope.kamacite_receipt_hash,
    })
}

fn read_kamacite_identity(
    root: &ReleaseCapabilityRoot,
    relative_path: &str,
    expected_digest_blake3: &str,
) -> Result<FunctionAddressKamaciteReceiptIdentity, RunError> {
    let bytes = read_bounded_receipt(ReceiptReadRequest {
        root,
        relative_path,
        expected_digest_blake3,
        label: "Kamacite receipt",
    })?;
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
    Ok(())
}

fn prepare_output_parent(parent: &Path) -> Result<(), RunError> {
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
