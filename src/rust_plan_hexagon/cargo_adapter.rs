use std::path::Path;
use std::process::Command;

pub(crate) trait CargoProcessAdapter {
    fn run_cargo(
        &self,
        root: &Path,
        cargo: &Path,
        arguments: &[String],
    ) -> Result<super::ProcessObservation, super::RustPlanAdapterError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ProcessRustPlanTools;

impl CargoProcessAdapter for ProcessRustPlanTools {
    fn run_cargo(
        &self,
        root: &Path,
        cargo: &Path,
        arguments: &[String],
    ) -> Result<super::ProcessObservation, super::RustPlanAdapterError> {
        let output = Command::new(cargo).args(arguments).current_dir(root).output().map_err(|error| {
            super::RustPlanAdapterError::launch(mantle_rust_plan::RustPlanCapability::CargoOracle, cargo, &error)
        })?;
        debug_assert!(root.is_absolute() || !root.as_os_str().is_empty());
        debug_assert!(!arguments.is_empty());
        Ok(super::ProcessObservation::from_output(output))
    }
}

#[derive(Debug, Clone)]
pub struct SuppliedCargoOracleAdapter {
    pub facts: mantle_rust_plan_core::CargoOracleFacts,
}

impl mantle_rust_plan::CargoOraclePort for SuppliedCargoOracleAdapter {
    fn capture_oracle(
        &mut self,
        request: mantle_rust_plan::CargoOracleRequest,
    ) -> Result<mantle_rust_plan::CargoOracleObservation, mantle_rust_plan::RustPlanPortError> {
        if request.workspace_identity.is_empty() || request.profile.is_empty() {
            return Err(mantle_rust_plan::RustPlanPortError::new(
                mantle_rust_plan::RustPlanCapability::CargoOracle,
                "cargo-oracle-request-empty",
                false,
            ));
        }
        debug_assert!(!self.facts.metadata_identity.is_empty());
        debug_assert!(!self.facts.unit_graph_identity.is_empty());
        Ok(mantle_rust_plan::CargoOracleObservation {
            facts: self.facts.clone(),
        })
    }
}
