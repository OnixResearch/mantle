//! Build-script execution units, shared target facts, and `flags` propagation.

mod support;

use std::collections::BTreeMap;

use mantle_cargo_unit_plan::BuildScriptRunBinding;
use mantle_cargo_unit_plan::ExternBinding;
use mantle_cargo_unit_plan::PlanBindings;
use mantle_cargo_unit_plan::SourceSlice;
use mantle_cargo_unit_plan::UnitBinding;
use mantle_cargo_unit_plan::lower;
use mantle_rust_plan_core::ExistingUnitEffect;
use mantle_rust_plan_core::ExistingUnitFacts;
use mantle_rust_plan_core::plan_existing_unit_effects;
use serde_json::Value;
use serde_json::json;
use support::refresh_wait_units;

const PATH_HASH: &str = "01234567890123456789012345678901";
const SOURCE_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TRIPLE: &str = "x86_64-unknown-linux-gnu";

fn store(name: &str) -> String {
    format!("/mantle/store/{PATH_HASH}-{name}")
}

fn wire(id: &str) -> String {
    format!("u.{}", blake3::hash(id.as_bytes()).to_hex())
}

fn token(id: &str, output: &str) -> String {
    format!("{{{{mantle-unit-output:{}:{output}}}}}", wire(id))
}

struct Spec {
    id: &'static str,
    package: &'static str,
    kind: &'static str,
    execution: &'static str,
    deps: &'static [&'static str],
}

/// leaf (links = "leaf", build script) <- mid (build script reading DEP_LEAF_*) <- app.
const GRAPH: &[Spec] = &[
    Spec {
        id: "leaf-script",
        package: "leaf",
        kind: "custom-build",
        execution: "host",
        deps: &[],
    },
    Spec {
        id: "leaf-run",
        package: "leaf",
        kind: "custom-build",
        execution: "build-script-run",
        deps: &["leaf-script"],
    },
    Spec {
        id: "leaf",
        package: "leaf",
        kind: "lib",
        execution: "target",
        deps: &["leaf-run"],
    },
    Spec {
        id: "mid-script",
        package: "mid",
        kind: "custom-build",
        execution: "host",
        deps: &[],
    },
    Spec {
        id: "mid-run",
        package: "mid",
        kind: "custom-build",
        execution: "build-script-run",
        deps: &["mid-script", "leaf-run"],
    },
    Spec {
        id: "mid",
        package: "mid",
        kind: "lib",
        execution: "target",
        deps: &["leaf", "mid-run"],
    },
    Spec {
        id: "app",
        package: "app",
        kind: "bin",
        execution: "target",
        deps: &["mid"],
    },
    Spec {
        id: "other",
        package: "other",
        kind: "lib",
        execution: "target",
        deps: &[],
    },
];

fn is_run(spec: &Spec) -> bool {
    spec.execution == "build-script-run"
}

fn effects() -> Vec<ExistingUnitEffect> {
    let facts = GRAPH
        .iter()
        .enumerate()
        .map(|(rank, spec)| ExistingUnitFacts {
            unit_id: spec.id.into(),
            package_id: format!("path+file:///fixture/{0}#{0}@0.1.0", spec.package),
            target_name: if spec.kind == "custom-build" {
                "build-script-build".into()
            } else {
                spec.package.into()
            },
            target_kind: spec.kind.into(),
            execution_kind: spec.execution.into(),
            dependency_unit_ids: spec.deps.iter().map(ToString::to_string).collect(),
            arguments: if is_run(spec) {
                vec!["run-build-script".into()]
            } else {
                vec!["--crate-type=rlib".into(), "--edition=2024".into()]
            },
            environment: vec![("CARGO_PKG_NAME".into(), spec.package.into())],
            input_identities: vec![format!("source:{}", spec.package)],
            expected_outputs: if is_run(spec) {
                vec!["out".into(), "flags".into()]
            } else {
                vec!["out".into()]
            },
            execution_order: u32::try_from(rank).unwrap(),
        })
        .collect();
    plan_existing_unit_effects(facts).unwrap()
}

fn run_binding(links: Option<&str>) -> BuildScriptRunBinding {
    BuildScriptRunBinding {
        links: links.map(Into::into),
        ..BuildScriptRunBinding::default()
    }
}

fn bindings() -> PlanBindings {
    let mut sources = Vec::new();
    for package in ["leaf", "mid", "app", "other"] {
        sources.push(SourceSlice {
            id: package.into(),
            producer_output: "sources".into(),
            subpath: package.into(),
            store_name: format!("source-{package}"),
            nar_blake3: SOURCE_DIGEST.into(),
        });
    }
    let units = GRAPH
        .iter()
        .map(|spec| UnitBinding {
            unit_id: spec.id.into(),
            source_id: spec.package.into(),
            source_kind: "path".into(),
            source_entry: if spec.kind == "custom-build" {
                "build.rs".into()
            } else {
                "src/lib.rs".into()
            },
            source_label: format!("{}-0.1.0", spec.package),
            rustc_metadata_hash: "abcdef1234".into(),
            triple: TRIPLE.into(),
            externs: match spec.id {
                "mid" => vec![ExternBinding {
                    name: "leaf".into(),
                    unit_id: "leaf".into(),
                }],
                "app" => vec![ExternBinding {
                    name: "mid".into(),
                    unit_id: "mid".into(),
                }],
                _ => Vec::new(),
            },
            build_script_run: match spec.id {
                "leaf-run" => Some(run_binding(Some("leaf"))),
                "mid-run" => Some(run_binding(None)),
                _ => None,
            },
        })
        .collect();
    let mut plan = PlanBindings {
        store_prefix: "/mantle/store".into(),
        system: "x86_64-linux".into(),
        target_triple: TRIPLE.into(),
        host_triple: TRIPLE.into(),
        helper: store("unit-helper"),
        rustc: format!("{}/bin/rustc", store("rust-toolchain")),
        toolchain: store("oracle-toolchain"),
        oracle_inputs: vec![store("oracle-toolchain")],
        linker: format!("{}/bin/cc", store("linker")),
        target_linker: None,
        runtime_inputs: Vec::new(),
        wait_units: BTreeMap::new(),
        sources,
        units,
        roots: vec!["app".into()],
    };
    refresh_wait_units(&mut plan, &effects());
    plan
}

#[test]
fn producer_oracle_roots_cannot_be_reintroduced_as_execution_capabilities() {
    for role in ["compiler", "helper", "linker", "runtime", "native", "target-linker"] {
        let mut plan = bindings();
        let oracle = plan.oracle_inputs[0].clone();
        match role {
            "compiler" => plan.rustc = format!("{oracle}/bin/rustc"),
            "helper" => plan.helper = format!("{oracle}/bin/helper"),
            "linker" => plan.linker = format!("{oracle}/bin/cc"),
            "runtime" => plan.runtime_inputs.push(oracle.clone()),
            "native" => {
                let run = binding_mut(&mut plan, "leaf-run").build_script_run.as_mut().unwrap();
                run.native_inputs.push(oracle.clone());
            }
            "target-linker" => {
                plan.target_linker = Some(mantle_cargo_unit_plan::TargetLinkerBinding {
                    executable: format!("{oracle}/bin/ld"),
                    arguments: Vec::new(),
                    inputs: vec![oracle.clone()],
                });
            }
            _ => unreachable!(),
        }
        let blockers = lower(&effects(), &plan).unwrap_err();
        assert_eq!(blockers[0].code, "unit-plan-oracle-execution-overlap", "{role}: {blockers:?}");
        assert_eq!(blockers[0].detail, oracle, "{role}");
    }
}

fn binding_mut<'a>(plan: &'a mut PlanBindings, id: &str) -> &'a mut UnitBinding {
    plan.units.iter_mut().find(|unit| unit.unit_id == id).unwrap()
}

fn lowered(plan: &PlanBindings) -> Value {
    let (bytes, _) = lower(&effects(), plan).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn unit<'a>(plan: &'a Value, id: &str) -> &'a Value {
    plan["units"].as_array().unwrap().iter().find(|unit| unit["id"] == wire(id)).unwrap()
}

fn env<'a>(unit: &'a Value, key: &str) -> &'a str {
    unit["derivation"]["env"][key].as_str().unwrap_or_else(|| panic!("missing {key}"))
}

fn json_env(unit: &Value, key: &str) -> Value {
    serde_json::from_str(env(unit, key)).unwrap()
}

fn has_input(unit: &Value, expected: &Value) -> bool {
    unit["derivation"]["inputs"].as_array().unwrap().contains(expected)
}

fn target_fact_units(plan: &Value) -> Vec<&Value> {
    plan["units"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|unit| unit["derivation"]["args"] == json!(["target-facts"]))
        .collect()
}

#[test]
fn two_build_scripts_share_one_target_description_unit() {
    let plan = lowered(&bindings());
    let facts = target_fact_units(&plan);
    assert_eq!(facts.len(), 1, "one target-description unit per toolchain and triple");
    let facts_id = facts[0]["id"].as_str().unwrap();
    assert_eq!(facts[0]["derivation"]["env"]["MANTLE_UNIT_TARGET"], TRIPLE);
    assert_eq!(facts[0]["derivation"]["env"]["MANTLE_UNIT_HOST"], TRIPLE);
    for run in ["leaf-run", "mid-run"] {
        let run_unit = unit(&plan, run);
        assert_eq!(env(run_unit, "MANTLE_UNIT_TARGET_FACTS"), format!("{{{{mantle-unit-output:{facts_id}:out}}}}"));
        assert!(has_input(run_unit, &json!({"kind": "unit_output", "unit": facts_id, "output": "out"})));
    }
    assert_eq!(plan["units"].as_array().unwrap().len(), GRAPH.len() + 1);
}

#[test]
fn toolchain_change_gives_the_target_description_unit_a_new_identity() {
    let before = lowered(&bindings());
    let mut changed = bindings();
    changed.toolchain = format!("/mantle/store/{}-oracle-toolchain-next", "1".repeat(32));
    changed.oracle_inputs = vec![changed.toolchain.clone()];
    changed.rustc = format!("{}/bin/rustc", store("rust-toolchain-next"));
    refresh_wait_units(&mut changed, &effects());
    let after = lowered(&changed);
    let (old, new) = (target_fact_units(&before)[0], target_fact_units(&after)[0]);
    assert_ne!(old["id"], new["id"]);
    assert_ne!(old["derivation"]["inputs"], new["derivation"]["inputs"]);
}

#[test]
fn execution_units_run_the_compiled_script_with_declared_outputs() {
    let plan = lowered(&bindings());
    let leaf_run = unit(&plan, "leaf-run");
    assert_eq!(leaf_run["derivation"]["args"], json!(["run-build-script"]));
    assert_eq!(leaf_run["derivation"]["outputs"], json!(["out", "flags"]));
    assert_eq!(leaf_run["requested_outputs"], json!(["out", "flags"]));
    assert_eq!(env(leaf_run, "MANTLE_UNIT_BUILD_SCRIPT"), token("leaf-script", "out"));
    assert_eq!(env(leaf_run, "MANTLE_UNIT_SOURCE"), "{{mantle-source:leaf}}");
    let script_env = json_env(leaf_run, "MANTLE_UNIT_SCRIPT_ENV");
    assert_eq!(script_env["CARGO_MANIFEST_LINKS"], "leaf");
    assert_eq!(script_env["CARGO_PKG_NAME"], "leaf");
    assert_eq!(script_env["RUSTC"], format!("{}/bin/rustc", store("rust-toolchain")));
    assert!(script_env.get("OUT_DIR").is_none() && script_env.get("TARGET").is_none());
    assert_eq!(json_env(leaf_run, "MANTLE_UNIT_SCRIPT_PATH"), json!([]));
    let leaf = unit(&plan, "leaf");
    assert_eq!(env(leaf, "MANTLE_UNIT_BUILD_SCRIPT_OUT"), token("leaf-run", "out"));
    assert_eq!(env(leaf, "MANTLE_UNIT_BUILD_SCRIPT_FLAGS"), token("leaf-run", "flags"));
    assert_eq!(env(leaf, "MANTLE_UNIT_DEPENDENCIES"), "[]", "execution units are not rustc dependencies");
    assert!(has_input(leaf, &json!({"kind": "unit_output", "unit": wire("leaf-run"), "output": "out"})));
    assert!(has_input(leaf, &json!({"kind": "unit_output", "unit": wire("leaf-run"), "output": "flags"})));
    assert_eq!(env(leaf, "MANTLE_UNIT_TARGET_KIND"), "lib");
}

#[test]
fn link_search_flags_propagate_transitively_in_topology_order() {
    let plan = lowered(&bindings());
    let app = unit(&plan, "app");
    assert_eq!(
        json_env(app, "MANTLE_UNIT_TRANSITIVE_FLAGS"),
        json!([token("leaf-run", "flags"), token("mid-run", "flags")])
    );
    assert!(app["derivation"]["env"].get("MANTLE_UNIT_BUILD_SCRIPT_FLAGS").is_none());
    for run in ["leaf-run", "mid-run"] {
        assert!(has_input(app, &json!({"kind": "unit_output", "unit": wire(run), "output": "flags"})));
        assert!(!has_input(app, &json!({"kind": "unit_output", "unit": wire(run), "output": "out"})));
    }
    let mid = unit(&plan, "mid");
    assert_eq!(json_env(mid, "MANTLE_UNIT_TRANSITIVE_FLAGS"), json!([token("leaf-run", "flags")]));
    assert_eq!(env(mid, "MANTLE_UNIT_BUILD_SCRIPT_FLAGS"), token("mid-run", "flags"));
    assert_eq!(json_env(unit(&plan, "leaf"), "MANTLE_UNIT_TRANSITIVE_FLAGS"), json!([]));
}

#[test]
fn links_metadata_reaches_the_dependent_build_script() {
    let plan = lowered(&bindings());
    let mid_run = unit(&plan, "mid-run");
    assert_eq!(json_env(mid_run, "MANTLE_UNIT_LINKS_DEPENDENCIES"), json!([token("leaf-run", "flags")]));
    assert!(has_input(mid_run, &json!({"kind": "unit_output", "unit": wire("leaf-run"), "output": "flags"})));
    assert_eq!(json_env(unit(&plan, "leaf-run"), "MANTLE_UNIT_LINKS_DEPENDENCIES"), json!([]));
    assert!(json_env(mid_run, "MANTLE_UNIT_SCRIPT_ENV").get("CARGO_MANIFEST_LINKS").is_none());
}

#[test]
fn lowering_is_independent_of_effect_and_binding_order() {
    let (bytes, digest) = lower(&effects(), &bindings()).unwrap();
    let mut reversed = effects();
    reversed.reverse();
    let mut plan = bindings();
    plan.units.reverse();
    plan.sources.reverse();
    assert_eq!(lower(&reversed, &plan).unwrap(), (bytes, digest));
}

#[test]
fn declared_native_roots_reach_the_script_and_every_compilation_that_links_its_flags() {
    let mut plan = bindings();
    let bzip2 = store("bzip2-1.0.8");
    let run = binding_mut(&mut plan, "leaf-run").build_script_run.as_mut().unwrap();
    run.native_inputs = vec![bzip2.clone(), store("pkg-config")];
    run.native_env = BTreeMap::from([("PKG_CONFIG_PATH".into(), format!("{bzip2}/lib/pkgconfig"))]);
    run.native_path = vec![format!("{}/bin", store("pkg-config"))];
    refresh_wait_units(&mut plan, &effects());
    let lowered = lowered(&plan);
    let leaf_run = unit(&lowered, "leaf-run");
    let root = json!({"kind": "store_path", "path": bzip2});
    assert!(has_input(leaf_run, &root));
    assert_eq!(json_env(leaf_run, "MANTLE_UNIT_SCRIPT_ENV")["PKG_CONFIG_PATH"], format!("{bzip2}/lib/pkgconfig"));
    assert_eq!(json_env(leaf_run, "MANTLE_UNIT_SCRIPT_PATH"), json!([format!("{}/bin", store("pkg-config"))]));
    assert_eq!(json_env(leaf_run, "MANTLE_UNIT_NATIVE_INPUTS"), json!([bzip2, store("pkg-config")]));
    // The sys package's own lib (own flags) and the dependent lib and bin
    // (transitive flags) link against the declared library, exactly once each.
    for consumer in ["leaf", "mid", "app"] {
        let inputs = unit(&lowered, consumer)["derivation"]["inputs"].as_array().unwrap();
        assert_eq!(inputs.iter().filter(|input| **input == root).count(), 1, "{consumer}");
    }
    // Units that never receive leaf's flags stay without the declared library.
    for unrelated in ["mid-run", "mid-script", "leaf-script", "other"] {
        assert!(!has_input(unit(&lowered, unrelated), &root), "{unrelated}");
    }
}

fn blocker_code(plan: &PlanBindings) -> &'static str {
    lower(&effects(), plan).unwrap_err()[0].code
}

type Mutation = fn(&mut BuildScriptRunBinding);

#[test]
fn native_inputs_reject_host_paths_and_reserved_keys() {
    let cases: [(&str, Mutation); 4] = [
        ("unit-plan-host-path", |run| run.native_inputs = vec![format!("{}/lib", store("bzip2"))]),
        ("unit-plan-host-path", |run| run.native_inputs = vec!["/usr/lib".into()]),
        ("native-input-env-reserved", |run| run.native_env = BTreeMap::from([("OUT_DIR".into(), "x".into())])),
        ("unit-plan-host-path", |run| run.native_path = vec!["/usr/bin".into()]),
    ];
    for (code, mutate) in cases {
        let mut plan = bindings();
        mutate(binding_mut(&mut plan, "leaf-run").build_script_run.as_mut().unwrap());
        assert_eq!(blocker_code(&plan), code);
    }
}

#[test]
fn duplicate_links_keys_block_lowering() {
    let mut plan = bindings();
    binding_mut(&mut plan, "mid-run").build_script_run = Some(run_binding(Some("leaf")));
    let blockers = lower(&effects(), &plan).unwrap_err();
    assert_eq!(blockers[0].code, "build-script-links-conflict");
    assert!(blockers[0].detail.contains("leaf"));
}

#[test]
fn malformed_execution_units_keep_the_build_script_blocker() {
    let mut missing_binding = bindings();
    binding_mut(&mut missing_binding, "leaf-run").build_script_run = None;
    assert_eq!(blocker_code(&missing_binding), "unit-plan-build-script-run-unsupported");
    let mut stray_binding = bindings();
    binding_mut(&mut stray_binding, "leaf").build_script_run = Some(run_binding(None));
    assert_eq!(blocker_code(&stray_binding), "unit-plan-build-script-run-unsupported");
    let mut no_script = effects();
    no_script
        .iter_mut()
        .find(|effect| effect.unit_id.0 == "leaf-run")
        .unwrap()
        .dependency_unit_ids
        .clear();
    assert_eq!(lower(&no_script, &bindings()).unwrap_err()[0].code, "unit-plan-build-script-run-unsupported");
    let mut foreign = effects();
    let app = foreign.iter_mut().find(|effect| effect.unit_id.0 == "app").unwrap();
    app.dependency_unit_ids.push(mantle_rust_plan_core::UnitId("leaf-run".into()));
    assert_eq!(lower(&foreign, &bindings()).unwrap_err()[0].code, "unit-plan-build-script-run-unsupported");
}
