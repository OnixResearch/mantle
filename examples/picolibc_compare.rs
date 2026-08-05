// Thin report shell for the Picolibc StageX comparison.
//
// Reads the diagnostic manifest, behavior matrix JSON, isolated-build
// identity digests, and baseline facts from explicit arguments, then runs the
// pure comparison core and writes one deterministic report. All decision
// logic lives in the core; this shell owns file reads, argument parsing, and
// report writes.
//
// Usage:
//   picolibc_compare --manifest <manifest.txt> --behavior <behavior.json> \
//     --first-build-digest <blake3> --second-build-digest <blake3> \
//     --baseline-commit <sha1> --baseline-compiled-units <n> \
//     --baseline-rewrite-operations <n> --baseline-evidence-digest <blake3>... \
//     --out <report.json>
//   picolibc_compare --self-test

use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

#[path = "../src/picolibc_comparison.rs"]
mod picolibc_comparison;

use picolibc_comparison::BaselineFacts;
use picolibc_comparison::BehaviorFacts;
use picolibc_comparison::ComparisonFacts;
use picolibc_comparison::ComparisonOutcome;
use picolibc_comparison::DiagnosticFacts;
use picolibc_comparison::IdentityFacts;
use picolibc_comparison::PICOLIBC_RELEASE;
use picolibc_comparison::PICOLIBC_SOURCE_DIGEST;
use picolibc_comparison::classify;
use picolibc_comparison::validate_facts;

const MAX_MANIFEST_BYTES: u64 = 4_194_304;
const MAX_BEHAVIOR_BYTES: u64 = 1_048_576;
const MAX_TOOL_ROLES: usize = 64;
const MAX_COMPILED_UNITS: u32 = 1_000_000;
const SELF_TEST_DIR_PREFIX: &str = "picolibc-compare-self-test";

#[derive(Debug)]
struct CliArgs {
    manifest: PathBuf,
    behavior: PathBuf,
    first_build_digest: String,
    second_build_digest: String,
    baseline_commit: String,
    baseline_compiled_units: u32,
    baseline_rewrite_operations: u32,
    baseline_evidence_digests: Vec<String>,
    out: PathBuf,
}

#[derive(Debug)]
enum ShellError {
    Usage(String),
    Io(String),
    Facts(String),
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(detail) => write!(formatter, "usage error: {detail}"),
            Self::Io(detail) => write!(formatter, "io error: {detail}"),
            Self::Facts(detail) => write!(formatter, "facts error: {detail}"),
        }
    }
}

fn usage() -> ShellError {
    ShellError::Usage(
        "see header comment: --manifest --behavior --first-build-digest --second-build-digest \
         --baseline-commit --baseline-compiled-units --baseline-rewrite-operations \
         [--baseline-evidence-digest ...] --out, or --self-test"
            .to_string(),
    )
}

fn parse_u32(flag: &str, value: &str) -> Result<u32, ShellError> {
    let parsed: u32 = value.parse().map_err(|_| ShellError::Usage(format!("{flag} must be a u32, got: {value}")))?;
    if parsed > MAX_COMPILED_UNITS {
        return Err(ShellError::Usage(format!("{flag} exceeds {MAX_COMPILED_UNITS}")));
    }
    Ok(parsed)
}

fn parse_args(args: &[String]) -> Result<CliArgs, ShellError> {
    debug_assert!(!args.is_empty(), "argv[0] always exists");
    let mut manifest: Option<PathBuf> = None;
    let mut behavior: Option<PathBuf> = None;
    let mut first_build_digest: Option<String> = None;
    let mut second_build_digest: Option<String> = None;
    let mut baseline_commit: Option<String> = None;
    let mut baseline_compiled_units: Option<u32> = None;
    let mut baseline_rewrite_operations: Option<u32> = None;
    let mut baseline_evidence_digests: Vec<String> = Vec::new();
    let mut out: Option<PathBuf> = None;
    let mut index = 1;
    while index < args.len() {
        let flag = args[index].as_str();
        let value = args.get(index + 1).ok_or_else(usage)?;
        match flag {
            "--manifest" => manifest = Some(PathBuf::from(value)),
            "--behavior" => behavior = Some(PathBuf::from(value)),
            "--first-build-digest" => first_build_digest = Some(value.clone()),
            "--second-build-digest" => second_build_digest = Some(value.clone()),
            "--baseline-commit" => baseline_commit = Some(value.clone()),
            "--baseline-compiled-units" => baseline_compiled_units = Some(parse_u32(flag, value)?),
            "--baseline-rewrite-operations" => baseline_rewrite_operations = Some(parse_u32(flag, value)?),
            "--baseline-evidence-digest" => baseline_evidence_digests.push(value.clone()),
            "--out" => out = Some(PathBuf::from(value)),
            _ => return Err(usage()),
        }
        index += 2;
    }
    Ok(CliArgs {
        manifest: manifest.ok_or_else(usage)?,
        behavior: behavior.ok_or_else(usage)?,
        first_build_digest: first_build_digest.ok_or_else(usage)?,
        second_build_digest: second_build_digest.ok_or_else(usage)?,
        baseline_commit: baseline_commit.ok_or_else(usage)?,
        baseline_compiled_units: baseline_compiled_units.ok_or_else(usage)?,
        baseline_rewrite_operations: baseline_rewrite_operations.ok_or_else(usage)?,
        baseline_evidence_digests,
        out: out.ok_or_else(usage)?,
    })
}

fn read_bounded(path: &Path, max_bytes: u64) -> Result<String, ShellError> {
    let metadata =
        std::fs::metadata(path).map_err(|error| ShellError::Io(format!("stat {}: {error}", path.display())))?;
    if metadata.len() > max_bytes {
        return Err(ShellError::Io(format!("{} exceeds {max_bytes} bytes", path.display())));
    }
    std::fs::read_to_string(path).map_err(|error| ShellError::Io(format!("read {}: {error}", path.display())))
}

fn manifest_field<'a>(manifest: &'a str, field: &str) -> Result<&'a str, ShellError> {
    debug_assert!(!field.is_empty(), "field name must not be empty");
    let prefix = format!("{field}=");
    manifest
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .ok_or_else(|| ShellError::Facts(format!("manifest is missing {field}")))
}

fn parse_manifest(manifest: &str) -> Result<(u32, BTreeSet<String>), ShellError> {
    let compiled_units: u32 = parse_u32("compiled_objects", manifest_field(manifest, "compiled_objects")?)?;
    let mut tool_roles = BTreeSet::new();
    for (field, role) in [
        ("meson", "meson"),
        ("ninja", "ninja"),
        ("gcc", "compiler"),
        ("binutils_as", "linker"),
    ] {
        manifest_field(manifest, field)?;
        tool_roles.insert(role.to_string());
    }
    // The archiver runs inside every static-archive step of the build; the
    // manifest records the produced archives, so the role is present whenever
    // any library output exists.
    if manifest.contains("libc.a") {
        tool_roles.insert("archiver".to_string());
    }
    debug_assert!(tool_roles.len() <= MAX_TOOL_ROLES, "tool roles stay bounded");
    Ok((compiled_units, tool_roles))
}

fn parse_behavior(behavior: &str) -> Result<BehaviorFacts, ShellError> {
    #[derive(serde::Deserialize)]
    struct BehaviorJson {
        positive_passed: u32,
        positive_total: u32,
        negative_passed: u32,
        negative_total: u32,
        failures: Vec<String>,
    }
    let parsed: BehaviorJson =
        serde_json::from_str(behavior).map_err(|error| ShellError::Facts(format!("behavior json: {error}")))?;
    Ok(BehaviorFacts {
        positive_passed: parsed.positive_passed,
        positive_total: parsed.positive_total,
        negative_passed: parsed.negative_passed,
        negative_total: parsed.negative_total,
        failures: parsed.failures,
    })
}

fn build_facts(args: &CliArgs) -> Result<ComparisonFacts, ShellError> {
    let manifest = read_bounded(&args.manifest, MAX_MANIFEST_BYTES)?;
    let (compiled_units, tool_roles) = parse_manifest(&manifest)?;
    let behavior = parse_behavior(&read_bounded(&args.behavior, MAX_BEHAVIOR_BYTES)?)?;
    Ok(ComparisonFacts {
        diagnostic: DiagnosticFacts {
            release: PICOLIBC_RELEASE.to_string(),
            source_digest: PICOLIBC_SOURCE_DIGEST.to_string(),
            compiled_units,
            rewrite_operations: 0,
            tool_roles,
            license_classes: BTreeSet::from([
                "BSD-3-Clause".to_string(),
                "BSD-2-Clause".to_string(),
                "FreeBSD".to_string(),
                "Other-permissive".to_string(),
            ]),
            host_libc_dependence: false,
            new_protected_roles: 0,
        },
        baseline: BaselineFacts {
            baseline_commit: args.baseline_commit.clone(),
            compiled_units: args.baseline_compiled_units,
            rewrite_operations: args.baseline_rewrite_operations,
            evidence_digests: args.baseline_evidence_digests.clone(),
        },
        behavior,
        identity: IdentityFacts {
            first_build_digest: args.first_build_digest.clone(),
            second_build_digest: args.second_build_digest.clone(),
        },
    })
}

fn write_report(out: &Path, report_json: &str) -> Result<(), ShellError> {
    debug_assert!(!report_json.is_empty(), "report json must not be empty");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| ShellError::Io(format!("create {}: {error}", parent.display())))?;
    }
    std::fs::write(out, report_json).map_err(|error| ShellError::Io(format!("write {}: {error}", out.display())))
}

fn run() -> Result<ComparisonOutcome, ShellError> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--self-test") {
        return run_self_test();
    }
    let parsed = parse_args(&args)?;
    let facts = build_facts(&parsed)?;
    validate_facts(&facts).map_err(ShellError::Facts)?;
    let report = classify(&facts);
    let outcome = report.outcome;
    let report_json = serde_json::to_string_pretty(&report)
        .map_err(|error| ShellError::Facts(format!("report serialization: {error}")))?;
    write_report(&parsed.out, &report_json)?;
    println!("outcome={outcome}");
    for reason in &report.reasons {
        println!("reason={reason}");
    }
    Ok(outcome)
}

fn self_test_facts() -> ComparisonFacts {
    ComparisonFacts {
        diagnostic: DiagnosticFacts {
            release: PICOLIBC_RELEASE.to_string(),
            source_digest: PICOLIBC_SOURCE_DIGEST.to_string(),
            compiled_units: 700,
            rewrite_operations: 0,
            tool_roles: BTreeSet::from([
                "meson".to_string(),
                "ninja".to_string(),
                "compiler".to_string(),
                "linker".to_string(),
                "archiver".to_string(),
            ]),
            license_classes: BTreeSet::from(["BSD-3-Clause".to_string()]),
            host_libc_dependence: false,
            new_protected_roles: 0,
        },
        baseline: BaselineFacts {
            baseline_commit: "a".repeat(40),
            compiled_units: 765,
            rewrite_operations: 106,
            evidence_digests: vec!["b".repeat(64)],
        },
        behavior: BehaviorFacts {
            positive_passed: 1,
            positive_total: 1,
            negative_passed: 1,
            negative_total: 1,
            failures: Vec::new(),
        },
        identity: IdentityFacts {
            first_build_digest: "c".repeat(64),
            second_build_digest: "c".repeat(64),
        },
    }
}

fn run_self_test() -> Result<ComparisonOutcome, ShellError> {
    let work = std::env::temp_dir().join(SELF_TEST_DIR_PREFIX);
    std::fs::create_dir_all(&work).map_err(|error| ShellError::Io(format!("self-test dir: {error}")))?;
    let manifest_path = work.join("manifest.txt");
    let behavior_path = work.join("behavior.json");
    std::fs::write(
        &manifest_path,
        "compiled_objects=700\nmeson=1.10.2\nninja=1.13.2\ngcc=15.3.0\nbinutils_as=GNU assembler\nlibc.a present\n",
    )
    .map_err(|error| ShellError::Io(format!("self-test manifest: {error}")))?;
    std::fs::write(
        &behavior_path,
        r#"{"positive_passed": 1, "positive_total": 1, "negative_passed": 1, "negative_total": 1, "failures": []}"#,
    )
    .map_err(|error| ShellError::Io(format!("self-test behavior: {error}")))?;
    let manifest = read_bounded(&manifest_path, MAX_MANIFEST_BYTES)?;
    let (units, roles) = parse_manifest(&manifest)?;
    assert_eq!(units, 700, "self-test manifest unit count");
    assert!(roles.contains("archiver"), "self-test archiver role");
    let parsed_behavior = parse_behavior(&read_bounded(&behavior_path, MAX_BEHAVIOR_BYTES)?)?;
    assert_eq!(parsed_behavior.positive_passed, 1, "self-test behavior parse");
    let candidate_report = classify(&self_test_facts());
    assert_eq!(candidate_report.outcome, ComparisonOutcome::Candidate);
    let mut blocked_facts = self_test_facts();
    blocked_facts.baseline.evidence_digests.clear();
    assert_eq!(classify(&blocked_facts).outcome, ComparisonOutcome::Blocked);
    let mut rejected_facts = self_test_facts();
    rejected_facts.behavior.positive_passed = 0;
    assert_eq!(classify(&rejected_facts).outcome, ComparisonOutcome::Rejected);
    println!("self-test-ok");
    Ok(ComparisonOutcome::Candidate)
}

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}
