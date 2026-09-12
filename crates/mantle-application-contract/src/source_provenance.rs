//! SourceProvenance command family.
//!
//! SourceProvenance commands share one typed request, blocker set, port, and result, so
//! the composition root maps CLI DTOs into this command and renders the typed
//! result while the port owns the family's effects.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use mantle_rust_plan_core::Blake3Digest;

use crate::envelope::ApplicationBlocker;
use crate::envelope::CapabilityError;

/// Maximum admitted declared entries for one source provenance command.
pub const MAX_SOURCE_PROVENANCE_DECLARED_ENTRIES: u32 = 256;

/// Admitted blocker slots for one source provenance command beyond its declared entries.
const MAX_SOURCE_PROVENANCE_BLOCKERS: usize = 4;

/// SourceProvenance operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourceProvenanceOperation {
    /// Admit operation.
    Admit,
    /// Bundle operation.
    Bundle,
    /// Hydrate operation.
    Hydrate,
    /// Attest operation.
    Attest,
    /// Export operation.
    Export,
}

impl SourceProvenanceOperation {
    /// Every operation in canonical order.
    pub fn all() -> Vec<Self> {
        let entries = vec![Self::Admit, Self::Bundle, Self::Hydrate, Self::Attest, Self::Export];
        debug_assert_eq!(entries.len(), 5);
        debug_assert!(!entries.is_empty());
        entries
    }

    /// Stable operation label.
    pub fn as_str(self) -> &'static str {
        let label = match self {
            Self::Admit => "admit",
            Self::Bundle => "bundle",
            Self::Hydrate => "hydrate",
            Self::Attest => "attest",
            Self::Export => "export",
        };
        debug_assert!(!label.is_empty());
        debug_assert!(SourceProvenanceOperation::all().contains(&self));
        label
    }

    /// Whether the operation requires at least one declared entry.
    pub fn requires_declared_entries(self) -> bool {
        let is_required = matches!(self, Self::Admit | Self::Hydrate | Self::Attest);
        debug_assert!(SourceProvenanceOperation::all().contains(&self));
        debug_assert!(!self.as_str().is_empty());
        is_required
    }
}

/// One typed SourceProvenance command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProvenanceCommand {
    /// Command root that produced this command.
    pub root: String,
    pub operation: SourceProvenanceOperation,
    /// Primary subject of the command, such as a path, name, or selector.
    pub subject: String,
    /// Declared entries the operation consumes, in caller order.
    pub declared_entries: Vec<String>,
    /// Expected digest.
    pub expected_digest: Option<Blake3Digest>,
}

/// Domain blocker specific to SourceProvenance commands.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourceProvenanceBlocker {
    /// The request named no subject.
    MissingSubject,
    /// The operation needs a declared entry and the request named none.
    MissingDeclaredEntry,
    /// The request exceeded the admitted declared-entry bound.
    TooManyDeclaredEntries,
    /// The operation needs an input specific to this family.
    DigestRequired,
    /// Domain policy rejected the request for another reason.
    Domain(ApplicationBlocker),
}

/// Typed SourceProvenance result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProvenanceResult {
    /// Admitted records count.
    pub admitted_records_count: u32,
    /// Materialized bytes count.
    pub materialized_bytes_count: u64,
    /// Bundle identity.
    pub bundle_identity: Option<Blake3Digest>,
}

/// Terminal SourceProvenance outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceProvenanceOutcome {
    /// The operation ran and produced a result.
    Completed(SourceProvenanceResult),
    /// Domain policy rejected the request before any effect ran.
    Blocked(Vec<SourceProvenanceBlocker>),
}

/// Port: execute one typed SourceProvenance command.
pub trait SourceProvenancePort {
    fn run(&mut self, request: &SourceProvenanceCommand) -> Result<SourceProvenanceOutcome, CapabilityError>;
}

/// Validate one SourceProvenance command before any port is called.
pub fn validate_source_provenance(request: &SourceProvenanceCommand) -> Vec<SourceProvenanceBlocker> {
    let declared_count = request.declared_entries.len();
    let blocker_slots = declared_count.saturating_add(MAX_SOURCE_PROVENANCE_BLOCKERS);
    let mut blockers: Vec<SourceProvenanceBlocker> = Vec::with_capacity(blocker_slots);
    if request.root.is_empty() {
        blockers.push(SourceProvenanceBlocker::Domain(ApplicationBlocker::new(
            "missing-command-root",
            "source_provenance",
            "a command must name its command root",
        )));
    }
    if request.subject.trim().is_empty() {
        blockers.push(SourceProvenanceBlocker::MissingSubject);
    }
    if request.operation.requires_declared_entries() && declared_count == 0 {
        blockers.push(SourceProvenanceBlocker::MissingDeclaredEntry);
    }
    let is_declared_count_admissible =
        u32::try_from(declared_count).is_ok_and(|count| count <= MAX_SOURCE_PROVENANCE_DECLARED_ENTRIES);
    if !is_declared_count_admissible {
        blockers.push(SourceProvenanceBlocker::TooManyDeclaredEntries);
    }
    if matches!(request.operation, SourceProvenanceOperation::Attest | SourceProvenanceOperation::Hydrate)
        && request.expected_digest.is_none()
    {
        blockers.push(SourceProvenanceBlocker::DigestRequired);
    }
    blockers.sort();
    blockers.dedup();
    debug_assert!(blockers.len() <= declared_count.saturating_add(MAX_SOURCE_PROVENANCE_BLOCKERS));
    let empty_entries = request.declared_entries.iter().filter(|entry| entry.is_empty()).count();
    debug_assert!(empty_entries <= declared_count);
    blockers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> SourceProvenanceCommand {
        SourceProvenanceCommand {
            root: String::from("source-provenance"),
            operation: SourceProvenanceOperation::all()[0],
            subject: String::from("subject"),
            declared_entries: vec![String::from("first")],
            expected_digest: Some(Blake3Digest::from_slice(&[0u8; 32])),
        }
    }

    #[test]
    fn a_complete_request_is_admissible() {
        let request = sample_request();
        assert!(validate_source_provenance(&request).is_empty());
        assert_eq!(SourceProvenanceOperation::all().len(), 5);
        let declared_required = SourceProvenanceOperation::all()
            .iter()
            .filter(|operation| operation.requires_declared_entries())
            .count();
        assert_eq!(declared_required, 3);
    }

    #[test]
    fn missing_root_subject_and_declared_entries_are_rejected() {
        let mut request = sample_request();
        request.root = String::new();
        request.subject = String::from("   ");
        request.declared_entries.clear();
        let blockers = validate_source_provenance(&request);
        assert!(blockers.iter().any(|blocker| matches!(blocker, SourceProvenanceBlocker::Domain(_))));
        assert!(blockers.contains(&SourceProvenanceBlocker::MissingSubject));
        assert!(!SourceProvenanceOperation::all().is_empty());
    }

    #[test]
    fn the_family_specific_requirement_is_enforced() {
        let mut request = sample_request();
        request.expected_digest = None;
        request.operation = SourceProvenanceOperation::Attest;
        let blockers = validate_source_provenance(&request);
        assert!(blockers.contains(&SourceProvenanceBlocker::DigestRequired));
        assert!(!blockers.is_empty());
        let empty_labels = SourceProvenanceOperation::all()
            .iter()
            .map(|operation| operation.as_str())
            .filter(|label| label.is_empty())
            .count();
        assert_eq!(empty_labels, 0);
    }
}
