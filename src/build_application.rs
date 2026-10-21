//! Project-backed realization preparation ports.
use std::path::Path;
use std::path::PathBuf;

use mantle_application_contract::EffectKind;
use mantle_application_contract::realization::RealizationEffect;

use crate::RunError;
use crate::build_observation;
use crate::project_build;
use crate::project_retention_fact;

fn output_host_path(info: &snix_store::path_info::PathInfo, output_dir: &Path) -> PathBuf {
    output_dir.join(info.store_path.to_string())
}

pub(crate) fn first_output_path(
    result: &crunch_pipeline::PipelineResult,
    output_dir: &Path,
) -> Result<Option<PathBuf>, RunError> {
    let Some(first) = result.outcomes.first() else {
        return Ok(None);
    };
    if first.outputs.is_empty() {
        return Ok(None);
    }
    let effects = build_observation::plan(RealizationEffect::OutputSelection)?;
    let mut selected = None;
    for outcome in &result.outcomes {
        let Some(info) = outcome.outputs.get("out").or_else(|| outcome.outputs.values().next()) else {
            break;
        };
        let path = output_host_path(info, output_dir);
        if path.exists() {
            selected = Some(path);
            break;
        }
    }
    build_observation::classify(&effects, "realization-output", EffectKind::ReadFiles, selected.is_some())?;
    Ok(selected)
}

pub(crate) struct SelectedRunOutputRequest<'a> {
    pub(crate) result: &'a crunch_pipeline::PipelineResult,
    pub(crate) output_dir: &'a Path,
    pub(crate) store_prefix: &'a str,
    pub(crate) target_label: &'a str,
}

pub(crate) fn selected_run_output_path(request: SelectedRunOutputRequest<'_>) -> Result<PathBuf, RunError> {
    debug_assert!(request.store_prefix.starts_with('/'));
    debug_assert!(!request.target_label.is_empty());
    if !request.result.failed.is_empty() {
        return Err(RunError::Build(format!("run target {} has failed build roots", request.target_label)));
    }
    let outcome_count = request.result.outcomes.len();
    if outcome_count != crate::EXPECTED_RUN_OUTCOME_COUNT {
        return Err(RunError::Internal(format!(
            "run target {} produced {outcome_count} derivations; expected exactly {}",
            request.target_label,
            crate::EXPECTED_RUN_OUTCOME_COUNT
        )));
    }
    let outcome = request
        .result
        .outcomes
        .first()
        .ok_or_else(|| RunError::Internal(format!("run target {} produced no outputs", request.target_label)))?;
    let info = outcome
        .outputs
        .get("out")
        .or_else(|| outcome.outputs.values().next())
        .ok_or_else(|| RunError::Internal(format!("run target {} produced no outputs", request.target_label)))?;
    let effects = build_observation::plan(RealizationEffect::OutputSelection)?;
    let host_path = output_host_path(info, request.output_dir);
    let present = host_path.exists();
    build_observation::classify(&effects, "realization-output", EffectKind::ReadFiles, present)?;
    if !present {
        return Err(RunError::Internal(format!("selected run output is missing on disk: {}", host_path.display())));
    }
    Ok(host_path)
}

pub(crate) fn watch_time() -> Result<std::time::Duration, RunError> {
    let effects = build_observation::plan(RealizationEffect::Clock)?;
    let observed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    build_observation::classify(&effects, "realization-clock", EffectKind::ReadClock, observed.is_ok())?;
    observed.map_err(|error| RunError::Internal(format!("watch run clock: {error}")))
}

pub(crate) fn persist_causal_trace(state_dir: &Path, trace: &mantle_causal_trace_core::Trace) -> Result<(), RunError> {
    let effects = build_observation::plan(RealizationEffect::Reporting)?;
    let persisted =
        crate::causal_trace_diagnostic::write_trace(state_dir, trace)
            .map_err(RunError::Internal)
            .and_then(|path| {
                let bytes = std::fs::read(&path)
                    .map_err(|error| RunError::Internal(format!("reading causal trace {}: {error}", path.display())))?;
                let observed: mantle_causal_trace_core::Trace = serde_json::from_slice(&bytes)
                    .map_err(|error| RunError::Internal(format!("decoding persisted causal trace: {error}")))?;
                if &observed != trace {
                    return Err(RunError::Internal(
                        "persisted causal trace contradicts worker observation".to_string(),
                    ));
                }
                Ok(())
            });
    build_observation::classify(&effects, "realization-reporting", EffectKind::WriteFiles, persisted.is_ok())?;
    persisted
}

pub(crate) fn read_compile_driver_runtime(
    path: &Path,
) -> Result<crunch_build::compile_driver::CompileDriverRuntime, RunError> {
    admit_compile_driver_environment()?;
    let effects = build_observation::plan(RealizationEffect::RuntimeRead)?;
    let runtime = crunch_build::compile_driver::CompileDriverRuntime::read(path);
    build_observation::classify(&effects, "realization-runtime-read", EffectKind::ReadFiles, runtime.is_ok())?;
    runtime.map_err(|error| RunError::Build(format!("compile driver runtime: {error}")))
}

pub(crate) fn admit_compile_driver_environment() -> Result<(), RunError> {
    let effects = build_observation::plan(RealizationEffect::RuntimeEnvironment)?;
    let admitted = crunch_build::compile_driver::reject_inherited_runtime_environment();
    build_observation::classify(
        &effects,
        "realization-runtime-environment",
        EffectKind::ReadEnvironment,
        admitted.is_ok(),
    )?;
    admitted.map_err(|error| RunError::Build(error.to_string()))
}

pub(crate) fn resolve_project_target(
    target: &project_build::BuildTarget,
    cwd: &Path,
    import_paths: &[PathBuf],
) -> Result<project_build::ResolvedProject, RunError> {
    let effects = build_observation::plan(RealizationEffect::ProjectRead)?;
    let resolved = project_build::resolve_project_target(target, cwd, import_paths).and_then(|project| {
        if project.root_file.as_os_str().is_empty() {
            return Err(RunError::Internal("project resolution returned no manifest identity".to_string()));
        }
        Ok(project)
    });
    build_observation::classify(&effects, "realization-project", EffectKind::ReadFiles, resolved.is_ok())?;
    resolved
}

pub(crate) fn preflight_backend(
    backend: crunch_store::StoreBackend,
    state_dir: &Path,
    store_prefix: &str,
    base_state_dirs: &[PathBuf],
    source_admission: Option<&dyn crunch_store::StoreSourceAdmission>,
) -> Result<(), RunError> {
    let effects = build_observation::plan(RealizationEffect::Preparation)?;
    let identity = crunch_store::StoreConfig::preflight_backend_identity_for(
        backend,
        state_dir,
        store_prefix,
        base_state_dirs,
        source_admission,
    );
    build_observation::classify(&effects, "realization-preparation", EffectKind::StoreAccess, identity.is_ok())?;
    identity.map_err(|error| RunError::Internal(format!("opening store: {error}")))
}

const PROJECT_RETENTION_IDENTITY_DOMAIN: &[u8] = b"mantle.project.retention-identity.v1";
const CURRENT_PROJECT_LOCK_FILE: &str = "mantle.lock";
const LEGACY_PROJECT_LOCK_FILE: &str = "crunch.lock";

pub(crate) fn project_output_root_registration(
    project: &project_build::ResolvedProject,
) -> Result<crunch_store::RootRegistration, RunError> {
    let project_dir = project.root_file.parent().ok_or_else(|| {
        RunError::Internal(format!("project root file has no parent: {}", project.root_file.display()))
    })?;
    let effects = build_observation::plan(RealizationEffect::RetentionRead)?;
    let facts = (|| {
        let manifest = project_retention_fact::read_bounded_project_retention_fact(&project.root_file)?;
        let current_lock = project_dir.join(CURRENT_PROJECT_LOCK_FILE);
        let legacy_lock = project_dir.join(LEGACY_PROJECT_LOCK_FILE);
        if current_lock.exists() && legacy_lock.exists() {
            return Err(RunError::Internal(format!(
                "both {CURRENT_PROJECT_LOCK_FILE} and {LEGACY_PROJECT_LOCK_FILE} exist in {}",
                project_dir.display()
            )));
        }
        let lock = if current_lock.exists() {
            Some(project_retention_fact::read_bounded_project_retention_fact(&current_lock)?)
        } else if legacy_lock.exists() {
            Some(project_retention_fact::read_bounded_project_retention_fact(&legacy_lock)?)
        } else {
            None
        };
        Ok((manifest, lock))
    })();
    build_observation::classify(&effects, "realization-retention", EffectKind::ReadFiles, facts.is_ok())?;
    let (manifest, lock) = facts?;
    Ok(crunch_store::RootRegistration {
        class: crunch_store::GcRootClass::ProjectOutputGeneration,
        owner_scope: "project".to_string(),
        project_identity: Some(project_retention_identity(&manifest)),
        selector: Some(project_retention_selector(&project.target)),
        generation: None,
        generation_identity: Some(project_generation_identity(lock.as_deref())),
        lease: None,
        removal_requested: false,
    })
}

fn project_retention_identity(manifest_bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(PROJECT_RETENTION_IDENTITY_DOMAIN);
    hash_project_retention_fact(&mut hasher, manifest_bytes);
    format!("b3:{}", hasher.finalize().to_hex())
}

fn project_generation_identity(lock_bytes: Option<&[u8]>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle.project.retention-generation.v1");
    match lock_bytes {
        Some(bytes) => {
            hasher.update(&[1]);
            hash_project_retention_fact(&mut hasher, bytes);
        }
        None => {
            hasher.update(&[0]);
        }
    }
    format!("b3:{}", hasher.finalize().to_hex())
}

fn hash_project_retention_fact(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u128).to_be_bytes());
    hasher.update(bytes);
}

pub(crate) fn project_retention_selector(target: &project_build::ProjectTarget) -> String {
    match target {
        project_build::ProjectTarget::Default => "default".to_string(),
        project_build::ProjectTarget::Attribute(segments) => segments.join("."),
        project_build::ProjectTarget::AllPackages => "all-packages".to_string(),
        project_build::ProjectTarget::AllChecks => "all-checks".to_string(),
        project_build::ProjectTarget::DefaultShell => "default-shell".to_string(),
        project_build::ProjectTarget::NamedShell(name) => format!("shell.{name}"),
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn a_failed_root_prevents_running_an_existing_successful_output() {
        let directory = tempfile::tempdir().expect("output directory");
        let path =
            nix_compat::store_path::StorePath::from_name_and_digest_fixed("application", [3; 20]).expect("output path");
        let selected = directory.path().join(path.to_string());
        std::fs::write(directory.path().join("target"), b"application").expect("target file");
        std::os::unix::fs::symlink("target", &selected).expect("materialized store symlink");
        let info = snix_store::path_info::PathInfo {
            store_path: path,
            node: snix_castore::Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("target").expect("target"),
            },
            references: Vec::new(),
            nar_size: 1,
            nar_sha256: [0; 32],
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        };
        let mut result = crunch_pipeline::PipelineResult {
            outcomes: vec![crunch_build::BuildOutcome {
                drv_path: nix_compat::store_path::StorePath::from_name_and_digest_fixed("application.drv", [4; 20])
                    .expect("drv path"),
                outputs: std::collections::BTreeMap::from([("out".to_string(), info)]),
                substitutions: Default::default(),
                cached: true,
                log: None,
            }],
            failed: Vec::new(),
            fod_mismatches: Vec::new(),
            root_labels: Default::default(),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
            build_environment_reports: Vec::new(),
            finish_gate_reports: Vec::new(),
            network_policy_reports: Vec::new(),
            workspace_reports: Vec::new(),
            action_result_reports: Vec::new(),
            native_dynamic_plans: Vec::new(),
            plan_output_bindings: Vec::new(),
            priority_decisions: Vec::new(),
            overlay_report: None,
            store_layer_selections: Vec::new(),
        };
        assert_eq!(
            selected_run_output_path(SelectedRunOutputRequest {
                result: &result,
                output_dir: directory.path(),
                store_prefix: "/mantle/store",
                target_label: "application",
            })
            .expect("successful output is runnable"),
            selected
        );
        result.failed.push(crunch_build::FailedGoal {
            drv_key: "/mantle/store/failed.drv".to_string(),
            origin_drv_key: "/mantle/store/failed.drv".to_string(),
            error: "dependency build failed".to_string(),
            origin_error: "dependency build failed".to_string(),
            build_log: None,
        });
        assert!(matches!(
            selected_run_output_path(SelectedRunOutputRequest {
                result: &result,
                output_dir: directory.path(),
                store_prefix: "/mantle/store",
                target_label: "application",
            }),
            Err(RunError::Build(_))
        ));
    }
}
