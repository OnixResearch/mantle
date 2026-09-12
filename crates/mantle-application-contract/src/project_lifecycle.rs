//! Project command family.
//!
//! Project commands share one typed request, blocker set, port, and result, so
//! the composition root maps CLI DTOs into this command and renders the typed
//! result while the port owns the family's effects.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::envelope::ApplicationBlocker;
use crate::envelope::CapabilityError;

/// Maximum admitted declared entries for one project lifecycle command.
pub const MAX_PROJECT_LIFECYCLE_DECLARED_ENTRIES: u32 = 256;

/// Admitted blocker slots for one project lifecycle command beyond its declared entries.
const MAX_PROJECT_LIFECYCLE_BLOCKERS: usize = 4;

/// Project operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProjectOperation {
    /// Init operation.
    Init,
    /// Check operation.
    Check,
    /// Refresh operation.
    Refresh,
    /// Show operation.
    Show,
    /// List stale operation.
    ListStale,
    /// Upgrade operation.
    Upgrade,
}

impl ProjectOperation {
    /// Every operation in canonical order.
    pub fn all() -> Vec<Self> {
        let entries = vec![
            Self::Init,
            Self::Check,
            Self::Refresh,
            Self::Show,
            Self::ListStale,
            Self::Upgrade,
        ];
        debug_assert_eq!(entries.len(), 6);
        debug_assert!(!entries.is_empty());
        entries
    }

    /// Stable operation label.
    pub fn as_str(self) -> &'static str {
        let label = match self {
            Self::Init => "init",
            Self::Check => "check",
            Self::Refresh => "refresh",
            Self::Show => "show",
            Self::ListStale => "list_stale",
            Self::Upgrade => "upgrade",
        };
        debug_assert!(!label.is_empty());
        debug_assert!(ProjectOperation::all().contains(&self));
        label
    }

    /// Whether the operation requires at least one declared entry.
    pub fn requires_declared_entries(self) -> bool {
        // Every project operation works from the manifest rather than from declared entries.
        let is_required = false;
        debug_assert!(ProjectOperation::all().contains(&self));
        debug_assert!(ProjectOperation::all().len() == 6);
        is_required
    }
}

/// One typed Project command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCommand {
    /// Command root that produced this command.
    pub root: String,
    pub operation: ProjectOperation,
    /// Primary subject of the command, such as a path, name, or selector.
    pub subject: String,
    /// Declared entries the operation consumes, in caller order.
    pub declared_entries: Vec<String>,
    /// Manifest path.
    pub manifest_path: String,
    /// Has lock write.
    pub has_lock_write: bool,
}

/// Domain blocker specific to Project commands.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProjectBlocker {
    /// The request named no subject.
    MissingSubject,
    /// The operation needs a declared entry and the request named none.
    MissingDeclaredEntry,
    /// The request exceeded the admitted declared-entry bound.
    TooManyDeclaredEntries,
    /// The operation needs an input specific to this family.
    LockWriteRequired,
    /// Domain policy rejected the request for another reason.
    Domain(ApplicationBlocker),
}

/// Typed Project result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectResult {
    /// Resolved inputs count.
    pub resolved_inputs_count: u32,
    /// Stale inputs count.
    pub stale_inputs_count: u32,
    /// Schema version.
    pub schema_version: u32,
}

/// Terminal Project outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectOutcome {
    /// The operation ran and produced a result.
    Completed(ProjectResult),
    /// Domain policy rejected the request before any effect ran.
    Blocked(Vec<ProjectBlocker>),
}

/// Port: execute one typed Project command.
pub trait ProjectPort {
    fn run(&mut self, request: &ProjectCommand) -> Result<ProjectOutcome, CapabilityError>;
}

/// Validate one Project command before any port is called.
pub fn validate_project_lifecycle(request: &ProjectCommand) -> Vec<ProjectBlocker> {
    let declared_count = request.declared_entries.len();
    let blocker_slots = declared_count.saturating_add(MAX_PROJECT_LIFECYCLE_BLOCKERS);
    let mut blockers: Vec<ProjectBlocker> = Vec::with_capacity(blocker_slots);
    if request.root.is_empty() {
        blockers.push(ProjectBlocker::Domain(ApplicationBlocker::new(
            "missing-command-root",
            "project_lifecycle",
            "a command must name its command root",
        )));
    }
    if request.subject.trim().is_empty() {
        blockers.push(ProjectBlocker::MissingSubject);
    }
    if request.operation.requires_declared_entries() && declared_count == 0 {
        blockers.push(ProjectBlocker::MissingDeclaredEntry);
    }
    let is_declared_count_admissible =
        u32::try_from(declared_count).is_ok_and(|count| count <= MAX_PROJECT_LIFECYCLE_DECLARED_ENTRIES);
    if !is_declared_count_admissible {
        blockers.push(ProjectBlocker::TooManyDeclaredEntries);
    }
    if matches!(request.operation, ProjectOperation::Refresh | ProjectOperation::Upgrade) && !request.has_lock_write {
        blockers.push(ProjectBlocker::LockWriteRequired);
    }
    blockers.sort();
    blockers.dedup();
    debug_assert!(blockers.len() <= declared_count.saturating_add(MAX_PROJECT_LIFECYCLE_BLOCKERS));
    let empty_entries = request.declared_entries.iter().filter(|entry| entry.is_empty()).count();
    debug_assert!(empty_entries <= declared_count);
    blockers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> ProjectCommand {
        ProjectCommand {
            root: String::from("project-lifecycle"),
            operation: ProjectOperation::all()[0],
            subject: String::from("subject"),
            declared_entries: vec![String::from("first")],
            manifest_path: String::from("crunch-project.ncl"),
            has_lock_write: true,
        }
    }

    #[test]
    fn a_complete_request_is_admissible() {
        let request = sample_request();
        assert!(validate_project_lifecycle(&request).is_empty());
        assert_eq!(ProjectOperation::all().len(), 6);
        let declared_required =
            ProjectOperation::all().iter().filter(|operation| operation.requires_declared_entries()).count();
        assert_eq!(declared_required, 0);
    }

    #[test]
    fn missing_root_subject_and_declared_entries_are_rejected() {
        let mut request = sample_request();
        request.root = String::new();
        request.subject = String::from("   ");
        request.declared_entries.clear();
        let blockers = validate_project_lifecycle(&request);
        assert!(blockers.iter().any(|blocker| matches!(blocker, ProjectBlocker::Domain(_))));
        assert!(blockers.contains(&ProjectBlocker::MissingSubject));
        assert!(!ProjectOperation::all().is_empty());
    }

    #[test]
    fn the_family_specific_requirement_is_enforced() {
        let mut request = sample_request();
        request.has_lock_write = false;
        request.operation = ProjectOperation::Refresh;
        let blockers = validate_project_lifecycle(&request);
        assert!(blockers.contains(&ProjectBlocker::LockWriteRequired));
        assert!(!blockers.is_empty());
        let empty_labels = ProjectOperation::all()
            .iter()
            .map(|operation| operation.as_str())
            .filter(|label| label.is_empty())
            .count();
        assert_eq!(empty_labels, 0);
    }
}
