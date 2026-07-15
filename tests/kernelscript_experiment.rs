use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use crunch_glue::CrunchDerivation;
use crunch_kernelscript_core::ExperimentProfile;
use crunch_kernelscript_core::GeneratedProjectFacts;
use crunch_kernelscript_core::ObservedGeneratedFile;
use crunch_kernelscript_core::classify_generated_project;
use crunch_kernelscript_core::validate_profile;
use serde_json::Value;

const FIXTURE_ROOT: &str = "tests/fixtures/kernelscript-experiment";
const PROFILE_FIXTURE: &str = "tests/fixtures/kernelscript-experiment/profile-positive.ncl";
const SOURCE_DERIVATION: &str = "packages/kernelscript-experiment/source.ncl";
const UPSTREAM_PINS: &str = "packages/kernelscript-experiment/upstream-pins.ncl";
const PRODUCTION_NIX: &str = "nix/kernelscript-experiment.nix";
const OFFICIAL_SOURCE_SRI: &str = "sha256-mgC5bh8SfUgGwosHbycKzcS/SoxVjKY2v9n0kmi0ecE=";
const OFFICIAL_SOURCE_BLAKE3: &str = "439431f81df45b043c218f4f5a41917ddd616e0defa35ff134c1cf5273124a57";
const EXPECTED_GENERATED_FILE_COUNT: usize = 6;
const EXPECTED_COMPILER_DEPENDENCY_COUNT: usize = 4;
const EXPECTED_CORE_ADAPTER_INVOCATIONS: usize = 2;
const GENERATED_PATHS: &[(&str, &str)] = &[
    ("generated-userspace-probe/demo.c", "demo.c"),
    ("generated-userspace-probe/demo.ebpf.c", "demo.ebpf.c"),
    ("generated-userspace-probe/Makefile", "Makefile"),
    ("generated-kfunc-module/demo.mod.c", "demo.mod.c"),
    ("generated-kfunc-module/Kbuild", "Kbuild"),
    ("generated-kfunc-module/demo.test.c", "demo.test.c"),
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn import_paths() -> Vec<OsString> {
    vec![
        repo_root().join("lib").into_os_string(),
        repo_root().join(FIXTURE_ROOT).into_os_string(),
    ]
}

fn profile_fixture() -> ExperimentProfile {
    crunch_eval::evaluate_and_deserialize(&repo_root().join(PROFILE_FIXTURE), &import_paths()).unwrap()
}

fn observed_generated_files() -> Vec<ObservedGeneratedFile> {
    GENERATED_PATHS
        .iter()
        .map(|(fixture_path, manifest_path)| ObservedGeneratedFile {
            relative_path: String::from(*manifest_path),
            bytes: std::fs::read(repo_root().join(FIXTURE_ROOT).join(fixture_path)).unwrap(),
            executable: false,
        })
        .collect()
}

#[test]
fn typed_nickel_profile_matches_pure_core_and_exact_fixture_files() {
    let profile = profile_fixture();
    let validation = validate_profile(profile.clone());
    let classified = classify_generated_project(GeneratedProjectFacts {
        profile: profile.clone(),
        files: observed_generated_files(),
    });

    assert!(validation.blockers.is_empty(), "blockers: {:?}", validation.blockers);
    assert!(validation.profile_identity_blake3.is_some());
    assert_eq!(profile.compiler.dependency_lock.dependencies.len(), EXPECTED_COMPILER_DEPENDENCY_COUNT);
    assert_eq!(
        usize::try_from(profile.compiler.dependency_lock.dependency_count).unwrap(),
        profile.compiler.dependency_lock.dependencies.len()
    );
    assert!(classified.blockers.is_empty(), "blockers: {:?}", classified.blockers);
    assert_eq!(classified.manifest.unwrap().members.len(), EXPECTED_GENERATED_FILE_COUNT);
}

#[test]
fn nickel_contract_rejects_default_enable_and_unknown_production_field() {
    let enable_source = r#"
let mantle = import "lib.ncl" in
let profile = import "profile-positive.ncl" in
(std.record.insert "enabled_by_default" true profile) | mantle.KernelScriptExperimentProfile
"#;
    let production_source = r#"
let mantle = import "lib.ncl" in
let profile = import "profile-positive.ncl" in
(std.record.insert "production_ready" true profile) | mantle.KernelScriptExperimentProfile
"#;
    let enabled = crunch_eval::evaluate_str_to_json(enable_source, &import_paths());
    let production = crunch_eval::evaluate_str_to_json(production_source, &import_paths());

    assert!(enabled.is_err());
    assert!(production.is_err());
    assert!(enabled.unwrap_err().to_string().contains("enabled_by_default"));
    assert!(production.unwrap_err().to_string().contains("production_ready"));
}

#[test]
fn official_source_derivation_is_flat_fixed_output_and_does_not_claim_compiler_closure() {
    let derivation: CrunchDerivation =
        crunch_eval::evaluate_and_deserialize(&repo_root().join(SOURCE_DERIVATION), &import_paths()).unwrap();
    let pins: Value = crunch_eval::evaluate_and_deserialize(&repo_root().join(UPSTREAM_PINS), &import_paths()).unwrap();
    let fixed = derivation.fixed_output.expect("source archive must be fixed-output");

    assert_eq!(derivation.builder, "builtin:fetchurl");
    assert_eq!(fixed.hash, OFFICIAL_SOURCE_SRI);
    assert_eq!(fixed.mode, "flat");
    assert_eq!(pins["source_archive"]["blake3"], OFFICIAL_SOURCE_BLAKE3);
    assert_eq!(pins["compiler_dependency_lock"]["authoritative_lock_available"], false);
    assert_eq!(pins["release_binary"]["accepted_as_compiler_closure"], false);
}

#[test]
fn fixture_source_blake3_matches_typed_profile_without_embedding_source_in_receipts() {
    let profile = profile_fixture();
    let source = std::fs::read(repo_root().join(FIXTURE_ROOT).join("program.ks")).unwrap();
    let digest = blake3::hash(&source).to_hex().to_string();

    assert_eq!(digest, profile.source.digest_blake3.as_str());
    assert_eq!(u64::try_from(source.len()).unwrap(), profile.source.size_bytes);
    assert!(!serde_json::to_string(&profile).unwrap().contains("observe_exit"));
}

#[test]
fn production_shell_delegates_shape_and_receipt_semantics_to_the_core_adapter() {
    let source = std::fs::read_to_string(repo_root().join(PRODUCTION_NIX)).unwrap();
    let invocation_count = source.matches("\"$CORE_ADAPTER\" \"$").count();

    assert_eq!(invocation_count, EXPECTED_CORE_ADAPTER_INVOCATIONS);
    assert!(source.contains("mantle-kernelscript-core-adapter"));
    assert!(source.contains("probe-core-report.json"));
    assert!(source.contains("kfunc-core-report.json"));
    assert!(source.contains("generated-shapes-admitted-receipts-blocked-on-external-target-authority"));
    assert!(source.contains("checked-nix-module-build"));
    assert!(source.contains("checked-by-separate-nixos-vm-smoke"));
    assert!(!source.contains("blocked-no-checked-nix-build-or-vm-load-gate"));
    assert!(!source.contains("exact_shape()"));
    assert!(!source.contains("generated shape drift"));
    assert!(!source.contains("find \"$directory\""));
    assert!(!source.contains("diff -u \"$expected\""));
}

#[test]
fn fixture_files_are_regular_and_makefile_is_evidence_not_an_executable_test_input() {
    for (fixture_path, _) in GENERATED_PATHS {
        let path = repo_root().join(FIXTURE_ROOT).join(fixture_path);
        let metadata = std::fs::symlink_metadata(&path).unwrap();
        assert!(metadata.file_type().is_file(), "{}", path.display());
        assert!(!metadata.file_type().is_symlink(), "{}", path.display());
    }
    let makefile_path = repo_root().join(FIXTURE_ROOT).join("generated-userspace-probe/Makefile");
    assert!(Path::new(&makefile_path).ends_with("Makefile"));
}
