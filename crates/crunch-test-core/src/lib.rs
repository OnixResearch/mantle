#![no_std]
//! Bounded Rust libtest contract. Scheduling and signed reuse remain Mantle's.
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crunch_spacewasm_core::HarnessLine;
use crunch_spacewasm_core::parse_libtest_events;
use serde::Deserialize;
use serde::Serialize;

pub const TARGET_SCHEMA: &str = "mantle-test-target-v1";
pub const REPORT_SCHEMA: &str = "mantle-test-report-v1";
pub const COHORT: &str = "rust-1.90.0-llvm-20.1.8-libtest-json-v1";
pub const MAX_TESTS: usize = 256;
pub const MAX_CAPTURE: u64 = 1_048_576;
type Result<T, E = String> = core::result::Result<T, E>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    pub root: String,
    pub member: String,
    pub blake3: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub profdata: Tool,
    pub cov: Tool,
    pub source_root: String,
    pub source_blake3: String,
    pub max_profile_bytes: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    pub memory_bytes: u64,
    pub output_bytes: u64,
    pub timeout_ms: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub schema: String,
    pub name: String,
    pub cohort: String,
    pub rustc_verbose_version: String,
    pub store_prefix: String,
    pub platform: String,
    pub executable: Tool,
    pub runner: Tool,
    /// Every listed root must have been admitted by Mantle; the string is a NAR BLAKE3 identity.
    pub inputs: BTreeMap<String, String>,
    pub runfiles: BTreeMap<String, String>,
    pub environment: BTreeMap<String, String>,
    pub tests: Vec<String>,
    pub resources: Resources,
    pub deterministic: bool,
    pub external_services: bool,
    pub coverage: Option<Coverage>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Invocation {
    pub filter: Option<String>,
    pub exact: bool,
    pub shard_index: u32,
    pub shard_count: u32,
    pub timeout_ms: Option<u64>,
    pub seed: u64,
    pub repeat: u32,
    /// Required unique invocation token when bypassing cached results.
    pub force_rerun: Option<String>,
    pub coverage: bool,
}
impl Default for Invocation {
    fn default() -> Self {
        Self {
            filter: None,
            exact: false,
            shard_index: 0,
            shard_count: 1,
            timeout_ms: None,
            seed: 0,
            repeat: 1,
            force_rerun: None,
            coverage: false,
        }
    }
}
/// Validated selection and its wire identity, carried through the application once.
#[derive(Debug)]
pub struct SelectedRequest {
    pub target: Target,
    pub invocation: Invocation,
    pub selected: Vec<String>,
    pub identity: String,
}
pub struct ClassifiedCapture {
    pub name: String,
    pub outcome: Outcome,
}
/// One test's harness transcript, bound for classification.
struct CaptureTranscript<'a> {
    name: &'a str,
    text: &'a str,
    exit: Option<i32>,
}
pub struct AttemptSummary {
    pub attempts: Vec<Attempt>,
    pub outcome: Outcome,
}
pub struct ReportSummary {
    pub report: Report,
    pub text: String,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    Pass,
    AssertionFailure,
    Timeout,
    Cancelled,
    InfrastructureFailure,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub repeat: u32,
    pub test: String,
    pub outcome: Outcome,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CoverageReport {
    pub cohort: String,
    pub source_root: String,
    pub source_blake3: String,
    pub profdata_blake3: String,
    pub cov_blake3: String,
    pub export_blake3: String,
    pub covered_lines: u64,
    pub total_lines: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CollectionFailure {
    pub outcome: Outcome,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub schema: String,
    pub identity: String,
    pub target: String,
    pub selected: Vec<String>,
    pub outcome: Outcome,
    pub attempts: Vec<Attempt>,
    pub coverage: Option<CoverageReport>,
    pub collection_failure: Option<CollectionFailure>,
}
fn digest(value: &impl Serialize) -> Result<String> {
    let bytes = serde_json::to_vec(value).map_err(|_| "test-encoding".to_string())?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}
fn hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn relative(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && value.split('/').all(|s| !s.is_empty() && s != "." && s != ".." && !s.contains('\0'))
}
/// A store prefix; the newtype keeps a prefix distinct from a store path.
#[derive(Clone, Copy, Debug)]
struct StorePrefix<'a>(&'a str);

fn root(value: &str, prefix: StorePrefix<'_>) -> bool {
    value
        .strip_prefix(prefix.0)
        .and_then(|s| s.strip_prefix('/'))
        .is_some_and(|s| !s.is_empty() && !s.contains('/') && s != "." && s != "..")
}
fn tool(target: &Target, value: &Tool) -> bool {
    target.inputs.contains_key(&value.root) && relative(&value.member) && hex(&value.blake3)
}
fn validate(target: &Target) -> Result<()> {
    validate_identity(target)?;
    validate_inputs(target)?;
    validate_inventory(target)?;
    validate_environment(target)?;
    Ok(())
}
/// Schema, toolchain, platform, store and naming identity of a target.
fn validate_identity(target: &Target) -> Result<()> {
    if target.schema != TARGET_SCHEMA || target.cohort != COHORT {
        return Err("test-schema-or-cohort-unsupported".into());
    }
    if !target.rustc_verbose_version.starts_with("rustc 1.90.0 (1159e78c4 2025-09-14)\n")
        || !target.rustc_verbose_version.contains("LLVM version: 20.1.8")
    {
        return Err("test-rustc-cohort-mismatch".into());
    }
    if !target.deterministic || target.external_services {
        return Err("test-nondeterministic-or-external-service-unsupported".into());
    }
    if target.platform != "x86_64-linux" && target.platform != "aarch64-linux" {
        return Err("test-platform-unsupported".into());
    }
    if !target.store_prefix.starts_with('/') || target.store_prefix.ends_with('/') || target.store_prefix.contains("..")
    {
        return Err("test-store-prefix".into());
    }
    if target.name.is_empty()
        || target.name.len() > 80
        || !target.name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("test-name".into());
    }
    debug_assert!(target.schema == TARGET_SCHEMA);
    debug_assert!(target.cohort == COHORT);
    debug_assert!(target.rustc_verbose_version.contains("LLVM version: 20.1.8"));
    debug_assert!(target.deterministic);
    debug_assert!(!target.external_services);
    debug_assert!(target.platform == "x86_64-linux" || target.platform == "aarch64-linux");
    debug_assert!(target.store_prefix.starts_with('/'));
    debug_assert!(!target.store_prefix.ends_with('/'));
    debug_assert!(!target.name.is_empty());
    debug_assert!(target.name.len() <= 80);
    Ok(())
}
/// Store inputs declared by a target and the tools drawn from them.
fn validate_inputs(target: &Target) -> Result<()> {
    if target.inputs.is_empty()
        || target.inputs.len() > 128
        || target.inputs.iter().any(|(p, h)| !root(p, StorePrefix(&target.store_prefix)) || !hex(h))
    {
        return Err("test-inputs".into());
    }
    if !tool(target, &target.executable) || !tool(target, &target.runner) {
        return Err("test-tool-undeclared".into());
    }
    debug_assert!(!target.inputs.is_empty());
    debug_assert!(target.inputs.len() <= 128);
    debug_assert!(target.inputs.contains_key(&target.executable.root));
    debug_assert!(target.inputs.contains_key(&target.runner.root));
    Ok(())
}
/// The test inventory a target declares and the resources it may consume.
fn validate_inventory(target: &Target) -> Result<()> {
    if target.tests.is_empty() || target.tests.len() > MAX_TESTS {
        return Err("test-inventory".into());
    }
    if target
        .tests
        .iter()
        .any(|n| n.is_empty() || n.starts_with('-') || n.len() > 512 || n.chars().any(char::is_control))
    {
        return Err("test-inventory".into());
    }
    if target.tests.iter().collect::<BTreeSet<_>>().len() != target.tests.len() {
        return Err("test-inventory".into());
    }
    if target.resources.timeout_ms == 0 || target.resources.timeout_ms > 3_600_000 {
        return Err("test-resource-bounds".into());
    }
    if target.resources.output_bytes == 0 || target.resources.output_bytes > MAX_CAPTURE {
        return Err("test-resource-bounds".into());
    }
    if target.resources.memory_bytes < 16_777_216 || target.resources.memory_bytes > 68_719_476_736 {
        return Err("test-resource-bounds".into());
    }
    debug_assert!(!target.tests.is_empty());
    debug_assert!(target.tests.len() <= MAX_TESTS);
    debug_assert!(target.resources.timeout_ms > 0);
    debug_assert!(target.resources.output_bytes > 0);
    debug_assert!(target.resources.memory_bytes >= 16_777_216);
    Ok(())
}
/// Environment options, runfiles and the optional coverage contract of a target.
fn validate_environment(target: &Target) -> Result<()> {
    if target.environment.len() > 64
        || target.environment.iter().any(|(k, v)| {
            k == "TEST_SEED"
                || k == "TEST_REPEAT"
                || !k.starts_with("TEST_")
                || !k.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
                || v.len() > 4096
                || v.contains('\0')
        })
    {
        return Err("test-environment-option-unsupported".into());
    }
    if target.runfiles.len() > 128
        || target.runfiles.iter().any(|(k, v)| !relative(k) || !target.inputs.contains_key(v))
    {
        return Err("test-runfiles-undeclared".into());
    }
    if let Some(c) = &target.coverage {
        if !tool(target, &c.profdata) || !tool(target, &c.cov) {
            return Err("test-coverage-contract".into());
        }
        if target.inputs.get(&c.source_root) != Some(&c.source_blake3) {
            return Err("test-coverage-contract".into());
        }
        if c.max_profile_bytes == 0 || c.max_profile_bytes > 67_108_864 {
            return Err("test-coverage-contract".into());
        }
        debug_assert!(target.inputs.get(&c.source_root) == Some(&c.source_blake3));
        debug_assert!(c.max_profile_bytes > 0);
        debug_assert!(c.max_profile_bytes <= 67_108_864);
    }
    debug_assert!(target.environment.len() <= 64);
    debug_assert!(target.runfiles.len() <= 128);
    Ok(())
}
pub fn select(target: Target, invocation: Invocation) -> Result<SelectedRequest> {
    validate(&target)?;
    if invocation.shard_count == 0 || invocation.shard_count > 256 {
        return Err("test-selection-bounds".into());
    }
    if invocation.shard_index >= invocation.shard_count {
        return Err("test-selection-bounds".into());
    }
    if invocation.repeat == 0 || invocation.repeat > 16 {
        return Err("test-selection-bounds".into());
    }
    if invocation.exact && invocation.filter.is_none() {
        return Err("test-filter-invalid".into());
    }
    if invocation.filter.as_ref().is_some_and(|s| s.len() > 512 || s.chars().any(char::is_control)) {
        return Err("test-filter-invalid".into());
    }
    if invocation.timeout_ms.is_some_and(|n| n == 0 || n > target.resources.timeout_ms) {
        return Err("test-timeout-exceeds-target".into());
    }
    if invocation
        .force_rerun
        .as_ref()
        .is_some_and(|n| n.is_empty() || n.len() > 128 || !n.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
    {
        return Err("test-rerun-token-invalid".into());
    }
    if invocation.coverage && target.coverage.is_none() {
        return Err("test-coverage-not-declared".into());
    }
    let mut names: Vec<_> = target.tests.iter().collect();
    names.sort();
    names.retain(|n| {
        invocation.filter.as_ref().is_none_or(|f| {
            if invocation.exact {
                n.as_str() == f.as_str()
            } else {
                n.contains(f)
            }
        })
    });
    let shard_count = invocation.shard_count;
    debug_assert!(shard_count != 0);
    let selected: Vec<_> = names
        .into_iter()
        .enumerate()
        .filter(|(i, _)| (*i as u32) % shard_count == invocation.shard_index)
        .map(|(_, n)| n.clone())
        .collect();
    if selected.is_empty() {
        return Err("test-selection-empty".into());
    }
    let identity = digest(&(TARGET_SCHEMA, &target, &invocation))?;
    debug_assert!(invocation.shard_count != 0);
    debug_assert!(invocation.shard_index < invocation.shard_count);
    debug_assert!(!selected.is_empty());
    Ok(SelectedRequest {
        target,
        invocation,
        selected,
        identity,
    })
}
pub fn admit_output(request: SelectedRequest, path: String) -> Result<SelectedRequest> {
    if !root(&path, StorePrefix(&request.target.store_prefix)) {
        return Err("test-output-not-store-root".into());
    }
    Ok(request)
}
/// A complete one-test harness transcript is required, even for assertion failures.
pub fn classify_capture(name: String, text: String, exit: Option<i32>) -> ClassifiedCapture {
    let outcome = classify_capture_inner(CaptureTranscript {
        name: &name,
        text: &text,
        exit,
    });
    ClassifiedCapture { name, outcome }
}
fn classify_capture_inner(capture: CaptureTranscript<'_>) -> Outcome {
    if capture.text.len() as u64 > MAX_CAPTURE || !capture.text.ends_with('\n') {
        return Outcome::InfrastructureFailure;
    }
    let Ok(lines) = parse_libtest_events(capture.text) else {
        return Outcome::InfrastructureFailure;
    };
    let values: core::result::Result<Vec<serde_json::Value>, _> =
        capture.text.lines().map(serde_json::from_str).collect();
    let Ok(values) = values else {
        return Outcome::InfrastructureFailure;
    };
    if values.len() != 4 || values[0]["test_count"].as_u64() != Some(1) {
        return Outcome::InfrastructureFailure;
    }
    let expected_failure = u64::from(capture.exit == Some(101));
    let expected_passed = 1u64.saturating_sub(expected_failure);
    let terminal = &values[3];
    if terminal["passed"].as_u64() != Some(expected_passed) {
        return Outcome::InfrastructureFailure;
    }
    if terminal["failed"].as_u64() != Some(expected_failure) {
        return Outcome::InfrastructureFailure;
    }
    if terminal["ignored"].as_u64() != Some(0) {
        return Outcome::InfrastructureFailure;
    }
    if terminal["measured"].as_u64() != Some(0) {
        return Outcome::InfrastructureFailure;
    }
    debug_assert!(values.len() == 4);
    debug_assert!(expected_failure <= 1);
    debug_assert!(expected_passed <= 1);
    classify_four(capture.name, &lines, capture.exit)
}
fn classify_four(name: &str, lines: &[HarnessLine], exit: Option<i32>) -> Outcome {
    let [
        HarnessLine::Suite { event: start },
        HarnessLine::TestStarted { name: begun },
        HarnessLine::Test { name: ended, event },
        HarnessLine::Suite { event: finish },
    ] = lines
    else {
        return Outcome::InfrastructureFailure;
    };
    if start != "started" || begun != name || ended != name {
        return Outcome::InfrastructureFailure;
    }
    match (event.as_str(), finish.as_str(), exit) {
        ("ok", "ok", Some(0)) => Outcome::Pass,
        ("failed", "failed", Some(101)) => Outcome::AssertionFailure,
        _ => Outcome::InfrastructureFailure,
    }
}
pub fn aggregate(attempts: Vec<Attempt>) -> AttemptSummary {
    let outcome = aggregate_inner(&attempts);
    AttemptSummary { attempts, outcome }
}
fn aggregate_inner(attempts: &[Attempt]) -> Outcome {
    for class in [
        Outcome::InfrastructureFailure,
        Outcome::Cancelled,
        Outcome::Timeout,
        Outcome::AssertionFailure,
    ] {
        if attempts.iter().any(|a| a.outcome == class) {
            return class;
        }
    }
    if attempts.is_empty() {
        Outcome::InfrastructureFailure
    } else {
        Outcome::Pass
    }
}
/// This is an additional policy restriction AFTER existing signed admission.
/// Transient teardown/deadline/worker-loss observations are never reusable.
pub fn cache_eligible(outcome: Outcome) -> bool {
    matches!(outcome, Outcome::Pass | Outcome::AssertionFailure)
}
pub fn summary(report: Report) -> ReportSummary {
    let text = format!(
        "{}: {:?}; selected={} attempts={} identity={}",
        report.target,
        report.outcome,
        report.selected.len(),
        report.attempts.len(),
        report.identity
    );
    ReportSummary { report, text }
}
