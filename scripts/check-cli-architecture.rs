#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"
---

//! Architecture guard for the thin CLI composition root.
//!
//! The contract crate owns typed commands, blockers, and ports; presentation
//! modules render typed values; adapters own host capability. This guard checks
//! lexical boundaries from source text; comments, literals and cfg(test) items
//! do not introduce production host authority. `--semantic` delegates to the
//! existing no-std rail for macro-expanded HIR, typed HIR and resolved normal
//! runtime dependency checks. `--semantic --self-test` executes real compiler
//! and Cargo fixtures. Neither rail claims arbitrary semantic/domain correctness.

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
const CONTRACT_FORBIDDEN_DEPENDENCIES: &[&str] = &[
    "tokio",
    "clap",
    "mantle =",
    "crunch-store =",
    "crunch-build =",
    "crunch-eval =",
    "crunch-pipeline =",
    "crunch-glue =",
    "snix-build =",
    "snix-castore =",
    "snix-store =",
];

/// Shell crate paths a core source must not name: cores are leaves, and every
/// dependency points inward from adapters and the root toward cores and the
/// application contract.
const CORE_DIRECTION_FORBIDDEN: &[&str] = &[
    "crunch_store::",
    "crunch_build::",
    "crunch_eval::",
    "crunch_pipeline::",
    "crunch_glue::",
    "crunch_nar::",
    "mantle_application_contract::",
    "snix_",
];

/// Host implementations forbidden in both cores and the application contract.
///
/// These are authority paths/types, not words such as `Provider` or `render`:
/// typed provider observations, store DTOs, alloc, serde, and pure formatting
/// remain legitimate core inputs and computations.
const CONTRACT_FORBIDDEN: &[&str] = &[
    // CLI ownership and presentation adapters.
    "clap", "mantle", "crate::cli", "crate::errors", "crate::presentation",
    "RunContext", "RunError",
    // Snix and store-service implementations.
    "snix_build", "snix_castore", "snix_store", "snix_eval", "snix_glue",
    "crunch_store", "StoreHandle", "BlobService", "DirectoryService", "PathInfoService",
    // Filesystem, process, runtime, and ambient environment authority.
    "std::fs", "std::os::unix::fs", "std::os::windows::fs", "rustix::fs",
    "std::process", "std::thread", "tokio", "async_std", "smol",
    "std::env", "dotenv", "dotenvy",
    // Clock values such as Duration remain allowed; ambient clock reads do not.
    "std::time::Instant", "std::time::SystemTime", "web_time::Instant",
    "web_time::SystemTime", "chrono::Utc", "chrono::Local",
    "rand::random", "rand::random_iter", "rand::random_range", "rand::random_bool",
    "rand::random_ratio", "rand::fill", "rand::rng", "rand::thread_rng", "rand::rngs::thread",
    "rand::rngs::ThreadRng", "rand::rngs::OsRng", "getrandom",
    "rand_core::OsRng", "rand_core::os::OsRng", "uuid::Uuid::new_v4",
    "fastrand::Rng::new", "fastrand::global_rng",
    "std::net", "reqwest", "ureq", "hyper", "rustix::net",
    // Concrete provider adapters, not provider evidence or policy records.
    "crate::full_source_provider", "crate::rust_source_provider",
    "crate::source_root_provider", "crate::stagex_provider",
    "crate::provider_output_publication",
    // Operator output belongs to presentation, not pure decision code.
    "std::io::stdout", "std::io::stderr", "std::io::stdin",
    "std::io::stdio::_print", "std::io::stdio::_eprint",
    "print!", "println!", "eprint!", "eprintln!",
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
    if env::args().any(|argument| matches!(argument.as_str(), "--semantic" | "--semantic-self-test")) {
        let mode = if env::args().any(|argument| matches!(argument.as_str(), "--self-test" | "--semantic-self-test")) {
            "architecture-self-test"
        } else {
            "architecture"
        };
        let mut command = std::process::Command::new("python3");
        command.args(["scripts/no_std_core_checks.py", mode]);
        for authority in CONTRACT_FORBIDDEN {
            command.args(["--authority", authority]);
        }
        for authority in CORE_DIRECTION_FORBIDDEN {
            command.args(["--core-authority", authority]);
        }
        return match command.status() {
            Ok(status) if status.success() => ExitCode::SUCCESS,
            Ok(_) => ExitCode::FAILURE,
            Err(error) => {
                eprintln!("cli architecture compiler rail: FAIL: {error}");
                ExitCode::FAILURE
            }
        };
    }
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
    violations.extend(scan_host_sources(&root.join(CONTRACT_SOURCES), "contract host authority")?);
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

/// Scan every core and application source for the CLI error type and for
/// untyped error returns.
///
/// A core is a crate whose directory name ends in `-core`; its functions must
/// return a typed error, because the shell maps that error at the boundary.
fn scan_core_sources(directory: &Path) -> Result<Vec<String>, String> {
    scan_core_sources_in(directory, false)
}

/// Scan one directory tree, tracking whether it belongs to a core crate.
fn scan_core_sources_in(directory: &Path, is_core: bool) -> Result<Vec<String>, String> {
    if !directory.is_dir() {
        return Err(format!("scanned directory is missing: {}", directory.display()));
    }
    let mut violations = Vec::new();
    for entry in fs::read_dir(directory).map_err(|error| format!("reading {}: {error}", directory.display()))? {
        let path = entry.map_err(|error| format!("reading directory entry: {error}"))?.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
            if name.ends_with("-core") {
                violations.extend(scan_host_sources(&path.join("src"), "core host authority")?);
            }
            violations.extend(scan_core_sources_in(&path, is_core || name.ends_with("-core"))?);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let is_test_target = path
            .components()
            .any(|component| component.as_os_str() == "tests");
        if is_test_target {
            continue;
        }
        violations.extend(scan_file_skipping_comments(&path, CORE_FORBIDDEN, "core error ownership")?);
        if is_core {
            violations.extend(scan_untyped_error_returns(&path)?);
            violations.extend(scan_file_skipping_comments(
                &path,
                CORE_DIRECTION_FORBIDDEN,
                "core dependency direction",
            )?);
        }
    }
    debug_assert!(violations.len() <= 10_000);
    Ok(violations)
}

/// Scan one core file for a function that returns an untyped error.
fn scan_untyped_error_returns(path: &Path) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("reading {}: {error}", path.display()))?;
    let production = production_prefix(&text);
    let mut violations = Vec::new();
    let mut offset: usize = 0;
    for line in production.lines() {
        let is_untyped = line.contains("-> Result<") && line.trim_end().ends_with(", String> {");
        if is_untyped {
            violations.push(format!(
                "{}:{}: core error ownership: a core function returns an untyped String error; return a typed error instead",
                path.display(),
                offset + 1
            ));
        }
        offset += 1;
    }
    debug_assert!(violations.len() <= offset.saturating_add(1));
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

/// One lexical source token. Literals are retained only to resolve test files.
#[derive(Clone, Copy)]
struct SourceToken<'a> {
    text: &'a str,
    line: usize,
    literal: bool,
}

/// Tokenize without comments; quoted strings/chars cannot introduce authority.
fn source_tokens(text: &str) -> Vec<SourceToken<'_>> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    let mut line = 1;
    while index < bytes.len() {
        let start = index;
        let mut literal = false;
        let mut comment = false;
        if text[index..].starts_with("//") {
            index += text[index..].find('\n').unwrap_or(bytes.len() - index);
            comment = true;
        } else if text[index..].starts_with("/*") {
            index += 2;
            let mut depth = 1;
            while index < bytes.len() && depth > 0 {
                if text[index..].starts_with("/*") {
                    depth += 1;
                    index += 2;
                } else if text[index..].starts_with("*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    index += text[index..].chars().next().expect("source character").len_utf8();
                }
            }
            comment = true;
        } else {
            // Raw strings may contain quotes, braces, and fake cfg markers.
            let raw_start = if bytes[index] == b'b' { index + 1 } else { index };
            let mut quote = raw_start + 1;
            if bytes.get(raw_start) == Some(&b'r') {
                while bytes.get(quote) == Some(&b'#') {
                    quote += 1;
                }
            }
            if bytes.get(raw_start) == Some(&b'r') && bytes.get(quote) == Some(&b'"') {
                let closing = format!("\"{}", &text[raw_start + 1..quote]);
                index = text[quote + 1..].find(&closing)
                    .map_or(bytes.len(), |offset| quote + 1 + offset + closing.len());
                literal = true;
            } else if bytes[index] == b'"'
                || (bytes[index] == b'\''
                    && (bytes.get(index + 2) == Some(&b'\'') || bytes.get(index + 1) == Some(&b'\\')))
            {
                let closing = bytes[index];
                index += 1;
                while index < bytes.len() {
                    if bytes[index] == b'\\' {
                        index = (index + 2).min(bytes.len());
                    } else if bytes[index] == closing {
                        index += 1;
                        break;
                    } else {
                        index += text[index..].chars().next().expect("literal character").len_utf8();
                    }
                }
                literal = true;
            } else if bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_' {
                index += 1;
                while index < bytes.len() && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_') {
                    index += 1;
                }
            } else if text[index..].starts_with("::") {
                index += 2;
            } else {
                index += text[index..].chars().next().expect("source character").len_utf8();
            }
        }
        let fragment = &text[start..index];
        if !comment && !fragment.trim().is_empty() {
            tokens.push(SourceToken { text: fragment, line, literal });
        }
        line += fragment.bytes().filter(|byte| *byte == b'\n').count();
    }
    tokens
}

/// Remove cfg(test) items, not the production items that may follow them.
/// Return omitted spans as well, so external test modules can be excluded.
fn host_production_tokens<'a>(tokens: &[SourceToken<'a>]) -> (Vec<SourceToken<'a>>, Vec<Vec<SourceToken<'a>>>) {
    let marker = ["#", "[", "cfg", "(", "test", ")", "]"];
    let mut production: Vec<SourceToken<'a>> = Vec::new();
    let mut tests = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index..].iter().take(marker.len()).map(|token| token.text).eq(marker) {
            let start = index;
            // Attributes may precede cfg(test), including #[path = "..."]:
            // move that item's attributes into the excluded span as well.
            let mut attributes = Vec::new();
            while production.last().is_some_and(|token| token.text == "]") {
                let mut start = production.len() - 1;
                let mut depth = 1_usize;
                while start > 0 && depth > 0 {
                    start -= 1;
                    match production[start].text { "]" => depth += 1, "[" => depth -= 1, _ => {} }
                }
                if start == 0 || production[start - 1].text != "#" { break; }
                drop(attributes.splice(0..0, production.drain(start - 1..)));
            }
            index += marker.len();
            let mut braces = 0_usize;
            while index < tokens.len() {
                let token = tokens[index].text;
                index += 1;
                if token == "{" {
                    braces += 1;
                } else if token == "}" {
                    braces = braces.saturating_sub(1);
                    if braces == 0 { break; }
                } else if token == ";" && braces == 0 {
                    break;
                }
            }
            attributes.extend_from_slice(&tokens[start..index]);
            tests.push(attributes);
        } else {
            production.push(tokens[index]);
            index += 1;
        }
    }
    (production, tests)
}

/// Expand nested use trees and retain their local bindings for alias resolution.
fn import_paths(
    tokens: &[SourceToken<'_>],
    prefix: &str,
    paths: &mut Vec<(String, usize)>,
    aliases: &mut Vec<(String, String)>,
) {
    let mut path = prefix.to_string();
    let mut index = 0;
    while index < tokens.len() {
        let token = tokens[index];
        match token.text {
            "{" => {
                let start = index + 1;
                let mut depth = 1;
                index += 1;
                while index < tokens.len() && depth > 0 {
                    match tokens[index].text { "{" => depth += 1, "}" => depth -= 1, _ => {} }
                    index += 1;
                }
                import_paths(&tokens[start..index.saturating_sub(1)], &path, paths, aliases);
                path.clear();
                index = index.saturating_sub(1);
            }
            "," | ";" => {
                record_import_binding(&path, None, aliases);
                path = prefix.to_string();
            }
            "as" => {
                if let Some(alias) = tokens.get(index + 1) {
                    record_import_binding(&path, Some(alias.text), aliases);
                }
                path.clear();
                index += 1;
            }
            "self" if !path.is_empty() => {
                path.truncate(path.trim_end_matches("::").len());
                paths.push((path.clone(), token.line));
            }
            _ if !token.literal => {
                path.push_str(token.text);
                paths.push((path.clone(), token.line));
            }
            _ => {}
        }
        index += 1;
    }
    record_import_binding(&path, None, aliases);
}

fn record_import_binding(path: &str, alias: Option<&str>, aliases: &mut Vec<(String, String)>) {
    let path = path.trim_start_matches("::").trim_end_matches("::");
    let name = alias.unwrap_or_else(|| path.rsplit("::").next().unwrap_or(path));
    if !name.is_empty() && name != "_" && name != "*" && name != path {
        aliases.push((name.to_string(), path.to_string()));
    }
}

/// Expand lexical aliases to a fixed point without following cyclic bindings.
/// Scope/shadowing and cross-module re-exports remain semantic review obligations.
fn expand_aliases(paths: &mut Vec<(String, usize)>, aliases: &[(String, String)]) {
    let mut seen: std::collections::BTreeSet<(String, usize)> = paths.iter().cloned().collect();
    for _ in 0..aliases.len() {
        let start = seen.len();
        let current: Vec<_> = seen.iter().cloned().collect();
        for (path, line) in current {
            for (name, target) in aliases {
                let path = path.trim_start_matches("::");
                if path == name || path.strip_prefix(name).is_some_and(|suffix| suffix.starts_with("::")) {
                    seen.insert((format!("{target}{}", &path[name.len()..]), line));
                }
            }
        }
        if seen.len() == start { break; }
    }
    *paths = seen.into_iter().collect();
}

/// Match a whole authority path, not lookalikes such as `ProviderOutputRole`.
fn authority_matches(path: &str, forbidden: &str) -> bool {
    let path = path.trim_start_matches("::");
    let path = if forbidden.ends_with('!') { path } else { path.trim_end_matches('!') };
    path == forbidden || path.strip_prefix(forbidden).is_some_and(|suffix| suffix.starts_with("::"))
}

/// Resolve literal include paths and Rust module paths within one source file.
/// This is source-shape ownership, not cfg evaluation or macro expansion.
fn source_references(tokens: &[SourceToken<'_>], parent: &Path, module_directory: &Path, path_base: &Path, paths: &mut Vec<std::path::PathBuf>) {
    let mut index = 0;
    let mut explicit_path = None;
    while index < tokens.len() {
        if index >= 2 && tokens[index - 2].text == "#" && tokens[index - 1].text == "["
            && tokens[index].text == "path" && tokens.get(index + 1).is_some_and(|token| token.text == "=")
        {
            explicit_path = tokens.get(index + 2).filter(|token| token.literal).map(|token| token.text.trim_matches('"'));
        }
        if tokens[index].text == "include"
            && tokens.get(index + 1).is_some_and(|token| token.text == "!")
            && tokens.get(index + 2).is_some_and(|token| token.text == "(")
        {
            if let Some(literal) = tokens.get(index + 3).filter(|token| token.literal) {
                paths.push(parent.join(literal.text.trim_matches('"')));
            }
        }
        if tokens[index].text == "mod" && index + 2 < tokens.len() {
            let name = tokens[index + 1].text;
            if tokens[index + 2].text == ";" {
                if let Some(relative) = explicit_path.take() {
                    paths.push(path_base.join(relative));
                } else {
                    paths.push(module_directory.join(format!("{name}.rs")));
                    paths.push(module_directory.join(name));
                }
            } else if tokens[index + 2].text == "{" {
                let start = index + 3;
                index = start;
                let mut depth = 1;
                while index < tokens.len() && depth > 0 {
                    match tokens[index].text { "{" => depth += 1, "}" => depth -= 1, _ => {} }
                    index += 1;
                }
                let nested = explicit_path.take().map_or_else(|| module_directory.join(name), |relative| path_base.join(relative));
                source_references(&tokens[start..index.saturating_sub(1)], parent, &nested, &nested, paths);
                continue;
            }
        }
        if tokens[index].text == ";" { explicit_path = None; }
        index += 1;
    }
}

/// Scan the same host-authority rule over production core/contract source only.
fn scan_host_sources(directory: &Path, boundary: &str) -> Result<Vec<String>, String> {
    fn collect(directory: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
        for entry in fs::read_dir(directory).map_err(|error| format!("reading {}: {error}", directory.display()))? {
            let path = entry.map_err(|error| format!("reading directory entry: {error}"))?.path();
            if path.is_dir() {
                collect(&path, files)?;
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                files.push(path);
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    collect(directory, &mut files)?;
    files.sort();
    let mut sources = Vec::new();
    let mut excluded = Vec::new();
    let mut production_references = Vec::new();
    for path in &files {
        let text = fs::read_to_string(path).map_err(|error| format!("reading {}: {error}", path.display()))?;
        let tokens = source_tokens(&text);
        let (production, tests) = host_production_tokens(&tokens);
        let parent = path.parent().expect("source parent");
        let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or_default();
        let module_directory = if matches!(stem, "lib" | "main" | "mod") { parent.to_path_buf() } else { parent.join(stem) };
        let mut references = Vec::new();
        source_references(&production, parent, &module_directory, parent, &mut references);
        production_references.push(references);
        for test in tests {
            source_references(&test, parent, &module_directory, parent, &mut excluded);
        }
        sources.push(text);
    }
    // A file shared with a production module/include is production, even if a
    // test also references it. Follow those source references to a fixed point.
    let mut included: Vec<std::path::PathBuf> = Vec::new();
    loop {
        let previous = included.len();
        for (path, references) in files.iter().zip(&production_references) {
            if !excluded.iter().any(|excluded| path.starts_with(excluded))
                || included.iter().any(|included| path.starts_with(included))
            {
                included.extend(references.iter().cloned());
            }
        }
        included.sort();
        included.dedup();
        if previous == included.len() { break; }
    }
    let mut violations = Vec::new();
    for (path, text) in files.iter().zip(&sources) {
        if excluded.iter().any(|excluded| path.starts_with(excluded))
            && !included.iter().any(|included| path.starts_with(included))
        { continue; }
        let tokens = source_tokens(text);
        let (production, _) = host_production_tokens(&tokens);
        let mut paths = Vec::new();
        let mut aliases = Vec::new();
        let mut index = 0;
        while index < production.len() {
            if production[index].text == "use" {
                let start = index + 1;
                while index < production.len() && production[index].text != ";" { index += 1; }
                import_paths(&production[start..index], "", &mut paths, &mut aliases);
            } else if production[index].text == "extern"
                && production.get(index + 1).is_some_and(|token| token.text == "crate")
            {
                if let Some(target) = production.get(index + 2) {
                    paths.push((target.text.to_string(), target.line));
                    if production.get(index + 3).is_some_and(|token| token.text == "as") {
                        if let Some(alias) = production.get(index + 4) {
                            record_import_binding(target.text, Some(alias.text), &mut aliases);
                        }
                    }
                }
            } else if !production[index].literal {
                let line = production[index].line;
                let mut path = production[index].text.to_string();
                let mut next = index + 1;
                while next + 1 < production.len() && production[next].text == "::" && !production[next + 1].literal {
                    path.push_str("::");
                    path.push_str(production[next + 1].text);
                    next += 2;
                }
                if production.get(next).is_some_and(|token| token.text == "!") { path.push('!'); }
                // Bare lower-case values are not crate authority. Imports and
                // extern crates are recorded above; qualified paths/macros and
                // concrete authority types remain lexical evidence. Resolved
                // scope/shadowing is the compiler rail's responsibility.
                if next > index + 1 || path.ends_with('!')
                    || path.chars().next().is_some_and(char::is_uppercase)
                {
                    paths.push((path, line));
                }
            }
            index += 1;
        }
        expand_aliases(&mut paths, &aliases);
        for (path_text, line) in paths {
            for forbidden in CONTRACT_FORBIDDEN {
                if authority_matches(&path_text, forbidden) {
                    violations.push(format!("{boundary} boundary: {}:{line}: forbidden authority `{forbidden}`", path.display()));
                }
            }
        }
    }
    violations.sort();
    violations.dedup();
    Ok(violations)
}

/// The contract crate root must declare no-std.
fn scan_no_std_declaration(path: &Path) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("reading {}: {error}", path.display()))?;
    let mut violations = Vec::new();
    let tokens = source_tokens(&text);
    let marker = ["#", "!", "[", "no_std", "]"];
    let mut depth = 0_usize;
    let mut declared = false;
    for (index, token) in tokens.iter().enumerate() {
        if depth == 0 && tokens[index..].iter().take(marker.len()).map(|token| token.text).eq(marker) {
            declared = true;
            break;
        }
        match token.text {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    if !declared {
        violations.push(format!("contract no-std boundary: {}: must declare `#![no_std]`", path.display()));
    }
    debug_assert!(!text.is_empty());
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
    // Each authority family is rejected in both protected production scopes.
    // These are validator inputs, not diagnostic wording or copied-source tests.
    let rejected_authority = [
        ("CLI parser", "use clap::Parser;\n"),
        ("CLI shell", "use mantle::command_input::CommandInput;\n"),
        ("bare crate import", "use mantle;\n"),
        ("grouped crate import", "use {mantle as shell};\n"),
        ("bare extern crate", "extern crate mantle;\n"),
        ("renamed shell crate", "extern crate mantle as shell;\npub fn sample() { shell::command_input::CommandInput::new(); }\n"),
        ("qualified shell crate", "pub fn sample() { mantle::command_input::sample(); }\n"),
        ("nested shell module", "mod nested { pub fn sample() { mantle::command_input::sample(); } }\n"),
        ("shell macro", "pub fn sample() { mantle!(); }\n"),
        ("Snix", "use snix_store::path_info::PathInfo;\n"),
        ("filesystem", "use std::{fs as host_fs};\n"),
        ("platform filesystem", "use std::os::unix::fs::PermissionsExt;\n"),
        ("process", "pub fn execute() { std :: process :: Command::new(\"cc\"); }\n"),
        ("thread", "pub fn execute() { std::thread::yield_now(); }\n"),
        ("async runtime", "#[tokio::main]\nasync fn execute() {}\n"),
        ("alternate runtime", "use async_std::task;\n"),
        ("environment", "use std::{\n    env::{var as ambient_var},\n};\n"),
        ("clock", "use std::time::{Duration, SystemTime as Clock};\n"),
        ("monotonic clock", "pub fn sample() { std::time::Instant::now(); }\n"),
        ("random", "pub fn sample() { rand::random::<u64>(); }\n"),
        ("OS random", "use rand_core::OsRng;\n"),
        ("system random", "pub fn sample() { getrandom::fill(&mut [0; 8]); }\n"),
        ("network", "use ::std::net::TcpStream;\n"),
        ("HTTP client", "use reqwest::Client;\n"),
        ("provider", "use crate::full_source_provider::admit_full_source_provider;\n"),
        ("store service", "pub fn execute(_: &StoreHandle) {}\n"),
        ("blob service", "pub fn execute(_: &dyn BlobService) {}\n"),
        ("presentation adapter", "use crate::presentation::semantic_graph;\n"),
        ("operator rendering", "pub fn render() { println!(\"plan\"); }\n"),
        ("operator stream", "use std::io::{self, stdout};\n"),
        ("nested import sibling", "use std::{collections::{BTreeMap}, fs};\n"),
        ("module alias", "use std as host;\npub fn sample() { host::env::vars(); }\n"),
        ("chained grouped alias", "use std::{self as host};\nuse host::{time as clocks};\npub fn sample() { clocks::SystemTime::now(); }\n"),
        ("extern crate alias", "extern crate std as host;\npub fn sample() { host::fs::read(\"input\"); }\n"),
        ("re-exported module alias", "pub use std as host;\npub fn sample() { host::process::Command::new(\"cc\"); }\n"),
        ("production after tests", "#[cfg(test)]\nmod tests { use std::fs; }\npub fn execute() { std::env::vars(); }\n"),
    ];
    for (directory, boundary) in [(&core, "core host authority"), (&contract, "contract host authority")] {
        let fixture = directory.join("authority.rs");
        for (family, source) in rejected_authority {
            fs::write(&fixture, source).map_err(|error| format!("fixture: {error}"))?;
            let rejected = scan_repository(&root)?;
            if !rejected.iter().any(|entry| entry.contains(boundary)) {
                return Err(format!("{family} authority must be rejected in {boundary}, saw {rejected:?}"));
            }
        }
        fs::remove_file(&fixture).map_err(|error| format!("cleaning fixture: {error}"))?;
    }

    // Pure allocations/serialization, explicit values, provider/store facts,
    // formatting, and test-only host helpers remain legal in both scopes.
    let accepted_authority = [
        "extern crate alloc;\nuse alloc::{string::String, vec::Vec};\nuse serde::{Serialize, Deserialize};\n#[derive(Serialize, Deserialize)]\npub struct Plan { pub paths: Vec<String> }\n",
        "use core::time::Duration;\nuse std::{collections::{BTreeMap}, time::Duration as Interval};\npub struct Observed { pub epoch_seconds: u64, pub random_seed: u64, pub environment: BTreeMap<String, String> }\n",
        "use std as host;\nuse host::{collections::BTreeMap, time::Duration};\npub struct Facts { pub values: BTreeMap<u64, Duration> }\n",
        "pub struct ProviderOutputRole;\npub struct StoreObservation;\npub struct StoreHandleId;\npub fn render(value: u64) -> String { format!(\"{value}\") }\n",
        "pub fn compare(mantle: &str, clap: &str, getrandom: u32) -> bool { let mantle = mantle.trim(); mantle == clap && getrandom == 7 }\n",
        "pub fn compare() { let mantle = 1; let observed = mantle; let _ = (mantle, observed); }\n",
        "// std::fs and clap::Parser document the boundary.\n/* outer /* std::process::Command */ tokio::spawn */\npub const HELP: &str = \"std::env and rand::random\";\npub const RAW: &str = r##\"fake \\\" #[cfg(test)] mod tests { } std::net::TcpStream\"##;\n",
        "#[cfg(test)]\nextern crate std;\n#[cfg(test)]\nmod host_helpers { use std::fs; fn output() { println!(\"test\"); } }\npub fn plan() {}\n",
    ];
    for directory in [&core, &contract] {
        let fixture = directory.join("authority.rs");
        for source in accepted_authority {
            fs::write(&fixture, source).map_err(|error| format!("fixture: {error}"))?;
            let accepted = scan_repository(&root)?;
            if !accepted.is_empty() {
                return Err(format!("pure/test-only authority fixture must pass, saw {accepted:?}"));
            }
        }
        let external_source = "#[path = \"host_fixture.rs\"]\n#[cfg(test)]\nmod host_helpers;\n#[cfg(test)]\nmod included { include!(\"included_fixture.rs\"); }\n#[cfg(test)]\nmod default_helpers;\n#[cfg(test)]\nmod inline { mod helpers; }\npub fn plan() {}\n";
        fs::write(&fixture, external_source).map_err(|error| format!("fixture: {error}"))?;
        for name in ["host_fixture.rs", "included_fixture.rs", "authority/default_helpers.rs", "authority/inline/helpers.rs"] {
            let path = directory.join(name);
            fs::create_dir_all(path.parent().expect("fixture parent")).map_err(|error| format!("fixture: {error}"))?;
            fs::write(path, "pub fn host_helper() { std::fs::read(\"fixture\"); std::println!(\"test\"); }\n")
                .map_err(|error| format!("fixture: {error}"))?;
        }
        let external_tests = scan_repository(&root)?;
        if !external_tests.is_empty() {
            return Err(format!("external test-only sources must pass, saw {external_tests:?}"));
        }
        fs::write(&fixture, format!("{external_source}include!(\"host_fixture.rs\");\n"))
            .map_err(|error| format!("fixture: {error}"))?;
        let shared_with_production = scan_repository(&root)?;
        if !shared_with_production.iter().any(|entry| entry.contains("host authority") && entry.contains("host_fixture.rs")) {
            return Err(format!("a test-shared production include must be rejected, saw {shared_with_production:?}"));
        }
        for name in ["authority.rs", "host_fixture.rs", "included_fixture.rs"] {
            fs::remove_file(directory.join(name)).map_err(|error| format!("cleaning fixture: {error}"))?;
        }
        fs::remove_dir_all(directory.join("authority")).map_err(|error| format!("cleaning fixture: {error}"))?;
    }

    // The same implementation remains permitted in an adapter and integration
    // test target: neither belongs to a core's production src authority scope.
    let adapter = root.join("crates/fixture-adapter/src");
    let integration_tests = root.join("crates/mantle-rust-plan-core/tests");
    for directory in [&adapter, &integration_tests] {
        fs::create_dir_all(directory).map_err(|error| format!("creating fixture: {error}"))?;
        fs::write(directory.join("host.rs"), "pub fn sample() { std::env::vars(); std::println!(\"adapter\"); }\n")
            .map_err(|error| format!("fixture: {error}"))?;
    }
    let shell_authority = scan_repository(&root)?;
    if !shell_authority.is_empty() {
        return Err(format!("adapter and integration-test host authority must pass, saw {shell_authority:?}"));
    }

    fs::write(contract.join("dirty.rs"), "use std::fs;\npub fn read() {}\n")
        .map_err(|error| format!("fixture: {error}"))?;
    let contract_violations = scan_repository(&root)?;
    if !contract_violations
        .iter()
        .any(|entry| entry.contains("contract host authority") && entry.contains("std::fs"))
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

    fs::write(core.join("lib.rs"), "pub use crunch_store::StoreHandle;\n")
        .map_err(|error| format!("fixture: {error}"))?;
    let direction_violations = scan_repository(&root)?;
    if !direction_violations
        .iter()
        .any(|entry| entry.contains("core dependency direction") && entry.contains("crunch_store::"))
    {
        return Err(format!(
            "core direction fixture must report crunch_store, saw {direction_violations:?}"
        ));
    }
    fs::write(core.join("lib.rs"), "pub fn plan() -> u32 { 1 }\n")
        .map_err(|error| format!("fixture: {error}"))?;

    fs::write(
        root.join(CONTRACT_MANIFEST),
        "[dependencies]\ncrunch-store = { path = \"../crunch-store\" }\n",
    )
    .map_err(|error| format!("fixture: {error}"))?;
    let manifest_violations = scan_repository(&root)?;
    if !manifest_violations
        .iter()
        .any(|entry| entry.contains("contract dependency boundary") && entry.contains("crunch-store"))
    {
        return Err(format!(
            "contract manifest fixture must report crunch-store, saw {manifest_violations:?}"
        ));
    }
    fs::write(root.join(CONTRACT_MANIFEST), "[dependencies]\nserde = \"1.0\"\n")
        .map_err(|error| format!("fixture: {error}"))?;

    for source in [
        "pub fn plan() {}\n",
        "// #![no_std]\npub fn plan() {}\n",
        "pub const DOCUMENTATION: &str = \"#![no_std]\";\n",
        "#[cfg(test)]\nmod tests { #![no_std]\npub fn plan() {} }\n",
        "macro_rules! documented { () => (#![no_std]); }\n",
    ] {
        fs::write(contract.join("lib.rs"), source).map_err(|error| format!("fixture: {error}"))?;
        let no_std_violations = scan_repository(&root)?;
        if !no_std_violations.iter().any(|entry| entry.contains("contract no-std boundary")) {
            return Err(format!("contract fixture must report the missing no-std declaration, saw {no_std_violations:?}"));
        }
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

    // Untyped core error returns: a core function returning a bare String error
    // is rejected, and a typed one is accepted.
    fs::write(core.join("lib.rs"), "pub fn plan()\n    -> Result<(), String> {\n    Ok(())\n}\n")
        .map_err(|error| format!("fixture: {error}"))?;
    let untyped_core = scan_repository(&root)?;
    if !untyped_core.iter().any(|entry| entry.contains("returns an untyped String error")) {
        return Err(format!("untyped core return fixture must be reported, saw {untyped_core:?}"));
    }
    fs::write(
        core.join("lib.rs"),
        "pub enum CoreError { Rejected }\npub fn plan() -> Result<(), CoreError> { Ok(()) }\n",
    )
    .map_err(|error| format!("fixture: {error}"))?;
    let typed_core = scan_repository(&root)?;
    if !typed_core.is_empty() {
        return Err(format!("typed core return fixture must pass, saw {typed_core:?}"));
    }

    fs::remove_dir_all(&root).map_err(|error| format!("cleaning {}: {error}", root.display()))?;
    assert!(!root.exists());
    Ok(())
}
