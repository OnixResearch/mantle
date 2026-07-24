use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use crunch_glue::Input;
use serde::Deserialize;

const CATALOG_PATH: &str = "examples/catalog.ncl";
const EVAL_RAIL: &str = "eval";
const SELECTED_SEED_PROVIDER_NAME: &str = "full-source-seed-toolchain";
const MISSING_SEED_EXAMPLE: &str = r#"
let mantle = import "lib.ncl" in
let seed = import "seed.ncl" in
{
  name = "missing-seed-example",
  builder = "/bin/sh",
  args = ["-c", "echo missing seed > $out"],
  inputs = [seed.bash],
} | mantle.Derivation
"#;
const MALFORMED_NICKEL_EXAMPLE: &str = "let mantle = import \"lib.ncl\" in { name =";

#[derive(Debug, Deserialize)]
struct Catalog {
    examples: Vec<CatalogExample>,
}

#[derive(Debug, Deserialize)]
struct CatalogExample {
    id: String,
    path: String,
    validation_rails: Vec<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn stdlib_import_path() -> Vec<OsString> {
    vec![crunch_eval::stdlib::stdlib_import_path().unwrap().into_os_string()]
}

fn example_path(name: &str) -> PathBuf {
    repo_root().join("examples").join(name)
}

fn catalog_path() -> PathBuf {
    repo_root().join(CATALOG_PATH)
}

fn load_catalog() -> Catalog {
    crunch_eval::evaluate_and_deserialize(&catalog_path(), &stdlib_import_path()).unwrap()
}

fn eval_example(name: &str) -> CrunchDerivation {
    let path = example_path(name);
    crunch_eval::evaluate_and_deserialize(&path, &stdlib_import_path()).unwrap()
}

fn cataloged_eval_paths(catalog: &Catalog) -> Vec<&CatalogExample> {
    catalog
        .examples
        .iter()
        .filter(|example| example.validation_rails.iter().any(|rail| rail == EVAL_RAIL))
        .collect()
}

fn evaluate_json(path: &Path) -> String {
    crunch_eval::evaluate_to_json(path, &stdlib_import_path()).unwrap()
}

#[test]
fn cataloged_eval_examples_export_json() {
    let catalog = load_catalog();
    let examples = cataloged_eval_paths(&catalog);

    assert!(!examples.is_empty(), "catalog should mark eval examples");
    assert!(examples.iter().any(|example| example.path == "examples/hello.ncl"));
    assert!(examples.iter().any(|example| example.path == "examples/fetch-crate-crc64.ncl"));

    for example in examples {
        let json = evaluate_json(&repo_root().join(&example.path));
        assert!(!json.trim().is_empty(), "{} should export JSON", example.id);
        assert!(json.trim_start().starts_with('{'), "{} should export an object: {json}", example.id);
    }
}

#[test]
fn malformed_example_fails_before_export() {
    let err = crunch_eval::evaluate_str_to_json(MALFORMED_NICKEL_EXAMPLE, &stdlib_import_path()).unwrap_err();
    let rendered = err.to_string();

    assert!(!rendered.is_empty());
    assert!(rendered.contains("error") || rendered.contains("parse"), "unexpected error: {rendered}");
}

#[test]
fn seed_dependent_example_without_seed_fails_loudly() {
    let err = crunch_eval::evaluate_str_to_json(MISSING_SEED_EXAMPLE, &stdlib_import_path()).unwrap_err();
    let rendered = err.to_string();

    assert!(!rendered.is_empty());
    assert!(rendered.contains("seed.ncl") || rendered.contains("import"), "unexpected error: {rendered}");
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

    let input_names: Vec<String> = drv
        .inputs
        .iter()
        .filter_map(|input| match input {
            Input::Derivation(dep) => Some(dep.name.clone()),
            _ => None,
        })
        .collect();
    assert!(
        input_names.iter().any(|name| name == SELECTED_SEED_PROVIDER_NAME),
        "example should use selected full-source seed provider: {input_names:?}"
    );

    assert!(!input_names.is_empty(), "build example should depend on fetched/built derivations");
}

#[test]
fn eval_bootstrap_no_nix_example_uses_shared_seed() {
    let drv = eval_example("bootstrap-no-nix.ncl");
    assert_eq!(drv.name, "hello-no-nix");

    let input_names: Vec<String> = drv
        .inputs
        .iter()
        .filter_map(|input| match input {
            Input::Derivation(dep) => Some(dep.name.clone()),
            _ => None,
        })
        .collect();
    assert!(
        input_names.iter().any(|name| name == SELECTED_SEED_PROVIDER_NAME),
        "example should use selected full-source seed provider: {input_names:?}"
    );
}

#[test]
fn bootstrap_no_nix_example_uses_store_env_glob() {
    let text = std::fs::read_to_string(example_path("bootstrap-no-nix.ncl")).unwrap();

    assert!(text.contains("for d in $NIX_STORE/*-%{seed_name}; do"));
    assert!(!text.contains("for d in /nix/store/*-%{seed_name}; do"));
}
