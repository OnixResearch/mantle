//! Rust-plan execution receipts: run one named plan mode and print its receipt.
//!
//! Each helper pairs the plan receipt with the execution result the owning
//! `rust_plan` entry point returns, then prints through that module. The mode
//! routing here is the receipt projection boundary; receipt contents stay with
//! `crate::rust_plan`.

use crate::RunError;
use crate::RustPlanExecutionMode;
use crate::RustPlanExecutionRequest;
use crate::rust_plan;

pub(crate) fn execute_rust_plan_receipt(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    match request.mode {
        RustPlanExecutionMode::FirstSupported => execute_first_supported_rust_plan(request),
        RustPlanExecutionMode::FirstDependencyChain => execute_first_dependency_chain_rust_plan(request),
        RustPlanExecutionMode::TargetTopology => execute_target_topology_rust_plan(request),
        RustPlanExecutionMode::HostArtifactTopology => execute_host_artifact_topology_rust_plan(request),
        RustPlanExecutionMode::Topology => execute_topology_rust_plan(request),
        RustPlanExecutionMode::DevDependencyTestTopology => execute_dev_dependency_test_topology_rust_plan(request),
        RustPlanExecutionMode::WorkspaceDependencyTopology => execute_workspace_dependency_topology_rust_plan(request),
        RustPlanExecutionMode::PatchSourceTopology => execute_patch_source_topology_rust_plan(request),
        RustPlanExecutionMode::PrintOnly => rust_plan::print_rust_plan_receipt(&request.receipt, request.json),
    }
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
    })
}

fn execute_first_supported_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-first-supported-unit")?;
    let unit_execution =
        rust_plan::execute_first_supported_rust_unit(&request.receipt.unit_derivation_graph, &options)?;
    rust_plan::print_rust_plan_execution_receipt(
        &rust_plan::RustPlanExecutionReceipt {
            rust_plan: request.receipt,
            unit_execution,
        },
        request.json,
    )
}

fn execute_first_dependency_chain_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-first-dependency-chain")?;
    let dependency_chain_execution =
        rust_plan::execute_first_rust_unit_dependency_chain(&request.receipt.unit_derivation_graph, &options)?;
    rust_plan::print_rust_plan_dependency_chain_execution_receipt(
        &rust_plan::RustPlanDependencyChainExecutionReceipt {
            rust_plan: request.receipt,
            dependency_chain_execution,
        },
        request.json,
    )
}

fn execute_target_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-target-topology")?;
    let target_topology_execution =
        rust_plan::execute_rust_target_unit_topology(&request.receipt.unit_derivation_graph, &options)?;
    rust_plan::print_rust_plan_target_topology_execution_receipt(
        &rust_plan::RustPlanTargetTopologyExecutionReceipt {
            rust_plan: request.receipt,
            target_topology_execution,
        },
        request.json,
    )
}

fn execute_host_artifact_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-host-artifact-topology")?;
    let execution = rust_plan::execute_rust_host_artifact_topology(
        &request.receipt.native_registry_source_planning,
        &request.receipt.native_host_unit_graph_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    )?;
    rust_plan::print_rust_plan_host_artifact_topology_execution_receipt(
        &rust_plan::RustPlanHostArtifactTopologyExecutionReceipt {
            rust_plan: request.receipt,
            host_artifact_topology_execution: execution,
        },
        request.json,
    )
}

fn execute_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-topology")?;
    let topology_execution = rust_plan::execute_rust_unit_topology(
        &request.receipt.native_registry_source_planning,
        &request.receipt.native_host_unit_graph_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    )?;
    rust_plan::print_rust_plan_topology_execution_receipt(
        &rust_plan::RustPlanTopologyExecutionReceipt {
            rust_plan: request.receipt,
            topology_execution,
        },
        request.json,
    )
}

fn execute_dev_dependency_test_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-dev-dependency-test-topology")?;
    let execution = rust_plan::execute_native_rust_dev_dependency_test_topology(
        &request.receipt.native_package_target_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    )?;
    rust_plan::print_rust_plan_dev_dependency_test_topology_execution_receipt(
        &rust_plan::RustPlanDevDependencyTestTopologyExecutionReceipt {
            rust_plan: request.receipt,
            native_rust_dev_dependency_test_topology_execution: execution,
        },
        request.json,
    )
}

fn execute_workspace_dependency_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-workspace-dependency-topology")?;
    let execution = rust_plan::execute_native_registry_workspace_dependency_topology(
        &request.receipt.native_package_target_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    )?;
    rust_plan::print_rust_plan_workspace_dependency_topology_execution_receipt(
        &rust_plan::RustPlanWorkspaceDependencyTopologyExecutionReceipt {
            rust_plan: request.receipt,
            native_registry_workspace_dependency_topology_execution: execution,
        },
        request.json,
    )
}

fn execute_patch_source_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-patch-source-topology")?;
    let execution = rust_plan::execute_native_registry_patch_source_topology(
        &request.receipt.native_registry_source_planning,
        &request.receipt.native_package_target_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    )?;
    rust_plan::print_rust_plan_patch_source_topology_execution_receipt(
        &rust_plan::RustPlanPatchSourceTopologyExecutionReceipt {
            rust_plan: request.receipt,
            native_registry_patch_source_topology_execution: execution,
        },
        request.json,
    )
}
