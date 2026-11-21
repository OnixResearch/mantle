// r[verify foreign_derivation_import.cache_inputs_build_roots]
//! CLI qualification for the cache-inputs build route: a loopback signed Nix
//! cache supplies the selected root's direct input output and input source,
//! then Mantle executes the root in the sandbox.

use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

#[path = "support/remote_endpoint_binding.rs"]
mod remote_endpoint_binding;

use assert_cmd::Command;
use nix_compat::narinfo::Flags;
use nix_compat::narinfo::NarInfo;
use nix_compat::narinfo::SigningKey;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::store_path::StorePathRef;
use serde_json::Value;
use sha2::Digest;
use sha2::Sha256;
use tempfile::TempDir;

const CHILD_OUTPUT: &str = "/nix/store/11111111111111111111111111111111-child";
const PARENT_OUTPUT: &str = "/nix/store/22222222222222222222222222222222-parent";
const BUILDER_SOURCE: &str = "/nix/store/33333333333333333333333333333333-foreign-builder";
const CHILD_CONTENT: &[u8] = b"child\n";
const NIX_PROFILE: &str = "config/foreign-execution-profiles/generated/nix.json";
const KEY_NAME: &str = "cache-inputs-test-1";
const TRUST_SCOPE: &str = "trusted-binary-cache";

fn mantle_cmd() -> Command {
    if let Ok(path) = std::env::var("MANTLE_TEST_BIN") {
        return Command::new(path);
    }
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

fn path_str(path: &Path) -> &str {
    path.to_str().expect("utf-8 fixture path")
}

fn json_file(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("artifact json")).expect("parse artifact json")
}

fn manifest_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn builder_script() -> Vec<u8> {
    fs::read(manifest_root().join("tests/fixtures/foreign-import/realize-two-node-builder.sh"))
        .expect("builder fixture")
}

fn nar_string(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(bytes);
    let padding = (8 - (bytes.len() % 8)) % 8;
    out.resize(out.len().saturating_add(padding), 0);
}

/// Minimal NAR for one regular file, optionally executable.
fn regular_file_nar(contents: &[u8], executable: bool) -> Vec<u8> {
    let mut out = Vec::new();
    nar_string(&mut out, b"nix-archive-1");
    nar_string(&mut out, b"(");
    nar_string(&mut out, b"type");
    nar_string(&mut out, b"regular");
    if executable {
        nar_string(&mut out, b"executable");
        nar_string(&mut out, b"");
    }
    nar_string(&mut out, b"contents");
    nar_string(&mut out, contents);
    nar_string(&mut out, b")");
    out
}

struct TestCache {
    _directory: TempDir,
    directory: PathBuf,
    url_with_key: String,
    base_url: String,
    requests: Arc<Mutex<Vec<String>>>,
    signer_secret: [u8; 32],
}

impl TestCache {
    fn start(entries: &[(&str, Vec<u8>, bool)]) -> Self {
        let cache_temp = tempfile::tempdir().expect("cache dir");
        let directory = cache_temp.path().to_path_buf();
        fs::create_dir_all(directory.join("nar")).expect("nar dir");
        fs::write(directory.join("nix-cache-info"), b"StoreDir: /nix/store\n").expect("cache info");
        let secret = [7u8; 32];
        let dalek = ed25519_dalek::SigningKey::from_bytes(&secret);
        let public = VerifyingKey::new(KEY_NAME.to_string(), dalek.verifying_key()).to_string();
        Self::write_entries(&directory, entries, &secret);
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind cache");
        let addr = listener.local_addr().expect("cache addr");
        let base_url = format!("http://{addr}");
        let encoded = public.replace('+', "%2B").replace('/', "%2F").replace('=', "%3D");
        let raw_url_with_key = format!("{base_url}/?trusted_public_keys[0]={encoded}");
        let url_with_key = url::Url::parse(&raw_url_with_key).expect("canonical cache url").to_string();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let served = Arc::clone(&requests);
        let root = directory.clone();
        std::thread::spawn(move || serve(listener, root, served));
        Self {
            _directory: cache_temp,
            directory,
            url_with_key,
            base_url,
            requests,
            signer_secret: secret,
        }
    }

    fn write_entries(directory: &Path, entries: &[(&str, Vec<u8>, bool)], secret: &[u8; 32]) {
        let dalek = ed25519_dalek::SigningKey::from_bytes(secret);
        let signer = SigningKey::new(KEY_NAME.to_string(), dalek);
        for (path, contents, executable) in entries {
            let nar = regular_file_nar(contents, *executable);
            let nar_hash: [u8; 32] = Sha256::digest(&nar).into();
            let nar_name = format!("{}.nar", hex_lower(&nar_hash));
            fs::write(directory.join("nar").join(&nar_name), &nar).expect("write nar");
            let url = format!("nar/{nar_name}");
            let store_path =
                StorePathRef::from_absolute_path_with_prefix(path.as_bytes(), "/nix/store").expect("valid store path");
            let mut narinfo = NarInfo {
                flags: Flags::empty(),
                store_path,
                nar_hash,
                nar_size: nar.len() as u64,
                references: Vec::new(),
                signatures: Vec::new(),
                ca: None,
                system: None,
                deriver: None,
                url: &url,
                compression: Some("none"),
                file_hash: None,
                file_size: None,
            };
            narinfo.add_signature_with_store_dir(&signer, "/nix/store");
            let text = narinfo.to_string_with_store_dir("/nix/store");
            let basename = path.rsplit('/').next().expect("basename");
            let digest = &basename[..32];
            fs::write(directory.join(format!("{digest}.narinfo")), text).expect("write narinfo");
        }
    }

    fn score_with_key(&self, entries: &[(&str, Vec<u8>, bool)], secret: &[u8; 32]) {
        Self::write_entries(&self.directory, entries, secret);
    }

    fn remove_entry(&self, path: &str) {
        let basename = path.rsplit('/').next().expect("basename");
        let _ = fs::remove_file(self.directory.join(format!("{}.narinfo", &basename[..32])));
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    data_encoding::HEXLOWER.encode(bytes)
}

fn serve(listener: TcpListener, root: PathBuf, requests: Arc<Mutex<Vec<String>>>) {
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
        let mut request_line = String::new();
        if reader.read_line(&mut request_line).is_err() {
            continue;
        }
        let path = request_line.split_whitespace().nth(1).unwrap_or("/").to_string();
        requests.lock().expect("request log").push(path.clone());
        let body = if path == "/nix-cache-info" {
            Some(fs::read(root.join("nix-cache-info")).unwrap_or_default())
        } else {
            let relative = path.trim_start_matches('/');
            let candidate = root.join(relative);
            if candidate.starts_with(&root) {
                fs::read(candidate).ok()
            } else {
                None
            }
        };
        match body {
            Some(body) => {
                let _ =
                    write!(stream, "HTTP/1.0 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                let _ = stream.write_all(&body);
            }
            None => {
                let _ = write!(stream, "HTTP/1.0 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            }
        }
    }
}

struct Fixture {
    plan: PathBuf,
    receipt: PathBuf,
    source_bundle: PathBuf,
    source_bundle_blake3: String,
    graph: PathBuf,
    index: PathBuf,
    policy: PathBuf,
}

fn write_fixture(temp: &TempDir, cache_url_with_key: &str, recompute: bool) -> Fixture {
    let fixture_dir = manifest_root().join("tests/fixtures/foreign-import");
    let mut graph: Value = json_file(&fixture_dir.join("realize-two-node.graph.json"));
    graph["nodes"][0]["cache_hints"] = serde_json::json!([{
        "cache_url": cache_url_with_key,
        "trust_scope": TRUST_SCOPE,
    }]);
    graph["nodes"][1]["cache_hints"] = serde_json::json!([{
        "cache_url": cache_url_with_key,
        "trust_scope": TRUST_SCOPE,
    }]);
    let grandchild = serde_json::json!({
        "node_id": "nix:grandchild",
        "original_derivation": "/nix/store/66666666666666666666666666666666-grandchild.drv",
        "name": "grandchild",
        "system": "x86_64-linux",
        "builder": "/bin/sh",
        "args": ["-c", "exit 1"],
        "env": { "out": "/nix/store/77777777777777777777777777777777-grandchild" },
        "outputs": {
            "out": { "path": "/nix/store/77777777777777777777777777777777-grandchild", "hash": null }
        },
        "input_derivations": [],
        "source_refs": [],
        "fixed_output": null,
        "builtin": "nix.derivation",
        "declared_references": [],
        "sandbox_capabilities": [],
        "unsupported_features": [],
        "cache_hints": []
    });
    graph["nodes"].as_array_mut().expect("nodes").push(grandchild);
    graph["nodes"][0]["input_derivations"] = serde_json::json!([{ "node_id": "nix:grandchild", "output_name": "out" }]);
    graph["nodes"][0]["env"]["grandchild"] =
        serde_json::json!("/nix/store/77777777777777777777777777777777-grandchild");
    graph["nodes"][0]["declared_references"] =
        serde_json::json!([BUILDER_SOURCE, "/nix/store/77777777777777777777777777777777-grandchild"]);
    let graph_path = temp.path().join("graph.json");
    fs::write(&graph_path, serde_json::to_vec(&graph).expect("graph json")).expect("write graph");
    let index_path = temp.path().join("index.json");
    fs::copy(fixture_dir.join("realize-two-node.index.json"), &index_path).expect("copy index");
    let mut policy: Value = json_file(&fixture_dir.join("realize-policy.json"));
    policy["source_prefixes"] = serde_json::json!(["/nix/store"]);
    policy["target_prefix"] = serde_json::json!("/nix/store");
    policy["output_path_recompute_mode"] = serde_json::json!(if recompute {
        "recompute-blake3-v1"
    } else {
        "preserve-cache-paths-v1"
    });
    policy["trusted_cache_scopes"] = serde_json::json!([TRUST_SCOPE]);
    let policy_path = temp.path().join("policy.json");
    fs::write(&policy_path, serde_json::to_vec(&policy).expect("policy json")).expect("write policy");
    Fixture {
        plan: temp.path().join("plan.json"),
        receipt: temp.path().join("import-receipt.json"),
        source_bundle: temp.path().join("source-bundle.json"),
        source_bundle_blake3: String::new(),
        graph: graph_path,
        index: index_path,
        policy: policy_path,
    }
}

fn write_binding(temp: &TempDir, base_url: &str, url_with_key: &str) -> PathBuf {
    let binding = temp.path().join("binding.json");
    remote_endpoint_binding::write_endpoint_bindings(
        &temp.path().join("endpoint-state"),
        &binding,
        remote_endpoint_binding::Bindings {
            store_prefix: "/nix/store",
            builder: None,
            substituters: &[url_with_key, base_url],
        },
    );
    binding
}

fn plan_route(fixture: &Fixture, realization_route: Option<&str>) -> std::process::Output {
    let mut command = mantle_cmd();
    command.args([
        "--json",
        "foreign-import",
        "plan",
        "--graph",
        path_str(&fixture.graph),
        "--package-index",
        path_str(&fixture.index),
        "--policy",
        path_str(&fixture.policy),
        "--package",
        "two-node",
        "--system",
        "x86_64-linux",
        "--execution-profile",
        path_str(&manifest_root().join(NIX_PROFILE)),
        "--plan-out",
        path_str(&fixture.plan),
        "--receipt-out",
        path_str(&fixture.receipt),
    ]);
    if let Some(route) = realization_route {
        command.args(["--realization-route", route]);
    }
    command.output().expect("run plan")
}

fn prepare_empty_sources(fixture: &Fixture) {
    mantle_cmd()
        .args([
            "--json",
            "foreign-import",
            "prepare-sources",
            "--plan",
            path_str(&fixture.plan),
            "--out",
            path_str(&fixture.source_bundle),
        ])
        .assert()
        .success();
}

fn realize(temp: &TempDir, fixture: &Fixture, binding: &Path) -> std::process::Output {
    let state_dir = temp.path().join("state");
    let output_dir = temp.path().join("output");
    fs::create_dir_all(&output_dir).expect("output dir");
    mantle_cmd()
        .args([
            "--json",
            "--state-dir",
            path_str(&state_dir),
            "--store",
            path_str(&output_dir),
            "--store-prefix",
            "/nix/store",
            "--remote-binding-config",
            path_str(binding),
            "foreign-import",
            "realize",
            "--plan",
            path_str(&fixture.plan),
            "--import-receipt",
            path_str(&fixture.receipt),
            "--source-bundle",
            path_str(&fixture.source_bundle),
            "--source-bundle-blake3",
            &fixture.source_bundle_blake3,
            "--execution-profile",
            path_str(&manifest_root().join(NIX_PROFILE)),
            "--cache-closure-policy",
            path_str(&write_policy(temp)),
            "--substitute",
            "--receipt-out",
            path_str(&temp.path().join("realization-receipt.json")),
        ])
        .output()
        .expect("run realize")
}

fn write_policy(temp: &TempDir) -> PathBuf {
    let path = temp.path().join("cache-closure-policy.json");
    fs::write(
        &path,
        serde_json::to_vec(&serde_json::json!({
            "schema": "mantle-foreign-cache-closure-policy-v1",
            "max_paths": 64,
            "max_references": 128,
            "max_nar_bytes": 1_048_576,
            "max_depth": 8,
        }))
        .expect("policy json"),
    )
    .expect("write closure policy");
    path
}

fn prepare_route(temp: &TempDir, cache: &TestCache) -> Fixture {
    let mut fixture = write_fixture(temp, &cache.url_with_key, false);
    let plan = plan_route(&fixture, Some("cache-inputs-build-roots-v1"));
    assert!(plan.status.success(), "plan must accept the route: {}", String::from_utf8_lossy(&plan.stderr));
    prepare_empty_sources(&fixture);
    let bundle = json_file(&fixture.source_bundle);
    fixture.source_bundle_blake3 = bundle["manifest_blake3"].as_str().expect("manifest digest").to_string();
    fixture
}

fn positive_cache() -> TestCache {
    TestCache::start(&[
        (CHILD_OUTPUT, CHILD_CONTENT.to_vec(), false),
        (BUILDER_SOURCE, builder_script(), true),
    ])
}

#[test]
fn cache_inputs_route_builds_the_root_from_hydrated_inputs() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    let fixture = prepare_route(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);
    let result = realize(&temp, &fixture, &binding);
    assert!(result.status.success(), "realize failed: {}", String::from_utf8_lossy(&result.stderr));
    let receipt = json_file(&temp.path().join("realization-receipt.json"));
    assert_eq!(receipt["status"], "complete");
    assert_eq!(receipt["strongest_state"], "realized");
    assert_eq!(receipt["cache_policy"]["realization_route"], "cache-inputs-build-roots-v1");
    let units = receipt["units"].as_array().expect("units");
    let parent = units.iter().find(|unit| unit["node_id"] == "nix:parent").expect("parent unit");
    let child = units.iter().find(|unit| unit["node_id"] == "nix:child").expect("child unit");
    assert_eq!(parent["execution_class"], "built");
    assert!(matches!(child["execution_class"].as_str(), Some("substituted" | "already-present")));
    assert!(!receipt["cache_closure"].as_array().expect("closure facts").is_empty());
    let grandchild = units.iter().find(|unit| unit["node_id"] == "nix:grandchild").expect("grandchild unit");
    assert_eq!(grandchild["execution_class"], "not-required-cache-only");
    let output = temp.path().join("output").join("22222222222222222222222222222222-parent");
    assert_eq!(fs::read_to_string(output).expect("built root output"), "child\n");
    let requests = cache.requests.lock().expect("requests");
    assert!(requests.iter().any(|path| path.ends_with(".narinfo")));
    assert!(requests.iter().any(|path| path.starts_with("/nar/")));
    assert!(!requests.iter().any(|path| path.contains("/action-results/")));
}

#[test]
fn cache_inputs_route_refuses_a_root_offered_by_the_cache() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    cache.score_with_key(&[(PARENT_OUTPUT, b"cached\n".to_vec(), false)], &cache.signer_secret);
    let fixture = prepare_route(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);
    let result = realize(&temp, &fixture, &binding);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("refuses a root offered by the cache"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!temp.path().join("realization-receipt.json").exists());
}

#[test]
fn cache_inputs_route_refuses_a_missing_input() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    cache.remove_entry(CHILD_OUTPUT);
    let fixture = prepare_route(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);
    let result = realize(&temp, &fixture, &binding);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("missing-narinfo")
            || String::from_utf8_lossy(&result.stderr).contains("http-closure"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!temp.path().join("realization-receipt.json").exists());
}

#[test]
fn cache_inputs_route_refuses_an_untrusted_signer() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    let mut other = [9u8; 32];
    other[0] = 9;
    cache.score_with_key(&[(CHILD_OUTPUT, CHILD_CONTENT.to_vec(), false)], &other);
    let fixture = prepare_route(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);
    let result = realize(&temp, &fixture, &binding);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("untrusted-narinfo"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!temp.path().join("realization-receipt.json").exists());
    assert!(!cache.requests.lock().expect("requests").iter().any(|path| path.starts_with("/nar/")));
}

#[test]
fn cache_inputs_route_is_refused_on_a_recompute_plan() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    let fixture = write_fixture(&temp, &cache.url_with_key, true);
    let plan = plan_route(&fixture, Some("cache-inputs-build-roots-v1"));
    assert!(!plan.status.success());
    let output = format!("{}{}", String::from_utf8_lossy(&plan.stdout), String::from_utf8_lossy(&plan.stderr));
    assert!(
        output.contains("cache-inputs-requires-preserve")
            || output.contains("foreign-plan-cache-inputs-requires-preserve"),
        "unexpected output: {output}"
    );
}

#[test]
fn cache_inputs_route_rejects_a_nonempty_source_bundle() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    let fixture = prepare_route(&temp, &cache);
    let mut bundle: Value = json_file(&fixture.source_bundle);
    bundle["manifest_blake3"] = serde_json::json!("0".repeat(64));
    fs::write(&fixture.source_bundle, serde_json::to_vec(&bundle).expect("bundle json")).expect("write bundle");
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);
    let result = realize(&temp, &fixture, &binding);
    assert!(!result.status.success());
    assert!(!temp.path().join("realization-receipt.json").exists());
    assert!(cache.directory.join("nar").is_dir());
}
