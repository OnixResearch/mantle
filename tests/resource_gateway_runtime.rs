#![cfg(target_os = "linux")]
#[path = "support/gateway_runtime.rs"]
mod gateway_runtime;

use std::fs;
use std::path::PathBuf;

use gateway_runtime::GatewayRuntime;
use gateway_runtime::PROJECT;
use gateway_runtime::private_dir;
use gateway_runtime::private_json;
use serde_json::Value;
use serde_json::json;

fn inputs() -> PathBuf {
    PathBuf::from(std::env::var_os("MANTLE_GATEWAY_INPUT_PATHS").expect("signed input paths descriptor"))
}
fn artifact_root() -> PathBuf {
    let root = PathBuf::from(std::env::var_os("MANTLE_GATEWAY_ARTIFACT_ROOT").expect("fresh retained artifact root"));
    private_dir(&root);
    root
}
fn submit(runtime: &mut GatewayRuntime) -> String {
    let (ok, response) = runtime.call(
        json!({"operation":"submit","drv_path":runtime.drv_path,
        "idempotency_key":"fixed-runtime-request"}),
        PROJECT,
    );
    assert!(ok, "real signed submission rejected: {response}");
    response["response"]["Submitted"]["attempt_id"].as_str().unwrap().into()
}
fn final_record(runtime: &GatewayRuntime, id: &str) -> Value {
    let state = runtime.coordinator();
    private_json(&runtime.root.join("final-coordinator.json"), &state);
    state["gateway_attempts"][id].clone()
}
fn usage(runtime: &mut GatewayRuntime) -> Value {
    let (ok, response) = runtime.call(json!({"operation":"usage","items":16}), PROJECT);
    assert!(ok, "fresh UsageRead invocation failed: {response}");
    response["response"]["Usage"].clone()
}
fn confirm_output(runtime: &mut GatewayRuntime, id: &str) {
    let (ok, response) = runtime.call(json!({"operation":"result","attempt_id":id}), PROJECT);
    assert!(ok, "signed result/CAS verification failed: {response}");
    let physical = runtime.store.join(runtime.output_path.rsplit('/').next().unwrap());
    assert_eq!(fs::read(physical).unwrap(), b"gateway-success\n");
    confirm_events(runtime, id, "Succeeded");
}
fn quota_snapshot(runtime: &GatewayRuntime) -> crunch_remote_core::resource_quota::Ledger {
    crunch_build::distributed::ResourceQuotaStore::open(&runtime.root.join("quota"))
        .unwrap()
        .snapshot()
        .unwrap()
}

fn confirm_events(runtime: &mut GatewayRuntime, id: &str, phase: &str) {
    let (ok, response) = runtime.call(json!({"operation":"events","attempt_id":id,"items":16}), PROJECT);
    assert!(ok, "EventRead invocation failed: {response}");
    assert_eq!(response["response"]["Events"]["values"].as_array().unwrap().last().unwrap()["phase"], phase);
}

#[test]
#[ignore = "requires signed static BusyBox descriptor and bwrap; retains actual private gateway/CLI artifacts"]
fn real_gateway_public_usage_process_only_and_authority_negatives() {
    let mut runtime = GatewayRuntime::prepare(&artifact_root().join("process-only"), &inputs(), None, "simple");
    let gateway = runtime.start();
    let id = submit(&mut runtime);
    let status = runtime.wait_terminal(&id);
    assert_eq!(status["response"]["Status"]["observation"]["phase"], "Succeeded");
    confirm_output(&mut runtime, &id);
    let summary = usage(&mut runtime);
    assert_eq!(summary["accounting"]["project_reason_counts"]["partial_observation"], 1);
    assert_eq!(summary["accounting"]["project_reserved_units"], 100);
    assert_eq!(summary["accounting"]["project_charged_units"], 0);
    assert_eq!(summary["accounting"]["schedule_version"], 1);
    assert_eq!(summary["accounting"]["cpu_ms_per_unit"], 1000);
    let before = quota_snapshot(&runtime);
    for count in [0, crunch_nix_gateway::api::MAX_USAGE_ITEMS + 1] {
        let (ok, response) = runtime.call(json!({"operation":"usage","items":count}), PROJECT);
        assert!(!ok);
        assert_eq!(response["error"], "gateway-limit-exceeded");
    }
    assert!(!runtime.call(json!({"operation":"usage","items":1}), "unknown-project").0);
    let (ok, response) = runtime.call_bound(
        json!({"operation":"status","attempt_id":id}),
        json!({"operation":"usage","items":1}),
        PROJECT,
    );
    assert!(!ok);
    assert_eq!(response["error"], "gateway-authority-denied");
    assert_eq!(quota_snapshot(&runtime), before, "denied reads cannot mutate quota");
    let record = final_record(&runtime, &id);
    assert_eq!(record["usage"]["facts"]["completeness"], "ProcessOnly");
    assert_eq!(record["usage"]["facts"]["oom"], "unavailable");
    assert_eq!(record["retries"], json!([]));
    drop(gateway);
    runtime.retain_final_identity();
}

#[test]
#[ignore = "requires CPU+memory delegated cgroup-v2 root with swap disabled, signed BusyBox descriptor and bwrap"]
fn real_gateway_signed_oom_retry_usage_and_negative_controls() {
    let root = artifact_root();
    let cgroup = PathBuf::from(std::env::var_os("MANTLE_TEST_CGROUP_ROOT").expect("empty delegated cgroup root"));
    for (name, workload, enabled, opted_in, retry_expected) in [
        ("positive", "oom", true, true, true),
        ("retry-disabled", "oom", false, true, false),
        ("not-opted-in", "oom", true, false, false),
        ("sigkill", "sigkill", true, true, false),
    ] {
        let mut runtime = GatewayRuntime::prepare(&root.join(name), &inputs(), Some(&cgroup), workload);
        runtime.config["resources"]["oom_retry_enabled"] = enabled.into();
        if !opted_in {
            runtime.config["resources"]["opted_in_projects"] = json!([]);
        }
        let gateway = runtime.start();
        let id = submit(&mut runtime);
        let status = runtime.wait_terminal(&id);
        let record = final_record(&runtime, &id);
        if retry_expected {
            assert_eq!(status["response"]["Status"]["observation"]["phase"], "Succeeded");
            verify_successor(&runtime, &record);
            confirm_output(&mut runtime, &id);
            let summary = usage(&mut runtime);
            assert_eq!(summary["accounting"]["project_reason_counts"]["settled"], 2);
            assert_eq!(summary["accounting"]["project_reserved_units"], 0);
            assert_eq!(summary["accounting"]["project_charged_units"], summary["accounting"]["account_charged_units"]);
            assert_eq!(quota_snapshot(&runtime).entries.len(), 2);
        } else {
            assert_eq!(status["response"]["Status"]["observation"]["phase"], "Failed");
            assert_eq!(record["retries"], json!([]));
            assert_eq!(record["binding"]["fence_generation"], 1);
            let facts = &record["usage"]["facts"];
            assert_eq!(facts["completeness"], "DedicatedCgroup");
            if workload == "sigkill" {
                assert_eq!(facts["oom"], "dedicated-cgroup-no-oom");
            } else {
                assert!(facts["oom"]["dedicated-cgroup-oom"]["oom_kill_delta"].as_u64().unwrap() > 0);
            }
            assert_eq!(quota_snapshot(&runtime).entries.len(), 1);
            usage(&mut runtime);
            confirm_events(&mut runtime, &id, "Failed");
        }
        assert_eq!(runtime.coordinator()["resource_leases"], json!({}));
        drop(gateway);
        runtime.retain_final_identity();
    }
}

fn verify_successor(runtime: &GatewayRuntime, record: &Value) {
    let retries = record["retries"].as_array().unwrap();
    assert_eq!(retries.len(), 1);
    let retry = &retries[0];
    assert_eq!(retry["successor"], record["binding"]);
    assert_eq!(retry["predecessor"]["job_id"], retry["successor"]["job_id"]);
    assert_ne!(retry["predecessor"]["attempt_id"], retry["successor"]["attempt_id"]);
    assert!(
        retry["successor"]["fence_generation"].as_u64().unwrap()
            > retry["predecessor"]["fence_generation"].as_u64().unwrap()
    );
    assert_eq!(retry["signed_usage"]["facts"]["binding"], retry["predecessor"]);
    assert_eq!(record["usage"]["facts"]["binding"], retry["successor"]);
    assert!(retry["signed_usage"]["facts"]["oom"]["dedicated-cgroup-oom"]["oom_kill_delta"].as_u64().unwrap() > 0);
    assert_eq!(record["usage"]["facts"]["oom"], "dedicated-cgroup-no-oom");
    let ledger = quota_snapshot(runtime);
    let predecessor = retry["predecessor"]["attempt_id"].as_str().unwrap();
    let successor = retry["successor"]["attempt_id"].as_str().unwrap();
    assert_eq!(ledger.resource_contexts[successor].predecessor.as_ref().unwrap().as_str(), predecessor);
    assert_eq!(ledger.resource_contexts[predecessor].machine_class, "small");
    assert_eq!(ledger.resource_contexts[successor].machine_class, "large");
    let leaf = retry["predecessor_evidence"].as_str().unwrap();
    let bytes = fs::read(runtime.root.join("evidence").join(leaf)).unwrap();
    assert_eq!(format!("{}.json", blake3::hash(&bytes).to_hex()), leaf);
    let bundle: valence_core::build_service::BuildServiceBundle = serde_json::from_slice(&bytes).unwrap();
    assert!(valence_core::build_service::validate_build_service_bundle(&bundle).valid);
}
