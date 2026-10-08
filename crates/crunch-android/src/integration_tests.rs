use std::collections::HashMap;
use std::fs;
use std::num::NonZeroUsize;
use std::sync::Arc;

use crunch_build::BuildOutcome;
use crunch_build::Builder;
use crunch_build::DerivationRegistry;
use crunch_build::DispatchBuildService;
use crunch_build::FetchBuildService;
use crunch_build::populate_registry;
use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use crunch_glue::FixedOutput;
use crunch_store::BuilderStoreParts;
use crunch_store::PipelineStoreParts;
use crunch_store::StoreHandle;
use crunch_store::StoreHandleServices;
use crunch_store::VerifiedSourceIngestRequest;
use snix_build::buildservice::BubblewrapBuildService;
use snix_castore::blobservice::MemoryBlobService;
use snix_castore::directoryservice::RedbDirectoryService;
use snix_castore::directoryservice::RedbDirectoryServiceConfig;
use snix_store::pathinfoservice::LruPathInfoService;

use super::*;

const TEST_KEY: &str =
    "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";
mod official_proof;

type Service = DispatchBuildService<FetchBuildService, BubblewrapBuildService<MemoryBlobService, RedbDirectoryService>>;

fn has_bwrap() -> bool {
    std::process::Command::new("bwrap")
        .args(["--ro-bind", "/", "/", "--", "/bin/true"])
        .status()
        .is_ok_and(|status| status.success())
}

fn fetch(name: &str, from: &Path, sha256: &str, executable: bool) -> CrunchDerivation {
    let mut env = HashMap::from([("url".into(), format!("file://{}", from.display()))]);
    if executable {
        env.insert("executable".into(), "1".into());
    }
    CrunchDerivation {
        name: name.into(),
        builder: "builtin:fetchurl".into(),
        system: "x86_64-linux".into(),
        args: Vec::new(),
        outputs: vec!["out".into()],
        dynamic_plan_outputs: Vec::new(),
        env,
        inputs: Vec::new(),
        fixed_output: Some(FixedOutput {
            hash: sha256.into(),
            algo: "sha256".into(),
            mode: "flat".into(),
        }),
        addressing_mode: "input-addressed".into(),
        provenance: None,
    }
}

fn extraction_as_derivation(step: &ExtractionDerivation) -> CrunchDerivation {
    CrunchDerivation {
        name: step.name.clone(),
        builder: step.builder.clone(),
        system: step.system.clone(),
        args: step.args.clone(),
        outputs: step.outputs.clone(),
        dynamic_plan_outputs: Vec::new(),
        env: step.env.clone().into_iter().collect(),
        inputs: step.inputs.iter().cloned().map(crunch_glue::Input::Source).collect(),
        fixed_output: None,
        addressing_mode: step.addressing_mode.clone(),
        provenance: None,
    }
}

fn tool_as_derivation(step: &DerivationStep) -> CrunchDerivation {
    CrunchDerivation {
        name: step.name.clone(),
        builder: step.builder.clone(),
        system: step.system.clone(),
        args: step.args.clone(),
        outputs: step.outputs.clone(),
        dynamic_plan_outputs: Vec::new(),
        env: step.env.clone().into_iter().collect(),
        inputs: step.inputs.iter().cloned().map(crunch_glue::Input::Source).collect(),
        fixed_output: None,
        addressing_mode: step.addressing_mode.clone(),
        provenance: None,
    }
}

async fn realize(builder: &mut Builder<Service>, drv: CrunchDerivation, prefix: &str) -> BuildOutcome {
    let mut cache = ConversionCache::new(prefix);
    let (path, _) = crunch_glue::convert(&drv, &mut cache).unwrap();
    let mut registry = DerivationRegistry::new(prefix);
    populate_registry(&mut registry, cache.iter_entries());
    let mut report = builder
        .build_all_report(&[path], &mut registry, 1)
        .await
        .unwrap_or_else(|error| panic!("real sandbox build {} could not dispatch: {error}", drv.name));
    assert!(report.failed.is_empty(), "real sandbox build {} failed: {:#?}", drv.name, report.failed);
    report.outcomes.pop().expect("successful root must have one outcome")
}

#[test]
fn sandbox_executes_real_extraction_and_unsigned_apk_derivation_steps() {
    if !has_bwrap() {
        eprintln!("skipping: bwrap/user namespace unavailable");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let prefix = dir.path().join("store");
    fs::create_dir(&prefix).unwrap();
    let (mut plan, mut inputs) = super::tests::fixture(dir.path(), false);
    let original_source = inputs.source_root.clone();
    let source_path = prefix.join(format!("{}-android-source", "a".repeat(32)));
    fs::rename(&original_source, &source_path).unwrap();
    let static_busybox = std::env::var_os("SNIX_BUILD_SANDBOX_SHELL").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(
            "/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox",
        )
    });
    assert!(static_busybox.is_file(), "static busybox utility is required for the real sandbox fixture");
    let utility_store_root = prefix.join(format!("{}-apk-utility", "b".repeat(32)));
    fs::create_dir(&utility_store_root).unwrap();
    let utility_member = utility_store_root.join("busybox");
    fs::copy(static_busybox, &utility_member).unwrap();
    let runtime_glibc_root = prefix.join(format!("{}-runtime-glibc", "c".repeat(32)));
    let runtime_libgcc_root = prefix.join(format!("{}-runtime-libgcc", "d".repeat(32)));
    fs::rename(&inputs.runtime_glibc.store_path, &runtime_glibc_root).unwrap();
    fs::rename(&inputs.runtime_libgcc.store_path, &runtime_libgcc_root).unwrap();
    inputs.runtime_glibc.store_path = runtime_glibc_root.clone();
    inputs.runtime_libgcc.store_path = runtime_libgcc_root.clone();
    inputs.source_root = source_path;
    inputs.utility = utility_member;
    let reviewed = plan.toolchain.clone();
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(async {
        let prefix_text = prefix.to_str().unwrap();
        let blob = MemoryBlobService::default();
        let directory =
            RedbDirectoryService::new_temporary("apk-sandbox".into(), RedbDirectoryServiceConfig::default()).unwrap();
        let pathinfo = LruPathInfoService::with_capacity("apk-sandbox".into(), NonZeroUsize::new(128).unwrap());
        let sandbox = BubblewrapBuildService::new(dir.path().join("sandbox"), blob.clone(), directory.clone());
        let mut store = StoreHandle::from_services_with_store_dir(
            StoreHandleServices {
                blob_service: Arc::new(blob),
                directory_service: Arc::new(directory),
                pathinfo_service: Arc::new(pathinfo),
                remote_pathinfo: None,
                state_dir: dir.path().join("state"),
                output_dir_str: prefix_text.to_string(),
                publishers: Vec::new(),
            },
            prefix_text.to_string(),
        );
        let key = crunch_build::load_keypair(TEST_KEY).unwrap();
        let trust = crunch_build::build_trusted_keys(&key, None);
        store
            .source_admission()
            .ingest(VerifiedSourceIngestRequest {
                source_path: &inputs.source_root,
                logical_store_path: inputs.source_root.to_str().unwrap(),
                source_name: "android-source",
                signing_key: &key.signing_key,
            })
            .await
            .unwrap();
        store
            .source_admission()
            .ingest(VerifiedSourceIngestRequest {
                source_path: &utility_store_root,
                logical_store_path: utility_store_root.to_str().unwrap(),
                source_name: "apk-utility",
                signing_key: &key.signing_key,
            })
            .await
            .unwrap();
        for (runtime, name) in [
            (&mut inputs.runtime_glibc, "runtime-glibc"),
            (&mut inputs.runtime_libgcc, "runtime-libgcc"),
        ] {
            let admitted = store
                .source_admission()
                .ingest(VerifiedSourceIngestRequest {
                    source_path: &runtime.store_path,
                    logical_store_path: runtime.store_path.to_str().unwrap(),
                    source_name: name,
                    signing_key: &key.signing_key,
                })
                .await
                .unwrap();
            runtime.identity.nar_sha256 = admitted.nar_sha256.iter().map(|byte| format!("{byte:02x}")).collect();
            runtime.identity.nar_size = admitted.nar_size;
        }
        let PipelineStoreParts {
            build_store,
            action_results,
            slice_admission,
            build_service_store,
            output_lookup,
            root_registry: _,
        } = store.into_pipeline_store_parts();
        let fetcher = FetchBuildService::new(build_service_store);
        let mut builder = Builder::from_store_parts(
            BuilderStoreParts {
                build_store,
                action_results,
                slice_admission,
            },
            DispatchBuildService::new(fetcher, sandbox),
            key,
            trust.clone(),
            false,
            false,
        );
        for archive in [&mut inputs.build_tools, &mut inputs.jdk, &mut inputs.platform] {
            let drv = fetch(
                &format!("{}-fixture", archive.identity.component),
                &archive.store_path,
                &archive.identity.sha256,
                false,
            );
            let output = realize(&mut builder, drv, prefix_text).await;
            assert!(!output.cached);
            archive.store_path = prefix.join(output.outputs["out"].store_path.to_string());
            assert!(archive.store_path.is_file());
        }
        plan.toolchain = reviewed.clone();
        let reviewed_runtime = super::tests::runtime_cohort(&inputs);
        let mut prepared = PreparedApk::bind(plan, inputs, &reviewed, &reviewed_runtime, &prefix).unwrap();
        let mut extraction_results = Vec::new();
        for extraction in prepared.extractions(&prefix).unwrap() {
            extraction_results.push(realize(&mut builder, extraction_as_derivation(&extraction), prefix_text).await);
        }
        let outcomes: [BuildOutcome; 3] = extraction_results.try_into().unwrap();
        let trees = verified_same_run_extractions(&prepared, &outcomes, &output_lookup, &trust, &prefix)
            .await
            .expect("same-run extraction provenance");
        let mut swapped = outcomes.clone();
        swapped.swap(0, 1);
        assert!(verified_same_run_extractions(&prepared, &swapped, &output_lookup, &trust, &prefix).await.is_err());
        assert!(verified_same_run_extractions(&prepared, &outcomes, &output_lookup, &[], &prefix).await.is_err());
        let mut cached = outcomes.clone();
        cached[0].cached = true;
        assert!(verified_same_run_extractions(&prepared, &cached, &output_lookup, &trust, &prefix).await.is_err());
        let runtime_digest = prepared.inputs.runtime_glibc.identity.nar_sha256.clone();
        prepared.inputs.runtime_glibc.identity.nar_sha256 = "0".repeat(64);
        assert!(
            verified_same_run_extractions(&prepared, &outcomes, &output_lookup, &trust, &prefix).await.is_err(),
            "a changed runtime tree digest must stop before an SDK stage",
        );
        prepared.inputs.runtime_glibc.identity.nar_sha256 = runtime_digest;
        let mut previous = BTreeMap::new();
        let mut observations = Vec::new();
        for step in prepared.steps() {
            let derivation = prepared.derivation(step.kind, &previous, &trees, &prefix).unwrap();
            assert!(
                derivation.inputs.contains(&prepared.inputs.runtime_glibc.store_path.display().to_string())
                    && derivation.inputs.contains(&prepared.inputs.runtime_libgcc.store_path.display().to_string()),
                "every SDK stage must bind both immutable runtime roots",
            );
            let output = realize(&mut builder, tool_as_derivation(&derivation), prefix_text).await;
            let output_dir = prefix.join(output.outputs["out"].store_path.to_string());
            assert!(output_dir.join(&step.output).exists(), "missing {:?} output", step.kind);
            observations
                .push(fs::read_to_string(output_dir.join("argv.log")).expect("stub invocation inside real sandbox"));
            previous.insert(step.kind, output_dir);
        }
        assert!(observations[0].contains("aapt2|compile"));
        assert!(observations[1].contains("aapt2|link"));
        assert!(observations[2].contains("javac|"));
        assert!(observations[3].contains("d8|"));
        assert!(observations[4].contains("jar|") && observations[4].contains("zipalign|"));
        assert_eq!(previous.len(), 5);
        assert!(previous[&StepKind::Zipalign].join("unsigned.apk").is_file());
    });
}
