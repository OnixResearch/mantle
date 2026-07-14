use std::path::Path;

use crunch_wasm_component_core::AotAdmission;
use crunch_wasm_component_core::AotAdmissionRequest;
use crunch_wasm_component_core::AotMode;
use crunch_wasm_component_core::AotReceipt;
use crunch_wasm_component_core::BoundedComponentClaim;
use crunch_wasm_component_core::ComponentStageKind;
use crunch_wasm_component_core::ComponentStageStatus;
use crunch_wasm_component_core::PortableAdmission;
use crunch_wasm_component_core::admit_aot;

use crate::Error;
use crate::preflight::PreparedPipeline;
use crate::reporting::ExecutionState;
use crate::reporting::StageEvidence;
use crate::stages::StageInvocation;
use crate::stages::StageWorkspace;
use crate::stages::invoke;
use crate::stages::measure_artifact;
use crate::stages::publish_validated_artifact;

const AOT_OUTPUT_NAME: &str = "component.cwasm";

pub(crate) fn run_aot_stage(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    portable_path: &Path,
    portable: &PortableAdmission,
) -> Result<Option<AotAdmission>, Error> {
    debug_assert!(portable.admitted);
    debug_assert!(!portable.artifact.logical_path.is_empty());
    if prepared.request.manifest.aot.mode == AotMode::Disabled {
        state.push_stage(
            "aot",
            ComponentStageKind::Aot,
            ComponentStageStatus::NotRun,
            StageEvidence::new(None, Some(prepared.toolchain.tool_digest("wasmtime")?), None, Vec::new()),
        )?;
        return Ok(None);
    }
    let output = workspace.root.join(AOT_OUTPUT_NAME);
    let mut args = vec!["compile".to_string()];
    args.extend(prepared.request.aot_configuration_args.clone());
    args.extend([
        "--target".to_string(),
        prepared.request.aot_target.clone(),
        "-o".to_string(),
        output.display().to_string(),
        portable_path.display().to_string(),
    ]);
    let run = invoke(prepared, workspace, StageInvocation {
        stage_key: "aot",
        tool_name: "wasmtime",
        args,
        output_path: Some(output.clone()),
    })?;
    state.add_receipt(run.receipt)?;
    if !run.success {
        state.push_stage(
            "aot",
            ComponentStageKind::Aot,
            ComponentStageStatus::Failed,
            StageEvidence::new(
                None,
                Some(prepared.toolchain.tool_digest("wasmtime")?),
                prepared.aot_configuration_blake3.clone(),
                Vec::new(),
            ),
        )?;
        state.block(
            "wasmtime-precompile-failed",
            "aot",
            "pinned Wasmtime did not produce the declared target-specific precompile output",
        );
        return Ok(None);
    }
    admit_completed_aot(prepared, workspace, state, &output, portable)
}

fn admit_completed_aot(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    output: &Path,
    portable: &PortableAdmission,
) -> Result<Option<AotAdmission>, Error> {
    let output_object = measure_artifact(output, AOT_OUTPUT_NAME, workspace)?;
    let configuration = prepared
        .aot_configuration_blake3
        .clone()
        .ok_or_else(|| Error::Invalid("enabled AOT stage omitted its validated configuration identity".to_string()))?;
    let admission_result = admit_aot(AotAdmissionRequest {
        mode: prepared.request.manifest.aot.mode,
        portable_admission: portable.clone(),
        receipt: AotReceipt {
            source_component_blake3: portable.artifact.digest_blake3.clone(),
            output: output_object.clone(),
            target: prepared.request.aot_target.clone(),
            cpu_features: prepared.request.aot_cpu_features.clone(),
            wasmtime_configuration_blake3: configuration.clone(),
            cohort_blake3: prepared.toolchain.manifest.cohort_identity_blake3.clone(),
            wit_profile_blake3: prepared.wit_profile_blake3.clone(),
            build_inputs_blake3: prepared.request_blake3.clone(),
        },
        expected_target: prepared.request.aot_target.clone(),
        expected_cpu_features: prepared.request.aot_cpu_features.clone(),
        expected_wasmtime_configuration_blake3: configuration.clone(),
        expected_cohort_blake3: prepared.toolchain.manifest.cohort_identity_blake3.clone(),
        expected_wit_profile_blake3: prepared.wit_profile_blake3.clone(),
        expected_build_inputs_blake3: prepared.request_blake3.clone(),
    });
    let Some(admission) = admission_result.admission else {
        for blocker in admission_result.blockers {
            state.block(&blocker.code, "aot", blocker.message);
        }
        state.push_stage(
            "aot",
            ComponentStageKind::Aot,
            ComponentStageStatus::Denied,
            StageEvidence::new(
                Some(output_object),
                Some(prepared.toolchain.tool_digest("wasmtime")?),
                Some(configuration),
                Vec::new(),
            ),
        )?;
        return Ok(None);
    };
    publish_validated_artifact(output, AOT_OUTPUT_NAME, &output_object, workspace)?;
    state.add_artifact("target-specific-precompiled-component", &output_object)?;
    state.push_stage(
        "aot",
        ComponentStageKind::Aot,
        ComponentStageStatus::Succeeded,
        StageEvidence::new(
            Some(output_object),
            Some(prepared.toolchain.tool_digest("wasmtime")?),
            Some(configuration),
            vec![BoundedComponentClaim::TargetSpecificNativeReceiptBound],
        ),
    )?;
    debug_assert_eq!(admission.source_component_blake3, portable.artifact.digest_blake3);
    debug_assert_eq!(admission.build_inputs_blake3, prepared.request_blake3);
    Ok(Some(admission))
}
