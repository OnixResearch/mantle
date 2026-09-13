#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"
---

//! Architecture guard for the thin CLI composition root.
//!
//! The contract crate owns typed commands, blockers, and ports; presentation
//! modules render typed values; adapters own host capability. This guard checks
//! those three boundaries from source text, and it approximates test-code
//! exclusion by ignoring everything from the first `#[cfg(test)]` marker to the
//! end of each scanned file.

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

/// Contract sources that must stay free of host capability and CLI errors.
const CONTRACT_SOURCES: &str = "crates/mantle-application-contract/src";

/// Presentation sources that must render typed values only.
const PRESENTATION_SOURCES: &str = "src/presentation";

/// Root sources whose authority must stay bounded.
const ROOT_SOURCES: &str = "src/main.rs";

/// Tokens that mean the contract reaches a host capability or a CLI error.
const CONTRACT_FORBIDDEN: &[&str] = &[
    "std::fs",
    "std::env",
    "std::process",
    "std::net",
    "std::thread",
    "std::time::Instant",
    "std::time::SystemTime",
    "tokio::",
    "RunError",
];

/// Tokens that mean presentation reads ambient state instead of typed inputs.
///
/// `RunError` itself is allowed here: presentation is the boundary where typed
/// blockers and capability errors become the CLI error type.
const PRESENTATION_FORBIDDEN: &[&str] = &[
    "RunContext",
    "std::env",
    "std::process",
    "std::net",
    "std::thread",
    "std::time::SystemTime",
    "tokio::",
];

/// Tokens the root may not use directly because adapters own them.
const ROOT_FORBIDDEN: &[&str] = &["std::process::Command", "std::fs::write", "std::fs::read"];

fn main() -> ExitCode {
    if env::args().any(|argument| argument == "--self-test") {
        return match self_test() {
            Ok(()) => {
                println!("cli-architecture self-test: PASS");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("cli-architecture self-test: FAIL: {error}");
                ExitCode::FAILURE
            }
        };
    }
    match scan_repository(Path::new(".")) {
        Ok(violations) if violations.is_empty() => {
            println!("cli architecture: PASS");
            ExitCode::SUCCESS
        }
        Ok(violations) => {
            for violation in violations {
                eprintln!("{violation}");
            }
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("cli architecture: FAIL: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Scan the repository boundaries and return every violation.
fn scan_repository(root: &Path) -> Result<Vec<String>, String> {
    let mut violations = Vec::new();
    violations.extend(scan_directory(&root.join(CONTRACT_SOURCES), CONTRACT_FORBIDDEN, "contract")?);
    violations.extend(scan_directory(&root.join(PRESENTATION_SOURCES), PRESENTATION_FORBIDDEN, "presentation")?);
    violations.extend(scan_file(&root.join(ROOT_SOURCES), ROOT_FORBIDDEN, "root")?);
    debug_assert!(violations.is_empty() || violations.iter().all(|entry| entry.contains(':')));
    Ok(violations)
}

/// Scan every Rust file in one directory tree.
fn scan_directory(directory: &Path, forbidden: &[&str], boundary: &str) -> Result<Vec<String>, String> {
    if !directory.is_dir() {
        return Err(format!("scanned directory is missing: {}", directory.display()));
    }
    let mut violations = Vec::new();
    for entry in fs::read_dir(directory).map_err(|error| format!("reading {}: {error}", directory.display()))? {
        let path = entry.map_err(|error| format!("reading directory entry: {error}"))?.path();
        if path.is_dir() {
            violations.extend(scan_directory(&path, forbidden, boundary)?);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        violations.extend(scan_file(&path, forbidden, boundary)?);
    }
    debug_assert!(violations.len() <= 10_000);
    Ok(violations)
}

/// Scan one file for forbidden tokens before its test module.
fn scan_file(path: &Path, forbidden: &[&str], boundary: &str) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("reading {}: {error}", path.display()))?;
    let production = production_prefix(&text);
    let mut violations = Vec::new();
    for (index, line) in production.lines().enumerate() {
        for token in forbidden {
            if line.contains(token) {
                violations.push(format!(
                    "{boundary} boundary: {}:{}: forbidden token `{token}`",
                    path.display(),
                    index + 1
                ));
            }
        }
    }
    debug_assert!(violations.is_empty() || !text.is_empty());
    Ok(violations)
}

/// Everything before the first test-module marker.
fn production_prefix(text: &str) -> &str {
    let marker = text.find("#[cfg(test)]\nmod tests {");
    match marker {
        Some(index) => &text[..index],
        None => text,
    }
}

/// Fixture-based self-test: accepted shapes pass, rejected shapes fail.
fn self_test() -> Result<(), String> {
    let root = env::temp_dir().join(format!("mantle-cli-arch-selftest-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).map_err(|error| format!("cleaning {}: {error}", root.display()))?;
    }
    let contract = root.join(CONTRACT_SOURCES);
    let presentation = root.join(PRESENTATION_SOURCES);
    let root_source = root.join(ROOT_SOURCES);
    fs::create_dir_all(&contract).map_err(|error| format!("creating fixture: {error}"))?;
    fs::create_dir_all(&presentation).map_err(|error| format!("creating fixture: {error}"))?;
    fs::create_dir_all(root_source.parent().expect("root parent")).map_err(|error| format!("creating fixture root: {error}"))?;

    fs::write(contract.join("clean.rs"), "pub fn plan() -> u32 { 1 }\n").map_err(|error| format!("fixture: {error}"))?;
    fs::write(
        presentation.join("clean.rs"),
        "pub fn render(value: u32) -> String { value.to_string() }\n",
    )
    .map_err(|error| format!("fixture: {error}"))?;
    fs::write(root_source.as_path(), "fn main() {}\n").map_err(|error| format!("fixture: {error}"))?;
    let baseline = scan_repository(&root)?;
    if !baseline.is_empty() {
        return Err(format!("accepted fixture must produce no violations, saw {baseline:?}"));
    }

    fs::write(contract.join("dirty.rs"), "use std::fs;\npub fn read() {}\n").map_err(|error| format!("fixture: {error}"))?;
    let contract_violations = scan_repository(&root)?;
    if !contract_violations.iter().any(|entry| entry.contains("contract boundary") && entry.contains("std::fs")) {
        return Err(format!("contract fixture must report std::fs, saw {contract_violations:?}"));
    }
    fs::write(contract.join("dirty.rs"), "pub fn read() {}\n").map_err(|error| format!("fixture: {error}"))?;

    fs::write(presentation.join("dirty.rs"), "pub fn render() -> String { std::env::var(\"X\").unwrap_or_default() }\n")
        .map_err(|error| format!("fixture: {error}"))?;
    let presentation_violations = scan_repository(&root)?;
    if !presentation_violations
        .iter()
        .any(|entry| entry.contains("presentation boundary") && entry.contains("std::env"))
    {
        return Err(format!("presentation fixture must report std::env, saw {presentation_violations:?}"));
    }
    fs::write(presentation.join("dirty.rs"), "pub fn render() -> String { String::new() }\n")
        .map_err(|error| format!("fixture: {error}"))?;

    fs::write(root_source.as_path(), "fn main() { std::process::Command::new(\"x\").status(); }\n")
        .map_err(|error| format!("fixture: {error}"))?;
    let root_violations = scan_repository(&root)?;
    if !root_violations
        .iter()
        .any(|entry| entry.contains("root boundary") && entry.contains("std::process::Command"))
    {
        return Err(format!("root fixture must report std::process::Command, saw {root_violations:?}"));
    }

    // Test-module text is excluded from every boundary.
    fs::write(
        root_source.as_path(),
        "fn main() {}\n#[cfg(test)]\nmod tests {\n    use std::process::Command;\n}\n",
    )
    .map_err(|error| format!("fixture: {error}"))?;
    let excluded = scan_repository(&root)?;
    if !excluded.is_empty() {
        return Err(format!("test-module text must be excluded, saw {excluded:?}"));
    }

    fs::remove_dir_all(&root).map_err(|error| format!("cleaning {}: {error}", root.display()))?;
    assert!(!root.exists());
    Ok(())
}
