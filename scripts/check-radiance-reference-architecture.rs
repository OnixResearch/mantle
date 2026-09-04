#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"
---

//! Enforce the optional Radiance reference functional-core and shell boundary.
// r[verify mantle.bootstrap.radiance_reference.source_cohort]
// r[verify mantle.bootstrap.radiance_reference.lineage]
// r[verify mantle.bootstrap.radiance_reference.claim_boundary]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const CORE_ROOT: &str = "crates/crunch-radiance-reference-core";
const SHELL_ROOT: &str = "src/radiance";
const SHELL_MODULE: &str = "src/radiance/mod.rs";
const POSITIVE_FIXTURE: &str = "fixtures/radiance-reference-architecture/positive/pure-core.rs";
const NEGATIVE_FIXTURE_ROOT: &str = "fixtures/radiance-reference-architecture/negative";
const RUST_EXTENSION: &str = "rs";
const SOURCE_FILE_COUNT_MAX: usize = 64;
const SOURCE_BYTES_MAX: u64 = 1_048_576;
const FINDING_COUNT_MAX: usize = 128;
const REQUIRED_CORE_MARKERS: &[&str] = &[
    "pub fn admit_source_cohort",
    "pub fn plan_radiance_reference",
    "pub fn classify_route_convergence",
    "pub fn classify_cross_route",
    "pub fn seal_radiance_reference_receipt",
    "pub fn validate_radiance_reference_receipt",
    "pub source_projection: alloc::string::String",
];
const REQUIRED_SHELL_MARKERS: &[&str] = &[
    "plan_vcs_checkout_source_bundle",
    "import_source_bundle",
    "materialize_source_record_payload",
    "ProtectedExecPolicy::from_action_plan",
    "install_exec_supervisor",
    "install_pre_exec",
    "publish_immutable_source_bytes",
];
const FORBIDDEN_CORE_MARKERS: &[(&str, &str)] = &[
    ("std::fs", "filesystem"),
    ("std::path", "path"),
    ("PathBuf", "path"),
    ("std::process", "process"),
    ("Command::new", "process"),
    ("std::env", "environment"),
    ("std::time", "clock"),
    ("std::net", "network"),
    ("reqwest", "network"),
    ("gix", "provider"),
    ("snix_", "provider"),
    ("crunch_store", "store"),
    ("tokio", "async-runtime"),
    ("rand::", "random"),
    ("println!", "rendering"),
    ("eprintln!", "rendering"),
];
const NEGATIVE_FIXTURES: &[(&str, &str)] = &[
    ("filesystem.rs", "filesystem"),
    ("path.rs", "path"),
    ("process.rs", "process"),
    ("environment.rs", "environment"),
    ("network.rs", "network"),
    ("provider.rs", "provider"),
    ("store.rs", "store"),
    ("async-runtime.rs", "async-runtime"),
    ("random.rs", "random"),
    ("rendering.rs", "rendering"),
];

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct Finding {
    path: String,
    authority: String,
    marker: String,
}

fn main() -> ExitCode {
    let root = configured_root().unwrap_or_else(|error| fail_now("arguments", &error));
    let findings = inspect_repository(&root).unwrap_or_else(|error| fail_now("repository", &error));
    inspect_fixtures(&root).unwrap_or_else(|error| fail_now("fixtures", &error));
    if findings.is_empty() {
        println!(
            "Radiance reference architecture verified: findings=0 negative-fixtures={}",
            NEGATIVE_FIXTURES.len()
        );
        return ExitCode::SUCCESS;
    }
    for finding in findings.iter().take(FINDING_COUNT_MAX) {
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
    let core_root = root.join(CORE_ROOT);
    let library = read_bounded(&core_root.join("src/lib.rs"))?;
    let manifest = read_bounded(&core_root.join("Cargo.toml"))?;
    let combined_core = combined_rust_sources(&core_root.join("src"))?;
    let combined_shell = combined_shell_sources(root)?;
    let mut findings = Vec::new();
    if !library.contains("#![no_std]") || !library.contains("extern crate alloc") {
        findings.push(finding(CORE_ROOT, "core-shape", "missing-no-std-or-alloc"));
    }
    if manifest.contains("[features]") || manifest.contains("std =") {
        findings.push(finding(CORE_ROOT, "core-dependency", "std-feature"));
    }
    for marker in REQUIRED_CORE_MARKERS {
        if !combined_core.contains(marker) {
            findings.push(finding(CORE_ROOT, "core-contract", marker));
        }
    }
    scan_text(CORE_ROOT, &combined_core, FORBIDDEN_CORE_MARKERS, &mut findings);
    for marker in REQUIRED_SHELL_MARKERS {
        if !combined_shell.contains(marker) {
            findings.push(finding(SHELL_ROOT, "shell-adapter", marker));
        }
    }
    if combined_shell.contains("Command::new(\"") {
        findings.push(finding(SHELL_ROOT, "ambient-process", "literal-command-name"));
    }
    findings.sort();
    findings.dedup();
    if findings.len() > FINDING_COUNT_MAX {
        return Err(format!("finding count {} exceeds {FINDING_COUNT_MAX}", findings.len()));
    }
    Ok(findings)
}

fn combined_shell_sources(root: &Path) -> Result<String, String> {
    let mut combined = read_bounded(&root.join(SHELL_MODULE))?;
    combined.push_str(&combined_rust_sources(&root.join(SHELL_ROOT))?);
    if combined.len() as u64 > SOURCE_BYTES_MAX {
        return Err(format!("combined Radiance shell exceeds {SOURCE_BYTES_MAX} bytes"));
    }
    Ok(combined)
}

fn inspect_fixtures(root: &Path) -> Result<(), String> {
    let positive = read_bounded(&root.join(POSITIVE_FIXTURE))?;
    let mut positive_findings = Vec::new();
    scan_text(POSITIVE_FIXTURE, &positive, FORBIDDEN_CORE_MARKERS, &mut positive_findings);
    if !positive_findings.is_empty() {
        return Err("positive fixture produced an authority finding".to_string());
    }
    for (name, authority) in NEGATIVE_FIXTURES {
        let relative = format!("{NEGATIVE_FIXTURE_ROOT}/{name}");
        let text = read_bounded(&root.join(&relative))?;
        let mut findings = Vec::new();
        scan_text(&relative, &text, FORBIDDEN_CORE_MARKERS, &mut findings);
        if !findings.iter().any(|finding| finding.authority == *authority) {
            return Err(format!("negative fixture {name} did not produce {authority}"));
        }
    }
    Ok(())
}

fn combined_rust_sources(root: &Path) -> Result<String, String> {
    let mut paths = Vec::new();
    collect_rust_sources(root, &mut paths)?;
    paths.sort();
    if paths.len() > SOURCE_FILE_COUNT_MAX {
        return Err(format!("source file count {} exceeds {SOURCE_FILE_COUNT_MAX}", paths.len()));
    }
    let mut combined = String::new();
    for path in paths {
        combined.push_str(&read_bounded(&path)?);
        combined.push('\n');
    }
    Ok(combined)
}

fn collect_rust_sources(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|error| format!("read {}: {error}", root.display()))? {
        let path = entry.map_err(|error| format!("read directory entry: {error}"))?.path();
        if path.is_dir() {
            collect_rust_sources(&path, output)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some(RUST_EXTENSION) {
            output.push(path);
        }
    }
    Ok(())
}

fn read_bounded(path: &Path) -> Result<String, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("metadata {}: {error}", path.display()))?;
    if metadata.len() > SOURCE_BYTES_MAX {
        return Err(format!("{} exceeds {SOURCE_BYTES_MAX} bytes", path.display()));
    }
    fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn scan_text(path: &str, text: &str, markers: &[(&str, &str)], findings: &mut Vec<Finding>) {
    for (marker, authority) in markers {
        if text.contains(marker) {
            findings.push(finding(path, authority, marker));
        }
    }
}

fn finding(path: &str, authority: &str, marker: &str) -> Finding {
    Finding {
        path: path.to_string(),
        authority: authority.to_string(),
        marker: marker.to_string(),
    }
}

fn fail_now(stage: &str, message: &str) -> ! {
    eprintln!("Radiance reference architecture check failed at {stage}: {message}");
    std::process::exit(1)
}
