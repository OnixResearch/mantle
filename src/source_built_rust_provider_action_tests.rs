use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use super::*;

const CHILD_MODE_ENV: &str = "MANTLE_TEST_RUST_PROVIDER_ACTION";
const CHILD_MODE_VALUE: &str = "run";
const TEST_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TEST_EVENT_MAX: u32 = 64;
const TEST_FD_MAX: u32 = 64;
const TEST_STORAGE_MAX: u64 = 1_048_576;
const V85_EXEC_EVENTS_PER_STAGE_MAX: u32 = 262_144;
const V85_STAGE_COUNT: u32 = 5;
const V85_AGGREGATE_EVENT_COUNT: u32 = 391_207;
const V85_AGGREGATE_EVENT_COUNT_MAX: u32 = 1_310_720;

#[test]
fn rust_provider_action_runtime_pins_outputs_and_emits_complete_evidence() {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() == Some(CHILD_MODE_VALUE) {
        run_child();
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg(
            "source_built_rust_provider_action::tests::rust_provider_action_runtime_pins_outputs_and_emits_complete_evidence",
        )
        .arg("--nocapture")
        .env(CHILD_MODE_ENV, CHILD_MODE_VALUE)
        .env("MANTLE_TEST_RUST_PROVIDER_ACTION_ROOT", dir.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "child stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(dir.path().join("evidence").join(RUST_PROVIDER_ACTION_PLAN_FILE).is_file());
    assert!(dir.path().join("evidence").join(RUST_PROVIDER_ACTION_AUDIT_FILE).is_file());
    assert!(dir.path().join("evidence").join(RUST_PROVIDER_ACTION_RECONCILIATION_FILE).is_file());
    assert!(dir.path().join("evidence/stages/stage-a-audit.json").is_file());
}

fn run_child() {
    let root = PathBuf::from(std::env::var_os("MANTLE_TEST_RUST_PROVIDER_ACTION_ROOT").unwrap());
    let context = context(&root);
    let route = route();
    let route_digest = digest_serialized(b"test-route\0", &route).unwrap();
    let mut runtime =
        RustProviderActionRuntime::start(&context, &route, &route_digest, &root.join("evidence"), limits()).unwrap();
    let plan_path = write_text(&root.join("stage-plan.json"), "plan\n", false);
    let script_path = write_text(&root.join("stage-script.sh"), "#!/bin/sh\nexit 0\n", true);
    let sources_path = write_text(&root.join("stage-sources.json"), "{}\n", false);
    let output_root = root.join("stage-output");
    fs::create_dir(&output_root).unwrap();
    let scope = runtime
        .begin_stage(RustProviderStageActionInput {
            stage_id: "stage-a".to_string(),
            stage_kind: RustSourceProviderBootstrapStageKind::MrustcSeed,
            predecessor_stage_id: None,
            plan_path,
            script_path,
            sources_manifest_path: sources_path,
            bootstrap_metadata_path: None,
            output_roots: vec![output_root.clone()],
        })
        .unwrap();
    let native_backend = root.join("native/bin/g++.real");
    let mut native_command = Command::new(&native_backend);
    let native_status = runtime.run_status(&mut native_command).unwrap();
    let generated = write_text(&output_root.join("generated-rustc"), "#!/bin/sh\nexit 0\n", true);
    let mut generated_command = Command::new(&generated);
    let status = runtime.run_status(&mut generated_command).unwrap();
    runtime.end_stage(scope, native_status.success() && status.success()).unwrap();
    let evidence = runtime.finish().unwrap();
    let revalidated = validate_rust_provider_action_evidence(&root.join("evidence")).unwrap();

    assert!(native_status.success());
    assert!(status.success());
    assert!(evidence.plan_path.is_file());
    assert!(evidence.audit_path.is_file());
    assert!(evidence.reconciliation_path.is_file());
    assert_eq!(evidence.plan_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    assert_eq!(evidence.audit_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    assert_eq!(evidence.reconciliation_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    assert_eq!(revalidated.plan_digest_blake3, evidence.plan_digest_blake3);
    assert_eq!(revalidated.reconciliation_digest_blake3, evidence.reconciliation_digest_blake3);
}

#[test]
fn rust_provider_aggregate_event_limit_scales_with_stages_and_rejects_invalid_factors() {
    let aggregate_max = aggregate_exec_event_count_max(V85_EXEC_EVENTS_PER_STAGE_MAX, V85_STAGE_COUNT).unwrap();
    let zero_stage_error = aggregate_exec_event_count_max(V85_EXEC_EVENTS_PER_STAGE_MAX, 0).unwrap_err();
    let overflow_error = aggregate_exec_event_count_max(u32::MAX, V85_STAGE_COUNT).unwrap_err();

    assert_eq!(aggregate_max, V85_AGGREGATE_EVENT_COUNT_MAX);
    assert!(V85_AGGREGATE_EVENT_COUNT <= aggregate_max);
    assert!(zero_stage_error.to_string().contains("factors must be positive"));
    assert!(overflow_error.to_string().contains("aggregate event limit overflow"));
}

#[test]
fn rust_provider_action_core_rejects_overlapping_roots_and_denied_events() {
    let root = PathBuf::from("/tmp/rust-provider-stage");
    let overlap =
        normalized_output_roots("stage-a", vec![root.clone(), root.join("nested")], TEST_EVENT_MAX).unwrap_err();
    let event = ProtectedSeccompAuditEvent {
        pid: 1,
        syscall: "execve".to_string(),
        executable_path: root.join("tool"),
        tracee_path: root.join("tool"),
        resolved_host_path: root.join("tool"),
        digest_hex: TEST_DIGEST.to_string(),
        reason: "denied fixture".to_string(),
        phase: "protected".to_string(),
        inventory_entry_id: None,
        policy_decision: "denied".to_string(),
    };
    let audit = stage_audit(TEST_DIGEST, std::slice::from_ref(&event), &[]).unwrap();
    let reconciliation = stage_reconciliation(TEST_DIGEST, true, &[event], &[], TEST_EVENT_MAX).unwrap();

    assert!(overlap.to_string().contains("output roots overlap"));
    assert_eq!(audit.raw_event_count, 1);
    assert_eq!(audit.raw_events[0].policy_decision, "denied");
    assert_eq!(reconciliation.denied_event_count, 1);
    assert!(reconciliation.blockers.iter().any(|blocker| blocker == "denied-stage-exec-events"));
}

fn limits() -> RustProviderActionLimits {
    RustProviderActionLimits {
        parallel_jobs_max: 1,
        open_file_descriptors_max: TEST_FD_MAX,
        storage_bytes_max: TEST_STORAGE_MAX,
        exec_events_per_stage_max: TEST_EVENT_MAX,
    }
}

fn route() -> RustSourceProviderBootstrapPlan {
    RustSourceProviderBootstrapPlan {
        schema: "test-route-v1".to_string(),
        provider_id: "test-provider".to_string(),
        route: "test".to_string(),
        host_triple: "x86_64-unknown-linux-musl".to_string(),
        target_triple: "x86_64-unknown-linux-musl".to_string(),
        final_version: "1.0.0".to_string(),
        policy: crate::source_toolchain_closure::RustSourceProviderBootstrapPolicy {
            source_built: true,
            uses_prebuilt_rust: false,
            forbids_prebuilt_rust: true,
            reference: "test".to_string(),
        },
        sources: Vec::new(),
        stages: vec![crate::source_toolchain_closure::RustSourceProviderBootstrapStage {
            id: "stage-a".to_string(),
            kind: RustSourceProviderBootstrapStageKind::MrustcSeed,
            source_ids: Vec::new(),
            bootstrap_stage_id: String::new(),
            rust_version: "1.0.0".to_string(),
            outputs: vec![crate::source_toolchain_closure::RustProviderRole::Rustc],
            notes: Vec::new(),
        }],
        final_outputs: Vec::new(),
    }
}

fn context(root: &Path) -> FullSourceRustExecutionContext {
    let native = root.join("native");
    for (relative, role) in crate::full_source_rust_binding::required_full_source_native_artifacts() {
        let executable = native_role_is_executable(*role);
        write_text(&native.join(relative), "#!/bin/sh\nexit 0\n", executable);
    }
    let shell = fs::canonicalize("/bin/sh").unwrap();
    let shell_digest = crate::protected_exec::blake3_file_hex(&shell).unwrap();
    let archive = root.join("archives");
    let headers = root.join("headers");
    fs::create_dir_all(&archive).unwrap();
    fs::create_dir_all(&headers).unwrap();
    FullSourceRustExecutionContext {
        native_provider_dir: native,
        rust_source_archive_dir: archive,
        linux_headers_root: headers,
        admission: crate::full_source_rust_binding::FullSourceNativeProviderAdmissionIdentity {
            schema: "test-admission-v1".to_string(),
            status: "admitted".to_string(),
            provider_id: "test-native".to_string(),
            provider_target: "x86_64-linux-musl".to_string(),
            compiler_target: "x86_64-unknown-linux-musl".to_string(),
            admission_report_digest_blake3: TEST_DIGEST.to_string(),
            metadata_digest_blake3: TEST_DIGEST.to_string(),
            output_digest_blake3: TEST_DIGEST.to_string(),
            expected_output_digest_blake3: TEST_DIGEST.to_string(),
            source_closure_manifest_blake3: TEST_DIGEST.to_string(),
            expected_source_closure_manifest_blake3: TEST_DIGEST.to_string(),
            source_closure_record_count: 1,
        },
        host_tools: crate::full_source_rust_binding_shell::FullSourceRustHostToolObservation {
            manifest: crate::full_source_rust_binding::FullSourceRustHostToolManifest {
                schema: "test-host-tools-v1".to_string(),
                source_policy: "authenticated-offline-only".to_string(),
                ambient_tool_discovery: false,
                tools: vec![crate::full_source_rust_binding::FullSourceRustHostToolBinding {
                    role: crate::full_source_rust_binding::FullSourceRustHostToolRole::Busybox,
                    path: shell.display().to_string(),
                    content_digest_blake3: shell_digest,
                    source_id: "test-shell-source".to_string(),
                    construction_receipt_path: root.join("shell-receipt.json").display().to_string(),
                    construction_receipt_digest_blake3: TEST_DIGEST.to_string(),
                }],
                support_inputs: Vec::new(),
            },
            manifest_digest_blake3: TEST_DIGEST.to_string(),
        },
    }
}

fn write_text(path: &Path, text: &str, executable: bool) -> PathBuf {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
    if executable {
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }
    path.to_path_buf()
}
