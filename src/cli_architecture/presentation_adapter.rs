pub(crate) fn finish(outcome: mantle_application_core::ApplicationOutcome) -> Result<(), crate::RunError> {
    match outcome.status {
        mantle_application_core::ApplicationObservationStatus::Succeeded => {
            if outcome.failure.is_some() {
                return Err(crate::RunError::Internal("successful application outcome carried a failure".to_string()));
            }
            Ok(())
        }
        mantle_application_core::ApplicationObservationStatus::Failed => {
            let failure = outcome.failure.ok_or_else(|| {
                crate::RunError::Internal("failed application outcome omitted its failure".to_string())
            })?;
            Err(run_error(failure))
        }
    }
}

pub(crate) fn application_failure(error: mantle_application::ApplicationFailure) -> crate::RunError {
    match error {
        mantle_application::ApplicationFailure::Core(error) => {
            crate::RunError::Internal(format!("application dispatch core rejected the command: {error:?}"))
        }
        mantle_application::ApplicationFailure::PortBindingMismatch(field) => {
            crate::RunError::Internal(format!("application port returned a mismatched {field}"))
        }
    }
}

pub(super) fn run_error(failure: mantle_application_core::CapabilityFailure) -> crate::RunError {
    match failure.class {
        mantle_application_core::ApplicationErrorClass::Evaluation => crate::RunError::Eval(failure.message),
        mantle_application_core::ApplicationErrorClass::Build => crate::RunError::Build(failure.message),
        mantle_application_core::ApplicationErrorClass::Internal
        | mantle_application_core::ApplicationErrorClass::Capability => crate::RunError::Internal(failure.message),
        mantle_application_core::ApplicationErrorClass::Reported => match failure.reported_exit_code {
            Some(code) => crate::RunError::Reported(code),
            None => crate::RunError::Internal("reported application failure omitted its exit code".to_string()),
        },
    }
}
