//! Application-contract fixtures: family taxonomy, envelope classification,
//! and the realization family contract.

use mantle_application_contract::ApplicationBlocker;
use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::CapabilityError;
use mantle_application_contract::CommandFamily;
use mantle_application_contract::EffectId;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::RealizationBlocker;
use mantle_application_contract::RealizeCommand;
use mantle_application_contract::RealizeOutcome;
use mantle_application_contract::RealizePort;
use mantle_application_contract::classify_observations;
use mantle_application_contract::plan_effects;
use mantle_application_contract::validate_realize_command;
use mantle_rust_plan_core::BuildProfile;

/// The public command roots as the CLI declares them today.
const PUBLIC_ROOTS: &[&str] = &[
    "artifact",
    "attest",
    "bootstrap",
    "build",
    "check",
    "dependents",
    "develop",
    "doctor",
    "eval",
    "evaluator-worker",
    "evaluator-worker-fixture",
    "export",
    "filegen",
    "foreign-import",
    "graph",
    "import",
    "init",
    "list-stale",
    "log",
    "mantlepkgs",
    "nix-free-demo",
    "operator-contract",
    "receipt",
    "refactor",
    "refresh",
    "release",
    "remote",
    "remote-secret-worker",
    "run",
    "rust-cache",
    "rust-plan",
    "self-build",
    "shell",
    "show",
    "source",
    "stage0-inventory",
    "store",
    "transcript",
    "upgrade",
    "wasm-component",
    "why",
];

#[test]
fn every_public_root_maps_to_exactly_one_family() {
    for root in PUBLIC_ROOTS {
        let family = CommandFamily::of_root(root).unwrap_or_else(|| panic!("root {root} must be classified"));
        let owners: Vec<CommandFamily> =
            CommandFamily::all().into_iter().filter(|candidate| candidate.roots().contains(root)).collect();
        assert_eq!(owners, vec![family], "root {root} must have one owning family");
    }
}

#[test]
fn family_roots_are_unique_and_complete() {
    let mut roots: Vec<&str> = CommandFamily::all().into_iter().flat_map(|family| family.roots()).collect();
    let total = roots.len();
    roots.sort_unstable();
    roots.dedup();
    assert_eq!(roots.len(), total, "no root may appear in two families");
    assert_eq!(roots.len(), PUBLIC_ROOTS.len(), "taxonomy must cover every public root");
    for root in &roots {
        assert!(PUBLIC_ROOTS.contains(root), "unknown root {root} in taxonomy");
    }
}

#[test]
fn unclassified_roots_are_rejected() {
    assert_eq!(CommandFamily::of_root("nonsense"), None);
    assert_eq!(CommandFamily::of_root(""), None);
    assert_eq!(CommandFamily::all().len(), 11);
}

#[test]
fn effect_plans_are_bounded_and_typed() {
    let plan = plan_effects(CommandFamily::Realization, &["read-files", "run-process"]).expect("plan builds");
    assert_eq!(plan.effects.len(), 2);
    assert_eq!(plan.effects[0].family, CommandFamily::Realization);

    let over_bound: Vec<&str> = std::iter::repeat_n("read-files", 4_097).collect();
    assert!(plan_effects(CommandFamily::Realization, &over_bound).is_none());
}

#[test]
fn observation_classification_is_exact() {
    let plan = plan_effects(CommandFamily::Realization, &["read-files", "write-files"]).expect("plan builds");
    let succeeded = |id: &str| Observation {
        effect_id: EffectId(String::from(id)),
        status: ObservationStatus::Succeeded,
        diagnostics_code: None,
    };
    let completed = vec![succeeded("read-files"), succeeded("write-files")];
    assert_eq!(classify_observations(&plan, &completed), ApplicationOutcome::Completed);

    let mut failed = completed.clone();
    failed[0].status = ObservationStatus::Failed;
    assert_eq!(classify_observations(&plan, &failed), ApplicationOutcome::Failed { failed_effect_count: 1 });

    let mut skipped = completed.clone();
    skipped[1].status = ObservationStatus::Skipped;
    assert_eq!(classify_observations(&plan, &skipped), ApplicationOutcome::Failed { failed_effect_count: 1 });

    let mut unknown = completed.clone();
    unknown.push(succeeded("unplanned"));
    assert_eq!(classify_observations(&plan, &unknown), ApplicationOutcome::Rejected {
        unknown_effect_count: 1,
        missing_effect_count: 0
    });

    let mut duplicated = completed.clone();
    duplicated.push(succeeded("read-files"));
    assert_eq!(classify_observations(&plan, &duplicated), ApplicationOutcome::Rejected {
        unknown_effect_count: 1,
        missing_effect_count: 0
    });

    let mut missing = completed.clone();
    missing.pop();
    assert_eq!(classify_observations(&plan, &missing), ApplicationOutcome::Rejected {
        unknown_effect_count: 0,
        missing_effect_count: 1
    });
}

#[test]
fn realization_commands_are_validated_before_any_port_call() {
    let valid = RealizeCommand {
        root: String::from("build"),
        roots: vec![String::from("mantle")],
        profile: BuildProfile::Release,
        requested_jobs: Some(4),
        dry_run: false,
    };
    assert!(validate_realize_command(&valid).is_empty());

    let mut missing_roots = valid.clone();
    missing_roots.roots = Vec::new();
    assert_eq!(validate_realize_command(&missing_roots), vec![RealizationBlocker::MissingRoots]);

    let mut missing_root = valid.clone();
    missing_root.root = String::new();
    let blockers = validate_realize_command(&missing_root);
    assert!(blockers
        .iter()
        .any(|blocker| matches!(blocker, RealizationBlocker::Domain(ApplicationBlocker { code, .. }) if code == "missing-command-root")));

    let mut too_many = valid;
    too_many.roots = std::iter::repeat_n(String::from("root"), 257).collect();
    assert!(validate_realize_command(&too_many).contains(&RealizationBlocker::TooManyRoots));
}

/// Port fake: succeeds for a valid command, blocks otherwise.
struct FakeRealizePort;

impl RealizePort for FakeRealizePort {
    fn realize(&mut self, command: &RealizeCommand) -> Result<RealizeOutcome, CapabilityError> {
        let blockers = validate_realize_command(command);
        if !blockers.is_empty() {
            return Ok(RealizeOutcome::Blocked(blockers));
        }
        if command.dry_run {
            return Err(CapabilityError::new("dry-run-unsupported", "fixture port does not plan dry runs"));
        }
        Ok(RealizeOutcome::Completed(mantle_application_contract::RealizeResult {
            planned_units: 2,
            executed_units: 2,
            cache_hits: 0,
            output_identities: Vec::new(),
            receipt_preimage: None,
        }))
    }
}

#[test]
fn realization_port_reports_blocked_completed_and_capability_failures() {
    let mut port = FakeRealizePort;
    let command = RealizeCommand {
        root: String::from("build"),
        roots: vec![String::from("mantle")],
        profile: BuildProfile::Dev,
        requested_jobs: None,
        dry_run: false,
    };
    let outcome = port.realize(&command).expect("port succeeds");
    let RealizeOutcome::Completed(result) = outcome else {
        panic!("expected completion")
    };
    assert_eq!(result.planned_units, 2);

    let mut blocked = command.clone();
    blocked.roots = Vec::new();
    let outcome = port.realize(&blocked).expect("blockers are not capability failures");
    assert!(matches!(outcome, RealizeOutcome::Blocked(_)));

    let mut dry_run = command;
    dry_run.dry_run = true;
    let error = port.realize(&dry_run).expect_err("capability failure must propagate");
    assert_eq!(error.code, "dry-run-unsupported");
}
