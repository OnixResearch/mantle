use crate::conversion_cache::ConversionCache;
use crate::convert::convert;
use crate::types::*;

/// Helper: minimal derivation with just name/builder.
fn minimal_drv(name: &str, builder: &str) -> CrunchDerivation {
    CrunchDerivation {
        name: name.to_string(),
        builder: builder.to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: Default::default(),
        inputs: vec![],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    }
}

#[test]
fn derivation_payload_json_roundtrips_for_remote_dispatch() {
    let dep = minimal_drv("dep", "/bin/sh");
    let drv = CrunchDerivation {
        inputs: vec![
            Input::Source("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-source".to_string()),
            Input::Derivation(Box::new(dep)),
        ],
        fixed_output: Some(FixedOutput {
            hash: "sha256-CIE8vumQPGK+TFAnJqQYowoNpFALLTadOvkob0gVzro=".to_string(),
            algo: "sha256".to_string(),
            mode: "flat".to_string(),
        }),
        ..minimal_drv("remote", "/bin/sh")
    };
    let json = serde_json::to_string(&drv).expect("remote payload serializes");
    let roundtrip: CrunchDerivation = serde_json::from_str(&json).expect("remote payload parses");
    let mut original_cache = ConversionCache::default();
    let mut roundtrip_cache = ConversionCache::default();
    let (original_drv_path, original_nix_drv) = convert(&drv, &mut original_cache).expect("original converts");
    let (roundtrip_drv_path, roundtrip_nix_drv) =
        convert(&roundtrip, &mut roundtrip_cache).expect("roundtrip converts");

    assert_eq!(original_drv_path, roundtrip_drv_path);
    assert_eq!(original_nix_drv, roundtrip_nix_drv);
}

#[test]
fn convert_minimal() {
    let drv = minimal_drv("hello", "/bin/sh");
    let mut kp = ConversionCache::default();
    let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();

    // drv path has .drv suffix
    assert!(drv_path.to_string().ends_with("hello.drv"));

    // Output path was computed
    let out = nix_drv.outputs.get("out").unwrap();
    assert!(out.path.is_some());

    // Environment auto-populated
    assert_eq!(nix_drv.environment.get("system").unwrap(), "x86_64-linux");
    assert_eq!(nix_drv.environment.get("builder").unwrap(), "/bin/sh");
    assert_eq!(nix_drv.environment.get("name").unwrap(), "hello");

    // "out" env entry is the computed output path
    let out_path = out.path.as_ref().unwrap().to_absolute_path();
    let out_env: &[u8] = nix_drv.environment.get("out").unwrap().as_ref();
    assert_eq!(out_env, out_path.as_bytes());
}

#[test]
fn convert_deterministic() {
    let drv = minimal_drv("test", "/bin/sh");

    let mut kp1 = ConversionCache::default();
    let (path1, nix1) = convert(&drv, &mut kp1).unwrap();

    let mut kp2 = ConversionCache::default();
    let (path2, nix2) = convert(&drv, &mut kp2).unwrap();

    assert_eq!(path1, path2, "drv path must be deterministic");
    assert_eq!(nix1.outputs, nix2.outputs, "output paths must be deterministic");
}

#[test]
fn convert_different_names_different_paths() {
    let drv_a = minimal_drv("alpha", "/bin/sh");
    let drv_b = minimal_drv("beta", "/bin/sh");

    let mut kp = ConversionCache::default();
    let (path_a, _) = convert(&drv_a, &mut kp).unwrap();
    let (path_b, _) = convert(&drv_b, &mut kp).unwrap();

    assert_ne!(path_a, path_b);
}

#[test]
fn provenance_claims_do_not_change_derivation_hash_by_default() {
    let plain = minimal_drv("claims-test", "/bin/sh");
    let mut claimed = minimal_drv("claims-test", "/bin/sh");
    claimed.provenance = Some(crunch_attestation::Claims {
        supplier: Some("Example Supplier".to_string()),
        homepage: Some("https://example.invalid/claims".to_string()),
        license: Some("MIT".to_string()),
        source_aliases: vec!["origin".to_string(), "mirror".to_string()],
        ..Default::default()
    });

    let mut kp_plain = ConversionCache::default();
    let (plain_path, plain_drv) = convert(&plain, &mut kp_plain).unwrap();

    let mut kp_claimed = ConversionCache::default();
    let (claimed_path, claimed_drv) = convert(&claimed, &mut kp_claimed).unwrap();

    assert_eq!(plain_path, claimed_path, "claims should not change drv path by default");
    assert_eq!(plain_drv, claimed_drv, "claims should not affect nix derivation hashing inputs");
    assert!(!claimed_drv.environment.contains_key("provenance"));
}

#[test]
fn dynamic_plan_outputs_deserialize_when_declared_outputs_exist() {
    let json = r#"{
        "name": "plan-producer",
        "builder": "/bin/sh",
        "outputs": ["out", "plan"],
        "dynamic_plan_outputs": ["plan"]
    }"#;

    let drv: CrunchDerivation = serde_json::from_str(json).unwrap();
    assert_eq!(drv.outputs, vec!["out".to_string(), "plan".to_string()]);
    assert_eq!(drv.dynamic_plan_outputs, vec!["plan".to_string()]);
}

#[test]
fn dynamic_plan_outputs_reject_undeclared_output() {
    let json = r#"{
        "name": "bad-plan-producer",
        "builder": "/bin/sh",
        "outputs": ["out"],
        "dynamic_plan_outputs": ["plan"]
    }"#;

    let err = serde_json::from_str::<CrunchDerivation>(json).unwrap_err().to_string();
    assert!(err.contains("dynamic_plan_outputs entry 'plan'"), "error should name bad output: {err}");
    assert!(err.contains("outputs [out]"), "error should list declared outputs: {err}");
}

#[test]
fn dynamic_plan_outputs_reject_duplicate_name() {
    let json = r#"{
        "name": "dup-plan-producer",
        "builder": "/bin/sh",
        "outputs": ["out", "plan"],
        "dynamic_plan_outputs": ["plan", "plan"]
    }"#;

    let err = serde_json::from_str::<CrunchDerivation>(json).unwrap_err().to_string();
    assert!(err.contains("duplicate dynamic_plan_outputs entry 'plan'"), "error should name duplicate: {err}");
}

#[test]
fn dynamic_plan_outputs_do_not_change_derivation_hash_by_default() {
    let plain = minimal_drv("plan-hash", "/bin/sh");
    let mut declared = minimal_drv("plan-hash", "/bin/sh");
    declared.outputs = vec!["out".to_string(), "plan".to_string()];
    declared.dynamic_plan_outputs = vec!["plan".to_string()];

    let mut without_plan_metadata = declared.clone();
    without_plan_metadata.dynamic_plan_outputs = Vec::new();

    let mut plain_cache = ConversionCache::default();
    let (plain_path, plain_drv) = convert(&without_plan_metadata, &mut plain_cache).unwrap();
    let mut declared_cache = ConversionCache::default();
    let (declared_path, declared_drv) = convert(&declared, &mut declared_cache).unwrap();

    assert_eq!(plain_path, declared_path, "dynamic plan metadata should not change drv path by default");
    assert_eq!(plain_drv, declared_drv, "dynamic plan metadata should not affect nix derivation hashing inputs");
    assert!(!plain_drv.environment.contains_key(PLAN_OUTPUT_BINDINGS_ENV_KEY));
    assert!(!declared_drv.environment.contains_key(PLAN_OUTPUT_BINDINGS_ENV_KEY));
    assert!(plain.dynamic_plan_outputs.is_empty());
    let entry = declared_cache.get_by_drv_path(&declared_path.to_absolute_path()).unwrap();
    assert_eq!(entry.dynamic_plan_outputs, vec!["plan".to_string()]);
}

fn plan_producer() -> CrunchDerivation {
    let mut producer = minimal_drv("plan-producer", "/bin/sh");
    producer.outputs = vec!["out".to_string(), "plan".to_string(), "other".to_string()];
    producer.dynamic_plan_outputs = vec!["plan".to_string(), "other".to_string()];
    producer
}

fn plan_reference(name: &str) -> PlanOutputRef {
    PlanOutputRef {
        name: name.to_string(),
        producer: plan_producer(),
        plan_output: "plan".to_string(),
        root: "unit.app".to_string(),
        unit_output: "out".to_string(),
    }
}

fn plan_consumer(reference: PlanOutputRef) -> CrunchDerivation {
    let marker = format!("{{{{mantle-plan-output:{}}}}}", reference.name);
    let mut consumer = minimal_drv("plan-consumer", "/bin/sh");
    consumer.args = vec![format!("build {marker}/bin/app")];
    consumer.env.insert("APP_PATH".to_string(), format!("{marker}/bin/app"));
    consumer.inputs = vec![Input::PlanOutput(Box::new(reference))];
    consumer
}

#[test]
// r[verify mantle.dynamic_plan_output_inputs.typed_reference]
fn plan_output_input_deserializes_as_closed_validated_record() {
    let input = Input::PlanOutput(Box::new(plan_reference("app")));
    let encoded = serde_json::to_value(&input).unwrap();
    assert_eq!(encoded.as_object().unwrap().len(), 5);
    let decoded: Input = serde_json::from_value(encoded.clone()).unwrap();
    assert!(matches!(decoded, Input::PlanOutput(reference) if reference.name == "app"));

    let mut invalid = encoded.clone();
    invalid["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Input>(invalid).unwrap_err().to_string().contains("outside"));
    let mut invalid = encoded.clone();
    invalid["name"] = serde_json::json!("Bad");
    assert!(serde_json::from_value::<Input>(invalid).unwrap_err().to_string().contains("name"));
    let mut invalid = encoded.clone();
    invalid["root"] = serde_json::json!("Unit.App");
    assert!(serde_json::from_value::<Input>(invalid).unwrap_err().to_string().contains("root"));
    let mut invalid = encoded.clone();
    invalid["unit_output"] = serde_json::json!("bad.name");
    assert!(serde_json::from_value::<Input>(invalid).unwrap_err().to_string().contains("output name"));
    let mut invalid = encoded;
    invalid["producer"]["dynamic_plan_outputs"] = serde_json::json!([]);
    assert!(
        serde_json::from_value::<Input>(invalid)
            .unwrap_err()
            .to_string()
            .contains("declared dynamic_plan_outputs")
    );
}

#[test]
fn plan_output_conversion_binds_selected_edge_and_canonical_markers() {
    let mut consumer = plan_consumer(plan_reference("zeta"));
    consumer.inputs.push(Input::PlanOutput(Box::new(plan_reference("alpha"))));
    let mut cache = ConversionCache::default();
    let (_, converted) = convert(&consumer, &mut cache).unwrap();
    let records: serde_json::Value =
        serde_json::from_slice(converted.environment[PLAN_OUTPUT_BINDINGS_ENV_KEY].as_ref()).unwrap();
    let records = records.as_array().unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["name"], "alpha");
    assert_eq!(records[1]["name"], "zeta");
    let producer_path = records[0]["producer_drv_path"].as_str().unwrap();
    assert!(producer_path.starts_with("/nix/store/"));
    assert_eq!(records[1]["producer_drv_path"], producer_path);
    assert_eq!(records[0]["plan_output"], "plan");
    assert_eq!(records[0]["root"], "unit.app");
    assert_eq!(records[0]["unit_output"], "out");
    assert_eq!(records[0].as_object().unwrap().len(), 6);
    let placeholder = nix_compat::store_path::hash_placeholder("mantle-plan-output:zeta");
    assert_eq!(records[1]["placeholder"], placeholder);
    assert_eq!(converted.arguments, vec![format!("build {placeholder}/bin/app")]);
    let env_path: &[u8] = converted.environment["APP_PATH"].as_ref();
    assert_eq!(env_path, format!("{placeholder}/bin/app").as_bytes());
    let (_, outputs) = converted.input_derivations.iter().next().unwrap();
    assert_eq!(converted.input_derivations.len(), 1);
    assert_eq!(outputs.iter().map(String::as_str).collect::<Vec<_>>(), ["plan"]);
    let mut reversed = consumer.clone();
    reversed.inputs.reverse();
    let (first, _) = convert(&consumer, &mut ConversionCache::default()).unwrap();
    let (second, _) = convert(&reversed, &mut ConversionCache::default()).unwrap();
    assert_eq!(first, second);
    let (_, under_other_prefix) = convert(&consumer, &mut ConversionCache::new("/opt/crunch")).unwrap();
    let bindings: serde_json::Value =
        serde_json::from_slice(under_other_prefix.environment[PLAN_OUTPUT_BINDINGS_ENV_KEY].as_ref()).unwrap();
    assert!(bindings[0]["producer_drv_path"].as_str().unwrap().starts_with("/opt/crunch/"));
}

#[test]
// r[verify mantle.dynamic_plan_output_inputs.request_identity]
fn plan_output_identity_depends_on_each_binding_component() {
    let base = plan_consumer(plan_reference("app"));
    let (base_path, base_nix) = convert(&base, &mut ConversionCache::default()).unwrap();
    assert!(base_nix.outputs["out"].path.is_some());
    let mut variants = Vec::new();
    let mut altered = base.clone();
    if let Input::PlanOutput(reference) = &mut altered.inputs[0] {
        reference.producer.args.push("changed".to_string());
    }
    variants.push(altered);
    let mut altered = base.clone();
    if let Input::PlanOutput(reference) = &mut altered.inputs[0] {
        reference.plan_output = "other".to_string();
    }
    variants.push(altered);
    let mut altered = base.clone();
    if let Input::PlanOutput(reference) = &mut altered.inputs[0] {
        reference.root = "unit.other".to_string();
    }
    variants.push(altered);
    let mut altered = base.clone();
    if let Input::PlanOutput(reference) = &mut altered.inputs[0] {
        reference.unit_output = "dev".to_string();
    }
    variants.push(altered);
    for altered in variants {
        let (path, converted) = convert(&altered, &mut ConversionCache::default()).unwrap();
        assert_ne!(path, base_path);
        assert_ne!(converted.outputs["out"].path, base_nix.outputs["out"].path);
    }
}

#[test]
// r[verify mantle.dynamic_plan_output_inputs.binding_failures]
fn plan_output_rejects_unknown_unterminated_duplicate_and_over_limit() {
    let mut consumer = plan_consumer(plan_reference("app"));
    consumer.args[0] = "{{mantle-plan-output:missing}}".to_string();
    assert!(
        convert(&consumer, &mut ConversionCache::default())
            .unwrap_err()
            .to_string()
            .contains("unknown plan-output marker")
    );
    let mut unbound = minimal_drv("unbound", "/bin/sh");
    unbound.env.insert("TOKEN".to_string(), "{{mantle-plan-output:missing}}".to_string());
    assert!(
        convert(&unbound, &mut ConversionCache::default())
            .unwrap_err()
            .to_string()
            .contains("unknown plan-output marker")
    );
    consumer.env.insert("out".to_string(), "{{mantle-plan-output:missing}}".to_string());
    assert!(
        convert(&consumer, &mut ConversionCache::default())
            .unwrap_err()
            .to_string()
            .contains("overwritten environment key")
    );
    consumer.env.remove("out");
    consumer.args[0] = "{{mantle-plan-output:app".to_string();
    assert!(
        convert(&consumer, &mut ConversionCache::default())
            .unwrap_err()
            .to_string()
            .contains("unterminated plan-output marker")
    );
    consumer = plan_consumer(plan_reference("app"));
    consumer.inputs.push(Input::PlanOutput(Box::new(plan_reference("app"))));
    assert!(
        convert(&consumer, &mut ConversionCache::default())
            .unwrap_err()
            .to_string()
            .contains("duplicate plan-output")
    );
    consumer.inputs =
        (0..17).map(|index| Input::PlanOutput(Box::new(plan_reference(&format!("app{index}"))))).collect();
    assert!(
        convert(&consumer, &mut ConversionCache::default())
            .unwrap_err()
            .to_string()
            .contains("exceed limit of 16")
    );
    let over_limit_json = serde_json::to_string(&consumer).unwrap();
    assert!(
        serde_json::from_str::<CrunchDerivation>(&over_limit_json)
            .unwrap_err()
            .to_string()
            .contains("exceed limit of 16")
    );
    consumer = plan_consumer(plan_reference("app"));
    consumer.env.insert(PLAN_OUTPUT_BINDINGS_ENV_KEY.to_string(), String::new());
    assert!(
        convert(&consumer, &mut ConversionCache::default())
            .unwrap_err()
            .to_string()
            .contains("reserved plan-output")
    );
    let reserved_json = serde_json::to_string(&consumer).unwrap();
    assert!(
        serde_json::from_str::<CrunchDerivation>(&reserved_json)
            .unwrap_err()
            .to_string()
            .contains("reserved plan-output")
    );
}

#[test]
fn derivation_file_input_deserializes_and_roundtrips() {
    let json = r#"{"derivation_file":"bootstrap/stage0-posix.ncl"}"#;
    let input: Input = serde_json::from_str(json).unwrap();
    match &input {
        Input::DerivationFile(reference) => {
            assert_eq!(reference.path, "bootstrap/stage0-posix.ncl");
            assert!(reference.output.is_none());
            assert!(!reference.path.starts_with('/'));
        }
        other => panic!("expected derivation-file input, got {other:?}"),
    }
    assert_eq!(serde_json::to_string(&input).unwrap(), json);
}

#[test]
fn derivation_file_input_deserializes_selected_output() {
    let json = r#"{"derivation_file":"dep.ncl","output":"dev"}"#;
    let input: Input = serde_json::from_str(json).unwrap();
    match &input {
        Input::DerivationFile(reference) => {
            assert_eq!(reference.path, "dep.ncl");
            assert_eq!(reference.output.as_deref(), Some("dev"));
        }
        other => panic!("expected selected derivation-file input, got {other:?}"),
    }
    assert_eq!(serde_json::to_string(&input).unwrap(), json);
}

#[test]
fn derivation_file_input_rejects_mixed_derivation_fields() {
    let json = r#"{"derivation_file":"dep.ncl","name":"dep","builder":"/bin/sh"}"#;
    let error = serde_json::from_str::<Input>(json).unwrap_err().to_string();
    assert!(error.contains("cannot mix"));
    assert!(error.contains("derivation_file"));
}

#[test]
fn conversion_rejects_unresolved_derivation_file() {
    let drv = CrunchDerivation {
        inputs: vec![Input::DerivationFile(DerivationFileRef {
            path: "dep.ncl".to_string(),
            output: None,
        })],
        ..minimal_drv("unresolved", "/bin/sh")
    };
    let error = convert(&drv, &mut ConversionCache::default()).unwrap_err().to_string();
    assert!(error.contains("unresolved derivation-file input"));
    assert!(error.contains("dep.ncl"));
}

#[test]
fn conversion_accepts_pipeline_resolved_derivation_edge() {
    let dep = minimal_drv("resolved-dep", "/bin/sh");
    let mut cache = ConversionCache::default();
    let (dep_path, _) = convert(&dep, &mut cache).unwrap();
    let drv = CrunchDerivation {
        inputs: vec![Input::ResolvedDerivation(ResolvedDerivationRef {
            drv_path: dep_path.to_absolute_path(),
            outputs: vec!["out".to_string()],
        })],
        ..minimal_drv("resolved-root", "/bin/sh")
    };
    let (_, converted) = convert(&drv, &mut cache).unwrap();
    assert_eq!(converted.input_derivations.len(), 1);
    assert!(converted.input_derivations.contains_key(&dep_path));
}

#[test]
fn convert_with_source_input() {
    let drv = CrunchDerivation {
        name: "hello".to_string(),
        builder: "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash/bin/bash".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec!["-c".to_string(), "echo hi > $out".to_string()],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: Default::default(),
        inputs: vec![Input::Source(
            "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash".to_string(),
        )],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut kp = ConversionCache::default();
    let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

    assert_eq!(nix_drv.input_sources.len(), 1);
    let source = nix_drv.input_sources.iter().next().unwrap();
    assert!(source.to_string().contains("bash"));
}

#[test]
fn convert_with_derivation_input() {
    let dep = CrunchDerivation {
        name: "libfoo".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: Default::default(),
        inputs: vec![],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let drv = CrunchDerivation {
        name: "myapp".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: Default::default(),
        inputs: vec![Input::Derivation(Box::new(dep))],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut kp = ConversionCache::default();
    let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

    // libfoo should appear in input_derivations
    assert_eq!(nix_drv.input_derivations.len(), 1);
    let (dep_path, dep_outputs) = nix_drv.input_derivations.iter().next().unwrap();
    assert!(dep_path.to_string().contains("libfoo.drv"));
    assert!(dep_outputs.contains("out"));
}

#[test]
fn convert_fixed_output_sha256() {
    let drv = CrunchDerivation {
        name: "src".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: Default::default(),
        inputs: vec![],
        fixed_output: Some(FixedOutput {
            hash: "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba".to_string(),
            algo: "sha256".to_string(),
            mode: "recursive".to_string(),
        }),
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut kp = ConversionCache::default();
    let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

    let out = nix_drv.outputs.get("out").unwrap();
    assert!(out.ca_hash.is_some(), "FOD should have ca_hash");
    assert!(out.path.is_some(), "FOD should have output path");
}

#[test]
fn convert_multiple_outputs() {
    let drv = CrunchDerivation {
        name: "multi".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string(), "lib".to_string(), "dev".to_string()],
        dynamic_plan_outputs: vec![],
        env: Default::default(),
        inputs: vec![],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut kp = ConversionCache::default();
    let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

    assert_eq!(nix_drv.outputs.len(), 3);
    for name in &["out", "lib", "dev"] {
        let output = nix_drv.outputs.get(*name).unwrap();
        assert!(output.path.is_some(), "output '{name}' should have a path");
    }

    // All output paths should be different
    let paths: Vec<_> = nix_drv.outputs.values().map(|o| o.path.as_ref().unwrap().to_absolute_path()).collect();
    assert_ne!(paths[0], paths[1]);
    assert_ne!(paths[1], paths[2]);
    assert_ne!(paths[0], paths[2]);
}

#[test]
fn convert_diamond_dependency() {
    // D is a shared dependency of B and C. A depends on B and C.
    let d = CrunchDerivation {
        name: "d".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: Default::default(),
        inputs: vec![],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let b = CrunchDerivation {
        name: "b".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: Default::default(),
        inputs: vec![Input::Derivation(Box::new(d.clone()))],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let c = CrunchDerivation {
        name: "c".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: Default::default(),
        inputs: vec![Input::Derivation(Box::new(d))],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let a = CrunchDerivation {
        name: "a".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: Default::default(),
        inputs: vec![Input::Derivation(Box::new(b)), Input::Derivation(Box::new(c))],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut kp = ConversionCache::default();
    let (_, nix_drv) = convert(&a, &mut kp).unwrap();

    // A should have B and C as input derivations
    assert_eq!(nix_drv.input_derivations.len(), 2);
}

#[test]
fn convert_circular_dependency_detected() {
    // We can't express true cycles in the tree structure (CrunchDerivation
    // contains nested CrunchDerivations), but we detect identity-level
    // cycles. Same name+builder+system appearing recursively.
    //
    // This tests that the cycle detector catches a derivation that
    // transitively depends on itself by identity.
    let inner = CrunchDerivation {
        name: "loop".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: Default::default(),
        inputs: vec![], // can't nest itself due to ownership, but identity match triggers
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    // Outer has same identity as inner
    let outer = CrunchDerivation {
        name: "loop".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: Default::default(),
        inputs: vec![Input::Derivation(Box::new(inner))],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut kp = ConversionCache::default();
    let result = convert(&outer, &mut kp);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("circular") || err.contains("Circular"), "error should mention cycle: {err}");
}

#[test]
fn convert_user_env_preserved() {
    let mut env = std::collections::HashMap::new();
    env.insert("CC".to_string(), "/usr/bin/gcc".to_string());
    env.insert("CFLAGS".to_string(), "-O2".to_string());

    let drv = CrunchDerivation {
        name: "with-env".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env,
        inputs: vec![],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut kp = ConversionCache::default();
    let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

    assert_eq!(nix_drv.environment.get("CC").unwrap(), "/usr/bin/gcc");
    assert_eq!(nix_drv.environment.get("CFLAGS").unwrap(), "-O2");
    // Auto-populated entries also present
    assert!(nix_drv.environment.contains_key("system"));
    assert!(nix_drv.environment.contains_key("out"));
}

#[test]
fn convert_invalid_source_path() {
    let drv = CrunchDerivation {
        name: "bad".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: Default::default(),
        inputs: vec![Input::Source("/tmp/not-a-store-path".to_string())],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut kp = ConversionCache::default();
    let result = convert(&drv, &mut kp);
    assert!(result.is_err());
}

#[test]
fn convert_invalid_hash_algo() {
    let drv = CrunchDerivation {
        name: "bad-algo".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: Default::default(),
        inputs: vec![],
        fixed_output: Some(FixedOutput {
            hash: "deadbeef".to_string(),
            algo: "crc32".to_string(),
            mode: "flat".to_string(),
        }),
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut kp = ConversionCache::default();
    let result = convert(&drv, &mut kp);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("hash algorithm"));
}

#[test]
fn convert_from_json_serde() {
    // Simulate what crunch-eval does: JSON → CrunchDerivation via serde
    let json = r#"{
        "name": "from-json",
        "builder": "/bin/sh",
        "system": "x86_64-linux",
        "args": ["-c", "echo hi"],
        "outputs": ["out"],
        "env": {},
        "inputs": [],
        "addressing_mode": "input-addressed"
    }"#;

    let drv: CrunchDerivation = serde_json::from_str(json).unwrap();
    let mut kp = ConversionCache::default();
    let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();

    assert!(drv_path.to_string().ends_with("from-json.drv"));
    assert!(nix_drv.outputs.get("out").unwrap().path.is_some());
}

/// Verify that extra fields from mkDerivation (pname, version, meta,
/// passthru) are silently ignored during deserialization.
#[test]
fn json_with_extra_mkderivation_fields_deserializes() {
    let json = r#"{
        "name": "hello-1.0",
        "builder": "/bin/sh",
        "system": "x86_64-linux",
        "args": ["-c", "echo hi > $out"],
        "outputs": ["out"],
        "env": {},
        "inputs": [],
        "addressing_mode": "content-addressed",
        "pname": "hello",
        "version": "1.0",
        "meta": {"description": "a test", "license": "MIT"},
        "passthru": {"tests": "some-path"}
    }"#;

    let drv: CrunchDerivation = serde_json::from_str(json).unwrap();
    assert_eq!(drv.name, "hello-1.0");
    assert_eq!(drv.builder, "/bin/sh");
    assert_eq!(drv.addressing_mode, "content-addressed");

    // Extra fields don't appear on the struct — they're dropped.
    let mut kp = ConversionCache::default();
    let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();
    assert!(drv_path.to_string().ends_with("hello-1.0.drv"));

    // Environment should NOT contain pname/version/meta/passthru —
    // only the fields convert() explicitly sets.
    assert!(!nix_drv.environment.contains_key("pname"));
    assert!(!nix_drv.environment.contains_key("version"));
    assert!(!nix_drv.environment.contains_key("meta"));
    assert!(!nix_drv.environment.contains_key("passthru"));
}
