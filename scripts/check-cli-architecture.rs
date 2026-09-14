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

/// Every core and application crate's sources.
const CORE_SOURCES: &str = "crates";

/// Contract crate root that must declare no-std.
const CONTRACT_LIB: &str = "crates/mantle-application-contract/src/lib.rs";

/// Contract manifest that must stay free of CLI and async-runtime dependencies.
const CONTRACT_MANIFEST: &str = "crates/mantle-application-contract/Cargo.toml";

/// Tokens that mean the CLI error type leaked into a core or application crate.
const CORE_FORBIDDEN: &[&str] = &["RunError"];

/// Dependency names the contract crate may not declare.
const CONTRACT_FORBIDDEN_DEPENDENCIES: &[&str] = &["tokio", "clap", "mantle ="];

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
    violations.extend(scan_core_sources(&root.join(CORE_SOURCES))?);
    violations.extend(scan_no_std_declaration(&root.join(CONTRACT_LIB))?);
    violations.extend(scan_contract_manifest(&root.join(CONTRACT_MANIFEST))?);
    violations.extend(scan_command_root_classification(
        &root.join(ROOT_SOURCES),
        &root.join(CONTRACT_SOURCES).join("family.rs"),
    )?);
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

/// Scan every core and application source for the CLI error type.
fn scan_core_sources(directory: &Path) -> Result<Vec<String>, String> {
    if !directory.is_dir() {
        return Err(format!("scanned directory is missing: {}", directory.display()));
    }
    let mut violations = Vec::new();
    for entry in fs::read_dir(directory).map_err(|error| format!("reading {}: {error}", directory.display()))? {
        let path = entry.map_err(|error| format!("reading directory entry: {error}"))?.path();
        if path.is_dir() {
            violations.extend(scan_core_sources(&path)?);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        violations.extend(scan_file_skipping_comments(&path, CORE_FORBIDDEN, "core error ownership")?);
    }
    debug_assert!(violations.len() <= 10_000);
    Ok(violations)
}

/// Scan one file, ignoring comment lines that merely document a boundary.
fn scan_file_skipping_comments(path: &Path, forbidden: &[&str], boundary: &str) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("reading {}: {error}", path.display()))?;
    let production = production_prefix(&text);
    let mut violations = Vec::new();
    for (index, line) in production.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
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

/// The contract crate root must declare no-std.
fn scan_no_std_declaration(path: &Path) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("reading {}: {error}", path.display()))?;
    let mut violations = Vec::new();
    if !text.contains("#![no_std]") {
        violations.push(format!("contract no-std boundary: {}: must declare `#![no_std]`", path.display()));
    }
    debug_assert!(!text.is_empty());
    debug_assert!(violations.is_empty() || !text.contains("#![no_std]"));
    Ok(violations)
}

/// The contract manifest must not pull the CLI or an async runtime.
fn scan_contract_manifest(path: &Path) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("reading {}: {error}", path.display()))?;
    let mut violations = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        for dependency in CONTRACT_FORBIDDEN_DEPENDENCIES {
            if trimmed.starts_with(dependency) {
                violations.push(format!(
                    "contract dependency boundary: {}: forbidden dependency `{dependency}`",
                    path.display()
                ));
            }
        }
    }
    debug_assert!(violations.is_empty() || text.contains("[dependencies]"));
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
/// Scan the CLI root labels against the contract's command-family taxonomy.
///
/// Every public root the CLI names must be classified by the contract's
/// `of_root` table, so the taxonomy stays a description of the real CLI.
fn scan_command_root_classification(roots_source: &Path, family_source: &Path) -> Result<Vec<String>, String> {
    if !roots_source.is_file() || !family_source.is_file() {
        // The rule applies when both the CLI root source and the taxonomy exist.
        debug_assert!(!roots_source.as_os_str().is_empty());
        return Ok(Vec::new());
    }
    let roots_text = fs::read_to_string(roots_source)
        .map_err(|error| format!("reading {}: {error}", roots_source.display()))?;
    let family_text = fs::read_to_string(family_source)
        .map_err(|error| format!("reading {}: {error}", family_source.display()))?;
    let cli_roots = command_root_labels(&roots_text);
    let contract_roots = classified_roots(&family_text);
    let mut violations = Vec::new();
    for root in &cli_roots {
        if !contract_roots.contains(root) {
            violations.push(format!(
                "{}: command root {root} is not classified by the application contract taxonomy",
                roots_source.display()
            ));
        }
    }
    debug_assert!(violations.len() <= cli_roots.len());
    debug_assert!(cli_roots.is_empty() || violations.len() <= cli_roots.len());
    Ok(violations)
}

/// Collect every command root label the CLI's label functions name.
fn command_root_labels(text: &str) -> Vec<String> {
    let mut roots: Vec<String> = Vec::new();
    for line in text.lines() {
        let is_label_line = line.trim_start().starts_with("Command::") && line.contains("=>");
        if !is_label_line {
            continue;
        }
        let Some(quoted) = line.split('"').nth(1) else {
            continue;
        };
        // The CLI hides internal roots behind a `__` prefix; the taxonomy names
        // the conceptual root without it.
        let visible = quoted.split('.').next().unwrap_or(quoted);
        let root = visible.strip_prefix("__").unwrap_or(visible);
        if !root.is_empty() {
            roots.push(root.to_string());
        }
    }
    roots.sort();
    roots.dedup();
    debug_assert!(roots.iter().all(|root| !root.is_empty()));
    roots
}

/// Collect every root string inside the contract's `of_root` table.
fn classified_roots(text: &str) -> Vec<String> {
    let Some(start) = text.find("pub fn of_root(") else {
        return Vec::new();
    };
    let body = &text[start..];
    let end = body.find("\n    }").map_or(body.len(), |offset| offset);
    let table = &body[..end];
    let mut roots: Vec<String> = table
        .split('"')
        .skip(1)
        .step_by(2)
        .map(|entry| entry.to_string())
        .collect();
    roots.sort();
    roots.dedup();
    debug_assert!(roots.iter().all(|root| !root.contains('"')));
    roots
}

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
    fs::create_dir_all(root_source.parent().expect("root parent"))
        .map_err(|error| format!("creating fixture root: {error}"))?;

    fs::write(contract.join("clean.rs"), "pub fn plan() -> u32 { 1 }\n")
        .map_err(|error| format!("fixture: {error}"))?;
    fs::write(presentation.join("clean.rs"), "pub fn render(value: u32) -> String { value.to_string() }\n")
        .map_err(|error| format!("fixture: {error}"))?;
    fs::write(root_source.as_path(), "fn main() {}\n").map_err(|error| format!("fixture: {error}"))?;
    let core = root.join("crates/mantle-rust-plan-core/src");
    fs::create_dir_all(&core).map_err(|error| format!("creating fixture: {error}"))?;
    fs::write(core.join("lib.rs"), "pub fn plan() -> u32 { 1 }\n").map_err(|error| format!("fixture: {error}"))?;
    fs::write(contract.join("lib.rs"), "#![no_std]\npub fn plan() {}\n")
        .map_err(|error| format!("fixture: {error}"))?;
    fs::write(root.join(CONTRACT_MANIFEST), "[dependencies]\nserde = \"1.0\"\n")
        .map_err(|error| format!("fixture: {error}"))?;
    let baseline = scan_repository(&root)?;
    if !baseline.is_empty() {
        return Err(format!("accepted fixture must produce no violations, saw {baseline:?}"));
    }

    fs::write(contract.join("dirty.rs"), "use std::fs;\npub fn read() {}\n")
        .map_err(|error| format!("fixture: {error}"))?;
    let contract_violations = scan_repository(&root)?;
    if !contract_violations
        .iter()
        .any(|entry| entry.contains("contract boundary") && entry.contains("std::fs"))
    {
        return Err(format!("contract fixture must report std::fs, saw {contract_violations:?}"));
    }
    fs::write(contract.join("dirty.rs"), "pub fn read() {}\n").map_err(|error| format!("fixture: {error}"))?;

    fs::write(
        presentation.join("dirty.rs"),
        "pub fn render() -> String { std::env::var(\"X\").unwrap_or_default() }\n",
    )
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

    // Core error ownership: a core that names the CLI error type is rejected.
    let core = root.join("crates/mantle-rust-plan-core/src");
    fs::create_dir_all(&core).map_err(|error| format!("creating fixture: {error}"))?;
    fs::write(core.join("lib.rs"), "pub fn plan() -> u32 { 1 }\n").map_err(|error| format!("fixture: {error}"))?;
    fs::write(contract.join("lib.rs"), "#![no_std]\npub fn plan() {}\n")
        .map_err(|error| format!("fixture: {error}"))?;
    fs::write(root.join(CONTRACT_MANIFEST), "[dependencies]\nserde = \"1.0\"\n")
        .map_err(|error| format!("fixture: {error}"))?;
    let clean_boundaries = scan_repository(&root)?;
    if !clean_boundaries.is_empty() {
        return Err(format!("accepted fixtures must produce no violations, saw {clean_boundaries:?}"));
    }

    fs::write(core.join("lib.rs"), "pub enum Error { Cli(RunError) }\n")
        .map_err(|error| format!("fixture: {error}"))?;
    let core_violations = scan_repository(&root)?;
    if !core_violations
        .iter()
        .any(|entry| entry.contains("core error ownership") && entry.contains("RunError"))
    {
        return Err(format!("core fixture must report RunError, saw {core_violations:?}"));
    }
    fs::write(core.join("lib.rs"), "/// No RunError is reachable here.\npub fn plan() {}\n")
        .map_err(|error| format!("fixture: {error}"))?;
    let commented = scan_repository(&root)?;
    if !commented.is_empty() {
        return Err(format!("comment lines must be ignored, saw {commented:?}"));
    }

    fs::write(contract.join("lib.rs"), "pub fn plan() {}\n").map_err(|error| format!("fixture: {error}"))?;
    let no_std_violations = scan_repository(&root)?;
    if !no_std_violations.iter().any(|entry| entry.contains("contract no-std boundary")) {
        return Err(format!("contract fixture must report the missing no-std declaration, saw {no_std_violations:?}"));
    }
    fs::write(contract.join("lib.rs"), "#![no_std]\npub fn plan() {}\n")
        .map_err(|error| format!("fixture: {error}"))?;

    fs::write(root.join(CONTRACT_MANIFEST), "[dependencies]\ntokio = \"1\"\n")
        .map_err(|error| format!("fixture: {error}"))?;
    let dependency_violations = scan_repository(&root)?;
    if !dependency_violations.iter().any(|entry| entry.contains("contract dependency boundary")) {
        return Err(format!("contract manifest fixture must report tokio, saw {dependency_violations:?}"));
    }

    // Command-root classification: an unclassified CLI root is rejected and a
    // classified one is accepted.
    fs::write(root.join(CONTRACT_MANIFEST), "[dependencies]\nserde = \"1.0\"\n")
        .map_err(|error| format!("fixture: {error}"))?;
    fs::write(
        contract.join("family.rs"),
        "pub fn of_root(root: &str) -> Option<u8> {\n    match root {\n        \"build\" => Some(1),\n        \"doctor\" => Some(2),\n        _ => None,\n    }\n}\n",
    )
    .map_err(|error| format!("fixture: {error}"))?;
    fs::write(
        root_source.as_path(),
        "fn command_label(command: &Command) -> &'static str {\n    match command {\n        Command::Build { .. } => \"unknown-root\",\n    }\n}\n",
    )
    .map_err(|error| format!("fixture: {error}"))?;
    let root_violations = scan_repository(&root)?;
    if !root_violations
        .iter()
        .any(|entry| entry.contains("unknown-root") && entry.contains("not classified"))
    {
        return Err(format!("unclassified root fixture must be reported, saw {root_violations:?}"));
    }
    fs::write(
        root_source.as_path(),
        "fn command_label(command: &Command) -> &'static str {\n    match command {\n        Command::Build { .. } => \"build\",\n        Command::Doctor { .. } => \"__doctor\",\n    }\n}\n",
    )
    .map_err(|error| format!("fixture: {error}"))?;
    let classified = scan_repository(&root)?;
    if !classified.is_empty() {
        return Err(format!("classified root fixture must pass, saw {classified:?}"));
    }

    fs::remove_dir_all(&root).map_err(|error| format!("cleaning {}: {error}", root.display()))?;
    assert!(!root.exists());
    Ok(())
}
