//! Optional current-state observations. They never authorize a build, cache, or proof.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crunch_coordination::BestEffortPublisher;
use crunch_live_state_core::Fact;
use crunch_live_state_core::FactKind;
use crunch_live_state_core::fact_id;
use crunch_service_readiness_core::RestartPolicy;
use crunch_service_readiness_core::ServiceAssertion;
use crunch_service_readiness_core::ServiceDeclaration;
use crunch_service_readiness_core::ServiceEvent;
use crunch_service_readiness_core::ServiceGraph;
use crunch_service_readiness_core::ServiceState;

const SOCKET_ENV: &str = "MANTLE_COORDINATION_SOCKET";
const END_FLUSH: Duration = Duration::from_millis(90);
static NEXT_OWNER: AtomicU64 = AtomicU64::new(0);

struct Publisher {
    session: BestEffortPublisher,
    owner: String,
}

impl Publisher {
    fn connect(scope: &str) -> Option<Self> {
        let socket = std::env::var_os(SOCKET_ENV)?;
        let session = BestEffortPublisher::new(Path::new(&socket)).ok()?;
        let sequence = NEXT_OWNER.fetch_add(1, Ordering::Relaxed);
        Some(Self {
            session,
            owner: format!("mantle-{scope}-{}-{sequence}", std::process::id()),
        })
    }

    fn publish(&self, assertion: &ServiceAssertion) -> bool {
        if self.session.is_degraded() || assertion.validate().is_err() {
            return false;
        }
        let Ok(state) = serde_json::to_string(assertion) else {
            return false;
        };
        let Ok(fact) = Fact::new(&self.owner, FactKind::ServiceReadiness, &assertion.subject(), &state) else {
            return false;
        };
        self.session.try_publish(fact).is_ok()
    }

    fn retract(&self, subject: &str) {
        let id = fact_id(&self.owner, FactKind::ServiceReadiness, subject);
        let _ = self.session.try_retract(&id);
    }

    fn finish(self) {
        let _ = self.session.finish(END_FLUSH);
    }
}

/// A one-component observation. The unset/unavailable endpoint is inert.
pub(crate) struct ReadinessObserver {
    inner: Option<(Publisher, ServiceDeclaration)>,
    started: Cell<bool>,
    ready: Cell<bool>,
}

impl ReadinessObserver {
    pub(crate) fn start(service_id: &str, restart_policy: RestartPolicy, dependencies: &[&str]) -> Self {
        let inner = Publisher::connect(service_id).and_then(|publisher| {
            let declaration = ServiceDeclaration {
                service_id: service_id.to_owned(),
                custom_states: Vec::new(),
                dependencies: dependencies.iter().map(|id| (*id).to_owned()).collect(),
                restart_policy,
            };
            declaration.validate().ok().map(|()| (publisher, declaration))
        });
        Self {
            inner,
            started: Cell::new(false),
            ready: Cell::new(false),
        }
    }

    fn assertion(&self, state: ServiceState) -> Option<ServiceAssertion> {
        let (_, declaration) = self.inner.as_ref()?;
        Some(ServiceAssertion {
            schema: crunch_service_readiness_core::SCHEMA.to_owned(),
            service_id: declaration.service_id.clone(),
            state,
            custom_states: declaration.custom_states.clone(),
            dependencies: declaration.dependencies.clone(),
            restart_policy: declaration.restart_policy,
            blocked_by: None,
            coordination_state: true,
        })
    }

    pub(crate) fn started(&self) {
        if let (Some((publisher, _)), Some(assertion)) = (&self.inner, self.assertion(ServiceState::Started)) {
            self.started.set(publisher.publish(&assertion));
        }
    }

    pub(crate) fn ready(&self) {
        if self.started.get()
            && let (Some((publisher, _)), Some(assertion)) = (&self.inner, self.assertion(ServiceState::Ready))
        {
            self.ready.set(publisher.publish(&assertion));
        }
    }

    pub(crate) fn failed(&self) {
        if let Some((publisher, _)) = &self.inner {
            if self.ready.get() {
                publisher.retract(&self.assertion(ServiceState::Ready).expect("active declaration").subject());
            }
            if self.started.get() {
                // A terminal assertion supersedes started; the daemon also retires it.
                if let Some(failed) = self.assertion(ServiceState::Failed) {
                    let _ = publisher.publish(&failed);
                }
            }
        }
    }

    pub(crate) fn complete(&self) {
        if !self.ready.get() {
            self.failed();
            return;
        }
        if let (Some((publisher, _)), Some(complete)) = (&self.inner, self.assertion(ServiceState::Complete)) {
            let _ = publisher.publish(&complete);
        }
    }

    pub(crate) fn finish(self) {
        if let Some((publisher, _)) = self.inner {
            publisher.finish();
        }
    }
}

/// One session owns every stage, so output-reference dependencies are checked
/// against the same current fact set, not another process's history.
pub(crate) struct ReadinessGroup {
    publisher: Option<Publisher>,
    graph: Option<ServiceGraph>,
    service_ids: Vec<String>,
    published: BTreeMap<String, ServiceAssertion>,
}

impl ReadinessGroup {
    pub(crate) fn configured() -> bool {
        std::env::var_os(SOCKET_ENV).is_some()
    }

    pub(crate) fn start(declarations: Vec<ServiceDeclaration>) -> Self {
        let Some(publisher) = Publisher::connect("proof-stages") else {
            return Self {
                publisher: None,
                graph: None,
                service_ids: Vec::new(),
                published: BTreeMap::new(),
            };
        };
        let service_ids = declarations.iter().map(|declaration| declaration.service_id.clone()).collect();
        let graph = ServiceGraph::new(declarations).ok();
        Self {
            publisher: Some(publisher),
            graph,
            service_ids,
            published: BTreeMap::new(),
        }
    }

    pub(crate) fn event(&mut self, id: &str, event: ServiceEvent) {
        let (Some(publisher), Some(graph)) = (&self.publisher, &mut self.graph) else {
            return;
        };
        if graph.apply(id, event).is_err() {
            return;
        }
        let mut desired = Vec::new();
        for id in &self.service_ids {
            let Ok(assertions) = graph.assertions(id) else { return };
            desired.extend(assertions);
        }
        let next = desired.iter().map(|assertion| (assertion.subject(), assertion.clone())).collect::<BTreeMap<_, _>>();
        // Publish a terminal state before retiring started: a departed prerequisite
        // must not turn a valid failed/complete assertion into an unknown service.
        for assertion in desired
            .iter()
            .filter(|assertion| matches!(assertion.state, ServiceState::Complete | ServiceState::Failed))
        {
            if self.published.get(&assertion.subject()) != Some(assertion) {
                let _ = publisher.publish(assertion);
            }
        }
        for subject in self.published.keys() {
            if !next.contains_key(subject) {
                publisher.retract(subject);
            }
        }
        // `ServiceGraph::assertions` orders started before ready. Keep that order
        // across the single publisher queue, including when a blocker disappears.
        for assertion in desired
            .iter()
            .filter(|assertion| !matches!(assertion.state, ServiceState::Complete | ServiceState::Failed))
        {
            if self.published.get(&assertion.subject()) != Some(assertion) {
                let _ = publisher.publish(assertion);
            }
        }
        self.published = next;
    }

    pub(crate) fn finish(self) {
        if let Some(publisher) = self.publisher {
            publisher.finish();
        }
    }
}
