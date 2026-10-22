//! RemoteExecution command family.
//!
//! RemoteExecution commands share one typed request, blocker set, port, and result, so
//! the composition root maps CLI DTOs into this command and renders the typed
//! result while the port owns the family's effects.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::envelope::ApplicationBlocker;
use crate::envelope::CapabilityError;

/// Maximum admitted declared entries for one remote execution command.
pub const MAX_REMOTE_EXECUTION_DECLARED_ENTRIES: u32 = 256;

/// Admitted blocker slots for one remote execution command beyond its declared entries.
const MAX_REMOTE_EXECUTION_BLOCKERS: usize = 4;

/// RemoteExecution operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RemoteExecutionOperation {
    /// Build operation.
    Build,
    /// Stage operation.
    Stage,
    /// Fetch operation.
    Fetch,
    /// Doctor operation.
    Doctor,
    /// Secret profile operation.
    SecretProfile,
}

impl RemoteExecutionOperation {
    /// Every operation in canonical order.
    pub fn all() -> Vec<Self> {
        let entries = vec![Self::Build, Self::Stage, Self::Fetch, Self::Doctor, Self::SecretProfile];
        debug_assert_eq!(entries.len(), 5);
        debug_assert!(!entries.is_empty());
        entries
    }

    /// Stable operation label.
    pub fn as_str(self) -> &'static str {
        let label = match self {
            Self::Build => "build",
            Self::Stage => "stage",
            Self::Fetch => "fetch",
            Self::Doctor => "doctor",
            Self::SecretProfile => "secret_profile",
        };
        debug_assert!(!label.is_empty());
        debug_assert!(RemoteExecutionOperation::all().contains(&self));
        label
    }

    /// Whether the operation requires at least one declared entry.
    pub fn requires_declared_entries(self) -> bool {
        let is_required = matches!(self, Self::Build | Self::Fetch);
        debug_assert!(RemoteExecutionOperation::all().contains(&self));
        debug_assert!(!self.as_str().is_empty());
        is_required
    }
}

/// One typed RemoteExecution command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteExecutionCommand {
    /// Command root that produced this command.
    pub root: String,
    pub operation: RemoteExecutionOperation,
    /// Primary subject of the command, such as a path, name, or selector.
    pub subject: String,
    /// Declared entries the operation consumes, in caller order.
    pub declared_entries: Vec<String>,
    /// Builder uri.
    pub builder_uri: String,
    /// Secret profile.
    pub secret_profile: Option<String>,
    /// Has ticket.
    pub has_ticket: bool,
}

/// Domain blocker specific to RemoteExecution commands.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RemoteExecutionBlocker {
    /// The request named no subject.
    MissingSubject,
    /// The operation needs a declared entry and the request named none.
    MissingDeclaredEntry,
    /// The request exceeded the admitted declared-entry bound.
    TooManyDeclaredEntries,
    /// The operation needs an input specific to this family.
    CredentialRequired,
    /// Domain policy rejected the request for another reason.
    Domain(ApplicationBlocker),
}

/// Typed RemoteExecution result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteExecutionResult {
    /// Transferred bytes count.
    pub transferred_bytes_count: u64,
    /// Accepted outputs count.
    pub accepted_outputs_count: u32,
    /// Trust basis.
    pub trust_basis: String,
}

/// Terminal RemoteExecution outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteExecutionOutcome {
    /// The operation ran and produced a result.
    Completed(RemoteExecutionResult),
    /// Domain policy rejected the request before any effect ran.
    Blocked(Vec<RemoteExecutionBlocker>),
}

/// Port: execute one typed RemoteExecution command.
pub trait RemoteExecutionPort {
    fn run(&mut self, request: &RemoteExecutionCommand) -> Result<RemoteExecutionOutcome, CapabilityError>;
}

/// Validate one RemoteExecution command before any port is called.
pub fn validate_remote_execution(request: &RemoteExecutionCommand) -> Vec<RemoteExecutionBlocker> {
    let declared_count = request.declared_entries.len();
    let blocker_slots = declared_count.saturating_add(MAX_REMOTE_EXECUTION_BLOCKERS);
    let mut blockers: Vec<RemoteExecutionBlocker> = Vec::with_capacity(blocker_slots);
    if request.root.is_empty() {
        blockers.push(RemoteExecutionBlocker::Domain(ApplicationBlocker::new(
            "missing-command-root",
            "remote_execution",
            "a command must name its command root",
        )));
    }
    if request.subject.trim().is_empty() {
        blockers.push(RemoteExecutionBlocker::MissingSubject);
    }
    if request.operation.requires_declared_entries() && declared_count == 0 {
        blockers.push(RemoteExecutionBlocker::MissingDeclaredEntry);
    }
    let is_declared_count_admissible =
        u32::try_from(declared_count).is_ok_and(|count| count <= MAX_REMOTE_EXECUTION_DECLARED_ENTRIES);
    if !is_declared_count_admissible {
        blockers.push(RemoteExecutionBlocker::TooManyDeclaredEntries);
    }
    if matches!(request.operation, RemoteExecutionOperation::SecretProfile) && request.secret_profile.is_none() {
        blockers.push(RemoteExecutionBlocker::CredentialRequired);
    }
    blockers.sort();
    blockers.dedup();
    debug_assert!(blockers.len() <= declared_count.saturating_add(MAX_REMOTE_EXECUTION_BLOCKERS));
    let empty_entries = request.declared_entries.iter().filter(|entry| entry.is_empty()).count();
    debug_assert!(empty_entries <= declared_count);
    blockers
}

/// Remote shell leaves not already governed by a fenced remote effect session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteOperation {
    TicketCreate,
    TicketList,
    TicketInspect,
    TicketRevoke,
    TicketMigrate,
    TicketMigratePreview,
    RotateKeys,
    Status,
    ServeFixture,
    ServerOpening,
    EndpointConfiguration,
    DebugInspect,
    DebugReplayPlan,
    DebugGc,
    ReadinessQuery,
    LiveServe,
    LiveSubscribe,
    SecretResolve,
    SecretWorker,
    UsageExecutor,
    Build,
    Replay,
}

impl RemoteOperation {
    pub const fn effect_id(self) -> &'static str {
        match self {
            Self::TicketCreate => "remote-ticket-create",
            Self::TicketList => "remote-ticket-list",
            Self::TicketInspect => "remote-ticket-inspect",
            Self::TicketRevoke => "remote-ticket-revoke",
            Self::TicketMigrate => "remote-ticket-migrate",
            Self::TicketMigratePreview => "remote-ticket-migrate-preview",
            Self::RotateKeys => "remote-key-rotation",
            Self::Status => "remote-status",
            Self::ServeFixture => "remote-serve-fixture",
            Self::ServerOpening => "remote-server-opening",
            Self::EndpointConfiguration => "remote-endpoint-configuration",
            Self::DebugInspect => "remote-debug-inspect",
            Self::DebugReplayPlan => "remote-debug-replay-plan",
            Self::ReadinessQuery => "remote-readiness-query",
            Self::DebugGc => "remote-debug-gc",
            Self::LiveServe => "remote-live-serve",
            Self::LiveSubscribe => "remote-live-subscribe",
            Self::SecretResolve => "remote-secret-resolve",
            Self::SecretWorker => "remote-secret-worker",
            Self::UsageExecutor => "remote-usage-executor",
            Self::Build => "remote-build-import",
            Self::Replay => "remote-replay-import",
        }
    }

    pub const fn effect_kind(self) -> crate::EffectKind {
        match self {
            Self::TicketList
            | Self::TicketInspect
            | Self::Status
            | Self::TicketMigratePreview
            | Self::DebugInspect
            | Self::DebugReplayPlan
            | Self::ServerOpening
            | Self::EndpointConfiguration => crate::EffectKind::ReadFiles,
            Self::ServeFixture | Self::SecretResolve | Self::SecretWorker | Self::UsageExecutor => {
                crate::EffectKind::RunProcess
            }
            Self::LiveServe | Self::LiveSubscribe | Self::ReadinessQuery => crate::EffectKind::UseNetwork,
            _ => crate::EffectKind::StoreAccess,
        }
    }
}

/// Policy retained by the caller before entering the owning host port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteOperationPlan {
    pub operation: RemoteOperation,
    pub effects: crate::EffectPlan,
}

pub fn plan_remote_operation(operation: RemoteOperation) -> Result<RemoteOperationPlan, crate::PlanError> {
    let effects = crate::plan_effects(crate::CommandFamily::RemoteExecution, &[crate::EffectSpec {
        effect_id: operation.effect_id(),
        kind: operation.effect_kind(),
        limit: crate::EffectMeasure::Calls(1),
        expected_output: crate::ExpectedOutput::None,
    }])?;
    Ok(RemoteOperationPlan { operation, effects })
}

/// Facts from real records, read-back state, key parsing, or protocol IO.
/// No variant is a handler return status or an authorization to skip execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteOperationFacts<'a> {
    Snapshot {
        source_records: usize,
        reported_records: usize,
    },
    StatusSnapshot {
        expected_endpoint: &'a str,
        observed_endpoint: &'a str,
        source_jobs: usize,
        reported_jobs: usize,
        source_workers: usize,
        reported_workers: usize,
        source_tickets: usize,
        reported_tickets: usize,
    },
    Identity {
        expected: &'a str,
        observed: &'a str,
    },
    TicketState {
        expected_sequence: u64,
        observed_sequence: u64,
        expected_records: usize,
        observed_records: usize,
        mismatched_records: usize,
    },
    Migration {
        dry_run: bool,
        executed: bool,
        missing_invalidations: usize,
        schema: u32,
        expected_schema: u32,
    },
    Readiness {
        schema: &'a str,
        expected_schema: &'a str,
        classification: &'a str,
        expected_classification: &'a str,
        evidence_eligible: bool,
        invalid_components: usize,
    },
    Bundle {
        bundle_digest: &'a str,
        replay_digest: Option<&'a str>,
    },
    Retention {
        failed_deletions: usize,
    },
    Keys {
        verifier_id: &'a str,
        signer_id: &'a str,
    },
    ServerOpening {
        expected_endpoint: &'a str,
        observed_endpoint: &'a str,
        expected_prefix: &'a str,
        observed_prefix: &'a str,
        request_id: &'a str,
        outputs: usize,
    },
    EndpointConfiguration {
        bytes: usize,
        bytes_max: u64,
        schema: &'a str,
        expected_schema: &'a str,
        bindings: usize,
        bindings_max: usize,
    },
    Socket {
        device: u64,
        inode: u64,
        shutdown_requested: bool,
    },
    Subscription {
        snapshots: u64,
        malformed_frames: u64,
    },
    Outputs {
        expected: usize,
        admitted: usize,
        unevidenced: usize,
    },
}

fn is_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub fn classify_remote_operation(
    plan: &RemoteOperationPlan,
    facts: RemoteOperationFacts<'_>,
) -> crate::ApplicationOutcome {
    use RemoteOperation as Op;
    use RemoteOperationFacts as Facts;
    let admitted = match (plan.operation, facts) {
        (
            Op::TicketList,
            Facts::Snapshot {
                source_records,
                reported_records,
            },
        ) => source_records == reported_records,
        (
            Op::Status,
            Facts::StatusSnapshot {
                expected_endpoint,
                observed_endpoint,
                source_jobs,
                reported_jobs,
                source_workers,
                reported_workers,
                source_tickets,
                reported_tickets,
            },
        ) => {
            !expected_endpoint.is_empty()
                && expected_endpoint == observed_endpoint
                && source_jobs == reported_jobs
                && source_workers == reported_workers
                && source_tickets == reported_tickets
        }
        (Op::TicketInspect, Facts::Identity { expected, observed }) => !expected.is_empty() && expected == observed,
        (
            Op::TicketCreate | Op::TicketRevoke | Op::RotateKeys,
            Facts::TicketState {
                expected_sequence,
                observed_sequence,
                expected_records,
                observed_records,
                mismatched_records,
            },
        ) => expected_sequence == observed_sequence && expected_records == observed_records && mismatched_records == 0,
        (
            Op::TicketMigrate | Op::TicketMigratePreview,
            Facts::Migration {
                dry_run,
                executed,
                missing_invalidations,
                schema,
                expected_schema,
            },
        ) => {
            dry_run == (plan.operation == Op::TicketMigratePreview)
                && executed != dry_run
                && missing_invalidations == 0
                && schema == expected_schema
        }
        (
            Op::DebugInspect,
            Facts::Bundle {
                bundle_digest,
                replay_digest: None,
            },
        ) => is_digest(bundle_digest),
        (
            Op::DebugReplayPlan,
            Facts::Bundle {
                bundle_digest,
                replay_digest: Some(replay_digest),
            },
        ) => is_digest(bundle_digest) && is_digest(replay_digest),
        (Op::DebugGc, Facts::Retention { failed_deletions }) => failed_deletions == 0,
        (Op::SecretResolve | Op::SecretWorker, Facts::Keys { verifier_id, signer_id }) => {
            !verifier_id.is_empty() && !signer_id.is_empty()
        }
        (
            Op::ServerOpening,
            Facts::ServerOpening {
                expected_endpoint,
                observed_endpoint,
                expected_prefix,
                observed_prefix,
                request_id,
                outputs,
            },
        ) => {
            !request_id.is_empty()
                && outputs > 0
                && expected_endpoint == observed_endpoint
                && expected_prefix == observed_prefix
        }
        (
            Op::EndpointConfiguration,
            Facts::EndpointConfiguration {
                bytes,
                bytes_max,
                schema,
                expected_schema,
                bindings,
                bindings_max,
            },
        ) => {
            bytes > 0
                && u64::try_from(bytes).is_ok_and(|bytes| bytes <= bytes_max)
                && schema == expected_schema
                && bindings <= bindings_max
        }
        (
            Op::ReadinessQuery,
            Facts::Readiness {
                schema,
                expected_schema,
                classification,
                expected_classification,
                evidence_eligible,
                invalid_components,
            },
        ) => {
            schema == expected_schema
                && classification == expected_classification
                && !evidence_eligible
                && invalid_components == 0
        }
        (
            Op::LiveServe,
            Facts::Socket {
                device: _,
                inode,
                shutdown_requested,
            },
        ) => inode != 0 && shutdown_requested,
        (
            Op::LiveSubscribe,
            Facts::Subscription {
                snapshots,
                malformed_frames,
            },
        ) => snapshots > 0 && malformed_frames == 0,
        (
            Op::Build | Op::Replay | Op::ServeFixture | Op::UsageExecutor,
            Facts::Outputs {
                expected,
                admitted,
                unevidenced,
            },
        ) => expected > 0 && expected == admitted && unevidenced == 0,
        _ => false,
    };
    let observation = crate::Observation {
        effect_id: crate::EffectId(String::from(plan.operation.effect_id())),
        kind: plan.operation.effect_kind(),
        status: if admitted {
            crate::ObservationStatus::Succeeded
        } else {
            crate::ObservationStatus::Failed
        },
        output: crate::EffectOutput::None,
        usage: crate::EffectMeasure::Calls(1),
        diagnostics_code: if admitted {
            None
        } else {
            Some(String::from("remote-operation-evidence-contradicted"))
        },
    };
    crate::classify_observations(&plan.effects, &[observation])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_operation_build_requires_every_declared_evidenced_output() {
        let plan = plan_remote_operation(RemoteOperation::Build).unwrap();
        assert_eq!(
            classify_remote_operation(&plan, RemoteOperationFacts::Outputs {
                expected: 2,
                admitted: 2,
                unevidenced: 0,
            }),
            crate::ApplicationOutcome::Completed
        );
        for facts in [
            RemoteOperationFacts::Outputs {
                expected: 2,
                admitted: 1,
                unevidenced: 0,
            },
            RemoteOperationFacts::Outputs {
                expected: 2,
                admitted: 2,
                unevidenced: 1,
            },
            RemoteOperationFacts::Outputs {
                expected: 0,
                admitted: 0,
                unevidenced: 0,
            },
        ] {
            assert_eq!(classify_remote_operation(&plan, facts), crate::ApplicationOutcome::Failed {
                failed_effect_count: 1
            });
        }
    }

    #[test]
    fn remote_operation_status_rejects_a_missing_active_job() {
        let plan = plan_remote_operation(RemoteOperation::Status).unwrap();
        let facts = RemoteOperationFacts::StatusSnapshot {
            expected_endpoint: "worker-a",
            observed_endpoint: "worker-a",
            source_jobs: 2,
            reported_jobs: 1,
            source_workers: 1,
            reported_workers: 1,
            source_tickets: 0,
            reported_tickets: 0,
        };
        assert_eq!(classify_remote_operation(&plan, facts), crate::ApplicationOutcome::Failed {
            failed_effect_count: 1
        });
    }

    #[test]
    fn remote_operation_rejects_facts_from_another_leaf() {
        let plan = plan_remote_operation(RemoteOperation::TicketRevoke).unwrap();
        assert_eq!(
            classify_remote_operation(&plan, RemoteOperationFacts::Snapshot {
                source_records: 1,
                reported_records: 1,
            }),
            crate::ApplicationOutcome::Failed { failed_effect_count: 1 }
        );
    }

    fn sample_request() -> RemoteExecutionCommand {
        RemoteExecutionCommand {
            root: String::from("remote-execution"),
            operation: RemoteExecutionOperation::all()[0],
            subject: String::from("subject"),
            declared_entries: vec![String::from("first")],
            builder_uri: String::from("ssh://builder"),
            secret_profile: Some(String::from("default")),
            has_ticket: true,
        }
    }

    #[test]
    fn a_complete_request_is_admissible() {
        let request = sample_request();
        assert!(validate_remote_execution(&request).is_empty());
        assert_eq!(RemoteExecutionOperation::all().len(), 5);
        let declared_required = RemoteExecutionOperation::all()
            .iter()
            .filter(|operation| operation.requires_declared_entries())
            .count();
        assert_eq!(declared_required, 2);
    }

    #[test]
    fn missing_root_subject_and_declared_entries_are_rejected() {
        let mut request = sample_request();
        request.root = String::new();
        request.subject = String::from("   ");
        request.declared_entries.clear();
        let blockers = validate_remote_execution(&request);
        assert!(blockers.iter().any(|blocker| matches!(blocker, RemoteExecutionBlocker::Domain(_))));
        assert!(blockers.contains(&RemoteExecutionBlocker::MissingSubject));
        assert!(!RemoteExecutionOperation::all().is_empty());
    }

    #[test]
    fn the_family_specific_requirement_is_enforced() {
        let mut request = sample_request();
        request.secret_profile = None;
        request.operation = RemoteExecutionOperation::SecretProfile;
        let blockers = validate_remote_execution(&request);
        assert!(blockers.contains(&RemoteExecutionBlocker::CredentialRequired));
        assert!(!blockers.is_empty());
        let empty_labels = RemoteExecutionOperation::all()
            .iter()
            .map(|operation| operation.as_str())
            .filter(|label| label.is_empty())
            .count();
        assert_eq!(empty_labels, 0);
    }
}
