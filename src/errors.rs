//! Error types for the crunch CLI.
//!
//! `RunError` classifies all failures into eval, build, or internal errors.
//! Each variant maps to a fixed exit code and produces actionable output.

use std::fmt;
use std::process::ExitCode;

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
        match self {
            RunError::Eval(msg) => format!("error: evaluation failed\n{msg}"),
            RunError::Build(msg) => {
                let body = extract_build_body(msg);
                let suggestions = build_suggestions(msg);
                if suggestions.is_empty() {
                    format!("error: build failed\n{body}")
                } else {
                    format!("error: build failed\n{body}\n\nsuggestions:\n{suggestions}")
                }
            }
            RunError::Internal(msg) => {
                let suggestions = internal_suggestions(msg);
                if suggestions.is_empty() {
                    format!("error: {msg}")
                } else {
                    format!("error: {msg}\n\nsuggestions:\n{suggestions}")
                }
            }
            RunError::Reported(_) => String::new(),
        }
    }

    /// Format as a JSON object for `--json` mode.
    pub fn format_json(&self) -> String {
        if matches!(self, RunError::Reported(_)) {
            return String::new();
        }

        // Manual formatting to avoid pulling serde into this module for
        // a three-field object. The message is escaped for JSON safety.
        let escaped = self
            .message()
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        let code: u8 = match self {
            RunError::Eval(_) => 2,
            RunError::Build(_) => 1,
            RunError::Internal(_) => 3,
            RunError::Reported(_) => 4, // reported errors should have been handled before JSON formatting
        };
        format!(r#"{{"error":"{}","code":{},"kind":"{}"}}"#, escaped, code, self.kind())
    }
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

/// Produce newline-separated suggestions for common build failures.
#[allow(tigerstyle::unbounded_collection_growth)] // bounded by fixed number of pattern matches below
fn build_suggestions(msg: &str) -> String {
    debug_assert!(!msg.is_empty());
    let mut suggestions = Vec::with_capacity(8);
    let lower = msg.to_lowercase();
    debug_assert_eq!(lower.len(), msg.len());

    if lower.contains("nonzero exit code") {
        suggestions.push("  - Check the build log: ls $XDG_STATE_HOME/crunch/logs/ (or ~/.local/state/crunch/logs/)");
        suggestions.push("  - Run with --verbose / --log-level=debug to see sandbox details");
    }

    if lower.contains("bwrap") && lower.contains("can't") {
        suggestions.push("  - bwrap namespace setup failed. Check that unprivileged user namespaces are enabled:");
        suggestions.push("    sysctl kernel.unprivileged_userns_clone  (should be 1)");
    }

    if lower.contains("source input not found in store") || lower.contains("sourcenotfound") {
        suggestions.push("  - A store path referenced by your derivation doesn't exist on disk");
        suggestions.push("  - Re-run `crunch bootstrap` to regenerate seed.ncl with current store paths");
        suggestions.push("  - If using Nix seeds, pin paths as GC roots to prevent collection");
        suggestions.push("  - Or use `crunch bootstrap --fetch` to avoid Nix store dependencies entirely");
    }

    if lower.contains("fod hash mismatch") {
        suggestions.push("  - The fixed-output derivation produced content with a different hash than declared");
        suggestions.push("  - Update the hash in your .ncl file, or check that the fetcher is deterministic");
    }

    if lower.contains("builds are not supported") || lower.contains("only supported on linux") {
        suggestions.push("  - Building requires Linux with bubblewrap (bwrap) installed");
        suggestions.push("  - Install bwrap: https://github.com/containers/bubblewrap");
        suggestions.push("  - Or run `crunch self-build` to bootstrap bwrap from source");
    }

    if lower.contains("output not produced by build") {
        suggestions.push("  - The builder script didn't write to all declared output paths");
        suggestions.push("  - Make sure your build script creates $out (and any other declared outputs)");
    }

    suggestions.join("\n")
}

/// Suggestions for internal (non-build) errors.
fn internal_suggestions(msg: &str) -> String {
    let mut suggestions = Vec::new();
    let lower = msg.to_lowercase();

    if lower.contains("store directory") && lower.contains("does not exist") {
        suggestions.push("  - The default store is /nix/store. Create it or pass --store <path>");
    }

    if lower.contains("stdlib") {
        suggestions.push("  - crunch can't find its stdlib. Check your installation or set CRUNCH_STDLIB_DIR");
    }

    if lower.contains("failed to run nix") || lower.contains("failed to resolve") {
        suggestions.push("  - Make sure nix or nix-build is on your PATH, or use `crunch bootstrap --fetch` instead");
    }

    suggestions.join("\n")
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
        assert!(s.contains("crunch/logs/"), "should mention log dir: {s}");
        assert!(s.contains("--verbose"), "should suggest verbose: {s}");
    }

    #[test]
    fn build_source_not_found_suggests_bootstrap() {
        let e = RunError::Build("source input not found in store: /nix/store/xxx-bash".into());
        let s = e.format_human();
        assert!(s.contains("crunch bootstrap"), "should suggest bootstrap: {s}");
        assert!(s.contains("GC roots"), "should mention GC roots: {s}");
    }

    #[test]
    fn build_fod_mismatch_suggests_update() {
        let e = RunError::Build("FOD hash mismatch for foo: expected abc, got def".into());
        let s = e.format_human();
        assert!(s.contains("Update the hash"), "should suggest hash update: {s}");
    }

    #[test]
    fn build_not_linux_suggests_bwrap() {
        let e = RunError::Build("building is only supported on Linux".into());
        let s = e.format_human();
        assert!(s.contains("bubblewrap"), "should mention bwrap: {s}");
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
    fn internal_nix_missing_suggests_install() {
        let e = RunError::Internal("failed to run nix: No such file".into());
        let s = e.format_human();
        assert!(s.contains("bootstrap --fetch"), "should suggest fetch bootstrap: {s}");
    }

    #[test]
    fn no_suggestions_for_generic_build_error() {
        let e = RunError::Build("something unexpected".into());
        let s = e.format_human();
        // Should not contain "suggestions:" since nothing matched
        assert!(!s.contains("suggestions:"), "generic errors should have no suggestions: {s}");
    }
}
