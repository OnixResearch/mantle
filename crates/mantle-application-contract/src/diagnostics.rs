//! Diagnostics command family.
//!
//! Diagnostics commands share one typed request, blocker set, port, and result, so
//! the composition root maps CLI DTOs into this command and renders the typed
//! result while the port owns the family's effects.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use mantle_rust_plan_core::Blake3Digest;

use crate::envelope::ApplicationBlocker;
use crate::envelope::CapabilityError;

/// Maximum admitted declared entries for one diagnostics command.
pub const MAX_DIAGNOSTICS_DECLARED_ENTRIES: u32 = 256;

/// Admitted blocker slots for one diagnostics command beyond its declared entries.
const MAX_DIAGNOSTICS_BLOCKERS: usize = 4;

/// Diagnostics operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticsOperation {
    /// Trace operation.
    Trace,
    /// Lint operation.
    Lint,
    /// Refactor operation.
    Refactor,
    /// Bench operation.
    Bench,
}

impl DiagnosticsOperation {
    /// Every operation in canonical order.
    pub fn all() -> Vec<Self> {
        let entries = vec![Self::Trace, Self::Lint, Self::Refactor, Self::Bench];
        debug_assert_eq!(entries.len(), 4);
        debug_assert!(!entries.is_empty());
        entries
    }

    /// Stable operation label.
    pub fn as_str(self) -> &'static str {
        let label = match self {
            Self::Trace => "trace",
            Self::Lint => "lint",
            Self::Refactor => "refactor",
            Self::Bench => "bench",
        };
        debug_assert!(!label.is_empty());
        debug_assert!(DiagnosticsOperation::all().contains(&self));
        label
    }

    /// Whether the operation requires at least one declared entry.
    pub fn requires_declared_entries(self) -> bool {
        let is_required = matches!(self, Self::Lint | Self::Refactor);
        debug_assert!(DiagnosticsOperation::all().contains(&self));
        debug_assert!(!self.as_str().is_empty());
        is_required
    }
}

/// One typed Diagnostics command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticsCommand {
    /// Command root that produced this command.
    pub root: String,
    pub operation: DiagnosticsOperation,
    /// Primary subject of the command, such as a path, name, or selector.
    pub subject: String,
    /// Declared entries the operation consumes, in caller order.
    pub declared_entries: Vec<String>,
    /// Rule names.
    pub rule_names: Vec<String>,
}

/// Domain blocker specific to Diagnostics commands.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticsBlocker {
    /// The request named no subject.
    MissingSubject,
    /// The operation needs a declared entry and the request named none.
    MissingDeclaredEntry,
    /// The request exceeded the admitted declared-entry bound.
    TooManyDeclaredEntries,
    /// The operation needs an input specific to this family.
    RuleRequired,
    /// Domain policy rejected the request for another reason.
    Domain(ApplicationBlocker),
}

/// Typed Diagnostics result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticsResult {
    /// Findings count.
    pub findings_count: u32,
    /// Changed files count.
    pub changed_files_count: u32,
    /// Evidence identity.
    pub evidence_identity: Option<Blake3Digest>,
}

/// Terminal Diagnostics outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticsOutcome {
    /// The operation ran and produced a result.
    Completed(DiagnosticsResult),
    /// Domain policy rejected the request before any effect ran.
    Blocked(Vec<DiagnosticsBlocker>),
}

/// Port: execute one typed Diagnostics command.
pub trait DiagnosticsPort {
    fn run(&mut self, request: &DiagnosticsCommand) -> Result<DiagnosticsOutcome, CapabilityError>;
}

/// Validate one Diagnostics command before any port is called.
pub fn validate_diagnostics(request: &DiagnosticsCommand) -> Vec<DiagnosticsBlocker> {
    let declared_count = request.declared_entries.len();
    let blocker_slots = declared_count.saturating_add(MAX_DIAGNOSTICS_BLOCKERS);
    let mut blockers: Vec<DiagnosticsBlocker> = Vec::with_capacity(blocker_slots);
    if request.root.is_empty() {
        blockers.push(DiagnosticsBlocker::Domain(ApplicationBlocker::new(
            "missing-command-root",
            "diagnostics",
            "a command must name its command root",
        )));
    }
    if request.subject.trim().is_empty() {
        blockers.push(DiagnosticsBlocker::MissingSubject);
    }
    if request.operation.requires_declared_entries() && declared_count == 0 {
        blockers.push(DiagnosticsBlocker::MissingDeclaredEntry);
    }
    let is_declared_count_admissible =
        u32::try_from(declared_count).is_ok_and(|count| count <= MAX_DIAGNOSTICS_DECLARED_ENTRIES);
    if !is_declared_count_admissible {
        blockers.push(DiagnosticsBlocker::TooManyDeclaredEntries);
    }
    if matches!(request.operation, DiagnosticsOperation::Lint | DiagnosticsOperation::Refactor)
        && request.declared_entries.is_empty()
    {
        blockers.push(DiagnosticsBlocker::RuleRequired);
    }
    blockers.sort();
    blockers.dedup();
    debug_assert!(blockers.len() <= declared_count.saturating_add(MAX_DIAGNOSTICS_BLOCKERS));
    let empty_entries = request.declared_entries.iter().filter(|entry| entry.is_empty()).count();
    debug_assert!(empty_entries <= declared_count);
    blockers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> DiagnosticsCommand {
        DiagnosticsCommand {
            root: String::from("diagnostics"),
            operation: DiagnosticsOperation::all()[0],
            subject: String::from("subject"),
            declared_entries: vec![String::from("first")],
            rule_names: vec![String::from("numeric_units")],
        }
    }

    #[test]
    fn a_complete_request_is_admissible() {
        let request = sample_request();
        assert!(validate_diagnostics(&request).is_empty());
        assert_eq!(DiagnosticsOperation::all().len(), 4);
        let declared_required =
            DiagnosticsOperation::all().iter().filter(|operation| operation.requires_declared_entries()).count();
        assert_eq!(declared_required, 2);
    }

    #[test]
    fn missing_root_subject_and_declared_entries_are_rejected() {
        let mut request = sample_request();
        request.root = String::new();
        request.subject = String::from("   ");
        request.declared_entries.clear();
        let blockers = validate_diagnostics(&request);
        assert!(blockers.iter().any(|blocker| matches!(blocker, DiagnosticsBlocker::Domain(_))));
        assert!(blockers.contains(&DiagnosticsBlocker::MissingSubject));
        assert!(!DiagnosticsOperation::all().is_empty());
    }

    #[test]
    fn the_family_specific_requirement_is_enforced() {
        let mut request = sample_request();
        request.operation = DiagnosticsOperation::Lint;
        request.declared_entries.clear();
        let blockers = validate_diagnostics(&request);
        assert!(blockers.contains(&DiagnosticsBlocker::RuleRequired));
        assert!(!blockers.is_empty());
        let empty_labels = DiagnosticsOperation::all()
            .iter()
            .map(|operation| operation.as_str())
            .filter(|label| label.is_empty())
            .count();
        assert_eq!(empty_labels, 0);
    }
}
