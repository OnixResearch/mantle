//! Best-effort current-state facts for the Rust/C compiler cache daemon.
//! They are never cache admission, build evidence, or a reason to fail work.

use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crunch_coordination::BestEffortPublisher;
use crunch_live_state_core::Fact;
use crunch_live_state_core::FactKind;
use crunch_service_readiness_core::RestartPolicy;
use crunch_service_readiness_core::ServiceAssertion;
use crunch_service_readiness_core::ServiceState;

const SOCKET_ENV: &str = "MANTLE_COORDINATION_SOCKET";
const SERVICE_ID: &str = "rust-cache-daemon";
const END_FLUSH: Duration = Duration::from_millis(90);
static NEXT_OWNER: AtomicU64 = AtomicU64::new(0);

pub(super) struct DaemonReadiness {
    publisher: BestEffortPublisher,
    owner: String,
    ready: bool,
}

impl DaemonReadiness {
    /// Construct only after policy checks and socket binding have succeeded.
    pub(super) fn start() -> Option<Self> {
        let socket = std::env::var_os(SOCKET_ENV)?;
        let publisher = BestEffortPublisher::new(Path::new(&socket)).ok()?;
        let sequence = NEXT_OWNER.fetch_add(1, Ordering::Relaxed);
        let observation = Self {
            publisher,
            owner: format!("mantle-rust-cache-{}-{sequence}", std::process::id()),
            ready: false,
        };
        observation.publish(ServiceState::Started);
        Some(observation)
    }

    fn publish(&self, state: ServiceState) -> bool {
        if self.publisher.is_degraded() {
            return false;
        }
        let assertion = ServiceAssertion {
            schema: crunch_service_readiness_core::SCHEMA.to_owned(),
            service_id: SERVICE_ID.to_owned(),
            state,
            custom_states: Vec::new(),
            dependencies: Vec::new(),
            restart_policy: RestartPolicy::OnError,
            blocked_by: None,
            coordination_state: true,
        };
        if assertion.validate().is_err() {
            return false;
        }
        let Ok(json) = serde_json::to_string(&assertion) else {
            return false;
        };
        let Ok(fact) = Fact::new(&self.owner, FactKind::ServiceReadiness, &assertion.subject(), &json) else {
            return false;
        };
        self.publisher.try_publish(fact).is_ok()
    }

    /// Called after the first successful admitted compiler or C cache response.
    pub(super) fn handled_request(&mut self) {
        if !self.ready && self.publish(ServiceState::Ready) {
            self.ready = true;
        }
    }

    pub(super) fn finish(self, served_without_error: bool) {
        let terminal = if served_without_error && self.ready {
            ServiceState::Complete
        } else {
            ServiceState::Failed
        };
        self.publish(terminal);
        let _ = self.publisher.finish(END_FLUSH);
    }
}
