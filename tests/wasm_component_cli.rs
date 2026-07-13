use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use assert_cmd::Command;
use crunch_wasm_component::ComponentPipelineRequest;
use crunch_wasm_component::CompositionDependency;
use crunch_wasm_component::CompositionDependencySource;
use crunch_wasm_component::PIPELINE_REQUEST_SCHEMA;
use crunch_wasm_component::PipelineExecutionReport;
use crunch_wasm_component::ToolRecord;
use crunch_wasm_component::ToolchainManifest;
use crunch_wasm_component_core::AotConfig;
use crunch_wasm_component_core::AotMode;
use crunch_wasm_component_core::Blake3Identity;
use crunch_wasm_component_core::ComponentManifest;
use crunch_wasm_component_core::Composition;
use crunch_wasm_component_core::CompositionNode;
use crunch_wasm_component_core::GeneratedInputCandidate;
use crunch_wasm_component_core::GeneratedInputOwner;
use crunch_wasm_component_core::OciProtocol;
use crunch_wasm_component_core::OutputClass;
use crunch_wasm_component_core::OutputDeclaration;
use crunch_wasm_component_core::PackageKind;
use crunch_wasm_component_core::PackageMaterialization;
use crunch_wasm_component_core::PackageRequirement;
use crunch_wasm_component_core::PackageResolution;
use crunch_wasm_component_core::RegistryBackend;
use crunch_wasm_component_core::RegistryMapping;
use crunch_wasm_component_core::RustImplementation;
use crunch_wasm_component_core::RustProfile;
use crunch_wasm_component_core::StoreObject;
use crunch_wasm_component_core::ToolCohort;
use crunch_wasm_component_core::ToolIdentity;
use crunch_wasm_component_core::ValidationProfiles;
use crunch_wasm_component_core::VirtualizationConfig;
use crunch_wasm_component_core::VirtualizationMode;
use crunch_wasm_component_core::VirtualizationRule;
use crunch_wasm_component_core::WasiSubsystem;
use crunch_wasm_component_core::WitSelection;
use crunch_wasm_component_core::WizerConfig;
use crunch_wasm_component_core::WizerMode;
use sha2::Digest;
use sha2::Sha256;

const PACKAGE_VERSION: &str = "0.1.0";
const EXPECTED_STDOUT: &str = "mantle wasm production\n";
const TOOLCHAIN_ENV: &str = "MANTLE_WASM_COMPONENT_TOOLCHAIN";
const REQUIRED_RECEIPT_TOOL_COUNT: usize = 7;

struct CliFixture {
    _temp: tempfile::TempDir,
    request: ComponentPipelineRequest,
}

impl CliFixture {
    fn new() -> Self {
        let toolchain_root = PathBuf::from(std::env::var_os(TOOLCHAIN_ENV).expect("set pinned toolchain path"));
        let toolchain_manifest_path = toolchain_root.join("share/mantle/wasm-component-toolchain.json");
        let toolchain: ToolchainManifest =
            serde_json::from_slice(&fs::read(&toolchain_manifest_path).unwrap()).unwrap();
        let temp = tempfile::tempdir().unwrap();
        let registry = temp.path().join("registry");
        let source = temp.path().join("source");
        fs::create_dir(&registry).unwrap();
        create_source(&source);
        let package_path = prepare_registry_and_lock(&toolchain_root, temp.path(), &registry, &source);
        let package_object = file_object(&package_path);
        let protocol_digest = crunch_wasm_component_core::OciSha256Digest::parse(format!(
            "sha256:{:x}",
            Sha256::digest(fs::read(&package_path).unwrap())
        ))
        .unwrap();
        let source_object = source_object(&source, temp.path());
        let mut request = request(
            &toolchain_root,
            &toolchain_manifest_path,
            &toolchain,
            &source,
            &registry,
            source_object,
            package_object.clone(),
            protocol_digest,
        );
        request.generated_inputs = generated_inputs(&request, &package_object.digest_blake3);
        Self { _temp: temp, request }
    }

    fn run(&self, request: &ComponentPipelineRequest, label: &str) -> (std::process::Output, PathBuf) {
        let request_path = self._temp.path().join(format!("{label}.ncl"));
        let output = self._temp.path().join(format!("{label}-output"));
        let store = self._temp.path().join(format!("{label}-store"));
        let state = self._temp.path().join(format!("{label}-state"));
        write_pipeline_request_expression(&request_path, request);
        let command_output = Command::cargo_bin("mantle")
            .unwrap()
            .args(["--json", "--store"])
            .arg(&store)
            .arg("--state-dir")
            .arg(&state)
            .args(["wasm-component", "build"])
            .arg(&request_path)
            .arg("--out")
            .arg(&output)
            .output()
            .unwrap();
        (command_output, output)
    }
}

#[test]
fn production_cli_executes_pinned_pipeline_and_persists_octet_blocker() {
    let fixture = CliFixture::new();
    let (command_output, output) = fixture.run(&fixture.request, "positive");

    assert!(!command_output.status.success(), "Octet absence must keep consumer admission fail-closed");
    let report = report_from_output(&command_output);
    assert_eq!(report.final_status, "blocked");
    assert!(
        report.blockers.iter().any(|blocker| blocker.code == "octet-wasm-artifact-rail-unavailable"),
        "pipeline stopped before the authoritative Octet blocker: blockers={:?}\nstderr={}",
        report.blockers,
        String::from_utf8_lossy(&command_output.stderr)
    );
    assert!(report.materialization_bundle.is_none());
    assert!(report.component_report.is_some());
    assert!(output.join("execution-report.json").is_file());
    assert!(output.join("artifacts/compiled.wasm").is_file());
    assert!(output.join("artifacts/composed.wasm").is_file());
    assert!(output.join("artifacts/virtualized.wasm").is_file());
    assert!(!output.join("source").exists(), "work source must not enter the bounded publication");
    assert!(!output.join("target").exists(), "compiler scratch must not enter the bounded publication");
    assert_required_tools_executed(&report);
    let stderr = String::from_utf8_lossy(&command_output.stderr);
    assert!(stderr.contains("authoritative blocker"), "{stderr}");
}

#[test]
fn production_cli_fails_closed_on_identity_interface_composition_and_runtime_drift() {
    let fixture = CliFixture::new();

    let mut tool_drift = fixture.request.clone();
    tool_drift.manifest.cohort.wkg.executable.digest_blake3 = Blake3Identity::from_slice(b"tampered-wkg");
    let (output, evidence) = fixture.run(&tool_drift, "tool-drift");
    assert_preflight_rejected(&output, &evidence, "declared component cohort drifts from pinned tool `wkg`");

    let mut package_drift = fixture.request.clone();
    package_drift.package_materializations[0].object.digest_blake3 = Blake3Identity::from_slice(b"tampered-package");
    let (output, evidence) = fixture.run(&package_drift, "package-drift");
    assert_preflight_rejected(&output, &evidence, "package materialization bytes or digest drifted");

    let mut composition_drift = fixture.request.clone();
    composition_drift.composition_dependencies[0].package = "missing:component".to_string();
    let (output, _) = fixture.run(&composition_drift, "composition-drift");
    let report = report_from_output(&output);
    assert!(report.blockers.iter().any(|blocker| {
        blocker.code == "pipeline-execution-error"
            && blocker.message.contains("composition dependency bindings do not exactly match")
    }));

    let mut interface_drift = fixture.request.clone();
    interface_drift.manifest.virtualization.expected_remaining_imports.clear();
    let (output, evidence) = fixture.run(&interface_drift, "interface-drift");
    let report = report_from_output(&output);
    assert!(report.blockers.iter().any(|blocker| blocker.code == "remaining-import-drift"));
    assert!(
        !evidence.join("artifacts/virtualized.wasm").exists(),
        "interface-mismatched portable bytes must not be published"
    );

    let mut runtime_drift = fixture.request.clone();
    runtime_drift.expected_runtime_stdout = "unexpected runtime output\n".to_string();
    let (output, _) = fixture.run(&runtime_drift, "runtime-drift");
    let report = report_from_output(&output);
    assert!(report.blockers.iter().any(|blocker| blocker.code == "wasmtime-stdout-mismatch"));
}

fn report_from_output(output: &std::process::Output) -> PipelineExecutionReport {
    assert!(!output.status.success(), "fail-closed pipeline invocation unexpectedly succeeded");
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "pipeline stdout was not a report: {error}\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn assert_preflight_rejected(output: &std::process::Output, evidence: &Path, expected: &str) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "preflight rejection must not emit an admitted report");
    assert!(!evidence.exists(), "preflight rejection must not publish evidence");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(expected), "unexpected preflight diagnostic: {stderr}");
}

fn create_source(source: &Path) {
    fs::create_dir_all(source.join("src")).unwrap();
    fs::create_dir_all(source.join("wit")).unwrap();
    fs::write(
        source.join("Cargo.toml"),
        "[package]\nname = \"mantle-wasm-production-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    fs::write(
        source.join("Cargo.lock"),
        "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"mantle-wasm-production-fixture\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(source.join("src/main.rs"), format!("fn main() {{ println!(\"{}\"); }}\n", EXPECTED_STDOUT.trim()))
        .unwrap();
    fs::write(
        source.join("wit/app.wit"),
        "package demo:app@0.1.0;\n\nworld app {\n  import demo:dep/add@0.1.0;\n}\n",
    )
    .unwrap();
}

fn prepare_registry_and_lock(toolchain: &Path, root: &Path, registry: &Path, source: &Path) -> PathBuf {
    let dependency = root.join("dependency");
    let cache = root.join("wkg-cache");
    let config = root.join("wkg-config.toml");
    fs::create_dir(&dependency).unwrap();
    fs::create_dir(&cache).unwrap();
    fs::write(
        dependency.join("dep.wit"),
        "package demo:dep@0.1.0;\n\ninterface add {\n  add: func(a: u32, b: u32) -> u32;\n}\n",
    )
    .unwrap();
    fs::write(
        &config,
        format!(
            "default_registry = \"local\"\n\n[namespace_registries]\n\"demo\" = \"local\"\n\n[registry.\"local\"]\ndefault = \"local\"\n[registry.\"local\".local]\nroot = {}\n",
            serde_json::to_string(&registry.display().to_string()).unwrap()
        ),
    )
    .unwrap();
    let package = root.join("demo-dep.wasm");
    run_setup_tool(toolchain, &dependency, &[
        "wit",
        "build",
        "--config",
        config.to_str().unwrap(),
        "--cache",
        cache.to_str().unwrap(),
        "--wit-dir",
        dependency.to_str().unwrap(),
        "--output",
        package.to_str().unwrap(),
    ]);
    run_setup_tool(toolchain, root, &[
        "publish",
        "--config",
        config.to_str().unwrap(),
        "--cache",
        cache.to_str().unwrap(),
        "--registry",
        "local",
        package.to_str().unwrap(),
    ]);
    run_setup_tool(toolchain, source, &[
        "wit",
        "fetch",
        "--config",
        config.to_str().unwrap(),
        "--cache",
        cache.to_str().unwrap(),
        "--wit-dir",
        source.join("wit").to_str().unwrap(),
        "--type",
        "wasm",
    ]);
    fs::remove_dir_all(source.join("wit/deps")).unwrap();
    registry.join(format!("demo/dep/{PACKAGE_VERSION}.wasm"))
}

fn run_setup_tool(toolchain: &Path, cwd: &Path, args: &[&str]) {
    let wkg = toolchain.join("bin/wkg");
    let output = ProcessCommand::new(&wkg).args(args).current_dir(cwd).env_clear().output().unwrap();
    assert!(output.status.success(), "wkg fixture setup failed: {}", String::from_utf8_lossy(&output.stderr));
}

#[allow(clippy::too_many_arguments)]
fn request(
    toolchain_root: &Path,
    toolchain_manifest_path: &Path,
    toolchain: &ToolchainManifest,
    source: &Path,
    registry: &Path,
    source_object: StoreObject,
    package_object: StoreObject,
    protocol_digest: crunch_wasm_component_core::OciSha256Digest,
) -> ComponentPipelineRequest {
    let cohort = cohort(toolchain_root, toolchain);
    let pending = StoreObject {
        logical_path: "/mantle/pending/compiled.wasm".to_string(),
        digest_blake3: Blake3Identity::from_slice(b"pending-component"),
        size_bytes: 1,
    };
    let wit_source = file_object(&source.join("wit/app.wit"));
    let profile = Blake3Identity::from_slice(b"fixture-profile");
    let expected_imports = expected_virtualized_imports();
    ComponentPipelineRequest {
        schema: PIPELINE_REQUEST_SCHEMA.to_string(),
        manifest: ComponentManifest {
            schema: crunch_wasm_component_core::COMPONENT_MANIFEST_SCHEMA.to_string(),
            name: "mantle-wasm-production-fixture".to_string(),
            package_resolution: PackageResolution {
                default_registry: "local".to_string(),
                registries: vec![RegistryMapping {
                    namespace: "demo".to_string(),
                    registry: "local".to_string(),
                    backend: RegistryBackend::Local,
                    oci_registry: None,
                    namespace_prefix: String::new(),
                    protocol: OciProtocol::Https,
                    local_root: Some(registry.display().to_string()),
                    credential_handle: None,
                }],
                requirements: vec![PackageRequirement {
                    package: "demo:dep".to_string(),
                    requirement: format!("={PACKAGE_VERSION}"),
                    registry: "local".to_string(),
                    kind: PackageKind::Wit,
                }],
                local_overrides: Vec::new(),
                lock_path: "wkg.lock".to_string(),
            },
            wit: WitSelection {
                package: "demo:app".to_string(),
                world: "app".to_string(),
                source: wit_source,
            },
            implementation: RustImplementation {
                source: source_object,
                package: "mantle-wasm-production-fixture".to_string(),
                crate_name: "mantle_wasm_production_fixture".to_string(),
                features: Vec::new(),
                profile: RustProfile::Release,
            },
            cohort,
            composition: Composition {
                package: "demo:composition".to_string(),
                source_wac: "package demo:composition;\nlet app = new root:component { ... };\nexport app.run;\n"
                    .to_string(),
                nodes: vec![CompositionNode {
                    id: "app".to_string(),
                    package: "root:component".to_string(),
                    world: "root".to_string(),
                    artifact: pending,
                    required_imports: Vec::new(),
                }],
                edges: Vec::new(),
                output_world: "root".to_string(),
                import_dependencies: false,
            },
            virtualization: VirtualizationConfig {
                defaults_overridden: true,
                rules: vec![VirtualizationRule {
                    subsystem: WasiSubsystem::Stdio,
                    mode: VirtualizationMode::Allow,
                    value: None,
                    value_identity_blake3: None,
                    input: None,
                    guest_path: None,
                    review_id: Some("fixture-reviewed-stdio".to_string()),
                }],
                expected_remaining_imports: expected_imports,
            },
            validation_profiles: ValidationProfiles {
                octet_artifact_profile_identity_blake3: profile.clone(),
                expected_runtime_profile_identity_blake3: profile,
            },
            wizer: WizerConfig {
                mode: WizerMode::Disabled,
                initialization_entrypoint: None,
                deterministic_virtual_imports: Vec::new(),
            },
            aot: AotConfig {
                mode: AotMode::Disabled,
                target: None,
                cpu_features: Vec::new(),
                wasmtime_configuration_identity_blake3: None,
            },
            outputs: vec![OutputDeclaration {
                name: "component".to_string(),
                path: "component.wasm".to_string(),
                class: OutputClass::ValidatedPortableComponent,
            }],
            non_claims: vec![
                "not-component-behavior-correctness".to_string(),
                "not-runtime-authority".to_string(),
                "not-runtime-sandboxing".to_string(),
                "not-release-eligibility".to_string(),
                "not-octet-policy-interpretation".to_string(),
            ],
        },
        generated_inputs: Vec::new(),
        toolchain_manifest: toolchain_manifest_path.display().to_string(),
        source_root: source.display().to_string(),
        wit_relative_path: "wit".to_string(),
        package_materializations: vec![PackageMaterialization {
            package: "demo:dep".to_string(),
            version: PACKAGE_VERSION.to_string(),
            registry: "local".to_string(),
            protocol_digest: protocol_digest.clone(),
            object: package_object,
        }],
        cargo_component_relative_path: "wasm32-wasip2/release/mantle-wasm-production-fixture.wasm".to_string(),
        composition_dependencies: vec![CompositionDependency {
            package: "root:component".to_string(),
            source: CompositionDependencySource::CompiledComponent,
        }],
        package_resolution_network: false,
        expected_runtime_stdout: EXPECTED_STDOUT.to_string(),
        aot_target: "x86_64-unknown-linux-gnu".to_string(),
        aot_cpu_features: Vec::new(),
        aot_configuration_args: Vec::new(),
    }
}

fn cohort(toolchain_root: &Path, toolchain: &ToolchainManifest) -> ToolCohort {
    ToolCohort {
        rust_toolchain: tool_identity(toolchain_root, toolchain, "rustc"),
        rust_target: "wasm32-wasip2".to_string(),
        wkg: tool_identity(toolchain_root, toolchain, "wkg"),
        wit_bindgen: tool_identity(toolchain_root, toolchain, "wit-bindgen"),
        wasm_component_ld: tool_identity(toolchain_root, toolchain, "wasm-component-ld"),
        wasm_tools: tool_identity(toolchain_root, toolchain, "wasm-tools"),
        wac: tool_identity(toolchain_root, toolchain, "wac"),
        wasi_virt: tool_identity(toolchain_root, toolchain, "wasi-virt"),
        wizer: tool_identity(toolchain_root, toolchain, "wizer"),
        wasmtime: tool_identity(toolchain_root, toolchain, "wasmtime"),
    }
}

fn tool_identity(root: &Path, manifest: &ToolchainManifest, name: &str) -> ToolIdentity {
    let record: &ToolRecord = manifest.tools.iter().find(|record| record.name == name).unwrap();
    let path = root.join(&record.path);
    ToolIdentity {
        version: record.version.clone(),
        executable: StoreObject {
            logical_path: path.display().to_string(),
            digest_blake3: record.binary_digest_blake3.clone(),
            size_bytes: fs::metadata(path).unwrap().len(),
        },
        configuration_identity_blake3: record.binary_digest_blake3.clone(),
    }
}

fn file_object(path: &Path) -> StoreObject {
    let bytes = fs::read(path).unwrap();
    StoreObject {
        logical_path: path.display().to_string(),
        digest_blake3: Blake3Identity::from_slice(&bytes),
        size_bytes: u64::try_from(bytes.len()).unwrap(),
    }
}

fn source_object(source: &Path, root: &Path) -> StoreObject {
    let copy = root.join("source-hash-copy");
    let object = crunch_wasm_component::copy_source_tree(source, &copy).unwrap();
    fs::remove_dir_all(copy).unwrap();
    object
}

fn write_pipeline_request_expression(path: &Path, request: &ComponentPipelineRequest) {
    let request_json = serde_json::to_string(request).unwrap();
    let nickel_string = serde_json::to_string(&request_json).unwrap();
    fs::write(path, format!("std.deserialize 'Json {nickel_string}\n")).unwrap();
}

fn generated_inputs(request: &ComponentPipelineRequest, dependency: &Blake3Identity) -> Vec<GeneratedInputCandidate> {
    let source = request.manifest.implementation.source.digest_blake3.clone();
    let registry = request.manifest.package_resolution.registries.first().unwrap();
    let local_root = registry.local_root.as_ref().unwrap();
    let wkg_config = format!(
        "default_registry = \"local\"\n\n[namespace_registries]\n\"demo\" = \"local\"\n\n[registry.\"local\"]\ndefault = \"local\"\n[registry.\"local\".local]\nroot = {}\n",
        serde_json::to_string(local_root).unwrap()
    );
    [
        (
            "wkg-manifest",
            ".mantle/wasm-component/wkg.toml",
            OutputClass::ToolInput,
            "[overrides]\n".to_string(),
        ),
        ("wkg-config", ".mantle/wasm-component/wkg-config.toml", OutputClass::ToolInput, wkg_config),
        (
            "wac-source",
            ".mantle/wasm-component/composition.wac",
            OutputClass::CompositionInput,
            request.manifest.composition.source_wac.clone(),
        ),
        ("command-plan", ".mantle/wasm-component/command-plan.json", OutputClass::ToolInput, "{}".to_string()),
        (
            "runtime-profile",
            ".mantle/wasm-component/runtime-profile.json",
            OutputClass::RuntimeProfile,
            "{}".to_string(),
        ),
    ]
    .into_iter()
    .map(|(name, target, output_class, content)| GeneratedInputCandidate {
        name: name.to_string(),
        target: target.to_string(),
        output_class,
        owner: GeneratedInputOwner {
            schema: crunch_wasm_component_core::GENERATED_INPUT_RECEIPT_SCHEMA.to_string(),
            generator: crunch_wasm_component_core::GENERATED_INPUT_OWNER.to_string(),
            export_name: name.to_string(),
            source_identity_blake3: source.clone(),
            dependency_identity_blake3: dependency.clone(),
        },
        content,
    })
    .collect()
}

fn expected_virtualized_imports() -> Vec<String> {
    [
        "wasi:clocks/wall-clock@0.2.3",
        "wasi:io/error@0.2.3",
        "wasi:io/poll@0.2.3",
        "wasi:io/streams@0.2.3",
        "wasi:cli/stdin@0.2.3",
        "wasi:cli/stdout@0.2.3",
        "wasi:cli/stderr@0.2.3",
        "wasi:cli/terminal-input@0.2.3",
        "wasi:cli/terminal-output@0.2.3",
        "wasi:cli/terminal-stdin@0.2.3",
        "wasi:cli/terminal-stdout@0.2.3",
        "wasi:cli/terminal-stderr@0.2.3",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn assert_required_tools_executed(report: &PipelineExecutionReport) {
    let tools = [
        "wkg",
        "wit-bindgen",
        "cargo",
        "wac",
        "wasi-virt",
        "wasm-tools",
        "wasmtime",
    ];
    assert_eq!(tools.len(), REQUIRED_RECEIPT_TOOL_COUNT);
    for tool in tools {
        assert!(
            report.stage_receipts.iter().any(|receipt| receipt.program.ends_with(&format!("/bin/{tool}"))),
            "missing production receipt for {tool}"
        );
    }
    assert!(
        report
            .stage_receipts
            .iter()
            .all(|receipt| !receipt.read_only_inputs.is_empty() || receipt.program.ends_with("/bin/cargo"))
    );
}
