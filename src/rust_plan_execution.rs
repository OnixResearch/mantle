//! Rust operation capability adapters and execution receipt projection.

use mantle_application_contract::rust_operations::RustEvidence;
use mantle_application_contract::rust_operations::RustOperation;
use mantle_application_contract::rust_operations::classify_rust_operation;
use mantle_application_contract::rust_operations::plan_rust_operation;

use crate::RunError;
use crate::RustPlanExecutionMode;
use crate::RustPlanExecutionRequest;
use crate::rust_plan;

/// Plan before invoking the capability. Classification is deliberately separate
/// from rendering so rejected execution receipts can still be printed truthfully.
pub(crate) struct RustOperationGate {
    operation: RustOperation,
    plan: mantle_application_contract::EffectPlan,
}

impl RustOperationGate {
    pub(crate) fn plan(operation: RustOperation) -> Result<Self, RunError> {
        let plan = plan_rust_operation(operation)
            .map_err(|error| RunError::Internal(format!("Rust operation plan rejected: {error:?}")))?;
        Ok(Self { operation, plan })
    }

    pub(crate) fn classify(&self, evidence: &RustEvidence<'_>) -> Result<(), RunError> {
        let outcome = classify_rust_operation(&self.plan, self.operation, evidence);
        if outcome == mantle_application_contract::ApplicationOutcome::Completed {
            Ok(())
        } else {
            Err(RunError::Build(format!("Rust operation {:?} evidence rejected: {outcome:?}", self.operation)))
        }
    }

    pub(crate) fn classify_result<T>(
        &self,
        result: &Result<T, RunError>,
        observe: impl for<'a> FnOnce(&'a T) -> RustEvidence<'a>,
    ) -> Result<(), RunError> {
        match result {
            Ok(report) => self.classify(&observe(report)),
            Err(_) => self.classify(&RustEvidence::Failed),
        }
    }
}

/// A capability port is invoked only after its owning typed effect is planned.
pub(crate) fn through_rust_port<T>(
    operation: RustOperation,
    port: impl mantle_application_contract::rust_operations::RustOperationPort<Completion = Result<T, RunError>>,
    observe: impl for<'a> FnOnce(&'a T) -> RustEvidence<'a>,
) -> Result<T, RunError> {
    let gate = RustOperationGate::plan(operation)?;
    let result = port.execute();
    let classification = gate.classify_result(&result, observe);
    let report = result?;
    classification?;
    Ok(report)
}

pub(crate) fn current_workspace_directory() -> Result<std::path::PathBuf, RunError> {
    through_rust_port(RustOperation::ResolveWorkspace, crate::current_dir_or_error, |path| RustEvidence::Workspace {
        path: path.as_os_str().as_encoded_bytes(),
        absolute: path.is_absolute(),
    })
}

fn unit_evidence<'a>(
    receipt_hash: &'a str,
    status: &str,
    units: &[rust_plan::RustUnitExecutionReceipt],
) -> RustEvidence<'a> {
    RustEvidence::Units {
        receipt_hash,
        execution_succeeded: status == "success",
        units: units.len(),
        failed_units: units.iter().filter(|unit| unit.execution_status != "success" || unit.blocker.is_some()).count(),
        missing_artifacts: units
            .iter()
            .filter(|unit| {
                unit.output_artifact_digests.is_empty()
                    || unit
                        .output_artifact_digests
                        .iter()
                        .any(|artifact| artifact.path.is_empty() || artifact.blake3.is_empty())
            })
            .count(),
    }
}

pub(crate) fn reportable_execution(
    classification: Result<(), RunError>,
    status: &str,
) -> Result<Result<(), RunError>, RunError> {
    if status == "success" {
        classification?;
        Ok(Ok(()))
    } else {
        Ok(classification)
    }
}

pub(crate) fn execute_rust_plan_receipt(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let diagnostics = request.wait_diagnostics.clone();
    let result = match request.mode {
        RustPlanExecutionMode::FirstSupported => execute_first_supported_rust_plan(request),
        RustPlanExecutionMode::FirstDependencyChain => execute_first_dependency_chain_rust_plan(request),
        RustPlanExecutionMode::TargetTopology => execute_target_topology_rust_plan(request),
        RustPlanExecutionMode::HostArtifactTopology => execute_host_artifact_topology_rust_plan(request),
        RustPlanExecutionMode::Topology => execute_topology_rust_plan(request),
        RustPlanExecutionMode::DevDependencyTestTopology => execute_dev_dependency_test_topology_rust_plan(request),
        RustPlanExecutionMode::WorkspaceDependencyTopology => execute_workspace_dependency_topology_rust_plan(request),
        RustPlanExecutionMode::PatchSourceTopology => execute_patch_source_topology_rust_plan(request),
        RustPlanExecutionMode::PrintOnly => rust_plan::print_rust_plan_receipt(&request.receipt, request.json),
    };
    if let Some(diagnostics) = diagnostics {
        let outcome = match &result {
            Ok(()) => crunch_rustc_wrapper::waits::RunOutcome::Success,
            Err(RunError::Build(_)) => crunch_rustc_wrapper::waits::RunOutcome::Rejected,
            Err(_) => crunch_rustc_wrapper::waits::RunOutcome::Infrastructure,
        };
        let publication_error = match diagnostics.lock() {
            Ok(mut runtime) => runtime.finish(outcome).and_then(|report| report.publication_error),
            Err(_) => None,
        };
        if let Some(error) = publication_error {
            tracing::warn!(%error, "native Rust wait publication unavailable");
        }
    }
    result
}

fn rust_plan_execution_options(
    request: &RustPlanExecutionRequest<'_>,
    flag: &str,
) -> Result<rust_plan::RustUnitExecutionOptions, RunError> {
    let output_root = request
        .output_root
        .ok_or_else(|| RunError::Internal(format!("{flag} requires --execution-output-root")))?;
    Ok(rust_plan::RustUnitExecutionOptions {
        rustc: request.rustc.to_path_buf(),
        output_root: output_root.to_path_buf(),
        compiler_policy: request.compiler_policy.clone(),
        local_cache: request.local_cache.clone(),
        wait_diagnostics: request.wait_diagnostics.clone(),
    })
}

fn execute_first_supported_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-first-supported-unit")?;
    let gate = RustOperationGate::plan(RustOperation::ExecuteUnits)?;
    let result = rust_plan::execute_first_supported_rust_unit(&request.receipt.unit_derivation_graph, &options);
    let classification = gate.classify_result(&result, |unit| {
        unit_evidence(&unit.receipt_hash, &unit.execution_status, std::slice::from_ref(unit))
    });
    let unit_execution = result?;
    let classification = reportable_execution(classification, &unit_execution.execution_status)?;
    rust_plan::print_rust_plan_execution_receipt(
        &rust_plan::RustPlanExecutionReceipt {
            rust_plan: request.receipt,
            unit_execution,
        },
        request.json,
    )?;
    classification
}

fn execute_first_dependency_chain_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-first-dependency-chain")?;
    let gate = RustOperationGate::plan(RustOperation::ExecuteUnits)?;
    let result = rust_plan::execute_first_rust_unit_dependency_chain(&request.receipt.unit_derivation_graph, &options);
    let classification = gate.classify_result(&result, |execution| {
        unit_evidence(&execution.receipt_hash, &execution.execution_status, &execution.unit_executions)
    });
    let dependency_chain_execution = result?;
    let classification = reportable_execution(classification, &dependency_chain_execution.execution_status)?;
    rust_plan::print_rust_plan_dependency_chain_execution_receipt(
        &rust_plan::RustPlanDependencyChainExecutionReceipt {
            rust_plan: request.receipt,
            dependency_chain_execution,
        },
        request.json,
    )?;
    classification
}

fn execute_target_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-target-topology")?;
    let gate = RustOperationGate::plan(RustOperation::ExecuteUnits)?;
    let result = rust_plan::execute_rust_target_unit_topology(&request.receipt.unit_derivation_graph, &options);
    let classification = gate.classify_result(&result, |execution| {
        unit_evidence(&execution.receipt_hash, &execution.execution_status, &execution.unit_executions)
    });
    let target_topology_execution = result?;
    let classification = reportable_execution(classification, &target_topology_execution.execution_status)?;
    rust_plan::print_rust_plan_target_topology_execution_receipt(
        &rust_plan::RustPlanTargetTopologyExecutionReceipt {
            rust_plan: request.receipt,
            target_topology_execution,
        },
        request.json,
    )?;
    classification
}

fn execute_host_artifact_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-host-artifact-topology")?;
    let gate = RustOperationGate::plan(RustOperation::ExecuteUnits)?;
    let result = rust_plan::execute_rust_host_artifact_topology(
        &request.receipt.native_registry_source_planning,
        &request.receipt.native_host_unit_graph_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    );
    let classification = gate.classify_result(&result, |execution| {
        unit_evidence(&execution.receipt_hash, &execution.execution_status, &execution.unit_executions)
    });
    let execution = result?;
    let classification = reportable_execution(classification, &execution.execution_status)?;
    rust_plan::print_rust_plan_host_artifact_topology_execution_receipt(
        &rust_plan::RustPlanHostArtifactTopologyExecutionReceipt {
            rust_plan: request.receipt,
            host_artifact_topology_execution: execution,
        },
        request.json,
    )?;
    classification
}

fn execute_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-topology")?;
    let gate = RustOperationGate::plan(RustOperation::ExecuteUnits)?;
    let result = rust_plan::execute_rust_unit_topology(
        &request.receipt.native_registry_source_planning,
        &request.receipt.native_host_unit_graph_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    );
    let classification = gate.classify_result(&result, |execution| {
        unit_evidence(&execution.receipt_hash, &execution.execution_status, &execution.unit_executions)
    });
    let topology_execution = result?;
    let classification = reportable_execution(classification, &topology_execution.execution_status)?;
    rust_plan::print_rust_plan_topology_execution_receipt(
        &rust_plan::RustPlanTopologyExecutionReceipt {
            rust_plan: request.receipt,
            topology_execution,
        },
        request.json,
    )?;
    classification
}

fn execute_dev_dependency_test_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-dev-dependency-test-topology")?;
    let gate = RustOperationGate::plan(RustOperation::ExecuteUnits)?;
    let result = rust_plan::execute_native_rust_dev_dependency_test_topology(
        &request.receipt.native_package_target_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    );
    let classification = gate.classify_result(&result, |execution| {
        unit_evidence(&execution.receipt_hash, &execution.execution_status, &execution.unit_executions)
    });
    let execution = result?;
    let classification = reportable_execution(classification, &execution.execution_status)?;
    rust_plan::print_rust_plan_dev_dependency_test_topology_execution_receipt(
        &rust_plan::RustPlanDevDependencyTestTopologyExecutionReceipt {
            rust_plan: request.receipt,
            native_rust_dev_dependency_test_topology_execution: execution,
        },
        request.json,
    )?;
    classification
}

fn execute_workspace_dependency_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-workspace-dependency-topology")?;
    let gate = RustOperationGate::plan(RustOperation::ExecuteUnits)?;
    let result = rust_plan::execute_native_registry_workspace_dependency_topology(
        &request.receipt.native_package_target_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    );
    let classification = gate.classify_result(&result, |execution| {
        unit_evidence(&execution.receipt_hash, &execution.execution_status, &execution.unit_executions)
    });
    let execution = result?;
    let classification = reportable_execution(classification, &execution.execution_status)?;
    rust_plan::print_rust_plan_workspace_dependency_topology_execution_receipt(
        &rust_plan::RustPlanWorkspaceDependencyTopologyExecutionReceipt {
            rust_plan: request.receipt,
            native_registry_workspace_dependency_topology_execution: execution,
        },
        request.json,
    )?;
    classification
}

fn execute_patch_source_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-patch-source-topology")?;
    let gate = RustOperationGate::plan(RustOperation::ExecuteUnits)?;
    let result = rust_plan::execute_native_registry_patch_source_topology(
        &request.receipt.native_registry_source_planning,
        &request.receipt.native_package_target_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    );
    let classification = gate.classify_result(&result, |execution| {
        unit_evidence(&execution.receipt_hash, &execution.execution_status, &execution.unit_executions)
    });
    let execution = result?;
    let classification = reportable_execution(classification, &execution.execution_status)?;
    rust_plan::print_rust_plan_patch_source_topology_execution_receipt(
        &rust_plan::RustPlanPatchSourceTopologyExecutionReceipt {
            rust_plan: request.receipt,
            native_registry_patch_source_topology_execution: execution,
        },
        request.json,
    )?;
    classification
}
