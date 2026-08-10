#!/usr/bin/env -S nix shell "github:nix-community/fenix?rev=092bd452904e749efa39907aa4a20a42678ac31e#minimal.toolchain" nixpkgs#gcc -c cargo -q -Zscript
---
[package]
edition = "2024"

[dependencies]
sha2 = "0.10"
---

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use sha2::Digest as _;
use sha2::Sha256;

const UPSTREAM_COMMIT: &str = "2cfc0f90ed83ea3cc983e5c305f89494a6df073e";
const PACKAGE_VERSION: &str = "0.1.0";
const PACKAGE_CHECKSUM_SHA256: &str = "a5d03dfde06a8ce7e0e007f4795ab74c546e5c7d6c6375a08ceb20677ec8a074";
const PACKAGE_LICENSE: &str = "Apache-2.0";
const SUPPORTED_NIX_VERSION: &str = "2.34";
const ADAPTER_PATH: &str = "src/nix_derivation_adapter.rs";
const ADMISSION_CONFIG_PATH: &str = "config/nix-derivation-admission.ncl";
const WORKSPACE_MANIFEST_PATH: &str = "Cargo.toml";
const WORKSPACE_LOCK_PATH: &str = "Cargo.lock";
const MAIN_PATH: &str = "src/main.rs";
const PRODUCER_SHELL_PATH: &str = "src/foreign_import_cmd.rs";
const DIRECT_IMPORT: &str = "nix_derivation::";
const DIRECT_USE: &str = "use nix_derivation";
const MAX_SCAN_FILES: usize = 50_000;
const MAX_SCAN_DEPTH: u32 = 64;
const MAX_SCAN_BYTES: u64 = 1_073_741_824;
const HEX_ALPHABET_LEN: usize = 16;
const HEX_CHARS_PER_BYTE: usize = 2;
const NIBBLE_BITS: u32 = 4;
const LOW_NIBBLE_MASK: u8 = 0x0f;
const DEPENDENCY_IDENTITY_FINDING_COUNT: usize = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileEntry {
    path: String,
    bytes: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Cli {
    root: PathBuf,
    package_root: Option<PathBuf>,
    upstream_root: Option<PathBuf>,
    crate_archive: Option<PathBuf>,
    self_test: bool,
}

fn validate_direct_imports(files: &[FileEntry]) -> Vec<String> {
    let mut findings = Vec::new();
    for file in files {
        if !file.path.ends_with(".rs") || file.path.starts_with("scripts/") {
            continue;
        }
        let text = String::from_utf8_lossy(&file.bytes);
        let imports_dependency = text.contains(DIRECT_IMPORT) || text.contains(DIRECT_USE);
        if imports_dependency && file.path != ADAPTER_PATH {
            findings.push(format!("direct-nix-derivation-import:{}", file.path));
        }
        if file.path == PRODUCER_SHELL_PATH && text.contains("nix_compat::derivation::Derivation::from_aterm_bytes") {
            findings.push("legacy-nix-parser-in-producer-shell".to_string());
        }
    }
    findings.sort();
    assert!(findings.windows(2).all(|pair| pair[0] < pair[1]));
    findings
}

fn validate_contract_files(files: &[FileEntry]) -> Vec<String> {
    let mut findings = Vec::new();
    require_fragment(
        files,
        WORKSPACE_MANIFEST_PATH,
        &format!("nix-derivation = \"={PACKAGE_VERSION}\""),
        &mut findings,
    );
    require_fragment(files, WORKSPACE_LOCK_PATH, "name = \"nix-derivation\"", &mut findings);
    require_fragment(files, WORKSPACE_LOCK_PATH, &format!("checksum = \"{PACKAGE_CHECKSUM_SHA256}\""), &mut findings);
    require_fragment(files, MAIN_PATH, "mod nix_derivation_adapter;", &mut findings);
    require_fragment(
        files,
        PRODUCER_SHELL_PATH,
        "parse_prefix_aware_aterm_bundle(NIX_LOGICAL_STORE_PREFIX, inputs)",
        &mut findings,
    );
    require_fragment(files, ADMISSION_CONFIG_PATH, UPSTREAM_COMMIT, &mut findings);
    require_fragment(files, ADMISSION_CONFIG_PATH, PACKAGE_CHECKSUM_SHA256, &mut findings);
    require_fragment(files, ADMISSION_CONFIG_PATH, PACKAGE_LICENSE, &mut findings);
    require_fragment(files, ADMISSION_CONFIG_PATH, SUPPORTED_NIX_VERSION, &mut findings);
    findings
}

fn require_fragment(files: &[FileEntry], path: &str, fragment: &str, findings: &mut Vec<String>) {
    let Some(file) = files.iter().find(|file| file.path == path) else {
        findings.push(format!("missing-contract-file:{path}"));
        return;
    };
    let text = String::from_utf8_lossy(&file.bytes);
    if !text.contains(fragment) {
        findings.push(format!("missing-contract-fragment:{path}:{fragment}"));
    }
}

fn validate_source_parity(
    package_files: &[FileEntry],
    upstream_files: &[FileEntry],
    vcs_info: &str,
    archive_sha256: &str,
) -> Vec<String> {
    let mut findings = validate_dependency_identity(vcs_info, archive_sha256);
    let package = package_projection(package_files, &mut findings);
    let upstream = file_map(upstream_files, &mut findings, "upstream");
    for (path, expected) in &upstream {
        match package.get(path) {
            None => findings.push(format!("package-source-missing:{path}")),
            Some(actual) if actual != expected => findings.push(format!("package-source-drift:{path}")),
            Some(_) => {}
        }
    }
    for path in package.keys() {
        if !upstream.contains_key(path) {
            findings.push(format!("package-source-unreviewed:{path}"));
        }
    }
    findings.sort();
    findings.dedup();
    assert!(findings.windows(2).all(|pair| pair[0] < pair[1]));
    findings
}

fn validate_dependency_identity(vcs_info: &str, archive_sha256: &str) -> Vec<String> {
    let mut findings = Vec::new();
    if !vcs_info.contains(UPSTREAM_COMMIT) {
        findings.push("package-vcs-commit-mismatch".to_string());
    }
    if archive_sha256 != PACKAGE_CHECKSUM_SHA256 {
        findings.push("package-archive-checksum-mismatch".to_string());
    }
    findings
}

fn package_projection(files: &[FileEntry], findings: &mut Vec<String>) -> BTreeMap<String, Vec<u8>> {
    let mut projected = BTreeMap::new();
    for file in files {
        if generated_package_file(&file.path) {
            continue;
        }
        let path = if file.path == "Cargo.toml.orig" {
            "Cargo.toml"
        } else {
            file.path.as_str()
        };
        if projected.insert(path.to_string(), file.bytes.clone()).is_some() {
            findings.push(format!("duplicate-package-projection:{path}"));
        }
    }
    projected
}

fn generated_package_file(path: &str) -> bool {
    matches!(path, ".cargo-ok" | ".cargo_vcs_info.json" | "Cargo.lock" | "Cargo.toml")
}

fn file_map(files: &[FileEntry], findings: &mut Vec<String>, label: &str) -> BTreeMap<String, Vec<u8>> {
    let mut mapped = BTreeMap::new();
    for file in files {
        if mapped.insert(file.path.clone(), file.bytes.clone()).is_some() {
            findings.push(format!("duplicate-{label}-path:{}", file.path));
        }
    }
    mapped
}

fn collect_repo_files(root: &Path) -> Result<Vec<FileEntry>, String> {
    collect_files(root, |path| skipped_repo_directory(path))
}

fn collect_package_files(root: &Path) -> Result<Vec<FileEntry>, String> {
    collect_files(root, |_| false)
}

fn collect_upstream_files(root: &Path) -> Result<Vec<FileEntry>, String> {
    collect_files(root, |path| path.file_name().and_then(|name| name.to_str()) == Some(".git"))
}

fn collect_files<F>(root: &Path, skip_directory: F) -> Result<Vec<FileEntry>, String>
where F: Fn(&Path) -> bool {
    let mut pending = vec![(root.to_path_buf(), 0_u32)];
    let mut files = Vec::new();
    let mut total_bytes = 0_u64;
    while let Some((directory, depth)) = pending.pop() {
        if depth > MAX_SCAN_DEPTH {
            return Err(format!("source scan exceeded depth {MAX_SCAN_DEPTH}"));
        }
        let entries = fs::read_dir(&directory).map_err(|error| format!("reading {}: {error}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("reading entry in {}: {error}", directory.display()))?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|error| format!("reading type {}: {error}", path.display()))?;
            if file_type.is_dir() {
                if !skip_directory(&path) {
                    pending.push((path, depth.saturating_add(1)));
                }
                continue;
            }
            if !file_type.is_file() {
                return Err(format!("source scan rejects non-regular path: {}", path.display()));
            }
            push_file(root, &path, &mut files, &mut total_bytes)?;
        }
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    assert!(files.len() <= MAX_SCAN_FILES);
    assert!(total_bytes <= MAX_SCAN_BYTES);
    Ok(files)
}

fn push_file(root: &Path, path: &Path, files: &mut Vec<FileEntry>, total_bytes: &mut u64) -> Result<(), String> {
    if files.len() >= MAX_SCAN_FILES {
        return Err(format!("source scan exceeded file count {MAX_SCAN_FILES}"));
    }
    let bytes = fs::read(path).map_err(|error| format!("reading {}: {error}", path.display()))?;
    let bytes_len = u64::try_from(bytes.len()).map_err(|error| error.to_string())?;
    let next_total =
        total_bytes.checked_add(bytes_len).ok_or_else(|| "source scan byte count overflowed".to_string())?;
    if next_total > MAX_SCAN_BYTES {
        return Err(format!("source scan exceeded byte count {MAX_SCAN_BYTES}"));
    }
    let relative = path.strip_prefix(root).map_err(|error| error.to_string())?;
    files.push(FileEntry {
        path: relative.to_string_lossy().replace('\\', "/"),
        bytes,
    });
    *total_bytes = next_total;
    Ok(())
}

fn skipped_repo_directory(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | ".pi" | "target" | "vendor" | "vendor-deps")
    )
}

fn archive_sha256(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("reading crate archive {}: {error}", path.display()))?;
    let digest = Sha256::digest(bytes);
    Ok(encode_hex(&digest))
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; HEX_ALPHABET_LEN] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(HEX_CHARS_PER_BYTE));
    for byte in bytes {
        encoded.push(HEX[usize::from(byte >> NIBBLE_BITS)] as char);
        encoded.push(HEX[usize::from(byte & LOW_NIBBLE_MASK)] as char);
    }
    encoded
}

fn run_boundary_guard(root: &Path) -> Result<Vec<String>, String> {
    let files = collect_repo_files(root)?;
    let mut findings = validate_direct_imports(&files);
    findings.extend(validate_contract_files(&files));
    findings.sort();
    findings.dedup();
    Ok(findings)
}

fn run_source_parity(cli: &Cli) -> Result<Vec<String>, String> {
    let package_root = cli.package_root.as_ref().ok_or_else(|| "--package-root is required".to_string())?;
    let upstream_root = cli.upstream_root.as_ref().ok_or_else(|| "--upstream-root is required".to_string())?;
    let crate_archive = cli.crate_archive.as_ref().ok_or_else(|| "--crate-archive is required".to_string())?;
    let package_files = collect_package_files(package_root)?;
    let upstream_files = collect_upstream_files(upstream_root)?;
    let vcs_info = fs::read_to_string(package_root.join(".cargo_vcs_info.json"))
        .map_err(|error| format!("reading package VCS metadata: {error}"))?;
    let checksum = archive_sha256(crate_archive)?;
    Ok(validate_source_parity(&package_files, &upstream_files, &vcs_info, &checksum))
}

fn run_self_test() -> Result<(), String> {
    let positive_import = vec![FileEntry {
        path: ADAPTER_PATH.to_string(),
        bytes: b"use nix_derivation::Derivation;".to_vec(),
    }];
    let negative_import = vec![FileEntry {
        path: "src/foreign_derivation_import.rs".to_string(),
        bytes: b"use nix_derivation::Derivation;".to_vec(),
    }];
    let negative_legacy_parser = vec![FileEntry {
        path: PRODUCER_SHELL_PATH.to_string(),
        bytes: b"nix_compat::derivation::Derivation::from_aterm_bytes(bytes);".to_vec(),
    }];
    if !validate_direct_imports(&positive_import).is_empty() {
        return Err("adapter import was rejected".to_string());
    }
    if validate_direct_imports(&negative_import) != ["direct-nix-derivation-import:src/foreign_derivation_import.rs"] {
        return Err("direct-import negative fixture was not rejected".to_string());
    }
    if validate_direct_imports(&negative_legacy_parser) != ["legacy-nix-parser-in-producer-shell"] {
        return Err("legacy-parser negative fixture was not rejected".to_string());
    }
    self_test_source_parity()?;
    if !validate_dependency_identity(UPSTREAM_COMMIT, PACKAGE_CHECKSUM_SHA256).is_empty() {
        return Err("valid dependency identity was rejected".to_string());
    }
    if validate_dependency_identity("stale", "stale").len() != DEPENDENCY_IDENTITY_FINDING_COUNT {
        return Err("stale dependency identity was not rejected".to_string());
    }
    Ok(())
}

fn self_test_source_parity() -> Result<(), String> {
    let upstream = vec![FileEntry {
        path: "Cargo.toml".to_string(),
        bytes: b"reviewed".to_vec(),
    }];
    let package = vec![
        FileEntry {
            path: "Cargo.toml.orig".to_string(),
            bytes: b"reviewed".to_vec(),
        },
        FileEntry {
            path: ".cargo_vcs_info.json".to_string(),
            bytes: UPSTREAM_COMMIT.as_bytes().to_vec(),
        },
    ];
    if !validate_source_parity(&package, &upstream, UPSTREAM_COMMIT, PACKAGE_CHECKSUM_SHA256).is_empty() {
        return Err("matching source projection was rejected".to_string());
    }
    let mut drifted = package.clone();
    drifted[0].bytes = b"drifted".to_vec();
    let findings = validate_source_parity(&drifted, &upstream, UPSTREAM_COMMIT, PACKAGE_CHECKSUM_SHA256);
    if findings != ["package-source-drift:Cargo.toml"] {
        return Err("source-drift negative fixture was not rejected".to_string());
    }
    Ok(())
}

fn parse_cli() -> Result<Cli, String> {
    let mut cli = Cli {
        root: PathBuf::from("."),
        ..Cli::default()
    };
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => cli.root = next_path(&mut arguments, "--root")?,
            "--package-root" => cli.package_root = Some(next_path(&mut arguments, "--package-root")?),
            "--upstream-root" => cli.upstream_root = Some(next_path(&mut arguments, "--upstream-root")?),
            "--crate-archive" => cli.crate_archive = Some(next_path(&mut arguments, "--crate-archive")?),
            "--self-test" => cli.self_test = true,
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(cli)
}

fn next_path(arguments: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    arguments.next().map(PathBuf::from).ok_or_else(|| format!("{flag} requires a path"))
}

fn emit_findings(label: &str, findings: &[String]) -> ExitCode {
    if findings.is_empty() {
        println!("{label}: PASS");
        return ExitCode::SUCCESS;
    }
    for finding in findings {
        eprintln!("{label}: {finding}");
    }
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    let cli = match parse_cli() {
        Ok(cli) => cli,
        Err(error) => {
            eprintln!("nix-derivation boundary: {error}");
            return ExitCode::FAILURE;
        }
    };
    if cli.self_test {
        return match run_self_test() {
            Ok(()) => emit_findings("nix-derivation boundary self-test", &[]),
            Err(error) => emit_findings("nix-derivation boundary self-test", &[error]),
        };
    }
    let mut findings = match run_boundary_guard(&cli.root) {
        Ok(findings) => findings,
        Err(error) => return emit_findings("nix-derivation boundary", &[error]),
    };
    let parity_requested = cli.package_root.is_some() || cli.upstream_root.is_some() || cli.crate_archive.is_some();
    if parity_requested {
        match run_source_parity(&cli) {
            Ok(parity_findings) => findings.extend(parity_findings),
            Err(error) => findings.push(error),
        }
    }
    findings.sort();
    findings.dedup();
    emit_findings("nix-derivation boundary", &findings)
}
