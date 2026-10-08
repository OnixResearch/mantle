use mantle_cargo_unit_plan::ExternBinding;
use mantle_cargo_unit_plan::PlanBindings;
use mantle_cargo_unit_plan::SourceSlice;
use mantle_cargo_unit_plan::UnitBinding;
use mantle_cargo_unit_plan::lower;
use mantle_rust_plan_core::ExistingUnitFacts;
use mantle_rust_plan_core::plan_existing_unit_effects;
use serde_json::Value;

const PATH_HASH: &str = "01234567890123456789012345678901";
const SOURCE_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn store(name: &str) -> String {
    format!("/mantle/store/{PATH_HASH}-{name}")
}
fn fact(id: &str, rank: u32, deps: &[&str], kind: &str, execution: &str) -> ExistingUnitFacts {
    ExistingUnitFacts {
        unit_id: id.to_string(),
        package_id: format!("path+file:///fixture#{id}@0.1.0"),
        target_name: id.to_string(),
        target_kind: kind.to_string(),
        execution_kind: execution.to_string(),
        dependency_unit_ids: deps.iter().map(ToString::to_string).collect(),
        arguments: vec!["--crate-type=rlib".to_string(), "--edition=2024".to_string()],
        environment: Vec::new(),
        input_identities: vec![format!("source:{id}")],
        expected_outputs: vec!["out".to_string()],
        execution_order: rank,
    }
}
fn bindings() -> PlanBindings {
    let mut units = Vec::new();
    let mut sources = Vec::new();
    for (id, externs) in [
        ("leaf", vec![]),
        ("mid", vec![ExternBinding {
            name: "leaf".into(),
            unit_id: "leaf".into(),
        }]),
        ("app", vec![ExternBinding {
            name: "mid".into(),
            unit_id: "mid".into(),
        }]),
    ] {
        sources.push(SourceSlice {
            id: id.into(),
            producer_output: "sources".into(),
            subpath: id.into(),
            store_name: format!("source-{id}"),
            nar_blake3: SOURCE_DIGEST.into(),
        });
        units.push(UnitBinding {
            unit_id: id.into(),
            source_id: id.into(),
            source_kind: "path".into(),
            source_entry: "src/lib.rs".into(),
            source_label: format!("{id}-0.1.0"),
            rustc_metadata_hash: "abcdef1234".into(),
            triple: "x86_64-unknown-linux-gnu".into(),
            externs,
        });
    }
    PlanBindings {
        store_prefix: "/mantle/store".into(),
        system: "x86_64-linux".into(),
        target_triple: "x86_64-unknown-linux-gnu".into(),
        host_triple: "x86_64-unknown-linux-gnu".into(),
        helper: store("unit-helper"),
        rustc: format!("{}/bin/rustc", store("rust-toolchain")),
        toolchain: store("rust-toolchain"),
        linker: format!("{}/bin/cc", store("linker")),
        sources,
        units,
        roots: vec!["app".into()],
    }
}
fn effects() -> Vec<mantle_rust_plan_core::ExistingUnitEffect> {
    plan_existing_unit_effects(vec![
        fact("app", 2, &["mid"], "bin", "target"),
        fact("mid", 1, &["leaf"], "lib", "target"),
        fact("leaf", 0, &[], "lib", "target"),
    ])
    .unwrap()
}

#[test]
fn direct_dependencies_and_manifest_contract_are_canonical() {
    let (bytes, digest) = lower(&effects(), &bindings()).unwrap();
    let plan: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(plan["schema"], "mantle-plan-v2");
    let app = plan["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|unit| unit["derivation"]["env"]["MANTLE_UNIT_SOURCE"] == "{{mantle-source:app}}")
        .unwrap();
    let mid_id = format!("u.{}", blake3::hash(b"mid").to_hex());
    let inputs = app["derivation"]["inputs"].as_array().unwrap();
    assert_eq!(inputs.iter().filter(|entry| entry["kind"] == "unit_output").count(), 1);
    assert!(inputs.iter().any(|entry| entry["kind"] == "unit_output" && entry["unit"] == mid_id));
    let tools = inputs
        .iter()
        .filter(|entry| entry["kind"] == "store_path")
        .map(|entry| entry["path"].as_str().unwrap().to_string())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        tools,
        std::collections::BTreeSet::from([store("unit-helper"), store("rust-toolchain"), store("linker")])
    );
    assert_eq!(app["derivation"]["env"]["MANTLE_UNIT_RUSTC"], format!("{}/bin/rustc", store("rust-toolchain")));
    assert_eq!(app["derivation"]["env"]["MANTLE_UNIT_LINKER"], format!("{}/bin/cc", store("linker")));
    let token = format!("{{{{mantle-unit-output:{mid_id}:out}}}}");
    assert_eq!(app["derivation"]["env"]["MANTLE_UNIT_DEPENDENCIES"], serde_json::to_string(&vec![&token]).unwrap());
    assert_eq!(
        app["derivation"]["env"]["MANTLE_UNIT_EXTERNS"],
        serde_json::to_string(&std::collections::BTreeMap::from([("mid", token)])).unwrap()
    );
    assert!(
        app["derivation"]["args"]
            .as_array()
            .unwrap()
            .iter()
            .any(|arg| arg == "--remap-path-prefix={{mantle-source:app}}=app-0.1.0")
    );
    let mut alternate = effects();
    alternate.reverse();
    let mut bindings = bindings();
    bindings.units.reverse();
    bindings.sources.reverse();
    let (bytes_alt, digest_alt) = lower(&alternate, &bindings).unwrap();
    assert_eq!((bytes, digest), (bytes_alt, digest_alt));
}

#[test]
fn multiple_roots_sort_by_wire_identity_not_raw_cargo_identity() {
    let mut input = bindings();
    input.roots = vec!["mid".into(), "app".into()];
    let (bytes, digest) = lower(&effects(), &input).unwrap();
    let plan: Value = serde_json::from_slice(&bytes).unwrap();
    let mut expected = vec![
        format!("u.{}", blake3::hash(b"mid").to_hex()),
        format!("u.{}", blake3::hash(b"app").to_hex()),
    ];
    expected.sort();
    assert_eq!(plan["roots"], serde_json::to_value(expected).unwrap());
    input.roots.reverse();
    assert_eq!(lower(&effects(), &input).unwrap(), (bytes, digest));
}

#[test]
fn unsupported_execution_modes_and_unbound_paths_fail_closed() {
    let mut cases = Vec::new();
    let mut script = effects();
    script[0].execution_kind = "build-script-run".into();
    cases.push((script, bindings(), "unit-plan-build-script-run-unsupported"));
    let mut mode = effects();
    mode[1].target_kind = "test".into();
    cases.push((mode, bindings(), "unit-plan-mode-unsupported"));
    let mut path = effects();
    path[0].arguments.push("-Lnative=/home/host".into());
    cases.push((path, bindings(), "unit-plan-host-path"));
    let mut triple = bindings();
    triple.units[0].triple = "unknown-foreign-triple".into();
    cases.push((effects(), triple, "unit-plan-triple-unsupported"));
    let mut source = bindings();
    source.sources[0].subpath = "../escape".into();
    cases.push((effects(), source, "unit-plan-source-unsupported"));
    let mut input = bindings();
    input.units[0].externs.push(ExternBinding {
        name: "unexpected".into(),
        unit_id: "app".into(),
    });
    cases.push((effects(), input, "unit-plan-mode-unsupported"));
    for (effects, bindings, code) in cases {
        let blocker = lower(&effects, &bindings).unwrap_err();
        assert_eq!(blocker[0].code, code, "{blocker:?}");
    }
}

#[test]
fn proc_macro_host_unit_and_git_source_boundary() {
    let mut planned = effects();
    planned.iter_mut().find(|unit| unit.unit_id.0 == "leaf").unwrap().target_kind = "proc-macro".into();
    planned.iter_mut().find(|unit| unit.unit_id.0 == "leaf").unwrap().execution_kind = "host".into();
    let (bytes, _) = lower(&planned, &bindings()).unwrap();
    let plan: Value = serde_json::from_slice(&bytes).unwrap();
    let macro_unit = plan["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|unit| unit["derivation"]["env"]["MANTLE_UNIT_SOURCE"] == "{{mantle-source:leaf}}")
        .unwrap();
    assert_eq!(macro_unit["derivation"]["addressing_mode"], "input-addressed");
    assert!(macro_unit["derivation"]["args"].as_array().unwrap().iter().any(|arg| arg == "metadata=abcdef1234"));
    let mut git = bindings();
    git.units[0].source_kind = "git".into();
    assert_eq!(lower(&planned, &git).unwrap_err()[0].code, "unit-plan-source-unsupported");
}

#[test]
fn full_digest_source_id_is_distinct_from_bounded_slice_path() {
    let mut input = bindings();
    let digest = "a".repeat(64);
    let id = format!("s.{digest}");
    input.sources[0].id = id.clone();
    input.sources[0].subpath = format!("packages/{digest}");
    input.units[0].source_id = id.clone();
    let (bytes, _) = lower(&effects(), &input).unwrap();
    let plan: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        plan["sources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|source| source["id"] == id && source["subpath"] == format!("packages/{digest}"))
    );
}

fn extend_with_units(
    effects: &mut Vec<mantle_rust_plan_core::ExistingUnitEffect>,
    bindings: &mut PlanBindings,
    count: u32,
    large_argument: bool,
) {
    for index in 0..count {
        let mut effect = effects[0].clone();
        let id = format!("extra-{index}");
        effect.effect_id.0 = format!("effect:{id}");
        effect.unit_id.0 = id.clone();
        effect.dependency_unit_ids.clear();
        effect.execution_order = index + 3;
        if large_argument {
            effect.arguments.push("a".repeat(16_000));
        }
        effects.push(effect);
        let mut binding = bindings.units[0].clone();
        binding.unit_id = id;
        bindings.units.push(binding);
    }
}

#[test]
fn generic_plan_unit_input_and_byte_limits_block_before_publication() {
    let mut too_many = effects();
    too_many.extend(std::iter::repeat_n(too_many[0].clone(), 4094));
    assert_eq!(lower(&too_many, &bindings()).unwrap_err()[0].code, "unit-plan-limit");
    let mut input_effects = effects();
    let mut input_bindings = bindings();
    extend_with_units(&mut input_effects, &mut input_bindings, 253, false);
    let app = input_effects.iter_mut().find(|effect| effect.unit_id.0 == "app").unwrap();
    app.dependency_unit_ids
        .extend((0..253).map(|index| mantle_rust_plan_core::UnitId(format!("extra-{index}"))));
    assert_eq!(lower(&input_effects, &input_bindings).unwrap_err()[0].code, "unit-plan-limit");
    let mut large = effects();
    let mut large_bindings = bindings();
    extend_with_units(&mut large, &mut large_bindings, 300, true);
    let blocker = lower(&large, &large_bindings).unwrap_err();
    assert_eq!(blocker[0].code, "unit-plan-limit");
    assert!(blocker[0].detail.contains("plan bytes"));
}
