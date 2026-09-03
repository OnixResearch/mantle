#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    Core(crunch_remote_core::RemoteCoreError),
    MissingOutcome,
}

impl From<crunch_remote_core::RemoteCoreError> for Failure {
    fn from(error: crunch_remote_core::RemoteCoreError) -> Self {
        Self::Core(error)
    }
}

pub fn run(
    command: crunch_remote_core::RemoteCommand,
    ports: &mut crate::RemotePortSet<'_>,
) -> Result<crunch_remote_core::RemoteOutcome, Failure> {
    let mut session = crunch_remote_core::start_remote_session(command)?;
    for _step in 0..crunch_remote_core::MAX_REMOTE_EFFECT_STEPS {
        if let Some(outcome) = session.outcome.clone() {
            return Ok(outcome);
        }
        let effect = session.pending_effect.clone().ok_or(Failure::MissingOutcome)?;
        let observation = match execute_effect(ports, effect.clone()) {
            Ok(observation) => observation,
            Err(error) => port_failure(&effect, error)?,
        };
        session = crunch_remote_core::apply_remote_observation(session, observation)?;
    }
    Err(crunch_remote_core::RemoteCoreError::TransitionLimitExceeded.into())
}

pub fn execute_effect(
    ports: &mut crate::RemotePortSet<'_>,
    effect: crunch_remote_core::RemoteEffect,
) -> Result<crunch_remote_core::RemoteObservation, crate::RemotePortError> {
    debug_assert_eq!(effect.capability, effect.kind.capability());
    match effect.kind {
        crunch_remote_core::RemoteEffectKind::SendHandshake | crunch_remote_core::RemoteEffectKind::TransferInputs => {
            ports.transport.execute_transport(effect)
        }
        crunch_remote_core::RemoteEffectKind::LoadAttempt
        | crunch_remote_core::RemoteEffectKind::PersistAttempt
        | crunch_remote_core::RemoteEffectKind::ReserveLease
        | crunch_remote_core::RemoteEffectKind::ReleaseLease => ports.attempts.execute_attempt(effect),
        crunch_remote_core::RemoteEffectKind::LaunchExecutor => ports.executor.execute_build(effect),
        crunch_remote_core::RemoteEffectKind::AdmitOutputs => ports.store.execute_admission(effect),
        crunch_remote_core::RemoteEffectKind::VerifyCredential => ports.credentials.execute_credential_check(effect),
        crunch_remote_core::RemoteEffectKind::ObserveClock => ports.clock.execute_clock_observation(effect),
        crunch_remote_core::RemoteEffectKind::GenerateAttemptId => {
            ports.identifiers.execute_identifier_generation(effect)
        }
        crunch_remote_core::RemoteEffectKind::PublishTelemetry => ports.telemetry.execute_telemetry_publication(effect),
    }
}

pub fn port_failure(
    effect: &crunch_remote_core::RemoteEffect,
    error: crate::RemotePortError,
) -> Result<crunch_remote_core::RemoteObservation, Failure> {
    if effect.capability != error.capability {
        return Err(crunch_remote_core::RemoteCoreError::invalid("remote-port-capability-mismatch").into());
    }
    if error.code.is_empty() {
        return Err(crunch_remote_core::RemoteCoreError::invalid("remote-port-error-code-empty").into());
    }
    let failure_class = if error.retryable {
        crunch_remote_core::RemoteFailureClass::Transient
    } else {
        crunch_remote_core::RemoteFailureClass::Permanent
    };
    debug_assert_eq!(effect.capability, error.capability);
    debug_assert!(!error.code.is_empty());
    Ok(crunch_remote_core::failed_observation(effect, failure_class, error.code))
}

#[cfg(test)]
mod tests;
