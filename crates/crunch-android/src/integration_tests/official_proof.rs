//! Explicit, ignored real-SDK proof. The outer rail supplies a Nickel-exported
//! example and preflighted original-HTTPS source state; this test never fetches.

use std::path::Path;
use std::path::PathBuf;

use crunch_android_core::SourceIdentity;
use crunch_android_core::Toolchain;
use crunch_android_core::UnpackShape;
use crunch_build::FetchSourceOverride;
use crunch_build::FetchSourceOverrideKind;
use crunch_build::FetchSourcePolicy;
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;

use super::*;

const GLIBC_ORIGIN: &str = "/nix/store/n51dhmdbik1kfrsm62j5knavmigwrl1a-glibc-2.42-84";
const LIBGCC_ORIGIN: &str = "/nix/store/ssvq1r0xd8f7paf6zqgpfql1a4drwhy2-xgcc-15.3.0-libgcc";
const GLIBC_NAR_SHA256: &str = "428a192cf9765f6fddaa77b31a028e3cda7351bc61c5b2f02f34d2a0e843f47a";
const LIBGCC_NAR_SHA256: &str = "70cbaf1ca29943bbace24a54038d2d3716699104118afb7a7d6b591c03d570af";

#[derive(Deserialize)]
struct Example {
    plan: ApkPlan,
    manifest: String,
    java: String,
    resources_xml: String,
}

fn source(
    component: &str,
    version: &str,
    url: &str,
    sha256: &str,
    record_identity_blake3: &str,
    archive: &str,
    root: &str,
) -> SourceIdentity {
    SourceIdentity {
        component: component.into(),
        version: version.into(),
        url: url.into(),
        sha256: sha256.into(),
        record_identity_blake3: record_identity_blake3.into(),
        unpack_shape: UnpackShape {
            archive: archive.into(),
            root: root.into(),
        },
        platform: "x86_64-linux".into(),
    }
}

fn reviewed_toolchain() -> Toolchain {
    Toolchain {
        build_tools: source(
            "android-build-tools",
            "35.0.0",
            "https://dl.google.com/android/repository/build-tools_r35_linux.zip",
            "bd3a4966912eb8b30ed0d00b0cda6b6543b949d5ffe00bea54c04c81e1561d88",
            "5bc394f89b8d8f36d1d916873b34021bb142db17107460da180e07c4d0b67e7f",
            "zip",
            "android-15/",
        ),
        jdk: source(
            "temurin-jdk",
            "17.0.17+10",
            "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.17%2B10/OpenJDK17U-jdk_x64_linux_hotspot_17.0.17_10.tar.gz",
            "992f96e7995075ac7636bb1a8de52b0c61d71ed3137fafc979ab96b4ab78dd75",
            "5891c7afb04cf3c783a9294b9033ed22092ce83206c850af8bab79a6d2b49d35",
            "tar-gz",
            "jdk-17.0.17+10/",
        ),
        platform: source(
            "android-platform",
            "35_r02",
            "https://dl.google.com/android/repository/platform-35_r02.zip",
            "0988cacad01b38a18a47bac14a0695f246bc76c1b06c0eeb8eb0dc825ab0c8e0",
            "d77b395b209a6e6b2becf69a924c19ff2580fb2c2c4113a4b38aa8bb09e126cf",
            "zip",
            "android-35/",
        ),
    }
}

fn reviewed_runtime() -> RuntimeCohort {
    RuntimeCohort {
        glibc: RuntimeIdentity {
            source_cohort: GLIBC_ORIGIN.into(),
            nar_sha256: GLIBC_NAR_SHA256.into(),
            nar_size: 35_073_128,
        },
        libgcc: RuntimeIdentity {
            source_cohort: LIBGCC_ORIGIN.into(),
            nar_sha256: LIBGCC_NAR_SHA256.into(),
            nar_size: 197_672,
        },
    }
}

fn env_path(key: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(key).unwrap_or_else(|| panic!("missing proof prerequisite: {key}")))
}

fn digest_hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn fetch_original(name: &str, identity: &SourceIdentity) -> CrunchDerivation {
    CrunchDerivation {
        name: name.into(),
        builder: "builtin:fetchurl".into(),
        system: "x86_64-linux".into(),
        args: Vec::new(),
        outputs: vec!["out".into()],
        dynamic_plan_outputs: Vec::new(),
        env: HashMap::from([("url".into(), identity.url.clone())]),
        inputs: Vec::new(),
        fixed_output: Some(FixedOutput {
            hash: identity.sha256.clone(),
            algo: "sha256".into(),
            mode: "flat".into(),
        }),
        addressing_mode: "input-addressed".into(),
        provenance: None,
    }
}

fn write_member(root: &Path, member: &str, bytes: &[u8]) {
    let path = root.join(member);
    fs::create_dir_all(path.parent().expect("source member parent")).unwrap();
    fs::write(path, bytes).unwrap();
}

async fn admit_source(
    store: &mut StoreHandle,
    source_path: &Path,
    logical_path: &Path,
    name: &str,
    key: &crunch_build::KeyPair,
) -> snix_store::path_info::PathInfo {
    store
        .source_admission()
        .ingest(VerifiedSourceIngestRequest {
            source_path,
            logical_store_path: logical_path.to_str().unwrap(),
            source_name: name,
            signing_key: &key.signing_key,
        })
        .await
        .unwrap_or_else(|error| panic!("signing {name} source root failed: {error}"))
}

async fn run_case(
    root: &Path,
    example: &Example,
    altered_java: bool,
    reviewed: &Toolchain,
    runtime: &RuntimeCohort,
    archive_paths: &[PathBuf; 3],
    source_state_blake3: &str,
    static_busybox: &Path,
    keystore: &Path,
    password: &Path,
) -> Value {
    fs::create_dir(root).expect("fresh independent run directory");
    let prefix = root.join("store");
    fs::create_dir(&prefix).unwrap();
    let source_root = prefix.join(format!("{}-android-java-source", "f".repeat(32)));
    fs::create_dir(&source_root).unwrap();
    let java = if altered_java {
        let changed = example.java.replace("Hello, Mantle!", "Hello, Mantle, changed!");
        assert_ne!(changed, example.java, "Java perturbation must change exactly the authored source");
        changed
    } else {
        example.java.clone()
    };
    write_member(&source_root, &example.plan.manifest_member, example.manifest.as_bytes());
    assert_eq!(example.plan.java_sources.len(), 1);
    assert_eq!(example.plan.resources.len(), 1);
    write_member(&source_root, &example.plan.java_sources[0], java.as_bytes());
    write_member(&source_root, &example.plan.resources[0], example.resources_xml.as_bytes());

    let utility_root = prefix.join(format!("{}-apk-utility", "b".repeat(32)));
    fs::create_dir(&utility_root).unwrap();
    let utility = utility_root.join("busybox");
    fs::copy(static_busybox, &utility).unwrap();
    let runtime_glibc_root = prefix.join(format!("{}-runtime-glibc", "c".repeat(32)));
    let runtime_libgcc_root = prefix.join(format!("{}-runtime-libgcc", "d".repeat(32)));
    let keystore_root = prefix.join(format!("{}-proof-keystore", "g".repeat(32)));
    let password_root = prefix.join(format!("{}-proof-password", "h".repeat(32)));

    let prefix_text = prefix.to_str().unwrap();
    let blob = MemoryBlobService::default();
    let directory =
        RedbDirectoryService::new_temporary("official-apk".into(), RedbDirectoryServiceConfig::default()).unwrap();
    let pathinfo = LruPathInfoService::with_capacity("official-apk".into(), NonZeroUsize::new(256).unwrap());
    let sandbox = BubblewrapBuildService::new(root.join("sandbox"), blob.clone(), directory.clone());
    let mut store = StoreHandle::from_services_with_store_dir(
        crunch_store::StoreBackend::Snix,
        StoreHandleServices {
            blob_service: Arc::new(blob),
            directory_service: Arc::new(directory),
            pathinfo_service: Arc::new(pathinfo),
            remote_pathinfo: None,
            state_dir: root.join("state"),
            output_dir_str: prefix_text.into(),
            publishers: Vec::new(),
        },
        prefix_text.into(),
    )
    .expect("official APK Snix store fixture");
    let key = crunch_build::load_keypair(TEST_KEY).unwrap();
    let trust = crunch_build::build_trusted_keys(&key, None);
    admit_source(&mut store, &source_root, &source_root, "java-source", &key).await;
    admit_source(&mut store, &utility_root, &utility_root, "apk-utility", &key).await;
    for (origin, logical, expected, name) in [
        (GLIBC_ORIGIN, &runtime_glibc_root, &runtime.glibc, "runtime-glibc"),
        (LIBGCC_ORIGIN, &runtime_libgcc_root, &runtime.libgcc, "runtime-libgcc"),
    ] {
        assert_eq!(expected.source_cohort, origin);
        let admitted = admit_source(&mut store, Path::new(origin), logical, name, &key).await;
        assert_eq!(digest_hex(&admitted.nar_sha256), expected.nar_sha256, "{name} NAR SHA-256 drift");
        assert_eq!(admitted.nar_size, expected.nar_size, "{name} NAR size drift");
    }
    admit_source(&mut store, keystore, &keystore_root, "proof-keystore", &key).await;
    admit_source(&mut store, password, &password_root, "proof-password", &key).await;

    let PipelineStoreParts {
        build_store,
        action_results,
        slice_admission,
        build_service_store,
        output_lookup,
        root_registry: _,
    } = store.into_pipeline_store_parts();
    let overrides = [
        (&reviewed.build_tools, &archive_paths[0]),
        (&reviewed.jdk, &archive_paths[1]),
        (&reviewed.platform, &archive_paths[2]),
    ]
    .into_iter()
    .map(|(identity, payload_path)| FetchSourceOverride {
        url: identity.url.clone(),
        kind: FetchSourceOverrideKind::File,
        rev: None,
        payload_path: payload_path.clone(),
        source_state_blake3: source_state_blake3.into(),
    })
    .collect();
    let fetcher = FetchBuildService::new(build_service_store)
        .with_source_overrides(overrides)
        .with_source_policy(FetchSourcePolicy::RequireOverride);
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

    let mut archive_outputs = Vec::new();
    let mut archives = Vec::new();
    for identity in [&reviewed.build_tools, &reviewed.jdk, &reviewed.platform] {
        let outcome =
            realize(&mut builder, fetch_original(&format!("{}-reviewed", identity.component), identity), prefix_text)
                .await;
        assert!(!outcome.cached, "fresh store must build each reviewed FOD");
        let store_path = prefix.join(outcome.outputs["out"].store_path.to_string());
        archive_outputs
            .push(json!({"source_identity": identity, "drv": outcome.drv_path.to_string(), "store_path": store_path}));
        archives.push(BoundArchive {
            identity: identity.clone(),
            store_path,
        });
    }
    let [build_tools, jdk, platform]: [BoundArchive; 3] = archives.try_into().unwrap();
    let inputs = BuildInputs {
        source_root,
        build_tools,
        jdk,
        platform,
        runtime_glibc: BoundRuntime {
            identity: runtime.glibc.clone(),
            store_path: runtime_glibc_root,
        },
        runtime_libgcc: BoundRuntime {
            identity: runtime.libgcc.clone(),
            store_path: runtime_libgcc_root,
        },
        utility,
        keystore: Some(keystore_root),
        key_password_file: Some(password_root),
    };
    let prepared = PreparedApk::bind(example.plan.clone(), inputs, reviewed, runtime, &prefix).unwrap();
    let mut extractions = Vec::new();
    let mut extraction_outcomes = Vec::new();
    for extraction in prepared.extractions(&prefix).unwrap() {
        let outcome = realize(&mut builder, extraction_as_derivation(&extraction), prefix_text).await;
        assert!(!outcome.cached, "same-run extraction cannot use cached provenance");
        extractions.push(json!({"name": extraction.name, "drv": outcome.drv_path.to_string(), "store_path": prefix.join(outcome.outputs["out"].store_path.to_string()), "source_record_identity_blake3": extraction.record_identity_blake3, "archive_sha256": extraction.archive_sha256}));
        extraction_outcomes.push(outcome);
    }
    let extraction_outcomes: [BuildOutcome; 3] = extraction_outcomes.try_into().unwrap();
    let trees = verified_same_run_extractions(&prepared, &extraction_outcomes, &output_lookup, &trust, &prefix)
        .await
        .expect("signed same-run extraction and runtime source authority");
    let mut previous = BTreeMap::new();
    let mut steps = Vec::new();
    let mut apk_path = None;
    for step in prepared.steps() {
        let derivation = prepared.derivation(step.kind, &previous, &trees, &prefix).unwrap();
        for runtime_root in [
            &prepared.inputs.runtime_glibc.store_path,
            &prepared.inputs.runtime_libgcc.store_path,
        ] {
            assert!(derivation.inputs.contains(&runtime_root.display().to_string()));
        }
        let outcome = realize(&mut builder, tool_as_derivation(&derivation), prefix_text).await;
        assert!(!outcome.cached, "fresh store must execute every SDK step");
        let signed_output = outcome.outputs.get("out").unwrap();
        let info = output_lookup.find(&signed_output.store_path).await.unwrap().expect("SDK step PathInfo");
        assert_eq!(info.node, signed_output.node);
        assert_eq!(info.nar_sha256, signed_output.nar_sha256);
        assert!(crunch_build::verify_pathinfo_signatures_with_store_dir(&info, &trust, prefix_text).is_trusted());
        let output_dir = prefix.join(signed_output.store_path.to_string());
        let produced = output_dir.join(&step.output);
        assert!(produced.exists(), "missing real {:?} output", step.kind);
        if step.kind == StepKind::Apksigner {
            apk_path = Some(produced.clone());
        }
        steps.push(json!({"kind": format!("{:?}", step.kind), "drv": outcome.drv_path.to_string(), "store_path": output_dir, "output": produced, "output_nar_sha256": digest_hex(&signed_output.nar_sha256), "env": derivation.env, "inputs": derivation.inputs}));
        previous.insert(step.kind, output_dir);
    }
    assert_eq!(steps.len(), 6, "signed plan must execute all six SDK stages");
    json!({"altered_java": altered_java, "apk": apk_path.expect("signed APK stage"), "archive_fods": archive_outputs, "extractions": extractions, "steps": steps})
}

#[test]
#[ignore = "explicit connected-prefetch and reviewed SDK/signed-runtime proof prerequisites"]
fn reviewed_signed_java_apk_clean_rebuilds_and_perturbation() {
    assert!(has_bwrap(), "proof requires Bubblewrap/user namespaces");
    let run_root = env_path("APK_PROOF_RUN_DIR");
    let example: Example = serde_json::from_slice(&fs::read(env_path("APK_PROOF_EXAMPLE_JSON")).unwrap()).unwrap();
    let reviewed = reviewed_toolchain();
    assert_eq!(example.plan.toolchain, reviewed, "Nickel plan must equal independent reviewed SDK records");
    assert!(example.plan.signing.is_some(), "the example must request real signing");
    let runtime = reviewed_runtime();
    let archive_paths = [
        env_path("APK_PROOF_BUILD_TOOLS_ARCHIVE"),
        env_path("APK_PROOF_JDK_ARCHIVE"),
        env_path("APK_PROOF_PLATFORM_ARCHIVE"),
    ];
    let source_state_blake3 = std::env::var("APK_PROOF_SOURCE_STATE_BLAKE3").expect("offline preflight receipt digest");
    assert_eq!(source_state_blake3.len(), 64);
    assert!(source_state_blake3.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let static_busybox = env_path("SNIX_BUILD_SANDBOX_SHELL");
    assert!(static_busybox.is_file());
    let keystore = env_path("APK_PROOF_KEYSTORE");
    let password = env_path("APK_PROOF_PASSWORD_FILE");
    assert!(keystore.is_file() && password.is_file());
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let runs = rt.block_on(async {
        let mut runs = Vec::new();
        for (name, altered) in [("clean-one", false), ("clean-two", false), ("java-perturbed", true)] {
            eprintln!("APK_PROOF_REAL_BUILD_START {name}");
            let observation = run_case(
                &run_root.join(name),
                &example,
                altered,
                &reviewed,
                &runtime,
                &archive_paths,
                &source_state_blake3,
                &static_busybox,
                &keystore,
                &password,
            )
            .await;
            eprintln!("APK_PROOF_REAL_BUILD_DONE {name}");
            runs.push(json!({"run": name, "observation": observation}));
        }
        runs
    });
    let observation_path = run_root.join("builder-observations.json");
    fs::write(&observation_path, serde_json::to_vec_pretty(&json!({"source_state_blake3": source_state_blake3, "runtime": {"glibc": runtime.glibc.nar_sha256, "libgcc": runtime.libgcc.nar_sha256}, "runs": runs})).unwrap()).unwrap();
    eprintln!("APK_PROOF_OBSERVATIONS {}", observation_path.display());
}
