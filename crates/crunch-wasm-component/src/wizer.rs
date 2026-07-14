use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use crunch_wasm_component_core::BoundedComponentClaim;
use crunch_wasm_component_core::ComponentStageKind;
use crunch_wasm_component_core::ComponentStageStatus;
use crunch_wasm_component_core::TransformAdmission;
use crunch_wasm_component_core::TransformAdmissionRequest;
use crunch_wasm_component_core::WizerMode;
use crunch_wasm_component_core::admit_transform;

use crate::Error;
use crate::preflight::PreparedPipeline;
use crate::reporting::ExecutionState;
use crate::reporting::StageEvidence;
use crate::stages::ExecutedArtifact;
use crate::stages::StageInvocation;
use crate::stages::StageWorkspace;
use crate::stages::ValidationStage;
use crate::stages::invoke;
use crate::stages::measure_artifact;
use crate::stages::publish_validated_artifact;
use crate::stages::run_validation_stage;

const FIRST_WIZER_OUTPUT: &str = "wizer-first-core.wasm";
const REPEATED_WIZER_OUTPUT: &str = "wizer-repeated-core.wasm";
const COMPONENTIZED_OUTPUT: &str = "componentized.wasm";

pub(crate) struct WizerPipelineOutput {
    pub component: ExecutedArtifact,
    pub admission: Option<TransformAdmission>,
}

pub(crate) fn run_wizer_pipeline(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    compiled: ExecutedArtifact,
) -> Result<Option<WizerPipelineOutput>, Error> {
    if prepared.request.manifest.wizer.mode == WizerMode::Disabled {
        record_disabled_stage(prepared, state)?;
        return Ok(Some(WizerPipelineOutput {
            component: compiled,
            admission: None,
        }));
    }
    let Some(transformed) = run_repeated_transform(prepared, workspace, state, &compiled)? else {
        return Ok(None);
    };
    let Some(componentized) = run_componentization(prepared, workspace, state, &transformed.first)? else {
        return Ok(None);
    };
    if run_validation_stage(prepared, workspace, state, ValidationStage {
        stage_key: "componentized-validation",
        artifact_path: &componentized.path,
        artifact: &componentized.object,
        claim: BoundedComponentClaim::PortableBytesValidated,
    })?
    .is_none()
    {
        return Ok(None);
    }
    let admission = admit_wizer_output(prepared, &compiled, &transformed, &componentized);
    let Some(admission) = admission else {
        state.block("wizer-admission-failed", "wizer", "pure Wizer admission rejected the execution facts");
        return Ok(None);
    };
    debug_assert_eq!(admission.componentized_output, componentized.object);
    debug_assert!(
        admission.deterministic_outputs_match || prepared.request.manifest.wizer.mode != WizerMode::Deterministic
    );
    Ok(Some(WizerPipelineOutput {
        component: componentized,
        admission: Some(admission),
    }))
}

struct RepeatedTransform {
    first: ExecutedArtifact,
    repeated: ExecutedArtifact,
}

fn run_repeated_transform(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    compiled: &ExecutedArtifact,
) -> Result<Option<RepeatedTransform>, Error> {
    let first_path = workspace.root.join(FIRST_WIZER_OUTPUT);
    let repeated_path = workspace.root.join(REPEATED_WIZER_OUTPUT);
    let Some(first) = run_transform(prepared, workspace, state, TransformRun {
        stage_key: "wizer-first",
        input: &compiled.path,
        output: first_path,
    })?
    else {
        record_failed_wizer_stage(prepared, state)?;
        return Ok(None);
    };
    let Some(repeated) = run_transform(prepared, workspace, state, TransformRun {
        stage_key: "wizer-repeated",
        input: &compiled.path,
        output: repeated_path,
    })?
    else {
        record_failed_wizer_stage(prepared, state)?;
        return Ok(None);
    };
    let is_output_match = first.object.digest_blake3 == repeated.object.digest_blake3
        && first.object.size_bytes == repeated.object.size_bytes;
    if !is_output_match {
        record_failed_wizer_stage(prepared, state)?;
        state.block("wizer-output-drift", "wizer", "repeated clean Wizer outputs differ");
        return Ok(None);
    }
    publish_validated_artifact(&first.path, FIRST_WIZER_OUTPUT, &first.object, workspace)?;
    publish_validated_artifact(&repeated.path, REPEATED_WIZER_OUTPUT, &repeated.object, workspace)?;
    state.add_artifact("wizer-core-output", &first.object)?;
    state.add_artifact("wizer-repeated-core-output", &repeated.object)?;
    state.push_stage(
        "wizer",
        ComponentStageKind::Wizer,
        ComponentStageStatus::Succeeded,
        StageEvidence::new(
            Some(first.object.clone()),
            Some(prepared.toolchain.tool_digest("wizer")?),
            prepared.wizer_configuration_blake3.clone(),
            vec![BoundedComponentClaim::RepeatedTransformMatched],
        ),
    )?;
    debug_assert!(is_output_match);
    debug_assert_eq!(first.object.size_bytes, repeated.object.size_bytes);
    Ok(Some(RepeatedTransform { first, repeated }))
}

struct TransformRun<'a> {
    stage_key: &'a str,
    input: &'a Path,
    output: PathBuf,
}

fn run_transform(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    transform: TransformRun<'_>,
) -> Result<Option<ExecutedArtifact>, Error> {
    let entrypoint = prepared
        .request
        .manifest
        .wizer
        .initialization_entrypoint
        .as_ref()
        .ok_or_else(|| Error::Invalid("validated Wizer request omitted its entrypoint".to_string()))?;
    let args = vec![
        "--init-func".to_string(),
        entrypoint.clone(),
        "--inherit-env=false".to_string(),
        "--inherit-stdio=false".to_string(),
        "-o".to_string(),
        transform.output.display().to_string(),
        transform.input.display().to_string(),
    ];
    let run = invoke(prepared, workspace, StageInvocation {
        stage_key: transform.stage_key,
        tool_name: "wizer",
        args,
        environment_overrides: BTreeMap::new(),
        tool_dependencies: Vec::new(),
        output_path: Some(transform.output.clone()),
    })?;
    let receipt_blake3 = run.receipt.receipt_blake3.clone();
    state.add_receipt(run.receipt)?;
    if !run.success {
        state.block("wizer-execution-failed", transform.stage_key, stderr_summary(&run.stderr));
        return Ok(None);
    }
    let artifact_name = transform
        .output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Error::Invalid("Wizer output has no UTF-8 file name".to_string()))?;
    let object = measure_artifact(&transform.output, artifact_name, workspace)?;
    debug_assert!(object.size_bytes > 0);
    debug_assert!(transform.output.is_file());
    Ok(Some(ExecutedArtifact {
        path: transform.output,
        object,
        receipt_blake3,
    }))
}

fn run_componentization(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    transformed: &ExecutedArtifact,
) -> Result<Option<ExecutedArtifact>, Error> {
    let config = prepared
        .request
        .manifest
        .wizer
        .componentization
        .as_ref()
        .ok_or_else(|| Error::Invalid("validated Wizer request omitted componentization".to_string()))?;
    let output = workspace.root.join(COMPONENTIZED_OUTPUT);
    let mut args = vec![
        "component".to_string(),
        "new".to_string(),
        transformed.path.display().to_string(),
    ];
    args.extend(config.args.clone());
    args.extend(["-o".to_string(), output.display().to_string()]);
    let run = invoke(prepared, workspace, StageInvocation {
        stage_key: "componentization",
        tool_name: &config.tool,
        args,
        environment_overrides: BTreeMap::new(),
        tool_dependencies: Vec::new(),
        output_path: Some(output.clone()),
    })?;
    let receipt_blake3 = run.receipt.receipt_blake3.clone();
    state.add_receipt(run.receipt)?;
    if !run.success {
        state.push_stage(
            "componentization",
            ComponentStageKind::Componentization,
            ComponentStageStatus::Failed,
            StageEvidence::new(None, Some(prepared.toolchain.tool_digest(&config.tool)?), None, Vec::new()),
        )?;
        state.block("wizer-componentization-failed", "componentization", stderr_summary(&run.stderr));
        return Ok(None);
    }
    let object = measure_artifact(&output, COMPONENTIZED_OUTPUT, workspace)?;
    publish_validated_artifact(&output, COMPONENTIZED_OUTPUT, &object, workspace)?;
    state.add_artifact("wizer-componentized-output", &object)?;
    state.push_stage(
        "componentization",
        ComponentStageKind::Componentization,
        ComponentStageStatus::Succeeded,
        StageEvidence::new(
            Some(object.clone()),
            Some(prepared.toolchain.tool_digest(&config.tool)?),
            prepared.wizer_configuration_blake3.clone(),
            vec![BoundedComponentClaim::ExactInputIdentities],
        ),
    )?;
    debug_assert!(output.is_file());
    debug_assert!(object.size_bytes > 0);
    Ok(Some(ExecutedArtifact {
        path: output,
        object,
        receipt_blake3,
    }))
}

fn admit_wizer_output(
    prepared: &PreparedPipeline,
    compiled: &ExecutedArtifact,
    transformed: &RepeatedTransform,
    componentized: &ExecutedArtifact,
) -> Option<TransformAdmission> {
    let configuration_blake3 = prepared.wizer_configuration_blake3.clone()?;
    let entrypoint = prepared.request.manifest.wizer.initialization_entrypoint.clone()?;
    let result = admit_transform(TransformAdmissionRequest {
        mode: prepared.request.manifest.wizer.mode,
        input: compiled.object.clone(),
        first_output: transformed.first.object.clone(),
        repeated_output: Some(transformed.repeated.object.clone()),
        componentized_output: componentized.object.clone(),
        cohort_blake3: prepared.toolchain.manifest.cohort_identity_blake3.clone(),
        configuration_blake3,
        compilation_receipt_blake3: compiled.receipt_blake3.clone(),
        first_transform_receipt_blake3: transformed.first.receipt_blake3.clone(),
        repeated_transform_receipt_blake3: Some(transformed.repeated.receipt_blake3.clone()),
        componentization_receipt_blake3: componentized.receipt_blake3.clone(),
        initialization_entrypoint: entrypoint,
        virtual_imports: Vec::new(),
        ambient_observations: Vec::new(),
    });
    debug_assert_eq!(result.admission.is_some(), result.blockers.is_empty());
    debug_assert!(result.blockers.iter().all(|blocker| !blocker.code.is_empty()));
    result.admission
}

fn record_disabled_stage(prepared: &PreparedPipeline, state: &mut ExecutionState) -> Result<(), Error> {
    state.push_stage(
        "wizer",
        ComponentStageKind::Wizer,
        ComponentStageStatus::NotRun,
        StageEvidence::new(None, Some(prepared.toolchain.tool_digest("wizer")?), None, Vec::new()),
    )
}

fn record_failed_wizer_stage(prepared: &PreparedPipeline, state: &mut ExecutionState) -> Result<(), Error> {
    state.push_stage(
        "wizer",
        ComponentStageKind::Wizer,
        ComponentStageStatus::Failed,
        StageEvidence::new(
            None,
            Some(prepared.toolchain.tool_digest("wizer")?),
            prepared.wizer_configuration_blake3.clone(),
            Vec::new(),
        ),
    )
}

fn stderr_summary(stderr: &[u8]) -> String {
    let summary = String::from_utf8_lossy(stderr).trim().to_string();
    if summary.is_empty() {
        "tool exited unsuccessfully without stderr".to_string()
    } else {
        summary
    }
}
