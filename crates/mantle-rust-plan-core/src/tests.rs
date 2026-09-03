use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use pretty_assertions::assert_eq;
use serde::Deserialize;

const SUCCESS_DIGEST_BYTE: char = 'a';
const SECOND_DIGEST_BYTE: char = 'b';
const THIRD_DIGEST_BYTE: char = 'c';
const SUCCESS_STATUS_CODE: i32 = 0;
const FAILURE_STATUS_CODE: i32 = 9;
const EXPECTED_VALID_UNIT_COUNT: usize = 5;

#[derive(Deserialize)]
struct NativeUnitIdentityFixture {
    package_id: String,
    target_name: String,
    target_kind: String,
    mode: String,
    profile: String,
    source_algorithm: String,
    source_value: String,
    features: Vec<String>,
    dependencies: Vec<(String, String)>,
    expected: String,
}

#[derive(Deserialize)]
struct CompatibilityFixture {
    cases: Vec<CompatibilityCase>,
}

#[derive(Deserialize)]
struct CompatibilityCase {
    no_cargo_oracle: bool,
    blockers: Vec<String>,
    class: String,
    status: String,
    surface_ids: Vec<String>,
}

#[derive(Deserialize)]
struct ReceiptFixture {
    preimage: crate::RustPlanReceiptPreimage,
    expected_blake3: String,
}

fn digest(byte: char) -> String {
    core::iter::repeat_n(byte, crate::BLAKE3_HEX_CHARS).collect()
}

fn target(name: &str, kind: crate::TargetKind, byte: char) -> crate::TargetFact {
    crate::TargetFact {
        name: name.to_string(),
        crate_name: crate::rust_crate_name(name),
        kind,
        crate_types: vec![kind.label().to_string()],
        source_identity: digest(byte),
        edition: "2024".to_string(),
    }
}

fn package(
    package_id: &str,
    workspace_member: bool,
    targets: Vec<crate::TargetFact>,
    dependencies: Vec<crate::DependencyFact>,
    features: Vec<crate::FeatureFact>,
    byte: char,
) -> crate::PackageFact {
    crate::PackageFact {
        package_id: package_id.to_string(),
        name: package_id.to_string(),
        version: "1.0.0".to_string(),
        manifest_identity: digest(byte),
        source_identity: digest(byte),
        workspace_member,
        targets,
        dependencies,
        features,
    }
}

fn planning_input() -> crate::RustPlanningInput {
    let util = package(
        "util",
        false,
        vec![target("util", crate::TargetKind::Library, SECOND_DIGEST_BYTE)],
        Vec::new(),
        Vec::new(),
        SECOND_DIGEST_BYTE,
    );
    let proc_macro = package(
        "macro-kit",
        false,
        vec![target("macro-kit", crate::TargetKind::ProcMacro, THIRD_DIGEST_BYTE)],
        Vec::new(),
        Vec::new(),
        THIRD_DIGEST_BYTE,
    );
    let extra =
        package("extra", false, vec![target("extra", crate::TargetKind::Library, 'd')], Vec::new(), Vec::new(), 'd');
    let app = package(
        "app",
        true,
        vec![
            target("build-script-build", crate::TargetKind::BuildScript, SUCCESS_DIGEST_BYTE),
            target("app", crate::TargetKind::Library, SUCCESS_DIGEST_BYTE),
            target("app-cli", crate::TargetKind::Binary, SUCCESS_DIGEST_BYTE),
        ],
        vec![
            crate::DependencyFact {
                package_id: "util".to_string(),
                role: crate::DependencyRole::Normal,
                optional: false,
                activates_on: Vec::new(),
            },
            crate::DependencyFact {
                package_id: "macro-kit".to_string(),
                role: crate::DependencyRole::Build,
                optional: false,
                activates_on: Vec::new(),
            },
            crate::DependencyFact {
                package_id: "extra".to_string(),
                role: crate::DependencyRole::Normal,
                optional: true,
                activates_on: vec!["extra".to_string()],
            },
        ],
        vec![
            crate::FeatureFact {
                name: "default".to_string(),
                activations: vec![crate::FeatureActivation::Feature {
                    name: "fast".to_string(),
                }],
            },
            crate::FeatureFact {
                name: "fast".to_string(),
                activations: Vec::new(),
            },
            crate::FeatureFact {
                name: "extra".to_string(),
                activations: vec![crate::FeatureActivation::Dependency {
                    package_id: "extra".to_string(),
                }],
            },
        ],
        SUCCESS_DIGEST_BYTE,
    );
    crate::RustPlanningInput {
        workspace: crate::StructuralWorkspaceFacts {
            schema: crate::WORKSPACE_FACTS_SCHEMA.to_string(),
            workspace_identity: digest(SUCCESS_DIGEST_BYTE),
            packages: vec![app, util, proc_macro, extra],
        },
        toolchain: crate::ToolchainFact {
            compiler_identity: digest(SECOND_DIGEST_BYTE),
            compiler_version_identity: digest(THIRD_DIGEST_BYTE),
            host_triple: "x86_64-unknown-linux-gnu".to_string(),
            target_triple: "wasm32-unknown-unknown".to_string(),
        },
        oracle: None,
    }
}

fn request() -> crate::RustPlanRequest {
    crate::RustPlanRequest {
        workspace_members: vec!["app".to_string()],
        features: Vec::new(),
        all_features: false,
        no_default_features: false,
        profile: "release".to_string(),
    }
}

fn plan() -> crate::RustPlan {
    let admitted = crate::admit_workspace(planning_input(), request()).unwrap();
    crate::plan_workspace(admitted).unwrap()
}

#[test]
fn plans_package_feature_host_target_and_effect_topology() {
    let plan = plan();
    assert!(plan.receipt_preimage.blockers.is_empty());
    assert_eq!(plan.receipt_preimage.selected_package_ids, vec!["app", "macro-kit", "util"]);
    assert_eq!(plan.receipt_preimage.selected_features["app"], vec!["default", "fast"]);
    assert_eq!(plan.receipt_preimage.units.len(), EXPECTED_VALID_UNIT_COUNT);
    assert!(plan.receipt_preimage.units.iter().any(|unit| {
        unit.target_kind == crate::TargetKind::ProcMacro && unit.execution_kind == crate::ExecutionKind::Host
    }));
    assert!(plan.receipt_preimage.units.iter().all(|unit| !unit.effect.arguments.is_empty()));
    assert!(plan.receipt_preimage.units.iter().all(|unit| !unit.effect.expected_outputs.is_empty()));
}

#[test]
fn equivalent_collection_orders_produce_one_receipt() {
    let first = plan();
    let mut reordered = planning_input();
    reordered.workspace.packages.reverse();
    for package in &mut reordered.workspace.packages {
        package.targets.reverse();
        package.dependencies.reverse();
        package.features.reverse();
    }
    let second = crate::plan_workspace(crate::admit_workspace(reordered, request()).unwrap()).unwrap();
    assert_eq!(first.receipt_blake3, second.receipt_blake3);
    assert_eq!(first.receipt_preimage, second.receipt_preimage);
}

#[test]
fn all_features_select_optional_dependency() {
    let mut request = request();
    request.all_features = true;
    let admitted = crate::admit_workspace(planning_input(), request).unwrap();
    assert!(admitted.selected_package_ids.contains(&"extra".to_string()));
    assert!(admitted.selected_features["app"].contains(&"extra".to_string()));
}

#[test]
fn no_default_features_excludes_default_closure() {
    let mut request = request();
    request.no_default_features = true;
    let admitted = crate::admit_workspace(planning_input(), request).unwrap();
    assert!(admitted.selected_features["app"].is_empty());
    assert_eq!(admitted.selected_package_ids, vec!["app", "macro-kit", "util"]);
}

#[test]
fn target_classification_preserves_cargo_proc_macro_precedence() {
    let kinds = vec!["lib".to_string()];
    let crate_types = vec!["proc-macro".to_string()];
    assert_eq!(crate::classify_target_kind(&kinds, &crate_types), Some("proc-macro"));
    assert!(crate::target_kind_uses_host("proc-macro"));
    assert!(!crate::target_kind_uses_host("lib"));
}

#[test]
fn compatibility_matches_retained_status_matrix() {
    let blocked = vec!["unsupported-target".to_string()];
    assert_eq!(crate::compatibility_status(false, &[]), "oracle");
    assert_eq!(crate::compatibility_status(true, &[]), "supported");
    assert_eq!(crate::compatibility_status(true, &blocked), "blocked");
    assert_eq!(crate::compatibility_class(false, &[]), "cargo-oracle-evidence");
    assert_eq!(crate::compatibility_class(true, &[]), "cargo-free-bounded-topology");
}

#[test]
fn duplicate_package_ids_fail_admission() {
    let mut input = planning_input();
    input.workspace.packages.push(input.workspace.packages[0].clone());
    let error = crate::admit_workspace(input, request()).unwrap_err();
    assert!(matches!(error, crate::RustPlanCoreError::AmbiguousReference {
        code: "duplicate-package-id",
        ..
    }));
}

#[test]
fn missing_dependency_fails_admission() {
    let mut input = planning_input();
    input.workspace.packages.retain(|package| package.package_id != "util");
    let error = crate::admit_workspace(input, request()).unwrap_err();
    assert!(matches!(error, crate::RustPlanCoreError::MissingReference {
        code: "dependency-package",
        ..
    }));
}

#[test]
fn unknown_requested_feature_fails_admission() {
    let mut request = request();
    request.features.push(crate::FeatureRequest {
        package_id: "app".to_string(),
        feature: "unknown".to_string(),
    });
    let error = crate::admit_workspace(planning_input(), request).unwrap_err();
    assert!(matches!(error, crate::RustPlanCoreError::MissingReference {
        code: "requested-feature",
        ..
    }));
}

#[test]
fn cyclic_package_topology_returns_fail_closed_blocker() {
    let mut input = planning_input();
    let util = input.workspace.packages.iter_mut().find(|package| package.package_id == "util").unwrap();
    util.dependencies.push(crate::DependencyFact {
        package_id: "app".to_string(),
        role: crate::DependencyRole::Normal,
        optional: false,
        activates_on: Vec::new(),
    });
    let admitted = crate::admit_workspace(input, request()).unwrap();
    let plan = crate::plan_workspace(admitted).unwrap();
    assert!(plan.receipt_preimage.blockers.iter().any(|blocker| blocker.class == "unit-topology-cycle"));
    assert!(plan.receipt_preimage.units.len() < EXPECTED_VALID_UNIT_COUNT);
}

#[test]
fn successful_observation_produces_bound_receipt() {
    let effect = plan().receipt_preimage.units[0].effect.clone();
    let artifact = crate::ObservedArtifact {
        name: effect.expected_outputs[0].clone(),
        digest_blake3: digest(SUCCESS_DIGEST_BYTE),
    };
    let outcome = crate::classify_unit_observation(&effect, crate::RustUnitObservation {
        effect_id: effect.effect_id.clone(),
        status_code: SUCCESS_STATUS_CODE,
        stdout_blake3: digest(SECOND_DIGEST_BYTE),
        stderr_blake3: digest(THIRD_DIGEST_BYTE),
        stdout_bytes: 0,
        stderr_bytes: 0,
        artifacts: vec![artifact],
    })
    .unwrap();
    assert_eq!(outcome.status, "success");
    assert_eq!(outcome.receipt_blake3.len(), crate::BLAKE3_HEX_CHARS);
}

#[test]
fn failed_observation_is_a_stable_outcome() {
    let effect = plan().receipt_preimage.units[0].effect.clone();
    let outcome = crate::classify_unit_observation(&effect, crate::RustUnitObservation {
        effect_id: effect.effect_id.clone(),
        status_code: FAILURE_STATUS_CODE,
        stdout_blake3: digest(SECOND_DIGEST_BYTE),
        stderr_blake3: digest(THIRD_DIGEST_BYTE),
        stdout_bytes: 0,
        stderr_bytes: 0,
        artifacts: Vec::new(),
    })
    .unwrap();
    assert_eq!(outcome.status, "failed");
    assert_eq!(outcome.blocker.unwrap().class, "compiler-execution-failed");
}

#[test]
fn wrong_effect_observation_fails_closed() {
    let effect = plan().receipt_preimage.units[0].effect.clone();
    let error = crate::classify_unit_observation(&effect, crate::RustUnitObservation {
        effect_id: digest(SECOND_DIGEST_BYTE),
        status_code: SUCCESS_STATUS_CODE,
        stdout_blake3: digest(SECOND_DIGEST_BYTE),
        stderr_blake3: digest(THIRD_DIGEST_BYTE),
        stdout_bytes: 0,
        stderr_bytes: 0,
        artifacts: Vec::new(),
    })
    .unwrap_err();
    assert!(matches!(error, crate::RustPlanCoreError::ObservationMismatch {
        code: "effect-identity",
        ..
    }));
}

#[test]
fn cache_fault_blocks_without_execution_fallback() {
    let effect = plan().receipt_preimage.units[0].effect.clone();
    let decision = crate::classify_cache_observation(&effect, crate::CacheObservation {
        effect_id: effect.effect_id.clone(),
        status: crate::CacheObservationStatus::Failed,
        artifacts: Vec::new(),
        failure_code: Some("cache-offline".to_string()),
    })
    .unwrap();
    assert!(matches!(decision, crate::CacheDecision::Block { .. }));
}

#[test]
fn observation_byte_limit_is_enforced() {
    let effect = plan().receipt_preimage.units[0].effect.clone();
    let error = crate::classify_unit_observation(&effect, crate::RustUnitObservation {
        effect_id: effect.effect_id.clone(),
        status_code: FAILURE_STATUS_CODE,
        stdout_blake3: digest(SECOND_DIGEST_BYTE),
        stderr_blake3: digest(THIRD_DIGEST_BYTE),
        stdout_bytes: effect.limits.stdout_bytes_max.saturating_add(1),
        stderr_bytes: 0,
        artifacts: Vec::new(),
    })
    .unwrap_err();
    assert!(matches!(error, crate::RustPlanCoreError::ObservationMismatch {
        code: "stdout-limit",
        ..
    }));
}

#[test]
fn unit_effect_environment_is_declared_only() {
    let effect = plan().receipt_preimage.units[0].effect.clone();
    assert_eq!(effect.environment.len(), crate::UNIT_ENVIRONMENT_ENTRY_COUNT);
    assert_eq!(effect.environment.get("PROFILE"), Some(&"release".to_string()));
    assert!(!effect.environment.contains_key("RUSTFLAGS"));
    assert_eq!(effect.environment.keys().cloned().collect::<Vec<_>>(), {
        let mut keys = vec!["HOST", "MANTLE_RUST_UNIT_EXECUTION_KIND", "PROFILE", "TARGET"];
        keys.sort();
        keys.into_iter().map(str::to_string).collect::<Vec<_>>()
    });
}

#[test]
fn receipt_json_is_canonical_for_btree_environment() {
    let first = plan();
    let second = plan();
    assert_eq!(first.receipt_blake3, second.receipt_blake3);
    assert_eq!(
        first.receipt_preimage.selected_features,
        BTreeMap::from([
            ("app".to_string(), vec!["default".to_string(), "fast".to_string()]),
            ("macro-kit".to_string(), Vec::new()),
            ("util".to_string(), Vec::new()),
        ])
    );
}

#[test]
fn native_unit_identity_matches_golden_legacy_preimage() {
    let fixture: NativeUnitIdentityFixture =
        serde_json::from_str(include_str!("../../../fixtures/rust-plan-hexagon/golden/native-unit-identity.json"))
            .unwrap();
    let identity = crate::native_unit_identity(&crate::NativeUnitIdentityInput {
        package_id: fixture.package_id,
        target_name: fixture.target_name,
        target_kind: fixture.target_kind,
        mode: fixture.mode,
        profile: fixture.profile,
        source_algorithm: fixture.source_algorithm,
        source_value: fixture.source_value,
        features: fixture.features,
        dependencies: fixture.dependencies,
    });
    assert_eq!(identity, fixture.expected);
    assert!(identity.starts_with("native:"));
}

#[test]
fn cargo_oracle_fixture_is_normalized_before_core_use() {
    let oracle: crate::CargoOracleFacts =
        serde_json::from_str(include_str!("../../../fixtures/rust-plan-hexagon/golden/cargo-oracle-facts.json"))
            .unwrap();
    let mut input = planning_input();
    input.oracle = Some(oracle);
    let admitted = crate::admit_workspace(input, request()).unwrap();
    let normalized = admitted.oracle.unwrap();
    assert_eq!(normalized.package_ids, vec!["app", "util"]);
    assert_eq!(normalized.unit_ids, vec!["app:lib", "util:lib"]);
}

#[test]
fn compatibility_matrix_matches_golden_legacy_outputs() {
    let fixture: CompatibilityFixture =
        serde_json::from_str(include_str!("../../../fixtures/rust-plan-hexagon/golden/compatibility.json")).unwrap();
    for case in fixture.cases {
        assert_eq!(crate::compatibility_class(case.no_cargo_oracle, &case.blockers), case.class);
        assert_eq!(crate::compatibility_status(case.no_cargo_oracle, &case.blockers), case.status);
        assert_eq!(crate::compatibility_surface_ids(case.no_cargo_oracle, &case.blockers), case.surface_ids);
    }
}

#[test]
fn receipt_preimage_matches_golden_identity() {
    let fixture: ReceiptFixture =
        serde_json::from_str(include_str!("../../../fixtures/rust-plan-hexagon/golden/receipt-preimage.json")).unwrap();
    let actual = crate::receipt_preimage_identity(&fixture.preimage).unwrap();
    assert_eq!(actual, fixture.expected_blake3);
    assert_eq!(actual.len(), crate::BLAKE3_HEX_CHARS);
}
