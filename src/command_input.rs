//! Clap DTO mapping for the store and release families.
//!
//! The CLI DTOs are mapped here, once, into typed contract commands or into an
//! explicit statement that the contract does not administer the action.
//! Presentation stays in `crate::presentation`; this module only decides what
//! the operator asked for.

use mantle_application_contract::ReleaseCommand;
use mantle_application_contract::ReleaseOperation;
use mantle_application_contract::StoreAdministrationCommand;
use mantle_application_contract::StoreOperation;
use mantle_application_contract::validate_release_command;
use mantle_application_contract::validate_store_command;

use crate::ReleaseAction;
use crate::RunError;
use crate::StoreAction;

/// Command root the store DTO maps onto.
const STORE_COMMAND_ROOT: &str = "store";
/// Command root the release DTO maps onto.
const RELEASE_COMMAND_ROOT: &str = "release";

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

/// Shared test support: the CLI enum is large enough that parsing it needs a
/// bigger stack than a libtest thread provides.
#[cfg(test)]
pub(crate) mod test_support {
    use clap::Parser;

    use crate::Args;

    /// Stack size for a CLI parse in tests.
    pub(crate) const CLI_PARSE_TEST_STACK_BYTES: usize = 8_388_608;
    const _: () = assert!(CLI_PARSE_TEST_STACK_BYTES > 0);

    /// Parse one argument vector on a thread with room for the CLI enum.
    pub(crate) fn parse_args_on_cli_test_stack(args: Vec<&'static str>) -> Result<Args, String> {
        debug_assert!(!args.is_empty());
        std::thread::Builder::new()
            .stack_size(CLI_PARSE_TEST_STACK_BYTES)
            .spawn(move || Args::try_parse_from(args).map_err(|error| error.to_string()))
            .map_err(|error| format!("starting CLI parser test thread: {error}"))?
            .join()
            .map_err(|_| "CLI parser test thread panicked".to_string())?
    }
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

/// What one release DTO admits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReleaseCommandAdmission {
    /// The DTO maps onto an operation the release contract administers.
    Modeled(ReleaseCommand),
    /// The contract does not administer this action.
    NotModeled,
}

/// Map one release DTO onto its admission.
///
/// The directory a modeled operation consumes is the bundle directory, except
/// for the witness rebuild, which consumes its exported request directory.
pub(crate) fn admit_release_action(action: &ReleaseAction) -> ReleaseCommandAdmission {
    match action {
        ReleaseAction::Create { bundle_dir, .. } => ReleaseCommandAdmission::Modeled(release_command(
            ReleaseOperation::Create,
            path_text(bundle_dir.as_deref()),
        )),
        ReleaseAction::Verify { bundle_dir, .. } => {
            ReleaseCommandAdmission::Modeled(release_command(ReleaseOperation::Verify, path_text(Some(bundle_dir))))
        }
        ReleaseAction::Attest { bundle_dir, .. } => {
            ReleaseCommandAdmission::Modeled(release_command(ReleaseOperation::Attest, path_text(Some(bundle_dir))))
        }
        ReleaseAction::WitnessExport { bundle_dir, .. } => ReleaseCommandAdmission::Modeled(release_command(
            ReleaseOperation::WitnessExport,
            path_text(Some(bundle_dir)),
        )),
        ReleaseAction::WitnessRebuild { request_dir, .. } => ReleaseCommandAdmission::Modeled(release_command(
            ReleaseOperation::WitnessRebuild,
            path_text(Some(request_dir)),
        )),
        ReleaseAction::Transport { .. }
        | ReleaseAction::FunctionAddressBind { .. }
        | ReleaseAction::Reproduce { .. }
        | ReleaseAction::GlobalReproducibility { .. }
        | ReleaseAction::GlobalReproducibilityEvidence { .. }
        | ReleaseAction::Gauntlet { .. }
        | ReleaseAction::NixWitness { .. } => ReleaseCommandAdmission::NotModeled,
    }
}

/// Reject one release DTO the contract forbids before any effect runs.
pub(crate) fn admit_release_action_or_block(action: &ReleaseAction) -> Result<ReleaseCommandAdmission, RunError> {
    let admission = admit_release_action(action);
    let ReleaseCommandAdmission::Modeled(command) = &admission else {
        return Ok(admission);
    };
    let blockers = validate_release_command(command);
    let Some(blocker) = blockers.first() else {
        return Ok(admission);
    };
    debug_assert!(!blockers.is_empty());
    Err(RunError::Internal(format!("release {} request rejected: {blocker:?}", command.operation.as_str())))
}

/// Build one typed release command.
fn release_command(operation: ReleaseOperation, bundle_dir: String) -> ReleaseCommand {
    let command = ReleaseCommand {
        root: String::from(RELEASE_COMMAND_ROOT),
        operation,
        bundle_dir,
        required_proofs: Vec::new(),
        dry_run: false,
    };
    command
}

/// Render an optional path as text, treating an absent path as an empty string.
fn path_text(path: Option<&std::path::Path>) -> String {
    path.map(|path| path.display().to_string()).unwrap_or_default()
}

#[cfg(test)]
mod release_tests {
    use clap::Parser;

    use super::*;

    /// Parse one real `release` command line into its CLI DTO.
    fn release_action(parts: &[&'static str]) -> ReleaseAction {
        let mut argv: Vec<&'static str> = vec!["mantle", "release"];
        argv.extend_from_slice(parts);
        let args = test_support::parse_args_on_cli_test_stack(argv).expect("release command line must parse");
        let crate::Command::Release { action } = args.command else {
            panic!("expected a release command");
        };
        action
    }

    #[test]
    fn the_five_modeled_actions_map_to_their_operations_and_directories() {
        let create = release_action(&[
            "create",
            "--release-id",
            "v1",
            "--bundle-dir",
            "/tmp/release-evidence/v1",
            "--binary",
            "/tmp/aaaa-crunch/bin/crunch",
            "--proof-bundle",
            "/tmp/proof",
        ]);
        let ReleaseCommandAdmission::Modeled(command) = admit_release_action(&create) else {
            panic!("create must be modeled");
        };
        assert_eq!(command.operation, ReleaseOperation::Create);
        assert_eq!(command.bundle_dir, "/tmp/release-evidence/v1");
        assert_eq!(command.root, RELEASE_COMMAND_ROOT);
        assert!(!command.operation.requires_bundle());

        let verify = release_action(&["verify", "/tmp/release-evidence/v1"]);
        let ReleaseCommandAdmission::Modeled(command) = admit_release_action(&verify) else {
            panic!("verify must be modeled");
        };
        assert_eq!(command.operation, ReleaseOperation::Verify);
        assert_eq!(command.bundle_dir, "/tmp/release-evidence/v1");
        assert!(command.operation.requires_bundle());

        let attest = release_action(&["attest", "/tmp/release-evidence/v1"]);
        let ReleaseCommandAdmission::Modeled(command) = admit_release_action(&attest) else {
            panic!("attest must be modeled");
        };
        assert_eq!(command.operation, ReleaseOperation::Attest);

        let export = release_action(&["witness-export", "/tmp/release-evidence/v1"]);
        let ReleaseCommandAdmission::Modeled(command) = admit_release_action(&export) else {
            panic!("witness export must be modeled");
        };
        assert_eq!(command.operation, ReleaseOperation::WitnessExport);

        let rebuild = release_action(&["witness-rebuild", "/tmp/witness-request"]);
        let ReleaseCommandAdmission::Modeled(command) = admit_release_action(&rebuild) else {
            panic!("witness rebuild must be modeled");
        };
        assert_eq!(command.operation, ReleaseOperation::WitnessRebuild);
        assert_eq!(command.bundle_dir, "/tmp/witness-request");
        assert!(command.operation.requires_bundle());
    }

    #[test]
    fn a_create_without_an_explicit_bundle_directory_is_admissible() {
        let create = release_action(&[
            "create",
            "--release-id",
            "v1",
            "--binary",
            "/tmp/aaaa-crunch/bin/crunch",
            "--proof-bundle",
            "/tmp/proof",
        ]);
        let ReleaseCommandAdmission::Modeled(command) = admit_release_action_or_block(&create).expect("admissible")
        else {
            panic!("create must be modeled");
        };
        assert!(command.bundle_dir.is_empty());
        assert!(!command.operation.requires_bundle());
    }

    #[test]
    fn a_modeled_action_without_its_directory_fails_closed() {
        let mut verify = release_action(&["verify", "/tmp/release-evidence/v1"]);
        let crate::ReleaseAction::Verify { bundle_dir, .. } = &mut verify else {
            panic!("expected a verify action");
        };
        *bundle_dir = std::path::PathBuf::new();
        let error = admit_release_action_or_block(&verify).expect_err("a missing bundle must fail closed");
        assert!(error.to_string().contains("verify"), "{error}");
        assert!(error.to_string().contains("MissingBundle"), "{error}");
    }

    #[test]
    fn unmodeled_actions_stay_outside_the_contract() {
        let global = release_action(&[
            "global-reproducibility-evidence",
            "--universe",
            "universe.json",
            "--policy",
            "policy.json",
            "--bundle-dir",
            "/tmp/bundle",
            "--verification-dir",
            "/tmp/verification",
            "--release-verify-json",
            "/tmp/verify.json",
            "--evidence-path",
            "/tmp/evidence.json",
        ]);
        assert_eq!(admit_release_action(&global), ReleaseCommandAdmission::NotModeled);
        assert!(matches!(admit_release_action_or_block(&global), Ok(ReleaseCommandAdmission::NotModeled)));
    }
}
