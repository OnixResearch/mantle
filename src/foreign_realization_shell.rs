//! Imperative shell for receipt-bound foreign realization.
// r[impl foreign_derivation_import.realization_adapter]
// r[impl foreign_derivation_import.source_materialization]

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
use nix_compat::narinfo::SigningKey;
use serde::Deserialize;
use serde::Serialize;

use crate::RunError;
use crate::foreign_derivation_import::ImportReceipt;
use crate::foreign_executable_plan::ExecutableNativeUnit;
use crate::foreign_executable_plan::ForeignExecutablePlan;
use crate::foreign_realization::AdmittedForeignSource;
use crate::foreign_realization::ForeignRealizationAdmission;
use crate::foreign_realization::ForeignSourceAdmission;
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
    let admitted = validate_foreign_realization_admission(ForeignRealizationAdmission {
        plan: request.plan,
        import_receipt: request.import_receipt,
        selected_root_node_ids: request.selected_root_node_ids,
        execution_profiles: request.execution_profiles,
        remote_execution_requested: request.remote_execution_requested,
    })
    .map_err(|error| RunError::Internal(error.to_string()))?;
    let admitted_sources = validate_foreign_source_admission(ForeignSourceAdmission {
        source_requirements: &request.plan.source_requirements,
        source_bundle: request.source_bundle,
        expected_manifest_blake3: request.expected_source_bundle_blake3,
    })
    .map_err(|error| RunError::Internal(error.to_string()))?;
    let cache_urls = if request.substitution_enabled {
        receipt_bound_cache_urls(request.plan)?
    } else {
        Vec::new()
    };
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
    let ingested_sources = ingest_admitted_foreign_sources(
        &admitted_sources,
        request.source_bundle,
        &mut store,
        &request.keypair.signing_key,
    )
    .await?;

    let mut registry = DerivationRegistry::new(&request.plan.target_store_prefix);
    let roots = admitted.selected_root_paths.clone();
    let mut expected_outputs = Vec::new();
    for unit in &admitted.units {
        for (output_name, output) in &unit.derivation.outputs {
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
                unit.derivation.clone(),
                false,
                None,
                unit.execution_profile.clone(),
            )
            .map_err(|error| {
                RunError::Internal(format!("registering foreign realization unit {}: {error}", unit.node_id))
            })?;
    }
    let retained_outputs = selected_root_output_paths(request.plan, request.selected_root_node_ids)?;
    let source_fetch_overrides =
        foreign_source_fetch_overrides(request.plan, &ingested_sources.facts, &ingested_sources.payload_paths)?;
    let source_policy = if request.offline {
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
        substituter_urls: cache_urls.clone(),
        hermeticity_mode: HermeticityMode::Strict,
        base_state_dirs: request.base_state_dirs.to_vec(),
        keypair: request.keypair.clone(),
        trusted_keys: request.trusted_keys.to_vec(),
        trust_unsigned: false,
        root_retention_source: None,
        source_fetch_overrides,
        remote_enabled: false,
    };
    let build_result =
        crunch_pipeline::build_registered_derivations(&build_config, store, &mut registry, RegisteredBuildRequest {
            roots: &roots,
            expected_outputs: &expected_outputs,
            retained_outputs: &retained_outputs,
            source_policy,
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
    store: &mut StoreHandle,
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
        store
            .preflight_verified_source(VerifiedSourceIngestRequest {
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
        let path_info = store
            .ingest_verified_source(VerifiedSourceIngestRequest {
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
        if path_info.store_path.to_absolute_path_with_prefix(store.store_dir()) != admitted.target_path {
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
