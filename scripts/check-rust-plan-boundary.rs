#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"
---

//! Boundary guard for the Rust-plan functional core and application contract.
//!
//! The core owns deterministic planning meaning; the application crate owns
//! ports and orchestration. Neither may reach host capability directly:
//! filesystem, process, environment, clock, network, async runtime, CLI error,
//! path, rendering, or cargo/rustc invocation tokens must not appear in their
//! sources, and their manifests may only depend on the reviewed cohort.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

const CORE_SOURCES: &str = "crates/mantle-rust-plan-core/src";
const APP_SOURCES: &str = "crates/mantle-rust-plan-app/src";
const CORE_MANIFEST: &str = "crates/mantle-rust-plan-core/Cargo.toml";
const APP_MANIFEST: &str = "crates/mantle-rust-plan-app/Cargo.toml";
const FILE_COUNT_MAX: u32 = 512;
const DIRECTORY_DEPTH_MAX: u32 = 8;

/// Tokens that must never appear in the pure crates.
const FORBIDDEN_TOKENS: &[&str] = &[
    "std::fs",
    "std::process",
    "std::env",
    "std::time",
    "std::thread",
    "std::path",
    "std::net",
    "std::io",
    "tokio::",
    "reqwest::",
    "Command::new",
    "println!",
    "eprintln!",
    "RunError",
    "serde_json::to_string(",
    "serde_json::to_string_pretty",
];

/// Reviewed dependency cohort per manifest.
const CORE_ALLOWED_DEPENDENCIES: &[&str] = &["blake3", "serde", "serde_json"];
const APP_ALLOWED_DEPENDENCIES: &[&str] = &["mantle-rust-plan-core", "serde"];

fn main() -> ExitCode {
    if env::args().any(|argument| argument == "--self-test") {
        return match run_self_test() {
            Ok(()) => {
                println!("rust-plan-boundary self-test: PASS");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("rust-plan-boundary self-test: FAIL: {error}");
                ExitCode::FAILURE
            }
        };
    }
    let root = PathBuf::from(".");
    let files = match collect_sources(&root) {
        Ok(files) => files,
        Err(error) => {
            eprintln!("rust-plan-boundary guard: FAIL: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut findings = validate_sources(&files);
    findings.extend(validate_manifest_dependencies(
        &root,
        CORE_MANIFEST,
        CORE_ALLOWED_DEPENDENCIES,
    ));
    findings.extend(validate_manifest_dependencies(
        &root,
        APP_MANIFEST,
        APP_ALLOWED_DEPENDENCIES,
    ));
    if findings.is_empty() {
        println!("rust-plan-boundary guard: PASS ({} files)", files.len());
        return ExitCode::SUCCESS;
    }
    for finding in findings {
        eprintln!("rust-plan-boundary guard: {finding}");
    }
    ExitCode::FAILURE
}

fn collect_sources(root: &Path) -> Result<Vec<(String, String)>, String> {
    let mut files = Vec::new();
    for relative in [CORE_SOURCES, APP_SOURCES] {
        collect_directory(&root.join(relative), relative, 0, &mut files)?;
    }
    if files.is_empty() {
        return Err(String::from("no Rust-plan sources found"));
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(files)
}

fn collect_directory(
    directory: &Path,
    prefix: &str,
    depth: u32,
    files: &mut Vec<(String, String)>,
) -> Result<(), String> {
    if depth > DIRECTORY_DEPTH_MAX {
        return Err(format!("directory depth exceeded under {prefix}"));
    }
    let entries = fs::read_dir(directory).map_err(|error| format!("reading {prefix}: {error}"))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("reading entry under {prefix}: {error}"))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let relative = format!("{prefix}/{name}");
        let metadata = entry
            .metadata()
            .map_err(|error| format!("stat {relative}: {error}"))?;
        if metadata.is_dir() {
            collect_directory(&path, &relative, depth.saturating_add(1), files)?;
            continue;
        }
        if !name.ends_with(".rs") {
            continue;
        }
        if u32::try_from(files.len()).unwrap_or(FILE_COUNT_MAX) >= FILE_COUNT_MAX {
            return Err(String::from("source file count exceeded"));
        }
        let text = fs::read_to_string(&path).map_err(|error| format!("reading {relative}: {error}"))?;
        files.push((relative, text));
    }
    Ok(())
}

fn validate_sources(files: &[(String, String)]) -> Vec<String> {
    let mut findings = Vec::new();
    for (path, text) in files {
        for (index, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("//!") {
                continue;
            }
            for token in FORBIDDEN_TOKENS {
                if line.contains(token) {
                    findings.push(format!("{}:{} forbidden token {token}", path, index + 1));
                }
            }
        }
    }
    findings
}

fn validate_manifest_dependencies(root: &Path, manifest: &str, allowed: &[&str]) -> Vec<String> {
    let mut findings = Vec::new();
    let path = root.join(manifest);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => return vec![format!("{manifest}: unreadable: {error}")],
    };
    let mut in_dependencies = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_dependencies = trimmed == "[dependencies]";
            continue;
        }
        if !in_dependencies || trimmed.is_empty() {
            continue;
        }
        let name = trimmed.split('=').next().unwrap_or_default().trim();
        if name.is_empty() {
            continue;
        }
        if !allowed.contains(&name) {
            findings.push(format!("{manifest}: unreviewed dependency {name}"));
        }
    }
    findings
}

fn run_self_test() -> Result<(), String> {
    let allowed_source = String::from(
        "use alloc::string::String;\npub fn plan() -> String { String::from(\"ok\") }\n",
    );
    let positive = vec![(String::from("fixture/allowed.rs"), allowed_source)];
    if !validate_sources(&positive).is_empty() {
        return Err(String::from("positive fixture must pass"));
    }
    for token in ["std::fs::read", "std::process::Command", "println!"] {
        let negative_source = format!("pub fn reach() {{ {token} }}\n");
        let negative = vec![(String::from("fixture/forbidden.rs"), negative_source)];
        let findings = validate_sources(&negative);
        if findings.is_empty() {
            return Err(format!("negative fixture for {token} must fail"));
        }
    }
    let allowed_manifest = String::from("[dependencies]\nserde = \"1.0\"\n");
    if !manifest_findings_for(&allowed_manifest, &["serde"]).is_empty() {
        return Err(String::from("allowed manifest must pass"));
    }
    let forbidden_manifest = String::from("[dependencies]\ntokio = \"1\"\n");
    if manifest_findings_for(&forbidden_manifest, &["serde"]).is_empty() {
        return Err(String::from("unreviewed manifest dependency must fail"));
    }
    Ok(())
}

/// Manifest dependency scan over supplied text, shared by the self-test.
fn manifest_findings_for(text: &str, allowed: &[&str]) -> Vec<String> {
    let mut findings = Vec::new();
    let mut in_dependencies = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_dependencies = trimmed == "[dependencies]";
            continue;
        }
        if !in_dependencies || trimmed.is_empty() {
            continue;
        }
        let name = trimmed.split('=').next().unwrap_or_default().trim();
        if !name.is_empty() && !allowed.contains(&name) {
            findings.push(format!("unreviewed dependency {name}"));
        }
    }
    findings
}
