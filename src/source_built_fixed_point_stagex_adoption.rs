//! Adopt two independently verified cold StageX exports into fresh local signed store state.
//! This is an operator consumer, not a producer proof or native-row qualification.

use std::fs;
use std::io::Write;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use bounded_tree_core::SymlinkPolicy;
use cap_fs_ext::DirExt;
use cap_std::ambient_authority;
use cap_std::fs::Dir;
use crunch_store::StoreBackend;
use crunch_store::StoreConfig;
use crunch_store::StoreFallbackMode;
use crunch_store::StoreHandle;
use crunch_store::StoreMutationGuard;
use nix_compat::narinfo::fingerprint_with_store_dir;
use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;
use crate::source_built_fixed_point::ProofOutputRole;
use crate::source_built_fixed_point::STAGEX_PROVIDER_LOGICAL_PATH;
use crate::source_built_fixed_point::STAGEX_PROVIDER_STORE_BASENAME;
use crate::source_built_fixed_point::STAGEX_TRANSITION_HANDOFF_DIRECTORIES;
use crate::source_built_fixed_point::STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES;
use crate::source_built_fixed_point::STAGEX_TRANSITION_LOGICAL_PATH;
use crate::source_built_fixed_point::STAGEX_TRANSITION_STORE_BASENAME;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point_trust_report::EXTENSION_FIELD;
use crate::source_built_fixed_point_trust_report::FINAL_RECEIPT_FILE;
use crate::source_built_fixed_point_trust_report::PROOF_PLAN_FILE;
use crate::source_built_fixed_point_trust_report::STAGE_EVIDENCE_FILE;
use crate::source_built_fixed_point_trust_report::SourceBuiltReceiptExtension;
use crate::source_built_fixed_point_trust_report::SourceBuiltStageEvidence;
use crate::source_built_fixed_point_trust_report::evidence_path;
use crate::source_built_fixed_point_trust_report::measure_artifact;
use crate::source_built_fixed_point_trust_report::read_bound_json;
use crate::source_built_fixed_point_trust_report::verify_source_built_fixed_point_receipt;

// r[impl bootstrap_inventory.cold_stagex_native_row_adoption]
const SCHEMA: &str = "mantle-cold-stagex-store-adoption-v1";
const PREFIX: &str = "/mantle/store";
const SNAPSHOT_ENTRIES_MAX: u32 = 1_000_000;
const NON_CLAIMS: [&str; 5] = [
    "No new StageX provider construction is established by this adoption.",
    "No new source-built fixed point is established by this adoption.",
    "No compiler correctness is established by this adoption.",
    "No native-row acceptance is established by this adoption.",
    "No release authority is established by this adoption.",
];

fn failure(message: impl Into<String>) -> RunError {
    RunError::Internal(format!("cold StageX adoption: {}", message.into()))
}

#[derive(Debug, Clone)]
pub struct StagexAdoptionOptions<'a> {
    pub proof_root: &'a Path,
    pub expected_plan_blake3: &'a str,
    pub expected_receipt_bytes_blake3: &'a str,
    pub output_dir: &'a Path,
    /// Absent for Snix; for Casita, a directory containing only an operator-provisioned trust
    /// policy.
    pub state_dir: &'a Path,
    pub signing_key: &'a Path,
    pub report: &'a Path,
    pub backend: StoreBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StagexAdoptedOutput {
    pub role: String,
    pub logical_path: String,
    pub source_relative_path: String,
    pub source_artifact_digest_blake3: String,
    pub published_tree_digest_blake3: String,
    pub nar_sha256: String,
    pub nar_size: u64,
    pub signature_count: u32,
}

// machine-artifact-public: bootstrap.cold-stagex-adoption-report
#[derive(Debug, Clone, Serialize)]
pub struct StagexAdoptionReport {
    pub schema: String,
    pub adopted: bool,
    pub proof_root: PathBuf,
    pub proof_receipt_bytes_blake3: String,
    pub proof_plan_digest_blake3: String,
    pub source_authority_digest_blake3: String,
    pub output_dir: PathBuf,
    pub state_dir: PathBuf,
    pub logical_store_prefix: String,
    pub trusted_public_key: String,
    pub signing_key_selected: bool,
    pub strict: bool,
    pub outputs: [StagexAdoptedOutput; 2],
    pub non_claims: [&'static str; 5],
}

#[derive(Clone, Copy)]
struct VerifiedExport<'a> {
    role: &'static str,
    logical_path: &'static str,
    basename: &'static str,
    source_relative_path: &'a str,
    digest: &'a str,
}

#[cfg_attr(test, derive(Clone))]
struct StagedOutput<'a> {
    export: VerifiedExport<'a>,
    path: PathBuf,
    digest: String,
}

fn digest_input(label: &str, digest: &str) -> Result<(), RunError> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        return Err(failure(format!("{label} must be a lowercase BLAKE3 hex digest")));
    }
    Ok(())
}

fn export<'a>(
    stages: &'a [SourceBuiltStageEvidence],
    role: ProofOutputRole,
    label: &'static str,
    logical_path: &'static str,
    basename: &'static str,
) -> Result<VerifiedExport<'a>, RunError> {
    let mut matching = stages.iter().filter(|stage| stage.output_role == role);
    let stage = matching.next().ok_or_else(|| failure(format!("missing {label} canonical stage")))?;
    if matching.next().is_some()
        || stage.status != "complete"
        || !stage.authority_violations.is_empty()
        || !stage.fallback_events.is_empty()
        || stage.producer_action_id.is_empty()
        || stage.output_id.is_empty()
    {
        return Err(failure(format!("ambiguous or incomplete {label} canonical stage")));
    }
    Ok(VerifiedExport {
        role: label,
        logical_path,
        basename,
        source_relative_path: &stage.output_path,
        digest: &stage.output_digest_blake3,
    })
}

/// Reject aliases and nesting before creating either private state directory.
/// All existing ancestors must be real directories (not symlinks); the final
/// component of an output/report or Snix state is required to be absent.
fn fresh_target(path: &Path) -> Result<PathBuf, RunError> {
    if !path.is_absolute()
        || path.file_name().is_none()
        || path.components().any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        return Err(failure(format!("target must be an absolute normalized path: {}", path.display())));
    }
    let parent = path.parent().ok_or_else(|| failure("target has no parent"))?;
    let mut ancestor = PathBuf::from("/");
    for component in parent.components() {
        if let Component::Normal(name) = component {
            ancestor.push(name);
            let metadata = fs::symlink_metadata(&ancestor)
                .map_err(|error| failure(format!("inspecting target ancestor {}: {error}", ancestor.display())))?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(failure(format!("target ancestor is not a real directory: {}", ancestor.display())));
            }
        }
    }
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(path.to_path_buf()),
        Err(error) => Err(failure(format!("inspecting fresh target {}: {error}", path.display()))),
        Ok(_) => Err(failure(format!("target already exists: {}", path.display()))),
    }
}

fn fresh_state_target(path: &Path, backend: StoreBackend) -> Result<PathBuf, RunError> {
    if backend == StoreBackend::Snix {
        return fresh_target(path);
    }
    // Casita's persistent trust is operator-owned. Admit only its preprovisioned
    // policy, never an existing backend, signing key, snapshot or store identity.
    fresh_target(&path.join("verified-transition-full-snapshot"))?;
    let mut entries =
        fs::read_dir(path).map_err(|error| failure(format!("opening policy-only Casita state: {error}")))?;
    let entry = entries
        .next()
        .transpose()
        .map_err(|error| failure(format!("reading policy-only Casita state: {error}")))?
        .ok_or_else(|| failure("Casita requires an operator-provisioned casita-trusted-public-keys file"))?;
    let kind = entry.file_type().map_err(|error| failure(format!("inspecting Casita trust policy: {error}")))?;
    if entry.file_name() != "casita-trusted-public-keys" || !kind.is_file() || entries.next().is_some() {
        return Err(failure("Casita state may contain only its regular operator-provisioned trust policy"));
    }
    Ok(path.to_path_buf())
}

fn disjoint(left: &Path, right: &Path) -> bool {
    !left.starts_with(right) && !right.starts_with(left)
}

#[cfg(unix)]
fn private_directory(path: &Path) -> Result<(), RunError> {
    use std::os::unix::fs::DirBuilderExt;
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|error| failure(format!("creating private directory {}: {error}", path.display())))
}

#[cfg(not(unix))]
fn private_directory(_path: &Path) -> Result<(), RunError> {
    Err(failure("cold StageX adoption requires Unix file modes"))
}

#[cfg(unix)]
fn copy_mode(source: &Path, target: &Path) -> Result<(), RunError> {
    let metadata = fs::symlink_metadata(source)
        .map_err(|error| failure(format!("inspecting source mode {}: {error}", source.display())))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(failure(format!("source directory is not a real directory: {}", source.display())));
    }
    fs::set_permissions(target, metadata.permissions())
        .map_err(|error| failure(format!("preserving directory mode {}: {error}", target.display())))?;
    use std::os::unix::fs::PermissionsExt;
    let copied = fs::symlink_metadata(target)
        .map_err(|error| failure(format!("checking copied directory mode {}: {error}", target.display())))?;
    if !copied.is_dir() || copied.permissions().mode() & 0o7777 != metadata.permissions().mode() & 0o7777 {
        return Err(failure(format!("copied directory mode differs: {}", target.display())));
    }
    Ok(())
}

#[cfg(not(unix))]
fn copy_mode(_source: &Path, _target: &Path) -> Result<(), RunError> {
    Err(failure("cold StageX adoption requires Unix file modes"))
}

fn measure_tree(path: &Path) -> Result<String, RunError> {
    let parent = path.parent().ok_or_else(|| failure("artifact tree has no parent"))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| failure("artifact tree name is not UTF-8"))?;
    Ok(measure_artifact(parent, name)?.1)
}

/// Open every ambient component through a directory capability, never following
/// a symlink even when a source path is replaced between admission and copy.
fn open_tree_root(path: &Path) -> Result<Dir, RunError> {
    if !path.is_absolute() {
        return Err(failure("copy root must be absolute"));
    }
    let mut current = Dir::open_ambient_dir("/", ambient_authority())
        .map_err(|error| failure(format!("opening filesystem root: {error}")))?;
    for component in path.components() {
        match component {
            Component::RootDir => (),
            Component::Normal(name) => {
                current = current
                    .open_dir_nofollow(name)
                    .map_err(|error| failure(format!("opening no-follow tree root {}: {error}", path.display())))?;
            }
            _ => return Err(failure(format!("copy root is not normalized: {}", path.display()))),
        }
    }
    Ok(current)
}

fn copy_tree(source: &Path, destination: &Path, disk_bound: u64) -> Result<(), RunError> {
    let source_dir = open_tree_root(source)?;
    let limits = bounded_tree_core::TreeLimits::new(bounded_tree_core::LimitValues {
        entries: u64::from(SNAPSHOT_ENTRIES_MAX),
        depth: 128,
        path_bytes: 4096,
        file_bytes: disk_bound,
        total_bytes: disk_bound,
        symlink_target_bytes: 4096,
    })
    .map_err(|error| failure(format!("admitting proof-bounded tree limits: {error:?}")))?;
    let prepared = bounded_tree_cap::prepare(&source_dir, limits, SymlinkPolicy::PreserveInternal)
        .map_err(|error| failure(format!("planning bounded copy {}: {error:?}", source.display())))?;
    private_directory(destination)?;
    let destination_dir = open_tree_root(destination)?;
    bounded_tree_cap::execute(&prepared, &destination_dir)
        .map_err(|error| failure(format!("copying bounded tree {}: {error:?}", source.display())))?;
    Ok(())
}

fn snapshot(source: &Path, destination: &Path, expected_digest: &str, disk_bound: u64) -> Result<(), RunError> {
    copy_tree(source, destination, disk_bound)?;
    copy_mode(source, destination)?;
    let measured = measure_tree(destination)?;
    if measured != expected_digest {
        return Err(failure(format!(
            "owned full canonical snapshot differs from verified export: {}",
            source.display()
        )));
    }
    Ok(())
}

fn require_regular(root: &Path, relative: &str) -> Result<(), RunError> {
    let path = evidence_path(root, relative)?;
    if !fs::symlink_metadata(&path)
        .map_err(|error| failure(format!("inspecting {relative}: {error}")))?
        .is_file()
    {
        return Err(failure(format!("required StageX tool is not a regular file: {relative}")));
    }
    Ok(())
}

fn transition_projection(snapshot: &Path, destination: &Path, disk_bound: u64) -> Result<String, RunError> {
    private_directory(destination)?;
    for required in STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES {
        require_regular(snapshot, required)?;
    }
    for relative in STAGEX_TRANSITION_HANDOFF_DIRECTORIES {
        let source = evidence_path(snapshot, relative)?;
        if !fs::symlink_metadata(&source)
            .map_err(|error| failure(format!("inspecting projection {relative}: {error}")))?
            .is_dir()
        {
            return Err(failure(format!("StageX projection is not a directory: {relative}")));
        }
        let expected_subtree = measure_tree(&source)?;
        let target = destination.join(relative);
        let runtime = target.parent().ok_or_else(|| failure("projection runtime parent missing"))?;
        let stage = runtime.parent().ok_or_else(|| failure("projection stage parent missing"))?;
        private_directory(stage)?;
        private_directory(runtime)?;
        copy_tree(&source, &target, disk_bound)?;
        copy_mode(&source, &target)?;
        copy_mode(source.parent().ok_or_else(|| failure("source runtime parent missing"))?, runtime)?;
        copy_mode(
            source.parent().and_then(Path::parent).ok_or_else(|| failure("source stage parent missing"))?,
            stage,
        )?;
        check_staged(&target, &expected_subtree)?;
    }
    copy_mode(snapshot, destination)?;
    for required in STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES {
        require_regular(destination, required)?;
    }
    measure_tree(destination)
}

fn provider_projection(
    snapshot: &Path,
    destination: &Path,
    expected_digest: &str,
    disk_bound: u64,
) -> Result<String, RunError> {
    copy_tree(snapshot, destination, disk_bound)?;
    copy_mode(snapshot, destination)?;
    for required in [
        "bin/x86_64-linux-musl-tcc",
        "bin/x86_64-linux-musl-tcc-0.9.26",
        "bin/x86_64-linux-musl-tcc-selfhost",
    ] {
        require_regular(destination, required)?;
    }
    for required in [
        "lib/tcc",
        "share/mantle-stagex/tcc-source-patched",
        "x86_64-linux-musl/include",
        "x86_64-linux-musl/lib",
    ] {
        let path = evidence_path(destination, required)?;
        if !fs::symlink_metadata(&path)
            .map_err(|error| failure(format!("inspecting {required}: {error}")))?
            .is_dir()
        {
            return Err(failure(format!("required provider directory missing: {required}")));
        }
    }
    check_staged(destination, expected_digest)?;
    Ok(expected_digest.to_string())
}

fn check_staged(directory: &Path, expected: &str) -> Result<(), RunError> {
    if measure_tree(directory)? != expected {
        return Err(failure(format!("staged tree changed before signed publication: {}", directory.display())));
    }
    Ok(())
}

fn strict_store_config(state_dir: &Path, output_dir: &Path, backend: StoreBackend) -> StoreConfig {
    StoreConfig {
        backend,
        state_dir: state_dir.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        remote_cache_urls: Vec::new(),
        fallback_mode: StoreFallbackMode::Strict,
        store_dir: PREFIX.into(),
        base_state_dirs: Vec::new(),
        source_admission: None,
    }
}

fn bind_captured_metadata(
    receipt: &serde_json::Value,
    plan: &SourceBuiltFixedPointPlan,
    stages_digest: &str,
) -> Result<(), RunError> {
    crate::source_built_fixed_point::validate_source_built_fixed_point_plan(plan)
        .map_err(|error| failure(format!("validating captured semantic plan: {error}")))?;
    let extension: SourceBuiltReceiptExtension = serde_json::from_value(
        receipt.get(EXTENSION_FIELD).cloned().ok_or_else(|| failure("captured v2 extension is missing"))?,
    )
    .map_err(|error| failure(format!("reading captured v2 extension: {error}")))?;
    if extension.stage_evidence_path != STAGE_EVIDENCE_FILE
        || extension.stage_evidence_digest_blake3 != stages_digest
        || extension.plan_digest_blake3 != plan.plan_digest_blake3
        || extension.source_authority_digest_blake3 != plan.source_authority_digest_blake3
    {
        return Err(failure("captured receipt does not bind stage evidence and semantic plan"));
    }
    Ok(())
}

fn verify_authority(
    options: &StagexAdoptionOptions<'_>,
    proof_root: &Path,
) -> Result<(SourceBuiltFixedPointPlan, Vec<SourceBuiltStageEvidence>, String), RunError> {
    digest_input("expected plan", options.expected_plan_blake3)?;
    digest_input("expected receipt bytes", options.expected_receipt_bytes_blake3)?;
    let (receipt, receipt_digest) = read_bound_json::<serde_json::Value>(proof_root, FINAL_RECEIPT_FILE)?;
    if receipt_digest != options.expected_receipt_bytes_blake3 {
        return Err(failure("raw proof receipt bytes differ from caller expectation"));
    }
    let (plan, _) = read_bound_json::<SourceBuiltFixedPointPlan>(proof_root, PROOF_PLAN_FILE)?;
    if plan.plan_digest_blake3 != options.expected_plan_blake3 || plan.logical_store_prefix != PREFIX {
        return Err(failure("semantic plan or logical store prefix differs from caller expectation"));
    }
    let (stages, stages_digest) = read_bound_json::<Vec<SourceBuiltStageEvidence>>(proof_root, STAGE_EVIDENCE_FILE)?;
    bind_captured_metadata(&receipt, &plan, &stages_digest)?;
    verify_source_built_fixed_point_receipt(proof_root, &proof_root.join(FINAL_RECEIPT_FILE))?;
    let (_, receipt_after) = read_bound_json::<serde_json::Value>(proof_root, FINAL_RECEIPT_FILE)?;
    let (plan_after, _) = read_bound_json::<SourceBuiltFixedPointPlan>(proof_root, PROOF_PLAN_FILE)?;
    let (_, stages_after) = read_bound_json::<Vec<SourceBuiltStageEvidence>>(proof_root, STAGE_EVIDENCE_FILE)?;
    if receipt_after != receipt_digest || plan_after != plan || stages_after != stages_digest {
        return Err(failure("proof authority changed during verification"));
    }
    Ok((plan, stages, receipt_digest))
}

fn stage_exports<'a>(
    proof_root: &Path,
    state_dir: &Path,
    output_dir: &Path,
    stages: &'a [SourceBuiltStageEvidence],
    disk_bound: u64,
) -> Result<[StagedOutput<'a>; 2], RunError> {
    let transition = export(
        stages,
        ProofOutputRole::StagexTransition,
        "stagex-transition",
        STAGEX_TRANSITION_LOGICAL_PATH,
        STAGEX_TRANSITION_STORE_BASENAME,
    )?;
    let provider = export(
        stages,
        ProofOutputRole::StagexProvider,
        "stagex-provider",
        STAGEX_PROVIDER_LOGICAL_PATH,
        STAGEX_PROVIDER_STORE_BASENAME,
    )?;
    // Reconciled producer/output references, not an arbitrary caller-selected tree.
    let transition_source = evidence_path(proof_root, transition.source_relative_path)?;
    let provider_source = evidence_path(proof_root, provider.source_relative_path)?;
    for source in [&transition_source, &provider_source] {
        if !fs::symlink_metadata(source)
            .map_err(|error| failure(format!("inspecting canonical export: {error}")))?
            .is_dir()
        {
            return Err(failure(format!("canonical export is not a tree: {}", source.display())));
        }
    }
    let transition_snapshot = state_dir.join("verified-transition-full-snapshot");
    let provider_snapshot = state_dir.join("verified-provider-full-snapshot");
    snapshot(&transition_source, &transition_snapshot, transition.digest, disk_bound)?;
    snapshot(&provider_source, &provider_snapshot, provider.digest, disk_bound)?;
    let transition_path = output_dir.join(transition.basename);
    let provider_path = output_dir.join(provider.basename);
    let transition_digest = transition_projection(&transition_snapshot, &transition_path, disk_bound)?;
    let provider_digest = provider_projection(&provider_snapshot, &provider_path, provider.digest, disk_bound)?;
    check_staged(&transition_path, &transition_digest)?;
    check_staged(&provider_path, &provider_digest)?;
    Ok([
        StagedOutput {
            export: transition,
            path: transition_path,
            digest: transition_digest,
        },
        StagedOutput {
            export: provider,
            path: provider_path,
            digest: provider_digest,
        },
    ])
}

async fn readback_one(
    lookup: &crunch_store::OutputLookup,
    item: &StagedOutput<'_>,
    keypair: &crunch_build::signing::KeyPair,
) -> Result<StagexAdoptedOutput, RunError> {
    let parsed = StorePath::from_absolute_path_with_prefix(item.export.logical_path.as_bytes(), PREFIX)
        .map_err(|error| failure(format!("parsing reserved store path: {error}")))?;
    let info = lookup
        .find(&parsed)
        .await
        .map_err(|error| failure(format!("reading back {} PathInfo: {error}", item.export.role)))?
        .ok_or_else(|| failure(format!("missing {} signed PathInfo", item.export.role)))?;
    if info.store_path != parsed || info.signatures.is_empty() || info.nar_size == 0 || !info.references.is_empty() {
        return Err(failure(format!("{} signed local PathInfo identity is invalid", item.export.role)));
    }
    let fingerprint = fingerprint_with_store_dir(
        &info.store_path.as_ref(),
        &info.nar_sha256,
        info.nar_size,
        std::iter::empty(),
        PREFIX,
    );
    if !info
        .signatures
        .iter()
        .any(|signature| keypair.verifying_key.verify(&fingerprint, &signature.as_ref()))
    {
        return Err(failure(format!("{} lacks the selected signer's valid signature", item.export.role)));
    }
    let request = crunch_nar::FilesystemNarRequest::new(
        nix_compat::nixhash::HashAlgo::Sha256,
        crunch_nar::CaseHackPolicy::native(),
        info.nar_size,
    );
    let observed = crunch_nar::observe_path_blocking(item.path.clone(), request)
        .await
        .map_err(|error| failure(format!("observing {} current NAR: {error}", item.export.role)))?;
    if observed.nar_size != info.nar_size || observed.digest.digest_as_bytes() != info.nar_sha256 {
        return Err(failure(format!("{} current NAR differs from signed PathInfo", item.export.role)));
    }
    check_staged(&item.path, &item.digest)?;
    let signature_count = u32::try_from(info.signatures.len())
        .map_err(|_| failure("signed PathInfo signature count does not fit u32"))?;
    Ok(StagexAdoptedOutput {
        role: item.export.role.into(),
        logical_path: item.export.logical_path.into(),
        source_relative_path: item.export.source_relative_path.into(),
        source_artifact_digest_blake3: item.export.digest.into(),
        published_tree_digest_blake3: item.digest.clone(),
        nar_sha256: data_encoding::HEXLOWER.encode(&info.nar_sha256),
        nar_size: info.nar_size,
        signature_count,
    })
}

fn publish_signed<'a>(
    state_dir: &Path,
    output_dir: &Path,
    backend: StoreBackend,
    keypair: &crunch_build::signing::KeyPair,
    stage: impl FnOnce() -> Result<[StagedOutput<'a>; 2], RunError>,
) -> Result<[StagexAdoptedOutput; 2], RunError> {
    let guard = StoreMutationGuard::acquire_wait(state_dir)
        .map_err(|error| failure(format!("acquiring adoption mutation guard: {error}")))?;
    let runtime =
        tokio::runtime::Runtime::new().map_err(|error| failure(format!("creating store runtime: {error}")))?;
    runtime.block_on(async {
        let mut store = StoreHandle::open(strict_store_config(state_dir, output_dir, backend))
            .await
            .map_err(|error| failure(format!("opening strict local store: {error}")))?;
        if backend == StoreBackend::Casita {
            store
                .recover_casita_gc_under_guard(&guard)
                .await
                .map_err(|error| failure(format!("recovering fresh Casita state: {error}")))?;
        }
        // Establish backend identity before retaining snapshots in its state directory.
        let items = stage()?;
        for item in &items {
            check_staged(&item.path, &item.digest)?;
            store
                .source_admission()
                .adopt_local_output(item.export.logical_path, "out", &keypair.signing_key, None)
                .await
                .map_err(|error| failure(format!("signing {}: {error}", item.export.role)))?;
        }
        drop(store);
        let reopened = StoreHandle::open(strict_store_config(state_dir, output_dir, backend))
            .await
            .map_err(|error| failure(format!("reopening durable signed store: {error}")))?;
        let parts = reopened.into_pipeline_store_parts();
        Ok([
            readback_one(&parts.output_lookup, &items[0], keypair).await?,
            readback_one(&parts.output_lookup, &items[1], keypair).await?,
        ])
    })
}

fn write_success_report(report_path: &Path, report: &StagexAdoptionReport) -> Result<(), RunError> {
    let bytes = serde_json::to_vec_pretty(report).map_err(|error| failure(format!("serializing report: {error}")))?;
    let report_parent = report_path.parent().ok_or_else(|| failure("report has no parent"))?;
    let mut pending = tempfile::Builder::new()
        .prefix(".stagex-adoption-")
        .tempfile_in(report_parent)
        .map_err(|error| failure(format!("creating temporary report: {error}")))?;
    pending
        .write_all(&bytes)
        .and_then(|()| pending.write_all(b"\n"))
        .and_then(|()| pending.as_file().sync_all())
        .map_err(|error| failure(format!("writing temporary report: {error}")))?;
    pending.persist_noclobber(report_path).map_err(|error| {
        failure(format!("publishing create-new success report {}: {}", report_path.display(), error.error))
    })?;
    Ok(())
}

/// Verified adoption does not claim a new fixed point, compiler correctness,
/// native-row acceptance or release authority. A partial effect retains private
/// state/output evidence and never publishes a successful report.
pub fn adopt_source_built_stagex(options: &StagexAdoptionOptions<'_>) -> Result<StagexAdoptionReport, RunError> {
    let proof_root =
        fs::canonicalize(options.proof_root).map_err(|error| failure(format!("opening proof root: {error}")))?;
    if !fs::symlink_metadata(&proof_root)
        .map_err(|error| failure(format!("inspecting proof root: {error}")))?
        .is_dir()
    {
        return Err(failure("proof root is not a directory"));
    }
    let output_dir = fresh_target(options.output_dir)?;
    let state_dir = fresh_state_target(options.state_dir, options.backend)?;
    let report_path = fresh_target(options.report)?;
    if !disjoint(&proof_root, &output_dir)
        || !disjoint(&proof_root, &state_dir)
        || !disjoint(&proof_root, &report_path)
        || !disjoint(&output_dir, &state_dir)
        || !disjoint(&output_dir, &report_path)
        || !disjoint(&state_dir, &report_path)
    {
        return Err(failure("proof, output, state and report locations must be disjoint"));
    }
    let (plan, stages, receipt_digest) = verify_authority(options, &proof_root)?;
    let (keypair, _) = crate::signing_key::load_existing_signing_keypair(Some(options.signing_key), &state_dir)?;
    strict_store_config(&state_dir, &output_dir, options.backend)
        .preflight_backend_identity()
        .map_err(|error| failure(format!("preflighting backend identity: {error}")))?;
    if options.backend == StoreBackend::Snix {
        private_directory(&state_dir)?;
    }
    private_directory(&output_dir)?;
    // Both permission-sensitive projections pass before any PathInfo publication.
    let outputs = publish_signed(&state_dir, &output_dir, options.backend, &keypair, || {
        stage_exports(&proof_root, &state_dir, &output_dir, &stages, plan.resource_bounds.disk_bytes_max)
    })?;
    let report = StagexAdoptionReport {
        schema: SCHEMA.into(),
        adopted: true,
        proof_root,
        proof_receipt_bytes_blake3: receipt_digest,
        proof_plan_digest_blake3: plan.plan_digest_blake3,
        source_authority_digest_blake3: plan.source_authority_digest_blake3,
        output_dir,
        state_dir,
        logical_store_prefix: PREFIX.into(),
        trusted_public_key: keypair.verifying_key.to_string(),
        signing_key_selected: true,
        strict: true,
        outputs,
        non_claims: NON_CLAIMS,
    };
    write_success_report(&report_path, &report)?;
    Ok(report)
}

// r[verify bootstrap_inventory.cold_stagex_native_row_adoption]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_locations_reject_overlap_aliases_and_existing_state() {
        let root = tempfile::tempdir().unwrap();
        let output = root.path().join("output");
        let state = root.path().join("state");
        assert!(disjoint(&output, &state));
        assert!(!disjoint(&output, &output.join("child")));
        assert_eq!(fresh_target(&output).unwrap(), output);
        fs::create_dir(&state).unwrap();
        assert!(fresh_target(&state).is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.path(), root.path().join("alias")).unwrap();
            assert!(fresh_target(&root.path().join("alias/out")).is_err());
        }
    }

    #[test]
    fn missing_verified_receipt_never_creates_state_or_success_report() {
        let root = tempfile::tempdir().unwrap();
        let proof = root.path().join("proof");
        fs::create_dir(&proof).unwrap();
        let output = root.path().join("store");
        let state = root.path().join("state");
        let report = root.path().join("adoption.json");
        let signing_key = root.path().join("signing-key");
        let (_, key_line) = crunch_build::signing::generate_keypair();
        fs::write(&signing_key, key_line).unwrap();
        let plan_pin = "a".repeat(64);
        let receipt_pin = "b".repeat(64);
        let options = StagexAdoptionOptions {
            proof_root: &proof,
            expected_plan_blake3: &plan_pin,
            expected_receipt_bytes_blake3: &receipt_pin,
            output_dir: &output,
            state_dir: &state,
            signing_key: &signing_key,
            report: &report,
            backend: StoreBackend::Snix,
        };
        assert!(adopt_source_built_stagex(&options).is_err());
        assert!(!output.exists() && !state.exists() && !report.exists());
    }

    #[test]
    fn complete_v2_fixture_wrong_plan_or_receipt_pin_has_no_effects() {
        use crate::source_built_fixed_point_trust_report::source_built_fixed_point_receipt_fixture::SyntheticReceiptFixture;

        let fixture = SyntheticReceiptFixture::new();
        let scratch = tempfile::tempdir().unwrap();
        let proof = fixture.root.path();
        let (plan, _) = read_bound_json::<SourceBuiltFixedPointPlan>(proof, PROOF_PLAN_FILE).unwrap();
        let (_, receipt_digest) = read_bound_json::<serde_json::Value>(proof, FINAL_RECEIPT_FILE).unwrap();
        let mut wrong_receipt = receipt_digest.clone();
        let receipt_replacement = if wrong_receipt.starts_with('a') { "b" } else { "a" };
        wrong_receipt.replace_range(..1, receipt_replacement);
        let mut wrong_plan = plan.plan_digest_blake3.clone();
        let plan_replacement = if wrong_plan.starts_with('a') { "b" } else { "a" };
        wrong_plan.replace_range(..1, plan_replacement);
        let signing_key = scratch.path().join("selected-key");
        let (_, key_line) = crunch_build::signing::generate_keypair();
        fs::write(&signing_key, key_line).unwrap();
        let output = scratch.path().join("store");
        let state = scratch.path().join("state");
        let report = scratch.path().join("adoption.json");
        let mut options = StagexAdoptionOptions {
            proof_root: proof,
            expected_plan_blake3: &plan.plan_digest_blake3,
            expected_receipt_bytes_blake3: &wrong_receipt,
            output_dir: &output,
            state_dir: &state,
            signing_key: &signing_key,
            report: &report,
            backend: StoreBackend::Snix,
        };
        assert!(adopt_source_built_stagex(&options).is_err());
        options.expected_plan_blake3 = &wrong_plan;
        options.expected_receipt_bytes_blake3 = &receipt_digest;
        assert!(adopt_source_built_stagex(&options).is_err());
        assert!(!output.exists() && !state.exists() && !report.exists());
    }

    #[test]
    fn captured_v2_stage_binding_rejects_metadata_substitution() {
        use crate::source_built_fixed_point_trust_report::source_built_fixed_point_receipt_fixture::SyntheticReceiptFixture;

        let fixture = SyntheticReceiptFixture::new();
        let proof = fixture.root.path();
        let (mut receipt, _) = read_bound_json::<serde_json::Value>(proof, FINAL_RECEIPT_FILE).unwrap();
        let (plan, _) = read_bound_json::<SourceBuiltFixedPointPlan>(proof, PROOF_PLAN_FILE).unwrap();
        let (_, stages_digest) = read_bound_json::<Vec<SourceBuiltStageEvidence>>(proof, STAGE_EVIDENCE_FILE).unwrap();
        bind_captured_metadata(&receipt, &plan, &stages_digest).unwrap();
        let captured_good = receipt.clone();
        let wrong_stages = if stages_digest.starts_with('a') {
            "b".repeat(64)
        } else {
            "a".repeat(64)
        };
        receipt[EXTENSION_FIELD]["stage_evidence_digest_blake3"] = serde_json::Value::String(wrong_stages);
        assert!(bind_captured_metadata(&receipt, &plan, &stages_digest).is_err());
        let mut invalid_plan = plan.clone();
        invalid_plan.source_authority_digest_blake3 = if plan.source_authority_digest_blake3.starts_with('a') {
            "b".repeat(64)
        } else {
            "a".repeat(64)
        };
        assert!(bind_captured_metadata(&captured_good, &invalid_plan, &stages_digest).is_err());
    }

    #[test]
    fn provider_projection_refuses_wrong_verified_identity() {
        let root = tempfile::tempdir().unwrap();
        let provider = root.path().join("provider");
        for relative in [
            "bin",
            "lib/tcc",
            "share/mantle-stagex/tcc-source-patched",
            "x86_64-linux-musl/include",
            "x86_64-linux-musl/lib",
        ] {
            fs::create_dir_all(provider.join(relative)).unwrap();
        }
        for relative in [
            "bin/x86_64-linux-musl-tcc",
            "bin/x86_64-linux-musl-tcc-0.9.26",
            "bin/x86_64-linux-musl-tcc-selfhost",
        ] {
            fs::write(provider.join(relative), b"canonical binary").unwrap();
        }
        let actual = measure_tree(&provider).unwrap();
        let wrong = if actual.starts_with('a') {
            "b".repeat(64)
        } else {
            "a".repeat(64)
        };
        let output = root.path().join("published");
        assert!(provider_projection(&provider, &output, &wrong, 1_000_000).is_err());
        assert_eq!(measure_tree(&output).unwrap(), actual);
    }

    #[test]
    fn owned_snapshot_detects_drift_and_rejects_escaping_links() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("tool"), b"original").unwrap();
        let digest = measure_tree(&source).unwrap();
        fs::write(source.join("tool"), b"changed").unwrap();
        assert!(snapshot(&source, &root.path().join("copy"), &digest, 1_000_000).is_err());
        #[cfg(unix)]
        {
            let unsafe_tree = root.path().join("unsafe");
            fs::create_dir(&unsafe_tree).unwrap();
            std::os::unix::fs::symlink("../source/tool", unsafe_tree.join("escape")).unwrap();
            assert!(snapshot(&unsafe_tree, &root.path().join("unsafe-copy"), &digest, 1_000_000).is_err());
        }
    }

    #[test]
    fn transition_projection_omits_unselected_artifacts_and_detects_copy_drift() {
        let root = tempfile::tempdir().unwrap();
        let canonical = root.path().join("canonical");
        fs::create_dir(&canonical).unwrap();
        for directory in STAGEX_TRANSITION_HANDOFF_DIRECTORIES {
            fs::create_dir_all(canonical.join(directory)).unwrap();
        }
        for tool in STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES {
            let path = canonical.join(tool);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, b"canonical tool").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&path, fs::Permissions::from_mode(0o751)).unwrap();
            }
        }
        fs::write(canonical.join("unselected-payload"), b"not an input").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("cp", canonical.join("coreutils-stage/runtime/output/bin/cp-alias")).unwrap();
        }
        let snapshot_path = root.path().join("snapshot");
        snapshot(&canonical, &snapshot_path, &measure_tree(&canonical).unwrap(), 1_000_000).unwrap();
        let projected = root.path().join("projected");
        let digest = transition_projection(&snapshot_path, &projected, 1_000_000).unwrap();
        assert!(!projected.join("unselected-payload").exists());
        assert_eq!(digest, measure_tree(&projected).unwrap());
        #[cfg(unix)]
        {
            assert_eq!(
                fs::read_link(projected.join("coreutils-stage/runtime/output/bin/cp-alias")).unwrap(),
                Path::new("cp")
            );
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let tool = STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES[0];
            assert_eq!(fs::metadata(projected.join(tool)).unwrap().permissions().mode() & 0o7777, 0o751);
        }
        fs::write(projected.join(STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES[0]), b"tampered").unwrap();
        assert!(check_staged(&projected, &digest).is_err());
    }
    #[test]
    fn signed_publication_reopens_and_rejects_other_signers_and_tree_drift() {
        for backend in [StoreBackend::Snix, StoreBackend::Casita] {
            let root = tempfile::tempdir().unwrap();
            let output = root.path().join("store");
            let state = root.path().join("state");
            fs::create_dir(&output).unwrap();
            fs::create_dir(&state).unwrap();
            let canonical = root.path().join("canonical");
            fs::create_dir(&canonical).unwrap();
            fs::write(canonical.join("tool"), b"canonical publication fixture").unwrap();
            let canonical_digest = measure_tree(&canonical).unwrap();
            let items = [
                ("stagex-transition", STAGEX_TRANSITION_LOGICAL_PATH, STAGEX_TRANSITION_STORE_BASENAME),
                ("stagex-provider", STAGEX_PROVIDER_LOGICAL_PATH, STAGEX_PROVIDER_STORE_BASENAME),
            ]
            .map(|(role, logical_path, basename)| {
                let path = output.join(basename);
                snapshot(&canonical, &path, &canonical_digest, 1_000_000).unwrap();
                StagedOutput {
                    export: VerifiedExport {
                        role,
                        logical_path,
                        basename,
                        source_relative_path: "canonical",
                        digest: &canonical_digest,
                    },
                    digest: measure_tree(&path).unwrap(),
                    path,
                }
            });
            let (selected, _) = crunch_build::signing::generate_keypair();
            let trust_policy = format!("{}\n", selected.verifying_key);
            if backend == StoreBackend::Casita {
                fs::write(state.join("casita-trusted-public-keys"), &trust_policy).unwrap();
                fresh_state_target(&state, backend).unwrap();
            }
            publish_signed(&state, &output, backend, &selected, || {
                snapshot(&canonical, &state.join("verified-transition-full-snapshot"), &canonical_digest, 1_000_000)?;
                Ok(items.clone())
            })
            .unwrap();
            if backend == StoreBackend::Casita {
                assert_eq!(fs::read_to_string(state.join("casita-trusted-public-keys")).unwrap(), trust_policy);
            }
            let (unselected, _) = crunch_build::signing::generate_keypair();
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let reopened = StoreHandle::open(strict_store_config(&state, &output, backend)).await.unwrap();
                let parts = reopened.into_pipeline_store_parts();
                let error = readback_one(&parts.output_lookup, &items[0], &unselected).await.unwrap_err();
                assert!(error.to_string().contains("selected signer's valid signature"));
                fs::write(items[0].path.join("tool"), b"tampered publication fixture").unwrap();
                assert!(readback_one(&parts.output_lookup, &items[0], &selected).await.is_err());
            });
        }
    }

    #[test]
    fn fresh_casita_state_requires_only_operator_owned_policy() {
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        fs::create_dir(&state).unwrap();
        assert!(fresh_state_target(&state, StoreBackend::Casita).is_err());
        fs::write(state.join("casita-trusted-public-keys"), b"operator-provisioned-policy\n").unwrap();
        assert_eq!(fresh_state_target(&state, StoreBackend::Casita).unwrap(), state);
        fs::write(state.join("store-identity.json"), b"existing backend").unwrap();
        assert!(fresh_state_target(&state, StoreBackend::Casita).is_err());
        assert!(fresh_state_target(&state, StoreBackend::Snix).is_err());
    }
}
