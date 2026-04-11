use std::ffi::OsString;
use std::path::PathBuf;

use crunch_glue::CrunchDerivation;
use crunch_glue::Input;

const BOOTSTRAP_ENTRYPOINTS: &[&str] = &[
    "make.ncl",
    "dash.ncl",
    "binutils.ncl",
    "musl.ncl",
    "gcc.ncl",
    "integration-test.ncl",
    "busybox.ncl",
    "bwrap.ncl",
    "rust.ncl",
    "crunch.ncl",
    "selftest.ncl",
];

fn bootstrap_import_paths() -> Vec<OsString> {
    let mut paths = Vec::new();
    paths.push(crunch_eval::stdlib::stdlib_import_path().unwrap().into_os_string());
    paths.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap").into_os_string());
    paths
}

fn bootstrap_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap").join(name)
}

fn eval_bootstrap(name: &str) -> CrunchDerivation {
    crunch_eval::evaluate_and_deserialize(&bootstrap_path(name), &bootstrap_import_paths()).unwrap()
}

fn input_derivation_names(drv: &CrunchDerivation) -> Vec<String> {
    let mut names = Vec::new();
    for input in &drv.inputs {
        if let Input::Derivation(dep) = input {
            names.push(dep.name.clone());
        }
    }
    names
}

#[test]
fn eval_make_bootstrap_imports_shared_seed() {
    let drv = eval_bootstrap("make.ncl");
    let input_names = input_derivation_names(&drv);

    assert_eq!(drv.name, "gnumake");
    assert!(input_names.contains(&"musl-seed-toolchain".to_string()));
    assert!(input_names.contains(&"make-src".to_string()));
}

#[test]
fn eval_crunch_bootstrap_imports_shared_seed() {
    let drv = eval_bootstrap("crunch.ncl");
    let input_names = input_derivation_names(&drv);

    assert_eq!(drv.name, "crunch");
    assert!(input_names.contains(&"musl-seed-toolchain".to_string()));
    assert!(input_names.contains(&"gcc".to_string()));
    assert!(input_names.contains(&"rust".to_string()));
}

#[test]
fn all_bootstrap_entrypoints_evaluate() {
    for entrypoint in BOOTSTRAP_ENTRYPOINTS {
        let drv = eval_bootstrap(entrypoint);
        assert!(!drv.name.is_empty(), "entrypoint must evaluate: {entrypoint}");
        assert!(!drv.builder.is_empty(), "builder must stay present: {entrypoint}");
    }
}
