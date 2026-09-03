use std::path::Path;
use std::process::Command;

pub(crate) trait CompilerProcessAdapter {
    fn run_tool_version(
        &self,
        tool: &Path,
        arguments: &[&str],
    ) -> Result<super::ProcessObservation, super::RustPlanAdapterError>;
}

impl CompilerProcessAdapter for super::cargo_adapter::ProcessRustPlanTools {
    fn run_tool_version(
        &self,
        tool: &Path,
        arguments: &[&str],
    ) -> Result<super::ProcessObservation, super::RustPlanAdapterError> {
        let output = Command::new(tool).args(arguments).output().map_err(|error| {
            super::RustPlanAdapterError::launch(mantle_rust_plan::RustPlanCapability::CompilerInspection, tool, &error)
        })?;
        debug_assert!(!tool.as_os_str().is_empty());
        debug_assert!(!arguments.is_empty());
        Ok(super::ProcessObservation::from_output(output))
    }
}

#[derive(Debug, Clone)]
pub struct SuppliedCompilerInspectionAdapter {
    pub fact: mantle_rust_plan_core::ToolchainFact,
}

impl mantle_rust_plan::CompilerInspectionPort for SuppliedCompilerInspectionAdapter {
    fn inspect_compiler(
        &mut self,
        request: mantle_rust_plan::CompilerInspectionRequest,
    ) -> Result<mantle_rust_plan::CompilerInspectionObservation, mantle_rust_plan::RustPlanPortError> {
        if request.compiler_hint.is_empty() || request.target_triple != self.fact.target_triple {
            return Err(mantle_rust_plan::RustPlanPortError::new(
                mantle_rust_plan::RustPlanCapability::CompilerInspection,
                "compiler-inspection-request-invalid",
                false,
            ));
        }
        debug_assert!(!self.fact.compiler_identity.is_empty());
        debug_assert!(!self.fact.compiler_version_identity.is_empty());
        Ok(mantle_rust_plan::CompilerInspectionObservation {
            fact: self.fact.clone(),
        })
    }
}
