#!/usr/bin/env -S cargo -Zscript
---cargo
[package]
edition = "2024"

[dependencies]
---

//! Fail-closed source/evidence check for the provisional GCC 4.0 configure bridge.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

const DERIVATION_PATH: &str = "bootstrap/gcc-4.0-native.ncl";
const RECEIPT_PATH: &str = "bootstrap/evidence/gcc-4.0-configure-preprocess-bridge.json";
const CONFIGURE_BRIDGE_SOURCE_BYTES_MAX: u64 = 65_536;
const CONFIGURE_BRIDGE_INVOCATION_COUNT_MAX: u32 = 4_096;
const ALLOWED_CONFIGURE_CLASS_COUNT: usize = 3;
const ALLOWED_SOURCE_SPELLING_COUNT: usize = 2;

const REQUIRED_DERIVATION_MARKERS: &[&str] = &[
    "CONFIGURE_PROBE_SOURCE_BYTES_MAX=65536",
    "CONFIGURE_PROBE_INVOCATION_COUNT_MAX=4096",
    "MANTLE_GCC40_CONFIGURE_PROBE_DIR=\"$configure_probe_dir\"",
    "MANTLE_GCC40_CONFIGURE_PROBE_CLASS=\"$dir\"",
    "MANTLE_GCC40_CONFIGURE_PROBE_AUDIT=\"$CONFIGURE_PROBE_AUDIT\"",
    "MANTLE_GCC40_CONFIGURE_PROBE_COUNT=\"$CONFIGURE_PROBE_COUNT\"",
    "libiberty|libcpp|gcc) ;;",
    "conftest.c|./conftest.c) ;;",
    "test -z \"\\$output_file\" || reject_probe explicit-output-forbidden",
    "test \"\\$source_parent\" = \"\\$probe_dir\" || reject_probe source-escape",
    "test \"\\$source_bytes\" -le \"\\$CONFIGURE_PROBE_SOURCE_BYTES_MAX\" || reject_probe source-too-large",
    "test \"\\$probe_count\" -lt \"\\$CONFIGURE_PROBE_INVOCATION_COUNT_MAX\" || reject_probe invocation-budget-exhausted",
    "test \"\\$next_probe_count\" -le \"\\$CONFIGURE_PROBE_INVOCATION_COUNT_MAX\" || reject_probe invocation-budget-exhausted",
    "audit-write-failed",
    "GCC configure preprocess probe produced no audit",
    "GCC configure probe audit directory mismatch",
    "GCC configure probe audit source mismatch",
];

const REQUIRED_RECEIPT_MARKERS: &[&str] = &[
    "\"schema\": \"mantle-gcc40-configure-preprocess-bridge-v1\"",
    "\"derivation\": \"bootstrap/gcc-4.0-native.ncl\"",
    "\"status\": \"bounded-provisional\"",
    "\"mode\": \"configure-only-compile-backed-preprocess-probe\"",
    "\"source_bytes_max\": 65536",
    "\"explicit_output_allowed\": false",
    "\"count_max\": 4096",
    "\"audit_required_before_compiler_execution\": true",
    "\"parent_verification_required\": true",
    "\"provider_eligible\": false",
    "\"general preprocessing correctness\"",
    "\"native GCC 4.0 correctness\"",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct BridgeFacts<'a> {
    authority_dir: &'a str,
    current_dir: &'a str,
    source_spelling: &'a str,
    source_parent: &'a str,
    configure_class: &'a str,
    source_bytes: u64,
    accepted_invocation_count: u32,
    has_explicit_output: bool,
    has_audit_path: bool,
    has_count_path: bool,
}

fn decide_bridge_admission(facts: &BridgeFacts<'_>) -> Result<(), &'static str> {
    assert!(CONFIGURE_BRIDGE_SOURCE_BYTES_MAX > 1);
    assert!(CONFIGURE_BRIDGE_INVOCATION_COUNT_MAX > 1);
    if facts.authority_dir.is_empty() {
        return Err("missing-authority-directory");
    }
    if facts.current_dir != facts.authority_dir {
        return Err("current-directory-mismatch");
    }
    if !matches!(facts.configure_class, "libiberty" | "libcpp" | "gcc") {
        return Err("unknown-configure-class");
    }
    if !matches!(facts.source_spelling, "conftest.c" | "./conftest.c") {
        return Err("non-conftest-source");
    }
    if facts.source_parent != facts.authority_dir {
        return Err("source-escape");
    }
    if facts.has_explicit_output {
        return Err("explicit-output-forbidden");
    }
    if facts.source_bytes > CONFIGURE_BRIDGE_SOURCE_BYTES_MAX {
        return Err("source-too-large");
    }
    if facts.accepted_invocation_count >= CONFIGURE_BRIDGE_INVOCATION_COUNT_MAX {
        return Err("invocation-budget-exhausted");
    }
    if !facts.has_audit_path {
        return Err("missing-audit-path");
    }
    if !facts.has_count_path {
        return Err("missing-count-path");
    }
    Ok(())
}

fn validate_contract(derivation: &str, receipt: &str) -> Result<(), Vec<String>> {
    assert!(!DERIVATION_PATH.is_empty());
    assert!(!RECEIPT_PATH.is_empty());
    let mut issues = Vec::new();
    require_markers(derivation, REQUIRED_DERIVATION_MARKERS, "derivation", &mut issues);
    require_markers(receipt, REQUIRED_RECEIPT_MARKERS, "receipt", &mut issues);
    require_exact_count(receipt, "\"gcc\"", 1, "receipt gcc class", &mut issues);
    require_exact_count(receipt, "\"libcpp\"", 1, "receipt libcpp class", &mut issues);
    require_exact_count(receipt, "\"libiberty\"", 1, "receipt libiberty class", &mut issues);
    require_exact_count(receipt, "\"./conftest.c\"", 1, "receipt dotted source spelling", &mut issues);
    require_exact_count(receipt, "\"conftest.c\"", 1, "receipt source spelling", &mut issues);
    validate_preprocess_order(derivation, &mut issues);
    if issues.is_empty() { Ok(()) } else { Err(issues) }
}

fn require_markers(content: &str, markers: &[&str], owner: &str, issues: &mut Vec<String>) {
    assert!(!owner.is_empty());
    assert!(!markers.is_empty());
    for marker in markers {
        if !content.contains(marker) {
            issues.push(format!("{owner} missing required marker: {marker}"));
        }
    }
}

fn require_exact_count(content: &str, needle: &str, expected_count: usize, label: &str, issues: &mut Vec<String>) {
    assert!(!needle.is_empty());
    assert!(expected_count > 0);
    let actual_count = content.matches(needle).count();
    if actual_count != expected_count {
        issues.push(format!("{label} count mismatch: expected={expected_count} actual={actual_count}"));
    }
}

fn validate_preprocess_order(derivation: &str, issues: &mut Vec<String>) {
    const BRANCH_START: &str = "if [ \"\\$mode\" = preprocess ]; then";
    const BRANCH_END: &str = "if [ \"\\$mode\" = link ]; then";
    const AUDIT_WRITE: &str = "audit-write-failed";
    const FIRST_COMPILER: &str = "\"\\$REAL_TCC\" -c -I\"\\$MUSL_INCLUDE\" \"\\$scratch_source\"";
    assert!(!BRANCH_START.is_empty());
    assert!(!BRANCH_END.is_empty());
    let Some(start_index) = derivation.find(BRANCH_START) else {
        issues.push("derivation preprocess branch start missing".to_string());
        return;
    };
    let Some(relative_end_index) = derivation[start_index..].find(BRANCH_END) else {
        issues.push("derivation preprocess branch end missing".to_string());
        return;
    };
    let branch = &derivation[start_index..start_index.saturating_add(relative_end_index)];
    let audit_index = branch.find(AUDIT_WRITE);
    let compiler_index = branch.find(FIRST_COMPILER);
    match (audit_index, compiler_index) {
        (Some(audit), Some(compiler)) if audit < compiler => {}
        (Some(_), Some(_)) => issues.push("configure bridge invokes compiler before durable audit".to_string()),
        (None, _) => issues.push("configure bridge audit write missing".to_string()),
        (_, None) => issues.push("configure bridge compiler marker missing".to_string()),
    }
}

fn valid_facts() -> BridgeFacts<'static> {
    BridgeFacts {
        authority_dir: "/tmp/build/libcpp",
        current_dir: "/tmp/build/libcpp",
        source_spelling: "conftest.c",
        source_parent: "/tmp/build/libcpp",
        configure_class: "libcpp",
        source_bytes: CONFIGURE_BRIDGE_SOURCE_BYTES_MAX,
        accepted_invocation_count: CONFIGURE_BRIDGE_INVOCATION_COUNT_MAX.saturating_sub(1),
        has_explicit_output: false,
        has_audit_path: true,
        has_count_path: true,
    }
}

fn run_self_tests(derivation: &str, receipt: &str) -> Result<(), String> {
    assert_eq!(ALLOWED_CONFIGURE_CLASS_COUNT, 3);
    assert_eq!(ALLOWED_SOURCE_SPELLING_COUNT, 2);
    decide_bridge_admission(&valid_facts()).map_err(str::to_string)?;

    let negative_cases = [
        (
            "missing authority",
            BridgeFacts {
                authority_dir: "",
                ..valid_facts()
            },
            "missing-authority-directory",
        ),
        (
            "directory mismatch",
            BridgeFacts {
                current_dir: "/tmp/other",
                ..valid_facts()
            },
            "current-directory-mismatch",
        ),
        (
            "unknown class",
            BridgeFacts {
                configure_class: "fixincludes",
                ..valid_facts()
            },
            "unknown-configure-class",
        ),
        (
            "non-conftest",
            BridgeFacts {
                source_spelling: "source.c",
                ..valid_facts()
            },
            "non-conftest-source",
        ),
        (
            "source escape",
            BridgeFacts {
                source_parent: "/tmp/other",
                ..valid_facts()
            },
            "source-escape",
        ),
        (
            "explicit output",
            BridgeFacts {
                has_explicit_output: true,
                ..valid_facts()
            },
            "explicit-output-forbidden",
        ),
        (
            "oversized source",
            BridgeFacts {
                source_bytes: CONFIGURE_BRIDGE_SOURCE_BYTES_MAX.saturating_add(1),
                ..valid_facts()
            },
            "source-too-large",
        ),
        (
            "count exhausted",
            BridgeFacts {
                accepted_invocation_count: CONFIGURE_BRIDGE_INVOCATION_COUNT_MAX,
                ..valid_facts()
            },
            "invocation-budget-exhausted",
        ),
        (
            "missing audit",
            BridgeFacts {
                has_audit_path: false,
                ..valid_facts()
            },
            "missing-audit-path",
        ),
        (
            "missing count",
            BridgeFacts {
                has_count_path: false,
                ..valid_facts()
            },
            "missing-count-path",
        ),
    ];
    for (label, facts, expected_error) in negative_cases {
        let actual_error = match decide_bridge_admission(&facts) {
            Ok(()) => return Err(format!("negative case unexpectedly admitted: {label}")),
            Err(error) => error,
        };
        if actual_error != expected_error {
            return Err(format!("negative case {label}: expected={expected_error} actual={actual_error}"));
        }
    }

    validate_contract(derivation, receipt).map_err(|issues| issues.join("\n"))?;
    let drifted_derivation = derivation.replacen("audit-write-failed", "audit-disabled", 1);
    if validate_contract(&drifted_derivation, receipt).is_ok() {
        return Err("negative source-marker drift was accepted".to_string());
    }
    let drifted_receipt = receipt.replacen("\"provider_eligible\": false", "\"provider_eligible\": true", 1);
    if validate_contract(derivation, &drifted_receipt).is_ok() {
        return Err("negative provider-eligibility drift was accepted".to_string());
    }
    Ok(())
}

fn read_contract(root: &Path) -> Result<(String, String), String> {
    assert!(!DERIVATION_PATH.is_empty());
    assert!(!RECEIPT_PATH.is_empty());
    let derivation_path = root.join(DERIVATION_PATH);
    let receipt_path = root.join(RECEIPT_PATH);
    let derivation = fs::read_to_string(&derivation_path)
        .map_err(|error| format!("reading {}: {error}", derivation_path.display()))?;
    let receipt =
        fs::read_to_string(&receipt_path).map_err(|error| format!("reading {}: {error}", receipt_path.display()))?;
    Ok((derivation, receipt))
}

fn parse_args() -> Result<(PathBuf, bool), String> {
    let mut root = PathBuf::from(".");
    let mut is_self_test = false;
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--root" => {
                let value = args.next().ok_or_else(|| "--root requires a path".to_string())?;
                root = PathBuf::from(value);
            }
            "--self-test" => is_self_test = true,
            _ => return Err(format!("unknown argument: {argument}")),
        }
    }
    Ok((root, is_self_test))
}

fn run() -> Result<(), String> {
    let (root, is_self_test) = parse_args()?;
    let (derivation, receipt) = read_contract(&root)?;
    validate_contract(&derivation, &receipt).map_err(|issues| issues.join("\n"))?;
    if is_self_test {
        run_self_tests(&derivation, &receipt)?;
    }
    println!(
        "gcc40 configure preprocess bridge: PASS (classes={ALLOWED_CONFIGURE_CLASS_COUNT}, source_spellings={ALLOWED_SOURCE_SPELLING_COUNT}, source_bytes_max={CONFIGURE_BRIDGE_SOURCE_BYTES_MAX}, invocation_count_max={CONFIGURE_BRIDGE_INVOCATION_COUNT_MAX})"
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("gcc40 configure preprocess bridge check failed: {error}");
            ExitCode::FAILURE
        }
    }
}
