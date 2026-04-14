use std::path::Path;
use std::path::PathBuf;

use crunch_attestation::ArtifactAttestation;
use crunch_attestation::ArtifactReference;
use crunch_attestation::Canonicalize;
use crunch_attestation::ClosureAttestation;
use crunch_attestation::ClosureSemantics;
use crunch_attestation::ProjectAttestation;
use crunch_project::Lockfile;
use crunch_project::ProjectManifest;
use crunch_project::synthesize_project_attestation;
use crunch_store::StoreConfig;
use crunch_store::StoreHandle;
use crunch_store::artifact_attestation_file_path;
use crunch_store::closure_attestation_file_path;
use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::errors::RunError;

const MANIFEST_FILE: &str = "crunch-project.ncl";
const LOCK_FILE: &str = "crunch.lock";

#[derive(Debug)]
enum AttestationDocument {
    Artifact(ArtifactAttestation),
    Closure(ClosureAttestation),
    Project(ProjectAttestation),
}

#[derive(Debug, Deserialize)]
struct AttestationEnvelopeInput {
    kind: String,
    #[serde(default)]
    digest: Option<String>,
    attestation: Value,
}

#[derive(Debug, Serialize)]
struct AttestationEnvelopeOutput {
    kind: &'static str,
    digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stored_path: Option<String>,
    attestation: Value,
}

pub fn cmd_attest(
    action: crate::AttestAction,
    current_dir: &Path,
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
) -> Result<(), RunError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;
    rt.block_on(cmd_attest_async(action, current_dir, output_dir, state_dir, store_dir))
}

async fn cmd_attest_async(
    action: crate::AttestAction,
    current_dir: &Path,
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
) -> Result<(), RunError> {
    match action {
        crate::AttestAction::Show { path } => {
            let store = open_store(output_dir, state_dir, store_dir).await?;
            let (document, stored_path) = load_artifact_document(&store, &path).await?;
            print_document(&document, Some(stored_path))
        }
        crate::AttestAction::Closure { roots } => {
            let store = open_store(output_dir, state_dir, store_dir).await?;
            let root_paths = resolve_roots(&roots, store_dir, output_dir)?;
            let stored = store
                .runtime_closure_attestation(&root_paths)
                .await
                .map_err(|e| RunError::Internal(format!("loading closure attestation: {e}")))?;
            let path = closure_attestation_file_path(
                store.state_dir(),
                store.store_dir(),
                &root_paths,
                ClosureSemantics::Runtime,
            );
            print_document(&AttestationDocument::Closure(stored.attestation), Some(path))
        }
        crate::AttestAction::Verify { target } => {
            cmd_verify(target, current_dir, output_dir, state_dir, store_dir).await
        }
        crate::AttestAction::Diff { left, right } => {
            let store = open_store(output_dir, state_dir, store_dir).await?;
            let left_document = load_document_input(Some(&store), &left).await?;
            let right_document = load_document_input(Some(&store), &right).await?;
            print_diff(&left, &left_document, &right, &right_document)
        }
        crate::AttestAction::Project { roots } => {
            let store = open_store(output_dir, state_dir, store_dir).await?;
            let document = load_project_document(current_dir, &store, &roots).await?;
            print_document(&document, None)
        }
    }
}

async fn cmd_verify(
    target: crate::AttestVerifyAction,
    current_dir: &Path,
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
) -> Result<(), RunError> {
    match target {
        crate::AttestVerifyAction::Artifact { path } => {
            let store = open_store(output_dir, state_dir, store_dir).await?;
            let (document, stored_path) = load_artifact_document(&store, &path).await?;
            verify_persisted_document(&document, &stored_path)
        }
        crate::AttestVerifyAction::Closure { roots } => {
            let store = open_store(output_dir, state_dir, store_dir).await?;
            let root_paths = resolve_roots(&roots, store_dir, output_dir)?;
            let stored = store
                .runtime_closure_attestation(&root_paths)
                .await
                .map_err(|e| RunError::Internal(format!("loading closure attestation: {e}")))?;
            let path = closure_attestation_file_path(
                store.state_dir(),
                store.store_dir(),
                &root_paths,
                ClosureSemantics::Runtime,
            );
            verify_persisted_document(&AttestationDocument::Closure(stored.attestation), &path)
        }
        crate::AttestVerifyAction::Project { file, digest, roots } => {
            let store = open_store(output_dir, state_dir, store_dir).await?;
            let document = load_project_document(current_dir, &store, &roots).await?;
            verify_project_document(&document, file.as_deref(), digest.as_deref())
        }
    }
}

async fn open_store(output_dir: &Path, state_dir: &Path, store_dir: &str) -> Result<StoreHandle, RunError> {
    let config = StoreConfig {
        state_dir: state_dir.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        remote_cache_url: None,
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: store_dir.to_string(),
    };
    StoreHandle::open(config).await.map_err(|e| RunError::Internal(format!("opening store: {e}")))
}

async fn load_artifact_document(
    store: &StoreHandle,
    selector: &str,
) -> Result<(AttestationDocument, PathBuf), RunError> {
    let store_path = resolve_store_path(selector, store.store_dir(), store.output_dir_str())?;
    let stored = store
        .get_artifact_attestation(&store_path)
        .await
        .map_err(|e| RunError::Internal(format!("loading artifact attestation: {e}")))?
        .ok_or_else(|| RunError::Internal(format!("no artifact attestation for {}", selector)))?;
    let stored_path = artifact_attestation_file_path(store.state_dir(), store.store_dir(), &store_path);
    Ok((AttestationDocument::Artifact(stored.attestation), stored_path))
}

async fn load_project_document(
    current_dir: &Path,
    store: &StoreHandle,
    roots: &[String],
) -> Result<AttestationDocument, RunError> {
    let (manifest_text, manifest) = load_manifest(current_dir)?;
    let (lock_text, lock) = load_lockfile(current_dir)?;
    let selected_roots = load_selected_roots(store, roots).await?;
    let attestation = synthesize_project_attestation(&manifest_text, &lock_text, &manifest, &lock, &selected_roots)
        .map_err(|e| RunError::Internal(format!("project attestation: {e}")))?;
    Ok(AttestationDocument::Project(attestation))
}

async fn load_selected_roots(store: &StoreHandle, roots: &[String]) -> Result<Vec<ArtifactReference>, RunError> {
    if roots.is_empty() {
        return Err(RunError::Internal("provide at least one root path".to_string()));
    }

    let mut selected = Vec::new();
    for root in roots {
        let store_path = resolve_store_path(root, store.store_dir(), store.output_dir_str())?;
        let stored = store
            .get_artifact_attestation(&store_path)
            .await
            .map_err(|e| RunError::Internal(format!("loading artifact attestation: {e}")))?
            .ok_or_else(|| RunError::Internal(format!("no artifact attestation for root {}", root)))?;
        selected.push(ArtifactReference {
            node_id: stored.attestation.subject_node_id.clone(),
            logical_path: stored.attestation.facts.logical_path.clone(),
            attestation_digest: stored.digest,
        });
    }
    Ok(selected)
}

fn load_manifest(current_dir: &Path) -> Result<(String, ProjectManifest), RunError> {
    let path = current_dir.join(MANIFEST_FILE);
    let text =
        std::fs::read_to_string(&path).map_err(|e| RunError::Internal(format!("reading {}: {e}", path.display())))?;
    let import_paths = vec![current_dir.as_os_str().to_owned()];
    let manifest = crunch_eval::evaluate_and_deserialize(&path, &import_paths)
        .map_err(|e| RunError::Eval(format!("loading {MANIFEST_FILE}: {e}")))?;
    Ok((text, manifest))
}

fn load_lockfile(current_dir: &Path) -> Result<(String, Lockfile), RunError> {
    let path = current_dir.join(LOCK_FILE);
    let text =
        std::fs::read_to_string(&path).map_err(|e| RunError::Internal(format!("reading {}: {e}", path.display())))?;
    let lock = Lockfile::from_json(&text).map_err(|e| RunError::Internal(format!("parsing {LOCK_FILE}: {e}")))?;
    Ok((text, lock))
}

fn resolve_roots(roots: &[String], store_dir: &str, output_dir: &Path) -> Result<Vec<StorePath<String>>, RunError> {
    if roots.is_empty() {
        return Err(RunError::Internal("provide at least one root path".to_string()));
    }

    roots
        .iter()
        .map(|root| resolve_store_path(root, store_dir, &output_dir.display().to_string()))
        .collect()
}

fn resolve_store_path(selector: &str, store_dir: &str, output_dir: &str) -> Result<StorePath<String>, RunError> {
    if let Ok(store_path) = StorePath::from_absolute_path_with_prefix(selector.as_bytes(), store_dir) {
        return Ok(store_path);
    }

    if let Ok(store_path) = StorePath::from_absolute_path_with_prefix(selector.as_bytes(), output_dir) {
        return Ok(store_path);
    }

    if let Ok(store_path) = StorePath::from_absolute_path(selector.as_bytes()) {
        return Ok(store_path);
    }

    if let Ok(store_path) = StorePath::from_bytes(selector.as_bytes()) {
        return Ok(store_path);
    }

    Err(RunError::Internal(format!(
        "could not parse '{selector}' as a store path; use a logical path under {store_dir} or an exported path under {output_dir}"
    )))
}

fn print_document(document: &AttestationDocument, stored_path: Option<PathBuf>) -> Result<(), RunError> {
    let text = render_document(document, stored_path.as_deref())?;
    println!("{text}");
    Ok(())
}

fn verify_persisted_document(document: &AttestationDocument, stored_path: &Path) -> Result<(), RunError> {
    let canonical_bytes = canonical_document_bytes(document)?;
    let stored_bytes = std::fs::read(stored_path)
        .map_err(|e| RunError::Internal(format!("reading {}: {e}", stored_path.display())))?;
    if stored_bytes != canonical_bytes {
        return Err(RunError::Build(format!(
            "attestation bytes differ from canonical form: {}",
            stored_path.display()
        )));
    }

    let digest = document_digest_hex(document)?;
    println!("OK {} digest={} path={}", document.kind(), digest, stored_path.display());
    Ok(())
}

fn verify_project_document(
    document: &AttestationDocument,
    expected_file: Option<&Path>,
    expected_digest: Option<&str>,
) -> Result<(), RunError> {
    if let Some(path) = expected_file {
        let (expected_document, envelope_digest) = load_document_file_with_digest(path)?;
        if expected_document.kind() != document.kind() {
            return Err(RunError::Build(format!(
                "expected {} attestation in {}, found {}",
                document.kind(),
                path.display(),
                expected_document.kind()
            )));
        }

        let expected_canonical = canonical_document_bytes(&expected_document)?;
        let actual_canonical = canonical_document_bytes(document)?;
        if expected_canonical != actual_canonical {
            return Err(RunError::Build(format!(
                "saved project attestation does not match canonical reconstruction: {}",
                path.display()
            )));
        }

        let actual_digest = document_digest_hex(document)?;
        if let Some(envelope_digest) = envelope_digest {
            if envelope_digest != actual_digest {
                return Err(RunError::Build(format!(
                    "saved project attestation digest mismatch: expected {}, got {}",
                    envelope_digest, actual_digest
                )));
            }
        }

        println!("OK {} digest={} file={}", document.kind(), actual_digest, path.display());
        return Ok(());
    }

    if let Some(expected_digest) = expected_digest {
        let actual_digest = document_digest_hex(document)?;
        if expected_digest != actual_digest {
            return Err(RunError::Build(format!(
                "project attestation digest mismatch: expected {}, got {}",
                expected_digest, actual_digest
            )));
        }
        println!("OK {} digest={}", document.kind(), actual_digest);
        return Ok(());
    }

    Err(RunError::Internal(
        "project verification requires --file <saved-attestation.json> or --digest <hex>".to_string(),
    ))
}

async fn load_document_input(store: Option<&StoreHandle>, input: &str) -> Result<AttestationDocument, RunError> {
    if let Some(store) = store {
        if let Ok((document, _stored_path)) = load_artifact_document(store, input).await {
            return Ok(document);
        }
    }

    let path = Path::new(input);
    if path.exists() {
        let (document, _digest) = load_document_file_with_digest(path)?;
        return Ok(document);
    }

    Err(RunError::Internal(format!(
        "input '{}' is neither an attestation file nor a known artifact selector",
        input
    )))
}

fn load_document_file_with_digest(path: &Path) -> Result<(AttestationDocument, Option<String>), RunError> {
    let text =
        std::fs::read_to_string(path).map_err(|e| RunError::Internal(format!("reading {}: {e}", path.display())))?;
    parse_document_text(&text)
}

fn parse_document_text(text: &str) -> Result<(AttestationDocument, Option<String>), RunError> {
    if let Ok(envelope) = serde_json::from_str::<AttestationEnvelopeInput>(text) {
        let document = parse_document_value(&envelope.kind, envelope.attestation)?;
        return Ok((document, envelope.digest));
    }

    if let Ok(value) = serde_json::from_str::<ArtifactAttestation>(text) {
        return Ok((AttestationDocument::Artifact(value), None));
    }
    if let Ok(value) = serde_json::from_str::<ClosureAttestation>(text) {
        return Ok((AttestationDocument::Closure(value), None));
    }
    if let Ok(value) = serde_json::from_str::<ProjectAttestation>(text) {
        return Ok((AttestationDocument::Project(value), None));
    }

    Err(RunError::Internal("input is not a supported attestation document".to_string()))
}

fn parse_document_value(kind: &str, value: Value) -> Result<AttestationDocument, RunError> {
    match kind {
        "artifact" => serde_json::from_value::<ArtifactAttestation>(value)
            .map(AttestationDocument::Artifact)
            .map_err(|e| RunError::Internal(format!("parsing artifact attestation: {e}"))),
        "closure" => serde_json::from_value::<ClosureAttestation>(value)
            .map(AttestationDocument::Closure)
            .map_err(|e| RunError::Internal(format!("parsing closure attestation: {e}"))),
        "project" => serde_json::from_value::<ProjectAttestation>(value)
            .map(AttestationDocument::Project)
            .map_err(|e| RunError::Internal(format!("parsing project attestation: {e}"))),
        other => Err(RunError::Internal(format!("unsupported attestation kind '{other}'"))),
    }
}

fn render_document(document: &AttestationDocument, stored_path: Option<&Path>) -> Result<String, RunError> {
    let attestation_value = document_value(document)?;
    let envelope = AttestationEnvelopeOutput {
        kind: document.kind(),
        digest: document_digest_hex(document)?,
        stored_path: stored_path.map(|path| path.display().to_string()),
        attestation: attestation_value,
    };
    serde_json::to_string_pretty(&envelope)
        .map_err(|e| RunError::Internal(format!("serializing attestation output: {e}")))
}

fn document_value(document: &AttestationDocument) -> Result<Value, RunError> {
    match document {
        AttestationDocument::Artifact(value) => {
            serde_json::to_value(value).map_err(|e| RunError::Internal(format!("serializing artifact: {e}")))
        }
        AttestationDocument::Closure(value) => {
            serde_json::to_value(value).map_err(|e| RunError::Internal(format!("serializing closure: {e}")))
        }
        AttestationDocument::Project(value) => {
            serde_json::to_value(value).map_err(|e| RunError::Internal(format!("serializing project: {e}")))
        }
    }
}

fn canonical_document_bytes(document: &AttestationDocument) -> Result<Vec<u8>, RunError> {
    match document {
        AttestationDocument::Artifact(value) => {
            value.canonical_bytes().map_err(|e| RunError::Internal(format!("artifact canonicalization: {e}")))
        }
        AttestationDocument::Closure(value) => {
            value.canonical_bytes().map_err(|e| RunError::Internal(format!("closure canonicalization: {e}")))
        }
        AttestationDocument::Project(value) => {
            value.canonical_bytes().map_err(|e| RunError::Internal(format!("project canonicalization: {e}")))
        }
    }
}

fn document_digest_hex(document: &AttestationDocument) -> Result<String, RunError> {
    match document {
        AttestationDocument::Artifact(value) => value
            .canonical_digest()
            .map(|digest| digest.to_hex())
            .map_err(|e| RunError::Internal(format!("artifact digest: {e}"))),
        AttestationDocument::Closure(value) => value
            .canonical_digest()
            .map(|digest| digest.to_hex())
            .map_err(|e| RunError::Internal(format!("closure digest: {e}"))),
        AttestationDocument::Project(value) => value
            .canonical_digest()
            .map(|digest| digest.to_hex())
            .map_err(|e| RunError::Internal(format!("project digest: {e}"))),
    }
}

fn print_diff(
    left_label: &str,
    left_document: &AttestationDocument,
    right_label: &str,
    right_document: &AttestationDocument,
) -> Result<(), RunError> {
    let left_text = canonical_pretty_json(left_document)?;
    let right_text = canonical_pretty_json(right_document)?;
    if left_text == right_text {
        println!("no differences");
        return Ok(());
    }

    println!("--- {left_label}");
    println!("+++ {right_label}");
    for line in render_line_diff(&left_text, &right_text) {
        println!("{line}");
    }
    Ok(())
}

fn canonical_pretty_json(document: &AttestationDocument) -> Result<String, RunError> {
    let bytes = canonical_document_bytes(document)?;
    let value: Value =
        serde_json::from_slice(&bytes).map_err(|e| RunError::Internal(format!("parsing canonical json: {e}")))?;
    serde_json::to_string_pretty(&value).map_err(|e| RunError::Internal(format!("formatting canonical json: {e}")))
}

fn render_line_diff(left: &str, right: &str) -> Vec<String> {
    let left_lines: Vec<&str> = left.lines().collect();
    let right_lines: Vec<&str> = right.lines().collect();
    let max_lines = left_lines.len().max(right_lines.len());
    let mut diff = Vec::new();

    for index in 0..max_lines {
        let left_line = left_lines.get(index).copied();
        let right_line = right_lines.get(index).copied();
        if left_line == right_line {
            continue;
        }
        if let Some(line) = left_line {
            diff.push(format!("-{line}"));
        }
        if let Some(line) = right_line {
            diff.push(format!("+{line}"));
        }
    }

    diff
}

impl AttestationDocument {
    fn kind(&self) -> &'static str {
        match self {
            Self::Artifact(_) => "artifact",
            Self::Closure(_) => "closure",
            Self::Project(_) => "project",
        }
    }
}
