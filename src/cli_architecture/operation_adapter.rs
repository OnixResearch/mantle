pub(crate) struct CliOperationAdapter<'a> {
    pub(crate) args: &'a crate::Args,
    pub(crate) ctx: &'a crate::RunContext,
}

impl CliOperationAdapter<'_> {
    fn execute(
        &mut self,
        effect: mantle_application_core::ApplicationEffect,
    ) -> Result<mantle_application_core::ApplicationObservation, mantle_application::ApplicationPortError> {
        let expected = super::inbound_adapter::application_command(self.args);
        if expected != effect.command {
            return Err(mantle_application::ApplicationPortError::for_effect(
                &effect,
                mantle_application_core::ApplicationErrorClass::Capability,
                "cli-operation-effect-mismatch",
                "planned CLI operation does not match the selected command",
                None,
            ));
        }
        match crate::cli_application::dispatch_command(self.args, self.ctx) {
            Ok(()) => Ok(mantle_application_core::successful_observation(&effect)),
            Err(error) => Err(port_error(&effect, error)),
        }
    }
}

pub(super) fn port_error(
    effect: &mantle_application_core::ApplicationEffect,
    error: crate::RunError,
) -> mantle_application::ApplicationPortError {
    match error {
        crate::RunError::Eval(message) => mantle_application::ApplicationPortError::for_effect(
            effect,
            mantle_application_core::ApplicationErrorClass::Evaluation,
            "evaluation",
            message,
            None,
        ),
        crate::RunError::Build(message) => mantle_application::ApplicationPortError::for_effect(
            effect,
            mantle_application_core::ApplicationErrorClass::Build,
            "build",
            message,
            None,
        ),
        crate::RunError::Internal(message) => mantle_application::ApplicationPortError::for_effect(
            effect,
            mantle_application_core::ApplicationErrorClass::Internal,
            "internal",
            message,
            None,
        ),
        crate::RunError::Reported(code) => mantle_application::ApplicationPortError::for_effect(
            effect,
            mantle_application_core::ApplicationErrorClass::Reported,
            "reported",
            "",
            Some(code),
        ),
    }
}

macro_rules! cli_operation_port {
    ($trait_name:ident, $method:ident) => {
        impl mantle_application::$trait_name for CliOperationAdapter<'_> {
            fn $method(
                &mut self,
                effect: mantle_application_core::ApplicationEffect,
            ) -> Result<mantle_application_core::ApplicationObservation, mantle_application::ApplicationPortError> {
                self.execute(effect)
            }
        }
    };
}

cli_operation_port!(BuildOperationPort, execute_build_operation);
cli_operation_port!(RustPlanOperationPort, execute_rust_plan_operation);
cli_operation_port!(RemoteOperationPort, execute_remote_operation);
cli_operation_port!(StoreOperationPort, execute_store_operation);
cli_operation_port!(SourceOperationPort, execute_source_operation);
cli_operation_port!(ReleaseOperationPort, execute_release_operation);
cli_operation_port!(ProjectOperationPort, execute_project_operation);
cli_operation_port!(BootstrapOperationPort, execute_bootstrap_operation);
cli_operation_port!(ArtifactOperationPort, execute_artifact_operation);
cli_operation_port!(EvaluationOperationPort, execute_evaluation_operation);
cli_operation_port!(PackageOperationPort, execute_package_operation);
cli_operation_port!(UtilityOperationPort, execute_utility_operation);
