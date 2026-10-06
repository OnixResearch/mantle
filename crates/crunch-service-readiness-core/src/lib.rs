#![no_std]
//! Bounded service declarations, live state assertions, and pure readiness decisions.
//!
//! A service's `started` and `ready` assertions have different subjects and coexist.
//! The caller owns fact/session lifetimes and retracts facts when an assertion ceases to
//! appear in the current snapshot. No process supervision or coordination I/O occurs here.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

pub const SCHEMA: &str = "mantle-service-readiness-v1";
pub const MAX_ASSERTION_BYTES: usize = 1_536;
pub const MAX_SERVICE_ID_BYTES: usize = 64;
pub const MAX_CUSTOM_STATE_BYTES: usize = 48;
pub const MAX_DEPENDENCIES: usize = 8;
pub const MAX_CUSTOM_STATES: usize = 8;
pub const MAX_SERVICES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RestartPolicy {
    Always,
    OnError,
    All,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceState {
    Started,
    Ready,
    Complete,
    Failed,
    Custom(String),
}

impl ServiceState {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Started => "started",
            Self::Ready => "ready",
            Self::Complete => "complete",
            Self::Failed => "failed",
            Self::Custom(value) => value,
        }
    }
}

impl Serialize for ServiceState {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ServiceState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "started" => Self::Started,
            "ready" => Self::Ready,
            "complete" => Self::Complete,
            "failed" => Self::Failed,
            _ => Self::Custom(value),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceDeclaration {
    pub service_id: String,
    pub custom_states: Vec<String>,
    pub dependencies: Vec<String>,
    pub restart_policy: RestartPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceAssertion {
    pub schema: String,
    pub service_id: String,
    pub state: ServiceState,
    pub custom_states: Vec<String>,
    pub dependencies: Vec<String>,
    pub restart_policy: RestartPolicy,
    pub blocked_by: Option<String>,
    pub coordination_state: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidIdentifier(&'static str),
    TooMany(&'static str),
    Duplicate(&'static str),
    SelfDependency,
    UnknownDependency(String),
    DependencyCycle,
    UnknownService(String),
    UnknownState(String),
    UnsupportedSchema,
    NotCoordinationState,
    AssertionTooLarge,
    DeclarationMismatch(String),
    MissingStarted(String),
    BlockedBy(String),
    InvalidBlocker,
    InvalidTransition,
    InactiveAssertion,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

fn identifier(value: &str, limit: usize, field: &'static str) -> Result<(), Error> {
    if value.len() > limit
        || !value.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
        || !value.as_bytes().iter().all(|b| b.is_ascii_alphanumeric() || matches!(*b, b'_' | b'-' | b'.'))
    {
        return Err(Error::InvalidIdentifier(field));
    }
    Ok(())
}

fn validate_declaration(service_id: &str, custom_states: &[String], dependencies: &[String]) -> Result<(), Error> {
    identifier(service_id, MAX_SERVICE_ID_BYTES, "service_id")?;
    if dependencies.len() > MAX_DEPENDENCIES {
        return Err(Error::TooMany("dependencies"));
    }
    if custom_states.len() > MAX_CUSTOM_STATES {
        return Err(Error::TooMany("custom_states"));
    }
    for (index, dependency) in dependencies.iter().enumerate() {
        identifier(dependency, MAX_SERVICE_ID_BYTES, "dependency")?;
        if dependency == service_id {
            return Err(Error::SelfDependency);
        }
        if dependencies[..index].contains(dependency) {
            return Err(Error::Duplicate("dependency"));
        }
    }
    for (index, state) in custom_states.iter().enumerate() {
        identifier(state, MAX_CUSTOM_STATE_BYTES, "custom_state")?;
        if matches!(state.as_str(), "started" | "ready" | "complete" | "failed") {
            return Err(Error::Duplicate("built-in state"));
        }
        if custom_states[..index].contains(state) {
            return Err(Error::Duplicate("custom_state"));
        }
    }
    Ok(())
}

impl ServiceDeclaration {
    pub fn validate(&self) -> Result<(), Error> {
        validate_declaration(&self.service_id, &self.custom_states, &self.dependencies)
    }
}

impl ServiceAssertion {
    pub fn declaration(&self) -> ServiceDeclaration {
        ServiceDeclaration {
            service_id: self.service_id.clone(),
            custom_states: self.custom_states.clone(),
            dependencies: self.dependencies.clone(),
            restart_policy: self.restart_policy,
        }
    }

    fn matches_declaration(&self, declaration: &ServiceDeclaration) -> bool {
        self.service_id == declaration.service_id
            && self.custom_states == declaration.custom_states
            && self.dependencies == declaration.dependencies
            && self.restart_policy == declaration.restart_policy
    }

    /// Stable identity component for a live fact: one subject per asserted state.
    pub fn subject(&self) -> String {
        assertion_subject(&self.service_id, &self.state)
    }

    /// Run after deserializing JSON, before accepting any externally supplied assertion.
    pub fn validate(&self) -> Result<(), Error> {
        if self.schema != SCHEMA {
            return Err(Error::UnsupportedSchema);
        }
        if !self.coordination_state {
            return Err(Error::NotCoordinationState);
        }
        validate_declaration(&self.service_id, &self.custom_states, &self.dependencies)?;
        if let ServiceState::Custom(value) = &self.state
            && !self.custom_states.contains(value)
        {
            return Err(Error::UnknownState(value.clone()));
        }
        if let Some(blocker) = &self.blocked_by
            && (self.state != ServiceState::Started || !self.dependencies.contains(blocker))
        {
            return Err(Error::InvalidBlocker);
        }
        // Identifiers admit no JSON escaping. This is the exact compact JSON byte
        // count for this struct's serde representation (including option/null).
        let array_bytes =
            |items: &[String]| items.iter().map(|item| item.len() + 2).sum::<usize>() + items.len().saturating_sub(1);
        let base = r#"{"schema":"","service_id":"","state":"","custom_states":[],"dependencies":[],"restart_policy":"","blocked_by":null,"coordination_state":true}"#.len();
        let policy_len = match self.restart_policy {
            RestartPolicy::Always => 6,
            RestartPolicy::OnError => 8,
            RestartPolicy::All => 3,
            RestartPolicy::Never => 5,
        };
        let bytes = base - 4
            + SCHEMA.len()
            + self.service_id.len()
            + self.state.as_str().len()
            + array_bytes(&self.custom_states)
            + array_bytes(&self.dependencies)
            + policy_len
            + self.blocked_by.as_ref().map_or(4, |blocker| blocker.len() + 2);
        if bytes > MAX_ASSERTION_BYTES {
            return Err(Error::AssertionTooLarge);
        }
        Ok(())
    }
}

fn insert_declaration(
    declarations: &mut BTreeMap<String, ServiceDeclaration>,
    assertion: &ServiceAssertion,
) -> Result<bool, Error> {
    if let Some(previous) = declarations.get(&assertion.service_id) {
        if !assertion.matches_declaration(previous) {
            return Err(Error::DeclarationMismatch(assertion.service_id.clone()));
        }
        return Ok(true);
    }
    declarations.insert(assertion.service_id.clone(), assertion.declaration());
    Ok(false)
}

pub fn assertion_subject(service_id: &str, state: &ServiceState) -> String {
    alloc::format!("service/{service_id}/{}", state.as_str())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartDecision {
    None,
    Individual,
    WholeService,
}

/// This is an individual process's exit policy; a startup exit is handled
/// separately as terminal failure by the transition evaluator.
pub fn restart_decision(policy: RestartPolicy, abnormal: bool) -> RestartDecision {
    match (policy, abnormal) {
        (RestartPolicy::Always, _) | (RestartPolicy::OnError, true) => RestartDecision::Individual,
        (RestartPolicy::All, true) => RestartDecision::WholeService,
        _ => RestartDecision::None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceEvent {
    Started,
    Ready,
    Complete,
    Failed,
    Exit { abnormal: bool },
    AssertCustom(String),
    RetractCustom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalState {
    Complete,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSnapshot {
    pub service_id: String,
    pub started: bool,
    pub ready: bool,
    pub terminal: Option<TerminalState>,
    pub blocked_by: Option<String>,
    pub custom_states: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub snapshot: ServiceSnapshot,
    /// The entire set of presently true assertions for this service. The caller
    /// retracts previously owned subjects absent from this set.
    pub assertions: Vec<ServiceAssertion>,
    pub restart: RestartDecision,
}

#[derive(Debug, Clone, Default)]
struct RuntimeState {
    started: bool,
    signalled_ready: bool,
    terminal: Option<TerminalState>,
    restart_pending: bool,
    custom_states: BTreeSet<String>,
}

// Complete satisfies dependents without started. Ready needs an active started
// assertion and transitively satisfied dependencies; stale ready facts do not.
fn active_available<'a>(
    id: &str,
    active: &BTreeMap<String, BTreeSet<String>>,
    declarations: &'a BTreeMap<String, ServiceDeclaration>,
    visiting: &mut BTreeSet<&'a str>,
) -> bool {
    let Some(states) = active.get(id) else { return false };
    if states.contains("failed") {
        return false;
    }
    if states.contains("complete") {
        return true;
    }
    let Some((canonical_id, declaration)) = declarations.get_key_value(id) else {
        return false;
    };
    if !states.contains("started") || !states.contains("ready") || !visiting.insert(canonical_id) {
        return false;
    }
    let available = declaration
        .dependencies
        .iter()
        .all(|dependency| active_available(dependency, active, declarations, visiting));
    visiting.remove(canonical_id.as_str());
    available
}

fn validate_dependency_graph(
    declarations: &BTreeMap<String, ServiceDeclaration>,
    allow_departed: bool,
) -> Result<(), Error> {
    if declarations.len() > MAX_SERVICES {
        return Err(Error::TooMany("services"));
    }
    if !allow_departed {
        for declaration in declarations.values() {
            for dependency in &declaration.dependencies {
                if !declarations.contains_key(dependency) {
                    return Err(Error::UnknownDependency(dependency.clone()));
                }
            }
        }
    }
    fn visit<'a>(
        id: &'a str,
        declarations: &'a BTreeMap<String, ServiceDeclaration>,
        visiting: &mut BTreeSet<&'a str>,
        visited: &mut BTreeSet<&'a str>,
    ) -> Result<(), Error> {
        if visited.contains(id) {
            return Ok(());
        }
        if !visiting.insert(id) {
            return Err(Error::DependencyCycle);
        }
        for dependency in &declarations[id].dependencies {
            if declarations.contains_key(dependency) {
                visit(dependency, declarations, visiting, visited)?;
            }
        }
        visiting.remove(id);
        visited.insert(id);
        Ok(())
    }
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for id in declarations.keys() {
        visit(id, declarations, &mut visiting, &mut visited)?;
    }
    Ok(())
}

pub struct ServiceGraph {
    declarations: BTreeMap<String, ServiceDeclaration>,
    runtime: BTreeMap<String, RuntimeState>,
}

impl ServiceGraph {
    pub fn new(declarations: Vec<ServiceDeclaration>) -> Result<Self, Error> {
        if declarations.len() > MAX_SERVICES {
            return Err(Error::TooMany("services"));
        }
        let mut by_id = BTreeMap::new();
        for declaration in declarations {
            declaration.validate()?;
            if by_id.insert(declaration.service_id.clone(), declaration).is_some() {
                return Err(Error::Duplicate("service_id"));
            }
        }
        validate_dependency_graph(&by_id, false)?;
        let runtime = by_id.keys().map(|id| (id.clone(), RuntimeState::default())).collect();
        Ok(Self {
            declarations: by_id,
            runtime,
        })
    }

    fn blocker(&self, id: &str) -> Result<Option<String>, Error> {
        let declaration = self.declarations.get(id).ok_or_else(|| Error::UnknownService(id.into()))?;
        Ok(declaration
            .dependencies
            .iter()
            .find(|dependency| {
                let dependency_state = &self.runtime[*dependency];
                dependency_state.terminal != Some(TerminalState::Complete)
                    && !(dependency_state.started
                        && dependency_state.signalled_ready
                        && self.blocker(dependency).expect("validated acyclic graph").is_none())
            })
            .cloned())
    }

    pub fn snapshot(&self, id: &str) -> Result<ServiceSnapshot, Error> {
        let runtime = self.runtime.get(id).ok_or_else(|| Error::UnknownService(id.into()))?;
        let blocked_by = if runtime.started && runtime.terminal.is_none() {
            self.blocker(id)?
        } else {
            None
        };
        Ok(ServiceSnapshot {
            service_id: id.into(),
            started: runtime.started,
            ready: runtime.started && runtime.signalled_ready && runtime.terminal.is_none() && blocked_by.is_none(),
            terminal: runtime.terminal,
            blocked_by,
            custom_states: runtime.custom_states.iter().cloned().collect(),
        })
    }

    pub fn assertions(&self, id: &str) -> Result<Vec<ServiceAssertion>, Error> {
        let snapshot = self.snapshot(id)?;
        let declaration = &self.declarations[id];
        let make = |state, blocked_by| ServiceAssertion {
            schema: SCHEMA.into(),
            service_id: id.into(),
            state,
            custom_states: declaration.custom_states.clone(),
            dependencies: declaration.dependencies.clone(),
            restart_policy: declaration.restart_policy,
            blocked_by,
            coordination_state: true,
        };
        let mut assertions = Vec::new();
        if snapshot.started {
            assertions.push(make(ServiceState::Started, snapshot.blocked_by));
            if snapshot.ready {
                assertions.push(make(ServiceState::Ready, None));
            }
            for custom in snapshot.custom_states {
                assertions.push(make(ServiceState::Custom(custom), None));
            }
        }
        if let Some(terminal) = snapshot.terminal {
            assertions.push(make(
                match terminal {
                    TerminalState::Complete => ServiceState::Complete,
                    TerminalState::Failed => ServiceState::Failed,
                },
                None,
            ));
        }
        Ok(assertions)
    }

    /// Apply a successful real request as `Ready`; `Started` alone never implies
    /// readiness. A dependency's loss removes ready from snapshots immediately.
    pub fn apply(&mut self, id: &str, event: ServiceEvent) -> Result<Transition, Error> {
        let declaration = self.declarations.get(id).ok_or_else(|| Error::UnknownService(id.into()))?;
        let before = self.snapshot(id)?;
        let mut restart = RestartDecision::None;
        let state = self.runtime.get_mut(id).expect("declaration has runtime");
        match event {
            ServiceEvent::Started if !state.started && (state.terminal.is_none() || state.restart_pending) => {
                state.terminal = None;
                state.restart_pending = false;
                state.started = true;
            }
            ServiceEvent::Ready if state.started && state.terminal.is_none() => {
                if let Some(blocker) = before.blocked_by {
                    return Err(Error::BlockedBy(blocker));
                }
                state.signalled_ready = true;
            }
            ServiceEvent::Complete | ServiceEvent::Failed if state.started && state.terminal.is_none() => {
                state.terminal = Some(if matches!(event, ServiceEvent::Complete) && state.signalled_ready {
                    TerminalState::Complete
                } else {
                    TerminalState::Failed
                });
                state.started = false;
                state.signalled_ready = false;
                state.custom_states.clear();
            }
            ServiceEvent::Exit { abnormal } if state.started && state.terminal.is_none() => {
                // Failure before the first successful ready request remains
                // failed, independently of the policy's restart decision.
                restart = restart_decision(declaration.restart_policy, abnormal);
                state.restart_pending = restart != RestartDecision::None;
                state.terminal = if !state.signalled_ready {
                    Some(TerminalState::Failed)
                } else if restart == RestartDecision::None {
                    Some(TerminalState::Complete)
                } else {
                    None
                };
                state.started = false;
                state.signalled_ready = false;
                state.custom_states.clear();
            }
            ServiceEvent::AssertCustom(custom) if state.started && state.terminal.is_none() => {
                if !declaration.custom_states.contains(&custom) {
                    return Err(Error::UnknownState(custom));
                }
                state.custom_states.insert(custom);
            }
            ServiceEvent::RetractCustom(custom) if state.started && state.terminal.is_none() => {
                if !state.custom_states.remove(&custom) {
                    return Err(Error::UnknownState(custom));
                }
            }
            _ => return Err(Error::InvalidTransition),
        }
        Ok(Transition {
            snapshot: self.snapshot(id)?,
            assertions: self.assertions(id)?,
            restart,
        })
    }

    /// Admission against a graph whose runtime reflects currently active facts.
    /// Reject a fabricated ready assertion or declaration that differs from the
    /// registered declaration, even if its individual payload is well-formed.
    pub fn admit(&self, assertion: &ServiceAssertion) -> Result<(), Error> {
        assertion.validate()?;
        let declaration = self
            .declarations
            .get(&assertion.service_id)
            .ok_or_else(|| Error::UnknownService(assertion.service_id.clone()))?;
        if !assertion.matches_declaration(declaration) {
            return Err(Error::DeclarationMismatch(assertion.service_id.clone()));
        }
        if !self.assertions(&assertion.service_id)?.contains(assertion) {
            return Err(Error::InactiveAssertion);
        }
        Ok(())
    }

    /// Check ingress against active assertions from ONE owner/session. Unlike
    /// `admit`, this does not require the daemon to retain an evaluator instance.
    /// The producer must first publish dependency declarations (as started or
    /// terminal facts); an absent referenced service is not silently satisfied.
    pub fn admit_active(existing: &[ServiceAssertion], incoming: &ServiceAssertion) -> Result<(), Error> {
        incoming.validate()?;
        let mut declarations = BTreeMap::new();
        let mut active = BTreeMap::<String, BTreeSet<String>>::new();
        for assertion in existing {
            assertion.validate()?;
            insert_declaration(&mut declarations, assertion)?;
            if !active.entry(assertion.service_id.clone()).or_default().insert(assertion.state.as_str().into()) {
                return Err(Error::Duplicate("active assertion"));
            }
        }
        let previously_declared = insert_declaration(&mut declarations, incoming)?;
        // An existing service remains declared when one of its dependency's
        // last facts disappears. New services must reference known services;
        // otherwise an unregistered name could masquerade as a blocker.
        if !previously_declared {
            for dependency in &incoming.dependencies {
                if !declarations.contains_key(dependency) {
                    return Err(Error::UnknownDependency(dependency.clone()));
                }
            }
        }
        validate_dependency_graph(&declarations, true)?;
        let states = active.get(&incoming.service_id);
        let has = |state: &str| states.is_some_and(|states| states.contains(state));
        match &incoming.state {
            ServiceState::Started if !has("complete") && !has("failed") => {
                let blocker = declarations[&incoming.service_id]
                    .dependencies
                    .iter()
                    .find(|dependency| !active_available(dependency, &active, &declarations, &mut BTreeSet::new()))
                    .cloned();
                if incoming.blocked_by != blocker {
                    return Err(Error::InvalidBlocker);
                }
            }
            ServiceState::Ready => {
                if !has("started") || has("failed") || has("complete") {
                    return Err(Error::MissingStarted(incoming.service_id.clone()));
                }
                for dependency in &declarations[&incoming.service_id].dependencies {
                    if !active_available(dependency, &active, &declarations, &mut BTreeSet::new()) {
                        return Err(Error::BlockedBy(dependency.clone()));
                    }
                }
                if existing.iter().any(|assertion| {
                    assertion.service_id == incoming.service_id
                        && assertion.state == ServiceState::Started
                        && assertion.blocked_by.is_some()
                }) {
                    return Err(Error::InvalidBlocker);
                }
            }
            ServiceState::Custom(_) if !has("started") || has("failed") || has("complete") => {
                return Err(Error::MissingStarted(incoming.service_id.clone()));
            }
            ServiceState::Complete | ServiceState::Failed if has("complete") || has("failed") => {
                return Err(Error::InvalidTransition);
            }
            ServiceState::Started => return Err(Error::InvalidTransition),
            _ => {}
        }
        Ok(())
    }

    /// Subjects of existing ready assertions that must be retracted after a
    /// dependency/started fact disappears. Supply the same owner's active facts.
    pub fn invalid_ready_subjects(existing: &[ServiceAssertion]) -> Result<Vec<String>, Error> {
        let mut active = BTreeMap::<String, BTreeSet<String>>::new();
        let mut declarations = BTreeMap::new();
        for assertion in existing {
            assertion.validate()?;
            insert_declaration(&mut declarations, assertion)?;
            if !active.entry(assertion.service_id.clone()).or_default().insert(assertion.state.as_str().into()) {
                return Err(Error::Duplicate("active assertion"));
            }
        }
        let mut invalid = Vec::new();
        for assertion in existing.iter().filter(|a| a.state == ServiceState::Ready) {
            let local = &active[&assertion.service_id];
            if local.contains("complete")
                || local.contains("failed")
                || !active_available(&assertion.service_id, &active, &declarations, &mut BTreeSet::new())
            {
                invalid.push(assertion.subject());
            }
        }
        Ok(invalid)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn declaration(id: &str, deps: &[&str], policy: RestartPolicy) -> ServiceDeclaration {
        ServiceDeclaration {
            service_id: id.into(),
            custom_states: vec!["healthy".into()],
            dependencies: deps.iter().map(|dep| (*dep).into()).collect(),
            restart_policy: policy,
        }
    }

    #[test]
    fn started_and_ready_are_concurrent_distinct_facts_and_dependency_loss_reblocks() {
        let mut graph = ServiceGraph::new(vec![
            declaration("database", &[], RestartPolicy::Always),
            declaration("api", &["database"], RestartPolicy::OnError),
        ])
        .unwrap();
        graph.apply("api", ServiceEvent::Started).unwrap();
        assert_eq!(graph.snapshot("api").unwrap().blocked_by.as_deref(), Some("database"));
        assert_eq!(graph.apply("api", ServiceEvent::Ready), Err(Error::BlockedBy("database".into())));
        graph.apply("database", ServiceEvent::Started).unwrap();
        graph.apply("database", ServiceEvent::Ready).unwrap();
        let transition = graph.apply("api", ServiceEvent::Ready).unwrap();
        assert_eq!(transition.assertions.len(), 2);
        let started = &transition.assertions[0];
        let ready = &transition.assertions[1];
        assert_ne!(started.subject(), ready.subject());
        assert_eq!(started.subject(), "service/api/started");
        assert_eq!(ready.subject(), "service/api/ready");
        assert!(transition.snapshot.ready);
        graph.apply("database", ServiceEvent::Exit { abnormal: true }).unwrap();
        let blocked = graph.snapshot("api").unwrap();
        assert!(!blocked.ready);
        assert_eq!(blocked.blocked_by.as_deref(), Some("database"));
        assert_eq!(graph.assertions("api").unwrap().len(), 1);
        graph.apply("database", ServiceEvent::Started).unwrap();
        graph.apply("database", ServiceEvent::Ready).unwrap();
        assert!(graph.snapshot("api").unwrap().ready);
    }

    #[test]
    fn wire_ingress_rejects_unknown_policy_state_and_mutated_declaration() {
        let base = r#"{"schema":"mantle-service-readiness-v1","service_id":"api","state":"started","custom_states":["healthy"],"dependencies":[],"restart_policy":"always","blocked_by":null,"coordination_state":true}"#;
        let assertion: ServiceAssertion = serde_json::from_str(base).unwrap();
        assertion.validate().unwrap();
        assert!(serde_json::from_str::<ServiceAssertion>(&base.replace("always", "sometimes")).is_err());
        let state: ServiceAssertion = serde_json::from_str(&base.replace("\"started\"", "\"mystery\"")).unwrap();
        assert_eq!(state.validate(), Err(Error::UnknownState("mystery".into())));
        let absent = base.replace(",\"restart_policy\":\"always\"", "");
        assert!(serde_json::from_str::<ServiceAssertion>(&absent).is_err());
        let mut graph = ServiceGraph::new(vec![declaration("api", &[], RestartPolicy::Always)]).unwrap();
        graph.apply("api", ServiceEvent::Started).unwrap();
        let mut forged = graph.assertions("api").unwrap().remove(0);
        forged.restart_policy = RestartPolicy::Never;
        assert_eq!(graph.admit(&forged), Err(Error::DeclarationMismatch("api".into())));
    }

    #[test]
    fn maximal_valid_assertion_fits_fact_state_byte_budget() {
        let deps: Vec<String> = (0..MAX_DEPENDENCIES)
            .map(|i| alloc::format!("{i}{}", "d".repeat(MAX_SERVICE_ID_BYTES - 1)))
            .collect();
        let assertion = ServiceAssertion {
            schema: SCHEMA.into(),
            service_id: "s".repeat(MAX_SERVICE_ID_BYTES),
            state: ServiceState::Started,
            custom_states: (0..MAX_CUSTOM_STATES)
                .map(|i| alloc::format!("{i}{}", "c".repeat(MAX_CUSTOM_STATE_BYTES - 1)))
                .collect(),
            dependencies: deps.clone(),
            restart_policy: RestartPolicy::OnError,
            blocked_by: Some(deps[0].clone()),
            coordination_state: true,
        };
        assertion.validate().unwrap();
        assert!(serde_json::to_string(&assertion).unwrap().len() <= MAX_ASSERTION_BYTES);
    }

    #[test]
    fn graph_rejects_duplicates_self_reference_unknown_dependency_and_cycles() {
        let one = declaration("a", &[], RestartPolicy::Always);
        assert!(matches!(ServiceGraph::new(vec![one.clone(), one]), Err(Error::Duplicate("service_id"))));
        assert_eq!(
            ServiceGraph::new(vec![declaration("a", &["a"], RestartPolicy::Always)]).err(),
            Some(Error::SelfDependency)
        );
        assert_eq!(
            ServiceGraph::new(vec![declaration("a", &["missing"], RestartPolicy::Always)]).err(),
            Some(Error::UnknownDependency("missing".into()))
        );
        assert_eq!(
            ServiceGraph::new(vec![declaration("a", &["b", "b"], RestartPolicy::Always)]).err(),
            Some(Error::Duplicate("dependency"))
        );
        assert_eq!(
            ServiceGraph::new(vec![
                declaration("a", &["b"], RestartPolicy::Always),
                declaration("b", &["a"], RestartPolicy::Always)
            ])
            .err(),
            Some(Error::DependencyCycle)
        );
    }

    #[test]
    fn exit_before_readiness_fails_even_when_normal_or_never() {
        for policy in [
            RestartPolicy::Always,
            RestartPolicy::OnError,
            RestartPolicy::All,
            RestartPolicy::Never,
        ] {
            for abnormal in [false, true] {
                let mut graph = ServiceGraph::new(vec![declaration("worker", &[], policy)]).unwrap();
                graph.apply("worker", ServiceEvent::Started).unwrap();
                let exited = graph.apply("worker", ServiceEvent::Exit { abnormal }).unwrap();
                assert_eq!(exited.restart, restart_decision(policy, abnormal));
                assert_eq!(exited.snapshot.terminal, Some(TerminalState::Failed));
                assert_eq!(exited.assertions[0].state, ServiceState::Failed);
                if exited.restart != RestartDecision::None {
                    assert!(graph.apply("worker", ServiceEvent::Started).unwrap().snapshot.started);
                } else {
                    assert_eq!(graph.apply("worker", ServiceEvent::Started), Err(Error::InvalidTransition));
                }
            }
        }
    }

    #[test]
    fn four_restart_policies_distinguish_normal_and_abnormal_after_ready() {
        let cases = [
            (RestartPolicy::Always, RestartDecision::Individual, RestartDecision::Individual),
            (RestartPolicy::OnError, RestartDecision::None, RestartDecision::Individual),
            (RestartPolicy::All, RestartDecision::None, RestartDecision::WholeService),
            (RestartPolicy::Never, RestartDecision::None, RestartDecision::None),
        ];
        for (policy, normal, abnormal) in cases {
            for (exit_abnormal, expected) in [(false, normal), (true, abnormal)] {
                let mut graph = ServiceGraph::new(vec![declaration("worker", &[], policy)]).unwrap();
                graph.apply("worker", ServiceEvent::Started).unwrap();
                graph.apply("worker", ServiceEvent::Ready).unwrap();
                let exited = graph
                    .apply("worker", ServiceEvent::Exit {
                        abnormal: exit_abnormal,
                    })
                    .unwrap();
                assert_eq!(exited.restart, expected);
                assert_eq!(
                    exited.snapshot.terminal,
                    if expected != RestartDecision::None {
                        None
                    } else {
                        Some(TerminalState::Complete)
                    }
                );
            }
        }
    }

    #[test]
    fn active_admission_rejects_forged_ready_and_retracted_dependency() {
        let mut graph = ServiceGraph::new(vec![
            declaration("db", &[], RestartPolicy::Never),
            declaration("api", &["db"], RestartPolicy::Always),
        ])
        .unwrap();
        let db = graph.apply("db", ServiceEvent::Started).unwrap().assertions.remove(0);
        let api = graph.apply("api", ServiceEvent::Started).unwrap().assertions.remove(0);
        let mut forged_ready = api.clone();
        forged_ready.state = ServiceState::Ready;
        forged_ready.blocked_by = None;
        assert_eq!(
            ServiceGraph::admit_active(&[db.clone(), api.clone()], &forged_ready),
            Err(Error::BlockedBy("db".into()))
        );
        let db_ready = graph.apply("db", ServiceEvent::Ready).unwrap().assertions.remove(1);
        let api_unblocked = graph.assertions("api").unwrap().remove(0);
        assert!(ServiceGraph::admit_active(&[db.clone(), db_ready.clone(), api_unblocked], &forged_ready).is_ok());
        assert_eq!(ServiceGraph::invalid_ready_subjects(&[db, api.clone(), forged_ready.clone()]).unwrap(), vec![
            forged_ready.subject()
        ]);
        let mut switched = forged_ready;
        switched.dependencies.clear();
        assert_eq!(ServiceGraph::admit_active(&[api], &switched), Err(Error::DeclarationMismatch("api".into())));
    }

    #[test]
    fn one_byte_dependency_is_a_valid_named_blocker() {
        let mut graph = ServiceGraph::new(vec![
            declaration("a", &[], RestartPolicy::Never),
            declaration("api", &["a"], RestartPolicy::Always),
        ])
        .unwrap();
        let blocked = graph.apply("api", ServiceEvent::Started).unwrap();
        assert_eq!(blocked.snapshot.blocked_by.as_deref(), Some("a"));
        blocked.assertions[0].validate().unwrap();
        assert_eq!(blocked.assertions[0].blocked_by.as_deref(), Some("a"));
    }

    #[test]
    fn stale_ready_cannot_satisfy_transitive_dependents() {
        let mut graph = ServiceGraph::new(vec![
            declaration("leaf", &[], RestartPolicy::Always),
            declaration("middle", &["leaf"], RestartPolicy::Always),
            declaration("top", &["middle"], RestartPolicy::Always),
        ])
        .unwrap();
        for id in ["leaf", "middle", "top"] {
            graph.apply(id, ServiceEvent::Started).unwrap();
            graph.apply(id, ServiceEvent::Ready).unwrap();
        }
        let leaf = graph.assertions("leaf").unwrap().remove(0);
        let middle = graph.assertions("middle").unwrap();
        let top = graph.assertions("top").unwrap();
        let active = [vec![leaf], middle, top.clone()].concat();
        assert_eq!(ServiceGraph::invalid_ready_subjects(&active).unwrap(), vec![
            "service/middle/ready",
            "service/top/ready"
        ],);
        assert_eq!(ServiceGraph::admit_active(&active, &top[1]), Err(Error::BlockedBy("middle".into())),);
    }

    #[test]
    fn admitted_service_can_fail_after_last_dependency_fact_disappears() {
        let mut graph = ServiceGraph::new(vec![
            declaration("db", &[], RestartPolicy::Never),
            declaration("api", &["db"], RestartPolicy::Always),
        ])
        .unwrap();
        let db = graph.apply("db", ServiceEvent::Started).unwrap().assertions.remove(0);
        let api = graph.apply("api", ServiceEvent::Started).unwrap().assertions.remove(0);
        ServiceGraph::admit_active(&[db], &api).unwrap();
        let mut failed = api.clone();
        failed.state = ServiceState::Failed;
        failed.blocked_by = None;
        assert_eq!(ServiceGraph::admit_active(&[], &api), Err(Error::UnknownDependency("db".into())),);
        ServiceGraph::admit_active(&[api], &failed).unwrap();
    }
}
