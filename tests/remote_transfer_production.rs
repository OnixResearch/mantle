#[path = "support/remote_endpoint_binding.rs"]
mod remote_endpoint_binding;

use std::collections::BTreeMap;
use std::fs;
use std::os::fd::AsRawFd as _;
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;

use base64::Engine;
use serde_json::Value;
use sha2::Digest;

// r[verify examples.resumable_remote_transfer_workflow]

const STORE_PREFIX: &str = "/mantle/store";
const INPUT_STORE_PATH: &str = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-production-input";
const INPUT_BYTES: usize = 196_608;
const OUTPUT_BYTES: usize = 262_144;
const PRODUCTION_SCALE_OUTPUT_BYTES: usize = 8_388_608;
const MIN_PRODUCTION_SCALE_OUTPUT_CHUNKS: usize = 100;
const PATTERN_MODULUS: usize = 251;
const INTERRUPT_AFTER_CHUNKS: &str = "1";
const TICKET_ID: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const TICKET_SECRET: &str = "QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI";
const TICKET_KEY: &str = "ticket-key-1:QUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUE";
const TICKET_VERIFIER_DOMAIN: &[u8] = b"mantle-remote-ticket-verifier-v2\0";
const RESULT_SIGNING_KEY: &str =
    "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";
const BUILDER_ID: &str = "production-builder";
const MAX_BUILD_TIME_SECS: u64 = 600;
const MAX_UPLOAD_BYTES: u64 = 1_073_741_824;
const REJECTING_UPLOAD_BYTES: u64 = 1;
const TICKET_USES: u32 = 3;
const TEST_TICKET_TTL_SECS: u64 = 3_600;
const INPUT_INTERRUPT_ENV: &str = "MANTLE_TEST_REMOTE_INTERRUPT_AFTER_INPUT_CHUNKS";
const INTERRUPT_ENV: &str = "MANTLE_TEST_REMOTE_INTERRUPT_AFTER_OUTPUT_CHUNKS";
const MAX_TEST_REMOTE_WORKERS: usize = 16;
const VALID_TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
const VALID_TRACESTATE: &str = "vendor=value";
const TRACEPARENT_ENV: &str = "TRACEPARENT";
const TRACESTATE_ENV: &str = "TRACESTATE";
const OTLP_OUTAGE_ENDPOINT: &str = "http://127.0.0.1:9/v1/metrics";
const OTLP_TEST_TIMEOUT_MS: u32 = 100;
const MALFORMED_TRACEPARENT_BYTES: usize = 4_096;
const REMOTE_ATTEMPT_LOG_DIR: &str = "remote-attempt-logs";
const REMOTE_FAILURE_DEBUG_BUNDLE_DIR: &str = "remote-failure-debug/bundles";
const EXPECTED_FAILURE_DEBUG_BUNDLES: usize = 1;
const MAX_DIAGNOSTIC_SCAN_FILES: usize = 4_096;
const MAX_DIAGNOSTIC_SCAN_BYTES: usize = 1_048_576;
const MAX_DIAGNOSTIC_JSON_NODES: usize = 65_536;
const PRIORITY_LIFECYCLE_PREFIX_COUNT: usize = 5;
const WRONG_BUILDER_KEY: &str = "wrong-builder-key";
const CAPTURED_TRACE: &str = "bounded-trace-event";
const DISALLOWED_TRACE: &str = "DO_NOT_CAPTURE";
const EXPECTED_WORKER_AND_COORDINATOR_BUNDLES: usize = 2;
const BLAKE3_HEX_CHARS: usize = 64;
const REJECTING_CAPTURE_FILE_BYTES: u64 = 4;
const ACCEPTING_CAPTURE_FILE_BYTES: u64 = 4_096;
const TEST_REAL_BWRAP_ENV: &str = "MANTLE_TEST_REAL_BWRAP";
const GALLERY_PROJECT_RELATIVE_PATH: &str = "examples/projects/remote-build-loopback";
const GALLERY_RESUMABLE_SELECTOR: &str = ".#resumable-payload";
const GALLERY_PAYLOAD_BYTES: usize = 262_144;
const REMOTE_TRANSFER_RECEIVER_DIR: &str = "remote-transfer-receiver";
const REMOTE_TRANSFER_CHUNK_DIR: &str = "chunks";
const ACKNOWLEDGED_CHUNK_MISSING_REASON: &str = "acknowledged-chunk-missing";
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
const PRIVATE_FILE_MODE: u32 = 0o600;
const REMOTE_TICKET_INPUT_FD: i32 = 9;
const REMOTE_TICKET_INPUT_FILENAME: &str = "remote-ticket.secret";
const REMOTE_BINDING_CONFIG_FILENAME: &str = "remote-endpoint-binding.json";

/// Serializes the transfer tests that spawn the binary with ticket descriptors.
///
/// These tests race on descriptor and pipe setup when libtest runs them in
/// parallel, which surfaces as rotating failures in different tests of this
/// file. Holding one process-wide lock keeps each test's descriptors private.
fn serial_transfer_guard() -> std::sync::MutexGuard<'static, ()> {
    static GUARD: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    let mutex = GUARD.get_or_init(|| std::sync::Mutex::new(()));
    let guard = match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    debug_assert!(std::ptr::eq(mutex as *const _, GUARD.get().expect("guard initialized") as *const _));
    debug_assert!(guard_is_held(&guard));
    guard
}

/// Whether the caller holds the serialization guard for the transfer tests.
const fn guard_is_held(_guard: &std::sync::MutexGuard<'static, ()>) -> bool {
    true
}

/// Pool mode: one admitted endpoint executes a ready goal through the ordinary
/// Worker, using the same ticket, binding, and local-build receiver as the
/// single-endpoint path.
struct SinglePoolOutcome {
    report: Value,
    _root: tempfile::TempDir,
}

fn run_single_endpoint_pool_build(
    guard: std::sync::MutexGuard<'static, ()>,
    write_build: impl FnOnce(&Path),
) -> SinglePoolOutcome {
    let (output, _worker_store, root) = run_single_endpoint_pool_build_with(guard, write_build, |_| {}, true, None);
    let report: Value = serde_json::from_slice(&output.stdout).expect("pool build JSON report");
    SinglePoolOutcome { report, _root: root }
}

fn run_single_endpoint_pool_build_with(
    _guard: std::sync::MutexGuard<'static, ()>,
    write_build: impl FnOnce(&Path),
    prepare: impl FnOnce(&Path),
    expect_success: bool,
    seam_env: Option<(&str, &str)>,
) -> (std::process::Output, PathBuf, tempfile::TempDir) {
    let root = tempfile::tempdir().expect("pool transfer tempdir");
    let state_dir = root.path().join("state");
    let client_store = root.path().join("client-store");
    let worker_store = root.path().join("worker-store");
    let worker_state = root.path().join("worker-state");
    let build_file = root.path().join("pool-goal.ncl");
    let pool_config = root.path().join("pool.ncl");
    fs::create_dir_all(&state_dir).expect("state dir");
    fs::create_dir_all(&client_store).expect("client store");
    fs::create_dir_all(&worker_store).expect("worker store");
    fs::create_dir_all(&worker_state).expect("worker state");
    write_ticket_state_full(&state_dir, MAX_UPLOAD_BYTES, 8, &[BUILDER_ID]);
    prepare(&state_dir);
    write_build(&build_file);

    let binary = PathBuf::from(env!("CARGO_BIN_EXE_mantle"));
    // The debug binary carries debug info far beyond the transport-program bound;
    // a tiny exec wrapper is the exact transport executable the pool pins.
    let wrapper = root.path().join("mantle-transport.sh");
    fs::write(&wrapper, format!("#!/bin/sh\nexec {} \"$@\"\n", binary.display())).expect("transport wrapper");
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).expect("wrapper mode");
    let binary_digest = blake3::hash(&fs::read(&wrapper).expect("wrapper bytes")).to_hex().to_string();
    let signing_material = RESULT_SIGNING_KEY.split_once(':').expect("signing key material").1;
    let binding_config = remote_binding_config_path(&state_dir);
    let worker_args = vec![
        "--state-dir".to_string(),
        state_dir.display().to_string(),
        "--store".to_string(),
        worker_store.display().to_string(),
        "--store-prefix".to_string(),
        STORE_PREFIX.to_string(),
        "--store-backend".to_string(),
        "snix".to_string(),
        "--remote-binding-config".to_string(),
        binding_config.display().to_string(),
        "remote".to_string(),
        "serve".to_string(),
        "--endpoint-id".to_string(),
        BUILDER_ID.to_string(),
        "--binding".to_string(),
        "stdio-once".to_string(),
        "--executor".to_string(),
        "local-build".to_string(),
        "--execution-state-dir".to_string(),
        worker_state.display().to_string(),
    ];
    let args_json = serde_json::to_string(&worker_args).expect("worker args JSON");
    let pool_source = format!(
        r#"{{
  pools = [{{
    pool_id = "native-pool",
    endpoints = [{{
      endpoint_id = "{builder}",
      transport = 'stdio,
      credential_reference = "pool-bearer",
      transport_program = "{binary}",
      transport_program_digest_blake3 = "{digest}",
      transport_args = {args},
      trusted_output_keys = [{{ public_key_name = "cache.example.com-1", public_key_base64 = "{material}" }}],
    }}],
    session = {{ max_concurrency = 1, job_count_max = 4, session_bytes_max = 1048576 }},
  }}],
}}
"#,
        builder = BUILDER_ID,
        binary = wrapper.display(),
        digest = binary_digest,
        args = args_json,
        material = signing_material,
    );
    fs::write(&pool_config, pool_source).expect("pool configuration");

    let mut command = Command::new(&binary);
    configure_test_sandbox_command(&mut command);
    command.args([
        "--json",
        "--state-dir",
        state_dir.to_str().expect("UTF-8 state dir"),
        "--store",
        client_store.to_str().expect("UTF-8 client store"),
        "--store-prefix",
        STORE_PREFIX,
        "build",
        build_file.to_str().expect("UTF-8 build file"),
        "--no-substitute",
        "--builder-pool",
        "native-pool",
        "--builder-config",
        pool_config.to_str().expect("UTF-8 pool config"),
        "--builder-ticket-fd",
        &format!("{BUILDER_ID}={REMOTE_TICKET_INPUT_FD}"),
        "--remote-build-time-secs",
        &MAX_BUILD_TIME_SECS.to_string(),
    ]);
    command.arg("--remote-binding-config").arg(&binding_config);
    command.env("CREDENTIALS_DIRECTORY", state_dir.join("credentials"));
    if let Some((key, value)) = seam_env {
        command.env(key, value);
    }
    configure_ticket_input_fd(&mut command, &state_dir);
    let output = command.output().expect("run pool build");
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() && expect_success {
        let report: Value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
        let log = report["failed"][0]["saved_log_path"]
            .as_str()
            .and_then(|path| fs::read_to_string(path).ok())
            .unwrap_or_default();
        panic!("pool build status={:?} stderr={stderr} failed={} log={log}", output.status, report["failed"]);
    }
    (output, worker_store, root)
}

fn assert_one_pool_outcome(report: &Value) {
    let outcomes = report["outcomes"].as_array().expect("outcomes array");
    assert_eq!(outcomes.len(), 1, "one routed goal outcome: {report}");
    assert!(report["failed"].as_array().is_some_and(Vec::is_empty));
    assert!(outcomes[0]["outputs"].as_array().is_some_and(|outputs| !outputs.is_empty()));
}

#[test]
fn production_pool_dispatches_one_goal_through_the_worker() {
    let outcome = run_single_endpoint_pool_build(serial_transfer_guard(), write_replayable_derivation);
    assert_one_pool_outcome(&outcome.report);
    let observations = outcome.report["pool_goal_observations"].as_array().expect("pool observations");
    assert_eq!(observations.len(), 1, "one recorded goal observation: {}", outcome.report);
    assert_eq!(observations[0]["endpoint_id"], BUILDER_ID);
    assert!(observations[0]["fence_generation"].as_u64().is_some_and(|fence| fence >= 1));
    assert!(
        observations[0]["trusted_output_key_names"]
            .as_array()
            .is_some_and(|keys| keys.iter().any(|key| key == "cache.example.com-1")),
        "the observation must name the admitted output trust set: {}",
        observations[0]
    );
}

#[test]
fn production_pool_rejects_a_malformed_ticket_credential() {
    let (output, worker_store, _root) = run_single_endpoint_pool_build_with(
        serial_transfer_guard(),
        write_replayable_derivation,
        |state_dir| {
            fs::write(state_dir.join(REMOTE_TICKET_INPUT_FILENAME), b"not-a-ticket\n").expect("malformed ticket");
        },
        false,
        None,
    );
    assert!(!output.status.success(), "a malformed credential must fail the build");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    let message = report["failed"][0]["message"].as_str().unwrap_or(&stderr);
    assert!(
        message.contains("ticket")
            || message.contains("credential")
            || stderr.contains("ticket")
            || stderr.contains("credential"),
        "the failure must name the malformed credential: {message} stderr={stderr}"
    );
    let entries = fs::read_dir(&worker_store).expect("worker store").count();
    assert_eq!(entries, 0, "a malformed credential must not admit or build a goal");
}

/// A producer plan output is scanned, two dependent units are registered, the
/// builder source input is transferred, and the chain is built through the
/// pool: `unit.main` reads the output path of `unit.dep`.
#[test]
fn production_pool_admits_and_builds_a_returned_dynamic_plan_chain() {
    let _guard = serial_transfer_guard();
    let root = tempfile::tempdir().expect("pool dynamic plan tempdir");
    let state_dir = root.path().join("state");
    let client_store = root.path().join("client-store");
    let worker_store = root.path().join("worker-store");
    let worker_state = root.path().join("worker-state");
    let build_file = root.path().join("producer.ncl");
    let pool_config = root.path().join("pool.ncl");
    for dir in [&state_dir, &client_store, &worker_store, &worker_state] {
        fs::create_dir_all(dir).expect("pool dynamic plan directory");
    }
    write_ticket_state_full(&state_dir, MAX_UPLOAD_BYTES, 8, &[BUILDER_ID]);
    let builder_name = "00000000000000000000000000000001-dynplan-builder";
    let builder_logical = format!("{STORE_PREFIX}/{builder_name}");
    let builder_host = client_store.join(builder_name);
    fs::write(
        &builder_host,
        "#!/bin/sh\ncase \"$1\" in\n  dep) printf dep-ok > \"$out\" ;;\n  main) IFS= read -r dep < \"$2\"; printf 'main-%s' \"$dep\" > \"$out\" ;;\nesac\n",
    )
    .expect("plan builder script");
    fs::set_permissions(&builder_host, fs::Permissions::from_mode(0o755)).expect("plan builder mode");
    let plan_name = "00000000000000000000000000000002-dynplan.json";
    let plan_logical = format!("{STORE_PREFIX}/{plan_name}");
    let plan_host = client_store.join(plan_name);
    let plan = serde_json::json!({
        "schema": "mantle-plan-v1",
        "producer": { "logical_name": "producer", "goal_hint": null },
        "sources": [],
        "units": [
            {
                "id": "unit.dep",
                "derivation": {
                    "name": "unit-dep",
                    "builder": builder_logical,
                    "system": "x86_64-linux",
                    "args": ["dep"],
                    "outputs": ["out"],
                    "env": {},
                    "inputs": [{ "kind": "store_path", "path": builder_logical }],
                    "fixed_output": null,
                    "addressing_mode": "input-addressed",
                    "sandbox": "native",
                    "dynamic_plan_outputs": []
                },
                "requested_outputs": ["out"],
                "policy": {
                    "sandbox": "inherit",
                    "substitutions": "inherit",
                    "store_prefix": "inherit",
                    "host_paths": "none"
                }
            },
            {
                "id": "unit.main",
                "derivation": {
                    "name": "unit-main",
                    "builder": builder_logical,
                    "system": "x86_64-linux",
                    "args": ["main", "{{mantle-unit-output:unit.dep:out}}"],
                    "outputs": ["out"],
                    "env": {},
                    "inputs": [
                        { "kind": "store_path", "path": builder_logical },
                        { "kind": "unit_output", "unit": "unit.dep", "output": "out" }
                    ],
                    "fixed_output": null,
                    "addressing_mode": "input-addressed",
                    "sandbox": "native",
                    "dynamic_plan_outputs": []
                },
                "requested_outputs": ["out"],
                "policy": {
                    "sandbox": "inherit",
                    "substitutions": "inherit",
                    "store_prefix": "inherit",
                    "host_paths": "none"
                }
            }
        ],
        "roots": ["unit.main"],
        "provenance": {}
    });
    fs::write(&plan_host, serde_json::to_vec(&plan).expect("plan JSON")).expect("plan file");
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    let producer_source = format!(
        r#"let mantle = import "{}" in
{{
  name = "pool-dynplan-producer",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  args = ["-c", "IFS= read -r line < {}; printf '%s' \"$line\" > \"$plan\""],
  outputs = ["plan"],
  dynamic_plan_outputs = ["plan"],
  env = {{}},
  inputs = ["{}"],
  addressing_mode = 'input-addressed,
  sandbox = 'native,
}} | mantle.Derivation
"#,
        lib.display(),
        plan_logical,
        plan_logical,
    );
    fs::write(&build_file, producer_source).expect("producer derivation");
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_mantle"));
    let wrapper = root.path().join("mantle-transport.sh");
    fs::write(&wrapper, format!("#!/bin/sh\nexec {} \"$@\"\n", binary.display())).expect("transport wrapper");
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).expect("wrapper mode");
    let binary_digest = blake3::hash(&fs::read(&wrapper).expect("wrapper bytes")).to_hex().to_string();
    let signing_material = RESULT_SIGNING_KEY.split_once(':').expect("signing key material").1;
    let binding_config = remote_binding_config_path(&state_dir);
    let worker_args = vec![
        "--state-dir".to_string(),
        state_dir.display().to_string(),
        "--store".to_string(),
        worker_store.display().to_string(),
        "--store-prefix".to_string(),
        STORE_PREFIX.to_string(),
        "--store-backend".to_string(),
        "snix".to_string(),
        "--remote-binding-config".to_string(),
        binding_config.display().to_string(),
        "remote".to_string(),
        "serve".to_string(),
        "--endpoint-id".to_string(),
        BUILDER_ID.to_string(),
        "--binding".to_string(),
        "stdio-once".to_string(),
        "--executor".to_string(),
        "local-build".to_string(),
        "--execution-state-dir".to_string(),
        worker_state.display().to_string(),
    ];
    let args_json = serde_json::to_string(&worker_args).expect("worker args JSON");
    let pool_source = format!(
        r#"{{
  pools = [{{
    pool_id = "native-pool",
    endpoints = [{{
      endpoint_id = "{builder}",
      transport = 'stdio,
      credential_reference = "pool-bearer",
      transport_program = "{binary}",
      transport_program_digest_blake3 = "{digest}",
      transport_args = {args},
      trusted_output_keys = [{{ public_key_name = "cache.example.com-1", public_key_base64 = "{material}" }}],
    }}],
    session = {{ max_concurrency = 1, job_count_max = 8, session_bytes_max = 1048576 }},
  }}],
}}
"#,
        builder = BUILDER_ID,
        binary = wrapper.display(),
        digest = binary_digest,
        args = args_json,
        material = signing_material,
    );
    fs::write(&pool_config, pool_source).expect("pool configuration");
    let local_store = root.path().join("local-store");
    let local_state = root.path().join("local-state");
    fs::create_dir_all(&local_store).expect("local store");
    fs::create_dir_all(&local_state).expect("local state");
    fs::copy(&builder_host, local_store.join(builder_name)).expect("local builder copy");
    fs::copy(&plan_host, local_store.join(plan_name)).expect("local plan copy");
    let local = Command::new(&binary)
        .args([
            "--json",
            "--state-dir",
            local_state.to_str().expect("UTF-8 state dir"),
            "--store",
            local_store.to_str().expect("UTF-8 store"),
            "--store-prefix",
            STORE_PREFIX,
            "build",
            build_file.to_str().expect("UTF-8 build file"),
            "--no-substitute",
        ])
        .output()
        .expect("run local dynamic plan build");
    assert!(
        local.status.success(),
        "local dynamic plan build failed: stdout={} stderr={}",
        String::from_utf8_lossy(&local.stdout),
        String::from_utf8_lossy(&local.stderr)
    );
    let output = pool_build_command(&binary, &state_dir, &client_store, &pool_config, &build_file)
        .output()
        .expect("run pool dynamic plan build");
    if !output.status.success() {
        let report: Value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
        let log = report["failed"][0]["saved_log_path"]
            .as_str()
            .and_then(|path| fs::read_to_string(path).ok())
            .unwrap_or_default();
        panic!(
            "pool dynamic plan build failed: failed={} log={log} stderr={}",
            report["failed"],
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let report: Value = serde_json::from_slice(&output.stdout).expect("pool dynamic plan JSON report");
    assert!(report["failed"].as_array().is_some_and(Vec::is_empty));
    let plans = report["native_dynamic_plans"].as_array().expect("native dynamic plans");
    assert!(
        plans
            .iter()
            .any(|row| row["accepted_unit_ids"].as_array().is_some_and(|ids| ids.iter().any(|id| id == "unit.main"))),
        "the returned plan must be accepted: {report}"
    );
    let mut found = false;
    for outcome in report["outcomes"].as_array().expect("outcomes") {
        for output in outcome["outputs"].as_array().expect("outputs") {
            let path = Path::new(output["path"].as_str().expect("output path"));
            let candidate = if path.is_file() {
                path.to_path_buf()
            } else {
                path.join("out")
            };
            if let Ok(bytes) = fs::read(&candidate)
                && bytes == b"main-dep-ok"
            {
                found = true;
            }
        }
    }
    assert!(found, "the dependent dynamic plan unit output must be built: {report}");
}

/// Admitted pre-dispatch fallback (`fallback_policy = 'always`) runs the goal
/// locally when no endpoint fits; the default policy fails instead.
/// Pool mode: an operator-declared requirement larger than the endpoint's
/// declared inventory is refused; the same requirement within the inventory
/// dispatches normally.
#[test]
fn production_pool_enforces_declared_resource_quantities() {
    let _guard = serial_transfer_guard();
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_mantle"));
    let run = |inventory_cpu: u32, requirement_cpu: u32| -> (std::process::Output, PathBuf) {
        let root = tempfile::tempdir().expect("resource pool tempdir");
        let state_dir = root.path().join("state");
        let client_store = root.path().join("client-store");
        let worker_store = root.path().join("worker-store");
        let worker_state = root.path().join("worker-state");
        let build_file = root.path().join("goal.ncl");
        let pool_config = root.path().join("pool.ncl");
        let requirements = root.path().join("requirements.json");
        for dir in [&state_dir, &client_store, &worker_store, &worker_state] {
            fs::create_dir_all(dir).expect("resource pool directory");
        }
        write_ticket_state_full(&state_dir, MAX_UPLOAD_BYTES, 4, &[BUILDER_ID]);
        write_replayable_derivation(&build_file);
        fs::write(
            &requirements,
            format!(
                r#"{{"quantities":{{"cpu_units":{requirement_cpu},"memory_bytes":1073741824,"scratch_bytes":1073741824}},"semantic_accelerator_classes":[]}}"#
            ),
        )
        .expect("requirements JSON");
        let wrapper = root.path().join("mantle-transport.sh");
        fs::write(&wrapper, format!("#!/bin/sh\nexec {} \"$@\"\n", binary.display())).expect("transport wrapper");
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).expect("wrapper mode");
        let binary_digest = blake3::hash(&fs::read(&wrapper).expect("wrapper bytes")).to_hex().to_string();
        let signing_material = RESULT_SIGNING_KEY.split_once(':').expect("signing key material").1;
        let binding_config = remote_binding_config_path(&state_dir);
        let worker_args = vec![
            "--state-dir".to_string(),
            state_dir.display().to_string(),
            "--store".to_string(),
            worker_store.display().to_string(),
            "--store-prefix".to_string(),
            STORE_PREFIX.to_string(),
            "--store-backend".to_string(),
            "snix".to_string(),
            "--remote-binding-config".to_string(),
            binding_config.display().to_string(),
            "remote".to_string(),
            "serve".to_string(),
            "--endpoint-id".to_string(),
            BUILDER_ID.to_string(),
            "--binding".to_string(),
            "stdio-once".to_string(),
            "--executor".to_string(),
            "local-build".to_string(),
            "--execution-state-dir".to_string(),
            worker_state.display().to_string(),
        ];
        let args_json = serde_json::to_string(&worker_args).expect("worker args JSON");
        let pool_source = format!(
            r#"{{
  pools = [{{
    pool_id = "native-pool",
    endpoints = [{{
      endpoint_id = "{builder}",
      transport = 'stdio,
      credential_reference = "pool-bearer",
      transport_program = "{binary}",
      transport_program_digest_blake3 = "{digest}",
      transport_args = {args},
      trusted_output_keys = [{{ public_key_name = "cache.example.com-1", public_key_base64 = "{material}" }}],
      profile = {{
        system = "x86_64-linux",
        max_concurrency = 1,
        resource_inventory = {{ total = {{ cpu_units = {inventory_cpu}, memory_bytes = 2147483648, scratch_bytes = 2147483648 }} }},
      }},
    }}],
    session = {{ max_concurrency = 1, job_count_max = 4, session_bytes_max = 1048576 }},
  }}],
}}
"#,
            builder = BUILDER_ID,
            binary = wrapper.display(),
            digest = binary_digest,
            args = args_json,
            material = signing_material,
            inventory_cpu = inventory_cpu,
        );
        fs::write(&pool_config, pool_source).expect("pool configuration");
        let mut command = pool_build_command(&binary, &state_dir, &client_store, &pool_config, &build_file);
        command.arg("--remote-resource-requirements").arg(&requirements);
        let output = command.output().expect("run resource pool build");
        (output, root.path().to_path_buf())
    };
    let (fitting, _fitting_root) = run(4, 2);
    if !fitting.status.success() {
        let report: Value = serde_json::from_slice(&fitting.stdout).unwrap_or(Value::Null);
        let log = report["failed"][0]["saved_log_path"]
            .as_str()
            .and_then(|path| fs::read_to_string(path).ok())
            .unwrap_or_default();
        panic!(
            "fitting requirement failed: failed={} log={log} stderr={}",
            report["failed"],
            String::from_utf8_lossy(&fitting.stderr)
        );
    }
    let report: Value = serde_json::from_slice(&fitting.stdout).expect("resource pool JSON report");
    assert!(report["failed"].as_array().is_some_and(Vec::is_empty), "{report}");
    let (oversized, _oversized_root) = run(1, 2);
    assert!(!oversized.status.success(), "a requirement above the inventory must be refused");
    let text = format!("{}{}", String::from_utf8_lossy(&oversized.stdout), String::from_utf8_lossy(&oversized.stderr));
    assert!(
        text.contains("placement-unavailable") || text.contains("no admitted endpoint fits"),
        "unexpected oversized-requirement failure: {text}"
    );
    let unbound = tempfile::tempdir().expect("unbound requirements tempdir");
    let unbound_file = unbound.path().join("goal.ncl");
    let unbound_requirements = unbound.path().join("requirements.json");
    write_replayable_derivation(&unbound_file);
    fs::write(&unbound_requirements, br#"{"quantities":{"cpu_units":1,"memory_bytes":1073741824,"scratch_bytes":1073741824},"semantic_accelerator_classes":[]}"#)
        .expect("unbound requirements JSON");
    let rejected = Command::new(&binary)
        .args([
            "--json",
            "build",
            unbound_file.to_str().expect("UTF-8 build file"),
            "--no-substitute",
            "--remote-resource-requirements",
            unbound_requirements.to_str().expect("UTF-8 requirements"),
        ])
        .output()
        .expect("run unbound requirements build");
    assert!(!rejected.status.success(), "requirements without a remote builder must be refused");
    let rejected_text =
        format!("{}{}", String::from_utf8_lossy(&rejected.stdout), String::from_utf8_lossy(&rejected.stderr));
    assert!(
        rejected_text.contains("requires --builder or --builder-pool"),
        "unexpected unbound-requirements failure: {rejected_text}"
    );
}

/// Pool mode: a job reassigned to a newer fence between admission and
/// completion must reject the stale completion before exporting its outputs.
#[test]
fn production_pool_rejects_a_stale_completion_after_reassignment() {
    let (output, _worker_store, root) = run_single_endpoint_pool_build_with(
        serial_transfer_guard(),
        write_replayable_derivation,
        |_| {},
        false,
        Some(("MANTLE_TEST_REASSIGN_POOL_ATTEMPT", "1")),
    );
    assert!(!output.status.success(), "a stale completion must fail the goal");
    let report: Value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(
        text.contains("attempt-stale") || text.contains("stale-report-rejected"),
        "unexpected stale-completion failure: {text}"
    );
    assert!(
        report["outcomes"].as_array().is_some_and(Vec::is_empty),
        "a stale completion must not report an outcome: {report}"
    );
    let client_entries = fs::read_dir(root.path().join("client-store")).expect("client store entries").count();
    assert_eq!(client_entries, 0, "a stale attempt must not materialize outputs");
    let state: Value = serde_json::from_slice(
        &fs::read(root.path().join("state").join("remote-coordinator-state.json")).expect("coordinator state"),
    )
    .expect("coordinator state JSON");
    let successor = state["jobs"]
        .as_object()
        .expect("jobs")
        .values()
        .any(|job| job["current_attempt"]["fence_generation"].as_u64().is_some_and(|fence| fence > 1));
    assert!(successor, "the reassigned attempt must own a newer fence: {state}");
}

/// A source input produced by an earlier local build lives only in the client
/// store backend; the pool must upload that live store object to the worker.
#[test]
fn production_pool_admits_a_live_private_source_input() {
    let input_path = std::rc::Rc::new(std::cell::RefCell::new(None::<String>));
    let slot = input_path.clone();
    let (output, worker_store, _root) = run_single_endpoint_pool_build_with(
        serial_transfer_guard(),
        move |path| {
            let input = slot.borrow().clone().expect("live input path");
            write_seen_input_derivation(path, &input);
        },
        |state_dir| {
            let client_store = state_dir.parent().expect("state parent").join("client-store");
            let input = run_local_private_input_build(state_dir, &client_store);
            *input_path.borrow_mut() = Some(input);
        },
        true,
        None,
    );
    let report: Value = serde_json::from_slice(&output.stdout).expect("pool private-input JSON report");
    let outcomes = report["outcomes"].as_array().expect("outcomes");
    assert_eq!(outcomes.len(), 1, "one routed goal: {report}");
    let reported = Path::new(outcomes[0]["outputs"][0]["path"].as_str().expect("output path"));
    let output_path = if reported.exists() {
        reported.to_path_buf()
    } else {
        Path::new(report["output_dir"].as_str().expect("output dir"))
            .join(reported.file_name().expect("output basename"))
    };
    assert_eq!(fs::read(&output_path).expect("pool output bytes"), b"seen-private-ok");
    let input = input_path.borrow().clone().expect("live input path");
    let input_file = worker_store.join(Path::new(&input).file_name().expect("input basename"));
    assert_eq!(
        fs::read(&input_file).expect("transferred live input bytes"),
        b"private-ok",
        "the worker store must receive the live input NAR"
    );
}

fn run_local_private_input_build(state_dir: &Path, client_store: &Path) -> String {
    let build_file = state_dir.parent().expect("state parent").join("private-input.ncl");
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    fs::write(
        &build_file,
        format!(
            r#"let mantle = import "{}" in
{{
  name = "production-private-input",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  args = ["-c", "printf private-ok > $out"],
  outputs = ["out"],
  env = {{}},
  inputs = [],
  addressing_mode = 'input-addressed,
  sandbox = 'native,
}} | mantle.Derivation
"#,
            lib.display()
        ),
    )
    .expect("private input derivation");
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_mantle"));
    let mut command = Command::new(&binary);
    configure_test_sandbox_command(&mut command);
    let output = command
        .args([
            "--json",
            "--state-dir",
            state_dir.to_str().expect("UTF-8 state dir"),
            "--store",
            client_store.to_str().expect("UTF-8 client store"),
            "--store-prefix",
            STORE_PREFIX,
            "build",
            build_file.to_str().expect("UTF-8 build file"),
            "--no-substitute",
        ])
        .output()
        .expect("run private input build");
    assert!(output.status.success(), "private input build failed: {}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).expect("private input JSON report");
    let reported = Path::new(report["outcomes"][0]["outputs"][0]["path"].as_str().expect("output path"));
    format!("{STORE_PREFIX}/{}", reported.file_name().expect("output basename").to_string_lossy())
}

fn write_seen_input_derivation(path: &Path, input: &str) {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    let source = format!(
        r#"let mantle = import "{}" in
{{
  name = "production-seen-input",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  args = ["-c", "IFS= read -r value < {}; printf 'seen-%s' \"$value\" > \"$out\""],
  outputs = ["out"],
  env = {{}},
  inputs = ["{}"],
  addressing_mode = 'input-addressed,
  sandbox = 'native,
}} | mantle.Derivation
"#,
        lib.display(),
        input,
        input,
    );
    fs::write(path, source).expect("seen-input derivation");
}

#[test]
fn production_pool_local_fallback_runs_when_no_endpoint_fits() {
    let _guard = serial_transfer_guard();
    let root = tempfile::tempdir().expect("pool fallback tempdir");
    let state_dir = root.path().join("state");
    let client_store = root.path().join("client-store");
    let build_file = root.path().join("goal.ncl");
    let fallback_config = root.path().join("fallback-pool.ncl");
    let strict_config = root.path().join("strict-pool.ncl");
    fs::create_dir_all(&state_dir).expect("state dir");
    fs::create_dir_all(&client_store).expect("client store");
    write_ticket_state_full(&state_dir, MAX_UPLOAD_BYTES, 4, &[BUILDER_ID]);
    write_replayable_derivation(&build_file);
    let program = PathBuf::from("/run/current-system/sw/bin/true");
    let digest = blake3::hash(&fs::read(&program).expect("program bytes")).to_hex().to_string();
    let pool_source = |policy: &str| {
        format!(
            r#"{{
  pools = [{{
    pool_id = "native-pool",
    fallback_policy = '{policy},
    endpoints = [{{
      endpoint_id = "{builder}",
      transport = 'stdio,
      credential_reference = "pool-bearer",
      transport_program = "{}",
      transport_program_digest_blake3 = "{}",
      transport_args = [],
      profile = {{ system = "aarch64-linux" }},
      trusted_output_keys = [{{ public_key_name = "cache.example.com-1", public_key_base64 = "AAAA" }}],
    }}],
    session = {{ max_concurrency = 1, job_count_max = 4, session_bytes_max = 1048576 }},
  }}],
}}
"#,
            program.display(),
            digest,
            builder = BUILDER_ID,
        )
    };
    fs::write(&fallback_config, pool_source("always")).expect("fallback pool configuration");
    fs::write(&strict_config, pool_source("trusted-only")).expect("strict pool configuration");
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_mantle"));
    let strict = pool_build_command(&binary, &state_dir, &client_store, &strict_config, &build_file)
        .output()
        .expect("run strict pool build");
    assert!(!strict.status.success(), "without fallback an unplaceable goal must fail");
    let strict_text = format!("{}{}", String::from_utf8_lossy(&strict.stdout), String::from_utf8_lossy(&strict.stderr));
    assert!(
        strict_text.contains("placement-unavailable") || strict_text.contains("no admitted endpoint fits"),
        "unexpected strict failure: {strict_text}"
    );
    let fallback = pool_build_command(&binary, &state_dir, &client_store, &fallback_config, &build_file)
        .output()
        .expect("run fallback pool build");
    assert!(
        fallback.status.success(),
        "admitted fallback must build locally: stdout={} stderr={}",
        String::from_utf8_lossy(&fallback.stdout),
        String::from_utf8_lossy(&fallback.stderr)
    );
    let report: Value = serde_json::from_slice(&fallback.stdout).expect("fallback JSON report");
    let outcomes = report["outcomes"].as_array().expect("outcomes array");
    assert_eq!(outcomes.len(), 1, "one local outcome: {report}");
    assert!(report["failed"].as_array().is_some_and(Vec::is_empty));
}

/// Two trusted keys with the same name must fail pool admission before dispatch.
#[test]
fn production_pool_rejects_ambiguous_output_keys_before_dispatch() {
    let _guard = serial_transfer_guard();
    let root = tempfile::tempdir().expect("pool ambiguous key tempdir");
    let state_dir = root.path().join("state");
    let client_store = root.path().join("client-store");
    let build_file = root.path().join("goal.ncl");
    let pool_config = root.path().join("pool.ncl");
    fs::create_dir_all(&state_dir).expect("state dir");
    fs::create_dir_all(&client_store).expect("client store");
    write_ticket_state_full(&state_dir, MAX_UPLOAD_BYTES, 4, &[BUILDER_ID]);
    write_replayable_derivation(&build_file);
    let pool_source = format!(
        r#"{{
  pools = [{{
    pool_id = "native-pool",
    endpoints = [{{
      endpoint_id = "{builder}",
      transport = 'stdio,
      credential_reference = "pool-bearer",
      transport_program = "/bin/true",
      transport_program_digest_blake3 = "{digest}",
      transport_args = [],
      trusted_output_keys = [
        {{ public_key_name = "cache.example.com-1", public_key_base64 = "AAAA" }},
        {{ public_key_name = "cache.example.com-1", public_key_base64 = "BBBB" }},
      ],
    }}],
    session = {{ max_concurrency = 1, job_count_max = 4, session_bytes_max = 1048576 }},
  }}],
}}
"#,
        builder = BUILDER_ID,
        digest = "0".repeat(64),
    );
    fs::write(&pool_config, pool_source).expect("pool configuration");
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_mantle"));
    let output = pool_build_command(&binary, &state_dir, &client_store, &pool_config, &build_file)
        .output()
        .expect("run ambiguous-key pool build");
    assert!(!output.status.success(), "ambiguous output keys must reject the pool");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("trusted output keys must be unique"), "unexpected failure: {stderr}");
    assert_eq!(fs::read_dir(&client_store).expect("client store entries").count(), 0);
}

/// A second pool build of the same derivation must hit the client store cache
/// without spawning another worker.
#[test]
fn production_pool_second_build_hits_the_client_cache() {
    let _guard = serial_transfer_guard();
    let root = tempfile::tempdir().expect("pool cache tempdir");
    let state_dir = root.path().join("state");
    let client_store = root.path().join("client-store");
    let worker_store = root.path().join("worker-store");
    let worker_state = root.path().join("worker-state");
    let build_file = root.path().join("goal.ncl");
    let pool_config = root.path().join("pool.ncl");
    let lifecycle_log = root.path().join("worker-lifecycle.log");
    for dir in [&state_dir, &client_store, &worker_store, &worker_state] {
        fs::create_dir_all(dir).expect("pool cache directory");
    }
    write_ticket_state_full(&state_dir, MAX_UPLOAD_BYTES, 8, &[BUILDER_ID]);
    write_replayable_derivation(&build_file);
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_mantle"));
    let wrapper = root.path().join("mantle-transport.sh");
    fs::write(
        &wrapper,
        format!(
            "#!/bin/sh\nprintf 'start %s %s\\n' \"$(date +%s%N)\" \"$$\" >> {}\n{} \"$@\"\nstatus=$?\nprintf 'end %s %s\\n' \"$(date +%s%N)\" \"$$\" >> {}\nexit $status\n",
            lifecycle_log.display(),
            binary.display(),
            lifecycle_log.display()
        ),
    )
    .expect("transport wrapper");
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).expect("wrapper mode");
    let binary_digest = blake3::hash(&fs::read(&wrapper).expect("wrapper bytes")).to_hex().to_string();
    let signing_material = RESULT_SIGNING_KEY.split_once(':').expect("signing key material").1;
    let binding_config = remote_binding_config_path(&state_dir);
    let worker_args = vec![
        "--state-dir".to_string(),
        state_dir.display().to_string(),
        "--store".to_string(),
        worker_store.display().to_string(),
        "--store-prefix".to_string(),
        STORE_PREFIX.to_string(),
        "--store-backend".to_string(),
        "snix".to_string(),
        "--remote-binding-config".to_string(),
        binding_config.display().to_string(),
        "remote".to_string(),
        "serve".to_string(),
        "--endpoint-id".to_string(),
        BUILDER_ID.to_string(),
        "--binding".to_string(),
        "stdio-once".to_string(),
        "--executor".to_string(),
        "local-build".to_string(),
        "--execution-state-dir".to_string(),
        worker_state.display().to_string(),
    ];
    let args_json = serde_json::to_string(&worker_args).expect("worker args JSON");
    let pool_source = format!(
        r#"{{
  pools = [{{
    pool_id = "native-pool",
    endpoints = [{{
      endpoint_id = "{builder}",
      transport = 'stdio,
      credential_reference = "pool-bearer",
      transport_program = "{binary}",
      transport_program_digest_blake3 = "{digest}",
      transport_args = {args},
      trusted_output_keys = [{{ public_key_name = "cache.example.com-1", public_key_base64 = "{material}" }}],
    }}],
    session = {{ max_concurrency = 1, job_count_max = 4, session_bytes_max = 1048576 }},
  }}],
}}
"#,
        builder = BUILDER_ID,
        binary = wrapper.display(),
        digest = binary_digest,
        args = args_json,
        material = signing_material,
    );
    fs::write(&pool_config, pool_source).expect("pool configuration");
    let run = || {
        let output = pool_build_command(&binary, &state_dir, &client_store, &pool_config, &build_file)
            .output()
            .expect("run pool build");
        assert!(output.status.success(), "pool build failed: {}", String::from_utf8_lossy(&output.stderr));
        serde_json::from_slice::<Value>(&output.stdout).expect("pool JSON report")
    };
    let first = run();
    assert_eq!(first["outcomes"][0]["cached"], false, "first build must execute: {first}");
    let second = run();
    assert_eq!(second["outcomes"][0]["cached"], true, "second build must hit the cache: {second}");
    let log = fs::read_to_string(&lifecycle_log).expect("worker lifecycle log");
    assert_eq!(worker_lifetimes(&log).len(), 1, "cache hit must not spawn a worker: {log}");
}

/// Single-builder and pool builds of the same derivation must produce identical
/// output bytes.
#[test]
fn production_single_builder_and_pool_outputs_match() {
    let _guard = serial_transfer_guard();
    let root = tempfile::tempdir().expect("single builder comparison tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    let build_file = root.path().join("single.ncl");
    fs::create_dir_all(&state_dir).expect("state dir");
    fs::create_dir_all(&store_dir).expect("store dir");
    write_ticket_state(&state_dir);
    write_replayable_derivation(&build_file);
    let output = remote_build_command(&state_dir, &store_dir, &build_file)
        .output()
        .expect("run single-builder build");
    assert!(output.status.success(), "single-builder build failed: {}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).expect("single-builder JSON report");
    let reported = Path::new(report["outcomes"][0]["outputs"][0]["path"].as_str().expect("output path"));
    let single_path = if reported.exists() {
        reported.to_path_buf()
    } else {
        Path::new(report["output_dir"].as_str().expect("output dir"))
            .join(reported.file_name().expect("output basename"))
    };
    let single_bytes = fs::read(&single_path).expect("single-builder output bytes");
    let pool_outcome = run_single_endpoint_pool_build(_guard, write_replayable_derivation);
    let pool_report = &pool_outcome.report;
    let reported = Path::new(pool_report["outcomes"][0]["outputs"][0]["path"].as_str().expect("pool output path"));
    let pool_path = if reported.exists() {
        reported.to_path_buf()
    } else {
        Path::new(pool_report["output_dir"].as_str().expect("pool output dir"))
            .join(reported.file_name().expect("output basename"))
    };
    let pool_bytes = fs::read(&pool_path).expect("pool output bytes");
    assert!(!single_bytes.is_empty());
    assert_eq!(pool_bytes, single_bytes, "pool output must match the single-builder build");
    eprintln!(
        "pool-parity single-builder: bytes={} blake3={}",
        single_bytes.len(),
        blake3::hash(&single_bytes).to_hex()
    );
}

/// Local and pool builds of the same derivation must produce identical output
/// bytes.
#[test]
fn production_pool_output_matches_local_build() {
    let local = tempfile::tempdir().expect("local comparison tempdir");
    let local_store = local.path().join("store");
    let local_state = local.path().join("state");
    fs::create_dir_all(&local_store).expect("local store");
    fs::create_dir_all(&local_state).expect("local state");
    let local_file = local.path().join("local.ncl");
    write_replayable_derivation(&local_file);
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_mantle"));
    let local_output = Command::new(&binary)
        .args([
            "--json",
            "--state-dir",
            local_state.to_str().expect("UTF-8 state dir"),
            "--store",
            local_store.to_str().expect("UTF-8 local store"),
            "--store-prefix",
            STORE_PREFIX,
            "build",
            local_file.to_str().expect("UTF-8 build file"),
            "--no-substitute",
        ])
        .output()
        .expect("run local build");
    assert!(
        local_output.status.success(),
        "local build failed: {}",
        String::from_utf8_lossy(&local_output.stderr)
    );
    let local_report: Value = serde_json::from_slice(&local_output.stdout).expect("local JSON report");
    assert!(
        local_report.get("pool_goal_observations").is_none(),
        "ordinary local JSON must not gain the pool observation field"
    );
    let local_path = Path::new(local_report["outcomes"][0]["outputs"][0]["path"].as_str().expect("local output path"));
    let local_bytes = fs::read(local_path).expect("local output bytes");
    let pool_outcome = run_single_endpoint_pool_build(serial_transfer_guard(), write_replayable_derivation);
    let pool_report = &pool_outcome.report;
    let reported = Path::new(pool_report["outcomes"][0]["outputs"][0]["path"].as_str().expect("pool output path"));
    // Pool outcomes may report the logical store path; the physical export is
    // the report's output_dir plus the store basename.
    let pool_path = if reported.exists() {
        reported.to_path_buf()
    } else {
        Path::new(pool_report["output_dir"].as_str().expect("pool output dir"))
            .join(reported.file_name().expect("output basename"))
    };
    let pool_bytes = fs::read(&pool_path)
        .unwrap_or_else(|error| panic!("pool output bytes at {pool_path:?}: {error}; report={pool_report}"));
    assert!(!local_bytes.is_empty());
    assert_eq!(pool_bytes, local_bytes, "pool output must match the local build");
    eprintln!("pool-parity local: bytes={} blake3={}", local_bytes.len(), blake3::hash(&local_bytes).to_hex());
}

#[test]
fn production_pool_diamond_runs_on_one_endpoint_at_concurrency_one() {
    let outcome = run_single_endpoint_pool_build(serial_transfer_guard(), |path| {
        write_pool_diamond_derivation(path, Some("input-addressed"))
    });
    assert_one_pool_outcome(&outcome.report);
    let outputs = outcome.report["outcomes"][0]["outputs"].as_array().expect("root outputs");
    let graph = fs::read_to_string(Path::new(outputs[0]["path"].as_str().expect("root output path")).join("graph.txt"))
        .expect("diamond graph output");
    assert!(graph.contains("shared foundation"), "graph carries the shared dependency: {graph}");
    assert!(graph.contains("left branch"), "graph carries the left branch: {graph}");
    assert!(graph.contains("right branch"), "graph carries the right branch: {graph}");
}

#[test]
fn production_pool_dispatches_one_content_addressed_goal_through_the_worker() {
    let outcome = run_single_endpoint_pool_build(serial_transfer_guard(), write_content_addressed_derivation);
    assert_one_pool_outcome(&outcome.report);
}

/// Pool mode: a failed exchange settles its attempt so the endpoint's
/// concurrency slot is not leaked to the next goal.
#[test]
fn production_pool_failures_release_attempt_capacity() {
    let _guard = serial_transfer_guard();
    let root = tempfile::tempdir().expect("pool failure tempdir");
    let state_dir = root.path().join("state");
    let client_store = root.path().join("client-store");
    let worker_store = root.path().join("worker-store");
    let worker_state = root.path().join("worker-state");
    let failing_file = root.path().join("failing.ncl");
    let good_file = root.path().join("good.ncl");
    let second_good_file = root.path().join("good-two.ncl");
    let third_good_file = root.path().join("good-three.ncl");
    let missing_input_file = root.path().join("missing-input.ncl");
    let wrong_key_pool_config = root.path().join("wrong-key-pool.ncl");
    let pool_config = root.path().join("pool.ncl");
    let dead_pool_config = root.path().join("dead-pool.ncl");
    let dead_worker_pool_config = root.path().join("dead-worker-pool.ncl");
    let slow_worker_death_file = root.path().join("slow-worker-death.ncl");
    for dir in [&state_dir, &client_store, &worker_store, &worker_state] {
        fs::create_dir_all(dir).expect("pool failure directory");
    }
    write_ticket_state_full(&state_dir, MAX_UPLOAD_BYTES, 8, &[BUILDER_ID]);
    write_failing_capture_derivation(&failing_file);
    write_replayable_derivation(&good_file);
    write_named_replay_derivation(&second_good_file, "production-remote-replay-two");
    write_named_replay_derivation(&third_good_file, "production-remote-replay-three");
    write_missing_input_derivation(&missing_input_file);
    write_slow_derivation(&slow_worker_death_file);
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_mantle"));
    let wrapper = root.path().join("mantle-transport.sh");
    fs::write(&wrapper, format!("#!/bin/sh\nexec {} \"$@\"\n", binary.display())).expect("transport wrapper");
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).expect("wrapper mode");
    let binary_digest = blake3::hash(&fs::read(&wrapper).expect("wrapper bytes")).to_hex().to_string();
    // A transport that dies before the protocol starts models worker death.
    let dead_wrapper = root.path().join("dead-transport.sh");
    fs::write(&dead_wrapper, "#!/bin/sh\nexit 1\n").expect("dead transport wrapper");
    fs::set_permissions(&dead_wrapper, fs::Permissions::from_mode(0o755)).expect("dead wrapper mode");
    let dead_digest = blake3::hash(&fs::read(&dead_wrapper).expect("dead wrapper bytes")).to_hex().to_string();
    // A worker that dies mid-exchange: start it, then SIGKILL it after a second.
    let dead_worker_wrapper = root.path().join("dead-worker.sh");
    fs::write(
        &dead_worker_wrapper,
        format!(
            "#!/bin/sh\n{} \"$@\" &\nworker=$!\n( sleep 1; kill -9 \"$worker\" 2>/dev/null ) &\nwait \"$worker\"\n",
            binary.display()
        ),
    )
    .expect("dead worker wrapper");
    fs::set_permissions(&dead_worker_wrapper, fs::Permissions::from_mode(0o755)).expect("dead worker mode");
    let dead_worker_digest =
        blake3::hash(&fs::read(&dead_worker_wrapper).expect("dead worker bytes")).to_hex().to_string();
    let signing_material = RESULT_SIGNING_KEY.split_once(':').expect("signing key material").1;
    let binding_config = remote_binding_config_path(&state_dir);
    let worker_args = vec![
        "--state-dir".to_string(),
        state_dir.display().to_string(),
        "--store".to_string(),
        worker_store.display().to_string(),
        "--store-prefix".to_string(),
        STORE_PREFIX.to_string(),
        "--store-backend".to_string(),
        "snix".to_string(),
        "--remote-binding-config".to_string(),
        binding_config.display().to_string(),
        "remote".to_string(),
        "serve".to_string(),
        "--endpoint-id".to_string(),
        BUILDER_ID.to_string(),
        "--binding".to_string(),
        "stdio-once".to_string(),
        "--executor".to_string(),
        "local-build".to_string(),
        "--execution-state-dir".to_string(),
        worker_state.display().to_string(),
    ];
    let args_json = serde_json::to_string(&worker_args).expect("worker args JSON");
    let pool_source = format!(
        r#"{{
  pools = [{{
    pool_id = "native-pool",
    endpoints = [{{
      endpoint_id = "{builder}",
      transport = 'stdio,
      credential_reference = "pool-bearer",
      transport_program = "{binary}",
      transport_program_digest_blake3 = "{digest}",
      transport_args = {args},
      trusted_output_keys = [{{ public_key_name = "cache.example.com-1", public_key_base64 = "{material}" }}],
    }}],
    session = {{ max_concurrency = 1, job_count_max = 4, session_bytes_max = 1048576 }},
  }}],
}}
"#,
        builder = BUILDER_ID,
        binary = wrapper.display(),
        digest = binary_digest,
        args = args_json,
        material = signing_material,
    );
    fs::write(&pool_config, &pool_source).expect("pool configuration");
    let dead_pool_source = pool_source
        .replace(&format!("\"{binary}\"", binary = wrapper.display()), &format!("\"{}\"", dead_wrapper.display()))
        .replace(&binary_digest, &dead_digest);
    fs::write(&dead_pool_config, dead_pool_source).expect("dead pool configuration");
    let wrong_key_pool_source = pool_source.replace("cache.example.com-1", WRONG_BUILDER_KEY);
    fs::write(&wrong_key_pool_config, wrong_key_pool_source).expect("wrong-key pool configuration");
    let dead_worker_pool_source = pool_source
        .replace(
            &format!("\"{binary}\"", binary = wrapper.display()),
            &format!("\"{}\"", dead_worker_wrapper.display()),
        )
        .replace(&binary_digest, &dead_worker_digest);
    fs::write(&dead_worker_pool_config, dead_worker_pool_source).expect("dead-worker pool configuration");
    let run = |selected_pool: &Path, build_file: &Path, tamper_output: bool| -> (bool, Value) {
        let mut command = Command::new(&binary);
        configure_test_sandbox_command(&mut command);
        command.args([
            "--json",
            "--state-dir",
            state_dir.to_str().expect("UTF-8 state dir"),
            "--store",
            client_store.to_str().expect("UTF-8 client store"),
            "--store-prefix",
            STORE_PREFIX,
            "build",
            build_file.to_str().expect("UTF-8 build file"),
            "--no-substitute",
            "--builder-pool",
            "native-pool",
            "--builder-config",
            selected_pool.to_str().expect("UTF-8 pool config"),
            "--builder-ticket-fd",
            &format!("{BUILDER_ID}={REMOTE_TICKET_INPUT_FD}"),
            "--remote-build-time-secs",
            &MAX_BUILD_TIME_SECS.to_string(),
        ]);
        command.arg("--remote-binding-config").arg(&binding_config);
        command.env("CREDENTIALS_DIRECTORY", state_dir.join("credentials"));
        if tamper_output {
            command.env("MANTLE_TEST_TAMPER_POOL_OUTPUT_NAR", "1");
        }
        configure_ticket_input_fd(&mut command, &state_dir);
        let output = command.output().expect("run pool build");
        let report: Value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
        (output.status.success(), report)
    };
    let (dead_ok, dead_report) = run(&dead_pool_config, &good_file, false);
    assert!(!dead_ok, "a dead transport must fail the goal: {dead_report}");
    assert!(dead_report["failed"].as_array().is_some_and(|failed| !failed.is_empty()));
    let (failed_ok, failed_report) = run(&pool_config, &failing_file, false);
    assert!(!failed_ok, "the failing goal must fail: {failed_report}");
    assert!(failed_report["failed"].as_array().is_some_and(|failed| !failed.is_empty()));
    let published_before = fs::read_dir(&client_store).expect("client store entries").count();
    let (wrong_key_ok, wrong_key_report) = run(&wrong_key_pool_config, &third_good_file, false);
    assert!(!wrong_key_ok, "an untrusted output key must fail the goal: {wrong_key_report}");
    assert!(
        wrong_key_report.to_string().contains("untrusted-output-key"),
        "unexpected wrong-key failure: {wrong_key_report}"
    );
    assert_eq!(
        fs::read_dir(&client_store).expect("client store entries").count(),
        published_before,
        "a rejected output must not publish to the client store"
    );
    let tampered_file = root.path().join("tampered.ncl");
    write_named_replay_derivation(&tampered_file, "production-remote-tampered");
    let published_before_tamper = fs::read_dir(&client_store).expect("client store entries").count();
    let (tampered_ok, tampered_report) = run(&pool_config, &tampered_file, true);
    assert!(!tampered_ok, "a tampered output NAR must fail the goal: {tampered_report}");
    let tampered_failure = tampered_report.to_string();
    assert!(
        tampered_failure.contains("output-nar-digest-mismatch") || tampered_failure.contains("output-nar-ingest"),
        "unexpected tampered-output failure: {tampered_report}"
    );
    assert_eq!(
        fs::read_dir(&client_store).expect("client store entries").count(),
        published_before_tamper,
        "a tampered output must not publish"
    );
    let published_after_wrong_key = fs::read_dir(&client_store).expect("client store entries").count();
    let (missing_ok, missing_report) = run(&pool_config, &missing_input_file, false);
    assert!(!missing_ok, "a missing source input must fail the goal: {missing_report}");
    assert!(
        missing_report.to_string().contains("source input not found in store"),
        "unexpected missing-input failure: {missing_report}"
    );
    assert_eq!(
        fs::read_dir(&client_store).expect("client store entries").count(),
        published_after_wrong_key,
        "a goal with a missing input must not publish"
    );
    let published_before_worker_death = fs::read_dir(&client_store).expect("client store entries").count();
    let (worker_death_ok, worker_death_report) = run(&dead_worker_pool_config, &slow_worker_death_file, false);
    assert!(!worker_death_ok, "a mid-exchange worker death must fail the goal: {worker_death_report}");
    assert!(
        worker_death_report.to_string().contains("stdio remote child")
            || worker_death_report.to_string().contains("TransportSetup"),
        "unexpected worker-death failure: {worker_death_report}"
    );
    assert_eq!(
        fs::read_dir(&client_store).expect("client store entries").count(),
        published_before_worker_death,
        "a dead worker must not publish"
    );
    let (final_ok, final_report) = run(&pool_config, &second_good_file, false);
    assert!(final_ok, "a later goal must be admitted after attempt settlement: {final_report}");
    assert_eq!(final_report["outcomes"].as_array().map(Vec::len), Some(1));
}

/// Pool mode: a revoked durable binding fails selection before any dispatch.
#[test]
fn production_pool_rejects_revoked_endpoint_binding_before_dispatch() {
    let _guard = serial_transfer_guard();
    let root = tempfile::tempdir().expect("pool revoked tempdir");
    let state_dir = root.path().join("state");
    let client_store = root.path().join("client-store");
    let build_file = root.path().join("goal.ncl");
    let pool_config = root.path().join("pool.ncl");
    fs::create_dir_all(&state_dir).expect("state dir");
    fs::create_dir_all(&client_store).expect("client store");
    write_ticket_state_full(&state_dir, MAX_UPLOAD_BYTES, 4, &[BUILDER_ID]);
    remote_endpoint_binding::write_revoked_endpoint_binding(
        &state_dir,
        &remote_binding_config_path(&state_dir),
        STORE_PREFIX,
        BUILDER_ID,
    );
    write_replayable_derivation(&build_file);
    let pool_source = format!(
        r#"{{
  pools = [{{
    pool_id = "native-pool",
    endpoints = [{{
      endpoint_id = "{builder}",
      transport = 'stdio,
      credential_reference = "pool-bearer",
      transport_program = "/bin/true",
      transport_program_digest_blake3 = "{digest}",
      transport_args = [],
      trusted_output_keys = [{{ public_key_name = "cache.example.com-1", public_key_base64 = "AAAA" }}],
    }}],
    session = {{ max_concurrency = 1, job_count_max = 4, session_bytes_max = 1048576 }},
  }}],
}}
"#,
        builder = BUILDER_ID,
        digest = "0".repeat(64),
    );
    fs::write(&pool_config, pool_source).expect("pool configuration");
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_mantle"));
    let mut command = Command::new(&binary);
    configure_test_sandbox_command(&mut command);
    command.args([
        "--json",
        "--state-dir",
        state_dir.to_str().expect("UTF-8 state dir"),
        "--store",
        client_store.to_str().expect("UTF-8 client store"),
        "--store-prefix",
        STORE_PREFIX,
        "build",
        build_file.to_str().expect("UTF-8 build file"),
        "--no-substitute",
        "--builder-pool",
        "native-pool",
        "--builder-config",
        pool_config.to_str().expect("UTF-8 pool config"),
        "--builder-ticket-fd",
        &format!("{BUILDER_ID}={REMOTE_TICKET_INPUT_FD}"),
        "--remote-build-time-secs",
        &MAX_BUILD_TIME_SECS.to_string(),
    ]);
    command.arg("--remote-binding-config").arg(remote_binding_config_path(&state_dir));
    command.env("CREDENTIALS_DIRECTORY", state_dir.join("credentials"));
    configure_ticket_input_fd(&mut command, &state_dir);
    let output = command.output().expect("run revoked pool build");
    assert!(!output.status.success(), "a revoked binding must reject the pool");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("remote-pool"), "unexpected revocation failure: {stderr}");
    assert_eq!(fs::read_dir(&client_store).expect("client store entries").count(), 0);
}

/// Pool mode: SIGINT during an admitted local fallback stops the local goal
/// before it completes; the cancelled build publishes no output.
#[test]
fn production_pool_local_fallback_cancellation_stops_the_local_goal() {
    let _guard = serial_transfer_guard();
    let root = tempfile::tempdir().expect("fallback cancellation tempdir");
    let state_dir = root.path().join("state");
    let client_store = root.path().join("client-store");
    let slow_file = root.path().join("slow.ncl");
    let pool_config = root.path().join("pool.ncl");
    fs::create_dir_all(&state_dir).expect("state dir");
    fs::create_dir_all(&client_store).expect("client store");
    write_ticket_state_full(&state_dir, MAX_UPLOAD_BYTES, 4, &[BUILDER_ID]);
    write_slow_derivation(&slow_file);
    let program = PathBuf::from("/run/current-system/sw/bin/true");
    let digest = blake3::hash(&fs::read(&program).expect("program bytes")).to_hex().to_string();
    let pool_source = format!(
        r#"{{
  pools = [{{
    pool_id = "native-pool",
    fallback_policy = 'always,
    endpoints = [{{
      endpoint_id = "{builder}",
      transport = 'stdio,
      credential_reference = "pool-bearer",
      transport_program = "{}",
      transport_program_digest_blake3 = "{}",
      transport_args = [],
      profile = {{ system = "aarch64-linux" }},
      trusted_output_keys = [{{ public_key_name = "cache.example.com-1", public_key_base64 = "AAAA" }}],
    }}],
    session = {{ max_concurrency = 1, job_count_max = 4, session_bytes_max = 1048576 }},
  }}],
}}
"#,
        program.display(),
        digest,
        builder = BUILDER_ID,
    );
    fs::write(&pool_config, pool_source).expect("fallback pool configuration");
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_mantle"));
    let stdout_path = root.path().join("stdout.log");
    let stderr_path = root.path().join("stderr.log");
    let stdout_file = fs::File::create(&stdout_path).expect("stdout file");
    let stderr_file = fs::File::create(&stderr_path).expect("stderr file");
    let started = Instant::now();
    let mut child = pool_build_command(&binary, &state_dir, &client_store, &pool_config, &slow_file)
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .expect("spawn fallback build");
    // The fallback is admitted immediately and the local sandbox then runs the
    // 20-second builder; 2 seconds is inside that build on every observed run.
    std::thread::sleep(Duration::from_secs(2));
    let killed = unsafe { libc::kill(child.id() as i32, libc::SIGINT) };
    assert_eq!(killed, 0, "SIGINT must be delivered");
    let stop_deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        match child.try_wait().expect("poll cancelled fallback build") {
            Some(status) => break status,
            None if Instant::now() <= stop_deadline => std::thread::sleep(Duration::from_millis(50)),
            None => {
                let _ = child.kill();
                panic!("cancelled fallback build did not exit");
            }
        }
    };
    let elapsed = started.elapsed();
    assert!(!status.success(), "a cancelled fallback must not report success");
    assert!(
        elapsed < Duration::from_secs(15),
        "the local goal must stop before its 20-second builder ends: {elapsed:?}"
    );
    let published = fs::read_dir(&client_store).expect("client store entries").flatten().count();
    assert_eq!(published, 0, "a cancelled local goal must not publish an output");
}

/// Pool mode: SIGINT cancels the in-flight transport, the attempt is settled,
/// and a later goal is admitted on the same endpoint.
#[test]
fn production_pool_cancellation_settles_the_attempt() {
    let _guard = serial_transfer_guard();
    let root = tempfile::tempdir().expect("pool cancellation tempdir");
    let state_dir = root.path().join("state");
    let client_store = root.path().join("client-store");
    let worker_store = root.path().join("worker-store");
    let worker_state = root.path().join("worker-state");
    let slow_file = root.path().join("slow.ncl");
    let followup_file = root.path().join("followup.ncl");
    let pool_config = root.path().join("pool.ncl");
    for dir in [&state_dir, &client_store, &worker_store, &worker_state] {
        fs::create_dir_all(dir).expect("pool cancellation directory");
    }
    write_ticket_state_full(&state_dir, MAX_UPLOAD_BYTES, 8, &[BUILDER_ID]);
    write_slow_derivation(&slow_file);
    write_named_replay_derivation(&followup_file, "production-remote-cancel-followup");
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_mantle"));
    let wrapper = root.path().join("mantle-transport.sh");
    let worker_pid_file = root.path().join("worker-pid");
    fs::write(
        &wrapper,
        format!("#!/bin/sh\necho $$ > {} \nexec {} \"$@\"\n", worker_pid_file.display(), binary.display()),
    )
    .expect("transport wrapper");
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).expect("wrapper mode");
    let binary_digest = blake3::hash(&fs::read(&wrapper).expect("wrapper bytes")).to_hex().to_string();
    let signing_material = RESULT_SIGNING_KEY.split_once(':').expect("signing key material").1;
    let binding_config = remote_binding_config_path(&state_dir);
    let worker_args = vec![
        "--state-dir".to_string(),
        state_dir.display().to_string(),
        "--store".to_string(),
        worker_store.display().to_string(),
        "--store-prefix".to_string(),
        STORE_PREFIX.to_string(),
        "--store-backend".to_string(),
        "snix".to_string(),
        "--remote-binding-config".to_string(),
        binding_config.display().to_string(),
        "remote".to_string(),
        "serve".to_string(),
        "--endpoint-id".to_string(),
        BUILDER_ID.to_string(),
        "--binding".to_string(),
        "stdio-once".to_string(),
        "--executor".to_string(),
        "local-build".to_string(),
        "--execution-state-dir".to_string(),
        worker_state.display().to_string(),
    ];
    let args_json = serde_json::to_string(&worker_args).expect("worker args JSON");
    let pool_source = format!(
        r#"{{
  pools = [{{
    pool_id = "native-pool",
    endpoints = [{{
      endpoint_id = "{builder}",
      transport = 'stdio,
      credential_reference = "pool-bearer",
      transport_program = "{binary}",
      transport_program_digest_blake3 = "{digest}",
      transport_args = {args},
      trusted_output_keys = [{{ public_key_name = "cache.example.com-1", public_key_base64 = "{material}" }}],
    }}],
    session = {{ max_concurrency = 1, job_count_max = 4, session_bytes_max = 1048576 }},
  }}],
}}
"#,
        builder = BUILDER_ID,
        binary = wrapper.display(),
        digest = binary_digest,
        args = args_json,
        material = signing_material,
    );
    fs::write(&pool_config, pool_source).expect("pool configuration");
    let cancel_stderr = root.path().join("cancel-stderr.log");
    let cancel_stdout = root.path().join("cancel-stdout.log");
    let stderr_file = fs::File::create(&cancel_stderr).expect("cancel stderr file");
    let stdout_file = fs::File::create(&cancel_stdout).expect("cancel stdout file");
    let mut child = pool_build_command(&binary, &state_dir, &client_store, &pool_config, &slow_file)
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .expect("spawn cancelled pool build");
    let state_path = state_dir.join("remote-coordinator-state.json");
    let start_deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let running = fs::read_to_string(&state_path)
            .map(|state| state.contains("\"phase\": \"running\""))
            .unwrap_or(false);
        // The transport child writes its pid before exec; wait for it so the
        // signal lands on a live process instead of the admission window.
        if running && worker_pid_file.exists() {
            break;
        }
        if Instant::now() > start_deadline {
            let _ = child.kill();
            let stderr = fs::read_to_string(&cancel_stderr).unwrap_or_default();
            let stdout = fs::read_to_string(&cancel_stdout).unwrap_or_default();
            let state = fs::read_to_string(&state_path).unwrap_or_default();
            panic!("pool attempt never reached the running phase; stdout={stdout} stderr={stderr} state={state}");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let killed = unsafe { libc::kill(child.id() as i32, libc::SIGINT) };
    assert_eq!(killed, 0, "SIGINT must be delivered");
    let stop_deadline = Instant::now() + Duration::from_secs(90);
    let status = loop {
        match child.try_wait().expect("poll cancelled build") {
            Some(status) => break status,
            None if Instant::now() <= stop_deadline => std::thread::sleep(Duration::from_millis(50)),
            None => {
                let _ = child.kill();
                panic!("cancelled pool build did not exit");
            }
        }
    };
    assert!(!status.success(), "a cancelled build must not report success");
    let settled = fs::read_to_string(&state_path).expect("coordinator state after cancellation");
    assert!(!settled.contains("\"phase\": \"running\""), "the cancelled attempt must be settled: {settled}");
    assert!(settled.contains("\"phase\": \"failed\""), "the attempt must be failed: {settled}");
    let worker_pid: i32 = fs::read_to_string(&worker_pid_file)
        .expect("transport child pid recorded")
        .trim()
        .parse()
        .expect("transport child pid parses");
    let teardown_deadline = Instant::now() + Duration::from_secs(10);
    loop {
        // SAFETY: signal 0 only probes whether the recorded pid still exists.
        let alive = unsafe { libc::kill(worker_pid, 0) } == 0;
        if !alive {
            break;
        }
        if Instant::now() > teardown_deadline {
            panic!("transport child {worker_pid} is still alive after cancellation");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let output = pool_build_command(&binary, &state_dir, &client_store, &pool_config, &followup_file)
        .output()
        .expect("run follow-up pool build");
    assert!(
        output.status.success(),
        "a follow-up goal must run after cancellation: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn pool_build_command(
    binary: &Path,
    state_dir: &Path,
    client_store: &Path,
    pool_config: &Path,
    build_file: &Path,
) -> Command {
    let mut command = Command::new(binary);
    configure_test_sandbox_command(&mut command);
    command.args([
        "--json",
        "--state-dir",
        state_dir.to_str().expect("UTF-8 state dir"),
        "--store",
        client_store.to_str().expect("UTF-8 client store"),
        "--store-prefix",
        STORE_PREFIX,
        "build",
        build_file.to_str().expect("UTF-8 build file"),
        "--no-substitute",
        "--builder-pool",
        "native-pool",
        "--builder-config",
        pool_config.to_str().expect("UTF-8 pool config"),
        "--builder-ticket-fd",
        &format!("{BUILDER_ID}={REMOTE_TICKET_INPUT_FD}"),
        "--remote-build-time-secs",
        &MAX_BUILD_TIME_SECS.to_string(),
    ]);
    command.arg("--remote-binding-config").arg(remote_binding_config_path(state_dir));
    command.env("CREDENTIALS_DIRECTORY", state_dir.join("credentials"));
    configure_ticket_input_fd(&mut command, state_dir);
    command
}

const POOL_WORKER_A: &str = "pool-builder-a";
const POOL_WORKER_B: &str = "pool-builder-b";
const POOL_TICKET_USES: u32 = 8;
const POOL_TICKET_FD_A: i32 = 9;
const POOL_TICKET_FD_B: i32 = 8;

/// Pool mode: two admitted endpoints execute a real diamond through ordinary
/// Worker scheduling: one shared dependency, two branches, one join.
struct PoolDiamondOutcome {
    // The worker stores live under this root; the outcome must retain it for as
    // long as callers inspect the worker stores and reported outputs.
    _root: tempfile::TempDir,
    report: Value,
    worker_a_store: PathBuf,
    worker_b_store: PathBuf,
}

fn run_two_worker_pool_build(
    _guard: std::sync::MutexGuard<'static, ()>,
    write_build: impl FnOnce(&Path),
    lifecycle_log: Option<&Path>,
    max_jobs: Option<u32>,
    session_concurrency: u32,
) -> PoolDiamondOutcome {
    let root = tempfile::tempdir().expect("pool diamond tempdir");
    let state_dir = root.path().join("state");
    let client_store = root.path().join("client-store");
    let worker_a_store = root.path().join("worker-a-store");
    let worker_b_store = root.path().join("worker-b-store");
    let worker_a_state = root.path().join("worker-a-state");
    let worker_b_state = root.path().join("worker-b-state");
    let pool_config = root.path().join("pool.ncl");
    let build_file = root.path().join("pool-diamond.ncl");
    for dir in [
        &state_dir,
        &client_store,
        &worker_a_store,
        &worker_b_store,
        &worker_a_state,
        &worker_b_state,
    ] {
        fs::create_dir_all(dir).expect("pool diamond directory");
    }
    write_ticket_state_full(&state_dir, MAX_UPLOAD_BYTES, POOL_TICKET_USES, &[POOL_WORKER_A, POOL_WORKER_B]);
    write_build(&build_file);
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_mantle"));
    let wrapper = root.path().join("mantle-transport.sh");
    let wrapper_source = match lifecycle_log {
        Some(log) => format!(
            "#!/bin/sh\nprintf 'start %s %s\\n' \"$(date +%s%N)\" \"$$\" >> {}\n{} \"$@\"\nstatus=$?\nprintf 'end %s %s\\n' \"$(date +%s%N)\" \"$$\" >> {}\nexit $status\n",
            log.display(),
            binary.display(),
            log.display()
        ),
        None => format!("#!/bin/sh\nexec {} \"$@\"\n", binary.display()),
    };
    fs::write(&wrapper, wrapper_source).expect("transport wrapper");
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).expect("wrapper mode");
    let binary_digest = blake3::hash(&fs::read(&wrapper).expect("wrapper bytes")).to_hex().to_string();
    let signing_material = RESULT_SIGNING_KEY.split_once(':').expect("signing key material").1;
    let binding_config = remote_binding_config_path(&state_dir);
    let worker_args = |endpoint: &str, store: &Path, execution: &Path| -> Vec<String> {
        vec![
            "--state-dir".to_string(),
            state_dir.display().to_string(),
            "--store".to_string(),
            store.display().to_string(),
            "--store-prefix".to_string(),
            STORE_PREFIX.to_string(),
            "--store-backend".to_string(),
            "snix".to_string(),
            "--remote-binding-config".to_string(),
            binding_config.display().to_string(),
            "remote".to_string(),
            "serve".to_string(),
            "--endpoint-id".to_string(),
            endpoint.to_string(),
            "--binding".to_string(),
            "stdio-once".to_string(),
            "--executor".to_string(),
            "local-build".to_string(),
            "--execution-state-dir".to_string(),
            execution.display().to_string(),
        ]
    };
    let args_a = serde_json::to_string(&worker_args(POOL_WORKER_A, &worker_a_store, &worker_a_state))
        .expect("worker A args JSON");
    let args_b = serde_json::to_string(&worker_args(POOL_WORKER_B, &worker_b_store, &worker_b_state))
        .expect("worker B args JSON");
    let endpoint = |name: &str, credential: &str, args: &str| {
        format!(
            r#"    {{
      endpoint_id = "{name}",
      transport = 'stdio,
      credential_reference = "{credential}",
      transport_program = "{binary}",
      transport_program_digest_blake3 = "{digest}",
      transport_args = {args},
      profile = {{
        resource_inventory = {{
          total = {{ cpu_units = 4, memory_bytes = 8589934592, scratch_bytes = 10737418240 }},
        }},
      }},
      trusted_output_keys = [{{ public_key_name = "cache.example.com-1", public_key_base64 = "{material}" }}],
    }}"#,
            binary = wrapper.display(),
            digest = binary_digest,
            material = signing_material,
        )
    };
    let pool_source = format!(
        r#"{{
  pools = [{{
    pool_id = "native-pool",
    endpoints = [
{endpoint_a},
{endpoint_b},
    ],
    session = {{ max_concurrency = {session_concurrency}, job_count_max = 8, session_bytes_max = 1048576 }},
  }}],
}}
"#,
        endpoint_a = endpoint(POOL_WORKER_A, "pool-bearer-a", &args_a),
        endpoint_b = endpoint(POOL_WORKER_B, "pool-bearer-b", &args_b),
        session_concurrency = session_concurrency,
    );
    fs::write(&pool_config, pool_source).expect("pool configuration");

    let mut command = Command::new(&binary);
    configure_test_sandbox_command(&mut command);
    command.args([
        "--json",
        "--state-dir",
        state_dir.to_str().expect("UTF-8 state dir"),
        "--store",
        client_store.to_str().expect("UTF-8 client store"),
        "--store-prefix",
        STORE_PREFIX,
        "build",
        build_file.to_str().expect("UTF-8 build file"),
        "--no-substitute",
        "--builder-pool",
        "native-pool",
        "--builder-config",
        pool_config.to_str().expect("UTF-8 pool config"),
        "--builder-ticket-fd",
        &format!("{POOL_WORKER_A}={POOL_TICKET_FD_A}"),
        "--builder-ticket-fd",
        &format!("{POOL_WORKER_B}={POOL_TICKET_FD_B}"),
        "--remote-build-time-secs",
        &MAX_BUILD_TIME_SECS.to_string(),
    ]);
    if let Some(jobs) = max_jobs {
        command.args(["--jobs", &jobs.to_string()]);
    }
    command.arg("--remote-binding-config").arg(&binding_config);
    command.env("CREDENTIALS_DIRECTORY", state_dir.join("credentials"));
    configure_ticket_input_fds(&mut command, &state_dir, &[POOL_TICKET_FD_A, POOL_TICKET_FD_B]);
    let output = command.output().expect("run pool diamond build");
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        let report: Value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
        let log = report["failed"][0]["saved_log_path"]
            .as_str()
            .and_then(|path| fs::read_to_string(path).ok())
            .unwrap_or_default();
        panic!("pool diamond status={:?} stderr={stderr} failed={} log={log}", output.status, report["failed"]);
    }
    PoolDiamondOutcome {
        _root: root,
        report: serde_json::from_slice(&output.stdout).expect("pool diamond JSON report"),
        worker_a_store,
        worker_b_store,
    }
}

fn assert_pool_diamond(outcome: &PoolDiamondOutcome) {
    let report = &outcome.report;
    let outcomes = report["outcomes"].as_array().expect("outcomes array");
    assert_eq!(outcomes.len(), 1, "one requested root outcome: {report}");
    assert!(report["failed"].as_array().is_some_and(Vec::is_empty));
    let outputs = outcomes[0]["outputs"].as_array().expect("root outputs");
    assert_eq!(outputs.len(), 1, "one root output: {report}");
    let result_path = Path::new(outputs[0]["path"].as_str().expect("root output path"));
    let graph = fs::read_to_string(result_path.join("graph.txt")).expect("diamond graph output");
    assert!(graph.contains("shared foundation"), "graph carries the shared dependency: {graph}");
    assert!(graph.contains("left branch"), "graph carries the left branch: {graph}");
    assert!(graph.contains("right branch"), "graph carries the right branch: {graph}");
    for (label, store) in [
        ("worker A", &outcome.worker_a_store),
        ("worker B", &outcome.worker_b_store),
    ] {
        let entries = fs::read_dir(store).expect("worker store entries").count();
        assert!(entries > 0, "{label} executed at least one goal");
    }
}

#[test]
fn production_pool_dispatches_a_diamond_across_two_workers() {
    let outcome = run_two_worker_pool_build(
        serial_transfer_guard(),
        |path| write_pool_diamond_derivation(path, Some("input-addressed")),
        None,
        None,
        2,
    );
    assert_pool_diamond(&outcome);
}

#[test]
fn production_pool_diamond_worker_lifetimes_overlap() {
    let root = tempfile::tempdir().expect("pool overlap tempdir");
    let lifecycle_log = root.path().join("worker-lifecycle.log");
    let outcome = run_two_worker_pool_build(
        serial_transfer_guard(),
        |path| write_pool_diamond_derivation(path, Some("input-addressed")),
        Some(&lifecycle_log),
        None,
        2,
    );
    assert_pool_diamond(&outcome);
    let log = fs::read_to_string(&lifecycle_log).expect("worker lifecycle log");
    let starts = worker_lifetimes(&log);
    assert_eq!(starts.len(), 4, "one transport lifetime per diamond goal: {log}");
    assert!(starts[2].0 < starts[1].1, "the two branch workers must overlap: {starts:?}");
}

fn worker_lifetimes(log: &str) -> Vec<(u128, u128)> {
    let mut starts: Vec<(u128, u128)> = Vec::new();
    let mut open = std::collections::BTreeMap::<u128, u128>::new();
    for line in log.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(fields.len(), 3, "unexpected lifecycle line: {line}");
        let timestamp: u128 = fields[1].parse().expect("nanosecond timestamp");
        let pid: u128 = fields[2].parse().expect("worker pid");
        match fields[0] {
            "start" => {
                open.insert(pid, timestamp);
            }
            "end" => {
                let start = open.remove(&pid).expect("end without start");
                starts.push((start, timestamp));
            }
            other => panic!("unexpected lifecycle kind: {other}"),
        }
    }
    assert!(open.is_empty(), "worker lifecycle log has unfinished entries");
    starts.sort_unstable();
    starts
}

#[test]
fn production_pool_respects_the_global_job_limit() {
    let root = tempfile::tempdir().expect("pool jobs tempdir");
    let lifecycle_log = root.path().join("worker-lifecycle.log");
    let outcome = run_two_worker_pool_build(
        serial_transfer_guard(),
        |path| write_pool_diamond_derivation(path, Some("input-addressed")),
        Some(&lifecycle_log),
        Some(1),
        2,
    );
    let report = &outcome.report;
    let outcomes = report["outcomes"].as_array().expect("outcomes array");
    assert_eq!(outcomes.len(), 1, "one requested root outcome: {report}");
    assert!(report["failed"].as_array().is_some_and(Vec::is_empty));
    let outputs = outcomes[0]["outputs"].as_array().expect("root outputs");
    let graph = fs::read_to_string(Path::new(outputs[0]["path"].as_str().expect("root path")).join("graph.txt"))
        .expect("diamond graph output");
    assert!(graph.contains("left branch") && graph.contains("right branch"), "{graph}");
    let log = fs::read_to_string(&lifecycle_log).expect("worker lifecycle log");
    let starts = worker_lifetimes(&log);
    assert_eq!(starts.len(), 4, "one transport lifetime per diamond goal: {log}");
    for pair in starts.windows(2) {
        assert!(pair[1].0 >= pair[0].1, "--jobs 1 must serialize goal execution: {starts:?}");
    }
}

#[test]
fn production_pool_respects_the_session_concurrency_limit() {
    let root = tempfile::tempdir().expect("pool session tempdir");
    let lifecycle_log = root.path().join("worker-lifecycle.log");
    let outcome = run_two_worker_pool_build(
        serial_transfer_guard(),
        |path| write_pool_diamond_derivation(path, Some("input-addressed")),
        Some(&lifecycle_log),
        None,
        1,
    );
    let report = &outcome.report;
    let outcomes = report["outcomes"].as_array().expect("outcomes array");
    assert_eq!(outcomes.len(), 1, "one requested root outcome: {report}");
    assert!(report["failed"].as_array().is_some_and(Vec::is_empty));
    let log = fs::read_to_string(&lifecycle_log).expect("worker lifecycle log");
    let starts = worker_lifetimes(&log);
    assert_eq!(starts.len(), 4, "one transport lifetime per diamond goal: {log}");
    for pair in starts.windows(2) {
        assert!(pair[1].0 >= pair[0].1, "session max_concurrency 1 must serialize goal execution: {starts:?}");
    }
}

#[test]
fn production_pool_dispatches_a_content_addressed_diamond_across_two_workers() {
    let outcome = run_two_worker_pool_build(
        serial_transfer_guard(),
        |path| write_pool_diamond_derivation(path, None),
        None,
        None,
        2,
    );
    assert_pool_diamond(&outcome);
}

#[test]
fn production_stdio_resumes_missing_chunks_and_imports_output() {
    let _guard = serial_transfer_guard();
    if !cfg!(debug_assertions) {
        eprintln!("SKIP: the deterministic transfer-interruption seam is disabled in release binaries");
        return;
    }
    let root = tempfile::tempdir().expect("production transfer tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    let output_source = root.path().join("output-source.bin");
    let build_file = root.path().join("remote-production.ncl");
    let local_route_sentinel = root.path().join("client-local-route-entered");
    fs::create_dir_all(&state_dir).expect("state dir");
    fs::create_dir_all(&store_dir).expect("store dir");
    fs::write(&output_source, patterned_bytes(OUTPUT_BYTES)).expect("output source");
    let input_host_path = physical_store_path(&store_dir, INPUT_STORE_PATH);
    fs::write(&input_host_path, patterned_bytes(INPUT_BYTES)).expect("input source");
    write_ticket_state(&state_dir);
    write_fetch_derivation(&build_file, &output_source);

    let interrupted = remote_build_command(&state_dir, &store_dir, &build_file)
        .env("MANTLE_TEST_LOCAL_ROUTE_SENTINEL", &local_route_sentinel)
        .env(INTERRUPT_ENV, INTERRUPT_AFTER_CHUNKS)
        .output()
        .expect("run interrupted production transfer");
    assert!(!interrupted.status.success(), "first run must interrupt after a durable chunk");
    assert!(!local_route_sentinel.exists(), "remote preflight must not enter the client local route");
    let interrupted_stderr = String::from_utf8_lossy(&interrupted.stderr);
    assert!(
        interrupted_stderr.contains("remote-production-transfer-interrupted-after-checkpoint"),
        "unexpected interruption stderr: {interrupted_stderr}"
    );
    assert!(state_diagnostics_contain(&state_dir, "transfer-cutoff"));
    assert!(!state_diagnostics_contain(&state_dir, "execution-failed"));
    assert!(!state_diagnostics_contain(&state_dir, "output-admitted"));
    let failure_bundles = failure_debug_bundle_dirs(&state_dir);
    assert_eq!(failure_bundles.len(), EXPECTED_FAILURE_DEBUG_BUNDLES);
    let failure_bundle = &failure_bundles[0];
    let manifest: Value = serde_json::from_slice(&fs::read(failure_bundle.join("manifest.json")).unwrap()).unwrap();
    let bundle_digest = manifest["bundle_blake3"].as_str().expect("bundle digest");
    assert_eq!(manifest["failure_phase"], "input-transfer");
    assert_eq!(manifest["capture_outcome_code"], "metadata-only");
    assert_eq!(manifest["cleanup_status_code"], "cleanup-not-observed");
    assert!(manifest["immutable_log"].is_object());
    let bundle_before_inspect = snapshot_file_tree(failure_bundle);
    let inspect = Command::new(env!("CARGO_BIN_EXE_mantle"))
        .args(["--json", "--state-dir"])
        .arg(&state_dir)
        .args(["remote", "debug", "inspect", bundle_digest])
        .output()
        .expect("inspect failure debug bundle in clean process");
    assert!(inspect.status.success(), "inspect stderr={}", String::from_utf8_lossy(&inspect.stderr));
    let inspect_json: Value = serde_json::from_slice(&inspect.stdout).expect("inspect JSON");
    assert_eq!(inspect_json["bundle_blake3"], bundle_digest);
    assert_eq!(inspect_json["captured_artifact_count"], 0);
    let replay_plan = Command::new(env!("CARGO_BIN_EXE_mantle"))
        .args(["--json", "--state-dir"])
        .arg(&state_dir)
        .args(["remote", "debug", "replay-plan", bundle_digest])
        .output()
        .expect("plan failure debug replay in clean process");
    assert!(replay_plan.status.success(), "replay-plan stderr={}", String::from_utf8_lossy(&replay_plan.stderr));
    let replay_json: Value = serde_json::from_slice(&replay_plan.stdout).expect("replay plan JSON");
    assert_eq!(replay_json["executable"], false);
    assert!(replay_json["blockers"].as_array().unwrap().iter().any(|blocker| blocker == "replay-policy-denied"));
    assert_eq!(snapshot_file_tree(failure_bundle), bundle_before_inspect);
    let status = Command::new(env!("CARGO_BIN_EXE_mantle"))
        .args(["--json", "--state-dir"])
        .arg(&state_dir)
        .args(["remote", "status"])
        .output()
        .expect("render remote failure status");
    assert!(status.status.success(), "status stderr={}", String::from_utf8_lossy(&status.stderr));
    let status_json: Value = serde_json::from_slice(&status.stdout).expect("remote status JSON");
    let failure_debug = &status_json["active_jobs"][0]["failure_debug"];
    assert_eq!(failure_debug["bundle_ref"], format!("remote-failure-debug:{bundle_digest}"));
    assert_eq!(failure_debug["capture_outcome_code"], "metadata-only");
    let status_text = String::from_utf8(status.stdout).unwrap();
    assert!(!status_text.contains(TICKET_SECRET));
    assert!(!status_text.contains(root.path().to_str().unwrap()));
    let first_run_checkpoints = transfer_checkpoints(&state_dir);
    let completed_upload = first_run_checkpoints
        .iter()
        .filter(|summary| summary.acknowledged_chunks > 1)
        .max_by_key(|summary| summary.transferred_bytes)
        .expect("completed multi-chunk production input upload checkpoint");
    assert!(completed_upload.transferred_bytes > u64::try_from(INPUT_BYTES).unwrap());
    let partial = partial_output_checkpoint(&state_dir);
    assert_eq!(partial.acknowledged_chunks, 1, "one output chunk must be durable before restart");
    assert!(partial.transferred_bytes > 0, "partial checkpoint records transferred bytes");

    let resumed = remote_build_command(&state_dir, &store_dir, &build_file)
        .env("MANTLE_TEST_LOCAL_ROUTE_SENTINEL", &local_route_sentinel)
        .output()
        .expect("run resumed production transfer");
    assert!(resumed.status.success(), "resume stderr={}", String::from_utf8_lossy(&resumed.stderr));
    assert!(!local_route_sentinel.exists(), "remote success must not enter the client local route");
    let report: Value = serde_json::from_slice(&resumed.stdout).expect("production build JSON report");
    let substitution = &report["outcomes"][0]["outputs"][0]["substitution"];
    assert_eq!(substitution["mode"], "streaming");
    assert_production_lifecycle_sequence(&report, true);
    assert!(substitution["reused_bytes"].as_u64().unwrap_or(0) >= partial.transferred_bytes);
    let output_path =
        PathBuf::from(report["outcomes"][0]["outputs"][0]["path"].as_str().expect("imported output path"));
    assert_eq!(fs::read(output_path).expect("imported output bytes"), patterned_bytes(OUTPUT_BYTES));
    let completed = output_checkpoint_with_most_acknowledgements(&state_dir);
    assert!(completed.acknowledged_chunks > partial.acknowledged_chunks);
    assert!(completed.transferred_bytes > partial.transferred_bytes);
}

#[test]
fn production_stdio_resumes_interrupted_multi_chunk_input_upload() {
    let _guard = serial_transfer_guard();
    if !cfg!(debug_assertions) {
        eprintln!("SKIP: the deterministic transfer-interruption seam is disabled in release binaries");
        return;
    }
    let root = tempfile::tempdir().expect("production input resume tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    let output_source = root.path().join("output-source.bin");
    let build_file = root.path().join("remote-production.ncl");
    fs::create_dir_all(&state_dir).expect("state dir");
    fs::create_dir_all(&store_dir).expect("store dir");
    fs::write(&output_source, patterned_bytes(OUTPUT_BYTES)).expect("output source");
    fs::write(physical_store_path(&store_dir, INPUT_STORE_PATH), patterned_bytes(INPUT_BYTES)).expect("input source");
    write_ticket_state(&state_dir);
    write_fetch_derivation(&build_file, &output_source);

    let interrupted = remote_build_command(&state_dir, &store_dir, &build_file)
        .env(INPUT_INTERRUPT_ENV, INTERRUPT_AFTER_CHUNKS)
        .output()
        .expect("run interrupted production input transfer");
    assert!(!interrupted.status.success());
    let interrupted_stderr = String::from_utf8_lossy(&interrupted.stderr);
    assert!(
        interrupted_stderr.contains("remote-production-input-transfer-interrupted-after-checkpoint"),
        "unexpected input interruption stderr: {interrupted_stderr}"
    );
    let partial = partial_output_checkpoint(&state_dir);
    assert_eq!(partial.acknowledged_chunks, 1);
    assert!(partial.transferred_bytes > 0);

    let resumed = remote_build_command(&state_dir, &store_dir, &build_file)
        .output()
        .expect("resume production input transfer");
    assert!(resumed.status.success(), "resume stderr={}", String::from_utf8_lossy(&resumed.stderr));
    let completed = read_checkpoint_summary(&partial.path);
    assert!(completed.acknowledged_chunks > partial.acknowledged_chunks);
    assert!(completed.transferred_bytes > u64::try_from(INPUT_BYTES).unwrap());
    let report: Value = serde_json::from_slice(&resumed.stdout).expect("resumed production JSON report");
    let imported_path =
        PathBuf::from(report["outcomes"][0]["outputs"][0]["path"].as_str().expect("imported output path"));
    assert_eq!(fs::read(imported_path).unwrap(), patterned_bytes(OUTPUT_BYTES));
}

#[test]
fn production_stdio_rejects_ticket_upload_quota_before_checkpoint_or_admission() {
    let _guard = serial_transfer_guard();
    let root = tempfile::tempdir().expect("production quota tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    let output_source = root.path().join("output-source.bin");
    let build_file = root.path().join("remote-production.ncl");
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    fs::write(&output_source, patterned_bytes(OUTPUT_BYTES)).unwrap();
    fs::write(physical_store_path(&store_dir, INPUT_STORE_PATH), patterned_bytes(INPUT_BYTES)).unwrap();
    write_ticket_state_with_upload_limit(&state_dir, REJECTING_UPLOAD_BYTES);
    write_fetch_derivation(&build_file, &output_source);

    let rejected = remote_build_command(&state_dir, &store_dir, &build_file)
        .output()
        .expect("run quota-rejected production transfer");
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("upload-byte-limit-exceeded"));
    assert!(transfer_checkpoints(&state_dir).is_empty());
    assert_eq!(fs::read_dir(&store_dir).unwrap().count(), 1);
    assert_eq!(fs::read(physical_store_path(&store_dir, INPUT_STORE_PATH)).unwrap(), patterned_bytes(INPUT_BYTES));
}

#[test]
fn production_stdio_delta_unavailable_falls_back_to_bounded_full_nar_and_admits() {
    let _guard = serial_transfer_guard();
    let root = tempfile::tempdir().expect("production delta fallback tempdir");
    let fixture = setup_production_fixture(root.path(), OUTPUT_BYTES, MAX_UPLOAD_BYTES);
    let completed = remote_build_command(&fixture.state_dir, &fixture.store_dir, &fixture.build_file)
        .arg("--remote-delta")
        .output()
        .expect("run production delta-unavailable fallback");
    assert!(completed.status.success(), "fallback stderr={}", String::from_utf8_lossy(&completed.stderr));
    let report: Value = serde_json::from_slice(&completed.stdout).expect("fallback production JSON report");
    let substitution = &report["outcomes"][0]["outputs"][0]["substitution"];
    assert_eq!(substitution["mode"], "full");
    assert_eq!(substitution["fallback_reason"], "delta-unavailable");
    let imported_path =
        PathBuf::from(report["outcomes"][0]["outputs"][0]["path"].as_str().expect("fallback imported output path"));
    assert_eq!(fs::read(imported_path).unwrap(), patterned_bytes(OUTPUT_BYTES));
    let output_checkpoint = output_checkpoint_with_most_acknowledgements(&fixture.state_dir);
    assert!(output_checkpoint.acknowledged_chunks > 1);
    assert!(output_checkpoint.transferred_bytes > u64::try_from(OUTPUT_BYTES).unwrap());
}

#[test]
fn production_stdio_streams_and_admits_8_mib_output() {
    let _guard = serial_transfer_guard();
    let root = tempfile::tempdir().expect("production 8 MiB tempdir");
    let fixture = setup_production_fixture(root.path(), PRODUCTION_SCALE_OUTPUT_BYTES, MAX_UPLOAD_BYTES);
    let completed = remote_build_command(&fixture.state_dir, &fixture.store_dir, &fixture.build_file)
        .output()
        .expect("run production 8 MiB transfer");
    assert!(completed.status.success(), "8 MiB stderr={}", String::from_utf8_lossy(&completed.stderr));
    let report: Value = serde_json::from_slice(&completed.stdout).expect("8 MiB production JSON report");
    let substitution = &report["outcomes"][0]["outputs"][0]["substitution"];
    assert_eq!(substitution["mode"], "streaming");
    assert_eq!(substitution["fallback_reason"], Value::Null);
    let imported_path =
        PathBuf::from(report["outcomes"][0]["outputs"][0]["path"].as_str().expect("8 MiB imported output path"));
    assert_eq!(fs::read(imported_path).unwrap(), patterned_bytes(PRODUCTION_SCALE_OUTPUT_BYTES));
    let output_checkpoint = output_checkpoint_with_most_acknowledgements(&fixture.state_dir);
    assert!(output_checkpoint.acknowledged_chunks > MIN_PRODUCTION_SCALE_OUTPUT_CHUNKS);
    assert!(output_checkpoint.transferred_bytes > u64::try_from(PRODUCTION_SCALE_OUTPUT_BYTES).unwrap());
}

#[test]
fn gallery_resumable_remote_transfer_resumes_verified_chunks_and_admits_once() {
    let _guard = serial_transfer_guard();
    if !cfg!(debug_assertions) {
        eprintln!("SKIP: the deterministic transfer-interruption seam is disabled in release binaries");
        return;
    }
    let root = tempfile::tempdir().expect("gallery resumable transfer tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    fs::create_dir_all(&state_dir).expect("gallery state dir");
    fs::create_dir_all(&store_dir).expect("gallery store dir");
    write_ticket_state(&state_dir);

    let interrupted = remote_gallery_build_command(&state_dir, &store_dir)
        .env(INTERRUPT_ENV, INTERRUPT_AFTER_CHUNKS)
        .output()
        .expect("interrupt gallery production transfer");
    assert!(!interrupted.status.success(), "first gallery run must interrupt after a durable chunk");
    assert!(
        String::from_utf8_lossy(&interrupted.stderr)
            .contains("remote-production-transfer-interrupted-after-checkpoint"),
        "gallery interruption stderr={}",
        String::from_utf8_lossy(&interrupted.stderr),
    );
    assert_eq!(fs::read_dir(&store_dir).unwrap().count(), 0);
    assert!(!state_diagnostics_contain(&state_dir, "output-admitted"));
    let partial = partial_output_checkpoint(&state_dir);
    assert_eq!(partial.acknowledged_chunks, 1);
    assert!(partial.transferred_bytes > 0);
    assert_eq!(partial.manifest_digest_blake3.len(), BLAKE3_HEX_CHARS);

    let resumed = remote_gallery_build_command(&state_dir, &store_dir)
        .output()
        .expect("resume gallery production transfer");
    assert!(resumed.status.success(), "gallery resume stderr={}", String::from_utf8_lossy(&resumed.stderr));
    let report: Value = serde_json::from_slice(&resumed.stdout).expect("gallery production JSON report");
    let substitution = &report["outcomes"][0]["outputs"][0]["substitution"];
    assert_eq!(substitution["mode"], "streaming");
    assert!(substitution["reused_bytes"].as_u64().unwrap_or(0) >= partial.transferred_bytes);
    assert_production_lifecycle_sequence(&report, true);

    let output_path = PathBuf::from(report["outcomes"][0]["outputs"][0]["path"].as_str().expect("gallery output path"));
    assert_eq!(
        fs::read_to_string(output_path.join("payload.txt")).unwrap(),
        "resumable remote transfer gallery payload\n"
    );
    let payload = fs::read(output_path.join("payload.bin")).expect("gallery payload bytes");
    assert_eq!(payload.len(), GALLERY_PAYLOAD_BYTES);
    assert_eq!(blake3::hash(&payload), blake3::hash(&vec![0_u8; GALLERY_PAYLOAD_BYTES]));
    assert_eq!(fs::read_dir(&store_dir).unwrap().count(), 1);

    let completed = read_checkpoint_summary(&partial.path);
    assert_eq!(completed.session_id, partial.session_id);
    assert_eq!(completed.manifest_digest_blake3, partial.manifest_digest_blake3);
    assert!(completed.acknowledged_chunks > partial.acknowledged_chunks);
    assert!(completed.transferred_bytes > partial.transferred_bytes);
}

#[test]
fn gallery_resumable_remote_transfer_rejects_tampered_acknowledged_content() {
    let _guard = serial_transfer_guard();
    if !cfg!(debug_assertions) {
        eprintln!("SKIP: the deterministic transfer-interruption seam is disabled in release binaries");
        return;
    }
    let root = tempfile::tempdir().expect("gallery tampered transfer tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    fs::create_dir_all(&state_dir).expect("gallery state dir");
    fs::create_dir_all(&store_dir).expect("gallery store dir");
    write_ticket_state(&state_dir);

    let interrupted = remote_gallery_build_command(&state_dir, &store_dir)
        .env(INTERRUPT_ENV, INTERRUPT_AFTER_CHUNKS)
        .output()
        .expect("interrupt gallery transfer before tamper");
    assert!(!interrupted.status.success());
    let partial = partial_output_checkpoint(&state_dir);
    let acknowledged_digest = partial.acknowledged_chunk_digests.first().expect("acknowledged chunk digest");
    let receiver_chunk = state_dir
        .join(REMOTE_TRANSFER_RECEIVER_DIR)
        .join(&partial.session_id)
        .join(REMOTE_TRANSFER_CHUNK_DIR)
        .join(acknowledged_digest);
    assert!(receiver_chunk.is_file());
    fs::write(&receiver_chunk, b"tampered acknowledged gallery chunk").expect("tamper acknowledged chunk");

    let rejected = remote_gallery_build_command(&state_dir, &store_dir)
        .output()
        .expect("retry tampered gallery transfer");
    assert!(!rejected.status.success(), "tampered acknowledged content must block resume");
    assert!(String::from_utf8_lossy(&rejected.stderr).contains(ACKNOWLEDGED_CHUNK_MISSING_REASON));
    assert_eq!(fs::read_dir(&store_dir).unwrap().count(), 0);
    assert!(!state_diagnostics_contain(&state_dir, "output-admitted"));
}

#[test]
fn failed_remote_sandbox_captures_allowlisted_artifact_before_cleanup_without_changing_failure_truth() {
    let _guard = serial_transfer_guard();
    // The capture worker runs a real bubblewrap sandbox build; a Nix build
    // sandbox forbids the nested namespace setup, so skip there. `nix develop`
    // also sets NIX_BUILD_TOP, but only a development shell sets IN_NIX_SHELL.
    if std::env::var_os("NIX_BUILD_TOP").is_some() && std::env::var_os("IN_NIX_SHELL").is_none() {
        eprintln!("SKIP: nested sandbox builds are unavailable inside a Nix build sandbox");
        return;
    }
    let root = tempfile::Builder::new()
        .prefix("failure-debug-capture")
        .tempdir()
        .expect("failure debug capture tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    let build_file = root.path().join("remote-failure.ncl");
    let config = root.path().join("failure-debug.ncl");
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    write_ticket_state(&state_dir);
    write_failing_capture_derivation(&build_file);
    write_capture_debug_config(&config);

    let failed = remote_build_command(&state_dir, &store_dir, &build_file)
        .arg("--remote-observability-config")
        .arg(&config)
        .env("CRUNCH_NO_FUSE", "1")
        .output()
        .expect("run captured remote sandbox failure");
    assert!(!failed.status.success());
    let stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(stderr.contains("remote-failure-debug:"), "missing worker bundle ref: {stderr}");
    assert!(stderr.contains("remote-worker-execution-failed"), "missing original execution failure: {stderr}");
    let bundles = recursive_failure_debug_bundle_dirs(&state_dir);
    assert_eq!(bundles.len(), EXPECTED_WORKER_AND_COORDINATOR_BUNDLES);
    let capture_outcomes = bundles
        .iter()
        .map(|bundle| {
            let manifest: Value = serde_json::from_slice(&fs::read(bundle.join("manifest.json")).unwrap()).unwrap();
            manifest["capture_outcome_code"].as_str().unwrap().to_string()
        })
        .collect::<Vec<_>>();
    let captured_bundle = bundles
        .iter()
        .find(|bundle| {
            let manifest: Value = serde_json::from_slice(&fs::read(bundle.join("manifest.json")).unwrap()).unwrap();
            manifest["capture_outcome_code"] == "captured"
        })
        .unwrap_or_else(|| panic!("worker capture bundle missing; outcomes={capture_outcomes:?}; stderr={stderr}"));
    let manifest: Value = serde_json::from_slice(&fs::read(captured_bundle.join("manifest.json")).unwrap()).unwrap();
    let worker_bundle_digest = manifest["bundle_blake3"].as_str().unwrap();
    assert_eq!(worker_bundle_digest.len(), BLAKE3_HEX_CHARS);
    assert_eq!(manifest["failure_phase"], "execution");
    assert_eq!(manifest["cleanup_status_code"], "cleanup-attempted-after-capture");
    let capture_manifest_digest = manifest["captured_artifact_manifest_ref"]["digest_blake3"].as_str().unwrap();
    let capture_manifest_path = captured_bundle.join("objects").join(format!("{capture_manifest_digest}.json"));
    let capture_manifest: Value = serde_json::from_slice(&fs::read(capture_manifest_path).unwrap()).unwrap();
    let artifacts = capture_manifest["payload"]["artifacts"].as_array().unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0]["relative_path"], "build/trace.json");
    let object_digest = artifacts[0]["object_blake3"].as_str().unwrap();
    let captured_bytes = fs::read(captured_bundle.join("capture-cas").join(format!("{object_digest}.bin"))).unwrap();
    assert_eq!(captured_bytes, CAPTURED_TRACE.as_bytes());
    let all_bundle_bytes = snapshot_file_tree(captured_bundle)
        .keys()
        .flat_map(|relative| fs::read(captured_bundle.join(relative)).unwrap())
        .collect::<Vec<_>>();
    assert!(!String::from_utf8_lossy(&all_bundle_bytes).contains(DISALLOWED_TRACE));
    let inspect = Command::new(env!("CARGO_BIN_EXE_mantle"))
        .args(["--json", "--state-dir"])
        .arg(&state_dir)
        .args(["remote", "debug", "inspect", worker_bundle_digest])
        .output()
        .expect("inspect portable worker capture bundle after cleanup");
    assert!(inspect.status.success(), "worker inspect stderr={}", String::from_utf8_lossy(&inspect.stderr));
    let inspect_json: Value = serde_json::from_slice(&inspect.stdout).unwrap();
    assert_eq!(inspect_json["captured_artifact_count"], 1);
    assert!(!String::from_utf8(inspect.stdout).unwrap().contains(root.path().to_str().unwrap()));
    let status = Command::new(env!("CARGO_BIN_EXE_mantle"))
        .args(["--json", "--state-dir"])
        .arg(&state_dir)
        .args(["remote", "status"])
        .output()
        .expect("render worker capture status");
    assert!(status.status.success());
    let status_json: Value = serde_json::from_slice(&status.stdout).unwrap();
    let debug_status = &status_json["recent_failures"][0]["failure_debug"];
    assert_eq!(debug_status["capture_outcome_code"], "captured");
    assert_eq!(debug_status["worker_bundle_ref"], format!("remote-failure-debug:{worker_bundle_digest}"));
    assert!(failure_workspace_directories(&state_dir).is_empty());
    assert!(fs::read_dir(&store_dir).unwrap().next().is_none());
}

#[test]
fn oversized_remote_sandbox_artifact_is_rejected_without_rewriting_execution_failure() {
    let _guard = serial_transfer_guard();
    let root = tempfile::Builder::new()
        .prefix("failure-debug-oversized")
        .tempdir()
        .expect("oversized failure debug tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    let build_file = root.path().join("remote-failure.ncl");
    let config = root.path().join("failure-debug.ncl");
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    write_ticket_state(&state_dir);
    write_failing_capture_derivation(&build_file);
    write_capture_debug_config_with_file_bytes(&config, REJECTING_CAPTURE_FILE_BYTES);

    let failed = remote_build_command(&state_dir, &store_dir, &build_file)
        .arg("--remote-observability-config")
        .arg(&config)
        .env("CRUNCH_NO_FUSE", "1")
        .output()
        .expect("run oversized captured remote sandbox failure");
    assert!(!failed.status.success());
    let stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(stderr.contains("remote-worker-execution-failed"), "stderr={stderr}");
    assert!(stderr.contains("remote-failure-debug:"), "stderr={stderr}");
    let bundles = recursive_failure_debug_bundle_dirs(&state_dir);
    assert_eq!(bundles.len(), EXPECTED_WORKER_AND_COORDINATOR_BUNDLES);
    let rejected_bundle = bundles
        .iter()
        .find(|bundle| {
            let manifest: Value = serde_json::from_slice(&fs::read(bundle.join("manifest.json")).unwrap()).unwrap();
            manifest["capture_outcome_code"] == "capture-rejected"
        })
        .expect("oversized capture rejection bundle");
    assert!(!rejected_bundle.join("capture-cas").read_dir().unwrap().any(|entry| entry.is_ok()));
    assert!(failure_workspace_directories(&state_dir).is_empty());
    assert!(fs::read_dir(&store_dir).unwrap().next().is_none());
}

#[test]
fn failed_output_admission_replays_under_new_fence_without_rewriting_original_bundle() {
    let _guard = serial_transfer_guard();
    let root = tempfile::Builder::new()
        .prefix("failure-debug-replay")
        .tempdir()
        .expect("failure debug replay tempdir");
    let fixture = setup_replay_fixture(root.path());
    let config = root.path().join("failure-debug.ncl");
    write_failure_debug_config(&config, true);

    let failed = remote_build_command(&fixture.state_dir, &fixture.store_dir, &fixture.build_file)
        .arg("--remote-observability-config")
        .arg(&config)
        .arg("--trusted-builder-key")
        .arg(WRONG_BUILDER_KEY)
        .env("CRUNCH_NO_FUSE", "1")
        .output()
        .expect("run output-admission failure");
    assert!(!failed.status.success());
    let failed_stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(failed_stderr.contains("untrusted-output-key"), "unexpected failure stderr: {failed_stderr}");
    let bundles = failure_debug_bundle_dirs(&fixture.state_dir);
    assert_eq!(bundles.len(), EXPECTED_FAILURE_DEBUG_BUNDLES);
    let bundle_dir = &bundles[0];
    let manifest_bytes = fs::read(bundle_dir.join("manifest.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    let bundle_digest = manifest["bundle_blake3"].as_str().unwrap();
    let original_job_id = manifest["original_job_id"].as_str().unwrap();
    let original_attempt_id = manifest["original_attempt_id"].as_str().unwrap();
    let original_fence = manifest["original_fence_generation"].as_u64().unwrap();
    assert_eq!(manifest["failure_phase"], "execution");

    let rejected_replay = remote_failure_replay_command(&fixture.state_dir, &fixture.store_dir, bundle_digest)
        .arg("--trusted-builder-key")
        .arg(WRONG_BUILDER_KEY)
        .output()
        .expect("execute rejected remote failure replay");
    assert!(!rejected_replay.status.success());
    let rejected_replay_stderr = String::from_utf8_lossy(&rejected_replay.stderr);
    assert!(
        rejected_replay_stderr.contains("untrusted-output-key"),
        "rejected replay stderr={rejected_replay_stderr}"
    );
    let coordinator: Value = serde_json::from_slice(
        &fs::read(fixture.state_dir.join("remote-coordinator-state.json")).expect("replay coordinator state"),
    )
    .expect("replay coordinator JSON");
    assert_eq!(coordinator["live_output_claims"].as_object().unwrap().len(), 0);
    assert_eq!(fs::read(bundle_dir.join("manifest.json")).unwrap(), manifest_bytes);

    let replay = remote_failure_replay_command(&fixture.state_dir, &fixture.store_dir, bundle_digest)
        .output()
        .expect("execute remote failure replay");
    assert!(replay.status.success(), "replay stderr={}", String::from_utf8_lossy(&replay.stderr));
    let report: Value = serde_json::from_slice(&replay.stdout).expect("replay report JSON");
    let replay_identity = report["replay_attempt_identity"].as_str().unwrap();
    let replay_parts = replay_identity.split(':').collect::<Vec<_>>();
    assert_eq!(replay_parts.len(), 3);
    assert_ne!(replay_parts[0], original_job_id);
    assert_ne!(replay_parts[1], original_attempt_id);
    assert!(replay_parts[2].parse::<u64>().unwrap() > 0);
    assert!(original_fence > 0);
    assert_eq!(report["comparison"]["class"], "diverged");
    assert_eq!(report["original_result_immutable"], true);
    assert_eq!(report["ordinary_output_admission_applied"], true);
    assert_eq!(report["admitted_output_count"], 1);
    assert_eq!(fs::read(bundle_dir.join("manifest.json")).unwrap(), manifest_bytes);
}

#[test]
fn production_stdio_exports_prometheus_and_propagates_bounded_trace_context() {
    let _guard = serial_transfer_guard();
    let root = tempfile::Builder::new()
        .prefix("observability-positive")
        .tempdir()
        .expect("production fixture tempdir");
    let fixture = setup_production_fixture(root.path(), OUTPUT_BYTES, MAX_UPLOAD_BYTES);
    let config = root.path().join("observability.ncl");
    let prometheus = root.path().join("mantle-remote.prom");
    write_observability_config(&config, Some(&prometheus), None);

    let output = remote_build_command(&fixture.state_dir, &fixture.store_dir, &fixture.build_file)
        .arg("--remote-observability-config")
        .arg(&config)
        .env(TRACEPARENT_ENV, VALID_TRACEPARENT)
        .env(TRACESTATE_ENV, VALID_TRACESTATE)
        .output()
        .expect("run trace-enabled production transfer");
    assert!(output.status.success(), "trace-enabled stderr={}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).expect("trace-enabled production report");
    let observability = &report["remote_observability"][0];
    assert_eq!(observability["health"]["trace_context"]["status"], "accepted");
    assert_eq!(observability["health"]["exporters"]["prometheus"]["status"], "succeeded");
    assert!(observability["immutable_log"]["head_record_blake3"].as_str().is_some());
    assert!(observability["immutable_log"]["next_cursor"].as_u64().unwrap_or(0) > 0);
    assert_production_lifecycle_sequence(&report, false);
    let rendered = String::from_utf8(output.stdout).unwrap();
    let metrics = fs::read_to_string(prometheus).expect("Prometheus textfile");
    assert!(!rendered.contains(VALID_TRACEPARENT));
    assert!(!metrics.contains(VALID_TRACEPARENT));
    assert!(metrics.contains("mantle_remote_admission_total"));
}

#[test]
fn production_stdio_drops_malformed_trace_and_survives_otlp_outage() {
    let _guard = serial_transfer_guard();
    let root = tempfile::Builder::new()
        .prefix("observability-outage")
        .tempdir()
        .expect("production fixture tempdir");
    let fixture = setup_production_fixture(root.path(), OUTPUT_BYTES, MAX_UPLOAD_BYTES);
    let config = root.path().join("observability.ncl");
    write_observability_config(&config, None, Some(OTLP_OUTAGE_ENDPOINT));
    let malformed_traceparent = "x".repeat(MALFORMED_TRACEPARENT_BYTES);

    let output = remote_build_command(&fixture.state_dir, &fixture.store_dir, &fixture.build_file)
        .arg("--remote-observability-config")
        .arg(&config)
        .env(TRACEPARENT_ENV, &malformed_traceparent)
        .env(TRACESTATE_ENV, VALID_TRACESTATE)
        .output()
        .expect("run outage-isolated production transfer");
    assert!(output.status.success(), "outage stderr={}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).expect("outage production report");
    let health = &report["remote_observability"][0]["health"];
    assert_eq!(health["trace_context"]["status"], "dropped");
    assert_eq!(health["exporters"]["otlp"]["status"], "failed");
    assert_eq!(health["immutable_log"]["status"], "succeeded");
    assert_production_lifecycle_sequence(&report, false);
    let output_path = PathBuf::from(report["outcomes"][0]["outputs"][0]["path"].as_str().unwrap());
    assert_eq!(fs::read(output_path).unwrap(), patterned_bytes(OUTPUT_BYTES));
    assert!(!String::from_utf8(output.stdout).unwrap().contains(&malformed_traceparent));
}

struct ProductionFixture {
    state_dir: PathBuf,
    store_dir: PathBuf,
    build_file: PathBuf,
}

fn setup_replay_fixture(root: &Path) -> ProductionFixture {
    let state_dir = root.join("state");
    let store_dir = root.join("store");
    let build_file = root.join("remote-replay.ncl");
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    write_ticket_state(&state_dir);
    write_replayable_derivation(&build_file);
    ProductionFixture {
        state_dir,
        store_dir,
        build_file,
    }
}

fn setup_production_fixture(root: &Path, output_bytes: usize, max_upload_bytes: u64) -> ProductionFixture {
    let state_dir = root.join("state");
    let store_dir = root.join("store");
    let output_source = root.join("output-source.bin");
    let build_file = root.join("remote-production.ncl");
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    fs::write(&output_source, patterned_bytes(output_bytes)).unwrap();
    fs::write(physical_store_path(&store_dir, INPUT_STORE_PATH), patterned_bytes(INPUT_BYTES)).unwrap();
    write_ticket_state_with_upload_limit(&state_dir, max_upload_bytes);
    write_fetch_derivation(&build_file, &output_source);
    assert!(output_bytes > 0);
    assert!(max_upload_bytes > 0);
    ProductionFixture {
        state_dir,
        store_dir,
        build_file,
    }
}

fn write_capture_debug_config(path: &Path) {
    write_capture_debug_config_with_file_bytes(path, ACCEPTING_CAPTURE_FILE_BYTES);
}

fn write_capture_debug_config_with_file_bytes(path: &Path, file_bytes_max: u64) {
    let contract = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/remote-builders.ncl");
    let source = format!(
        "let remote = import \"{}\" in {{ pools = [], failure_debug = ({{ capture = {{ enabled = true, allowed_relative_paths = [\"build/trace.json\"], sensitivity = 'restricted-diagnostic, file_count_max = 1, total_bytes_max = {file_bytes_max}, file_bytes_max = {file_bytes_max}, depth_max = 4, failure_mode = 'diagnostic-only }} }} | remote.RemoteFailureDebug) }}",
        contract.display(),
    );
    fs::write(path, source).expect("capture debug config");
}

fn write_failure_debug_config(path: &Path, replay_enabled: bool) {
    let contract = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/remote-builders.ncl");
    let source = format!(
        "let remote = import \"{}\" in {{ pools = [], failure_debug = ({{ replay_enabled = {replay_enabled} }} | remote.RemoteFailureDebug) }}",
        contract.display(),
    );
    fs::write(path, source).expect("failure debug config");
}

fn write_observability_config(path: &Path, prometheus: Option<&Path>, otlp_endpoint: Option<&str>) {
    let contract = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/remote-builders.ncl");
    let prometheus_config = prometheus.map_or_else(
        || "{ enabled = false }".to_string(),
        |path| format!("{{ enabled = true, textfile_path = \"{}\" }}", path.display()),
    );
    let otlp_config = otlp_endpoint.map_or_else(
        || "{ enabled = false }".to_string(),
        |endpoint| format!("{{ enabled = true, endpoint = \"{endpoint}\", timeout_ms = {OTLP_TEST_TIMEOUT_MS} }}"),
    );
    let source = format!(
        "let remote = import \"{}\" in {{ pools = [], telemetry = ({{ prometheus = {prometheus_config}, otlp = {otlp_config} }} | remote.RemoteTelemetry), trace_context = {{ enabled = true }} }}",
        contract.display(),
    );
    fs::write(path, source).expect("observability config");
}

fn assert_production_lifecycle_sequence(report: &Value, expect_resume: bool) {
    let reasons = report["remote_telemetry_events"]
        .as_array()
        .expect("remote telemetry array")
        .iter()
        .map(|event| event["reason"].as_str().expect("telemetry reason"))
        .collect::<Vec<_>>();
    assert_eq!(&reasons[..PRIORITY_LIFECYCLE_PREFIX_COUNT], [
        "priority-selected",
        "route-selected",
        "queue-admitted",
        "worker-assigned",
        "fence-accepted"
    ]);
    assert_eq!(reasons.last(), Some(&"output-admitted"));
    assert!(reasons.contains(&"transfer-demand"));
    assert!(reasons.contains(&"transfer-credit"));
    assert!(reasons.contains(&"transfer-completed"));
    assert!(reasons.contains(&"execution-completed"));
    assert_eq!(reasons.contains(&"transfer-resumed"), expect_resume);
    let priority_event = &report["remote_telemetry_events"][0];
    assert_eq!(priority_event["measurement"], "candidate-count");
    assert_eq!(priority_event["value"], 1);
    let decisions = report["scheduler_priority_decisions"].as_array().expect("priority decisions");
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0]["competing_goal_count"], 1);
    assert_eq!(decisions[0]["known_critical_path_nodes"], 0);
    assert_eq!(decisions[0]["known_critical_path_work_units"], 0);
    assert_eq!(decisions[0]["blocked_root_count"], 0);
    assert_eq!(decisions[0]["history_basis"], "structural-fallback-missing");
    let execution = reasons.iter().position(|reason| *reason == "execution-completed").unwrap();
    let admission = reasons.iter().position(|reason| *reason == "output-admitted").unwrap();
    assert!(execution < admission);
}

#[derive(Debug)]
struct CheckpointSummary {
    path: PathBuf,
    session_id: String,
    manifest_digest_blake3: String,
    acknowledged_chunk_digests: Vec<String>,
    acknowledged_chunks: usize,
    transferred_bytes: u64,
}

fn partial_output_checkpoint(state_dir: &Path) -> CheckpointSummary {
    let checkpoints = transfer_checkpoints(state_dir);
    checkpoints
        .into_iter()
        .filter(|summary| summary.acknowledged_chunks == 1)
        .max_by_key(|summary| summary.transferred_bytes)
        .expect("partial output checkpoint")
}

fn output_checkpoint_with_most_acknowledgements(state_dir: &Path) -> CheckpointSummary {
    transfer_checkpoints(state_dir)
        .into_iter()
        .max_by_key(|summary| summary.acknowledged_chunks)
        .expect("completed output checkpoint")
}

fn transfer_checkpoints(state_dir: &Path) -> Vec<CheckpointSummary> {
    let mut transfer_dirs = vec![state_dir.join("remote-transfers")];
    let workers_dir = state_dir.join("remote-workers");
    if let Ok(workers) = fs::read_dir(workers_dir) {
        for worker in workers.filter_map(Result::ok).take(MAX_TEST_REMOTE_WORKERS) {
            transfer_dirs.push(worker.path().join("state/remote-transfers"));
        }
    }
    let mut checkpoints = Vec::new();
    for transfer_dir in transfer_dirs {
        let Ok(entries) = fs::read_dir(transfer_dir) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            if entry.path().extension().is_none_or(|extension| extension != "json") {
                continue;
            }
            checkpoints.push(read_checkpoint_summary(&entry.path()));
        }
    }
    checkpoints
}

fn recursive_failure_debug_bundle_dirs(state_dir: &Path) -> Vec<PathBuf> {
    let mut pending = vec![state_dir.to_path_buf()];
    let mut bundles = Vec::new();
    let mut visited = 0_usize;
    while let Some(path) = pending.pop() {
        assert!(visited < MAX_DIAGNOSTIC_SCAN_FILES, "failure debug bundle scan limit exceeded");
        visited = visited.saturating_add(1);
        if path.ends_with(REMOTE_FAILURE_DEBUG_BUNDLE_DIR) {
            bundles.extend(
                fs::read_dir(&path)
                    .unwrap()
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|entry| entry.is_dir()),
            );
            continue;
        }
        if path.is_dir() {
            pending.extend(fs::read_dir(path).unwrap().filter_map(Result::ok).map(|entry| entry.path()));
        }
    }
    bundles.sort();
    bundles
}

fn failure_workspace_directories(state_dir: &Path) -> Vec<PathBuf> {
    let mut pending = vec![state_dir.to_path_buf()];
    let mut workspaces = Vec::new();
    let mut visited = 0_usize;
    while let Some(path) = pending.pop() {
        assert!(visited < MAX_DIAGNOSTIC_SCAN_FILES, "failure workspace scan limit exceeded");
        visited = visited.saturating_add(1);
        if path.file_name().is_some_and(|name| name == "remote-failure-workspaces") {
            workspaces.extend(
                fs::read_dir(&path)
                    .unwrap()
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|entry| entry.is_dir()),
            );
            continue;
        }
        if path.is_dir() {
            pending.extend(fs::read_dir(path).unwrap().filter_map(Result::ok).map(|entry| entry.path()));
        }
    }
    workspaces
}

fn failure_debug_bundle_dirs(state_dir: &Path) -> Vec<PathBuf> {
    let root = state_dir.join(REMOTE_FAILURE_DEBUG_BUNDLE_DIR);
    let mut bundles = fs::read_dir(root)
        .expect("remote failure debug bundle root")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    bundles.sort();
    bundles
}

fn snapshot_file_tree(root: &Path) -> BTreeMap<PathBuf, String> {
    let mut pending = vec![root.to_path_buf()];
    let mut snapshot = BTreeMap::new();
    let mut visited = 0_usize;
    while let Some(path) = pending.pop() {
        assert!(visited < MAX_DIAGNOSTIC_SCAN_FILES, "bundle snapshot file limit exceeded");
        visited = visited.saturating_add(1);
        if path.is_dir() {
            pending.extend(fs::read_dir(&path).unwrap().filter_map(Result::ok).map(|entry| entry.path()));
            continue;
        }
        let relative = path.strip_prefix(root).unwrap().to_path_buf();
        let bytes = fs::read(&path).expect("bundle snapshot file reads");
        snapshot.insert(relative, blake3::hash(&bytes).to_hex().to_string());
    }
    assert!(!snapshot.is_empty());
    snapshot
}

fn state_diagnostics_contain(state_dir: &Path, needle: &str) -> bool {
    assert!(!needle.is_empty());
    let mut pending = vec![state_dir.join(REMOTE_ATTEMPT_LOG_DIR)];
    let mut scanned_files = 0_usize;
    while let Some(path) = pending.pop() {
        if scanned_files >= MAX_DIAGNOSTIC_SCAN_FILES {
            return false;
        }
        if path.is_dir() {
            let Ok(entries) = fs::read_dir(path) else {
                continue;
            };
            pending.extend(entries.filter_map(Result::ok).map(|entry| entry.path()));
            continue;
        }
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        scanned_files = scanned_files.saturating_add(1);
        let retained = &bytes[..bytes.len().min(MAX_DIAGNOSTIC_SCAN_BYTES)];
        if let Ok(value) = serde_json::from_slice::<Value>(retained)
            && json_diagnostic_contains(&value, needle)
        {
            return true;
        }
    }
    assert!(scanned_files <= MAX_DIAGNOSTIC_SCAN_FILES);
    false
}

fn json_diagnostic_contains(root: &Value, needle: &str) -> bool {
    assert!(!needle.is_empty());
    let mut pending = vec![root];
    let mut visited = 0_usize;
    while let Some(value) = pending.pop() {
        if visited >= MAX_DIAGNOSTIC_JSON_NODES {
            return false;
        }
        visited = visited.saturating_add(1);
        match value {
            Value::String(text) => {
                if text.contains(needle) {
                    return true;
                }
            }
            Value::Array(values) => {
                let payload = values
                    .iter()
                    .map(|value| value.as_u64().and_then(|byte| u8::try_from(byte).ok()))
                    .collect::<Option<Vec<_>>>();
                if payload.is_some_and(|bytes| String::from_utf8_lossy(&bytes).contains(needle)) {
                    return true;
                }
                pending.extend(values);
            }
            Value::Object(fields) => pending.extend(fields.values()),
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
        }
    }
    assert!(visited <= MAX_DIAGNOSTIC_JSON_NODES);
    false
}

fn read_checkpoint_summary(path: &Path) -> CheckpointSummary {
    let value: Value = serde_json::from_slice(&fs::read(path).expect("checkpoint bytes")).expect("checkpoint JSON");
    let acknowledged_chunk_digests = value["checkpoint"]["acknowledged_chunk_digests"]
        .as_array()
        .expect("acknowledged chunk array")
        .iter()
        .map(|digest| digest.as_str().expect("acknowledged chunk digest").to_string())
        .collect::<Vec<_>>();
    CheckpointSummary {
        path: path.to_path_buf(),
        session_id: value["checkpoint"]["scope"]["session_id"].as_str().expect("checkpoint session id").to_string(),
        manifest_digest_blake3: value["checkpoint"]["scope"]["manifest_digest_blake3"]
            .as_str()
            .expect("checkpoint manifest digest")
            .to_string(),
        acknowledged_chunks: acknowledged_chunk_digests.len(),
        acknowledged_chunk_digests,
        transferred_bytes: value["checkpoint"]["transferred_bytes"].as_u64().expect("transferred byte count"),
    }
}

fn remote_failure_replay_command(state_dir: &Path, store_dir: &Path, bundle_digest: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mantle"));
    configure_test_sandbox_command(&mut command);
    command.args(["--json", "--state-dir"]).arg(state_dir).arg("--store").arg(store_dir).args([
        "--store-prefix",
        STORE_PREFIX,
        "remote",
        "debug",
        "replay",
        bundle_digest,
        "--builder",
        BUILDER_ID,
        "--ticket-fd",
        &REMOTE_TICKET_INPUT_FD.to_string(),
        "--remote-build-time-secs",
        &MAX_BUILD_TIME_SECS.to_string(),
        "--remote-secret-manifest",
        secret_manifest().to_str().expect("UTF-8 secret manifest"),
    ]);
    command.arg("--remote-binding-config").arg(remote_binding_config_path(state_dir));
    command.env("CREDENTIALS_DIRECTORY", state_dir.join("credentials"));
    configure_ticket_input_fd(&mut command, state_dir);
    command
}

fn remote_build_command(state_dir: &Path, store_dir: &Path, build_file: &Path) -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin("mantle"));
    configure_test_sandbox_command(&mut command);
    command.args([
        "--json",
        "--state-dir",
        state_dir.to_str().expect("UTF-8 state dir"),
        "--store",
        store_dir.to_str().expect("UTF-8 store dir"),
        "--store-prefix",
        STORE_PREFIX,
        "build",
        build_file.to_str().expect("UTF-8 build file"),
        "--no-substitute",
        "--builder",
        BUILDER_ID,
        "--ticket-fd",
        &REMOTE_TICKET_INPUT_FD.to_string(),
        "--remote-build-time-secs",
        &MAX_BUILD_TIME_SECS.to_string(),
        "--remote-secret-manifest",
        secret_manifest().to_str().expect("UTF-8 secret manifest"),
    ]);
    command.arg("--remote-binding-config").arg(remote_binding_config_path(state_dir));
    command.env("CREDENTIALS_DIRECTORY", state_dir.join("credentials"));
    configure_ticket_input_fd(&mut command, state_dir);
    command
}

fn remote_gallery_build_command(state_dir: &Path, store_dir: &Path) -> Command {
    let project_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(GALLERY_PROJECT_RELATIVE_PATH);
    assert!(project_dir.join("mantle-project.ncl").is_file());
    assert!(project_dir.join("README.md").is_file());
    let mut command = remote_build_command(state_dir, store_dir, Path::new(GALLERY_RESUMABLE_SELECTOR));
    command.current_dir(project_dir);
    command
}

fn configure_test_sandbox_command(command: &mut Command) {
    command.env("CRUNCH_NO_FUSE", "1");
    let Some(bwrap_path) = std::env::var_os(TEST_REAL_BWRAP_ENV) else {
        return;
    };
    assert!(Path::new(&bwrap_path).is_file(), "declared test bwrap must be a file");
    command.env("SNIX_BUILD_BWRAP", bwrap_path);
}

fn write_ticket_state(state_dir: &Path) {
    write_ticket_state_with_upload_limit(state_dir, MAX_UPLOAD_BYTES);
}

fn write_ticket_state_with_upload_limit(state_dir: &Path, max_upload_bytes: u64) {
    write_ticket_state_full(state_dir, max_upload_bytes, TICKET_USES, &[BUILDER_ID]);
}

fn write_ticket_state_full(state_dir: &Path, max_upload_bytes: u64, uses_remaining: u32, builders: &[&str]) {
    let ticket_dir = state_dir.join("remote-builders");
    let created_unix_s = test_unix_time_now_s();
    let expires_unix_s = created_unix_s.checked_add(TEST_TICKET_TTL_SECS).unwrap();
    fs::create_dir_all(&ticket_dir).expect("ticket state dir");
    fs::set_permissions(&ticket_dir, fs::Permissions::from_mode(PRIVATE_DIRECTORY_MODE)).expect("private ticket dir");
    let state = serde_json::json!({
        "schema_version": 2,
        "next_ticket_sequence": 1,
        "tickets": {
            TICKET_ID: {
                "id": TICKET_ID,
                "display_name": "production transfer test",
                "verifier_key_id": "ticket-key-1",
                "verifier": ticket_verifier(),
                "created_unix_s": created_unix_s,
                "expires_unix_s": expires_unix_s,
                "uses_remaining": uses_remaining,
                "max_build_time_secs": MAX_BUILD_TIME_SECS,
                "max_upload_bytes": max_upload_bytes,
                "bound_client_endpoint": null,
                "revoked": false
            }
        },
        "invalidated_legacy_ticket_ids": []
    });
    let state_path = ticket_dir.join("tickets.json");
    fs::write(&state_path, serde_json::to_vec_pretty(&state).expect("ticket state JSON")).expect("ticket state");
    fs::set_permissions(state_path, fs::Permissions::from_mode(PRIVATE_FILE_MODE)).expect("private ticket file");
    let credential_path = state_dir.join(REMOTE_TICKET_INPUT_FILENAME);
    fs::write(&credential_path, format!("{TICKET_ID}:{TICKET_SECRET}\n")).expect("ticket input file");
    fs::set_permissions(credential_path, fs::Permissions::from_mode(PRIVATE_FILE_MODE))
        .expect("private ticket input file");
    write_service_credentials(state_dir);
    // The client and its generated local receiver both re-resolve this binding.
    if builders.len() == 1 {
        remote_endpoint_binding::write_endpoint_bindings(
            state_dir,
            &remote_binding_config_path(state_dir),
            remote_endpoint_binding::Bindings {
                store_prefix: STORE_PREFIX,
                builder: builders.first().copied(),
                substituters: &[],
            },
        );
    } else {
        remote_endpoint_binding::write_pool_endpoint_bindings(
            state_dir,
            &remote_binding_config_path(state_dir),
            STORE_PREFIX,
            builders,
        );
    }
}

fn remote_binding_config_path(state_dir: &Path) -> PathBuf {
    state_dir.parent().expect("state dir has a parent").join(REMOTE_BINDING_CONFIG_FILENAME)
}

fn test_unix_time_now_s() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
}

fn configure_ticket_input_fd(command: &mut Command, state_dir: &Path) {
    configure_ticket_input_fds(command, state_dir, &[REMOTE_TICKET_INPUT_FD]);
}

fn configure_ticket_input_fds(command: &mut Command, state_dir: &Path, fds: &[i32]) {
    // Each target descriptor needs its own open file description: the reader
    // consumes the ticket to EOF, so duplicated descriptors would share an
    // offset and the second endpoint would observe an empty ticket.
    let ticket_path = state_dir.join(REMOTE_TICKET_INPUT_FILENAME);
    let ticket_files = fds.iter().map(|_| fs::File::open(&ticket_path).expect("ticket input file")).collect::<Vec<_>>();
    let targets = fds.to_vec();
    unsafe {
        command.pre_exec(move || {
            // Duplicate every source above the target range first: a parent fd
            // that happens to equal another target would otherwise be
            // overwritten before its own dup2 runs.
            let mut sources = Vec::with_capacity(ticket_files.len());
            for ticket_file in &ticket_files {
                let high = libc::fcntl(ticket_file.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 64);
                if high < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                sources.push(high);
            }
            for (source, fd) in sources.iter().zip(&targets) {
                if libc::dup2(*source, *fd) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                libc::close(*source);
            }
            Ok(())
        });
    }
}

fn ticket_verifier() -> String {
    let mut input = Vec::from(TICKET_VERIFIER_DOMAIN);
    input.extend_from_slice(&[0x42_u8; 32]);
    let key = [0x41_u8; 32];
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(blake3::keyed_hash(&key, &input).as_bytes())
}

fn write_service_credentials(state_dir: &Path) {
    let credentials_dir = state_dir.join("credentials");
    fs::create_dir_all(&credentials_dir).expect("credentials dir");
    fs::write(credentials_dir.join("TICKET_VERIFIER_KEY"), TICKET_KEY).expect("ticket key credential");
    fs::write(credentials_dir.join("RESULT_SIGNING_KEY"), RESULT_SIGNING_KEY).expect("signing key credential");
}

fn secret_manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("secretspec.toml")
}

fn write_replayable_derivation(path: &Path) {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    let source = format!(
        r#"let mantle = import "{}" in
{{
  name = "production-remote-replay",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  args = ["-c", "printf replay-ok > $out"],
  outputs = ["out"],
  env = {{}},
  inputs = [],
  addressing_mode = 'input-addressed,
  sandbox = 'native,
}} | mantle.Derivation
"#,
        lib.display(),
    );
    fs::write(path, source).expect("replayable derivation");
}

fn write_pool_diamond_derivation(path: &Path, addressing_mode: Option<&str>) {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    let addressing = match addressing_mode {
        Some(mode) => format!("addressing_mode = '{mode},\n  "),
        None => String::new(),
    };
    let source = format!(
        r#"let mantle = import "{}" in
let shared = {{
  name = "pool-diamond-shared",
  builder = "/bin/sh",
  args = ["-c", "printf '%s\n' 'shared foundation' > \"$out\""],
  outputs = ["out"],
  env = {{}},
  inputs = [],
  {addressing}sandbox = 'native,
}} | mantle.Derivation in
let branch = fun branch_name branch_message => {{
  name = branch_name,
  builder = "/bin/sh",
  inputs = [shared],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      shared_input=""
      for candidate in "$NIX_STORE"/*-pool-diamond-shared; do
        if [ -f "$candidate" ]; then
          shared_input="$candidate"
          break
        fi
      done
      if [ -z "$shared_input" ]; then
        echo 'shared diamond input was not mounted' >&2
        exit 1
      fi
      $BB cat "$shared_input" > "$out"
      $BB printf '%s\n' '%{{branch_message}}' >> "$out"
    "%
  ],
  outputs = ["out"],
  env = {{}},
  {addressing}sandbox = 'native,
}} | mantle.Derivation in
let left = branch "pool-diamond-left" "left branch" in
let right = branch "pool-diamond-right" "right branch" in
{{
  name = "pool-diamond-result",
  builder = "/bin/sh",
  inputs = [left, right],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      left_input=""
      right_input=""
      for candidate in "$NIX_STORE"/*-pool-diamond-left; do
        if [ -f "$candidate" ]; then
          left_input="$candidate"
          break
        fi
      done
      for candidate in "$NIX_STORE"/*-pool-diamond-right; do
        if [ -f "$candidate" ]; then
          right_input="$candidate"
          break
        fi
      done
      if [ -z "$left_input" ] || [ -z "$right_input" ]; then
        echo 'both diamond branches must be mounted' >&2
        exit 1
      fi
      $BB mkdir -p "$out"
      $BB cat "$left_input" > "$out/graph.txt"
      $BB cat "$right_input" >> "$out/graph.txt"
    "%
  ],
  outputs = ["out"],
  env = {{}},
  {addressing}sandbox = 'native,
}} | mantle.Derivation
"#,
        lib.display(),
    );
    fs::write(path, source).expect("pool diamond derivation");
}

fn write_slow_derivation(path: &Path) {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    let source = format!(
        r#"let mantle = import "{}" in
{{
  name = "production-remote-slow",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  args = ["-c", "/bin/busybox sleep 20; printf slow-ok > $out"],
  outputs = ["out"],
  env = {{}},
  inputs = [],
  addressing_mode = 'input-addressed,
  sandbox = 'native,
}} | mantle.Derivation
"#,
        lib.display(),
    );
    fs::write(path, source).expect("slow derivation");
}

fn write_missing_input_derivation(path: &Path) {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    let source = format!(
        r#"let mantle = import "{}" in
{{
  name = "production-remote-missing-input",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  args = ["-c", "printf missing-input-ok > $out"],
  outputs = ["out"],
  env = {{}},
  inputs = ["{}"],
  addressing_mode = 'input-addressed,
  sandbox = 'native,
}} | mantle.Derivation
"#,
        lib.display(),
        INPUT_STORE_PATH,
    );
    fs::write(path, source).expect("missing-input derivation");
}

fn write_named_replay_derivation(path: &Path, name: &str) {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    let source = format!(
        r#"let mantle = import "{}" in
{{
  name = "{}",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  args = ["-c", "printf replay-ok > $out"],
  outputs = ["out"],
  env = {{}},
  inputs = [],
  addressing_mode = 'input-addressed,
  sandbox = 'native,
}} | mantle.Derivation
"#,
        lib.display(),
        name,
    );
    fs::write(path, source).expect("named replayable derivation");
}

fn write_content_addressed_derivation(path: &Path) {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    let source = format!(
        r#"let mantle = import "{}" in
{{
  name = "production-remote-ca-replay",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  args = ["-c", "printf ca-replay-ok > $out"],
  outputs = ["out"],
  env = {{}},
  inputs = [],
  sandbox = 'native,
}} | mantle.Derivation
"#,
        lib.display(),
    );
    fs::write(path, source).expect("content-addressed derivation");
}

fn write_failing_capture_derivation(path: &Path) {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    let source = format!(
        r#"let mantle = import "{}" in
{{
  name = "production-remote-failure",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  args = ["-c", "printf '{}' > trace.json; printf 'DO_NOT_' > secret.txt; printf 'CAPTURE' >> secret.txt; exit 7"],
  outputs = ["out"],
  env = {{}},
  inputs = [],
  addressing_mode = 'input-addressed,
  sandbox = 'native,
}} | mantle.Derivation
"#,
        lib.display(),
        CAPTURED_TRACE,
    );
    fs::write(path, source).expect("failing capture derivation");
}

fn write_fetch_derivation(path: &Path, output_source: &Path) {
    let bytes = fs::read(output_source).expect("output source bytes");
    let digest = sha2::Sha256::digest(&bytes);
    let hash = format!("sha256-{}", base64::engine::general_purpose::STANDARD.encode(digest));
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    let source = format!(
        r#"let mantle = import "{}" in
{{
  name = "production-remote-output",
  builder = "builtin:fetchurl",
  system = 'x86_64-linux,
  args = [],
  outputs = ["out"],
  env = {{ url = "file://{}" }},
  inputs = ["{}"],
  fixed_output = {{ hash = "{}", algo = 'sha256, mode = 'flat }},
  addressing_mode = 'input-addressed,
  sandbox = 'native,
}} | mantle.Derivation
"#,
        lib.display(),
        output_source.display(),
        INPUT_STORE_PATH,
        hash,
    );
    fs::write(path, source).expect("production transfer derivation");
}

fn physical_store_path(store_dir: &Path, logical_path: &str) -> PathBuf {
    let name = logical_path.rsplit('/').next().expect("logical store basename");
    store_dir.join(name)
}

fn patterned_bytes(size: usize) -> Vec<u8> {
    (0..size).map(|index| u8::try_from(index % PATTERN_MODULUS).expect("pattern byte")).collect()
}
