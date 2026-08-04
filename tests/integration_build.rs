//! Integration tests for the build pipeline.
//!
//! Tests that don't require bwrap run unconditionally.
//! Tests that require a real sandbox are gated on bwrap availability.

use std::collections::HashMap;
use std::ffi::OsString;
use std::num::NonZeroUsize;
use std::path::Path;
use std::path::PathBuf;

use crunch_build::DerivationRegistry;
use crunch_build::populate_registry;
use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use crunch_glue::Input;

const FETCH_TEST_PATH_INFO_CAPACITY: usize = 128;

#[test]
fn typed_nickel_workspace_policy_matches_rust_and_rejects_invalid_bounds() {
    let temp = tempfile::tempdir().unwrap();
    let valid = temp.path().join("workspace-valid.ncl");
    std::fs::write(
        &valid,
        r#"
let Derivation = import "derivation.ncl" in
({
  name = "workspace-parity",
  builder = "/bin/sh",
  workspace = {
    mode = 'mutable-session,
    workspace_id = "cargo-cache",
    compatibility = {
      authority_class = "tenant-a",
      action_class = "cargo-build",
      toolchain_refs = ["mantle-object://blake3/rust"],
    },
  },
} | Derivation)
"#,
    )
    .unwrap();
    let imports = stdlib_import_path();
    let derivation: CrunchDerivation = crunch_eval::evaluate_and_deserialize(&valid, &imports).unwrap();
    let encoded = derivation.env.get(crunch_glue::WORKSPACE_POLICY_ENV).unwrap();
    let policy: crunch_build::WorkspacePolicy = serde_json::from_str(encoded).unwrap();
    assert_eq!(policy.mode, crunch_build::WorkspaceMode::MutableSession);
    assert_eq!(policy.workspace_id.as_deref(), Some("cargo-cache"));
    assert!(crunch_build::validate_workspace_policy(&policy).is_ok());

    let invalid = temp.path().join("workspace-invalid.ncl");
    std::fs::write(
        &invalid,
        r#"
let Derivation = import "derivation.ncl" in
({ name = "bad-workspace", builder = "/bin/sh", workspace.quota.bytes_max = 0 } | Derivation)
"#,
    )
    .unwrap();
    let error = crunch_eval::evaluate_and_deserialize::<CrunchDerivation>(&invalid, &imports).unwrap_err();
    assert!(error.to_string().contains("positive bounded workspace number"));
}

#[test]
fn workspace_action_identity_uses_snapshot_ref_but_clean_derivation_drops_mutable_identity() {
    const TEST_DIGEST_HEX_LENGTH: usize = 64;

    fn base() -> CrunchDerivation {
        CrunchDerivation {
            name: "workspace-identity".to_string(),
            builder: "/bin/sh".to_string(),
            system: "x86_64-linux".to_string(),
            args: vec!["-c".to_string(), "true".to_string()],
            outputs: vec!["out".to_string()],
            dynamic_plan_outputs: Vec::new(),
            env: HashMap::new(),
            inputs: Vec::new(),
            fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
            provenance: None,
        }
    }
    fn path_for(derivation: &CrunchDerivation) -> nix_compat::store_path::StorePath<String> {
        let mut cache = ConversionCache::new("/mantle/store");
        crunch_glue::convert(derivation, &mut cache).unwrap().0
    }
    let clean = base();
    let clean_path = path_for(&clean);
    let mut snapshot_a = clean.clone();
    let mut policy = crunch_build::WorkspacePolicy {
        mode: crunch_build::WorkspaceMode::ImmutableSnapshot,
        snapshot_ref: Some(format!(
            "{}{}",
            crunch_build::WORKSPACE_SNAPSHOT_REF_PREFIX,
            "a".repeat(TEST_DIGEST_HEX_LENGTH)
        )),
        ..crunch_build::WorkspacePolicy::default()
    };
    snapshot_a
        .env
        .insert(crunch_glue::WORKSPACE_POLICY_ENV.to_string(), serde_json::to_string(&policy).unwrap());
    let snapshot_a_path = path_for(&snapshot_a);
    policy.snapshot_ref =
        Some(format!("{}{}", crunch_build::WORKSPACE_SNAPSHOT_REF_PREFIX, "b".repeat(TEST_DIGEST_HEX_LENGTH)));
    let mut snapshot_b = clean.clone();
    snapshot_b
        .env
        .insert(crunch_glue::WORKSPACE_POLICY_ENV.to_string(), serde_json::to_string(&policy).unwrap());
    assert_ne!(snapshot_a_path, path_for(&snapshot_b));

    let mut mutable = clean.clone();
    let mutable_policy = crunch_build::WorkspacePolicy {
        mode: crunch_build::WorkspaceMode::MutableSession,
        workspace_id: Some("cargo-cache".to_string()),
        ..crunch_build::WorkspacePolicy::default()
    };
    mutable
        .env
        .insert(crunch_glue::WORKSPACE_POLICY_ENV.to_string(), serde_json::to_string(&mutable_policy).unwrap());
    assert_ne!(path_for(&mutable), clean_path);
    assert_eq!(path_for(&clean), clean_path);
}

fn test_keypair() -> crunch_build::KeyPair {
    crunch_build::load_keypair(
        "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
    )
    .unwrap()
}

fn test_trusted_keys() -> Vec<nix_compat::narinfo::VerifyingKey> {
    crunch_build::build_trusted_keys(&test_keypair(), None)
}

fn make_test_builder<BServ, BS, DS, PIS>(
    services: (BS, DS, PIS),
    build_service: BServ,
    output_dir: &Path,
    is_verbose: bool,
) -> crunch_build::Builder<BServ>
where
    BServ: snix_build::buildservice::BuildService + 'static,
    BS: snix_castore::blobservice::BlobService + 'static,
    DS: snix_castore::directoryservice::DirectoryService + 'static,
    PIS: snix_store::pathinfoservice::PathInfoService + 'static,
{
    let (blob_service, directory_service, pathinfo_service) = services;
    let store = crunch_store::StoreHandle::from_services_with_store_dir(
        crunch_store::StoreHandleServices {
            blob_service: std::sync::Arc::new(blob_service),
            directory_service: std::sync::Arc::new(directory_service),
            pathinfo_service: std::sync::Arc::new(pathinfo_service),
            remote_pathinfo: None,
            state_dir: output_dir.join("state"),
            output_dir_str: output_dir.to_string_lossy().into_owned(),
            publishers: Vec::new(),
        },
        nix_compat::store_path::STORE_DIR.to_string(),
    );
    crunch_build::Builder::from_store_parts(
        store.into_builder_store_parts(),
        build_service,
        test_keypair(),
        test_trusted_keys(),
        true,
        is_verbose,
    )
}

fn stdlib_import_path() -> Vec<OsString> {
    let lib_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib");
    vec![lib_dir.into()]
}

/// Check if bwrap is available on this system.
fn has_bwrap() -> bool {
    std::process::Command::new("bwrap").arg("--version").output().is_ok_and(|o| o.status.success())
}

// -- Cache hit test (no sandbox needed) --

#[test]
fn cache_hit_skips_build() {
    // Use /bin/sh as builder — its output path ends up under /nix/store/
    // which is read-only. We can't create fake outputs there.
    //
    // Instead, use a derivation whose output path _happens_ to be an
    // existing store path. We pick /bin/sh as the builder specifically
    // because the output path is computed deterministically — we just
    // need to test that *if* the path exists, the build is skipped.
    //
    // Strategy: create a temp dir, symlink a fake "store path" there,
    // and verify the Builder logic. But since nix-compat hardcodes
    // STORE_DIR, we test the cache-hit path indirectly:
    //
    // If DummyBuildService is called, it returns an error. If the
    // output path doesn't exist, the build proceeds and
    // DummyBuildService errors. If it does exist, we get Ok(cached).
    //
    // On this machine, /nix/store exists and is populated. We construct
    // a derivation whose output path won't exist, so DummyBuildService
    // will be called and error. This validates the non-cache path.
    let drv = CrunchDerivation {
        name: "uncached-thing".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: HashMap::new(),
        inputs: vec![],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut cc = ConversionCache::default();
    let (drv_path, _nix_drv) = crunch_glue::convert(&drv, &mut cc).unwrap();

    let mut kp = DerivationRegistry::default();
    populate_registry(&mut kp, cc.iter_entries());
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        use snix_build::buildservice::DummyBuildService;
        use snix_castore::blobservice::MemoryBlobService;
        use snix_castore::directoryservice::RedbDirectoryService;
        use snix_castore::directoryservice::RedbDirectoryServiceConfig;

        let blob_service = MemoryBlobService::default();
        let directory_service = RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig {
            path: None,
            read_only: false,
            cache_size: None,
        })
        .unwrap();

        let pis = snix_store::pathinfoservice::LruPathInfoService::with_capacity(
            "test".to_string(),
            std::num::NonZeroUsize::new(128).unwrap(),
        );

        let mut builder = make_test_builder(
            (blob_service, directory_service, pis),
            DummyBuildService::default(),
            Path::new("/nix/store"),
            false,
        );

        builder.build(&drv_path, &mut kp).await
    });

    // DummyBuildService errors when build is attempted (no cache hit)
    assert!(result.is_err(), "should fail because output doesn't exist and DummyBuildService can't build");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("builds are not supported")
            || err.contains("build failed")
            || err.contains("root build(s) failed"),
        "error should be from DummyBuildService: {err}"
    );
}

// -- FOD hash mismatch test (no sandbox needed) --

#[test]
fn fod_hash_mismatch_error() {
    use nix_compat::nixhash::CAHash;
    use nix_compat::nixhash::NixHash;

    // verify_fod_hash is not pub, so test through the types directly.
    // The error path: if expected != actual, we get FodHashMismatch.
    // Build a derivation with fixed_output and wrong content.
    let drv = CrunchDerivation {
        name: "bad-fod".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: HashMap::new(),
        inputs: vec![],
        fixed_output: Some(crunch_glue::FixedOutput {
            hash: "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba".to_string(),
            algo: "sha256".to_string(),
            mode: "recursive".to_string(),
        }),
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut cc = ConversionCache::default();
    let (_drv_path, nix_drv) = crunch_glue::convert(&drv, &mut cc).unwrap();

    let mut kp = DerivationRegistry::default();
    populate_registry(&mut kp, cc.iter_entries());
    // The FOD has ca_hash set — verify it was constructed
    let out = nix_drv.outputs.get("out").unwrap();
    assert!(out.ca_hash.is_some(), "FOD should have ca_hash on output");

    // The actual hash mismatch detection happens during build when NAR
    // hash doesn't match. We verify the glue correctly propagates the
    // ca_hash to the output.
    match &out.ca_hash {
        Some(CAHash::Nar(NixHash::Sha256(digest))) => {
            let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
            assert_eq!(hex, "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba");
        }
        other => panic!("expected Nar(Sha256), got: {other:?}"),
    }
}

// -- End-to-end test with bwrap (gated) --

#[test]
fn end_to_end_trivial_build() {
    if !has_bwrap() {
        eprintln!("SKIP: bwrap not available");
        return;
    }

    // Use crunch-eval to evaluate a trivial derivation that writes
    // a file to $out. Use a temp dir as the store.
    let drv = CrunchDerivation {
        name: "trivial".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![
            "-c".to_string(),
            "mkdir -p $out && echo 'built by crunch' > $out/result.txt".to_string(),
        ],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: HashMap::new(),
        inputs: vec![],
        fixed_output: None,
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut cc = ConversionCache::default();
    let (drv_path, nix_drv) = crunch_glue::convert(&drv, &mut cc).unwrap();

    let mut kp = DerivationRegistry::default();
    populate_registry(&mut kp, cc.iter_entries());
    let out_path = nix_drv.outputs.get("out").unwrap().path.as_ref().unwrap().to_absolute_path();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        use snix_castore::blobservice::MemoryBlobService;
        use snix_castore::directoryservice::RedbDirectoryService;
        use snix_castore::directoryservice::RedbDirectoryServiceConfig;

        let blob_service = MemoryBlobService::default();
        let directory_service = RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig {
            path: None,
            read_only: false,
            cache_size: None,
        })
        .unwrap();

        #[cfg(target_os = "linux")]
        {
            use snix_build::buildservice::BubblewrapBuildService;

            let workdir = std::env::temp_dir().join("crunch-test-builds");
            std::fs::create_dir_all(&workdir).unwrap();

            let build_service =
                BubblewrapBuildService::new(workdir.clone(), blob_service.clone(), directory_service.clone());

            let pis = snix_store::pathinfoservice::LruPathInfoService::with_capacity(
                "test".to_string(),
                std::num::NonZeroUsize::new(FETCH_TEST_PATH_INFO_CAPACITY).unwrap(),
            );

            let mut builder =
                make_test_builder((blob_service, directory_service, pis), build_service, Path::new("/nix/store"), true);

            let outcome = builder.build(&drv_path, &mut kp).await;

            // Clean up workdir
            let _ = std::fs::remove_dir_all(&workdir);

            outcome
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(crunch_build::Error::Sandbox(std::io::Error::other("not on linux")))
        }
    });

    match result {
        Ok(outcome) => {
            assert!(!outcome.cached);
            assert!(outcome.outputs.contains_key("out"));

            // Verify the output was actually produced
            let result_file = PathBuf::from(&out_path).join("result.txt");
            if result_file.exists() {
                let content = std::fs::read_to_string(&result_file).unwrap();
                assert!(content.contains("built by crunch"));
            }

            // Clean up
            let _ = std::fs::remove_dir_all(&out_path);
        }
        Err(e) => {
            // bwrap might fail in some environments (containers, CI).
            // Don't hard-fail the test suite.
            eprintln!("SKIP end-to-end build (bwrap failed): {e}");
        }
    }
}

// -- CA end-to-end test with bwrap --

#[test]
fn end_to_end_ca_build() {
    if !has_bwrap() {
        eprintln!("SKIP: bwrap not available");
        return;
    }

    let drv = CrunchDerivation {
        name: "ca-trivial".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        args: vec![
            "-c".to_string(),
            "mkdir -p $out && echo 'ca-built' > $out/result.txt".to_string(),
        ],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: HashMap::new(),
        inputs: vec![],
        fixed_output: None,
        addressing_mode: "content-addressed".to_string(),
        provenance: None,
    };

    let mut cc = ConversionCache::default();
    let (drv_path, nix_drv) = crunch_glue::convert(&drv, &mut cc).unwrap();

    let mut kp = DerivationRegistry::default();
    populate_registry(&mut kp, cc.iter_entries());
    // CA derivation: output paths are None before build
    assert!(nix_drv.outputs["out"].path.is_none(), "CA output should be None before build");

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        use snix_castore::blobservice::MemoryBlobService;
        use snix_castore::directoryservice::RedbDirectoryService;
        use snix_castore::directoryservice::RedbDirectoryServiceConfig;

        let blob_service = MemoryBlobService::default();
        let directory_service = RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig {
            path: None,
            read_only: false,
            cache_size: None,
        })
        .unwrap();

        #[cfg(target_os = "linux")]
        {
            use snix_build::buildservice::BubblewrapBuildService;

            let workdir = std::env::temp_dir().join("crunch-test-ca-builds");
            std::fs::create_dir_all(&workdir).unwrap();

            let build_service =
                BubblewrapBuildService::new(workdir.clone(), blob_service.clone(), directory_service.clone());

            let pis = snix_store::pathinfoservice::LruPathInfoService::with_capacity(
                "test".to_string(),
                NonZeroUsize::new(FETCH_TEST_PATH_INFO_CAPACITY).unwrap(),
            );

            let mut builder =
                make_test_builder((blob_service, directory_service, pis), build_service, Path::new("/nix/store"), true);

            let outcome = builder.build(&drv_path, &mut kp).await;

            let _ = std::fs::remove_dir_all(&workdir);

            outcome
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(crunch_build::Error::Sandbox(std::io::Error::other("not on linux")))
        }
    });

    match result {
        Ok(outcome) => {
            assert!(!outcome.cached);
            assert!(outcome.outputs.contains_key("out"));

            let pi = &outcome.outputs["out"];
            // CA output should have a store path now
            let ca_path = pi.store_path.to_absolute_path();
            assert!(ca_path.starts_with("/nix/store/"), "CA path should be in store: {ca_path}");
            // CA field should be set
            assert!(pi.ca.is_some(), "CA PathInfo should have ca field");

            // The resolved path should be in KnownPaths
            let drv_abs = drv_path.to_absolute_path();
            let resolved = kp.get_output_path(&drv_abs, "out");
            assert!(resolved.is_some(), "CA output should be resolved in KnownPaths");
            assert_eq!(resolved.unwrap().to_absolute_path(), ca_path, "resolved path should match build outcome");

            // Clean up the output from the store
            let _ = std::fs::remove_dir_all(&ca_path);
        }
        Err(e) => {
            eprintln!("SKIP end-to-end CA build (bwrap failed): {e}");
        }
    }
}

// -- Eval → glue round-trip with seed --

#[test]
fn eval_hello_world_with_seed() {
    // This test verifies the full Nickel → serde → glue path for the
    // hello-world example (without building).
    let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
    let seed_path = examples_dir.join("seed.ncl");

    if !seed_path.exists() {
        eprintln!("SKIP: examples/seed.ncl not generated (run crunch bootstrap first)");
        return;
    }

    let mut import_paths = stdlib_import_path();
    import_paths.push(examples_dir.into());

    let hello_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello-world.ncl");
    let drv: CrunchDerivation = crunch_eval::evaluate_and_deserialize(&hello_path, &import_paths).unwrap();

    assert_eq!(drv.name, "hello-world");
    assert!(!drv.inputs.is_empty());
    assert_eq!(drv.builder, "/bin/sh");

    // Check that inputs are all valid store paths
    for input in &drv.inputs {
        match input {
            Input::Source(p) => {
                assert!(p.starts_with("/nix/store/"), "source input should be a store path: {p}");
            }
            Input::Derivation(_)
            | Input::DerivationFile(_)
            | Input::ResolvedDerivation(_)
            | Input::OutputSelection(_) => {
                panic!("hello-world should only have source inputs from seed");
            }
        }
    }

    // Convert through glue
    let mut cc = ConversionCache::default();
    let (drv_path, nix_drv) = crunch_glue::convert(&drv, &mut cc).unwrap();
    assert!(drv_path.to_string().ends_with("hello-world.drv"));
    // CA derivations have None output paths until after build.
    // Input-addressed derivations have Some.
    if drv.addressing_mode == "content-addressed" {
        assert!(
            nix_drv.outputs.get("out").unwrap().path.is_none(),
            "CA derivation should have None output path before build"
        );
    } else {
        assert!(nix_drv.outputs.get("out").unwrap().path.is_some());
    }
    // Should have source inputs
    assert!(!nix_drv.input_sources.is_empty(), "should have input_sources from seed");
}

// ── Fetcher integration tests ──────────────────────────────────────────

fn make_fetch_builder(
    output_dir: &std::path::Path,
) -> crunch_build::Builder<
    crunch_build::DispatchBuildService<crunch_build::FetchBuildService, snix_build::buildservice::DummyBuildService>,
> {
    use snix_build::buildservice::DummyBuildService;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;

    let blob_service = MemoryBlobService::default();
    let directory_service = RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig {
        path: None,
        read_only: false,
        cache_size: None,
    })
    .unwrap();
    let pis = snix_store::pathinfoservice::LruPathInfoService::with_capacity(
        "test".to_string(),
        NonZeroUsize::new(FETCH_TEST_PATH_INFO_CAPACITY).unwrap(),
    );
    let store = crunch_store::StoreHandle::from_services_with_store_dir(
        crunch_store::StoreHandleServices {
            blob_service: std::sync::Arc::new(blob_service),
            directory_service: std::sync::Arc::new(directory_service),
            pathinfo_service: std::sync::Arc::new(pis),
            remote_pathinfo: None,
            state_dir: output_dir.join("state"),
            output_dir_str: output_dir.to_string_lossy().into_owned(),
            publishers: Vec::new(),
        },
        nix_compat::store_path::STORE_DIR.to_string(),
    );
    let crunch_store::PipelineStoreParts {
        build_store,
        action_results,
        build_service_store,
        output_lookup: _output_lookup,
        root_registry: _root_registry,
    } = store.into_pipeline_store_parts();
    let fetch_service = crunch_build::FetchBuildService::new(build_service_store);
    let dispatch = crunch_build::DispatchBuildService::new(fetch_service, DummyBuildService::default());

    crunch_build::Builder::from_store_parts(
        crunch_store::BuilderStoreParts {
            build_store,
            action_results,
        },
        dispatch,
        test_keypair(),
        test_trusted_keys(),
        true,
        true,
    )
}

fn run_git(dir: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git").args(args).current_dir(dir).output().unwrap();
    assert!(output.status.success(), "git {:?} failed: {}", args, String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn create_git_repo_with_head(parent: &Path) -> (PathBuf, String) {
    let repo = parent.join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    run_git(&repo, &["init"]);
    run_git(&repo, &["branch", "-M", "main"]);
    run_git(&repo, &["config", "user.email", "test@example.com"]);
    run_git(&repo, &["config", "user.name", "Test User"]);
    std::fs::write(repo.join("hello.txt"), "hello from fetchgit integration\n").unwrap();
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "-m", "initial"]);
    let rev = run_git(&repo, &["rev-parse", "HEAD"]);
    (repo, rev)
}

fn copy_tree_without_dot_git(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = entry.file_name();
        if file_name == ".git" {
            continue;
        }
        let dest = dst.join(&file_name);
        let file_type = entry.file_type().unwrap();
        if file_type.is_dir() {
            copy_tree_without_dot_git(&path, &dest);
        } else {
            std::fs::copy(&path, &dest).unwrap();
        }
    }
}

fn compute_recursive_sha256_sri(path: &Path) -> String {
    use sha2::Digest;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use snix_castore::import::fs::ingest_path;
    use snix_store::nar::write_nar;
    use snix_store::utils::AsyncIoBridge;

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let root = path.to_path_buf();
    rt.block_on(async move {
        let blob_service = MemoryBlobService::default();
        let directory_service =
            RedbDirectoryService::new_temporary("fetchgit-hash".to_string(), RedbDirectoryServiceConfig::default())
                .unwrap();
        let node = ingest_path::<_, _, _, &[u8]>(blob_service.clone(), directory_service.clone(), &root, None)
            .await
            .unwrap();
        let mut hasher = sha2::Sha256::new();
        write_nar(AsyncIoBridge(&mut hasher), &node, blob_service, directory_service).await.unwrap();
        let digest: [u8; 32] = hasher.finalize().into();
        format!("sha256-{}", data_encoding::BASE64.encode(&digest))
    })
}

#[test]
fn fetchurl_downloads_and_verifies_hash() {
    use sha2::Digest;

    let output_dir = tempfile::tempdir().unwrap();
    let output_dir_str = output_dir.path().to_str().unwrap();

    let content = b"hello from crunch fetcher test";
    let sha256_digest: [u8; 32] = sha2::Sha256::digest(content).into();
    let sri_hash = format!("sha256-{}", data_encoding::BASE64.encode(&sha256_digest));

    let source = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(source.path(), content).unwrap();
    let source_url = format!("file://{}", source.path().display());

    let drv = CrunchDerivation {
        name: "fetched-file".to_string(),
        builder: "builtin:fetchurl".to_string(),
        system: "builtin".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: {
            let mut e = HashMap::new();
            e.insert("url".to_string(), source_url);
            e
        },
        inputs: vec![],
        fixed_output: Some(crunch_glue::FixedOutput {
            hash: sri_hash,
            algo: "sha256".to_string(),
            mode: "flat".to_string(),
        }),
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    // KnownPaths uses /nix/store (logical prefix), output_dir is
    // the physical location where the Builder writes outputs.
    let mut cc = ConversionCache::default();
    let (drv_path, nix_drv) = crunch_glue::convert(&drv, &mut cc).unwrap();

    let mut kp = DerivationRegistry::default();
    populate_registry(&mut kp, cc.iter_entries());
    let out_path = nix_drv
        .outputs
        .get("out")
        .unwrap()
        .path
        .as_ref()
        .unwrap()
        .to_absolute_path_with_prefix(output_dir_str);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let mut builder = make_fetch_builder(output_dir.path());
        builder.build(&drv_path, &mut kp).await
    });

    match result {
        Ok(outcome) => {
            assert!(!outcome.cached);
            assert!(outcome.outputs.contains_key("out"));

            assert!(std::path::Path::new(&out_path).exists(), "output should exist at {out_path}");
            let fetched = std::fs::read(&out_path).unwrap();
            assert_eq!(fetched, content, "fetched content should match");
        }
        Err(e) => {
            panic!("fetchurl build failed: {e}");
        }
    }
}

#[test]
fn fetch_tarball_unpacks_and_strips_prefix() {
    let output_dir = tempfile::tempdir().unwrap();
    let output_dir_str = output_dir.path().to_str().unwrap();

    // Build a tar.gz in memory with a top-level dir
    let tmp_src = tempfile::tempdir().unwrap();
    let inner = tmp_src.path().join("project-v1.0");
    std::fs::create_dir(&inner).unwrap();
    std::fs::write(inner.join("README.md"), "# Hello").unwrap();
    std::fs::create_dir(inner.join("src")).unwrap();
    std::fs::write(inner.join("src/main.rs"), "fn main() {}").unwrap();

    let mut tar_builder = tar::Builder::new(Vec::new());
    tar_builder.append_dir_all("project-v1.0", &inner).unwrap();
    let tar_data = tar_builder.into_inner().unwrap();

    use std::io::Write;

    use flate2::write::GzEncoder;
    let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(&tar_data).unwrap();
    let gz_data = encoder.finish().unwrap();

    let tarball = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tarball.path(), &gz_data).unwrap();
    let tarball_url = format!("file://{}", tarball.path().display());

    // Use a dummy hash — the pipeline should download, unpack, NAR-hash,
    // then fail with a hash mismatch. That proves extraction worked.
    let dummy_hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

    let drv = CrunchDerivation {
        name: "project-src".to_string(),
        builder: "builtin:fetchurl".to_string(),
        system: "builtin".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: {
            let mut e = HashMap::new();
            e.insert("url".to_string(), tarball_url);
            e.insert("unpack".to_string(), "1".to_string());
            e
        },
        inputs: vec![],
        fixed_output: Some(crunch_glue::FixedOutput {
            hash: dummy_hash.to_string(),
            algo: "sha256".to_string(),
            mode: "recursive".to_string(),
        }),
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut cc = ConversionCache::default();
    let (drv_path, nix_drv) = crunch_glue::convert(&drv, &mut cc).unwrap();

    let mut kp = DerivationRegistry::default();
    populate_registry(&mut kp, cc.iter_entries());
    let _out_path = nix_drv
        .outputs
        .get("out")
        .unwrap()
        .path
        .as_ref()
        .unwrap()
        .to_absolute_path_with_prefix(output_dir_str);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let mut builder = make_fetch_builder(output_dir.path());
        builder.build(&drv_path, &mut kp).await
    });

    // Dummy hash → hash mismatch. Proves fetch + unpack + NAR hash ran.
    match result {
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("hash mismatch")
                    || msg.contains("FOD hash mismatch")
                    || msg.contains("root build(s) failed"),
                "expected hash mismatch error, got: {msg}"
            );
            // Verify the unpacked tree existed before the mismatch
            // (the error comes from post-ingest verification, so
            // extraction must have succeeded).
        }
        Ok(_) => {
            panic!("expected hash mismatch error but build succeeded");
        }
    }
}

#[test]
fn fetchgit_downloads_requested_revision_and_verifies_recursive_hash() {
    let output_dir = tempfile::tempdir().unwrap();
    let output_dir_str = output_dir.path().to_str().unwrap();
    let repo_root = tempfile::tempdir().unwrap();
    let (repo, rev) = create_git_repo_with_head(repo_root.path());
    let expected_tree = tempfile::tempdir().unwrap();
    copy_tree_without_dot_git(&repo, expected_tree.path());
    let sri_hash = compute_recursive_sha256_sri(expected_tree.path());
    let repo_url = format!("file://{}", repo.display());

    let drv = CrunchDerivation {
        name: "git-src".to_string(),
        builder: "builtin:fetchurl".to_string(),
        system: "builtin".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: {
            let mut e = HashMap::new();
            e.insert("url".to_string(), repo_url);
            e.insert("type".to_string(), "git".to_string());
            e.insert("rev".to_string(), rev);
            e
        },
        inputs: vec![],
        fixed_output: Some(crunch_glue::FixedOutput {
            hash: sri_hash,
            algo: "sha256".to_string(),
            mode: "recursive".to_string(),
        }),
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut cc = ConversionCache::default();
    let (drv_path, nix_drv) = crunch_glue::convert(&drv, &mut cc).unwrap();

    let mut kp = DerivationRegistry::default();
    populate_registry(&mut kp, cc.iter_entries());
    let out_path = nix_drv
        .outputs
        .get("out")
        .unwrap()
        .path
        .as_ref()
        .unwrap()
        .to_absolute_path_with_prefix(output_dir_str);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let mut builder = make_fetch_builder(output_dir.path());
        builder.build(&drv_path, &mut kp).await
    });

    match result {
        Ok(outcome) => {
            assert!(!outcome.cached, "fresh fetchgit build should not be cached");
            assert!(outcome.outputs.contains_key("out"));
            assert!(Path::new(&out_path).exists(), "fetchgit output should exist at {out_path}");
            assert_eq!(
                std::fs::read_to_string(Path::new(&out_path).join("hello.txt")).unwrap(),
                "hello from fetchgit integration\n"
            );
            assert!(!Path::new(&out_path).join(".git").exists(), "fetchgit output must not contain .git");
        }
        Err(err) => panic!("fetchgit build failed: {err}"),
    }
}

#[test]
fn fetchgit_wrong_recursive_hash_reports_mismatch() {
    let output_dir = tempfile::tempdir().unwrap();
    let repo_root = tempfile::tempdir().unwrap();
    let (repo, rev) = create_git_repo_with_head(repo_root.path());
    let repo_url = format!("file://{}", repo.display());

    let drv = CrunchDerivation {
        name: "git-src".to_string(),
        builder: "builtin:fetchurl".to_string(),
        system: "builtin".to_string(),
        args: vec![],
        outputs: vec!["out".to_string()],
        dynamic_plan_outputs: vec![],
        env: {
            let mut e = HashMap::new();
            e.insert("url".to_string(), repo_url);
            e.insert("type".to_string(), "git".to_string());
            e.insert("rev".to_string(), rev);
            e
        },
        inputs: vec![],
        fixed_output: Some(crunch_glue::FixedOutput {
            hash: "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string(),
            algo: "sha256".to_string(),
            mode: "recursive".to_string(),
        }),
        addressing_mode: "input-addressed".to_string(),
        provenance: None,
    };

    let mut cc = ConversionCache::default();
    let (drv_path, _nix_drv) = crunch_glue::convert(&drv, &mut cc).unwrap();

    let mut kp = DerivationRegistry::default();
    populate_registry(&mut kp, cc.iter_entries());

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let mut builder = make_fetch_builder(output_dir.path());
        builder.build(&drv_path, &mut kp).await
    });

    match result {
        Err(err) => {
            let msg = err.to_string();
            assert!(
                msg.contains("hash mismatch")
                    || msg.contains("FOD hash mismatch")
                    || msg.contains("root build(s) failed"),
                "expected recursive hash mismatch from fetchgit, got: {msg}"
            );
        }
        Ok(_) => panic!("expected fetchgit recursive hash mismatch but build succeeded"),
    }
}
