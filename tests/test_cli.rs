use std::collections::BTreeMap;
use std::path::Path;

use assert_cmd::Command;
use crunch_store::StoreBackend;
use crunch_store::StoreConfig;
use crunch_store::StoreFallbackMode;
use crunch_store::StoreHandle;
use crunch_store::VerifiedSourceIngestRequest;
use crunch_test_core::COHORT;
use crunch_test_core::Resources;
use crunch_test_core::TARGET_SCHEMA;
use crunch_test_core::Target;
use crunch_test_core::Tool;
use nix_compat::store_path::DIGEST_SIZE;
use nix_compat::store_path::StorePath;
use tempfile::TempDir;

fn cmd_for(bin: &str) -> Command {
    Command::cargo_bin(bin).expect("binary should be built")
}

fn hex(ch: char) -> String {
    std::iter::repeat_n(ch, 64).collect()
}

fn write_private_signing_key(path: &Path, line: &str) {
    std::fs::write(path, format!("{line}\n")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

fn sample_target(store_prefix: &str, logical_input: &str, input_blake3: &str) -> Target {
    let tool = Tool {
        root: logical_input.into(),
        member: "bin/test".into(),
        blake3: hex('1'),
    };
    Target {
        schema: TARGET_SCHEMA.into(),
        name: "cli-integration".into(),
        cohort: COHORT.into(),
        rustc_verbose_version: "rustc 1.90.0 (1159e78c4 2025-09-14)\nLLVM version: 20.1.8\n".into(),
        store_prefix: store_prefix.into(),
        platform: "x86_64-linux".into(),
        executable: tool.clone(),
        runner: tool,
        inputs: BTreeMap::from([(logical_input.into(), input_blake3.into())]),
        runfiles: BTreeMap::new(),
        environment: BTreeMap::new(),
        tests: vec!["tests::smoke".into()],
        resources: Resources {
            memory_bytes: 67_108_864,
            output_bytes: 65_536,
            timeout_ms: 5_000,
        },
        deterministic: true,
        external_services: false,
        coverage: None,
    }
}

#[test]
fn test_subcommand_help_parity_across_mantle_and_crunch() {
    let mantle_help = cmd_for("mantle").args(["test", "--help"]).output().unwrap();
    let crunch_help = cmd_for("crunch").args(["test", "--help"]).output().unwrap();
    assert!(mantle_help.status.success());
    assert!(crunch_help.status.success());
    let mantle_text = String::from_utf8(mantle_help.stdout).unwrap();
    let crunch_text = String::from_utf8(crunch_help.stdout).unwrap();
    for flag in [
        "--filter",
        "--exact",
        "--timeout-ms",
        "--shard-index",
        "--shard-count",
        "--seed",
        "--repeat",
        "--force-rerun",
        "--coverage",
        "--signing-key",
        "--trusted-public-key",
    ] {
        assert!(mantle_text.contains(flag), "mantle help missing {flag}:\n{mantle_text}");
        assert!(crunch_text.contains(flag), "crunch help missing {flag}:\n{crunch_text}");
    }
}

#[test]
fn test_subcommand_rejects_unknown_flag_with_clap_exit_code_2() {
    for bin in ["mantle", "crunch"] {
        let output = cmd_for(bin).args(["test", "--unknown-test-flag", "target.json"]).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "{bin} stderr: {}", String::from_utf8_lossy(&output.stderr));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn test_subcommand_emits_json_error_envelope_on_missing_target_for_both_binaries() {
    for bin in ["mantle", "crunch"] {
        let dir = TempDir::new().unwrap();
        let state = dir.path().join("state");
        let store = dir.path().join("store");
        let missing = dir.path().join("does-not-exist.json");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::create_dir_all(&store).unwrap();

        let output = cmd_for(bin)
            .env("CRUNCH_CONFIG_DIR", &state)
            .args([
                "--json",
                "--store",
                store.to_str().unwrap(),
                "--state-dir",
                state.to_str().unwrap(),
                "test",
                missing.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(3), "{bin} stderr: {}", String::from_utf8_lossy(&output.stderr));
        assert!(output.stdout.is_empty(), "{bin} unexpected stdout: {}", String::from_utf8_lossy(&output.stdout));
        let envelope: serde_json::Value = serde_json::from_slice(&output.stderr).expect("valid JSON error envelope");
        assert_eq!(envelope["kind"], "internal");
        assert!(envelope["error"].as_str().is_some_and(|m| !m.is_empty()));
    }
}

#[test]
fn test_subcommand_rejects_invalid_target_and_invalid_invocation() {
    let dir = TempDir::new().unwrap();
    let state = dir.path().join("state");
    let store = dir.path().join("store");
    std::fs::create_dir_all(&state).unwrap();
    std::fs::create_dir_all(&store).unwrap();

    let logical = StorePath::<String>::from_name_and_digest_fixed("invalid-target-input", [0x11; DIGEST_SIZE])
        .unwrap()
        .to_absolute_path_with_prefix("/mantle/store");
    let mut bad_target = sample_target("/mantle/store", &logical, &hex('a'));
    bad_target.deterministic = false;
    let bad_target_path = dir.path().join("nondeterministic.json");
    std::fs::write(&bad_target_path, serde_json::to_vec(&bad_target).unwrap()).unwrap();

    let output = cmd_for("mantle")
        .env("CRUNCH_CONFIG_DIR", &state)
        .args([
            "--store",
            store.to_str().unwrap(),
            "--state-dir",
            state.to_str().unwrap(),
            "test",
            bad_target_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("test-nondeterministic-or-external-service-unsupported"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let good_target = sample_target("/mantle/store", &logical, &hex('a'));
    let good_target_path = dir.path().join("valid.json");
    std::fs::write(&good_target_path, serde_json::to_vec(&good_target).unwrap()).unwrap();
    let bad_invocation = cmd_for("crunch")
        .env("CRUNCH_CONFIG_DIR", &state)
        .args([
            "--store",
            store.to_str().unwrap(),
            "--state-dir",
            state.to_str().unwrap(),
            "test",
            good_target_path.to_str().unwrap(),
            "--exact",
        ])
        .output()
        .unwrap();
    assert_eq!(bad_invocation.status.code(), Some(3));
    assert!(
        String::from_utf8_lossy(&bad_invocation.stderr).contains("test-filter-invalid"),
        "stderr: {}",
        String::from_utf8_lossy(&bad_invocation.stderr)
    );
}

#[test]
fn test_subcommand_rejects_store_prefix_mismatch_and_unsigned_store_root() {
    let dir = TempDir::new().unwrap();
    let state = dir.path().join("state");
    let store = dir.path().join("store");
    std::fs::create_dir_all(&state).unwrap();
    std::fs::create_dir_all(&store).unwrap();

    let (keypair, key_line) = crunch_build::signing::generate_keypair();
    let key_path = dir.path().join("signing-key");
    write_private_signing_key(&key_path, &key_line);

    let nix_logical = StorePath::<String>::from_name_and_digest_fixed("prefix-mismatch", [0x22; DIGEST_SIZE])
        .unwrap()
        .to_absolute_path_with_prefix("/nix/store");
    let target_nix_prefix = sample_target("/nix/store", &nix_logical, &hex('b'));
    let mismatch_path = dir.path().join("prefix-mismatch.json");
    std::fs::write(&mismatch_path, serde_json::to_vec(&target_nix_prefix).unwrap()).unwrap();

    let mismatch_out = cmd_for("mantle")
        .env("CRUNCH_CONFIG_DIR", &state)
        .args([
            "--store",
            store.to_str().unwrap(),
            "--state-dir",
            state.to_str().unwrap(),
            "test",
            mismatch_path.to_str().unwrap(),
            "--signing-key",
            key_path.to_str().unwrap(),
            "--trusted-public-key",
            &keypair.verifying_key.to_string(),
        ])
        .output()
        .unwrap();
    assert_eq!(mismatch_out.status.code(), Some(3));
    assert!(
        String::from_utf8_lossy(&mismatch_out.stderr).contains("store prefix differs"),
        "stderr: {}",
        String::from_utf8_lossy(&mismatch_out.stderr)
    );

    let mantle_logical = StorePath::<String>::from_name_and_digest_fixed("unsigned-root", [0x33; DIGEST_SIZE])
        .unwrap()
        .to_absolute_path_with_prefix("/mantle/store");
    let target_unsigned = sample_target("/mantle/store", &mantle_logical, &hex('c'));
    let unsigned_path = dir.path().join("unsigned.json");
    std::fs::write(&unsigned_path, serde_json::to_vec(&target_unsigned).unwrap()).unwrap();

    let unsigned_out = cmd_for("crunch")
        .env("CRUNCH_CONFIG_DIR", &state)
        .args([
            "--store",
            store.to_str().unwrap(),
            "--state-dir",
            state.to_str().unwrap(),
            "test",
            unsigned_path.to_str().unwrap(),
            "--signing-key",
            key_path.to_str().unwrap(),
            "--trusted-public-key",
            &keypair.verifying_key.to_string(),
        ])
        .output()
        .unwrap();
    assert_eq!(unsigned_out.status.code(), Some(3));
    assert!(unsigned_out.stdout.is_empty());
}

#[test]
fn test_subcommand_rejects_signed_input_nar_blake3_mismatch() {
    let dir = TempDir::new().unwrap();
    let state = dir.path().join("state");
    let store = dir.path().join("store");
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&state).unwrap();
    std::fs::create_dir_all(&store).unwrap();
    std::fs::create_dir_all(src_dir.join("bin")).unwrap();
    std::fs::write(src_dir.join("bin/test"), b"#!/bin/sh\nexit 0\n").unwrap();

    let (keypair, key_line) = crunch_build::signing::generate_keypair();
    let key_path = dir.path().join("signing-key");
    write_private_signing_key(&key_path, &key_line);

    let store_path: StorePath<String> =
        StorePath::from_name_and_digest_fixed("signed-input", [0x44; DIGEST_SIZE]).unwrap();
    let logical = store_path.to_absolute_path_with_prefix("/mantle/store");

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut handle = StoreHandle::open(StoreConfig {
            source_admission: None,
            backend: StoreBackend::Snix,
            state_dir: state.clone(),
            output_dir: store.clone(),
            remote_cache_urls: Vec::new(),
            fallback_mode: StoreFallbackMode::Practical,
            store_dir: "/mantle/store".into(),
            base_state_dirs: Vec::new(),
        })
        .await
        .unwrap();
        handle
            .ingest_verified_source(VerifiedSourceIngestRequest {
                source_path: &src_dir,
                logical_store_path: &logical,
                source_name: "signed-input",
                signing_key: &keypair.signing_key,
            })
            .await
            .unwrap();
    });

    let exported_root = store.join(store_path.to_string());
    std::fs::create_dir_all(exported_root.join("bin")).unwrap();
    std::fs::write(exported_root.join("bin/test"), b"#!/bin/sh\nexit 0\n").unwrap();

    let target = sample_target("/mantle/store", &logical, &hex('d'));
    let target_path = dir.path().join("mismatch-nar.json");
    std::fs::write(&target_path, serde_json::to_vec(&target).unwrap()).unwrap();

    let output = cmd_for("mantle")
        .env("CRUNCH_CONFIG_DIR", &state)
        .args([
            "--store",
            store.to_str().unwrap(),
            "--state-dir",
            state.to_str().unwrap(),
            "test",
            target_path.to_str().unwrap(),
            "--signing-key",
            key_path.to_str().unwrap(),
            "--trusted-public-key",
            &keypair.verifying_key.to_string(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("NAR BLAKE3 mismatch"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
