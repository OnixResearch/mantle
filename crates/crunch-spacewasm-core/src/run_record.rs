//! Run records for SpaceWasm report producers (version 1).
//!
//! A run record is the one-way bridge between exact raw run evidence and the
//! stable bundle identity: it binds the stable report identity to the exact
//! stdout/stderr captures and process outcome of one execution. The stable
//! bundle never refers back to a run record or to raw capture identities, so
//! nondeterminism cannot reenter through a parent edge.

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

/// Run record schema identifier.
pub const RUN_RECORD_SCHEMA: &str = "mantle-spacewasm-run-record-v1";

/// Run record encoding version.
pub const RUN_RECORD_ENCODING_VERSION: u32 = 1;

/// Maximum admitted raw captures per run record.
pub const MAX_RUN_CAPTURES: u32 = 8;

/// Maximum admitted capture role bytes.
pub const MAX_CAPTURE_ROLE_BYTES: usize = 128;

/// Admitted producer process outcomes.
pub const ADMITTED_RUN_STATUSES: [&str; 6] = ["passed", "failed", "signal", "timeout", "cancelled", "unavailable"];

/// One exact raw capture bound by a run record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawCapture {
    pub role: String,
    pub blake3: Blake3Digest,
    pub size_bytes: u64,
}

/// Admission request for one run record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRecordRequest {
    /// Stable report identity produced by this exact run.
    pub stable_identity_blake3: Blake3Digest,
    pub suite: String,
    pub command: String,
    /// Process outcome class from `ADMITTED_RUN_STATUSES`.
    pub process_status: String,
    /// Exit code when the process completed normally.
    pub exit_code: Option<i32>,
    /// Termination signal when the process was signaled.
    pub signal: Option<u32>,
    /// Exact raw captures retained for this run.
    pub captures: Vec<RawCapture>,
}

/// Admitted run record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunRecord {
    pub schema: String,
    pub stable_identity_blake3: Blake3Digest,
    pub suite: String,
    pub command: String,
    pub process_status: String,
    pub exit_code: Option<i32>,
    pub signal: Option<u32>,
    pub captures: Vec<RawCapture>,
    pub encoding_version: u32,
    pub run_record_identity_blake3: Blake3Digest,
}

/// Admission outcome: a run record or ordered error diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRecordResult {
    pub record: Option<RunRecord>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Admit a run record over exact capture facts.
///
/// A missing stable identity is unrepresentable: `Blake3Digest` admits only
/// exact 64-character lowercase hex values, so admission covers the
/// remaining fact space only.
///
/// Rejections cover empty suite and command, unknown statuses, contradictory
/// exit/signal facts, duplicate or over-limit captures, and capture-role
/// bound violations.
pub fn admit_run_record(request: &RunRecordRequest) -> RunRecordResult {
    let mut diagnostics = Vec::new();
    if request.suite.is_empty() {
        diagnostics.push(error(ErrorDiagnostic {
            code: "run-record-empty-suite",
            subject: "suite",
            message: "run record suite identity must not be empty",
        }));
    }
    if request.command.is_empty() {
        diagnostics.push(error(ErrorDiagnostic {
            code: "run-record-empty-command",
            subject: "command",
            message: "run record command identity must not be empty",
        }));
    }
    if !ADMITTED_RUN_STATUSES.contains(&request.process_status.as_str()) {
        diagnostics.push(error(ErrorDiagnostic {
            code: "run-record-unknown-status",
            subject: "process-status",
            message: "run record process status is outside the admitted set",
        }));
    }
    let declares_signal = request.signal.is_some();
    if declares_signal && request.process_status != "signal" {
        diagnostics.push(error(ErrorDiagnostic {
            code: "run-record-status-contradiction",
            subject: "process-status",
            message: "a signaled run must declare the signal status",
        }));
    }
    if request.process_status == "signal" && !declares_signal {
        diagnostics.push(error(ErrorDiagnostic {
            code: "run-record-status-contradiction",
            subject: "process-status",
            message: "the signal status requires a termination signal",
        }));
    }
    if request.process_status == "passed" && request.exit_code.is_some_and(|code| code != 0) {
        diagnostics.push(error(ErrorDiagnostic {
            code: "run-record-status-contradiction",
            subject: "exit-code",
            message: "a passing run must not carry a nonzero exit code",
        }));
    }
    validate_captures(request, &mut diagnostics);
    if !diagnostics.is_empty() {
        return rejected(ordered(diagnostics));
    }
    let mut captures = request.captures.clone();
    captures.sort_by(|left, right| left.role.cmp(&right.role));
    debug_assert!(captures.windows(2).all(|pair| pair[0].role != pair[1].role));
    let identity = canonical_identity(RunRecordIdentityInput {
        schema: String::from(RUN_RECORD_SCHEMA),
        stable_identity_blake3: request.stable_identity_blake3.clone(),
        suite: request.suite.clone(),
        command: request.command.clone(),
        process_status: request.process_status.clone(),
        captures: captures.clone(),
        encoding_version: RUN_RECORD_ENCODING_VERSION,
    });
    let identity = match identity {
        Ok(identity) => identity,
        Err(_) => {
            return rejected(ordered(vec![error(ErrorDiagnostic {
                code: "run-record-identity-failed",
                subject: "run-record",
                message: "run record could not be canonically identified",
            })]));
        }
    };
    debug_assert!(!identity.as_str().is_empty());
    RunRecordResult {
        record: Some(RunRecord {
            schema: String::from(RUN_RECORD_SCHEMA),
            stable_identity_blake3: request.stable_identity_blake3.clone(),
            suite: request.suite.clone(),
            command: request.command.clone(),
            process_status: request.process_status.clone(),
            exit_code: request.exit_code,
            signal: request.signal,
            captures,
            encoding_version: RUN_RECORD_ENCODING_VERSION,
            run_record_identity_blake3: identity,
        }),
        diagnostics: Vec::new(),
    }
}

fn validate_captures(request: &RunRecordRequest, diagnostics: &mut Vec<Diagnostic>) {
    if count_exceeds(request.captures.len(), MAX_RUN_CAPTURES) {
        diagnostics.push(error(ErrorDiagnostic {
            code: "run-record-capture-bound",
            subject: "captures",
            message: "run record exceeds the fixed capture bound",
        }));
        return;
    }
    let mut roles: Vec<&str> = Vec::new();
    for capture in &request.captures {
        if capture.role.is_empty() || capture.role.len() > MAX_CAPTURE_ROLE_BYTES {
            diagnostics.push(error(ErrorDiagnostic {
                code: "run-record-capture-role",
                subject: "captures",
                message: "capture roles must be non-empty and within the named bound",
            }));
            continue;
        }
        if roles.contains(&capture.role.as_str()) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "run-record-duplicate-capture",
                subject: capture.role.as_str(),
                message: "duplicate capture roles are errors and are never deduplicated",
            }));
            continue;
        }
        roles.push(capture.role.as_str());
    }
    debug_assert!(roles.len() <= request.captures.len());
}

/// Reject an identity cycle: no stable bundle member path may be the run
/// record path, and no stable member may carry a raw capture role.
pub fn classify_identity_cycle(
    stable_member_paths: &[String],
    run_record_path: &str,
    raw_capture_roles: &[String],
) -> Result<(), Diagnostic> {
    for member in stable_member_paths {
        if member == run_record_path {
            return Err(error(ErrorDiagnostic {
                code: "identity-cycle-run-record-member",
                subject: member.as_str(),
                message: "the run record must not be a stable bundle member",
            }));
        }
        for role in raw_capture_roles {
            if member.ends_with(role.as_str()) {
                return Err(error(ErrorDiagnostic {
                    code: "identity-cycle-raw-capture-member",
                    subject: member.as_str(),
                    message: "raw captures must not be stable bundle members",
                }));
            }
        }
    }
    debug_assert!(!stable_member_paths.is_empty() || run_record_path.is_empty());
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
struct RunRecordIdentityInput {
    schema: String,
    stable_identity_blake3: Blake3Digest,
    suite: String,
    command: String,
    process_status: String,
    captures: Vec<RawCapture>,
    encoding_version: u32,
}

fn rejected(diagnostics: Vec<Diagnostic>) -> RunRecordResult {
    debug_assert!(!diagnostics.is_empty());
    RunRecordResult {
        record: None,
        diagnostics,
    }
}
