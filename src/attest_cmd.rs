// machine-artifact-public: attestation.release-envelope
// machine-artifact-public: attestation.general-envelopes
use std::path::Path;
use std::path::PathBuf;

use crunch_attestation::ArtifactAttestation;
use crunch_attestation::ArtifactReference;
use crunch_attestation::Canonicalize;
use crunch_attestation::ClosureAttestation;
use crunch_attestation::ClosureSemantics;
use crunch_attestation::IndependentAgreementStatus;
use crunch_attestation::ProjectAttestation;
use crunch_attestation::ReleaseAttestation;
use crunch_attestation::WitnessAttestation;
use crunch_project::Lockfile;
use crunch_project::ProjectAttestationInput;
use crunch_project::ProjectManifest;
use crunch_project::synthesize_project_attestation;
use crunch_store::AttestationStore;
use crunch_store::StoreConfig;
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
use crate::release_attestation::PolicyProfileRequest;
use crate::release_attestation::ReleaseVerificationOutput;
use crate::release_attestation::WITNESS_SOURCE_ACQUISITION_MODE_MANUAL;
use crate::release_attestation::create_policy_files;
use crate::release_attestation::create_witness_attestation;
use crate::release_attestation::load_release_attestation_document;
use crate::release_attestation::load_witness_documents;

const MIN_ATTESTATION_BASE_LAYER_INDEX: u32 = 1;
const MAX_ATTESTATION_BASE_LAYERS: u32 = 8;
const MAX_ATTESTATION_SELECTED_LAYERS: usize = 65_536;
use crate::release_attestation::verify_release_attestation_directory;
use crate::witness_handoff::import_witness_material;

const MANIFEST_FILE: &str = "crunch-project.ncl";
const LOCK_FILE: &str = "crunch.lock";
const MAX_DIFF_ROWS_PER_LINE: usize = 2;

#[derive(Clone, Copy)]
struct AttestCommandContext<'a> {
    current_dir: &'a Path,
    output_dir: &'a Path,
    state_dir: &'a Path,
    store_dir: &'a str,
    base_state_dirs: &'a [PathBuf],
    is_json: bool,
}

struct ShowRequest<'a> {
    output_dir: &'a Path,
    state_dir: &'a Path,
    store_dir: &'a str,
    base_state_dirs: &'a [PathBuf],
    selector: &'a str,
}

struct DiffRequest<'a> {
    output_dir: &'a Path,
    state_dir: &'a Path,
    store_dir: &'a str,
    base_state_dirs: &'a [PathBuf],
    left: &'a str,
    right: &'a str,
}

struct WitnessCreateCommand<'a> {
    current_dir: &'a Path,
    state_dir: &'a Path,
    is_json: bool,
    verification_dir: &'a Path,
    rebuilt_binary: &'a [PathBuf],
    identity: Option<&'a str>,
    system: &'a str,
    toolchain: &'a str,
    host_class: &'a str,
    signing_key: Option<&'a Path>,
}

struct PolicyInitCommand<'a> {
    current_dir: &'a Path,
    is_json: bool,
    verification_dir: &'a Path,
    profile: PolicyInitProfile,
    trusted_release_signers: &'a [String],
    trusted_witness_identities: &'a [String],
    min_matching_witnesses: Option<u32>,
    independence_field: Option<&'a str>,
    is_force: bool,
}

struct TrustedPublicKeyOutput<'a> {
    trusted_public_key: &'a str,
    key_name: &'a str,
    source_path: &'a Path,
    is_json: bool,
}

struct StorePathInput<'a> {
    selector: &'a str,
    store_dir: &'a str,
    output_dir: &'a str,
}

struct LineDiffInput<'a> {
    left: &'a str,
    right: &'a str,
}

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
    #[serde(default = "absent_digest")]
    digest: Option<String>,
    attestation: Value,
}

#[derive(Debug, Serialize)]
struct AttestationEnvelopeOutput {
    kind: &'static str,
    digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stored_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_layer: Option<crunch_store::layer::StoreLayer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_layers: Option<std::collections::BTreeMap<String, crunch_store::layer::StoreLayer>>,
    attestation: Value,
}

fn absent_digest() -> Option<String> {
    None
}

// Stable CLI dispatch compatibility: `main.rs` supplies these independently
// typed fields positionally, while the implementation immediately names them.
#[allow(
    tigerstyle::too_many_parameters,
    reason = "stable CLI shell preserved while the internal command uses AttestCommandContext"
)]
pub fn cmd_attest(
    action: crate::AttestAction,
    current_dir: &Path,
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    base_state_dirs: &[PathBuf],
    is_json: bool,
) -> Result<(), RunError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;
    rt.block_on(cmd_attest_async(action, AttestCommandContext {
        current_dir,
        output_dir,
        state_dir,
        store_dir,
        base_state_dirs,
        is_json,
    }))
}

// Exhaustive enum dispatch is intentionally centralized so every attestation
// action shares the same path, store, and output-mode context.
#[allow(
    tigerstyle::function_length,
    reason = "single exhaustive AttestAction dispatcher keeps CLI authority centralized"
)]
async fn cmd_attest_async(action: crate::AttestAction, context: AttestCommandContext<'_>) -> Result<(), RunError> {
    let AttestCommandContext {
        current_dir,
        output_dir,
        state_dir,
        store_dir,
        base_state_dirs,
        is_json,
    } = context;
    debug_assert!(!store_dir.is_empty());
    debug_assert!(Path::new(store_dir).is_absolute());
    match action {
        crate::AttestAction::Show { path } => {
            cmd_show(ShowRequest {
                output_dir,
                state_dir,
                store_dir,
                base_state_dirs,
                selector: &path,
            })
            .await
        }
        crate::AttestAction::Closure { roots } => {
            cmd_closure(output_dir, state_dir, store_dir, base_state_dirs, &roots).await
        }
        crate::AttestAction::Verify { target } => {
            cmd_verify(target, current_dir, output_dir, state_dir, store_dir, base_state_dirs).await
        }
        crate::AttestAction::Diff { left, right } => {
            cmd_diff(DiffRequest {
                output_dir,
                state_dir,
                store_dir,
                base_state_dirs,
                left: &left,
                right: &right,
            })
            .await
        }
        crate::AttestAction::Project { roots } => {
            cmd_project(current_dir, output_dir, state_dir, store_dir, base_state_dirs, &roots).await
        }
        crate::AttestAction::ReleaseShow { verification_dir } => {
            let (attestation, stored_path) = load_release_attestation_document(&verification_dir)?;
            print_document(&AttestationDocument::Release(attestation), Some(stored_path))
        }
        crate::AttestAction::KeyShow { signing_key } => {
            cmd_key_show(current_dir, state_dir, is_json, signing_key.as_deref())
        }
        crate::AttestAction::WitnessCreate {
            verification_dir,
            rebuilt_binary,
            identity,
            system,
            toolchain,
            host_class,
            signing_key,
        } => cmd_witness_create(WitnessCreateCommand {
            current_dir,
            state_dir,
            is_json,
            verification_dir: &verification_dir,
            rebuilt_binary: &rebuilt_binary,
            identity: identity.as_deref(),
            system: &system,
            toolchain: &toolchain,
            host_class: &host_class,
            signing_key: signing_key.as_deref(),
        }),
        crate::AttestAction::WitnessShow {
            verification_dir,
            identity,
        } => cmd_witness_show(&verification_dir, identity.as_deref()),
        crate::AttestAction::WitnessImport {
            verification_dir,
            source,
        } => cmd_witness_import(current_dir, is_json, &verification_dir, &source),
        crate::AttestAction::PolicyInit {
            verification_dir,
            profile,
            trusted_release_signer,
            trusted_witness_identity,
            min_matching_witnesses,
            independence_field,
            force,
        } => cmd_policy_init(PolicyInitCommand {
            current_dir,
            is_json,
            verification_dir: &verification_dir,
            profile: map_policy_profile(profile),
            trusted_release_signers: &trusted_release_signer,
            trusted_witness_identities: &trusted_witness_identity,
            min_matching_witnesses,
            independence_field: independence_field.as_deref(),
            is_force: force,
        }),
        crate::AttestAction::ReleaseVerify {
            verification_dir,
            trusted_public_keys,
        } => cmd_release_verify(&verification_dir, &trusted_public_keys, state_dir, is_json),
    }
}

async fn cmd_show(request: ShowRequest<'_>) -> Result<(), RunError> {
    let store = open_store(request.output_dir, request.state_dir, request.store_dir, request.base_state_dirs).await?;
    let (document, stored_path, selected_layer) = load_artifact_document(&store, request.selector).await?;
    print_document_with_layers(&document, Some(stored_path), Some(selected_layer), None)
}

async fn cmd_closure(
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    base_state_dirs: &[PathBuf],
    roots: &[String],
) -> Result<(), RunError> {
    let store = open_store(output_dir, state_dir, store_dir, base_state_dirs).await?;
    let root_paths = resolve_roots(roots, store_dir, output_dir)?;
    let stored = store
        .runtime_closure_attestation(&root_paths)
        .await
        .map_err(|e| RunError::Internal(format!("loading closure attestation: {e}")))?;
    let path =
        closure_attestation_file_path(store.state_dir(), store.store_dir(), &root_paths, ClosureSemantics::Runtime);
    print_document_with_layers(
        &AttestationDocument::Closure(stored.attestation),
        Some(path),
        None,
        Some(stored.selected_layers),
    )
}

async fn cmd_diff(request: DiffRequest<'_>) -> Result<(), RunError> {
    let store = open_store(request.output_dir, request.state_dir, request.store_dir, request.base_state_dirs).await?;
    let left_document = load_document_input(Some(&store), request.left).await?;
    let right_document = load_document_input(Some(&store), request.right).await?;
    print_diff(request.left, &left_document, request.right, &right_document)
}

async fn cmd_project(
    current_dir: &Path,
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    base_state_dirs: &[PathBuf],
    roots: &[String],
) -> Result<(), RunError> {
    let store = open_store(output_dir, state_dir, store_dir, base_state_dirs).await?;
    let document = load_project_document(current_dir, &store, roots).await?;
    print_document(&document, None)
}

async fn cmd_verify(
    target: crate::AttestVerifyAction,
    current_dir: &Path,
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    base_state_dirs: &[PathBuf],
) -> Result<(), RunError> {
    match target {
        crate::AttestVerifyAction::Artifact { path } => {
            let store = open_store(output_dir, state_dir, store_dir, base_state_dirs).await?;
            let (document, stored_path, _selected_layer) = load_artifact_document(&store, &path).await?;
            verify_persisted_document(&document, &stored_path)
        }
        crate::AttestVerifyAction::Closure { roots } => {
            let store = open_store(output_dir, state_dir, store_dir, base_state_dirs).await?;
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
            let store = open_store(output_dir, state_dir, store_dir, base_state_dirs).await?;
            let document = load_project_document(current_dir, &store, &roots).await?;
            verify_project_document(&document, file.as_deref(), digest.as_deref())
        }
    }
}

fn cmd_key_show(current_dir: &Path, state_dir: &Path, json: bool, signing_key: Option<&Path>) -> Result<(), RunError> {
    let resolved_signing_key = signing_key.map(|path| resolve_cli_path(current_dir, path));
    let (keypair, source_path) = load_existing_signing_keypair(resolved_signing_key.as_deref(), state_dir)?;
    let trusted_public_key = keypair.verifying_key.to_string();
    print_trusted_public_key(TrustedPublicKeyOutput {
        trusted_public_key: &trusted_public_key,
        key_name: keypair.verifying_key.name(),
        source_path: &source_path,
        is_json: json,
    })
}

fn cmd_witness_create(request: WitnessCreateCommand<'_>) -> Result<(), RunError> {
    let resolved_verification_dir = resolve_cli_path(request.current_dir, request.verification_dir);
    let resolved_binaries = resolve_cli_paths(request.current_dir, request.rebuilt_binary);
    let created = create_witness_attestation(
        &resolved_verification_dir,
        &resolved_binaries,
        request.identity,
        request.system,
        request.toolchain,
        request.host_class,
        WITNESS_SOURCE_ACQUISITION_MODE_MANUAL,
        request.signing_key,
        request.state_dir,
    )?;
    print_created_witness_attestation(&created, request.is_json)
}

fn cmd_witness_import(current_dir: &Path, json: bool, verification_dir: &Path, source: &Path) -> Result<(), RunError> {
    let resolved_verification_dir = resolve_cli_path(current_dir, verification_dir);
    let resolved_source = resolve_cli_path(current_dir, source);
    let handoff_receipt = import_witness_material(&resolved_verification_dir, &resolved_source)?;
    debug_assert_eq!(handoff_receipt.verification_dir, resolved_verification_dir);
    debug_assert!(
        handoff_receipt.imported_witness_identities.capacity() >= handoff_receipt.imported_witness_identities.len()
    );
    if json {
        let rendered = serde_json::json!({
            "kind": "mantle-witness-import",
            "verification_dir": handoff_receipt.verification_dir.display().to_string(),
            "imported_witness_identities": handoff_receipt.imported_witness_identities,
            "skipped_duplicate_identities": handoff_receipt.skipped_duplicate_identities,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing witness import output: {err}")))?
        );
        return Ok(());
    }

    println!("verification dir: {}", handoff_receipt.verification_dir.display());
    println!(
        "imported witness identities: {}",
        if handoff_receipt.imported_witness_identities.is_empty() {
            "(none)".to_string()
        } else {
            handoff_receipt.imported_witness_identities.join(", ")
        }
    );
    println!(
        "skipped exact duplicates: {}",
        if handoff_receipt.skipped_duplicate_identities.is_empty() {
            "(none)".to_string()
        } else {
            handoff_receipt.skipped_duplicate_identities.join(", ")
        }
    );
    Ok(())
}

fn cmd_policy_init(request: PolicyInitCommand<'_>) -> Result<(), RunError> {
    let resolved_verification_dir = resolve_cli_path(request.current_dir, request.verification_dir);
    let created = create_policy_files(
        &resolved_verification_dir,
        PolicyProfileRequest {
            profile: request.profile,
            min_matching_witnesses: request.min_matching_witnesses,
            independence_field: request.independence_field,
            trusted_release_signers: request.trusted_release_signers,
            trusted_witness_identities: request.trusted_witness_identities,
        },
        request.is_force,
    )?;
    print_created_policy_files(&created, request.is_json)
}

fn map_policy_profile(profile: crate::AttestPolicyProfileArg) -> PolicyInitProfile {
    match profile {
        crate::AttestPolicyProfileArg::SelfProofOnly => PolicyInitProfile::SelfProofOnly,
        crate::AttestPolicyProfileArg::OptionalWitness => PolicyInitProfile::OptionalWitness,
        crate::AttestPolicyProfileArg::SingleWitness => PolicyInitProfile::SingleWitness,
        crate::AttestPolicyProfileArg::WitnessQuorum => PolicyInitProfile::WitnessQuorum,
    }
}

fn cmd_witness_show(verification_dir: &Path, requested_identity: Option<&str>) -> Result<(), RunError> {
    let documents = load_witness_documents(verification_dir)?;
    debug_assert!(documents.capacity() >= documents.len());
    debug_assert!(!verification_dir.as_os_str().is_empty());
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

    let document_count = documents.len();
    let mut rendered = Vec::with_capacity(document_count);
    for document in documents {
        let text = render_document(
            &AttestationDocument::Witness(document.attestation),
            Some(document.attestation_path.as_path()),
        )?;
        let envelope: Value = serde_json::from_str(&text)
            .map_err(|err| RunError::Internal(format!("parsing witness envelope json: {err}")))?;
        rendered.push(envelope);
    }
    debug_assert_eq!(rendered.len(), document_count);
    debug_assert!(rendered.capacity() >= rendered.len());
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
    is_json: bool,
) -> Result<(), RunError> {
    let trusted_public_keys = resolve_release_verify_keys(explicit_trusted_public_keys, state_dir)?;
    let output = verify_release_attestation_directory(verification_dir, &trusted_public_keys)?;
    print_release_verification_output(&output, is_json)
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

fn print_trusted_public_key(output: TrustedPublicKeyOutput<'_>) -> Result<(), RunError> {
    if output.is_json {
        let rendered = serde_json::json!({
            "kind": "crunch-trusted-public-key",
            "trusted_public_key": output.trusted_public_key,
            "key_name": output.key_name,
            "source_path": output.source_path.display().to_string(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing trusted public key output: {err}")))?
        );
        return Ok(());
    }

    println!("{}", output.trusted_public_key);
    Ok(())
}

fn print_created_witness_attestation(created: &CreatedWitnessAttestation, json: bool) -> Result<(), RunError> {
    debug_assert!(!created.digest_hex.is_empty());
    debug_assert!(!created.signer_key_name.is_empty());
    if json {
        let attestation = serde_json::to_value(&created.attestation)
            .map_err(|err| RunError::Internal(format!("serializing witness attestation: {err}")))?;
        let rendered = serde_json::json!({
            "kind": "mantle-witness-attestation",
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
    debug_assert!(!created.policy_path.as_os_str().is_empty());
    debug_assert!(!created.revocations_path.as_os_str().is_empty());
    if json {
        let policy = serde_json::to_value(&created.policy)
            .map_err(|err| RunError::Internal(format!("serializing policy json: {err}")))?;
        let revocations = serde_json::to_value(&created.revocations)
            .map_err(|err| RunError::Internal(format!("serializing revocations json: {err}")))?;
        let rendered = serde_json::json!({
            "kind": "mantle-release-policy-init",
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
    println!("independence field: {}", created.policy.independence_field);
    println!("trusted release signers: {}", created.policy.trusted_release_signers.join(", "));
    println!("trusted witness identities: {}", created.policy.trusted_witness_signers.join(", "));
    Ok(())
}

fn print_release_verification_output(output: &ReleaseVerificationOutput, is_json: bool) -> Result<(), RunError> {
    if !is_json {
        eprintln!("witness quorum status: {}", agreement_status_label(output.witness_quorum_status));
        eprintln!("independent agreement status: {}", agreement_status_label(output.independent_agreement_status));
    }
    let text = serde_json::to_string_pretty(output)
        .map_err(|err| RunError::Internal(format!("serializing release verification output: {err}")))?;
    println!("{text}");
    Ok(())
}

fn agreement_status_label(status: IndependentAgreementStatus) -> &'static str {
    match status {
        IndependentAgreementStatus::NotRequired => "not-required",
        IndependentAgreementStatus::Satisfied => "satisfied",
        IndependentAgreementStatus::Insufficient => "insufficient",
    }
}

async fn open_store(
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    base_state_dirs: &[PathBuf],
) -> Result<AttestationStore, RunError> {
    let config = StoreConfig {
        state_dir: state_dir.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        remote_cache_urls: Vec::new(),
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: store_dir.to_string(),
        base_state_dirs: base_state_dirs.to_vec(),
    };
    AttestationStore::open(config).await.map_err(|e| RunError::Internal(format!("opening store: {e}")))
}

async fn load_artifact_document(
    store: &AttestationStore,
    selector: &str,
) -> Result<(AttestationDocument, PathBuf, crunch_store::layer::StoreLayer), RunError> {
    let store_path = resolve_store_path(StorePathInput {
        selector,
        store_dir: store.store_dir(),
        output_dir: store.output_dir_str(),
    })?;
    let stored = store
        .get_artifact_attestation(&store_path)
        .await
        .map_err(|e| RunError::Internal(format!("loading artifact attestation: {e}")))?
        .ok_or_else(|| RunError::Internal(format!("no artifact attestation for {}", selector)))?;
    let stored_path = artifact_attestation_file_path(store.state_dir(), store.store_dir(), &store_path);
    Ok((AttestationDocument::Artifact(stored.attestation), stored_path, stored.selected_layer))
}

async fn load_project_document(
    current_dir: &Path,
    store: &AttestationStore,
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

async fn load_selected_roots(store: &AttestationStore, roots: &[String]) -> Result<Vec<ArtifactReference>, RunError> {
    if roots.is_empty() {
        return Err(RunError::Internal("provide at least one root path".to_string()));
    }

    let mut selected = Vec::with_capacity(roots.len());
    for root in roots {
        let store_path = resolve_store_path(StorePathInput {
            selector: root,
            store_dir: store.store_dir(),
            output_dir: store.output_dir_str(),
        })?;
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
    debug_assert_eq!(selected.len(), roots.len());
    debug_assert!(selected.capacity() >= selected.len());
    Ok(selected)
}

fn load_manifest(current_dir: &Path) -> Result<(String, ProjectManifest), RunError> {
    let path = current_dir.join(MANIFEST_FILE);
    let text =
        std::fs::read_to_string(&path).map_err(|e| RunError::Internal(format!("reading {}: {e}", path.display())))?;
    let evaluator_search_paths = vec![current_dir.as_os_str().to_owned()];
    let manifest = crunch_eval::evaluate_and_deserialize(&path, &evaluator_search_paths)
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
        .map(|root| {
            let output_dir_text = output_dir.display().to_string();
            resolve_store_path(StorePathInput {
                selector: root,
                store_dir,
                output_dir: &output_dir_text,
            })
        })
        .collect()
}

fn resolve_store_path(input: StorePathInput<'_>) -> Result<StorePath<String>, RunError> {
    if let Ok(store_path) = StorePath::from_absolute_path_with_prefix(input.selector.as_bytes(), input.store_dir) {
        return Ok(store_path);
    }

    if let Ok(store_path) = StorePath::from_absolute_path_with_prefix(input.selector.as_bytes(), input.output_dir) {
        return Ok(store_path);
    }

    if let Ok(store_path) = StorePath::from_absolute_path(input.selector.as_bytes()) {
        return Ok(store_path);
    }

    if let Ok(store_path) = StorePath::from_bytes(input.selector.as_bytes()) {
        return Ok(store_path);
    }

    Err(RunError::Internal(format!(
        "could not parse '{}' as a store path; use a logical path under {} or an exported path under {}",
        input.selector, input.store_dir, input.output_dir
    )))
}

fn print_document(document: &AttestationDocument, stored_path: Option<PathBuf>) -> Result<(), RunError> {
    print_document_with_layers(document, stored_path, None, None)
}

fn print_document_with_layers(
    document: &AttestationDocument,
    stored_path: Option<PathBuf>,
    selected_layer: Option<crunch_store::layer::StoreLayer>,
    selected_layers: Option<std::collections::BTreeMap<String, crunch_store::layer::StoreLayer>>,
) -> Result<(), RunError> {
    let text = render_document_with_layers(document, stored_path.as_deref(), selected_layer, selected_layers)?;
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
        debug_assert_eq!(expected_document.kind(), document.kind());
        debug_assert_eq!(expected_canonical, actual_canonical);

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

async fn load_document_input(store: Option<&AttestationStore>, input: &str) -> Result<AttestationDocument, RunError> {
    if let Some(store) = store
        && let Ok((document, _stored_path, _selected_layer)) = load_artifact_document(store, input).await
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
        debug_assert!(!envelope.kind.is_empty());
        debug_assert_eq!(document.kind(), envelope.kind);
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
        "mantle-release-attestation" => serde_json::from_value::<ReleaseAttestation>(value)
            .map(AttestationDocument::Release)
            .map_err(|e| RunError::Internal(format!("parsing release attestation: {e}"))),
        "mantle-witness-attestation" => serde_json::from_value::<WitnessAttestation>(value)
            .map(AttestationDocument::Witness)
            .map_err(|e| RunError::Internal(format!("parsing witness attestation: {e}"))),
        other => Err(RunError::Internal(format!("unsupported attestation kind '{other}'"))),
    }
}

fn render_document(document: &AttestationDocument, stored_path: Option<&Path>) -> Result<String, RunError> {
    render_document_with_layers(document, stored_path, None, None)
}

fn render_document_with_layers(
    document: &AttestationDocument,
    stored_path: Option<&Path>,
    selected_layer: Option<crunch_store::layer::StoreLayer>,
    selected_layers: Option<std::collections::BTreeMap<String, crunch_store::layer::StoreLayer>>,
) -> Result<String, RunError> {
    validate_attestation_layer_evidence(selected_layer, selected_layers.as_ref())?;
    let attestation_value = document_value(document)?;
    let envelope = AttestationEnvelopeOutput {
        kind: document.kind(),
        digest: document_digest_hex(document)?,
        stored_path: stored_path.map(|path| path.display().to_string()),
        selected_layer,
        selected_layers,
        attestation: attestation_value,
    };
    serde_json::to_string_pretty(&envelope)
        .map_err(|e| RunError::Internal(format!("serializing attestation output: {e}")))
}

fn validate_attestation_layer_evidence(
    selected_layer: Option<crunch_store::layer::StoreLayer>,
    selected_layers: Option<&std::collections::BTreeMap<String, crunch_store::layer::StoreLayer>>,
) -> Result<(), RunError> {
    let valid_layer = |layer: crunch_store::layer::StoreLayer| match layer {
        crunch_store::layer::StoreLayer::Overlay => true,
        crunch_store::layer::StoreLayer::Base { index } => {
            (MIN_ATTESTATION_BASE_LAYER_INDEX..=MAX_ATTESTATION_BASE_LAYERS).contains(&index)
        }
    };
    if selected_layer.is_some_and(|layer| !valid_layer(layer)) {
        return Err(RunError::Internal("attestation selected layer is invalid".to_string()));
    }
    if let Some(selected_layers) = selected_layers {
        if selected_layers.len() > MAX_ATTESTATION_SELECTED_LAYERS {
            return Err(RunError::Internal("attestation selected layer limit exceeded".to_string()));
        }
        if selected_layers.iter().any(|(path, layer)| path.is_empty() || !valid_layer(*layer)) {
            return Err(RunError::Internal("attestation selected layer evidence is invalid".to_string()));
        }
    }
    Ok(())
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
    for line in render_line_diff(LineDiffInput {
        left: &left_text,
        right: &right_text,
    }) {
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

fn render_line_diff(input: LineDiffInput<'_>) -> Vec<String> {
    let left_lines: Vec<&str> = input.left.lines().collect();
    let right_lines: Vec<&str> = input.right.lines().collect();
    let max_lines = left_lines.len().max(right_lines.len());
    let diff_row_count_max = max_lines.saturating_mul(MAX_DIFF_ROWS_PER_LINE);
    let mut diff = Vec::with_capacity(diff_row_count_max);

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
    debug_assert!(diff.len() <= diff_row_count_max);
    debug_assert!(diff.capacity() >= diff.len());
    diff
}

impl AttestationDocument {
    fn kind(&self) -> &'static str {
        match self {
            Self::Artifact(_) => "artifact",
            Self::Closure(_) => "closure",
            Self::Project(_) => "project",
            Self::Release(_) => "mantle-release-attestation",
            Self::Witness(_) => "mantle-witness-attestation",
        }
    }
}

#[cfg(test)]
mod machine_contract_tests {
    use super::*;

    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const DIGEST_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const FIXTURE_PATH: &str = "schemas/machine-contracts/fixtures/release-attestation-envelope.valid.json";
    const NO_STORED_PATH_FIXTURE: &str =
        "schemas/machine-contracts/fixtures/release-attestation-envelope-without-stored-path.valid.json";
    const STORED_PATH: &str = "release-verification/example/release-attestation.json";

    fn digest(value: &str) -> crunch_attestation::AttestationDigest {
        crunch_attestation::AttestationDigest::parse_hex(value.to_string()).expect("valid fixture digest")
    }

    fn release_attestation() -> ReleaseAttestation {
        ReleaseAttestation {
            schema: "mantle-release-attestation-v1".to_string(),
            release_id: "release-example".to_string(),
            release_evidence_manifest_digest_blake3: digest(DIGEST_B),
            proof_bundle_digest_blake3: digest(DIGEST_C),
            proof_mode: "fixed-point".to_string(),
            declared_effect_claims: None,
            observed_effect_facts: None,
            workflow: crunch_attestation::Workflow {
                command: "scripts/prove-self-hosting.sh".to_string(),
                version: "crunch-self-hosting-proof-v2".to_string(),
            },
            binary_digests: vec![crunch_attestation::BinaryDigest {
                name: "mantle".to_string(),
                algorithm: "blake3".to_string(),
                digest: DIGEST_D.to_string(),
            }],
        }
    }

    fn fixture(path: &str) -> Value {
        serde_json::from_str(
            &std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect("read release fixture"),
        )
        .expect("parse release fixture")
    }

    #[test]
    fn attestation_envelope_reports_selected_base_layer() {
        let rendered = render_document_with_layers(
            &AttestationDocument::Release(release_attestation()),
            None,
            Some(crunch_store::layer::StoreLayer::Base {
                index: MIN_ATTESTATION_BASE_LAYER_INDEX,
            }),
            None,
        )
        .expect("serialize selected base layer");
        let value: Value = serde_json::from_str(&rendered).expect("parse selected base layer envelope");

        assert_eq!(value["selected_layer"]["kind"], "base");
        assert_eq!(value["selected_layer"]["index"], MIN_ATTESTATION_BASE_LAYER_INDEX);
    }

    #[test]
    fn attestation_envelope_rejects_invalid_base_layer() {
        let error = validate_attestation_layer_evidence(Some(crunch_store::layer::StoreLayer::Base { index: 0 }), None)
            .expect_err("zero base layer must fail");

        assert!(error.message().contains("selected layer is invalid"));
    }

    #[test]
    fn release_envelope_serializes_to_registered_positive_fixtures() {
        let with_path =
            render_document(&AttestationDocument::Release(release_attestation()), Some(Path::new(STORED_PATH)))
                .expect("serialize release envelope with stored path");
        let with_path: Value = serde_json::from_str(&with_path).expect("parse release envelope with stored path");
        assert_eq!(with_path, fixture(FIXTURE_PATH));

        let without_path = render_document(&AttestationDocument::Release(release_attestation()), None)
            .expect("serialize release envelope without stored path");
        let without_path: Value =
            serde_json::from_str(&without_path).expect("parse release envelope without stored path");
        assert_eq!(without_path, fixture(NO_STORED_PATH_FIXTURE));
    }
}
