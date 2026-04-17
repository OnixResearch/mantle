use std::path::Path;
use std::path::PathBuf;

use crate::errors::RunError;
use crate::release_evidence::DEFAULT_PROOF_WORKFLOW_COMMAND;
use crate::release_evidence::DEFAULT_PROOF_WORKFLOW_VERSION;
use crate::release_evidence::ReleaseBundleCreateRequest;
use crate::release_evidence::create_release_evidence_bundle;
use crate::release_evidence::verify_release_evidence_bundle;
use crate::release_source::write_tracked_source_archive;

pub(crate) fn cmd_release(action: crate::ReleaseAction, current_dir: &Path, json: bool) -> Result<(), RunError> {
    match action {
        crate::ReleaseAction::Create {
            release_id,
            bundle_dir,
            binary,
            proof_bundle,
            workflow_command,
            workflow_version,
        } => cmd_release_create(
            current_dir,
            json,
            release_id,
            bundle_dir,
            binary,
            proof_bundle,
            workflow_command,
            workflow_version,
        ),
        crate::ReleaseAction::Verify { bundle_dir } => cmd_release_verify(json, bundle_dir),
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_release_create(
    current_dir: &Path,
    json: bool,
    release_id: String,
    bundle_dir: Option<PathBuf>,
    binary: Vec<PathBuf>,
    proof_bundle: PathBuf,
    workflow_command: String,
    workflow_version: String,
) -> Result<(), RunError> {
    let resolved_bundle_dir = resolve_bundle_dir(current_dir, &release_id, bundle_dir);
    let normalized_workflow_command = normalize_workflow_value(&workflow_command, DEFAULT_PROOF_WORKFLOW_COMMAND);
    let normalized_workflow_version = normalize_workflow_value(&workflow_version, DEFAULT_PROOF_WORKFLOW_VERSION);
    let source_archive_file = tempfile::NamedTempFile::new()
        .map_err(|err| RunError::Internal(format!("creating temp source archive file: {err}")))?;
    write_tracked_source_archive(current_dir, source_archive_file.path())?;
    let request = ReleaseBundleCreateRequest {
        release_id,
        bundle_dir: resolved_bundle_dir.clone(),
        source_archive_path: source_archive_file.path().to_path_buf(),
        binary_paths: resolve_input_paths(current_dir, binary),
        proof_bundle_dir: resolve_input_path(current_dir, proof_bundle),
        workflow_command: normalized_workflow_command,
        workflow_version: normalized_workflow_version,
    };
    let manifest = create_release_evidence_bundle(&request)?;
    if json {
        let rendered = serde_json::to_string(&manifest)
            .map_err(|err| RunError::Internal(format!("serializing release evidence manifest: {err}")))?;
        println!("{rendered}");
    } else {
        println!("release evidence bundle: {}", request.bundle_dir.display());
        println!("release id: {}", manifest.release_id);
        println!("source archive: {}", manifest.source_archive.relative_path);
        println!("proof bundle: {}", manifest.proof_bundle.relative_path);
        println!("binaries: {}", manifest.binaries.len());
        println!("manifest: manifest.json");
    }
    Ok(())
}

fn cmd_release_verify(json: bool, bundle_dir: PathBuf) -> Result<(), RunError> {
    let manifest = verify_release_evidence_bundle(&bundle_dir)?;
    if json {
        let rendered = serde_json::to_string(&manifest)
            .map_err(|err| RunError::Internal(format!("serializing verified release evidence manifest: {err}")))?;
        println!("{rendered}");
    } else {
        println!("release evidence verified: {}", bundle_dir.display());
        println!("release id: {}", manifest.release_id);
        println!("binaries: {}", manifest.binaries.len());
        println!("proof mode: {}", manifest.proof_linkage.proof_mode);
    }
    Ok(())
}

fn resolve_bundle_dir(current_dir: &Path, release_id: &str, bundle_dir: Option<PathBuf>) -> PathBuf {
    match bundle_dir {
        Some(path) => resolve_input_path(current_dir, path),
        None => current_dir.join("target").join("release-evidence").join(release_id),
    }
}

fn resolve_input_paths(current_dir: &Path, paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.into_iter().map(|path| resolve_input_path(current_dir, path)).collect()
}

fn resolve_input_path(current_dir: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        current_dir.join(path)
    }
}

fn normalize_workflow_value(value: &str, default_value: &str) -> String {
    if value.trim().is_empty() {
        default_value.to_string()
    } else {
        value.to_string()
    }
}
