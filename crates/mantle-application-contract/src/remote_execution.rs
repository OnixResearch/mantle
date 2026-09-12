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

#[cfg(test)]
mod tests {
    use super::*;

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
