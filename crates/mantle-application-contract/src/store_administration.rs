//! Store administration command family.
//!
//! Store commands share one typed request, blocker set, port, and result, so
//! the root maps CLI DTOs into this command and renders the typed result while
//! the port owns path-info, cache, and store effects.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use mantle_rust_plan_core::Blake3Digest;

use crate::envelope::ApplicationBlocker;
use crate::envelope::CapabilityError;

/// Maximum admitted selectors for one store command.
pub const MAX_STORE_SELECTORS: u32 = 1_024;

/// Store administration operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StoreOperation {
    /// Remove unreachable store entries.
    Gc,
    /// Re-hash and compare on-disk content.
    Verify,
    /// Attach store signatures.
    Sign,
    /// Import paths from a cache or bundle.
    Import,
    /// Export paths to a directory or cache.
    Export,
}

impl StoreOperation {
    /// Every store operation in canonical order.
    pub fn all() -> Vec<Self> {
        vec![Self::Gc, Self::Verify, Self::Sign, Self::Import, Self::Export]
    }

    /// Stable operation label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gc => "gc",
            Self::Verify => "verify",
            Self::Sign => "sign",
            Self::Import => "import",
            Self::Export => "export",
        }
    }

    /// Whether the operation may run without explicit selectors.
    pub fn is_selector_optional(self) -> bool {
        let optional = matches!(self, Self::Gc);
        debug_assert!(optional || Self::all().contains(&self));
        debug_assert!(!self.as_str().is_empty());
        optional
    }
}

/// One typed store administration command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreAdministrationCommand {
    /// Command root that produced this command (`store`, `cache`, ...).
    pub root: String,
    pub operation: StoreOperation,
    /// Logical store paths or selectors in caller order.
    pub selectors: Vec<String>,
    /// Whether the operator requested a dry run.
    pub dry_run: bool,
}

/// Domain blocker specific to store administration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StoreBlocker {
    /// The operation needs selectors and the request named none.
    MissingSelectors,
    /// The request exceeded the admitted selector bound.
    TooManySelectors,
    /// A selector was empty after trimming.
    EmptySelector,
    /// Domain policy rejected the request for another reason.
    Domain(ApplicationBlocker),
}

/// Typed store administration result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreAdministrationResult {
    /// Store paths examined.
    pub examined_paths: u32,
    /// Store paths changed on disk or in the database.
    pub changed_paths: u32,
    /// Store paths skipped because policy or state said so.
    pub skipped_paths: u32,
    /// Store receipt identity when the operation produced one.
    pub receipt_identity: Option<Blake3Digest>,
}

/// Terminal store administration outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreAdministrationOutcome {
    /// The operation ran and produced a result.
    Completed(StoreAdministrationResult),
    /// Domain policy rejected the request before any effect ran.
    Blocked(Vec<StoreBlocker>),
}

/// Port: execute one typed store administration command.
pub trait StoreAdministrationPort {
    fn administer(
        &mut self,
        command: &StoreAdministrationCommand,
    ) -> Result<StoreAdministrationOutcome, CapabilityError>;
}

/// Validate one store command before any port is called.
pub fn validate_store_command(command: &StoreAdministrationCommand) -> Vec<StoreBlocker> {
    let mut blockers = Vec::new();
    if command.root.is_empty() {
        blockers.push(StoreBlocker::Domain(ApplicationBlocker::new(
            "missing-command-root",
            "store",
            "a store command must name its command root",
        )));
    }
    if command.selectors.is_empty() && !command.operation.is_selector_optional() {
        blockers.push(StoreBlocker::MissingSelectors);
    }
    let is_selector_count_admissible =
        u32::try_from(command.selectors.len()).is_ok_and(|count| count <= MAX_STORE_SELECTORS);
    if !is_selector_count_admissible {
        blockers.push(StoreBlocker::TooManySelectors);
    }
    for selector in &command.selectors {
        if selector.trim().is_empty() {
            blockers.push(StoreBlocker::EmptySelector);
        }
    }
    blockers.sort();
    blockers.dedup();
    debug_assert!(blockers.len() <= command.selectors.len() + 3);
    blockers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command(operation: StoreOperation, selectors: &[&str]) -> StoreAdministrationCommand {
        StoreAdministrationCommand {
            root: String::from("store"),
            operation,
            selectors: selectors.iter().map(|selector| String::from(*selector)).collect(),
            dry_run: false,
        }
    }

    #[test]
    fn a_selector_bearing_operation_with_selectors_is_admissible() {
        let request = command(StoreOperation::Verify, &["/mantle/store/aaaaaaaa", "/mantle/store/bbbbbbbb"]);
        assert!(validate_store_command(&request).is_empty());
        assert!(!StoreOperation::Verify.is_selector_optional());
    }

    #[test]
    fn garbage_collection_may_run_without_selectors() {
        let request = command(StoreOperation::Gc, &[]);
        assert!(validate_store_command(&request).is_empty());
        assert!(StoreOperation::Gc.is_selector_optional());
    }

    #[test]
    fn a_selector_bearing_operation_without_selectors_is_rejected() {
        let request = command(StoreOperation::Sign, &[]);
        let blockers = validate_store_command(&request);
        assert_eq!(blockers, vec![StoreBlocker::MissingSelectors]);
        assert_eq!(StoreOperation::Sign.as_str(), "sign");
    }

    #[test]
    fn empty_selectors_and_missing_roots_are_rejected() {
        let mut request = command(StoreOperation::Export, &["   "]);
        request.root = String::new();
        let blockers = validate_store_command(&request);
        assert!(blockers.contains(&StoreBlocker::EmptySelector));
        assert!(blockers.iter().any(|blocker| matches!(blocker, StoreBlocker::Domain(_))));
        assert_eq!(StoreOperation::all().len(), 5);
    }
}
