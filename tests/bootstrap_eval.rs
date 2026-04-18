use std::ffi::OsString;
use std::path::PathBuf;

use crunch_glue::CrunchDerivation;
use crunch_glue::Input;
use serde::Deserialize;

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
    vec![
        crunch_eval::stdlib::stdlib_import_path().unwrap().into_os_string(),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap").into_os_string(),
    ]
}

fn bootstrap_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap").join(name)
}

fn eval_bootstrap(name: &str) -> CrunchDerivation {
    crunch_eval::evaluate_and_deserialize(&bootstrap_path(name), &bootstrap_import_paths()).unwrap()
}

#[derive(Debug, Deserialize)]
struct SeedRawArtifact {
    name: String,
    url: String,
    hash: String,
}

#[derive(Debug, Deserialize)]
struct SeedProviderMetadata {
    id: String,
    summary: String,
    raw: SeedRawArtifact,
    retained_tools: Vec<String>,
    dropped_components: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SeedModule {
    name: String,
    target: String,
    dynamic_linker: String,
    toolchain: CrunchDerivation,
    provider: SeedProviderMetadata,
}

fn eval_seed_module() -> SeedModule {
    crunch_eval::evaluate_and_deserialize(&bootstrap_path("seed.ncl"), &bootstrap_import_paths()).unwrap()
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
fn eval_seed_module_exposes_reduced_provider_metadata() {
    let seed = eval_seed_module();

    assert_eq!(seed.name, "musl-seed-toolchain");
    assert_eq!(seed.toolchain.name, "musl-seed-toolchain");
    assert_eq!(seed.target, "x86_64-linux-musl");
    assert_eq!(seed.dynamic_linker, "ld-musl-x86_64.so.1");
    assert_eq!(seed.provider.id, "musl.cc-native-reduced-v1");
    assert!(seed.provider.summary.contains("Reduced C/C++ bootstrap seed"));
    assert_eq!(seed.provider.raw.name, "musl-gcc-raw");
    assert!(seed.provider.raw.url.contains("musl.cc"));
    assert!(seed.provider.raw.hash.starts_with("sha256-"));
    assert!(seed.provider.retained_tools.contains(&"x86_64-linux-musl-gcc".to_string()));
    assert!(seed.provider.dropped_components.iter().any(|item| item.contains("Fortran")));
}

#[test]
fn bootstrap_entrypoints_do_not_inline_raw_seed_provider_details() {
    for entrypoint in BOOTSTRAP_ENTRYPOINTS {
        let text = std::fs::read_to_string(bootstrap_path(entrypoint)).unwrap();
        assert!(!text.contains("https://musl.cc/"), "raw seed URL leaked into {entrypoint}");
        assert!(!text.contains("musl-gcc-raw"), "raw seed name leaked into {entrypoint}");
        assert!(text.contains("import \"seed.ncl\""), "shared seed import missing in {entrypoint}");
    }
}

#[test]
fn seed_derivation_writes_shared_provider_metadata_schema() {
    let text = std::fs::read_to_string(bootstrap_path("seed.ncl")).unwrap();

    assert!(text.contains("\"raw_size_bytes\": $RAW_SIZE_BYTES"));
    assert!(text.contains("\"reduced_size_bytes\": $REDUCED_SIZE_BYTES"));
    assert!(text.contains("\"retained_tools\": %{std.serialize 'Json provider_retained_tools}"));
    assert!(text.contains("\"notes\": %{std.serialize 'Json provider_notes_list}"));
    assert!(!text.contains("raw_size_mb"));
    assert!(!text.contains("reduced_size_mb"));
}

#[test]
fn busybox_and_bwrap_use_normalized_seed_sysroot_headers_only() {
    for entrypoint in ["busybox.ncl", "bwrap.ncl"] {
        let text = std::fs::read_to_string(bootstrap_path(entrypoint)).unwrap();
        assert!(text.contains("$SEED_ROOT/%{seed_target}/include"), "missing target include in {entrypoint}");
        assert!(!text.contains("$SEED_ROOT/include"), "unexpected top-level include fallback in {entrypoint}");
    }
}

#[test]
fn all_bootstrap_entrypoints_evaluate() {
    for entrypoint in BOOTSTRAP_ENTRYPOINTS {
        let drv = eval_bootstrap(entrypoint);
        assert!(!drv.name.is_empty(), "entrypoint must evaluate: {entrypoint}");
        assert!(!drv.builder.is_empty(), "builder must stay present: {entrypoint}");
    }
}
