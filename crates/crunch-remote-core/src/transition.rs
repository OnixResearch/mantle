use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::MAX_REMOTE_EFFECT_STEPS;
use crate::REMOTE_CORE_NON_CLAIM;
use crate::REMOTE_EFFECT_SCHEMA;
use crate::REMOTE_OBSERVATION_SCHEMA;
use crate::REMOTE_RECEIPT_PREIMAGE_SCHEMA;
use crate::RemoteCommand;
use crate::RemoteCoreError;
use crate::RemoteEffect;
use crate::RemoteEffectKind;
use crate::RemoteEffectLimits;
use crate::RemoteEvent;
use crate::RemoteFailureClass;
use crate::RemoteObservation;
use crate::RemoteObservationKind;
use crate::RemoteObservationStatus;
use crate::RemoteOutcome;
use crate::RemoteOutcomeStatus;
use crate::RemotePhase;
use crate::RemoteReceiptPreimage;
use crate::RemoteSession;
use crate::admit_remote_command;
use crate::remote_command_identity;
use crate::remote_effect_identity;
use crate::remote_receipt_preimage_identity;

pub fn start_remote_session(command: RemoteCommand) -> Result<RemoteSession, RemoteCoreError> {
    let command = admit_remote_command(command)?;
    let command_blake3 = remote_command_identity(command.clone())?;
    let mut session = RemoteSession {
        command,
        command_blake3,
        phase: RemotePhase::Authenticating,
        sequence: 0,
        attempt_ordinal: 0,
        attempt_id: None,
        pending_effect: None,
        retry_after_release: false,
        events: Vec::new(),
        outcome: None,
    };
    plan_effect(
        &mut session,
        RemoteEffectKind::VerifyCredential,
        RemotePhase::Authenticating,
        "remote-request-admitted",
    )?;
    debug_assert!(session.pending_effect.is_some());
    debug_assert!(session.outcome.is_none());
    Ok(session)
}

pub fn apply_remote_observation(
    mut session: RemoteSession,
    observation: RemoteObservation,
) -> Result<RemoteSession, RemoteCoreError> {
    if session.outcome.is_some() {
        return Err(RemoteCoreError::TerminalSession);
    }
    let effect = session.pending_effect.take().ok_or(RemoteCoreError::TerminalSession)?;
    validate_observation(&session, &effect, &observation)?;
    if observation.status == RemoteObservationStatus::Failed {
        return handle_failed_observation(session, effect, observation);
    }
    apply_success_observation(&mut session, &effect, &observation)?;
    debug_assert!(session.pending_effect.is_some() || session.outcome.is_some());
    debug_assert!(session.sequence <= MAX_REMOTE_EFFECT_STEPS);
    Ok(session)
}

fn validate_observation(
    session: &RemoteSession,
    effect: &RemoteEffect,
    observation: &RemoteObservation,
) -> Result<(), RemoteCoreError> {
    if observation.schema != REMOTE_OBSERVATION_SCHEMA {
        return Err(RemoteCoreError::invalid("remote-observation-schema-unsupported"));
    }
    if observation.effect_id_blake3 != effect.effect_id_blake3 {
        return Err(RemoteCoreError::WrongEffectIdentity);
    }
    if observation.kind != effect.kind.expected_observation() {
        return Err(RemoteCoreError::WrongObservationKind);
    }
    if observation.fence_generation != session.command.fence_generation {
        return Err(RemoteCoreError::StaleFence);
    }
    validate_observation_attempt(session, effect, observation)?;
    validate_observation_status(observation)?;
    validate_observed_limits(session, effect, observation)?;
    debug_assert_eq!(observation.kind, effect.kind.expected_observation());
    debug_assert_eq!(observation.fence_generation, session.command.fence_generation);
    Ok(())
}

fn validate_observation_attempt(
    session: &RemoteSession,
    effect: &RemoteEffect,
    observation: &RemoteObservation,
) -> Result<(), RemoteCoreError> {
    if effect.kind == RemoteEffectKind::GenerateAttemptId {
        if observation.attempt_id.as_deref().is_none_or(str::is_empty) {
            return Err(RemoteCoreError::invalid("remote-attempt-id-observation-empty"));
        }
        return Ok(());
    }
    if observation.attempt_id != session.attempt_id {
        return Err(RemoteCoreError::StaleAttempt);
    }
    Ok(())
}

fn validate_observation_status(observation: &RemoteObservation) -> Result<(), RemoteCoreError> {
    match observation.status {
        RemoteObservationStatus::Succeeded => {
            if observation.failure_class.is_some() || observation.reason_code.is_some() {
                return Err(RemoteCoreError::invalid("remote-success-observation-carries-failure"));
            }
        }
        RemoteObservationStatus::Failed => {
            if observation.failure_class.is_none() || observation.reason_code.as_deref().is_none_or(str::is_empty) {
                return Err(RemoteCoreError::invalid("remote-failure-observation-incomplete"));
            }
        }
    }
    Ok(())
}

fn validate_observed_limits(
    session: &RemoteSession,
    effect: &RemoteEffect,
    observation: &RemoteObservation,
) -> Result<(), RemoteCoreError> {
    if observation.observed_bytes > effect.limits.bytes_max {
        return Err(RemoteCoreError::invalid("remote-observed-byte-limit-exceeded"));
    }
    if observation.observed_outputs > effect.limits.outputs_max {
        return Err(RemoteCoreError::invalid("remote-observed-output-limit-exceeded"));
    }
    let expected_outputs = u32::try_from(session.command.expected_outputs.len())
        .map_err(|_| RemoteCoreError::invalid("remote-expected-output-count-overflow"))?;
    if observation.kind == RemoteObservationKind::OutputsAdmitted && observation.observed_outputs != expected_outputs {
        return Err(RemoteCoreError::invalid("remote-output-admission-count-mismatch"));
    }
    debug_assert!(observation.observed_bytes <= effect.limits.bytes_max);
    debug_assert!(observation.observed_outputs <= effect.limits.outputs_max);
    Ok(())
}

fn apply_success_observation(
    session: &mut RemoteSession,
    effect: &RemoteEffect,
    observation: &RemoteObservation,
) -> Result<(), RemoteCoreError> {
    match effect.kind {
        RemoteEffectKind::VerifyCredential => {
            plan_next(session, RemoteEffectKind::SendHandshake, "credential-verified")
        }
        RemoteEffectKind::SendHandshake => plan_next(session, RemoteEffectKind::ObserveClock, "handshake-sent"),
        RemoteEffectKind::ObserveClock => plan_next(session, RemoteEffectKind::GenerateAttemptId, "clock-observed"),
        RemoteEffectKind::GenerateAttemptId => accept_attempt_id(session, observation),
        RemoteEffectKind::LoadAttempt => plan_next(session, RemoteEffectKind::PersistAttempt, "attempt-loaded"),
        RemoteEffectKind::PersistAttempt => plan_next(session, RemoteEffectKind::ReserveLease, "attempt-persisted"),
        RemoteEffectKind::ReserveLease => plan_next(session, RemoteEffectKind::TransferInputs, "lease-reserved"),
        RemoteEffectKind::TransferInputs => plan_next(session, RemoteEffectKind::LaunchExecutor, "inputs-transferred"),
        RemoteEffectKind::LaunchExecutor => plan_next(session, RemoteEffectKind::AdmitOutputs, "executor-completed"),
        RemoteEffectKind::AdmitOutputs => accept_output_admission(session, observation),
        RemoteEffectKind::ReleaseLease => accept_lease_release(session),
        RemoteEffectKind::PublishTelemetry => complete_session(session),
    }
}

fn accept_attempt_id(session: &mut RemoteSession, observation: &RemoteObservation) -> Result<(), RemoteCoreError> {
    let attempt_id = observation
        .attempt_id
        .clone()
        .ok_or_else(|| RemoteCoreError::invalid("remote-attempt-id-observation-empty"))?;
    let next_ordinal = session
        .attempt_ordinal
        .checked_add(1)
        .ok_or_else(|| RemoteCoreError::invalid("remote-attempt-ordinal-overflow"))?;
    if next_ordinal > session.command.policy.attempts_max {
        return fail_session(session, "remote-attempt-limit-exhausted");
    }
    session.attempt_ordinal = next_ordinal;
    session.attempt_id = Some(attempt_id);
    plan_next(session, RemoteEffectKind::LoadAttempt, "attempt-id-generated")
}

fn accept_output_admission(
    session: &mut RemoteSession,
    observation: &RemoteObservation,
) -> Result<(), RemoteCoreError> {
    if session.command.policy.require_output_trust && !observation.output_trusted {
        return fail_session(session, "untrusted-output");
    }
    plan_next(session, RemoteEffectKind::ReleaseLease, "outputs-admitted")
}

fn accept_lease_release(session: &mut RemoteSession) -> Result<(), RemoteCoreError> {
    if session.retry_after_release {
        session.retry_after_release = false;
        session.attempt_id = None;
        return plan_next(session, RemoteEffectKind::GenerateAttemptId, "retry-lease-released");
    }
    plan_next(session, RemoteEffectKind::PublishTelemetry, "lease-released")
}

fn complete_session(session: &mut RemoteSession) -> Result<(), RemoteCoreError> {
    record_terminal_event(session, RemotePhase::Succeeded, "remote-session-succeeded")?;
    set_outcome(session, RemoteOutcomeStatus::Succeeded, "remote-session-succeeded")
}

fn handle_failed_observation(
    mut session: RemoteSession,
    effect: RemoteEffect,
    observation: RemoteObservation,
) -> Result<RemoteSession, RemoteCoreError> {
    let failure_class = observation
        .failure_class
        .ok_or_else(|| RemoteCoreError::invalid("remote-failure-observation-incomplete"))?;
    let reason = observation.reason_code.unwrap_or_else(|| "remote-effect-failed".to_string());
    if should_retry(&session, effect.kind, failure_class) {
        session.retry_after_release = true;
        plan_effect(
            &mut session,
            RemoteEffectKind::ReleaseLease,
            RemotePhase::ReleasingLease,
            "remote-transient-retry-planned",
        )?;
        return Ok(session);
    }
    let terminal_reason = terminal_failure_reason(failure_class, &reason);
    fail_session(&mut session, &terminal_reason)?;
    Ok(session)
}

fn should_retry(session: &RemoteSession, kind: RemoteEffectKind, failure_class: RemoteFailureClass) -> bool {
    let is_retryable_effect = matches!(
        kind,
        RemoteEffectKind::TransferInputs | RemoteEffectKind::LaunchExecutor | RemoteEffectKind::AdmitOutputs
    );
    session.command.policy.allow_transient_retry
        && failure_class == RemoteFailureClass::Transient
        && is_retryable_effect
        && session.attempt_id.is_some()
        && session.attempt_ordinal < session.command.policy.attempts_max
}

fn terminal_failure_reason(failure_class: RemoteFailureClass, reason: &str) -> String {
    match failure_class {
        RemoteFailureClass::StaleFence => "stale-fence".to_string(),
        RemoteFailureClass::LimitExceeded => "remote-limit-exceeded".to_string(),
        RemoteFailureClass::UntrustedOutput => "untrusted-output".to_string(),
        RemoteFailureClass::Protocol => "remote-protocol-failed".to_string(),
        RemoteFailureClass::Transient | RemoteFailureClass::Permanent => reason.to_string(),
    }
}

fn plan_next(session: &mut RemoteSession, kind: RemoteEffectKind, reason: &str) -> Result<(), RemoteCoreError> {
    plan_effect(session, kind, phase_for_effect(kind), reason)
}

fn phase_for_effect(kind: RemoteEffectKind) -> RemotePhase {
    match kind {
        RemoteEffectKind::VerifyCredential => RemotePhase::Authenticating,
        RemoteEffectKind::SendHandshake => RemotePhase::SendingHandshake,
        RemoteEffectKind::ObserveClock => RemotePhase::ObservingClock,
        RemoteEffectKind::GenerateAttemptId => RemotePhase::GeneratingAttempt,
        RemoteEffectKind::LoadAttempt => RemotePhase::LoadingAttempt,
        RemoteEffectKind::PersistAttempt => RemotePhase::PersistingAttempt,
        RemoteEffectKind::ReserveLease => RemotePhase::ReservingLease,
        RemoteEffectKind::TransferInputs => RemotePhase::TransferringInputs,
        RemoteEffectKind::LaunchExecutor => RemotePhase::Executing,
        RemoteEffectKind::AdmitOutputs => RemotePhase::AdmittingOutputs,
        RemoteEffectKind::ReleaseLease => RemotePhase::ReleasingLease,
        RemoteEffectKind::PublishTelemetry => RemotePhase::PublishingTelemetry,
    }
}

fn plan_effect(
    session: &mut RemoteSession,
    kind: RemoteEffectKind,
    phase: RemotePhase,
    reason: &str,
) -> Result<(), RemoteCoreError> {
    let sequence = next_sequence(session)?;
    let mut effect = RemoteEffect {
        schema: REMOTE_EFFECT_SCHEMA.to_string(),
        effect_id_blake3: String::new(),
        sequence,
        job_id: session.command.job_id.clone(),
        attempt_id: session.attempt_id.clone(),
        fence_generation: session.command.fence_generation,
        capability: kind.capability(),
        kind,
        limits: effect_limits(&session.command, kind)?,
    };
    effect.effect_id_blake3 = remote_effect_identity(effect.clone())?;
    record_event(session, phase, reason, Some(effect.effect_id_blake3.clone()), sequence)?;
    session.pending_effect = Some(effect);
    debug_assert_eq!(session.pending_effect.as_ref().map(|value| value.capability), Some(kind.capability()));
    debug_assert_eq!(session.phase, phase);
    Ok(())
}

fn next_sequence(session: &RemoteSession) -> Result<u32, RemoteCoreError> {
    let sequence = session
        .sequence
        .checked_add(1)
        .ok_or_else(|| RemoteCoreError::invalid("remote-transition-sequence-overflow"))?;
    if sequence > MAX_REMOTE_EFFECT_STEPS {
        return Err(RemoteCoreError::TransitionLimitExceeded);
    }
    Ok(sequence)
}

fn effect_limits(command: &RemoteCommand, kind: RemoteEffectKind) -> Result<RemoteEffectLimits, RemoteCoreError> {
    let outputs_max = u32::try_from(command.expected_outputs.len())
        .map_err(|_| RemoteCoreError::invalid("remote-expected-output-count-overflow"))?;
    let bytes_max = match kind {
        RemoteEffectKind::TransferInputs => command.policy.input_bytes_max,
        RemoteEffectKind::LaunchExecutor | RemoteEffectKind::AdmitOutputs => command.policy.output_bytes_max,
        RemoteEffectKind::VerifyCredential
        | RemoteEffectKind::SendHandshake
        | RemoteEffectKind::ObserveClock
        | RemoteEffectKind::GenerateAttemptId
        | RemoteEffectKind::LoadAttempt
        | RemoteEffectKind::PersistAttempt
        | RemoteEffectKind::ReserveLease
        | RemoteEffectKind::ReleaseLease
        | RemoteEffectKind::PublishTelemetry => 0,
    };
    debug_assert_eq!(usize::try_from(outputs_max).ok(), Some(command.expected_outputs.len()));
    debug_assert!(bytes_max <= command.policy.input_bytes_max.max(command.policy.output_bytes_max));
    Ok(RemoteEffectLimits {
        bytes_max,
        outputs_max,
        build_time_ms_max: command.policy.build_time_ms_max,
        cpu_units_max: command.policy.cpu_units_max,
        memory_bytes_max: command.policy.memory_bytes_max,
    })
}

fn record_event(
    session: &mut RemoteSession,
    to: RemotePhase,
    reason: &str,
    effect_id_blake3: Option<String>,
    sequence: u32,
) -> Result<(), RemoteCoreError> {
    let event_count_max = usize::try_from(MAX_REMOTE_EFFECT_STEPS)
        .map_err(|_| RemoteCoreError::invalid("remote-event-limit-conversion-failed"))?;
    if session.events.len() >= event_count_max {
        return Err(RemoteCoreError::TransitionLimitExceeded);
    }
    let from = session.phase;
    session.events.push(RemoteEvent {
        sequence,
        from,
        to,
        reason_code: reason.to_string(),
        effect_id_blake3,
    });
    session.phase = to;
    session.sequence = sequence;
    debug_assert_eq!(session.events.last().map(|event| event.to), Some(to));
    debug_assert_eq!(session.sequence, sequence);
    Ok(())
}

fn record_terminal_event(session: &mut RemoteSession, phase: RemotePhase, reason: &str) -> Result<(), RemoteCoreError> {
    let sequence = next_sequence(session)?;
    record_event(session, phase, reason, None, sequence)?;
    session.pending_effect = None;
    debug_assert!(matches!(phase, RemotePhase::Succeeded | RemotePhase::Failed));
    debug_assert!(session.pending_effect.is_none());
    Ok(())
}

fn fail_session(session: &mut RemoteSession, reason: &str) -> Result<(), RemoteCoreError> {
    record_terminal_event(session, RemotePhase::Failed, reason)?;
    set_outcome(session, RemoteOutcomeStatus::Failed, reason)
}

fn set_outcome(session: &mut RemoteSession, status: RemoteOutcomeStatus, reason: &str) -> Result<(), RemoteCoreError> {
    session.outcome = Some(RemoteOutcome {
        status,
        reason_code: reason.to_string(),
        attempts: session.attempt_ordinal,
        receipt_preimage_blake3: String::new(),
        non_claim: REMOTE_CORE_NON_CLAIM.to_string(),
    });
    let preimage = remote_receipt_preimage(session);
    let receipt_preimage_blake3 = remote_receipt_preimage_identity(preimage)?;
    let outcome = session.outcome.as_mut().ok_or(RemoteCoreError::TerminalSession)?;
    outcome.receipt_preimage_blake3 = receipt_preimage_blake3;
    debug_assert!(session.pending_effect.is_none());
    debug_assert!(session.outcome.as_ref().is_some_and(|value| !value.receipt_preimage_blake3.is_empty()));
    Ok(())
}

#[must_use]
pub fn remote_receipt_preimage(session: &RemoteSession) -> RemoteReceiptPreimage {
    RemoteReceiptPreimage {
        schema: REMOTE_RECEIPT_PREIMAGE_SCHEMA.to_string(),
        command_blake3: session.command_blake3.clone(),
        job_id: session.command.job_id.clone(),
        worker_id: session.command.worker_id.clone(),
        attempt_id: session.attempt_id.clone(),
        fence_generation: session.command.fence_generation,
        phase: session.phase,
        event_reason_codes: session.events.iter().map(|event| event.reason_code.clone()).collect(),
        pending_effect_blake3: session.pending_effect.as_ref().map(|effect| effect.effect_id_blake3.clone()),
        outcome_status: session.outcome.as_ref().map(|outcome| outcome.status),
        outcome_reason_code: session.outcome.as_ref().map(|outcome| outcome.reason_code.clone()),
        non_claim: REMOTE_CORE_NON_CLAIM.to_string(),
    }
}

#[must_use]
pub fn successful_observation(effect: &RemoteEffect) -> RemoteObservation {
    RemoteObservation {
        schema: REMOTE_OBSERVATION_SCHEMA.to_string(),
        effect_id_blake3: effect.effect_id_blake3.clone(),
        attempt_id: effect.attempt_id.clone(),
        fence_generation: effect.fence_generation,
        kind: effect.kind.expected_observation(),
        status: RemoteObservationStatus::Succeeded,
        failure_class: None,
        reason_code: None,
        observed_bytes: 0,
        observed_outputs: 0,
        output_trusted: false,
    }
}

#[must_use]
pub fn failed_observation(
    effect: &RemoteEffect,
    failure_class: RemoteFailureClass,
    reason_code: String,
) -> RemoteObservation {
    RemoteObservation {
        schema: REMOTE_OBSERVATION_SCHEMA.to_string(),
        effect_id_blake3: effect.effect_id_blake3.clone(),
        attempt_id: effect.attempt_id.clone(),
        fence_generation: effect.fence_generation,
        kind: effect.kind.expected_observation(),
        status: RemoteObservationStatus::Failed,
        failure_class: Some(failure_class),
        reason_code: Some(reason_code),
        observed_bytes: 0,
        observed_outputs: 0,
        output_trusted: false,
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;
    use crate::REMOTE_COMMAND_SCHEMA;
    use crate::RemoteOutputExpectation;
    use crate::RemotePolicy;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const INPUT_BYTES_MAX: u64 = 1_024;
    const OUTPUT_BYTES_MAX: u64 = 2_048;
    const BUILD_TIME_MS_MAX: u64 = 60_000;
    const CPU_UNITS_MAX: u32 = 4;
    const MEMORY_BYTES_MAX: u64 = 1_048_576;
    const ATTEMPTS_MAX: u32 = 2;

    fn command() -> RemoteCommand {
        RemoteCommand {
            schema: REMOTE_COMMAND_SCHEMA.to_string(),
            job_id: "job-a".to_string(),
            request_blake3: DIGEST.to_string(),
            worker_id: "worker-a".to_string(),
            fence_generation: 1,
            input_refs: vec!["source-b".to_string(), "source-a".to_string()],
            expected_outputs: vec![RemoteOutputExpectation {
                name: "out".to_string(),
                logical_path: Some("/mantle/store/example".to_string()),
            }],
            policy: RemotePolicy {
                attempts_max: ATTEMPTS_MAX,
                input_bytes_max: INPUT_BYTES_MAX,
                output_bytes_max: OUTPUT_BYTES_MAX,
                build_time_ms_max: BUILD_TIME_MS_MAX,
                cpu_units_max: CPU_UNITS_MAX,
                memory_bytes_max: MEMORY_BYTES_MAX,
                allow_transient_retry: true,
                require_output_trust: true,
            },
        }
    }

    fn advance_success(mut session: RemoteSession) -> RemoteSession {
        loop {
            let effect = match session.pending_effect.clone() {
                Some(effect) => effect,
                None => return session,
            };
            let mut observation = successful_observation(&effect);
            if effect.kind == RemoteEffectKind::GenerateAttemptId {
                observation.attempt_id = Some(alloc::format!("attempt-{}", session.attempt_ordinal + 1));
            }
            if effect.kind == RemoteEffectKind::TransferInputs {
                observation.observed_bytes = INPUT_BYTES_MAX;
            }
            if effect.kind == RemoteEffectKind::LaunchExecutor {
                observation.observed_bytes = OUTPUT_BYTES_MAX;
                observation.observed_outputs = 1;
            }
            if effect.kind == RemoteEffectKind::AdmitOutputs {
                observation.observed_bytes = OUTPUT_BYTES_MAX;
                observation.observed_outputs = 1;
                observation.output_trusted = true;
            }
            session = apply_remote_observation(session, observation).expect("valid scripted observation");
        }
    }

    #[test]
    fn successful_observations_produce_terminal_receipt() {
        let session = advance_success(start_remote_session(command()).expect("valid command"));
        let outcome = session.outcome.expect("terminal outcome");
        assert_eq!(outcome.status, RemoteOutcomeStatus::Succeeded);
        assert_eq!(outcome.attempts, 1);
        assert_eq!(outcome.receipt_preimage_blake3.len(), crate::BLAKE3_HEX_LENGTH);
    }

    #[test]
    fn equivalent_collection_orders_replay_identically() {
        let first = advance_success(start_remote_session(command()).expect("first command"));
        let mut reordered = command();
        reordered.input_refs.reverse();
        let second = advance_success(start_remote_session(reordered).expect("second command"));
        assert_eq!(first.command_blake3, second.command_blake3);
        assert_eq!(first.events, second.events);
        assert_eq!(first.outcome, second.outcome);
    }

    #[test]
    fn wrong_effect_and_stale_fence_fail_closed() {
        let session = start_remote_session(command()).expect("valid command");
        let effect = session.pending_effect.clone().expect("pending effect");
        let mut wrong_effect = successful_observation(&effect);
        wrong_effect.effect_id_blake3 = DIGEST.to_string();
        assert_eq!(apply_remote_observation(session.clone(), wrong_effect), Err(RemoteCoreError::WrongEffectIdentity));
        let mut stale = successful_observation(&effect);
        stale.fence_generation = effect.fence_generation + 1;
        assert_eq!(apply_remote_observation(session, stale), Err(RemoteCoreError::StaleFence));
    }

    #[test]
    fn transient_executor_failure_retries_with_new_attempt() {
        let mut session = start_remote_session(command()).expect("valid command");
        loop {
            let effect = session.pending_effect.clone().expect("pending effect");
            if effect.kind == RemoteEffectKind::LaunchExecutor {
                let failed = failed_observation(&effect, RemoteFailureClass::Transient, "worker-lost".to_string());
                session = apply_remote_observation(session, failed).expect("retry planned");
                break;
            }
            let mut observation = successful_observation(&effect);
            if effect.kind == RemoteEffectKind::GenerateAttemptId {
                observation.attempt_id = Some("attempt-one".to_string());
            }
            session = apply_remote_observation(session, observation).expect("advance to executor");
        }
        assert!(session.retry_after_release);
        assert_eq!(session.pending_effect.as_ref().map(|effect| effect.kind), Some(RemoteEffectKind::ReleaseLease));
        assert_eq!(session.attempt_ordinal, 1);
    }

    #[test]
    fn transfer_observation_above_effect_limit_fails_closed() {
        let mut session = start_remote_session(command()).expect("valid command");
        loop {
            let effect = session.pending_effect.clone().expect("pending effect");
            let mut observation = successful_observation(&effect);
            if effect.kind == RemoteEffectKind::GenerateAttemptId {
                observation.attempt_id = Some("attempt-one".to_string());
            }
            if effect.kind == RemoteEffectKind::TransferInputs {
                observation.observed_bytes = INPUT_BYTES_MAX.checked_add(1).expect("fixture byte bound");
                let error = apply_remote_observation(session, observation).unwrap_err();
                assert_eq!(error.code(), "remote-observed-byte-limit-exceeded");
                assert!(matches!(error, RemoteCoreError::Invalid { .. }));
                break;
            }
            session = apply_remote_observation(session, observation).expect("advance to transfer");
        }
    }

    #[test]
    fn untrusted_output_cannot_become_success() {
        let mut session = start_remote_session(command()).expect("valid command");
        loop {
            let effect = session.pending_effect.clone().expect("pending effect");
            let mut observation = successful_observation(&effect);
            if effect.kind == RemoteEffectKind::GenerateAttemptId {
                observation.attempt_id = Some("attempt-one".to_string());
            }
            if effect.kind == RemoteEffectKind::AdmitOutputs {
                observation.observed_outputs = 1;
                session = apply_remote_observation(session, observation).expect("domain rejection");
                break;
            }
            session = apply_remote_observation(session, observation).expect("advance to admission");
        }
        assert_eq!(session.phase, RemotePhase::Failed);
        assert_eq!(session.outcome.as_ref().map(|outcome| outcome.reason_code.as_str()), Some("untrusted-output"));
    }
}
