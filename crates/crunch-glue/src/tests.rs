use crate::convert::convert;
use crate::conversion_cache::ConversionCache;
use crate::types::*;

/// Helper: minimal derivation with just name/builder.
fn minimal_drv(name: &str, builder: &str) -> CrunchDerivation {
    CrunchDerivation {
        name: name.to_string(),
        builder: builder.to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        env: Default::default(),
        inputs: vec![],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
    }
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
fn convert_with_source_input() {
    let drv = CrunchDerivation {
        name: "hello".to_string(),
        builder: "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash/bin/bash".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec!["-c".to_string(), "echo hi > $out".to_string()],
        outputs: vec!["out".to_string()],
        env: Default::default(),
        inputs: vec![
            Input::Source("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash".to_string()),
        ],
        fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
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
        env: Default::default(),
        inputs: vec![],
        fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
    };

    let drv = CrunchDerivation {
        name: "myapp".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        env: Default::default(),
        inputs: vec![Input::Derivation(Box::new(dep))],
        fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
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
        env: Default::default(),
        inputs: vec![],
        fixed_output: Some(FixedOutput {
            hash: "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba".to_string(),
            algo: "sha256".to_string(),
            mode: "recursive".to_string(),
        }),
        addressing_mode: "input-addressed".to_string(),
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
        env: Default::default(),
        inputs: vec![],
        fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
    };

    let mut kp = ConversionCache::default();
    let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

    assert_eq!(nix_drv.outputs.len(), 3);
    for name in &["out", "lib", "dev"] {
        let output = nix_drv.outputs.get(*name).unwrap();
        assert!(output.path.is_some(), "output '{name}' should have a path");
    }

    // All output paths should be different
    let paths: Vec<_> = nix_drv
        .outputs
        .values()
        .map(|o| o.path.as_ref().unwrap().to_absolute_path())
        .collect();
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
        env: Default::default(),
        inputs: vec![],
        fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
    };

    let b = CrunchDerivation {
        name: "b".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        env: Default::default(),
        inputs: vec![Input::Derivation(Box::new(d.clone()))],
        fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
    };

    let c = CrunchDerivation {
        name: "c".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        env: Default::default(),
        inputs: vec![Input::Derivation(Box::new(d))],
        fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
    };

    let a = CrunchDerivation {
        name: "a".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        env: Default::default(),
        inputs: vec![
            Input::Derivation(Box::new(b)),
            Input::Derivation(Box::new(c)),
        ],
        fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
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
        env: Default::default(),
        inputs: vec![], // can't nest itself due to ownership, but identity match triggers
        fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
    };

    // Outer has same identity as inner
    let outer = CrunchDerivation {
        name: "loop".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        env: Default::default(),
        inputs: vec![Input::Derivation(Box::new(inner))],
        fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
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
        env,
        inputs: vec![],
        fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
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
        env: Default::default(),
        inputs: vec![Input::Source("/tmp/not-a-store-path".to_string())],
        fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
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
        env: Default::default(),
        inputs: vec![],
        fixed_output: Some(FixedOutput {
            hash: "deadbeef".to_string(),
            algo: "crc32".to_string(),
            mode: "flat".to_string(),
        }),
        addressing_mode: "input-addressed".to_string(),
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
