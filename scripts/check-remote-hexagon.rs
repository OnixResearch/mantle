#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"
---

//! Enforce the remote-build functional-core and adapter authority boundary.
// r[verify remote_builds.hexagonal_core]
// r[verify remote_builds.application_owned_ports]
// r[verify remote_builds.remote_effect_plans]
// r[verify remote_builds.hexagonal_compatibility]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const CORE_ROOT: &str = "crates/crunch-remote-core";
const APPLICATION_ROOT: &str = "crates/crunch-remote";
const DISTRIBUTED_SOURCE: &str = "crates/crunch-build/src/distributed.rs";
const SNIX_ADAPTER_SOURCE: &str = "crates/crunch-build/src/distributed/snix_adapter.rs";
const POSITIVE_FIXTURE: &str = "fixtures/remote-hexagon-architecture/positive/mantle-contract.rs";
const NEGATIVE_FIXTURE_ROOT: &str = "fixtures/remote-hexagon-architecture/negative";
const RUST_EXTENSION: &str = "rs";
const TEST_MODULE_MARKER: &str = "#[cfg(test)]";
const MAX_SOURCE_FILES: usize = 512;
const MAX_SOURCE_BYTES: u64 = 4_194_304;
const MAX_FINDINGS: usize = 256;
const REQUIRED_ADAPTER_FILES: &[(&str, &str)] = &[
    ("src/remote_hexagon/attempt_adapter.rs", "AttemptPersistencePort"),
    ("src/remote_hexagon/authority_adapter.rs", "CredentialVerificationPort"),
    ("src/remote_hexagon/executor_adapter.rs", "ExecutorPort"),
    ("src/remote_hexagon/store_adapter.rs", "StoreAdmissionPort"),
    ("src/remote_hexagon/telemetry_adapter.rs", "TelemetryPublicationPort"),
    ("src/remote_hexagon/transport_adapter.rs", "TransportPort"),
];
const FORBIDDEN_CORE_MARKERS: &[(&str, &str)] = &[
    ("snix_", "snix"),
    ("crunch_store", "store"),
    ("tokio", "tokio"),
    ("std::fs", "filesystem"),
    ("std::path", "path"),
    ("PathBuf", "path"),
    ("std::process", "process"),
    ("std::env", "environment"),
    ("std::time", "clock"),
    ("SystemTime", "clock"),
    ("rand::", "random"),
    ("reqwest", "network"),
    ("ureq", "network"),
    ("remote_credentials", "credential"),
    ("clap", "cli"),
    ("tracing", "rendering"),
    ("println!", "rendering"),
    ("eprintln!", "rendering"),
];
const FORBIDDEN_DISTRIBUTED_MARKERS: &[&str] = &[
    "snix_build::",
    "snix_store::",
    "snix_castore::",
    "crunch_store::",
];
const NEGATIVE_FIXTURES: &[(&str, &str)] = &[
    ("snix.rs", "snix"),
    ("store.rs", "store"),
    ("tokio.rs", "tokio"),
    ("filesystem.rs", "filesystem"),
    ("path.rs", "path"),
    ("process.rs", "process"),
    ("environment.rs", "environment"),
    ("clock.rs", "clock"),
    ("random.rs", "random"),
    ("network.rs", "network"),
    ("credential.rs", "credential"),
    ("cli.rs", "cli"),
    ("rendering.rs", "rendering"),
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
        println!("remote hexagon architecture verified: findings=0 negative-fixtures={}", NEGATIVE_FIXTURES.len());
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
    inspect_distributed_adapter_boundary(root, &mut findings)?;
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
    scan_text("crates/crunch-remote-core/Cargo.toml", &cargo, FORBIDDEN_CORE_MARKERS, findings);
    for source in rust_sources(&core_root.join("src"))? {
        let relative = relative_string(root, &source)?;
        let text = read_bounded(&source)?;
        scan_text(&relative, &text, FORBIDDEN_CORE_MARKERS, findings);
    }
    Ok(())
}

fn inspect_application(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let application_root = root.join(APPLICATION_ROOT);
    for source in rust_sources(&application_root.join("src"))? {
        let relative = relative_string(root, &source)?;
        let text = read_bounded(&source)?;
        scan_text(&relative, &text, FORBIDDEN_CORE_MARKERS, findings);
    }
    Ok(())
}

fn inspect_distributed_adapter_boundary(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let source = read_bounded(&root.join(DISTRIBUTED_SOURCE))?;
    let production = source.split(TEST_MODULE_MARKER).next().unwrap_or(&source);
    for marker in FORBIDDEN_DISTRIBUTED_MARKERS {
        if production.contains(marker) {
            findings.push(finding(DISTRIBUTED_SOURCE, "vendor-port", *marker));
        }
    }
    let adapter = read_bounded(&root.join(SNIX_ADAPTER_SOURCE))?;
    if !adapter.contains("snix_build::buildservice::BuildRequest") {
        findings.push(finding(SNIX_ADAPTER_SOURCE, "adapter-shape", "missing-snix-projection"));
    }
    if !source.contains("pub mod snix_adapter") {
        findings.push(finding(DISTRIBUTED_SOURCE, "adapter-shape", "missing-snix-adapter-module"));
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
        let path = root.join(NEGATIVE_FIXTURE_ROOT).join(name);
        let text = read_bounded(&path)?;
        let mut findings = Vec::new();
        scan_text(name, &text, FORBIDDEN_CORE_MARKERS, &mut findings);
        if !findings.iter().any(|finding| finding.authority == *expected_authority) {
            return Err(format!("negative fixture {name} did not report {expected_authority}"));
        }
    }
    Ok(true)
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
    eprintln!("remote hexagon architecture check failed during {stage}: {reason}");
    std::process::exit(1)
}
