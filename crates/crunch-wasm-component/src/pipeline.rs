use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crunch_wasm_component_core::AotMode;
use crunch_wasm_component_core::BoundedComponentClaim;
use crunch_wasm_component_core::BuildValidationBinding;
use crunch_wasm_component_core::ComponentStageKind;
use crunch_wasm_component_core::ComponentStageStatus;
use crunch_wasm_component_core::PortableAdmissionRequest;
use crunch_wasm_component_core::StoreObject;
use crunch_wasm_component_core::TransformAdmission;
use crunch_wasm_component_core::bind_portable_admission;
use crunch_wasm_component_core::cohort_identity;

use crate::ComponentPipelineRequest;
use crate::Error;
use crate::PipelineExecutionReport;
use crate::PipelinePaths;
use crate::aot::run_aot_stage;
use crate::materialization::MaterializationOptions;
use crate::materialization::materialize_bundle;
use crate::octet::run_octet_validation;
use crate::preflight::PreparedPipeline;
use crate::preflight::prepare_pipeline;
use crate::reporting::ExecutionState;
use crate::reporting::StageEvidence;
use crate::reporting::finish_execution;
use crate::stages::ExecutedArtifact;
use crate::stages::StageWorkspace;
use crate::stages::ValidationStage;
use crate::stages::create_workspace;
use crate::stages::publish_validated_artifact;
use crate::stages::run_binding_stage;
use crate::stages::run_compilation_stage;
use crate::stages::run_composition_stage;
use crate::stages::run_metadata_normalization_stage;
use crate::stages::run_runtime_smoke;
use crate::stages::run_validation_stage;
use crate::stages::run_virtualization_stage;
use crate::stages::run_wkg_stage;
use crate::stages::stage_source_and_inputs;
use crate::wizer::run_wizer_pipeline;

pub fn run_component_pipeline(
    request: ComponentPipelineRequest,
    paths: PipelinePaths,
) -> Result<PipelineExecutionReport, Error> {
    validate_paths(&paths)?;
    let prepared = prepare_pipeline(request)?;
    let work_temporary = tempfile::Builder::new()
        .prefix(".mantle-wasm-component-work-")
        .tempdir_in(&paths.scratch_parent)
        .map_err(|error| Error::io("creating component work directory", &paths.scratch_parent, error))?;
    let publication_temporary = tempfile::Builder::new()
        .prefix(".mantle-wasm-component-publication-")
        .tempdir_in(&paths.scratch_parent)
        .map_err(|error| Error::io("creating component publication directory", &paths.scratch_parent, error))?;
    let work_root = work_temporary.path().to_path_buf();
    let publication_root = publication_temporary.path().to_path_buf();
    let workspace = create_workspace(work_root, publication_root.clone(), paths.evidence_dir.clone())?;
    let mut state = ExecutionState::new(prepared.request_blake3.clone(), prepared.request.manifest.non_claims.clone())?;
    state.cohort_blake3 = Some(prepared.toolchain.manifest.cohort_identity_blake3.clone());
    let execution = execute_available_pipeline(&prepared, &workspace, &mut state);
    if let Err(error) = execution {
        state.block("pipeline-execution-error", "pipeline", error.to_string());
    }
    let pipeline_evidence = finish_execution(state, publication_root, &paths.evidence_dir)?;
    drop(publication_temporary);
    drop(work_temporary);
    debug_assert!(paths.evidence_dir.join("execution-report.json").is_file());
    debug_assert_eq!(pipeline_evidence.final_status == "succeeded", pipeline_evidence.blockers.is_empty());
    Ok(pipeline_evidence)
}

struct StageArtifact {
    path: PathBuf,
    object: StoreObject,
}

struct PortableOutput {
    stage: StageArtifact,
    build_validation: BuildValidationBinding,
}

fn execute_available_pipeline(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
) -> Result<(), Error> {
    debug_assert!(workspace.root.is_absolute());
    debug_assert_eq!(state.cohort_blake3, Some(prepared.toolchain.manifest.cohort_identity_blake3.clone()));
    stage_source_and_inputs(prepared, workspace, state)?;
    if !run_wkg_stage(prepared, workspace, state)? || !run_binding_stage(prepared, workspace, state)? {
        add_not_run_bundle(state)?;
        return Ok(());
    }
    let Some(compiled) = compile_and_validate(prepared, workspace, state)? else {
        add_not_run_bundle(state)?;
        return Ok(());
    };
    let Some(wizer) = run_wizer_pipeline(prepared, workspace, state, compiled)? else {
        add_not_run_bundle(state)?;
        return Ok(());
    };
    let Some(composed) = compose_and_validate(prepared, workspace, state, &wizer.component.path)? else {
        add_not_run_bundle(state)?;
        return Ok(());
    };
    let Some(virtualized) = virtualize_and_validate(prepared, workspace, state, &composed.path)? else {
        add_not_run_bundle(state)?;
        return Ok(());
    };
    let Some(portable) = normalize_and_validate(prepared, workspace, state, &virtualized.path)? else {
        add_not_run_bundle(state)?;
        return Ok(());
    };
    if !run_runtime_smoke(prepared, workspace, state, &portable.stage.path, &portable.stage.object)? {
        add_not_run_bundle(state)?;
        return Ok(());
    }
    admit_and_materialize(prepared, workspace, state, portable, wizer.admission)
}

fn compile_and_validate(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
) -> Result<Option<ExecutedArtifact>, Error> {
    let Some(compiled) = run_compilation_stage(prepared, workspace, state)? else {
        return Ok(None);
    };
    let is_core_module = prepared.request.cargo_core_module_relative_path.is_some();
    let validation_claim = if is_core_module {
        BoundedComponentClaim::CoreModuleValidated
    } else {
        BoundedComponentClaim::PortableBytesValidated
    };
    if run_validation_stage(prepared, workspace, state, ValidationStage {
        stage_key: "compiled-validation",
        artifact_path: &compiled.path,
        artifact: &compiled.object,
        claim: validation_claim,
    })?
    .is_none()
    {
        return Ok(None);
    }
    let published_name = if is_core_module {
        "compiled-core.wasm"
    } else {
        "compiled.wasm"
    };
    let artifact_role = if is_core_module {
        "compiled-core-module"
    } else {
        "compiled-component"
    };
    publish_validated_artifact(&compiled.path, published_name, &compiled.object, workspace)?;
    state.add_artifact(artifact_role, &compiled.object)?;
    debug_assert!(compiled.path.is_file());
    debug_assert_eq!(is_core_module, published_name == "compiled-core.wasm");
    Ok(Some(compiled))
}

fn compose_and_validate(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    compiled_path: &Path,
) -> Result<Option<StageArtifact>, Error> {
    let Some((path, object)) = run_composition_stage(prepared, workspace, state, compiled_path)? else {
        return Ok(None);
    };
    if run_validation_stage(prepared, workspace, state, ValidationStage {
        stage_key: "composed-validation",
        artifact_path: &path,
        artifact: &object,
        claim: BoundedComponentClaim::PortableBytesValidated,
    })?
    .is_none()
    {
        return Ok(None);
    }
    publish_validated_artifact(&path, "composed.wasm", &object, workspace)?;
    state.add_artifact("composed-component", &object)?;
    Ok(Some(StageArtifact { path, object }))
}

fn virtualize_and_validate(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    composed_path: &Path,
) -> Result<Option<StageArtifact>, Error> {
    let Some((path, object)) = run_virtualization_stage(prepared, workspace, state, composed_path)? else {
        return Ok(None);
    };
    if run_validation_stage(prepared, workspace, state, ValidationStage {
        stage_key: "virtualized-validation",
        artifact_path: &path,
        artifact: &object,
        claim: BoundedComponentClaim::PortableBytesValidated,
    })?
    .is_none()
    {
        return Ok(None);
    }
    publish_validated_artifact(&path, "virtualized.wasm", &object, workspace)?;
    state.add_artifact("virtualized-component", &object)?;
    Ok(Some(StageArtifact { path, object }))
}

fn normalize_and_validate(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    virtualized_path: &Path,
) -> Result<Option<PortableOutput>, Error> {
    debug_assert!(virtualized_path.is_absolute());
    debug_assert!(virtualized_path.is_file());
    let Some((path, object)) = run_metadata_normalization_stage(prepared, workspace, state, virtualized_path)? else {
        return Ok(None);
    };
    let Some(build_validation) = run_validation_stage(prepared, workspace, state, ValidationStage {
        stage_key: "portable-validation",
        artifact_path: &path,
        artifact: &object,
        claim: BoundedComponentClaim::PortableBytesValidated,
    })?
    else {
        return Ok(None);
    };
    publish_validated_artifact(&path, "normalized.wasm", &object, workspace)?;
    state.add_artifact("normalized-portable-component", &object)?;
    Ok(Some(PortableOutput {
        stage: StageArtifact { path, object },
        build_validation,
    }))
}

fn admit_and_materialize(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    portable: PortableOutput,
    wizer: Option<TransformAdmission>,
) -> Result<(), Error> {
    let Some(octet) = run_octet_validation(prepared, workspace, state, &portable.stage.path, &portable.stage.object)?
    else {
        add_not_run_bundle(state)?;
        return Ok(());
    };
    let admission_decision = bind_portable_admission(PortableAdmissionRequest {
        artifact: portable.stage.object.clone(),
        build_validation: portable.build_validation,
        octet_validation: Some(octet.binding),
        octet_required: true,
        expected_octet_profile_blake3: prepared.toolchain.manifest.octet.profile_identity_blake3.clone(),
        expected_octet_cohort_blake3: prepared.toolchain.manifest.octet.wasm_tools_cohort_identity_blake3.clone(),
    });
    let Some(admission) = admission_decision.admission else {
        for blocker in admission_decision.blockers {
            state.block(&blocker.code, "portable-admission", blocker.message);
        }
        add_not_run_bundle(state)?;
        return Ok(());
    };
    debug_assert_eq!(octet.evidence.artifact.digest_blake3, admission.artifact.digest_blake3);
    let aot = run_aot_stage(prepared, workspace, state, &portable.stage.path, &admission)?;
    if prepared.request.manifest.aot.mode == AotMode::TrustedNative && aot.is_none() {
        add_not_run_bundle(state)?;
        return Ok(());
    }
    materialize_bundle(prepared, workspace, state, MaterializationOptions {
        final_portable: &portable.stage.object,
        wizer,
        aot,
    })?;
    Ok(())
}

fn add_not_run_bundle(state: &mut ExecutionState) -> Result<(), Error> {
    if state.stage_inputs.iter().any(|stage| stage.stage_key == "materialization-bundle") {
        return Ok(());
    }
    state.push_stage(
        "materialization-bundle",
        ComponentStageKind::MaterializationBundle,
        ComponentStageStatus::NotRun,
        StageEvidence::new(None, None, None, Vec::new()),
    )
}

fn validate_paths(paths: &PipelinePaths) -> Result<(), Error> {
    if !paths.evidence_dir.is_absolute() || !paths.scratch_parent.is_absolute() {
        return Err(Error::Invalid("component output and scratch parent must be absolute".to_string()));
    }
    if paths.evidence_dir.exists() {
        return Err(Error::Invalid(format!("component output already exists: {}", paths.evidence_dir.display())));
    }
    fs::create_dir_all(&paths.scratch_parent)
        .map_err(|error| Error::io("creating component scratch parent", &paths.scratch_parent, error))?;
    let evidence_parent = paths
        .evidence_dir
        .parent()
        .ok_or_else(|| Error::Invalid("component output must have a parent directory".to_string()))?;
    let scratch = fs::canonicalize(&paths.scratch_parent)
        .map_err(|error| Error::io("canonicalizing scratch parent", &paths.scratch_parent, error))?;
    fs::create_dir_all(evidence_parent)
        .map_err(|error| Error::io("creating component evidence parent", evidence_parent, error))?;
    let evidence = fs::canonicalize(evidence_parent)
        .map_err(|error| Error::io("canonicalizing evidence parent", evidence_parent, error))?;
    if scratch != evidence {
        return Err(Error::Invalid(
            "component scratch and output must share one atomic publication parent".to_string(),
        ));
    }
    debug_assert!(Path::new(&scratch).is_absolute());
    debug_assert_eq!(scratch, evidence);
    Ok(())
}

pub fn declared_cohort_identity(
    request: &ComponentPipelineRequest,
) -> Result<crunch_wasm_component_core::Blake3Identity, Error> {
    cohort_identity(request.manifest.cohort.clone())
        .map_err(|blockers| crate::preflight::core_blockers("declared cohort", &blockers))
}
