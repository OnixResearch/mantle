//! Deterministic stale Crunch/crunch branding classifier for the Mantle rename.
//!
//! This intentionally uses only `git ls-files` plus substring allow rules so it
//! can run on a fresh checkout without external grep/regex dependencies.

use std::process::Command;
use std::process::ExitCode;

const BRAND_TOKENS: [&str; 3] = ["Crunch", "crunch", "CRUNCH"];

const SKIP_PATH_PREFIXES: [&str; 20] = [
    "openspec/changes/archive/",
    "openspec/changes/live-",
    "vendor/",
    "target/",
    ".git/",
    ".autoresearch/",
    "assets/",
    "bootstrap/",
    "builders/",
    "dev-tools/",
    "lib/",
    "packages/",
    "patches/",
    "src/",
    "tests/",
    "tmp/",
    "vendor-deps/",
    "crates/",
    "benches/",
    ".agent/",
];

const SKIP_PATHS: [&str; 7] = [
    "Cargo.lock",
    "Cargo.toml",
    "AGENTS.md",
    "flake.nix",
    "dylint.toml",
    "openspec/changes/rename-crunch-to-mantle/evidence/identity-inventory.md",
    "scripts/check-stale-branding.rs",
];

const ALLOWED_PATH_SUBSTRINGS: [&str; 5] = [
    "tests/",
    "crates/crunch-",
    "examples/benchmark_",
    "openspec/changes/rename-crunch-to-mantle/",
    "adr/",
];

const ALLOWED_LINE_SUBSTRINGS: &[&str] = &[
    "CRUNCH_",
    "CRUNCH-",
    "crunch-",
    "crunch_",
    "crunch::",
    "crunch/",
    "crunch.fetch",
    "crunch.ncl",
    "crunch.lock",
    "crunch-project.ncl",
    ".crunch",
    "/crunch/store",
    "crunch-build-report-v1",
    "crunch-benchmark-bundle-v1",
    "CrunchDerivation",
    "crunch_eval::",
    "crunch_glue::",
    "crunch-pipeline::",
    "crunch_glue",
    "crunch_eval",
    "crunch_store",
    "nix-community/",
    "github.com/",
    "gitlab.com/",
    "legacy `crunch`",
    "legacy crunch",
    "legacy Crunch",
    "compatibility",
    "Compatibility",
    "migration",
    "Migration",
    "pre-rename",
    "historical",
    "Historical",
    "archive",
    "archived",
    "external",
    "upstream",
    "former",
];

fn main() -> ExitCode {
    let output = Command::new("git").args(["ls-files"]).output().expect("git ls-files must run");
    if !output.status.success() {
        eprintln!("error: git ls-files failed");
        return ExitCode::FAILURE;
    }

    let mut failures = Vec::new();
    let files = String::from_utf8(output.stdout).expect("git paths are utf-8");
    for path in files.lines() {
        if is_skipped_path(path) {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        for (idx, line) in content.lines().enumerate() {
            if !BRAND_TOKENS.iter().any(|token| line.contains(token)) {
                continue;
            }
            if is_allowed_occurrence(path, line) {
                continue;
            }
            failures.push(format!("{}:{}: {}", path, idx + 1, line.trim()));
        }
    }

    if failures.is_empty() {
        println!("stale branding check passed");
        ExitCode::SUCCESS
    } else {
        eprintln!("stale Crunch/crunch branding requires classification:");
        for failure in failures {
            eprintln!("  {failure}");
        }
        ExitCode::FAILURE
    }
}

fn is_skipped_path(path: &str) -> bool {
    SKIP_PATHS.contains(&path) || SKIP_PATH_PREFIXES.iter().any(|prefix| path.starts_with(prefix))
}

fn is_allowed_occurrence(path: &str, line: &str) -> bool {
    ALLOWED_PATH_SUBSTRINGS.iter().any(|part| path.contains(part))
        || ALLOWED_LINE_SUBSTRINGS.iter().any(|part| line.contains(part))
}
