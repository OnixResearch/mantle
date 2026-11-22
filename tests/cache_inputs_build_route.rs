// r[verify foreign_derivation_import.cache_inputs_build_roots]
//! CLI qualification for the cache-inputs build route: a loopback signed Nix
//! cache supplies the selected root's direct input output and input source,
//! then Mantle executes the root in the sandbox.

use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
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
const CHILD_DEV_OUTPUT: &str = "/nix/store/88888888888888888888888888888888-child-dev";
const PARENT_OUTPUT: &str = "/nix/store/22222222222222222222222222222222-parent";
const CHANGED_PARENT_OUTPUT: &str = "/nix/store/99999999999999999999999999999999-parent";
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
    graph["nodes"][0]["outputs"]["dev"] = serde_json::json!({ "path": CHILD_DEV_OUTPUT, "hash": null });
    graph["nodes"][0]["env"]["dev"] = serde_json::json!(CHILD_DEV_OUTPUT);
    graph["nodes"][0]["declared_references"]
        .as_array_mut()
        .expect("declared references")
        .push(serde_json::json!(CHILD_DEV_OUTPUT));
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
    realize_with_receipt(temp, fixture, binding, "realization-receipt.json")
}

fn realize_with_receipt(temp: &TempDir, fixture: &Fixture, binding: &Path, receipt_name: &str) -> std::process::Output {
    realize_with_options(temp, fixture, binding, receipt_name, None, &[])
}

fn realize_with_options(
    temp: &TempDir,
    fixture: &Fixture,
    binding: &Path,
    receipt_name: &str,
    signing_key: Option<&Path>,
    trusted_builder_public_keys: &[String],
) -> std::process::Output {
    realize_command(temp, fixture, binding, receipt_name, signing_key, trusted_builder_public_keys)
        .output()
        .expect("run realize")
}

fn realize_command(
    temp: &TempDir,
    fixture: &Fixture,
    binding: &Path,
    receipt_name: &str,
    signing_key: Option<&Path>,
    trusted_builder_public_keys: &[String],
) -> Command {
    let mut command = mantle_cmd();
    command.args(realize_arguments(temp, fixture, binding, receipt_name, signing_key, trusted_builder_public_keys));
    command
}

fn mantle_binary() -> PathBuf {
    if let Ok(path) = std::env::var("MANTLE_TEST_BIN") {
        return PathBuf::from(path);
    }
    assert_cmd::cargo::cargo_bin("mantle")
}

fn realize_arguments(
    temp: &TempDir,
    fixture: &Fixture,
    binding: &Path,
    receipt_name: &str,
    signing_key: Option<&Path>,
    trusted_builder_public_keys: &[String],
) -> Vec<String> {
    let state_dir = temp.path().join("state");
    let output_dir = temp.path().join("output");
    let policy = write_policy(temp);
    fs::create_dir_all(&output_dir).expect("output dir");
    let mut arguments = vec![
        "--json".to_string(),
        "--state-dir".to_string(),
        state_dir.display().to_string(),
        "--store".to_string(),
        output_dir.display().to_string(),
        "--store-prefix".to_string(),
        "/nix/store".to_string(),
        "--remote-binding-config".to_string(),
        binding.display().to_string(),
        "foreign-import".to_string(),
        "realize".to_string(),
        "--plan".to_string(),
        fixture.plan.display().to_string(),
        "--import-receipt".to_string(),
        fixture.receipt.display().to_string(),
        "--source-bundle".to_string(),
        fixture.source_bundle.display().to_string(),
        "--source-bundle-blake3".to_string(),
        fixture.source_bundle_blake3.clone(),
        "--execution-profile".to_string(),
        manifest_root().join(NIX_PROFILE).display().to_string(),
        "--cache-closure-policy".to_string(),
        policy.display().to_string(),
        "--substitute".to_string(),
        "--receipt-out".to_string(),
        temp.path().join(receipt_name).display().to_string(),
    ];
    if let Some(signing_key) = signing_key {
        arguments.push("--signing-key".to_string());
        arguments.push(signing_key.display().to_string());
    }
    for key in trusted_builder_public_keys {
        arguments.push("--trusted-builder-public-key".to_string());
        arguments.push(key.clone());
    }
    arguments
}

fn write_nix_signing_key(path: &Path, secret: [u8; 32]) -> String {
    let dalek = ed25519_dalek::SigningKey::from_bytes(&secret);
    let mut combined = Vec::with_capacity(64);
    combined.extend_from_slice(&secret);
    combined.extend_from_slice(dalek.verifying_key().as_bytes());
    let line = format!("{KEY_NAME}:{}", data_encoding::BASE64.encode(&combined));
    fs::write(path, format!("{line}\n")).expect("write signing key");
    let mut permissions = fs::metadata(path).expect("signing key metadata").permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(path, permissions).expect("signing key mode");
    VerifyingKey::new(KEY_NAME.to_string(), dalek.verifying_key()).to_string()
}

fn path_signatures(temp: &TempDir, path: &str) -> Vec<String> {
    let state_dir = temp.path().join("state");
    let output_dir = temp.path().join("output");
    let output = mantle_cmd()
        .args([
            "--json",
            "--state-dir",
            path_str(&state_dir),
            "--store",
            path_str(&output_dir),
            "--store-prefix",
            "/nix/store",
            "store",
            "info",
            path.trim_start_matches("/nix/store/"),
        ])
        .output()
        .expect("run store info");
    assert!(output.status.success(), "store info failed: {}", String::from_utf8_lossy(&output.stderr));
    let value: Value = serde_json::from_slice(&output.stdout).expect("parse store info");
    value["paths"][0]["signatures"]
        .as_array()
        .expect("store info signatures")
        .iter()
        .map(|signature| signature.as_str().expect("signature").to_string())
        .collect()
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
        (CHILD_DEV_OUTPUT, b"child-dev\n".to_vec(), false),
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
    let closure_paths = receipt["cache_closure"]
        .as_array()
        .expect("closure facts")
        .iter()
        .map(|fact| fact["logical_path"].as_str().expect("logical path"))
        .collect::<std::collections::BTreeSet<_>>();
    assert!(!closure_paths.is_empty());
    assert!(closure_paths.contains(CHILD_OUTPUT));
    assert!(closure_paths.contains(CHILD_DEV_OUTPUT));
    assert!(child["outputs"].as_array().expect("child outputs").len() >= 2);
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

fn prepare_route_v2(temp: &TempDir, cache: &TestCache) -> Fixture {
    let mut fixture = write_fixture(temp, &cache.url_with_key, false);
    let plan = plan_route(&fixture, Some("cache-inputs-build-roots-v2"));
    assert!(plan.status.success(), "plan must accept the route: {}", String::from_utf8_lossy(&plan.stderr));
    prepare_empty_sources(&fixture);
    let bundle = json_file(&fixture.source_bundle);
    fixture.source_bundle_blake3 = bundle["manifest_blake3"].as_str().expect("manifest digest").to_string();
    fixture
}

fn built_root_index_path_from_receipt(temp: &TempDir, receipt: &Value) -> PathBuf {
    let derivation = receipt["roots"][0]["target_derivation"].as_str().expect("root derivation");
    let basename = derivation.rsplit('/').next().expect("root derivation basename");
    temp.path()
        .join("state")
        .join("foreign-realization")
        .join("built-roots")
        .join(format!("{basename}.json"))
}

fn prepare_changed_route_v2(temp: &TempDir, cache: &TestCache) -> Fixture {
    let mut fixture = write_fixture(temp, &cache.url_with_key, false);
    let mut graph = json_file(&fixture.graph);
    let parent = &mut graph["nodes"][1];
    parent["env"]["out"] = serde_json::json!(CHANGED_PARENT_OUTPUT);
    parent["env"]["variant"] = serde_json::json!("changed-input");
    parent["outputs"]["out"]["path"] = serde_json::json!(CHANGED_PARENT_OUTPUT);
    fs::write(&fixture.graph, serde_json::to_vec(&graph).expect("changed graph json")).expect("write changed graph");
    let plan = plan_route(&fixture, Some("cache-inputs-build-roots-v2"));
    assert!(
        plan.status.success(),
        "changed plan must accept route v2: {}",
        String::from_utf8_lossy(&plan.stderr)
    );
    prepare_empty_sources(&fixture);
    let bundle = json_file(&fixture.source_bundle);
    fixture.source_bundle_blake3 = bundle["manifest_blake3"].as_str().expect("manifest digest").to_string();
    fixture
}

#[test]
fn cache_inputs_build_or_reuse_route_reuses_an_unchanged_root() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    let fixture = prepare_route_v2(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);

    let first = realize_with_receipt(&temp, &fixture, &binding, "first-receipt.json");
    assert!(first.status.success(), "first realize failed: {}", String::from_utf8_lossy(&first.stderr));
    let built = json_file(&temp.path().join("first-receipt.json"));
    assert_eq!(built["schema"], "mantle-foreign-realization-receipt-v2");
    assert_eq!(built["cache_policy"]["realization_route"], "cache-inputs-build-roots-v2");
    let roots = built["roots"].as_array().expect("roots");
    assert_eq!(roots.len(), 1, "one selected root output: {roots:?}");
    assert_eq!(roots[0]["disposition"], "built");
    assert!(roots[0]["built"].is_object());
    assert!(roots[0]["reused"].is_null());
    assert_eq!(roots[0]["signer"]["role"], "builder");

    let derivation = roots[0]["target_derivation"].as_str().expect("root derivation");
    let basename = derivation.rsplit('/').next().expect("root derivation basename");
    let index_path = temp
        .path()
        .join("state")
        .join("foreign-realization")
        .join("built-roots")
        .join(format!("{basename}.json"));
    assert!(index_path.is_file(), "built-root index must be written at {}", index_path.display());
    let indexed = json_file(&index_path);
    assert_eq!(indexed["receipt_blake3"], built["receipt_blake3"]);

    let requests_before = cache.requests.lock().expect("requests").len();
    let second = realize_with_receipt(&temp, &fixture, &binding, "second-receipt.json");
    assert!(second.status.success(), "second realize failed: {}", String::from_utf8_lossy(&second.stderr));
    let reused = json_file(&temp.path().join("second-receipt.json"));
    let roots = reused["roots"].as_array().expect("roots");
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0]["disposition"], "reused");
    assert!(roots[0]["built"].is_null());
    assert_eq!(roots[0]["reused"]["source"], "local");
    assert_eq!(roots[0]["reused"]["builds"], 0);
    assert_eq!(roots[0]["reused"]["downloads"], 0);
    assert_eq!(roots[0]["reused"]["original_receipt_blake3"], built["receipt_blake3"]);
    assert_ne!(reused["receipt_blake3"], built["receipt_blake3"]);
    let parent = reused["units"]
        .as_array()
        .expect("units")
        .iter()
        .find(|unit| unit["node_id"] == "nix:parent")
        .expect("parent unit");
    assert_eq!(parent["execution_class"], "already-present");

    let requests = cache.requests.lock().expect("requests");
    assert_eq!(requests.len(), requests_before, "reuse must not consult the cache: {requests:?}");
    assert!(!requests[requests_before..].iter().any(|path| path.starts_with("/nar/")));
}

#[test]
fn cache_inputs_build_or_reuse_route_builds_a_changed_root_derivation() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    let fixture = prepare_route_v2(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);

    let first = realize_with_receipt(&temp, &fixture, &binding, "first-receipt.json");
    assert!(first.status.success(), "first realize failed: {}", String::from_utf8_lossy(&first.stderr));
    let built = json_file(&temp.path().join("first-receipt.json"));
    assert_eq!(built["roots"][0]["disposition"], "built");
    assert_eq!(built["roots"][0]["path"], PARENT_OUTPUT.trim_start_matches("/nix/store/"));

    let changed = prepare_changed_route_v2(&temp, &cache);
    let result = realize_with_receipt(&temp, &changed, &binding, "changed-receipt.json");
    assert!(result.status.success(), "changed realize failed: {}", String::from_utf8_lossy(&result.stderr));
    let receipt = json_file(&temp.path().join("changed-receipt.json"));
    assert_eq!(receipt["schema"], "mantle-foreign-realization-receipt-v2");
    assert_eq!(receipt["roots"][0]["disposition"], "built");
    assert_eq!(receipt["roots"][0]["path"], CHANGED_PARENT_OUTPUT.trim_start_matches("/nix/store/"));
    assert!(receipt["roots"][0]["built"].is_object());
    assert!(receipt["roots"][0]["reused"].is_null());
    assert_ne!(receipt["roots"][0]["target_derivation"], built["roots"][0]["target_derivation"]);
    let index = built_root_index_path_from_receipt(&temp, &receipt);
    assert!(index.is_file(), "changed root must be indexed at {}", index.display());
    assert_eq!(json_file(&index)["receipt_blake3"], receipt["receipt_blake3"]);
}

#[test]
fn cache_inputs_build_or_reuse_route_refuses_a_present_root_without_a_built_receipt() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    let v1 = prepare_route(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);
    let first = realize(&temp, &v1, &binding);
    assert!(first.status.success(), "v1 realize failed: {}", String::from_utf8_lossy(&first.stderr));
    let v1_receipt = json_file(&temp.path().join("realization-receipt.json"));
    assert_eq!(v1_receipt["schema"], "mantle-foreign-realization-receipt-v1");
    let signatures_before = path_signatures(&temp, PARENT_OUTPUT);

    // The same derivation is now present locally, built by route v1, so the v2 index has no entry.
    let fixture = prepare_route_v2(&temp, &cache);
    let result = realize_with_receipt(&temp, &fixture, &binding, "refused-receipt.json");
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("refusing to reuse present foreign root"), "unexpected stderr: {stderr}");
    assert!(stderr.contains("no-built-receipt"), "unexpected stderr: {stderr}");
    assert!(!temp.path().join("refused-receipt.json").exists());
    assert_eq!(path_signatures(&temp, PARENT_OUTPUT), signatures_before, "refusal must not mint a signature");
}

#[test]
fn cache_inputs_build_or_reuse_route_refuses_a_tampered_index_entry() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    let fixture = prepare_route_v2(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);
    let first = realize_with_receipt(&temp, &fixture, &binding, "first-receipt.json");
    assert!(first.status.success(), "first realize failed: {}", String::from_utf8_lossy(&first.stderr));
    let built = json_file(&temp.path().join("first-receipt.json"));
    let index = built_root_index_path_from_receipt(&temp, &built);
    let mut entry = json_file(&index);
    entry["trust_policy_blake3"] = serde_json::json!("0".repeat(64));
    fs::write(&index, serde_json::to_vec(&entry).expect("tampered index json")).expect("write tampered index");
    let signatures_before = path_signatures(&temp, PARENT_OUTPUT);

    let result = realize_with_receipt(&temp, &fixture, &binding, "refused-receipt.json");
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("receipt-invalid"), "unexpected stderr: {stderr}");
    assert!(!temp.path().join("refused-receipt.json").exists());
    assert_eq!(path_signatures(&temp, PARENT_OUTPUT), signatures_before, "refusal must not mint a signature");
    assert_eq!(json_file(&index)["trust_policy_blake3"], "0".repeat(64), "refusal must not rewrite the index");
}

#[test]
fn cache_inputs_build_or_reuse_route_rotated_signing_key_requires_the_retired_public_key() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    let fixture = prepare_route_v2(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);
    let old_key_path = temp.path().join("old-signing-key");
    let mut old_secret = [0u8; 32];
    old_secret[0] = 11;
    old_secret[31] = 13;
    let old_public = write_nix_signing_key(&old_key_path, old_secret);
    let new_key_path = temp.path().join("new-signing-key");
    let mut new_secret = [0u8; 32];
    new_secret[0] = 17;
    new_secret[31] = 19;
    write_nix_signing_key(&new_key_path, new_secret);

    let first = realize_with_options(&temp, &fixture, &binding, "first-receipt.json", Some(&old_key_path), &[]);
    assert!(first.status.success(), "first realize failed: {}", String::from_utf8_lossy(&first.stderr));
    let built = json_file(&temp.path().join("first-receipt.json"));
    assert_eq!(built["roots"][0]["disposition"], "built");

    let refused = realize_with_options(&temp, &fixture, &binding, "refused-receipt.json", Some(&new_key_path), &[]);
    assert!(!refused.status.success());
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(stderr.contains("signer-mismatch"), "unexpected stderr: {stderr}");
    assert!(!temp.path().join("refused-receipt.json").exists());

    let reused = realize_with_options(
        &temp,
        &fixture,
        &binding,
        "reused-receipt.json",
        Some(&new_key_path),
        std::slice::from_ref(&old_public),
    );
    assert!(reused.status.success(), "retired-key reuse failed: {}", String::from_utf8_lossy(&reused.stderr));
    let receipt = json_file(&temp.path().join("reused-receipt.json"));
    assert_eq!(receipt["roots"][0]["disposition"], "reused");
    assert_eq!(receipt["roots"][0]["reused"]["original_receipt_blake3"], built["receipt_blake3"]);
    assert_eq!(receipt["roots"][0]["signer"]["role"], "builder");
}

#[test]
fn cache_inputs_build_or_reuse_route_concurrent_realizations_build_once() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    let fixture = prepare_route_v2(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);

    let spawn = |receipt_name: &str| {
        let mut command = std::process::Command::new(mantle_binary());
        command.args(realize_arguments(&temp, &fixture, &binding, receipt_name, None, &[]));
        command.spawn().expect("spawn realize")
    };
    let mut first = spawn("concurrent-a.json");
    let mut second = spawn("concurrent-b.json");
    let first_status = first.wait().expect("wait first realize");
    let second_status = second.wait().expect("wait second realize");
    assert!(first_status.success(), "first concurrent realize failed");
    assert!(second_status.success(), "second concurrent realize failed");

    let left = json_file(&temp.path().join("concurrent-a.json"));
    let right = json_file(&temp.path().join("concurrent-b.json"));
    let mut dispositions = vec![
        left["roots"][0]["disposition"].as_str().expect("left disposition"),
        right["roots"][0]["disposition"].as_str().expect("right disposition"),
    ];
    dispositions.sort();
    assert_eq!(dispositions, ["built", "reused"], "one run must build and one must reuse");
    let (built, reused) = if left["roots"][0]["disposition"] == "built" {
        (left, right)
    } else {
        (right, left)
    };
    assert_eq!(reused["roots"][0]["reused"]["original_receipt_blake3"], built["receipt_blake3"]);
    assert_eq!(reused["roots"][0]["reused"]["builds"], 0);
    assert_eq!(reused["roots"][0]["reused"]["downloads"], 0);
}

#[test]
fn cache_inputs_build_or_reuse_route_refuses_a_root_offered_by_the_cache() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    cache.score_with_key(&[(PARENT_OUTPUT, b"cached\n".to_vec(), false)], &cache.signer_secret);
    let fixture = prepare_route_v2(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);
    let result = realize_with_receipt(&temp, &fixture, &binding, "refused-receipt.json");
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("refuses a root offered by the cache"), "unexpected stderr: {stderr}");
    assert!(!temp.path().join("refused-receipt.json").exists());
    assert!(!cache.requests.lock().expect("requests").iter().any(|path| path.starts_with("/nar/")));
}

#[test]
fn cache_inputs_build_roots_v1_still_refuses_a_present_root_and_keeps_receipt_v1() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    let fixture = prepare_route(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);
    let first = realize(&temp, &fixture, &binding);
    assert!(first.status.success(), "first v1 realize failed: {}", String::from_utf8_lossy(&first.stderr));
    let built = json_file(&temp.path().join("realization-receipt.json"));
    assert_eq!(built["schema"], "mantle-foreign-realization-receipt-v1");
    assert!(built.get("roots").is_none(), "a v1 receipt has no roots array");
    let signatures_before = path_signatures(&temp, PARENT_OUTPUT);

    let second = realize_with_receipt(&temp, &fixture, &binding, "second-receipt.json");
    assert!(!second.status.success());
    let stderr = String::from_utf8_lossy(&second.stderr);
    assert!(stderr.contains("refuses a root already present in the store"), "unexpected stderr: {stderr}");
    assert!(!temp.path().join("second-receipt.json").exists());
    assert_eq!(path_signatures(&temp, PARENT_OUTPUT), signatures_before, "refusal must not mint a signature");
}

fn realize_with_build_cores(
    temp: &TempDir,
    fixture: &Fixture,
    binding: &Path,
    receipt_name: &str,
    build_cores: Option<u32>,
) -> std::process::Output {
    let mut command = realize_command(temp, fixture, binding, receipt_name, None, &[]);
    if let Some(cores) = build_cores {
        command.args(["--build-cores", &cores.to_string()]);
    }
    command.output().expect("run realize with build cores")
}

fn prepare_build_cores_route_v2(temp: &TempDir, cache: &TestCache) -> Fixture {
    let mut fixture = write_fixture(temp, &cache.url_with_key, false);
    let mut graph = json_file(&fixture.graph);
    graph["nodes"][1]["args"] = serde_json::json!(["parent-cores"]);
    fs::write(&fixture.graph, serde_json::to_vec(&graph).expect("cores graph json")).expect("write cores graph");
    let plan = plan_route(&fixture, Some("cache-inputs-build-roots-v2"));
    assert!(plan.status.success(), "cores plan must accept route v2: {}", String::from_utf8_lossy(&plan.stderr));
    prepare_empty_sources(&fixture);
    let bundle = json_file(&fixture.source_bundle);
    fixture.source_bundle_blake3 = bundle["manifest_blake3"].as_str().expect("manifest digest").to_string();
    fixture
}

fn root_output_file(temp: &TempDir) -> String {
    fs::read_to_string(temp.path().join("output").join("22222222222222222222222222222222-parent"))
        .expect("built root output")
}

#[test]
fn cache_inputs_build_or_reuse_route_sets_and_records_build_cores() {
    let cache = positive_cache();

    let explicit_dir = TempDir::new().expect("tempdir");
    let fixture = prepare_build_cores_route_v2(&explicit_dir, &cache);
    let binding = write_binding(&explicit_dir, &cache.base_url, &cache.url_with_key);
    let explicit = realize_with_build_cores(&explicit_dir, &fixture, &binding, "cores-four.json", Some(4));
    assert!(explicit.status.success(), "cores realize failed: {}", String::from_utf8_lossy(&explicit.stderr));
    let receipt = json_file(&explicit_dir.path().join("cores-four.json"));
    assert_eq!(receipt["roots"][0]["disposition"], "built");
    assert_eq!(receipt["roots"][0]["built"]["build_cores"], 4);
    assert_eq!(root_output_file(&explicit_dir), "4\n");

    // A fresh store, so the default run builds the root instead of reusing the one built above.
    let default_dir = TempDir::new().expect("tempdir");
    let fixture = prepare_build_cores_route_v2(&default_dir, &cache);
    let binding = write_binding(&default_dir, &cache.base_url, &cache.url_with_key);
    let default = realize_with_build_cores(&default_dir, &fixture, &binding, "cores-default.json", None);
    assert!(
        default.status.success(),
        "default cores realize failed: {}",
        String::from_utf8_lossy(&default.stderr)
    );
    let receipt = json_file(&default_dir.path().join("cores-default.json"));
    assert_eq!(receipt["roots"][0]["disposition"], "built");
    // The default leaves the build environment and the receipt exactly as before the option existed.
    assert!(receipt["roots"][0]["built"].get("build_cores").is_none());
    assert_eq!(root_output_file(&default_dir), "unset\n");
}

#[test]
fn cache_inputs_build_or_reuse_route_refuses_build_cores_on_route_v1() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    let fixture = prepare_route(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);

    let refused = realize_with_build_cores(&temp, &fixture, &binding, "cores-v1.json", Some(4));
    assert!(!refused.status.success());
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(stderr.contains("no field for --build-cores 4"), "unexpected stderr: {stderr}");
    assert!(!temp.path().join("cores-v1.json").exists());
}

#[test]
fn cache_inputs_build_or_reuse_route_rejects_out_of_range_build_cores() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    let fixture = prepare_route_v2(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);

    for cores in [0u32, 65u32] {
        let refused = realize_with_build_cores(&temp, &fixture, &binding, "cores-range.json", Some(cores));
        assert!(!refused.status.success(), "{cores} cores must be refused");
        let stderr = String::from_utf8_lossy(&refused.stderr);
        assert!(stderr.contains("build-cores"), "unexpected stderr: {stderr}");
        assert!(!temp.path().join("cores-range.json").exists());
    }
}

/// A receipt v2 written before `built.build_cores` existed; the digest is recomputed without the
/// field so the index parser accepts it.
#[derive(serde::Serialize)]
struct LegacySigner {
    key_name: String,
    public_key_blake3: String,
    role: String,
}

#[derive(serde::Serialize)]
struct LegacyBuilt {
    mantle_version: String,
    execution_profile_digest_blake3: String,
    input_closure_blake3: String,
}

#[derive(serde::Serialize)]
struct LegacyRoot {
    node_id: String,
    target_derivation: String,
    output_name: String,
    path: String,
    nar_sha256: String,
    nar_size: u64,
    references: Vec<String>,
    disposition: String,
    signer: LegacySigner,
    trust_policy_blake3: String,
    built: LegacyBuilt,
    reused: Option<serde_json::Value>,
}

#[derive(serde::Serialize)]
struct LegacyCachePolicy {
    substitution_enabled: bool,
    offline: bool,
    ordered_cache_urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    realization_route: Option<String>,
}

#[derive(serde::Serialize)]
struct LegacyReceiptV2 {
    schema: String,
    status: String,
    strongest_state: String,
    receipt_blake3: String,
    plan_blake3: String,
    import_receipt_blake3: String,
    source_bundle_manifest_blake3: String,
    build_report_blake3: String,
    selected_root_node_ids: Vec<String>,
    selected_root_paths: Vec<String>,
    execution_profiles: Vec<serde_json::Value>,
    cache_policy: LegacyCachePolicy,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_closure_policy_blake3: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cache_closure: Vec<serde_json::Value>,
    sources: Vec<serde_json::Value>,
    units: Vec<serde_json::Value>,
    roots: Vec<LegacyRoot>,
    failure: Option<serde_json::Value>,
    non_claims: Vec<String>,
}

#[test]
fn cache_inputs_build_or_reuse_route_reuses_an_index_receipt_without_build_cores() {
    let temp = TempDir::new().expect("tempdir");
    let cache = positive_cache();
    let fixture = prepare_route_v2(&temp, &cache);
    let binding = write_binding(&temp, &cache.base_url, &cache.url_with_key);
    let first = realize_with_receipt(&temp, &fixture, &binding, "first-receipt.json");
    assert!(first.status.success(), "first realize failed: {}", String::from_utf8_lossy(&first.stderr));
    let built = json_file(&temp.path().join("first-receipt.json"));
    let root = &built["roots"][0];
    let mut legacy = LegacyReceiptV2 {
        schema: built["schema"].as_str().expect("schema").to_string(),
        status: "complete".to_string(),
        strongest_state: "realized".to_string(),
        receipt_blake3: String::new(),
        plan_blake3: built["plan_blake3"].as_str().expect("plan digest").to_string(),
        import_receipt_blake3: built["import_receipt_blake3"].as_str().expect("import digest").to_string(),
        source_bundle_manifest_blake3: built["source_bundle_manifest_blake3"]
            .as_str()
            .expect("source bundle digest")
            .to_string(),
        build_report_blake3: built["build_report_blake3"].as_str().expect("build report digest").to_string(),
        selected_root_node_ids: vec![root["node_id"].as_str().expect("node id").to_string()],
        selected_root_paths: vec![root["path"].as_str().expect("root path").to_string()],
        execution_profiles: Vec::new(),
        cache_policy: LegacyCachePolicy {
            substitution_enabled: true,
            offline: false,
            ordered_cache_urls: Vec::new(),
            realization_route: Some("cache-inputs-build-roots-v2".to_string()),
        },
        cache_closure_policy_blake3: None,
        cache_closure: Vec::new(),
        sources: Vec::new(),
        units: Vec::new(),
        roots: vec![LegacyRoot {
            node_id: root["node_id"].as_str().expect("node id").to_string(),
            target_derivation: root["target_derivation"].as_str().expect("derivation").to_string(),
            output_name: root["output_name"].as_str().expect("output name").to_string(),
            path: root["path"].as_str().expect("path").to_string(),
            nar_sha256: root["nar_sha256"].as_str().expect("nar sha").to_string(),
            nar_size: root["nar_size"].as_u64().expect("nar size"),
            references: Vec::new(),
            disposition: "built".to_string(),
            signer: LegacySigner {
                key_name: root["signer"]["key_name"].as_str().expect("key name").to_string(),
                public_key_blake3: root["signer"]["public_key_blake3"]
                    .as_str()
                    .expect("public key fingerprint")
                    .to_string(),
                role: "builder".to_string(),
            },
            trust_policy_blake3: root["trust_policy_blake3"].as_str().expect("trust policy").to_string(),
            built: LegacyBuilt {
                mantle_version: root["built"]["mantle_version"].as_str().expect("mantle version").to_string(),
                execution_profile_digest_blake3: root["built"]["execution_profile_digest_blake3"]
                    .as_str()
                    .expect("profile digest")
                    .to_string(),
                input_closure_blake3: root["built"]["input_closure_blake3"]
                    .as_str()
                    .expect("closure digest")
                    .to_string(),
            },
            reused: None,
        }],
        failure: None,
        non_claims: Vec::new(),
    };
    let canonical = serde_json::to_vec(&legacy).expect("legacy receipt bytes");
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-foreign-realization-receipt-v2\0");
    hasher.update(&canonical);
    let digest = hasher.finalize().to_hex().to_string();
    legacy.receipt_blake3 = digest.clone();
    let index_path = built_root_index_path_from_receipt(&temp, &built);
    fs::write(&index_path, serde_json::to_vec_pretty(&legacy).expect("legacy index bytes"))
        .expect("write legacy index");

    let reused = realize_with_receipt(&temp, &fixture, &binding, "reused-receipt.json");
    assert!(reused.status.success(), "reuse failed: {}", String::from_utf8_lossy(&reused.stderr));
    let receipt = json_file(&temp.path().join("reused-receipt.json"));
    assert_eq!(receipt["roots"][0]["disposition"], "reused");
    assert_eq!(receipt["roots"][0]["reused"]["original_receipt_blake3"], digest);
}
