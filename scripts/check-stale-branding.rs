//! Deterministic stale Crunch/crunch branding classifier for the Mantle rename.
//!
//! This intentionally uses only `git ls-files` plus substring allow rules so it
//! can run on a fresh checkout without external grep/regex dependencies.

use std::env;
use std::process::Command;
use std::process::ExitCode;

const BRAND_TOKENS: [&str; 3] = ["Crunch", "crunch", "CRUNCH"];
const LINE_NUMBER_OFFSET: usize = 1;

const SKIP_PATH_PREFIXES: [&str; 23] = [
    "cairn/archive/",
    "cairn/changes/",
    "cairn/specs/",
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
    "openspec/specs/project-identity/",
    "adr/",
];

const ALLOWED_LINE_SUBSTRINGS: &[&str] = &[
    "CRUNCH_",
    "CRUNCH-",
    "CRUNCH bridge TinyCC builtin va_list",
    "crunch-",
    "crunch_",
    "crunch::",
    "crunch/",
    "crunch.fetch",
    "crunch.ncl",
    "crunch.lock",
    "crunch.self-build",
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
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse(env::args().skip(1))?;
    if args.help {
        print_usage();
        return Ok(());
    }
    if args.self_test {
        return run_self_test();
    }
    let files = git_ls_files()?;
    let failures = classify_tracked_files(&files);
    if failures.is_empty() {
        println!("stale branding check passed");
        Ok(())
    } else {
        eprintln!("stale Crunch/crunch branding requires classification:");
        for failure in &failures {
            eprintln!("  {failure}");
        }
        Err(format!("{} unclassified branding occurrence(s)", failures.len()))
    }
}

#[derive(Debug)]
struct Args {
    help: bool,
    self_test: bool,
}

impl Args {
    fn parse<I>(args: I) -> Result<Self, String>
    where I: Iterator<Item = String> {
        let mut parsed = Args {
            help: false,
            self_test: false,
        };
        for arg in args {
            match arg.as_str() {
                "--self-test" => parsed.self_test = true,
                "--help" | "-h" => parsed.help = true,
                other => return Err(format!("unknown argument: {other}")),
            }
        }
        Ok(parsed)
    }
}

fn print_usage() {
    println!("Usage: check-stale-branding [--self-test]");
}

fn git_ls_files() -> Result<String, String> {
    let output = Command::new("git").args(["ls-files"]).output().map_err(|err| format!("git ls-files: {err}"))?;
    if !output.status.success() {
        return Err("git ls-files failed".to_string());
    }
    String::from_utf8(output.stdout).map_err(|err| format!("git paths are not UTF-8: {err}"))
}

fn classify_tracked_files(files: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for path in files.lines() {
        if is_skipped_path(path) {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        failures.extend(classify_content(path, &content));
    }
    failures
}

fn classify_content(path: &str, content: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        if !BRAND_TOKENS.iter().any(|token| line.contains(token)) {
            continue;
        }
        if is_allowed_occurrence(path, line) {
            continue;
        }
        failures.push(format!("{}:{}: {}", path, idx + LINE_NUMBER_OFFSET, line.trim()));
    }
    failures
}

fn is_skipped_path(path: &str) -> bool {
    SKIP_PATHS.contains(&path) || SKIP_PATH_PREFIXES.iter().any(|prefix| path.starts_with(prefix))
}

fn is_allowed_occurrence(path: &str, line: &str) -> bool {
    ALLOWED_PATH_SUBSTRINGS.iter().any(|part| path.contains(part))
        || ALLOWED_LINE_SUBSTRINGS.iter().any(|part| line.contains(part))
}

fn run_self_test() -> Result<(), String> {
    assert_allowed("docs/compat.md", "The legacy crunch.ncl compatibility file stays accepted.")?;
    assert_allowed("docs/report.md", "The exact row id crunch.self-build remains part of old parity receipts.")?;
    assert_allowed("scripts/check.rs", "CRUNCH bridge TinyCC builtin va_list is an embedded bootstrap marker.")?;
    assert_rejected("docs/new-user.md", "Crunch is the build tool for new projects.")?;
    assert_rejected("README.md", "Use Crunch for operator proof workflows.")?;
    println!("stale branding checker self-test passed");
    Ok(())
}

fn assert_allowed(path: &str, line: &str) -> Result<(), String> {
    let failures = classify_content(path, line);
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("self-test expected allowed occurrence: {failures:?}"))
    }
}

fn assert_rejected(path: &str, line: &str) -> Result<(), String> {
    let failures = classify_content(path, line);
    if failures.is_empty() {
        Err(format!("self-test expected rejection for {path}: {line}"))
    } else {
        Ok(())
    }
}
