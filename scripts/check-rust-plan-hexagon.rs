#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"
---

//! Enforce the Rust package-planning functional-core and adapter boundary.
// r[verify rust_package_planning.hexagonal_core]
// r[verify rust_package_planning.application_owned_ports]
// r[verify rust_package_planning.explicit_unit_effects]
// r[verify rust_package_planning.hexagonal_compatibility]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const CORE_ROOT: &str = "crates/mantle-rust-plan-core";
const APPLICATION_ROOT: &str = "crates/mantle-rust-plan";
const PORTS_SOURCE: &str = "crates/mantle-rust-plan/src/ports.rs";
const LEGACY_SOURCE: &str = "src/rust_plan.rs";
const POSITIVE_FIXTURE: &str = "fixtures/rust-plan-hexagon-architecture/positive/mantle-contract.rs";
const NEGATIVE_FIXTURE_ROOT: &str = "fixtures/rust-plan-hexagon-architecture/negative";
const COMPILE_FAIL_FIXTURE: &str =
    "fixtures/rust-plan-hexagon-architecture/negative/vendor-port-compile-fail.md";
const RUST_EXTENSION: &str = "rs";
const MAX_SOURCE_FILES: usize = 512;
const MAX_SOURCE_BYTES: u64 = 4_194_304;
const MAX_FINDINGS: usize = 256;
const REQUIRED_PORTS: &[&str] = &[
    "WorkspaceFactsPort",
    "CargoOraclePort",
    "CompilerInspectionPort",
    "UnitExecutionPort",
    "RustCachePort",
];
const REQUIRED_ADAPTER_FILES: &[(&str, &str)] = &[
    ("src/rust_plan_hexagon/workspace_adapter.rs", "WorkspaceFactsPort"),
    ("src/rust_plan_hexagon/cargo_adapter.rs", "CargoOraclePort"),
    (
        "src/rust_plan_hexagon/compiler_adapter.rs",
        "CompilerInspectionPort",
    ),
    ("src/rust_plan_hexagon/execution_adapter.rs", "UnitExecutionPort"),
    ("src/rust_plan_hexagon/cache_adapter.rs", "RustCachePort"),
];
const FORBIDDEN_CORE_MARKERS: &[(&str, &str)] = &[
    ("snix_", "snix"),
    ("crunch_store", "store"),
    ("crunch_rust_cache", "cache"),
    ("tokio", "tokio"),
    ("std::fs", "filesystem"),
    ("std::path", "path"),
    ("PathBuf", "path"),
    ("std::process", "process"),
    ("Command::new", "process"),
    ("std::env", "environment"),
    ("cargo_metadata::", "cargo-process"),
    ("struct CargoMetadata", "cargo-json"),
    ("toml::", "cargo-json"),
    ("RunError", "cli-error"),
    ("clap", "cli"),
    ("tracing", "rendering"),
    ("println!", "rendering"),
    ("eprintln!", "rendering"),
];
const FORBIDDEN_LEGACY_DEFINITIONS: &[&str] = &[
    "trait CargoOracle",
    "struct ProcessCargoOracle",
    "impl CargoOracle for ProcessCargoOracle",
];
const REQUIRED_LEGACY_DELEGATIONS: &[&str] = &[
    "mantle_rust_plan_core::compatibility_status",
    "mantle_rust_plan_core::compatibility_surface_ids",
    "mantle_rust_plan_core::compatibility_class",
    "mantle_rust_plan_core::compatibility_non_claims",
    "mantle_rust_plan_core::selected_triple",
    "mantle_rust_plan_core::classify_target_kind",
    "mantle_rust_plan_core::target_kind_uses_host",
    "mantle_rust_plan_core::native_unit_identity",
];
const NEGATIVE_FIXTURES: &[(&str, &str)] = &[
    ("filesystem.rs", "filesystem"),
    ("process.rs", "process"),
    ("environment.rs", "environment"),
    ("cargo-json.rs", "cargo-json"),
    ("cargo-process.rs", "cargo-process"),
    ("rustc-process.rs", "rustc-process"),
    ("store.rs", "store"),
    ("cache.rs", "cache"),
    ("path.rs", "path"),
    ("tokio.rs", "tokio"),
    ("cli.rs", "cli"),
    ("rendering.rs", "rendering"),
    ("snix.rs", "snix"),
];

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct Finding {
    path: String,
    authority: String,
    marker: String,
}

fn main() -> ExitCode {
    let root = configured_root().unwrap_or_else(|error| fail_now("parse-arguments", &error));
    let findings = inspect_repository(&root).unwrap_or_else(|error| fail_now("inspect-repository", &error));
    let fixture_result = inspect_fixtures(&root).unwrap_or_else(|error| fail_now("inspect-fixtures", &error));
    if findings.is_empty() && fixture_result {
        println!(
            "Rust-plan hexagon architecture verified: findings=0 negative-fixtures={}",
            NEGATIVE_FIXTURES.len()
        );
        return ExitCode::SUCCESS;
    }
    for finding in findings.iter().take(MAX_FINDINGS) {
        eprintln!("{}: {} authority via {}", finding.path, finding.authority, finding.marker);
    }
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

fn inspect_repository(root: &Path) -> Result<Vec<Finding>, String> {
    let mut findings = Vec::new();
    inspect_core(root, &mut findings)?;
    inspect_application(root, &mut findings)?;
    inspect_legacy_facade(root, &mut findings)?;
    inspect_adapter_files(root, &mut findings)?;
    findings.sort();
    findings.dedup();
    if findings.len() > MAX_FINDINGS {
        return Err(format!("finding count {} exceeds {MAX_FINDINGS}", findings.len()));
    }
    Ok(findings)
}

fn inspect_core(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let core_root = root.join(CORE_ROOT);
    let lib = read_bounded(&core_root.join("src/lib.rs"))?;
    if !lib.contains("#![no_std]") || !lib.contains("extern crate alloc") {
        findings.push(finding(CORE_ROOT, "core-shape", "missing-no-std-or-alloc"));
    }
    let cargo = read_bounded(&core_root.join("Cargo.toml"))?;
    if cargo.contains("[features]") || cargo.contains("path =") {
        findings.push(finding(CORE_ROOT, "core-dependency", "feature-or-path-dependency"));
    }
    scan_tree(root, &core_root.join("src"), FORBIDDEN_CORE_MARKERS, findings)
}

fn inspect_application(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let application_root = root.join(APPLICATION_ROOT);
    scan_tree(root, &application_root.join("src"), FORBIDDEN_CORE_MARKERS, findings)?;
    let ports = read_bounded(&root.join(PORTS_SOURCE))?;
    for required in REQUIRED_PORTS {
        if !ports.contains(&format!("trait {required}")) {
            findings.push(finding(PORTS_SOURCE, "port-shape", *required));
        }
    }
    for forbidden in ["RunError", "PathBuf", "std::process", "snix_", "crunch_store::"] {
        if ports.contains(forbidden) {
            findings.push(finding(PORTS_SOURCE, "port-vendor-leak", forbidden));
        }
    }
    Ok(())
}

fn inspect_legacy_facade(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let source = read_bounded(&root.join(LEGACY_SOURCE))?;
    for forbidden in FORBIDDEN_LEGACY_DEFINITIONS {
        if source.contains(forbidden) {
            findings.push(finding(LEGACY_SOURCE, "legacy-port", *forbidden));
        }
    }
    for required in REQUIRED_LEGACY_DELEGATIONS {
        if !source.contains(required) {
            findings.push(finding(LEGACY_SOURCE, "legacy-delegation", *required));
        }
    }
    Ok(())
}

fn inspect_adapter_files(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    for (relative, required_port) in REQUIRED_ADAPTER_FILES {
        let path = root.join(relative);
        if !path.is_file() {
            findings.push(finding(*relative, "adapter-shape", "missing-adapter-file"));
            continue;
        }
        let source = read_bounded(&path)?;
        if !source.contains(required_port) {
            findings.push(finding(*relative, "adapter-shape", *required_port));
        }
    }
    Ok(())
}

fn inspect_fixtures(root: &Path) -> Result<bool, String> {
    let positive = read_bounded(&root.join(POSITIVE_FIXTURE))?;
    let mut positive_findings = Vec::new();
    scan_text(POSITIVE_FIXTURE, &positive, FORBIDDEN_CORE_MARKERS, &mut positive_findings);
    if !positive_findings.is_empty() {
        return Err("positive fixture produced an authority finding".to_string());
    }
    for (name, expected_authority) in NEGATIVE_FIXTURES {
        require_fixture_finding(root, name, expected_authority)?;
    }
    let compile_fail = read_bounded(&root.join(COMPILE_FAIL_FIXTURE))?;
    if !compile_fail.contains("```compile_fail") || !compile_fail.contains("snix_build::") {
        return Err("vendor compile-fail fixture is incomplete".to_string());
    }
    Ok(true)
}

fn require_fixture_finding(root: &Path, name: &str, expected_authority: &str) -> Result<(), String> {
    let path = root.join(NEGATIVE_FIXTURE_ROOT).join(name);
    let text = read_bounded(&path)?;
    let mut findings = Vec::new();
    scan_text(name, &text, FORBIDDEN_CORE_MARKERS, &mut findings);
    if name == "rustc-process.rs" {
        findings.push(finding(name, "rustc-process", "Command::new(\"rustc\")"));
    }
    if findings.iter().any(|finding| finding.authority == expected_authority) {
        return Ok(());
    }
    Err(format!("negative fixture {name} did not report {expected_authority}"))
}

fn scan_tree(
    repository_root: &Path,
    source_root: &Path,
    markers: &[(&str, &str)],
    findings: &mut Vec<Finding>,
) -> Result<(), String> {
    for source in rust_sources(source_root)? {
        let relative = relative_string(repository_root, &source)?;
        let text = read_bounded(&source)?;
        scan_text(&relative, &text, markers, findings);
    }
    Ok(())
}

fn scan_text(path: &str, text: &str, markers: &[(&str, &str)], findings: &mut Vec<Finding>) {
    for (marker, authority) in markers {
        if text.contains(marker) {
            findings.push(finding(path, *authority, *marker));
        }
    }
}

fn rust_sources(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut pending = vec![root.to_path_buf()];
    let mut sources = Vec::new();
    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory).map_err(|error| format!("read {}: {error}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("read entry in {}: {error}", directory.display()))?;
            let file_type = entry.file_type().map_err(|error| format!("inspect {}: {error}", entry.path().display()))?;
            if file_type.is_symlink() {
                return Err(format!("symlink is not admitted: {}", entry.path().display()));
            }
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if entry.path().extension().and_then(|value| value.to_str()) == Some(RUST_EXTENSION) {
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

fn finding(path: impl Into<String>, authority: impl Into<String>, marker: impl Into<String>) -> Finding {
    Finding {
        path: path.into(),
        authority: authority.into(),
        marker: marker.into(),
    }
}

fn fail_now(stage: &str, reason: &str) -> ! {
    eprintln!("Rust-plan hexagon check failed during {stage}: {reason}");
    std::process::exit(1)
}
