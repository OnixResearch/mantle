#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"
---

//! Enforce resource-policy functional-core and adapter authority boundaries.
// r[verify remote_builds.replayable_resource_selection]
// r[verify remote_builds.positive_oom_retry]
// r[verify remote_builds.usage_reservation_and_reconciliation]
// r[verify remote_builds.authorized_result_sharing]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const CORE_ROOT: &str = "crates/crunch-resource-policy-core";
const APPLICATION_ROOT: &str = "crates/crunch-resource-policy";
const ROOT_ADAPTER: &str = "src/resource_policy.rs";
const COORDINATOR_ADAPTER: &str = "src/remote_build.rs";
const POSITIVE_FIXTURE: &str = "fixtures/resource-policy-architecture/positive/pure-core.rs";
const NEGATIVE_FIXTURE_ROOT: &str = "fixtures/resource-policy-architecture/negative";
const RUST_EXTENSION: &str = "rs";
const MAX_SOURCE_FILES: usize = 256;
const MAX_SOURCE_BYTES: u64 = 4_194_304;
const MAX_FINDINGS: usize = 256;
const REQUIRED_APPLICATION_PORTS: &[&str] = &["UsageLedgerPort", "ResourcePolicyEvidencePort"];
const REQUIRED_ROOT_MARKERS: &[&str] = &[
    "plan_remote_attempt_assignment",
    "StrongReusePlan",
    "statically_eligible_endpoint_ids",
    "plan_coordinator_resource_policy",
];
const REQUIRED_COORDINATOR_MARKERS: &[&str] = &[
    "resource_policy_selection",
    "effective_coordinator_resource_requirements",
    "plan_coordinator_resource_policy",
];
const FORBIDDEN_CORE_MARKERS: &[(&str, &str)] = &[
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
    ("crunch_store", "store"),
    ("snix_", "store"),
    ("crunch_build", "executor"),
    ("crunch_action_result", "result-authority"),
    ("valence_core", "evidence-authority"),
    ("clap", "cli"),
    ("println!", "rendering"),
    ("eprintln!", "rendering"),
];
const FORBIDDEN_APPLICATION_MARKERS: &[(&str, &str)] = &[
    ("std::fs", "filesystem"),
    ("std::process", "process"),
    ("std::env", "environment"),
    ("SystemTime", "clock"),
    ("rand::", "random"),
    ("reqwest", "network"),
    ("ureq", "network"),
    ("crunch_store", "store"),
    ("snix_", "store"),
    ("clap", "cli"),
    ("println!", "rendering"),
    ("eprintln!", "rendering"),
];
const NEGATIVE_FIXTURES: &[(&str, &str)] = &[
    ("filesystem.rs", "filesystem"),
    ("path.rs", "path"),
    ("process.rs", "process"),
    ("environment.rs", "environment"),
    ("clock.rs", "clock"),
    ("random.rs", "random"),
    ("network.rs", "network"),
    ("store.rs", "store"),
    ("executor.rs", "executor"),
    ("authority-result.rs", "result-authority"),
    ("evidence-authority.rs", "evidence-authority"),
    ("rendering.rs", "rendering"),
];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Finding {
    path: String,
    authority: String,
    marker: String,
}

fn main() -> ExitCode {
    let root = configured_root().unwrap_or_else(|error| fail_now("arguments", &error));
    let findings = inspect_repository(&root).unwrap_or_else(|error| fail_now("repository", &error));
    let fixtures = inspect_fixtures(&root).unwrap_or_else(|error| fail_now("fixtures", &error));
    if findings.is_empty() && fixtures {
        println!(
            "resource-policy architecture verified: findings=0 negative-fixtures={}",
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
    require_markers(root, ROOT_ADAPTER, REQUIRED_ROOT_MARKERS, &mut findings)?;
    require_markers(root, COORDINATOR_ADAPTER, REQUIRED_COORDINATOR_MARKERS, &mut findings)?;
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
        scan_text(&relative, &text, FORBIDDEN_APPLICATION_MARKERS, findings);
    }
    let ports = read_bounded(&application_root.join("src/ports.rs"))?;
    for port in REQUIRED_APPLICATION_PORTS {
        if !ports.contains(port) {
            findings.push(finding("crates/crunch-resource-policy/src/ports.rs", "port-shape", *port));
        }
    }
    Ok(())
}

fn require_markers(
    root: &Path,
    relative: &str,
    markers: &[&str],
    findings: &mut Vec<Finding>,
) -> Result<(), String> {
    let source = read_bounded(&root.join(relative))?;
    for marker in markers {
        if !source.contains(marker) {
            findings.push(finding(relative, "adapter-shape", *marker));
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
    eprintln!("resource-policy architecture check failed during {stage}: {reason}");
    std::process::exit(1)
}
