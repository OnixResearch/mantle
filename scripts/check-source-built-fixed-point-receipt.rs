#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"

[dependencies]
mantle = { path = ".." }
serde_json = "1"
tempfile = "3"
---

//! Portable verification entrypoint for a copied source-built proof bundle.
//!
//! Build this checker from the reviewed verifier checkout. At runtime it only
//! opens the explicitly selected proof root; receipt paths never select a
//! producer checkout. The checker reruns Mantle's complete receipt and action
//! reconciliation checks rather than accepting stored success booleans.

use std::path::PathBuf;
use std::process::ExitCode;

fn run() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let mut root = None;
    let mut json = false;
    let mut self_test = false;
    while let Some(argument) = args.next() {
        if argument == "--proof-root" {
            if root.is_some() {
                return Err("--proof-root may be supplied only once".into());
            }
            root = Some(PathBuf::from(args.next().ok_or("--proof-root requires a path")?));
        } else if argument == "--json" {
            json = true;
        } else if argument == "--self-test" {
            self_test = true;
        } else if argument == "--help" || argument == "-h" {
            println!(
                "Usage: cargo -Zscript scripts/check-source-built-fixed-point-receipt.rs --proof-root DIR [--json]"
            );
            println!("       cargo -Zscript scripts/check-source-built-fixed-point-receipt.rs --self-test [--json]");
            println!(
                "Rechecks the v2 receipt, full action records, reconciliation, and bundle contents without executing a proof."
            );
            return Ok(());
        } else {
            return Err(format!("unknown argument: {}", argument.to_string_lossy()));
        }
    }
    if self_test {
        if root.is_some() {
            return Err("--self-test cannot be combined with --proof-root".into());
        }
        let scratch = tempfile::tempdir().map_err(|error| format!("creating self-test directory: {error}"))?;
        mantle::source_built_fixed_point_verifier_self_test::self_test_bootstrap_trust_report(scratch.path())
            .map_err(|error| error.to_string())?;
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "self_test": "passed", "verified": false, "proof_executed": false,
                    "scope": ["empty-and-malformed-bundle-rejection", "root-relative-path-guards",
                        "durable-journal-omission-and-tamper-rejection", "development-authority-rejection"],
                    "non_claim": "Verifier guard self-checks only; no accepted receipt or executed proof is established."
                })
            );
        } else {
            println!("portable verifier self-test passed (guard checks only; no proof executed or accepted)");
        }
        return Ok(());
    }
    let root = root.ok_or("--proof-root is required")?;
    mantle::source_built_fixed_point_trust_report::cmd_bootstrap_trust_report(&root, json)
        .map_err(|error| error.to_string())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", serde_json::json!({"error": error, "verified": false}));
            ExitCode::FAILURE
        }
    }
}
