#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationFailure {
    Core(mantle_application_core::ApplicationCoreError),
    PortBindingMismatch(&'static str),
}

impl From<mantle_application_core::ApplicationCoreError> for ApplicationFailure {
    fn from(error: mantle_application_core::ApplicationCoreError) -> Self {
        Self::Core(error)
    }
}

pub fn run(
    command: mantle_application_core::ApplicationCommand,
    ports: &mut impl crate::ApplicationPorts,
) -> Result<mantle_application_core::ApplicationOutcome, ApplicationFailure> {
    let effect = mantle_application_core::plan_dispatch(command)?;
    let observation = match execute_effect(ports, effect.clone()) {
        Ok(observation) => observation,
        Err(error) => port_failure(&effect, error)?,
    };
    let outcome = mantle_application_core::classify_observation(&effect, observation)?;
    debug_assert_eq!(effect.command.family, outcome.family);
    debug_assert_eq!(effect.effect_id_blake3, outcome.effect_id_blake3);
    Ok(outcome)
}

fn execute_effect(
    ports: &mut impl crate::ApplicationPorts,
    effect: mantle_application_core::ApplicationEffect,
) -> Result<mantle_application_core::ApplicationObservation, crate::ApplicationPortError> {
    match effect.command.family {
        mantle_application_core::CommandFamily::Build => ports.execute_build_operation(effect),
        mantle_application_core::CommandFamily::RustPlan => ports.execute_rust_plan_operation(effect),
        mantle_application_core::CommandFamily::Remote => ports.execute_remote_operation(effect),
        mantle_application_core::CommandFamily::Store => ports.execute_store_operation(effect),
        mantle_application_core::CommandFamily::Source => ports.execute_source_operation(effect),
        mantle_application_core::CommandFamily::Release => ports.execute_release_operation(effect),
        mantle_application_core::CommandFamily::Project => ports.execute_project_operation(effect),
        mantle_application_core::CommandFamily::Bootstrap => ports.execute_bootstrap_operation(effect),
        mantle_application_core::CommandFamily::Artifact => ports.execute_artifact_operation(effect),
        mantle_application_core::CommandFamily::Evaluation => ports.execute_evaluation_operation(effect),
        mantle_application_core::CommandFamily::Package => ports.execute_package_operation(effect),
        mantle_application_core::CommandFamily::Utility => ports.execute_utility_operation(effect),
    }
}

fn port_failure(
    effect: &mantle_application_core::ApplicationEffect,
    error: crate::ApplicationPortError,
) -> Result<mantle_application_core::ApplicationObservation, ApplicationFailure> {
    if error.family != effect.command.family {
        return Err(ApplicationFailure::PortBindingMismatch("command-family"));
    }
    if error.effect_id_blake3 != effect.effect_id_blake3 {
        return Err(ApplicationFailure::PortBindingMismatch("effect-identity"));
    }
    let failure = mantle_application_core::CapabilityFailure {
        family: error.family,
        effect_id_blake3: error.effect_id_blake3,
        class: error.class,
        code: error.code,
        message: error.message,
        reported_exit_code: error.reported_exit_code,
    };
    debug_assert_eq!(failure.family, effect.command.family);
    debug_assert_eq!(failure.effect_id_blake3, effect.effect_id_blake3);
    mantle_application_core::failed_observation(effect, failure).map_err(Into::into)
}
