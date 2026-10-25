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

/// Pure admission/evaluation failures. Formatting belongs at build/report
/// boundaries; these variants do not absorb filesystem or process I/O errors.
#[derive(Debug, thiserror::Error)]
pub enum GateDenial {
    #[error("policy freshness blocker: {0}")]
    InvalidPolicyDigests(#[source] serde_json::Error),
    #[error("policy freshness blocker: regenerate finish-gate deterministic export")]
    StalePolicy,
    #[error("invalid finish-gate policy export: {0}")]
    InvalidPolicyExport(#[source] serde_json::Error),
    #[error("finish-gate policy export has unsupported limits or schema")]
    UnsupportedPolicy,
    #[error("finish-gate declaration exceeds 16384 bytes")]
    DeclarationTooLarge,
    #[error("invalid finish-gate declaration: {0}")]
    InvalidDeclaration(#[source] serde_json::Error),
    #[error("unsupported finish-gate schema")]
    UnsupportedSchema,
    #[error("finish-gate version command is required")]
    VersionCommandRequired,
    #[error("finish-gate pinned version is required")]
    PinnedVersionRequired,
    #[error("invalid finish-gate version command or pinned version")]
    InvalidVersionCommandOrExpected,
    #[error("finish-gate version command must name a safe output-relative bin/ path")]
    UnsafeVersionCommand,
    #[error("invalid finish-gate allowlisted environment")]
    InvalidEnvironment,
    #[error("invalid native reference leak policy")]
    InvalidReferencePolicy,
    #[error("invalid build-platform reference paths")]
    InvalidBuildPlatformPaths,
    #[error("invalid optional dlopen sonames")]
    InvalidOptionalSonames,
    #[error("dlopen audit requires version gate")]
    DlopenRequiresVersion,
    #[error("dlopen audit reserves LD_AUDIT")]
    ReservedAuditEnvironment,
    #[error("relocation rerun requires version gate")]
    RelocationRequiresVersion,
    #[error("finish-gate reference candidates exceed 128 build-only paths")]
    ReferenceCandidateLimit,
    #[error("finish-gate reference findings exceed 128")]
    ReferenceFindingLimit,
    #[error("version gate did not produce finish evidence")]
    MissingVersionEvidence,
    #[error("relocation gate did not produce finish evidence")]
    MissingRelocationEvidence,
    #[error("dlopen gate has no build log")]
    MissingAuditLog,
    #[error("dlopen audit exceeded event bound")]
    AuditEventLimit,
    #[error("dlopen audit exceeded callback or soname bound")]
    AuditCallbackLimit,
    #[error("dlopen audit event precedes module initialization")]
    AuditBeforeInitialization,
    #[error("malformed dlopen audit event")]
    MalformedAuditEvent,
    #[error("malformed dlopen audit soname length")]
    MalformedAuditNameLength,
    #[error("invalid dlopen audit hex")]
    InvalidAuditHex,
    #[error("non-UTF-8 dlopen audit soname")]
    NonUtf8AuditName,
    #[error("invalid dlopen audit soname")]
    InvalidAuditName,
    #[error("unknown dlopen audit event")]
    UnknownAuditEvent,
    #[error("dlopen audit module did not execute in the version process")]
    MissingAuditInitialization,
    #[error("version command missing")]
    VersionCommandMissing,
    #[error("expected version missing")]
    ExpectedVersionMissing,
    #[error("dlopen audit needs native Linux with the glibc loader")]
    UnsupportedAuditHost,
}

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

fn policy() -> Result<(), GateDenial> {
    let digests: SourceDigests = serde_json::from_str(SOURCE_DIGESTS).map_err(GateDenial::InvalidPolicyDigests)?;
    for (source, digest) in [
        (POLICY_SOURCE, digests.default),
        (POLICY_CONTRACT, digests.contracts),
        (GATE_CONTRACT, digests.gate),
    ] {
        if blake3::hash(source.as_bytes()).to_hex().as_str() != digest {
            return Err(GateDenial::StalePolicy);
        }
    }
    let policy: FinishPolicy = serde_json::from_str(POLICY_JSON).map_err(GateDenial::InvalidPolicyExport)?;
    if policy.schema != "mantle-derivation-finish-policy-v1" {
        return Err(GateDenial::UnsupportedPolicy);
    }
    if policy.max_command_args != 16 || policy.max_command_arg_bytes != 4096 {
        return Err(GateDenial::UnsupportedPolicy);
    }
    if policy.max_scan_hits != 128 || policy.max_excerpt_bytes != 160 || policy.max_optional_sonames != 32 {
        return Err(GateDenial::UnsupportedPolicy);
    }
    debug_assert_eq!(u32::try_from(MAX_COMMAND_ARGS).ok(), Some(policy.max_command_args));
    debug_assert_eq!(u32::try_from(MAX_ARG_BYTES).ok(), Some(policy.max_command_arg_bytes));
    Ok(())
}

pub fn parse_gate(raw: &str) -> Result<GateDeclaration, GateDenial> {
    if raw.len() > MAX_DECLARATION_BYTES {
        return Err(GateDenial::DeclarationTooLarge);
    }
    let gates: GateDeclaration = serde_json::from_str(raw).map_err(GateDenial::InvalidDeclaration)?;
    if gates.schema != "mantle-derivation-finish-gates-v1" {
        return Err(GateDenial::UnsupportedSchema);
    }
    if gates.version.enabled {
        let command = gates.version.command.as_ref().ok_or(GateDenial::VersionCommandRequired)?;
        let expected = gates.version.expected.as_ref().ok_or(GateDenial::PinnedVersionRequired)?;
        if command.is_empty() || command.len() > MAX_COMMAND_ARGS {
            return Err(GateDenial::InvalidVersionCommandOrExpected);
        }
        if command.iter().any(|arg| arg.is_empty() || arg.len() > MAX_ARG_BYTES || arg.contains('\0')) {
            return Err(GateDenial::InvalidVersionCommandOrExpected);
        }
        if expected.is_empty() || expected.len() > MAX_EXPECTED_BYTES {
            return Err(GateDenial::InvalidVersionCommandOrExpected);
        }
        let path = &command[0];
        if !path.starts_with("bin/")
            || path.split('/').any(|part| part.is_empty() || part == ".." || part == ".")
            || !path.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'_' | b'.' | b'+'))
        {
            return Err(GateDenial::UnsafeVersionCommand);
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
        return Err(GateDenial::InvalidEnvironment);
    }
    if gates.reference_leak.native != "report" && gates.reference_leak.native != "deny" {
        return Err(GateDenial::InvalidReferencePolicy);
    }
    if gates.reference_leak.build_platform_paths.len() > 128
        || gates.reference_leak.build_platform_paths.iter().any(|p| !p.starts_with('/'))
    {
        return Err(GateDenial::InvalidBuildPlatformPaths);
    }
    if gates.dlopen.optional_sonames.len() > 32
        || gates.dlopen.optional_sonames.iter().any(|s| {
            s.is_empty()
                || s.len() > 256
                || !s.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_' | b'+'))
        })
    {
        return Err(GateDenial::InvalidOptionalSonames);
    }
    validate_gate_dependencies(&gates)?;
    debug_assert!(!gates.dlopen.enabled || gates.version.enabled);
    debug_assert!(!gates.relocation.enabled || gates.version.enabled);
    Ok(gates)
}

fn validate_gate_dependencies(gates: &GateDeclaration) -> Result<(), GateDenial> {
    if gates.dlopen.enabled && !gates.version.enabled {
        return Err(GateDenial::DlopenRequiresVersion);
    }
    if gates.dlopen.enabled && gates.version.environment.contains_key("LD_AUDIT") {
        return Err(GateDenial::ReservedAuditEnvironment);
    }
    if gates.relocation.enabled && !gates.version.enabled {
        return Err(GateDenial::RelocationRequiresVersion);
    }
    Ok(())
}

/// Candidates for a read-only contextual pass over CAS output bytes.
/// The Snix ReferenceScanner, not a second grep implementation, detects hits.
pub fn reference_candidates(
    gates: &GateDeclaration,
    logical_store_prefix: &str,
    input_paths: impl Iterator<Item = String>,
    cross_build: bool,
) -> Result<Vec<String>, GateDenial> {
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
        return Err(GateDenial::ReferenceCandidateLimit);
    }
    let mut candidates = Vec::with_capacity(extra.len().saturating_add(1));
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
pub struct ReferenceScanContext<'a> {
    pub logical_store_prefix: &'a str,
    pub output: &'a str,
    pub cross_build: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReferenceDecision {
    Report,
    DenyBuildPlatform,
    DenyCrossBuild,
    DenyNativePolicy,
}

fn reference_decision(
    gates: &GateDeclaration,
    is_build_only: bool,
    context: &ReferenceScanContext<'_>,
) -> ReferenceDecision {
    if is_build_only {
        ReferenceDecision::DenyBuildPlatform
    } else if context.cross_build {
        ReferenceDecision::DenyCrossBuild
    } else if gates.reference_leak.native == "deny" {
        ReferenceDecision::DenyNativePolicy
    } else {
        ReferenceDecision::Report
    }
}

pub fn evaluate_references(
    gates: &GateDeclaration,
    observed: impl Iterator<Item = ReferenceObservation>,
    context: ReferenceScanContext<'_>,
) -> Result<Vec<ReferenceFinding>, GateDenial> {
    if !gates.reference_leak.enabled {
        return Ok(Vec::new());
    }
    let mut findings = Vec::new();
    let prefix = format!("{}/", context.logical_store_prefix);
    for hit in observed {
        if findings.len() >= 128 {
            return Err(GateDenial::ReferenceFindingLimit);
        }
        let is_build_only = hit.reference != prefix;
        let decision = reference_decision(gates, is_build_only, &context);
        findings.push(ReferenceFinding {
            output: context.output.to_string(),
            file: hit.file,
            reference: hit.reference,
            excerpt: hit.excerpt,
            denied: decision != ReferenceDecision::Report,
        });
        debug_assert!(findings.len() <= 128);
    }
    debug_assert!(findings.iter().all(|finding| finding.reference == prefix || finding.denied));
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
) -> Result<FinishGateReport, GateDenial> {
    if gates.version.enabled && !build_log.is_some_and(|log| log.contains("finish-gate version passed:")) {
        return Err(GateDenial::MissingVersionEvidence);
    }
    if gates.relocation.enabled && !build_log.is_some_and(|log| log.contains("finish-gate relocation passed:")) {
        return Err(GateDenial::MissingRelocationEvidence);
    }
    let (missing_sonames, audit_error) = if gates.dlopen.enabled {
        match build_log.ok_or(GateDenial::MissingAuditLog).and_then(missing_dlopen_sonames) {
            Ok(names) => (names, None),
            Err(reason) => (Vec::new(), Some(reason)),
        }
    } else {
        (Vec::new(), None)
    };
    let is_denied_dlopen =
        audit_error.is_some() || missing_sonames.iter().any(|name| !gates.dlopen.optional_sonames.contains(name));
    let is_denied = reference_findings.iter().any(|finding| finding.denied);
    let gate_result = FinishGateReport {
        schema: "mantle-derivation-finish-gate-report-v1",
        derivation: derivation.to_string(),
        version: if gates.version.enabled { "passed" } else { "opted-out" }.to_string(),
        command: gates.version.command.clone(),
        expected: gates.version.expected.clone(),
        reference_leak: if !gates.reference_leak.enabled {
            "opted-out"
        } else if is_denied {
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
        } else if is_denied_dlopen {
            "denied"
        } else {
            "passed"
        }
        .to_string(),
        optional_sonames: gates.dlopen.optional_sonames.clone(),
        missing_sonames,
        audit_error: audit_error.map(|denial| denial.to_string()),
    };
    debug_assert_eq!(gate_result.reference_leak == "denied", gates.reference_leak.enabled && is_denied);
    debug_assert_eq!(gate_result.dlopen == "denied", gates.dlopen.enabled && is_denied_dlopen);
    Ok(gate_result)
}
/// Builder failures have no output CAS node to inspect. Preserve an explicit
/// not-run result instead of claiming that a gate silently passed.
pub fn failed_build_report(derivation: &str, gates: &GateDeclaration, failure: &str) -> FinishGateReport {
    let version_passed_at = failure.rfind("finish-gate version passed:");
    let version_failed_at = failure.rfind("finish-gate version failed:");
    let builder_failed_at = failure.rfind("finish-gate builder failed before gates:");
    let is_builder_failed = builder_failed_at.is_some_and(|index| {
        version_passed_at.is_none_or(|passed| index > passed) && version_failed_at.is_none_or(|failed| index > failed)
    });
    let latest_version_failure =
        version_failed_at.filter(|failed| version_passed_at.is_none_or(|passed| *failed > passed));
    let is_relocation_failed = !is_builder_failed
        && gates.relocation.enabled
        && latest_version_failure.is_some_and(|index| {
            failure[index..].starts_with("finish-gate version failed: path=/tmp/mantle-finish-relocated-")
        });
    let is_version_failed = !is_builder_failed && latest_version_failure.is_some() && !is_relocation_failed;
    let is_version_passed = !is_builder_failed && version_passed_at.is_some() && !is_version_failed;
    let gate_result = FinishGateReport {
        schema: "mantle-derivation-finish-gate-report-v1",
        derivation: derivation.to_string(),
        version: if !gates.version.enabled {
            "opted-out"
        } else if is_version_passed {
            "passed"
        } else if is_version_failed {
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
        } else if is_relocation_failed {
            "denied"
        } else {
            "not-run"
        }
        .to_string(),
        dlopen: if gates.dlopen.enabled { "not-run" } else { "opted-out" }.to_string(),
        optional_sonames: gates.dlopen.optional_sonames.clone(),
        missing_sonames: Vec::new(),
        audit_error: None,
    };
    debug_assert_eq!(gate_result.version == "denied", gates.version.enabled && is_version_failed);
    debug_assert_eq!(gate_result.relocation == "denied", gates.relocation.enabled && is_relocation_failed);
    gate_result
}

/// Parse actual glibc loader callbacks. An absent version row, malformed row,
/// or bounded-log overflow cannot silently turn an unevaluated gate green.
fn missing_dlopen_sonames(log: &str) -> Result<Vec<String>, GateDenial> {
    let mut is_audited = false;
    let mut events = 0usize;
    let mut searched = BTreeSet::new();
    let mut opened = BTreeSet::new();
    let mut missing = BTreeSet::new();
    for line in log.lines().filter_map(|line| line.strip_prefix(AUDIT_PREFIX)) {
        events = events.saturating_add(1);
        if events > MAX_AUDIT_EVENTS {
            return Err(GateDenial::AuditEventLimit);
        }
        if line == "V" {
            missing.extend(searched.difference(&opened).cloned());
            searched.clear();
            opened.clear();
            is_audited = true;
            continue;
        }
        if line == "X" {
            return Err(GateDenial::AuditCallbackLimit);
        }
        if !is_audited {
            return Err(GateDenial::AuditBeforeInitialization);
        }
        let (kind, encoded) = line.split_once('\t').ok_or(GateDenial::MalformedAuditEvent)?;
        if encoded.len() > 512 || encoded.len() % 2 != 0 {
            return Err(GateDenial::MalformedAuditNameLength);
        }
        let bytes = data_encoding::HEXLOWER.decode(encoded.as_bytes()).map_err(|_| GateDenial::InvalidAuditHex)?;
        let name = String::from_utf8(bytes).map_err(|_| GateDenial::NonUtf8AuditName)?;
        if name.is_empty() || name.contains('\0') {
            return Err(GateDenial::InvalidAuditName);
        }
        match kind {
            "S" => {
                // The contract audits sonames, not slash-bearing path lookups.
                // Absolute DT_NEEDED paths are not missing sonames.
                if !name.contains('/') {
                    searched.insert(name);
                }
            }
            "O" => {
                opened.insert(name.rsplit('/').next().unwrap_or(&name).to_string());
            }
            _ => return Err(GateDenial::UnknownAuditEvent),
        }
        debug_assert!(searched.len() <= events);
    }
    if !is_audited {
        return Err(GateDenial::MissingAuditInitialization);
    }
    debug_assert!(opened.len() <= events);
    missing.extend(searched.difference(&opened).cloned());
    Ok(missing.into_iter().collect())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn render_version_run(gates: &GateDeclaration, root: &str) -> Result<String, GateDenial> {
    let command = gates.version.command.as_ref().ok_or(GateDenial::VersionCommandMissing)?;
    let expected = gates.version.expected.as_ref().ok_or(GateDenial::ExpectedVersionMissing)?;
    let executable = command.first().ok_or(GateDenial::VersionCommandMissing)?;
    if executable.is_empty() || expected.is_empty() {
        return Err(GateDenial::InvalidVersionCommandOrExpected);
    }
    let mut run = String::from("/bin/busybox env -i");
    if gates.dlopen.enabled {
        run.push_str(" LD_AUDIT=/build/mantle-finish-dlopen-audit.so");
    }
    for (name, value) in &gates.version.environment {
        run.push(' ');
        run.push_str(&shell_quote(&format!("{name}={value}")));
    }
    run.push_str(&format!(" \"${{{root}}}/{executable}\""));
    for arg in command.iter().skip(1) {
        run.push(' ');
        run.push_str(&shell_quote(arg));
    }
    let command_display = command.join(" ");
    let rendered = format!(
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
    );
    debug_assert!(rendered.contains("/bin/busybox timeout -s KILL 30"));
    debug_assert!(rendered.contains("if [ \"$gate_bytes\" -gt 65536 ]"));
    Ok(rendered)
}

/// Append to an adopted shell derivation's script, after its install and fixup.
/// The static BusyBox in the sandbox is used to avoid ambient PATH and libc.
pub fn render_finish_shell(gates: &GateDeclaration) -> Result<String, GateDenial> {
    policy()?;
    if gates.dlopen.enabled && !cfg!(mantle_native_dlopen) {
        return Err(GateDenial::UnsupportedAuditHost);
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
    debug_assert!(script.starts_with("\n# r[impl mantle.derivation_finish_gates.shared_contract]\nset -eu\n"));
    Ok(script)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unbounded_gate_input() {
        assert!(matches!(parse_gate(&"x".repeat(MAX_DECLARATION_BYTES + 1)), Err(GateDenial::DeclarationTooLarge)));
        assert!(matches!(parse_gate("{}"), Err(GateDenial::InvalidDeclaration(_))));
    }

    #[test]
    fn declaration_denials_distinguish_unsafe_commands_and_gate_dependencies() {
        let mut exported: serde_json::Value = serde_json::from_str(POLICY_JSON).unwrap();
        let mut declaration = exported["gates"].take();
        declaration["version"]["enabled"] = serde_json::json!(true);
        declaration["version"]["environment"] = serde_json::json!({});
        declaration["dlopen"]["enabled"] = serde_json::json!(false);
        declaration["relocation"]["enabled"] = serde_json::json!(false);
        declaration["version"]["command"] = serde_json::json!(["bin/probe", "--version"]);
        declaration["version"]["expected"] = serde_json::json!("1.0");
        let mut missing = declaration.clone();
        missing["version"].as_object_mut().unwrap().remove("command");
        assert!(matches!(parse_gate(&missing.to_string()), Err(GateDenial::VersionCommandRequired)));
        let mut unsafe_command = declaration.clone();
        unsafe_command["version"]["command"] = serde_json::json!(["bin/../outside"]);
        assert!(matches!(parse_gate(&unsafe_command.to_string()), Err(GateDenial::UnsafeVersionCommand)));
        let mut disabled = declaration.clone();
        disabled["version"]["enabled"] = serde_json::json!(false);
        disabled["relocation"]["enabled"] = serde_json::json!(true);
        assert!(matches!(parse_gate(&disabled.to_string()), Err(GateDenial::RelocationRequiresVersion)));
        disabled["dlopen"]["enabled"] = serde_json::json!(true);
        assert!(matches!(parse_gate(&disabled.to_string()), Err(GateDenial::DlopenRequiresVersion)));
        declaration["dlopen"]["enabled"] = serde_json::json!(true);
        declaration["version"]["environment"] = serde_json::json!({"LD_AUDIT": "/outside/audit.so"});
        assert!(matches!(parse_gate(&declaration.to_string()), Err(GateDenial::ReservedAuditEnvironment)));
    }

    #[test]
    fn reference_bounds_and_required_execution_evidence_fail_closed() {
        let mut exported: serde_json::Value = serde_json::from_str(POLICY_JSON).unwrap();
        let mut gates: GateDeclaration = serde_json::from_value(exported["gates"].take()).unwrap();
        gates.reference_leak.enabled = true;
        gates.reference_leak.native = "report".into();
        gates.version.enabled = true;
        gates.relocation.enabled = false;
        let inputs = |count| (0..count).map(|index| format!("/build/platform-{index}"));
        let admitted = reference_candidates(&gates, "/mantle/store", inputs(127), true).unwrap();
        assert_eq!(admitted.first().map(String::as_str), Some("/mantle/store/"));
        assert!(admitted.contains(&"/build/platform-126".to_string()));
        assert!(matches!(
            reference_candidates(&gates, "/mantle/store", inputs(128), true),
            Err(GateDenial::ReferenceCandidateLimit)
        ));
        let observations = |count| {
            (0..count).map(|index| ReferenceObservation {
                file: format!("bin/program-{index}"),
                reference: "/mantle/store/".into(),
                excerpt: "/mantle/store/".into(),
            })
        };
        let context = || ReferenceScanContext {
            logical_store_prefix: "/mantle/store",
            output: "out",
            cross_build: false,
        };
        let admitted_findings = evaluate_references(&gates, observations(128), context()).unwrap();
        assert!(admitted_findings.iter().all(|finding| !finding.denied));
        assert_eq!(admitted_findings.last().map(|finding| finding.file.as_str()), Some("bin/program-127"));
        assert!(matches!(
            evaluate_references(&gates, observations(129), context()),
            Err(GateDenial::ReferenceFindingLimit)
        ));
        assert!(matches!(result_report("probe", &gates, Vec::new(), None), Err(GateDenial::MissingVersionEvidence)));
        gates.relocation.enabled = true;
        assert!(matches!(
            result_report("probe", &gates, Vec::new(), Some("finish-gate version passed: probe")),
            Err(GateDenial::MissingRelocationEvidence)
        ));
    }

    #[test]
    fn reference_policy_reports_native_but_denies_cross_and_build_only_hits() {
        let mut exported: serde_json::Value = serde_json::from_str(POLICY_JSON).unwrap();
        let mut gates: GateDeclaration = serde_json::from_value(exported["gates"].take()).unwrap();
        gates.reference_leak.native = "report".into();
        let observe = |reference: &str| ReferenceObservation {
            file: "bin/compiler".into(),
            reference: reference.into(),
            excerpt: reference.into(),
        };
        for (reference, cross_build, denied) in [
            ("/mantle/store/", false, false),
            ("/mantle/store/", true, true),
            ("/build/platform-input", false, true),
        ] {
            let context = ReferenceScanContext {
                logical_store_prefix: "/mantle/store",
                output: "out",
                cross_build,
            };
            let findings = evaluate_references(&gates, std::iter::once(observe(reference)), context).unwrap();
            assert_eq!(findings[0].denied, denied);
            assert_eq!(findings[0].file, "bin/compiler");
            assert_eq!(findings[0].reference, reference);
        }
        gates.reference_leak.native = "deny".into();
        let context = ReferenceScanContext {
            logical_store_prefix: "/mantle/store",
            output: "out",
            cross_build: false,
        };
        assert!(evaluate_references(&gates, std::iter::once(observe("/mantle/store/")), context).unwrap()[0].denied);
    }

    #[test]
    fn audit_missing_soname_cannot_be_satisfied_by_another_run() {
        let event =
            |kind, name: &str| format!("{AUDIT_PREFIX}{kind}\t{}\n", data_encoding::HEXLOWER.encode(name.as_bytes()));
        let first = format!("{AUDIT_PREFIX}V\n{}", event('S', "liboptional.so"));
        let second = format!(
            "{AUDIT_PREFIX}V\n{}{}{}{}",
            event('S', "liboptional.so"),
            event('O', "/declared/liboptional.so"),
            event('S', "/declared/libabsolute.so"),
            event('O', "/declared/libabsolute.so"),
        );
        assert_eq!(missing_dlopen_sonames(&second).unwrap(), Vec::<String>::new());
        assert_eq!(missing_dlopen_sonames(&(first + &second)).unwrap(), ["liboptional.so"]);
        assert!(matches!(
            missing_dlopen_sonames(&event('S', "liboptional.so")),
            Err(GateDenial::AuditBeforeInitialization)
        ));
        assert!(matches!(
            missing_dlopen_sonames(&format!("{AUDIT_PREFIX}V\n{}", event('S', "lib\0evil.so"))),
            Err(GateDenial::InvalidAuditName)
        ));
        assert!(matches!(
            missing_dlopen_sonames(&format!("{AUDIT_PREFIX}V\n{AUDIT_PREFIX}X\n")),
            Err(GateDenial::AuditCallbackLimit)
        ));
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
