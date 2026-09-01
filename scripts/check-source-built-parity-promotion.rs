#!/usr/bin/env -S cargo -Zscript
---cargo
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
---

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use serde_json::Value;

#[path = "../src/source_built_parity_promotion.rs"]
mod source_built_parity_promotion;
#[path = "../src/source_built_parity_promotion_shell.rs"]
mod source_built_parity_promotion_shell;

const DEFAULT_DESCRIPTOR: &str = "bootstrap/evidence/real-self-build-proof-parity.json";
const SELF_TEST_BAD_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("source-built parity promotion check failed: {error}");
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
    let descriptor = read_descriptor(&args.root, &args.descriptor)?;
    if args.self_test {
        return self_test(&args.root, &descriptor);
    }
    let summary = source_built_parity_promotion_shell::validate_bound_promotion(&args.root, &descriptor)?;
    println!("{}", serde_json::to_string_pretty(&summary).map_err(|error| format!("serialize summary: {error}"))?);
    Ok(())
}

#[derive(Debug)]
struct Args {
    root: PathBuf,
    descriptor: PathBuf,
    self_test: bool,
    help: bool,
}

impl Args {
    fn parse<I>(mut arguments: I) -> Result<Self, String>
    where I: Iterator<Item = String> {
        let mut root = PathBuf::from(".");
        let mut descriptor = PathBuf::from(DEFAULT_DESCRIPTOR);
        let mut self_test = false;
        let mut help = false;
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--root" => root = next_path(&mut arguments, "--root")?,
                "--descriptor" => descriptor = next_path(&mut arguments, "--descriptor")?,
                "--self-test" => self_test = true,
                "-h" | "--help" => help = true,
                unknown => return Err(format!("unknown argument: {unknown}")),
            }
        }
        if !root.is_dir() {
            return Err(format!("root is not a directory: {}", root.display()));
        }
        Ok(Self {
            root,
            descriptor,
            self_test,
            help,
        })
    }
}

fn next_path<I>(arguments: &mut I, option: &str) -> Result<PathBuf, String>
where I: Iterator<Item = String> {
    arguments.next().map(PathBuf::from).ok_or_else(|| format!("{option} requires a path"))
}

fn print_usage() {
    println!("Usage: cargo -Zscript scripts/check-source-built-parity-promotion.rs [--root ROOT] [--descriptor PATH]");
    println!("       cargo -Zscript scripts/check-source-built-parity-promotion.rs --self-test");
}

fn read_descriptor(root: &Path, descriptor: &Path) -> Result<Value, String> {
    let path = root.join(descriptor);
    let bytes = fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn self_test(root: &Path, descriptor: &Value) -> Result<(), String> {
    let positive = source_built_parity_promotion_shell::validate_bound_promotion(root, descriptor)?;
    if !positive.local_only || positive.planned_actions != positive.matched_actions {
        return Err("positive fixture did not prove complete local action coverage".to_string());
    }

    let mut digest_tamper = descriptor.clone();
    digest_tamper["bound_evidence"]["action_plan"]["blake3"] = Value::String(SELF_TEST_BAD_DIGEST.to_string());
    let digest_error = source_built_parity_promotion_shell::validate_bound_promotion(root, &digest_tamper)
        .expect_err("digest tamper must fail");
    if !digest_error.contains("digest mismatch") {
        return Err(format!("digest tamper returned the wrong error: {digest_error}"));
    }

    let mut path_escape = descriptor.clone();
    path_escape["bound_evidence"]["action_plan"]["path"] = Value::String("../escape.json".to_string());
    let path_error = source_built_parity_promotion_shell::validate_bound_promotion(root, &path_escape)
        .expect_err("path escape must fail");
    if !path_error.contains("forbidden component") {
        return Err(format!("path escape returned the wrong error: {path_error}"));
    }

    println!("source-built parity promotion self-test: PASS");
    println!("  positive actions: {}", positive.planned_actions);
    println!("  positive events: {}", positive.observed_events);
    println!("  negative cases: digest-tamper,path-escape");
    Ok(())
}
