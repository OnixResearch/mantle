#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"
---

//! Materialize Mantle's locked Cargo vendor tree for the reviewed Nickel cohort.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

const VENDOR_DIR: &str = "vendor-deps";
const LOCK_FILE: &str = "Cargo.lock";
const EVALUATOR_MANIFEST: &str = "crates/crunch-eval/Cargo.toml";
const FACADE_PIN: &str = "nickel-lang = \"=2.2.0\"";
const CORE_PIN: &str = "nickel-lang-core = { version = \"=0.18.0\", default-features = false }";
const EXPECTED_VENDOR_PACKAGES: &[&str] = &[
    "nickel-lang-2.2.0",
    "nickel-lang-core-0.18.0",
    "nickel-lang-parser-0.3.0",
    "nickel-lang-vector-0.2.0",
];
const MAX_ARGUMENT_COUNT: usize = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Request {
    root: PathBuf,
    cargo: Option<PathBuf>,
    execute: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments.as_slice() == ["--self-test"] {
        return self_test();
    }
    let request = parse_request(&arguments)?;
    validate_source(&request.root)?;
    if request.root.join(VENDOR_DIR).exists() {
        return Err(format!("{VENDOR_DIR} already exists; importer refuses replacement"));
    }
    let command = vendor_command(&request)?;
    if !request.execute {
        println!("preview: {}", render_command(&command));
        return Ok(());
    }
    execute_vendor(command)?;
    validate_result(&request.root)?;
    println!("Nickel cohort vendor refresh complete: root={}", request.root.display());
    Ok(())
}

fn parse_request(arguments: &[String]) -> Result<Request, String> {
    if arguments.len() > MAX_ARGUMENT_COUNT {
        return Err(usage());
    }
    let mut root = env::current_dir().map_err(|error| format!("resolve current directory: {error}"))?;
    let mut cargo = None;
    let mut execute = false;
    let mut index = 0_usize;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--root" => {
                index = index.checked_add(1).ok_or_else(|| "argument index overflow".to_string())?;
                root = PathBuf::from(arguments.get(index).ok_or_else(usage)?);
            }
            "--cargo" => {
                index = index.checked_add(1).ok_or_else(|| "argument index overflow".to_string())?;
                cargo = Some(PathBuf::from(arguments.get(index).ok_or_else(usage)?));
            }
            "--execute" => execute = true,
            _ => return Err(usage()),
        }
        index = index.checked_add(1).ok_or_else(|| "argument index overflow".to_string())?;
    }
    if execute && cargo.is_none() {
        return Err("--execute requires an explicit --cargo path".to_string());
    }
    Ok(Request { root, cargo, execute })
}

fn usage() -> String {
    "usage: refresh-nickel-cohort.rs [--root PATH] [--cargo PATH] [--execute] | --self-test".to_string()
}

fn validate_source(root: &Path) -> Result<(), String> {
    if !root.join(LOCK_FILE).is_file() {
        return Err(format!("missing {LOCK_FILE}"));
    }
    let manifest = fs::read_to_string(root.join(EVALUATOR_MANIFEST))
        .map_err(|error| format!("read {EVALUATOR_MANIFEST}: {error}"))?;
    if manifest.matches(FACADE_PIN).count() != 1 {
        return Err("embedded Nickel facade pin is not exact".to_string());
    }
    if manifest.matches(CORE_PIN).count() != 1 {
        return Err("embedded Nickel core pin is not exact".to_string());
    }
    Ok(())
}

fn vendor_command(request: &Request) -> Result<Command, String> {
    let cargo = request
        .cargo
        .as_ref()
        .ok_or_else(|| "preview requires --cargo to render the exact command".to_string())?;
    if !cargo.is_absolute() {
        return Err("--cargo must be an absolute path".to_string());
    }
    let mut command = Command::new(cargo);
    command
        .current_dir(&request.root)
        .arg("vendor")
        .arg("--locked")
        .arg("--versioned-dirs")
        .arg(VENDOR_DIR);
    Ok(command)
}

fn render_command(command: &Command) -> String {
    let program = command.get_program().to_string_lossy();
    let arguments = command.get_args().map(|argument| argument.to_string_lossy()).collect::<Vec<_>>().join(" ");
    format!("{program} {arguments}")
}

fn execute_vendor(mut command: Command) -> Result<(), String> {
    let status = command.status().map_err(|error| format!("launch cargo vendor: {error}"))?;
    if !status.success() {
        return Err(format!("cargo vendor failed with {status}"));
    }
    Ok(())
}

fn validate_result(root: &Path) -> Result<(), String> {
    let vendor = root.join(VENDOR_DIR);
    if !vendor.is_dir() {
        return Err("cargo vendor did not create vendor-deps".to_string());
    }
    for package in EXPECTED_VENDOR_PACKAGES {
        let directory = vendor.join(package);
        if !directory.is_dir() {
            return Err(format!("cargo vendor omitted {package}"));
        }
        if !directory.join(".cargo-checksum.json").is_file() {
            return Err(format!("cargo vendor omitted {package}/.cargo-checksum.json"));
        }
    }
    Ok(())
}

fn self_test() -> Result<(), String> {
    let preview = parse_request(&[
        "--root".to_string(),
        "/tmp/source".to_string(),
        "--cargo".to_string(),
        "/toolchain/bin/cargo".to_string(),
    ])?;
    if preview.execute || preview.cargo.as_deref() != Some(Path::new("/toolchain/bin/cargo")) {
        return Err("positive importer request parsed incorrectly".to_string());
    }
    let execute_without_cargo = parse_request(&["--execute".to_string()]);
    if execute_without_cargo.is_ok() {
        return Err("execute without explicit Cargo passed".to_string());
    }
    let relative_cargo = Request {
        root: PathBuf::from("/tmp/source"),
        cargo: Some(PathBuf::from("cargo")),
        execute: false,
    };
    if vendor_command(&relative_cargo).is_ok() {
        return Err("relative Cargo authority passed".to_string());
    }
    println!("Nickel cohort importer positive and negative self-tests passed");
    Ok(())
}
