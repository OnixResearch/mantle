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
use crate::foreign_realization::ForeignCacheClosurePolicy;
use crate::foreign_realization::ForeignRealizationAdmission;
use crate::foreign_realization::ForeignSourceAdmission;
use crate::foreign_realization::is_cache_only_foreign_plan;
use crate::foreign_realization::validate_cache_only_source_bundle;
use crate::foreign_realization::validate_foreign_cache_closure_policy;
use crate::foreign_realization::validate_foreign_realization_admission;
use crate::foreign_realization::validate_foreign_source_admission;
use crate::foreign_realization_receipt::ForeignRealizationReceipt;
use crate::foreign_realization_receipt::ForeignReceiptInput;
use crate::foreign_realization_receipt::build_foreign_realization_receipt;
use crate::source_bundle::SourceBundleManifest;
use crate::source_bundle::materialize_source_record_exact_payload;

const FOREIGN_SOURCE_SCRATCH_PREFIX: &str = "mantle-foreign-source-";
const FOREIGN_SOURCE_PAYLOAD_NAME: &str = "payload";
const FOREIGN_PLAN_FILE_IDENTITY: &str = "foreign-executable-plan";
const CACHE_ONLY_OBSERVER_BUILDER: &str = "builtin:cache-only-observation";
const MAX_RECEIPT_BOUND_CACHE_URLS: usize = 64;
const MAX_FOREIGN_SOURCE_FETCH_OVERRIDES: usize = 65_536;
const FETCH_MODE_RECURSIVE: &str = "recursive";

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
    pub(crate) base_state_dirs: &'a [std::path::PathBuf],
    pub(crate) keypair: &'a KeyPair,
    pub(crate) trusted_keys: &'a [nix_compat::narinfo::VerifyingKey],
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
) -> Result<ForeignRealizationReceipt, RunError> {
    if request.max_jobs == 0 {
        return Err(RunError::Internal("foreign realization job bound must be nonzero".to_string()));
    }
    if request.offline && request.substitution_enabled {
        return Err(RunError::Internal("offline foreign realization cannot enable cache substitution".to_string()));
    }
    let cache_only = is_cache_only_foreign_plan(request.plan);
    if cache_only && (!request.substitution_enabled || request.offline) {
        return Err(RunError::Internal("cache-only foreign realization requires online substitution".to_string()));
    }
    let admitted = validate_foreign_realization_admission(ForeignRealizationAdmission {
        plan: request.plan,
        import_receipt: request.import_receipt,
        selected_root_node_ids: request.selected_root_node_ids,
        execution_profiles: request.execution_profiles,
        remote_execution_requested: request.remote_execution_requested,
    })
    .map_err(|error| RunError::Internal(error.to_string()))?;
    let admitted_sources = if cache_only {
        let policy = request.cache_closure_policy.ok_or_else(|| {
            RunError::Internal("cache-only foreign realization requires a cache closure policy".to_string())
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
    let mut store = StoreHandle::open(crunch_store::StoreConfig {
        state_dir: request.state_dir.to_path_buf(),
        output_dir: request.output_dir.to_path_buf(),
        remote_cache_urls: cache_urls.clone(),
        fallback_mode: crunch_store::StoreFallbackMode::Strict,
        store_dir: request.plan.target_store_prefix.clone(),
        base_state_dirs: request.base_state_dirs.to_vec(),
    })
    .await
    .map_err(|error| RunError::Internal(format!("opening foreign realization store: {error}")))?;
    let ingested_sources = if cache_only {
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
    let cache_closure = if cache_only {
        let policy = request.cache_closure_policy.ok_or_else(|| {
            RunError::Internal("cache-only foreign realization requires a cache closure policy".to_string())
        })?;
        let hydration = hydrate_cache_only_runtime_closure(
            &mut store,
            &retained_outputs,
            request.plan,
            policy,
            request.trusted_keys,
        )
        .await?;
        drop(store);
        store = StoreHandle::open(crunch_store::StoreConfig {
            state_dir: request.state_dir.to_path_buf(),
            output_dir: request.output_dir.to_path_buf(),
            remote_cache_urls: Vec::new(),
            fallback_mode: crunch_store::StoreFallbackMode::Strict,
            store_dir: request.plan.target_store_prefix.clone(),
            base_state_dirs: request.base_state_dirs.to_vec(),
        })
        .await
        .map_err(|error| RunError::Internal(format!("reopening hydrated foreign realization store: {error}")))?;
        Some(hydration)
    } else {
        None
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
    let source_policy = if request.offline || cache_only {
        FetchSourcePolicy::RequireOverride
    } else {
        FetchSourcePolicy::AllowNetwork
    };
    let build_config = BuildConfig {
        file: FOREIGN_PLAN_FILE_IDENTITY.into(),
        import_paths: Vec::new(),
        output_dir: request.output_dir.to_path_buf(),
        state_dir: request.state_dir.to_path_buf(),
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
        substituter_urls: if cache_only { Vec::new() } else { cache_urls.clone() },
        hermeticity_mode: HermeticityMode::Strict,
        base_state_dirs: request.base_state_dirs.to_vec(),
        keypair: request.keypair.clone(),
        trusted_keys: request.trusted_keys.to_vec(),
        trust_unsigned: false,
        root_retention_source: None,
        root_registration: None,
        source_fetch_overrides,
        remote_enabled: false,
        interchange_dir: None,
    };
    let build_result =
        crunch_pipeline::build_registered_derivations(&build_config, store, &mut registry, RegisteredBuildRequest {
            roots: &roots,
            expected_outputs: &expected_outputs,
            retained_outputs: &retained_outputs,
            source_policy,
            cache_only,
        })
        .await
        .map_err(|error| RunError::Internal(format!("realizing foreign derivation graph: {error}")))?;
    build_foreign_realization_receipt(ForeignReceiptInput {
        plan: request.plan,
        source_bundle_manifest_blake3: request.expected_source_bundle_blake3,
        selected_root_node_ids: &admitted.selected_root_node_ids,
        source_facts: &ingested_sources.facts,
        build_result: &build_result,
        substitution_enabled: request.substitution_enabled,
        offline: request.offline,
        ordered_cache_urls: &cache_urls,
        cache_closure: cache_closure.as_ref(),
    })
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

async fn hydrate_cache_only_runtime_closure(
    store: &mut StoreHandle,
    roots: &[StorePath<String>],
    plan: &ForeignExecutablePlan,
    policy: &ForeignCacheClosurePolicy,
    trusted_keys: &[nix_compat::narinfo::VerifyingKey],
) -> Result<CacheClosureHydration, RunError> {
    validate_foreign_cache_closure_policy(policy).map_err(|error| RunError::Internal(error.to_string()))?;
    if trusted_keys.is_empty() {
        return Err(RunError::Internal("cache-only foreign realization has no trusted PathInfo keys".to_string()));
    }
    let [root] = roots else {
        return Err(RunError::Internal(
            "cache-only foreign realization requires exactly one selected output root".to_string(),
        ));
    };
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
    let root_path = root.to_absolute_path_with_prefix(&plan.target_store_prefix);
    if !permitted_paths.contains(&root_path) {
        return Err(RunError::Internal(format!("cache-only selected root is outside the exact path map: {root_path}")));
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
        root,
        &pull_options,
        limits,
        |closure_plan| {
            validate_prepared_cache_closure(
                closure_plan,
                &root.to_string(),
                &plan.target_store_prefix,
                &permitted_paths,
            )
        },
    )
    .await
    .map_err(|error| RunError::Internal(format!("hydrating preflighted foreign cache closure: {error}")))?;
    if !report.root_admitted {
        return Err(RunError::Internal("foreign cache closure root was not admitted after hydration".to_string()));
    }
    let imported_paths = report.pull.paths.iter().map(|path| path.store_path.as_str()).collect::<BTreeSet<_>>();
    let mut facts = Vec::with_capacity(report.plan.members.len());
    for member in &report.plan.members {
        let path = StorePath::<String>::from_bytes(member.store_path.as_bytes()).map_err(|error| {
            RunError::Internal(format!("parsing hydrated foreign cache closure member {}: {error}", member.store_path))
        })?;
        let path_info = store
            .export_cached_path_info(&path)
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
    expected_root: &str,
    target_store_prefix: &str,
    permitted_paths: &BTreeSet<String>,
) -> Result<(), String> {
    if closure_plan.root != expected_root {
        return Err(format!(
            "foreign cache closure root changed during preflight: expected {expected_root}, got {}",
            closure_plan.root
        ));
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
        let mut closure_plan = crunch_store::HttpClosurePlan {
            schema: crunch_store::HTTP_CLOSURE_PLAN_SCHEMA.to_string(),
            cache_identity: "https://cache.example.test/".to_string(),
            trust_policy_blake3: "trust".to_string(),
            store_dir: "/nix/store".to_string(),
            root: ROOT.to_string(),
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
        assert!(validate_prepared_cache_closure(&closure_plan, ROOT, "/nix/store", &permitted).is_ok());

        permitted.remove(&format!("/nix/store/{DEPENDENCY}"));
        let error = validate_prepared_cache_closure(&closure_plan, ROOT, "/nix/store", &permitted).unwrap_err();
        assert!(error.contains("reference is outside the exact path map"));

        closure_plan.root = DEPENDENCY.to_string();
        let error = validate_prepared_cache_closure(&closure_plan, ROOT, "/nix/store", &permitted).unwrap_err();
        assert!(error.contains("root changed during preflight"));
    }
}
