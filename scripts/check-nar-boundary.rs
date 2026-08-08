#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"
---

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

const FILE_COUNT_MAX: u32 = 20_000;
const DIRECTORY_DEPTH_MAX: u32 = 32;
const ADAPTER_PREFIX: &str = "crates/crunch-nar/";
const DIRECT_IMPORT: &str = "nix_archive::nar";
const BLOCKED_DECODE: &str = "decode_events";
const BLOCKED_RESTORE: &str = "restore_path";
const UPSTREAM_VERSION: &str = "0.1.0";
const UPSTREAM_COMMIT: &str = "14362ab589daa4869bda744d4fbe26a1914b5491";
const UPSTREAM_CHECKSUM: &str = "70e73d0af2e2dce844911f162414cb04cda4bca5a4847328a71034b244a6acf1";
const ADAPTER_MANIFEST: &str = "crates/crunch-nar/Cargo.toml";
const WORKSPACE_LOCK: &str = "Cargo.lock";
const BOUNDARY_DOC: &str = "docs/nix-archive-nar-boundary.md";
const FIXTURE_MANIFEST: &str = "crates/crunch-nar/fixtures/upstream/manifest.ncl";
const VENDORED_VCS_INFO: &str = "vendor-deps/nix-archive/.cargo_vcs_info.json";
const VENDORED_CHECKSUM: &str = "vendor-deps/nix-archive/.cargo-checksum.json";

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceFile {
    path: String,
    text: String,
}

fn validate_sources(files: &[SourceFile]) -> Vec<String> {
    let mut findings = Vec::new();
    for file in files {
        if !file.path.ends_with(".rs") || file.path.starts_with("scripts/") {
            continue;
        }
        if file.text.contains(DIRECT_IMPORT) && !file.path.starts_with(ADAPTER_PREFIX) {
            findings.push(format!("direct-nix-archive-import:{}", file.path));
        }
        if !file.path.starts_with(ADAPTER_PREFIX)
            && (file.text.contains(BLOCKED_DECODE) || file.text.contains(BLOCKED_RESTORE))
        {
            findings.push(format!("full-buffer-production-api:{}", file.path));
        }
    }
    findings
}

fn validate_dependency_identity(vcs_info: &str, checksum: &str) -> Vec<String> {
    let mut findings = Vec::new();
    if !vcs_info.contains(UPSTREAM_COMMIT) {
        findings.push("vendored-source-commit-mismatch".to_string());
    }
    if !checksum.contains(UPSTREAM_CHECKSUM) {
        findings.push("vendored-package-checksum-mismatch".to_string());
    }
    findings
}

fn validate_vendored_dependency(root: &Path) -> Vec<String> {
    let vcs_info = match fs::read_to_string(root.join(VENDORED_VCS_INFO)) {
        Ok(text) => text,
        Err(error) => return vec![format!("reading-vendored-source-identity:{error}")],
    };
    let checksum = match fs::read_to_string(root.join(VENDORED_CHECKSUM)) {
        Ok(text) => text,
        Err(error) => return vec![format!("reading-vendored-package-checksum:{error}")],
    };
    validate_dependency_identity(&vcs_info, &checksum)
}

fn validate_contract_files(files: &[SourceFile]) -> Vec<String> {
    let mut findings = Vec::new();
    require_fragment(files, ADAPTER_MANIFEST, "nix-archive = \"=0.1.0\"", &mut findings);
    require_fragment(files, WORKSPACE_LOCK, "name = \"nix-archive\"", &mut findings);
    require_fragment(files, WORKSPACE_LOCK, &format!("checksum = \"{UPSTREAM_CHECKSUM}\""), &mut findings);
    require_fragment(files, FIXTURE_MANIFEST, UPSTREAM_COMMIT, &mut findings);
    require_fragment(files, FIXTURE_MANIFEST, UPSTREAM_VERSION, &mut findings);
    require_fragment(files, FIXTURE_MANIFEST, UPSTREAM_CHECKSUM, &mut findings);
    require_fragment(files, "crates/crunch-store/src/archive.rs", "ingest_nar_and_hash", &mut findings);
    require_fragment(files, "crates/crunch-store/src/pull.rs", "ingest_nar_and_hash", &mut findings);
    require_fragment(files, "crates/crunch-store/src/handle.rs", "write_nar", &mut findings);
    require_fragment(files, "crates/crunch-rust-cache/src/shared.rs", "ingest_nar_and_hash", &mut findings);
    require_fragment(files, "crates/crunch-rust-cache/src/shared.rs", "write_nar", &mut findings);
    require_fragment(files, BOUNDARY_DOC, "fresh staging", &mut findings);
    require_fragment(files, BOUNDARY_DOC, "no-replace publication", &mut findings);
    require_fragment(files, BOUNDARY_DOC, "post-publication verification", &mut findings);
    findings
}

fn require_fragment(files: &[SourceFile], path: &str, fragment: &str, findings: &mut Vec<String>) {
    let Some(file) = files.iter().find(|file| file.path == path) else {
        findings.push(format!("missing-contract-file:{path}"));
        return;
    };
    if !file.text.contains(fragment) {
        findings.push(format!("missing-contract-fragment:{path}:{fragment}"));
    }
}

fn collect_sources(root: &Path) -> Result<Vec<SourceFile>, String> {
    let mut pending = vec![(root.to_path_buf(), 0_u32)];
    let mut files = Vec::new();
    while let Some((directory, depth)) = pending.pop() {
        if depth > DIRECTORY_DEPTH_MAX {
            return Err(format!("source scan exceeded directory depth {DIRECTORY_DEPTH_MAX}"));
        }
        let entries = fs::read_dir(&directory)
            .map_err(|error| format!("reading {}: {error}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("reading entry in {}: {error}", directory.display()))?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|error| format!("reading type {}: {error}", path.display()))?;
            if file_type.is_dir() {
                if skipped_directory(&path) {
                    continue;
                }
                pending.push((path, depth.saturating_add(1)));
                continue;
            }
            if !file_type.is_file() || !included_file(&path) {
                continue;
            }
            let relative = path.strip_prefix(root).map_err(|error| error.to_string())?;
            let text = fs::read_to_string(&path).map_err(|error| format!("reading {}: {error}", path.display()))?;
            files.push(SourceFile {
                path: relative.to_string_lossy().replace('\\', "/"),
                text,
            });
            if files.len() > usize::try_from(FILE_COUNT_MAX).unwrap() {
                return Err(format!("source scan exceeded file count {FILE_COUNT_MAX}"));
            }
        }
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    assert!(files.len() <= usize::try_from(FILE_COUNT_MAX).unwrap());
    assert!(files.windows(2).all(|pair| pair[0].path < pair[1].path));
    Ok(files)
}

fn skipped_directory(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | "target" | "vendor" | "vendor-deps" | ".pi")
    )
}

fn included_file(path: &Path) -> bool {
    if matches!(path.extension().and_then(|extension| extension.to_str()), Some("rs" | "toml" | "md" | "ncl")) {
        return true;
    }
    path.file_name().and_then(|name| name.to_str()) == Some("Cargo.lock")
}

fn run_self_test() -> Result<(), String> {
    let positive = vec![
        SourceFile {
            path: "crates/crunch-nar/src/lib.rs".to_string(),
            text: "use nix_archive::nar;".to_string(),
        },
        SourceFile {
            path: "crates/crunch-store/src/archive.rs".to_string(),
            text: "ingest_nar_and_hash(reader);".to_string(),
        },
    ];
    let negative_import = vec![SourceFile {
        path: "crates/crunch-store/src/query.rs".to_string(),
        text: "use nix_archive::nar;".to_string(),
    }];
    let negative_restore = vec![SourceFile {
        path: "src/source_bundle.rs".to_string(),
        text: "restore_path(bytes, output);".to_string(),
    }];
    if !validate_sources(&positive).is_empty() {
        return Err("positive source fixture was rejected".to_string());
    }
    if validate_sources(&negative_import) != ["direct-nix-archive-import:crates/crunch-store/src/query.rs"] {
        return Err("direct-import negative fixture was not rejected".to_string());
    }
    if validate_sources(&negative_restore) != ["full-buffer-production-api:src/source_bundle.rs"] {
        return Err("restore negative fixture was not rejected".to_string());
    }
    if !validate_dependency_identity(UPSTREAM_COMMIT, UPSTREAM_CHECKSUM).is_empty() {
        return Err("positive dependency identity was rejected".to_string());
    }
    if validate_dependency_identity("stale", "stale").len() != 2 {
        return Err("stale dependency identity was not rejected".to_string());
    }
    Ok(())
}

fn main() -> ExitCode {
    if env::args().any(|argument| argument == "--self-test") {
        return match run_self_test() {
            Ok(()) => {
                println!("nar-boundary self-test: PASS");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("nar-boundary self-test: FAIL: {error}");
                ExitCode::FAILURE
            }
        };
    }
    let root = PathBuf::from(".");
    let files = match collect_sources(&root) {
        Ok(files) => files,
        Err(error) => {
            eprintln!("nar-boundary guard: FAIL: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut findings = validate_sources(&files);
    findings.extend(validate_contract_files(&files));
    findings.extend(validate_vendored_dependency(&root));
    if findings.is_empty() {
        println!("nar-boundary guard: PASS ({} files)", files.len());
        return ExitCode::SUCCESS;
    }
    for finding in findings {
        eprintln!("nar-boundary guard: {finding}");
    }
    ExitCode::FAILURE
}
