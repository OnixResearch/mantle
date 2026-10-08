//! Application-contract fixtures: family taxonomy, envelope classification,
//! and the realization family contract.

use mantle_application_contract::ApplicationBlocker;
use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::CapabilityError;
use mantle_application_contract::CommandFamily;
use mantle_application_contract::EffectId;
use mantle_application_contract::EffectKind;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::EffectPlan;
use mantle_application_contract::EffectSpec;
use mantle_application_contract::ExpectedOutput;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::PlanError;
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

fn spec<'a>(id: &'a str, output: &'a str) -> EffectSpec<'a> {
    EffectSpec {
        effect_id: id,
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Items(2),
        expected_output: ExpectedOutput::Identity(output),
    }
}

#[test]
fn effect_declarations_require_independent_id_kind_and_nonzero_limit() {
    let plan =
        plan_effects(CommandFamily::Realization, &[spec("verify-paths", "verified"), spec("read-graph", "graph")])
            .expect("two read effects have unique identities");
    assert_eq!(plan.effects[0].kind, EffectKind::ReadFiles);
    assert_eq!(plan.effects[1].kind, EffectKind::ReadFiles);
    assert_eq!(plan.effects[0].family, CommandFamily::Realization);
    assert_eq!(plan_effects(CommandFamily::Realization, &[]), Err(PlanError::EmptyPlan));
    assert_eq!(
        plan_effects(CommandFamily::Realization, &[spec("duplicate", "first"), spec("duplicate", "second")]),
        Err(PlanError::DuplicateEffectId { index: 1 })
    );
    assert_eq!(
        plan_effects(CommandFamily::Realization, &[spec("", "output")]),
        Err(PlanError::EmptyEffectId { index: 0 })
    );
    assert_eq!(
        plan_effects(CommandFamily::Realization, &[spec("read-graph", "")]),
        Err(PlanError::EmptyExpectedOutput { index: 0 })
    );
    let zero = EffectSpec {
        limit: EffectMeasure::Calls(0),
        ..spec("zero", "output")
    };
    assert_eq!(plan_effects(CommandFamily::Realization, &[zero]), Err(PlanError::ZeroLimit { index: 0 }));
    let over_bound = vec![spec("same", "output"); 4_097];
    assert_eq!(plan_effects(CommandFamily::Realization, &over_bound), Err(PlanError::TooManyEffects));
}

#[test]
fn reconstructed_plans_cannot_vacuously_complete() {
    assert_eq!(classify_observations(&EffectPlan { effects: Vec::new() }, &[]), ApplicationOutcome::Contradicted {
        effect_count: 1
    });
    let planned =
        plan_effects(CommandFamily::Realization, &[spec("verify-paths", "verified")]).expect("valid declaration");
    let actual = [Observation {
        effect_id: EffectId(String::from("verify-paths")),
        kind: EffectKind::ReadFiles,
        status: ObservationStatus::Succeeded,
        output: EffectOutput::Identity(String::from("verified")),
        usage: EffectMeasure::Items(1),
        diagnostics_code: None,
    }];
    let mut zero_limit = planned.clone();
    zero_limit.effects[0].limit = EffectMeasure::Items(0);
    assert_eq!(classify_observations(&zero_limit, &actual), ApplicationOutcome::Contradicted { effect_count: 1 });
    let mut mixed_families = planned;
    mixed_families.effects.push(mixed_families.effects[0].clone());
    mixed_families.effects[1].effect_id = EffectId(String::from("read-graph"));
    mixed_families.effects[1].family = CommandFamily::Planning;
    assert_eq!(classify_observations(&mixed_families, &actual), ApplicationOutcome::Contradicted {
        effect_count: 1
    });
}

#[test]
fn observation_classification_checks_actual_authority_output_and_usage() {
    let plan =
        plan_effects(CommandFamily::Realization, &[spec("verify-paths", "verified"), spec("read-graph", "graph")])
            .expect("plan builds");
    let succeeded = |id: &str, output: &str| Observation {
        effect_id: EffectId(String::from(id)),
        kind: EffectKind::ReadFiles,
        status: ObservationStatus::Succeeded,
        output: EffectOutput::Identity(String::from(output)),
        usage: EffectMeasure::Items(2),
        diagnostics_code: None,
    };
    let completed = vec![succeeded("verify-paths", "verified"), succeeded("read-graph", "graph")];
    assert_eq!(classify_observations(&plan, &completed), ApplicationOutcome::Completed);
    let mut failed = completed.clone();
    failed[0].status = ObservationStatus::Failed;
    failed[0].output = EffectOutput::None;
    failed[0].usage = EffectMeasure::Items(0);
    assert_eq!(classify_observations(&plan, &failed), ApplicationOutcome::Failed { failed_effect_count: 1 });
    failed[0].output = EffectOutput::Identity(String::from("unrequested"));
    assert_eq!(classify_observations(&plan, &failed), ApplicationOutcome::Contradicted { effect_count: 1 });
    let mut skipped = completed.clone();
    skipped[1].status = ObservationStatus::Skipped;
    skipped[1].output = EffectOutput::None;
    skipped[1].usage = EffectMeasure::Items(0);
    assert_eq!(classify_observations(&plan, &skipped), ApplicationOutcome::Failed { failed_effect_count: 1 });

    let mut wrong_kind = completed.clone();
    wrong_kind[0].kind = EffectKind::ReadRandom;
    assert_eq!(classify_observations(&plan, &wrong_kind), ApplicationOutcome::Contradicted { effect_count: 1 });
    let mut wrong_output = completed.clone();
    wrong_output[1].output = EffectOutput::Identity(String::from("unrequested"));
    assert_eq!(classify_observations(&plan, &wrong_output), ApplicationOutcome::Contradicted { effect_count: 1 });
    let mut overlimit = completed.clone();
    overlimit[0].usage = EffectMeasure::Items(3);
    assert_eq!(classify_observations(&plan, &overlimit), ApplicationOutcome::Contradicted { effect_count: 1 });
    let mut wrong_dimension = completed.clone();
    wrong_dimension[0].usage = EffectMeasure::Calls(1);
    assert_eq!(classify_observations(&plan, &wrong_dimension), ApplicationOutcome::Contradicted { effect_count: 1 });

    let mut unknown = completed.clone();
    unknown.push(succeeded("unplanned", "none"));
    assert_eq!(classify_observations(&plan, &unknown), ApplicationOutcome::Rejected {
        unknown_effect_count: 1,
        missing_effect_count: 0
    });
    let mut duplicated = completed.clone();
    duplicated.push(succeeded("verify-paths", "verified"));
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
        profile: Some(BuildProfile::Release),
        requested_jobs: Some(4),
        dry_run: false,
    };
    assert!(validate_realize_command(&valid).is_empty());

    let mut missing_roots = valid.clone();
    missing_roots.roots = Vec::new();
    assert_eq!(validate_realize_command(&missing_roots), vec![RealizationBlocker::MissingRoots]);

    let mut blank_root = valid.clone();
    blank_root.roots = vec![String::from("   ")];
    assert_eq!(validate_realize_command(&blank_root), vec![RealizationBlocker::EmptyRoot]);

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
        }))
    }
}

#[test]
fn realization_port_reports_blocked_completed_and_capability_failures() {
    let mut port = FakeRealizePort;
    let command = RealizeCommand {
        root: String::from("build"),
        roots: vec![String::from("mantle")],
        profile: Some(BuildProfile::Dev),
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
