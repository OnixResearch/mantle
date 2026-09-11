//! Stable SpaceWasm test-fact evidence (version 1).
//!
//! The stable report admits exactly the facts that must reproduce across
//! identical derivations: the suite, the command identity, the selected
//! test inventory, and each test's outcome in canonical order. Durations,
//! compilation order, completion order, progress text, and elapsed times
//! are presentation fields and never enter the stable identity. Exact raw
//! run output is retained separately by the shell as run evidence and is
//! bound to, but never part of, the stable bundle identity.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blake3Digest;
use crate::Diagnostic;
use crate::diagnostic::ErrorDiagnostic;
use crate::diagnostic::error;
use crate::diagnostic::ordered;
use crate::digest::canonical_identity;
use crate::digest::count_exceeds;

/// Stable report schema identifier.
pub const STABLE_REPORT_SCHEMA: &str = "mantle-spacewasm-stable-report-v1";

/// Structured encoding version of the admitted libtest JSON grammar.
pub const STABLE_REPORT_ENCODING_VERSION: u32 = 1;

/// Maximum admitted test records in one report.
pub const MAX_STABLE_TESTS: u32 = 4_096;

/// Maximum admitted raw harness lines in one capture.
pub const MAX_HARNESS_LINES: u32 = 8_192;

/// Maximum admitted test-name bytes.
pub const MAX_TEST_NAME_BYTES: usize = 1_024;

/// Maximum admitted per-test captured stdout bytes.
pub const MAX_TEST_STDOUT_BYTES: usize = 65_536;

/// Maximum admitted suite wall-clock seconds.
pub const MAX_SUITE_SECONDS: f64 = 86_400.0;

/// Admitted test outcome classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StableTestStatus {
    Passed,
    Failed,
    Skipped,
}

/// One admitted test fact: identity plus outcome, nothing else.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StableTestRecord {
    pub name: String,
    pub status: StableTestStatus,
}

/// One line of the closed libtest JSON harness grammar (version 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessLine {
    /// `{"type":"test","name":...,"event":"ok"|"failed"|"ignored"}`
    Test { name: String, event: String },
    /// `{"type":"test","event":"started","name":...}` (carries no fact)
    TestStarted { name: String },
    /// `{"type":"suite","event":"started"|"ok"|"failed"}`
    Suite { event: String },
}

/// Admission request over parsed harness facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StableReportRequest {
    /// Suite identity, for example `upstream-unit-tests`.
    pub suite: String,
    /// Exact command identity that produced the capture.
    pub command: String,
    /// Required test names; every one must appear exactly once.
    pub expected_tests: Vec<String>,
    /// Parsed harness lines in capture order.
    pub lines: Vec<HarnessLine>,
}

/// Admitted stable report with its canonical identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StableReport {
    pub schema: String,
    pub suite: String,
    pub command: String,
    /// Required test facts in canonical (name-ordered) sequence.
    pub tests: Vec<StableTestRecord>,
    pub encoding_version: u32,
    pub stable_identity_blake3: Blake3Digest,
}

/// Admission outcome: a report or ordered error diagnostics, never both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StableReportResult {
    pub report: Option<StableReport>,
    pub diagnostics: Vec<Diagnostic>,
}

/// One admitted harness JSON line with `deny_unknown_fields`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LibtestJsonLine {
    #[serde(rename = "type")]
    line_type: String,
    name: Option<String>,
    event: Option<String>,
    test_count: Option<u64>,
    passed: Option<u64>,
    failed: Option<u64>,
    ignored: Option<u64>,
    measured: Option<u64>,
    filtered_out: Option<u64>,
    exec_time: Option<f64>,
    stdout: Option<String>,
}

/// Parse the closed libtest JSON grammar from raw harness text.
///
/// One JSON object per line. Unknown types, unknown events, missing or
/// unknown fields, and truncated final lines are rejections, never silent
/// omissions. Line count is bounded.
pub fn parse_libtest_events(text: &str) -> Result<Vec<HarnessLine>, Vec<Diagnostic>> {
    let mut lines: Vec<HarnessLine> = Vec::with_capacity(usize::try_from(MAX_HARNESS_LINES).unwrap_or(0));
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<LibtestJsonLine>(line) {
            Ok(parsed) => match parsed.into_line() {
                Some(admitted) => {
                    if count_exceeds(lines.len(), MAX_HARNESS_LINES) {
                        return Err(ordered(vec![error(ErrorDiagnostic {
                            code: "stable-report-line-bound",
                            subject: "harness-capture",
                            message: "harness capture exceeds the fixed line bound",
                        })]));
                    }
                    lines.push(admitted);
                }
                None => return unknown_line(),
            },
            Err(_) => return unknown_line(),
        }
    }
    if lines.is_empty() {
        return Err(ordered(vec![error(ErrorDiagnostic {
            code: "stable-report-empty-capture",
            subject: "harness-capture",
            message: "harness capture contains no admitted harness lines",
        })]));
    }
    debug_assert!(lines.len() <= usize::try_from(MAX_HARNESS_LINES).unwrap_or(0));
    Ok(lines)
}

fn unknown_line() -> Result<Vec<HarnessLine>, Vec<Diagnostic>> {
    Err(ordered(vec![error(ErrorDiagnostic {
        code: "stable-report-unknown-line",
        subject: "harness-capture",
        message: "harness capture contains a line outside the versioned grammar",
    })]))
}

impl LibtestJsonLine {
    /// Reject implausible summary counts before they enter the grammar.
    fn counts_within_bound(&self) -> bool {
        let total = self
            .test_count
            .unwrap_or(0)
            .saturating_add(self.passed.unwrap_or(0))
            .saturating_add(self.failed.unwrap_or(0))
            .saturating_add(self.ignored.unwrap_or(0))
            .saturating_add(self.measured.unwrap_or(0))
            .saturating_add(self.filtered_out.unwrap_or(0));
        let bound = u64::from(MAX_STABLE_TESTS).saturating_mul(2);
        let is_exec_time_seconds_within_bounds = self
            .exec_time
            .is_none_or(|seconds| seconds.is_finite() && (0.0..=MAX_SUITE_SECONDS).contains(&seconds));
        let is_stdout_bounded = self.stdout.as_ref().is_none_or(|captured| captured.len() <= MAX_TEST_STDOUT_BYTES);
        debug_assert!(bound >= u64::from(MAX_STABLE_TESTS));
        total <= bound && is_exec_time_seconds_within_bounds && is_stdout_bounded
    }

    fn into_line(self) -> Option<HarnessLine> {
        let has_plausible_counts = self.counts_within_bound();
        match (self.line_type.as_str(), self.name, self.event) {
            ("test", Some(name), Some(event)) if event == "ok" || event == "failed" || event == "ignored" => {
                debug_assert!(!name.is_empty());
                Some(HarnessLine::Test { name, event })
            }
            ("test", Some(name), Some(event)) if event == "started" => {
                debug_assert!(!name.is_empty());
                Some(HarnessLine::TestStarted { name })
            }
            ("suite", None, Some(event))
                if (event == "started" || event == "ok" || event == "failed") && has_plausible_counts =>
            {
                debug_assert!(!event.is_empty());
                Some(HarnessLine::Suite { event })
            }
            _ => None,
        }
    }
}

/// Admit a stable report from parsed harness lines.
///
/// Rejections cover missing expected tests, duplicate test records,
/// malformed or over-long names, contradictory suite outcomes, and
/// over-limit counts. Duplicate records are errors, not entries to
/// deduplicate.
pub fn admit_stable_report(request: &StableReportRequest) -> StableReportResult {
    let mut diagnostics = Vec::new();
    let blocker_count_before = diagnostics.len();
    if request.suite.is_empty() {
        diagnostics.push(error(ErrorDiagnostic {
            code: "stable-report-empty-suite",
            subject: "suite",
            message: "stable report suite identity must not be empty",
        }));
    }
    if request.command.is_empty() {
        diagnostics.push(error(ErrorDiagnostic {
            code: "stable-report-empty-command",
            subject: "command",
            message: "stable report command identity must not be empty",
        }));
    }
    if request.lines.is_empty() {
        diagnostics.push(error(ErrorDiagnostic {
            code: "stable-report-no-tests",
            subject: "harness-capture",
            message: "stable report requires at least one admitted harness line",
        }));
    }
    debug_assert!(diagnostics.len() >= blocker_count_before);
    debug_assert!(diagnostics.iter().all(|item| !item.code.is_empty() && !item.message.is_empty()));
    if !diagnostics.is_empty() {
        return rejected(ordered(diagnostics));
    }
    admit_tests(request)
}

/// Suite summary facts collected from one capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SuiteSummary {
    failed_seen: bool,
    ok_seen: bool,
    summary_line_count: usize,
}

impl SuiteSummary {
    fn empty() -> Self {
        Self {
            failed_seen: false,
            ok_seen: false,
            summary_line_count: 0,
        }
    }
}

/// Collect admitted test records and suite summary facts.
fn collect_admitted_tests(request: &StableReportRequest) -> (Vec<StableTestRecord>, SuiteSummary, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let mut tests: Vec<StableTestRecord> = Vec::with_capacity(request.lines.len());
    let mut summary = SuiteSummary::empty();
    let mut has_duplicate_summary = false;
    let mut has_test_bound_problem = false;
    let mut has_name_problem = false;
    for line in &request.lines {
        match line {
            HarnessLine::Suite { event } => {
                if event == "started" {
                    continue;
                }
                summary.summary_line_count += 1;
                if count_exceeds(summary.summary_line_count, 1) {
                    has_duplicate_summary = true;
                } else if event == "failed" {
                    summary.failed_seen = true;
                } else {
                    summary.ok_seen = true;
                }
            }
            HarnessLine::TestStarted { .. } => {}
            HarnessLine::Test { name, event } => {
                if count_exceeds(tests.len(), MAX_STABLE_TESTS) {
                    has_test_bound_problem = true;
                    break;
                }
                if name.is_empty() || name.len() > MAX_TEST_NAME_BYTES {
                    has_name_problem = true;
                    continue;
                }
                let status = match event.as_str() {
                    "ok" => StableTestStatus::Passed,
                    "failed" => StableTestStatus::Failed,
                    _ => StableTestStatus::Skipped,
                };
                tests.push(StableTestRecord {
                    name: name.clone(),
                    status,
                });
            }
        }
    }
    debug_assert!(tests.len() <= request.lines.len());
    if has_duplicate_summary {
        diagnostics.push(error(ErrorDiagnostic {
            code: "stable-report-duplicate-suite-summary",
            subject: "suite",
            message: "harness capture admits exactly one suite summary line",
        }));
    }
    if has_test_bound_problem {
        diagnostics.push(error(ErrorDiagnostic {
            code: "stable-report-test-bound",
            subject: "tests",
            message: "stable report exceeds the fixed test-record bound",
        }));
    }
    if has_name_problem {
        diagnostics.push(error(ErrorDiagnostic {
            code: "stable-report-test-name",
            subject: "tests",
            message: "test names must be non-empty and within the named bound",
        }));
    }
    tests.sort();
    (tests, summary, diagnostics)
}

/// Judge collected facts against the required inventory and summary.
fn judge_admitted_tests(
    request: &StableReportRequest,
    tests: &[StableTestRecord],
    summary: SuiteSummary,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let blocker_count_before = diagnostics.len();
    if tests.is_empty() {
        diagnostics.push(error(ErrorDiagnostic {
            code: "stable-report-no-tests",
            subject: "tests",
            message: "stable report requires at least one admitted test record",
        }));
    }
    if tests.windows(2).any(|pair| pair[0] == pair[1]) {
        diagnostics.push(error(ErrorDiagnostic {
            code: "stable-report-duplicate-test",
            subject: "tests",
            message: "duplicate test records are errors and are never deduplicated",
        }));
    }
    for expected in &request.expected_tests {
        let is_expected_present = tests.iter().any(|record| &record.name == expected);
        if !is_expected_present {
            diagnostics.push(error(ErrorDiagnostic {
                code: "stable-report-missing-expected-test",
                subject: expected.as_str(),
                message: "required test inventory member is missing from the capture",
            }));
        }
    }
    let has_failed_test = tests.iter().any(|record| record.status == StableTestStatus::Failed);
    let has_admitted_test = !tests.is_empty();
    let is_summary_contradictory = (summary.ok_seen && has_failed_test)
        || (summary.failed_seen && has_admitted_test && !has_failed_test)
        || (summary.summary_line_count == 0 && has_admitted_test);
    if is_summary_contradictory {
        diagnostics.push(error(ErrorDiagnostic {
            code: "stable-report-contradictory-summary",
            subject: "suite",
            message: "suite summary is missing from the capture or contradicts admitted test outcomes",
        }));
    }
    debug_assert!(diagnostics.len() >= blocker_count_before);
    debug_assert!(diagnostics.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn admit_tests(request: &StableReportRequest) -> StableReportResult {
    let (tests, summary, mut diagnostics) = collect_admitted_tests(request);
    judge_admitted_tests(request, &tests, summary, &mut diagnostics);
    if !diagnostics.is_empty() {
        return rejected(ordered(diagnostics));
    }
    debug_assert!(!tests.is_empty());
    debug_assert!(summary.summary_line_count == 1);
    build_stable_report(request, tests)
}

fn build_stable_report(request: &StableReportRequest, tests: Vec<StableTestRecord>) -> StableReportResult {
    let identity_input = StableReportIdentityInput {
        schema: String::from(STABLE_REPORT_SCHEMA),
        suite: request.suite.clone(),
        command: request.command.clone(),
        tests: tests.clone(),
        encoding_version: STABLE_REPORT_ENCODING_VERSION,
    };
    let identity = match canonical_identity(&identity_input) {
        Ok(identity) => identity,
        Err(_) => {
            return rejected(ordered(vec![error(ErrorDiagnostic {
                code: "stable-report-identity-failed",
                subject: "stable-report",
                message: "stable report could not be canonically identified",
            })]));
        }
    };
    debug_assert!(!identity.as_str().is_empty());
    StableReportResult {
        report: Some(StableReport {
            schema: String::from(STABLE_REPORT_SCHEMA),
            suite: request.suite.clone(),
            command: request.command.clone(),
            tests,
            encoding_version: STABLE_REPORT_ENCODING_VERSION,
            stable_identity_blake3: identity,
        }),
        diagnostics: Vec::new(),
    }
}

#[derive(Debug, Clone, Serialize)]
struct StableReportIdentityInput {
    schema: String,
    suite: String,
    command: String,
    tests: Vec<StableTestRecord>,
    encoding_version: u32,
}

fn rejected(diagnostics: Vec<Diagnostic>) -> StableReportResult {
    debug_assert!(!diagnostics.is_empty());
    StableReportResult {
        report: None,
        diagnostics,
    }
}
