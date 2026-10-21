use std::collections::BTreeMap;

use mantle_cargo_build_bundle_core::*;
use mantle_rust_plan_core::NativeInputDeclaration;
use mantle_rust_plan_core::resolve_native_inputs;

fn root(name: &str) -> String {
    format!("/nix/store/00000000000000000000000000000000-{name}")
}
fn artifact(name: &str) -> Artifact {
    Artifact {
        file: format!("{name}.json"),
        bytes: 1,
        blake3: "a".repeat(64),
    }
}
fn fixture() -> Manifest {
    let tools = ["rustc", "cargo", "toolchain", "producer", "planner", "helper", "linker"]
        .into_iter()
        .map(|role| (role.to_string(), root(role)))
        .collect::<BTreeMap<_, _>>();
    let producer_outputs = ["out", "plan", "sources"]
        .into_iter()
        .map(|role| (role.to_string(), root(role)))
        .collect::<BTreeMap<_, _>>();
    let required_store_paths = tools.values().chain(producer_outputs.values()).cloned().collect();
    let evidence = [
        "cargo-metadata",
        "cargo-unit-graph",
        "rust-plan-receipt",
        "cargo-unit-plan-evidence",
        "cargo-unit-bindings",
    ]
    .into_iter()
    .map(|key| (key.to_string(), artifact(key)))
    .collect::<BTreeMap<_, _>>();
    let mut manifest = Manifest {
        schema: SCHEMA.into(),
        version: 1,
        compatibility: Compatibility {
            system: "x86_64-linux".into(),
            host_triple: "x86_64-unknown-linux-gnu".into(),
            target_triple: "x86_64-unknown-linux-gnu".into(),
            store_prefix: "/nix/store".into(),
            sandbox: "native".into(),
            network: "denied".into(),
            policy_blake3: "b".repeat(64),
            trust_policy_blake3: "c".repeat(64),
            plan_schema: "mantle-plan-v2".into(),
            source_schema: "mantle-source-bundle-v1".into(),
            receipt_schema: "mantle-build-receipt-bundle-v1".into(),
            max_plan_bytes: 4_194_304,
            max_units: 4096,
            max_sources: 256,
            max_direct_inputs: 256,
        },
        profile: "release".into(),
        selected_features: vec!["fmt".into(), "sum".into()],
        declared_env: BTreeMap::new(),
        tools,
        required_store_paths,
        no_default_features: false,
        producer_outputs,
        source_manifest_blake3: "d".repeat(64),
        receipt_bundle_blake3: "e".repeat(64),
        lock_blake3: "f".repeat(64),
        canonical_plan_blake3: "0".repeat(64),
        package_sources: [
            ("engine".into(), "source-engine".into()),
            ("app".into(), "source-app".into()),
        ]
        .into(),
        registry_git_sources: BTreeMap::new(),
        vendor_store_path: None,
        units: [
            ("engine.lib".into(), Unit {
                package_id: Some("engine".into()),
                dependencies: vec![],
                features: vec![],
            }),
            ("app.bin".into(), Unit {
                package_id: Some("app".into()),
                dependencies: vec!["engine.lib".into()],
                features: vec![],
            }),
        ]
        .into(),
        roots: vec!["app.bin".into()],
        native_inputs: BTreeMap::new(),
        provenance: Provenance {
            cargo_unit_graph_version: 1,
            planner_receipt_schema_version: 1,
            producer_revision: "1".repeat(40),
            oracle_inputs: evidence.iter().map(|(key, item)| (key.clone(), item.blake3.clone())).collect(),
            evidence,
        },
        artifacts: ["source", "receipts", "plan", "goals", "store", "lock", "config"]
            .into_iter()
            .map(|key| (key.into(), artifact(key)))
            .collect(),
        bundle_blake3: String::new(),
        non_claim: NON_CLAIM.into(),
    };
    // Distinct transports may not alias filenames even if their bytes happen to match.
    manifest.artifacts.get_mut("source").unwrap().file = "sources.bundle.json".into();
    canonicalize(manifest).unwrap()
}

#[test]
fn canonical_identity_does_not_depend_on_transport_order_or_checkout() {
    let baseline = fixture();
    let mut reordered = baseline.clone();
    reordered.required_store_paths.reverse();
    reordered.selected_features.reverse();
    reordered.units.get_mut("app.bin").unwrap().dependencies.push("engine.lib".into());
    let repeated = canonicalize(reordered).unwrap();
    assert_eq!(encode(&baseline).unwrap(), encode(&repeated).unwrap());
    verify(&baseline, &baseline.compatibility).unwrap();
    // No exporter filesystem root exists in this identity: transport names are relative.
    let mut path_escape = baseline.clone();
    path_escape.artifacts.get_mut("goals").unwrap().file = "/tmp/exporter-a/goals.json".into();
    assert!(canonicalize(path_escape).is_err());
}

#[test]
fn tool_and_native_input_changes_invalidate_handoff_identity() {
    let baseline = fixture();
    let mut changed = baseline.clone();
    changed.tools.insert("rustc".into(), root("rustc-new"));
    changed.required_store_paths.push(root("rustc-new"));
    assert_ne!(canonicalize(changed).unwrap().bundle_blake3, baseline.bundle_blake3);
    let mut native = baseline.clone();
    let declaration = NativeInputDeclaration {
        inputs: [("library".into(), root("native-a"))].into(),
        env: BTreeMap::new(),
        path: vec![],
    };
    native.required_store_paths.push(root("native-a"));
    native.native_inputs.insert("engine".into(), NativePackage {
        schema_version: 1,
        resolved: resolve_native_inputs(&declaration).unwrap(),
        declaration,
    });
    let native = canonicalize(native).unwrap();
    let mut changed = native.clone();
    let package = changed.native_inputs.get_mut("engine").unwrap();
    package.declaration.inputs.insert("library".into(), root("native-b"));
    package.resolved = resolve_native_inputs(&package.declaration).unwrap();
    changed.required_store_paths.push(root("native-b"));
    assert_ne!(canonicalize(changed).unwrap().bundle_blake3, native.bundle_blake3);
    let mut stale = native.clone();
    stale.native_inputs.get_mut("engine").unwrap().resolved.inputs.clear();
    assert!(canonicalize(stale).is_err());
}

#[test]
fn mandatory_version_platform_policy_limits_and_tamper_fail_closed() {
    let baseline = fixture();
    let mutations: [fn(&mut Manifest); 15] = [
        |m| m.version = 2,
        |m| m.schema = "future".into(),
        |m| m.compatibility.plan_schema = "mantle-plan-v3".into(),
        |m| m.compatibility.source_schema = "future".into(),
        |m| m.compatibility.receipt_schema = "future".into(),
        |m| m.compatibility.target_triple = "aarch64-unknown-linux-gnu".into(),
        |m| m.compatibility.store_prefix = "/relocated/store".into(),
        |m| m.compatibility.network = "allowed".into(),
        |m| m.compatibility.max_direct_inputs = 257,
        |m| m.provenance.cargo_unit_graph_version = 2,
        |m| m.provenance.planner_receipt_schema_version = 2,
        |m| m.tools.remove("linker").map(drop).unwrap(),
        |m| m.required_store_paths.retain(|path| path != &root("helper")),
        |m| m.artifacts.remove("lock").map(drop).unwrap(),
        |m| m.provenance.evidence.clear(),
    ];
    for mutate in mutations {
        let mut invalid = baseline.clone();
        mutate(&mut invalid);
        assert!(canonicalize(invalid).is_err());
    }
    let mut tampered = baseline.clone();
    tampered.lock_blake3 = "a".repeat(64);
    assert!(verify(&tampered, &baseline.compatibility).is_err());
    let mut policy = baseline.compatibility.clone();
    policy.policy_blake3 = "d".repeat(64);
    assert!(verify(&baseline, &policy).is_err());
    let mut unknown = serde_json::to_value(&baseline).unwrap();
    unknown["new_mandatory_schema"] = true.into();
    assert!(serde_json::from_value::<Manifest>(unknown).is_err());
}

#[test]
fn unresolved_package_edges_cycles_and_native_roles_are_not_admitted() {
    let baseline = fixture();
    let mut missing = baseline.clone();
    missing.units.get_mut("app.bin").unwrap().dependencies.push("absent".into());
    assert!(canonicalize(missing).is_err());
    let mut cyclic = baseline.clone();
    cyclic.units.get_mut("engine.lib").unwrap().dependencies.push("app.bin".into());
    assert!(canonicalize(cyclic).is_err());
    let mut undeclared = baseline.clone();
    let declaration = NativeInputDeclaration {
        inputs: BTreeMap::new(),
        env: [("LIBRARY_PATH".into(), "{{role:absent}}/lib".into())].into(),
        path: vec![],
    };
    undeclared.native_inputs.insert("engine".into(), NativePackage {
        schema_version: 1,
        declaration,
        resolved: Default::default(),
    });
    assert!(canonicalize(undeclared).is_err());
    let mut missing_source = baseline;
    missing_source.package_sources.remove("engine");
    assert!(canonicalize(missing_source).is_err());
}
