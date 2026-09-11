//! Rust-plan core fixtures: positive planning, negative admission, and
//! observation classification.

use mantle_rust_plan_core::BuildProfile;
use mantle_rust_plan_core::DependencyFacts;
use mantle_rust_plan_core::DependencyKind;
use mantle_rust_plan_core::EffectId;
use mantle_rust_plan_core::FeatureFacts;
use mantle_rust_plan_core::FeatureReference;
use mantle_rust_plan_core::PackageFacts;
use mantle_rust_plan_core::PackageFeatureRequest;
use mantle_rust_plan_core::PackageSource;
use mantle_rust_plan_core::PackageSourceKind;
use mantle_rust_plan_core::PlanExecutionOutcome;
use mantle_rust_plan_core::PlanOutcome;
use mantle_rust_plan_core::PlanRequest;
use mantle_rust_plan_core::RustPlan;
use mantle_rust_plan_core::TargetFacts;
use mantle_rust_plan_core::TargetKind;
use mantle_rust_plan_core::UnitId;
use mantle_rust_plan_core::UnitLimitFacts;
use mantle_rust_plan_core::UnitObservation;
use mantle_rust_plan_core::UnitObservationStatus;
use mantle_rust_plan_core::build_receipt_preimage;
use mantle_rust_plan_core::classify_plan_observations;
use mantle_rust_plan_core::plan_rust_units;

fn lib_target(name: &str) -> TargetFacts {
    TargetFacts {
        name: String::from(name),
        kind: TargetKind::Lib,
        crate_types: vec![String::from("rlib")],
        required_features: Vec::new(),
    }
}

fn package(name: &str, targets: Vec<TargetFacts>, dependencies: Vec<DependencyFacts>) -> PackageFacts {
    PackageFacts {
        name: String::from(name),
        version: String::from("0.1.0"),
        source: PackageSource {
            kind: PackageSourceKind::WorkspaceMember,
            identity: String::from("workspace"),
        },
        targets,
        dependencies,
        features: Vec::new(),
        links: None,
    }
}

fn normal_dependency(package: &str) -> DependencyFacts {
    DependencyFacts {
        key: String::from(package),
        package: String::from(package),
        version_requirement: String::from("^0.1"),
        kind: DependencyKind::Normal,
        optional: false,
        target_predicate: None,
        features: Vec::new(),
        default_features: true,
        renamed: false,
    }
}

fn two_package_request() -> PlanRequest {
    PlanRequest {
        roots: vec![String::from("app")],
        packages: vec![
            package(
                "app",
                vec![lib_target("app"), TargetFacts {
                    name: String::from("app"),
                    kind: TargetKind::Bin,
                    crate_types: vec![String::from("bin")],
                    required_features: Vec::new(),
                }],
                vec![normal_dependency("core")],
            ),
            package("core", vec![lib_target("core")], Vec::new()),
        ],
        feature_requests: Vec::new(),
        profile: BuildProfile::Release,
        limits: UnitLimitFacts::default(),
    }
}

fn blocked_plan(request: &PlanRequest) -> RustPlan {
    let plan = plan_rust_units(request);
    assert_eq!(plan.outcome, PlanOutcome::Blocked);
    assert!(plan.units.is_empty());
    assert!(plan.effects.is_empty());
    assert!(!plan.blockers.is_empty());
    plan
}

fn blocker_codes(plan: &RustPlan) -> Vec<&str> {
    plan.blockers.iter().map(|blocker| blocker.code.as_str()).collect()
}

#[test]
fn root_and_dependency_units_are_planned_with_ordered_effects() {
    let plan = plan_rust_units(&two_package_request());
    assert_eq!(plan.outcome, PlanOutcome::Completed);
    assert!(plan.blockers.is_empty());
    assert_eq!(plan.units.len(), 3);
    assert_eq!(plan.effects.len(), plan.units.len());
    let unit_ids: Vec<&str> = plan.units.iter().map(|unit| unit.unit_id.0.as_str()).collect();
    assert!(unit_ids.contains(&"core@0.1.0:core:lib"));
    assert!(unit_ids.contains(&"app@0.1.0:app:bin"));
    let mut ordered: Vec<&str> = plan.effects.iter().map(|effect| effect.effect_id.0.as_str()).collect();
    let sorted = {
        let mut clone = ordered.clone();
        clone.sort_unstable();
        clone
    };
    assert_eq!(ordered, sorted);
    ordered.dedup();
    assert_eq!(ordered.len(), plan.effects.len());
}

#[test]
fn dependency_targets_are_planned_for_the_root_only() {
    let mut request = two_package_request();
    request.packages[1].targets.push(TargetFacts {
        name: String::from("core-bin"),
        kind: TargetKind::Bin,
        crate_types: vec![String::from("bin")],
        required_features: Vec::new(),
    });
    let plan = plan_rust_units(&request);
    assert_eq!(plan.outcome, PlanOutcome::Completed);
    let unit_ids: Vec<&str> = plan.units.iter().map(|unit| unit.unit_id.0.as_str()).collect();
    assert!(!unit_ids.contains(&"core@0.1.0:core-bin:bin"));
    assert!(unit_ids.contains(&"app@0.1.0:app:bin"));
}

#[test]
fn planning_is_deterministic_and_identified() {
    let first = plan_rust_units(&two_package_request());
    let second = plan_rust_units(&two_package_request());
    assert_eq!(first, second);
    assert_eq!(first.plan_blake3.as_str().len(), 64);
    assert_eq!(first.schema, "mantle-rust-plan-v1");
    let preimage = build_receipt_preimage(&first).expect("completed plan has a receipt preimage");
    assert_eq!(preimage.unit_count, u32::try_from(first.units.len()).expect("fits"));
    assert_eq!(preimage.schema, "mantle-rust-plan-receipt-preimage-v1");
}

#[test]
fn blocked_plans_have_no_receipt_preimage() {
    let mut request = two_package_request();
    request.roots = Vec::new();
    let plan = blocked_plan(&request);
    assert_eq!(blocker_codes(&plan), vec!["missing-roots"]);
    assert!(build_receipt_preimage(&plan).is_none());
}

#[test]
fn ambiguous_package_names_are_blocked() {
    let mut request = two_package_request();
    let mut second_version = request.packages[1].clone();
    second_version.version = String::from("0.2.0");
    request.packages.push(second_version);
    let plan = blocked_plan(&request);
    assert!(blocker_codes(&plan).contains(&"ambiguous-package-name"));
}

#[test]
fn missing_dependency_packages_are_blocked() {
    let mut request = two_package_request();
    request.packages.pop();
    let plan = blocked_plan(&request);
    assert!(blocker_codes(&plan).contains(&"missing-dependency-package"));
}

#[test]
fn duplicate_and_mismatched_targets_are_blocked() {
    let mut duplicate = two_package_request();
    duplicate.packages[0].targets.push(lib_target("app"));
    let plan = blocked_plan(&duplicate);
    assert!(blocker_codes(&plan).contains(&"duplicate-target"));

    let mut mismatch = two_package_request();
    mismatch.packages[1].targets = vec![TargetFacts {
        name: String::from("core"),
        kind: TargetKind::Lib,
        crate_types: vec![String::from("proc-macro")],
        required_features: Vec::new(),
    }];
    let plan = blocked_plan(&mismatch);
    assert!(blocker_codes(&plan).contains(&"proc-macro-mismatch"));
}

#[test]
fn malformed_feature_and_dependency_facts_are_blocked() {
    let mut unknown_feature = two_package_request();
    unknown_feature.packages[0].features = vec![FeatureFacts {
        name: String::from("extra"),
        enables: vec![FeatureReference::Feature(String::from("missing"))],
    }];
    let plan = blocked_plan(&unknown_feature);
    assert!(blocker_codes(&plan).contains(&"unknown-feature-reference"));

    let mut unknown_optional = two_package_request();
    unknown_optional.packages[0].features = vec![FeatureFacts {
        name: String::from("extra"),
        enables: vec![FeatureReference::Dependency(String::from("optional-dep"))],
    }];
    let plan = blocked_plan(&unknown_optional);
    assert!(blocker_codes(&plan).contains(&"unknown-optional-dependency-reference"));

    let mut malformed_dependency = two_package_request();
    malformed_dependency.packages[0].dependencies[0].version_requirement = String::new();
    let plan = blocked_plan(&malformed_dependency);
    assert!(blocker_codes(&plan).contains(&"dependency-identity"));
}

#[test]
fn declared_unit_limits_are_enforced() {
    let mut request = two_package_request();
    request.limits = UnitLimitFacts { max_units: 1 };
    let plan = blocked_plan(&request);
    assert!(blocker_codes(&plan).contains(&"unit-limit"));
}

#[test]
fn observations_classify_completion_failure_and_drift() {
    let plan = plan_rust_units(&two_package_request());
    let completed: Vec<UnitObservation> = plan
        .effects
        .iter()
        .map(|effect| UnitObservation {
            effect_id: effect.effect_id.clone(),
            unit_id: effect.unit_id.clone(),
            status: UnitObservationStatus::Succeeded,
            exit_code: Some(0),
            diagnostics_code: None,
        })
        .collect();
    assert_eq!(classify_plan_observations(&plan, &completed), PlanExecutionOutcome::Completed);

    let mut failing = completed.clone();
    failing[0].status = UnitObservationStatus::Failed;
    failing[0].exit_code = Some(101);
    assert_eq!(classify_plan_observations(&plan, &failing), PlanExecutionOutcome::Failed { failed_effect_count: 1 });

    let mut skipped = completed.clone();
    skipped[0].status = UnitObservationStatus::Skipped;
    assert_eq!(classify_plan_observations(&plan, &skipped), PlanExecutionOutcome::Failed { failed_effect_count: 1 });

    let unknown = UnitObservation {
        effect_id: EffectId(String::from("effect:unknown")),
        unit_id: UnitId(String::from("unknown@0.1.0:unknown:lib")),
        status: UnitObservationStatus::Succeeded,
        exit_code: Some(0),
        diagnostics_code: None,
    };
    let mut with_unknown = completed.clone();
    with_unknown.push(unknown);
    assert_eq!(classify_plan_observations(&plan, &with_unknown), PlanExecutionOutcome::Rejected {
        unknown_effect_count: 1,
        missing_effect_count: 0
    });

    let mut missing = completed.clone();
    missing.pop();
    assert_eq!(classify_plan_observations(&plan, &missing), PlanExecutionOutcome::Rejected {
        unknown_effect_count: 0,
        missing_effect_count: 1
    });

    let mut duplicated = completed.clone();
    duplicated.push(completed[0].clone());
    assert_eq!(classify_plan_observations(&plan, &duplicated), PlanExecutionOutcome::Rejected {
        unknown_effect_count: 1,
        missing_effect_count: 0
    });
}

#[test]
fn activated_optional_dependency_participates_in_planning() {
    // An optional dependency is invisible until a feature activates it.
    let mut request = two_package_request();
    request.packages[0].dependencies.push(DependencyFacts {
        key: String::from("extra"),
        package: String::from("extra"),
        version_requirement: String::from("^0.1"),
        kind: DependencyKind::Normal,
        optional: true,
        target_predicate: None,
        features: Vec::new(),
        default_features: true,
        renamed: false,
    });
    request.packages[0].features = vec![FeatureFacts {
        name: String::from("with-extra"),
        enables: vec![FeatureReference::Dependency(String::from("extra"))],
    }];

    let inactive = plan_rust_units(&request);
    assert_eq!(inactive.outcome, PlanOutcome::Completed);

    request.feature_requests = vec![PackageFeatureRequest {
        package: String::from("app"),
        features: vec![String::from("with-extra")],
        include_default_features: false,
    }];
    let activated = blocked_plan(&request);
    assert!(blocker_codes(&activated).contains(&"missing-dependency-package"));
}

#[test]
fn unknown_feature_requests_are_blocked() {
    let mut request = two_package_request();
    request.feature_requests = vec![PackageFeatureRequest {
        package: String::from("app"),
        features: vec![String::from("does-not-exist")],
        include_default_features: false,
    }];
    let plan = blocked_plan(&request);
    assert!(blocker_codes(&plan).contains(&"unknown-feature-request"));
}

#[test]
fn target_required_features_gate_units() {
    let mut request = two_package_request();
    request.packages[1].features = vec![FeatureFacts {
        name: String::from("simd"),
        enables: Vec::new(),
    }];
    request.packages[1].targets[0].required_features = vec![String::from("simd")];
    let gated = blocked_plan(&request);
    assert!(blocker_codes(&gated).contains(&"target-required-feature-missing"));

    request.feature_requests = vec![PackageFeatureRequest {
        package: String::from("core"),
        features: vec![String::from("simd")],
        include_default_features: false,
    }];
    let satisfied = plan_rust_units(&request);
    assert_eq!(satisfied.outcome, PlanOutcome::Completed);
    let core_unit = satisfied
        .units
        .iter()
        .find(|unit| unit.package_key.starts_with("core@"))
        .expect("core unit is planned");
    assert!(core_unit.activated_features.iter().any(|feature| feature == "simd"));
}

#[test]
fn feature_activation_changes_unit_identity_not_unit_set() {
    let mut plain = two_package_request();
    plain.packages[1].features = vec![FeatureFacts {
        name: String::from("simd"),
        enables: Vec::new(),
    }];
    let without = plan_rust_units(&plain);
    let mut with = plain.clone();
    with.feature_requests = vec![PackageFeatureRequest {
        package: String::from("core"),
        features: vec![String::from("simd")],
        include_default_features: false,
    }];
    let with_simd = plan_rust_units(&with);
    assert_eq!(without.units.len(), with_simd.units.len());
    assert_ne!(without.plan_blake3, with_simd.plan_blake3);
    let plain_core = without.units.iter().find(|unit| unit.package_key.starts_with("core@")).expect("unit");
    let simd_core = with_simd.units.iter().find(|unit| unit.package_key.starts_with("core@")).expect("unit");
    assert_ne!(plain_core.unit_blake3, simd_core.unit_blake3);
}
