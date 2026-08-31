use std::process::Command;

use super::*;

const CHILD_MODE_ENV: &str = "MANTLE_TEST_RUST_ACTION_RUNTIME";
const CHILD_MODE_VALUE: &str = "run";
const TEST_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TEST_EXEC_EVENTS_MAX: u32 = 64;
const TEST_FD_MAX: u32 = 64;
const TEST_JOBS_MAX: u32 = 1;
const TEST_STORAGE_BYTES_MAX: u64 = 1_048_576;

fn resources() -> crate::source_built_rust_action_plan::RustActionResourceLimits {
    crate::source_built_rust_action_plan::RustActionResourceLimits {
        parallel_jobs_max: TEST_JOBS_MAX,
        open_file_descriptors_max: TEST_FD_MAX,
        storage_bytes_max: TEST_STORAGE_BYTES_MAX,
        exec_events_per_action_max: TEST_EXEC_EVENTS_MAX,
    }
}

fn graph() -> crate::rust_plan::UnitDerivationGraphSummary {
    crate::rust_plan::UnitDerivationGraphSummary {
        derivation_count: 1,
        host_unit_count: 0,
        host_artifact_count: 0,
        ready: true,
        digest_blake3: TEST_DIGEST.to_string(),
        derivations: vec![crate::rust_plan::RustUnitDerivationSummary {
            unit_id: "test-unit".to_string(),
            package_id: "test-package".to_string(),
            target_name: "test".to_string(),
            target_kind: "lib".to_string(),
            execution_kind: "target".to_string(),
            selected_triple: "x86_64-unknown-linux-musl".to_string(),
            rustc_metadata_hash: TEST_DIGEST.to_string(),
            crate_types: vec!["lib".to_string()],
            mode: "build".to_string(),
            profile: "release".to_string(),
            source_digest: crate::rust_plan::SourceDigest {
                algorithm: "blake3".to_string(),
                value: TEST_DIGEST.to_string(),
            },
            dependency_artifacts: Vec::new(),
            consumed_host_artifacts: Vec::new(),
            metadata_dependencies: Vec::new(),
            generated_metadata: None,
            derivation: crate::rust_plan::ReviewableRustDerivation {
                name: "test".to_string(),
                builder: "rustc".to_string(),
                system: "x86_64-linux".to_string(),
                args: vec!["src/lib.rs".to_string()],
                outputs: vec!["out/libtest.rlib".to_string()],
                env: BTreeMap::new(),
                inputs: Vec::new(),
                addressing_mode: "input-addressed".to_string(),
            },
            rustc_args_digest_blake3: TEST_DIGEST.to_string(),
        }],
        blockers: Vec::new(),
    }
}

fn fixed_rustc(path: &Path) -> crate::source_built_rust_action_plan::RustFixedExecutableAuthority {
    fixed_tool("test-rustc", path, crate::source_built_rust_action_plan::RustFixedExecutableKind::Rustc)
}

fn fixed_tool(
    authority_id: &str,
    path: &Path,
    kind: crate::source_built_rust_action_plan::RustFixedExecutableKind,
) -> crate::source_built_rust_action_plan::RustFixedExecutableAuthority {
    let digest = crate::protected_exec::blake3_file_hex(path).unwrap();
    crate::source_built_rust_action_plan::RustFixedExecutableAuthority {
        authority_id: authority_id.to_string(),
        producer_action_id: "test-provider".to_string(),
        output_identity_blake3: digest.clone(),
        path: path_string(path),
        digest_blake3: digest,
        kind,
    }
}

#[test]
fn rust_action_runtime_records_complete_ptrace_reconciliation() {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() == Some(CHILD_MODE_VALUE) {
        run_runtime_child();
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("source_built_rust_action_shell::tests::rust_action_runtime_records_complete_ptrace_reconciliation")
        .arg("--nocapture")
        .env(CHILD_MODE_ENV, CHILD_MODE_VALUE)
        .env("MANTLE_TEST_RUST_ACTION_DIR", dir.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "child stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(dir.path().join(RUST_CHILD_ACTION_PLAN_FILE).is_file());
    assert!(dir.path().join(RUST_CHILD_ACTION_AUDIT_FILE).is_file());
    assert!(dir.path().join(RUST_CHILD_ACTION_RECONCILIATION_FILE).is_file());
}

fn run_runtime_child() {
    let evidence_dir = PathBuf::from(std::env::var_os("MANTLE_TEST_RUST_ACTION_DIR").unwrap());
    let rustc = evidence_dir.join("test-rustc");
    fs::write(&rustc, b"#!/bin/sh\nexit 0\n").unwrap();
    let mut permissions = fs::metadata(&rustc).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&rustc, permissions).unwrap();
    let rustc = fs::canonicalize(rustc).unwrap();
    let shell = fs::canonicalize("/bin/sh").unwrap();
    let authority = crate::source_built_rust_action_plan::rust_child_action_authority(
        "mantle-stage1".to_string(),
        resources(),
        vec![
            fixed_rustc(&rustc),
            fixed_tool("test-shell", &shell, crate::source_built_rust_action_plan::RustFixedExecutableKind::Shell),
        ],
    )
    .unwrap();
    let authority_path = evidence_dir.join("authority.json");
    write_new_json(&authority_path, &authority).unwrap();
    let runtime = SourceBuiltRustActionRuntime::start(&authority_path, &graph(), &evidence_dir).unwrap();
    let scope = runtime.begin_action("test-unit", RustChildActionExecutionPhase::CompileUnit).unwrap();
    let mut command = Command::new(&rustc);
    let status = runtime.run_output(&mut command).unwrap().status;
    runtime.end_action(scope).unwrap();
    let reconciliation = runtime.finish().unwrap();

    assert!(status.success());
    assert!(reconciliation.is_complete());
    assert!(reconciliation.observed_event_count >= 1);
}

#[test]
fn rust_action_runtime_rejects_fixed_executable_byte_drift_before_ptrace() {
    let dir = tempfile::tempdir().unwrap();
    let executable = dir.path().join("rustc");
    fs::write(&executable, b"#!/bin/sh\nexit 0\n").unwrap();
    let mut permissions = fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&executable, permissions).unwrap();
    let authority = crate::source_built_rust_action_plan::rust_child_action_authority(
        "mantle-stage1".to_string(),
        resources(),
        vec![fixed_rustc(&fs::canonicalize(&executable).unwrap())],
    )
    .unwrap();
    let authority_path = dir.path().join("authority.json");
    write_new_json(&authority_path, &authority).unwrap();
    fs::write(&executable, b"#!/bin/sh\nexit 1\n").unwrap();

    let error =
        SourceBuiltRustActionRuntime::start(&authority_path, &graph(), &dir.path().join("evidence")).unwrap_err();

    assert!(error.to_string().contains("fixed executable digest mismatch"));
    assert!(!dir.path().join("evidence").join(RUST_CHILD_ACTION_PLAN_FILE).exists());
}
