#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"
---

//! Enforce explicit build-planning observations and effect separation.
// r[verify realization_routing.explicit_observation_boundary]
// r[verify realization_routing.plan_execution_separation]
// r[verify realization_routing.typed_planning_blockers]
// r[verify build_scheduling.explicit_parallelism_facts]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const CORE_ROOT: &str = "crates/crunch-build-planning-core";
const ROUTING_FACADE: &str = "src/realization_routing.rs";
const BUILD_PLAN_SOURCE: &str = "src/build_plan.rs";
const PIPELINE_SOURCE: &str = "crates/crunch-pipeline/src/lib.rs";
const POSITIVE_FIXTURE: &str = "fixtures/build-planning-architecture/positive/explicit-facts.rs";
const NEGATIVE_FIXTURE_ROOT: &str = "fixtures/build-planning-architecture/negative";
const TEST_MODULE_MARKER: &str = "#[cfg(test)]";
const RUST_EXTENSION: &str = "rs";
const MAX_SOURCE_FILES: usize = 512;
const MAX_SOURCE_BYTES: u64 = 4_194_304;
const MAX_FINDINGS: usize = 256;
const REQUIRED_ADAPTER_FILES: &[(&str, &str)] = &[
    (
        "src/build_planning_hexagon/observation_adapter.rs",
        "SuppliedBuildPlanningObservations",
    ),
    (
        "src/build_planning_hexagon/remote_adapter.rs",
        "RemoteCandidateProjection",
    ),
    (
        "src/build_planning_hexagon/execution_adapter.rs",
        "execute_planned_effect",
    ),
];
const FORBIDDEN_CORE_MARKERS: &[(&str, &str)] = &[
    ("std::fs", "filesystem"),
    ("std::path", "path"),
    ("PathBuf", "path"),
    ("std::process", "process"),
    ("Command::new", "process"),
    ("std::env", "environment"),
    ("std::thread", "host-parallelism"),
    ("available_parallelism", "host-parallelism"),
    ("crunch_store", "store"),
    ("snix_", "provider"),
    ("VerifyingKey", "key"),
    ("collect_doctor_report", "doctor"),
    ("start_remote_session", "remote-session"),
    ("redeem", "credential"),
    ("persist_and_export", "mutation"),
    ("execute_build", "execution"),
    ("RunError", "shell-error"),
    ("clap", "cli"),
    ("tokio", "async-runtime"),
    ("println!", "rendering"),
    ("eprintln!", "rendering"),
];
const NEGATIVE_FIXTURES: &[(&str, &str)] = &[
    ("filesystem.rs", "filesystem"),
    ("store.rs", "store"),
    ("environment.rs", "environment"),
    ("parallelism.rs", "host-parallelism"),
    ("process.rs", "process"),
    ("doctor.rs", "doctor"),
    ("key.rs", "key"),
    ("remote-session.rs", "remote-session"),
    ("credential.rs", "credential"),
    ("mutation.rs", "mutation"),
    ("execution.rs", "execution"),
    ("rendering.rs", "rendering"),
    ("shell-error.rs", "shell-error"),
    ("provider.rs", "provider"),
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
    let fixtures_ok = inspect_fixtures(&root).unwrap_or_else(|error| fail_now("inspect-fixtures", &error));
    if findings.is_empty() && fixtures_ok {
        println!(
            "build-planning architecture verified: findings=0 negative-fixtures={}",
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
    inspect_routing_facade(root, &mut findings)?;
    inspect_delegations(root, &mut findings)?;
    inspect_adapters(root, &mut findings)?;
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

fn inspect_routing_facade(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let source = read_bounded(&root.join(ROUTING_FACADE))?;
    let production = source.split(TEST_MODULE_MARKER).next().unwrap_or(&source);
    if !production.contains("pub use crunch_build_planning_core::*") {
        findings.push(finding(ROUTING_FACADE, "facade-shape", "missing-core-reexport"));
    }
    for forbidden in ["struct RouteCandidateFacts", "fn plan_realization_route", "fn classify_candidate"] {
        if production.contains(forbidden) {
            findings.push(finding(ROUTING_FACADE, "duplicate-policy", forbidden));
        }
    }
    Ok(())
}

fn inspect_delegations(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let build_plan = read_bounded(&root.join(BUILD_PLAN_SOURCE))?;
    if !build_plan.contains("crunch_build_planning_core::select_build_action") {
        findings.push(finding(BUILD_PLAN_SOURCE, "route-delegation", "select_build_action"));
    }
    let pipeline = read_bounded(&root.join(PIPELINE_SOURCE))?;
    if !pipeline.contains("crunch_build_planning_core::plan_parallelism") {
        findings.push(finding(PIPELINE_SOURCE, "parallelism-delegation", "plan_parallelism"));
    }
    if !pipeline.contains("fn observe_available_parallelism()") {
        findings.push(finding(PIPELINE_SOURCE, "shell-observation", "observe_available_parallelism"));
    }
    Ok(())
}

fn inspect_adapters(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    for (relative, required) in REQUIRED_ADAPTER_FILES {
        let path = root.join(relative);
        if !path.is_file() {
            findings.push(finding(*relative, "adapter-shape", "missing-adapter-file"));
            continue;
        }
        let source = read_bounded(&path)?;
        if !source.contains(required) {
            findings.push(finding(*relative, "adapter-shape", *required));
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
    for (name, expected) in NEGATIVE_FIXTURES {
        let path = root.join(NEGATIVE_FIXTURE_ROOT).join(name);
        let text = read_bounded(&path)?;
        let mut findings = Vec::new();
        scan_text(name, &text, FORBIDDEN_CORE_MARKERS, &mut findings);
        if !findings.iter().any(|finding| finding.authority == *expected) {
            return Err(format!("negative fixture {name} did not report {expected}"));
        }
    }
    Ok(true)
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
    eprintln!("build-planning architecture check failed during {stage}: {reason}");
    std::process::exit(1)
}
