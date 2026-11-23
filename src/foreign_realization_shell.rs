//! Imperative shell for receipt-bound foreign realization.
// r[impl foreign_derivation_import.realization_adapter]
// r[impl foreign_derivation_import.source_materialization]
// r[impl foreign_derivation_import.cache_only_runtime_closure]
// r[impl foreign_derivation_import.live_nixpkgs_realization_proof]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

use crunch_build::DerivationRegistry;
use crunch_build::ExecutionProfile;
use crunch_build::FetchSourceOverride;
use crunch_build::FetchSourceOverrideKind;
use crunch_build::FetchSourcePolicy;
use crunch_build::HermeticityMode;
use crunch_build::KeyPair;
use crunch_build::PreferenceField;
use crunch_pipeline::BuildConfig;
use crunch_pipeline::RegisteredBuildRequest;
use crunch_pipeline::RegisteredOutputExpectation;
use crunch_pipeline::SchedulingPolicy;
use crunch_store::StoreHandle;
use crunch_store::VerifiedSourceIngestRequest;
use data_encoding::HEXLOWER;
use nix_compat::derivation::Derivation;
use nix_compat::narinfo::SigningKey;
use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;
use url::Url;

use crate::RunError;
use crate::foreign_derivation_import::ImportReceipt;
use crate::foreign_executable_plan::ExecutableNativeUnit;
use crate::foreign_executable_plan::ForeignExecutablePlan;
use crate::foreign_realization::AdmittedForeignSource;
use crate::foreign_realization::BuilderKey;
use crate::foreign_realization::ForeignCacheClosurePolicy;
use crate::foreign_realization::ForeignRealizationAdmission;
use crate::foreign_realization::ForeignSourceAdmission;
use crate::foreign_realization::IndexedBuiltReceipt;
use crate::foreign_realization::PresentRootInput;
use crate::foreign_realization::ReusedRootFacts;
use crate::foreign_realization::cache_inputs_closure;
use crate::foreign_realization::classify_present_root;
use crate::foreign_realization::is_cache_inputs_build_or_reuse_plan;
use crate::foreign_realization::is_cache_inputs_build_roots_plan;
use crate::foreign_realization::is_cache_only_foreign_plan;
use crate::foreign_realization::realization_trust_policy_blake3;
use crate::foreign_realization::reused_root_outcome_is_cache_hit;
use crate::foreign_realization::satisfied_input_derivations;
use crate::foreign_realization::validate_cache_inputs_roots;
use crate::foreign_realization::validate_cache_only_source_bundle;
use crate::foreign_realization::validate_foreign_cache_closure_policy;
use crate::foreign_realization::validate_foreign_realization_admission;
use crate::foreign_realization::validate_foreign_source_admission;
use crate::foreign_realization_receipt::BUILDER_SIGNER_ROLE;
use crate::foreign_realization_receipt::BuiltRootEntryInput;
use crate::foreign_realization_receipt::ForeignRealizationReceiptAny;
use crate::foreign_realization_receipt::ForeignRealizationRoot;
use crate::foreign_realization_receipt::ForeignRealizationSigner;
use crate::foreign_realization_receipt::ForeignReceiptInput;
use crate::foreign_realization_receipt::ForeignReceiptV2Input;
use crate::foreign_realization_receipt::ForeignRootBuilt;
use crate::foreign_realization_receipt::ReusedRootEntryInput;
use crate::foreign_realization_receipt::build_foreign_realization_receipt;
use crate::foreign_realization_receipt::build_foreign_realization_receipt_v2;
use crate::foreign_realization_receipt::built_root_entry;
use crate::foreign_realization_receipt::built_root_index_derivations;
use crate::foreign_realization_receipt::foreign_realization_receipt_any_bytes;
use crate::foreign_realization_receipt::foreign_realization_receipt_v2_digest;
use crate::foreign_realization_receipt::mark_reused_units_already_present;
use crate::foreign_realization_receipt::parse_foreign_realization_receipt_v2;
use crate::foreign_realization_receipt::realization_input_closure_blake3;
use crate::foreign_realization_receipt::reused_root_entry;
use crate::source_bundle::SourceBundleManifest;
use crate::source_bundle::materialize_source_record_exact_payload;

const FOREIGN_SOURCE_SCRATCH_PREFIX: &str = "mantle-foreign-source-";
const FOREIGN_SOURCE_PAYLOAD_NAME: &str = "payload";
const FOREIGN_PLAN_FILE_IDENTITY: &str = "foreign-executable-plan";
const CACHE_ONLY_OBSERVER_BUILDER: &str = "builtin:cache-only-observation";
const MAX_RECEIPT_BOUND_CACHE_URLS: usize = 64;
const MAX_FOREIGN_SOURCE_FETCH_OVERRIDES: usize = 65_536;
const FETCH_MODE_RECURSIVE: &str = "recursive";
const FOREIGN_REALIZATION_STATE_SUBDIR: &str = "foreign-realization";
const BUILT_ROOT_INDEX_SUBDIR: &str = "built-roots";
const ROOT_LOCK_SUBDIR: &str = "locks";
const BUILT_ROOT_INDEX_SUFFIX: &str = ".json";
const ROOT_LOCK_SUFFIX: &str = ".lock";
const MAX_SELECTED_ROOT_OUTPUTS: usize = 4_096;

/// One completed realization, holding its per-root locks until the caller publishes.
pub(crate) struct ForeignRealizationOutcome {
    pub(crate) receipt: ForeignRealizationReceiptAny,
    /// Write-once built-root index files, published only after the receipt itself.
    pub(crate) built_root_index: Vec<(std::path::PathBuf, Vec<u8>)>,
    /// Build-log persistence failures; they never change the receipt or the realization result.
    pub(crate) diagnostic_persistence_failures: Vec<crate::build_log::DiagnosticPersistenceFailure>,
    _root_locks: Vec<std::fs::File>,
}

pub(crate) struct ForeignRealizationRequest<'a> {
    pub(crate) plan: &'a ForeignExecutablePlan,
    pub(crate) import_receipt: &'a ImportReceipt,
    pub(crate) source_bundle: &'a SourceBundleManifest,
    pub(crate) expected_source_bundle_blake3: &'a str,
    pub(crate) selected_root_node_ids: &'a [String],
    pub(crate) execution_profiles: &'a BTreeMap<String, ExecutionProfile>,
    pub(crate) cache_closure_policy: Option<&'a ForeignCacheClosurePolicy>,
    pub(crate) output_dir: &'a Path,
    pub(crate) state_dir: &'a Path,
    pub(crate) backend: crunch_store::StoreBackend,
    pub(crate) base_state_dirs: &'a [std::path::PathBuf],
    pub(crate) source_admission: Option<&'a std::sync::Arc<dyn crunch_store::StoreSourceAdmission>>,
    pub(crate) keypair: &'a KeyPair,
    pub(crate) trusted_keys: &'a [nix_compat::narinfo::VerifyingKey],
    /// The plan's cache public keys, used only for the route-v2 trust-policy fingerprint.
    pub(crate) cache_keys: &'a [nix_compat::narinfo::VerifyingKey],
    /// Extra builder keys accepted for reuse only (verify-only, e.g. a retired key).
    pub(crate) trusted_builder_public_keys: &'a [nix_compat::narinfo::VerifyingKey],
    /// `NIX_BUILD_CORES` for every build this realization runs. Only route v2 records it; every
    /// other route is refused before store mutation unless it is the default 1.
    pub(crate) build_cores: u32,
    pub(crate) max_jobs: u32,
    pub(crate) substitution_enabled: bool,
    pub(crate) offline: bool,
    pub(crate) remote_execution_requested: bool,
    pub(crate) verbose: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignSourceStoreFact {
    pub(crate) payload_id: String,
    pub(crate) foreign_path: String,
    pub(crate) target_path: String,
    pub(crate) descriptor_blake3: String,
    pub(crate) source_record_identity: String,
    pub(crate) source_content_blake3: String,
    pub(crate) nar_sha256: String,
    pub(crate) nar_size: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignCacheClosureFact {
    pub(crate) logical_path: String,
    pub(crate) nar_sha256: String,
    pub(crate) nar_size: u64,
    pub(crate) references: Vec<String>,
    pub(crate) signature_names: Vec<String>,
    pub(crate) disposition: String,
    pub(crate) depth: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct CacheClosureHydration {
    pub(crate) policy_blake3: String,
    pub(crate) facts: Vec<ForeignCacheClosureFact>,
}

struct IngestedForeignSources {
    facts: Vec<ForeignSourceStoreFact>,
    payload_paths: BTreeMap<String, std::path::PathBuf>,
    _scratch_directories: Vec<tempfile::TempDir>,
}

pub(crate) async fn realize_foreign_plan(
    request: ForeignRealizationRequest<'_>,
) -> Result<ForeignRealizationOutcome, RunError> {
    if request.max_jobs == 0 {
        return Err(RunError::Internal("foreign realization job bound must be nonzero".to_string()));
    }
    if request.offline && request.substitution_enabled {
        return Err(RunError::Internal("offline foreign realization cannot enable cache substitution".to_string()));
    }
    let cache_only = is_cache_only_foreign_plan(request.plan);
    let build_roots = is_cache_inputs_build_roots_plan(request.plan);
    let build_or_reuse = is_cache_inputs_build_or_reuse_plan(request.plan);
    let build_route = build_roots || build_or_reuse;
    if [cache_only, build_roots, build_or_reuse].into_iter().filter(|selected| *selected).count() > 1 {
        return Err(RunError::Internal("foreign realization plan carries two routes".to_string()));
    }
    let cache_hydrated = cache_only || build_route;
    if cache_hydrated && (!request.substitution_enabled || request.offline) {
        return Err(RunError::Internal("cache-hydrated foreign realization requires online substitution".to_string()));
    }
    let admitted = validate_foreign_realization_admission(ForeignRealizationAdmission {
        plan: request.plan,
        import_receipt: request.import_receipt,
        selected_root_node_ids: request.selected_root_node_ids,
        execution_profiles: request.execution_profiles,
        remote_execution_requested: request.remote_execution_requested,
    })
    .map_err(|error| RunError::Internal(error.to_string()))?;
    let admitted_sources = if cache_hydrated {
        let policy = request.cache_closure_policy.ok_or_else(|| {
            RunError::Internal("cache-hydrated foreign realization requires a cache closure policy".to_string())
        })?;
        validate_foreign_cache_closure_policy(policy).map_err(|error| RunError::Internal(error.to_string()))?;
        validate_cache_only_source_bundle(request.source_bundle, request.expected_source_bundle_blake3)
            .map_err(|error| RunError::Internal(error.to_string()))?;
        Vec::new()
    } else {
        if request.cache_closure_policy.is_some() {
            return Err(RunError::Internal(
                "cache closure policy is only valid for a cache-only foreign plan".to_string(),
            ));
        }
        validate_foreign_source_admission(ForeignSourceAdmission {
            source_requirements: &request.plan.source_requirements,
            source_bundle: request.source_bundle,
            expected_manifest_blake3: request.expected_source_bundle_blake3,
        })
        .map_err(|error| RunError::Internal(error.to_string()))?
    };
    let cache_urls = if request.substitution_enabled {
        receipt_bound_cache_urls(request.plan)?
    } else {
        Vec::new()
    };
    let retained_outputs = selected_root_output_paths(request.plan, request.selected_root_node_ids)?;
    let builder_keys = builder_keys_for_realization(request.keypair, request.trusted_builder_public_keys);
    let mut root_locks = Vec::new();
    if build_or_reuse {
        root_locks = acquire_root_locks(request.state_dir, request.plan, request.selected_root_node_ids)?;
    }
    let mut store = open_foreign_store(&request, if build_or_reuse { Vec::new() } else { cache_urls.clone() }).await?;
    let reuse = if build_or_reuse {
        classify_present_selected_roots(
            &mut store,
            request.state_dir,
            request.plan,
            request.selected_root_node_ids,
            &builder_keys,
        )
        .await?
    } else {
        ReusePlan::default()
    };
    let ingested_sources = if cache_hydrated {
        IngestedForeignSources {
            facts: Vec::new(),
            payload_paths: BTreeMap::new(),
            _scratch_directories: Vec::new(),
        }
    } else {
        let mut source_admission = store.source_admission();
        ingest_admitted_foreign_sources(
            &admitted_sources,
            request.source_bundle,
            &mut source_admission,
            &request.keypair.signing_key,
        )
        .await?
    };
    let cache_closure = if cache_hydrated {
        let policy = request.cache_closure_policy.ok_or_else(|| {
            RunError::Internal("cache-hydrated foreign realization requires a cache closure policy".to_string())
        })?;
        if cache_only {
            Some(
                hydrate_signed_cache_closure(&mut store, &retained_outputs, request.plan, policy, request.trusted_keys)
                    .await?,
            )
        } else if build_or_reuse {
            let (hydration, reopened) =
                hydration_for_absent_roots(store, &request, policy, &reuse, &cache_urls).await?;
            store = reopened;
            hydration
        } else {
            let hydration = hydrate_cache_inputs(&mut store, &request, policy, &retained_outputs).await?;
            drop(store);
            store = open_foreign_store(&request, Vec::new()).await?;
            refuse_any_present_selected_root(&mut store, &retained_outputs).await?;
            hydration
        }
    } else {
        None
    };

    let satisfied_derivations = if build_route {
        let mut satisfied_paths = cache_closure
            .as_ref()
            .map(|hydration| hydration.facts.iter().map(|fact| fact.logical_path.clone()).collect::<BTreeSet<_>>())
            .unwrap_or_default();
        if build_or_reuse {
            // A reuse run hydrates nothing, so a present input closure is satisfied by presence
            // alone; this mirrors what the v1 hydration facts supply (and keeps an unneeded
            // dependency of a present input from being scheduled).
            satisfied_paths
                .extend(locally_present_input_paths(&mut store, request.plan, request.selected_root_node_ids).await?);
        }
        satisfied_input_derivations(request.plan, request.selected_root_node_ids, &satisfied_paths)
    } else {
        BTreeSet::new()
    };
    let mut registry = DerivationRegistry::new(&request.plan.target_store_prefix);
    let roots = admitted.selected_root_paths.clone();
    let selected_unit_ids = admitted.selected_root_node_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut expected_outputs = Vec::new();
    for unit in &admitted.units {
        if cache_only && !selected_unit_ids.contains(unit.node_id.as_str()) {
            continue;
        }
        let registry_derivation = if cache_only {
            cache_only_observer_derivation(&unit.derivation)
        } else {
            unit.derivation.clone()
        };
        for (output_name, output) in &registry_derivation.outputs {
            let store_path = output.path.clone().ok_or_else(|| {
                RunError::Internal(format!(
                    "foreign realization output path is unresolved: {}:{output_name}",
                    unit.node_id
                ))
            })?;
            expected_outputs.push(RegisteredOutputExpectation {
                unit_id: unit.node_id.clone(),
                output_name: output_name.clone(),
                store_path,
            });
        }
        registry
            .insert_with_execution_profile(
                unit.drv_path.clone(),
                unit.hdm,
                registry_derivation,
                false,
                None,
                unit.execution_profile.clone(),
            )
            .map_err(|error| {
                RunError::Internal(format!("registering foreign realization unit {}: {error}", unit.node_id))
            })?;
    }
    let source_fetch_overrides =
        foreign_source_fetch_overrides(request.plan, &ingested_sources.facts, &ingested_sources.payload_paths)?;
    let source_policy = if request.offline || cache_hydrated {
        FetchSourcePolicy::RequireOverride
    } else {
        FetchSourcePolicy::AllowNetwork
    };
    let build_config = BuildConfig {
        file: FOREIGN_PLAN_FILE_IDENTITY.into(),
        import_paths: Vec::new(),
        output_dir: request.output_dir.to_path_buf(),
        state_dir: request.state_dir.to_path_buf(),
        backend: request.backend,
        store_dir: request.plan.target_store_prefix.clone(),
        verbose: request.verbose,
        max_jobs: request.max_jobs,
        scheduling_policy: SchedulingPolicy {
            schema: crunch_build::scheduling::SCHEDULING_POLICY_SCHEMA.to_string(),
            policy_id: crunch_build::scheduling::DEFAULT_SCHEDULING_POLICY_ID.to_string(),
            preference_order: vec![
                PreferenceField::KnownGraph,
                PreferenceField::ResourceFit,
                PreferenceField::LocalityTransfer,
            ],
            aged_after_epochs: crunch_build::scheduling::DEFAULT_AGED_AFTER_EPOCHS,
            protected_after_epochs: crunch_build::scheduling::DEFAULT_PROTECTED_AFTER_EPOCHS,
        },
        substituter_urls: if cache_hydrated { Vec::new() } else { cache_urls.clone() },
        hermeticity_mode: HermeticityMode::Strict,
        build_cores: request.build_cores,
        base_state_dirs: request.base_state_dirs.to_vec(),
        source_admission: request.source_admission.cloned(),
        keypair: request.keypair.clone(),
        trusted_keys: request.trusted_keys.to_vec(),
        trust_unsigned: false,
        root_retention_source: None,
        root_registration: None,
        source_fetch_overrides,
        remote_enabled: false,
        compile_driver_runtime: None,
        rust_wait_runtime: None,
        interchange_dir: None,
        pool_executor_factory: None,
    };
    let build_result =
        crunch_pipeline::build_registered_derivations(&build_config, store, &mut registry, RegisteredBuildRequest {
            roots: &roots,
            expected_outputs: &expected_outputs,
            retained_outputs: &retained_outputs,
            source_policy,
            cache_only,
            forbid_host_store_fallback: build_route,
            satisfied_derivations: &satisfied_derivations,
        })
        .await
        .map_err(|error| RunError::Internal(format!("realizing foreign derivation graph: {error}")))?;
    if build_or_reuse {
        assert_reused_roots_are_cache_hits(&build_result, &reuse, &request.plan.target_store_prefix)?;
    }
    // r[impl foreign_derivation_import.realization_build_logs]
    let diagnostic_persistence_failures =
        write_realization_logs(request.state_dir, &request.plan.target_store_prefix, &build_result);
    let base = ForeignReceiptInput {
        plan: request.plan,
        source_bundle_manifest_blake3: request.expected_source_bundle_blake3,
        selected_root_node_ids: &admitted.selected_root_node_ids,
        source_facts: &ingested_sources.facts,
        build_result: &build_result,
        substitution_enabled: request.substitution_enabled,
        offline: request.offline,
        ordered_cache_urls: &cache_urls,
        cache_closure: cache_closure.as_ref(),
        allow_pruned_units: build_or_reuse,
    };
    if !build_or_reuse {
        return Ok(ForeignRealizationOutcome {
            receipt: ForeignRealizationReceiptAny::V1(build_foreign_realization_receipt(base)?),
            built_root_index: Vec::new(),
            diagnostic_persistence_failures,
            _root_locks: root_locks,
        });
    }
    let trust_policy_blake3 = realization_trust_policy_blake3(&builder_keys, request.cache_keys)?;
    let builder = builder_keys
        .first()
        .ok_or_else(|| RunError::Internal("foreign realization has no builder key".to_string()))?;
    let input_closure_blake3 = realization_input_closure_blake3(
        cache_closure.as_ref().map(|hydration| hydration.facts.as_slice()).unwrap_or(&[]),
    )?;
    let roots = build_v2_root_entries(
        &reuse,
        request.plan,
        &build_result,
        builder,
        &trust_policy_blake3,
        &input_closure_blake3,
        request.build_cores,
    )?;
    let mut receipt = build_foreign_realization_receipt_v2(ForeignReceiptV2Input { base, roots })?;
    mark_reused_units_already_present(&mut receipt.units, &reuse.reused_node_ids());
    receipt.receipt_blake3 = foreign_realization_receipt_v2_digest(&receipt)?;
    let index_derivations = built_root_index_derivations(&receipt);
    let any = ForeignRealizationReceiptAny::V2(receipt);
    let bytes = foreign_realization_receipt_any_bytes(&any)?;
    let mut built_root_index = Vec::new();
    for derivation in index_derivations {
        built_root_index.push((built_root_index_path(request.state_dir, &derivation)?, bytes.clone()));
    }
    Ok(ForeignRealizationOutcome {
        receipt: any,
        built_root_index,
        diagnostic_persistence_failures,
        _root_locks: root_locks,
    })
}

/// Persist every captured realization build log the way `mantle build` does.
///
/// A failed root writes its retained build-service log when one was kept, and otherwise the same
/// failure detail `mantle build` writes (gated by `should_write_failure_log`, with the `origin …`
/// detail for a dependency failure). Cached or reused outcomes carry no log and therefore write
/// nothing, and an existing `<state-dir>/logs/<drv>.log` is never touched by a reuse run. Failures
/// are returned for the caller to report; they never change the receipt or the realization result.
fn write_realization_logs(
    state_dir: &Path,
    store_dir: &str,
    result: &crunch_pipeline::RegisteredBuildResult,
) -> Vec<crate::build_log::DiagnosticPersistenceFailure> {
    use crate::build_log::DiagnosticPersistenceFailure;

    assert!(!state_dir.as_os_str().is_empty(), "realization log state directory must not be empty");
    assert!(!store_dir.is_empty(), "realization log store prefix must not be empty");
    let logs_dir = state_dir.join("logs");
    let mut failures = Vec::new();
    if let Err(error) = std::fs::create_dir_all(&logs_dir) {
        failures.push(DiagnosticPersistenceFailure::create_log_dir(&logs_dir, &error));
        return failures;
    }
    for outcome in &result.outcomes {
        let Some(log) = outcome.log.as_deref() else {
            continue;
        };
        if let Err(failure) =
            crate::build_cmd::write_log(&logs_dir, &outcome.drv_path, outcome.drv_path.name(), true, log)
        {
            failures.push(failure);
        }
    }
    for failed in &result.failed_roots {
        if let Some(log) = failed.build_log.as_deref() {
            let Some(drv_path) = crunch_pipeline::parse_drv_key(store_dir, &failed.drv_key) else {
                continue;
            };
            if let Err(failure) = crate::build_cmd::write_log(&logs_dir, &drv_path, drv_path.name(), false, log) {
                failures.push(failure);
            }
            continue;
        }
        if let Some(failure) = crate::build_cmd::write_failed_goal_log(&logs_dir, store_dir, failed, None) {
            failures.push(failure);
        }
    }
    debug_assert!(failures.len() <= result.outcomes.len().saturating_add(result.failed_roots.len()));
    debug_assert!(!logs_dir.as_os_str().is_empty());
    failures
}

async fn open_foreign_store(
    request: &ForeignRealizationRequest<'_>,
    remote_cache_urls: Vec<String>,
) -> Result<StoreHandle, RunError> {
    StoreHandle::open(crunch_store::StoreConfig {
        backend: request.backend,
        state_dir: request.state_dir.to_path_buf(),
        output_dir: request.output_dir.to_path_buf(),
        remote_cache_urls,
        fallback_mode: crunch_store::StoreFallbackMode::Strict,
        store_dir: request.plan.target_store_prefix.clone(),
        base_state_dirs: request.base_state_dirs.to_vec(),
        source_admission: request.source_admission.cloned(),
    })
    .await
    .map_err(|error| RunError::Internal(format!("opening foreign realization store: {error}")))
}

fn builder_keys_for_realization(
    keypair: &KeyPair,
    trusted_public_keys: &[nix_compat::narinfo::VerifyingKey],
) -> Vec<BuilderKey> {
    let mut keys = vec![BuilderKey::new(keypair.verifying_key.clone())];
    for key in trusted_public_keys {
        if !keys.iter().any(|existing| {
            existing.public_key_blake3 == crate::foreign_realization::realization_public_key_blake3(key)
        }) {
            keys.push(BuilderKey::new(key.clone()));
        }
    }
    keys
}

/// v1 route-2 behaviour: probe every selected root, hydrate the input seeds, then require the roots
/// absent.
async fn hydrate_cache_inputs(
    store: &mut StoreHandle,
    request: &ForeignRealizationRequest<'_>,
    policy: &ForeignCacheClosurePolicy,
    retained_outputs: &[StorePath<String>],
) -> Result<Option<CacheClosureHydration>, RunError> {
    let seed_strings = cache_inputs_closure(request.plan, request.selected_root_node_ids)
        .map_err(|error| RunError::Internal(error.to_string()))?;
    let root_strings = retained_outputs.iter().map(ToString::to_string).collect::<Vec<_>>();
    let cache_url = cache_only_http_cache_url(request.plan)?;
    let pull_options = crunch_store::PullOptions {
        trust_unsigned: false,
        trusted_public_keys: request.trusted_keys.to_vec(),
    };
    let mut offered = Vec::new();
    for root in retained_outputs {
        let is_offered = crunch_store::probe_http_cache_path(store, &cache_url, root, &pull_options)
            .await
            .map_err(|error| RunError::Internal(format!("probing the cache for selected root {root}: {error}")))?;
        if is_offered {
            offered.push(root.to_string());
        }
    }
    validate_cache_inputs_roots(&root_strings, &offered, &[]).map_err(|error| RunError::Internal(error.to_string()))?;
    if seed_strings.is_empty() {
        return Ok(None);
    }
    let mut seeds = Vec::with_capacity(seed_strings.len());
    for seed in &seed_strings {
        seeds.push(
            nix_compat::store_path::StorePath::from_absolute_path_with_prefix(
                seed.as_bytes(),
                &request.plan.target_store_prefix,
            )
            .map_err(|error| RunError::Internal(format!("parsing cache input seed {seed}: {error}")))?,
        );
    }
    Ok(Some(hydrate_signed_cache_closure(store, &seeds, request.plan, policy, request.trusted_keys).await?))
}

async fn refuse_any_present_selected_root(
    store: &mut StoreHandle,
    retained_outputs: &[StorePath<String>],
) -> Result<(), RunError> {
    let mut present = Vec::new();
    for root in retained_outputs {
        if store
            .export_cached_path_info(root)
            .await
            .map_err(|error| RunError::Internal(format!("checking selected root {root}: {error}")))?
            .is_some()
        {
            present.push(root.to_string());
        }
    }
    let root_strings = retained_outputs.iter().map(ToString::to_string).collect::<Vec<_>>();
    validate_cache_inputs_roots(&root_strings, &[], &present).map_err(|error| RunError::Internal(error.to_string()))
}

/// v2 reuse: every non-root unit whose outputs are all present locally is already realized.
async fn locally_present_input_paths(
    store: &mut StoreHandle,
    plan: &ForeignExecutablePlan,
    selected_root_node_ids: &[String],
) -> Result<BTreeSet<String>, RunError> {
    let mut present = BTreeSet::new();
    for unit in &plan.native_units {
        if selected_root_node_ids.binary_search(&unit.node_id).is_ok() || unit.outputs.is_empty() {
            continue;
        }
        let mut all_present = true;
        for path in unit.outputs.values() {
            let store_path = nix_compat::store_path::StorePath::from_absolute_path_with_prefix(
                path.as_bytes(),
                &plan.target_store_prefix,
            )
            .map_err(|error| RunError::Internal(format!("parsing foreign unit output {path}: {error}")))?;
            if store
                .cached_path_info(&store_path)
                .await
                .map_err(|error| RunError::Internal(format!("checking foreign unit output {path}: {error}")))?
                .is_none()
            {
                all_present = false;
                break;
            }
        }
        if all_present {
            present.extend(unit.outputs.values().cloned());
        }
    }
    Ok(present)
}

/// v2 route: hydrate only the inputs an absent selected root needs, then require it still absent.
async fn hydration_for_absent_roots(
    mut store: StoreHandle,
    request: &ForeignRealizationRequest<'_>,
    policy: &ForeignCacheClosurePolicy,
    reuse: &ReusePlan,
    cache_urls: &[String],
) -> Result<(Option<CacheClosureHydration>, StoreHandle), RunError> {
    let absent_roots = reuse.absent_output_paths();
    if absent_roots.is_empty() {
        return Ok((None, store));
    }
    let seed_strings = cache_inputs_closure(request.plan, request.selected_root_node_ids)
        .map_err(|error| RunError::Internal(error.to_string()))?;
    drop(store);
    store = open_foreign_store(request, cache_urls.to_vec()).await?;
    let absent_strings = absent_roots.iter().map(ToString::to_string).collect::<Vec<_>>();
    let cache_url = cache_only_http_cache_url(request.plan)?;
    let pull_options = crunch_store::PullOptions {
        trust_unsigned: false,
        trusted_public_keys: request.trusted_keys.to_vec(),
    };
    let mut offered = Vec::new();
    for root in &absent_roots {
        let is_offered = crunch_store::probe_http_cache_path(&store, &cache_url, root, &pull_options)
            .await
            .map_err(|error| RunError::Internal(format!("probing the cache for selected root {root}: {error}")))?;
        if is_offered {
            offered.push(root.to_string());
        }
    }
    validate_cache_inputs_roots(&absent_strings, &offered, &[])
        .map_err(|error| RunError::Internal(error.to_string()))?;
    let hydration = if seed_strings.is_empty() {
        None
    } else {
        let mut seeds = Vec::with_capacity(seed_strings.len());
        for seed in &seed_strings {
            seeds.push(
                nix_compat::store_path::StorePath::from_absolute_path_with_prefix(
                    seed.as_bytes(),
                    &request.plan.target_store_prefix,
                )
                .map_err(|error| RunError::Internal(format!("parsing cache input seed {seed}: {error}")))?,
            );
        }
        Some(hydrate_signed_cache_closure(&mut store, &seeds, request.plan, policy, request.trusted_keys).await?)
    };
    drop(store);
    let mut local = open_foreign_store(request, Vec::new()).await?;
    let mut present = Vec::new();
    for root in &absent_roots {
        if local
            .export_cached_path_info(root)
            .await
            .map_err(|error| RunError::Internal(format!("checking selected root {root}: {error}")))?
            .is_some()
        {
            present.push(root.to_string());
        }
    }
    validate_cache_inputs_roots(&absent_strings, &[], &present)
        .map_err(|error| RunError::Internal(error.to_string()))?;
    Ok((hydration, local))
}

struct SelectedRootOutput {
    output_name: String,
    path: StorePath<String>,
}

struct SelectedRootGroup {
    node_id: String,
    target_derivation: String,
    outputs: Vec<SelectedRootOutput>,
}

struct RootGroupDecision {
    group: SelectedRootGroup,
    /// Present and fully classified; `None` means the group must be built.
    output_facts: Option<Vec<ReusedRootFacts>>,
}

#[derive(Default)]
struct ReusePlan {
    groups: Vec<RootGroupDecision>,
}

impl ReusePlan {
    fn reused_node_ids(&self) -> BTreeSet<String> {
        self.groups
            .iter()
            .filter(|decision| decision.output_facts.is_some())
            .map(|decision| decision.group.node_id.clone())
            .collect()
    }

    fn absent_output_paths(&self) -> Vec<StorePath<String>> {
        self.groups
            .iter()
            .filter(|decision| decision.output_facts.is_none())
            .flat_map(|decision| decision.group.outputs.iter().map(|output| output.path.clone()))
            .collect()
    }
}

fn selected_root_groups(
    plan: &ForeignExecutablePlan,
    selected_root_node_ids: &[String],
) -> Result<Vec<SelectedRootGroup>, RunError> {
    let mut groups = Vec::new();
    let mut output_count = 0usize;
    for root in &plan.selected_roots {
        if selected_root_node_ids.binary_search(&root.node_id).is_err() {
            continue;
        }
        let mut outputs = root.target_outputs.iter().collect::<Vec<_>>();
        outputs.sort_by(|left, right| left.0.cmp(right.0));
        let mut parsed = Vec::with_capacity(outputs.len());
        for (output_name, path) in outputs {
            let store_path = nix_compat::store_path::StorePath::from_absolute_path_with_prefix(
                path.as_bytes(),
                &plan.target_store_prefix,
            )
            .map_err(|error| RunError::Internal(format!("parsing selected foreign root output {path}: {error}")))?;
            parsed.push(SelectedRootOutput {
                output_name: output_name.clone(),
                path: store_path,
            });
            output_count = output_count.saturating_add(1);
        }
        groups.push(SelectedRootGroup {
            node_id: root.node_id.clone(),
            target_derivation: root.target_derivation.clone(),
            outputs: parsed,
        });
    }
    if output_count > MAX_SELECTED_ROOT_OUTPUTS {
        return Err(RunError::Internal(format!(
            "foreign realization selected root output count exceeds {MAX_SELECTED_ROOT_OUTPUTS}"
        )));
    }
    Ok(groups)
}

fn root_derivation_basename(derivation: &str) -> Result<&str, RunError> {
    Path::new(derivation)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| RunError::Internal(format!("foreign root derivation has no basename: {derivation}")))
}

/// Directory holding the per-root realization locks.
pub(crate) fn root_lock_dir(state_dir: &Path) -> std::path::PathBuf {
    state_dir.join(FOREIGN_REALIZATION_STATE_SUBDIR).join(ROOT_LOCK_SUBDIR)
}

/// Directory holding the write-once built-root index entries.
pub(crate) fn built_root_index_dir(state_dir: &Path) -> std::path::PathBuf {
    state_dir.join(FOREIGN_REALIZATION_STATE_SUBDIR).join(BUILT_ROOT_INDEX_SUBDIR)
}

fn acquisition_paths(
    state_dir: &Path,
    subdir: &str,
    suffix: &str,
    derivations: &[String],
) -> Result<Vec<std::path::PathBuf>, RunError> {
    let dir = state_dir.join(FOREIGN_REALIZATION_STATE_SUBDIR).join(subdir);
    std::fs::create_dir_all(&dir)
        .map_err(|error| RunError::Internal(format!("creating {}: {error}", dir.display())))?;
    let mut names = derivations
        .iter()
        .map(|derivation| root_derivation_basename(derivation).map(str::to_string))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    names.dedup();
    Ok(names.into_iter().map(|name| dir.join(format!("{name}{suffix}"))).collect())
}

fn built_root_index_path(state_dir: &Path, derivation: &str) -> Result<std::path::PathBuf, RunError> {
    let dir = built_root_index_dir(state_dir);
    Ok(dir.join(format!("{}{BUILT_ROOT_INDEX_SUFFIX}", root_derivation_basename(derivation)?)))
}

fn acquire_root_locks(
    state_dir: &Path,
    plan: &ForeignExecutablePlan,
    selected_root_node_ids: &[String],
) -> Result<Vec<std::fs::File>, RunError> {
    let derivations = plan
        .selected_roots
        .iter()
        .filter(|root| selected_root_node_ids.binary_search(&root.node_id).is_ok())
        .map(|root| root.target_derivation.clone())
        .collect::<Vec<_>>();
    let mut locks = Vec::new();
    for path in acquisition_paths(state_dir, ROOT_LOCK_SUBDIR, ROOT_LOCK_SUFFIX, &derivations)? {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|error| RunError::Internal(format!("opening root lock {}: {error}", path.display())))?;
        fs2::FileExt::lock_exclusive(&file)
            .map_err(|error| RunError::Internal(format!("locking root lock {}: {error}", path.display())))?;
        locks.push(file);
    }
    Ok(locks)
}

fn read_indexed_built_receipt(state_dir: &Path, derivation: &str) -> Result<IndexedBuiltReceipt, RunError> {
    let path = built_root_index_path(state_dir, derivation)?;
    match std::fs::read(&path) {
        Ok(bytes) => Ok(match parse_foreign_realization_receipt_v2(&bytes) {
            Ok(receipt) => IndexedBuiltReceipt::Present(Box::new(receipt)),
            Err(_) => IndexedBuiltReceipt::Unparsable,
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(IndexedBuiltReceipt::Absent),
        Err(error) => Err(RunError::Internal(format!("reading built-root index {}: {error}", path.display()))),
    }
}

async fn classify_present_selected_roots(
    store: &mut StoreHandle,
    state_dir: &Path,
    plan: &ForeignExecutablePlan,
    selected_root_node_ids: &[String],
    builder_keys: &[BuilderKey],
) -> Result<ReusePlan, RunError> {
    let groups = selected_root_groups(plan, selected_root_node_ids)?;
    let mut decisions = Vec::with_capacity(groups.len());
    for group in groups {
        let mut present = Vec::with_capacity(group.outputs.len());
        let mut all_present = true;
        for output in &group.outputs {
            let Some(path_info) = store
                .export_cached_path_info(&output.path)
                .await
                .map_err(|error| RunError::Internal(format!("checking selected root {}: {error}", output.path)))?
            else {
                all_present = false;
                break;
            };
            let Some(node) = store
                .cached_node_for_path(&output.path)
                .await
                .map_err(|error| RunError::Internal(format!("loading selected root {}: {error}", output.path)))?
            else {
                all_present = false;
                break;
            };
            present.push((output, path_info, node));
        }
        if !all_present {
            decisions.push(RootGroupDecision {
                group,
                output_facts: None,
            });
            continue;
        }
        let indexed = read_indexed_built_receipt(state_dir, &group.target_derivation)?;
        let mut facts = Vec::with_capacity(present.len());
        for (output, path_info, node) in present {
            let (recomputed_nar_sha256, _nar_size) = store
                .recompute_nar_sha256(&node)
                .await
                .map_err(|error| RunError::Internal(format!("recomputed NAR for {}: {error}", output.path)))?;
            let path = output.path.to_string();
            let fact = classify_present_root(PresentRootInput {
                target_derivation: &group.target_derivation,
                output_name: &output.output_name,
                path: &path,
                store_dir: &plan.target_store_prefix,
                path_info: &path_info,
                recomputed_nar_sha256,
                builder_keys,
                indexed: read_indexed_receipt_for(&indexed)?,
            })
            .map_err(|refusal| {
                RunError::Internal(format!("refusing to reuse present foreign root {path}: {refusal}"))
            })?;
            facts.push(fact);
        }
        decisions.push(RootGroupDecision {
            group,
            output_facts: Some(facts),
        });
    }
    Ok(ReusePlan { groups: decisions })
}

/// Clone only what the pure classifier consumes, keeping one parse per root derivation.
fn read_indexed_receipt_for(indexed: &IndexedBuiltReceipt) -> Result<IndexedBuiltReceipt, RunError> {
    Ok(match indexed {
        IndexedBuiltReceipt::Absent => IndexedBuiltReceipt::Absent,
        IndexedBuiltReceipt::Unparsable => IndexedBuiltReceipt::Unparsable,
        IndexedBuiltReceipt::Present(receipt) => IndexedBuiltReceipt::Present(receipt.clone()),
    })
}

fn assert_reused_roots_are_cache_hits(
    build_result: &crunch_pipeline::RegisteredBuildResult,
    reuse: &ReusePlan,
    store_dir: &str,
) -> Result<(), RunError> {
    for decision in &reuse.groups {
        if decision.output_facts.is_none() {
            continue;
        }
        let derivation = decision.group.target_derivation.as_str();
        if !reused_root_outcome_is_cache_hit(&build_result.outcomes, derivation, store_dir) {
            return Err(RunError::Internal(format!(
                "reused foreign root {derivation} was not served from the store as a cache hit"
            )));
        }
    }
    Ok(())
}

fn build_v2_root_entries(
    reuse: &ReusePlan,
    plan: &ForeignExecutablePlan,
    build_result: &crunch_pipeline::RegisteredBuildResult,
    builder: &BuilderKey,
    trust_policy_blake3: &str,
    input_closure_blake3: &str,
    build_cores: u32,
) -> Result<Vec<ForeignRealizationRoot>, RunError> {
    assert!(build_cores > 0, "build cores must be at least one");
    if build_result.failed_roots.is_empty() {
        debug_assert!(!reuse.groups.is_empty(), "a successful build route has selected root groups");
    }
    let store_dir = plan.target_store_prefix.as_str();
    let built_signer = ForeignRealizationSigner {
        key_name: builder.name.clone(),
        public_key_blake3: builder.public_key_blake3.clone(),
        role: BUILDER_SIGNER_ROLE.to_string(),
    };
    let mut roots = Vec::new();
    for decision in &reuse.groups {
        let group = &decision.group;
        let unit = plan.native_units.iter().find(|unit| unit.node_id == group.node_id).ok_or_else(|| {
            RunError::Internal(format!("foreign root unit is absent from the plan: {}", group.node_id))
        })?;
        if let Some(facts) = decision.output_facts.as_ref() {
            for (output, fact) in group.outputs.iter().zip(facts) {
                let root_path = output.path.to_string();
                roots.push(reused_root_entry(ReusedRootEntryInput {
                    node_id: &group.node_id,
                    target_derivation: &group.target_derivation,
                    output_name: &output.output_name,
                    path: &root_path,
                    nar_sha256: &fact.nar_sha256,
                    nar_size: fact.nar_size,
                    references: &fact.references,
                    signer: ForeignRealizationSigner {
                        key_name: fact.signer_key_name.clone(),
                        public_key_blake3: fact.signer_public_key_blake3.clone(),
                        role: BUILDER_SIGNER_ROLE.to_string(),
                    },
                    trust_policy_blake3,
                    original_receipt_blake3: &fact.original_receipt_blake3,
                }));
            }
            continue;
        }
        for output in &group.outputs {
            let realized = build_result
                .outputs
                .iter()
                .find(|result| result.unit_id == group.node_id && result.output_name == output.output_name);
            let Some(realized) = realized else {
                if build_result.failed_roots.is_empty() {
                    return Err(RunError::Internal(format!(
                        "built foreign root {}:{} has no realized output",
                        group.node_id, output.output_name
                    )));
                }
                continue;
            };
            let mut references = realized
                .path_info
                .references
                .iter()
                .map(|reference| reference.to_absolute_path_with_prefix(store_dir))
                .collect::<Vec<_>>();
            references.sort();
            references.dedup();
            let nar_sha256 = HEXLOWER.encode(&realized.path_info.nar_sha256);
            let root_path = output.path.to_string();
            roots.push(built_root_entry(BuiltRootEntryInput {
                node_id: &group.node_id,
                target_derivation: &group.target_derivation,
                output_name: &output.output_name,
                path: &root_path,
                nar_sha256: &nar_sha256,
                nar_size: realized.path_info.nar_size,
                references: &references,
                signer: built_signer.clone(),
                trust_policy_blake3,
                built: ForeignRootBuilt {
                    mantle_version: env!("CARGO_PKG_VERSION").to_string(),
                    execution_profile_digest_blake3: unit.execution_profile_digest_blake3.clone(),
                    input_closure_blake3: input_closure_blake3.to_string(),
                    // Recorded only for an explicit override, so default receipts stay unchanged.
                    build_cores: (build_cores != crate::foreign_import_cmd::DEFAULT_FOREIGN_BUILD_CORES)
                        .then_some(build_cores),
                },
            }));
        }
    }
    Ok(roots)
}

fn foreign_source_fetch_overrides(
    plan: &ForeignExecutablePlan,
    source_facts: &[ForeignSourceStoreFact],
    payload_paths: &BTreeMap<String, std::path::PathBuf>,
) -> Result<Vec<FetchSourceOverride>, RunError> {
    let mut overrides = Vec::new();
    for unit in &plan.native_units {
        let Some(source_fact) = source_facts
            .iter()
            .find(|fact| unit.declared_references.iter().any(|reference| reference == &fact.target_path))
        else {
            continue;
        };
        let payload_path = payload_paths.get(&source_fact.payload_id).ok_or_else(|| {
            RunError::Internal(format!(
                "foreign source payload path is absent after ingestion: {}",
                source_fact.payload_id
            ))
        })?;
        append_unit_source_overrides(&mut overrides, unit, source_fact, payload_path)?;
    }
    assert!(overrides.len() <= MAX_FOREIGN_SOURCE_FETCH_OVERRIDES);
    Ok(overrides)
}

fn append_unit_source_overrides(
    overrides: &mut Vec<FetchSourceOverride>,
    unit: &ExecutableNativeUnit,
    source_fact: &ForeignSourceStoreFact,
    payload_path: &Path,
) -> Result<(), RunError> {
    use crate::foreign_graph_compiler::CompiledForeignBuiltin;

    let (candidates, kind, revision) = match &unit.builtin {
        CompiledForeignBuiltin::Download {
            candidates,
            mode,
            executable,
            ..
        } => {
            let kind = if *executable {
                FetchSourceOverrideKind::Executable
            } else if mode == FETCH_MODE_RECURSIVE {
                FetchSourceOverrideKind::Tarball
            } else {
                FetchSourceOverrideKind::File
            };
            (candidates, kind, None)
        }
        CompiledForeignBuiltin::GitDownload {
            candidates, revision, ..
        } => (candidates, FetchSourceOverrideKind::Git, Some(revision.clone())),
        CompiledForeignBuiltin::NativeDerivation | CompiledForeignBuiltin::FixedOutput => return Ok(()),
    };
    if overrides.len().saturating_add(candidates.len()) > MAX_FOREIGN_SOURCE_FETCH_OVERRIDES {
        return Err(RunError::Internal(format!(
            "foreign source fetch override count exceeds {MAX_FOREIGN_SOURCE_FETCH_OVERRIDES}"
        )));
    }
    for candidate in candidates {
        overrides.push(FetchSourceOverride {
            url: candidate.clone(),
            kind,
            rev: revision.clone(),
            payload_path: payload_path.to_path_buf(),
            source_state_blake3: source_fact.source_content_blake3.clone(),
        });
    }
    Ok(())
}

fn cache_only_observer_derivation(derivation: &Derivation) -> Derivation {
    let mut observer = derivation.clone();
    observer.input_derivations.clear();
    observer.input_sources.clear();
    observer.builder = CACHE_ONLY_OBSERVER_BUILDER.to_string();
    observer.arguments.clear();
    observer
}

async fn hydrate_signed_cache_closure(
    store: &mut StoreHandle,
    roots: &[StorePath<String>],
    plan: &ForeignExecutablePlan,
    policy: &ForeignCacheClosurePolicy,
    trusted_keys: &[nix_compat::narinfo::VerifyingKey],
) -> Result<CacheClosureHydration, RunError> {
    validate_foreign_cache_closure_policy(policy).map_err(|error| RunError::Internal(error.to_string()))?;
    if trusted_keys.is_empty() {
        return Err(RunError::Internal("cache-hydrated foreign realization has no trusted PathInfo keys".to_string()));
    }
    if roots.is_empty() {
        return Err(RunError::Internal("cache-hydrated foreign realization has no seed path".to_string()));
    }
    let policy_bytes = serde_json::to_vec(policy)
        .map_err(|error| RunError::Internal(format!("serializing foreign cache closure policy: {error}")))?;
    let policy_blake3 = blake3::hash(&policy_bytes).to_hex().to_string();
    let permitted_paths = plan
        .exact_path_maps
        .outputs
        .values()
        .chain(plan.exact_path_maps.sources.values())
        .cloned()
        .collect::<BTreeSet<_>>();
    for root in roots {
        let root_path = root.to_absolute_path_with_prefix(&plan.target_store_prefix);
        if !permitted_paths.contains(&root_path) {
            return Err(RunError::Internal(format!(
                "cache-hydrated seed path is outside the exact path map: {root_path}"
            )));
        }
    }
    let cache_url = cache_only_http_cache_url(plan)?;
    let limits = cache_only_http_closure_limits(policy)?;
    let pull_options = crunch_store::PullOptions {
        trust_unsigned: false,
        trusted_public_keys: trusted_keys.to_vec(),
    };
    let report = crunch_store::import_http_cache_closure_with_validator(
        store,
        &cache_url,
        roots,
        &pull_options,
        limits,
        |closure_plan| {
            validate_prepared_cache_closure(closure_plan, roots, &plan.target_store_prefix, &permitted_paths)
        },
    )
    .await
    .map_err(|error| RunError::Internal(format!("hydrating preflighted foreign cache closure: {error}")))?;
    if !report.roots_admitted {
        return Err(RunError::Internal("foreign cache closure seed path was not admitted after hydration".to_string()));
    }
    let imported_paths = report.pull.paths.iter().map(|path| path.store_path.as_str()).collect::<BTreeSet<_>>();
    let mut facts = Vec::with_capacity(report.plan.members.len());
    for member in &report.plan.members {
        let path = StorePath::<String>::from_bytes(member.store_path.as_bytes()).map_err(|error| {
            RunError::Internal(format!("parsing hydrated foreign cache closure member {}: {error}", member.store_path))
        })?;
        let path_info = store
            .cached_path_info(&path)
            .await
            .map_err(|error| {
                RunError::Internal(format!("checking hydrated foreign cache closure path {path}: {error}"))
            })?
            .ok_or_else(|| RunError::Internal(format!("hydrated foreign cache closure path is missing: {path}")))?;
        let verification = crunch_build::signing::verify_pathinfo_signatures_with_store_dir(
            &path_info,
            trusted_keys,
            &plan.target_store_prefix,
        );
        if !verification.is_trusted() {
            return Err(RunError::Internal(format!("foreign cache closure PathInfo has no trusted signature: {path}")));
        }
        let mut references = path_info
            .references
            .iter()
            .map(|reference| reference.to_absolute_path_with_prefix(&plan.target_store_prefix))
            .collect::<Vec<_>>();
        references.sort();
        references.dedup();
        let member_references = member
            .references
            .iter()
            .map(|reference| format!("{}/{}", plan.target_store_prefix, reference))
            .collect::<Vec<_>>();
        if path_info.nar_size != member.nar_size
            || HEXLOWER.encode(&path_info.nar_sha256) != member.nar_sha256_hex
            || references != member_references
        {
            return Err(RunError::Internal(format!(
                "hydrated foreign cache closure PathInfo changed after preflight: {path}"
            )));
        }
        let mut signature_names =
            path_info.signatures.iter().map(|signature| signature.name().to_string()).collect::<Vec<_>>();
        signature_names.sort();
        signature_names.dedup();
        facts.push(ForeignCacheClosureFact {
            logical_path: path.to_absolute_path_with_prefix(&plan.target_store_prefix),
            nar_sha256: member.nar_sha256_hex.clone(),
            nar_size: member.nar_size,
            references,
            signature_names,
            disposition: if imported_paths.contains(member.store_path.as_str()) {
                "remote-substituted".to_string()
            } else {
                "local-reuse".to_string()
            },
            depth: usize::try_from(member.depth)
                .map_err(|_| RunError::Internal("foreign cache closure depth does not fit usize".to_string()))?,
        });
    }
    facts.sort_by(|left, right| left.logical_path.cmp(&right.logical_path));
    Ok(CacheClosureHydration { policy_blake3, facts })
}

fn cache_only_http_cache_url(plan: &ForeignExecutablePlan) -> Result<Url, RunError> {
    let urls = receipt_bound_cache_urls(plan)?;
    let [raw_url] = urls.as_slice() else {
        return Err(RunError::Internal(
            "cache-only foreign realization requires exactly one receipt-bound cache URL".to_string(),
        ));
    };
    let mut url = Url::parse(raw_url)
        .map_err(|error| RunError::Internal(format!("parsing receipt-bound cache URL {raw_url}: {error}")))?;
    if url.fragment().is_some() {
        return Err(RunError::Internal("cache-only foreign cache URL must not contain a fragment".to_string()));
    }
    for (name, _) in url.query_pairs() {
        if !name.starts_with("trusted_public_keys[") || !name.ends_with(']') {
            return Err(RunError::Internal(format!(
                "cache-only foreign cache URL contains an unsupported query parameter: {name}"
            )));
        }
    }
    url.set_query(None);
    Ok(url)
}

fn cache_only_http_closure_limits(
    policy: &ForeignCacheClosurePolicy,
) -> Result<crunch_store::HttpClosureLimits, RunError> {
    let max_members = u32::try_from(policy.max_paths)
        .map_err(|_| RunError::Internal("foreign cache closure max_paths does not fit u32".to_string()))?;
    let max_references = u32::try_from(policy.max_references)
        .map_err(|_| RunError::Internal("foreign cache closure max_references does not fit u32".to_string()))?;
    let max_depth = u32::try_from(policy.max_depth)
        .map_err(|_| RunError::Internal("foreign cache closure max_depth does not fit u32".to_string()))?;
    Ok(crunch_store::HttpClosureLimits {
        max_members,
        max_references,
        max_depth,
        max_narinfo_bytes: crunch_store::MAX_HTTP_CLOSURE_NARINFO_BYTES,
        max_total_nar_bytes: policy.max_nar_bytes,
    })
}

fn validate_prepared_cache_closure(
    closure_plan: &crunch_store::HttpClosurePlan,
    expected_roots: &[StorePath<String>],
    target_store_prefix: &str,
    permitted_paths: &BTreeSet<String>,
) -> Result<(), String> {
    let expected_roots = expected_roots.iter().map(ToString::to_string).collect::<BTreeSet<_>>();
    let planned_roots = closure_plan.roots.iter().cloned().collect::<BTreeSet<_>>();
    if expected_roots != planned_roots {
        return Err("foreign cache closure seed set changed during preflight".to_string());
    }
    let planned_members = closure_plan.members.iter().map(|member| member.store_path.as_str()).collect::<BTreeSet<_>>();
    for root in &expected_roots {
        if !planned_members.contains(root.as_str()) {
            return Err(format!("foreign cache closure seed is absent from the plan: {root}"));
        }
    }
    for member in &closure_plan.members {
        let logical_member = format!("{target_store_prefix}/{}", member.store_path);
        if !permitted_paths.contains(&logical_member) {
            return Err(format!("foreign cache closure member is outside the exact path map: {logical_member}"));
        }
        for reference in &member.references {
            let logical_reference = format!("{target_store_prefix}/{reference}");
            if !permitted_paths.contains(&logical_reference) {
                return Err(format!(
                    "foreign cache closure reference is outside the exact path map: {logical_reference}"
                ));
            }
        }
    }
    Ok(())
}

fn receipt_bound_cache_urls(plan: &ForeignExecutablePlan) -> Result<Vec<String>, RunError> {
    let mut seen = BTreeSet::new();
    let mut urls = Vec::new();
    for audit in &plan.substitution_audit {
        if !audit.store_admission_required {
            return Err(RunError::Internal(format!(
                "foreign plan cache hint bypasses store admission: {}",
                audit.cache_url
            )));
        }
        if seen.insert(audit.cache_url.clone()) {
            urls.push(audit.cache_url.clone());
        }
    }
    if urls.len() > MAX_RECEIPT_BOUND_CACHE_URLS {
        return Err(RunError::Internal(format!("foreign plan cache URL count exceeds {MAX_RECEIPT_BOUND_CACHE_URLS}")));
    }
    Ok(urls)
}

fn selected_root_output_paths(
    plan: &ForeignExecutablePlan,
    selected_root_node_ids: &[String],
) -> Result<Vec<nix_compat::store_path::StorePath<String>>, RunError> {
    let mut outputs = Vec::new();
    for root in &plan.selected_roots {
        if selected_root_node_ids.binary_search(&root.node_id).is_err() {
            continue;
        }
        for output_path in root.target_outputs.values() {
            outputs.push(
                nix_compat::store_path::StorePath::from_absolute_path_with_prefix(
                    output_path.as_bytes(),
                    &plan.target_store_prefix,
                )
                .map_err(|error| {
                    RunError::Internal(format!("parsing selected foreign root output {output_path}: {error}"))
                })?,
            );
        }
    }
    outputs.sort();
    outputs.dedup();
    if outputs.is_empty() {
        return Err(RunError::Internal("foreign realization selected roots have no outputs".to_string()));
    }
    Ok(outputs)
}

async fn ingest_admitted_foreign_sources(
    admitted_sources: &[AdmittedForeignSource],
    source_bundle: &SourceBundleManifest,
    source_admission: &mut crunch_store::SourceAdmission<'_>,
    signing_key: &SigningKey<ed25519_dalek::SigningKey>,
) -> Result<IngestedForeignSources, RunError> {
    let mut prepared = Vec::with_capacity(admitted_sources.len());
    for admitted in admitted_sources {
        let record = source_bundle.records.get(admitted.record_index).ok_or_else(|| {
            RunError::Internal(format!("foreign source record index is out of range: {}", admitted.record_index))
        })?;
        if record.identity != admitted.record_identity || record.content_blake3 != admitted.content_blake3 {
            return Err(RunError::Internal(format!(
                "foreign source admission changed before ingestion: {}",
                admitted.payload_id
            )));
        }
        let scratch = tempfile::Builder::new()
            .prefix(FOREIGN_SOURCE_SCRATCH_PREFIX)
            .tempdir()
            .map_err(|error| RunError::Internal(format!("creating foreign source scratch directory: {error}")))?;
        let payload_path = scratch.path().join(FOREIGN_SOURCE_PAYLOAD_NAME);
        materialize_source_record_exact_payload(record, &payload_path)?;
        prepared.push((admitted, payload_path, scratch));
    }
    for (admitted, payload_path, _) in &prepared {
        source_admission
            .preflight(VerifiedSourceIngestRequest {
                source_path: payload_path,
                logical_store_path: &admitted.target_path,
                source_name: &admitted.payload_id,
                signing_key,
            })
            .await
            .map_err(|error| {
                RunError::Internal(format!(
                    "preflighting foreign source {} at {}: {error}",
                    admitted.payload_id, admitted.target_path
                ))
            })?;
    }

    let mut facts = Vec::with_capacity(prepared.len());
    let mut payload_paths = BTreeMap::new();
    let mut scratch_directories = Vec::with_capacity(prepared.len());
    for (admitted, payload_path, scratch) in prepared {
        let path_info = source_admission
            .ingest(VerifiedSourceIngestRequest {
                source_path: &payload_path,
                logical_store_path: &admitted.target_path,
                source_name: &admitted.payload_id,
                signing_key,
            })
            .await
            .map_err(|error| {
                RunError::Internal(format!(
                    "ingesting foreign source {} at {}: {error}",
                    admitted.payload_id, admitted.target_path
                ))
            })?;
        if path_info.store_path.to_absolute_path_with_prefix(source_admission.store_dir()) != admitted.target_path {
            return Err(RunError::Internal(format!(
                "foreign source store path changed during ingestion: {}",
                admitted.payload_id
            )));
        }
        if payload_paths.insert(admitted.payload_id.clone(), payload_path).is_some() {
            return Err(RunError::Internal(format!(
                "foreign source payload path repeats after admission: {}",
                admitted.payload_id
            )));
        }
        scratch_directories.push(scratch);
        facts.push(ForeignSourceStoreFact {
            payload_id: admitted.payload_id.clone(),
            foreign_path: admitted.foreign_path.clone(),
            target_path: admitted.target_path.clone(),
            descriptor_blake3: admitted.descriptor_blake3.clone(),
            source_record_identity: admitted.record_identity.clone(),
            source_content_blake3: admitted.content_blake3.clone(),
            nar_sha256: HEXLOWER.encode(&path_info.nar_sha256),
            nar_size: path_info.nar_size,
        });
    }
    assert_eq!(facts.len(), admitted_sources.len());
    assert_eq!(payload_paths.len(), admitted_sources.len());
    assert_eq!(scratch_directories.len(), admitted_sources.len());
    Ok(IngestedForeignSources {
        facts,
        payload_paths,
        _scratch_directories: scratch_directories,
    })
}

// r[verify foreign_derivation_import.cache_only_runtime_closure]
// r[verify foreign_derivation_import.live_nixpkgs_realization_proof]
#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn realization_logs_write_executed_outcomes_and_leave_cached_ones_alone() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        std::fs::create_dir_all(&state_dir).unwrap();
        let built = StorePath::<String>::from_name_and_digest_fixed("realize-log-built.drv", [5u8; 20]).unwrap();
        let reused = StorePath::<String>::from_name_and_digest_fixed("realize-log-reused.drv", [6u8; 20]).unwrap();
        let failed = StorePath::<String>::from_name_and_digest_fixed("realize-log-failed.drv", [7u8; 20]).unwrap();

        let mut first = empty_registered_build_result();
        first.outcomes.push(build_outcome(&built, Some("builder said hi\n"), false));
        first.outcomes.push(build_outcome(&reused, None, true));
        first.failed_roots.push(failed_goal(&failed, Some("output admission failed\n")));
        let failures = write_realization_logs(&state_dir, "/nix/store", &first);
        assert!(failures.is_empty(), "fresh logs directory must accept every log: {failures:?}");

        let logs_dir = state_dir.join("logs");
        let built_log = std::fs::read_to_string(logs_dir.join(format!("{built}.log"))).unwrap();
        assert!(built_log.contains("builder said hi"));
        assert!(built_log.contains("# status: success"));
        let failed_log = std::fs::read_to_string(logs_dir.join(format!("{failed}.log"))).unwrap();
        assert!(failed_log.contains("output admission failed"));
        assert!(failed_log.contains("# status: failure"));
        assert!(
            !failed_log.contains("realization root failed"),
            "a retained build log must win over the failure detail: {failed_log}"
        );
        assert!(!logs_dir.join(format!("{reused}.log")).exists(), "a cached outcome without a log writes nothing");

        std::fs::write(logs_dir.join(format!("{reused}.log")), "sentinel\n").unwrap();
        let mut second = empty_registered_build_result();
        second.outcomes.push(build_outcome(&reused, None, true));
        let failures = write_realization_logs(&state_dir, "/nix/store", &second);
        assert!(failures.is_empty());
        assert_eq!(std::fs::read_to_string(logs_dir.join(format!("{reused}.log"))).unwrap(), "sentinel\n");
    }

    #[test]
    fn realization_logs_write_failed_root_failure_detail_when_no_build_log_was_retained() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        std::fs::create_dir_all(&state_dir).unwrap();
        let root =
            StorePath::<String>::from_name_and_digest_fixed("realize-log-builder-failed.drv", [9u8; 20]).unwrap();
        let origin =
            StorePath::<String>::from_name_and_digest_fixed("realize-log-leaf-failed.drv", [10u8; 20]).unwrap();
        let mut result = empty_registered_build_result();
        result.failed_roots.push(failed_goal_with_error(
            &root,
            "builder failed: affinity list\n",
            Some((&origin, "leaf builder failed\n")),
            None,
        ));

        let failures = write_realization_logs(&state_dir, "/nix/store", &result);

        assert!(failures.is_empty(), "a fresh logs directory must accept the failure log: {failures:?}");
        let log = std::fs::read_to_string(state_dir.join("logs").join(format!("{root}.log"))).unwrap();
        assert!(log.contains("# status: failure"), "log header: {log}");
        assert!(
            log.contains(&format!("# derivation: {}", root.name())),
            "log must carry the mantle build header: {log}"
        );
        assert!(log.contains(&format!("# drv_path: {root}")), "log must name the failed drv: {log}");
        assert!(log.contains("builder failed: affinity list"), "log must carry the builder error: {log}");
        assert!(
            log.contains(&format!("origin {}: leaf builder failed", origin.to_absolute_path_with_prefix("/nix/store"))),
            "a dependency failure must keep the origin detail `mantle build` writes: {log}"
        );
    }

    #[test]
    fn realization_logs_skip_failures_that_should_write_failure_log_rejects() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        std::fs::create_dir_all(&state_dir).unwrap();
        let root = StorePath::<String>::from_name_and_digest_fixed("realize-log-preflight.drv", [11u8; 20]).unwrap();
        let mut result = empty_registered_build_result();
        result
            .failed_roots
            .push(failed_goal_with_error(&root, "opening store: pathinfo fallback missing", None, None));

        let failures = write_realization_logs(&state_dir, "/nix/store", &result);

        assert!(failures.is_empty(), "a gated-out failure writes nothing: {failures:?}");
        assert!(
            !state_dir.join("logs").join(format!("{root}.log")).exists(),
            "a preflight failure must not leave a log"
        );
    }

    #[test]
    fn realization_log_failures_are_diagnostics_not_errors() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        std::fs::create_dir_all(&state_dir).unwrap();
        std::fs::write(state_dir.join("logs"), b"not a directory").unwrap();
        let built = StorePath::<String>::from_name_and_digest_fixed("realize-log-blocked.drv", [8u8; 20]).unwrap();
        let mut result = empty_registered_build_result();
        result.outcomes.push(build_outcome(&built, Some("captured\n"), false));

        let failures = write_realization_logs(&state_dir, "/nix/store", &result);

        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].operation, "create-log-dir");
        assert_eq!(failures[0].artifact, "build-log");
        assert!(failures[0].attempted_path.ends_with("logs"), "attempted path: {}", failures[0].attempted_path);
    }

    fn empty_registered_build_result() -> crunch_pipeline::RegisteredBuildResult {
        crunch_pipeline::RegisteredBuildResult {
            outcomes: Vec::new(),
            failed_roots: Vec::new(),
            outputs: Vec::new(),
            hermeticity_audit_events: Vec::new(),
            build_environment_reports: Vec::new(),
            finish_gate_reports: Vec::new(),
            network_policy_reports: Vec::new(),
            workspace_reports: Vec::new(),
            action_result_reports: Vec::new(),
        }
    }

    fn build_outcome(drv_path: &StorePath<String>, log: Option<&str>, cached: bool) -> crunch_build::BuildOutcome {
        crunch_build::BuildOutcome {
            drv_path: drv_path.clone(),
            outputs: BTreeMap::new(),
            substitutions: BTreeMap::new(),
            cached,
            log: log.map(str::to_owned),
        }
    }

    fn failed_goal(drv_path: &StorePath<String>, build_log: Option<&str>) -> crunch_build::FailedGoal {
        failed_goal_with_error(drv_path, "realization root failed", None, build_log)
    }

    fn failed_goal_with_error(
        drv_path: &StorePath<String>,
        error: &str,
        origin: Option<(&StorePath<String>, &str)>,
        build_log: Option<&str>,
    ) -> crunch_build::FailedGoal {
        let drv_key = drv_path.to_absolute_path_with_prefix("/nix/store");
        let (origin_drv_key, origin_error) = match origin {
            Some((origin_path, origin_error)) => {
                (origin_path.to_absolute_path_with_prefix("/nix/store"), origin_error.to_string())
            }
            None => (drv_key.clone(), error.to_string()),
        };
        crunch_build::FailedGoal {
            origin_drv_key,
            drv_key,
            error: error.to_string(),
            origin_error,
            build_log: build_log.map(str::to_owned),
        }
    }

    #[test]
    fn cache_only_observer_removes_every_builder_dependency_and_argument() {
        let input_derivation = StorePath::<String>::from_absolute_path_with_prefix(
            b"/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-input.drv",
            "/nix/store",
        )
        .unwrap();
        let input_source = StorePath::<String>::from_absolute_path_with_prefix(
            b"/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-source",
            "/nix/store",
        )
        .unwrap();
        let mut derivation = Derivation {
            builder: "/bin/should-not-run".to_string(),
            arguments: vec!["--must-not-survive".to_string()],
            ..Default::default()
        };
        derivation.input_derivations.insert(input_derivation, BTreeSet::from(["out".to_string()]));
        derivation.input_sources.insert(input_source);

        let observer = cache_only_observer_derivation(&derivation);

        assert_eq!(observer.builder, CACHE_ONLY_OBSERVER_BUILDER);
        assert!(observer.arguments.is_empty());
        assert!(observer.input_derivations.is_empty());
        assert!(observer.input_sources.is_empty());
        assert_eq!(derivation.builder, "/bin/should-not-run");
        assert_eq!(derivation.arguments, vec!["--must-not-survive"]);
        assert!(!derivation.input_derivations.is_empty());
        assert!(!derivation.input_sources.is_empty());
    }

    #[test]
    fn prepared_cache_closure_accepts_only_exact_mapped_members_and_references() {
        const ROOT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-root";
        const DEPENDENCY: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-dependency";
        const TEST_HASH_BYTES: usize = 32;
        let mut permitted = BTreeSet::from([format!("/nix/store/{ROOT}"), format!("/nix/store/{DEPENDENCY}")]);
        let root =
            StorePath::<String>::from_absolute_path_with_prefix(format!("/nix/store/{ROOT}").as_bytes(), "/nix/store")
                .unwrap();
        let dependency = StorePath::<String>::from_absolute_path_with_prefix(
            format!("/nix/store/{DEPENDENCY}").as_bytes(),
            "/nix/store",
        )
        .unwrap();
        let mut closure_plan = crunch_store::HttpClosurePlan {
            schema: crunch_store::HTTP_CLOSURE_PLAN_SCHEMA.to_string(),
            cache_identity: "https://cache.example.test/".to_string(),
            trust_policy_blake3: "trust".to_string(),
            store_dir: "/nix/store".to_string(),
            roots: vec![ROOT.to_string()],
            limits: crunch_store::HttpClosureLimits::default(),
            total_nar_bytes: 1,
            members: vec![crunch_store::HttpClosurePlanMember {
                store_path: ROOT.to_string(),
                depth: 0,
                references: vec![DEPENDENCY.to_string()],
                nar_sha256_hex: "00".repeat(TEST_HASH_BYTES),
                nar_size: 1,
                narinfo_blake3: "00".repeat(TEST_HASH_BYTES),
            }],
            plan_blake3: "identity-not-used-by-this-pure-validator".to_string(),
        };
        assert!(
            validate_prepared_cache_closure(&closure_plan, std::slice::from_ref(&root), "/nix/store", &permitted)
                .is_ok()
        );

        permitted.remove(&format!("/nix/store/{DEPENDENCY}"));
        let error =
            validate_prepared_cache_closure(&closure_plan, std::slice::from_ref(&root), "/nix/store", &permitted)
                .unwrap_err();
        assert!(error.contains("reference is outside the exact path map"));

        closure_plan.roots = vec![DEPENDENCY.to_string()];
        let error = validate_prepared_cache_closure(&closure_plan, &[root], "/nix/store", &permitted).unwrap_err();
        assert!(error.contains("seed set changed during preflight"));
        assert!(!dependency.to_string().is_empty());
    }
}
