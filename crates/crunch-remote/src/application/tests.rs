use std::cell::RefCell;
use std::rc::Rc;

use crunch_remote_core::REMOTE_COMMAND_SCHEMA;
use crunch_remote_core::REMOTE_OBSERVATION_SCHEMA;
use crunch_remote_core::RemoteCapability;
use crunch_remote_core::RemoteObservationKind;
use crunch_remote_core::RemoteObservationStatus;
use crunch_remote_core::RemoteOutcomeStatus;
use crunch_remote_core::RemoteOutputExpectation;
use crunch_remote_core::RemotePolicy;
use crunch_remote_core::successful_observation;

use super::*;
use crate::AttemptPersistencePort;
use crate::ClockObservationPort;
use crate::CredentialVerificationPort;
use crate::ExecutorPort;
use crate::RandomIdentifierPort;
use crate::StoreAdmissionPort;
use crate::TelemetryPublicationPort;
use crate::TransportPort;

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const INPUT_BYTES_MAX: u64 = 1_024;
const OUTPUT_BYTES_MAX: u64 = 2_048;
const BUILD_TIME_MS_MAX: u64 = 60_000;
const CPU_UNITS_MAX: u32 = 4;
const MEMORY_BYTES_MAX: u64 = 1_048_576;
const ATTEMPTS_MAX: u32 = 2;

#[derive(Default)]
struct ScriptedState {
    generated_attempts: u32,
    fail_executor_once: bool,
    corrupt_effect_identity: bool,
}

#[derive(Clone)]
struct ScriptedAdapter {
    state: Rc<RefCell<ScriptedState>>,
}

impl ScriptedAdapter {
    fn new(state: Rc<RefCell<ScriptedState>>) -> Self {
        Self { state }
    }

    fn success(&self, effect: crunch_remote_core::RemoteEffect) -> crunch_remote_core::RemoteObservation {
        let mut observation = successful_observation(&effect);
        let mut state = self.state.borrow_mut();
        match effect.kind {
            crunch_remote_core::RemoteEffectKind::GenerateAttemptId => {
                state.generated_attempts = state.generated_attempts.saturating_add(1);
                observation.attempt_id = Some(format!("attempt-{}", state.generated_attempts));
            }
            crunch_remote_core::RemoteEffectKind::TransferInputs => observation.observed_bytes = INPUT_BYTES_MAX,
            crunch_remote_core::RemoteEffectKind::LaunchExecutor => {
                observation.observed_bytes = OUTPUT_BYTES_MAX;
                observation.observed_outputs = 1;
            }
            crunch_remote_core::RemoteEffectKind::AdmitOutputs => {
                observation.observed_bytes = OUTPUT_BYTES_MAX;
                observation.observed_outputs = 1;
                observation.output_trusted = true;
            }
            crunch_remote_core::RemoteEffectKind::VerifyCredential
            | crunch_remote_core::RemoteEffectKind::SendHandshake
            | crunch_remote_core::RemoteEffectKind::ObserveClock
            | crunch_remote_core::RemoteEffectKind::LoadAttempt
            | crunch_remote_core::RemoteEffectKind::PersistAttempt
            | crunch_remote_core::RemoteEffectKind::ReserveLease
            | crunch_remote_core::RemoteEffectKind::ReleaseLease
            | crunch_remote_core::RemoteEffectKind::PublishTelemetry => {}
        }
        if state.corrupt_effect_identity {
            observation.effect_id_blake3 = DIGEST.to_string();
        }
        observation
    }
}

impl TransportPort for ScriptedAdapter {
    fn execute_transport(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, crate::RemotePortError> {
        Ok(self.success(effect))
    }
}

impl AttemptPersistencePort for ScriptedAdapter {
    fn execute_attempt(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, crate::RemotePortError> {
        Ok(self.success(effect))
    }
}

impl ExecutorPort for ScriptedAdapter {
    fn execute_build(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, crate::RemotePortError> {
        let should_fail = self.state.borrow().fail_executor_once;
        if should_fail {
            self.state.borrow_mut().fail_executor_once = false;
            return Err(crate::RemotePortError::new(RemoteCapability::Executor, "worker-lost", true));
        }
        Ok(self.success(effect))
    }
}

impl StoreAdmissionPort for ScriptedAdapter {
    fn execute_admission(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, crate::RemotePortError> {
        Ok(self.success(effect))
    }
}

impl CredentialVerificationPort for ScriptedAdapter {
    fn execute_credential_check(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, crate::RemotePortError> {
        Ok(self.success(effect))
    }
}

impl ClockObservationPort for ScriptedAdapter {
    fn execute_clock_observation(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, crate::RemotePortError> {
        Ok(self.success(effect))
    }
}

impl RandomIdentifierPort for ScriptedAdapter {
    fn execute_identifier_generation(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, crate::RemotePortError> {
        Ok(self.success(effect))
    }
}

impl TelemetryPublicationPort for ScriptedAdapter {
    fn execute_telemetry_publication(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, crate::RemotePortError> {
        Ok(self.success(effect))
    }
}

fn command() -> crunch_remote_core::RemoteCommand {
    crunch_remote_core::RemoteCommand {
        schema: REMOTE_COMMAND_SCHEMA.to_string(),
        job_id: "job-a".to_string(),
        request_blake3: DIGEST.to_string(),
        worker_id: "worker-a".to_string(),
        fence_generation: 1,
        input_refs: vec!["source-a".to_string()],
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

fn with_ports<T>(state: Rc<RefCell<ScriptedState>>, operation: impl FnOnce(&mut crate::RemotePortSet<'_>) -> T) -> T {
    let mut transport = ScriptedAdapter::new(state.clone());
    let mut attempts = ScriptedAdapter::new(state.clone());
    let mut executor = ScriptedAdapter::new(state.clone());
    let mut store = ScriptedAdapter::new(state.clone());
    let mut credentials = ScriptedAdapter::new(state.clone());
    let mut clock = ScriptedAdapter::new(state.clone());
    let mut identifiers = ScriptedAdapter::new(state.clone());
    let mut telemetry = ScriptedAdapter::new(state);
    let mut ports = crate::RemotePortSet {
        transport: &mut transport,
        attempts: &mut attempts,
        executor: &mut executor,
        store: &mut store,
        credentials: &mut credentials,
        clock: &mut clock,
        identifiers: &mut identifiers,
        telemetry: &mut telemetry,
    };
    operation(&mut ports)
}

#[test]
fn application_executes_each_effect_before_the_next_transition() {
    let state = Rc::new(RefCell::new(ScriptedState::default()));
    let outcome = with_ports(state.clone(), |ports| run(command(), ports)).unwrap();
    assert_eq!(outcome.status, RemoteOutcomeStatus::Succeeded);
    assert_eq!(state.borrow().generated_attempts, 1);
}

#[test]
fn transient_executor_failure_retries_through_observed_release() {
    let state = Rc::new(RefCell::new(ScriptedState {
        fail_executor_once: true,
        ..ScriptedState::default()
    }));
    let outcome = with_ports(state, |ports| run(command(), ports)).unwrap();
    assert_eq!(outcome.status, RemoteOutcomeStatus::Succeeded);
    assert_eq!(outcome.attempts, ATTEMPTS_MAX);
}

#[test]
fn wrong_adapter_observation_fails_closed() {
    let state = Rc::new(RefCell::new(ScriptedState {
        corrupt_effect_identity: true,
        ..ScriptedState::default()
    }));
    let error = with_ports(state.clone(), |ports| run(command(), ports)).unwrap_err();
    assert_eq!(error, Failure::Core(crunch_remote_core::RemoteCoreError::WrongEffectIdentity));
    assert_eq!(state.borrow().generated_attempts, 0);
}

#[test]
fn wrong_port_capability_fails_before_observation() {
    let session = crunch_remote_core::start_remote_session(command()).expect("valid command");
    let effect = session.pending_effect.as_ref().expect("credential effect");
    let error = crate::RemotePortError::new(RemoteCapability::Transport, "wrong-port", false);
    let result = port_failure(effect, error);
    assert_eq!(
        result,
        Err(Failure::Core(crunch_remote_core::RemoteCoreError::Invalid {
            code: "remote-port-capability-mismatch".to_string(),
        }))
    );
    assert_eq!(effect.capability, RemoteCapability::CredentialVerification);
}

#[test]
fn malformed_observation_contract_is_rejected() {
    let session = crunch_remote_core::start_remote_session(command()).expect("valid command");
    let effect = session.pending_effect.as_ref().expect("credential effect");
    let observation = crunch_remote_core::RemoteObservation {
        schema: REMOTE_OBSERVATION_SCHEMA.to_string(),
        effect_id_blake3: effect.effect_id_blake3.clone(),
        attempt_id: None,
        fence_generation: effect.fence_generation,
        kind: RemoteObservationKind::ExecutorCompleted,
        status: RemoteObservationStatus::Failed,
        failure_class: None,
        reason_code: None,
        observed_bytes: 0,
        observed_outputs: 0,
        output_trusted: false,
    };
    let error = crunch_remote_core::apply_remote_observation(session, observation).unwrap_err();
    assert_eq!(error, crunch_remote_core::RemoteCoreError::WrongObservationKind);
    assert_eq!(error.code(), "remote-observation-kind-mismatch");
}
