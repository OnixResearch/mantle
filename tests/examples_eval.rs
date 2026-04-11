use std::ffi::OsString;
use std::path::PathBuf;

use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use crunch_glue::Input;

fn stdlib_import_path() -> Vec<OsString> {
    vec![crunch_eval::stdlib::stdlib_import_path().unwrap().into_os_string()]
}

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples").join(name)
}

fn eval_example(name: &str) -> CrunchDerivation {
    let path = example_path(name);
    crunch_eval::evaluate_and_deserialize(&path, &stdlib_import_path()).unwrap()
}

#[test]
fn eval_fetch_crate_crc64_example() {
    let drv = eval_example("fetch-crate-crc64.ncl");
    assert_eq!(drv.name, "crc64-src");
    assert!(drv.fixed_output.is_some(), "fetch example must stay fixed-output");
    assert_eq!(drv.builder, "builtin:fetchurl");

    let mut cache = ConversionCache::default();
    let (_drv_path, nix_drv) = crunch_glue::convert(&drv, &mut cache).unwrap();
    assert!(nix_drv.input_sources.is_empty(), "fetcher should not add source inputs");
    assert!(nix_drv.input_derivations.is_empty(), "fetcher should not add derivation inputs");
}

#[test]
fn eval_build_crate_crc64_example() {
    let drv = eval_example("build-crate-crc64.ncl");
    assert_eq!(drv.name, "crc64-2.0.0");
    assert!(drv.args.iter().any(|arg| arg.contains("cargo build --release --offline")));
    assert!(drv.inputs.len() >= 4, "example should wire toolchain, musl, rust, and src");
    assert!(
        drv.inputs.iter().all(|input| !matches!(input, Input::Source(_))),
        "example should build from derivation inputs only"
    );

    let mut cache = ConversionCache::default();
    let (_drv_path, nix_drv) = crunch_glue::convert(&drv, &mut cache).unwrap();
    assert!(!nix_drv.input_derivations.is_empty(), "build example should depend on fetched/built derivations");
    assert!(nix_drv.input_sources.is_empty(), "build example should stay self-contained");
}
