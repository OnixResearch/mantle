// machine-artifact-public: store.command-reports
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

const PATHINFO_SCAN_COUNT_MAX: u32 = 1_000_000;
const PATHINFO_SCAN_COUNT_MAX_USIZE: usize = 1_000_000;
const PATHINFO_INITIAL_CAPACITY: usize = 256;
const MAX_FOREIGN_RECEIPT_ROOT_PATHS: usize = 64;
const MAX_FOREIGN_REALIZATION_RECEIPT_BYTES: u64 = 16_777_216;
const FOREIGN_REALIZATION_RECEIPT_READ_LIMIT: u64 = MAX_FOREIGN_REALIZATION_RECEIPT_BYTES + 1;

use crate::build_cmd::load_configured_trusted_public_keys;
use crate::build_cmd::load_or_generate_signing_keypair;
use crate::errors::RunError;

pub fn cmd_store(
    action: crate::StoreAction,
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    is_json_output: bool,
) -> Result<(), RunError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;
    rt.block_on(async {
        cmd_store_async(action, StoreCommandContext {
            output_dir,
            state_dir,
            store_dir,
            is_json_output,
        })
        .await
    })
}

#[derive(Clone, Copy)]
struct StoreCommandContext<'a> {
    output_dir: &'a Path,
    state_dir: &'a Path,
    store_dir: &'a str,
    is_json_output: bool,
}

async fn cmd_store_async(action: crate::StoreAction, context: StoreCommandContext<'_>) -> Result<(), RunError> {
    match action {
        crate::StoreAction::List => {
            let svc = open_pathinfo_service(context.state_dir, true).await?;
            cmd_store_list(&svc).await
        }
        crate::StoreAction::Info { path } => {
            let svc = open_pathinfo_service(context.state_dir, true).await?;
            cmd_store_info(&svc, &path).await
        }
        crate::StoreAction::Roots => {
            let store = open_store(context.output_dir, context.state_dir, context.store_dir).await?;
            cmd_store_roots(&store)
        }
        other => cmd_store_mutation_or_transfer(other, context).await,
    }
}

async fn cmd_store_mutation_or_transfer(
    action: crate::StoreAction,
    context: StoreCommandContext<'_>,
) -> Result<(), RunError> {
    match action {
        crate::StoreAction::Pin { path } => {
            let _guard = store_mutation_guard(context.state_dir)?;
            let store = open_store(context.output_dir, context.state_dir, context.store_dir).await?;
            cmd_store_pin(&store, &path).await
        }
        crate::StoreAction::Unpin { path } => {
            let _guard = store_mutation_guard(context.state_dir)?;
            let store = open_store(context.output_dir, context.state_dir, context.store_dir).await?;
            cmd_store_unpin(&store, &path)
        }
        crate::StoreAction::Gc { is_dry_run } => cmd_store_gc_action(context, is_dry_run).await,
        crate::StoreAction::Verify {
            path,
            signing_key,
            trusted_public_keys,
            trust_unsigned,
        } => {
            cmd_store_verify_action(context, StoreVerifyAction {
                path_filter: path,
                signing_key_path: signing_key,
                trusted_public_keys,
                is_trust_unsigned: trust_unsigned,
            })
            .await
        }
        crate::StoreAction::Sign { path, all, signing_key } => {
            let _guard = store_mutation_guard(context.state_dir)?;
            let svc = open_pathinfo_service(context.state_dir, false).await?;
            cmd_store_sign(&svc, path.as_deref(), all, signing_key.as_deref(), context.state_dir).await
        }
        crate::StoreAction::RepairFinalNar {
            path,
            execute,
            signing_key,
        } => {
            let _guard = store_mutation_guard(context.state_dir)?;
            let store = open_store(context.output_dir, context.state_dir, context.store_dir).await?;
            cmd_store_repair_final_nar(
                &store,
                &path,
                execute,
                signing_key.as_deref(),
                context.state_dir,
                context.is_json_output,
            )
            .await
        }
        crate::StoreAction::Push {
            to,
            all,
            trust_unsigned,
            paths,
        } => {
            cmd_store_push_action(context, StorePushAction {
                destination_dir: to,
                is_all: all,
                is_trust_unsigned: trust_unsigned,
                paths,
            })
            .await
        }
        crate::StoreAction::Pull {
            from,
            all,
            closure,
            trust_unsigned,
            trusted_public_keys,
            foreign_realization_receipt,
            paths,
        } => {
            cmd_store_pull_action(context, StorePullAction {
                source_url: from,
                is_all: all,
                is_closure: closure,
                is_trust_unsigned: trust_unsigned,
                trusted_public_keys,
                foreign_realization_receipt,
                paths,
            })
            .await
        }
        crate::StoreAction::Archive { action } => cmd_store_archive(action, context).await,
        crate::StoreAction::List | crate::StoreAction::Info { .. } | crate::StoreAction::Roots => {
            Err(RunError::Internal("mutation dispatcher received a read-only store action".to_string()))
        }
    }
}

async fn cmd_store_gc_action(context: StoreCommandContext<'_>, is_dry_run: bool) -> Result<(), RunError> {
    let _guard = crunch_store::StoreMutationGuard::try_acquire(context.state_dir)
        .map_err(|e| RunError::Build(format!("{e}")))?;
    let rust_cache = crunch_rust_cache::RustCache::open_async(crunch_store::StoreConfig {
        state_dir: context.state_dir.to_path_buf(),
        output_dir: context.output_dir.to_path_buf(),
        remote_cache_urls: Vec::new(),
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: context.store_dir.to_string(),
        base_state_dirs: Vec::new(),
    })
    .await
    .map_err(|error| RunError::Build(format!("planning Rust cache retention: {error}")))?;
    let rust_retention = rust_cache
        .plan_retention_gc()
        .map_err(|error| RunError::Build(format!("planning Rust cache retention: {error}")))?;
    drop(rust_cache);
    let mut store = open_store(context.output_dir, context.state_dir, context.store_dir).await?;
    cmd_store_gc(&mut store, &rust_retention, is_dry_run).await
}

fn store_mutation_guard(state_dir: &Path) -> Result<crunch_store::StoreMutationGuard, RunError> {
    crunch_store::StoreMutationGuard::acquire_wait(state_dir)
        .map_err(|e| RunError::Internal(format!("acquiring store mutation lock: {e}")))
}

struct StoreVerifyAction {
    path_filter: Option<String>,
    signing_key_path: Option<PathBuf>,
    trusted_public_keys: Vec<String>,
    is_trust_unsigned: bool,
}

struct StorePushAction {
    destination_dir: PathBuf,
    is_all: bool,
    is_trust_unsigned: bool,
    paths: Vec<String>,
}

struct StorePullAction {
    source_url: String,
    is_all: bool,
    is_closure: bool,
    is_trust_unsigned: bool,
    trusted_public_keys: Vec<String>,
    foreign_realization_receipt: Option<PathBuf>,
    paths: Vec<String>,
}

async fn cmd_store_verify_action(context: StoreCommandContext<'_>, action: StoreVerifyAction) -> Result<(), RunError> {
    let svc = open_pathinfo_service(context.state_dir, true).await?;
    cmd_store_verify(&svc, StoreVerifyRequest {
        path_filter: action.path_filter.as_deref(),
        signing_key_path: action.signing_key_path.as_deref(),
        explicit_trusted_public_keys: &action.trusted_public_keys,
        is_trust_unsigned: action.is_trust_unsigned,
        state_dir: context.state_dir,
    })
    .await
}

async fn cmd_store_push_action(context: StoreCommandContext<'_>, action: StorePushAction) -> Result<(), RunError> {
    let _guard = store_mutation_guard(context.state_dir)?;
    let store = open_store(context.output_dir, context.state_dir, context.store_dir).await?;
    cmd_store_push(&store, StorePushRequest {
        destination_dir: &action.destination_dir,
        is_all: action.is_all,
        is_trust_unsigned: action.is_trust_unsigned,
        paths: &action.paths,
    })
    .await
}

async fn cmd_store_pull_action(context: StoreCommandContext<'_>, action: StorePullAction) -> Result<(), RunError> {
    let paths = resolve_foreign_receipt_pull_paths(
        &action.paths,
        action.foreign_realization_receipt.as_deref(),
        context.store_dir,
    )?;
    let _guard = store_mutation_guard(context.state_dir)?;
    let store = open_store(context.output_dir, context.state_dir, context.store_dir).await?;
    cmd_store_pull(&store, StorePullRequest {
        source_url: &action.source_url,
        is_all: action.is_all,
        is_closure: action.is_closure,
        is_trust_unsigned: action.is_trust_unsigned,
        explicit_trusted_public_keys: &action.trusted_public_keys,
        paths: &paths,
        state_dir: context.state_dir,
    })
    .await
}

fn resolve_foreign_receipt_pull_paths(
    explicit_paths: &[String],
    receipt_path: Option<&Path>,
    store_dir: &str,
) -> Result<Vec<String>, RunError> {
    let Some(receipt_path) = receipt_path else {
        return Ok(explicit_paths.to_vec());
    };
    if !explicit_paths.is_empty() {
        return Err(RunError::Internal(
            "foreign realization receipt cannot be combined with explicit store paths".to_string(),
        ));
    }
    let receipt_bytes = read_bounded_foreign_realization_receipt(receipt_path)?;
    let receipt: crate::foreign_realization_receipt::ForeignRealizationReceipt = serde_json::from_slice(&receipt_bytes)
        .map_err(|error| {
            RunError::Internal(format!("parsing foreign realization receipt {}: {error}", receipt_path.display()))
        })?;
    let paths = validate_foreign_receipt_pull_paths(&receipt)?;
    for path in &paths {
        nix_compat::store_path::StorePath::<String>::from_absolute_path_with_prefix(path.as_bytes(), store_dir)
            .map_err(|error| {
                RunError::Internal(format!(
                    "foreign realization receipt selected root is outside {store_dir}: {path}: {error}"
                ))
            })?;
    }
    Ok(paths)
}

fn read_bounded_foreign_realization_receipt(receipt_path: &Path) -> Result<Vec<u8>, RunError> {
    let file = fs::File::open(receipt_path).map_err(|error| {
        RunError::Internal(format!("opening foreign realization receipt {}: {error}", receipt_path.display()))
    })?;
    let mut receipt_bytes = Vec::new();
    file.take(FOREIGN_REALIZATION_RECEIPT_READ_LIMIT).read_to_end(&mut receipt_bytes).map_err(|error| {
        RunError::Internal(format!("reading foreign realization receipt {}: {error}", receipt_path.display()))
    })?;
    if u64::try_from(receipt_bytes.len()).unwrap_or(u64::MAX) > MAX_FOREIGN_REALIZATION_RECEIPT_BYTES {
        return Err(RunError::Internal(format!(
            "foreign realization receipt exceeds {MAX_FOREIGN_REALIZATION_RECEIPT_BYTES} bytes"
        )));
    }
    Ok(receipt_bytes)
}

fn validate_foreign_receipt_pull_paths(
    receipt: &crate::foreign_realization_receipt::ForeignRealizationReceipt,
) -> Result<Vec<String>, RunError> {
    use crate::foreign_realization_receipt::FOREIGN_REALIZATION_COMPLETE_STATUS;
    use crate::foreign_realization_receipt::FOREIGN_REALIZATION_REALIZED_STATE;
    use crate::foreign_realization_receipt::FOREIGN_REALIZATION_RECEIPT_SCHEMA;
    use crate::foreign_realization_receipt::foreign_realization_receipt_digest;

    if receipt.schema != FOREIGN_REALIZATION_RECEIPT_SCHEMA {
        return Err(RunError::Internal(format!("unsupported foreign realization receipt schema: {}", receipt.schema)));
    }
    if receipt.status != FOREIGN_REALIZATION_COMPLETE_STATUS
        || receipt.strongest_state != FOREIGN_REALIZATION_REALIZED_STATE
        || receipt.failure.is_some()
    {
        return Err(RunError::Internal("foreign realization receipt is not complete".to_string()));
    }
    let expected_digest = foreign_realization_receipt_digest(receipt)
        .map_err(|error| RunError::Internal(format!("hashing foreign realization receipt: {error}")))?;
    if receipt.receipt_blake3 != expected_digest {
        return Err(RunError::Internal("foreign realization receipt digest mismatch".to_string()));
    }
    if !foreign_receipt_selected_paths_are_bound(receipt) {
        return Err(RunError::Internal(
            "foreign realization receipt selected roots are not bound to unit outputs".to_string(),
        ));
    }
    if receipt.selected_root_paths.is_empty() || receipt.selected_root_paths.len() > MAX_FOREIGN_RECEIPT_ROOT_PATHS {
        return Err(RunError::Internal(format!(
            "foreign realization receipt root count must be between 1 and {MAX_FOREIGN_RECEIPT_ROOT_PATHS}"
        )));
    }
    if receipt.selected_root_paths.len() != 1 {
        return Err(RunError::Internal(
            "store pull --closure requires exactly one selected foreign realization root".to_string(),
        ));
    }
    Ok(receipt.selected_root_paths.clone())
}

fn foreign_receipt_selected_paths_are_bound(
    receipt: &crate::foreign_realization_receipt::ForeignRealizationReceipt,
) -> bool {
    let selected_root_ids = receipt.selected_root_node_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if selected_root_ids.is_empty() || selected_root_ids.len() != receipt.selected_root_node_ids.len() {
        return false;
    }
    let bound_paths = receipt
        .units
        .iter()
        .filter(|unit| selected_root_ids.contains(unit.node_id.as_str()) && unit.failure.is_none())
        .flat_map(|unit| unit.outputs.iter().map(|output| output.target_path.as_str()))
        .collect::<BTreeSet<_>>();
    !bound_paths.is_empty()
        && receipt.selected_root_paths.iter().all(|selected_path| bound_paths.contains(selected_path.as_str()))
}

async fn open_pathinfo_service(
    state_dir: &Path,
    is_read_only: bool,
) -> Result<snix_store::pathinfoservice::RedbPathInfoService, RunError> {
    use snix_store::pathinfoservice::RedbPathInfoService;
    use snix_store::pathinfoservice::RedbPathInfoServiceConfig;

    let db_path = state_dir.join("pathinfo.redb");
    if !db_path.exists() {
        return Err(RunError::Internal(format!(
            "PathInfo database {} does not exist (no builds yet?)",
            db_path.display()
        )));
    }
    RedbPathInfoService::new("crunch".to_string(), RedbPathInfoServiceConfig {
        path: Some(db_path.clone()),
        read_only: is_read_only,
        cache_size: None,
    })
    .await
    .map_err(|e| RunError::Internal(format!("opening PathInfo database {}: {e}", db_path.display())))
}

async fn open_store(
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
) -> Result<crunch_store::StoreHandle, RunError> {
    if store_dir.is_empty() || !Path::new(store_dir).is_absolute() {
        return Err(RunError::Internal(format!(
            "logical store directory must be absolute and non-empty: {store_dir:?}"
        )));
    }
    debug_assert!(!store_dir.is_empty());
    debug_assert!(Path::new(store_dir).is_absolute());
    crunch_store::StoreHandle::open(crunch_store::StoreConfig {
        state_dir: state_dir.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        remote_cache_urls: Vec::new(),
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: store_dir.to_string(),
        base_state_dirs: Vec::new(),
    })
    .await
    .map_err(|e| RunError::Internal(format!("opening store: {e}")))
}

async fn cmd_store_list(svc: &impl snix_store::pathinfoservice::PathInfoService) -> Result<(), RunError> {
    let entries = crunch_store::store_list(svc).await.map_err(|e| RunError::Internal(format!("{e}")))?;
    debug_assert!(entries.iter().all(|(store_path, _, _)| !store_path.is_empty()));
    debug_assert!(u32::try_from(entries.len()).is_ok());
    if entries.is_empty() {
        eprintln!("No paths in PathInfo database.");
    } else {
        for (store_path, deriver, nar_size) in &entries {
            println!("{store_path}  deriver={deriver}  nar_size={nar_size}");
        }
        eprintln!("{} path(s)", entries.len());
    }
    Ok(())
}

async fn cmd_store_info(svc: &impl snix_store::pathinfoservice::PathInfoService, path: &str) -> Result<(), RunError> {
    let details = crunch_store::store_info(svc, path).await.map_err(|e| RunError::Internal(format!("{e}")))?;
    if details.is_empty() {
        return Err(RunError::Internal(format!("no PathInfo matching '{path}'")));
    }
    debug_assert!(details.iter().all(|detail| !detail.store_path.is_empty()));
    debug_assert!(u32::try_from(details.len()).is_ok());
    for detail in &details {
        println!("store_path: {}", detail.store_path);
        println!("nar_size:   {}", detail.nar_size);
        println!("nar_sha256: {}", data_encoding::HEXLOWER.encode(&detail.nar_sha256));
        if let Some(ref deriver) = detail.deriver {
            println!("deriver:    {deriver}");
        }
        if !detail.references.is_empty() {
            println!("references:");
            for reference in &detail.references {
                println!("  {reference}");
            }
        }
        if let Some(ref ca) = detail.ca {
            println!("ca:         {ca}");
        }
        if detail.signatures.is_empty() {
            println!("signatures: (none)");
        } else {
            println!("signatures:");
            for sig in &detail.signatures {
                println!("  {sig}");
            }
        }
        println!("node:       {}", detail.node);
    }
    Ok(())
}

fn cmd_store_roots(store: &crunch_store::StoreHandle) -> Result<(), RunError> {
    let roots = store.list_retained_roots().map_err(|e| RunError::Internal(format!("{e}")))?;
    if roots.is_empty() {
        eprintln!("No retained GC roots.");
        return Ok(());
    }
    for root in &roots {
        println!("{}  source={}  created_unix_s={}", root.logical_path, root.source, root.created_unix_s);
    }
    eprintln!("{} retained root(s)", roots.len());
    Ok(())
}

async fn cmd_store_pin(store: &crunch_store::StoreHandle, path: &str) -> Result<(), RunError> {
    let root = store.pin_retained_root(path).await.map_err(|e| RunError::Build(format!("{e}")))?;
    println!("PINNED {}  source={}  created_unix_s={}", root.logical_path, root.source, root.created_unix_s);
    Ok(())
}

fn cmd_store_unpin(store: &crunch_store::StoreHandle, path: &str) -> Result<(), RunError> {
    let removed = store.unpin_retained_root(path).map_err(|e| RunError::Build(format!("{e}")))?;
    let Some(root) = removed else {
        return Err(RunError::Build(format!("retained root not found: {path}")));
    };
    println!("UNPINNED {}", root.logical_path);
    Ok(())
}

async fn cmd_store_gc(
    store: &mut crunch_store::StoreHandle,
    rust_retention: &crunch_rust_cache::RustCacheRetentionPlan,
    is_dry_run: bool,
) -> Result<(), RunError> {
    debug_assert!(!store.store_dir().is_empty());
    debug_assert!(Path::new(store.store_dir()).is_absolute());
    let gc_evidence = store
        .garbage_collect_with_castore_roots(is_dry_run, rust_retention.live_nodes())
        .await
        .map_err(|e| RunError::Build(format!("{e}")))?;
    rust_retention
        .apply(is_dry_run)
        .map_err(|error| RunError::Build(format!("applying Rust cache retention: {error}")))?;
    if !gc_report_has_candidates(&gc_evidence) && rust_retention.stale_result_count() == 0 {
        println!(
            "retained_roots={}  retained_rust_results={}  stale_rust_results={}  candidate_paths=0  reclaimable_bytes=0",
            gc_evidence.retained_root_count,
            gc_evidence.retained_castore_root_count,
            rust_retention.stale_result_count(),
        );
        if is_dry_run {
            eprintln!("dry-run: no changes made");
        } else {
            eprintln!("gc: nothing to do");
        }
        return Ok(());
    }

    println!(
        "retained_roots={}  retained_rust_results={}  stale_rust_results={}  candidate_paths={}  reclaimable_bytes={}",
        gc_evidence.retained_root_count,
        gc_evidence.retained_castore_root_count,
        rust_retention.stale_result_count(),
        gc_evidence.candidate_path_count,
        gc_evidence.reclaimable_bytes_total
    );
    println!(
        "candidate_exported_outputs={}  candidate_artifact_attestations={}  candidate_closure_attestations={}  candidate_blob_indexes={}  candidate_blob_chunks={}  candidate_action_result_records={}  candidate_action_result_indexes={}",
        gc_evidence.candidate_exported_output_count,
        gc_evidence.candidate_artifact_attestation_count,
        gc_evidence.candidate_closure_attestation_count,
        gc_evidence.candidate_blob_index_count,
        gc_evidence.candidate_blob_chunk_count,
        gc_evidence.candidate_action_result_record_count,
        gc_evidence.candidate_action_result_index_count,
    );
    for path in &gc_evidence.candidate_paths {
        println!("DELETE {path}");
    }
    if is_dry_run {
        eprintln!("dry-run: no changes made");
    } else {
        eprintln!(
            "gc: removed {} candidate path(s) and {} stale Rust result record(s)",
            gc_evidence.candidate_path_count,
            rust_retention.stale_result_count(),
        );
    }
    Ok(())
}

fn gc_report_has_candidates(report: &crunch_store::GcReport) -> bool {
    debug_assert_eq!(report.candidate_path_count == 0, report.candidate_paths.is_empty());
    debug_assert!(report.candidate_paths.iter().all(|path| !path.is_empty()));
    if report.candidate_path_count > 0 {
        return true;
    }
    if report.candidate_blob_index_count > 0 {
        return true;
    }
    if report.candidate_blob_chunk_count > 0 {
        return true;
    }
    if report.candidate_artifact_attestation_count > 0 {
        return true;
    }
    if report.candidate_closure_attestation_count > 0 {
        return true;
    }
    if report.candidate_exported_output_count > 0 {
        return true;
    }
    if report.candidate_action_result_record_count > 0 {
        return true;
    }
    if report.candidate_action_result_index_count > 0 {
        return true;
    }
    false
}

struct StoreVerifyRequest<'a> {
    path_filter: Option<&'a str>,
    signing_key_path: Option<&'a Path>,
    explicit_trusted_public_keys: &'a [String],
    is_trust_unsigned: bool,
    state_dir: &'a Path,
}

async fn cmd_store_verify(
    svc: &impl snix_store::pathinfoservice::PathInfoService,
    request: StoreVerifyRequest<'_>,
) -> Result<(), RunError> {
    let trusted_keys =
        resolve_store_verify_keys(request.signing_key_path, request.explicit_trusted_public_keys, request.state_dir)?;
    let hash_results = crunch_store::store_verify(svc, request.path_filter)
        .await
        .map_err(|e| RunError::Internal(format!("{e}")))?;
    let signature_results = crunch_store::store_verify_signatures(svc, request.path_filter, &trusted_keys)
        .await
        .map_err(|e| RunError::Internal(format!("{e}")))?;
    let signature_by_path = index_signature_results(signature_results)?;
    debug_assert_eq!(hash_results.len(), signature_by_path.len());
    debug_assert!(u32::try_from(hash_results.len()).is_ok());
    let summary = print_store_verify_results(&hash_results, &signature_by_path, request.is_trust_unsigned)?;

    eprintln!("{} checked, {} mismatches", summary.checked, summary.mismatches);
    if summary.mismatches > 0 {
        return Err(RunError::Build(format!("{} path(s) failed verification", summary.mismatches)));
    }
    Ok(())
}

struct VerifySummary {
    checked: u32,
    mismatches: u32,
}

fn resolve_store_verify_keys(
    signing_key_path: Option<&std::path::Path>,
    explicit_trusted_public_keys: &[String],
    state_dir: &Path,
) -> Result<Vec<nix_compat::narinfo::VerifyingKey>, RunError> {
    let parsed_explicit_keys = parse_trusted_public_keys(explicit_trusted_public_keys)?;
    let keypair = load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;
    let configured_trusted_keys = load_configured_trusted_public_keys(parsed_explicit_keys.as_deref(), state_dir)?;
    Ok(crunch_build::build_trusted_keys(&keypair, configured_trusted_keys.as_deref()))
}

fn parse_trusted_public_keys(
    explicit_trusted_public_keys: &[String],
) -> Result<Option<Vec<nix_compat::narinfo::VerifyingKey>>, RunError> {
    if explicit_trusted_public_keys.len() > PATHINFO_SCAN_COUNT_MAX_USIZE {
        return Err(RunError::Internal(format!("trusted key count exceeds {PATHINFO_SCAN_COUNT_MAX_USIZE} entries")));
    }
    debug_assert!(u32::try_from(explicit_trusted_public_keys.len()).is_ok());
    debug_assert!(explicit_trusted_public_keys.len() <= PATHINFO_SCAN_COUNT_MAX_USIZE);
    if explicit_trusted_public_keys.is_empty() {
        return Ok(None);
    }
    let mut keys = Vec::with_capacity(explicit_trusted_public_keys.len());
    for key_str in explicit_trusted_public_keys {
        let key = nix_compat::narinfo::VerifyingKey::parse(key_str)
            .map_err(|e| RunError::Internal(format!("invalid trusted public key '{key_str}': {e}")))?;
        keys.push(key);
    }
    Ok(Some(keys))
}

fn index_signature_results(
    signature_results: Vec<crunch_store::SignatureVerifyResult>,
) -> Result<std::collections::HashMap<String, crunch_store::SignatureVerifyResult>, RunError> {
    let mut signature_by_path = std::collections::HashMap::with_capacity(signature_results.len());
    for result in signature_results {
        let replaced = signature_by_path.insert(result.path.clone(), result);
        if replaced.is_some() {
            return Err(RunError::Internal("duplicate signature verification result for store path".to_string()));
        }
    }
    Ok(signature_by_path)
}

fn print_store_verify_results(
    hash_results: &[crunch_store::VerifyResult],
    signature_by_path: &std::collections::HashMap<String, crunch_store::SignatureVerifyResult>,
    is_trust_unsigned: bool,
) -> Result<VerifySummary, RunError> {
    let mut summary = VerifySummary {
        checked: 0,
        mismatches: 0,
    };
    for result in hash_results {
        let is_mismatched = print_store_verify_result(result, signature_by_path, is_trust_unsigned)?;
        if is_mismatched {
            summary.mismatches = summary.mismatches.saturating_add(1);
        }
        summary.checked = summary.checked.saturating_add(1);
    }
    Ok(summary)
}

fn print_store_verify_result(
    result: &crunch_store::VerifyResult,
    signature_by_path: &std::collections::HashMap<String, crunch_store::SignatureVerifyResult>,
    is_trust_unsigned: bool,
) -> Result<bool, RunError> {
    match result {
        crunch_store::VerifyResult::Ok(path) => print_verified_ok(path, signature_by_path, is_trust_unsigned),
        crunch_store::VerifyResult::Missing(path) => {
            println!("MISSING {path}");
            Ok(true)
        }
        crunch_store::VerifyResult::Mismatch {
            path,
            stored_hash,
            actual_hash,
        } => {
            println!("MISMATCH {path}  stored={stored_hash}  actual={actual_hash}");
            Ok(true)
        }
    }
}

fn print_verified_ok(
    path: &str,
    signature_by_path: &std::collections::HashMap<String, crunch_store::SignatureVerifyResult>,
    is_trust_unsigned: bool,
) -> Result<bool, RunError> {
    let sig_result = signature_by_path
        .get(path)
        .ok_or_else(|| RunError::Internal(format!("missing signature result for {path}")))?;
    debug_assert_eq!(sig_result.path, path);
    debug_assert!(sig_result.trusted_count <= sig_result.total_signatures);
    if is_trust_unsigned {
        println!("OK {path}  signatures=skipped");
        return Ok(false);
    }
    if sig_result.is_trusted() {
        println!("OK {path}  trusted_signatures={}/{}", sig_result.trusted_count, sig_result.total_signatures);
        return Ok(false);
    }
    if sig_result.total_signatures == 0 {
        println!("UNSIGNED {path}");
        return Ok(true);
    }
    let untrusted = sig_result.untrusted_names.join(",");
    println!("UNTRUSTED {path}  trusted_signatures=0/{}  signers={}", sig_result.total_signatures, untrusted);
    Ok(true)
}

async fn cmd_store_repair_final_nar(
    store: &crunch_store::StoreHandle,
    logical_store_path: &str,
    is_execute: bool,
    signing_key_path: Option<&Path>,
    state_dir: &Path,
    is_json_output: bool,
) -> Result<(), RunError> {
    assert!(!logical_store_path.is_empty(), "repair path must not be empty");
    assert!(!store.store_dir().is_empty(), "logical store prefix must not be empty");
    let inspection = crunch_store::inspect_final_nar_repair(store, logical_store_path)
        .await
        .map_err(|error| RunError::Internal(error.to_string()))?;
    let report = if is_execute && inspection.is_repair_required() {
        let keypair = load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;
        crunch_store::execute_final_nar_repair(store, inspection, &keypair.signing_key)
            .await
            .map_err(|error| RunError::Internal(error.to_string()))?
    } else {
        inspection.report(is_execute)
    };
    print_final_nar_repair_report(&report, is_json_output)
}

fn print_final_nar_repair_report(
    report: &crunch_store::FinalNarRepairReport,
    is_json_output: bool,
) -> Result<(), RunError> {
    if is_json_output {
        return print_json_report(report, "serializing final NAR repair report");
    }
    println!(
        "FINAL_NAR_REPAIR status={} path={} recorded_size={} recorded_sha256={} observed_size={} observed_sha256={} signatures={}->{} attestation={} execution_requested={} mutated={}",
        report.status.as_str(),
        report.store_path,
        report.recorded_nar_size,
        report.recorded_nar_sha256,
        report.observed_nar_size,
        report.observed_nar_sha256,
        report.old_signature_count,
        report.new_signature_count,
        report.artifact_attestation.as_str(),
        report.execution_requested,
        report.mutated,
    );
    if let Some(signer) = report.signer.as_deref() {
        eprintln!("repaired final NAR metadata with signer {signer}");
    }
    Ok(())
}

async fn cmd_store_sign(
    svc: &impl snix_store::pathinfoservice::PathInfoService,
    path_filter: Option<&str>,
    is_sign_all: bool,
    signing_key_path: Option<&std::path::Path>,
    state_dir: &Path,
) -> Result<(), RunError> {
    if path_filter.is_none() && !is_sign_all {
        return Err(RunError::Internal("provide a store path or use --all to sign all entries".to_string()));
    }
    debug_assert!(path_filter.is_some() || is_sign_all);
    debug_assert!(state_dir.components().next().is_some());

    let keypair = crate::build_cmd::load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;

    let results = crunch_store::store_sign(svc, &keypair.signing_key, path_filter, is_sign_all)
        .await
        .map_err(|e| RunError::Internal(format!("{e}")))?;

    let mut signed: u32 = 0;
    let mut appended: u32 = 0;
    let mut replaced: u32 = 0;
    for result in &results {
        if result.newly_signed {
            println!("SIGNED  {}", result.store_path);
            signed = signed.saturating_add(1);
            continue;
        }

        if result.appended {
            println!("APPEND  {}", result.store_path);
            appended = appended.saturating_add(1);
            continue;
        }

        if result.replaced {
            println!("REPLACE {}", result.store_path);
            replaced = replaced.saturating_add(1);
        }
    }

    eprintln!("{} signed, {} appended, {} replaced, {} total", signed, appended, replaced, results.len());
    Ok(())
}

struct StorePushRequest<'a> {
    destination_dir: &'a Path,
    is_all: bool,
    is_trust_unsigned: bool,
    paths: &'a [String],
}

async fn cmd_store_push(store: &crunch_store::StoreHandle, request: StorePushRequest<'_>) -> Result<(), RunError> {
    if request.paths.is_empty() && !request.is_all {
        return Err(RunError::Internal("provide store paths or use --all".to_string()));
    }
    debug_assert!(request.is_all || !request.paths.is_empty());
    debug_assert!(Path::new(store.store_dir()).is_absolute());

    // Collect matching PathInfo entries.
    let selected = if request.is_all {
        collect_all_pathinfos(store).await?
    } else {
        collect_matching_pathinfos(store, request.paths).await?
    };

    if selected.is_empty() {
        eprintln!("no matching paths found");
        return Ok(());
    }

    let options = crunch_store::PushOptions {
        trust_unsigned: request.is_trust_unsigned,
    };
    let push_evidence = crunch_store::export_paths_to_cache_dir(store, &selected, request.destination_dir, &options)
        .await
        .map_err(|e| RunError::Internal(format!("push: {e}")))?;

    for pushed in &push_evidence.paths {
        println!("PUSH {}", pushed.store_path);
    }

    if push_evidence.skipped_unsigned_count > 0 {
        eprintln!(
            "warning: {} unsigned path(s) skipped (use --trust-unsigned to include)",
            push_evidence.skipped_unsigned_count
        );
    }

    eprintln!(
        "pushed={} skipped_unsigned={} skipped_present={} nar_bytes={} narinfo_bytes={}",
        push_evidence.pushed_count,
        push_evidence.skipped_unsigned_count,
        push_evidence.skipped_already_present_count,
        push_evidence.total_nar_bytes,
        push_evidence.total_narinfo_bytes,
    );
    Ok(())
}

fn parse_pull_source(source: &str) -> Result<crunch_store::PullSource, RunError> {
    let is_http_source = source.starts_with("http://") || source.starts_with("https://");
    if is_http_source {
        let url = url::Url::parse(source)
            .map_err(|e| RunError::Internal(format!("invalid HTTP pull source '{source}': {e}")))?;
        if !url.username().is_empty() || url.password().is_some() {
            return Err(RunError::Internal(format!("HTTP pull source must not include URL credentials: {source}")));
        }
        return Ok(crunch_store::PullSource::Http(url));
    }
    if source.contains("://") {
        return Err(RunError::Internal(format!(
            "unsupported pull source URL scheme in '{source}'; only http:// and https:// are supported"
        )));
    }
    Ok(crunch_store::PullSource::Directory(std::path::PathBuf::from(source)))
}

fn parse_http_pull_paths(
    path_selectors: &[String],
    store_dir: &str,
) -> Result<Vec<nix_compat::store_path::StorePath<String>>, RunError> {
    let mut parsed_paths = Vec::with_capacity(path_selectors.len());
    for selector in path_selectors {
        let store_path =
            nix_compat::store_path::StorePath::from_absolute_path_with_prefix(selector.as_bytes(), store_dir).map_err(
                |_| {
                    RunError::Internal(format!(
                        "HTTP pull requires explicit logical store paths under {store_dir}; got '{selector}'"
                    ))
                },
            )?;
        parsed_paths.push(store_path);
    }
    Ok(parsed_paths)
}

struct StorePullRequest<'a> {
    source_url: &'a str,
    is_all: bool,
    is_closure: bool,
    is_trust_unsigned: bool,
    explicit_trusted_public_keys: &'a [String],
    paths: &'a [String],
    state_dir: &'a Path,
}

async fn cmd_store_pull(store: &crunch_store::StoreHandle, request: StorePullRequest<'_>) -> Result<(), RunError> {
    let pull_source = validate_pull_request(&request)?;
    let options = resolve_pull_options(&request)?;
    let pull_evidence = execute_pull(store, pull_source, &request, &options).await?;
    print_pull_evidence(&pull_evidence);
    Ok(())
}

fn validate_pull_request(request: &StorePullRequest<'_>) -> Result<crunch_store::PullSource, RunError> {
    let pull_source = parse_pull_source(request.source_url)?;
    if matches!(&pull_source, crunch_store::PullSource::Http(_)) && request.is_all {
        return Err(RunError::Internal("--all is not supported for HTTP caches; specify paths explicitly".to_string()));
    }
    if request.is_closure && !matches!(&pull_source, crunch_store::PullSource::Http(_)) {
        return Err(RunError::Internal("--closure is supported only for HTTP caches".to_string()));
    }
    if request.is_closure && request.paths.len() != 1 {
        return Err(RunError::Internal("--closure requires exactly one explicit logical store path".to_string()));
    }
    if request.paths.is_empty() && !request.is_all {
        let detail = if matches!(&pull_source, crunch_store::PullSource::Http(_)) {
            "HTTP pull requires explicit store path selectors"
        } else {
            "provide store paths or use --all"
        };
        return Err(RunError::Internal(detail.to_string()));
    }
    if let crunch_store::PullSource::Directory(source_dir) = &pull_source
        && !source_dir.exists()
    {
        return Err(RunError::Internal(format!("pull source directory does not exist: {}", source_dir.display())));
    }
    debug_assert!(request.is_all || !request.paths.is_empty());
    debug_assert!(request.state_dir.components().next().is_some());
    Ok(pull_source)
}

fn resolve_pull_options(request: &StorePullRequest<'_>) -> Result<crunch_store::PullOptions, RunError> {
    let parsed_explicit = parse_trusted_public_keys(request.explicit_trusted_public_keys)?;
    let keypair = load_or_generate_signing_keypair(None, request.state_dir, true)?;
    let configured_keys = load_configured_trusted_public_keys(parsed_explicit.as_deref(), request.state_dir)?;
    let trusted_keys = crunch_build::build_trusted_keys(&keypair, configured_keys.as_deref());
    Ok(crunch_store::PullOptions {
        trust_unsigned: request.is_trust_unsigned,
        trusted_public_keys: trusted_keys,
    })
}

enum StorePullEvidence {
    Explicit(crunch_store::PullReport),
    Closure(Box<crunch_store::HttpClosurePullReport>),
}

async fn execute_pull(
    store: &crunch_store::StoreHandle,
    pull_source: crunch_store::PullSource,
    request: &StorePullRequest<'_>,
    options: &crunch_store::PullOptions,
) -> Result<StorePullEvidence, RunError> {
    match pull_source {
        crunch_store::PullSource::Directory(source_dir) => {
            let paths_filter = (!request.is_all).then(|| request.paths.to_vec());
            crunch_store::import_paths_from_cache_dir(store, &source_dir, paths_filter.as_deref(), options)
                .await
                .map(StorePullEvidence::Explicit)
                .map_err(|error| RunError::Internal(format!("pull: {error}")))
        }
        crunch_store::PullSource::Http(cache_url) => {
            let requested_paths = parse_http_pull_paths(request.paths, store.store_dir())?;
            execute_http_pull(store, &cache_url, &requested_paths, request.is_closure, options).await
        }
    }
}

async fn execute_http_pull(
    store: &crunch_store::StoreHandle,
    cache_url: &url::Url,
    requested_paths: &[nix_compat::store_path::StorePath<String>],
    is_closure: bool,
    options: &crunch_store::PullOptions,
) -> Result<StorePullEvidence, RunError> {
    if is_closure {
        let root = requested_paths
            .first()
            .ok_or_else(|| RunError::Internal("closure root disappeared after validation".to_string()))?;
        return crunch_store::import_http_cache_closure(
            store,
            cache_url,
            root,
            options,
            crunch_store::HttpClosureLimits::default(),
        )
        .await
        .map(Box::new)
        .map(StorePullEvidence::Closure)
        .map_err(|error| RunError::Internal(format!("pull closure: {error}")));
    }
    crunch_store::import_paths_from_http_cache(store, cache_url, requested_paths, options)
        .await
        .map(StorePullEvidence::Explicit)
        .map_err(|error| RunError::Internal(format!("pull: {error}")))
}

fn print_pull_evidence(evidence: &StorePullEvidence) {
    match evidence {
        StorePullEvidence::Explicit(report) => print_pull_report(report),
        StorePullEvidence::Closure(report) => {
            println!(
                "CLOSURE plan_blake3={} members={} reused={} root={} admitted={}",
                report.plan.plan_blake3,
                report.plan.members.len(),
                report.reused_complete_count,
                report.plan.root,
                report.root_admitted,
            );
            print_pull_report(&report.pull);
        }
    }
}

fn print_pull_report(report: &crunch_store::PullReport) {
    debug_assert_eq!(u32::try_from(report.paths.len()).ok(), Some(report.imported_count));
    debug_assert!(report.paths.iter().all(|path| !path.store_path.is_empty()));
    for pulled in &report.paths {
        println!("PULL {}", pulled.store_path);
    }
    if report.skipped_untrusted_count > 0 {
        eprintln!(
            "warning: {} path(s) skipped (untrusted signature, use --trust-unsigned to include)",
            report.skipped_untrusted_count
        );
    }
    if report.skipped_store_dir_mismatch_count > 0 {
        eprintln!(
            "warning: {} path(s) skipped (store directory prefix mismatch)",
            report.skipped_store_dir_mismatch_count
        );
    }
    eprintln!(
        "imported={} skipped_present={} skipped_untrusted={} skipped_hash_mismatch={} skipped_missing_nar={} skipped_parse_error={} nar_bytes={}",
        report.imported_count,
        report.skipped_already_present_count,
        report.skipped_untrusted_count,
        report.skipped_hash_mismatch_count,
        report.skipped_missing_nar_count,
        report.skipped_parse_error_count,
        report.total_nar_bytes,
    );
}

async fn cmd_store_archive(
    action: crate::StoreArchiveAction,
    context: StoreCommandContext<'_>,
) -> Result<(), RunError> {
    match action {
        crate::StoreArchiveAction::Export {
            to,
            all,
            trust_unsigned,
            paths,
        } => {
            let store = open_store(context.output_dir, context.state_dir, context.store_dir).await?;
            cmd_store_archive_export(&store, StoreArchiveExportRequest {
                destination: &to,
                is_all: all,
                is_trust_unsigned: trust_unsigned,
                paths: &paths,
                is_json_output: context.is_json_output,
            })
            .await
        }
        crate::StoreArchiveAction::Import {
            from,
            trust_unsigned,
            trusted_public_keys,
            no_materialize,
        } => {
            let _guard = store_mutation_guard(context.state_dir)?;
            let store = open_store(context.output_dir, context.state_dir, context.store_dir).await?;
            cmd_store_archive_import(&store, StoreArchiveImportRequest {
                source: &from,
                is_trust_unsigned: trust_unsigned,
                explicit_trusted_public_keys: &trusted_public_keys,
                is_materialize: !no_materialize,
                state_dir: context.state_dir,
                is_json_output: context.is_json_output,
            })
            .await
        }
        crate::StoreArchiveAction::List { from } => cmd_store_archive_list(&from, context.is_json_output).await,
    }
}

struct StoreArchiveExportRequest<'a> {
    destination: &'a Path,
    is_all: bool,
    is_trust_unsigned: bool,
    paths: &'a [String],
    is_json_output: bool,
}

async fn cmd_store_archive_export(
    store: &crunch_store::StoreHandle,
    request: StoreArchiveExportRequest<'_>,
) -> Result<(), RunError> {
    if request.paths.is_empty() && !request.is_all {
        return Err(RunError::Internal("provide store paths or use --all".to_string()));
    }
    let selected = if request.is_all {
        collect_all_pathinfos(store).await?
    } else {
        collect_matching_pathinfos(store, request.paths).await?
    };
    if selected.is_empty() {
        return Err(RunError::Internal("no matching paths found".to_string()));
    }
    debug_assert!(request.is_all || !request.paths.is_empty());
    debug_assert!(!selected.is_empty());
    let options = crunch_store::ArchiveExportOptions {
        trust_unsigned: request.is_trust_unsigned,
    };
    let is_archive_stdout = is_stdio_path(request.destination);
    if is_archive_stdout && request.is_json_output {
        return Err(RunError::Internal("--json cannot be combined with archive export --to -".to_string()));
    }
    let archive_write_summary = if is_archive_stdout {
        let mut stdout = tokio::io::stdout();
        crunch_store::export_store_archive(store, &selected, &mut stdout, &options)
            .await
            .map_err(|e| RunError::Internal(format!("archive export: {e}")))?
    } else {
        let file = tokio::fs::File::create(request.destination)
            .await
            .map_err(|e| RunError::Internal(format!("creating archive {}: {e}", request.destination.display())))?;
        let mut writer = tokio::io::BufWriter::new(file);
        crunch_store::export_store_archive(store, &selected, &mut writer, &options)
            .await
            .map_err(|e| RunError::Internal(format!("archive export: {e}")))?
    };
    if is_archive_stdout {
        eprintln!(
            "exported={} payload_bytes={}",
            archive_write_summary.exported_count, archive_write_summary.total_payload_bytes
        );
        return Ok(());
    }
    print_archive_export_report(&archive_write_summary, request.is_json_output)
}

struct StoreArchiveImportRequest<'a> {
    source: &'a Path,
    is_trust_unsigned: bool,
    explicit_trusted_public_keys: &'a [String],
    is_materialize: bool,
    state_dir: &'a Path,
    is_json_output: bool,
}

async fn cmd_store_archive_import(
    store: &crunch_store::StoreHandle,
    request: StoreArchiveImportRequest<'_>,
) -> Result<(), RunError> {
    let trusted_public_keys = resolve_store_verify_keys(None, request.explicit_trusted_public_keys, request.state_dir)?;
    let options = crunch_store::ArchiveImportOptions {
        trust_unsigned: request.is_trust_unsigned,
        trusted_public_keys,
        materialize: request.is_materialize,
    };
    debug_assert_eq!(options.materialize, request.is_materialize);
    debug_assert!(u32::try_from(options.trusted_public_keys.len()).is_ok());
    let archive_read_summary = if is_stdio_path(request.source) {
        let mut stdin = tokio::io::stdin();
        crunch_store::import_store_archive(store, &mut stdin, &options)
            .await
            .map_err(|e| RunError::Internal(format!("archive import: {e}")))?
    } else {
        let file = tokio::fs::File::open(request.source)
            .await
            .map_err(|e| RunError::Internal(format!("opening archive {}: {e}", request.source.display())))?;
        let mut reader = tokio::io::BufReader::new(file);
        crunch_store::import_store_archive(store, &mut reader, &options)
            .await
            .map_err(|e| RunError::Internal(format!("archive import: {e}")))?
    };
    print_archive_import_report(&archive_read_summary, request.is_json_output)
}

async fn cmd_store_archive_list(source: &Path, is_json_output: bool) -> Result<(), RunError> {
    let archive_list_evidence = if is_stdio_path(source) {
        let mut stdin = tokio::io::stdin();
        crunch_store::list_store_archive(&mut stdin)
            .await
            .map_err(|e| RunError::Internal(format!("archive list: {e}")))?
    } else {
        let file = tokio::fs::File::open(source)
            .await
            .map_err(|e| RunError::Internal(format!("opening archive {}: {e}", source.display())))?;
        let mut reader = tokio::io::BufReader::new(file);
        crunch_store::list_store_archive(&mut reader)
            .await
            .map_err(|e| RunError::Internal(format!("archive list: {e}")))?
    };
    print_archive_list_report(&archive_list_evidence, is_json_output)
}

fn print_archive_export_report(
    report: &crunch_store::ArchiveExportReport,
    is_json_output: bool,
) -> Result<(), RunError> {
    if is_json_output {
        return print_json_report(report, "serializing archive export report");
    }
    for path in &report.paths {
        println!("ARCHIVE_EXPORT {} nar_size={} nar_sha256={}", path.store_path, path.nar_size, path.nar_sha256_hex);
    }
    eprintln!("exported={} payload_bytes={}", report.exported_count, report.total_payload_bytes);
    Ok(())
}

fn print_archive_import_report(
    report: &crunch_store::ArchiveImportReport,
    is_json_output: bool,
) -> Result<(), RunError> {
    if is_json_output {
        return print_json_report(report, "serializing archive import report");
    }
    for path in &report.paths {
        println!("ARCHIVE_IMPORT {} nar_size={} nar_sha256={}", path.store_path, path.nar_size, path.nar_sha256_hex);
    }
    eprintln!(
        "imported={} skipped_present={} payload_bytes={}",
        report.imported_count, report.skipped_already_present_count, report.total_payload_bytes
    );
    Ok(())
}

fn print_archive_list_report(report: &crunch_store::ArchiveListReport, is_json_output: bool) -> Result<(), RunError> {
    debug_assert!(!report.store_prefix.is_empty());
    debug_assert!(u32::try_from(report.paths.len()).is_ok());
    if is_json_output {
        return print_json_report(report, "serializing archive list report");
    }
    println!(
        "format={} store_prefix={} records={} payload_bytes={} compatibility={}",
        crunch_store::ARCHIVE_FORMAT_NAME,
        report.store_prefix,
        report.record_count,
        report.total_payload_bytes,
        report.compatibility
    );
    for path in &report.paths {
        println!(
            "ARCHIVE_PATH {} nar_size={} refs={} signatures={} ca={} root={} blake3={}",
            path.store_path,
            path.nar_size,
            path.reference_count,
            path.signature_count,
            path.ca.as_deref().unwrap_or("-"),
            path.root,
            path.payload_blake3
        );
    }
    Ok(())
}

fn print_json_report(report: &impl Serialize, context: &str) -> Result<(), RunError> {
    let rendered = serde_json::to_string_pretty(report).map_err(|e| RunError::Internal(format!("{context}: {e}")))?;
    println!("{rendered}");
    Ok(())
}

fn is_stdio_path(path: &Path) -> bool {
    path == Path::new("-")
}

async fn collect_all_pathinfos(
    store: &crunch_store::StoreHandle,
) -> Result<Vec<snix_store::path_info::PathInfo>, RunError> {
    use futures::StreamExt;
    use snix_store::pathinfoservice::PathInfoService;

    let mut stream = store.pathinfo_service().list();
    let mut results = Vec::with_capacity(PATHINFO_INITIAL_CAPACITY);
    for _ in 0..PATHINFO_SCAN_COUNT_MAX {
        let Some(result) = stream.next().await else {
            return Ok(results);
        };
        let path_info = result.map_err(|e| RunError::Internal(format!("listing pathinfo: {e}")))?;
        results.push(path_info);
    }
    if stream.next().await.is_none() {
        return Ok(results);
    }
    Err(RunError::Internal(format!("pathinfo scan exceeds {PATHINFO_SCAN_COUNT_MAX} entries")))
}

async fn collect_matching_pathinfos(
    store: &crunch_store::StoreHandle,
    selectors: &[String],
) -> Result<Vec<snix_store::path_info::PathInfo>, RunError> {
    use futures::StreamExt;
    use snix_store::pathinfoservice::PathInfoService;

    if selectors.is_empty() || selectors.iter().any(String::is_empty) {
        return Err(RunError::Internal("matching path selectors must be non-empty".to_string()));
    }
    debug_assert!(!selectors.is_empty());
    debug_assert!(selectors.iter().all(|selector| !selector.is_empty()));
    let mut stream = store.pathinfo_service().list();
    let mut results = Vec::with_capacity(selectors.len());
    for _ in 0..PATHINFO_SCAN_COUNT_MAX {
        let Some(result) = stream.next().await else {
            return Ok(results);
        };
        let path_info = result.map_err(|e| RunError::Internal(format!("listing pathinfo: {e}")))?;
        let store_path = path_info.store_path.to_string();
        if selectors.iter().any(|selector| store_path.contains(selector.as_str())) {
            results.push(path_info);
        }
    }
    if stream.next().await.is_none() {
        return Ok(results);
    }
    Err(RunError::Internal(format!("pathinfo scan exceeds {PATHINFO_SCAN_COUNT_MAX} entries")))
}
