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
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    event: Option<String>,
    #[serde(default)]
    test_count: Option<u64>,
    #[serde(default)]
    passed: Option<u64>,
    #[serde(default)]
    failed: Option<u64>,
    #[serde(default)]
    ignored: Option<u64>,
    #[serde(default)]
    measured: Option<u64>,
    #[serde(default)]
    filtered_out: Option<u64>,
    #[serde(default)]
    exec_time: Option<f64>,
    #[serde(default)]
    stdout: Option<String>,
}

/// Parse the closed libtest JSON grammar from raw harness text.
///
/// One JSON object per line. Unknown types, unknown events, missing or
/// unknown fields, and truncated final lines are rejections, never silent
/// omissions. Line count is bounded.
pub fn parse_libtest_events(text: &str) -> Result<Vec<HarnessLine>, Vec<Diagnostic>> {
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if count_exceeds(lines.len(), MAX_HARNESS_LINES) {
            return Err(ordered(vec![error(ErrorDiagnostic {
                code: "stable-report-line-bound",
                subject: "harness-capture",
                message: "harness capture exceeds the fixed line bound",
            })]));
        }
        match serde_json::from_str::<LibtestJsonLine>(line) {
            Ok(parsed) => match parsed.into_line() {
                Some(line) => lines.push(line),
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
    debug_assert!(lines.len() <= MAX_HARNESS_LINES as usize);
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
        let duration_is_plausible = self
            .exec_time
            .is_none_or(|seconds| seconds.is_finite() && (0.0..=MAX_SUITE_SECONDS).contains(&seconds));
        let stdout_is_bounded = self.stdout.as_ref().is_none_or(|captured| captured.len() <= MAX_TEST_STDOUT_BYTES);
        debug_assert!(bound >= u64::from(MAX_STABLE_TESTS));
        total <= bound && duration_is_plausible && stdout_is_bounded
    }

    fn into_line(self) -> Option<HarnessLine> {
        let counts_are_plausible = self.counts_within_bound();
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
                if (event == "started" || event == "ok" || event == "failed") && counts_are_plausible =>
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
    if !diagnostics.is_empty() {
        return rejected(ordered(diagnostics));
    }
    admit_tests(request)
}

fn admit_tests(request: &StableReportRequest) -> StableReportResult {
    let mut diagnostics = Vec::new();
    let mut tests: Vec<StableTestRecord> = Vec::new();
    let mut suite_failed_seen = false;
    let mut suite_ok_seen = false;
    let mut suite_summary_lines: usize = 0;
    for line in &request.lines {
        match line {
            HarnessLine::Suite { event } => {
                if event == "started" {
                    continue;
                }
                suite_summary_lines += 1;
                if count_exceeds(suite_summary_lines, 1) {
                    diagnostics.push(error(ErrorDiagnostic {
                        code: "stable-report-duplicate-suite-summary",
                        subject: "suite",
                        message: "harness capture admits exactly one suite summary line",
                    }));
                } else if event == "failed" {
                    suite_failed_seen = true;
                } else {
                    suite_ok_seen = true;
                }
            }
            HarnessLine::TestStarted { .. } => {}
            HarnessLine::Test { name, event } => {
                if count_exceeds(tests.len(), MAX_STABLE_TESTS) {
                    diagnostics.push(error(ErrorDiagnostic {
                        code: "stable-report-test-bound",
                        subject: "tests",
                        message: "stable report exceeds the fixed test-record bound",
                    }));
                    break;
                }
                if name.is_empty() || name.len() > MAX_TEST_NAME_BYTES {
                    diagnostics.push(error(ErrorDiagnostic {
                        code: "stable-report-test-name",
                        subject: "tests",
                        message: "test names must be non-empty and within the named bound",
                    }));
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
    tests.sort();
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
        let present = tests.iter().any(|record| &record.name == expected);
        if !present {
            diagnostics.push(error(ErrorDiagnostic {
                code: "stable-report-missing-expected-test",
                subject: expected.as_str(),
                message: "required test inventory member is missing from the capture",
            }));
        }
    }
    let any_failed = tests.iter().any(|record| record.status == StableTestStatus::Failed);
    let any_admitted_test = !tests.is_empty();
    let contradictory = (suite_ok_seen && any_failed)
        || (suite_failed_seen && any_admitted_test && !any_failed)
        || (suite_summary_lines == 0 && any_admitted_test);
    if contradictory {
        diagnostics.push(error(ErrorDiagnostic {
            code: "stable-report-contradictory-summary",
            subject: "suite",
            message: "suite summary is missing from the capture or contradicts admitted test outcomes",
        }));
    }
    if !diagnostics.is_empty() {
        return rejected(ordered(diagnostics));
    }
    debug_assert!(!tests.is_empty());
    debug_assert!(suite_summary_lines == 1);
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
