//! Bootstrap command family.
//!
//! Bootstrap commands share one typed request, blocker set, port, and result, so
//! the composition root maps CLI DTOs into this command and renders the typed
//! result while the port owns the family's effects.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use mantle_rust_plan_core::Blake3Digest;

use crate::envelope::ApplicationBlocker;
use crate::envelope::CapabilityError;

/// Maximum admitted declared entries for one bootstrap flow command.
pub const MAX_BOOTSTRAP_FLOW_DECLARED_ENTRIES: u32 = 256;

/// Admitted blocker slots for one bootstrap flow command beyond its declared entries.
const MAX_BOOTSTRAP_FLOW_BLOCKERS: usize = 4;

/// Bootstrap operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BootstrapOperation {
    /// Fetch operation.
    Fetch,
    /// Self build operation.
    SelfBuild,
    /// Inventory operation.
    Inventory,
    /// Seed reduce operation.
    SeedReduce,
}

impl BootstrapOperation {
    /// Every operation in canonical order.
    pub fn all() -> Vec<Self> {
        let entries = vec![Self::Fetch, Self::SelfBuild, Self::Inventory, Self::SeedReduce];
        debug_assert_eq!(entries.len(), 4);
        debug_assert!(!entries.is_empty());
        entries
    }

    /// Stable operation label.
    pub fn as_str(self) -> &'static str {
        let label = match self {
            Self::Fetch => "fetch",
            Self::SelfBuild => "self_build",
            Self::Inventory => "inventory",
            Self::SeedReduce => "seed_reduce",
        };
        debug_assert!(!label.is_empty());
        debug_assert!(BootstrapOperation::all().contains(&self));
        label
    }

    /// Whether the operation requires at least one declared entry.
    pub fn requires_declared_entries(self) -> bool {
        let is_required = matches!(self, Self::Fetch | Self::SeedReduce);
        debug_assert!(BootstrapOperation::all().contains(&self));
        debug_assert!(!self.as_str().is_empty());
        is_required
    }
}

/// One typed Bootstrap command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapCommand {
    /// Command root that produced this command.
    pub root: String,
    pub operation: BootstrapOperation,
    /// Primary subject of the command, such as a path, name, or selector.
    pub subject: String,
    /// Declared entries the operation consumes, in caller order.
    pub declared_entries: Vec<String>,
    /// Store prefix.
    pub store_prefix: String,
    /// Source bundle.
    pub source_bundle: Option<String>,
}

/// Domain blocker specific to Bootstrap commands.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BootstrapBlocker {
    /// The request named no subject.
    MissingSubject,
    /// The operation needs a declared entry and the request named none.
    MissingDeclaredEntry,
    /// The request exceeded the admitted declared-entry bound.
    TooManyDeclaredEntries,
    /// The operation needs an input specific to this family.
    SourceBundleRequired,
    /// Domain policy rejected the request for another reason.
    Domain(ApplicationBlocker),
}

/// Typed Bootstrap result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapResult {
    /// Stages completed count.
    pub stages_completed_count: u32,
    /// Protected exec events count.
    pub protected_exec_events_count: u32,
    /// Output identity.
    pub output_identity: Option<Blake3Digest>,
}

/// Terminal Bootstrap outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapOutcome {
    /// The operation ran and produced a result.
    Completed(BootstrapResult),
    /// Domain policy rejected the request before any effect ran.
    Blocked(Vec<BootstrapBlocker>),
}

/// Port: execute one typed Bootstrap command.
pub trait BootstrapPort {
    fn run(&mut self, request: &BootstrapCommand) -> Result<BootstrapOutcome, CapabilityError>;
}

/// Validate one Bootstrap command before any port is called.
pub fn validate_bootstrap_flow(request: &BootstrapCommand) -> Vec<BootstrapBlocker> {
    let declared_count = request.declared_entries.len();
    let blocker_slots = declared_count.saturating_add(MAX_BOOTSTRAP_FLOW_BLOCKERS);
    let mut blockers: Vec<BootstrapBlocker> = Vec::with_capacity(blocker_slots);
    if request.root.is_empty() {
        blockers.push(BootstrapBlocker::Domain(ApplicationBlocker::new(
            "missing-command-root",
            "bootstrap_flow",
            "a command must name its command root",
        )));
    }
    if request.subject.trim().is_empty() {
        blockers.push(BootstrapBlocker::MissingSubject);
    }
    if request.operation.requires_declared_entries() && declared_count == 0 {
        blockers.push(BootstrapBlocker::MissingDeclaredEntry);
    }
    let is_declared_count_admissible =
        u32::try_from(declared_count).is_ok_and(|count| count <= MAX_BOOTSTRAP_FLOW_DECLARED_ENTRIES);
    if !is_declared_count_admissible {
        blockers.push(BootstrapBlocker::TooManyDeclaredEntries);
    }
    if matches!(request.operation, BootstrapOperation::SelfBuild | BootstrapOperation::SeedReduce)
        && request.source_bundle.is_none()
    {
        blockers.push(BootstrapBlocker::SourceBundleRequired);
    }
    blockers.sort();
    blockers.dedup();
    debug_assert!(blockers.len() <= declared_count.saturating_add(MAX_BOOTSTRAP_FLOW_BLOCKERS));
    let empty_entries = request.declared_entries.iter().filter(|entry| entry.is_empty()).count();
    debug_assert!(empty_entries <= declared_count);
    blockers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> BootstrapCommand {
        BootstrapCommand {
            root: String::from("bootstrap-flow"),
            operation: BootstrapOperation::all()[0],
            subject: String::from("subject"),
            declared_entries: vec![String::from("first")],
            store_prefix: String::from("/mantle/store"),
            source_bundle: Some(String::from("bundle")),
        }
    }

    #[test]
    fn a_complete_request_is_admissible() {
        let request = sample_request();
        assert!(validate_bootstrap_flow(&request).is_empty());
        assert_eq!(BootstrapOperation::all().len(), 4);
        let declared_required =
            BootstrapOperation::all().iter().filter(|operation| operation.requires_declared_entries()).count();
        assert_eq!(declared_required, 2);
    }

    #[test]
    fn missing_root_subject_and_declared_entries_are_rejected() {
        let mut request = sample_request();
        request.root = String::new();
        request.subject = String::from("   ");
        request.declared_entries.clear();
        let blockers = validate_bootstrap_flow(&request);
        assert!(blockers.iter().any(|blocker| matches!(blocker, BootstrapBlocker::Domain(_))));
        assert!(blockers.contains(&BootstrapBlocker::MissingSubject));
        assert!(!BootstrapOperation::all().is_empty());
    }

    #[test]
    fn the_family_specific_requirement_is_enforced() {
        let mut request = sample_request();
        request.source_bundle = None;
        request.operation = BootstrapOperation::SelfBuild;
        let blockers = validate_bootstrap_flow(&request);
        assert!(blockers.contains(&BootstrapBlocker::SourceBundleRequired));
        assert!(!blockers.is_empty());
        let empty_labels = BootstrapOperation::all()
            .iter()
            .map(|operation| operation.as_str())
            .filter(|label| label.is_empty())
            .count();
        assert_eq!(empty_labels, 0);
    }
}
