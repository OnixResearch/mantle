use std::path::Path;
use std::path::PathBuf;

use crunch_attestation::ArtifactAttestation;
use crunch_attestation::ArtifactReference;
use crunch_attestation::Canonicalize;
use crunch_attestation::ClosureAttestation;
use crunch_attestation::ClosureSemantics;
use crunch_attestation::ProjectAttestation;
use crunch_attestation::ReleaseAttestation;
use crunch_attestation::WitnessAttestation;
use crunch_project::Lockfile;
use crunch_project::ProjectAttestationInput;
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

use crate::build_cmd::load_configured_trusted_public_keys;
use crate::build_cmd::load_existing_signing_keypair;
use crate::errors::RunError;
use crate::release_attestation::CreatedPolicyFiles;
use crate::release_attestation::CreatedWitnessAttestation;
use crate::release_attestation::PolicyInitProfile;
use crate::release_attestation::ReleaseVerificationOutput;
use crate::release_attestation::create_policy_files;
use crate::release_attestation::create_witness_attestation;
use crate::release_attestation::load_release_attestation_document;
use crate::release_attestation::load_witness_documents;
use crate::release_attestation::verify_release_attestation_directory;

const MANIFEST_FILE: &str = "crunch-project.ncl";
const LOCK_FILE: &str = "crunch.lock";

#[derive(Debug)]
enum AttestationDocument {
    Artifact(ArtifactAttestation),
    Closure(ClosureAttestation),
    Project(ProjectAttestation),
    Release(ReleaseAttestation),
    Witness(WitnessAttestation),
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
    json: bool,
) -> Result<(), RunError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;
    rt.block_on(cmd_attest_async(action, current_dir, output_dir, state_dir, store_dir, json))
}

async fn cmd_attest_async(
    action: crate::AttestAction,
    current_dir: &Path,
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    json: bool,
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
        crate::AttestAction::ReleaseShow { verification_dir } => {
            let (attestation, stored_path) = load_release_attestation_document(&verification_dir)?;
            print_document(&AttestationDocument::Release(attestation), Some(stored_path))
        }
        crate::AttestAction::KeyShow { signing_key } => {
            cmd_key_show(current_dir, state_dir, json, signing_key.as_deref())
        }
        crate::AttestAction::WitnessCreate {
            verification_dir,
            rebuilt_binary,
            identity,
            system,
            toolchain,
            host_class,
            signing_key,
        } => cmd_witness_create(
            current_dir,
            state_dir,
            json,
            &verification_dir,
            &rebuilt_binary,
            identity.as_deref(),
            &system,
            &toolchain,
            &host_class,
            signing_key.as_deref(),
        ),
        crate::AttestAction::WitnessShow {
            verification_dir,
            identity,
        } => cmd_witness_show(&verification_dir, identity.as_deref()),
        crate::AttestAction::PolicyInit {
            verification_dir,
            profile,
            trusted_release_signer,
            trusted_witness_identity,
            force,
        } => cmd_policy_init(
            current_dir,
            json,
            &verification_dir,
            map_policy_profile(profile),
            &trusted_release_signer,
            &trusted_witness_identity,
            force,
        ),
        crate::AttestAction::ReleaseVerify {
            verification_dir,
            trusted_public_keys,
        } => cmd_release_verify(&verification_dir, &trusted_public_keys, state_dir),
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

fn cmd_key_show(current_dir: &Path, state_dir: &Path, json: bool, signing_key: Option<&Path>) -> Result<(), RunError> {
    let resolved_signing_key = signing_key.map(|path| resolve_cli_path(current_dir, path));
    let (keypair, source_path) = load_existing_signing_keypair(resolved_signing_key.as_deref(), state_dir)?;
    print_trusted_public_key(&keypair.verifying_key.to_string(), keypair.verifying_key.name(), &source_path, json)
}

fn cmd_witness_create(
    current_dir: &Path,
    state_dir: &Path,
    json: bool,
    verification_dir: &Path,
    rebuilt_binary: &[PathBuf],
    identity: Option<&str>,
    system: &str,
    toolchain: &str,
    host_class: &str,
    signing_key: Option<&Path>,
) -> Result<(), RunError> {
    let resolved_verification_dir = resolve_cli_path(current_dir, verification_dir);
    let resolved_binaries = resolve_cli_paths(current_dir, rebuilt_binary);
    let created = create_witness_attestation(
        &resolved_verification_dir,
        &resolved_binaries,
        identity,
        system,
        toolchain,
        host_class,
        signing_key,
        state_dir,
    )?;
    print_created_witness_attestation(&created, json)
}

fn cmd_policy_init(
    current_dir: &Path,
    json: bool,
    verification_dir: &Path,
    profile: PolicyInitProfile,
    trusted_release_signers: &[String],
    trusted_witness_identities: &[String],
    force: bool,
) -> Result<(), RunError> {
    let resolved_verification_dir = resolve_cli_path(current_dir, verification_dir);
    let created = create_policy_files(
        &resolved_verification_dir,
        profile,
        trusted_release_signers,
        trusted_witness_identities,
        force,
    )?;
    print_created_policy_files(&created, json)
}

fn map_policy_profile(profile: crate::AttestPolicyProfileArg) -> PolicyInitProfile {
    match profile {
        crate::AttestPolicyProfileArg::SelfProofOnly => PolicyInitProfile::SelfProofOnly,
        crate::AttestPolicyProfileArg::SingleWitness => PolicyInitProfile::SingleWitness,
    }
}

fn cmd_witness_show(verification_dir: &Path, requested_identity: Option<&str>) -> Result<(), RunError> {
    let documents = load_witness_documents(verification_dir)?;
    if let Some(identity) = requested_identity {
        let document = documents
            .into_iter()
            .find(|document| document.attestation.witness_identity == identity)
            .ok_or_else(|| {
                RunError::Internal(format!(
                    "no witness attestation with identity '{}' in {}",
                    identity,
                    verification_dir.display()
                ))
            })?;
        return print_document(&AttestationDocument::Witness(document.attestation), Some(document.attestation_path));
    }

    let mut rendered = Vec::with_capacity(documents.len());
    for document in documents {
        let text = render_document(
            &AttestationDocument::Witness(document.attestation),
            Some(document.attestation_path.as_path()),
        )?;
        let envelope: Value = serde_json::from_str(&text)
            .map_err(|err| RunError::Internal(format!("parsing witness envelope json: {err}")))?;
        rendered.push(envelope);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&rendered)
            .map_err(|err| RunError::Internal(format!("serializing witness envelopes: {err}")))?
    );
    Ok(())
}

fn cmd_release_verify(
    verification_dir: &Path,
    explicit_trusted_public_keys: &[String],
    state_dir: &Path,
) -> Result<(), RunError> {
    let trusted_public_keys = resolve_release_verify_keys(explicit_trusted_public_keys, state_dir)?;
    let output = verify_release_attestation_directory(verification_dir, &trusted_public_keys)?;
    print_release_verification_output(&output)
}

fn resolve_release_verify_keys(
    explicit_trusted_public_keys: &[String],
    state_dir: &Path,
) -> Result<Vec<nix_compat::narinfo::VerifyingKey>, RunError> {
    let parsed_explicit_keys = parse_release_trusted_public_keys(explicit_trusted_public_keys)?;
    let configured = load_configured_trusted_public_keys(parsed_explicit_keys.as_deref(), state_dir)?;
    configured.ok_or_else(|| {
        RunError::Internal(
            "release verification requires trusted public keys via --trusted-public-key or configured trusted-public-keys"
                .to_string(),
        )
    })
}

fn resolve_cli_paths(current_dir: &Path, paths: &[PathBuf]) -> Vec<PathBuf> {
    paths.iter().map(|path| resolve_cli_path(current_dir, path.as_path())).collect()
}

fn resolve_cli_path(current_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    }
}

fn parse_release_trusted_public_keys(
    explicit_trusted_public_keys: &[String],
) -> Result<Option<Vec<nix_compat::narinfo::VerifyingKey>>, RunError> {
    if explicit_trusted_public_keys.is_empty() {
        return Ok(None);
    }

    let mut keys = Vec::with_capacity(explicit_trusted_public_keys.len());
    for key_str in explicit_trusted_public_keys {
        let key = nix_compat::narinfo::VerifyingKey::parse(key_str)
            .map_err(|err| RunError::Internal(format!("invalid trusted public key '{key_str}': {err}")))?;
        keys.push(key);
    }
    Ok(Some(keys))
}

fn print_trusted_public_key(
    trusted_public_key: &str,
    key_name: &str,
    source_path: &Path,
    json: bool,
) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::json!({
            "kind": "crunch-trusted-public-key",
            "trusted_public_key": trusted_public_key,
            "key_name": key_name,
            "source_path": source_path.display().to_string(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing trusted public key output: {err}")))?
        );
        return Ok(());
    }

    println!("{trusted_public_key}");
    Ok(())
}

fn print_created_witness_attestation(created: &CreatedWitnessAttestation, json: bool) -> Result<(), RunError> {
    if json {
        let attestation = serde_json::to_value(&created.attestation)
            .map_err(|err| RunError::Internal(format!("serializing witness attestation: {err}")))?;
        let rendered = serde_json::json!({
            "kind": "crunch-witness-attestation",
            "digest": created.digest_hex,
            "stored_path": created.attestation_path.display().to_string(),
            "signature_path": created.signature_path.display().to_string(),
            "signer": created.signer_key_name,
            "attestation": attestation,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing created witness output: {err}")))?
        );
        return Ok(());
    }

    println!("witness attestation: {}", created.attestation_path.display());
    println!("signature: {}", created.signature_path.display());
    println!("witness identity: {}", created.attestation.witness_identity);
    println!("digest: {}", created.digest_hex);
    println!("signer: {}", created.signer_key_name);
    Ok(())
}

fn print_created_policy_files(created: &CreatedPolicyFiles, json: bool) -> Result<(), RunError> {
    if json {
        let policy = serde_json::to_value(&created.policy)
            .map_err(|err| RunError::Internal(format!("serializing policy json: {err}")))?;
        let revocations = serde_json::to_value(&created.revocations)
            .map_err(|err| RunError::Internal(format!("serializing revocations json: {err}")))?;
        let rendered = serde_json::json!({
            "kind": "crunch-release-policy-init",
            "profile": created.profile.as_str(),
            "policy_path": created.policy_path.display().to_string(),
            "revocations_path": created.revocations_path.display().to_string(),
            "policy": policy,
            "revocations": revocations,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing created policy output: {err}")))?
        );
        return Ok(());
    }

    println!("policy: {}", created.policy_path.display());
    println!("revocations: {}", created.revocations_path.display());
    println!("profile: {}", created.profile.as_str());
    println!("min matching witnesses: {}", created.policy.min_matching_witnesses);
    println!("trusted release signers: {}", created.policy.trusted_release_signers.join(", "));
    println!("trusted witness identities: {}", created.policy.trusted_witness_signers.join(", "));
    Ok(())
}

fn print_release_verification_output(output: &ReleaseVerificationOutput) -> Result<(), RunError> {
    let text = serde_json::to_string_pretty(output)
        .map_err(|err| RunError::Internal(format!("serializing release verification output: {err}")))?;
    println!("{text}");
    Ok(())
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
    let attestation = synthesize_project_attestation(ProjectAttestationInput {
        manifest_text: &manifest_text,
        lock_text: &lock_text,
        manifest: &manifest,
        lock: &lock,
        selected_roots: &selected_roots,
    })
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
    let lock =
        Lockfile::from_json(text.clone()).map_err(|e| RunError::Internal(format!("parsing {LOCK_FILE}: {e}")))?;
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
        if let Some(envelope_digest) = envelope_digest
            && envelope_digest != actual_digest
        {
            return Err(RunError::Build(format!(
                "saved project attestation digest mismatch: expected {}, got {}",
                envelope_digest, actual_digest
            )));
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
    if let Some(store) = store
        && let Ok((document, _stored_path)) = load_artifact_document(store, input).await
    {
        return Ok(document);
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
    if let Ok(value) = serde_json::from_str::<ReleaseAttestation>(text) {
        return Ok((AttestationDocument::Release(value), None));
    }
    if let Ok(value) = serde_json::from_str::<WitnessAttestation>(text) {
        return Ok((AttestationDocument::Witness(value), None));
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
        "crunch-release-attestation" => serde_json::from_value::<ReleaseAttestation>(value)
            .map(AttestationDocument::Release)
            .map_err(|e| RunError::Internal(format!("parsing release attestation: {e}"))),
        "crunch-witness-attestation" => serde_json::from_value::<WitnessAttestation>(value)
            .map(AttestationDocument::Witness)
            .map_err(|e| RunError::Internal(format!("parsing witness attestation: {e}"))),
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
        AttestationDocument::Release(value) => {
            serde_json::to_value(value).map_err(|e| RunError::Internal(format!("serializing release: {e}")))
        }
        AttestationDocument::Witness(value) => {
            serde_json::to_value(value).map_err(|e| RunError::Internal(format!("serializing witness: {e}")))
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
        AttestationDocument::Release(value) => {
            value.canonical_bytes().map_err(|e| RunError::Internal(format!("release canonicalization: {e}")))
        }
        AttestationDocument::Witness(value) => {
            value.canonical_bytes().map_err(|e| RunError::Internal(format!("witness canonicalization: {e}")))
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
        AttestationDocument::Release(value) => value
            .canonical_digest()
            .map(|digest| digest.to_hex())
            .map_err(|e| RunError::Internal(format!("release digest: {e}"))),
        AttestationDocument::Witness(value) => value
            .canonical_digest()
            .map(|digest| digest.to_hex())
            .map_err(|e| RunError::Internal(format!("witness digest: {e}"))),
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
            Self::Release(_) => "crunch-release-attestation",
            Self::Witness(_) => "crunch-witness-attestation",
        }
    }
}
