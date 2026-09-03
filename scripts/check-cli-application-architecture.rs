#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"
---

//! Enforce Mantle's CLI composition-root and application dependency direction.
// r[verify application_architecture.thin_composition_root]
// r[verify application_architecture.application_owned_ports]
// r[verify application_architecture.typed_error_ownership]
// r[verify application_architecture.effect_observation_boundary]
// r[verify application_architecture.dependency_guard]

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

const ROOT_SOURCE: &str = "src/main.rs";
const ROOT_PRODUCTION_LINES_MAX: usize = 450;
const CORE_ROOT: &str = "crates/mantle-application-core";
const APPLICATION_ROOT: &str = "crates/mantle-application";
const PORTS_SOURCE: &str = "crates/mantle-application/src/ports.rs";
const APPLICATION_SOURCE: &str = "crates/mantle-application/src/application.rs";
const COMPOSITION_SOURCE: &str = "src/cli_architecture.rs";
const INBOUND_SOURCE: &str = "src/cli_architecture/inbound_adapter.rs";
const OPERATION_ADAPTER_SOURCE: &str = "src/cli_architecture/operation_adapter.rs";
const PRESENTATION_SOURCE: &str = "src/cli_architecture/presentation_adapter.rs";
const ARCHITECTURE_CONFIG: &str = "config/cli-application-architecture.ncl";
const ROOT_MANIFEST: &str = "Cargo.toml";
const FLAKE_SOURCE: &str = "flake.nix";
const POSITIVE_FIXTURE: &str = "fixtures/cli-application-architecture/positive/mantle-contract.rs";
const NEGATIVE_FIXTURE_ROOT: &str = "fixtures/cli-application-architecture/negative";
const COMPILE_FAIL_FIXTURE: &str = "crates/mantle-application/src/lib.rs";
const ROOT_POLICY_FIXTURE: &str = "fixtures/cli-application-architecture/negative/root-policy.rs";
const RUST_EXTENSION: &str = "rs";
const MAX_SOURCE_FILES: usize = 128;
const MAX_SOURCE_BYTES: u64 = 4_194_304;
const MAX_FINDINGS: usize = 256;
const EXPECTED_PORT_COUNT: usize = 12;
const EXPECTED_NEGATIVE_FIXTURE_COUNT: usize = 13;
const EXPECTED_COMPILE_FAIL_FIXTURE_COUNT: usize = 2;
const ALLOWED_ROOT_FUNCTIONS: &[&str] = &["main", "init_tracing", "output_mode"];
const REQUIRED_PORTS: &[&str] = &[
    "BuildOperationPort",
    "RustPlanOperationPort",
    "RemoteOperationPort",
    "StoreOperationPort",
    "SourceOperationPort",
    "ReleaseOperationPort",
    "ProjectOperationPort",
    "BootstrapOperationPort",
    "ArtifactOperationPort",
    "EvaluationOperationPort",
    "PackageOperationPort",
    "UtilityOperationPort",
];
const CORE_FORBIDDEN_MARKERS: &[(&str, &str)] = &[
    ("clap::", "cli"),
    ("snix_", "snix"),
    ("std::fs", "filesystem"),
    ("std::path", "filesystem"),
    ("PathBuf", "filesystem"),
    ("std::process", "process"),
    ("Command::new", "process"),
    ("tokio", "async-runtime"),
    ("async_std", "async-runtime"),
    ("std::env", "environment"),
    ("env::var", "environment"),
    ("std::time", "clock"),
    ("SystemTime", "clock"),
    ("Instant", "clock"),
    ("rand::", "random"),
    ("getrandom", "random"),
    ("std::net", "network"),
    ("reqwest", "network"),
    ("TcpStream", "network"),
    ("provider_sdk", "provider"),
    ("aws_sdk", "provider"),
    ("gix::", "provider"),
    ("crunch_store", "store-service"),
    ("PathInfoService", "store-service"),
    ("BlobService", "store-service"),
    ("println!", "rendering"),
    ("eprintln!", "rendering"),
    ("tracing::", "rendering"),
    ("ratatui", "rendering"),
    ("RunError", "cli-error"),
];
const ROOT_FORBIDDEN_MARKERS: &[(&str, &str)] = &[
    ("std::fs", "filesystem-effect"),
    ("std::process::Command", "process-effect"),
    ("Command::new", "process-effect"),
    ("tokio::", "async-effect"),
    ("reqwest", "network-effect"),
    ("crunch_store", "store-policy"),
    ("snix_", "provider-translation"),
    ("serde_json", "receipt-or-presentation-policy"),
    ("blake3", "identity-policy"),
    ("fn command_root", "command-policy"),
    ("fn command_label", "command-policy"),
    ("fn dispatch_command", "effect-dispatch"),
    ("fn plan_", "domain-policy"),
    ("fn classify_", "domain-policy"),
    ("fn retry_", "retry-policy"),
    ("fn trust_", "trust-policy"),
];
const PRESENTATION_FORBIDDEN_MARKERS: &[(&str, &str)] = &[
    ("std::fs", "filesystem"),
    ("std::process", "process"),
    ("tokio", "async-runtime"),
    ("std::env", "environment"),
    ("std::time", "clock"),
    ("rand::", "random"),
    ("std::net", "network"),
    ("reqwest", "network"),
    ("snix_", "snix"),
    ("crunch_store", "store-service"),
    ("dispatch_command", "effect-execution"),
];
const NEGATIVE_FIXTURES: &[(&str, &str)] = &[
    ("cli.rs", "cli"),
    ("snix.rs", "snix"),
    ("filesystem.rs", "filesystem"),
    ("process.rs", "process"),
    ("async-runtime.rs", "async-runtime"),
    ("environment.rs", "environment"),
    ("clock.rs", "clock"),
    ("random.rs", "random"),
    ("network.rs", "network"),
    ("provider.rs", "provider"),
    ("store-service.rs", "store-service"),
    ("rendering.rs", "rendering"),
];

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct Finding {
    path: String,
    owner: String,
    authority: String,
    marker: String,
}

struct Inspection {
    findings: Vec<Finding>,
    root_lines: usize,
}

fn main() -> ExitCode {
    let root = configured_root().unwrap_or_else(|error| fail_now("parse-arguments", &error));
    let inspection = inspect_repository(&root).unwrap_or_else(|error| fail_now("inspect-repository", &error));
    let fixture_findings = inspect_fixtures(&root).unwrap_or_else(|error| fail_now("inspect-fixtures", &error));
    if inspection.findings.is_empty() {
        print_fixture_diagnostics(&fixture_findings);
        print_success(inspection.root_lines);
        return ExitCode::SUCCESS;
    }
    print_findings(&inspection.findings);
    ExitCode::FAILURE
}

fn configured_root() -> Result<PathBuf, String> {
    let mut root = env::current_dir().map_err(|error| format!("read current directory: {error}"))?;
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--self-test" => {}
            "--root" => {
                let value = arguments.next().ok_or_else(|| "--root requires a path".to_string())?;
                root = PathBuf::from(value);
            }
            _ => return Err(format!("unsupported argument: {argument}")),
        }
    }
    Ok(root)
}

fn inspect_repository(root: &Path) -> Result<Inspection, String> {
    let mut findings = Vec::new();
    let root_lines = inspect_root(root, &mut findings)?;
    inspect_core(root, &mut findings)?;
    inspect_application(root, &mut findings)?;
    inspect_adapters(root, &mut findings)?;
    inspect_contract_sources(root, &mut findings)?;
    findings.sort();
    findings.dedup();
    if findings.len() > MAX_FINDINGS {
        return Err(format!("finding count {} exceeds {MAX_FINDINGS}", findings.len()));
    }
    Ok(Inspection { findings, root_lines })
}

fn inspect_root(root: &Path, findings: &mut Vec<Finding>) -> Result<usize, String> {
    let source = read_bounded(&root.join(ROOT_SOURCE))?;
    let line_count = source.lines().count();
    if line_count > ROOT_PRODUCTION_LINES_MAX {
        findings.push(finding(ROOT_SOURCE, "composition-root", "root-size", line_count.to_string()));
    }
    require_markers(
        ROOT_SOURCE,
        "composition-root",
        &source,
        &[
            "let args = Args::parse();",
            "cli_inbound::has_conflicting_machine_output_modes(&args)",
            "init_tracing(&args);",
            "match run(args)",
            "error.format_json()",
            "error.format_human()",
        ],
        findings,
    );
    scan_text(ROOT_SOURCE, "composition-root", &source, ROOT_FORBIDDEN_MARKERS, findings);
    inspect_root_functions(&source, findings);
    Ok(line_count)
}

fn inspect_root_functions(source: &str, findings: &mut Vec<Finding>) {
    for line in source.lines() {
        let Some(name) = function_name(line) else {
            continue;
        };
        if !ALLOWED_ROOT_FUNCTIONS.contains(&name.as_str()) {
            findings.push(finding(ROOT_SOURCE, "composition-root", "root-responsibility", name));
        }
    }
}

fn function_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let marker_index = trimmed.find("fn ")?;
    let prefix = &trimmed[..marker_index];
    if !prefix.is_empty() && !prefix.starts_with("pub") && !prefix.starts_with("async") {
        return None;
    }
    let tail = &trimmed[marker_index + "fn ".len()..];
    let end = tail.find(['(', '<'])?;
    Some(tail[..end].to_string())
}

fn inspect_core(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    inspect_no_std_crate(root, CORE_ROOT, "dispatch-core", findings)?;
    let cargo = read_bounded(&root.join(CORE_ROOT).join("Cargo.toml"))?;
    if cargo.contains("path =") || cargo.contains("[features]") {
        findings.push(finding(CORE_ROOT, "dispatch-core", "core-dependency", "path-or-feature"));
    }
    require_markers(
        "crates/mantle-application-core/src/lib.rs",
        "dispatch-core",
        &read_bounded(&root.join(CORE_ROOT).join("src/lib.rs"))?,
        &[
            "pub enum ApplicationCoreError",
            "pub fn plan_dispatch",
            "pub fn classify_observation",
            "pub struct ApplicationEffect",
            "pub struct ApplicationObservation",
        ],
        findings,
    );
    Ok(())
}

fn inspect_application(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    inspect_no_std_crate(root, APPLICATION_ROOT, "application", findings)?;
    inspect_application_dependencies(root, findings)?;
    let ports = read_bounded(&root.join(PORTS_SOURCE))?;
    for required in REQUIRED_PORTS {
        let explicit_trait = format!("trait {required}");
        let macro_trait = format!("operation_port!({required},");
        if !ports.contains(&explicit_trait) && !ports.contains(&macro_trait) {
            findings.push(finding(PORTS_SOURCE, "application", "missing-port", *required));
        }
    }
    for forbidden in ["RunError", "clap", "snix_", "crunch_store", "PathInfoService"] {
        if ports.contains(forbidden) {
            findings.push(finding(PORTS_SOURCE, "application", "port-vendor-leak", forbidden));
        }
    }
    let application = read_bounded(&root.join(APPLICATION_SOURCE))?;
    require_markers(
        APPLICATION_SOURCE,
        "application",
        &application,
        &[
            "fn execute_effect",
            "classify_observation",
            "ApplicationPortError",
            "error.effect_id_blake3 != effect.effect_id_blake3",
        ],
        findings,
    );
    Ok(())
}

fn inspect_no_std_crate(root: &Path, crate_root: &str, owner: &str, findings: &mut Vec<Finding>) -> Result<(), String> {
    let lib_path = root.join(crate_root).join("src/lib.rs");
    let lib = read_bounded(&lib_path)?;
    if !lib.contains("#![no_std]") || !lib.contains("extern crate alloc") {
        findings.push(finding(crate_root, owner, "core-shape", "missing-no-std-or-alloc"));
    }
    let source_root = root.join(crate_root).join("src");
    scan_tree(root, &source_root, owner, CORE_FORBIDDEN_MARKERS, findings)
}

fn inspect_application_dependencies(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let path = root.join(APPLICATION_ROOT).join("Cargo.toml");
    let cargo = read_bounded(&path)?;
    let dependencies = manifest_dependency_lines(&cargo);
    let expected = "mantle-application-core = { path = \"../mantle-application-core\" }";
    if dependencies != [expected] {
        findings.push(finding(APPLICATION_ROOT, "application", "dependency-direction", dependencies.join(" | ")));
    }
    Ok(())
}

fn manifest_dependency_lines(manifest: &str) -> Vec<&str> {
    let Some(section) = manifest.split("[dependencies]").nth(1) else {
        return Vec::new();
    };
    section
        .lines()
        .take_while(|line| !line.trim_start().starts_with('['))
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

fn inspect_adapters(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    inspect_inbound_adapter(root, findings)?;
    inspect_operation_adapter(root, findings)?;
    inspect_presentation_adapter(root, findings)?;
    let composition = read_bounded(&root.join(COMPOSITION_SOURCE))?;
    require_markers(
        COMPOSITION_SOURCE,
        "composition-adapter",
        &composition,
        &[
            "mantle_application::run",
            "CliOperationAdapter",
            "presentation_adapter::finish",
        ],
        findings,
    );
    Ok(())
}

fn inspect_inbound_adapter(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let source = read_bounded(&root.join(INBOUND_SOURCE))?;
    require_markers(
        INBOUND_SOURCE,
        "inbound-adapter",
        &source,
        &[
            "ApplicationCommand",
            "request_blake3",
            "command_family",
            "mutation_class",
        ],
        findings,
    );
    for forbidden in [
        "RunError",
        "dispatch_command",
        "std::fs",
        "std::process",
        "reqwest",
        "crunch_store",
    ] {
        if source.contains(forbidden) {
            findings.push(finding(INBOUND_SOURCE, "inbound-adapter", "inbound-effect-leak", forbidden));
        }
    }
    Ok(())
}

fn inspect_operation_adapter(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let source = read_bounded(&root.join(OPERATION_ADAPTER_SOURCE))?;
    require_markers(
        OPERATION_ADAPTER_SOURCE,
        "operation-adapter",
        &source,
        &[
            "dispatch_command",
            "ApplicationObservation",
            "ApplicationPortError",
            "expected != effect.command",
            "ApplicationPortError::for_effect",
            "port_error",
        ],
        findings,
    );
    for required in REQUIRED_PORTS {
        if !source.contains(required) {
            findings.push(finding(
                OPERATION_ADAPTER_SOURCE,
                "operation-adapter",
                "missing-port-implementation",
                *required,
            ));
        }
    }
    Ok(())
}

fn inspect_presentation_adapter(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let source = read_bounded(&root.join(PRESENTATION_SOURCE))?;
    require_markers(
        PRESENTATION_SOURCE,
        "presentation-adapter",
        &source,
        &[
            "ApplicationOutcome",
            "ApplicationFailure",
            "RunError",
            "reported_exit_code",
        ],
        findings,
    );
    scan_text(PRESENTATION_SOURCE, "presentation-adapter", &source, PRESENTATION_FORBIDDEN_MARKERS, findings);
    Ok(())
}

fn inspect_contract_sources(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    inspect_architecture_config(root, findings)?;
    inspect_root_manifest(root, findings)?;
    inspect_flake(root, findings)
}

fn inspect_architecture_config(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let source = read_bounded(&root.join(ARCHITECTURE_CONFIG))?;
    require_markers(
        ARCHITECTURE_CONFIG,
        "architecture-contract",
        &source,
        &[
            "mantle-cli-application-architecture-v1",
            "production_lines_max = 450",
            "src/cli_inbound.rs",
            "crates/mantle-application-core/src/lib.rs",
            "crates/mantle-application/src/application.rs",
            "src/cli_architecture/operation_adapter.rs",
            "src/cli_architecture/presentation_adapter.rs",
            "__evaluator-worker",
            "__evaluator-worker-fixture",
            "__remote-secret-worker",
        ],
        findings,
    );
    for port in REQUIRED_PORTS {
        if !source.contains(port) {
            findings.push(finding(ARCHITECTURE_CONFIG, "architecture-contract", "missing-port", *port));
        }
    }
    Ok(())
}

fn inspect_root_manifest(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let source = read_bounded(&root.join(ROOT_MANIFEST))?;
    require_markers(
        ROOT_MANIFEST,
        "workspace",
        &source,
        &[
            "\"crates/mantle-application-core\"",
            "\"crates/mantle-application\"",
            "mantle-application-core = { path = \"crates/mantle-application-core\" }",
            "mantle-application = { path = \"crates/mantle-application\" }",
        ],
        findings,
    );
    Ok(())
}

fn inspect_flake(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let source = read_bounded(&root.join(FLAKE_SOURCE))?;
    require_markers(
        FLAKE_SOURCE,
        "nix-checks",
        &source,
        &[
            "cliApplicationArchitectureCheck",
            "scripts/check-cli-application-architecture.rs --self-test",
            "scripts/check-cli-application-architecture.rs --root .",
            "application-core = craneLib.cargoTest",
            "-p mantle-application-core -p mantle-application --all-targets",
            "application-core-wasm = craneLib.cargoBuild",
            "-p mantle-application-core -p mantle-application --lib --target wasm32-unknown-unknown",
            "cli-application-architecture = cliApplicationArchitectureCheck",
        ],
        findings,
    );
    Ok(())
}

fn inspect_fixtures(root: &Path) -> Result<Vec<Finding>, String> {
    if NEGATIVE_FIXTURES.len() + 1 != EXPECTED_NEGATIVE_FIXTURE_COUNT {
        return Err("negative fixture declaration count drifted".to_string());
    }
    let positive = read_bounded(&root.join(POSITIVE_FIXTURE))?;
    let mut positive_findings = Vec::new();
    scan_text(POSITIVE_FIXTURE, "positive-fixture", &positive, CORE_FORBIDDEN_MARKERS, &mut positive_findings);
    if !positive_findings.is_empty() {
        return Err(format!("positive fixture produced findings: {positive_findings:?}"));
    }
    let mut expected_findings = Vec::with_capacity(EXPECTED_NEGATIVE_FIXTURE_COUNT);
    for (name, expected_authority) in NEGATIVE_FIXTURES {
        expected_findings.push(require_fixture_finding(root, name, expected_authority)?);
    }
    expected_findings.push(require_finding_with_markers(
        root,
        ROOT_POLICY_FIXTURE,
        "domain-policy",
        ROOT_FORBIDDEN_MARKERS,
    )?);
    inspect_compile_fail_fixture(root)?;
    Ok(expected_findings)
}

fn require_fixture_finding(root: &Path, name: &str, expected_authority: &str) -> Result<Finding, String> {
    let relative = format!("{NEGATIVE_FIXTURE_ROOT}/{name}");
    let text = read_bounded(&root.join(&relative))?;
    let mut findings = Vec::new();
    scan_text(&relative, "negative-fixture", &text, CORE_FORBIDDEN_MARKERS, &mut findings);
    findings
        .into_iter()
        .find(|finding| finding.authority == expected_authority)
        .ok_or_else(|| format!("negative fixture {name} did not report {expected_authority}"))
}

fn require_finding_with_markers(
    root: &Path,
    relative: &str,
    expected_authority: &str,
    markers: &[(&str, &str)],
) -> Result<Finding, String> {
    let text = read_bounded(&root.join(relative))?;
    let mut findings = Vec::new();
    scan_text(relative, "negative-fixture", &text, markers, &mut findings);
    findings
        .into_iter()
        .find(|finding| finding.authority == expected_authority)
        .ok_or_else(|| format!("negative fixture {relative} did not report {expected_authority}"))
}

fn inspect_compile_fail_fixture(root: &Path) -> Result<(), String> {
    let source = read_bounded(&root.join(COMPILE_FAIL_FIXTURE))?;
    let compile_fail_count = source.matches("```compile_fail").count();
    if compile_fail_count != EXPECTED_COMPILE_FAIL_FIXTURE_COUNT
        || !source.contains("snix_store::")
        || !source.contains("BuildOnly")
        || !source.contains("BuildOperationPort")
        || !source.contains("mantle_application::run")
    {
        return Err("application compile-fail fixture is incomplete".to_string());
    }
    Ok(())
}

fn require_markers(path: &str, owner: &str, source: &str, required: &[&str], findings: &mut Vec<Finding>) {
    for marker in required {
        if !source.contains(marker) {
            findings.push(finding(path, owner, "missing-required-structure", *marker));
        }
    }
}

fn scan_tree(
    repository_root: &Path,
    source_root: &Path,
    owner: &str,
    markers: &[(&str, &str)],
    findings: &mut Vec<Finding>,
) -> Result<(), String> {
    for source in rust_sources(source_root)? {
        if source.file_name().and_then(|name| name.to_str()) == Some("tests.rs") {
            continue;
        }
        let relative = relative_string(repository_root, &source)?;
        let text = production_source(&read_bounded(&source)?);
        scan_text(&relative, owner, &text, markers, findings);
    }
    Ok(())
}

fn production_source(source: &str) -> String {
    let without_test_std = source.replace("#[cfg(test)]\nextern crate std;", "");
    let Some((prefix, raw_documentation)) = without_test_std.split_once("#![doc = r#\"") else {
        return without_test_std;
    };
    let Some((_, suffix)) = raw_documentation.split_once("\"#]") else {
        return without_test_std;
    };
    format!("{prefix}{suffix}")
}

fn scan_text(path: &str, owner: &str, text: &str, markers: &[(&str, &str)], findings: &mut Vec<Finding>) {
    let compact: String = text.chars().filter(|character| !character.is_whitespace()).collect();
    for (marker, authority) in markers {
        let compact_marker: String = marker.chars().filter(|character| !character.is_whitespace()).collect();
        if compact.contains(&compact_marker) {
            findings.push(finding(path, owner, *authority, *marker));
        }
    }
    scan_grouped_std_imports(path, owner, &compact, findings);
}

fn scan_grouped_std_imports(path: &str, owner: &str, compact: &str, findings: &mut Vec<Finding>) {
    let mut remainder = compact;
    while let Some(start) = remainder.find("std::{") {
        let group = &remainder[start + "std::{".len()..];
        let Some(end) = group.find('}') else {
            return;
        };
        for item in group[..end].split(',') {
            let authority = grouped_std_authority(item);
            if let Some(authority) = authority {
                findings.push(finding(path, owner, authority, format!("std::{{{item}}}")));
            }
        }
        remainder = &group[end + 1..];
    }
}

fn grouped_std_authority(item: &str) -> Option<&'static str> {
    if item.starts_with("fs") || item.starts_with("path") {
        return Some("filesystem");
    }
    if item.starts_with("process") {
        return Some("process");
    }
    if item.starts_with("env") {
        return Some("environment");
    }
    if item.starts_with("time") {
        return Some("clock");
    }
    if item.starts_with("net") {
        return Some("network");
    }
    None
}

fn rust_sources(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut pending = vec![root.to_path_buf()];
    let mut sources = Vec::new();
    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory).map_err(|error| format!("read {}: {error}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("read entry in {}: {error}", directory.display()))?;
            let file_type =
                entry.file_type().map_err(|error| format!("inspect {}: {error}", entry.path().display()))?;
            if file_type.is_symlink() {
                return Err(format!("symlink is not admitted: {}", entry.path().display()));
            }
            if file_type.is_dir() {
                pending.push(entry.path());
                continue;
            }
            if entry.path().extension().and_then(|value| value.to_str()) == Some(RUST_EXTENSION) {
                sources.push(entry.path());
                if sources.len() > MAX_SOURCE_FILES {
                    return Err(format!("source file count exceeds {MAX_SOURCE_FILES}"));
                }
            }
        }
    }
    sources.sort();
    Ok(sources)
}

fn read_bounded(path: &Path) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| format!("inspect {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_SOURCE_BYTES {
        return Err(format!("source file shape or size is invalid: {}", path.display()));
    }
    fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn relative_string(root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(root)
        .map(|relative| relative.to_string_lossy().into_owned())
        .map_err(|error| format!("make relative path {}: {error}", path.display()))
}

fn finding(
    path: impl Into<String>,
    owner: impl Into<String>,
    authority: impl Into<String>,
    marker: impl Into<String>,
) -> Finding {
    Finding {
        path: path.into(),
        owner: owner.into(),
        authority: authority.into(),
        marker: marker.into(),
    }
}

fn print_fixture_diagnostics(findings: &[Finding]) {
    for finding in findings {
        println!(
            "negative fixture rejected: path={} owner={} authority={} dependency-path={}",
            finding.path, finding.owner, finding.authority, finding.marker
        );
    }
}

fn print_success(root_lines: usize) {
    debug_assert_eq!(REQUIRED_PORTS.len(), EXPECTED_PORT_COUNT);
    println!(
        "CLI application architecture verified: findings=0 root-production-lines={root_lines} ports={} negative-fixtures={} compile-fail-fixtures={}",
        REQUIRED_PORTS.len(),
        EXPECTED_NEGATIVE_FIXTURE_COUNT,
        EXPECTED_COMPILE_FAIL_FIXTURE_COUNT
    );
    println!(
        "owners: core={CORE_ROOT} application={APPLICATION_ROOT} inbound={INBOUND_SOURCE} operation={OPERATION_ADAPTER_SOURCE} presentation={PRESENTATION_SOURCE}"
    );
    println!(
        "boundaries: core-purity application-owned-ports adapter-direction explicit-composition typed-error-ownership presentation-separation no-std-targets"
    );
}

fn print_findings(findings: &[Finding]) {
    for finding in findings.iter().take(MAX_FINDINGS) {
        eprintln!(
            "{}: owner={} authority={} dependency-path={}",
            finding.path, finding.owner, finding.authority, finding.marker
        );
    }
}

fn fail_now(stage: &str, reason: &str) -> ! {
    eprintln!("CLI application architecture check failed during {stage}: {reason}");
    std::process::exit(1)
}
