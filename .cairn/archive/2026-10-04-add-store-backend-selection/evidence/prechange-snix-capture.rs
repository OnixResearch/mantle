// Copy this file to tests/backend_baseline.rs in historical commit
// 7ec5177718a6950297e04eb4eb957a10b02e23ce; run only this named test.
// The included historical test fixture pins the signing key in TEST_KEYPAIR.
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/store_archive_cli.rs"));

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

fn rail_historical_cmd(state: &Path, output: &Path) -> Command {
    let mut cmd = mantle_cmd();
    cmd.args(["--store-prefix", TEST_STORE_PREFIX])
        .arg("--state-dir")
        .arg(state)
        .arg("--store")
        .arg(output);
    cmd
}

fn rail_historical_json(state: &Path, output: &Path, args: &[&str]) -> Value {
    let output = rail_historical_cmd(state, output).arg("--json").args(args).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).unwrap()
}

fn rail_historical_seed(root: &Path) -> (PathInfo, PathInfo, PathInfo) {
    let state = root.join("state");
    let output = root.join("output");
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(state.join("signing-key"), TEST_KEYPAIR).unwrap();
    let raw = ed25519_dalek::SigningKey::from_bytes(&[19_u8; 32]);
    let mut keypair = Vec::from(raw.to_bytes());
    keypair.extend_from_slice(raw.verifying_key().as_bytes());
    std::fs::write(
        root.join("second-test-only-signing-key"),
        format!("archive-cli-2:{}", data_encoding::BASE64.encode(&keypair)),
    )
    .unwrap();
    run_async(async {
        let mut store = StoreHandle::open(StoreConfig::new(state, output.clone(), TEST_STORE_PREFIX.to_string()))
            .await
            .unwrap();
        let child = signed_file_pathinfo(&store, "rail-child", b"referenced NAR\n", true).await;
        let candidate = signed_file_pathinfo(&store, "rail-candidate", b"unretained NAR\n", true).await;
        let mut retained = signed_file_pathinfo(&store, "rail-retained", b"retained NAR\n", false).await;
        retained.references.push(child.store_path.clone());
        sign_pathinfo(&mut retained);
        for (info, bytes) in [
            (&child, b"referenced NAR\n".as_slice()),
            (&candidate, b"unretained NAR\n".as_slice()),
            (&retained, b"retained NAR\n".as_slice()),
        ] {
            store.pathinfo_service().put(info.clone()).await.unwrap();
            assert_eq!(store.export_cached_path_info(&info.store_path).await.unwrap(), Some(info.clone()));
            assert_eq!(std::fs::read(output.join(info.store_path.to_string())).unwrap(), bytes);
        }
        (retained, child, candidate)
    })
}

fn rail_historical_reopen(root: &Path, retained: &PathInfo, child: &PathInfo) -> Value {
    run_async(async {
        let store = StoreHandle::open(StoreConfig::new(
            root.join("state"),
            root.join("output"),
            TEST_STORE_PREFIX.to_string(),
        ))
        .await
        .unwrap();
        let reopened = store.pathinfo_service().get(*retained.store_path.digest()).await.unwrap().unwrap();
        assert_eq!(reopened, *retained);
        let ports = store.into_builder_store_parts();
        let closure = ports
            .build_store
            .resolve_closure(&retained.store_path, StoreFallbackMode::Strict)
            .await
            .unwrap();
        assert!(closure.audit_events.is_empty());
        assert_eq!(closure.paths, vec![retained.store_path.clone(), child.store_path.clone()]);
        let action = crunch_action_result_core::ActionResultRecord {
            schema: crunch_action_result_core::ACTION_RESULT_SCHEMA.to_string(),
            result_ref: String::new(),
            action_ref: String::new(),
            outputs: vec![crunch_action_result_core::ActionResultOutput {
                name: "out".to_string(),
                object_ref: String::new(),
                store_path: retained.store_path.to_absolute_path_with_prefix(TEST_STORE_PREFIX),
                path_info_ref: String::new(),
            }],
            action_receipt_ref: String::new(),
            reference_scan_refs: Vec::new(),
            sandbox_policy_ref: String::new(),
            network_policy_ref: String::new(),
            producer_identity: String::new(),
            producer_policy_ref: String::new(),
            signature_refs: Vec::new(),
            publication_policy_ref: String::new(),
            non_claims: Vec::new(),
        };
        let reused = ports.action_results.probe_outputs(&action).await.unwrap();
        assert_eq!(reused.outputs.get("out"), Some(retained));
        assert_eq!(reused.reused_nar_bytes, retained.nar_size);
        serde_json::json!({
            "reopened_signed_pathinfo_hex": data_encoding::HEXLOWER.encode(&serde_json::to_vec(&reopened).unwrap()),
            "closure_paths": closure.paths.iter().map(ToString::to_string).collect::<Vec<_>>(),
            "reused_nar_bytes": reused.reused_nar_bytes,
            "reused_output_signed_pathinfo_hex": data_encoding::HEXLOWER.encode(&serde_json::to_vec(reused.outputs.get("out").unwrap()).unwrap()),
        })
    })
}

fn rail_historical_archive(root: &Path, retained: &PathInfo, child: &PathInfo, candidate: &PathInfo) -> Value {
    let state = root.join("state");
    let output = root.join("output");
    let imported_state = root.join("imported-state");
    let imported_output = root.join("imported-output");
    let archive = root.join("closure.mnar");
    let fixture_key = trusted_public_key();
    let alternate_raw = ed25519_dalek::SigningKey::from_bytes(&[19_u8; 32]);
    let alternate_key =
        nix_compat::narinfo::VerifyingKey::new("archive-cli-2".to_string(), alternate_raw.verifying_key()).to_string();
    for info in [retained, child, candidate] {
        rail_historical_cmd(&state, &output)
            .args(["store", "verify", "--trusted-public-keys", &fixture_key, info.store_path.name()])
            .assert()
            .success()
            .stdout(predicate::str::contains("trusted_signatures=1/1"));
    }
    rail_historical_cmd(&state, &output)
        .args(["store", "archive", "export", "--to"])
        .arg(&archive)
        .arg(retained.store_path.name())
        .assert()
        .success();
    let listed = rail_historical_json(&state, &output, &["store", "archive", "list", "--from", archive.to_str().unwrap()]);
    std::fs::create_dir_all(&imported_state).unwrap();
    std::fs::write(imported_state.join("signing-key"), TEST_KEYPAIR).unwrap();
    rail_historical_cmd(&imported_state, &imported_output)
        .args(["store", "archive", "import", "--from"])
        .arg(&archive)
        .args(["--trusted-public-keys", &fixture_key])
        .assert()
        .success();
    let imported_retained =
        rail_historical_json(&imported_state, &imported_output, &["store", "info", retained.store_path.name()]);
    let imported_child =
        rail_historical_json(&imported_state, &imported_output, &["store", "info", child.store_path.name()]);
    rail_historical_cmd(&imported_state, &imported_output)
        .args(["store", "sign", retained.store_path.name(), "--signing-key"])
        .arg(root.join("second-test-only-signing-key"))
        .assert()
        .success();
    let resigned = rail_historical_json(&imported_state, &imported_output, &["store", "info", retained.store_path.name()]);
    rail_historical_cmd(&imported_state, &imported_output)
        .args(["store", "verify", "--trusted-public-keys", &fixture_key, "--trusted-public-keys", &alternate_key, retained.store_path.name()])
        .assert()
        .success()
        .stdout(predicate::str::contains("trusted_signatures=2/2"));
    serde_json::json!({
        "listed_paths": listed["paths"].as_array().unwrap().iter()
            .map(|path| path["store_path"].as_str().unwrap()).collect::<Vec<_>>(),
        "imported_retained": imported_retained["paths"][0],
        "imported_child": imported_child["paths"][0],
        "resigned_retained": resigned["paths"][0],
        "verified_signatures_after_sign": 2,
    })
}

fn rail_historical_gc(root: &Path, retained: &PathInfo, child: &PathInfo, candidate: &PathInfo) -> Value {
    let state = root.join("state");
    let output = root.join("output");
    let retained_logical = retained.store_path.to_absolute_path_with_prefix(TEST_STORE_PREFIX);
    let candidate_logical = candidate.store_path.to_absolute_path_with_prefix(TEST_STORE_PREFIX);
    let legacy_root = serde_json::json!({
        retained_logical.clone(): {
            "logical_path": retained_logical,
            "source": "build",
            "created_unix_s": 100,
        }
    });
    std::fs::write(state.join("gc-roots.json"), serde_json::to_vec(&legacy_root).unwrap()).unwrap();
    let plan = rail_historical_json(&state, &output, &["store", "gc", "--dry-run"]);
    assert_eq!(plan["candidate_paths"], serde_json::json!([candidate_logical.clone()]));
    rail_historical_cmd(&state, &output).args(["store", "pin", &candidate_logical]).assert().success();
    rail_historical_cmd(&state, &output)
        .args(["store", "gc", "--execute", "--plan-id", plan["plan_id"].as_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("stale-gc-plan"));
    assert!(output.join(candidate.store_path.to_string()).exists());
    let after_pin = rail_historical_json(&state, &output, &["store", "gc", "--dry-run"]);
    assert_eq!(after_pin["candidate_paths"], serde_json::json!([]));
    rail_historical_cmd(&state, &output).args(["store", "unpin", &candidate_logical]).assert().success();
    let fresh = rail_historical_json(&state, &output, &["store", "gc", "--dry-run"]);
    assert_eq!(fresh["candidate_paths"], serde_json::json!([candidate_logical]));
    let execution = rail_historical_json(
        &state,
        &output,
        &["store", "gc", "--execute", "--plan-id", fresh["plan_id"].as_str().unwrap()],
    );
    assert_eq!(execution["execution_complete"], true);
    assert_eq!(execution["failed_operations"], serde_json::json!([]));
    assert_eq!(
        std::fs::symlink_metadata(output.join(candidate.store_path.to_string())).unwrap_err().kind(),
        std::io::ErrorKind::NotFound,
    );
    for info in [retained, child] {
        let observed = rail_historical_json(&state, &output, &["store", "info", info.store_path.name()]);
        assert_eq!(observed["paths"][0]["nar_sha256"], data_encoding::HEXLOWER.encode(&info.nar_sha256));
        rail_historical_cmd(&state, &output)
            .args(["store", "verify", "--trusted-public-keys", &trusted_public_key(), info.store_path.name()])
            .assert()
            .success()
            .stdout(predicate::str::contains("trusted_signatures=1/1"));
    }
    serde_json::json!({
        "planned": plan,
        "after_pin_candidate_paths": after_pin["candidate_paths"],
        "fresh": fresh,
        "execution_complete": execution["execution_complete"],
        "execution_candidate_paths": execution["candidate_paths"],
    })
}

#[test]
fn capture_prechange_snix_rail_golden() {
    assert_eq!(std::env::var("LANG").unwrap(), "C");
    assert_eq!(std::env::var("LC_ALL").unwrap(), "C");
    assert_eq!(std::env::var("TZ").unwrap(), "UTC");
    assert_eq!(std::env::var("SOURCE_DATE_EPOCH").unwrap(), "1600000000");
    assert!(std::env::var_os("CRUNCH_CONFIG_DIR").is_none());
    assert!(std::env::var_os("MANTLE_STORE_BACKEND").is_none());
    let root = PathBuf::from(std::env::var("MANTLE_RAIL_FIXTURE_ROOT").unwrap());
    let golden_output = PathBuf::from(std::env::var("MANTLE_RAIL_GOLDEN_OUTPUT").unwrap());
    assert!(root.is_absolute() && !root.exists(), "rail fixture root must be a fresh absolute path");
    assert!(golden_output.is_absolute() && !golden_output.exists(), "supplemental golden must not replace old data");
    std::fs::create_dir(&root).unwrap();
    let (retained, child, candidate) = rail_historical_seed(&root);
    let reopened = rail_historical_reopen(&root, &retained, &child);
    let archive = rail_historical_archive(&root, &retained, &child, &candidate);
    let gc = rail_historical_gc(&root, &retained, &child, &candidate);
    let paths = [(&retained, b"retained NAR\n".as_slice()), (&child, b"referenced NAR\n".as_slice()), (&candidate, b"unretained NAR\n".as_slice())]
        .map(|(info, content)| serde_json::json!({
            "store_path": info.store_path.to_string(),
            "nar_sha256": data_encoding::HEXLOWER.encode(&info.nar_sha256),
            "nar_size": info.nar_size,
            "signed_pathinfo_json_hex": data_encoding::HEXLOWER.encode(&serde_json::to_vec(info).unwrap()),
            "exported_content_hex": data_encoding::HEXLOWER.encode(content),
        }));
    let golden = serde_json::json!({
        "historical_revision": "7ec5177718a6950297e04eb4eb957a10b02e23ce",
        "physical_fixture_root": root,
        "store_prefix": TEST_STORE_PREFIX,
        "signer_public_key": trusted_public_key(),
        "paths": paths,
        "reopen": reopened,
        "archive": archive,
        "gc": gc,
    });
    std::fs::write(&golden_output, format!("{}\n", serde_json::to_string(&golden).unwrap())).unwrap();
    println!("PRECHANGE_SNIX_RAIL_GOLDEN={}", golden_output.display());
    std::fs::remove_dir_all(&root).unwrap();
}
