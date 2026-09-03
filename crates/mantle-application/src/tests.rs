use alloc::string::ToString;
use alloc::vec::Vec;

const REQUEST_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const WRONG_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[derive(Clone, Copy)]
enum AdapterMode {
    Success,
    Failure,
    WrongObservation,
    WrongErrorFamily,
    WrongErrorEffect,
}

struct Adapter {
    mode: AdapterMode,
    observed_families: Vec<mantle_application_core::CommandFamily>,
}

impl Adapter {
    fn execute(
        &mut self,
        effect: mantle_application_core::ApplicationEffect,
    ) -> Result<mantle_application_core::ApplicationObservation, crate::ApplicationPortError> {
        self.observed_families.push(effect.command.family);
        match self.mode {
            AdapterMode::Success => Ok(mantle_application_core::successful_observation(&effect)),
            AdapterMode::Failure => Err(crate::ApplicationPortError::for_effect(
                &effect,
                mantle_application_core::ApplicationErrorClass::Build,
                "adapter-build-failed",
                "exact detail",
                None,
            )),
            AdapterMode::WrongObservation => {
                let mut observation = mantle_application_core::successful_observation(&effect);
                observation.effect_id_blake3 = WRONG_DIGEST.to_string();
                Ok(observation)
            }
            AdapterMode::WrongErrorFamily => {
                let mut error = crate::ApplicationPortError::for_effect(
                    &effect,
                    mantle_application_core::ApplicationErrorClass::Capability,
                    "wrong-family",
                    "wrong family",
                    None,
                );
                error.family = mantle_application_core::CommandFamily::Store;
                Err(error)
            }
            AdapterMode::WrongErrorEffect => {
                let mut error = crate::ApplicationPortError::for_effect(
                    &effect,
                    mantle_application_core::ApplicationErrorClass::Capability,
                    "wrong-effect",
                    "wrong effect",
                    None,
                );
                error.effect_id_blake3 = WRONG_DIGEST.to_string();
                Err(error)
            }
        }
    }
}

macro_rules! adapter_port {
    ($trait_name:ident, $method:ident) => {
        impl crate::$trait_name for Adapter {
            fn $method(
                &mut self,
                effect: mantle_application_core::ApplicationEffect,
            ) -> Result<mantle_application_core::ApplicationObservation, crate::ApplicationPortError> {
                self.execute(effect)
            }
        }
    };
}

adapter_port!(BuildOperationPort, execute_build_operation);
adapter_port!(RustPlanOperationPort, execute_rust_plan_operation);
adapter_port!(RemoteOperationPort, execute_remote_operation);
adapter_port!(StoreOperationPort, execute_store_operation);
adapter_port!(SourceOperationPort, execute_source_operation);
adapter_port!(ReleaseOperationPort, execute_release_operation);
adapter_port!(ProjectOperationPort, execute_project_operation);
adapter_port!(BootstrapOperationPort, execute_bootstrap_operation);
adapter_port!(ArtifactOperationPort, execute_artifact_operation);
adapter_port!(EvaluationOperationPort, execute_evaluation_operation);
adapter_port!(PackageOperationPort, execute_package_operation);
adapter_port!(UtilityOperationPort, execute_utility_operation);

fn command(family: mantle_application_core::CommandFamily) -> mantle_application_core::ApplicationCommand {
    mantle_application_core::ApplicationCommand {
        schema: mantle_application_core::APPLICATION_COMMAND_SCHEMA.to_string(),
        family,
        operation: "fixture.operation".to_string(),
        request_blake3: REQUEST_DIGEST.to_string(),
        mutation: mantle_application_core::MutationClass::ReadOnly,
    }
}

#[test]
fn every_family_dispatches_through_its_application_port() {
    let families = [
        mantle_application_core::CommandFamily::Build,
        mantle_application_core::CommandFamily::RustPlan,
        mantle_application_core::CommandFamily::Remote,
        mantle_application_core::CommandFamily::Store,
        mantle_application_core::CommandFamily::Source,
        mantle_application_core::CommandFamily::Release,
        mantle_application_core::CommandFamily::Project,
        mantle_application_core::CommandFamily::Bootstrap,
        mantle_application_core::CommandFamily::Artifact,
        mantle_application_core::CommandFamily::Evaluation,
        mantle_application_core::CommandFamily::Package,
        mantle_application_core::CommandFamily::Utility,
    ];
    let mut adapter = Adapter {
        mode: AdapterMode::Success,
        observed_families: Vec::with_capacity(families.len()),
    };
    for family in families {
        let outcome = crate::run(command(family), &mut adapter).unwrap();
        assert_eq!(outcome.family, family);
        assert_eq!(outcome.status, mantle_application_core::ApplicationObservationStatus::Succeeded);
    }
    assert_eq!(adapter.observed_families, families);
}

#[test]
fn capability_failure_retains_typed_error() {
    let mut adapter = Adapter {
        mode: AdapterMode::Failure,
        observed_families: Vec::new(),
    };
    let outcome = crate::run(command(mantle_application_core::CommandFamily::Build), &mut adapter).unwrap();
    let failure = outcome.failure.unwrap();
    assert_eq!(failure.class, mantle_application_core::ApplicationErrorClass::Build);
    assert_eq!(failure.message, "exact detail");
}

#[test]
fn wrong_observation_and_error_bindings_fail_closed() {
    let mut wrong_observation = Adapter {
        mode: AdapterMode::WrongObservation,
        observed_families: Vec::new(),
    };
    let mut wrong_family = Adapter {
        mode: AdapterMode::WrongErrorFamily,
        observed_families: Vec::new(),
    };
    let mut wrong_effect = Adapter {
        mode: AdapterMode::WrongErrorEffect,
        observed_families: Vec::new(),
    };
    assert!(matches!(
        crate::run(command(mantle_application_core::CommandFamily::Build), &mut wrong_observation),
        Err(crate::ApplicationFailure::Core(mantle_application_core::ApplicationCoreError::ObservationMismatch(
            "effect-identity"
        )))
    ));
    assert_eq!(
        crate::run(command(mantle_application_core::CommandFamily::Build), &mut wrong_family),
        Err(crate::ApplicationFailure::PortBindingMismatch("command-family"))
    );
    assert_eq!(
        crate::run(command(mantle_application_core::CommandFamily::Build), &mut wrong_effect),
        Err(crate::ApplicationFailure::PortBindingMismatch("effect-identity"))
    );
}
