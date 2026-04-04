//! Integration tests for the crunch Nickel stdlib.
//!
//! These tests evaluate Nickel expressions through crunch-eval,
//! using the stdlib from the source tree.

use std::ffi::OsString;

fn stdlib_import_path() -> Vec<OsString> {
    let lib_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib");
    assert!(lib_dir.join("lib.ncl").exists(), "stdlib not found at {lib_dir:?}");
    vec![lib_dir.into()]
}

#[test]
fn contract_catches_missing_name() {
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in { builder = "/bin/sh" } | crunch.Derivation"#,
        &stdlib_import_path(),
    );
    assert!(result.is_err(), "should fail: missing name");
}

#[test]
fn contract_catches_wrong_type() {
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in { name = 42, builder = "/bin/sh" } | crunch.Derivation"#,
        &stdlib_import_path(),
    );
    assert!(result.is_err(), "should fail: name is not a string");
}

#[test]
fn contract_catches_extra_field() {
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in { name = "x", builder = "/bin/sh", bogus = true } | crunch.Derivation"#,
        &stdlib_import_path(),
    );
    assert!(result.is_err(), "should fail: extra field bogus");
}

#[test]
fn enum_tags_deserialize_via_json() {
    #[derive(serde::Deserialize, Debug)]
    struct Drv {
        system: String,
        sandbox: String,
    }

    let drv: Drv = crunch_eval::evaluate_str_and_deserialize(
        r#"let crunch = import "lib.ncl" in { name = "t", builder = "/bin/sh" } | crunch.Derivation"#,
        &stdlib_import_path(),
    )
    .unwrap();

    assert_eq!(drv.system, "x86_64-linux");
    assert_eq!(drv.sandbox, "native");
}

#[test]
fn enum_tags_deserialize_direct_serde() {
    // This tests the direct to_serde() path (no JSON intermediate),
    // using CrunchDerivation which has NickelString-aware deserialization
    // for enum-backed fields.
    use crunch_glue::CrunchDerivation;

    let expr = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in { name = "t", builder = "/bin/sh" } | crunch.Derivation"#,
        &stdlib_import_path(),
    )
    .unwrap();

    let drv: CrunchDerivation = expr.to_serde().unwrap();
    assert_eq!(drv.system, "x86_64-linux");
    assert_eq!(drv.name, "t");
}

#[test]
fn store_path_validator_accepts_valid() {
    let expr = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash" | crunch.StorePath"#,
        &stdlib_import_path(),
    );
    assert!(expr.is_ok(), "valid store path should pass");
}

#[test]
fn store_path_validator_rejects_invalid() {
    let result = crunch_eval::evaluate_str(
        r#"let crunch = import "lib.ncl" in "/tmp/bad" | crunch.StorePath"#,
        &stdlib_import_path(),
    );
    assert!(result.is_err(), "invalid store path should fail");
}

#[test]
fn recursive_record_self_reference() {
    #[derive(serde::Deserialize, Debug)]
    struct Drv {
        name: String,
        env: std::collections::HashMap<String, String>,
    }

    let expr = crunch_eval::evaluate_str(
        r#"
        let crunch = import "lib.ncl" in
        {
            name = "myapp",
            builder = "/bin/sh",
            env = { APP_NAME = name },
        } | crunch.Derivation
        "#,
        &stdlib_import_path(),
    )
    .unwrap();

    let drv: Drv = expr.to_serde().unwrap();
    assert_eq!(drv.env.get("APP_NAME").unwrap(), "myapp");
}

#[test]
fn fixed_output_contract() {
    #[derive(serde::Deserialize, Debug)]
    struct Fo {
        hash: String,
        algo: String,
        mode: String,
    }
    #[derive(serde::Deserialize, Debug)]
    struct Drv {
        fixed_output: Option<Fo>,
    }

    let drv: Drv = crunch_eval::evaluate_str_and_deserialize(
        r#"
        let crunch = import "lib.ncl" in
        {
            name = "src",
            builder = "/bin/sh",
            fixed_output = { hash = "sha256-abc123", algo = 'sha256 },
        } | crunch.Derivation
        "#,
        &stdlib_import_path(),
    )
    .unwrap();

    let fo = drv.fixed_output.unwrap();
    assert_eq!(fo.algo, "sha256");
    assert_eq!(fo.mode, "flat"); // default
}

#[test]
fn full_serde_round_trip() {
    use crunch_glue::CrunchDerivation;

    let drv: CrunchDerivation = crunch_eval::evaluate_str_and_deserialize(
        r#"
        let crunch = import "lib.ncl" in
        {
            name = "hello",
            builder = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash/bin/bash",
            args = ["-c", "echo hi > $out"],
            inputs = ["/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash"],
        } | crunch.Derivation
        "#,
        &stdlib_import_path(),
    )
    .unwrap();

    assert_eq!(drv.name, "hello");
    assert_eq!(drv.args.len(), 2);
    assert_eq!(drv.inputs.len(), 1);

    // Convert through glue
    let mut kp = crunch_glue::KnownPaths::new();
    let (drv_path, nix_drv) = crunch_glue::convert(&drv, &mut kp).unwrap();
    assert!(drv_path.to_string().ends_with("hello.drv"));
    assert!(nix_drv.outputs.get("out").unwrap().path.is_some());
}
