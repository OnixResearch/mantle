#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"
---

//! Validate the operator proof guide against the current documented proof
//! command and evidence vocabulary. This guard is intentionally structural: it
//! catches stale snippets, missing bundle fields, and overbroad claim wording
//! without executing expensive proof workflows.

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

const GUIDE_PATH: &str = "docs/operator-proof-guide.md";
const README_PATH: &str = "README.md";

const REQUIRED_GUIDE_SECTIONS: &[&str] = &[
    "# Mantle operator proof guide",
    "## Workflow chooser",
    "## Self-build proof",
    "## Cargo-free fixed-point proof",
    "## Nix-free demo bundle validation",
    "## Cleanup",
    "## Drift check",
];

const REQUIRED_COMMANDS: &[&str] = &[
    "./scripts/prove-self-hosting.sh --check",
    "./scripts/prove-self-hosting.sh",
    "./scripts/prove-self-hosting.sh --non-nix-host",
    "./scripts/prove-self-hosting.sh --no-host-tools --stage0-inventory <file>",
    "mantle self-build --cargo-free --out /tmp/mantle-cargo-free",
    "mantle --json nix-free-demo validate <summary.json>",
    "mantle nix-free-demo readme <summary.json>",
    "nix develop -c cargo -Zscript scripts/prove-cargo-free-fixed-point.rs --check --root .",
    "nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs",
    "nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs --self-test",
];

const REQUIRED_README_FRAGMENTS: &[&str] = &[
    "docs/operator-proof-guide.md",
    "nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs",
    "./scripts/prove-self-hosting.sh --check",
    "./scripts/prove-self-hosting.sh --non-nix-host",
    "./scripts/prove-self-hosting.sh --no-host-tools --stage0-inventory <file>",
    "mantle self-build --cargo-free --out /tmp/mantle-cargo-free",
    "mantle --json nix-free-demo validate <summary.json>",
    "mantle nix-free-demo readme <summary.json>",
];

const REQUIRED_SELF_BUILD_PATHS: &[&str] = &[
    "target/self-hosting-proof/run-.../",
    "target/self-hosting-proof/latest",
    "manifest.json",
    "summary.txt",
    "binaries/stage1-mantle",
    "binaries/stage2-mantle",
    "stage0/stdout.txt",
    "stage0/stderr.txt",
    "stage0/diagnostics.txt",
    "stage2/stdout.txt",
    "stage2/stderr.txt",
    "stage2/diagnostics.txt",
    "stage0-prerequisites/inventory.md",
    "protected-exec-audit.json",
];

const REQUIRED_CARGO_FREE_PATHS: &[&str] = &[
    "mantle-cargo-free-fixed-point-proof-v1",
    "meta.json",
    "preflight.json",
    "non-claims.txt",
    "stage1/receipt.json",
    "stage2/receipt.json",
    "stage1/smoke-stdout.txt",
    "stage2/smoke-stdout.txt",
];

const REQUIRED_CURRENT_EVIDENCE_FRAGMENTS: &[&str] = &[
    "Current refreshed evidence (2026-07-03)",
    "/tmp/mantle-cargo-free-fixed-point-classifier2-20260703T203015Z",
    "blocker_diagnostic",
    "vendor-checksum-mismatch",
    "registry+https://github.com/rust-lang/crates.io-index#astral-tokio-tar@0.6.3",
    "stage1 blocked before binary",
    "not a Nix-free fixed-point success claim",
];

const REQUIRED_DEMO_FIELDS: &[&str] = &[
    "mantle-nix-free-demo-summary-v1",
    "source-root-cargo-free-fixed-point",
    "fixed_point_verdict",
    "stage1_binary_blake3",
    "stage2_binary_blake3",
    "source_root_identity",
    "toolchain_policy_digest_blake3",
    "command_owned_wrappers",
    "guards",
    "replay_hints",
    "non_claims",
    "cargo",
    "nix",
    "rustup",
    "ambient-wrapper",
    "missing-fixed-point-evidence",
    "missing-guard-evidence",
    "missing-non-claims",
];

const REQUIRED_OUTCOME_FRAGMENTS: &[&str] = &[
    "**Success:**",
    "**Blocked:**",
    "**Failed:**",
    "**Stale evidence:**",
    "Blocked evidence is not proof success.",
    "status = \"success\"",
    "status = \"blocked\"",
    "Demo claim: not claimable",
    "Nix-free fixed-point demo: claimable",
];

const REQUIRED_NON_CLAIMS: &[&str] = &[
    "does not prove compiler correctness",
    "does not prove full Cargo compatibility",
    "does not prove release reproducibility",
    "does not prove deploy success",
    "does not prove general Nix replacement completeness",
];

const FORBIDDEN_OVERBROAD_CLAIMS: &[&str] = &[
    "proves compiler correctness",
    "proves full cargo compatibility",
    "proves release reproducibility",
    "proves deploy success",
    "proves general nix replacement completeness",
    "blocked evidence is proof success",
    "stale evidence is proof success",
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
    let guide = read_to_string(GUIDE_PATH)?;
    let readme = read_to_string(README_PATH)?;
    validate_inputs(GuideInputs {
        guide: &guide,
        readme: &readme,
    })?;
    println!("operator proof guide drift check passed");
    Ok(())
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
    println!("Usage: cargo -Zscript scripts/check-operator-proof-guide.rs [--self-test]");
}

struct GuideInputs<'a> {
    guide: &'a str,
    readme: &'a str,
}

fn validate_inputs(inputs: GuideInputs<'_>) -> Result<(), String> {
    let mut errors = Vec::new();
    require_all(inputs.guide, "guide section", REQUIRED_GUIDE_SECTIONS, &mut errors);
    require_all(inputs.guide, "guide command", REQUIRED_COMMANDS, &mut errors);
    require_all(inputs.readme, "README linkage", REQUIRED_README_FRAGMENTS, &mut errors);
    require_all(inputs.guide, "self-build evidence path", REQUIRED_SELF_BUILD_PATHS, &mut errors);
    require_all(inputs.guide, "cargo-free evidence path", REQUIRED_CARGO_FREE_PATHS, &mut errors);
    require_all(
        inputs.guide,
        "current cargo-free evidence",
        REQUIRED_CURRENT_EVIDENCE_FRAGMENTS,
        &mut errors,
    );
    require_all(inputs.guide, "demo summary field", REQUIRED_DEMO_FIELDS, &mut errors);
    require_all(inputs.guide, "outcome vocabulary", REQUIRED_OUTCOME_FRAGMENTS, &mut errors);
    require_all(inputs.guide, "bounded non-claim", REQUIRED_NON_CLAIMS, &mut errors);
    reject_forbidden(inputs.guide, FORBIDDEN_OVERBROAD_CLAIMS, &mut errors);
    reject_forbidden(inputs.readme, FORBIDDEN_OVERBROAD_CLAIMS, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn require_all(haystack: &str, label: &str, needles: &[&str], errors: &mut Vec<String>) {
    for needle in needles {
        if !haystack.contains(needle) {
            errors.push(format!("missing {label}: {needle}"));
        }
    }
}

fn reject_forbidden(haystack: &str, forbidden: &[&str], errors: &mut Vec<String>) {
    let lower = haystack.to_ascii_lowercase();
    for phrase in forbidden {
        if lower.contains(phrase) {
            errors.push(format!("overbroad proof claim detected: {phrase}"));
        }
    }
}

fn read_to_string(path: impl AsRef<Path>) -> Result<String, String> {
    let path = path.as_ref();
    fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))
}

fn run_self_test() -> Result<(), String> {
    let guide = sample_valid_guide();
    let readme = sample_valid_readme();
    validate_inputs(GuideInputs {
        guide: &guide,
        readme: &readme,
    })?;
    assert_rejected(
        "stale cargo-free command",
        guide.replace(
            "mantle self-build --cargo-free --out /tmp/mantle-cargo-free",
            "mantle self-build --cargo-free --fixed-point",
        ),
        &readme,
    )?;
    assert_rejected(
        "missing demo field",
        guide.replace("toolchain_policy_digest_blake3", "toolchain_policy_digest"),
        &readme,
    )?;
    assert_rejected("missing cargo-free bundle path", guide.replace("stage1/receipt.json", "stage1/old-receipt.json"), &readme)?;
    assert_rejected(
        "stale current blocker",
        guide.replace("vendor-checksum-mismatch", "cargo-free-bounded-topology"),
        &readme,
    )?;
    let overbroad = format!("{guide}\nThis guide proves compiler correctness.\n");
    assert_rejected("overbroad claim", overbroad, &readme)?;
    println!("operator proof guide checker self-test passed");
    Ok(())
}

fn assert_rejected(label: &str, guide: String, readme: &str) -> Result<(), String> {
    match validate_inputs(GuideInputs { guide: &guide, readme }) {
        Ok(()) => Err(format!("self-test expected rejection for {label}")),
        Err(_) => Ok(()),
    }
}

fn sample_valid_guide() -> String {
    let mut text = String::new();
    append_lines(&mut text, REQUIRED_GUIDE_SECTIONS);
    append_lines(&mut text, REQUIRED_COMMANDS);
    append_lines(&mut text, REQUIRED_SELF_BUILD_PATHS);
    append_lines(&mut text, REQUIRED_CARGO_FREE_PATHS);
    append_lines(&mut text, REQUIRED_CURRENT_EVIDENCE_FRAGMENTS);
    append_lines(&mut text, REQUIRED_DEMO_FIELDS);
    append_lines(&mut text, REQUIRED_OUTCOME_FRAGMENTS);
    append_lines(&mut text, REQUIRED_NON_CLAIMS);
    text
}

fn sample_valid_readme() -> String {
    let mut text = String::new();
    append_lines(&mut text, REQUIRED_README_FRAGMENTS);
    text
}

fn append_lines(text: &mut String, lines: &[&str]) {
    for line in lines {
        text.push_str(line);
        text.push('\n');
    }
}
