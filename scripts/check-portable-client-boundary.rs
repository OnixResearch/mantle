#!/usr/bin/env -S nix shell "github:nix-community/fenix?rev=092bd452904e749efa39907aa4a20a42678ac31e#minimal.toolchain" nixpkgs#gcc -c cargo -q -Zscript
---
[package]
edition = "2024"
---

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

const CORE_MANIFEST: &str = "crates/mantle-portable-client-core/Cargo.toml";
const CORE_SOURCE: &str = "crates/mantle-portable-client-core/src/lib.rs";
const ROOT_SOURCE: &str = "src/main.rs";
const FORBIDDEN_DEPENDENCIES: [&str; 10] = [
    "bwrap",
    "fuse",
    "seccomp",
    "cgroup",
    "snix-build",
    "crunch-build",
    "crunch-pipeline",
    "crunch-store",
    "tokio",
    "libc",
];
const FORBIDDEN_CORE_SOURCE: [&str; 8] = [
    "std::",
    "std::process",
    "std::fs",
    "tokio::",
    "Command::new",
    "TcpStream",
    "UnixStream",
    "/nix/store",
];
const FORBIDDEN_SECRET_TOKENS: [&str; 4] = ["BEGIN PRIVATE KEY", "Bearer ", "ticket.secret", "secret_key"];

fn findings(manifest: &str, source: &str, root_source: &str) -> Vec<String> {
    let mut result = Vec::new();
    if !source.contains("#![no_std]") {
        result.push("core-missing-no-std-boundary".to_string());
    }
    for dependency in FORBIDDEN_DEPENDENCIES {
        if manifest.contains(dependency) {
            result.push(format!("forbidden-portable-dependency:{dependency}"));
        }
    }
    for token in FORBIDDEN_CORE_SOURCE {
        if source.contains(token) {
            result.push(format!("forbidden-portable-source:{token}"));
        }
    }
    for token in FORBIDDEN_SECRET_TOKENS {
        if manifest.contains(token) || source.contains(token) {
            result.push(format!("secret-token-in-portable-closure:{token}"));
        }
    }
    let admission = root_source.find("enforce_portable_command_admission");
    let state_override = root_source.find("apply_state_dir_override(&args)");
    match (admission, state_override) {
        (Some(admission), Some(state_override)) if admission < state_override => {}
        _ => result.push("portable-admission-does-not-precede-state-override".to_string()),
    }
    result
}

fn self_test() -> Result<(), String> {
    let positive_manifest = "[dependencies]\nserde = \"1\"\n";
    let positive_source = "#![no_std]\npub fn plan() {}\n";
    let positive_root = "enforce_portable_command_admission();\napply_state_dir_override(&args);\n";
    if !findings(positive_manifest, positive_source, positive_root).is_empty() {
        return Err("positive fixture produced findings".to_string());
    }
    let negative_manifest = "[dependencies]\nbwrap = \"1\"\n";
    let negative = findings(negative_manifest, positive_source, positive_root);
    if negative != ["forbidden-portable-dependency:bwrap"] {
        return Err(format!("negative fixture mismatch: {negative:?}"));
    }
    let side_effect_source = "#![no_std]\nfn shell() { std::fs::read(\"x\"); }\n";
    let negative = findings(positive_manifest, side_effect_source, positive_root);
    if !negative.iter().any(|finding| finding.starts_with("forbidden-portable-source:")) {
        return Err("side-effect fixture was accepted".to_string());
    }
    let secret_source = "#![no_std]\nconst VALUE: &str = \"Bearer token\";\n";
    let negative = findings(positive_manifest, secret_source, positive_root);
    if !negative.iter().any(|finding| finding.starts_with("secret-token-in-portable-closure:")) {
        return Err("secret fixture was accepted".to_string());
    }
    Ok(())
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("reading {}: {error}", path.display()))
}

fn run(root: &Path) -> Result<(), String> {
    self_test()?;
    let manifest = read(&root.join(CORE_MANIFEST))?;
    let source = read(&root.join(CORE_SOURCE))?;
    let root_source = read(&root.join(ROOT_SOURCE))?;
    let found = findings(&manifest, &source, &root_source);
    if !found.is_empty() {
        return Err(found.join("\n"));
    }
    println!("portable client boundary: PASS");
    Ok(())
}

fn main() -> ExitCode {
    let root = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    match run(Path::new(&root)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("portable client boundary: FAIL\n{error}");
            ExitCode::FAILURE
        }
    }
}
