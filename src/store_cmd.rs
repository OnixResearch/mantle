// machine-artifact-public: store.command-reports
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use mantle_application_contract::store_administration::StoreExecutionPort;
use mantle_application_contract::store_administration::StoreFactFailure;
use mantle_application_contract::store_administration::StoreFacts;
use mantle_application_contract::store_administration::StoreLeaf;
use mantle_application_contract::store_administration::StorePlan;
use mantle_application_contract::store_administration::classify_store_facts;
use serde::Serialize;

const PATHINFO_SCAN_COUNT_MAX: u32 = 1_000_000;
const PATHINFO_SCAN_COUNT_MAX_USIZE: usize = 1_000_000;
const MAX_FOREIGN_RECEIPT_ROOT_PATHS: usize = 64;
const MAX_FOREIGN_REALIZATION_RECEIPT_BYTES: u64 = 16_777_216;
const FOREIGN_REALIZATION_RECEIPT_READ_LIMIT: u64 = MAX_FOREIGN_REALIZATION_RECEIPT_BYTES + 1;
const COMPOSITION_REQUEST_BYTES_MAX: u64 = 1_048_576;
const COMPOSITION_REQUEST_READ_LIMIT: u64 = COMPOSITION_REQUEST_BYTES_MAX + 1;
const COMPOSITION_REQUEST_INITIAL_CAPACITY: usize = 8_192;
const NAR_SHA256_BYTES: usize = 32;
const NAR_SHA256_HEX_BYTES: usize = NAR_SHA256_BYTES * 2;
const BLAKE3_HEX_BYTES: usize = 64;

use crate::errors::RunError;
use crate::signing_key::load_configured_trusted_public_keys;
use crate::signing_key::load_or_generate_signing_keypair;

#[derive(Clone, Copy)]
struct StoreCommandPlan {
    operation: StorePlan,
    selection: Option<StorePlan>,
}

fn plan_store_command(action: &crate::StoreAction) -> StoreCommandPlan {
    use crate::StoreAction as Action;
    let leaf = match action {
        Action::List => StoreLeaf::List,
        Action::Info { .. } => StoreLeaf::Info,
        Action::Roots { migrate } => StoreLeaf::Roots { migrate: *migrate },
        Action::Usage => StoreLeaf::Usage,
        Action::Pin { .. } => StoreLeaf::Pin,
        Action::Unpin { .. } => StoreLeaf::Unpin,
        Action::Gc { execute, .. } => StoreLeaf::Gc { execute: *execute },
        Action::Verify { .. } => StoreLeaf::Verify,
        Action::Sign { .. } => StoreLeaf::Sign,
        Action::RepairFinalNar { execute, .. } => StoreLeaf::Repair { execute: *execute },
        Action::Push { .. } => StoreLeaf::Push,
        Action::Pull { from, closure, .. } => StoreLeaf::Pull {
            closure: *closure,
            http: from.starts_with("http://") || from.starts_with("https://"),
        },
        Action::Composition { action } => match action {
            crate::StoreCompositionAction::Plan { .. } => StoreLeaf::CompositionPlan,
            crate::StoreCompositionAction::Realize { .. } => StoreLeaf::CompositionRealize,
        },
        Action::Archive { action } => match action {
            crate::StoreArchiveAction::Export { to, .. } => StoreLeaf::ArchiveExport {
                stdio: is_stdio_path(to),
            },
            crate::StoreArchiveAction::Import { format, from, .. } => StoreLeaf::ArchiveImport {
                nario: *format == crate::StoreArchiveFormat::NarioV2,
                stdio: is_stdio_path(from),
            },
            crate::StoreArchiveAction::List { format, from } => StoreLeaf::ArchiveList {
                nario: *format == crate::StoreArchiveFormat::NarioV2,
                stdio: is_stdio_path(from),
            },
        },
    };
    let selection = match leaf {
        StoreLeaf::Push => Some(StorePlan::new(StoreLeaf::PushSelection)),
        StoreLeaf::ArchiveExport { .. } => Some(StorePlan::new(StoreLeaf::ArchiveSelection)),
        _ => None,
    };
    StoreCommandPlan {
        operation: StorePlan::new(leaf),
        selection,
    }
}

struct StoreCapability<F> {
    leaf: StoreLeaf,
    operation: F,
}

impl<F, T> StoreExecutionPort for StoreCapability<F>
where F: core::future::Future<Output = Result<T, RunError>>
{
    type Output = T;
    type Error = RunError;

    async fn execute(self, plan: &StorePlan) -> Result<T, RunError> {
        if self.leaf != plan.leaf {
            return Err(RunError::Internal(format!("store execution port does not match plan: {:?}", plan.leaf)));
        }
        self.operation.await
    }
}

async fn execute_store_port<T>(
    plan: StorePlan,
    leaf: StoreLeaf,
    operation: impl core::future::Future<Output = Result<T, RunError>>,
) -> Result<T, RunError> {
    StoreCapability { leaf, operation }.execute(&plan).await
}

fn classify_store_result(plan: StorePlan, facts: StoreFacts) -> Result<(), RunError> {
    classify_store_facts(plan, facts)
        .map_err(|error| RunError::Internal(format!("store {:?} observations were inconsistent: {error:?}", plan.leaf)))
}

pub fn cmd_store(
    action: crate::StoreAction,
    output_dir: &Path,
    state_dir: &Path,
    backend: crunch_store::StoreBackend,
    store_dir: &str,
    base_state_dirs: &[PathBuf],
    source_admission: Option<&std::sync::Arc<dyn crunch_store::StoreSourceAdmission>>,
    is_json_output: bool,
) -> Result<(), RunError> {
    crate::command_input::admit_store_action_or_block(&action)?;
    let plan = plan_store_command(&action);
    let rt = tokio::runtime::Runtime::new().map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;
    rt.block_on(async {
        cmd_store_async(action, StoreCommandContext {
            output_dir,
            state_dir,
            backend,
            store_dir,
            base_state_dirs,
            source_admission,
            is_json_output,
            plan,
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
    source_admission: Option<&'a std::sync::Arc<dyn crunch_store::StoreSourceAdmission>>,
    is_json_output: bool,
    plan: StoreCommandPlan,
}

async fn cmd_store_async(action: crate::StoreAction, context: StoreCommandContext<'_>) -> Result<(), RunError> {
    match action {
        crate::StoreAction::List => {
            let store = open_store(context).await?;
            cmd_store_list(&store, context.plan.operation, context.is_json_output).await
        }
        crate::StoreAction::Info { path } => {
            let store = open_store(context).await?;
            cmd_store_info(&store, context.plan.operation, &path, context.is_json_output).await
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
            cmd_store_roots(&mut store, migrate, context).await
        }
        crate::StoreAction::Usage => {
            preflight_store_backend(context)?;
            let _guard = if context.backend == crunch_store::StoreBackend::Casita {
                Some(
                    crunch_store::StoreMutationGuard::try_acquire(context.state_dir)
                        .map_err(|error| RunError::Build(format!("{error}")))?,
                )
            } else {
                None
            };
            let casita_store = if context.backend == crunch_store::StoreBackend::Casita {
                let mut store = open_store(context).await?;
                recover_pending_casita_gc(&mut store, context, _guard.as_ref()).await?;
                Some(store)
            } else {
                None
            };
            // Snix's cache handle must close its writable directory DB before
            // the command store opens; Casita recovery must precede verification.
            let rust_retention = plan_rust_cache_retention(context).await?;
            let mut store = match casita_store {
                Some(store) => store,
                None => open_store(context).await?,
            };
            cmd_store_usage(
                &mut store,
                context.plan.operation,
                _guard.as_ref(),
                rust_retention.as_ref(),
                context.is_json_output,
            )
            .await
        }
        other => cmd_store_mutation_or_transfer(other, context).await,
    }
}

async fn cmd_store_mutation_or_transfer(
    action: crate::StoreAction,
    context: StoreCommandContext<'_>,
) -> Result<(), RunError> {
    match action {
        crate::StoreAction::Pin { path, owner, reason } => {
            let _guard = store_mutation_guard(context)?;
            let store = open_store_under_guard(context, &_guard).await?;
            cmd_store_pin(&store, context.plan.operation, &path, &owner, &reason).await
        }
        crate::StoreAction::Unpin { path, owner, reason } => {
            let _guard = store_mutation_guard(context)?;
            let store = open_store_under_guard(context, &_guard).await?;
            cmd_store_unpin(&store, context.plan.operation, &path, &owner, reason.as_deref()).await
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
            let mut store = open_store_under_guard(context, &_guard).await?;
            let admin = store.store_admin();
            cmd_store_sign(
                &admin,
                context.plan.operation,
                path.as_deref(),
                all,
                signing_key.as_deref(),
                context.state_dir,
            )
            .await
        }
        crate::StoreAction::RepairFinalNar {
            path,
            execute,
            signing_key,
        } => {
            if !context.backend.profile().core.contains(&"store-repair-final-nar") {
                return Err(RunError::Build(format!("{}-repair-final-nar-unsupported", context.backend.as_str())));
            }
            let _guard = store_mutation_guard(context)?;
            let store = open_store_under_guard(context, &_guard).await?;
            cmd_store_repair_final_nar(
                &store,
                context.plan.operation,
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
    let _guard = if execute || context.backend == crunch_store::StoreBackend::Casita {
        Some(
            crunch_store::StoreMutationGuard::try_acquire(context.state_dir)
                .map_err(|error| RunError::Build(format!("{error}")))?,
        )
    } else {
        None
    };
    let casita_store = if context.backend == crunch_store::StoreBackend::Casita {
        let mut store = open_store(context).await?;
        recover_pending_casita_gc(&mut store, context, _guard.as_ref()).await?;
        Some(store)
    } else {
        None
    };
    // Snix's cache handle must close its writable directory DB before the
    // command store opens; Casita recovery must precede verification.
    let rust_retention = plan_rust_cache_retention(context).await?;
    let mut store = match casita_store {
        Some(store) => store,
        None => open_store(context).await?,
    };
    cmd_store_gc(
        &mut store,
        context.plan.operation,
        _guard.as_ref(),
        rust_retention.as_ref(),
        accepted_plan_id,
        context.is_json_output,
    )
    .await
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

async fn plan_rust_cache_retention(
    context: StoreCommandContext<'_>,
) -> Result<Option<crunch_rust_cache::RustCacheRetentionPlan>, RunError> {
    if !rust_cache_state_present(context.state_dir)
        .map_err(|error| RunError::Build(format!("inspecting Rust cache state: {error}")))?
    {
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
        source_admission: None,
    })
    .await
    .map_err(|error| RunError::Build(format!("planning Rust cache retention: {error}")))?;
    let plan = if context.backend == crunch_store::StoreBackend::Casita {
        rust_cache.plan_retention_gc_verified().await
    } else {
        rust_cache.plan_retention_gc()
    }
    .map_err(|error| RunError::Build(format!("planning Rust cache retention: {error}")))?;
    drop(rust_cache);
    Ok(Some(plan))
}

fn preflight_store_backend(context: StoreCommandContext<'_>) -> Result<(), RunError> {
    crunch_store::StoreConfig::preflight_backend_identity_for(
        context.backend,
        context.state_dir,
        context.store_dir,
        context.base_state_dirs,
        context.source_admission.map(std::sync::Arc::as_ref),
    )
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
    let mut store = open_store(context).await?;
    let admin = store.store_admin();
    cmd_store_verify(&admin, StoreVerifyRequest {
        plan: context.plan.operation,
        path_filter: action.path_filter.as_deref(),
        signing_key_path: action.signing_key_path.as_deref(),
        explicit_trusted_public_keys: &action.trusted_public_keys,
        backend: context.backend,
        is_trust_unsigned: action.is_trust_unsigned,
        state_dir: context.state_dir,
        physical_output_dir: context.output_dir,
    })
    .await
}

async fn cmd_store_push_action(context: StoreCommandContext<'_>, action: StorePushAction) -> Result<(), RunError> {
    let _guard = store_mutation_guard(context)?;
    let store = open_store_under_guard(context, &_guard).await?;
    cmd_store_push(&store, context.plan, StorePushRequest {
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
    cmd_store_pull(&store, context.plan.operation, StorePullRequest {
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
        source_admission: context.source_admission.cloned(),
    })
    .await
    .map_err(|e| RunError::Internal(format!("opening store: {e}")))
}

async fn cmd_store_list(
    store: &crunch_store::StoreHandle,
    plan: StorePlan,
    is_json_output: bool,
) -> Result<(), RunError> {
    let entries = execute_store_port(plan, StoreLeaf::List, async {
        store.list_pathinfos_with_layer().await.map_err(|error| RunError::Internal(format!("{error}")))
    })
    .await?;
    debug_assert!(entries.iter().all(|entry| !entry.value.store_path.to_string().is_empty()));
    debug_assert!(u32::try_from(entries.len()).is_ok());
    let overlay = store
        .overlay_report()
        .map_err(|error| RunError::Internal(format!("building overlay report: {error}")))?;
    classify_store_result(plan, StoreFacts::Rows {
        rows: entries.len(),
        invalid_rows: entries.iter().filter(|entry| entry.value.store_path.name().is_empty()).count(),
    })?;
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
        return Ok(());
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
    Ok(())
}

async fn cmd_store_info(
    store: &crunch_store::StoreHandle,
    plan: StorePlan,
    path: &str,
    is_json_output: bool,
) -> Result<(), RunError> {
    if path.is_empty() {
        return Err(RunError::Internal("store info path filter must not be empty".to_string()));
    }
    let entries = execute_store_port(plan, StoreLeaf::Info, async {
        store.list_pathinfos_with_layer().await.map_err(|error| RunError::Internal(format!("{error}")))
    })
    .await?;
    let details: Vec<_> =
        entries.into_iter().filter(|entry| entry.value.store_path.to_string().contains(path)).collect();
    if details.is_empty() {
        return Err(RunError::Internal(format!("no PathInfo matching '{path}'")));
    }
    debug_assert!(u32::try_from(details.len()).is_ok());
    let overlay = store
        .overlay_report()
        .map_err(|error| RunError::Internal(format!("building overlay report: {error}")))?;
    let interests = crunch_store::list_interest_reports(store.state_dir())
        .map_err(|error| RunError::Internal(format!("listing retention interests: {error}")))?;
    classify_store_result(plan, StoreFacts::Rows {
        rows: details.len(),
        invalid_rows: details.iter().filter(|entry| !entry.value.store_path.to_string().contains(path)).count(),
    })?;
    if is_json_output {
        let paths = details
            .iter()
            .map(|detail| {
                let path_info = &detail.value;
                let logical_path = path_info.store_path.to_absolute_path_with_prefix(store.store_dir());
                let retention_interests = interests.iter().find(|entry| entry.logical_path == logical_path);
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
                    "retention_interests": retention_interests,
                })
            })
            .collect::<Vec<_>>();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema": "mantle-store-info-v3",
                "backend": store.backend().as_str(),
                "backend_capabilities": store.backend().profile(),
                "overlay": overlay,
                "paths": paths,
            }))
            .map_err(|error| RunError::Internal(format!("serializing store info: {error}")))?
        );
        return Ok(());
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
        let logical_path = path_info.store_path.to_absolute_path_with_prefix(store.store_dir());
        if let Some(path_interests) = interests.iter().find(|entry| entry.logical_path == logical_path) {
            println!("retention_record_count: {}", path_interests.record_count);
            println!("retention_legacy_unmanaged: {}", path_interests.legacy_unmanaged);
            for fact in &path_interests.interests {
                println!(
                    "  retention owner={} reason={} identity={}",
                    fact.owner,
                    fact.reason,
                    fact.record_id.as_deref().unwrap_or("-")
                );
            }
        }
    }
    Ok(())
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

async fn cmd_store_roots(
    store: &mut crunch_store::StoreHandle,
    migrate: bool,
    context: StoreCommandContext<'_>,
) -> Result<(), RunError> {
    let plan = context.plan.operation;
    let roots = execute_store_port(plan, StoreLeaf::Roots { migrate }, async {
        if migrate {
            store
                .store_admin()
                .migrate_legacy_root_registry()
                .map_err(|error| RunError::Internal(format!("{error}")))
        } else {
            store.list_retained_roots().map_err(|error| RunError::Internal(format!("{error}")))
        }
    })
    .await?;
    let interests = crunch_store::list_interest_reports(context.state_dir)
        .map_err(|error| RunError::Internal(format!("listing retention interests: {error}")))?;
    let readback = store.list_retained_roots().map_err(|error| RunError::Internal(format!("{error}")))?;
    classify_store_result(plan, StoreFacts::Roots {
        rows: roots.len(),
        invalid_rows: roots.iter().filter(|root| root.logical_path.is_empty() || root.owner_scope.is_empty()).count(),
        readback_matches: roots == readback,
    })?;
    if context.is_json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema": "mantle-store-roots-v1",
                "roots": roots,
                "retention_interests": interests,
            }))
            .map_err(|error| RunError::Internal(format!("serializing root report: {error}")))?
        );
        return Ok(());
    }
    if roots.is_empty() {
        eprintln!("No retained GC roots.");
        return Ok(());
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
        if let Some(path) = interests.iter().find(|entry| entry.logical_path == root.logical_path) {
            println!("  record_count={} legacy_unmanaged={}", path.record_count, path.legacy_unmanaged);
            for fact in &path.interests {
                println!(
                    "  INTEREST owner={} reason={} identity={}",
                    fact.owner,
                    fact.reason,
                    fact.record_id.as_deref().unwrap_or("-")
                );
            }
        }
    }
    eprintln!("{} retained root(s)", roots.len());
    Ok(())
}

#[derive(Serialize)]
struct StoreUsageOutput<'a> {
    schema: &'static str,
    retention_interests: &'a [crunch_store::GcPathInterests],
    usage: &'a crunch_store::GcUsageReport,
    reclaim_observations: &'a [crunch_store::GcReclaimObservation],
}

async fn cmd_store_usage(
    store: &mut crunch_store::StoreHandle,
    plan: StorePlan,
    guard: Option<&crunch_store::StoreMutationGuard>,
    rust_retention: Option<&crunch_rust_cache::RustCacheRetentionPlan>,
    is_json_output: bool,
) -> Result<(), RunError> {
    let report = execute_store_port(
        plan,
        StoreLeaf::Usage,
        run_store_gc_plan(store, guard, None, rust_retention.map_or(&[][..], |plan| plan.live_nodes())),
    )
    .await?;
    let interests = crunch_store::list_interest_reports(store.state_dir())
        .map_err(|error| RunError::Internal(format!("listing retention interests: {error}")))?;
    let usage = &report.usage;
    classify_store_result(plan, StoreFacts::Usage {
        observed: usage.observed_bytes,
        classified: usage
            .retained_bytes
            .checked_add(usage.reclaimable_bytes)
            .and_then(|bytes| bytes.checked_add(usage.quarantined_bytes))
            .and_then(|bytes| bytes.checked_add(usage.unclassified_bytes)),
    })?;
    if is_json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&StoreUsageOutput {
                schema: "mantle-store-usage-v1",
                retention_interests: &interests,
                usage: &report.usage,
                reclaim_observations: &report.reclaim_observations,
            })
            .map_err(|error| RunError::Internal(format!("serializing usage report: {error}")))?
        );
        return Ok(());
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
    for interest in &interests {
        println!(
            "INTERESTS {} record_count={} legacy_unmanaged={}",
            interest.logical_path, interest.record_count, interest.legacy_unmanaged
        );
        for fact in &interest.interests {
            println!(
                "  owner={} reason={} identity={}",
                fact.owner,
                fact.reason,
                fact.record_id.as_deref().unwrap_or("-")
            );
        }
    }
    for observation in &report.reclaim_observations {
        if let Some(blocker) = observation.blocker.as_deref() {
            println!("UNKNOWN_BYTES {}  category={}  blocker={}", observation.path, observation.category, blocker,);
        }
    }
    Ok(())
}
async fn cmd_store_pin(
    store: &crunch_store::StoreHandle,
    plan: StorePlan,
    path: &str,
    owner: &str,
    reason: &str,
) -> Result<(), RunError> {
    let root = execute_store_port(plan, StoreLeaf::Pin, async {
        store.pin_retained_root(path, owner, reason).await.map_err(|e| RunError::Build(format!("{e}")))
    })
    .await?;
    let interests = crunch_store::list_interest_reports(store.state_dir())
        .map_err(|error| RunError::Internal(format!("listing retention interests: {error}")))?;
    let matching_after = interests
        .iter()
        .filter(|entry| entry.logical_path == root.logical_path)
        .flat_map(|entry| &entry.interests)
        .filter(|fact| fact.owner == root.owner_scope && fact.reason == reason)
        .count();
    classify_store_result(plan, StoreFacts::Retention {
        found: root.logical_path == path,
        matching_after,
    })?;
    println!("PINNED {}  source={}  created_unix_s={}", root.logical_path, root.source, root.created_unix_s);
    Ok(())
}

async fn cmd_store_unpin(
    store: &crunch_store::StoreHandle,
    plan: StorePlan,
    path: &str,
    owner: &str,
    reason: Option<&str>,
) -> Result<(), RunError> {
    let removed = execute_store_port(plan, StoreLeaf::Unpin, async {
        store.unpin_retained_root(path, owner, reason).map_err(|e| RunError::Build(format!("{e}")))
    })
    .await?;
    let Some(root) = removed else {
        return match classify_store_facts(plan, StoreFacts::Retention {
            found: false,
            matching_after: 0,
        }) {
            Err(StoreFactFailure::MissingRoot) => Err(RunError::Build(format!("retained root not found: {path}"))),
            other => Err(RunError::Internal(format!("store unpin observations were inconsistent: {other:?}"))),
        };
    };
    let interests = crunch_store::list_interest_reports(store.state_dir())
        .map_err(|error| RunError::Internal(format!("listing retention interests: {error}")))?;
    let matching_after = interests
        .iter()
        .filter(|entry| entry.logical_path == root.logical_path)
        .flat_map(|entry| &entry.interests)
        .filter(|fact| fact.owner == root.owner_scope && reason.is_none_or(|reason| fact.reason == reason))
        .count();
    classify_store_result(plan, StoreFacts::Retention {
        found: root.logical_path == path,
        matching_after,
    })?;
    println!("UNPINNED {}", root.logical_path);
    Ok(())
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
    plan: StorePlan,
    guard: Option<&crunch_store::StoreMutationGuard>,
    rust_retention: Option<&crunch_rust_cache::RustCacheRetentionPlan>,
    accepted_plan_id: Option<&str>,
    is_json_output: bool,
) -> Result<(), RunError> {
    debug_assert!(!store.store_dir().is_empty());
    debug_assert!(Path::new(store.store_dir()).is_absolute());
    let is_plan_only = accepted_plan_id.is_none();
    let gc_evidence = execute_store_port(
        plan,
        StoreLeaf::Gc { execute: !is_plan_only },
        run_store_gc_plan(store, guard, accepted_plan_id, rust_retention.map_or(&[][..], |plan| plan.live_nodes())),
    )
    .await?;
    if let Some(plan) = rust_retention.filter(|_| is_plan_only || gc_evidence.execution_complete) {
        plan.apply(is_plan_only)
            .map_err(|error| RunError::Build(format!("applying Rust cache retention: {error}")))?;
    }
    let classification = classify_store_facts(plan, StoreFacts::Gc {
        dry_run: gc_evidence.is_dry_run,
        plan_matches: accepted_plan_id
            .map_or(!gc_evidence.plan_id.is_empty(), |accepted| accepted == gc_evidence.plan_id),
        candidates: gc_evidence.candidate_path_count,
        candidate_rows: gc_evidence.candidate_paths.len(),
        complete: gc_evidence.execution_complete,
        failures: gc_evidence.failed_operations.len(),
    });
    let failure = match classification {
        Ok(()) => None,
        Err(StoreFactFailure::IncompleteGc) => Some(RunError::Build(format!(
            "GC execution completed {} operation(s) with {} failure(s); first failure: {}",
            gc_evidence.operations.len(),
            gc_evidence.failed_operations.len(),
            gc_evidence.failed_operation.as_deref().unwrap_or("unknown operation failure"),
        ))),
        Err(error) => return Err(RunError::Internal(format!("store GC observations were inconsistent: {error:?}"))),
    };
    if is_json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&gc_evidence)
                .map_err(|error| RunError::Internal(format!("serializing GC report: {error}")))?
        );
    } else {
        print_human_gc_report(&gc_evidence, rust_retention);
    }
    failure.map_or(Ok(()), Err)
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
    plan: StorePlan,
    path_filter: Option<&'a str>,
    signing_key_path: Option<&'a Path>,
    explicit_trusted_public_keys: &'a [String],
    backend: crunch_store::StoreBackend,
    is_trust_unsigned: bool,
    state_dir: &'a Path,
    /// Physical CLI output directory; unlike the logical fingerprint prefix,
    /// it may not be representable as UTF-8.
    physical_output_dir: &'a Path,
}

async fn cmd_store_verify(
    admin: &crunch_store::StoreAdmin<'_>,
    request: StoreVerifyRequest<'_>,
) -> Result<(), RunError> {
    let trusted_keys = resolve_store_verify_keys(
        request.signing_key_path,
        request.explicit_trusted_public_keys,
        request.state_dir,
        request.backend,
    )?;
    let (hash_results, signature_results) = execute_store_port(request.plan, StoreLeaf::Verify, async {
        let hashes = admin
            .verify_exported_pathinfos(request.path_filter, request.physical_output_dir)
            .await
            .map_err(|e| RunError::Internal(format!("{e}")))?;
        let signatures = admin
            .verify_pathinfo_signatures(request.path_filter, &trusted_keys)
            .await
            .map_err(|e| RunError::Internal(format!("{e}")))?;
        Ok((hashes, signatures))
    })
    .await?;
    let signature_by_path = index_signature_results(signature_results)?;
    let summary = summarize_store_verify_results(&hash_results, &signature_by_path, request.is_trust_unsigned)?;
    let failure = match classify_store_verify(request.plan, &summary, signature_by_path.len()) {
        Ok(()) => None,
        Err(error @ RunError::Build(_)) => Some(error),
        Err(error) => return Err(error),
    };
    print_store_verify_results(&hash_results, &signature_by_path, request.is_trust_unsigned)?;
    eprintln!("{} checked, {} mismatches", summary.checked, summary.mismatches);
    failure.map_or(Ok(()), Err)
}

fn classify_store_verify(plan: StorePlan, summary: &VerifySummary, signatures: usize) -> Result<(), RunError> {
    match classify_store_facts(plan, StoreFacts::Verify {
        checked: summary.checked,
        signatures,
        mismatches: summary.mismatches,
    }) {
        Ok(()) => Ok(()),
        Err(StoreFactFailure::VerificationMismatch) => {
            Err(RunError::Build(format!("{} path(s) failed verification", summary.mismatches)))
        }
        Err(other) => Err(RunError::Internal(format!("store verify observations were inconsistent: {other:?}"))),
    }
}

fn summarize_store_verify_results(
    hash_results: &[crunch_store::VerifyResult],
    signatures: &std::collections::HashMap<String, crunch_store::SignatureVerifyResult>,
    trust_unsigned: bool,
) -> Result<VerifySummary, RunError> {
    let checked = u32::try_from(hash_results.len())
        .map_err(|_| RunError::Internal("store verification result count exceeds u32".to_string()))?;
    let mut mismatches = 0;
    for result in hash_results {
        let mismatched = match result {
            crunch_store::VerifyResult::Ok(path) => {
                let signature = signatures
                    .get(path)
                    .ok_or_else(|| RunError::Internal(format!("missing signature result for {path}")))?;
                !trust_unsigned && !signature.is_trusted()
            }
            crunch_store::VerifyResult::Missing(_) | crunch_store::VerifyResult::Mismatch { .. } => true,
        };
        if mismatched {
            mismatches += 1;
        }
    }
    Ok(VerifySummary { checked, mismatches })
}

struct VerifySummary {
    checked: u32,
    mismatches: u32,
}

fn nar_hash_matches(hash: &[u8; NAR_SHA256_BYTES], encoded: &str) -> bool {
    let mut expected = [0_u8; NAR_SHA256_HEX_BYTES];
    data_encoding::HEXLOWER.encode_mut(hash, &mut expected);
    encoded.as_bytes() == expected.as_slice()
}

async fn readback_path_metadata<'a>(
    lookup: &crunch_store::OutputLookup,
    build_service: &crunch_store::BuildServiceStore,
    paths: impl IntoIterator<Item = (&'a str, u64, &'a str)>,
) -> Result<bool, RunError> {
    for (identity, nar_size, nar_hash) in paths {
        let parsed = if identity.starts_with('/') {
            nix_compat::store_path::StorePath::from_absolute_path_with_prefix(identity.as_bytes(), lookup.store_dir())
        } else {
            nix_compat::store_path::StorePath::from_bytes(identity.as_bytes())
        };
        let Ok(path) = parsed else {
            return Ok(false);
        };
        let persisted = lookup
            .find(&path)
            .await
            .map_err(|error| RunError::Internal(format!("reading store result {identity}: {error}")))?;
        let Some(persisted) = persisted else {
            return Ok(false);
        };
        if persisted.store_path != path
            || persisted.nar_size != nar_size
            || !nar_hash_matches(&persisted.nar_sha256, nar_hash)
        {
            return Ok(false);
        }
        if !build_service
            .has_complete_content(&persisted)
            .await
            .map_err(|error| RunError::Internal(format!("reading store content {identity}: {error}")))?
        {
            return Ok(false);
        }
    }
    Ok(true)
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
) -> Result<(), RunError> {
    for result in hash_results {
        print_store_verify_result(result, signature_by_path, is_trust_unsigned)?;
    }
    Ok(())
}

fn print_store_verify_result(
    result: &crunch_store::VerifyResult,
    signature_by_path: &std::collections::HashMap<String, crunch_store::SignatureVerifyResult>,
    is_trust_unsigned: bool,
) -> Result<(), RunError> {
    match result {
        crunch_store::VerifyResult::Ok(path) => print_verified_ok(path, signature_by_path, is_trust_unsigned),
        crunch_store::VerifyResult::Missing(path) => {
            println!("MISSING {path}");
            Ok(())
        }
        crunch_store::VerifyResult::Mismatch {
            path,
            stored_hash,
            actual_hash,
            stored_size,
            actual_size,
        } => {
            println!("MISMATCH {path}  stored={stored_hash}:{stored_size}  actual={actual_hash}:{actual_size}");
            Ok(())
        }
    }
}

fn print_verified_ok(
    path: &str,
    signature_by_path: &std::collections::HashMap<String, crunch_store::SignatureVerifyResult>,
    is_trust_unsigned: bool,
) -> Result<(), RunError> {
    let sig_result = signature_by_path
        .get(path)
        .ok_or_else(|| RunError::Internal(format!("missing signature result for {path}")))?;
    debug_assert_eq!(sig_result.path, path);
    debug_assert!(sig_result.trusted_count <= sig_result.total_signatures);
    if is_trust_unsigned {
        println!("OK {path}  signatures=skipped");
        return Ok(());
    }
    if sig_result.is_trusted() {
        println!("OK {path}  trusted_signatures={}/{}", sig_result.trusted_count, sig_result.total_signatures);
        return Ok(());
    }
    if sig_result.total_signatures == 0 {
        println!("UNSIGNED {path}");
        return Ok(());
    }
    let untrusted = sig_result.untrusted_names.join(",");
    println!("UNTRUSTED {path}  trusted_signatures=0/{}  signers={}", sig_result.total_signatures, untrusted);
    Ok(())
}

async fn cmd_store_repair_final_nar(
    store: &crunch_store::StoreHandle,
    plan: StorePlan,
    logical_store_path: &str,
    is_execute: bool,
    signing_key_path: Option<&Path>,
    state_dir: &Path,
    is_json_output: bool,
) -> Result<(), RunError> {
    assert!(!logical_store_path.is_empty(), "repair path must not be empty");
    assert!(!store.store_dir().is_empty(), "logical store prefix must not be empty");
    let report_and_required = execute_store_port(plan, StoreLeaf::Repair { execute: is_execute }, async {
        let inspection = crunch_store::inspect_final_nar_repair(store, logical_store_path)
            .await
            .map_err(|error| RunError::Internal(error.to_string()))?;
        let required = inspection.is_repair_required();
        let report = if is_execute && required {
            let keypair = load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;
            crunch_store::execute_final_nar_repair(store, inspection, &keypair.signing_key)
                .await
                .map_err(|error| RunError::Internal(error.to_string()))?
        } else {
            inspection.report(is_execute)
        };
        Ok((report, required))
    })
    .await?;
    let (report, required) = report_and_required;
    let readback = crunch_store::inspect_final_nar_repair(store, logical_store_path)
        .await
        .map_err(|error| RunError::Internal(error.to_string()))?;
    classify_store_result(plan, StoreFacts::Repair {
        path_matches: report.store_path == logical_store_path,
        execution_requested: report.execution_requested,
        required,
        mutated: report.mutated,
        readback_required: readback.is_repair_required(),
        signatures_match: readback.report(false).old_signature_count == report.new_signature_count,
    })?;
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
    admin: &crunch_store::StoreAdmin<'_>,
    plan: StorePlan,
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

    let keypair = crate::signing_key::load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;

    let results = execute_store_port(plan, StoreLeaf::Sign, async {
        admin
            .sign_pathinfos(&keypair.signing_key, path_filter, is_sign_all)
            .await
            .map_err(|e| RunError::Internal(format!("{e}")))
    })
    .await?;
    let signatures = admin
        .verify_pathinfo_signatures(path_filter, std::slice::from_ref(&keypair.verifying_key))
        .await
        .map_err(|e| RunError::Internal(format!("{e}")))?;
    let trusted_paths: std::collections::HashSet<_> = signatures
        .into_iter()
        .filter(|signature| signature.is_trusted())
        .map(|signature| signature.path)
        .collect();
    let trusted_after = results.iter().filter(|result| trusted_paths.contains(&result.store_path)).count();
    let transitions = results
        .iter()
        .filter(|result| {
            usize::from(result.newly_signed) + usize::from(result.appended) + usize::from(result.replaced) == 1
        })
        .count();
    classify_store_result(plan, StoreFacts::Sign {
        rows: results.len(),
        transitions,
        trusted_after,
    })?;

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

async fn cmd_store_push(
    store: &crunch_store::StoreHandle,
    plans: StoreCommandPlan,
    request: StorePushRequest<'_>,
) -> Result<(), RunError> {
    if request.paths.is_empty() && !request.is_all {
        return Err(RunError::Internal("provide store paths or use --all".to_string()));
    }
    debug_assert!(request.is_all || !request.paths.is_empty());
    debug_assert!(Path::new(store.store_dir()).is_absolute());

    let selection = plans.selection.ok_or_else(|| RunError::Internal("push selection plan missing".to_string()))?;
    let selected = execute_store_port(selection, StoreLeaf::PushSelection, async {
        if request.is_all {
            collect_all_pathinfos(store).await
        } else {
            collect_matching_pathinfos(store, request.paths).await
        }
    })
    .await?;
    classify_store_result(selection, StoreFacts::Rows {
        rows: selected.len(),
        invalid_rows: selected.iter().filter(|path| path.store_path.name().is_empty()).count(),
    })?;

    if selected.is_empty() {
        eprintln!("no matching paths found");
        return Ok(());
    }

    let options = crunch_store::PushOptions {
        trust_unsigned: request.is_trust_unsigned,
    };
    let push_evidence = execute_store_port(plans.operation, StoreLeaf::Push, async {
        crunch_store::export_paths_to_cache_dir(store, &selected, request.destination_dir, &options)
            .await
            .map_err(|e| RunError::Internal(format!("push: {e}")))
    })
    .await?;
    let selected_by_digest: std::collections::HashMap<_, _> =
        selected.iter().map(|path| (path.store_path.digest(), path)).collect();
    classify_store_result(plans.operation, StoreFacts::Push {
        selected: selected.len(),
        pushed: push_evidence.pushed_count,
        skipped_unsigned: push_evidence.skipped_unsigned_count,
        skipped_present: push_evidence.skipped_already_present_count,
        rows: push_evidence.paths.len(),
        invalid_rows: push_evidence
            .paths
            .iter()
            .filter(|row| {
                let Ok(path) = nix_compat::store_path::StorePath::from_bytes(row.store_path.as_bytes()) else {
                    return true;
                };
                let Some(selected) = selected_by_digest.get(&path.digest()) else {
                    return true;
                };
                selected.store_path != path
                    || selected.nar_size != row.nar_size
                    || !nar_hash_matches(&selected.nar_sha256, &row.nar_hash_hex)
            })
            .count(),
    })?;

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

async fn cmd_store_pull(
    store: &crunch_store::StoreHandle,
    plan: StorePlan,
    request: StorePullRequest<'_>,
) -> Result<(), RunError> {
    let pull_source = validate_pull_request(&request)?;
    let options = resolve_pull_options(&request, store.backend())?;
    let leaf = StoreLeaf::Pull {
        closure: request.is_closure,
        http: matches!(&pull_source, crunch_store::PullSource::Http(_)),
    };
    let pull_evidence = execute_store_port(plan, leaf, execute_pull(store, pull_source, &request, &options)).await?;
    let (report, root_admitted, expected_paths) = match &pull_evidence {
        StorePullEvidence::Explicit(report) => {
            (report, None, matches!(leaf, StoreLeaf::Pull { http: true, .. }).then_some(request.paths.len()))
        }
        StorePullEvidence::Closure(report) => {
            (&report.pull, Some(report.root_admitted), Some(report.plan.members.len()))
        }
    };
    let readback_matches = readback_path_metadata(
        &store.output_lookup(),
        &store.build_service_store(),
        report
            .paths
            .iter()
            .map(|path| (path.store_path.as_str(), path.nar_size, path.nar_hash_hex.as_str())),
    )
    .await?;
    let classification = classify_store_facts(plan, StoreFacts::Pull {
        imported: report.imported_count,
        rows: report.paths.len(),
        skipped: u64::from(report.skipped_already_present_count)
            + u64::from(report.skipped_untrusted_count)
            + u64::from(report.skipped_hash_mismatch_count)
            + u64::from(report.skipped_missing_nar_count)
            + u64::from(report.skipped_parse_error_count)
            + u64::from(report.skipped_store_dir_mismatch_count),
        expected_paths,
        payload_bytes: report.total_nar_bytes,
        row_bytes: report.paths.iter().try_fold(0_u64, |sum, path| sum.checked_add(path.nar_size)),
        invalid_rows: report.paths.iter().filter(|path| path.store_path.is_empty()).count(),
        readback_matches,
        root_admitted,
    });
    let failure = match classification {
        Ok(()) => None,
        Err(StoreFactFailure::ClosureRootNotAdmitted) => match &pull_evidence {
            StorePullEvidence::Closure(report) => {
                Some(RunError::Build(format!("closure pull did not admit root {}", report.plan.root)))
            }
            StorePullEvidence::Explicit(_) => unreachable!("explicit pull cannot produce a closure classification"),
        },
        Err(error) => return Err(RunError::Internal(format!("store pull observations were inconsistent: {error:?}"))),
    };
    print_pull_evidence(&pull_evidence);
    failure.map_or(Ok(()), Err)
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
            let plan = context.plan.operation;
            let (request, prepared) = execute_store_port(plan, StoreLeaf::CompositionPlan, async {
                let request = read_composition_request(&from)?;
                let prepared = crunch_store::plan_composition_request(&request)
                    .map_err(|error| RunError::Internal(error.to_string()))?;
                Ok((request, prepared))
            })
            .await?;
            classify_store_result(plan, StoreFacts::Composition {
                requested_bindings: request.plan.bindings.len(),
                bindings: prepared.bindings.len(),
                requested_decisions: request.plan.collision_decisions.len(),
                decisions: prepared.collision_decisions.len(),
                identity_matches: !prepared.plan_ref.is_empty() && !prepared.realization_policy_ref.is_empty(),
                readback_matches: prepared.realization_policy == request.realization_policy,
            })?;
            print_composition_plan(&prepared, context.is_json_output)
        }
        crate::StoreCompositionAction::Realize { from, receipt_out } => {
            let plan = context.plan.operation;
            let request = read_composition_request(&from)?;
            let prepared = crunch_store::plan_composition_request(&request)
                .map_err(|error| RunError::Internal(error.to_string()))?;
            let _guard = store_mutation_guard(context)?;
            let store = open_store_under_guard(context, &_guard).await?;
            let receipt = execute_store_port(plan, StoreLeaf::CompositionRealize, async {
                let receipt = crunch_store::realize_composition(&store, &request)
                    .await
                    .map_err(|error| RunError::Internal(error.to_string()))?;
                crate::source_bundle::write_json_atomically(&receipt_out, &receipt, "composition realization receipt")?;
                Ok(receipt)
            })
            .await?;
            let receipt_bytes = fs::read(&receipt_out)
                .map_err(|error| RunError::Internal(format!("reading composition realization receipt: {error}")))?;
            let persisted: crunch_composition_core::RealizationReceipt = serde_json::from_slice(&receipt_bytes)
                .map_err(|error| RunError::Internal(format!("parsing composition realization receipt: {error}")))?;
            let digest_bytes = data_encoding::HEXLOWER
                .decode(receipt.resulting_root.digest_blake3.as_bytes())
                .map_err(|error| RunError::Internal(format!("reading composition root digest: {error}")))?;
            let digest = snix_castore::B3Digest::try_from(digest_bytes.as_slice())
                .map_err(|error| RunError::Internal(format!("reading composition root digest: {error}")))?;
            let root = snix_castore::Node::Directory {
                digest,
                size: receipt.resulting_root.size,
            };
            let complete = store
                .castore_has_complete_content(&root)
                .await
                .map_err(|error| RunError::Internal(format!("reading composition root: {error}")))?;
            classify_store_result(plan, StoreFacts::Composition {
                requested_bindings: request.plan.bindings.len(),
                bindings: prepared.bindings.len(),
                requested_decisions: request.plan.collision_decisions.len(),
                decisions: prepared.collision_decisions.len(),
                identity_matches: receipt.plan_ref == prepared.plan_ref
                    && receipt.realization_policy_ref == prepared.realization_policy_ref,
                readback_matches: receipt == persisted && complete,
            })?;
            print_composition_receipt(&receipt, context.is_json_output)
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
            cmd_store_archive_export(&store, context.plan, StoreArchiveExportRequest {
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
            cmd_store_archive_import(&store, context.plan.operation, StoreArchiveImportRequest {
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
            cmd_store_archive_list(context.plan.operation, format, &from, context.is_json_output).await
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
    plans: StoreCommandPlan,
    request: StoreArchiveExportRequest<'_>,
) -> Result<(), RunError> {
    if request.paths.is_empty() && !request.is_all {
        return Err(RunError::Internal("provide store paths or use --all".to_string()));
    }
    let selection = plans.selection.ok_or_else(|| RunError::Internal("archive selection plan missing".to_string()))?;
    let selected = execute_store_port(selection, StoreLeaf::ArchiveSelection, async {
        if request.is_all {
            collect_all_pathinfos(store).await
        } else {
            collect_matching_pathinfos(store, request.paths).await
        }
    })
    .await?;
    classify_store_result(selection, StoreFacts::Rows {
        rows: selected.len(),
        invalid_rows: selected.iter().filter(|path| path.store_path.name().is_empty()).count(),
    })?;
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
    let archive_write_summary = execute_store_port(
        plans.operation,
        StoreLeaf::ArchiveExport {
            stdio: is_archive_stdout,
        },
        async {
            if is_archive_stdout {
                let mut stdout = tokio::io::stdout();
                crunch_store::export_store_archive(store, &selected, &mut stdout, &options)
                    .await
                    .map_err(|e| RunError::Internal(format!("archive export: {e}")))
            } else {
                let file = tokio::fs::File::create(request.destination).await.map_err(|e| {
                    RunError::Internal(format!("creating archive {}: {e}", request.destination.display()))
                })?;
                let mut writer = tokio::io::BufWriter::new(file);
                crunch_store::export_store_archive(store, &selected, &mut writer, &options)
                    .await
                    .map_err(|e| RunError::Internal(format!("archive export: {e}")))
            }
        },
    )
    .await?;
    // The exporter flushes its writer before returning. Reconcile its actual
    // record/payload accounting with the selected inputs; do not reopen output
    // streams or impose a new read-permission requirement on write-only files.
    let exported_paths: std::collections::HashMap<_, _> =
        archive_write_summary.paths.iter().map(|path| (path.store_path.as_str(), path)).collect();
    let selected_content_matches = selected.iter().all(|selected| {
        exported_paths.get(selected.store_path.to_string().as_str()).is_some_and(|exported| {
            exported.nar_size == selected.nar_size && nar_hash_matches(&selected.nar_sha256, &exported.nar_sha256_hex)
        })
    });
    classify_store_result(plans.operation, StoreFacts::Archive {
        reported: archive_write_summary.exported_count,
        rows: archive_write_summary.paths.len(),
        invalid_rows: archive_write_summary.paths.iter().filter(|path| path.store_path.is_empty()).count(),
        payload_bytes: archive_write_summary.total_payload_bytes,
        row_bytes: archive_write_summary.paths.iter().try_fold(0_u64, |sum, path| sum.checked_add(path.nar_size)),
        content_matches: selected_content_matches,
    })?;
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
    plan: StorePlan,
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
        let report = execute_store_port(
            plan,
            StoreLeaf::ArchiveImport {
                nario: true,
                stdio: is_stdio_path(request.source),
            },
            async {
                let result = if is_stdio_path(request.source) {
                    let mut stdin = tokio::io::stdin();
                    crunch_store::import_nario_v2(store, &mut stdin, &options).await
                } else {
                    let file = tokio::fs::File::open(request.source).await.map_err(|e| {
                        RunError::Internal(format!("opening archive {}: {e}", request.source.display()))
                    })?;
                    let mut reader = tokio::io::BufReader::new(file);
                    crunch_store::import_nario_v2(store, &mut reader, &options).await
                };
                result.map_err(|e| RunError::Internal(format!("nario-v2 archive import: {e}")))
            },
        )
        .await?;
        let readback_matches = readback_path_metadata(
            &store.output_lookup(),
            &store.build_service_store(),
            report
                .paths
                .iter()
                .map(|path| (path.store_path.as_str(), path.nar_size, path.nar_sha256_hex.as_str())),
        )
        .await?;
        let reported = report
            .imported_count
            .checked_add(report.skipped_already_present_count)
            .ok_or_else(|| RunError::Internal("Nario import count overflow".to_string()))?;
        classify_store_result(plan, StoreFacts::Archive {
            reported,
            rows: report.paths.len(),
            invalid_rows: report.paths.iter().filter(|path| !path.store_path.starts_with(report.store_prefix)).count(),
            payload_bytes: report.total_nar_bytes,
            row_bytes: report.paths.iter().try_fold(0_u64, |sum, path| sum.checked_add(path.nar_size)),
            content_matches: readback_matches,
        })?;
        return print_nario_import_report(&report, request.is_json_output);
    }
    let options = crunch_store::ArchiveImportOptions {
        trust_unsigned: request.is_trust_unsigned,
        trusted_public_keys,
        materialize: request.is_materialize,
    };
    let report = execute_store_port(
        plan,
        StoreLeaf::ArchiveImport {
            nario: false,
            stdio: is_stdio_path(request.source),
        },
        async {
            let result = if is_stdio_path(request.source) {
                let mut stdin = tokio::io::stdin();
                crunch_store::import_store_archive(store, &mut stdin, &options).await
            } else {
                let file = tokio::fs::File::open(request.source)
                    .await
                    .map_err(|e| RunError::Internal(format!("opening archive {}: {e}", request.source.display())))?;
                let mut reader = tokio::io::BufReader::new(file);
                crunch_store::import_store_archive(store, &mut reader, &options).await
            };
            result.map_err(|e| RunError::Internal(format!("archive import: {e}")))
        },
    )
    .await?;
    let readback_matches = readback_path_metadata(
        &store.output_lookup(),
        &store.build_service_store(),
        report
            .paths
            .iter()
            .map(|path| (path.store_path.as_str(), path.nar_size, path.nar_sha256_hex.as_str())),
    )
    .await?;
    classify_store_result(plan, StoreFacts::Archive {
        reported: report.imported_count,
        rows: report.paths.len(),
        invalid_rows: report.paths.iter().filter(|path| path.store_path.is_empty()).count(),
        payload_bytes: report.total_payload_bytes,
        row_bytes: report.paths.iter().try_fold(0_u64, |sum, path| sum.checked_add(path.nar_size)),
        content_matches: readback_matches,
    })?;
    print_archive_import_report(&report, request.is_json_output)
}

async fn cmd_store_archive_list(
    plan: StorePlan,
    format: crate::StoreArchiveFormat,
    source: &Path,
    is_json_output: bool,
) -> Result<(), RunError> {
    if format == crate::StoreArchiveFormat::NarioV2 {
        let report = execute_store_port(
            plan,
            StoreLeaf::ArchiveList {
                nario: true,
                stdio: is_stdio_path(source),
            },
            async {
                let result = if is_stdio_path(source) {
                    let mut stdin = tokio::io::stdin();
                    crunch_store::list_nario_v2(&mut stdin).await
                } else {
                    let file = tokio::fs::File::open(source)
                        .await
                        .map_err(|e| RunError::Internal(format!("opening archive {}: {e}", source.display())))?;
                    let mut reader = tokio::io::BufReader::new(file);
                    crunch_store::list_nario_v2(&mut reader).await
                };
                result.map_err(|e| RunError::Internal(format!("nario-v2 archive list: {e}")))
            },
        )
        .await?;
        classify_store_result(plan, StoreFacts::Archive {
            reported: report.record_count,
            rows: report.paths.len(),
            invalid_rows: report.paths.iter().filter(|path| !path.store_path.starts_with(report.store_prefix)).count(),
            payload_bytes: report.total_nar_bytes,
            row_bytes: report.paths.iter().try_fold(0_u64, |sum, path| sum.checked_add(path.nar_size)),
            content_matches: report.archive_blake3.len() == BLAKE3_HEX_BYTES,
        })?;
        return print_nario_list_report(&report, is_json_output);
    }
    let report = execute_store_port(
        plan,
        StoreLeaf::ArchiveList {
            nario: false,
            stdio: is_stdio_path(source),
        },
        async {
            let result = if is_stdio_path(source) {
                let mut stdin = tokio::io::stdin();
                crunch_store::list_store_archive(&mut stdin).await
            } else {
                let file = tokio::fs::File::open(source)
                    .await
                    .map_err(|e| RunError::Internal(format!("opening archive {}: {e}", source.display())))?;
                let mut reader = tokio::io::BufReader::new(file);
                crunch_store::list_store_archive(&mut reader).await
            };
            result.map_err(|e| RunError::Internal(format!("archive list: {e}")))
        },
    )
    .await?;
    classify_store_result(plan, StoreFacts::Archive {
        reported: report.record_count,
        rows: report.paths.len(),
        invalid_rows: report
            .paths
            .iter()
            .filter(|path| {
                path.store_path.is_empty()
                    || usize::try_from(path.reference_count).ok() != Some(path.references.len())
                    || usize::try_from(path.signature_count).ok() != Some(path.signatures.len())
            })
            .count(),
        payload_bytes: report.total_payload_bytes,
        row_bytes: report.paths.iter().try_fold(0_u64, |sum, path| sum.checked_add(path.nar_size)),
        content_matches: !report.store_prefix.is_empty(),
    })?;
    print_archive_list_report(&report, is_json_output)
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
    fn unsigned_verification_changes_with_explicit_trust_policy() {
        let hashes = [
            crunch_store::VerifyResult::Ok("path".to_string()),
            crunch_store::VerifyResult::Missing("missing".to_string()),
        ];
        let signatures = std::collections::HashMap::from([("path".to_string(), crunch_store::SignatureVerifyResult {
            path: "path".to_string(),
            trusted_count: 0,
            total_signatures: 0,
            untrusted_names: Vec::new(),
        })]);
        let strict = summarize_store_verify_results(&hashes, &signatures, false).unwrap();
        let unsigned = summarize_store_verify_results(&hashes, &signatures, true).unwrap();
        assert_eq!((strict.checked, strict.mismatches), (2, 2));
        assert_eq!((unsigned.checked, unsigned.mismatches), (2, 1));
    }

    #[test]
    fn missing_signature_evidence_cannot_be_overridden_by_trust_unsigned() {
        let hashes = [crunch_store::VerifyResult::Ok("path".to_string())];
        let signatures = std::collections::HashMap::new();
        assert!(summarize_store_verify_results(&hashes, &signatures, true).is_err());
    }
}
