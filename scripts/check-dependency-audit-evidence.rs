#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"
---

//! Validate that dependency-audit evidence used Mantle's checked-in policy.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

const REQUIRED_POLICY_TOKEN: &str = "--config deny.toml";
const REQUIRED_COMMAND_TOOL: &str = "cargo-deny check";
const REQUIRED_SUCCESS_TOKEN: &str = "advisories ok, bans ok, licenses ok, sources ok";
const FORBIDDEN_DEFAULT_POLICY_TOKENS: &[&str] = &[
    "default-policy output",
    "default policy output",
    "missing-policy output",
    "without --config deny.toml",
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
    let Some(path) = args.path else {
        return Err("missing evidence path".to_string());
    };
    let text = fs::read_to_string(&path).map_err(|err| format!("read {}: {err}", path.display()))?;
    validate_evidence(&text)?;
    println!("dependency audit evidence policy check passed");
    Ok(())
}

#[derive(Debug)]
struct Args {
    help: bool,
    self_test: bool,
    path: Option<PathBuf>,
}

impl Args {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: Iterator<Item = String>,
    {
        let mut parsed = Args {
            help: false,
            self_test: false,
            path: None,
        };
        for arg in args {
            match arg.as_str() {
                "--help" | "-h" => parsed.help = true,
                "--self-test" => parsed.self_test = true,
                other if parsed.path.is_none() => parsed.path = Some(PathBuf::from(other)),
                other => return Err(format!("unexpected argument: {other}")),
            }
        }
        Ok(parsed)
    }
}

fn print_usage() {
    println!("Usage: cargo -Zscript scripts/check-dependency-audit-evidence.rs [--self-test] <evidence.md>");
}

fn validate_evidence(text: &str) -> Result<(), String> {
    assert!(!REQUIRED_POLICY_TOKEN.is_empty());
    assert!(!REQUIRED_SUCCESS_TOKEN.is_empty());
    let lower_text = text.to_ascii_lowercase();
    let mut errors = Vec::new();
    require_contains(text, REQUIRED_COMMAND_TOOL, "audit command", &mut errors);
    require_contains(text, REQUIRED_POLICY_TOKEN, "checked-in policy flag", &mut errors);
    require_contains(text, REQUIRED_SUCCESS_TOKEN, "cargo-deny success summary", &mut errors);
    for token in FORBIDDEN_DEFAULT_POLICY_TOKENS {
        if lower_text.contains(token) {
            errors.push(format!("forbidden default-policy marker present: {token}"));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn require_contains(text: &str, needle: &str, label: &str, errors: &mut Vec<String>) {
    assert!(!label.is_empty());
    if !text.contains(needle) {
        errors.push(format!("missing {label}: {needle}"));
    }
}

fn run_self_test() -> Result<(), String> {
    let valid = format!(
        "Command: nix develop -c {tool} {policy}\nOutput: {success}\n",
        tool = REQUIRED_COMMAND_TOOL,
        policy = REQUIRED_POLICY_TOKEN,
        success = REQUIRED_SUCCESS_TOKEN
    );
    validate_evidence(&valid)?;
    assert_rejected("missing policy flag", valid.replace(REQUIRED_POLICY_TOKEN, ""))?;
    assert_rejected("missing success summary", valid.replace(REQUIRED_SUCCESS_TOKEN, "advisories failed"))?;
    assert_rejected(
        "default policy marker",
        format!("{valid}\nThis is default-policy output and must not be accepted.\n"),
    )?;
    println!("dependency audit evidence checker self-test passed");
    Ok(())
}

fn assert_rejected(label: &str, text: String) -> Result<(), String> {
    match validate_evidence(&text) {
        Ok(()) => Err(format!("self-test expected rejection for {label}")),
        Err(_) => Ok(()),
    }
}
