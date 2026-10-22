//! Store administration command family.
//!
//! Admission validates selector-bearing commands. Concrete leaf plans are chosen
//! before capabilities run; ports return owned domain data and the shell derives
//! read-result, accounting, or persisted-state facts before rendering success.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::envelope::ApplicationBlocker;

/// Maximum admitted selectors for one store command.
pub const MAX_STORE_SELECTORS: u32 = 1_024;

/// Fixed blocker slots one store command may add beyond its selectors.
const MAX_STORE_BLOCKERS: usize = 4;

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
        let is_selector_optional = matches!(self, Self::Gc);
        debug_assert!(is_selector_optional || Self::all().contains(&self));
        debug_assert!(!self.as_str().is_empty());
        is_selector_optional
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

/// Validate one store command before any port is called.
pub fn validate_store_command(command: &StoreAdministrationCommand) -> Vec<StoreBlocker> {
    let selector_count = command.selectors.len();
    let blocker_slots = selector_count.saturating_add(MAX_STORE_BLOCKERS);
    let mut blockers: Vec<StoreBlocker> = Vec::with_capacity(blocker_slots);
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
    let is_selector_count_admissible = u32::try_from(selector_count).is_ok_and(|count| count <= MAX_STORE_SELECTORS);
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
    debug_assert!(blockers.len() <= selector_count.saturating_add(MAX_STORE_BLOCKERS));
    blockers
}

/// A concrete store command leaf. Transport and stream choices are part of the
/// plan, rather than inferred from a successful handler return.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreLeaf {
    List,
    Info,
    Roots { migrate: bool },
    Usage,
    Pin,
    Unpin,
    Gc { execute: bool },
    Verify,
    Repair { execute: bool },
    Sign,
    PushSelection,
    Push,
    Pull { closure: bool, http: bool },
    CompositionPlan,
    CompositionRealize,
    ArchiveSelection,
    ArchiveExport { stdio: bool },
    ArchiveImport { nario: bool, stdio: bool },
    ArchiveList { nario: bool, stdio: bool },
}

/// Authority selected before opening stores, obtaining locks, or touching input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorePlan {
    pub leaf: StoreLeaf,
}

impl StorePlan {
    pub const fn new(leaf: StoreLeaf) -> Self {
        Self { leaf }
    }

    /// Capabilities required by this concrete leaf, including store setup and
    /// transport. No network observation is invented for directory-cache push.
    pub const fn effects(self) -> &'static [crate::EffectKind] {
        use crate::EffectKind::ReadFiles;
        use crate::EffectKind::StoreAccess;
        use crate::EffectKind::UseNetwork;
        use crate::EffectKind::WriteFiles;
        match self.leaf {
            StoreLeaf::CompositionPlan | StoreLeaf::ArchiveList { .. } => &[ReadFiles],
            StoreLeaf::List
            | StoreLeaf::Info
            | StoreLeaf::Roots { migrate: false }
            | StoreLeaf::Usage
            | StoreLeaf::Gc { execute: false }
            | StoreLeaf::Verify
            | StoreLeaf::PushSelection
            | StoreLeaf::ArchiveSelection
            | StoreLeaf::Repair { execute: false } => &[StoreAccess, ReadFiles],
            StoreLeaf::Pull { http: true, .. } => &[StoreAccess, ReadFiles, WriteFiles, UseNetwork],
            StoreLeaf::Pull { http: false, .. }
            | StoreLeaf::ArchiveImport { .. }
            | StoreLeaf::Push
            | StoreLeaf::Roots { migrate: true }
            | StoreLeaf::Pin
            | StoreLeaf::Unpin
            | StoreLeaf::Gc { execute: true }
            | StoreLeaf::Sign
            | StoreLeaf::CompositionRealize
            | StoreLeaf::ArchiveExport { .. }
            | StoreLeaf::Repair { execute: true } => &[StoreAccess, ReadFiles, WriteFiles],
        }
    }
}

/// Execution yields domain data, never an invented call count or success flag.
/// The shell owns domain errors and derives `StoreFacts` from this data/readback.
pub trait StoreExecutionPort {
    type Output;
    type Error;

    fn execute(self, plan: &StorePlan) -> impl core::future::Future<Output = Result<Self::Output, Self::Error>>;
}

/// Independently checkable facts extracted before rendering terminal success.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreFacts {
    Rows {
        rows: usize,
        invalid_rows: usize,
    },
    Roots {
        rows: usize,
        invalid_rows: usize,
        readback_matches: bool,
    },
    Usage {
        observed: u64,
        classified: Option<u64>,
    },
    Retention {
        found: bool,
        matching_after: usize,
    },
    Gc {
        dry_run: bool,
        plan_matches: bool,
        candidates: u32,
        candidate_rows: usize,
        complete: bool,
        failures: usize,
    },
    Verify {
        checked: u32,
        signatures: usize,
        mismatches: u32,
    },
    Repair {
        path_matches: bool,
        execution_requested: bool,
        required: bool,
        mutated: bool,
        readback_required: bool,
        signatures_match: bool,
    },
    Sign {
        rows: usize,
        transitions: usize,
        trusted_after: usize,
    },
    Push {
        selected: usize,
        pushed: u32,
        skipped_unsigned: u32,
        skipped_present: u32,
        rows: usize,
        invalid_rows: usize,
    },
    Pull {
        imported: u32,
        skipped: u64,
        expected_paths: Option<usize>,
        rows: usize,
        invalid_rows: usize,
        payload_bytes: u64,
        row_bytes: Option<u64>,
        readback_matches: bool,
        root_admitted: Option<bool>,
    },
    Composition {
        requested_bindings: usize,
        bindings: usize,
        requested_decisions: usize,
        decisions: usize,
        identity_matches: bool,
        readback_matches: bool,
    },
    Archive {
        reported: u32,
        rows: usize,
        invalid_rows: usize,
        payload_bytes: u64,
        row_bytes: Option<u64>,
        content_matches: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreFactFailure {
    WrongEvidence,
    InconsistentResult,
    MissingRoot,
    IncompleteGc,
    VerificationMismatch,
    ClosureRootNotAdmitted,
}

/// Whether the leaf reports the row accounting shape.
fn admits_rows_facts(leaf: StoreLeaf) -> bool {
    matches!(leaf, StoreLeaf::List | StoreLeaf::PushSelection | StoreLeaf::ArchiveSelection | StoreLeaf::Info)
}

/// Whether the leaf reports the root accounting shape.
fn admits_roots_facts(leaf: StoreLeaf) -> bool {
    matches!(leaf, StoreLeaf::Roots { .. })
}

/// Whether the leaf reports the usage accounting shape.
fn admits_usage_facts(leaf: StoreLeaf) -> bool {
    leaf == StoreLeaf::Usage
}

/// Whether the leaf reports the retention accounting shape.
fn admits_retention_facts(leaf: StoreLeaf) -> bool {
    matches!(leaf, StoreLeaf::Pin | StoreLeaf::Unpin)
}

/// Whether the leaf reports the garbage-collection accounting shape.
fn admits_gc_facts(leaf: StoreLeaf) -> bool {
    matches!(leaf, StoreLeaf::Gc { .. })
}

/// Whether the leaf reports the verification accounting shape.
fn admits_verify_facts(leaf: StoreLeaf) -> bool {
    leaf == StoreLeaf::Verify
}

/// Whether the leaf reports the repair accounting shape.
fn admits_repair_facts(leaf: StoreLeaf) -> bool {
    matches!(leaf, StoreLeaf::Repair { .. })
}

/// Whether the leaf reports the signature accounting shape.
fn admits_sign_facts(leaf: StoreLeaf) -> bool {
    leaf == StoreLeaf::Sign
}

/// Whether the leaf reports the push accounting shape.
fn admits_push_facts(leaf: StoreLeaf) -> bool {
    leaf == StoreLeaf::Push
}

/// Whether the leaf reports the pull accounting shape.
fn admits_pull_facts(leaf: StoreLeaf) -> bool {
    matches!(leaf, StoreLeaf::Pull { .. })
}

/// Whether the leaf reports the composition accounting shape.
fn admits_composition_facts(leaf: StoreLeaf) -> bool {
    matches!(leaf, StoreLeaf::CompositionPlan | StoreLeaf::CompositionRealize)
}

/// Whether the leaf reports the archive accounting shape.
fn admits_archive_facts(leaf: StoreLeaf) -> bool {
    matches!(
        leaf,
        StoreLeaf::ArchiveExport { .. } | StoreLeaf::ArchiveImport { .. } | StoreLeaf::ArchiveList { .. }
    )
}

/// Whether the reported rows account for the leaf's read.
fn is_rows_consistent(leaf: StoreLeaf, facts: StoreFacts) -> Result<bool, StoreFactFailure> {
    let StoreFacts::Rows { rows, invalid_rows } = facts else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    let is_leaf_admissible = admits_rows_facts(leaf);
    if !is_leaf_admissible {
        return Err(StoreFactFailure::WrongEvidence);
    }
    debug_assert!(is_leaf_admissible);
    debug_assert!(matches!(facts, StoreFacts::Rows { .. }));
    let is_info_complete = leaf != StoreLeaf::Info || rows > 0;
    Ok(invalid_rows == 0 && is_info_complete)
}

/// Whether the reported roots account for the leaf's read.
fn is_roots_consistent(leaf: StoreLeaf, facts: StoreFacts) -> Result<bool, StoreFactFailure> {
    let StoreFacts::Roots {
        invalid_rows,
        readback_matches,
        ..
    } = facts
    else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    let is_leaf_admissible = admits_roots_facts(leaf);
    if !is_leaf_admissible {
        return Err(StoreFactFailure::WrongEvidence);
    }
    debug_assert!(is_leaf_admissible);
    debug_assert!(matches!(facts, StoreFacts::Roots { .. }));
    Ok(invalid_rows == 0 && readback_matches)
}

/// Whether the reported usage partitions the observed bytes.
fn is_usage_consistent(leaf: StoreLeaf, facts: StoreFacts) -> Result<bool, StoreFactFailure> {
    let StoreFacts::Usage { observed, classified } = facts else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    let is_leaf_admissible = admits_usage_facts(leaf);
    if !is_leaf_admissible {
        return Err(StoreFactFailure::WrongEvidence);
    }
    debug_assert!(is_leaf_admissible);
    debug_assert!(matches!(facts, StoreFacts::Usage { .. }));
    Ok(classified == Some(observed))
}

/// Whether the reported retention matches the requested transition.
fn is_retention_consistent(leaf: StoreLeaf, facts: StoreFacts) -> Result<bool, StoreFactFailure> {
    let StoreFacts::Retention { found, matching_after } = facts else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    let is_leaf_admissible = admits_retention_facts(leaf);
    if !is_leaf_admissible {
        return Err(StoreFactFailure::WrongEvidence);
    }
    debug_assert!(is_leaf_admissible);
    debug_assert!(matches!(facts, StoreFacts::Retention { .. }));
    if !found {
        return Err(StoreFactFailure::MissingRoot);
    }
    debug_assert!(found);
    let is_pinned = leaf == StoreLeaf::Pin;
    Ok(if is_pinned {
        matching_after > 0
    } else {
        matching_after == 0
    })
}

/// Whether the reported garbage collection accounts for the requested work.
fn is_gc_consistent(leaf: StoreLeaf, facts: StoreFacts) -> Result<bool, StoreFactFailure> {
    let StoreLeaf::Gc { execute } = leaf else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    let StoreFacts::Gc {
        dry_run,
        plan_matches,
        candidates,
        candidate_rows,
        complete,
        failures,
    } = facts
    else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    debug_assert!(admits_gc_facts(leaf));
    debug_assert!(matches!(facts, StoreFacts::Gc { .. }));
    if execute && (!complete || failures > 0) {
        return Err(StoreFactFailure::IncompleteGc);
    }
    debug_assert!(!execute || (complete && failures == 0));
    let is_plan_consistent = dry_run != execute && plan_matches;
    let is_candidates_consistent = usize::try_from(candidates).ok() == Some(candidate_rows);
    Ok(is_plan_consistent && is_candidates_consistent)
}

/// Whether the reported verification accounts for every checked signature.
fn is_verify_consistent(leaf: StoreLeaf, facts: StoreFacts) -> Result<bool, StoreFactFailure> {
    let StoreFacts::Verify {
        checked,
        signatures,
        mismatches,
    } = facts
    else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    let is_leaf_admissible = admits_verify_facts(leaf);
    if !is_leaf_admissible {
        return Err(StoreFactFailure::WrongEvidence);
    }
    debug_assert!(is_leaf_admissible);
    debug_assert!(matches!(facts, StoreFacts::Verify { .. }));
    if mismatches > checked {
        return Err(StoreFactFailure::InconsistentResult);
    }
    if mismatches > 0 {
        return Err(StoreFactFailure::VerificationMismatch);
    }
    debug_assert!(mismatches <= checked);
    Ok(usize::try_from(checked).ok() == Some(signatures))
}

/// Whether the reported repair matches the requested execution mode.
fn is_repair_consistent(leaf: StoreLeaf, facts: StoreFacts) -> Result<bool, StoreFactFailure> {
    let StoreLeaf::Repair { execute } = leaf else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    let StoreFacts::Repair {
        path_matches,
        execution_requested,
        required,
        mutated,
        readback_required,
        signatures_match,
    } = facts
    else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    debug_assert!(admits_repair_facts(leaf));
    debug_assert!(matches!(facts, StoreFacts::Repair { .. }));
    let is_execution_consistent = execution_requested == execute;
    let is_mutation_consistent = mutated == (execute && required);
    let is_readback_settled = !execute || !readback_required;
    Ok(path_matches && is_execution_consistent && is_mutation_consistent && signatures_match && is_readback_settled)
}

/// Whether the reported signatures account for every transition.
fn is_sign_consistent(leaf: StoreLeaf, facts: StoreFacts) -> Result<bool, StoreFactFailure> {
    let StoreFacts::Sign {
        rows,
        transitions,
        trusted_after,
    } = facts
    else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    let is_leaf_admissible = admits_sign_facts(leaf);
    if !is_leaf_admissible {
        return Err(StoreFactFailure::WrongEvidence);
    }
    debug_assert!(is_leaf_admissible);
    debug_assert!(matches!(facts, StoreFacts::Sign { .. }));
    let is_transitions_accounted = rows == transitions;
    let is_trust_settled = rows == trusted_after;
    Ok(is_transitions_accounted && is_trust_settled)
}

/// Whether the reported push accounts for every selected path.
fn is_push_consistent(leaf: StoreLeaf, facts: StoreFacts) -> Result<bool, StoreFactFailure> {
    let StoreFacts::Push {
        selected,
        pushed,
        skipped_unsigned,
        skipped_present,
        rows,
        invalid_rows,
    } = facts
    else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    let is_leaf_admissible = admits_push_facts(leaf);
    if !is_leaf_admissible {
        return Err(StoreFactFailure::WrongEvidence);
    }
    debug_assert!(is_leaf_admissible);
    debug_assert!(matches!(facts, StoreFacts::Push { .. }));
    let accounted = u64::from(pushed)
        .saturating_add(u64::from(skipped_unsigned))
        .saturating_add(u64::from(skipped_present));
    let is_selection_accounted = u64::try_from(selected).ok() == Some(accounted);
    let is_rows_consistent = usize::try_from(pushed).ok() == Some(rows);
    Ok(is_selection_accounted && is_rows_consistent && invalid_rows == 0)
}

/// The root admission reported by a pull.
fn pull_root_admitted(facts: StoreFacts) -> Option<bool> {
    let StoreFacts::Pull { root_admitted, .. } = facts else {
        return None;
    };
    root_admitted
}

/// Whether the reported pull accounts for every selected path.
fn is_pull_accounting_consistent(facts: StoreFacts) -> bool {
    let StoreFacts::Pull {
        imported,
        skipped,
        expected_paths,
        ..
    } = facts
    else {
        return false;
    };
    let accounted = skipped.checked_add(u64::from(imported));
    let is_selection_accounted = expected_paths.is_none_or(|expected| u64::try_from(expected).ok() == accounted);
    is_selection_accounted && is_pull_row_accounting_consistent(facts)
}

/// Whether the reported pull accounts for every imported path.
fn is_pull_row_accounting_consistent(facts: StoreFacts) -> bool {
    let StoreFacts::Pull {
        imported,
        rows,
        invalid_rows,
        payload_bytes,
        row_bytes,
        readback_matches,
        ..
    } = facts
    else {
        return false;
    };
    let is_rows_consistent = usize::try_from(imported).ok() == Some(rows);
    let is_payload_accounted = row_bytes == Some(payload_bytes);
    is_rows_consistent && invalid_rows == 0 && readback_matches && is_payload_accounted
}

/// Whether the reported pull matches the requested root admission.
fn is_pull_root_consistent(root_admitted: Option<bool>, closure: bool) -> bool {
    if closure {
        root_admitted == Some(true)
    } else {
        root_admitted.is_none()
    }
}

/// Whether the reported pull accounts for the requested selection.
fn is_pull_consistent(leaf: StoreLeaf, facts: StoreFacts) -> Result<bool, StoreFactFailure> {
    let StoreLeaf::Pull { closure, .. } = leaf else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    let StoreFacts::Pull { .. } = facts else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    debug_assert!(admits_pull_facts(leaf));
    debug_assert!(matches!(facts, StoreFacts::Pull { .. }));
    let root_admitted = pull_root_admitted(facts);
    if closure && root_admitted == Some(false) {
        return Err(StoreFactFailure::ClosureRootNotAdmitted);
    }
    debug_assert!(!closure || root_admitted != Some(false));
    let is_accounting_consistent = is_pull_accounting_consistent(facts);
    let is_root_consistent = is_pull_root_consistent(root_admitted, closure);
    Ok(is_accounting_consistent && is_root_consistent)
}

/// Whether the reported composition accounts for every binding and decision.
fn is_composition_consistent(leaf: StoreLeaf, facts: StoreFacts) -> Result<bool, StoreFactFailure> {
    let StoreFacts::Composition {
        requested_bindings,
        bindings,
        requested_decisions,
        decisions,
        identity_matches,
        readback_matches,
    } = facts
    else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    let is_leaf_admissible = admits_composition_facts(leaf);
    if !is_leaf_admissible {
        return Err(StoreFactFailure::WrongEvidence);
    }
    debug_assert!(is_leaf_admissible);
    debug_assert!(matches!(facts, StoreFacts::Composition { .. }));
    let is_bindings_accounted = requested_bindings == bindings;
    let is_decisions_accounted = requested_decisions == decisions;
    Ok(is_bindings_accounted && is_decisions_accounted && identity_matches && readback_matches)
}

/// Whether the reported archive accounts for every recorded path.
fn is_archive_consistent(leaf: StoreLeaf, facts: StoreFacts) -> Result<bool, StoreFactFailure> {
    let StoreFacts::Archive {
        reported,
        rows,
        invalid_rows,
        payload_bytes,
        row_bytes,
        content_matches,
    } = facts
    else {
        return Err(StoreFactFailure::WrongEvidence);
    };
    let is_leaf_admissible = admits_archive_facts(leaf);
    if !is_leaf_admissible {
        return Err(StoreFactFailure::WrongEvidence);
    }
    debug_assert!(is_leaf_admissible);
    debug_assert!(matches!(facts, StoreFacts::Archive { .. }));
    let is_rows_consistent = usize::try_from(reported).ok() == Some(rows);
    let is_payload_accounted = row_bytes == Some(payload_bytes);
    Ok(is_rows_consistent && invalid_rows == 0 && is_payload_accounted && content_matches)
}

/// Whether the reported facts account for the leaf's result.
fn is_store_facts_consistent(plan: StorePlan, facts: StoreFacts) -> Result<bool, StoreFactFailure> {
    match facts {
        StoreFacts::Rows { .. } => is_rows_consistent(plan.leaf, facts),
        StoreFacts::Roots { .. } => is_roots_consistent(plan.leaf, facts),
        StoreFacts::Usage { .. } => is_usage_consistent(plan.leaf, facts),
        StoreFacts::Retention { .. } => is_retention_consistent(plan.leaf, facts),
        StoreFacts::Gc { .. } => is_gc_consistent(plan.leaf, facts),
        StoreFacts::Verify { .. } => is_verify_consistent(plan.leaf, facts),
        StoreFacts::Repair { .. } => is_repair_consistent(plan.leaf, facts),
        StoreFacts::Sign { .. } => is_sign_consistent(plan.leaf, facts),
        StoreFacts::Push { .. } => is_push_consistent(plan.leaf, facts),
        StoreFacts::Pull { .. } => is_pull_consistent(plan.leaf, facts),
        StoreFacts::Composition { .. } => is_composition_consistent(plan.leaf, facts),
        StoreFacts::Archive { .. } => is_archive_consistent(plan.leaf, facts),
    }
}

/// Classify result accounting and persisted state, not whether a port returned Ok.
pub fn classify_store_facts(plan: StorePlan, facts: StoreFacts) -> Result<(), StoreFactFailure> {
    let is_consistent = is_store_facts_consistent(plan, facts)?;
    if is_consistent {
        return Ok(());
    }
    Err(StoreFactFailure::InconsistentResult)
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
    }

    #[test]
    fn empty_selectors_and_missing_roots_are_rejected() {
        let mut request = command(StoreOperation::Export, &["   "]);
        request.root = String::new();
        let blockers = validate_store_command(&request);
        assert!(blockers.contains(&StoreBlocker::EmptySelector));
        assert!(blockers.iter().any(|blocker| matches!(blocker, StoreBlocker::Domain(_))));
    }

    #[test]
    fn retention_requires_the_requested_post_transition_state() {
        let retained = StoreFacts::Retention {
            found: true,
            matching_after: 1,
        };
        let released = StoreFacts::Retention {
            found: true,
            matching_after: 0,
        };
        assert_eq!(classify_store_facts(StorePlan::new(StoreLeaf::Pin), retained), Ok(()));
        assert_eq!(
            classify_store_facts(StorePlan::new(StoreLeaf::Pin), released),
            Err(StoreFactFailure::InconsistentResult)
        );
        assert_eq!(
            classify_store_facts(StorePlan::new(StoreLeaf::Unpin), retained),
            Err(StoreFactFailure::InconsistentResult)
        );
        assert_eq!(classify_store_facts(StorePlan::new(StoreLeaf::Unpin), released), Ok(()));
        assert_eq!(
            classify_store_facts(StorePlan::new(StoreLeaf::Unpin), StoreFacts::Retention {
                found: false,
                matching_after: 0,
            }),
            Err(StoreFactFailure::MissingRoot)
        );
    }

    #[test]
    fn incomplete_gc_is_not_a_successful_empty_collection() {
        let plan = StorePlan::new(StoreLeaf::Gc { execute: true });
        let facts = |complete, failures| StoreFacts::Gc {
            dry_run: false,
            plan_matches: true,
            candidates: 0,
            candidate_rows: 0,
            complete,
            failures,
        };
        assert_eq!(classify_store_facts(plan, facts(true, 0)), Ok(()));
        assert_eq!(classify_store_facts(plan, facts(false, 0)), Err(StoreFactFailure::IncompleteGc));
        assert_eq!(classify_store_facts(plan, facts(true, 1)), Err(StoreFactFailure::IncompleteGc));
    }

    #[test]
    fn pull_readback_and_closure_root_are_independent_requirements() {
        let plan = StorePlan::new(StoreLeaf::Pull {
            closure: true,
            http: true,
        });
        let facts = |readback_matches, root_admitted| StoreFacts::Pull {
            imported: 1,
            skipped: 0,
            expected_paths: Some(1),
            rows: 1,
            invalid_rows: 0,
            payload_bytes: 0,
            row_bytes: Some(0),
            readback_matches,
            root_admitted: Some(root_admitted),
        };
        assert_eq!(classify_store_facts(plan, facts(true, true)), Ok(()));
        assert_eq!(classify_store_facts(plan, facts(false, true)), Err(StoreFactFailure::InconsistentResult));
        assert_eq!(classify_store_facts(plan, facts(true, false)), Err(StoreFactFailure::ClosureRootNotAdmitted));
    }

    #[test]
    fn explicit_pull_allows_policy_skips_but_accounts_for_every_http_selector() {
        let plan = StorePlan::new(StoreLeaf::Pull {
            closure: false,
            http: true,
        });
        let facts = |skipped| StoreFacts::Pull {
            imported: 0,
            skipped,
            expected_paths: Some(1),
            rows: 0,
            invalid_rows: 0,
            payload_bytes: 0,
            row_bytes: Some(0),
            readback_matches: true,
            root_admitted: None,
        };
        assert_eq!(classify_store_facts(plan, facts(1)), Ok(()));
        assert_eq!(classify_store_facts(plan, facts(0)), Err(StoreFactFailure::InconsistentResult));
    }

    #[test]
    fn archive_record_counts_do_not_substitute_for_payload_accounting() {
        let plan = StorePlan::new(StoreLeaf::ArchiveExport { stdio: true });
        let facts = |row_bytes| StoreFacts::Archive {
            reported: 2,
            rows: 2,
            invalid_rows: 0,
            payload_bytes: 10,
            row_bytes,
            content_matches: true,
        };
        assert_eq!(classify_store_facts(plan, facts(Some(10))), Ok(()));
        assert_eq!(classify_store_facts(plan, facts(Some(9))), Err(StoreFactFailure::InconsistentResult));
        assert_eq!(classify_store_facts(plan, facts(None)), Err(StoreFactFailure::InconsistentResult));
    }

    #[test]
    fn repair_must_not_claim_mutation_in_inspect_mode_or_leave_execute_unrepaired() {
        let facts = |execution_requested, mutated, readback_required| StoreFacts::Repair {
            path_matches: true,
            execution_requested,
            required: true,
            mutated,
            readback_required,
            signatures_match: true,
        };
        let inspect = StorePlan::new(StoreLeaf::Repair { execute: false });
        let execute = StorePlan::new(StoreLeaf::Repair { execute: true });
        assert_eq!(classify_store_facts(inspect, facts(false, false, true)), Ok(()));
        assert_eq!(classify_store_facts(inspect, facts(false, true, true)), Err(StoreFactFailure::InconsistentResult));
        assert_eq!(classify_store_facts(execute, facts(true, true, false)), Ok(()));
        assert_eq!(classify_store_facts(execute, facts(true, true, true)), Err(StoreFactFailure::InconsistentResult));
    }

    #[test]
    fn usage_overflow_is_not_an_empty_or_complete_partition() {
        let plan = StorePlan::new(StoreLeaf::Usage);
        assert_eq!(
            classify_store_facts(plan, StoreFacts::Usage {
                observed: 0,
                classified: Some(0)
            }),
            Ok(())
        );
        assert_eq!(
            classify_store_facts(plan, StoreFacts::Usage {
                observed: u64::MAX,
                classified: None
            }),
            Err(StoreFactFailure::InconsistentResult)
        );
    }

    #[test]
    fn push_accounts_for_skips_without_requiring_every_path_to_be_written() {
        let plan = StorePlan::new(StoreLeaf::Push);
        let facts = |selected| StoreFacts::Push {
            selected,
            pushed: 1,
            skipped_unsigned: 1,
            skipped_present: 1,
            rows: 1,
            invalid_rows: 0,
        };
        assert_eq!(classify_store_facts(plan, facts(3)), Ok(()));
        assert_eq!(classify_store_facts(plan, facts(4)), Err(StoreFactFailure::InconsistentResult));
    }

    #[test]
    fn signatures_require_both_transition_accounting_and_trusted_readback() {
        let plan = StorePlan::new(StoreLeaf::Sign);
        let facts = |transitions, trusted_after| StoreFacts::Sign {
            rows: 2,
            transitions,
            trusted_after,
        };
        assert_eq!(classify_store_facts(plan, facts(2, 2)), Ok(()));
        assert_eq!(classify_store_facts(plan, facts(1, 2)), Err(StoreFactFailure::InconsistentResult));
        assert_eq!(classify_store_facts(plan, facts(2, 1)), Err(StoreFactFailure::InconsistentResult));
    }
}
