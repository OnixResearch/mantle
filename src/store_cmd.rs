// machine-artifact-public: store.command-reports
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

const PATHINFO_SCAN_COUNT_MAX: u32 = 1_000_000;
const PATHINFO_SCAN_COUNT_MAX_USIZE: usize = 1_000_000;
const MAX_FOREIGN_RECEIPT_ROOT_PATHS: usize = 64;
const MAX_FOREIGN_REALIZATION_RECEIPT_BYTES: u64 = 16_777_216;
const FOREIGN_REALIZATION_RECEIPT_READ_LIMIT: u64 = MAX_FOREIGN_REALIZATION_RECEIPT_BYTES + 1;
const COMPOSITION_REQUEST_BYTES_MAX: u64 = 1_048_576;
const COMPOSITION_REQUEST_READ_LIMIT: u64 = COMPOSITION_REQUEST_BYTES_MAX + 1;
const COMPOSITION_REQUEST_INITIAL_CAPACITY: usize = 8_192;

use crate::errors::RunError;
use crate::signing_key::load_configured_trusted_public_keys;
use crate::signing_key::load_or_generate_signing_keypair;

/// Effect identity of the store verification read.
const STORE_VERIFY_EFFECT: &str = "verify-paths";

/// Diagnostic code reported when verification finds a mismatch.
const STORE_VERIFY_MISMATCH_CODE: &str = "store-verify-mismatch";

/// Effect identity of the store listing read.
const STORE_LIST_EFFECT: &str = "read-files";

/// Effect identity of the store path-info read.
const STORE_INFO_EFFECT: &str = "read-files";
/// Effect identity of the retained-roots read (without migration).
const STORE_ROOTS_LIST_EFFECT: &str = "read-files";

/// Effect identity of the legacy-root migration write.
const STORE_ROOTS_MIGRATE_EFFECT: &str = "write-files";

/// Effect identity of the store usage plan read.
const STORE_USAGE_EFFECT: &str = "read-files";

/// Effect identity of the GC-root retention write.
const STORE_PIN_EFFECT: &str = "write-files";

/// Effect identity of the GC-root release write.
const STORE_UNPIN_EFFECT: &str = "write-files";

/// Diagnostic code reported when an unpin names an unknown root.
const STORE_UNPIN_UNKNOWN_ROOT_CODE: &str = "store-unpin-unknown-root";

/// Effect identity of the garbage-collection write.
const STORE_GC_EFFECT: &str = "write-files";

/// Diagnostic code reported when GC execution stops incomplete.
const STORE_GC_INCOMPLETE_CODE: &str = "store-gc-incomplete";

/// Effect identity of the final-NAR repair inspection read.
const STORE_REPAIR_INSPECT_EFFECT: &str = "read-files";

/// Effect identity of the final-NAR repair execution write.
const STORE_REPAIR_EXECUTE_EFFECT: &str = "write-files";

/// Effect identity of the PathInfo signing write.
const STORE_SIGN_EFFECT: &str = "write-files";

/// Effect identity of the push selection read.
const STORE_PUSH_SELECT_EFFECT: &str = "read-files";

/// Effect identity of the cache push write.
const STORE_PUSH_EFFECT: &str = "write-files";

/// Effect identity of the pull import write.
const STORE_PULL_EFFECT: &str = "write-files";

/// Diagnostic code reported when a closure pull does not admit its root.
const STORE_PULL_ROOT_NOT_ADMITTED_CODE: &str = "store-pull-closure-root-not-admitted";

/// Effect identity of the composition planning read.
const STORE_COMPOSITION_PLAN_EFFECT: &str = "read-files";

/// Effect identity of the composition realization write.
const STORE_COMPOSITION_REALIZE_EFFECT: &str = "write-files";

/// Effect identity of the archive export write.
const STORE_ARCHIVE_EXPORT_EFFECT: &str = "write-files";

/// Effect identity of the archive import write.
const STORE_ARCHIVE_IMPORT_EFFECT: &str = "write-files";

/// Effect identity of the archive listing read.
const STORE_ARCHIVE_LIST_EFFECT: &str = "read-files";

pub fn cmd_store(
    action: crate::StoreAction,
    output_dir: &Path,
    state_dir: &Path,
    backend: crunch_store::StoreBackend,
    store_dir: &str,
    base_state_dirs: &[PathBuf],
    is_json_output: bool,
) -> Result<(), RunError> {
    crate::command_input::admit_store_action_or_block(&action)?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;
    rt.block_on(async {
        cmd_store_async(action, StoreCommandContext {
            output_dir,
            state_dir,
            backend,
            store_dir,
            base_state_dirs,
            is_json_output,
        })
        .await
    })
}

#[derive(Clone, Copy)]
struct StoreCommandContext<'a> {
    output_dir: &'a Path,
    state_dir: &'a Path,
    backend: crunch_store::StoreBackend,
    store_dir: &'a str,
    base_state_dirs: &'a [PathBuf],
    is_json_output: bool,
}

async fn cmd_store_async(action: crate::StoreAction, context: StoreCommandContext<'_>) -> Result<(), RunError> {
    match action {
        crate::StoreAction::List => {
            let store = open_store(context).await?;
            cmd_store_list(&store, context.is_json_output).await
        }
        crate::StoreAction::Info { path } => {
            let store = open_store(context).await?;
            cmd_store_info(&store, &path, context.is_json_output).await
        }
        crate::StoreAction::Roots { migrate } => {
            let _guard = if migrate {
                Some(store_mutation_guard(context)?)
            } else {
                None
            };
            let mut store = if let Some(guard) = _guard.as_ref() {
                open_store_under_guard(context, guard).await?
            } else {
                open_store(context).await?
            };
            cmd_store_roots(&mut store, migrate, context)
        }
        crate::StoreAction::Usage => {
            preflight_store_backend(context)?;
            if context.backend == crunch_store::StoreBackend::Casita {
                ensure_no_unsupported_rust_cache_state(context.state_dir)?;
            }
            let _guard = if context.backend == crunch_store::StoreBackend::Casita {
                Some(
                    crunch_store::StoreMutationGuard::try_acquire(context.state_dir)
                        .map_err(|error| RunError::Build(format!("{error}")))?,
                )
            } else {
                None
            };
            let rust_retention = if context.backend == crunch_store::StoreBackend::Snix
                && !rust_cache_state_present(context.state_dir)
                    .map_err(|error| RunError::Build(format!("inspecting Rust cache state: {error}")))?
            {
                None
            } else {
                plan_rust_cache_retention(context).await?
            };
            let mut store = open_store(context).await?;
            recover_pending_casita_gc(&mut store, context, _guard.as_ref()).await?;
            cmd_store_usage(&mut store, _guard.as_ref(), rust_retention.as_ref(), context.is_json_output).await
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
            let _guard = store_mutation_guard(context)?;
            let store = open_store_under_guard(context, &_guard).await?;
            cmd_store_pin(&store, &path).await
        }
        crate::StoreAction::Unpin { path } => {
            let _guard = store_mutation_guard(context)?;
            let store = open_store_under_guard(context, &_guard).await?;
            cmd_store_unpin(&store, &path)
        }
        crate::StoreAction::Gc {
            execute,
            plan_id,
            legacy_dry_run,
        } => {
            debug_assert!(!legacy_dry_run || !execute);
            cmd_store_gc_action(context, execute, plan_id.as_deref()).await
        }
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
            let _guard = store_mutation_guard(context)?;
            let store = open_store_under_guard(context, &_guard).await?;
            let svc = store.pathinfo_service();
            cmd_store_sign(&*svc, path.as_deref(), all, signing_key.as_deref(), context.state_dir, context.store_dir)
                .await
        }
        crate::StoreAction::RepairFinalNar {
            path,
            execute,
            signing_key,
        } => {
            if context.backend == crunch_store::StoreBackend::Casita {
                return Err(RunError::Build("casita-repair-final-nar-unsupported".to_string()));
            }
            let _guard = store_mutation_guard(context)?;
            let store = open_store_under_guard(context, &_guard).await?;
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
        crate::StoreAction::Composition { action } => cmd_store_composition(action, context).await,
        crate::StoreAction::List
        | crate::StoreAction::Info { .. }
        | crate::StoreAction::Roots { .. }
        | crate::StoreAction::Usage => {
            Err(RunError::Internal("mutation dispatcher received a read-only store action".to_string()))
        }
    }
}

async fn cmd_store_gc_action(
    context: StoreCommandContext<'_>,
    execute: bool,
    accepted_plan_id: Option<&str>,
) -> Result<(), RunError> {
    if execute != accepted_plan_id.is_some() {
        return Err(RunError::Build("store gc execution requires both --execute and --plan-id".to_string()));
    }
    preflight_store_backend(context)?;
    if context.backend == crunch_store::StoreBackend::Casita {
        ensure_no_unsupported_rust_cache_state(context.state_dir)?;
    }
    let _guard = if execute || context.backend == crunch_store::StoreBackend::Casita {
        Some(
            crunch_store::StoreMutationGuard::try_acquire(context.state_dir)
                .map_err(|error| RunError::Build(format!("{error}")))?,
        )
    } else {
        None
    };
    let rust_retention = plan_rust_cache_retention(context).await?;
    let mut store = open_store(context).await?;
    recover_pending_casita_gc(&mut store, context, _guard.as_ref()).await?;
    cmd_store_gc(&mut store, _guard.as_ref(), rust_retention.as_ref(), accepted_plan_id, context.is_json_output).await
}

fn casita_gc_recovery_pending(context: StoreCommandContext<'_>) -> Result<bool, RunError> {
    if context.backend != crunch_store::StoreBackend::Casita {
        return Ok(false);
    }
    crunch_store::StoreHandle::casita_gc_fence_pending(context.state_dir)
        .map_err(|error| RunError::Build(format!("inspecting Casita GC recovery fence: {error}")))
}

async fn recover_pending_casita_gc(
    store: &mut crunch_store::StoreHandle,
    context: StoreCommandContext<'_>,
    guard: Option<&crunch_store::StoreMutationGuard>,
) -> Result<(), RunError> {
    if !casita_gc_recovery_pending(context)? {
        return Ok(());
    }
    let guard = guard.ok_or_else(|| {
        RunError::Build(
            "Casita GC recovery fence appeared during an unguarded plan; retry under the mutation guard".to_string(),
        )
    })?;
    store
        .recover_casita_gc_under_guard(guard)
        .await
        .map_err(|error| RunError::Build(format!("recovering Casita GC: {error}")))
}

fn rust_cache_state_present(state_dir: &Path) -> Result<bool, std::io::Error> {
    match std::fs::symlink_metadata(state_dir.join(crunch_rust_cache::RUST_CACHE_STATE_DIRECTORY)) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn ensure_no_unsupported_rust_cache_state(state_dir: &Path) -> Result<(), RunError> {
    let cache_path = state_dir.join(crunch_rust_cache::RUST_CACHE_STATE_DIRECTORY);
    match rust_cache_state_present(state_dir) {
        Ok(false) => Ok(()),
        Ok(true) => Err(RunError::Build(format!(
            "casita-rust-cache-unsupported: refusing GC or usage with Rust cache state at {}",
            cache_path.display(),
        ))),
        Err(error) => Err(RunError::Build(format!(
            "casita-rust-cache-unsupported: cannot inspect Rust cache state at {}: {error}",
            cache_path.display(),
        ))),
    }
}

async fn plan_rust_cache_retention(
    context: StoreCommandContext<'_>,
) -> Result<Option<crunch_rust_cache::RustCacheRetentionPlan>, RunError> {
    if context.backend == crunch_store::StoreBackend::Casita {
        ensure_no_unsupported_rust_cache_state(context.state_dir)?;
        return Ok(None);
    }
    let rust_cache = crunch_rust_cache::RustCache::open_async(crunch_store::StoreConfig {
        backend: context.backend,
        state_dir: context.state_dir.to_path_buf(),
        output_dir: context.output_dir.to_path_buf(),
        remote_cache_urls: Vec::new(),
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: context.store_dir.to_string(),
        base_state_dirs: Vec::new(),
    })
    .await
    .map_err(|error| RunError::Build(format!("planning Rust cache retention: {error}")))?;
    let plan = rust_cache
        .plan_retention_gc()
        .map_err(|error| RunError::Build(format!("planning Rust cache retention: {error}")))?;
    drop(rust_cache);
    Ok(Some(plan))
}

fn preflight_store_backend(context: StoreCommandContext<'_>) -> Result<(), RunError> {
    crunch_store::StoreConfig::new(
        context.backend,
        context.state_dir.to_path_buf(),
        context.output_dir.to_path_buf(),
        context.store_dir.to_string(),
    )
    .with_base_state_dirs(context.base_state_dirs.to_vec())
    .preflight_backend_identity()
    .map_err(|error| RunError::Internal(format!("opening store: {error}")))
}

fn store_mutation_guard(context: StoreCommandContext<'_>) -> Result<crunch_store::StoreMutationGuard, RunError> {
    preflight_store_backend(context)?;
    crunch_store::StoreMutationGuard::acquire_wait(context.state_dir)
        .map_err(|e| RunError::Internal(format!("acquiring store mutation lock: {e}")))
}

async fn open_store_under_guard(
    context: StoreCommandContext<'_>,
    guard: &crunch_store::StoreMutationGuard,
) -> Result<crunch_store::StoreHandle, RunError> {
    let mut store = open_store(context).await?;
    if context.backend == crunch_store::StoreBackend::Casita {
        store
            .recover_casita_gc_under_guard(guard)
            .await
            .map_err(|error| RunError::Build(format!("recovering Casita GC before store mutation: {error}")))?;
    }
    Ok(store)
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
    let store = open_store(context).await?;
    let svc = store.pathinfo_service();
    cmd_store_verify(&*svc, StoreVerifyRequest {
        path_filter: action.path_filter.as_deref(),
        signing_key_path: action.signing_key_path.as_deref(),
        explicit_trusted_public_keys: &action.trusted_public_keys,
        backend: context.backend,
        is_trust_unsigned: action.is_trust_unsigned,
        state_dir: context.state_dir,
        store_dir: context.output_dir,
        store_prefix: context.store_dir,
    })
    .await
}

async fn cmd_store_push_action(context: StoreCommandContext<'_>, action: StorePushAction) -> Result<(), RunError> {
    let _guard = store_mutation_guard(context)?;
    let store = open_store_under_guard(context, &_guard).await?;
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
    let _guard = store_mutation_guard(context)?;
    let store = open_store_under_guard(context, &_guard).await?;
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

async fn open_store(context: StoreCommandContext<'_>) -> Result<crunch_store::StoreHandle, RunError> {
    if context.store_dir.is_empty() || !Path::new(context.store_dir).is_absolute() {
        return Err(RunError::Internal(format!(
            "logical store directory must be absolute and non-empty: {:?}",
            context.store_dir
        )));
    }
    debug_assert!(!context.store_dir.is_empty());
    debug_assert!(Path::new(context.store_dir).is_absolute());
    crunch_store::StoreHandle::open(crunch_store::StoreConfig {
        backend: context.backend,
        state_dir: context.state_dir.to_path_buf(),
        output_dir: context.output_dir.to_path_buf(),
        remote_cache_urls: Vec::new(),
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: context.store_dir.to_string(),
        base_state_dirs: context.base_state_dirs.to_vec(),
    })
    .await
    .map_err(|e| RunError::Internal(format!("opening store: {e}")))
}

async fn cmd_store_list(store: &crunch_store::StoreHandle, is_json_output: bool) -> Result<(), RunError> {
    let entries = store.list_pathinfos_with_layer().await.map_err(|error| RunError::Internal(format!("{error}")))?;
    debug_assert!(entries.iter().all(|entry| !entry.value.store_path.to_string().is_empty()));
    debug_assert!(u32::try_from(entries.len()).is_ok());
    let overlay = store
        .overlay_report()
        .map_err(|error| RunError::Internal(format!("building overlay report: {error}")))?;
    if is_json_output {
        let paths = entries
            .iter()
            .map(|entry| {
                serde_json::json!({
                    "store_path": entry.value.store_path.to_string(),
                    "layer": entry.layer,
                    "shadows": entry.shadows,
                    "deriver": entry.value.deriver.as_ref().map(ToString::to_string),
                    "nar_size": entry.value.nar_size,
                })
            })
            .collect::<Vec<_>>();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema": "mantle-store-list-v2",
                "overlay": overlay,
                "paths": paths,
            }))
            .map_err(|error| RunError::Internal(format!("serializing store list: {error}")))?
        );
        return classify_store_effect(STORE_LIST_EFFECT, None);
    }
    print_overlay_summary(overlay.as_ref());
    if entries.is_empty() {
        eprintln!("No paths in the composed store.");
    } else {
        for entry in &entries {
            let deriver =
                entry.value.deriver.as_ref().map_or_else(|| "-".to_string(), |value| value.name().to_string());
            println!(
                "{}  layer={}  deriver={}  nar_size={}  shadows={:?}",
                entry.value.store_path, entry.layer, deriver, entry.value.nar_size, entry.shadows
            );
        }
        eprintln!("{} path(s)", entries.len());
    }
    classify_store_effect(STORE_LIST_EFFECT, None)
}

async fn cmd_store_info(store: &crunch_store::StoreHandle, path: &str, is_json_output: bool) -> Result<(), RunError> {
    if path.is_empty() {
        return Err(RunError::Internal("store info path filter must not be empty".to_string()));
    }
    let entries = store.list_pathinfos_with_layer().await.map_err(|error| RunError::Internal(format!("{error}")))?;
    let details: Vec<_> =
        entries.into_iter().filter(|entry| entry.value.store_path.to_string().contains(path)).collect();
    if details.is_empty() {
        return Err(RunError::Internal(format!("no PathInfo matching '{path}'")));
    }
    debug_assert!(u32::try_from(details.len()).is_ok());
    let overlay = store
        .overlay_report()
        .map_err(|error| RunError::Internal(format!("building overlay report: {error}")))?;
    if is_json_output {
        let paths = details
            .iter()
            .map(|detail| {
                let path_info = &detail.value;
                serde_json::json!({
                    "store_path": path_info.store_path.to_string(),
                    "layer": detail.layer,
                    "shadows": detail.shadows,
                    "nar_size": path_info.nar_size,
                    "nar_sha256": data_encoding::HEXLOWER.encode(path_info.nar_sha256.as_ref()),
                    "deriver": path_info.deriver.as_ref().map(ToString::to_string),
                    "references": path_info.references.iter().map(ToString::to_string).collect::<Vec<_>>(),
                    "signatures": path_info.signatures.iter().map(ToString::to_string).collect::<Vec<_>>(),
                    "ca": path_info.ca.as_ref().map(|value| format!("{value:?}")),
                    "node": format!("{:?}", path_info.node),
                })
            })
            .collect::<Vec<_>>();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema": "mantle-store-info-v2",
                "backend": store.backend().as_str(),
                "backend_capabilities": store.backend().profile(),
                "overlay": overlay,
                "paths": paths,
            }))
            .map_err(|error| RunError::Internal(format!("serializing store info: {error}")))?
        );
        return classify_store_effect(STORE_INFO_EFFECT, None);
    }
    print_overlay_summary(overlay.as_ref());
    let profile = store.backend().profile();
    println!("backend:    {}", store.backend().as_str());
    println!("capabilities:");
    for capability in profile.core {
        println!("  {capability}");
    }
    println!("  overlay-composition: {}", profile.overlay_composition);
    println!("  atomic-batch-import: {}", profile.atomic_batch_import);
    println!("  rust-unit-cache: {}", profile.rust_unit_cache);
    println!("  unsigned-admission: {}", profile.unsigned_admission);
    match profile.max_root_changes {
        Some(bound) => println!("  max-root-changes: {bound}"),
        None => println!("  max-root-changes: unbounded-by-backend"),
    }
    for detail in &details {
        let path_info = &detail.value;
        println!("store_path: {}", path_info.store_path);
        println!("layer:      {}", detail.layer);
        println!("shadows:    {:?}", detail.shadows);
        println!("nar_size:   {}", path_info.nar_size);
        println!("nar_sha256: {}", data_encoding::HEXLOWER.encode(path_info.nar_sha256.as_ref()));
        if let Some(deriver) = path_info.deriver.as_ref() {
            println!("deriver:    {deriver}");
        }
        if !path_info.references.is_empty() {
            println!("references:");
            for reference in &path_info.references {
                println!("  {reference}");
            }
        }
        if let Some(ca) = path_info.ca.as_ref() {
            println!("ca:         {ca:?}");
        }
        if path_info.signatures.is_empty() {
            println!("signatures: (none)");
        } else {
            println!("signatures:");
            for signature in &path_info.signatures {
                println!("  {signature}");
            }
        }
        println!("node:       {:?}", path_info.node);
    }
    classify_store_effect(STORE_INFO_EFFECT, None)
}

fn print_overlay_summary(report: Option<&crunch_store::StoreOverlayReport>) {
    let Some(report) = report else {
        eprintln!("store composition: single writable store");
        return;
    };
    eprintln!(
        "store composition: overlay plan={} no_backfill={} bases={}",
        report.plan_blake3,
        report.no_backfill,
        report.bases.len()
    );
    for base in &report.bases {
        eprintln!(
            "  base[{}] descriptor={} generation={} trust={} signers={}",
            base.declaration_index.saturating_add(1),
            base.descriptor_blake3,
            base.generation_blake3,
            base.trust_policy_id,
            base.accepted_signer_names.join(",")
        );
    }
}

fn cmd_store_roots(
    store: &mut crunch_store::StoreHandle,
    migrate: bool,
    context: StoreCommandContext<'_>,
) -> Result<(), RunError> {
    let roots = if migrate {
        store
            .store_admin()
            .migrate_legacy_root_registry()
            .map_err(|error| RunError::Internal(format!("{error}")))?
    } else {
        store.list_retained_roots().map_err(|error| RunError::Internal(format!("{error}")))?
    };
    let effect = if migrate {
        STORE_ROOTS_MIGRATE_EFFECT
    } else {
        STORE_ROOTS_LIST_EFFECT
    };
    if context.is_json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&roots)
                .map_err(|error| RunError::Internal(format!("serializing root report: {error}")))?
        );
        return classify_store_effect(effect, None);
    }
    if roots.is_empty() {
        eprintln!("No retained GC roots.");
        return classify_store_effect(effect, None);
    }
    for root in &roots {
        println!(
            "{}  class={}  owner={}  policy={}  source={}  created_unix_s={}  transition={}  transition_reason={}",
            root.logical_path,
            root.root_class,
            root.owner_scope,
            root.policy_blake3,
            root.source,
            root.created_unix_s,
            root.last_transition_id,
            root.last_transition_reason,
        );
    }
    eprintln!("{} retained root(s)", roots.len());
    classify_store_effect(effect, None)
}

#[derive(Serialize)]
struct StoreUsageOutput<'a> {
    usage: &'a crunch_store::GcUsageReport,
    reclaim_observations: &'a [crunch_store::GcReclaimObservation],
}

async fn cmd_store_usage(
    store: &mut crunch_store::StoreHandle,
    guard: Option<&crunch_store::StoreMutationGuard>,
    rust_retention: Option<&crunch_rust_cache::RustCacheRetentionPlan>,
    is_json_output: bool,
) -> Result<(), RunError> {
    let report =
        run_store_gc_plan(store, guard, None, rust_retention.map_or(&[][..], |plan| plan.live_nodes())).await?;
    if is_json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&StoreUsageOutput {
                usage: &report.usage,
                reclaim_observations: &report.reclaim_observations,
            })
            .map_err(|error| RunError::Internal(format!("serializing usage report: {error}")))?
        );
        return classify_store_effect(STORE_USAGE_EFFECT, None);
    }
    println!(
        "observed_bytes={}  retained_bytes={}  reclaimable_bytes={}  quarantined_bytes={}  unclassified_bytes={}  shared_bytes={}  unknown_objects={}",
        report.usage.observed_bytes,
        report.usage.retained_bytes,
        report.usage.reclaimable_bytes,
        report.usage.quarantined_bytes,
        report.usage.unclassified_bytes,
        report.usage.shared_bytes,
        report.usage.unknown_object_count,
    );
    for root in &report.usage.roots {
        println!(
            "ROOT {}  inclusive_bytes={}  unique_bytes={}  unknown_objects={}",
            root.root, root.inclusive_bytes, root.unique_bytes, root.unknown_object_count,
        );
    }
    for observation in &report.reclaim_observations {
        if let Some(blocker) = observation.blocker.as_deref() {
            println!("UNKNOWN_BYTES {}  category={}  blocker={}", observation.path, observation.category, blocker,);
        }
    }
    classify_store_effect(STORE_USAGE_EFFECT, None)
}
async fn cmd_store_pin(store: &crunch_store::StoreHandle, path: &str) -> Result<(), RunError> {
    let root = store.pin_retained_root(path).await.map_err(|e| RunError::Build(format!("{e}")))?;
    println!("PINNED {}  source={}  created_unix_s={}", root.logical_path, root.source, root.created_unix_s);
    classify_store_effect(STORE_PIN_EFFECT, None)
}

fn cmd_store_unpin(store: &crunch_store::StoreHandle, path: &str) -> Result<(), RunError> {
    let removed = store.unpin_retained_root(path).map_err(|e| RunError::Build(format!("{e}")))?;
    let Some(root) = removed else {
        return classify_store_effect(
            STORE_UNPIN_EFFECT,
            Some((STORE_UNPIN_UNKNOWN_ROOT_CODE, RunError::Build(format!("retained root not found: {path}")))),
        );
    };
    println!("UNPINNED {}", root.logical_path);
    classify_store_effect(STORE_UNPIN_EFFECT, None)
}

async fn run_store_gc_plan(
    store: &mut crunch_store::StoreHandle,
    guard: Option<&crunch_store::StoreMutationGuard>,
    accepted_plan_id: Option<&str>,
    retained_nodes: &[snix_castore::Node],
) -> Result<crunch_store::GcReport, RunError> {
    let report = if store.backend() == crunch_store::StoreBackend::Casita {
        let guard = guard
            .ok_or_else(|| RunError::Internal("Casita GC requires a selected store mutation guard".to_string()))?;
        store.garbage_collect_with_castore_roots_under_guard(guard, accepted_plan_id, retained_nodes).await
    } else {
        store.store_admin().garbage_collect_with_castore_roots(accepted_plan_id, retained_nodes).await
    };
    report.map_err(|error| RunError::Build(format!("{error}")))
}

async fn cmd_store_gc(
    store: &mut crunch_store::StoreHandle,
    guard: Option<&crunch_store::StoreMutationGuard>,
    rust_retention: Option<&crunch_rust_cache::RustCacheRetentionPlan>,
    accepted_plan_id: Option<&str>,
    is_json_output: bool,
) -> Result<(), RunError> {
    debug_assert!(!store.store_dir().is_empty());
    debug_assert!(Path::new(store.store_dir()).is_absolute());
    let is_plan_only = accepted_plan_id.is_none();
    let gc_evidence =
        run_store_gc_plan(store, guard, accepted_plan_id, rust_retention.map_or(&[][..], |plan| plan.live_nodes()))
            .await?;
    if let Some(plan) = rust_retention.filter(|_| is_plan_only || gc_evidence.execution_complete) {
        plan.apply(is_plan_only)
            .map_err(|error| RunError::Build(format!("applying Rust cache retention: {error}")))?;
    }
    if is_json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&gc_evidence)
                .map_err(|error| RunError::Internal(format!("serializing GC report: {error}")))?
        );
    } else {
        print_human_gc_report(&gc_evidence, rust_retention);
    }
    let incomplete_failure = (!is_plan_only && !gc_evidence.execution_complete).then(|| {
        (
            STORE_GC_INCOMPLETE_CODE,
            RunError::Build(format!(
                "GC execution completed {} operation(s) with {} failure(s); first failure: {}",
                gc_evidence.operations.len(),
                gc_evidence.failed_operations.len(),
                gc_evidence.failed_operation.as_deref().unwrap_or("unknown operation failure"),
            )),
        )
    });
    classify_store_effect(STORE_GC_EFFECT, incomplete_failure)
}

fn print_human_gc_report(
    report: &crunch_store::GcReport,
    rust_retention: Option<&crunch_rust_cache::RustCacheRetentionPlan>,
) {
    println!("plan_id={}  retention_plan_id={}", report.plan_id, report.retention_plan_id);
    if let Some(overlay_plan_blake3) = report.overlay_plan_blake3.as_deref() {
        println!("overlay_plan={}  retained_base_paths={}", overlay_plan_blake3, report.base_reachability.len());
        for path in &report.base_reachability {
            println!(
                "base_reachable: layer=base[{}] path={} retained_by={}",
                path.layer_index,
                path.path,
                path.retaining_roots.join(",")
            );
        }
    }
    println!(
        "retained_roots={}  retained_rust_results={}  stale_rust_results={}  candidate_paths={}  reclaimable_bytes={}",
        report.retained_root_count,
        report.retained_castore_root_count,
        rust_retention.map_or(0, |plan| plan.stale_result_count()),
        report.candidate_path_count,
        report.reclaimable_bytes_total,
    );
    println!(
        "observed_bytes={}  retained_bytes={}  shared_bytes={}  unknown_objects={}  unknown_reclaim_observations={}",
        report.usage.observed_bytes,
        report.usage.retained_bytes,
        report.usage.shared_bytes,
        report.usage.unknown_object_count,
        report.reclaim_observations.iter().filter(|observation| observation.bytes.is_none()).count(),
    );
    for explanation in &report.retention_explanations {
        println!(
            "ROOT {}  class={}  owner={}  policy={}  project={}  selector={}  generation={}  lease={}  transition={}  transition_reason={}  decision={}  reason={}",
            explanation.path,
            explanation.root_class,
            explanation.owner_scope,
            explanation.policy_blake3,
            explanation.project_identity.as_deref().unwrap_or("-"),
            explanation.selector.as_deref().unwrap_or("-"),
            explanation.generation.map_or_else(|| "-".to_string(), |value| value.to_string()),
            explanation.lease_id.as_deref().unwrap_or("-"),
            explanation.transition_id,
            explanation.transition_reason,
            explanation.disposition,
            explanation.reason,
        );
    }
    for path in &report.candidate_paths {
        println!("CANDIDATE {path}  reason=unreachable-from-retained-root");
    }
    for observation in &report.reclaim_observations {
        if let Some(blocker) = observation.blocker.as_deref() {
            println!("UNKNOWN_BYTES {}  category={}  blocker={}", observation.path, observation.category, blocker,);
        }
    }
    if report.is_dry_run {
        eprintln!("plan only: no changes made; execute with --execute --plan-id {}", report.plan_id);
    } else if report.execution_complete {
        eprintln!(
            "gc: removed {} candidate path(s) and {} stale Rust result record(s)",
            report.candidate_path_count,
            rust_retention.map_or(0, |plan| plan.stale_result_count()),
        );
    } else {
        eprintln!(
            "gc: incomplete after {} completed operation(s) and {} failure(s)",
            report.operations.len(),
            report.failed_operations.len(),
        );
        for failure in &report.failed_operations {
            eprintln!("gc operation failed: {failure}");
        }
    }
}

struct StoreVerifyRequest<'a> {
    path_filter: Option<&'a str>,
    signing_key_path: Option<&'a Path>,
    explicit_trusted_public_keys: &'a [String],
    backend: crunch_store::StoreBackend,
    is_trust_unsigned: bool,
    state_dir: &'a Path,
    /// Physical directory where build outputs are exported (the CLI `--store`
    /// value), not the logical store prefix.
    store_dir: &'a Path,
    /// Logical store prefix (the CLI `--store-prefix` value) used for
    /// signature fingerprints.
    store_prefix: &'a str,
}

async fn cmd_store_verify(
    svc: &dyn snix_store::pathinfoservice::PathInfoService,
    request: StoreVerifyRequest<'_>,
) -> Result<(), RunError> {
    let trusted_keys = resolve_store_verify_keys(
        request.signing_key_path,
        request.explicit_trusted_public_keys,
        request.state_dir,
        request.backend,
    )?;
    let hash_results = crunch_store::store_verify(svc, request.path_filter, request.store_dir)
        .await
        .map_err(|e| RunError::Internal(format!("{e}")))?;
    let signature_results =
        crunch_store::store_verify_signatures(svc, request.path_filter, &trusted_keys, request.store_prefix)
            .await
            .map_err(|e| RunError::Internal(format!("{e}")))?;
    let signature_by_path = index_signature_results(signature_results)?;
    debug_assert_eq!(hash_results.len(), signature_by_path.len());
    debug_assert!(u32::try_from(hash_results.len()).is_ok());
    let summary = print_store_verify_results(&hash_results, &signature_by_path, request.is_trust_unsigned)?;

    eprintln!("{} checked, {} mismatches", summary.checked, summary.mismatches);
    classify_store_verify(&summary)
}

/// Classify one finished verification: a mismatch is a failed observation.
///
/// The typed classification replaces the ad-hoc count check, so the failure
/// decision and its diagnostic code come from the contract vocabulary.
fn classify_store_verify(summary: &VerifySummary) -> Result<(), RunError> {
    let plan =
        mantle_application_contract::plan_effects(mantle_application_contract::CommandFamily::StoreAdministration, &[
            STORE_VERIFY_EFFECT,
        ])
        .ok_or_else(|| RunError::Internal("store verify effect plan exceeds its bound".to_string()))?;
    let observation = mantle_application_contract::Observation {
        effect_id: mantle_application_contract::EffectId(String::from(STORE_VERIFY_EFFECT)),
        status: if summary.mismatches == 0 {
            mantle_application_contract::ObservationStatus::Succeeded
        } else {
            mantle_application_contract::ObservationStatus::Failed
        },
        diagnostics_code: if summary.mismatches == 0 {
            None
        } else {
            Some(String::from(STORE_VERIFY_MISMATCH_CODE))
        },
    };
    match mantle_application_contract::classify_observations(&plan, &[observation]) {
        mantle_application_contract::ApplicationOutcome::Completed => Ok(()),
        mantle_application_contract::ApplicationOutcome::Failed { .. } => {
            Err(RunError::Build(format!("{} path(s) failed verification", summary.mismatches)))
        }
        other => Err(RunError::Internal(format!("store verify observations were inconsistent: {other:?}"))),
    }
}

struct VerifySummary {
    checked: u32,
    mismatches: u32,
}

/// Classify one finished store administration effect before success is reported.
///
/// Every store operation that reports after an effect shares this shape: one
/// planned effect, one observation of how it ended, and the exact failure the
/// command already used when the classification rejects the report.
fn classify_store_effect(effect: &str, failure: Option<(&str, RunError)>) -> Result<(), RunError> {
    let plan =
        mantle_application_contract::plan_effects(mantle_application_contract::CommandFamily::StoreAdministration, &[
            effect,
        ])
        .ok_or_else(|| RunError::Internal(format!("store {effect} effect plan exceeds its bound")))?;
    let observation = mantle_application_contract::Observation {
        effect_id: mantle_application_contract::EffectId(String::from(effect)),
        status: if failure.is_some() {
            mantle_application_contract::ObservationStatus::Failed
        } else {
            mantle_application_contract::ObservationStatus::Succeeded
        },
        diagnostics_code: failure.as_ref().map(|(code, _)| String::from(*code)),
    };
    match mantle_application_contract::classify_observations(&plan, &[observation]) {
        mantle_application_contract::ApplicationOutcome::Completed => Ok(()),
        mantle_application_contract::ApplicationOutcome::Failed { .. } => match failure {
            Some((_, error)) => Err(error),
            None => Err(RunError::Internal(format!("store {effect} classification failed without a recorded failure"))),
        },
        other => Err(RunError::Internal(format!("store {effect} observations were inconsistent: {other:?}"))),
    }
}

fn resolve_store_verify_keys(
    signing_key_path: Option<&std::path::Path>,
    explicit_trusted_public_keys: &[String],
    state_dir: &Path,
    backend: crunch_store::StoreBackend,
) -> Result<Vec<nix_compat::narinfo::VerifyingKey>, RunError> {
    let parsed_explicit_keys = parse_trusted_public_keys(explicit_trusted_public_keys)?;
    let configured_trusted_keys = load_configured_trusted_public_keys(parsed_explicit_keys.as_deref(), state_dir)?;
    if backend == crunch_store::StoreBackend::Casita && signing_key_path.is_none() {
        return Ok(configured_trusted_keys.unwrap_or_default());
    }
    let keypair = load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;
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
            stored_size,
            actual_size,
        } => {
            println!("MISMATCH {path}  stored={stored_hash}:{stored_size}  actual={actual_hash}:{actual_size}");
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
    let repair_was_required = inspection.is_repair_required();
    let report = if is_execute && repair_was_required {
        let keypair = load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;
        crunch_store::execute_final_nar_repair(store, inspection, &keypair.signing_key)
            .await
            .map_err(|error| RunError::Internal(error.to_string()))?
    } else {
        inspection.report(is_execute)
    };
    print_final_nar_repair_report(&report, is_json_output)?;
    let effect = if is_execute && repair_was_required {
        STORE_REPAIR_EXECUTE_EFFECT
    } else {
        STORE_REPAIR_INSPECT_EFFECT
    };
    classify_store_effect(effect, None)
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
    svc: &dyn snix_store::pathinfoservice::PathInfoService,
    path_filter: Option<&str>,
    is_sign_all: bool,
    signing_key_path: Option<&std::path::Path>,
    state_dir: &Path,
    store_prefix: &str,
) -> Result<(), RunError> {
    if path_filter.is_none() && !is_sign_all {
        return Err(RunError::Internal("provide a store path or use --all to sign all entries".to_string()));
    }
    debug_assert!(path_filter.is_some() || is_sign_all);
    debug_assert!(state_dir.components().next().is_some());

    let keypair = crate::signing_key::load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;

    let results = crunch_store::store_sign(svc, &keypair.signing_key, path_filter, is_sign_all, store_prefix)
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
    classify_store_effect(STORE_SIGN_EFFECT, None)
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
        return classify_store_effect(STORE_PUSH_SELECT_EFFECT, None);
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
    classify_store_effect(STORE_PUSH_EFFECT, None)
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
    let options = resolve_pull_options(&request, store.backend())?;
    let pull_evidence = execute_pull(store, pull_source, &request, &options).await?;
    print_pull_evidence(&pull_evidence);
    classify_pull_evidence(&pull_evidence)
}

/// Closure-pull failure decision: a closure pull that did not admit its root
/// under-delivered the one path the operator asked for, so it fails closed
/// instead of exiting zero with `admitted=false` buried in the report line.
fn closure_pull_failure(report: &crunch_store::HttpClosurePullReport) -> Option<(&'static str, RunError)> {
    (!report.root_admitted).then(|| {
        (
            STORE_PULL_ROOT_NOT_ADMITTED_CODE,
            RunError::Build(format!("closure pull did not admit root {}", report.plan.root)),
        )
    })
}

/// Classify one finished pull before the command reports success.
///
/// An explicit pull accounts for itself through its printed report; a closure
/// pull additionally proves its root was admitted.
fn classify_pull_evidence(evidence: &StorePullEvidence) -> Result<(), RunError> {
    match evidence {
        StorePullEvidence::Explicit(_) => classify_store_effect(STORE_PULL_EFFECT, None),
        StorePullEvidence::Closure(report) => classify_store_effect(STORE_PULL_EFFECT, closure_pull_failure(report)),
    }
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

fn resolve_pull_options(
    request: &StorePullRequest<'_>,
    backend: crunch_store::StoreBackend,
) -> Result<crunch_store::PullOptions, RunError> {
    let parsed_explicit = parse_trusted_public_keys(request.explicit_trusted_public_keys)?;
    let configured_keys = load_configured_trusted_public_keys(parsed_explicit.as_deref(), request.state_dir)?;
    let trusted_public_keys = if backend == crunch_store::StoreBackend::Casita {
        configured_keys.unwrap_or_default()
    } else {
        let keypair = load_or_generate_signing_keypair(None, request.state_dir, true)?;
        crunch_build::build_trusted_keys(&keypair, configured_keys.as_deref())
    };
    Ok(crunch_store::PullOptions {
        trust_unsigned: request.is_trust_unsigned,
        trusted_public_keys,
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

async fn cmd_store_composition(
    action: crate::StoreCompositionAction,
    context: StoreCommandContext<'_>,
) -> Result<(), RunError> {
    match action {
        crate::StoreCompositionAction::Plan { from } => {
            let request = read_composition_request(&from)?;
            let prepared = crunch_store::plan_composition_request(&request)
                .map_err(|error| RunError::Internal(error.to_string()))?;
            print_composition_plan(&prepared, context.is_json_output)?;
            classify_store_effect(STORE_COMPOSITION_PLAN_EFFECT, None)
        }
        crate::StoreCompositionAction::Realize { from, receipt_out } => {
            let request = read_composition_request(&from)?;
            let _guard = store_mutation_guard(context)?;
            let store = open_store_under_guard(context, &_guard).await?;
            let receipt = crunch_store::realize_composition(&store, &request)
                .await
                .map_err(|error| RunError::Internal(error.to_string()))?;
            crate::source_bundle::write_json_atomically(&receipt_out, &receipt, "composition realization receipt")?;
            print_composition_receipt(&receipt, context.is_json_output)?;
            classify_store_effect(STORE_COMPOSITION_REALIZE_EFFECT, None)
        }
    }
}

fn read_composition_request(path: &Path) -> Result<crunch_composition_core::CompositionRequest, RunError> {
    let file = fs::File::open(path)
        .map_err(|error| RunError::Internal(format!("opening composition request {}: {error}", path.display())))?;
    let mut bytes = Vec::with_capacity(COMPOSITION_REQUEST_INITIAL_CAPACITY);
    file.take(COMPOSITION_REQUEST_READ_LIMIT)
        .read_to_end(&mut bytes)
        .map_err(|error| RunError::Internal(format!("reading composition request {}: {error}", path.display())))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > COMPOSITION_REQUEST_BYTES_MAX {
        return Err(RunError::Internal(format!("composition request exceeds {COMPOSITION_REQUEST_BYTES_MAX} bytes")));
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| RunError::Internal(format!("parsing composition request {}: {error}", path.display())))
}

fn print_composition_plan(
    prepared: &crunch_composition_core::PreparedComposition,
    is_json_output: bool,
) -> Result<(), RunError> {
    if is_json_output {
        return print_json_report(prepared, "serializing composition plan");
    }
    println!(
        "COMPOSITION_PLAN plan_ref={} policy_ref={} bindings={} decisions={} experimental=true",
        prepared.plan_ref,
        prepared.realization_policy_ref,
        prepared.bindings.len(),
        prepared.collision_decisions.len(),
    );
    Ok(())
}

fn print_composition_receipt(
    receipt: &crunch_composition_core::RealizationReceipt,
    is_json_output: bool,
) -> Result<(), RunError> {
    if is_json_output {
        return print_json_report(receipt, "serializing composition receipt");
    }
    println!(
        "COMPOSITION_ROOT plan_ref={} policy_ref={} root={} receipt_ref={} experimental=true",
        receipt.plan_ref, receipt.realization_policy_ref, receipt.resulting_root.digest_blake3, receipt.receipt_ref,
    );
    Ok(())
}

async fn cmd_store_archive(
    action: crate::StoreArchiveAction,
    context: StoreCommandContext<'_>,
) -> Result<(), RunError> {
    match action {
        crate::StoreArchiveAction::Export {
            format,
            to,
            all,
            trust_unsigned,
            paths,
        } => {
            if format == crate::StoreArchiveFormat::NarioV2 {
                return Err(RunError::Internal(
                    "nario-v2 export is unsupported; list and import are read-only compatibility operations"
                        .to_string(),
                ));
            }
            let store = open_store(context).await?;
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
            format,
            from,
            trust_unsigned,
            trusted_public_keys,
            no_materialize,
        } => {
            let _guard = store_mutation_guard(context)?;
            let store = open_store_under_guard(context, &_guard).await?;
            cmd_store_archive_import(&store, StoreArchiveImportRequest {
                format,
                source: &from,
                is_trust_unsigned: trust_unsigned,
                explicit_trusted_public_keys: &trusted_public_keys,
                is_materialize: !no_materialize,
                state_dir: context.state_dir,
                is_json_output: context.is_json_output,
            })
            .await
        }
        crate::StoreArchiveAction::List { format, from } => {
            cmd_store_archive_list(format, &from, context.is_json_output).await
        }
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
        return classify_store_effect(STORE_ARCHIVE_EXPORT_EFFECT, None);
    }
    print_archive_export_report(&archive_write_summary, request.is_json_output)?;
    classify_store_effect(STORE_ARCHIVE_EXPORT_EFFECT, None)
}

struct StoreArchiveImportRequest<'a> {
    format: crate::StoreArchiveFormat,
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
    let trusted_public_keys =
        resolve_store_verify_keys(None, request.explicit_trusted_public_keys, request.state_dir, store.backend())?;
    debug_assert!(u32::try_from(trusted_public_keys.len()).is_ok());
    if request.format == crate::StoreArchiveFormat::NarioV2
        && trusted_public_keys.len() > crunch_store::NARIO_V2_TRUSTED_KEYS_MAX
    {
        return Err(RunError::Internal(format!(
            "Nario trusted key count exceeds {}",
            crunch_store::NARIO_V2_TRUSTED_KEYS_MAX
        )));
    }
    if request.format == crate::StoreArchiveFormat::NarioV2 {
        let options = crunch_store::NarioV2ImportOptions {
            trust_unsigned: request.is_trust_unsigned,
            trusted_public_keys,
            materialize: request.is_materialize,
        };
        let report = if is_stdio_path(request.source) {
            let mut stdin = tokio::io::stdin();
            crunch_store::import_nario_v2(store, &mut stdin, &options).await
        } else {
            let file = tokio::fs::File::open(request.source)
                .await
                .map_err(|e| RunError::Internal(format!("opening archive {}: {e}", request.source.display())))?;
            let mut reader = tokio::io::BufReader::new(file);
            crunch_store::import_nario_v2(store, &mut reader, &options).await
        }
        .map_err(|e| RunError::Internal(format!("nario-v2 archive import: {e}")))?;
        print_nario_import_report(&report, request.is_json_output)?;
        return classify_store_effect(STORE_ARCHIVE_IMPORT_EFFECT, None);
    }
    let options = crunch_store::ArchiveImportOptions {
        trust_unsigned: request.is_trust_unsigned,
        trusted_public_keys,
        materialize: request.is_materialize,
    };
    let report = if is_stdio_path(request.source) {
        let mut stdin = tokio::io::stdin();
        crunch_store::import_store_archive(store, &mut stdin, &options).await
    } else {
        let file = tokio::fs::File::open(request.source)
            .await
            .map_err(|e| RunError::Internal(format!("opening archive {}: {e}", request.source.display())))?;
        let mut reader = tokio::io::BufReader::new(file);
        crunch_store::import_store_archive(store, &mut reader, &options).await
    }
    .map_err(|e| RunError::Internal(format!("archive import: {e}")))?;
    print_archive_import_report(&report, request.is_json_output)?;
    classify_store_effect(STORE_ARCHIVE_IMPORT_EFFECT, None)
}

async fn cmd_store_archive_list(
    format: crate::StoreArchiveFormat,
    source: &Path,
    is_json_output: bool,
) -> Result<(), RunError> {
    if format == crate::StoreArchiveFormat::NarioV2 {
        let report = if is_stdio_path(source) {
            let mut stdin = tokio::io::stdin();
            crunch_store::list_nario_v2(&mut stdin).await
        } else {
            let file = tokio::fs::File::open(source)
                .await
                .map_err(|e| RunError::Internal(format!("opening archive {}: {e}", source.display())))?;
            let mut reader = tokio::io::BufReader::new(file);
            crunch_store::list_nario_v2(&mut reader).await
        }
        .map_err(|e| RunError::Internal(format!("nario-v2 archive list: {e}")))?;
        print_nario_list_report(&report, is_json_output)?;
        return classify_store_effect(STORE_ARCHIVE_LIST_EFFECT, None);
    }
    let report = if is_stdio_path(source) {
        let mut stdin = tokio::io::stdin();
        crunch_store::list_store_archive(&mut stdin).await
    } else {
        let file = tokio::fs::File::open(source)
            .await
            .map_err(|e| RunError::Internal(format!("opening archive {}: {e}", source.display())))?;
        let mut reader = tokio::io::BufReader::new(file);
        crunch_store::list_store_archive(&mut reader).await
    }
    .map_err(|e| RunError::Internal(format!("archive list: {e}")))?;
    print_archive_list_report(&report, is_json_output)?;
    classify_store_effect(STORE_ARCHIVE_LIST_EFFECT, None)
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

fn print_nario_import_report(report: &crunch_store::NarioV2ImportReport, is_json_output: bool) -> Result<(), RunError> {
    if is_json_output {
        return print_json_report(report, "serializing Nario v2 import report");
    }
    for path in &report.paths {
        println!("NARIO_IMPORT {} nar_size={} nar_sha256={}", path.store_path, path.nar_size, path.nar_sha256_hex);
    }
    eprintln!(
        "format={} producer_revision={} imported={} skipped_present={} nar_bytes={} archive_blake3={}",
        report.format,
        report.producer_revision,
        report.imported_count,
        report.skipped_already_present_count,
        report.total_nar_bytes,
        report.archive_blake3
    );
    Ok(())
}

fn print_nario_list_report(report: &crunch_store::NarioV2ListReport, is_json_output: bool) -> Result<(), RunError> {
    if is_json_output {
        return print_json_report(report, "serializing Nario v2 list report");
    }
    println!(
        "format={} producer={} revision={} direction={} store_prefix={} records={} nar_bytes={} archive_blake3={}",
        report.format,
        report.producer_version,
        report.producer_revision,
        report.supported_direction,
        report.store_prefix,
        report.record_count,
        report.total_nar_bytes,
        report.archive_blake3
    );
    for path in &report.paths {
        println!(
            "NARIO_PATH {} nar_size={} refs={} signatures={} ca={}",
            path.store_path,
            path.nar_size,
            path.references.len(),
            path.signatures.len(),
            path.ca.as_deref().unwrap_or("-")
        );
    }
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
    store
        .store_list_pathinfos_bounded(PATHINFO_SCAN_COUNT_MAX)
        .await
        .map_err(|err| RunError::Internal(err.to_string()))
}

async fn collect_matching_pathinfos(
    store: &crunch_store::StoreHandle,
    selectors: &[String],
) -> Result<Vec<snix_store::path_info::PathInfo>, RunError> {
    if selectors.is_empty() || selectors.iter().any(String::is_empty) {
        return Err(RunError::Internal("matching path selectors must be non-empty".to_string()));
    }
    debug_assert!(!selectors.is_empty());
    debug_assert!(selectors.iter().all(|selector| !selector.is_empty()));
    let all = collect_all_pathinfos(store).await?;
    let results = all
        .into_iter()
        .filter(|path_info| {
            let store_path = path_info.store_path.to_string();
            selectors.iter().any(|selector| store_path.contains(selector.as_str()))
        })
        .collect();
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clean_verification_classifies_as_completed() {
        let summary = VerifySummary {
            checked: 3,
            mismatches: 0,
        };
        assert!(classify_store_verify(&summary).is_ok());
        assert_eq!(STORE_VERIFY_EFFECT, "verify-paths");
    }

    #[test]
    fn a_mismatched_verification_reports_the_existing_message() {
        let summary = VerifySummary {
            checked: 3,
            mismatches: 2,
        };
        let error = classify_store_verify(&summary).expect_err("mismatches fail verification");
        let rendered = format!("{error}");
        assert!(rendered.contains("2 path(s) failed verification"));
        assert_eq!(STORE_VERIFY_MISMATCH_CODE, "store-verify-mismatch");
    }

    #[test]
    fn a_succeeded_store_effect_classifies_as_completed() {
        assert!(classify_store_effect(STORE_GC_EFFECT, None).is_ok());
        assert!(classify_store_effect(STORE_PULL_EFFECT, None).is_ok());
    }

    #[test]
    fn a_failed_store_effect_returns_its_recorded_failure() {
        let error = classify_store_effect(
            STORE_UNPIN_EFFECT,
            Some((
                STORE_UNPIN_UNKNOWN_ROOT_CODE,
                RunError::Build("retained root not found: /mantle/store/demo".to_string()),
            )),
        )
        .expect_err("a failed observation rejects the report");
        assert!(format!("{error}").contains("retained root not found: /mantle/store/demo"));
    }

    #[test]
    fn an_admitted_closure_pull_reports_no_failure() {
        let report = closure_pull_report(true);
        assert!(closure_pull_failure(&report).is_none());
    }

    #[test]
    fn a_rejected_closure_root_fails_closed_with_the_root_named() {
        let report = closure_pull_report(false);
        let (code, error) = closure_pull_failure(&report).expect("an unadmitted root fails closed");
        assert_eq!(code, STORE_PULL_ROOT_NOT_ADMITTED_CODE);
        assert!(format!("{error}").contains("closure pull did not admit root /mantle/store/0demo"));
    }

    fn closure_pull_report(root_admitted: bool) -> crunch_store::HttpClosurePullReport {
        crunch_store::HttpClosurePullReport {
            plan: crunch_store::HttpClosurePlan {
                schema: "mantle-http-closure-plan-v1".to_string(),
                cache_identity: "cache".to_string(),
                trust_policy_blake3: "trust".to_string(),
                store_dir: "/mantle/store".to_string(),
                root: "/mantle/store/0demo".to_string(),
                limits: crunch_store::HttpClosureLimits::default(),
                total_nar_bytes: 0,
                members: Vec::new(),
                plan_blake3: "plan".to_string(),
            },
            pull: crunch_store::PullReport {
                imported_count: 0,
                skipped_already_present_count: 0,
                skipped_untrusted_count: 0,
                skipped_hash_mismatch_count: 0,
                skipped_missing_nar_count: 0,
                skipped_parse_error_count: 0,
                skipped_store_dir_mismatch_count: 0,
                total_nar_bytes: 0,
                paths: Vec::new(),
            },
            reused_complete_count: 0,
            root_admitted,
        }
    }
}
