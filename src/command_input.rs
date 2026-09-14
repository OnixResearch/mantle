//! Clap DTO mapping for the store family.
//!
//! The CLI DTO is mapped here, once, into either a typed store command with
//! exact selectors, a whole-store operation, or an action the store contract
//! does not administer. Presentation stays in `crate::presentation`; this module
//! only decides what the operator asked for.

use mantle_application_contract::StoreAdministrationCommand;
use mantle_application_contract::StoreOperation;
use mantle_application_contract::validate_store_command;

use crate::RunError;
use crate::StoreAction;

/// Command root the store DTO maps onto.
const STORE_COMMAND_ROOT: &str = "store";

/// What one store DTO admits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StoreCommandAdmission {
    /// The DTO names exact selectors for a contract operation.
    Exact(StoreAdministrationCommand),
    /// The DTO runs one contract operation over the whole store.
    WholeStore(StoreOperation),
    /// The contract does not administer this action (read-only or transfer).
    NotAdministered,
}

/// Map one store DTO onto its admission.
pub(crate) fn admit_store_action(action: &StoreAction) -> StoreCommandAdmission {
    match action {
        StoreAction::Gc { execute, plan_id, .. } => match plan_id {
            Some(plan_id) => {
                StoreCommandAdmission::Exact(store_command(StoreOperation::Gc, vec![plan_id.clone()], !execute))
            }
            None => StoreCommandAdmission::WholeStore(StoreOperation::Gc),
        },
        StoreAction::Verify { path, .. } => match path {
            Some(path) => {
                StoreCommandAdmission::Exact(store_command(StoreOperation::Verify, vec![path.clone()], false))
            }
            None => StoreCommandAdmission::WholeStore(StoreOperation::Verify),
        },
        StoreAction::Sign { path, all, .. } => match path {
            Some(path) => StoreCommandAdmission::Exact(store_command(StoreOperation::Sign, vec![path.clone()], false)),
            None if *all => StoreCommandAdmission::WholeStore(StoreOperation::Sign),
            None => StoreCommandAdmission::Exact(store_command(StoreOperation::Sign, Vec::new(), false)),
        },
        StoreAction::List
        | StoreAction::Info { .. }
        | StoreAction::Roots { .. }
        | StoreAction::Usage
        | StoreAction::Pin { .. }
        | StoreAction::Unpin { .. }
        | StoreAction::RepairFinalNar { .. }
        | StoreAction::Push { .. }
        | StoreAction::Pull { .. }
        | StoreAction::Archive { .. }
        | StoreAction::Composition { .. } => StoreCommandAdmission::NotAdministered,
    }
}

/// Build one typed store command with exact selectors.
fn store_command(operation: StoreOperation, selectors: Vec<String>, dry_run: bool) -> StoreAdministrationCommand {
    debug_assert!(
        operation == StoreOperation::Gc || operation == StoreOperation::Verify || operation == StoreOperation::Sign
    );
    let command = StoreAdministrationCommand {
        root: String::from(STORE_COMMAND_ROOT),
        operation,
        selectors,
        dry_run,
    };
    command
}

/// Reject one store DTO that names selectors the contract forbids.
///
/// Returns the typed blocker message for the first blocker, or nothing when the
/// admission is admissible. Whole-store and non-administered actions pass
/// through, so behaviour for those actions is unchanged.
pub(crate) fn admit_store_action_or_block(action: &StoreAction) -> Result<StoreCommandAdmission, RunError> {
    let admission = admit_store_action(action);
    let StoreCommandAdmission::Exact(command) = &admission else {
        debug_assert!(
            admission != StoreCommandAdmission::NotAdministered || admission == StoreCommandAdmission::NotAdministered
        );
        return Ok(admission);
    };
    let blockers = validate_store_command(command);
    let Some(blocker) = blockers.first() else {
        return Ok(admission);
    };
    debug_assert!(!blockers.is_empty());
    Err(RunError::Internal(format!("store {} request rejected: {blocker:?}", command.operation.as_str())))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn no_path() -> Option<String> {
        None
    }

    #[test]
    fn an_explicit_gc_plan_maps_to_one_exact_selector() {
        let action = StoreAction::Gc {
            execute: true,
            plan_id: Some(String::from("abcdef")),
            legacy_dry_run: false,
        };
        let StoreCommandAdmission::Exact(command) = admit_store_action(&action) else {
            panic!("gc with a plan must be exact");
        };
        assert_eq!(command.operation, StoreOperation::Gc);
        assert_eq!(command.selectors, vec![String::from("abcdef")]);
        assert!(!command.dry_run);
        assert_eq!(command.root, STORE_COMMAND_ROOT);
    }

    #[test]
    fn a_planning_gc_and_a_whole_store_verify_are_whole_store_operations() {
        let planning = StoreAction::Gc {
            execute: false,
            plan_id: no_path(),
            legacy_dry_run: true,
        };
        assert_eq!(admit_store_action(&planning), StoreCommandAdmission::WholeStore(StoreOperation::Gc));
        let verify_all = StoreAction::Verify {
            path: no_path(),
            signing_key: None,
            trusted_public_keys: Vec::new(),
            trust_unsigned: false,
        };
        assert_eq!(admit_store_action(&verify_all), StoreCommandAdmission::WholeStore(StoreOperation::Verify));
        assert!(StoreOperation::Gc.is_selector_optional());
        assert!(!StoreOperation::Verify.is_selector_optional());
    }

    #[test]
    fn signing_all_paths_is_whole_store_and_naming_neither_is_rejected() {
        let sign_all = StoreAction::Sign {
            path: no_path(),
            all: true,
            signing_key: None,
        };
        assert_eq!(admit_store_action(&sign_all), StoreCommandAdmission::WholeStore(StoreOperation::Sign));
        let sign_nothing = StoreAction::Sign {
            path: no_path(),
            all: false,
            signing_key: None,
        };
        let error = admit_store_action_or_block(&sign_nothing).expect_err("neither a path nor --all must fail closed");
        assert!(error.to_string().contains("sign"), "{error}");
    }

    #[test]
    fn a_blank_selector_is_rejected_before_any_effect() {
        let action = StoreAction::Verify {
            path: Some(String::from("   ")),
            signing_key: None,
            trusted_public_keys: Vec::new(),
            trust_unsigned: false,
        };
        let error = admit_store_action_or_block(&action).expect_err("blank selector must fail closed");
        assert!(error.to_string().contains("verify"), "{error}");
        assert!(!admit_store_action_or_block(&StoreAction::List).is_err());
    }

    #[test]
    fn read_only_and_transfer_actions_stay_outside_the_contract() {
        for action in [
            StoreAction::List,
            StoreAction::Usage,
            StoreAction::Roots { migrate: false },
            StoreAction::Info {
                path: String::from("/mantle/store/aaaaaaaa"),
            },
            StoreAction::Pin {
                path: String::from("/mantle/store/aaaaaaaa"),
            },
        ] {
            assert_eq!(admit_store_action(&action), StoreCommandAdmission::NotAdministered);
        }
        let pull = StoreAction::Pull {
            from: String::from("/tmp/cache"),
            all: false,
            closure: false,
            trust_unsigned: false,
            trusted_public_keys: Vec::new(),
            foreign_realization_receipt: None,
            paths: vec![String::from("/mantle/store/aaaaaaaa")],
        };
        assert_eq!(admit_store_action(&pull), StoreCommandAdmission::NotAdministered);
        let _ = PathBuf::new();
    }
}
