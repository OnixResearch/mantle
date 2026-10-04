// Copy this file to tests/backend_baseline.rs in historical commit
// 7ec5177718a6950297e04eb4eb957a10b02e23ce; run only this named test.
// The included historical test fixture pins the signing key in TEST_KEYPAIR.
include!("store_archive_cli.rs");

#[test]
fn capture_prechange_snix_golden() {
    fn run_cli(state: &Path, output_dir: &Path, args: &[&str]) -> String {
        let output = mantle_cmd()
            .arg("--state-dir")
            .arg(state)
            .arg("--store")
            .arg(output_dir)
            .arg("--store-prefix")
            .arg(TEST_STORE_PREFIX)
            .arg("--json")
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8(output.stdout).unwrap()
    }

    assert_eq!(std::env::var("LANG").unwrap(), "C");
    assert_eq!(std::env::var("LC_ALL").unwrap(), "C");
    assert_eq!(std::env::var("TZ").unwrap(), "UTC");
    assert_eq!(std::env::var("SOURCE_DATE_EPOCH").unwrap(), "1600000000");
    assert!(std::env::var_os("CRUNCH_CONFIG_DIR").is_none());
    assert!(std::env::var_os("MANTLE_STORE_BACKEND").is_none());
    let root = std::path::PathBuf::from(std::env::var("MANTLE_BASELINE_FIXTURE_ROOT").unwrap());
    assert!(root.is_absolute() && !root.exists(), "fixture root must be a fresh absolute path");
    std::fs::create_dir(&root).unwrap();
    let state = root.join("state");
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(state.join("signing-key"), TEST_KEYPAIR).unwrap();
    let output_dir = root.join("store");
    let kept = run_async(seed_signed_file(&output_dir, &state, "baseline-keep", b"kept NAR\n", true));
    let removed = run_async(seed_signed_file(&output_dir, &state, "baseline-candidate", b"candidate NAR\n", true));
    let retained = kept.store_path.to_absolute_path_with_prefix(TEST_STORE_PREFIX);
    let legacy_root = serde_json::json!({
        retained.clone(): {
            "logical_path": retained,
            "source": "build",
            "created_unix_s": 100,
        }
    });
    std::fs::write(state.join("gc-roots.json"), serde_json::to_vec(&legacy_root).unwrap()).unwrap();
    let fresh_state = root.join("fresh-state");
    let fresh_store = root.join("fresh-store");
    std::fs::create_dir_all(&fresh_state).unwrap();
    std::fs::write(fresh_state.join("signing-key"), TEST_KEYPAIR).unwrap();
    let fresh_listing = run_cli(&fresh_state, &fresh_store, &["store", "list"]);
    let fresh_identity = std::fs::read(fresh_state.join("store-identity.json")).unwrap();
    let value = serde_json::json!({
        "historical_revision": "7ec5177718a6950297e04eb4eb957a10b02e23ce",
        "physical_fixture_root": root,
        "signer_public_key": trusted_public_key(),
        "store_prefix": TEST_STORE_PREFIX,
        "kept_store_path": kept.store_path.to_string(),
        "candidate_store_path": removed.store_path.to_string(),
        "kept_nar_sha256": data_encoding::HEXLOWER.encode(&kept.nar_sha256),
        "candidate_nar_sha256": data_encoding::HEXLOWER.encode(&removed.nar_sha256),
        "environment": {
            "LANG": "C",
            "LC_ALL": "C",
            "TZ": "UTC",
            "SOURCE_DATE_EPOCH": "1600000000",
            "CRUNCH_CONFIG_DIR": "unset",
            "MANTLE_STORE_BACKEND": "unset"
        },
        "kept_signed_pathinfo_json_hex": data_encoding::HEXLOWER.encode(&serde_json::to_vec(&kept).unwrap()),
        "candidate_signed_pathinfo_json_hex": data_encoding::HEXLOWER.encode(&serde_json::to_vec(&removed).unwrap()),
        "fresh_store_identity_json_hex": data_encoding::HEXLOWER.encode(&fresh_identity),
        "fresh_store_list_stdout": fresh_listing,
        "store_info_stdout": run_cli(&state, &output_dir, &["store", "info", kept.store_path.name()]),
        "store_roots_stdout": run_cli(&state, &output_dir, &["store", "roots"]),
        "store_gc_dry_run_stdout": run_cli(&state, &output_dir, &["store", "gc", "--dry-run"]),
    });
    println!("PRECHANGE_SNIX_GOLDEN={}", serde_json::to_string(&value).unwrap());
    std::fs::remove_dir_all(&root).unwrap();
}
