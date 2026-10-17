use std::path::Path;
use std::process::Command;
use std::process::Output;
use std::sync::Arc;
use std::sync::Barrier;

use crunch_store::StoreBackend;
use crunch_store::StoreConfig;
use crunch_store::StoreFallbackMode;
use crunch_store::StoreHandle;
use nix_compat::store_path::StorePath;
use snix_castore::Node;
use snix_castore::SymlinkTarget;
use snix_store::path_info::PathInfo;

fn run(state: &Path, output: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mantle"))
        .arg("--state-dir")
        .arg(state)
        .arg("--store")
        .arg(output)
        .arg("--nix-compat")
        .args(args)
        .output()
        .unwrap()
}

fn seed_path(state: &Path, output: &Path) -> String {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let store = StoreHandle::open(StoreConfig {
            backend: StoreBackend::Snix,
            state_dir: state.to_path_buf(),
            output_dir: output.to_path_buf(),
            remote_cache_urls: Vec::new(),
            base_state_dirs: Vec::new(),
            fallback_mode: StoreFallbackMode::Practical,
            store_dir: "/nix/store".to_string(),
        })
        .await
        .unwrap();
        let path = StorePath::from_name_and_digest_fixed("retention-two-owners", [42_u8; 20]).unwrap();
        let (keypair, _) = crunch_build::generate_keypair();
        let mut info = PathInfo {
            store_path: path.clone(),
            node: Node::Symlink {
                target: SymlinkTarget::try_from("retention-target").unwrap(),
            },
            references: Vec::new(),
            nar_size: 1,
            nar_sha256: [3_u8; 32],
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        };
        crunch_build::sign_pathinfo(&mut info, &keypair.signing_key);
        store.pathinfo_service().put(info).await.unwrap();
        path.to_absolute_path()
    })
}

fn json(output: &Output) -> serde_json::Value {
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn concurrent_cli_owners_remain_distinct_and_release_is_scoped() {
    let state = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    let logical = seed_path(state.path(), output.path());
    let barrier = Arc::new(Barrier::new(2));
    let jobs = [("alice", "interactive"), ("bob", "ci")].map(|(owner, reason)| {
        let barrier = Arc::clone(&barrier);
        let state = state.path().to_path_buf();
        let output = output.path().to_path_buf();
        let logical = logical.clone();
        std::thread::spawn(move || {
            barrier.wait();
            let result = run(&state, &output, &["store", "pin", &logical, "--owner", owner, "--reason", reason]);
            assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        })
    });
    for job in jobs {
        job.join().unwrap();
    }

    let roots = json(&run(state.path(), output.path(), &["--json", "store", "roots"]));
    assert_eq!(roots["schema"], "mantle-store-roots-v1");
    assert_eq!(roots["roots"].as_array().unwrap().len(), 1);
    let facts = &roots["retention_interests"][0];
    assert_eq!(facts["record_count"], 2);
    assert_eq!(facts["interests"][0]["owner"], "operator:alice");
    assert_eq!(facts["interests"][1]["owner"], "operator:bob");
    let record_id = facts["interests"][0]["record_id"].as_str().unwrap();
    let persisted = std::fs::read(state.path().join("retention-interests").join(format!("{record_id}.json"))).unwrap();
    assert_eq!(data_encoding::HEXLOWER.encode(blake3::hash(&persisted).as_bytes()), record_id);

    let foreign = run(state.path(), output.path(), &["store", "unpin", &logical, "--owner", "intruder"]);
    assert!(!foreign.status.success());
    assert!(String::from_utf8_lossy(&foreign.stderr).contains("foreign-owner"));
    let intact = json(&run(state.path(), output.path(), &["--json", "store", "roots"]));
    assert_eq!(intact["retention_interests"][0]["record_count"], 2);

    let removed = run(state.path(), output.path(), &[
        "store",
        "unpin",
        &logical,
        "--owner",
        "alice",
        "--reason",
        "interactive",
    ]);
    assert!(removed.status.success(), "{}", String::from_utf8_lossy(&removed.stderr));
    let remaining = json(&run(state.path(), output.path(), &["--json", "store", "roots"]));
    assert_eq!(remaining["retention_interests"][0]["record_count"], 1);
    assert_eq!(remaining["retention_interests"][0]["interests"][0]["owner"], "operator:bob");
    assert_eq!(remaining["roots"].as_array().unwrap().len(), 1);

    let info = json(&run(state.path(), output.path(), &["--json", "store", "info", "retention-two-owners"]));
    assert_eq!(info["schema"], "mantle-store-info-v3");
    assert_eq!(info["paths"][0]["retention_interests"]["record_count"], 1);
    assert_eq!(info["paths"][0]["retention_interests"]["interests"][0]["reason"], "ci");
    let usage = json(&run(state.path(), output.path(), &["--json", "store", "usage"]));
    assert_eq!(usage["schema"], "mantle-store-usage-v1");
    assert_eq!(usage["retention_interests"][0]["record_count"], 1);
}
