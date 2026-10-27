#![cfg(target_os = "linux")]
//! Opt-in actual CLI/gateway/daemon rail. No demand fact or coordinator request is constructed
//! here.
#[path = "support/gateway_runtime.rs"]
mod gateway_runtime;

use std::fs;
use std::io::Read as _;
use std::os::fd::AsRawFd as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Output;
use std::time::Duration;
use std::time::Instant;

use gateway_runtime::ENDPOINT;
use gateway_runtime::FIRST_MEMORY;
use gateway_runtime::GatewayRuntime;
use gateway_runtime::PROJECT;
use gateway_runtime::inherit_fd;
use gateway_runtime::private_json;
use serde_json::Value;
use serde_json::json;

struct LiveProcess(Child);
impl Drop for LiveProcess {
    fn drop(&mut self) {
        unsafe {
            libc::kill(self.0.id() as i32, libc::SIGTERM);
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        while self.0.try_wait().unwrap().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(25));
        }
        if self.0.try_wait().unwrap().is_none() {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}

fn retain_command(command: &mut Command, prefix: &Path) -> Output {
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|part| part.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    private_json(&prefix.with_extension("command.json"), &json!(argv));
    let output = command.output().unwrap();
    fs::write(prefix.with_extension("stdout"), &output.stdout).unwrap();
    fs::write(prefix.with_extension("stderr"), &output.stderr).unwrap();
    fs::write(prefix.with_extension("exit"), format!("{:?}\n", output.status.code())).unwrap();
    output
}

fn json_lines(path: &Path) -> Vec<Value> {
    fs::read(path)
        .unwrap_or_default()
        .split_inclusive(|byte| *byte == b'\n')
        .filter(|line| line.last() == Some(&b'\n'))
        .map(|line| serde_json::from_slice(line).expect("subscriber emitted invalid complete NDJSON"))
        .collect()
}

fn binary_digest(path: &Path) -> String {
    let mut file = fs::File::open(path).unwrap();
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 65_536];
    loop {
        let count = file.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    hasher.finalize().to_hex().to_string()
}

fn wait_for(message: &str, predicate: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !predicate() {
        assert!(Instant::now() < deadline, "{message}");
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn start_live(runtime: &GatewayRuntime, socket: &Path, subscribe: bool) -> LiveProcess {
    let name = if subscribe { "subscriber" } else { "daemon" };
    let mut command = runtime.command();
    command
        .args([
            "remote",
            "live",
            if subscribe { "subscribe" } else { "serve" },
            "--socket",
        ])
        .arg(socket)
        .stdout(fs::File::create(runtime.root.join(format!("{name}.ndjson"))).unwrap())
        .stderr(fs::File::create(runtime.root.join(format!("{name}.stderr"))).unwrap());
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|part| part.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    private_json(&runtime.root.join(format!("{name}.command.json")), &json!(argv));
    LiveProcess(command.spawn().unwrap())
}

fn ordinary_observation(runtime: &GatewayRuntime, name: &str, plan: bool) -> Value {
    let delivery = runtime.issue_ticket();
    let mut command = runtime.command();
    command
        .arg("--remote-binding-config")
        .arg(runtime.root.join("private/endpoint.json"))
        .arg("build")
        .arg(&runtime.source)
        .args([
            "--no-substitute",
            "--builder",
            ENDPOINT,
            "--ticket-fd",
            "9",
            "--builder-program",
        ])
        .arg(&runtime.binary)
        .arg("--remote-observability-config")
        .arg(runtime.root.join("private/observability.ncl"))
        .arg("--remote-resource-requirements")
        .arg(runtime.root.join("requirements.json"));
    for key in runtime.config["trusted_builder_keys"].as_array().unwrap() {
        command.arg("--trusted-builder-key").arg(key.as_str().unwrap());
    }
    for arg in runtime.config["builder_args"].as_array().unwrap() {
        command.arg(format!("--builder-arg={}", arg.as_str().unwrap()));
    }
    if plan {
        command.arg("--plan");
    }
    inherit_fd(&mut command, delivery.as_raw_fd());
    let output = retain_command(&mut command, &runtime.root.join(name));
    if plan {
        assert_eq!(output.status.code(), Some(1), "no-fit plan must remain blocked");
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["entries"][0]["drv_key"], runtime.drv_path);
        assert_eq!(report["entries"][0]["action"], "preflight-error");
        let placement = &report["entries"][0]["remote_placement"];
        assert_eq!(placement["blocker_reason"], "resource-memory-unavailable");
        placement["worker_demand"].clone()
    } else {
        assert_eq!(output.status.code(), Some(3), "normal fail-fast admission exit changed");
        let report: Value = String::from_utf8(output.stderr)
            .unwrap()
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .find(|value| value.get("worker_demand").is_some())
            .expect("actual build JSON omitted derived demand");
        assert_eq!(report["kind"], "internal");
        assert_eq!(report["code"], 3);
        assert_eq!(report["error"], "remote coordinator dispatch failed: resource-memory-unavailable");
        report["worker_demand"].clone()
    }
}

fn assert_no_capacity_grant(runtime: &GatewayRuntime) {
    let state = runtime.coordinator();
    for field in ["jobs", "resource_leases", "workspace_leases", "live_output_claims"] {
        assert_eq!(state[field], json!({}), "demand created {field}");
    }
    assert_eq!(state["workers"][ENDPOINT]["resource_inventory"]["total"]["memory_bytes"], FIRST_MEMORY / 2);
    assert_eq!(state["workers"][ENDPOINT]["concurrency"], 64);
    assert!(!runtime.store.join(Path::new(&runtime.output_path).file_name().unwrap()).exists());
}

#[test]
#[ignore = "requires the real CLI and retained signed Busybox input cache; parent runs serially"]
fn real_gateway_derived_demand_and_ordinary_reports_keep_quantified_blocker() {
    let root =
        PathBuf::from(std::env::var_os("MANTLE_DEMAND_RUNTIME_ROOT").expect("set a fresh retained runtime root"));
    let paths = PathBuf::from(std::env::var_os("MANTLE_RESOURCE_INPUT_PATHS").expect("set signed input descriptor"));
    let mut runtime = GatewayRuntime::prepare(&root, &paths, None, "simple");
    // Concurrency/presence are deliberately large; only explicit current inventory counts.
    runtime.write_observability_with_concurrency(FIRST_MEMORY / 2, 64);
    private_json(&root.join("requirements.json"), &gateway_runtime::requirements(FIRST_MEMORY));
    let socket_root = runtime.socket.parent().unwrap().to_path_buf();
    runtime.live_socket = Some(socket_root.join("absent.sock"));
    let coordinator_path = runtime.state.join("remote-coordinator-state.json");
    let before_plan = fs::read(&coordinator_path).ok();
    let absent_plan = ordinary_observation(&runtime, "absent-plan", true);
    assert_eq!(fs::read(&coordinator_path).ok(), before_plan, "readonly plan mutated coordinator state");
    let absent_build = ordinary_observation(&runtime, "absent-build", false);
    assert_eq!(absent_plan, absent_build, "plan/build did not restate one identical requirement");
    assert_no_capacity_grant(&runtime);

    let live = socket_root.join("live.sock");
    runtime.live_socket = Some(live.clone());
    let mut daemon = start_live(&runtime, &live, false);
    wait_for("real daemon socket unavailable", || live.exists());
    let mut subscriber = start_live(&runtime, &live, true);
    let events_path = root.join("subscriber.ndjson");
    wait_for("subscriber did not complete its real initial snapshot", || {
        json_lines(&events_path).last().is_some_and(|row| row["op"] == "snapshot-end")
    });
    let gateway = runtime.start();
    let mut rounds = Vec::new();
    let mut previous_events = json_lines(&events_path).len();
    for round in 0..2 {
        let (ok, response) = runtime.call(
            json!({"operation":"submit", "drv_path":runtime.drv_path,
            "idempotency_key":"bounded-unmet-requirement"}),
            PROJECT,
        );
        assert!(!ok, "no-fit request was accepted: {response}");
        wait_for("derived production demand did not publish and retract", || {
            let events = json_lines(&events_path);
            events[previous_events..].iter().any(|row| row["op"] == "retract")
        });
        let events = json_lines(&events_path);
        let changes = &events[previous_events..];
        assert_eq!(changes.len(), 2, "one owned request must publish once and retract once");
        assert_eq!(changes[0]["op"], "publish");
        assert_eq!(changes[0]["fact"]["value"]["kind"], "worker-demand");
        assert_eq!(changes[1]["op"], "retract");
        assert_eq!(changes[0]["owner_run_id"], changes[1]["owner_run_id"]);
        assert_eq!(changes[0]["fact_id"], changes[1]["fact_id"]);
        let demand = &changes[0]["fact"]["value"]["demand"];
        assert_eq!(demand["blocker_reason"], absent_build["blocker_reason"]);
        assert_eq!(demand["requirement"], absent_build["requirement"]);
        let owner = changes[0]["owner_run_id"].as_str().unwrap();
        let pid: u32 = owner.split('-').nth(1).unwrap().parse().unwrap();
        let mut cmdline = Vec::new();
        fs::File::open(format!("/proc/{pid}/cmdline")).unwrap().read_to_end(&mut cmdline).unwrap();
        assert!(cmdline.split(|byte| *byte == 0).any(|arg| arg == b"gateway"));
        assert!(cmdline.split(|byte| *byte == 0).any(|arg| arg == b"serve"));
        assert_eq!(unsafe { libc::kill(pid as i32, 0) }, 0, "owner exited instead of remaining a real gateway");
        rounds.push(json!({"round":round,"response":response,"events":changes,"owner_pid":pid}));
        previous_events = events.len();
        assert_no_capacity_grant(&runtime);
    }
    assert_eq!(
        rounds[0]["events"][0]["fact"]["value"]["demand"], rounds[1]["events"][0]["fact"]["value"]["demand"],
        "repeat request changed requirement identity"
    );
    let present_plan = ordinary_observation(&runtime, "present-plan", true);
    let present_build = ordinary_observation(&runtime, "present-build", false);
    assert_eq!(present_plan, absent_plan, "daemon availability changed plan");
    assert_eq!(present_build, absent_build, "daemon availability changed admission/error report");
    assert_no_capacity_grant(&runtime);
    assert!(daemon.0.try_wait().unwrap().is_none());
    assert!(subscriber.0.try_wait().unwrap().is_none());
    drop(gateway);
    runtime.retain_final_identity();
    private_json(
        &root.join("result.json"),
        &json!({
            "status":"passed", "scenario":"actual-long-lived-production-gateway-derived-demand",
            "binary":{"path":runtime.binary,"blake3":binary_digest(&runtime.binary)},
            "driver_blake3":blake3::hash(include_bytes!("demand_runtime.rs")).to_hex().to_string(),
            "fixture_blake3":blake3::hash(include_bytes!("support/gateway_runtime.rs")).to_hex().to_string(),
            "absent_plan_demand":absent_plan,"absent_build_demand":absent_build,
            "present_plan_demand":present_plan,"present_build_demand":present_build,"rounds":rounds,
            "source_drv_path":runtime.drv_path,"declared_memory_bytes":FIRST_MEMORY,
            "live_socket":live,
            "admitted_memory_bytes":FIRST_MEMORY/2,"jobs":0,"resource_leases":0,
            "non_claims":["no capacity inference","no supervisor provisioning","blocked work did not execute",
                          "repeat fail-fast request lifetimes are not repeated pending placement rounds"]
        }),
    );
}
