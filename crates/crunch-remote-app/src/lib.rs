#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::string::String;

use crunch_remote_core::attempt::RemoteAttemptApplyDisposition;
use crunch_remote_core::attempt::RemoteAttemptApplyPlan;
use crunch_remote_core::attempt::RemoteAttemptReport;
use crunch_remote_core::effect::Authority;
use crunch_remote_core::effect::EffectBlocker;
use crunch_remote_core::effect::EffectKind;
use crunch_remote_core::effect::EffectLimits;
use crunch_remote_core::effect::EffectSession;
use crunch_remote_core::effect::Observation;
use crunch_remote_core::effect::ObservedEffect;

/// Borrowed identity actually consumed by the store-admission command. The
/// owning std adapter retains PathInfo, NAR bytes, and signature authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputFacts<'a> {
    pub name: &'a str,
    pub logical_path: &'a str,
    pub nar_sha256: &'a [u8; 32],
    pub nar_size_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Transport,
    AttemptPersistence,
    Executor,
    StoreAdmission,
    Credentials,
    Clock,
    RandomIdentifier,
    Telemetry,
    Lease,
    InputTransfer,
    OutputTransfer,
    ExternalBatch,
}

impl Capability {
    const fn authority(self) -> Authority {
        match self {
            Self::Transport => Authority::Transport,
            Self::ExternalBatch => Authority::ExternalBatch,
            Self::AttemptPersistence => Authority::AttemptPersistence,
            Self::Executor => Authority::Executor,
            Self::StoreAdmission => Authority::OutputAdmission,
            Self::Credentials => Authority::CredentialVerification,
            Self::Clock => Authority::Clock,
            Self::RandomIdentifier => Authority::RandomIdentifier,
            Self::Telemetry => Authority::Telemetry,
            Self::Lease => Authority::Lease,
            Self::InputTransfer => Authority::InputTransfer,
            Self::OutputTransfer => Authority::OutputTransfer,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortError {
    pub capability: Capability,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationError {
    Decision(EffectBlocker),
    Port(PortError),
}

pub struct ExecutorCommand<'a> {
    pub request_id: &'a str,
    pub plan_digest_blake3: &'a str,
    pub expected_outputs: u32,
    pub upload_bytes: u64,
    pub timeout_secs: u64,
}

/// A bounded, owned observation of an already validated executable result.
/// The std adapter keeps its vendor result until the effect completes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionFacts {
    pub outputs: u32,
    pub output_digest_blake3: [u8; 32],
}

pub trait ExecutorPort {
    fn launch(&mut self, command: ExecutorCommand<'_>) -> Result<ExecutionFacts, PortError>;
}

/// The host keeps the provider operation and raw response; the application
/// admits only an observation bound to the exact fenced operation.
#[derive(Debug, Clone, Copy)]
pub struct BatchDispatchCommand<'a> {
    pub operation_id_blake3: &'a str,
    pub operation_digest_blake3: [u8; 32],
    pub job_id: &'a crunch_remote_core::attempt::RemoteJobId,
    pub attempt_id: &'a crunch_remote_core::attempt::RemoteAttemptId,
    pub fence_generation: crunch_remote_core::attempt::RemoteFenceGeneration,
    pub observed_unix_s: u64,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchDispatchFacts {
    pub operation_digest_blake3: [u8; 32],
    pub observed_unix_s: u64,
    pub state: crunch_remote_core::external_batch::BatchState,
}

pub trait ExternalBatchPort {
    fn dispatch(&mut self, command: BatchDispatchCommand<'_>) -> Result<BatchDispatchFacts, PortError>;
}

pub trait TransportPort {
    fn send(&mut self, payload: &[u8]) -> Result<u64, PortError>;
}
/// The host retains the child, full framed transcript and streamed receiver;
/// this borrowed observation is not authority to admit an output to the store.
#[derive(Debug, Clone, Copy)]
pub struct ClientExchangeCommand<'a> {
    pub request_id: &'a str,
    pub job_id: &'a crunch_remote_core::attempt::RemoteJobId,
    pub attempt_id: &'a crunch_remote_core::attempt::RemoteAttemptId,
    pub fence_generation: crunch_remote_core::attempt::RemoteFenceGeneration,
    pub timeout_secs: u64,
    pub frames_max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientExchangeFacts<'a> {
    pub request_id: &'a str,
    pub job_id: &'a crunch_remote_core::attempt::RemoteJobId,
    pub attempt_id: &'a crunch_remote_core::attempt::RemoteAttemptId,
    pub fence_generation: crunch_remote_core::attempt::RemoteFenceGeneration,
    pub received_frames: u32,
    pub final_receipt_received: bool,
    pub child_exit_success: bool,
}

pub trait ClientExchangePort {
    fn exchange<'a>(&mut self, command: ClientExchangeCommand<'a>) -> Result<ClientExchangeFacts<'a>, PortError>;
}


/// The adapter retains its concrete durable snapshot; this port only reports
/// whether the actual load or persistence finished.
pub trait AttemptPersistencePort {
    fn load(&mut self, request_id: &str) -> Result<(), PortError>;
    fn persist(&mut self, request_id: &str) -> Result<(), PortError>;
}

/// Commit a core-approved attempt transition, including any immutable log
/// publication and the coordinator snapshot, in its existing durable order.
pub trait AttemptTransitionPort {
    fn persist_transition(
        &mut self,
        report: &RemoteAttemptReport,
        plan: &RemoteAttemptApplyPlan,
    ) -> Result<(), PortError>;
}

pub trait LeasePort {
    fn reserve(&mut self, request_id: &str, refs: &[String]) -> Result<(), PortError>;
    fn renew(&mut self, request_id: &str, refs: &[String]) -> Result<(), PortError>;
    fn release(&mut self, request_id: &str) -> Result<(), PortError>;
    fn quarantine(&mut self, request_id: &str) -> Result<(), PortError>;
}

pub trait InputTransferPort {
    fn receive(&mut self, request_id: &str, bytes_max: u64) -> Result<u64, PortError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputTransferFacts {
    pub transferred_bytes: u64,
    pub reused_bytes: u64,
}

pub trait OutputTransferPort {
    fn transfer(&mut self, request_id: &str, bytes_max: u64) -> Result<OutputTransferFacts, PortError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreAdmissionFacts {
    pub persisted_outputs: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreAdmissionFailure {
    pub error: PortError,
    /// Already persisted outputs remain durable even when the rest fail.
    pub persisted_outputs: u32,
    pub current_output_may_be_durable: bool,
}

pub trait StoreAdmissionPort {
    fn admit<'a>(
        &mut self,
        request_id: &str,
        outputs: impl ExactSizeIterator<Item = OutputFacts<'a>>,
    ) -> impl Future<Output = Result<StoreAdmissionFacts, StoreAdmissionFailure>>;
}

pub trait CredentialVerificationPort {
    fn verify(&mut self, request_id: &str, credential: &[u8]) -> Result<(), PortError>;
}

pub trait ClockObservationPort {
    fn now_unix_s(&mut self) -> Result<u64, PortError>;
}

pub trait RandomIdentifierPort {
    fn random_bytes(&mut self, output: &mut [u8]) -> Result<(), PortError>;
}

pub trait TelemetryPort {
    fn publish(&mut self, request_id: &str, event: &[u8]) -> Result<u64, PortError>;
}
/// Publish the already bounded event slice without re-encoding or copying it.
/// The std port owns delivery and retains its exact adapter health report.
pub trait TelemetryEventsPort<E> {
    fn publish_events(&mut self, request_id: &str, events: &[E]) -> Result<u32, PortError>;
}


/// The shell must submit either the real port's success or its actual failure;
/// no dependent plan can run while an observation is pending.
fn record_observation<'a>(
    session: &mut EffectSession<'a>,
    id: crunch_remote_core::effect::EffectId<'a>,
    observed: Result<ObservedEffect, PortError>,
) -> Result<(), ApplicationError> {
    match observed {
        Ok(effect) => {
            match session.observe(Observation::Succeeded { id, effect }).map_err(ApplicationError::Decision)? {
                crunch_remote_core::effect::EffectEvent::Completed(completed) if completed == id => Ok(()),
                _ => Err(ApplicationError::Decision(EffectBlocker::WrongObservationKind)),
            }
        }
        Err(error) => record_port_failure(session, id, error),
    }
}

fn record_port_failure<'a, T>(
    session: &mut EffectSession<'a>,
    id: crunch_remote_core::effect::EffectId<'a>,
    error: PortError,
) -> Result<T, ApplicationError> {
    match session
        .observe(Observation::Failed {
            id,
            authority: error.capability.authority(),
        })
        .map_err(ApplicationError::Decision)?
    {
        crunch_remote_core::effect::EffectEvent::Failed(failed) if failed == id => Err(ApplicationError::Port(error)),
        _ => Err(ApplicationError::Decision(EffectBlocker::WrongObservationKind)),
    }
}

fn record_partial_admission<'a, T>(
    session: &mut EffectSession<'a>,
    id: crunch_remote_core::effect::EffectId<'a>,
    failure: StoreAdmissionFailure,
) -> Result<T, ApplicationError> {
    if failure.persisted_outputs == 0 && !failure.current_output_may_be_durable {
        return record_port_failure(session, id, failure.error);
    }
    if failure.error.capability != Capability::StoreAdmission {
        return Err(ApplicationError::Decision(EffectBlocker::WrongAuthority));
    }
    match session
        .observe(Observation::PartiallyFailed {
            id,
            effect: ObservedEffect::OutputsPartiallyAdmitted {
                outputs: failure.persisted_outputs,
                current_output_may_be_durable: failure.current_output_may_be_durable,
            },
        })
        .map_err(ApplicationError::Decision)?
    {
        crunch_remote_core::effect::EffectEvent::PartiallyFailed {
            id: failed,
            outputs,
            current_output_may_be_durable,
        } if failed == id
            && outputs == failure.persisted_outputs
            && current_output_may_be_durable == failure.current_output_may_be_durable =>
        {
            Err(ApplicationError::Port(failure.error))
        }
        _ => Err(ApplicationError::Decision(EffectBlocker::WrongObservationKind)),
    }
}

/// A rejected or duplicate report must never touch a port. An applied report
/// needs the exact attempt, event, authority, and observed persistence effect
/// before the shell may publish its proposed next state as committed.
// r[impl remote_builds.remote_effect_plans]
pub fn apply_attempt_transition<'a, P: AttemptTransitionPort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    report: &RemoteAttemptReport,
    plan: &RemoteAttemptApplyPlan,
) -> Result<(), ApplicationError> {
    if plan.disposition != RemoteAttemptApplyDisposition::Applied {
        return if plan.pending_effect.is_none() {
            Ok(())
        } else {
            Err(ApplicationError::Decision(EffectBlocker::WrongObservationKind))
        };
    }
    if session.request_id() != report.identity.attempt_id.as_str()
        || plan.next_state.job_id != report.identity.job_id
        || plan.next_state.attempt_id != report.identity.attempt_id
        || plan.next_state.fence_generation != report.identity.fence_generation
        || plan.next_state.applied_events.get(&report.identity.event_id) != Some(&report.identity.payload_digest)
    {
        return Err(ApplicationError::Decision(EffectBlocker::WrongEffectId));
    }
    if session.binding().is_some() {
        session
            .require_attempt(&report.identity.job_id, &report.identity.attempt_id, report.identity.fence_generation)
            .map_err(ApplicationError::Decision)?;
    }
    let Some(kind) = plan.pending_effect else {
        return Err(ApplicationError::Decision(EffectBlocker::WrongObservationKind));
    };
    if kind != EffectKind::AttemptPersist {
        return Err(ApplicationError::Decision(EffectBlocker::WrongObservationKind));
    }
    let effect = session
        .plan(kind, EffectLimits {
            bytes_max: 0,
            items_max: 1,
            time_secs_max: 0,
        })
        .map_err(ApplicationError::Decision)?;
    record_observation(
        session,
        effect.id,
        port.persist_transition(report, plan).map(|()| ObservedEffect::AttemptPersisted),
    )
}

// r[impl remote_builds.application_owned_ports]
// r[impl remote_builds.remote_effect_plans]
pub fn execute<'a, P: ExecutorPort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    command: ExecutorCommand<'_>,
) -> Result<ExecutionFacts, ApplicationError> {
    if session.request_id() != command.request_id {
        return Err(ApplicationError::Decision(EffectBlocker::WrongEffectId));
    }
    let effect = session
        .plan(EffectKind::ExecutorLaunch, EffectLimits {
            bytes_max: command.upload_bytes,
            items_max: command.expected_outputs,
            time_secs_max: command.timeout_secs,
        })
        .map_err(ApplicationError::Decision)?;
    let expected_outputs = command.expected_outputs;
    match port.launch(command) {
        Ok(facts) if facts.outputs == expected_outputs => {
            record_observation(session, effect.id, Ok(ObservedEffect::ExecutorCompleted { outputs: facts.outputs }))?;
            Ok(facts)
        }
        Ok(_) => record_port_failure(session, effect.id, PortError {
            capability: Capability::Executor,
            reason: String::from("remote-execution-output-count-exceeded"),
        }),
        Err(error) => record_port_failure(session, effect.id, error),
    }
}

/// A completed effect attests only to a validated provider response. An
/// `Unknown` job state is preserved for reconciliation, never build success.
pub fn dispatch_external_batch<'a, P: ExternalBatchPort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    command: BatchDispatchCommand<'_>,
) -> Result<BatchDispatchFacts, ApplicationError> {
    check_request(session, command.operation_id_blake3)?;
    session
        .require_attempt(command.job_id, command.attempt_id, command.fence_generation)
        .map_err(ApplicationError::Decision)?;
    let effect = session
        .plan(EffectKind::ExternalBatchDispatch, EffectLimits {
            bytes_max: 0,
            items_max: 1,
            time_secs_max: command.timeout_secs,
        })
        .map_err(ApplicationError::Decision)?;
    let expected_digest = command.operation_digest_blake3;
    let expected_unix_s = command.observed_unix_s;
    match port.dispatch(command) {
        Ok(facts) if facts.operation_digest_blake3 == expected_digest && facts.observed_unix_s == expected_unix_s => {
            record_observation(session, effect.id, Ok(ObservedEffect::ExternalBatchObserved { state: facts.state }))?;
            Ok(facts)
        }
        Ok(_) => record_port_failure(session, effect.id, PortError {
            capability: Capability::ExternalBatch,
            reason: String::from("external-batch-observation-identity-mismatch"),
        }),
        Err(error) => record_port_failure(session, effect.id, error),
    }
}

pub fn send<'a, P: TransportPort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    payload: &[u8],
) -> Result<(), ApplicationError> {
    let bytes_max =
        u64::try_from(payload.len()).map_err(|_| ApplicationError::Decision(EffectBlocker::EffectLimitExceeded))?;
    let effect = session
        .plan(EffectKind::TransportSend, EffectLimits {
            bytes_max,
            items_max: 1,
            time_secs_max: 0,
        })
        .map_err(ApplicationError::Decision)?;
    let observed = port.send(payload).and_then(|bytes| {
        if bytes != bytes_max {
            return Err(PortError {
                capability: Capability::Transport,
                reason: String::from("remote-transport-short-write"),
            });
        }
        Ok(ObservedEffect::TransportSent { bytes })
    });
    record_observation(session, effect.id, observed)
}
/// The process, frame receiver, final receipt and exit code must all be
/// observed under the same bounded transport effect before store admission.
pub fn exchange_client<'session, 'command, P: ClientExchangePort>(
    session: &mut EffectSession<'session>,
    port: &mut P,
    command: ClientExchangeCommand<'command>,
) -> Result<ClientExchangeFacts<'command>, ApplicationError> {
    check_request(session, command.request_id)?;
    session
        .require_attempt(command.job_id, command.attempt_id, command.fence_generation)
        .map_err(ApplicationError::Decision)?;
    let effect = session
        .plan(EffectKind::TransportExchange, EffectLimits {
            bytes_max: 0,
            items_max: command.frames_max,
            time_secs_max: command.timeout_secs,
        })
        .map_err(ApplicationError::Decision)?;
    match port.exchange(command) {
        Ok(facts)
            if facts.request_id == command.request_id
                && facts.job_id == command.job_id
                && facts.attempt_id == command.attempt_id
                && facts.fence_generation == command.fence_generation
                && facts.received_frames > 0
                && facts.received_frames <= command.frames_max
                && facts.final_receipt_received
                && facts.child_exit_success =>
        {
            record_observation(
                session,
                effect.id,
                Ok(ObservedEffect::TransportExchanged {
                    frames: facts.received_frames,
                }),
            )?;
            Ok(facts)
        }
        Ok(_) => record_port_failure(session, effect.id, PortError {
            capability: Capability::Transport,
            reason: String::from("remote-transport-exchange-receipt-invalid"),
        }),
        Err(error) => record_port_failure(session, effect.id, error),
    }
}


pub fn receive_inputs<'a, P: InputTransferPort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    request_id: &str,
    bytes_max: u64,
    time_secs_max: u64,
) -> Result<u64, ApplicationError> {
    if session.request_id() != request_id {
        return Err(ApplicationError::Decision(EffectBlocker::WrongEffectId));
    }
    let effect = session
        .plan(EffectKind::InputTransfer, EffectLimits {
            bytes_max,
            items_max: 1,
            time_secs_max,
        })
        .map_err(ApplicationError::Decision)?;
    let observed = port.receive(request_id, bytes_max);
    match observed {
        Ok(bytes) => {
            record_observation(session, effect.id, Ok(ObservedEffect::InputsTransferred { bytes }))?;
            Ok(bytes)
        }
        Err(error) => record_port_failure(session, effect.id, error),
    }
}

pub fn transfer_outputs<'a, P: OutputTransferPort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    request_id: &str,
    bytes_max: u64,
    time_secs_max: u64,
) -> Result<OutputTransferFacts, ApplicationError> {
    if session.request_id() != request_id {
        return Err(ApplicationError::Decision(EffectBlocker::WrongEffectId));
    }
    let effect = session
        .plan(EffectKind::OutputTransfer, EffectLimits {
            bytes_max,
            items_max: 1,
            time_secs_max,
        })
        .map_err(ApplicationError::Decision)?;
    match port.transfer(request_id, bytes_max) {
        Ok(facts) => {
            if facts.transferred_bytes > bytes_max {
                return record_port_failure(session, effect.id, PortError {
                    capability: Capability::OutputTransfer,
                    reason: String::from("remote-output-transfer-byte-limit-exceeded"),
                });
            }
            record_observation(
                session,
                effect.id,
                Ok(ObservedEffect::OutputsTransferred {
                    bytes: facts.transferred_bytes,
                }),
            )?;
            Ok(facts)
        }
        Err(error) => record_port_failure(session, effect.id, error),
    }
}

pub fn load_attempt<'a, P: AttemptPersistencePort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    request_id: &str,
) -> Result<(), ApplicationError> {
    check_request(session, request_id)?;
    let effect = plan_single(session, EffectKind::AttemptLoad, 0)?;
    record_observation(session, effect, port.load(request_id).map(|()| ObservedEffect::AttemptLoaded))
}

pub fn persist_attempt<'a, P: AttemptPersistencePort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    request_id: &str,
) -> Result<(), ApplicationError> {
    check_request(session, request_id)?;
    let effect = plan_single(session, EffectKind::AttemptPersist, 0)?;
    record_observation(session, effect, port.persist(request_id).map(|()| ObservedEffect::AttemptPersisted))
}

pub fn reserve_lease<'a, P: LeasePort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    request_id: &str,
    refs: &[String],
) -> Result<(), ApplicationError> {
    check_request(session, request_id)?;
    let effect = plan_single(session, EffectKind::LeaseReserve, 0)?;
    record_observation(session, effect, port.reserve(request_id, refs).map(|()| ObservedEffect::LeaseReserved))
}

pub fn renew_lease<'a, P: LeasePort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    request_id: &str,
    refs: &[String],
) -> Result<(), ApplicationError> {
    check_request(session, request_id)?;
    let effect = plan_single(session, EffectKind::LeaseRenew, 0)?;
    record_observation(session, effect, port.renew(request_id, refs).map(|()| ObservedEffect::LeaseRenewed))
}

pub fn release_lease<'a, P: LeasePort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    request_id: &str,
) -> Result<(), ApplicationError> {
    check_request(session, request_id)?;
    let effect = plan_single(session, EffectKind::LeaseRelease, 0)?;
    record_observation(session, effect, port.release(request_id).map(|()| ObservedEffect::LeaseReleased))
}

pub fn quarantine_lease<'a, P: LeasePort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    request_id: &str,
) -> Result<(), ApplicationError> {
    check_request(session, request_id)?;
    let effect = plan_single(session, EffectKind::LeaseQuarantine, 0)?;
    record_observation(session, effect, port.quarantine(request_id).map(|()| ObservedEffect::LeaseQuarantined))
}

pub async fn admit_outputs<'session, 'output, P, I>(
    session: &mut EffectSession<'session>,
    port: &mut P,
    request_id: &str,
    outputs: I,
) -> Result<StoreAdmissionFacts, ApplicationError>
where
    P: StoreAdmissionPort,
    I: ExactSizeIterator<Item = OutputFacts<'output>>,
{
    check_request(session, request_id)?;
    let items_max =
        u32::try_from(outputs.len()).map_err(|_| ApplicationError::Decision(EffectBlocker::EffectLimitExceeded))?;
    let effect = session
        .plan(EffectKind::OutputAdmission, EffectLimits {
            bytes_max: 0,
            items_max,
            time_secs_max: 0,
        })
        .map_err(ApplicationError::Decision)?;
    let admission = port.admit(request_id, outputs).await.and_then(|facts| {
        if facts.persisted_outputs != items_max {
            Err(StoreAdmissionFailure {
                error: PortError {
                    capability: Capability::StoreAdmission,
                    reason: String::from("remote-output-admission-count-mismatch"),
                },
                persisted_outputs: facts.persisted_outputs,
                current_output_may_be_durable: false,
            })
        } else {
            Ok(facts)
        }
    });
    match admission {
        Ok(facts) => {
            record_observation(
                session,
                effect.id,
                Ok(ObservedEffect::OutputsAdmitted {
                    outputs: facts.persisted_outputs,
                }),
            )?;
            Ok(facts)
        }
        Err(failure) => record_partial_admission(session, effect.id, failure),
    }
}

pub fn verify_credentials<'a, P: CredentialVerificationPort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    request_id: &str,
    credential: &[u8],
) -> Result<(), ApplicationError> {
    check_request(session, request_id)?;
    let bytes_max =
        u64::try_from(credential.len()).map_err(|_| ApplicationError::Decision(EffectBlocker::EffectLimitExceeded))?;
    let effect = plan_single(session, EffectKind::CredentialVerify, bytes_max)?;
    record_observation(
        session,
        effect,
        port.verify(request_id, credential).map(|()| ObservedEffect::CredentialVerified),
    )
}

pub fn observe_clock<'a, P: ClockObservationPort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    request_id: &str,
) -> Result<u64, ApplicationError> {
    check_request(session, request_id)?;
    let effect = plan_single(session, EffectKind::ClockObserve, 0)?;
    match port.now_unix_s() {
        Ok(now) => {
            record_observation(session, effect, Ok(ObservedEffect::ClockObserved))?;
            Ok(now)
        }
        Err(error) => record_port_failure(session, effect, error),
    }
}

pub fn generate_identifier<'a, P: RandomIdentifierPort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    request_id: &str,
    entropy: &mut [u8],
) -> Result<(), ApplicationError> {
    check_request(session, request_id)?;
    let bytes =
        u64::try_from(entropy.len()).map_err(|_| ApplicationError::Decision(EffectBlocker::EffectLimitExceeded))?;
    let effect = plan_single(session, EffectKind::RandomIdentifier, bytes)?;
    record_observation(
        session,
        effect,
        port.random_bytes(entropy).map(|()| ObservedEffect::IdentifierGenerated { bytes }),
    )
}

pub fn publish_telemetry<'a, P: TelemetryPort>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    request_id: &str,
    event: &[u8],
) -> Result<(), ApplicationError> {
    check_request(session, request_id)?;
    let bytes_max =
        u64::try_from(event.len()).map_err(|_| ApplicationError::Decision(EffectBlocker::EffectLimitExceeded))?;
    let effect = plan_single(session, EffectKind::TelemetryPublish, bytes_max)?;
    let published = port.publish(request_id, event).and_then(|bytes| {
        if bytes != bytes_max {
            return Err(PortError {
                capability: Capability::Telemetry,
                reason: String::from("remote-telemetry-short-publish"),
            });
        }
        Ok(ObservedEffect::TelemetryPublished { bytes })
    });
    record_observation(session, effect, published)
}
pub fn publish_event_batch<'a, E, P: TelemetryEventsPort<E>>(
    session: &mut EffectSession<'a>,
    port: &mut P,
    request_id: &str,
    events: &[E],
) -> Result<(), ApplicationError> {
    check_request(session, request_id)?;
    let events_max =
        u32::try_from(events.len()).map_err(|_| ApplicationError::Decision(EffectBlocker::EffectLimitExceeded))?;
    let effect = session
        .plan(EffectKind::TelemetryPublish, EffectLimits {
            bytes_max: 0,
            items_max: events_max,
            time_secs_max: 0,
        })
        .map_err(ApplicationError::Decision)?;
    let observed = port.publish_events(request_id, events).and_then(|published| {
        if published != events_max {
            return Err(PortError {
                capability: Capability::Telemetry,
                reason: String::from("remote-telemetry-event-count-mismatch"),
            });
        }
        Ok(ObservedEffect::TelemetryEventsPublished { events: published })
    });
    record_observation(session, effect.id, observed)
}


fn check_request(session: &EffectSession<'_>, request_id: &str) -> Result<(), ApplicationError> {
    if session.request_id() == request_id {
        Ok(())
    } else {
        Err(ApplicationError::Decision(EffectBlocker::WrongEffectId))
    }
}

fn plan_single<'a>(
    session: &mut EffectSession<'a>,
    kind: EffectKind,
    bytes_max: u64,
) -> Result<crunch_remote_core::effect::EffectId<'a>, ApplicationError> {
    session
        .plan(kind, EffectLimits {
            bytes_max,
            items_max: 1,
            time_secs_max: 0,
        })
        .map(|effect| effect.id)
        .map_err(ApplicationError::Decision)
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use core::future::Future;

    struct PartialAdmission;

    impl StoreAdmissionPort for PartialAdmission {
        async fn admit<'a>(
            &mut self,
            _request_id: &str,
            _outputs: impl ExactSizeIterator<Item = OutputFacts<'a>>,
        ) -> Result<StoreAdmissionFacts, StoreAdmissionFailure> {
            Err(StoreAdmissionFailure {
                error: PortError {
                    capability: Capability::StoreAdmission,
                    reason: String::from("storage-second-output-failed"),
                },
                persisted_outputs: 1,
                current_output_may_be_durable: true,
            })
        }
    }

    #[test]
    fn partially_admitted_outputs_cannot_complete_the_store_effect() {
        let mut session = EffectSession::new("request-3").unwrap();
        let nar_sha256 = [0; 32];
        let outputs = [
            OutputFacts {
                name: "out",
                logical_path: "/mantle/store/example",
                nar_sha256: &nar_sha256,
                nar_size_bytes: 0,
            },
            OutputFacts {
                name: "dev",
                logical_path: "/mantle/store/example-dev",
                nar_sha256: &nar_sha256,
                nar_size_bytes: 0,
            },
        ];
        let outcome = {
            let mut context = std::task::Context::from_waker(std::task::Waker::noop());
            let mut port = PartialAdmission;
            let mut future =
                std::pin::pin!(admit_outputs(&mut session, &mut port, "request-3", outputs.iter().copied()));
            match future.as_mut().poll(&mut context) {
                std::task::Poll::Ready(outcome) => outcome,
                std::task::Poll::Pending => panic!("synchronous store adapter should finish"),
            }
        };
        assert_eq!(
            outcome,
            Err(ApplicationError::Port(PortError {
                capability: Capability::StoreAdmission,
                reason: String::from("storage-second-output-failed"),
            })),
        );
        assert_eq!(
            session.plan(EffectKind::ExecutorLaunch, EffectLimits {
                bytes_max: 0,
                items_max: 1,
                time_secs_max: 0
            }),
            Err(EffectBlocker::SessionFailed),
        );
    }

    struct FailingTransport {
        capability: Capability,
    }

    impl TransportPort for FailingTransport {
        fn send(&mut self, _payload: &[u8]) -> Result<u64, PortError> {
            Err(PortError {
                capability: self.capability,
                reason: String::from("socket-disconnected"),
            })
        }
    }

    #[test]
    fn failed_transport_is_terminal_and_cannot_advance_execution() {
        let mut session = EffectSession::new("request-1").unwrap();
        let failure = PortError {
            capability: Capability::Transport,
            reason: String::from("socket-disconnected"),
        };
        assert_eq!(
            send(
                &mut session,
                &mut FailingTransport {
                    capability: Capability::Transport
                },
                b"frame"
            ),
            Err(ApplicationError::Port(failure)),
        );
        assert_eq!(
            session.plan(EffectKind::ExecutorLaunch, EffectLimits {
                bytes_max: 0,
                items_max: 1,
                time_secs_max: 0
            }),
            Err(EffectBlocker::SessionFailed),
        );
    }

    #[test]
    fn misrouted_transport_failure_never_counts_as_transport_observation() {
        let mut session = EffectSession::new("request-2").unwrap();
        assert_eq!(
            send(
                &mut session,
                &mut FailingTransport {
                    capability: Capability::Executor
                },
                b"frame"
            ),
            Err(ApplicationError::Decision(EffectBlocker::WrongAuthority)),
        );
        assert_eq!(
            session.plan(EffectKind::ExecutorLaunch, EffectLimits {
                bytes_max: 0,
                items_max: 1,
                time_secs_max: 0
            }),
            Err(EffectBlocker::PendingEffect),
        );
    }

    struct InMemoryAttemptPort {
        stored: Option<crunch_remote_core::attempt::RemoteAttemptState>,
        fail: bool,
    }

    impl AttemptTransitionPort for InMemoryAttemptPort {
        fn persist_transition(
            &mut self,
            _report: &RemoteAttemptReport,
            plan: &RemoteAttemptApplyPlan,
        ) -> Result<(), PortError> {
            if self.fail {
                return Err(PortError {
                    capability: Capability::AttemptPersistence,
                    reason: String::from("snapshot-write-failed"),
                });
            }
            self.stored = Some(plan.next_state.clone());
            Ok(())
        }
    }

    fn attempt_case() -> (crunch_remote_core::attempt::RemoteAttemptState, RemoteAttemptReport) {
        use crunch_remote_core::attempt::RemoteAssignmentNonce;
        use crunch_remote_core::attempt::RemoteAttemptAssignmentInput;
        use crunch_remote_core::attempt::RemoteAttemptReportPayload;
        use crunch_remote_core::attempt::RemoteAttemptRetryPolicy;
        use crunch_remote_core::attempt::RemoteAttemptTimeFacts;
        use crunch_remote_core::attempt::RemoteEventId;
        use crunch_remote_core::attempt::RemoteJobId;
        use crunch_remote_core::attempt::plan_remote_attempt_assignment_from_input;

        let job_id = RemoteJobId::new("job-1").unwrap();
        let state = plan_remote_attempt_assignment_from_input(RemoteAttemptAssignmentInput {
            job_id: &job_id,
            worker_endpoint_id: "worker-1",
            assignment_nonce: RemoteAssignmentNonce::new("a".repeat(64)).unwrap(),
            previous: None,
            retry_policy: RemoteAttemptRetryPolicy::default(),
            time: RemoteAttemptTimeFacts {
                now_unix_s: 1,
                failure_observed_unix_s: 1,
                overall_deadline_unix_s: 1_000,
            },
        })
        .unwrap();
        let report = RemoteAttemptReport::new(
            job_id,
            state.attempt_id.clone(),
            state.fence_generation,
            RemoteEventId::new("start-event").unwrap(),
            RemoteAttemptReportPayload::Start,
        )
        .unwrap();
        (state, report)
    }

    #[test]
    fn accepted_attempt_persists_only_after_exact_observation_and_duplicate_is_no_op() {
        use crunch_remote_core::attempt::RemoteAttemptAuthorizationFacts;
        use crunch_remote_core::attempt::plan_remote_attempt_report;

        let (queued, report) = attempt_case();
        let authorization = RemoteAttemptAuthorizationFacts {
            worker_authorized: true,
            output_admission_authorized: true,
        };
        let accepted = plan_remote_attempt_report(&queued, &report, authorization);
        assert_eq!(accepted.pending_effect, Some(EffectKind::AttemptPersist));
        let mut session = EffectSession::new(queued.attempt_id.as_str()).unwrap();
        let mut port = InMemoryAttemptPort {
            stored: None,
            fail: false,
        };
        apply_attempt_transition(&mut session, &mut port, &report, &accepted).unwrap();
        assert_eq!(port.stored.as_ref(), Some(&accepted.next_state));
        let duplicate = plan_remote_attempt_report(&accepted.next_state, &report, authorization);
        assert_eq!(duplicate.disposition, RemoteAttemptApplyDisposition::AlreadyApplied);
        assert_eq!(duplicate.pending_effect, None);
        apply_attempt_transition(&mut session, &mut port, &report, &duplicate).unwrap();
        assert_eq!(port.stored.as_ref(), Some(&accepted.next_state));
    }

    #[test]
    fn stale_fence_report_never_requests_attempt_persistence() {
        use crunch_remote_core::attempt::RemoteAssignmentNonce;
        use crunch_remote_core::attempt::RemoteAttemptAssignmentInput;
        use crunch_remote_core::attempt::RemoteAttemptAuthorizationFacts;
        use crunch_remote_core::attempt::RemoteAttemptReasonCode;
        use crunch_remote_core::attempt::RemoteAttemptRetryPolicy;
        use crunch_remote_core::attempt::RemoteAttemptTimeFacts;
        use crunch_remote_core::attempt::plan_remote_attempt_assignment_from_input;
        use crunch_remote_core::attempt::plan_remote_attempt_report;

        let (previous, stale_report) = attempt_case();
        let current = plan_remote_attempt_assignment_from_input(RemoteAttemptAssignmentInput {
            job_id: &previous.job_id,
            worker_endpoint_id: "worker-1",
            assignment_nonce: RemoteAssignmentNonce::new("b".repeat(64)).unwrap(),
            previous: Some(&previous),
            retry_policy: RemoteAttemptRetryPolicy::default(),
            time: RemoteAttemptTimeFacts {
                now_unix_s: 2,
                failure_observed_unix_s: 2,
                overall_deadline_unix_s: 1_000,
            },
        })
        .unwrap();
        let plan = plan_remote_attempt_report(&current, &stale_report, RemoteAttemptAuthorizationFacts {
            worker_authorized: true,
            output_admission_authorized: true,
        });
        assert_eq!(plan.reason_code, RemoteAttemptReasonCode::StaleReportRejected);
        assert_eq!(plan.next_state, current);
        let mut session = EffectSession::new(current.attempt_id.as_str()).unwrap();
        let mut port = InMemoryAttemptPort {
            stored: None,
            fail: false,
        };
        apply_attempt_transition(&mut session, &mut port, &stale_report, &plan).unwrap();
        assert!(port.stored.is_none());
        let effect = session
            .plan(EffectKind::AttemptPersist, EffectLimits {
                bytes_max: 0,
                items_max: 1,
                time_secs_max: 0,
            })
            .unwrap();
        assert_eq!(effect.id.sequence, 0);
    }

    #[test]
    fn wrong_attempt_identity_and_failed_persistence_cannot_commit_a_transition() {
        use crunch_remote_core::attempt::RemoteAttemptAuthorizationFacts;
        use crunch_remote_core::attempt::plan_remote_attempt_report;

        let (queued, report) = attempt_case();
        let plan = plan_remote_attempt_report(&queued, &report, RemoteAttemptAuthorizationFacts {
            worker_authorized: true,
            output_admission_authorized: true,
        });
        let mut port = InMemoryAttemptPort {
            stored: None,
            fail: false,
        };
        let mut wrong = EffectSession::new("other-attempt").unwrap();
        assert_eq!(
            apply_attempt_transition(&mut wrong, &mut port, &report, &plan),
            Err(ApplicationError::Decision(EffectBlocker::WrongEffectId)),
        );
        assert!(port.stored.is_none());
        port.fail = true;
        let mut session = EffectSession::new(queued.attempt_id.as_str()).unwrap();
        assert_eq!(
            apply_attempt_transition(&mut session, &mut port, &report, &plan),
            Err(ApplicationError::Port(PortError {
                capability: Capability::AttemptPersistence,
                reason: String::from("snapshot-write-failed"),
            })),
        );
        assert!(port.stored.is_none());
        assert_eq!(
            session.plan(EffectKind::ExecutorLaunch, EffectLimits {
                bytes_max: 0,
                items_max: 1,
                time_secs_max: 0,
            }),
            Err(EffectBlocker::SessionFailed),
        );
    }
    struct ObservedBatchPort {
        observed: Option<Result<BatchDispatchFacts, PortError>>,
        calls: u32,
    }

    impl ExternalBatchPort for ObservedBatchPort {
        fn dispatch(&mut self, _command: BatchDispatchCommand<'_>) -> Result<BatchDispatchFacts, PortError> {
            self.calls += 1;
            self.observed.take().expect("one physical provider operation")
        }
    }

    #[test]
    fn batch_provider_unknown_is_not_build_success_and_stale_fence_never_dispatches() {
        use crunch_remote_core::effect::RemoteEffectBinding;
        use crunch_remote_core::external_batch::BatchState;

        let (attempt, _) = attempt_case();
        let digest = [7; 32];
        let command = BatchDispatchCommand {
            operation_id_blake3: "provider-operation",
            operation_digest_blake3: digest,
            job_id: &attempt.job_id,
            attempt_id: &attempt.attempt_id,
            fence_generation: attempt.fence_generation,
            observed_unix_s: 1_700_000_000,
            timeout_secs: 5,
        };
        let mut port = ObservedBatchPort {
            observed: Some(Ok(BatchDispatchFacts {
                operation_digest_blake3: digest,
                observed_unix_s: command.observed_unix_s,
                state: BatchState::Unknown,
            })),
            calls: 0,
        };
        let mut unbound = EffectSession::new(command.operation_id_blake3).unwrap();
        assert_eq!(
            dispatch_external_batch(&mut unbound, &mut port, command),
            Err(ApplicationError::Decision(EffectBlocker::AttemptBindingMismatch)),
        );
        assert_eq!(port.calls, 0);
        let mut bound = EffectSession::for_attempt(
            command.operation_id_blake3,
            RemoteEffectBinding {
                job_id: command.job_id,
                attempt_id: command.attempt_id,
                fence_generation: command.fence_generation,
            },
        )
        .unwrap();
        let facts = dispatch_external_batch(&mut bound, &mut port, command).unwrap();
        assert_eq!(facts.state, BatchState::Unknown);
        assert!(!facts.state.is_terminal());
        assert_eq!(port.calls, 1);
    }

    #[test]
    fn provider_outage_or_mismatched_observation_never_allows_a_dependent_effect() {
        use crunch_remote_core::effect::RemoteEffectBinding;
        use crunch_remote_core::external_batch::BatchState;

        let (attempt, _) = attempt_case();
        let command = BatchDispatchCommand {
            operation_id_blake3: "provider-operation",
            operation_digest_blake3: [7; 32],
            job_id: &attempt.job_id,
            attempt_id: &attempt.attempt_id,
            fence_generation: attempt.fence_generation,
            observed_unix_s: 1_700_000_000,
            timeout_secs: 5,
        };
        for observed in [
            Err(PortError {
                capability: Capability::ExternalBatch,
                reason: String::from("external-batch-process-timeout"),
            }),
            Ok(BatchDispatchFacts {
                operation_digest_blake3: [8; 32],
                observed_unix_s: command.observed_unix_s,
                state: BatchState::Succeeded,
            }),
        ] {
            let mut session = EffectSession::for_attempt(
                command.operation_id_blake3,
                RemoteEffectBinding {
                    job_id: command.job_id,
                    attempt_id: command.attempt_id,
                    fence_generation: command.fence_generation,
                },
            )
            .unwrap();
            let mut port = ObservedBatchPort {
                observed: Some(observed),
                calls: 0,
            };
            assert!(dispatch_external_batch(&mut session, &mut port, command).is_err());
            assert_eq!(port.calls, 1);
            assert_eq!(
                session.plan(EffectKind::OutputAdmission, EffectLimits {
                    bytes_max: 0,
                    items_max: 1,
                    time_secs_max: 0,
                }),
                Err(EffectBlocker::SessionFailed),
            );
        }
    }
    struct ObservedClientPort {
        fault: Option<&'static str>,
        calls: usize,
    }

    impl ClientExchangePort for ObservedClientPort {
        fn exchange<'a>(&mut self, command: ClientExchangeCommand<'a>) -> Result<ClientExchangeFacts<'a>, PortError> {
            self.calls += 1;
            Ok(ClientExchangeFacts {
                request_id: command.request_id,
                job_id: command.job_id,
                attempt_id: command.attempt_id,
                fence_generation: if self.fault == Some("stale-fence") {
                    crunch_remote_core::attempt::RemoteFenceGeneration::new(command.fence_generation.get() + 1).unwrap()
                } else {
                    command.fence_generation
                },
                received_frames: if self.fault == Some("over-limit") { command.frames_max + 1 } else { 2 },
                final_receipt_received: self.fault != Some("missing-done"),
                child_exit_success: self.fault != Some("failed-exit"),
            })
        }
    }

    #[test]
    fn client_exchange_rejects_stale_receipt_failed_child_and_over_limit_frames_before_store() {
        use crunch_remote_core::effect::{EffectActor, RemoteEffectBinding};

        let (attempt, _) = attempt_case();
        let command = ClientExchangeCommand {
            request_id: "client-stdio-request",
            job_id: &attempt.job_id,
            attempt_id: &attempt.attempt_id,
            fence_generation: attempt.fence_generation,
            timeout_secs: 4,
            frames_max: 4,
        };
        for fault in ["stale-fence", "missing-done", "failed-exit", "over-limit"] {
            let mut session = EffectSession::for_actor_attempt(
                command.request_id,
                RemoteEffectBinding {
                    job_id: command.job_id,
                    attempt_id: command.attempt_id,
                    fence_generation: command.fence_generation,
                },
                EffectActor::Client,
            )
            .unwrap();
            let mut port = ObservedClientPort {
                fault: Some(fault),
                calls: 0,
            };
            assert!(matches!(exchange_client(&mut session, &mut port, command), Err(ApplicationError::Port(_))));
            assert_eq!(port.calls, 1);
            assert_eq!(
                session.plan(EffectKind::OutputAdmission, EffectLimits {
                    bytes_max: 0,
                    items_max: 1,
                    time_secs_max: 0,
                }),
                Err(EffectBlocker::SessionFailed),
            );
        }
        let mut session = EffectSession::for_actor_attempt(
            command.request_id,
            RemoteEffectBinding {
                job_id: command.job_id,
                attempt_id: command.attempt_id,
                fence_generation: command.fence_generation,
            },
            EffectActor::Client,
        )
        .unwrap();
        let mut port = ObservedClientPort {
            fault: None,
            calls: 0,
        };
        assert_eq!(exchange_client(&mut session, &mut port, command).unwrap().received_frames, 2);
        assert_eq!(port.calls, 1);
    }
}
