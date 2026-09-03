use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationPortError {
    pub family: mantle_application_core::CommandFamily,
    pub effect_id_blake3: String,
    pub class: mantle_application_core::ApplicationErrorClass,
    pub code: String,
    pub message: String,
    pub reported_exit_code: Option<u8>,
}

impl ApplicationPortError {
    #[must_use]
    pub fn for_effect(
        effect: &mantle_application_core::ApplicationEffect,
        class: mantle_application_core::ApplicationErrorClass,
        code: impl Into<String>,
        message: impl Into<String>,
        reported_exit_code: Option<u8>,
    ) -> Self {
        let code = code.into();
        let message = message.into();
        debug_assert_eq!(effect.effect_id_blake3.len(), mantle_application_core::BLAKE3_HEX_CHARS);
        debug_assert!(!code.is_empty());
        debug_assert!(
            class == mantle_application_core::ApplicationErrorClass::Reported || reported_exit_code.is_none()
        );
        Self {
            family: effect.command.family,
            effect_id_blake3: effect.effect_id_blake3.clone(),
            class,
            code,
            message,
            reported_exit_code,
        }
    }
}

macro_rules! operation_port {
    ($name:ident, $method:ident) => {
        pub trait $name {
            fn $method(
                &mut self,
                effect: mantle_application_core::ApplicationEffect,
            ) -> Result<mantle_application_core::ApplicationObservation, ApplicationPortError>;
        }
    };
}

operation_port!(BuildOperationPort, execute_build_operation);
operation_port!(RustPlanOperationPort, execute_rust_plan_operation);
operation_port!(RemoteOperationPort, execute_remote_operation);
operation_port!(StoreOperationPort, execute_store_operation);
operation_port!(SourceOperationPort, execute_source_operation);
operation_port!(ReleaseOperationPort, execute_release_operation);
operation_port!(ProjectOperationPort, execute_project_operation);
operation_port!(BootstrapOperationPort, execute_bootstrap_operation);
operation_port!(ArtifactOperationPort, execute_artifact_operation);
operation_port!(EvaluationOperationPort, execute_evaluation_operation);
operation_port!(PackageOperationPort, execute_package_operation);
operation_port!(UtilityOperationPort, execute_utility_operation);

pub trait ApplicationPorts:
    BuildOperationPort
    + RustPlanOperationPort
    + RemoteOperationPort
    + StoreOperationPort
    + SourceOperationPort
    + ReleaseOperationPort
    + ProjectOperationPort
    + BootstrapOperationPort
    + ArtifactOperationPort
    + EvaluationOperationPort
    + PackageOperationPort
    + UtilityOperationPort
{
}

impl<T> ApplicationPorts for T where T: BuildOperationPort
        + RustPlanOperationPort
        + RemoteOperationPort
        + StoreOperationPort
        + SourceOperationPort
        + ReleaseOperationPort
        + ProjectOperationPort
        + BootstrapOperationPort
        + ArtifactOperationPort
        + EvaluationOperationPort
        + PackageOperationPort
        + UtilityOperationPort
{
}
