#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"
---

//! Validate the foreign derivation import trust-model docs. The guard is
//! structural rather than prose-exact: it keeps trust-boundary headings,
//! non-claim language, examples, and operator links from silently disappearing.

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

const GUIDE_PATH: &str = "docs/foreign-derivation-import-trust-model.md";
const README_PATH: &str = "README.md";
const PROOF_GUIDE_PATH: &str = "docs/operator-proof-guide.md";

const REQUIRED_GUIDE_SECTIONS: &[&str] = &[
    "# Foreign derivation import trust model",
    "## What an import receipt binds",
    "## What an import receipt does not claim",
    "## Trust boundaries",
    "### Graph provenance",
    "### Policy digests",
    "### Hash domains",
    "### Source verification",
    "### Cache and substitution trust",
    "### Sandbox capabilities",
    "### Realization and output verification",
    "## Guix-like hello import",
    "## Nix-like hello import",
    "## Nixpkgs producer adapter levels",
    "## Claim-safe reporting checklist",
];

const REQUIRED_BOUNDARY_TERMS: &[&str] = &[
    "producer identity",
    "raw graph BLAKE3 digest",
    "translation policy BLAKE3 digest",
    "fetch/cache policy digest",
    "sandbox compatibility policy",
    "Nix-compatible hash domain",
    "BLAKE3 receipt domain",
    "source descriptors",
    "PathInfo signature",
    "NAR hash",
    "artifact attestation",
];

const REQUIRED_NON_CLAIMS: &[&str] = &[
    "does not claim build success",
    "does not claim package correctness",
    "does not claim bootstrap parity",
    "does not claim output trust",
    "does not claim reproducibility",
    "does not claim foreign-frontend availability",
    "Additional realization and verification evidence is required before claiming trusted outputs.",
    "Receipt existence is not proof of correctness",
    "cache hints remain subject to store/substitution trust policy",
    "does not bypass output admission or signature verification",
    "local rebuild compatibility is a separate level",
];

const REQUIRED_EXAMPLES: &[&str] = &[
    "Guix-like `hello` graph was admitted",
    "Nix-like `hello` graph was admitted",
    "Nixpkgs `hello` graph was admitted",
    "does not evaluate Nix expressions, flakes, overlays, or module-layer package selection",
    "Substitution-first planning may carry `cache.nixos.org`",
    "without Guix at consumption time",
];

const REQUIRED_LINKS: &[&str] = &[
    "docs/foreign-derivation-import-trust-model.md",
    "nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs",
    "nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test",
];

const FORBIDDEN_OVERCLAIMS: &[&str] = &[
    "receipt proves build success",
    "receipt proves package correctness",
    "receipt proves bootstrap parity",
    "receipt proves output trust",
    "receipt proves reproducibility",
    "receipt bypasses signature verification",
    "admission proves correctness",
    "nixpkgs import proves rebuild compatibility",
    "substitution-first proves package correctness",
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
    let proof_guide = read_to_string(PROOF_GUIDE_PATH)?;
    validate_inputs(GuideInputs {
        guide: &guide,
        readme: &readme,
        proof_guide: &proof_guide,
    })?;
    println!("foreign import trust-model doc check passed");
    Ok(())
}

#[derive(Debug)]
struct Args {
    help: bool,
    self_test: bool,
}

impl Args {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: Iterator<Item = String>,
    {
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
    println!(
        "Usage: cargo -Zscript scripts/check-foreign-import-trust-model.rs [--self-test]"
    );
}

struct GuideInputs<'a> {
    guide: &'a str,
    readme: &'a str,
    proof_guide: &'a str,
}

fn validate_inputs(inputs: GuideInputs<'_>) -> Result<(), String> {
    let mut errors = Vec::new();
    require_all(inputs.guide, "guide section", REQUIRED_GUIDE_SECTIONS, &mut errors);
    require_all(inputs.guide, "trust-boundary term", REQUIRED_BOUNDARY_TERMS, &mut errors);
    require_all(inputs.guide, "receipt non-claim", REQUIRED_NON_CLAIMS, &mut errors);
    require_all(inputs.guide, "foreign import example", REQUIRED_EXAMPLES, &mut errors);
    require_all(inputs.readme, "README linkage", REQUIRED_LINKS, &mut errors);
    require_all(inputs.proof_guide, "proof-guide linkage", REQUIRED_LINKS, &mut errors);
    reject_forbidden(inputs.guide, FORBIDDEN_OVERCLAIMS, &mut errors);
    reject_forbidden(inputs.readme, FORBIDDEN_OVERCLAIMS, &mut errors);
    reject_forbidden(inputs.proof_guide, FORBIDDEN_OVERCLAIMS, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn require_all(haystack: &str, label: &str, needles: &[&str], errors: &mut Vec<String>) {
    let normalized_haystack = normalize_for_match(haystack);
    for needle in needles {
        let normalized_needle = normalize_for_match(needle);
        if !normalized_haystack.contains(&normalized_needle) {
            errors.push(format!("missing {label}: {needle}"));
        }
    }
}

fn reject_forbidden(haystack: &str, forbidden: &[&str], errors: &mut Vec<String>) {
    let normalized_haystack = normalize_for_match(haystack);
    for phrase in forbidden {
        let normalized_phrase = normalize_for_match(phrase);
        if normalized_haystack.contains(&normalized_phrase) {
            errors.push(format!("overbroad foreign import claim detected: {phrase}"));
        }
    }
}

fn normalize_for_match(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn read_to_string(path: impl AsRef<Path>) -> Result<String, String> {
    let path = path.as_ref();
    fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))
}

fn run_self_test() -> Result<(), String> {
    let guide = sample_valid_guide();
    let linked_doc = sample_valid_linked_doc();
    validate_inputs(GuideInputs {
        guide: &guide,
        readme: &linked_doc,
        proof_guide: &linked_doc,
    })?;
    assert_rejected(
        "missing non-claim",
        guide.replace("does not claim output trust", "claims output trust"),
        &linked_doc,
        &linked_doc,
    )?;
    assert_rejected(
        "missing cache trust boundary",
        guide.replace("### Cache and substitution trust", "### Cache reuse"),
        &linked_doc,
        &linked_doc,
    )?;
    assert_rejected(
        "missing Nix-like example",
        guide.replace("Nix-like `hello` graph was admitted", "Nix-like graph example"),
        &linked_doc,
        &linked_doc,
    )?;
    assert_rejected(
        "missing README linkage",
        guide.clone(),
        &linked_doc.replace("scripts/check-foreign-import-trust-model.rs", "scripts/old.rs"),
        &linked_doc,
    )?;
    let overbroad = format!("{guide}\nThe receipt proves build success.\n");
    assert_rejected("overbroad claim", overbroad, &linked_doc, &linked_doc)?;
    println!("foreign import trust-model checker self-test passed");
    Ok(())
}

fn assert_rejected(
    label: &str,
    guide: String,
    readme: &str,
    proof_guide: &str,
) -> Result<(), String> {
    match validate_inputs(GuideInputs {
        guide: &guide,
        readme,
        proof_guide,
    }) {
        Ok(()) => Err(format!("self-test expected rejection for {label}")),
        Err(_) => Ok(()),
    }
}

fn sample_valid_guide() -> String {
    let mut text = String::new();
    append_lines(&mut text, REQUIRED_GUIDE_SECTIONS);
    append_lines(&mut text, REQUIRED_BOUNDARY_TERMS);
    append_lines(&mut text, REQUIRED_NON_CLAIMS);
    append_lines(&mut text, REQUIRED_EXAMPLES);
    text
}

fn sample_valid_linked_doc() -> String {
    let mut text = String::new();
    append_lines(&mut text, REQUIRED_LINKS);
    text
}

fn append_lines(text: &mut String, lines: &[&str]) {
    for line in lines {
        text.push_str(line);
        text.push('\n');
    }
}
