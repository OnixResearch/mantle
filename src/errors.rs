//! Error types for the crunch CLI.
//!
//! `RunError` classifies all failures into eval, build, or internal errors.
//! Each variant maps to a fixed exit code and produces actionable output.

// machine-artifact-public: cli.error-envelope
use std::fmt;
use std::process::ExitCode;

use crate::operator_contract::FailureFacts;
use crate::operator_contract::RemediationRecord;
use crate::operator_contract::classify_remediation;
use crate::operator_contract::render_remediation_human;
use crate::operator_contract::render_remediation_json;

const REPORTED_EXIT_CODE_KIND: &str = "reported";

/// Top-level error from running a crunch command.
#[derive(Debug)]
pub enum RunError {
    /// Nickel evaluation failed (parse, typecheck, contract violation).
    Eval(String),
    /// Build execution failed (sandbox, builder script, hash mismatch).
    Build(String),
    /// Internal error (IO, configuration, missing tools).
    Internal(String),
    /// The command already emitted its own structured output.
    Reported(u8),
}

impl RunError {
    /// Exit code for this error class.
    ///
    /// - `1` build failure
    /// - `2` evaluation error
    /// - `3` internal error
    pub fn exit_code(&self) -> ExitCode {
        match self {
            RunError::Eval(_) => ExitCode::from(2),
            RunError::Build(_) => ExitCode::from(1),
            RunError::Internal(_) => ExitCode::from(3),
            RunError::Reported(code) => ExitCode::from(*code),
        }
    }

    /// Short label for the error category.
    pub fn kind(&self) -> &'static str {
        match self {
            RunError::Eval(_) => "eval",
            RunError::Build(_) => "build",
            RunError::Internal(_) => "internal",
            RunError::Reported(_) => REPORTED_EXIT_CODE_KIND,
        }
    }

    /// The inner message.
    pub fn message(&self) -> &str {
        match self {
            RunError::Eval(msg) | RunError::Build(msg) | RunError::Internal(msg) => msg,
            RunError::Reported(_) => "",
        }
    }

    /// Format as a human-readable error string for stderr.
    pub fn format_human(&self) -> String {
        let base = match self {
            RunError::Eval(msg) => format!("error: evaluation failed\n{msg}"),
            RunError::Build(msg) => format!("error: build failed\n{}", extract_build_body(msg)),
            RunError::Internal(msg) => format!("error: {msg}"),
            RunError::Reported(_) => return String::new(),
        };
        match self.remediation() {
            Some(record) => format!("{base}\n\n{}", render_remediation_human(&record)),
            None => base,
        }
    }

    /// Format as a JSON object for `--json` mode.
    pub fn format_json(&self) -> String {
        if matches!(self, RunError::Reported(_)) {
            return String::new();
        }

        let code: u8 = match self {
            RunError::Eval(_) => 2,
            RunError::Build(_) => 1,
            RunError::Internal(_) => 3,
            RunError::Reported(_) => 4,
        };
        let Some(record) = self.remediation() else {
            let escaped = self
                .message()
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t");
            return format!(r#"{{"error":"{}","code":{},"kind":"{}"}}"#, escaped, code, self.kind());
        };
        let Some(remediation_value) = remediation_json_value(&record) else {
            return serialization_failure_envelope(code, self.kind());
        };
        let value = serde_json::json!({
            "error": self.message(),
            "code": code,
            "kind": self.kind(),
            "remediation": remediation_value,
        });
        match serde_json::to_string(&value) {
            Ok(encoded) => encoded,
            Err(_) => serialization_failure_envelope(code, self.kind()),
        }
    }

    fn remediation(&self) -> Option<RemediationRecord> {
        if matches!(self, RunError::Reported(_)) {
            return None;
        }
        classify_remediation(FailureFacts {
            kind: self.kind(),
            message: self.message(),
            safe_subject: None,
            remote_route_eligible: false,
        })
    }
}

fn remediation_json_value(record: &RemediationRecord) -> Option<serde_json::Value> {
    let encoded = render_remediation_json(record).ok()?;
    serde_json::from_str(&encoded).ok()
}

fn serialization_failure_envelope(code: u8, kind: &str) -> String {
    debug_assert!(!kind.is_empty());
    debug_assert!(kind.chars().all(|value| value.is_ascii_lowercase()));
    format!(r#"{{"error":"error envelope serialization failed","code":{code},"kind":"{kind}"}}"#)
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_human())
    }
}

impl From<crunch_pipeline::Error> for RunError {
    fn from(error: crunch_pipeline::Error) -> Self {
        match error {
            crunch_pipeline::Error::Eval(msg) => Self::Eval(msg),
            crunch_pipeline::Error::Deserialize(msg) => Self::Eval(msg),
            crunch_pipeline::Error::Convert(msg) => Self::Build(msg),
            crunch_pipeline::Error::Build(msg) => Self::Build(msg),
            crunch_pipeline::Error::Internal(msg) => Self::Internal(msg),
        }
    }
}

/// Extract the builder-relevant portion from a build error message.
///
/// The bwrap service wraps build failures in generic messages like
/// "nonzero exit code". When we detect that pattern, we point the user
/// at the build log directory instead of showing a useless message.
fn extract_build_body(msg: &str) -> &str {
    // The orchestrator wraps the bwrap error as "<drv-name>: <io-error>".
    // If the io-error is just "nonzero exit code", that's unhelpful.
    // Return it as-is (the suggestions will add guidance).
    msg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes() {
        assert_eq!(RunError::Build("x".into()).exit_code(), ExitCode::from(1));
        assert_eq!(RunError::Eval("x".into()).exit_code(), ExitCode::from(2));
        assert_eq!(RunError::Internal("x".into()).exit_code(), ExitCode::from(3));
        assert_eq!(RunError::Reported(4).exit_code(), ExitCode::from(4));
    }

    #[test]
    fn kind_labels() {
        assert_eq!(RunError::Eval("".into()).kind(), "eval");
        assert_eq!(RunError::Build("".into()).kind(), "build");
        assert_eq!(RunError::Internal("".into()).kind(), "internal");
        assert_eq!(RunError::Reported(4).kind(), "reported");
    }

    #[test]
    fn human_format_eval() {
        let e = RunError::Eval("type error at line 5".into());
        let s = e.format_human();
        assert!(s.starts_with("error: evaluation failed\n"));
        assert!(s.contains("type error at line 5"));
    }

    #[test]
    fn human_format_build() {
        let e = RunError::Build("exit code 2".into());
        let s = e.format_human();
        assert!(s.starts_with("error: build failed\n"));
        assert!(s.contains("exit code 2"));
    }

    #[test]
    fn human_format_internal() {
        let e = RunError::Internal("store dir missing".into());
        let s = e.format_human();
        assert_eq!(s, "error: store dir missing");
    }

    #[test]
    fn json_format_eval() {
        let e = RunError::Eval("bad type".into());
        let j = e.format_json();
        assert_eq!(j, r#"{"error":"bad type","code":2,"kind":"eval"}"#);
    }

    #[test]
    fn json_format_includes_structured_remediation_without_panicking() {
        let error = RunError::Build("source input not found".into());

        let encoded = error.format_json();
        let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();

        assert_eq!(value["kind"], "build");
        assert_eq!(value["remediation"]["code"], "mantle.source.missing-input");
    }

    #[test]
    fn remediation_json_preserves_legacy_composition_bytes() {
        let error = RunError::Build("source input not found".into());
        let record = error.remediation().unwrap();
        let remediation_json = crate::operator_contract::render_remediation_json(&record).unwrap();
        let remediation_value: serde_json::Value = serde_json::from_str(&remediation_json).unwrap();
        let legacy_value = serde_json::json!({
            "error": error.message(),
            "code": 1,
            "kind": error.kind(),
            "remediation": remediation_value,
        });
        let legacy_encoded = serde_json::to_string(&legacy_value).unwrap();

        assert_eq!(error.format_json(), legacy_encoded);
        assert!(legacy_encoded.contains("mantle.source.missing-input"));
    }

    #[test]
    fn json_format_build() {
        let e = RunError::Build("sandbox crashed".into());
        let j = e.format_json();
        assert_eq!(j, r#"{"error":"sandbox crashed","code":1,"kind":"build"}"#);
    }

    #[test]
    fn json_format_internal() {
        let e = RunError::Internal("no store".into());
        let j = e.format_json();
        assert_eq!(j, r#"{"error":"no store","code":3,"kind":"internal"}"#);
    }

    #[test]
    fn reported_format_is_empty() {
        let e = RunError::Reported(4);
        assert!(e.format_human().is_empty());
        assert!(e.format_json().is_empty());
        assert_eq!(e.message(), "");
    }

    #[test]
    fn json_escapes_special_chars() {
        let e = RunError::Build("line1\nline2\ttab \"quoted\"".into());
        let j = e.format_json();
        assert!(j.contains(r#"\n"#));
        assert!(j.contains(r#"\t"#));
        assert!(j.contains(r#"\""#));
        // Verify it's parseable JSON
        assert!(j.starts_with('{'));
        assert!(j.ends_with('}'));
    }

    #[test]
    fn display_matches_human_format() {
        let e = RunError::Eval("parse error".into());
        assert_eq!(e.to_string(), e.format_human());
    }

    #[test]
    fn message_accessor() {
        let e = RunError::Build("failed".into());
        assert_eq!(e.message(), "failed");
    }

    // -- Phase 2: build failure message improvement --

    #[test]
    fn build_nonzero_exit_suggests_logs() {
        let e = RunError::Build("my-pkg: nonzero exit code".into());
        let s = e.format_human();
        assert!(s.contains("mantle log"), "should name the canonical log command: {s}");
        assert!(s.contains("mutation: none"), "should state mutation behavior: {s}");
    }

    #[test]
    fn build_source_not_found_suggests_source_plan() {
        let e = RunError::Build("source input not found in store: /nix/store/xxx-bash".into());
        let s = e.format_human();
        assert!(s.contains("mantle source bundle plan"), "should name the canonical source command: {s}");
        assert!(s.contains("network: none"), "should state network behavior: {s}");
    }

    #[test]
    fn build_fod_mismatch_suggests_update() {
        let e = RunError::Build("FOD hash mismatch for foo: expected abc, got def".into());
        let s = e.format_human();
        assert!(s.contains("Update the hash"), "should suggest hash update: {s}");
    }

    #[test]
    fn build_not_linux_suggests_doctor() {
        let e = RunError::Build("building is only supported on Linux".into());
        let s = e.format_human();
        assert!(s.contains("mantle doctor"), "should name the canonical preflight command: {s}");
    }

    #[test]
    fn build_output_missing_suggests_out() {
        let e = RunError::Build("output not produced by build: dev".into());
        let s = e.format_human();
        assert!(s.contains("$out"), "should mention output var: {s}");
    }

    #[test]
    fn build_bwrap_namespace_suggests_sysctl() {
        let e = RunError::Build("bwrap: Can't create user namespace".into());
        let s = e.format_human();
        assert!(s.contains("unprivileged_userns_clone"), "should suggest sysctl: {s}");
    }

    #[test]
    fn internal_store_missing_suggests_flag() {
        let e = RunError::Internal("store directory /nix/store does not exist.".into());
        let s = e.format_human();
        assert!(s.contains("--store"), "should suggest --store flag: {s}");
    }

    #[test]
    fn internal_nix_missing_suggests_canonical_fetch() {
        let e = RunError::Internal("failed to run nix: No such file".into());
        let s = e.format_human();
        assert!(s.contains("mantle bootstrap --fetch"), "should name the canonical fetch command: {s}");
    }

    #[test]
    fn no_suggestions_for_generic_build_error() {
        let e = RunError::Build("something unexpected".into());
        let s = e.format_human();
        assert!(!s.contains("remediation:"), "generic errors should have no invented action: {s}");
    }
}
