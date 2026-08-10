#!/usr/bin/env -S nix shell "github:nix-community/fenix?rev=092bd452904e749efa39907aa4a20a42678ac31e#minimal.toolchain" nixpkgs#gcc -c cargo -q -Zscript
---
[package]
edition = "2024"
---

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const DYNAMIC_PATH: &str = "crates/crunch-build/src/dynamic.rs";
const WORKER_PATH: &str = "crates/crunch-build/src/worker.rs";
const REGISTRY_PATH: &str = "crates/crunch-build/src/registry.rs";
const LIB_PATH: &str = "crates/crunch-build/src/lib.rs";
const TEST_BOUNDARY: &str = "#[cfg(test)]";
const FILE_BYTES_MAX: u64 = 2_097_152;
const REQUIRED_STAGE_COUNT: usize = 4;

#[derive(Debug)]
struct Sources {
    dynamic: String,
    worker: String,
    registry: String,
    library: String,
}

fn read_bounded(path: &Path) -> Result<String, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("reading metadata for {}: {error}", path.display()))?;
    if metadata.len() > FILE_BYTES_MAX {
        return Err(format!("{} exceeds {FILE_BYTES_MAX} bytes", path.display()));
    }
    fs::read_to_string(path).map_err(|error| format!("reading {}: {error}", path.display()))
}

fn load_sources(root: &Path) -> Result<Sources, String> {
    Ok(Sources {
        dynamic: read_bounded(&root.join(DYNAMIC_PATH))?,
        worker: read_bounded(&root.join(WORKER_PATH))?,
        registry: read_bounded(&root.join(REGISTRY_PATH))?,
        library: read_bounded(&root.join(LIB_PATH))?,
    })
}

fn production_source(source: &str) -> &str {
    source.split_once(TEST_BOUNDARY).map_or(source, |(production, _)| production)
}

fn require(source: &str, token: &str, finding: &str, findings: &mut Vec<String>) {
    if !source.contains(token) {
        findings.push(finding.to_string());
    }
}

fn forbid(source: &str, token: &str, finding: &str, findings: &mut Vec<String>) {
    if source.contains(token) {
        findings.push(finding.to_string());
    }
}

fn validate_sources(sources: &Sources) -> Vec<String> {
    let dynamic = production_source(&sources.dynamic);
    let mut findings = Vec::new();
    for stage in [
        "ParsedDynamicDerivation",
        "ValidatedDynamicDerivation",
        "IdentityResolvedDynamicDerivation",
        "RegistryReadyDynamicDerivation",
    ] {
        require(dynamic, stage, "dynamic-admission-missing-private-stage", &mut findings);
    }
    let stage_count = [
        "ParsedDynamicDerivation",
        "ValidatedDynamicDerivation",
        "IdentityResolvedDynamicDerivation",
        "RegistryReadyDynamicDerivation",
    ]
    .iter()
    .filter(|stage| dynamic.contains(**stage))
    .count();
    if stage_count != REQUIRED_STAGE_COUNT {
        findings.push("dynamic-admission-stage-count-mismatch".to_string());
    }
    for forbidden in [
        "std::fs",
        "tokio::",
        "tracing::",
        "DerivationRegistry",
        "std::process",
        "SystemTime",
        "nix_derivation",
        "[0u8; 32]",
        "[0_u8; 32]",
        "unwrap_or(u64::MAX)",
        "unwrap_or(u32::MAX)",
        ".expect(",
    ] {
        forbid(dynamic, forbidden, "dynamic-admission-core-effect-or-fallback", &mut findings);
    }
    require(
        dynamic,
        "validate_registry_ready_batch",
        "dynamic-admission-missing-batch-preflight",
        &mut findings,
    );
    require(
        &sources.worker,
        "insert_registry_ready_dynamic",
        "dynamic-admission-worker-not-wired",
        &mut findings,
    );
    require(
        &sources.registry,
        "dynamic_admission_identity",
        "dynamic-admission-registry-identity-missing",
        &mut findings,
    );
    require(
        &sources.registry,
        "RegistryReadyDynamicDerivation",
        "dynamic-admission-registry-type-gate-missing",
        &mut findings,
    );
    forbid(
        &sources.library,
        "register_dynamic_drv",
        "dynamic-admission-legacy-register-export",
        &mut findings,
    );
    forbid(
        &sources.library,
        "parse_drv_bytes",
        "dynamic-admission-legacy-parse-export",
        &mut findings,
    );
    findings.sort();
    findings.dedup();
    findings
}

fn valid_fixture() -> Sources {
    Sources {
        dynamic: "struct ParsedDynamicDerivation; struct ValidatedDynamicDerivation; struct IdentityResolvedDynamicDerivation; struct RegistryReadyDynamicDerivation; fn validate_registry_ready_batch() {} #[cfg(test)] mod tests { let _ = [0_u8; 32]; }".to_string(),
        worker: "fn shell() { known_paths.insert_registry_ready_dynamic(&ready); }".to_string(),
        registry: "dynamic_admission_identity: Option<[u8; 32]>; fn insert(value: RegistryReadyDynamicDerivation) {}".to_string(),
        library: "pub use dynamic::DynamicDrv;".to_string(),
    }
}

fn run_self_test() -> Result<(), String> {
    let valid = valid_fixture();
    let findings = validate_sources(&valid);
    if !findings.is_empty() {
        return Err(format!("positive fixture failed: {findings:?}"));
    }
    for (token, expected) in [
        ("std::fs::read", "dynamic-admission-core-effect-or-fallback"),
        ("DerivationRegistry", "dynamic-admission-core-effect-or-fallback"),
        ("[0_u8; 32]", "dynamic-admission-core-effect-or-fallback"),
        (".expect(", "dynamic-admission-core-effect-or-fallback"),
    ] {
        let mut invalid = valid_fixture();
        invalid.dynamic = invalid.dynamic.replace(TEST_BOUNDARY, &format!("{token} {TEST_BOUNDARY}"));
        let findings = validate_sources(&invalid);
        if !findings.iter().any(|finding| finding == expected) {
            return Err(format!("negative fixture `{token}` missed `{expected}`: {findings:?}"));
        }
    }
    let mut invalid = valid_fixture();
    invalid.library.push_str(" pub use dynamic::register_dynamic_drv;");
    let findings = validate_sources(&invalid);
    if !findings
        .iter()
        .any(|finding| finding == "dynamic-admission-legacy-register-export")
    {
        return Err(format!("legacy export fixture was accepted: {findings:?}"));
    }
    println!("dynamic admission boundary self-test: PASS");
    Ok(())
}

fn run(root: &Path) -> Result<(), String> {
    let sources = load_sources(root)?;
    let findings = validate_sources(&sources);
    if !findings.is_empty() {
        return Err(format!("dynamic admission boundary findings: {}", findings.join(", ")));
    }
    println!("dynamic admission boundary: PASS");
    Ok(())
}

fn main() -> ExitCode {
    let mut root = PathBuf::from(".");
    let mut self_test = false;
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => {
                let Some(value) = arguments.next() else {
                    eprintln!("dynamic admission boundary: --root requires a path");
                    return ExitCode::FAILURE;
                };
                root = PathBuf::from(value);
            }
            "--self-test" => self_test = true,
            _ => {
                eprintln!("dynamic admission boundary: unknown argument: {argument}");
                return ExitCode::FAILURE;
            }
        }
    }
    let result = if self_test { run_self_test() } else { run(&root) };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("dynamic admission boundary: {error}");
            ExitCode::FAILURE
        }
    }
}
