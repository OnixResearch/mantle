#!/usr/bin/env -S rustc --edition=2024
//! Validate the compact i386 TinyCC 0.9.27 handoff summary.
//!
//! This is a diagnostic contract checker, not a bootstrap promotion proof. It
//! accepts either a real object-emission diagnostic summary or a deterministic
//! blocked summary, and rejects overclaims such as placeholder runtime archives
//! reported as handoff success.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

const EXPECTED_SCHEMA: &str = "mantle-i386-tinycc27-handoff-summary-v1";
const EXPECTED_INPUT: &str = "tcc-0.9.27/tcc.c";
const EXPECTED_OBJECT: &str = "share/spike-i386-mes-runtime-layout/artifacts/tcc27.o";
const REAL_RUNTIME: &str = "real";
const REAL_RUNTIME_SET: &str = "real-i386-mes-runtime";
const STATUS_BLOCKED: &str = "blocked";
const STATUS_OBJECT_SUCCESS: &str = "object-emission-success";
const BLOCKED_STEP_NONE: &str = "none";
const DIGEST_HEX_BYTES: usize = 64;
const EXIT_VALID: u8 = 0;
const EXIT_INVALID: u8 = 1;
const EXIT_USAGE: u8 = 2;

#[derive(Debug, Default)]
struct Config {
    summary_path: Option<PathBuf>,
    self_test: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SummaryVerdict {
    ObjectEmissionSuccess,
    Blocked { host_unsupported: bool },
}

#[derive(Debug, Clone)]
struct Summary {
    fields: BTreeMap<String, String>,
}

impl Summary {
    fn parse(text: &str) -> Result<Self, String> {
        let mut fields = BTreeMap::new();
        for (line_index, raw_line) in text.lines().enumerate() {
            let line_number = line_index + 1;
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(format!("line {line_number} is not key=value: {line}"));
            };
            if key.is_empty() {
                return Err(format!("line {line_number} has an empty key"));
            }
            if fields.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(format!("duplicate summary key: {key}"));
            }
        }
        if fields.is_empty() {
            return Err("summary is empty".to_owned());
        }
        Ok(Self { fields })
    }

    fn required(&self, key: &str) -> Result<&str, String> {
        self.fields.get(key).map(String::as_str).ok_or_else(|| format!("missing summary key: {key}"))
    }

    fn optional(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(verdict) => {
            if let Some(verdict) = verdict {
                println!("i386 handoff summary: valid ({})", verdict_label(&verdict));
            }
            ExitCode::from(EXIT_VALID)
        }
        Err(err) => {
            eprintln!("i386 handoff summary: invalid: {err}");
            ExitCode::from(if err.starts_with("usage:") {
                EXIT_USAGE
            } else {
                EXIT_INVALID
            })
        }
    }
}

fn run() -> Result<Option<SummaryVerdict>, String> {
    let config = parse_args()?;
    if config.self_test {
        run_self_tests()?;
    }
    let Some(path) = config.summary_path else {
        if config.self_test {
            return Ok(None);
        }
        return Err("usage: --summary PATH [--self-test]".to_owned());
    };
    let text = fs::read_to_string(&path).map_err(|err| format!("read {}: {err}", path.display()))?;
    let summary = Summary::parse(&text)?;
    validate_summary(&summary).map(Some)
}

fn parse_args() -> Result<Config, String> {
    let mut config = Config::default();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--self-test" => config.self_test = true,
            "--summary" => {
                let Some(path) = args.next() else {
                    return Err("usage: --summary requires a path".to_owned());
                };
                config.summary_path = Some(PathBuf::from(path));
            }
            "-h" | "--help" => return Err("usage: --summary PATH [--self-test]".to_owned()),
            unknown => return Err(format!("usage: unknown argument: {unknown}")),
        }
    }
    Ok(config)
}

fn validate_summary(summary: &Summary) -> Result<SummaryVerdict, String> {
    require_equal(summary, "schema", EXPECTED_SCHEMA)?;
    require_equal(summary, "claim", "diagnostic-only")?;
    require_equal(summary, "selected_input", EXPECTED_INPUT)?;
    require_equal(summary, "selected_object", EXPECTED_OBJECT)?;
    require_equal(summary, "runtime_artifacts", REAL_RUNTIME_SET)?;
    require_non_empty(summary, "host_i386_execution")?;
    require_equal(summary, "make_scope", "out-of-scope-until-tcc27-object-handoff-succeeds")?;
    require_non_empty(summary, "next_action")?;
    require_contains(summary, "non_claim", "diagnostic")?;

    match summary.required("status")? {
        STATUS_OBJECT_SUCCESS => validate_object_success(summary),
        STATUS_BLOCKED => validate_blocked(summary),
        status => Err(format!("unsupported status: {status}")),
    }
}

fn validate_object_success(summary: &Summary) -> Result<SummaryVerdict, String> {
    require_equal(summary, "runtime_libtcc1_object", REAL_RUNTIME)?;
    require_equal(summary, "runtime_libtcc1_archive", REAL_RUNTIME)?;
    require_equal(summary, "runtime_libc_archive", REAL_RUNTIME)?;
    require_equal(summary, "blocked_step", BLOCKED_STEP_NONE)?;
    require_equal(summary, "blocked_rc", "0")?;
    require_equal(summary, "blocked_signal", BLOCKED_STEP_NONE)?;
    require_equal(summary, "object_path", EXPECTED_OBJECT)?;
    require_digest(summary.required("object_digest_sha256")?)?;
    require_contains(summary, "smoke_boundary", "tcc26-i386 -c")?;
    require_contains(summary, "non_claim", "does not switch production bootstrap routing")?;
    Ok(SummaryVerdict::ObjectEmissionSuccess)
}

fn validate_blocked(summary: &Summary) -> Result<SummaryVerdict, String> {
    let blocked_step = summary.required("blocked_step")?;
    if blocked_step == BLOCKED_STEP_NONE {
        return Err("blocked summary must name a concrete blocked_step".to_owned());
    }
    require_non_empty(summary, "blocked_rc")?;
    require_non_empty(summary, "blocked_signal")?;
    if let Some(digest) = summary.optional("object_digest_sha256") {
        if digest != "absent" {
            require_digest(digest)?;
        }
    }
    if runtime_all_real(summary)? && blocked_step == "placeholder_runtime_artifacts" {
        return Err("placeholder_runtime_artifacts blocker contradicts real runtime fields".to_owned());
    }
    require_contains(summary, "non_claim", "does not prove TinyCC 0.9.27 i386 handoff success")?;
    Ok(SummaryVerdict::Blocked {
        host_unsupported: blocked_step == "host_i386_execution_unsupported",
    })
}

fn runtime_all_real(summary: &Summary) -> Result<bool, String> {
    let object = summary.required("runtime_libtcc1_object")?;
    let archive = summary.required("runtime_libtcc1_archive")?;
    let libc = summary.required("runtime_libc_archive")?;
    Ok(object == REAL_RUNTIME && archive == REAL_RUNTIME && libc == REAL_RUNTIME)
}

fn require_equal(summary: &Summary, key: &str, expected: &str) -> Result<(), String> {
    let actual = summary.required(key)?;
    if actual == expected {
        return Ok(());
    }
    Err(format!("{key} expected {expected:?}, got {actual:?}"))
}

fn require_non_empty(summary: &Summary, key: &str) -> Result<(), String> {
    let value = summary.required(key)?;
    if value.is_empty() {
        return Err(format!("{key} must be non-empty"));
    }
    Ok(())
}

fn require_contains(summary: &Summary, key: &str, needle: &str) -> Result<(), String> {
    let value = summary.required(key)?;
    if value.contains(needle) {
        return Ok(());
    }
    Err(format!("{key} must contain {needle:?}"))
}

fn require_digest(value: &str) -> Result<(), String> {
    if value.len() != DIGEST_HEX_BYTES {
        return Err(format!("object digest must be {DIGEST_HEX_BYTES} lowercase hex characters"));
    }
    if !value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err("object digest must be lowercase hexadecimal".to_owned());
    }
    Ok(())
}

fn verdict_label(verdict: &SummaryVerdict) -> &'static str {
    match verdict {
        SummaryVerdict::ObjectEmissionSuccess => "object-emission-success",
        SummaryVerdict::Blocked { host_unsupported: true } => "blocked-host-i386-unsupported",
        SummaryVerdict::Blocked {
            host_unsupported: false,
        } => "blocked-tinycc-handoff",
    }
}

fn run_self_tests() -> Result<(), String> {
    assert_verdict(success_summary(), SummaryVerdict::ObjectEmissionSuccess)?;
    assert_verdict(signal_blocker_summary(), SummaryVerdict::Blocked {
        host_unsupported: false,
    })?;
    assert_verdict(host_unsupported_summary(), SummaryVerdict::Blocked { host_unsupported: true })?;
    assert_invalid(placeholder_overclaim_summary(), "runtime_libtcc1_object expected")?;
    assert_invalid(missing_digest_success_summary(), "object digest")?;
    assert_invalid(blocked_none_summary(), "concrete blocked_step")?;
    println!("i386 handoff summary self-test: ok");
    Ok(())
}

fn assert_verdict(summary_text: String, expected: SummaryVerdict) -> Result<(), String> {
    let summary = Summary::parse(&summary_text)?;
    let actual = validate_summary(&summary)?;
    if actual != expected {
        return Err(format!("self-test verdict mismatch: expected {expected:?}, got {actual:?}"));
    }
    Ok(())
}

fn assert_invalid(summary_text: String, expected_error: &str) -> Result<(), String> {
    let summary = Summary::parse(&summary_text)?;
    match validate_summary(&summary) {
        Ok(verdict) => Err(format!("self-test expected invalid summary, got {verdict:?}")),
        Err(err) if err.contains(expected_error) => Ok(()),
        Err(err) => Err(format!("self-test expected error containing {expected_error:?}, got {err:?}")),
    }
}

fn base_summary(status: &str) -> String {
    format!(
        "schema={EXPECTED_SCHEMA}\nclaim=diagnostic-only\nselected_input={EXPECTED_INPUT}\nselected_object={EXPECTED_OBJECT}\nruntime_artifacts={REAL_RUNTIME_SET}\nruntime_libtcc1_object=real\nruntime_libtcc1_archive=real\nruntime_libc_archive=real\nhost_i386_execution=supported-by-spike-i386-tinycc26-cross-smoke\nmake_scope=out-of-scope-until-tcc27-object-handoff-succeeds\nstatus={status}\n"
    )
}

fn success_summary() -> String {
    let digest = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    format!(
        "{}blocked_step=none\nblocked_rc=0\nblocked_signal=none\nobject_path={EXPECTED_OBJECT}\nobject_digest_sha256={digest}\nsmoke_boundary=tcc26-i386 -c selected TinyCC 0.9.27 source to i386 object with real Mes runtime inputs\nnext_action=link/runtime handoff remains separate; GNU Make stays out of scope for this proof\nnon_claim=object-emission diagnostic only; does not switch production bootstrap routing\n",
        base_summary(STATUS_OBJECT_SUCCESS)
    )
}

fn signal_blocker_summary() -> String {
    format!(
        "{}blocked_step=tcc27_compile_object\nblocked_rc=139\nblocked_signal=11\nobject_digest_sha256=absent\nnext_action=repair the first blocked step below GNU Make before treating downstream i386 bootstrap stages as unblocked\nnon_claim=blocked diagnostic; does not prove TinyCC 0.9.27 i386 handoff success or production bootstrap readiness\n",
        base_summary(STATUS_BLOCKED)
    )
}

fn host_unsupported_summary() -> String {
    format!(
        "{}blocked_step=host_i386_execution_unsupported\nblocked_rc=126\nblocked_signal=none\nobject_digest_sha256=absent\nnext_action=repair host i386 execution before treating the TinyCC handoff as tested\nnon_claim=blocked diagnostic; does not prove TinyCC 0.9.27 i386 handoff success or production bootstrap readiness\n",
        base_summary(STATUS_BLOCKED)
    )
}

fn placeholder_overclaim_summary() -> String {
    success_summary().replace("runtime_libtcc1_object=real", "runtime_libtcc1_object=placeholder-crt1-continuation")
}

fn missing_digest_success_summary() -> String {
    success_summary().replace(
        "object_digest_sha256=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "object_digest_sha256=absent",
    )
}

fn blocked_none_summary() -> String {
    signal_blocker_summary().replace("blocked_step=tcc27_compile_object", "blocked_step=none")
}
