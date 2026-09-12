//! Component command family.
//!
//! Component commands share one typed request, blocker set, port, and result, so
//! the composition root maps CLI DTOs into this command and renders the typed
//! result while the port owns the family's effects.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use mantle_rust_plan_core::Blake3Digest;

use crate::envelope::ApplicationBlocker;
use crate::envelope::CapabilityError;

/// Maximum admitted declared entries for one component flow command.
pub const MAX_COMPONENT_FLOW_DECLARED_ENTRIES: u32 = 256;

/// Admitted blocker slots for one component flow command beyond its declared entries.
const MAX_COMPONENT_FLOW_BLOCKERS: usize = 4;

/// Component operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentOperation {
    /// Build operation.
    Build,
    /// Bundle operation.
    Bundle,
    /// Verify operation.
    Verify,
}

impl ComponentOperation {
    /// Every operation in canonical order.
    pub fn all() -> Vec<Self> {
        let entries = vec![Self::Build, Self::Bundle, Self::Verify];
        debug_assert_eq!(entries.len(), 3);
        debug_assert!(!entries.is_empty());
        entries
    }

    /// Stable operation label.
    pub fn as_str(self) -> &'static str {
        let label = match self {
            Self::Build => "build",
            Self::Bundle => "bundle",
            Self::Verify => "verify",
        };
        debug_assert!(!label.is_empty());
        debug_assert!(ComponentOperation::all().contains(&self));
        label
    }

    /// Whether the operation requires at least one declared entry.
    pub fn requires_declared_entries(self) -> bool {
        let is_required = matches!(self, Self::Bundle | Self::Verify);
        debug_assert!(ComponentOperation::all().contains(&self));
        debug_assert!(!self.as_str().is_empty());
        is_required
    }
}

/// One typed Component command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentCommand {
    /// Command root that produced this command.
    pub root: String,
    pub operation: ComponentOperation,
    /// Primary subject of the command, such as a path, name, or selector.
    pub subject: String,
    /// Declared entries the operation consumes, in caller order.
    pub declared_entries: Vec<String>,
    /// Component path.
    pub component_path: String,
    /// Evidence dir.
    pub evidence_dir: String,
}

/// Domain blocker specific to Component commands.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentBlocker {
    /// The request named no subject.
    MissingSubject,
    /// The operation needs a declared entry and the request named none.
    MissingDeclaredEntry,
    /// The request exceeded the admitted declared-entry bound.
    TooManyDeclaredEntries,
    /// The operation needs an input specific to this family.
    EvidenceRequired,
    /// Domain policy rejected the request for another reason.
    Domain(ApplicationBlocker),
}

/// Typed Component result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentResult {
    /// Bundle members count.
    pub bundle_members_count: u32,
    /// Verified contracts count.
    pub verified_contracts_count: u32,
    /// Bundle identity.
    pub bundle_identity: Option<Blake3Digest>,
}

/// Terminal Component outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentOutcome {
    /// The operation ran and produced a result.
    Completed(ComponentResult),
    /// Domain policy rejected the request before any effect ran.
    Blocked(Vec<ComponentBlocker>),
}

/// Port: execute one typed Component command.
pub trait ComponentPort {
    fn run(&mut self, request: &ComponentCommand) -> Result<ComponentOutcome, CapabilityError>;
}

/// Validate one Component command before any port is called.
pub fn validate_component_flow(request: &ComponentCommand) -> Vec<ComponentBlocker> {
    let declared_count = request.declared_entries.len();
    let blocker_slots = declared_count.saturating_add(MAX_COMPONENT_FLOW_BLOCKERS);
    let mut blockers: Vec<ComponentBlocker> = Vec::with_capacity(blocker_slots);
    if request.root.is_empty() {
        blockers.push(ComponentBlocker::Domain(ApplicationBlocker::new(
            "missing-command-root",
            "component_flow",
            "a command must name its command root",
        )));
    }
    if request.subject.trim().is_empty() {
        blockers.push(ComponentBlocker::MissingSubject);
    }
    if request.operation.requires_declared_entries() && declared_count == 0 {
        blockers.push(ComponentBlocker::MissingDeclaredEntry);
    }
    let is_declared_count_admissible =
        u32::try_from(declared_count).is_ok_and(|count| count <= MAX_COMPONENT_FLOW_DECLARED_ENTRIES);
    if !is_declared_count_admissible {
        blockers.push(ComponentBlocker::TooManyDeclaredEntries);
    }
    if matches!(request.operation, ComponentOperation::Verify) && request.evidence_dir.trim().is_empty() {
        blockers.push(ComponentBlocker::EvidenceRequired);
    }
    blockers.sort();
    blockers.dedup();
    debug_assert!(blockers.len() <= declared_count.saturating_add(MAX_COMPONENT_FLOW_BLOCKERS));
    let empty_entries = request.declared_entries.iter().filter(|entry| entry.is_empty()).count();
    debug_assert!(empty_entries <= declared_count);
    blockers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> ComponentCommand {
        ComponentCommand {
            root: String::from("component-flow"),
            operation: ComponentOperation::all()[0],
            subject: String::from("subject"),
            declared_entries: vec![String::from("first")],
            component_path: String::from("component.wasm"),
            evidence_dir: String::from("target/evidence"),
        }
    }

    #[test]
    fn a_complete_request_is_admissible() {
        let request = sample_request();
        assert!(validate_component_flow(&request).is_empty());
        assert_eq!(ComponentOperation::all().len(), 3);
        let declared_required =
            ComponentOperation::all().iter().filter(|operation| operation.requires_declared_entries()).count();
        assert_eq!(declared_required, 2);
    }

    #[test]
    fn missing_root_subject_and_declared_entries_are_rejected() {
        let mut request = sample_request();
        request.root = String::new();
        request.subject = String::from("   ");
        request.declared_entries.clear();
        let blockers = validate_component_flow(&request);
        assert!(blockers.iter().any(|blocker| matches!(blocker, ComponentBlocker::Domain(_))));
        assert!(blockers.contains(&ComponentBlocker::MissingSubject));
        assert!(!ComponentOperation::all().is_empty());
    }

    #[test]
    fn the_family_specific_requirement_is_enforced() {
        let mut request = sample_request();
        request.evidence_dir = String::new();
        request.operation = ComponentOperation::Verify;
        let blockers = validate_component_flow(&request);
        assert!(blockers.contains(&ComponentBlocker::EvidenceRequired));
        assert!(!blockers.is_empty());
        let empty_labels = ComponentOperation::all()
            .iter()
            .map(|operation| operation.as_str())
            .filter(|label| label.is_empty())
            .count();
        assert_eq!(empty_labels, 0);
    }
}
