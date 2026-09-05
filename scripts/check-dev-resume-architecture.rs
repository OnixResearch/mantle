#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"
---

//! Enforce the dev-resume functional-core and imperative-shell boundary.
//! r[verify source_built_fixed_point_improved_iteration.dev_cross_run_resume]

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

const CORE_ROOT: &str = "crates/crunch-dev-resume-core/src";
const SHELL_ROOT: &str = "src/source_built_fixed_point_shell/dev_resume";
const ROOT_SHELL: &str = "src/source_built_fixed_point_shell.rs";
const POSITIVE_FIXTURE: &str = "fixtures/dev-resume-architecture/positive/pure-core.rs";
const NEGATIVE_ROOT: &str = "fixtures/dev-resume-architecture/negative";
const MAX_SOURCE_FILES: u32 = 64;
const MAX_SOURCE_BYTES: u64 = 1_048_576;
const FORBIDDEN_CORE: &[(&str, &str)] = &[
    ("std::fs", "filesystem"),
    ("std::path", "path"),
    ("PathBuf", "path"),
    ("std::process", "process"),
    ("std::env", "environment"),
    ("SystemTime", "clock"),
    ("rand::", "random"),
    ("reqwest", "network"),
    ("crunch_store", "store"),
    ("println!", "rendering"),
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
    ("rendering.rs", "rendering"),
];
const REQUIRED_SHELL_MARKERS: &[&str] = &[
    "prepare_dev_resume",
    "publish_provider_prefix_manifest",
    "publish_fixed_point_complete",
    "FixedPointPublication",
    "published_bundle_identities",
    "write_dev_resume_report",
];
const REQUIRED_PUBLICATION_SITES: &[(&str, &[&str])] = &[
    (ROOT_SHELL, &[
        "ProviderPrefix::Transition",
        "ProviderPrefix::Stagex",
        "ProviderPrefix::Native",
        "ProviderPrefix::Complete",
        "cmd_cargo_free_fixed_point_self_build_with_publisher",
        "publish_fixed_point_complete",
    ]),
    ("src/cargo_free_self_build.rs", &[
        "FixedPointStage1Publisher",
        "publish_stage1_before_continuation",
        "publisher.publish_stage1()?",
        "continuation()",
    ]),
];
const REQUIRED_PROMOTED_GUARDS: &[&str] = &[
    "promoted proof checkpoints conflict with dev cache, resume, and fast-fail state",
    "dev resume requires --dev-provider-cache",
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Finding {
    path: String,
    authority: String,
    marker: String,
}

fn main() -> ExitCode {
    let root = configured_root().unwrap_or_else(|error| fail("arguments", &error));
    let findings = inspect_repository(&root).unwrap_or_else(|error| fail("repository", &error));
    inspect_fixtures(&root).unwrap_or_else(|error| fail("fixtures", &error));
    if findings.is_empty() {
        println!("dev-resume architecture verified: findings=0 negative-fixtures={}", NEGATIVE_FIXTURES.len());
        return ExitCode::SUCCESS;
    }
    for finding in findings {
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
                root = PathBuf::from(arguments.next().ok_or_else(|| "--root requires a path".to_string())?);
            }
            _ => return Err(format!("unsupported argument: {argument}")),
        }
    }
    Ok(root)
}

fn inspect_repository(root: &Path) -> Result<Vec<Finding>, String> {
    let core_root = root.join(CORE_ROOT);
    let lib = read_bounded(&core_root.join("lib.rs"))?;
    let mut findings = Vec::new();
    if !lib.contains("#![no_std]") || !lib.contains("extern crate alloc") {
        findings.push(finding(CORE_ROOT, "core-shape", "missing-no-std-or-alloc"));
    }
    for source in rust_sources(&core_root)? {
        let relative = source.strip_prefix(root).map_err(|error| error.to_string())?;
        scan(relative.to_string_lossy().as_ref(), &read_bounded(&source)?, &mut findings);
    }
    let shell = read_bounded(&root.join(SHELL_ROOT).join("../dev_resume.rs"))?;
    require_markers(SHELL_ROOT, &shell, REQUIRED_SHELL_MARKERS, &mut findings);
    for (path, markers) in REQUIRED_PUBLICATION_SITES {
        require_markers(path, &read_bounded(&root.join(path))?, markers, &mut findings);
    }
    let root_shell = read_bounded(&root.join(ROOT_SHELL))?;
    for marker in REQUIRED_PROMOTED_GUARDS {
        if !root_shell.contains(marker) {
            findings.push(finding(ROOT_SHELL, "promoted-guard", *marker));
        }
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

fn inspect_fixtures(root: &Path) -> Result<(), String> {
    inspect_marker_controls(REQUIRED_SHELL_MARKERS)?;
    for (_, markers) in REQUIRED_PUBLICATION_SITES {
        inspect_marker_controls(markers)?;
    }
    let mut positive = Vec::new();
    scan(POSITIVE_FIXTURE, &read_bounded(&root.join(POSITIVE_FIXTURE))?, &mut positive);
    if !positive.is_empty() {
        return Err("positive fixture produced an authority finding".to_string());
    }
    for (name, expected) in NEGATIVE_FIXTURES {
        let mut findings = Vec::new();
        scan(name, &read_bounded(&root.join(NEGATIVE_ROOT).join(name))?, &mut findings);
        if !findings.iter().any(|finding| finding.authority == *expected) {
            return Err(format!("negative fixture {name} did not report {expected}"));
        }
    }
    Ok(())
}

fn require_markers(path: &str, text: &str, markers: &[&str], findings: &mut Vec<Finding>) {
    for marker in markers {
        if !text.contains(marker) {
            findings.push(finding(path, "shell-shape", *marker));
        }
    }
}

fn inspect_marker_controls(markers: &[&str]) -> Result<(), String> {
    let positive = markers.join("\n");
    let mut findings = Vec::new();
    require_markers("positive-marker-control", &positive, markers, &mut findings);
    if !findings.is_empty() {
        return Err("positive shell marker control failed".to_string());
    }
    for missing in markers {
        let negative = markers.iter().filter(|marker| *marker != missing).copied().collect::<Vec<_>>().join("\n");
        let mut findings = Vec::new();
        require_markers("negative-marker-control", &negative, markers, &mut findings);
        if !findings.iter().any(|finding| finding.marker == *missing) {
            return Err(format!("negative shell marker control did not reject {missing}"));
        }
    }
    Ok(())
}

fn scan(path: &str, text: &str, findings: &mut Vec<Finding>) {
    for (marker, authority) in FORBIDDEN_CORE {
        if text.contains(marker) {
            findings.push(finding(path, *authority, *marker));
        }
    }
}

fn rust_sources(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut pending = vec![root.to_path_buf()];
    let mut sources = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).map_err(|error| format!("read {}: {error}", directory.display()))? {
            let entry = entry.map_err(|error| error.to_string())?;
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            if file_type.is_symlink() {
                return Err(format!("symlink is not admitted: {}", entry.path().display()));
            }
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if entry.path().extension().and_then(|value| value.to_str()) == Some("rs") {
                sources.push(entry.path());
                let count = u32::try_from(sources.len()).map_err(|_| "source count overflow".to_string())?;
                if count > MAX_SOURCE_FILES {
                    return Err("source file bound exceeded".to_string());
                }
            }
        }
    }
    sources.sort();
    Ok(sources)
}

fn read_bounded(path: &Path) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| format!("inspect {}: {error}", path.display()))?;
    if !metadata.is_file() || metadata.len() > MAX_SOURCE_BYTES {
        return Err(format!("source shape or size is invalid: {}", path.display()));
    }
    fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn finding(path: impl Into<String>, authority: impl Into<String>, marker: impl Into<String>) -> Finding {
    Finding {
        path: path.into(),
        authority: authority.into(),
        marker: marker.into(),
    }
}

fn fail(stage: &str, reason: &str) -> ! {
    eprintln!("dev-resume architecture check failed during {stage}: {reason}");
    std::process::exit(1)
}
