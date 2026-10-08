//! Pure policy and command construction for explicitly adopted derivation finish gates.
use std::collections::BTreeSet;

use serde::Deserialize;

pub const POLICY_ENV: &str = "__MANTLE_DERIVATION_FINISH_GATES";
const POLICY_JSON: &str = include_str!("../../../config/derivation-finish-gates/generated/default.json");
const POLICY_SOURCE: &str = include_str!("../../../config/derivation-finish-gates/default.ncl");
const POLICY_CONTRACT: &str = include_str!("../../../config/derivation-finish-gates/contracts.ncl");
const GATE_CONTRACT: &str = include_str!("../../../lib/finish_gates.ncl");
const SOURCE_DIGESTS: &str = include_str!("../../../config/derivation-finish-gates/generated/source-digests.json");
const MAX_DECLARATION_BYTES: usize = 16384;
const MAX_COMMAND_ARGS: usize = 16;
const MAX_ARG_BYTES: usize = 4096;
const MAX_EXPECTED_BYTES: usize = 256;
const AUDIT_PREFIX: &str = "finish-gate loader-audit-v1 ";
const MAX_AUDIT_EVENTS: usize = 260;
#[cfg(mantle_native_dlopen)]
pub const AUDIT_MODULE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mantle-finish-dlopen-audit.so"));

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceDigests {
    default: String,
    contracts: String,
    gate: String,
}

// The rendered policy also carries Nickel gate defaults; runtime consumes only
// these fixed limits and validates each derivation's declared gates separately.
#[derive(Deserialize)]
struct FinishPolicy {
    schema: String,
    max_command_args: u32,
    max_command_arg_bytes: u32,
    max_scan_hits: u32,
    max_excerpt_bytes: u32,
    max_optional_sonames: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateDeclaration {
    schema: String,
    pub version: VersionGate,
    pub reference_leak: ReferenceLeakGate,
    pub relocation: Toggle,
    pub dlopen: DlopenGate,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionGate {
    pub enabled: bool,
    pub command: Option<Vec<String>>,
    pub expected: Option<String>,
    pub environment: std::collections::BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceLeakGate {
    pub enabled: bool,
    pub native: String,
    pub build_platform_paths: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Toggle {
    pub enabled: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DlopenGate {
    pub enabled: bool,
    pub optional_sonames: Vec<String>,
}

fn policy() -> Result<(), String> {
    let digests: SourceDigests =
        serde_json::from_str(SOURCE_DIGESTS).map_err(|e| format!("policy freshness blocker: {e}"))?;
    for (source, digest) in [
        (POLICY_SOURCE, digests.default),
        (POLICY_CONTRACT, digests.contracts),
        (GATE_CONTRACT, digests.gate),
    ] {
        if blake3::hash(source.as_bytes()).to_hex().as_str() != digest {
            return Err("policy freshness blocker: regenerate finish-gate deterministic export".into());
        }
    }
    let policy: FinishPolicy =
        serde_json::from_str(POLICY_JSON).map_err(|e| format!("invalid finish-gate policy export: {e}"))?;
    if policy.schema != "mantle-derivation-finish-policy-v1"
        || policy.max_command_args != 16
        || policy.max_command_arg_bytes != 4096
        || policy.max_scan_hits != 128
        || policy.max_excerpt_bytes != 160
        || policy.max_optional_sonames != 32
    {
        return Err("finish-gate policy export has unsupported limits or schema".into());
    }
    Ok(())
}

pub fn parse_gate(raw: &str) -> Result<GateDeclaration, String> {
    if raw.len() > MAX_DECLARATION_BYTES {
        return Err("finish-gate declaration exceeds 16384 bytes".into());
    }
    let gates: GateDeclaration =
        serde_json::from_str(raw).map_err(|e| format!("invalid finish-gate declaration: {e}"))?;
    if gates.schema != "mantle-derivation-finish-gates-v1" {
        return Err("unsupported finish-gate schema".into());
    }
    if gates.version.enabled {
        let command = gates.version.command.as_ref().ok_or("finish-gate version command is required")?;
        let expected = gates.version.expected.as_ref().ok_or("finish-gate pinned version is required")?;
        if command.is_empty()
            || command.len() > MAX_COMMAND_ARGS
            || command.iter().any(|arg| arg.is_empty() || arg.len() > MAX_ARG_BYTES || arg.contains('\0'))
            || expected.is_empty()
            || expected.len() > MAX_EXPECTED_BYTES
        {
            return Err("invalid finish-gate version command or pinned version".into());
        }
        let path = &command[0];
        if !path.starts_with("bin/")
            || path.split('/').any(|part| part.is_empty() || part == ".." || part == ".")
            || !path.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'_' | b'.' | b'+'))
        {
            return Err("finish-gate version command must name a safe output-relative bin/ path".into());
        }
    }
    if gates.version.environment.len() > 16
        || gates.version.environment.iter().any(|(key, value)| {
            key.is_empty()
                || key.len() > 64
                || value.len() > 4096
                || value.contains('\0')
                || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                || key.as_bytes()[0].is_ascii_digit()
        })
    {
        return Err("invalid finish-gate allowlisted environment".into());
    }
    if gates.reference_leak.native != "report" && gates.reference_leak.native != "deny" {
        return Err("invalid native reference leak policy".into());
    }
    if gates.reference_leak.build_platform_paths.len() > 128
        || gates.reference_leak.build_platform_paths.iter().any(|p| !p.starts_with('/'))
    {
        return Err("invalid build-platform reference paths".into());
    }
    if gates.dlopen.optional_sonames.len() > 32
        || gates.dlopen.optional_sonames.iter().any(|s| {
            s.is_empty()
                || s.len() > 256
                || !s.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_' | b'+'))
        })
    {
        return Err("invalid optional dlopen sonames".into());
    }
    if gates.dlopen.enabled && !gates.version.enabled {
        return Err("dlopen audit requires version gate".into());
    }
    if gates.dlopen.enabled && gates.version.environment.contains_key("LD_AUDIT") {
        return Err("dlopen audit reserves LD_AUDIT".into());
    }
    if gates.relocation.enabled && !gates.version.enabled {
        return Err("relocation rerun requires version gate".into());
    }
    Ok(gates)
}

/// Candidates for a read-only contextual pass over CAS output bytes.
/// The Snix ReferenceScanner, not a second grep implementation, detects hits.
pub fn reference_candidates(
    gates: &GateDeclaration,
    logical_store_prefix: &str,
    input_paths: impl Iterator<Item = String>,
    cross_build: bool,
) -> Result<Vec<String>, String> {
    if !gates.reference_leak.enabled {
        return Ok(Vec::new());
    }
    let prefix = format!("{logical_store_prefix}/");
    let mut extra = gates.reference_leak.build_platform_paths.iter().cloned().collect::<BTreeSet<_>>();
    if cross_build {
        extra.extend(input_paths);
    }
    extra.remove(&prefix);
    if extra.len() >= 128 {
        return Err("finish-gate reference candidates exceed 128 build-only paths".into());
    }
    let mut candidates = Vec::with_capacity(extra.len() + 1);
    candidates.push(prefix);
    candidates.extend(extra);
    Ok(candidates)
}

#[derive(Debug, Clone)]
pub struct ReferenceObservation {
    pub file: String,
    pub reference: String,
    pub excerpt: String,
}

/// Pure policy decision over observations produced by the existing scanner.
pub fn evaluate_references(
    gates: &GateDeclaration,
    observed: impl Iterator<Item = ReferenceObservation>,
    logical_store_prefix: &str,
    output: &str,
    cross_build: bool,
) -> Result<Vec<ReferenceFinding>, String> {
    if !gates.reference_leak.enabled {
        return Ok(Vec::new());
    }
    let mut findings = Vec::new();
    let prefix = format!("{logical_store_prefix}/");
    for hit in observed {
        if findings.len() >= 128 {
            return Err("finish-gate reference findings exceed 128".into());
        }
        let build_only = hit.reference != prefix;
        let denied = build_only || cross_build || gates.reference_leak.native == "deny";
        findings.push(ReferenceFinding {
            output: output.to_string(),
            file: hit.file,
            reference: hit.reference,
            excerpt: hit.excerpt,
            denied,
        });
    }
    Ok(findings)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReferenceFinding {
    pub output: String,
    pub file: String,
    pub reference: String,
    pub excerpt: String,
    pub denied: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FinishGateReport {
    pub schema: &'static str,
    pub derivation: String,
    pub version: String,
    pub command: Option<Vec<String>>,
    pub expected: Option<String>,
    pub reference_leak: String,
    pub reference_findings: Vec<ReferenceFinding>,
    pub relocation: String,
    pub dlopen: String,
    pub optional_sonames: Vec<String>,
    pub missing_sonames: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_error: Option<String>,
}

pub fn result_report(
    derivation: &str,
    gates: &GateDeclaration,
    reference_findings: Vec<ReferenceFinding>,
    build_log: Option<&str>,
) -> Result<FinishGateReport, String> {
    if gates.version.enabled && !build_log.is_some_and(|log| log.contains("finish-gate version passed:")) {
        return Err("version gate did not produce finish evidence".into());
    }
    if gates.relocation.enabled && !build_log.is_some_and(|log| log.contains("finish-gate relocation passed:")) {
        return Err("relocation gate did not produce finish evidence".into());
    }
    let (missing_sonames, audit_error) = if gates.dlopen.enabled {
        match build_log.ok_or_else(|| "dlopen gate has no build log".to_string()).and_then(missing_dlopen_sonames) {
            Ok(names) => (names, None),
            Err(reason) => (Vec::new(), Some(reason)),
        }
    } else {
        (Vec::new(), None)
    };
    let denied_dlopen =
        audit_error.is_some() || missing_sonames.iter().any(|name| !gates.dlopen.optional_sonames.contains(name));
    let denied = reference_findings.iter().any(|finding| finding.denied);
    Ok(FinishGateReport {
        schema: "mantle-derivation-finish-gate-report-v1",
        derivation: derivation.to_string(),
        version: if gates.version.enabled { "passed" } else { "opted-out" }.to_string(),
        command: gates.version.command.clone(),
        expected: gates.version.expected.clone(),
        reference_leak: if !gates.reference_leak.enabled {
            "opted-out"
        } else if denied {
            "denied"
        } else if reference_findings.is_empty() {
            "passed"
        } else {
            "reported"
        }
        .to_string(),
        reference_findings,
        relocation: if gates.relocation.enabled {
            "passed"
        } else {
            "opted-out"
        }
        .to_string(),
        dlopen: if !gates.dlopen.enabled {
            "opted-out"
        } else if denied_dlopen {
            "denied"
        } else {
            "passed"
        }
        .to_string(),
        optional_sonames: gates.dlopen.optional_sonames.clone(),
        missing_sonames,
        audit_error,
    })
}
/// Builder failures have no output CAS node to inspect. Preserve an explicit
/// not-run result instead of claiming that a gate silently passed.
pub fn failed_build_report(derivation: &str, gates: &GateDeclaration, failure: &str) -> FinishGateReport {
    let version_passed_at = failure.rfind("finish-gate version passed:");
    let version_failed_at = failure.rfind("finish-gate version failed:");
    let builder_failed_at = failure.rfind("finish-gate builder failed before gates:");
    let builder_failed = builder_failed_at.is_some_and(|index| {
        version_passed_at.is_none_or(|passed| index > passed) && version_failed_at.is_none_or(|failed| index > failed)
    });
    let latest_version_failure =
        version_failed_at.filter(|failed| version_passed_at.is_none_or(|passed| *failed > passed));
    let relocation_failed = !builder_failed
        && gates.relocation.enabled
        && latest_version_failure.is_some_and(|index| {
            failure[index..].starts_with("finish-gate version failed: path=/tmp/mantle-finish-relocated-")
        });
    let version_failed = !builder_failed && latest_version_failure.is_some() && !relocation_failed;
    let version_passed = !builder_failed && version_passed_at.is_some() && !version_failed;
    FinishGateReport {
        schema: "mantle-derivation-finish-gate-report-v1",
        derivation: derivation.to_string(),
        version: if !gates.version.enabled {
            "opted-out"
        } else if version_passed {
            "passed"
        } else if version_failed {
            "denied"
        } else {
            "not-run"
        }
        .to_string(),
        command: gates.version.command.clone(),
        expected: gates.version.expected.clone(),
        reference_leak: if gates.reference_leak.enabled {
            "not-run"
        } else {
            "opted-out"
        }
        .to_string(),
        reference_findings: Vec::new(),
        relocation: if !gates.relocation.enabled {
            "opted-out"
        } else if relocation_failed {
            "denied"
        } else {
            "not-run"
        }
        .to_string(),
        dlopen: if gates.dlopen.enabled { "not-run" } else { "opted-out" }.to_string(),
        optional_sonames: gates.dlopen.optional_sonames.clone(),
        missing_sonames: Vec::new(),
        audit_error: None,
    }
}

/// Parse actual glibc loader callbacks. An absent version row, malformed row,
/// or bounded-log overflow cannot silently turn an unevaluated gate green.
fn missing_dlopen_sonames(log: &str) -> Result<Vec<String>, String> {
    let mut audited = false;
    let mut events = 0usize;
    let mut searched = BTreeSet::new();
    let mut opened = BTreeSet::new();
    for line in log.lines().filter_map(|line| line.strip_prefix(AUDIT_PREFIX)) {
        events += 1;
        if events > MAX_AUDIT_EVENTS {
            return Err("dlopen audit exceeded event bound".into());
        }
        if line == "V" {
            audited = true;
            continue;
        }
        if line == "X" {
            return Err("dlopen audit exceeded callback or soname bound".into());
        }
        let (kind, encoded) = line.split_once('\t').ok_or("malformed dlopen audit event")?;
        if encoded.len() > 512 || encoded.len() % 2 != 0 {
            return Err("malformed dlopen audit soname length".into());
        }
        let bytes = data_encoding::HEXLOWER.decode(encoded.as_bytes()).map_err(|_| "invalid dlopen audit hex")?;
        let name = String::from_utf8(bytes).map_err(|_| "non-UTF-8 dlopen audit soname")?;
        match kind {
            "S" => {
                searched.insert(name);
            }
            "O" => {
                opened.insert(name.rsplit('/').next().unwrap_or(&name).to_string());
            }
            _ => return Err("unknown dlopen audit event".into()),
        }
    }
    if !audited {
        return Err("dlopen audit module did not execute in the version process".into());
    }
    Ok(searched.into_iter().filter(|name| !opened.contains(name)).collect())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn render_version_run(gates: &GateDeclaration, root: &str) -> Result<String, String> {
    let command = gates.version.command.as_ref().ok_or("version command missing")?;
    let expected = gates.version.expected.as_ref().ok_or("expected version missing")?;
    let mut run = String::from("/bin/busybox env -i");
    if gates.dlopen.enabled {
        run.push_str(" LD_AUDIT=/build/mantle-finish-dlopen-audit.so");
    }
    for (name, value) in &gates.version.environment {
        run.push(' ');
        run.push_str(&shell_quote(&format!("{name}={value}")));
    }
    run.push_str(&format!(" \"${{{root}}}/{}\"", command[0]));
    for arg in command.iter().skip(1) {
        run.push(' ');
        run.push_str(&shell_quote(arg));
    }
    let command_display = command.join(" ");
    Ok(format!(
        "gate_status=0\n\
         gate_output_file=\"/build/.mantle-finish-version-$$\"\n\
         /bin/busybox timeout -s KILL 30 {run} > \"$gate_output_file\" 2>&1 || gate_status=$?\n\
         gate_bytes=$(/bin/busybox stat -c %s \"$gate_output_file\")\n\
         if [ \"$gate_bytes\" -gt 65536 ]; then\n\
           printf 'finish-gate version failed: path=%s output exceeds 65536 bytes\\n' \"${{{root}}}\" >&2\n\
           exit 1\n\
         fi\n\
         gate_output=$(/bin/busybox cat \"$gate_output_file\")\n\
         /bin/busybox rm -f \"$gate_output_file\"\n\
         case \"$gate_output\" in *{expected}*) gate_match=1 ;; *) gate_match=0 ;; esac\n\
         if [ \"$gate_status\" -ne 0 ] || [ \"$gate_match\" -ne 1 ]; then\n\
           printf 'finish-gate version failed: path=%s command=%s expected=%s status=%s output=%s\\n' \"${{{root}}}\" {display} {expected} \"$gate_status\" \"$gate_output\" >&2\n\
           exit 1\n\
         fi\n\
         printf 'finish-gate version passed: command=%s expected=%s\\n' {display} {expected}\n",
        expected = shell_quote(expected),
        display = shell_quote(&command_display),
    ))
}

/// Append to an adopted shell derivation's script, after its install and fixup.
/// The static BusyBox in the sandbox is used to avoid ambient PATH and libc.
pub fn render_finish_shell(gates: &GateDeclaration) -> Result<String, String> {
    policy()?;
    if gates.dlopen.enabled && !cfg!(mantle_native_dlopen) {
        return Err("dlopen audit needs native Linux with the glibc loader".into());
    }
    let mut script = String::from("\n# r[impl mantle.derivation_finish_gates.shared_contract]\nset -eu\n");
    if gates.dlopen.enabled {
        script.push_str("/bin/busybox rm -f /build/.mantle-finish-audit-events\n");
    }
    if gates.version.enabled {
        script.push_str("gate_root=\"$out\"\n");
        script.push_str(&render_version_run(gates, "gate_root")?);
    }
    if gates.relocation.enabled {
        script.push_str(
            "gate_relocated=\"/tmp/mantle-finish-relocated-$$\"\n\
             /bin/busybox cp -R \"$out\" \"$gate_relocated\"\n\
             gate_root=\"$gate_relocated\"\n\
             cd /\n",
        );
        script.push_str(&render_version_run(gates, "gate_root")?);
        script.push_str(
            "printf 'finish-gate relocation passed: copied_path=%s\\n' \"$gate_root\"\n\
             /bin/busybox rm -rf \"$gate_relocated\"\n",
        );
    }
    if gates.dlopen.enabled {
        script.push_str(
            "if [ -f /build/.mantle-finish-audit-events ]; then\n\
             while IFS= read -r gate_event; do\n\
               printf 'finish-gate loader-audit-v1 %s\\n' \"$gate_event\"\n\
             done < /build/.mantle-finish-audit-events\n\
             fi\n",
        );
    }
    Ok(script)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unbounded_gate_input() {
        assert!(parse_gate(&"x".repeat(MAX_DECLARATION_BYTES + 1)).is_err());
        assert!(parse_gate("{}").is_err());
    }
    #[cfg(mantle_native_dlopen)]
    #[test]
    fn real_glibc_audit_distinguishes_missing_and_opened_sonames() {
        use std::process::Command;

        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("probe.c");
        let program = dir.path().join("probe");
        let module = dir.path().join("finish-audit.so");
        let events = dir.path().join("events");
        std::fs::write(
            &source,
            "#include <dlfcn.h>\n#include <stdio.h>\nint main(void) {\n\
             if (dlopen(\"libmantle-required-missing.so\", RTLD_NOW)) return 9;\n\
             if (!dlopen(\"libc.so.6\", RTLD_NOW)) return 10;\n\
             puts(\"demo-1.0\"); return 0;\n}\n",
        )
        .unwrap();
        assert!(Command::new("cc").arg(&source).args(["-ldl", "-o"]).arg(&program).status().unwrap().success());
        let definition = format!("-DAUDIT_LOG=\"{}\"", events.display());
        assert!(
            Command::new("cc")
                .current_dir(env!("CARGO_MANIFEST_DIR"))
                .args(["-std=c11", "-O2", "-fPIC", "-shared"])
                .arg(definition)
                .arg("audit/finish_dlopen_audit.c")
                .arg("-o")
                .arg(&module)
                .status()
                .unwrap()
                .success()
        );
        let output = Command::new(&program).env("LD_AUDIT", &module).output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(output.stdout, b"demo-1.0\n");
        let actual_events = std::fs::read_to_string(&events).unwrap();
        let tagged = actual_events.lines().map(|line| format!("{AUDIT_PREFIX}{line}\n")).collect::<String>();
        let log = format!("finish-gate version passed: demo-1.0\n{tagged}");
        let mut exported: serde_json::Value = serde_json::from_str(POLICY_JSON).unwrap();
        let mut gates: GateDeclaration = serde_json::from_value(exported["gates"].take()).unwrap();
        gates.version.command = Some(vec!["bin/probe".into(), "--version".into()]);
        gates.version.expected = Some("demo-1.0".into());
        gates.dlopen.enabled = true;
        let denied = result_report("probe", &gates, Vec::new(), Some(&log)).unwrap();
        assert_eq!(denied.dlopen, "denied");
        assert_eq!(denied.missing_sonames, ["libmantle-required-missing.so"]);
        gates.dlopen.optional_sonames.push("libmantle-required-missing.so".into());
        let permitted = result_report("probe", &gates, Vec::new(), Some(&log)).unwrap();
        assert_eq!(permitted.dlopen, "passed");
        assert_eq!(permitted.missing_sonames, ["libmantle-required-missing.so"]);
    }
}
