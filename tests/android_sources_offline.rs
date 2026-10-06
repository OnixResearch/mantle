use std::path::PathBuf;

use crunch_build::FetchBuildService;
use crunch_build::FetchSourceOverride;
use crunch_build::FetchSourceOverrideKind;
use crunch_build::FetchSourcePolicy;
use crunch_eval::evaluate_str;
use crunch_store::StoreBackend;
use crunch_store::StoreConfig;
use crunch_store::StoreHandle;
use snix_build::buildservice::BuildRequest;
use snix_build::buildservice::BuildService;
use snix_build::buildservice::EnvVar;
use snix_castore::Node;

fn request(url: &str) -> BuildRequest {
    BuildRequest {
        command_args: vec!["builtin:fetchurl".to_string()],
        outputs: vec![PathBuf::from("nix/store/android-source")],
        environment_vars: vec![EnvVar {
            key: "url".to_string(),
            value: url.to_string().into(),
        }],
        ..BuildRequest::default()
    }
}

#[tokio::test]
async fn android_pinned_fetch_request_replays_only_matching_kind_and_url() {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib");
    let source = evaluate_str(
        r#"let mantle = import "lib.ncl" in
           let [jdk, _, _, _] = mantle.AndroidSources.cohort in
           mantle.AndroidSources.source jdk"#,
        &[stdlib.into_os_string()],
    )
    .unwrap();
    let derivation: crunch_glue::CrunchDerivation = source.to_serde().unwrap();
    assert_eq!(derivation.builder, "builtin:fetchurl");
    assert_eq!(derivation.fixed_output.as_ref().unwrap().mode, "flat");
    let official_url = derivation.env.get("url").unwrap();

    let root = tempfile::tempdir().unwrap();
    // This is a source-override routing fixture. The payload is deliberately
    // not a JDK archive; post-fetch SHA-256 verification is a separate stage.
    let payload = root.path().join("jdk-replayed-from-file.tar.gz");
    let file_bytes = b"android-source-file-replay-fixture\n";
    std::fs::write(&payload, file_bytes).unwrap();
    let store = StoreHandle::open(StoreConfig::new(
        StoreBackend::Snix,
        root.path().join("state"),
        root.path().join("store"),
        "/mantle/store".to_string(),
    ))
    .await
    .unwrap();
    let replay = FetchSourceOverride {
        url: official_url.clone(),
        kind: FetchSourceOverrideKind::File,
        rev: None,
        payload_path: payload.clone(),
        source_state_blake3: blake3::hash(b"android-offline-fixture-state").to_hex().to_string(),
    };
    let service = FetchBuildService::new(store.into_pipeline_store_parts().build_service_store)
        .with_source_overrides(vec![replay.clone()])
        .with_source_policy(FetchSourcePolicy::RequireOverride);
    let result = service.do_build(request(official_url)).await.unwrap();
    match &result.outputs[0].node {
        Node::File { digest, size, .. } => {
            assert_eq!(*digest, blake3::hash(file_bytes).into());
            assert_eq!(*size, file_bytes.len() as u64);
        }
        other => panic!("Android archive replay must be a flat file: {other:?}"),
    }

    let unmatched_url = format!("file://{}", root.path().join("unmatched.tar.gz").display());
    let failure = service.do_build(request(&unmatched_url)).await.unwrap_err();
    assert!(failure.to_string().contains("offline source policy rejected unmatched builtin fetch"));
    assert!(failure.to_string().contains(&unmatched_url));

    let second_store = StoreHandle::open(StoreConfig::new(
        StoreBackend::Snix,
        root.path().join("other-state"),
        root.path().join("other-store"),
        "/mantle/store".to_string(),
    ))
    .await
    .unwrap();
    let wrong_kind = FetchBuildService::new(second_store.into_pipeline_store_parts().build_service_store)
        .with_source_overrides(vec![FetchSourceOverride {
            kind: FetchSourceOverrideKind::Tarball,
            ..replay
        }])
        .with_source_policy(FetchSourcePolicy::RequireOverride);
    let failure = wrong_kind.do_build(request(official_url)).await.unwrap_err();
    assert!(failure.to_string().contains("offline source policy rejected unmatched builtin fetch"));
    assert!(failure.to_string().contains(official_url));
}

fn android_consumer_file(path: &std::path::Path) {
    std::fs::write(
        path,
        r#"let android = (import "lib.ncl").AndroidSources in
let [jdk, _, _, _] = android.cohort in
android.bind {
  identity = jdk,
  observed_sha256 = jdk.sha256,
  derivation = {
    name = "android-bound-tool-consumer",
    builder = "/bin/sh",
    args = ["-c", "printf ADMISSION_TOOL_EXECUTED > $out"],
  },
}
"#,
    )
    .unwrap();
}

fn pipeline_config(
    path: PathBuf,
    output_dir: &std::path::Path,
    state_dir: &std::path::Path,
) -> crunch_pipeline::BuildConfig {
    let keypair = crunch_build::load_keypair(
        "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
    )
    .unwrap();
    let trusted_keys = crunch_build::build_trusted_keys(&keypair, None);
    crunch_pipeline::BuildConfig {
        file: path,
        import_paths: vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()],
        output_dir: output_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        backend: StoreBackend::Snix,
        base_state_dirs: Vec::new(),
        store_dir: "/nix/store".to_string(),
        verbose: false,
        max_jobs: 2,
        scheduling_policy: crunch_pipeline::SchedulingPolicy::default(),
        substituter_urls: Vec::new(),
        hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
        keypair,
        trusted_keys,
        trust_unsigned: false,
        root_retention_source: None,
        root_registration: None,
        source_fetch_overrides: Vec::new(),
        remote_enabled: false,
        interchange_dir: None,
    }
}

const JDK_URL: &str = "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.17%2B10/OpenJDK17U-jdk_x64_linux_hotspot_17.0.17_10.tar.gz";
const JDK_SHA256: &str = "sha256-mS+W55lQdax2NrsajeUrDGHXHtMTf6/JeauWtKt43XU=";

fn override_file(path: &std::path::Path) -> FetchSourceOverride {
    FetchSourceOverride {
        url: JDK_URL.to_string(),
        kind: FetchSourceOverrideKind::File,
        rev: None,
        payload_path: path.to_path_buf(),
        source_state_blake3: blake3::hash(b"android-source-proof-fixture").to_hex().to_string(),
    }
}

#[tokio::test]
async fn bound_android_tool_cannot_execute_after_real_fixed_output_digest_drift() {
    let work = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let input = work.path().join("jdk-local-file-fixture.tar.gz");
    let root = work.path().join("android-consumer.ncl");
    let bytes = b"wrong prebuilt JDK bytes; never execute the bound consumer";
    std::fs::write(&input, bytes).unwrap();
    android_consumer_file(&root);
    let mut config = pipeline_config(root, output.path(), state.path());
    config.source_fetch_overrides = vec![override_file(&input)];

    let result = crunch_pipeline::build(&config).await.unwrap();
    assert!(result.outcomes.is_empty(), "a failed source must not produce a tool output");
    let mismatch = result
        .failed
        .iter()
        .find_map(|failure| crunch_pipeline::parse_fod_mismatch_error(&failure.origin_error))
        .unwrap_or_else(|| panic!("real fixed-output rejection missing in root-cause failures: {:?}", result.failed));
    let actual_sha256 = {
        use sha2::Digest;
        format!(
            "sha256-{}",
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, sha2::Sha256::digest(bytes))
        )
    };
    assert_eq!(mismatch.name, "android-jdk-17.0.17+10");
    assert_eq!(mismatch.expected_sri, JDK_SHA256);
    assert_eq!(mismatch.actual_sri, actual_sha256);
    let diagnostic = result.failed.iter().map(|failure| failure.origin_error.as_str()).collect::<Vec<_>>().join("\n");
    for required in ["android-jdk-17.0.17+10", JDK_SHA256, &actual_sha256] {
        assert!(diagnostic.contains(required), "FOD rejection omitted {required}: {diagnostic}");
    }
    assert!(
        std::fs::read_dir(output.path()).unwrap().next().is_none(),
        "tool-execution output marker must remain absent when source SHA-256 drifts"
    );

    let unmatched_output = tempfile::tempdir().unwrap();
    let unmatched_state = tempfile::tempdir().unwrap();
    let mut unmatched =
        pipeline_config(work.path().join("android-consumer.ncl"), unmatched_output.path(), unmatched_state.path());
    unmatched.source_fetch_overrides = vec![FetchSourceOverride {
        url: format!("file://{}", work.path().join("other-jdk.tar.gz").display()),
        ..override_file(&input)
    }];
    let blocked = crunch_pipeline::build(&unmatched).await.unwrap();
    assert!(blocked.outcomes.is_empty(), "offline unmatched URL must not execute a bound tool");
    assert!(blocked.fod_mismatches.is_empty(), "no unmatched source bytes should reach hash verification");
    assert!(
        blocked
            .failed
            .iter()
            .any(|failure| failure.origin_error.contains("offline source policy rejected unmatched builtin fetch")),
        "unmatched source must fail before acquisition: {:?}",
        blocked.failed
    );
    assert!(std::fs::read_dir(unmatched_output.path()).unwrap().next().is_none());
}
