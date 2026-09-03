#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"
---

//! Enforce the source-observation functional core and add-only ingest shell.
// r[verify source_transports.source_observations.contract]
// r[verify source_transports.source_observations.locator_boundary]
// r[verify source_transports.monotonic_ingest]
// r[verify mantle.release_provenance.source_observation_signature_boundary]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const CORE_ROOT: &str = "crates/crunch-source-core";
const OBSERVATION_ADAPTER: &str = "src/source_bundle/source_observation_adapter.rs";
const INGEST_ADAPTER: &str = "src/source_bundle/monotonic_ingest.rs";
const RELEASE_CORE: &str = "crates/crunch-release-core/src/manifest.rs";
const POSITIVE_FIXTURE: &str = "fixtures/source-observation-architecture/positive/pure-facts.rs";
const NEGATIVE_FIXTURE_ROOT: &str = "fixtures/source-observation-architecture/negative";
const TEST_MODULE_MARKER: &str = "#[cfg(test)]\nmod tests";
const RUST_EXTENSION: &str = "rs";
const MAX_SOURCE_FILES: usize = 64;
const MAX_SOURCE_BYTES: u64 = 1_048_576;
const MAX_FINDINGS: usize = 128;
const REQUIRED_CORE_MARKERS: &[&str] = &[
    "pub struct SourceObservationWire",
    "pub struct SourceObservationSubject",
    "pub fn build_source_observation",
    "pub fn project_legacy_source_observation",
    "pub fn plan_source_ingest",
    "pub fn validate_source_ingest_plan",
];
const FORBIDDEN_CORE_MARKERS: &[(&str, &str)] = &[
    ("std::fs", "filesystem"),
    ("std::path", "path"),
    ("PathBuf", "path"),
    ("std::process", "process"),
    ("Command::new", "process"),
    ("std::env", "environment"),
    ("std::time", "clock"),
    ("SystemTime", "clock"),
    ("std::net", "network"),
    ("reqwest", "network"),
    ("gix", "provider"),
    ("snix_", "provider"),
    ("crunch_store", "store"),
    ("tokio", "async-runtime"),
    ("rand::", "random"),
    ("ed25519", "signature-authority"),
    ("SourceReviewAttachment", "review-authority"),
    ("ReleaseAttestation", "release-authority"),
    ("WitnessAttestation", "witness-authority"),
    ("fs::rename", "mutation"),
    ("fs::write", "mutation"),
    ("create_dir", "mutation"),
    ("remove_file", "mutation"),
    ("println!", "rendering"),
    ("eprintln!", "rendering"),
];
const NEGATIVE_FIXTURES: &[(&str, &str)] = &[
    ("filesystem.rs", "filesystem"),
    ("path.rs", "path"),
    ("process.rs", "process"),
    ("environment.rs", "environment"),
    ("clock.rs", "clock"),
    ("network.rs", "network"),
    ("provider.rs", "provider"),
    ("store.rs", "store"),
    ("async-runtime.rs", "async-runtime"),
    ("random.rs", "random"),
    ("signature.rs", "signature-authority"),
    ("review.rs", "review-authority"),
    ("release.rs", "release-authority"),
    ("witness.rs", "witness-authority"),
    ("mutation.rs", "mutation"),
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
    inspect_fixtures(&root).unwrap_or_else(|error| fail_now("inspect-fixtures", &error));
    if findings.is_empty() {
        println!(
            "source-observation architecture verified: findings=0 negative-fixtures={}",
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
    inspect_observation_adapter(root, &mut findings)?;
    inspect_ingest_adapter(root, &mut findings)?;
    inspect_release_binding(root, &mut findings)?;
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
    let combined = combined_rust_sources(&core_root.join("src"))?;
    for required in REQUIRED_CORE_MARKERS {
        if !combined.contains(required) {
            findings.push(finding(CORE_ROOT, "core-contract", *required));
        }
    }
    scan_tree(root, &core_root.join("src"), FORBIDDEN_CORE_MARKERS, findings)
}

fn inspect_observation_adapter(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let source = read_bounded(&root.join(OBSERVATION_ADAPTER))?;
    for required in [
        "crunch_build::FetchUrl::new",
        "crunch_build::GitRevision::new",
        "project_legacy_source_observation",
        "SourceRecordProvenance::LegacyV1",
    ] {
        if !source.contains(required) {
            findings.push(finding(OBSERVATION_ADAPTER, "observation-adapter", required));
        }
    }
    Ok(())
}

fn inspect_ingest_adapter(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let source = read_bounded(&root.join(INGEST_ADAPTER))?;
    for required in [
        "plan_source_ingest",
        "validate_source_ingest_plan",
        "SourceIngestDisposition::Add",
        "publish_one_file",
        "ReplacementMode::NoReplace",
        "DurabilityMode::DurabilityRequired",
        "rollback_created_state",
    ] {
        if !source.contains(required) {
            findings.push(finding(INGEST_ADAPTER, "ingest-adapter", required));
        }
    }
    if source.contains("write_json_atomically") || source.contains("fs::rename(") {
        findings.push(finding(INGEST_ADAPTER, "replacement-authority", "replace-capable-publication"));
    }
    Ok(())
}

fn inspect_release_binding(root: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    let source = read_bounded(&root.join(RELEASE_CORE))?;
    let production = source.split(TEST_MODULE_MARKER).next().unwrap_or(&source);
    for required in [
        "pub source_observation: Option<SourceObservationSubject>",
        "with_derived_source_observation",
        "validate_source_observation_binding",
    ] {
        if !production.contains(required) {
            findings.push(finding(RELEASE_CORE, "release-binding", required));
        }
    }
    for forbidden in ["source_signature", "SourceObservationSignature", "trusted_source_signer"] {
        if production.contains(forbidden) {
            findings.push(finding(RELEASE_CORE, "signature-authority", forbidden));
        }
    }
    Ok(())
}

fn inspect_fixtures(root: &Path) -> Result<(), String> {
    let positive = read_bounded(&root.join(POSITIVE_FIXTURE))?;
    let mut positive_findings = Vec::new();
    scan_text(POSITIVE_FIXTURE, &positive, FORBIDDEN_CORE_MARKERS, &mut positive_findings);
    if !positive_findings.is_empty() {
        return Err("positive fixture produced an authority finding".to_string());
    }
    for (name, expected) in NEGATIVE_FIXTURES {
        let text = read_bounded(&root.join(NEGATIVE_FIXTURE_ROOT).join(name))?;
        let mut findings = Vec::new();
        scan_text(name, &text, FORBIDDEN_CORE_MARKERS, &mut findings);
        if !findings.iter().any(|finding| finding.authority == *expected) {
            return Err(format!("negative fixture {name} did not report {expected}"));
        }
    }
    Ok(())
}

fn combined_rust_sources(root: &Path) -> Result<String, String> {
    let mut combined = String::new();
    for source in rust_sources(root)? {
        combined.push_str(&read_bounded(&source)?);
        combined.push('\n');
    }
    Ok(combined)
}

fn scan_tree(
    repository_root: &Path,
    source_root: &Path,
    markers: &[(&str, &str)],
    findings: &mut Vec<Finding>,
) -> Result<(), String> {
    for source in rust_sources(source_root)? {
        let relative = relative_string(repository_root, &source)?;
        scan_text(&relative, &read_bounded(&source)?, markers, findings);
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
    eprintln!("source-observation architecture check failed during {stage}: {reason}");
    std::process::exit(1)
}
