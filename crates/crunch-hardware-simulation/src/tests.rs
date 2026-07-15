use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crunch_hardware_simulation_core::HardwareProfile;
use crunch_hardware_simulation_core::NIX_SEED_BOUNDARY;
use crunch_hardware_simulation_core::SYSTEM_X86_64_LINUX;
use crunch_hardware_simulation_core::TOOL_COHORT_SCHEMA;
use crunch_hardware_simulation_core::ToolCohort;
use crunch_hardware_simulation_core::ToolMember;
use crunch_hardware_simulation_core::ToolRole;
use tempfile::TempDir;

use super::*;

const TEST_FILE_MODE_EXECUTABLE: u32 = 0o755;
const TEST_TIMEOUT_MS: u64 = 2_000;
const TEST_SHORT_TIMEOUT_MS: u64 = 10;
const TEST_LOG_BOUND: u64 = 1_024;
const PROVEN_TOOL_CLOSURE_PATH_COUNT: usize = 53;
const PROVEN_TOOL_CLOSURE_DIGEST: &str = "32e93e2530eb0b3afcf3fd7062c1543441e02b5ba027a161e76631dcf31f31ed";
const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn deterministic_git_fixture_materialization_preserves_revision_and_tree_identity() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    write_fixture_source(&source);
    let git = resolve_git_for_test();
    let first = materialize_git_fixture(&git, &source, &temp.path().join("repo-a"), Path::new("sentinel.txt")).unwrap();
    let second =
        materialize_git_fixture(&git, &source, &temp.path().join("repo-b"), Path::new("sentinel.txt")).unwrap();

    assert_eq!(first.revision, second.revision);
    assert_eq!(first.measurement, second.measurement);
    assert_eq!(first.measurement.file_count, 2);
    assert!(!first.repository_path.join(".git").join("HEAD").is_symlink());
}

#[test]
fn git_fixture_rejects_existing_destination_and_unsupported_source_kind() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    write_fixture_source(&source);
    let git = resolve_git_for_test();
    let destination = temp.path().join("repo");
    fs::create_dir(&destination).unwrap();
    let existing = materialize_git_fixture(&git, &source, &destination, Path::new("sentinel.txt")).unwrap_err();

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("sentinel.txt", source.join("escape-link")).unwrap();
        let unsupported =
            materialize_git_fixture(&git, &source, &temp.path().join("repo-unsupported"), Path::new("sentinel.txt"))
                .unwrap_err();
        assert!(unsupported.to_string().contains("fixture-source-kind-unsupported"));
    }

    assert!(existing.to_string().contains("git-fixture-destination-exists"));
    assert!(destination.is_dir());
}

#[test]
fn typed_nickel_profile_evaluates_and_malformed_profiles_fail() {
    let fixture_root = hardware_fixture_root();
    let profile_path = fixture_root.join("profile/default.ncl");
    let profile: HardwareProfile = crunch_eval::evaluate_and_deserialize(&profile_path, &[]).unwrap();
    let validated = crunch_hardware_simulation_core::validate_profile(profile).unwrap();

    assert_eq!(validated.selected_source_ids, vec![String::from("selected-ip"), String::from("selected-vip")]);
    assert_eq!(validated.normalized.tool_cohort.members.len(), 5);
    for invalid_name in ["invalid-unknown-field.ncl", "invalid-zero-timeout.ncl"] {
        let invalid_path = fixture_root.join("profile").join(invalid_name);
        let invalid = crunch_eval::evaluate_and_deserialize::<HardwareProfile>(&invalid_path, &[]);
        assert!(invalid.is_err(), "{invalid_name} unexpectedly evaluated");
    }
}

#[test]
fn directory_measurement_rejects_byte_and_file_bounds() {
    let temp = TempDir::new().unwrap();
    write_fixture_source(temp.path());
    let file_bound = measure_directory(temp.path(), Path::new("sentinel.txt"), DirectoryMeasurementLimits {
        maximum_files: 1,
        maximum_file_bytes: TEST_LOG_BOUND,
        maximum_total_bytes: TEST_LOG_BOUND,
    })
    .unwrap_err();
    let byte_bound = measure_directory(temp.path(), Path::new("sentinel.txt"), DirectoryMeasurementLimits {
        maximum_files: 8,
        maximum_file_bytes: 1,
        maximum_total_bytes: TEST_LOG_BOUND,
    })
    .unwrap_err();

    assert!(file_bound.to_string().contains("fixture-file-count-bound-exceeded"));
    assert!(byte_bound.to_string().contains("regular-file-bound-invalid"));
    assert!(temp.path().join("rtl.sv").is_file());
}

#[test]
fn directory_measurement_handles_nested_trees_and_rejects_directory_overflow() {
    let temp = TempDir::new().unwrap();
    write_fixture_source(temp.path());
    let nested = temp.path().join("nested").join("deeper");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("unit.sv"), "module nested; endmodule\n").unwrap();
    let measured =
        measure_directory(temp.path(), Path::new("sentinel.txt"), DEFAULT_DIRECTORY_MEASUREMENT_LIMITS).unwrap();

    let overflow = temp.path().join("overflow");
    write_fixture_source(&overflow);
    for directory_index in 0..MAX_FIXTURE_DIRECTORY_COUNT {
        fs::create_dir(overflow.join(format!("directory-{directory_index}"))).unwrap();
    }
    let rejected =
        measure_directory(&overflow, Path::new("sentinel.txt"), DEFAULT_DIRECTORY_MEASUREMENT_LIMITS).unwrap_err();

    assert_eq!(measured.file_count, 3);
    assert!(measured.total_bytes > 0);
    assert!(rejected.to_string().contains("fixture-directory-count-bound-exceeded"));
}

#[test]
fn tool_observation_hashes_only_declared_member_files_and_closure_paths() {
    let temp = TempDir::new().unwrap();
    let mut members = Vec::new();
    for (index, role) in [
        ToolRole::Verilator,
        ToolRole::CxxCompiler,
        ToolRole::Linker,
        ToolRole::RuntimeSupport,
        ToolRole::Shell,
    ]
    .into_iter()
    .enumerate()
    {
        let root = temp.path().join(format!("tool-{index}"));
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::write(root.join("bin/tool"), format!("tool-{index}\n")).unwrap();
        members.push(ToolMember {
            role,
            package: format!("tool-{index}"),
            version: String::from("1"),
            store_path: root.display().to_string(),
            executable: String::from("bin/tool"),
            binary_blake3: String::from(DIGEST),
        });
    }
    let cohort = ToolCohort {
        schema: String::from(TOOL_COHORT_SCHEMA),
        identity_blake3: String::from(DIGEST),
        seed_boundary: String::from(NIX_SEED_BOUNDARY),
        system: String::from(SYSTEM_X86_64_LINUX),
        members,
        closure_paths_blake3: String::from(DIGEST),
    };
    let closure = cohort.members.iter().map(|member| PathBuf::from(&member.store_path)).collect::<Vec<_>>();
    let observation = observe_tool_cohort(&cohort, closure.clone()).unwrap();
    let mut oversized = cohort.clone();
    oversized.members.push(cohort.members[0].clone());
    let rejected = observe_tool_cohort(&oversized, closure.clone()).unwrap_err();

    assert_eq!(observation.member_binary_blake3.len(), cohort.members.len());
    assert_eq!(observation.closure_paths.len(), closure.len());
    assert_ne!(observation.closure_paths_blake3, DIGEST);
    assert!(observation.member_binary_blake3.values().all(|digest| digest.len() == DIGEST.len()));
    assert!(rejected.to_string().contains("tool-member-count-bound-exceeded"));
}

#[test]
fn checked_tool_closure_manifest_has_proven_identity() {
    let manifest = include_str!("../../../tests/fixtures/hardware-simulation/tool-closure-paths.txt");
    let paths = manifest.lines().map(PathBuf::from).collect::<Vec<_>>();
    let canonical = canonical_paths(paths).unwrap();

    assert_eq!(canonical.len(), PROVEN_TOOL_CLOSURE_PATH_COUNT);
    assert_eq!(hash_path_list(CLOSURE_HASH_DOMAIN, &canonical), PROVEN_TOOL_CLOSURE_DIGEST);
    assert!(canonical.iter().all(|path| path.starts_with("/nix/store")));
}

#[test]
fn strict_action_uses_exact_executable_and_bounded_logs() {
    let temp = TempDir::new().unwrap();
    let fake_bwrap = write_fake_bwrap(temp.path(), false);
    let program = write_program(temp.path(), "program.sh", "printf 'strict-ok'; printf 'strict-err' >&2\n");
    let writable = temp.path().join("work");
    let request = StrictActionRequest {
        bwrap: fake_bwrap,
        executable: program,
        args: Vec::new(),
        env: BTreeMap::new(),
        read_only_paths: vec![temp.path().to_path_buf()],
        read_only_bindings: Vec::new(),
        writable_root: writable,
        timeout_ms: TEST_TIMEOUT_MS,
        max_log_bytes: TEST_LOG_BOUND,
    };
    let result = run_strict_action(request).unwrap();

    assert_eq!(result.exit_code, Some(0));
    assert_eq!(result.stdout, b"strict-ok");
    assert_eq!(result.stderr, b"strict-err");
    assert!(result.elapsed_diagnostic_ns > 0);
}

#[test]
fn strict_action_preserves_nonzero_exit_status_as_explicit_evidence() {
    let temp = TempDir::new().unwrap();
    let fake_bwrap = write_fake_bwrap(temp.path(), true);
    let program = write_program(temp.path(), "program.sh", "printf 'unreachable'\n");
    let request = StrictActionRequest {
        bwrap: fake_bwrap,
        executable: program,
        args: Vec::new(),
        env: BTreeMap::new(),
        read_only_paths: vec![temp.path().to_path_buf()],
        read_only_bindings: Vec::new(),
        writable_root: temp.path().join("failed-work"),
        timeout_ms: TEST_TIMEOUT_MS,
        max_log_bytes: TEST_LOG_BOUND,
    };
    let result = run_strict_action(request).unwrap();

    assert_eq!(result.exit_code, Some(91));
    assert!(result.stdout.is_empty());
    assert!(result.stderr.is_empty());
}

#[test]
fn strict_action_rejects_relative_fallback_and_times_out() {
    let temp = TempDir::new().unwrap();
    let fake_bwrap = write_fake_bwrap(temp.path(), false);
    let program = write_program(temp.path(), "slow.sh", "while true; do :; done\n");
    let relative = StrictActionRequest {
        bwrap: PathBuf::from("bwrap"),
        executable: program.clone(),
        args: Vec::new(),
        env: BTreeMap::new(),
        read_only_paths: vec![temp.path().to_path_buf()],
        read_only_bindings: Vec::new(),
        writable_root: temp.path().join("relative-work"),
        timeout_ms: TEST_TIMEOUT_MS,
        max_log_bytes: TEST_LOG_BOUND,
    };
    let timed = StrictActionRequest {
        bwrap: fake_bwrap,
        executable: program,
        args: Vec::new(),
        env: BTreeMap::new(),
        read_only_paths: vec![temp.path().to_path_buf()],
        read_only_bindings: Vec::new(),
        writable_root: temp.path().join("timed-work"),
        timeout_ms: TEST_SHORT_TIMEOUT_MS,
        max_log_bytes: TEST_LOG_BOUND,
    };

    let mut invalid_binding = timed.clone();
    invalid_binding.read_only_bindings.push(ReadOnlyBinding {
        source: invalid_binding.executable.clone(),
        guest: PathBuf::from("bin/sh"),
    });

    assert!(run_strict_action(relative).unwrap_err().to_string().contains("executable-not-absolute"));
    assert!(
        run_strict_action(invalid_binding)
            .unwrap_err()
            .to_string()
            .contains("strict-action-binding-guest-invalid")
    );
    let timeout_result = run_strict_action(timed);
    assert!(matches!(timeout_result, Err(ShellError::Timeout { .. })), "{timeout_result:?}");
}

fn hardware_fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/hardware-simulation")
}

fn resolve_git_for_test() -> PathBuf {
    let path = std::env::var_os("PATH").expect("test PATH must be set");
    std::env::split_paths(&path)
        .map(|directory| directory.join("git"))
        .find(|candidate| candidate.is_file())
        .expect("test PATH must expose an exact git executable")
}

fn write_fixture_source(root: &Path) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join("sentinel.txt"), "selected-fixture-sentinel\n").unwrap();
    fs::write(root.join("rtl.sv"), "module tiny; endmodule\n").unwrap();
}

fn write_fake_bwrap(root: &Path, fail: bool) -> PathBuf {
    let fail_line = if fail { "exit 91" } else { "" };
    write_program(
        root,
        "fake-bwrap.sh",
        &format!(
            "{fail_line}\nwhile [ \"$#\" -gt 0 ]; do [ \"$1\" = -- ] && shift && break; shift; done\nexec \"$@\"\n"
        ),
    )
}

fn write_program(root: &Path, name: &str, body: &str) -> PathBuf {
    let path = root.join(name);
    fs::write(&path, format!("#!/bin/sh\nset -eu\n{body}")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(TEST_FILE_MODE_EXECUTABLE)).unwrap();
    }
    path
}
