//! Clap DTO mapping for admitted store, project, source, and execution commands.
//!
//! The CLI DTOs are mapped here, once, into typed contract commands or into an
//! explicit statement that the contract does not administer the action.
//! Presentation stays in `crate::presentation`; this module only decides what
//! the operator asked for.

use std::path::Path;

use mantle_application_contract::ComponentCommand;
use mantle_application_contract::ComponentOperation;
use mantle_application_contract::EvaluationCommand;
use mantle_application_contract::EvaluationOperation;
use mantle_application_contract::ProjectCommand;
use mantle_application_contract::ProjectOperation;
use mantle_application_contract::RealizeCommand;
use mantle_application_contract::RemoteExecutionCommand;
use mantle_application_contract::RemoteExecutionOperation;
use mantle_application_contract::SourceProvenanceCommand;
use mantle_application_contract::SourceProvenanceOperation;
use mantle_application_contract::StoreAdministrationCommand;
use mantle_application_contract::StoreOperation;
use mantle_application_contract::validate_component_flow;
use mantle_application_contract::validate_evaluation;
use mantle_application_contract::validate_project_lifecycle;
use mantle_application_contract::validate_realize_command;
use mantle_application_contract::validate_remote_execution;
use mantle_application_contract::validate_source_provenance;
use mantle_application_contract::validate_store_command;

use crate::FilegenCommandAction;
use crate::RunError;
use crate::SemanticGraphQueryKind;
use crate::SourceAction;
use crate::SourceBundleAction;
use crate::StoreAction;
use crate::WasmComponentAction;

/// Command root the store DTO maps onto.
const STORE_COMMAND_ROOT: &str = "store";
/// Command root the project-lifecycle DTO maps onto.
const PROJECT_COMMAND_ROOT: &str = "project";
/// Manifest file the project-lifecycle commands work from.
const PROJECT_MANIFEST_FILE: &str = crate::project_cmd::MANIFEST_FILE;
/// Command root the evaluation DTO maps onto.
const EVALUATION_COMMAND_ROOT: &str = "eval";
/// Command root the source DTO maps onto.
const SOURCE_COMMAND_ROOT: &str = "source";
/// Command root the component DTO maps onto.
const COMPONENT_COMMAND_ROOT: &str = "wasm-component";
/// Command root the filegen DTO maps onto.
const FILEGEN_COMMAND_ROOT: &str = "filegen";
/// Command root the run DTO maps onto.
const RUN_COMMAND_ROOT: &str = "run";
/// Command root the shell DTO maps onto.
const SHELL_COMMAND_ROOT: &str = "shell";
/// Command root the develop DTO maps onto.
const DEVELOP_COMMAND_ROOT: &str = "develop";
/// Command root the remote secret worker maps onto.
const REMOTE_SECRET_WORKER_COMMAND_ROOT: &str = "remote-secret-worker";

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
    StoreAdministrationCommand {
        root: String::from(STORE_COMMAND_ROOT),
        operation,
        selectors,
        dry_run,
    }
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

    /// Parse one argument vector and extract a value through `extract`.
    pub(crate) fn parse_action_from<T>(
        args: &[&'static str],
        extract: impl FnOnce(crate::Command) -> Option<T> + Send + 'static,
    ) -> T
    where
        T: Send + 'static,
    {
        let owned: Vec<&'static str> = args.to_vec();
        let parsed = parse_args_on_cli_test_stack(owned).expect("fixture command line must parse");
        extract(parsed.command).expect("fixture command line must produce the expected command")
    }

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

/// Build the typed project-lifecycle command for one CLI operation.
///
/// The CLI carries no subject for `init`, `show`, or `upgrade`; the subject is
/// the resolved project directory the shell hands in. `selected` carries the
/// names a refresh was asked to resolve.
pub(crate) fn project_lifecycle_command(
    operation: ProjectOperation,
    project_dir: &Path,
    selected: &[String],
) -> ProjectCommand {
    debug_assert!(ProjectOperation::all().contains(&operation));
    debug_assert!(!PROJECT_MANIFEST_FILE.is_empty());
    ProjectCommand {
        root: String::from(PROJECT_COMMAND_ROOT),
        operation,
        subject: project_dir.display().to_string(),
        declared_entries: selected.to_vec(),
        manifest_path: project_dir.join(PROJECT_MANIFEST_FILE).display().to_string(),
        has_lock_write: writes_project_lock(operation),
    }
}

/// Whether one project operation rewrites the lockfile.
fn writes_project_lock(operation: ProjectOperation) -> bool {
    let writes_lock = matches!(operation, ProjectOperation::Refresh | ProjectOperation::Upgrade);
    debug_assert!(!writes_lock || ProjectOperation::all().contains(&operation));
    writes_lock
}

/// Reject one project-lifecycle command the contract refuses.
pub(crate) fn admit_project_lifecycle(command: &ProjectCommand) -> Result<(), RunError> {
    let blockers = validate_project_lifecycle(command);
    let Some(blocker) = blockers.first() else {
        return Ok(());
    };
    debug_assert!(!blockers.is_empty());
    Err(RunError::Internal(format!("project {} request rejected: {blocker:?}", command.operation.as_str())))
}

/// Admit one project-lifecycle CLI operation in a single step.
pub(crate) fn admit_project_lifecycle_for(
    operation: ProjectOperation,
    project_dir: &Path,
    selected: &[String],
) -> Result<(), RunError> {
    admit_project_lifecycle(&project_lifecycle_command(operation, project_dir, selected))
}

#[cfg(test)]
mod project_tests {
    use super::*;

    fn project_dir() -> std::path::PathBuf {
        std::path::PathBuf::from("/tmp/example-project")
    }

    #[test]
    fn every_operation_maps_to_the_manifest_of_its_directory() {
        for operation in ProjectOperation::all() {
            let command = project_lifecycle_command(operation, &project_dir(), &[]);
            assert_eq!(command.operation, operation);
            assert_eq!(command.root, PROJECT_COMMAND_ROOT);
            assert_eq!(command.subject, "/tmp/example-project");
            assert_eq!(command.manifest_path, "/tmp/example-project/mantle-project.ncl");
            assert!(command.declared_entries.is_empty());
            assert_eq!(command.has_lock_write, writes_project_lock(operation));
        }
        assert_eq!(ProjectOperation::all().len(), 6);
    }

    #[test]
    fn the_operations_that_rewrite_the_lock_declare_it() {
        assert!(writes_project_lock(ProjectOperation::Refresh));
        assert!(writes_project_lock(ProjectOperation::Upgrade));
        assert!(!writes_project_lock(ProjectOperation::Init));
        assert!(!writes_project_lock(ProjectOperation::Check));
        assert!(!writes_project_lock(ProjectOperation::Show));
        assert!(!writes_project_lock(ProjectOperation::ListStale));
    }

    #[test]
    fn a_resolved_directory_admits_every_operation() {
        for operation in ProjectOperation::all() {
            assert!(admit_project_lifecycle_for(operation, &project_dir(), &[]).is_ok(), "{operation:?} must admit");
        }
    }

    #[test]
    fn an_empty_subject_fails_closed_and_selected_names_are_carried() {
        let empty = project_lifecycle_command(ProjectOperation::Refresh, Path::new(""), &[]);
        let error = admit_project_lifecycle(&empty).expect_err("an empty subject must fail closed");
        assert!(error.to_string().contains("refresh"), "{error}");
        assert!(error.to_string().contains("MissingSubject"), "{error}");

        let selected = vec![String::from("left-pad"), String::from("serde")];
        let refresh = project_lifecycle_command(ProjectOperation::Refresh, &project_dir(), &selected);
        assert_eq!(refresh.declared_entries, selected);
        assert!(admit_project_lifecycle(&refresh).is_ok());
    }

    #[test]
    fn a_lock_rewrite_without_the_lock_declared_fails_closed() {
        let mut command = project_lifecycle_command(ProjectOperation::Upgrade, &project_dir(), &[]);
        command.has_lock_write = false;
        let error = admit_project_lifecycle(&command).expect_err("a lock rewrite must declare its write");
        assert!(error.to_string().contains("LockWriteRequired"), "{error}");
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SemanticGraphCommandInput<'a> {
    pub(crate) query: SemanticGraphQueryKind,
    pub(crate) target: &'a str,
    pub(crate) graph_file: Option<&'a Path>,
}

impl<'a> SemanticGraphCommandInput<'a> {
    /// Map the parsed CLI DTO into the typed application command.
    pub(crate) fn command_request(&self) -> mantle_application_contract::GraphQueryRequest {
        let kind = match self.query {
            SemanticGraphQueryKind::Graph => mantle_application_contract::GraphQueryKind::Graph,
            SemanticGraphQueryKind::Why => mantle_application_contract::GraphQueryKind::Why,
            SemanticGraphQueryKind::Dependents => mantle_application_contract::GraphQueryKind::Dependents,
        };
        debug_assert!(!self.target.is_empty());
        debug_assert!(!kind.as_str().is_empty());
        mantle_application_contract::GraphQueryRequest {
            kind,
            target: self.target.to_string(),
        }
    }

    pub(crate) fn new(query: SemanticGraphQueryKind, target: &'a str, graph_file: Option<&'a Path>) -> Self {
        Self {
            query,
            target,
            graph_file,
        }
    }
}

/// Map a parsed semantic-graph DTO into its typed request.
#[cfg(test)]
mod graph_tests {
    use super::*;

    #[test]
    fn each_graph_query_kind_maps_to_its_contract_kind() {
        for (query, expected) in [
            (crate::SemanticGraphQueryKind::Graph, "graph"),
            (crate::SemanticGraphQueryKind::Why, "why"),
            (crate::SemanticGraphQueryKind::Dependents, "dependents"),
        ] {
            let input = SemanticGraphCommandInput::new(query, "target-name", None);
            let request = input.command_request();
            assert_eq!(request.target, "target-name");
            assert_eq!(request.kind.as_str(), expected);
        }
    }

    #[test]
    fn the_graph_file_stays_optional_and_is_kept_verbatim() {
        let path = Path::new("/tmp/semantic-graph.json");
        let with_file = SemanticGraphCommandInput::new(crate::SemanticGraphQueryKind::Graph, "root", Some(path));
        assert_eq!(with_file.graph_file, Some(path));
        let without_file = SemanticGraphCommandInput::new(crate::SemanticGraphQueryKind::Graph, "root", None);
        assert!(without_file.graph_file.is_none());
    }
}

/// Build the typed evaluation command for the `eval` CLI DTO.
///
/// The CLI evaluates one file and selects which roots it reaches; the contract
/// models that as one evaluate operation whose declared entries are the named
/// roots. An empty root list means the whole value or every root, which the
/// command carries as an empty declared-entry list.
pub(crate) fn evaluation_command(source: &Path, selected_roots: &[String]) -> EvaluationCommand {
    debug_assert!(!EVALUATION_COMMAND_ROOT.is_empty());
    EvaluationCommand {
        root: String::from(EVALUATION_COMMAND_ROOT),
        operation: EvaluationOperation::Evaluate,
        subject: source.display().to_string(),
        declared_entries: selected_roots.to_vec(),
        source_path: source.display().to_string(),
        worker_count: None,
    }
}

/// Reject one evaluation command the contract refuses.
pub(crate) fn admit_evaluation(command: &EvaluationCommand) -> Result<(), RunError> {
    let blockers = validate_evaluation(command);
    let Some(blocker) = blockers.first() else {
        return Ok(());
    };
    debug_assert!(!blockers.is_empty());
    Err(RunError::Internal(format!("eval {} request rejected: {blocker:?}", command.operation.as_str())))
}

#[cfg(test)]
mod evaluation_tests {
    use super::*;

    #[test]
    fn the_source_file_is_both_the_subject_and_the_source_path() {
        let source = Path::new("/tmp/example.ncl");
        let command = evaluation_command(source, &[]);
        assert_eq!(command.root, EVALUATION_COMMAND_ROOT);
        assert_eq!(command.operation, EvaluationOperation::Evaluate);
        assert_eq!(command.subject, "/tmp/example.ncl");
        assert_eq!(command.source_path, "/tmp/example.ncl");
        assert!(command.declared_entries.is_empty());
        assert_eq!(command.worker_count, None);
    }

    #[test]
    fn selected_roots_become_the_declared_entries() {
        let roots = vec![String::from("alpha"), String::from("beta")];
        let command = evaluation_command(Path::new("/tmp/example.ncl"), &roots);
        assert_eq!(command.declared_entries, roots);
        assert!(admit_evaluation(&command).is_ok());
    }

    #[test]
    fn a_whole_file_evaluation_admits_without_declared_entries() {
        let command = evaluation_command(Path::new("/tmp/example.ncl"), &[]);
        assert!(admit_evaluation(&command).is_ok());
        assert!(!EvaluationOperation::Evaluate.requires_declared_entries());
    }

    #[test]
    fn an_empty_source_fails_closed_with_both_blockers() {
        let command = evaluation_command(Path::new(""), &[]);
        let error = admit_evaluation(&command).expect_err("an empty source must fail closed");
        assert!(error.to_string().contains("evaluate"), "{error}");
        assert!(error.to_string().contains("MissingSubject"), "{error}");
    }
}

/// What one source DTO admits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SourceCommandAdmission {
    /// The DTO maps onto a source-provenance operation the contract administers.
    Modeled(SourceProvenanceCommand),
    /// The contract does not administer this bundle action.
    NotAdministered,
}

/// Map one source DTO onto its admission.
///
/// Export is the one bundle action the contract models directly: it names a
/// destination and the source selectors it exports. The self-build hydration is
/// deliberately not administered: the contract's hydrate operation requires
/// declared entries, and the CLI's hydration names only the bundle it consumes,
/// so admitting it would mean naming an input the operator never declared. The
/// remaining actions — plan, bootstrap profile, refresh, list, import, verify,
/// preflight, and hydration — have no operation the contract can check here.
pub(crate) fn admit_source_action(action: &SourceAction) -> SourceCommandAdmission {
    match action {
        SourceAction::Bundle { action } => match action {
            SourceBundleAction::Export { sources, to, .. } => SourceCommandAdmission::Modeled(source_command(
                SourceProvenanceOperation::Export,
                &to.display().to_string(),
                sources.clone(),
                None,
            )),
            SourceBundleAction::Plan { .. }
            | SourceBundleAction::BootstrapProfile { .. }
            | SourceBundleAction::RefreshMantleSource { .. }
            | SourceBundleAction::List { .. }
            | SourceBundleAction::Import { .. }
            | SourceBundleAction::HydrateSelfBuild { .. }
            | SourceBundleAction::Verify { .. }
            | SourceBundleAction::Preflight { .. } => SourceCommandAdmission::NotAdministered,
        },
    }
}

/// Reject one source command the contract refuses before any effect runs.
pub(crate) fn admit_source_action_or_block(action: &SourceAction) -> Result<SourceCommandAdmission, RunError> {
    let admission = admit_source_action(action);
    let SourceCommandAdmission::Modeled(command) = &admission else {
        return Ok(admission);
    };
    let blockers = validate_source_provenance(command);
    let Some(blocker) = blockers.first() else {
        return Ok(admission);
    };
    debug_assert!(!blockers.is_empty());
    Err(RunError::Internal(format!("source {} request rejected: {blocker:?}", command.operation.as_str())))
}

/// Build one typed source-provenance command.
fn source_command(
    operation: SourceProvenanceOperation,
    subject: &str,
    declared_entries: Vec<String>,
    expected_digest: Option<mantle_application_contract::Blake3Digest>,
) -> SourceProvenanceCommand {
    debug_assert!(!SOURCE_COMMAND_ROOT.is_empty());
    debug_assert!(!operation.as_str().is_empty());
    SourceProvenanceCommand {
        root: String::from(SOURCE_COMMAND_ROOT),
        operation,
        subject: String::from(subject),
        declared_entries,
        expected_digest,
    }
}

#[cfg(test)]
mod source_tests {
    use std::path::PathBuf;

    use super::*;

    fn export_action() -> SourceAction {
        super::test_support::parse_action_from(
            &["mantle", "source", "bundle", "export", "--to", "/tmp/out-bundle"],
            |command| match command {
                crate::Command::Source { action } => Some(action),
                _ => None,
            },
        )
    }

    #[test]
    fn an_export_maps_to_its_destination_and_selectors() {
        let action = export_action();
        let SourceCommandAdmission::Modeled(command) = admit_source_action(&action) else {
            panic!("export must be modeled");
        };
        assert_eq!(command.operation, SourceProvenanceOperation::Export);
        assert_eq!(command.root, SOURCE_COMMAND_ROOT);
        assert_eq!(command.subject, "/tmp/out-bundle");
        assert_eq!(command.expected_digest, None);
        assert!(admit_source_action_or_block(&action).is_ok());
    }

    #[test]
    fn a_self_build_hydration_is_not_administered() {
        let action = super::test_support::parse_action_from(
            &[
                "mantle",
                "source",
                "bundle",
                "hydrate-self-build",
                "--from",
                "/tmp/bundle",
                "--expected-manifest-blake3",
                "0000000000000000000000000000000000000000000000000000000000000000",
                "--checkout",
                "/tmp/checkout",
            ],
            |command| match command {
                crate::Command::Source { action } => Some(action),
                _ => None,
            },
        );
        assert_eq!(admit_source_action(&action), SourceCommandAdmission::NotAdministered);
        assert!(SourceProvenanceOperation::Hydrate.requires_declared_entries());
    }

    #[test]
    fn a_bundle_plan_is_not_administered() {
        let action =
            super::test_support::parse_action_from(&["mantle", "source", "bundle", "plan"], |command| match command {
                crate::Command::Source { action } => Some(action),
                _ => None,
            });
        assert_eq!(admit_source_action(&action), SourceCommandAdmission::NotAdministered);
        assert!(matches!(admit_source_action_or_block(&action), Ok(SourceCommandAdmission::NotAdministered)));
        let _ = PathBuf::new();
    }

    #[test]
    fn an_empty_subject_fails_closed() {
        let command = source_command(SourceProvenanceOperation::Export, "", Vec::new(), None);
        assert!(command.subject.is_empty());
        let error = admit_source_action_or_block(&SourceAction::Bundle {
            action: SourceBundleAction::Export {
                sources: Vec::new(),
                build_roots: Vec::new(),
                import_paths: Vec::new(),
                cached_fetches: Vec::new(),
                to: PathBuf::new(),
                fetch_missing: false,
            },
        })
        .expect_err("an empty destination must fail closed");
        assert!(error.to_string().contains("export"), "{error}");
        assert!(error.to_string().contains("MissingSubject"), "{error}");
    }
}

/// Admit one component DTO and return its typed command.
///
/// The CLI names its component through a typed Nickel request, so that request
/// path is the subject and the component path this command carries; the output
/// directory is where the build publishes artifacts and its execution report,
/// which is the evidence directory. The import paths are the declared entries
/// the build consumes. A future `bundle` or `verify` subcommand must appear in
/// this match, because the operation set it maps onto is exhaustive.
pub(crate) fn admit_component_action(action: &WasmComponentAction) -> Result<ComponentCommand, RunError> {
    let command = match action {
        WasmComponentAction::Build {
            request,
            out,
            import_paths,
            ..
        } => ComponentCommand {
            root: String::from(COMPONENT_COMMAND_ROOT),
            operation: ComponentOperation::Build,
            subject: request.display().to_string(),
            declared_entries: import_paths.iter().map(|path| path.display().to_string()).collect(),
            component_path: request.display().to_string(),
            evidence_dir: out.display().to_string(),
        },
    };
    debug_assert_eq!(command.root, COMPONENT_COMMAND_ROOT);
    let blockers = validate_component_flow(&command);
    let Some(blocker) = blockers.first() else {
        return Ok(command);
    };
    debug_assert!(!blockers.is_empty());
    Err(RunError::Internal(format!(
        "wasm-component {} request rejected: {blocker:?}",
        command.operation.as_str()
    )))
}

#[cfg(test)]
mod component_tests {
    use super::*;

    fn build_action() -> WasmComponentAction {
        super::test_support::parse_action_from(
            &[
                "mantle",
                "wasm-component",
                "build",
                "/tmp/component-request.ncl",
                "--out",
                "/tmp/component-out",
                "--import-path",
                "/tmp/imports",
            ],
            |command| match command {
                crate::Command::WasmComponent { action } => Some(action),
                _ => None,
            },
        )
    }

    #[test]
    fn a_build_maps_its_request_output_and_imports() {
        let action = build_action();
        let command = admit_component_action(&action).expect("a build must admit");
        assert_eq!(command.operation, ComponentOperation::Build);
        assert_eq!(command.root, COMPONENT_COMMAND_ROOT);
        assert_eq!(command.subject, "/tmp/component-request.ncl");
        assert_eq!(command.component_path, "/tmp/component-request.ncl");
        assert_eq!(command.evidence_dir, "/tmp/component-out");
        assert_eq!(command.declared_entries, vec![String::from("/tmp/imports")]);
        assert!(!ComponentOperation::Build.requires_declared_entries());
    }

    #[test]
    fn a_build_without_imports_still_admits() {
        let action = super::test_support::parse_action_from(
            &[
                "mantle",
                "wasm-component",
                "build",
                "/tmp/component-request.ncl",
                "--out",
                "/tmp/component-out",
            ],
            |command| match command {
                crate::Command::WasmComponent { action } => Some(action),
                _ => None,
            },
        );
        let command = admit_component_action(&action).expect("a build must admit");
        assert!(command.declared_entries.is_empty());
        assert!(!command.evidence_dir.is_empty());
    }

    #[test]
    fn an_empty_request_path_fails_closed() {
        let action = WasmComponentAction::Build {
            request: std::path::PathBuf::new(),
            out: std::path::PathBuf::from("/tmp/component-out"),
            import_paths: Vec::new(),
            scratch_parent: None,
        };
        let error = admit_component_action(&action).expect_err("an empty subject must fail closed");
        assert!(error.to_string().contains("build"), "{error}");
        assert!(error.to_string().contains("MissingSubject"), "{error}");
    }
}

/// Admit one filegen DTO and return its realization command.
///
/// Filegen is the realization root whose two actions pair a dry run with its
/// execution: planning renders a no-mutate plan, and applying performs the plan
/// after its drift checks. The manifest is the requested root, and the CLI
/// declares no build profile on this path, which the command carries as `None`.
pub(crate) fn admit_filegen_action(action: &FilegenCommandAction) -> Result<RealizeCommand, RunError> {
    let command = match action {
        FilegenCommandAction::Plan { manifest, .. } => realize_command(manifest, true),
        FilegenCommandAction::Apply { manifest, .. } => realize_command(manifest, false),
    };
    debug_assert_eq!(command.root, FILEGEN_COMMAND_ROOT);
    let blockers = validate_realize_command(&command);
    let Some(blocker) = blockers.first() else {
        return Ok(command);
    };
    debug_assert!(!blockers.is_empty());
    Err(RunError::Internal(format!("filegen request rejected: {blocker:?}")))
}

/// Build one realization command for a filegen action.
fn realize_command(manifest: &Path, dry_run: bool) -> RealizeCommand {
    debug_assert!(!FILEGEN_COMMAND_ROOT.is_empty());
    RealizeCommand {
        root: String::from(FILEGEN_COMMAND_ROOT),
        roots: vec![manifest.display().to_string()],
        profile: None,
        requested_jobs: None,
        dry_run,
    }
}

#[cfg(test)]
mod realization_tests {
    use super::*;

    fn filegen_action(parts: &[&'static str]) -> FilegenCommandAction {
        let mut argv: Vec<&'static str> = vec!["mantle", "filegen"];
        argv.extend_from_slice(parts);
        super::test_support::parse_action_from(&argv, |command| match command {
            crate::Command::Filegen { action } => Some(action),
            _ => None,
        })
    }

    #[test]
    fn planning_is_a_dry_run_and_applying_is_not() {
        let plan = admit_filegen_action(&filegen_action(&["plan"])).expect("planning must admit");
        assert_eq!(plan.root, FILEGEN_COMMAND_ROOT);
        assert!(plan.dry_run);
        assert_eq!(plan.profile, None);
        assert_eq!(plan.requested_jobs, None);
        assert_eq!(plan.roots.len(), 1);
        assert!(plan.roots[0].ends_with("mantle-project.ncl"), "{:?}", plan.roots);

        let apply =
            admit_filegen_action(&filegen_action(&["apply", "--plan", "/tmp/plan.json"])).expect("applying must admit");
        assert!(!apply.dry_run);
        assert_eq!(apply.roots, plan.roots);
        assert_eq!(apply.profile, plan.profile);
    }

    #[test]
    fn an_empty_manifest_fails_closed() {
        let action = FilegenCommandAction::Plan {
            manifest: std::path::PathBuf::new(),
            plan_out: None,
        };
        let error = admit_filegen_action(&action).expect_err("an empty root must fail closed");
        assert!(error.to_string().contains("EmptyRoot"), "{error}");
        assert!(error.to_string().contains("filegen"), "{error}");
    }
}

/// What one named-root realization DTO admits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NamedRealizationAdmission {
    /// The DTO named a root, so the command maps onto the contract.
    Modeled(RealizeCommand),
    /// The DTO named no root, which the contract's root list cannot express.
    NotAdministered,
}

/// Admit `run` when it names the root it runs.
pub(crate) fn admit_run_realization(
    name: Option<&str>,
    jobs: Option<u32>,
) -> Result<NamedRealizationAdmission, RunError> {
    admit_named_realization(RUN_COMMAND_ROOT, name, jobs)
}

/// Admit `shell` when it names the root it opens.
pub(crate) fn admit_shell_realization(
    name: Option<&str>,
    jobs: Option<u32>,
) -> Result<NamedRealizationAdmission, RunError> {
    admit_named_realization(SHELL_COMMAND_ROOT, name, jobs)
}

/// Admit `develop` when it names the root it opens.
pub(crate) fn admit_develop_realization(
    name: Option<&str>,
    jobs: Option<u32>,
) -> Result<NamedRealizationAdmission, RunError> {
    admit_named_realization(DEVELOP_COMMAND_ROOT, name, jobs)
}

/// Admit one named-root realization.
///
/// The contract's realization command carries requested root names, so a
/// command that names a root maps onto it with that name and the operator's job
/// limit. A command that names no root selects the default root, which the root
/// list cannot express, so it is not administered here rather than given a name
/// the operator never wrote.
fn admit_named_realization(
    root: &str,
    name: Option<&str>,
    jobs: Option<u32>,
) -> Result<NamedRealizationAdmission, RunError> {
    debug_assert!(!root.is_empty());
    let Some(name) = name else {
        debug_assert!(name.is_none());
        return Ok(NamedRealizationAdmission::NotAdministered);
    };
    let command = RealizeCommand {
        root: String::from(root),
        roots: vec![String::from(name)],
        profile: None,
        requested_jobs: jobs,
        dry_run: false,
    };
    let blockers = validate_realize_command(&command);
    let Some(blocker) = blockers.first() else {
        return Ok(NamedRealizationAdmission::Modeled(command));
    };
    debug_assert!(!blockers.is_empty());
    Err(RunError::Internal(format!("{root} request rejected: {blocker:?}")))
}

#[cfg(test)]
mod named_realization_tests {
    use super::*;

    #[test]
    fn a_named_run_maps_its_root_and_jobs() {
        let admission = admit_run_realization(Some("mantle"), Some(4)).expect("a named run must admit");
        let NamedRealizationAdmission::Modeled(command) = admission else {
            panic!("a named run must be modeled");
        };
        assert_eq!(command.root, RUN_COMMAND_ROOT);
        assert_eq!(command.roots, vec![String::from("mantle")]);
        assert_eq!(command.requested_jobs, Some(4));
        assert!(!command.dry_run);
        assert_eq!(command.profile, None);
    }

    #[test]
    fn shell_and_develop_carry_their_own_roots() {
        for (admission, expected_root) in [
            (admit_shell_realization(Some("dev-shell"), None), SHELL_COMMAND_ROOT),
            (admit_develop_realization(Some("dev-shell"), Some(2)), DEVELOP_COMMAND_ROOT),
        ] {
            let NamedRealizationAdmission::Modeled(command) = admission.expect("named roots must admit") else {
                panic!("a named root must be modeled");
            };
            assert_eq!(command.root, expected_root);
            assert_eq!(command.roots, vec![String::from("dev-shell")]);
        }
    }

    #[test]
    fn a_command_without_a_name_is_not_administered() {
        assert_eq!(
            admit_run_realization(None, None).expect("absence must admit"),
            NamedRealizationAdmission::NotAdministered
        );
        assert_eq!(
            admit_shell_realization(None, Some(8)).expect("absence must admit"),
            NamedRealizationAdmission::NotAdministered
        );
    }

    #[test]
    fn a_blank_root_name_fails_closed() {
        let error = admit_run_realization(Some("   "), None).expect_err("a blank name must fail closed");
        assert!(error.to_string().contains("EmptyRoot"), "{error}");
        assert!(error.to_string().contains(RUN_COMMAND_ROOT), "{error}");
    }
}

/// Admit the remote secret worker and return its typed command.
///
/// The worker names the manifest it resolves, the SecretSpec profile to use, and
/// the provider to use; the profile is the credential the contract's secret
/// profile operation requires.
pub(crate) fn admit_remote_secret_worker(manifest: &Path, profile: &str) -> Result<RemoteExecutionCommand, RunError> {
    let command = RemoteExecutionCommand {
        root: String::from(REMOTE_SECRET_WORKER_COMMAND_ROOT),
        operation: RemoteExecutionOperation::SecretProfile,
        subject: manifest.display().to_string(),
        declared_entries: Vec::new(),
        builder_uri: String::new(),
        secret_profile: Some(String::from(profile)),
        has_ticket: false,
    };
    debug_assert_eq!(command.root, REMOTE_SECRET_WORKER_COMMAND_ROOT);
    let blockers = validate_remote_execution(&command);
    let Some(blocker) = blockers.first() else {
        return Ok(command);
    };
    debug_assert!(!blockers.is_empty());
    Err(RunError::Internal(format!("remote-secret-worker request rejected: {blocker:?}")))
}
