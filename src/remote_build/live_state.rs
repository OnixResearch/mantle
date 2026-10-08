//! Pure, bounded projections of build and remote coordinator state for live observation.
//!
//! Normalization has no daemon, scheduler, or evidence authority. Local goal
//! snapshots come from the actual Worker registry; remote callers must supply
//! present worker sessions, never registrations as evidence of liveness.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Write;
use std::io::{self};

use serde::Deserialize;
use serde::Serialize;

use super::RemoteCoordinatorJobPhase;
use super::RemoteCoordinatorState;

pub mod daemon;
pub mod producer;

pub const LIVE_BUILD_FACT_SCHEMA: &str = "mantle-live-build-fact-v1";
pub const MAX_LIVE_FACTS: usize = 4_096;
pub const MAX_LIVE_FACT_BYTES: usize = 2_048;
pub const MAX_LIVE_ID_BYTES: usize = 256;
const FACT_ID_DOMAIN: &[u8] = b"mantle-live-build-fact-id-v1\0";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LiveGoalPhase {
    Discovered,
    Dispatched,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LiveTerminalOutcome {
    Finished,
    Lost,
    Succeeded,
    Failed,
    WorkerLost,
    Cancelled,
    NotStarted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum LiveBuildFactValue {
    WorkerPresence {
        endpoint_id: String,
        generation: u64,
    },
    Goal {
        job_id: String,
        phase: LiveGoalPhase,
        worker_endpoint_id: Option<String>,
        attempt_id: Option<String>,
        fence_generation: Option<u64>,
    },
    Reservation {
        lease_id_blake3: String,
        job_id: String,
        worker_endpoint_id: String,
        attempt_id: String,
        fence_generation: u64,
    },
    TerminalOutcome {
        job_id: String,
        attempt_id: Option<String>,
        outcome: LiveTerminalOutcome,
        terminal_phase: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LiveBuildFact {
    pub schema: &'static str,
    pub value: LiveBuildFactValue,
}

#[derive(Serialize)]
struct LiveBuildFactEnvelope<'a> {
    owner_run_id: &'a str,
    fact_id: &'a str,
    fact: &'a LiveBuildFact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveBuildSnapshot {
    owner_run_id: String,
    facts: BTreeMap<String, LiveBuildFact>,
}

impl LiveBuildSnapshot {
    pub fn empty(owner_run_id: String) -> Result<Self, LiveBuildFactError> {
        admit_identity(&owner_run_id)?;
        Ok(Self {
            owner_run_id,
            facts: BTreeMap::new(),
        })
    }

    pub fn owner_run_id(&self) -> &str {
        &self.owner_run_id
    }

    pub fn facts(&self) -> &BTreeMap<String, LiveBuildFact> {
        &self.facts
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiveBuildEmitAction<'a> {
    Publish { fact_id: &'a str, fact: &'a LiveBuildFact },
    Retract { fact_id: &'a str },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiveBuildFactError {
    InvalidIdentity,
    TooManyFacts,
    FactTooLarge,
    UnknownWorker,
    MismatchedWorker,
    MismatchedJob,
    MissingAttempt,
    MismatchedReservation,
    DuplicateFact,
    MismatchedOwner,
}

/// An evaluated root is a discovered goal, not evidence that its derivation
/// was scheduled or its output was admitted.
pub struct LiveStreamFactProjector {
    owner_run_id: String,
    stream_run_id: String,
    discovered: BTreeMap<String, String>,
    terminal: BTreeSet<String>,
}

pub enum LiveStreamFactTransition {
    Unchanged,
    Discovered {
        fact_id: String,
        fact: LiveBuildFact,
    },
    Terminal {
        retract: Option<String>,
        fact_id: String,
        fact: LiveBuildFact,
    },
}

impl LiveStreamFactProjector {
    pub fn new(owner_run_id: &str, stream_run_id: &str) -> Result<Self, LiveBuildFactError> {
        admit_identity(owner_run_id)?;
        admit_identity(stream_run_id)?;
        Ok(Self {
            owner_run_id: owner_run_id.to_string(),
            stream_run_id: stream_run_id.to_string(),
            discovered: BTreeMap::new(),
            terminal: BTreeSet::new(),
        })
    }

    pub fn observe(
        &mut self,
        record: &crunch_evaluation_stream_core::StreamRecordValue,
    ) -> Result<LiveStreamFactTransition, LiveBuildFactError> {
        use crunch_evaluation_stream_core::StreamRecordValue;
        use crunch_evaluation_stream_core::TerminalPhase;
        use crunch_evaluation_stream_core::TerminalState;

        match record {
            StreamRecordValue::RunStart(start) => {
                if start.run_id() != self.stream_run_id {
                    return Err(LiveBuildFactError::MismatchedOwner);
                }
                Ok(LiveStreamFactTransition::Unchanged)
            }
            StreamRecordValue::RootDiscovered(root) => {
                let root_id = root.root_id();
                admit_identity(&root_id)?;
                if self.discovered.contains_key(&root_id) || self.terminal.contains(&root_id) {
                    return Ok(LiveStreamFactTransition::Unchanged);
                }
                if self.discovered.len().saturating_add(self.terminal.len()) >= MAX_LIVE_FACTS {
                    return Err(LiveBuildFactError::TooManyFacts);
                }
                let fact = LiveBuildFact {
                    schema: LIVE_BUILD_FACT_SCHEMA,
                    value: LiveBuildFactValue::Goal {
                        job_id: root_id.clone(),
                        phase: LiveGoalPhase::Discovered,
                        worker_endpoint_id: None,
                        attempt_id: None,
                        fence_generation: None,
                    },
                };
                let fact_id = fact_identity(&self.owner_run_id, &fact.value)?;
                admit_fact_size(&self.owner_run_id, &fact_id, &fact)?;
                self.discovered.insert(root_id, fact_id.clone());
                Ok(LiveStreamFactTransition::Discovered { fact_id, fact })
            }
            StreamRecordValue::RootTerminal(outcome) => {
                let root_id = outcome.root().root_id();
                admit_identity(&root_id)?;
                if self.terminal.contains(&root_id) {
                    return Ok(LiveStreamFactTransition::Unchanged);
                }
                if self.discovered.len().saturating_add(self.terminal.len()) >= MAX_LIVE_FACTS
                    && !self.discovered.contains_key(&root_id)
                {
                    return Err(LiveBuildFactError::TooManyFacts);
                }
                let terminal_phase = match outcome.terminal_phase() {
                    TerminalPhase::Evaluation => "evaluation",
                    TerminalPhase::Conversion => "conversion",
                    TerminalPhase::Build => "build",
                    TerminalPhase::Coordination => "coordination",
                };
                let terminal_outcome = match outcome.terminal_state() {
                    TerminalState::Succeeded => LiveTerminalOutcome::Succeeded,
                    TerminalState::Failed => LiveTerminalOutcome::Failed,
                    TerminalState::WorkerLost => LiveTerminalOutcome::WorkerLost,
                    TerminalState::Cancelled => LiveTerminalOutcome::Cancelled,
                    TerminalState::NotStarted => LiveTerminalOutcome::NotStarted,
                };
                let fact = LiveBuildFact {
                    schema: LIVE_BUILD_FACT_SCHEMA,
                    value: LiveBuildFactValue::TerminalOutcome {
                        job_id: root_id.clone(),
                        attempt_id: None,
                        outcome: terminal_outcome,
                        terminal_phase: Some(terminal_phase.to_string()),
                    },
                };
                let fact_id = fact_identity(&self.owner_run_id, &fact.value)?;
                admit_fact_size(&self.owner_run_id, &fact_id, &fact)?;
                let retract = self.discovered.remove(&root_id);
                self.terminal.insert(root_id);
                Ok(LiveStreamFactTransition::Terminal { retract, fact_id, fact })
            }
            StreamRecordValue::RunSummary(summary) => {
                if summary.run_id() != self.stream_run_id {
                    return Err(LiveBuildFactError::MismatchedOwner);
                }
                Ok(LiveStreamFactTransition::Unchanged)
            }
        }
    }
}

/// Project observed in-process scheduler states. These goals have no remote
/// attempt fence or resource lease; a local dispatch must not fabricate either.
/// The endpoint identifies this build's actual `Worker` invocation, which
/// disappears with the owning publisher.
pub fn normalize_local_worker_live_facts(
    owner_run_id: &str,
    worker: &crunch_pipeline::WorkerLiveSnapshot,
) -> Result<LiveBuildSnapshot, LiveBuildFactError> {
    admit_identity(owner_run_id)?;
    if worker.goals.len() >= MAX_LIVE_FACTS {
        return Err(LiveBuildFactError::TooManyFacts);
    }
    let endpoint_id = format!("local-worker:{}", blake3::hash(owner_run_id.as_bytes()).to_hex());
    admit_identity(&endpoint_id)?;
    let mut facts = BTreeMap::new();
    insert_fact(&mut facts, owner_run_id, LiveBuildFactValue::WorkerPresence {
        endpoint_id: endpoint_id.clone(),
        generation: 1,
    })?;
    for goal in &worker.goals {
        admit_identity(&goal.drv_key)?;
        let value = match goal.state {
            crunch_build::GoalState::AwaitingDerivation
            | crunch_build::GoalState::Pending
            | crunch_build::GoalState::Waiting { .. }
            | crunch_build::GoalState::Ready => LiveBuildFactValue::Goal {
                job_id: goal.drv_key.clone(),
                phase: LiveGoalPhase::Discovered,
                worker_endpoint_id: None,
                attempt_id: None,
                fence_generation: None,
            },
            crunch_build::GoalState::Building => LiveBuildFactValue::Goal {
                job_id: goal.drv_key.clone(),
                phase: LiveGoalPhase::Dispatched,
                worker_endpoint_id: Some(endpoint_id.clone()),
                attempt_id: None,
                fence_generation: None,
            },
            crunch_build::GoalState::Done | crunch_build::GoalState::Failed => LiveBuildFactValue::TerminalOutcome {
                job_id: goal.drv_key.clone(),
                attempt_id: None,
                outcome: if goal.state == crunch_build::GoalState::Done {
                    LiveTerminalOutcome::Succeeded
                } else {
                    LiveTerminalOutcome::Failed
                },
                terminal_phase: Some("build".to_string()),
            },
        };
        insert_fact(&mut facts, owner_run_id, value)?;
    }
    Ok(LiveBuildSnapshot {
        owner_run_id: owner_run_id.to_string(),
        facts,
    })
}

/// Project currently true remote facts. `present_workers` is an explicit
/// session-lifetime observation, not the durable registration map.
pub fn normalize_remote_live_facts(
    owner_run_id: &str,
    state: &RemoteCoordinatorState,
    present_workers: &BTreeSet<String>,
) -> Result<LiveBuildSnapshot, LiveBuildFactError> {
    normalize_remote_live_facts_scoped(owner_run_id, state, present_workers, None)
}

/// Limit a build owner's facts to jobs admitted during that invocation; old
/// persisted terminal jobs belong to their former owner, not the new build.
pub fn normalize_remote_live_facts_for_jobs(
    owner_run_id: &str,
    state: &RemoteCoordinatorState,
    present_workers: &BTreeSet<String>,
    owned_jobs: &BTreeSet<super::RemoteJobId>,
) -> Result<LiveBuildSnapshot, LiveBuildFactError> {
    if owned_jobs.iter().any(|job_id| !state.jobs.contains_key(job_id)) {
        return Err(LiveBuildFactError::MismatchedJob);
    }
    normalize_remote_live_facts_scoped(owner_run_id, state, present_workers, Some(owned_jobs))
}

fn normalize_remote_live_facts_scoped(
    owner_run_id: &str,
    state: &RemoteCoordinatorState,
    present_workers: &BTreeSet<String>,
    owned_jobs: Option<&BTreeSet<super::RemoteJobId>>,
) -> Result<LiveBuildSnapshot, LiveBuildFactError> {
    admit_identity(owner_run_id)?;
    let job_count = owned_jobs.map_or(state.jobs.len(), BTreeSet::len);
    let lease_count = match owned_jobs {
        Some(owned) => state.resource_leases.values().filter(|lease| owned.contains(&lease.scope.job_id)).count(),
        None => state.resource_leases.len(),
    };
    let input_count = present_workers
        .len()
        .checked_add(job_count)
        .and_then(|count| count.checked_add(lease_count))
        .ok_or(LiveBuildFactError::TooManyFacts)?;
    if input_count > MAX_LIVE_FACTS {
        return Err(LiveBuildFactError::TooManyFacts);
    }
    let mut facts = BTreeMap::new();
    for endpoint_id in present_workers {
        admit_identity(endpoint_id)?;
        let worker = state.workers.get(endpoint_id).ok_or(LiveBuildFactError::UnknownWorker)?;
        if worker.endpoint_id != *endpoint_id || worker.worker_generation == 0 {
            return Err(LiveBuildFactError::MismatchedWorker);
        }
        insert_fact(&mut facts, owner_run_id, LiveBuildFactValue::WorkerPresence {
            endpoint_id: endpoint_id.clone(),
            generation: worker.worker_generation,
        })?;
    }
    for (job_id, job) in &state.jobs {
        if owned_jobs.is_some_and(|owned| !owned.contains(job_id)) {
            continue;
        }
        admit_identity(job_id.as_str())?;
        if job.job_id != *job_id {
            return Err(LiveBuildFactError::MismatchedJob);
        }
        let attempt = job.current_attempt.as_ref();
        if let Some(current) = attempt {
            admit_identity(current.attempt_id.as_str())?;
        }
        let value = match job.phase {
            RemoteCoordinatorJobPhase::Queued => LiveBuildFactValue::Goal {
                job_id: job_id.as_str().to_string(),
                phase: LiveGoalPhase::Discovered,
                worker_endpoint_id: None,
                attempt_id: attempt.map(|current| current.attempt_id.as_str().to_string()),
                fence_generation: attempt.map(|current| current.fence_generation.get()),
            },
            RemoteCoordinatorJobPhase::Running => {
                let worker_id = job.assigned_worker_endpoint_id.as_ref().ok_or(LiveBuildFactError::UnknownWorker)?;
                if !present_workers.contains(worker_id) {
                    continue;
                }
                let current = attempt.ok_or(LiveBuildFactError::MissingAttempt)?;
                LiveBuildFactValue::Goal {
                    job_id: job_id.as_str().to_string(),
                    phase: LiveGoalPhase::Dispatched,
                    worker_endpoint_id: Some(worker_id.clone()),
                    attempt_id: Some(current.attempt_id.as_str().to_string()),
                    fence_generation: Some(current.fence_generation.get()),
                }
            }
            RemoteCoordinatorJobPhase::Finished | RemoteCoordinatorJobPhase::Lost => {
                LiveBuildFactValue::TerminalOutcome {
                    job_id: job_id.as_str().to_string(),
                    attempt_id: attempt.map(|current| current.attempt_id.as_str().to_string()),
                    outcome: if job.phase == RemoteCoordinatorJobPhase::Finished {
                        LiveTerminalOutcome::Finished
                    } else {
                        LiveTerminalOutcome::Lost
                    },
                    terminal_phase: None,
                }
            }
        };
        insert_fact(&mut facts, owner_run_id, value)?;
    }
    for (lease_id, lease) in &state.resource_leases {
        let scope = &lease.scope;
        if owned_jobs.is_some_and(|owned| !owned.contains(&scope.job_id)) {
            continue;
        }
        if !present_workers.contains(&scope.worker_endpoint_id) {
            continue;
        }
        if lease.lease_id_blake3.len() != blake3::OUT_LEN * 2
            || !lease.lease_id_blake3.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(LiveBuildFactError::InvalidIdentity);
        }
        let job = state.jobs.get(&scope.job_id).ok_or(LiveBuildFactError::MismatchedReservation)?;
        let current = job.current_attempt.as_ref().ok_or(LiveBuildFactError::MismatchedReservation)?;
        if lease_id != &lease.lease_id_blake3
            || job.resource_lease_id_blake3.as_deref() != Some(lease_id.as_str())
            || !matches!(job.phase, RemoteCoordinatorJobPhase::Queued | RemoteCoordinatorJobPhase::Running)
            || job.assigned_worker_endpoint_id.as_deref() != Some(scope.worker_endpoint_id.as_str())
            || current.attempt_id != scope.attempt_id
            || current.fence_generation != scope.fence_generation
            || state.workers[&scope.worker_endpoint_id].worker_generation != scope.worker_generation
        {
            return Err(LiveBuildFactError::MismatchedReservation);
        }
        insert_fact(&mut facts, owner_run_id, LiveBuildFactValue::Reservation {
            lease_id_blake3: lease.lease_id_blake3.clone(),
            job_id: scope.job_id.as_str().to_string(),
            worker_endpoint_id: scope.worker_endpoint_id.clone(),
            attempt_id: scope.attempt_id.as_str().to_string(),
            fence_generation: scope.fence_generation.get(),
        })?;
    }
    Ok(LiveBuildSnapshot {
        owner_run_id: owner_run_id.to_string(),
        facts,
    })
}

/// Plan retractions before publications. Unchanged facts produce no action;
/// terminal facts remain observable while the owning build is present.
pub fn plan_remote_live_fact_changes<'a>(
    before: &'a LiveBuildSnapshot,
    after: &'a LiveBuildSnapshot,
) -> Result<Vec<LiveBuildEmitAction<'a>>, LiveBuildFactError> {
    if before.owner_run_id != after.owner_run_id {
        return Err(LiveBuildFactError::MismatchedOwner);
    }
    let mut actions = Vec::with_capacity(before.facts.len().saturating_add(after.facts.len()));
    for (fact_id, fact) in &before.facts {
        if after.facts.get(fact_id) != Some(fact) {
            actions.push(LiveBuildEmitAction::Retract { fact_id });
        }
    }
    for (fact_id, fact) in &after.facts {
        if before.facts.get(fact_id) != Some(fact) {
            actions.push(LiveBuildEmitAction::Publish { fact_id, fact });
        }
    }
    Ok(actions)
}

/// A caller can retract every fact when its owner stops. This does not provide
/// a daemon or claim that a retraction was delivered to a subscriber.
pub fn plan_remote_live_owner_stop(snapshot: &LiveBuildSnapshot) -> Vec<LiveBuildEmitAction<'_>> {
    snapshot.facts.keys().map(|fact_id| LiveBuildEmitAction::Retract { fact_id }).collect()
}

fn insert_fact(
    facts: &mut BTreeMap<String, LiveBuildFact>,
    owner_run_id: &str,
    value: LiveBuildFactValue,
) -> Result<(), LiveBuildFactError> {
    let fact = LiveBuildFact {
        schema: LIVE_BUILD_FACT_SCHEMA,
        value,
    };
    let fact_id = fact_identity(owner_run_id, &fact.value)?;
    admit_fact_size(owner_run_id, &fact_id, &fact)?;
    if facts.insert(fact_id, fact).is_some() {
        return Err(LiveBuildFactError::DuplicateFact);
    }
    Ok(())
}

fn fact_identity(owner_run_id: &str, value: &LiveBuildFactValue) -> Result<String, LiveBuildFactError> {
    let mut hash = blake3::Hasher::new();
    hash.update(FACT_ID_DOMAIN);
    hash_part(&mut hash, owner_run_id)?;
    match value {
        LiveBuildFactValue::WorkerPresence {
            endpoint_id,
            generation,
        } => {
            hash_part(&mut hash, "worker-presence")?;
            hash_part(&mut hash, endpoint_id)?;
            hash.update(&generation.to_be_bytes());
        }
        LiveBuildFactValue::Goal {
            job_id,
            attempt_id,
            fence_generation,
            ..
        } => {
            hash_part(&mut hash, "goal")?;
            hash_part(&mut hash, job_id)?;
            hash_part(&mut hash, attempt_id.as_deref().unwrap_or(""))?;
            hash.update(&fence_generation.unwrap_or(0).to_be_bytes());
        }
        LiveBuildFactValue::Reservation {
            lease_id_blake3,
            job_id,
            ..
        } => {
            hash_part(&mut hash, "reservation")?;
            hash_part(&mut hash, job_id)?;
            hash_part(&mut hash, lease_id_blake3)?;
        }
        LiveBuildFactValue::TerminalOutcome { job_id, attempt_id, .. } => {
            hash_part(&mut hash, "terminal-outcome")?;
            hash_part(&mut hash, job_id)?;
            hash_part(&mut hash, attempt_id.as_deref().unwrap_or(""))?;
        }
    }
    Ok(hash.finalize().to_hex().to_string())
}

fn hash_part(hash: &mut blake3::Hasher, part: &str) -> Result<(), LiveBuildFactError> {
    let length = u32::try_from(part.len()).map_err(|_| LiveBuildFactError::InvalidIdentity)?;
    hash.update(&length.to_be_bytes());
    hash.update(part.as_bytes());
    Ok(())
}

fn admit_identity(identity: &str) -> Result<(), LiveBuildFactError> {
    if identity.is_empty() || identity.len() > MAX_LIVE_ID_BYTES || identity.chars().any(char::is_control) {
        return Err(LiveBuildFactError::InvalidIdentity);
    }
    Ok(())
}

struct BoundedFactWriter {
    bytes: usize,
    limit: usize,
}

impl Write for BoundedFactWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .filter(|size| *size <= self.limit)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "live fact exceeds declared byte bound"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn admit_fact_size(owner_run_id: &str, fact_id: &str, fact: &LiveBuildFact) -> Result<(), LiveBuildFactError> {
    let envelope = LiveBuildFactEnvelope {
        owner_run_id,
        fact_id,
        fact,
    };
    let mut writer = BoundedFactWriter {
        bytes: 0,
        limit: MAX_LIVE_FACT_BYTES,
    };
    serde_json::to_writer(&mut writer, &envelope).map_err(|_| LiveBuildFactError::FactTooLarge)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fact_identity_is_fenced_across_worker_generations_and_build_owners() {
        let first = LiveBuildFactValue::WorkerPresence {
            endpoint_id: "builder-1".to_string(),
            generation: 1,
        };
        let next = LiveBuildFactValue::WorkerPresence {
            endpoint_id: "builder-1".to_string(),
            generation: 2,
        };
        assert_ne!(fact_identity("build-a", &first), fact_identity("build-a", &next));
        assert_ne!(fact_identity("build-a", &first), fact_identity("build-b", &first));
    }

    #[test]
    fn escaped_fields_cannot_bypass_the_serialized_fact_bound() {
        let escaped = "\"".repeat(MAX_LIVE_ID_BYTES);
        let fact = LiveBuildFact {
            schema: LIVE_BUILD_FACT_SCHEMA,
            value: LiveBuildFactValue::Reservation {
                lease_id_blake3: "a".repeat(blake3::OUT_LEN * 2),
                job_id: escaped.clone(),
                worker_endpoint_id: escaped.clone(),
                attempt_id: escaped.clone(),
                fence_generation: 1,
            },
        };
        let id = "b".repeat(blake3::OUT_LEN * 2);
        assert_eq!(admit_fact_size(&escaped, &id, &fact), Err(LiveBuildFactError::FactTooLarge));
        assert_eq!(admit_identity(&"x".repeat(MAX_LIVE_ID_BYTES + 1)), Err(LiveBuildFactError::InvalidIdentity));
    }

    #[test]
    fn local_scheduler_dispatch_retracts_only_the_changed_goal_without_invented_leases() {
        use crunch_pipeline::WorkerLiveGoal;
        use crunch_pipeline::WorkerLiveSnapshot;

        let snapshot = |alpha, beta| WorkerLiveSnapshot {
            goals: vec![
                WorkerLiveGoal {
                    drv_key: "alpha.drv".into(),
                    state: alpha,
                },
                WorkerLiveGoal {
                    drv_key: "beta.drv".into(),
                    state: beta,
                },
            ],
        };
        let discovered = normalize_local_worker_live_facts(
            "build-a",
            &snapshot(crunch_build::GoalState::Ready, crunch_build::GoalState::Ready),
        )
        .unwrap();
        let dispatched = normalize_local_worker_live_facts(
            "build-a",
            &snapshot(crunch_build::GoalState::Building, crunch_build::GoalState::Ready),
        )
        .unwrap();
        let terminal = normalize_local_worker_live_facts(
            "build-a",
            &snapshot(crunch_build::GoalState::Done, crunch_build::GoalState::Ready),
        )
        .unwrap();

        let alpha_goal = dispatched
            .facts()
            .iter()
            .find(|(_, fact)| matches!(&fact.value, LiveBuildFactValue::Goal { job_id, .. } if job_id == "alpha.drv"))
            .unwrap();
        assert!(matches!(
            &alpha_goal.1.value,
            LiveBuildFactValue::Goal {
                phase: LiveGoalPhase::Dispatched,
                worker_endpoint_id: Some(endpoint),
                attempt_id: None,
                fence_generation: None,
                ..
            } if endpoint.starts_with("local-worker:")
        ));
        assert!(
            !dispatched
                .facts()
                .values()
                .any(|fact| matches!(&fact.value, LiveBuildFactValue::Reservation { .. }))
        );
        let changes = plan_remote_live_fact_changes(&discovered, &dispatched).unwrap();
        assert_eq!(changes.len(), 2);
        assert!(matches!(changes[0], LiveBuildEmitAction::Retract { fact_id } if fact_id == alpha_goal.0));
        assert!(matches!(changes[1], LiveBuildEmitAction::Publish { fact_id, .. } if fact_id == alpha_goal.0));
        assert!(dispatched.facts().iter().any(|(id, fact)| {
            discovered.facts().get(id) == Some(fact)
                && matches!(&fact.value, LiveBuildFactValue::Goal { job_id, .. } if job_id == "beta.drv")
        }));

        let completion = plan_remote_live_fact_changes(&dispatched, &terminal).unwrap();
        assert_eq!(completion.len(), 2);
        assert!(matches!(completion[0], LiveBuildEmitAction::Retract { fact_id } if fact_id == alpha_goal.0));
        assert!(matches!(
            completion[1],
            LiveBuildEmitAction::Publish {
                fact: LiveBuildFact {
                    value: LiveBuildFactValue::TerminalOutcome {
                        job_id, outcome: LiveTerminalOutcome::Succeeded, terminal_phase: Some(phase), attempt_id: None,
                    },
                    ..
                },
                ..
            } if job_id == "alpha.drv" && phase == "build"
        ));
        let failed = normalize_local_worker_live_facts(
            "build-a",
            &snapshot(crunch_build::GoalState::Done, crunch_build::GoalState::Failed),
        )
        .unwrap();
        let failure = plan_remote_live_fact_changes(&terminal, &failed).unwrap();
        assert_eq!(failure.len(), 2);
        assert!(matches!(
            failure[0],
            LiveBuildEmitAction::Retract { fact_id }
                if matches!(&terminal.facts()[fact_id].value, LiveBuildFactValue::Goal { job_id, .. } if job_id == "beta.drv")
        ));
        assert!(matches!(
            failure[1],
            LiveBuildEmitAction::Publish {
                fact: LiveBuildFact {
                    value: LiveBuildFactValue::TerminalOutcome {
                        job_id, outcome: LiveTerminalOutcome::Failed, terminal_phase: Some(phase), attempt_id: None,
                    },
                    ..
                },
                ..
            } if job_id == "beta.drv" && phase == "build"
        ));
        assert!(failed.facts().iter().any(|(id, fact)| {
            terminal.facts().get(id) == Some(fact)
                && matches!(&fact.value, LiveBuildFactValue::TerminalOutcome { job_id, .. } if job_id == "alpha.drv")
        }));
        assert_eq!(plan_remote_live_owner_stop(&failed).len(), failed.facts().len());
    }

    #[test]
    fn local_worker_fact_limit_counts_worker_presence_with_goals() {
        use crunch_pipeline::WorkerLiveGoal;
        use crunch_pipeline::WorkerLiveSnapshot;

        let mut worker = WorkerLiveSnapshot {
            goals: (0..MAX_LIVE_FACTS - 1)
                .map(|index| WorkerLiveGoal {
                    drv_key: format!("goal-{index}.drv"),
                    state: crunch_build::GoalState::Ready,
                })
                .collect(),
        };
        let admitted = normalize_local_worker_live_facts("bounded-owner", &worker).unwrap();
        assert_eq!(admitted.facts().len(), MAX_LIVE_FACTS);
        worker.goals.push(WorkerLiveGoal {
            drv_key: "over-bound.drv".into(),
            state: crunch_build::GoalState::Ready,
        });
        assert_eq!(normalize_local_worker_live_facts("bounded-owner", &worker), Err(LiveBuildFactError::TooManyFacts));
    }

    #[test]
    fn identical_evaluation_runs_have_separate_live_owner_fact_identities() {
        use crunch_evaluation_stream_core::IdentityContext;
        use crunch_evaluation_stream_core::RootSet;
        use crunch_evaluation_stream_core::StreamRecordValue;

        let roots =
            RootSet::admit(IdentityContext::new("nickel-2".into(), "c".repeat(64), "all-roots".into()).unwrap(), vec![
                "alpha".into(),
            ])
            .unwrap();
        let run_id = roots.run_id().unwrap();
        let start = StreamRecordValue::run_start(&roots).unwrap();
        let root = StreamRecordValue::root_discovered(roots.roots().remove(0));
        let mut first = LiveStreamFactProjector::new("build-invocation-one", &run_id).unwrap();
        let mut second = LiveStreamFactProjector::new("build-invocation-two", &run_id).unwrap();
        first.observe(&start).unwrap();
        second.observe(&start).unwrap();
        let LiveStreamFactTransition::Discovered {
            fact_id: first_id,
            fact: first_fact,
        } = first.observe(&root).unwrap()
        else {
            panic!("first build did not publish discovery");
        };
        let LiveStreamFactTransition::Discovered {
            fact_id: second_id,
            fact: second_fact,
        } = second.observe(&root).unwrap()
        else {
            panic!("second build did not publish discovery");
        };
        assert_eq!(first_fact, second_fact, "NDJSON root identity and content must remain unchanged");
        assert_ne!(first_id, second_id, "separate build owners may publish the same root concurrently");
    }

    #[test]
    fn stream_root_projection_retracts_discoveries_before_worker_loss_terminal_facts() {
        use crunch_evaluation_stream_core::BoundedDiagnostic;
        use crunch_evaluation_stream_core::FailureScope;
        use crunch_evaluation_stream_core::IdentityContext;
        use crunch_evaluation_stream_core::OutcomeLedger;
        use crunch_evaluation_stream_core::RootSet;
        use crunch_evaluation_stream_core::SourceSequence;
        use crunch_evaluation_stream_core::StreamRecordValue;

        let roots =
            RootSet::admit(IdentityContext::new("nickel-2".into(), "a".repeat(64), "all-roots".into()).unwrap(), vec![
                "alpha".into(),
                "beta".into(),
            ])
            .unwrap();
        let run_id = roots.run_id().unwrap();
        let mut projector = LiveStreamFactProjector::new("first-invocation", &run_id).unwrap();
        let start = StreamRecordValue::run_start(&roots).unwrap();
        assert!(matches!(projector.observe(&start).unwrap(), LiveStreamFactTransition::Unchanged));

        let selected = roots.roots();
        let mut discovered = Vec::new();
        for root in &selected {
            let record = StreamRecordValue::root_discovered(root.clone());
            let LiveStreamFactTransition::Discovered { fact_id, fact } = projector.observe(&record).unwrap() else {
                panic!("each selected root must publish a discovered goal");
            };
            assert!(matches!(fact.value, LiveBuildFactValue::Goal {
                phase: LiveGoalPhase::Discovered,
                ..
            }));
            discovered.push(fact_id);
        }

        let started = OutcomeLedger::new(roots).start(SourceSequence::new(0).unwrap()).unwrap();
        let stopped = started
            .stop_remaining(FailureScope::CoordinatorFailure, BoundedDiagnostic::new("worker disappeared".into()))
            .unwrap()
            .into_ledger();
        for (index, outcome) in stopped.outcomes().into_iter().enumerate() {
            let record = StreamRecordValue::root_terminal(outcome);
            let LiveStreamFactTransition::Terminal { retract, fact, .. } = projector.observe(&record).unwrap() else {
                panic!("worker loss must replace the discovered goal with a terminal observation");
            };
            assert_eq!(retract.as_deref(), Some(discovered[index].as_str()));
            let expected = if index == 0 {
                LiveTerminalOutcome::WorkerLost
            } else {
                LiveTerminalOutcome::NotStarted
            };
            assert!(matches!(
                &fact.value,
                LiveBuildFactValue::TerminalOutcome { outcome, terminal_phase: Some(phase), .. }
                    if *outcome == expected && phase == "coordination"
            ));
            assert!(matches!(projector.observe(&record).unwrap(), LiveStreamFactTransition::Unchanged));
        }
    }

    #[test]
    fn stream_cancellation_retracts_started_and_undispatched_root_discoveries() {
        use crunch_evaluation_stream_core::BoundedDiagnostic;
        use crunch_evaluation_stream_core::FailureScope;
        use crunch_evaluation_stream_core::IdentityContext;
        use crunch_evaluation_stream_core::OutcomeLedger;
        use crunch_evaluation_stream_core::RootSet;
        use crunch_evaluation_stream_core::SourceSequence;
        use crunch_evaluation_stream_core::StreamRecordValue;

        let roots =
            RootSet::admit(IdentityContext::new("nickel-2".into(), "b".repeat(64), "all-roots".into()).unwrap(), vec![
                "alpha".into(),
                "beta".into(),
            ])
            .unwrap();
        let mut projector = LiveStreamFactProjector::new("cancel-invocation", &roots.run_id().unwrap()).unwrap();
        projector.observe(&StreamRecordValue::run_start(&roots).unwrap()).unwrap();
        let discovered: Vec<_> = roots
            .roots()
            .iter()
            .map(|root| {
                let LiveStreamFactTransition::Discovered { fact_id, .. } =
                    projector.observe(&StreamRecordValue::root_discovered(root.clone())).unwrap()
                else {
                    panic!("root discovery missing");
                };
                fact_id
            })
            .collect();
        let stopped = OutcomeLedger::new(roots)
            .start(SourceSequence::new(0).unwrap())
            .unwrap()
            .stop_remaining(FailureScope::Cancellation, BoundedDiagnostic::new("operator cancelled".into()))
            .unwrap()
            .into_ledger();
        for (index, outcome) in stopped.outcomes().into_iter().enumerate() {
            let LiveStreamFactTransition::Terminal { retract, fact, .. } =
                projector.observe(&StreamRecordValue::root_terminal(outcome)).unwrap()
            else {
                panic!("cancelled root terminal missing");
            };
            assert_eq!(retract.as_deref(), Some(discovered[index].as_str()));
            let expected = if index == 0 {
                LiveTerminalOutcome::Cancelled
            } else {
                LiveTerminalOutcome::NotStarted
            };
            assert!(matches!(
                fact.value,
                LiveBuildFactValue::TerminalOutcome { outcome, terminal_phase: Some(phase), .. }
                    if outcome == expected && phase == "coordination"
            ));
        }
    }
}
