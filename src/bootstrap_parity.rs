use std::fmt;
use std::fs;
use std::path::Path;

use clap::ValueEnum;
use serde::Serialize;

use crate::errors::RunError;

const REPORT_SCHEMA: &str = "crunch-bootstrap-parity-gap-report-v1";
const BOOTSTRAP_DIR: &str = "bootstrap";
const PLACEHOLDER_MARKERS: &[&str] = &[
    "placeholder",
    "stub",
    "TODO",
    "not yet implemented",
    "pass1 bridge",
    "bridge-only",
];

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, ValueEnum)]
pub enum ParityAxis {
    #[serde(rename = "live-bootstrap")]
    LiveBootstrap,
    #[serde(rename = "guix")]
    Guix,
    #[serde(rename = "stagex")]
    Stagex,
}

impl fmt::Display for ParityAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LiveBootstrap => write!(f, "live-bootstrap"),
            Self::Guix => write!(f, "guix"),
            Self::Stagex => write!(f, "stagex"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
pub enum StageStatus {
    Complete,
    Partial,
    Placeholder,
    Blocked,
    OutOfScopeReplaced,
    NotStarted,
}

impl StageStatus {
    fn blocks_parity(self) -> bool {
        !matches!(self, Self::Complete | Self::OutOfScopeReplaced)
    }
}

impl fmt::Display for StageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete => write!(f, "complete"),
            Self::Partial => write!(f, "partial"),
            Self::Placeholder => write!(f, "placeholder"),
            Self::Blocked => write!(f, "blocked"),
            Self::OutOfScopeReplaced => write!(f, "out-of-scope-replaced"),
            Self::NotStarted => write!(f, "not-started"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderKind {
    LegacyFetch,
    SourceRoot,
    StagexLineage,
    Unknown,
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LegacyFetch => write!(f, "legacy-fetch"),
            Self::SourceRoot => write!(f, "source-root"),
            Self::StagexLineage => write!(f, "stagex-lineage"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BootstrapParityReport {
    pub schema: &'static str,
    pub axes: Vec<AxisSummary>,
    pub rows: Vec<ParityRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AxisSummary {
    pub axis: ParityAxis,
    pub complete: bool,
    pub blocking_rows: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParityRow {
    pub id: &'static str,
    pub title: &'static str,
    pub axes: Vec<ParityAxis>,
    pub lineage: &'static str,
    pub derivation: Option<&'static str>,
    pub status: StageStatus,
    pub provider_kind: ProviderKind,
    pub graph_evidence: &'static str,
    pub semantic_evidence: &'static str,
    pub proof_evidence: &'static str,
    pub notes: String,
}

#[derive(Debug, Copy, Clone)]
struct StageSpec {
    id: &'static str,
    title: &'static str,
    axes: &'static [ParityAxis],
    lineage: &'static str,
    derivation: Option<&'static str>,
    expected_complete: bool,
    graph_evidence: &'static str,
    semantic_evidence: &'static str,
    proof_evidence: &'static str,
    notes: &'static str,
    evidence_check: EvidenceCheck,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum EvidenceCheck {
    None,
    SeedFullSourceRootContract,
    BinutilsTccToolTranscript,
}

const BINUTILS_TCC_TOOL_TRANSCRIPT: &str = "bootstrap/evidence/binutils-tcc-tool-smoke.json";
const BINUTILS_TCC_REQUIRED_TOOLS: &[&str] = &["as", "ld", "ar", "ranlib", "nm", "objcopy"];

pub fn cmd_bootstrap_parity_report(project_root: &Path, require: &[ParityAxis], json: bool) -> Result<(), RunError> {
    let report = collect_bootstrap_parity_report(project_root);
    if json {
        let rendered = serde_json::to_string_pretty(&report)
            .map_err(|err| RunError::Internal(format!("serializing bootstrap parity report: {err}")))?;
        println!("{rendered}");
    } else {
        println!("{}", render_bootstrap_parity_report(&report));
    }

    let failed_requirements: Vec<String> =
        require.iter().filter(|axis| !axis_complete(&report, **axis)).map(ToString::to_string).collect();
    if failed_requirements.is_empty() {
        Ok(())
    } else {
        Err(RunError::Reported(1))
    }
}

pub fn collect_bootstrap_parity_report(project_root: &Path) -> BootstrapParityReport {
    let rows: Vec<ParityRow> = parity_stage_specs().iter().map(|spec| evaluate_stage(project_root, spec)).collect();
    let axes = [ParityAxis::LiveBootstrap, ParityAxis::Guix, ParityAxis::Stagex]
        .into_iter()
        .map(|axis| summarize_axis(axis, &rows))
        .collect();
    BootstrapParityReport {
        schema: REPORT_SCHEMA,
        axes,
        rows,
    }
}

pub fn render_bootstrap_parity_report(report: &BootstrapParityReport) -> String {
    let mut out = String::new();
    out.push_str("Bootstrap parity gap report\n");
    out.push_str(&format!("schema: {}\n\n", report.schema));
    out.push_str("Axes:\n");
    for axis in &report.axes {
        let status = if axis.complete { "complete" } else { "incomplete" };
        out.push_str(&format!("- {}: {}", axis.axis, status));
        if !axis.blocking_rows.is_empty() {
            out.push_str(&format!(" (blocking: {})", axis.blocking_rows.join(", ")));
        }
        out.push('\n');
    }
    out.push_str("\nRows:\n");
    for row in &report.rows {
        out.push_str(&format!(
            "- {} [{}] axes={} provider={} derivation={}\n",
            row.id,
            row.status,
            row.axes.iter().map(ToString::to_string).collect::<Vec<_>>().join(","),
            row.provider_kind,
            row.derivation.unwrap_or("<none>"),
        ));
        out.push_str(&format!("  graph: {}\n", row.graph_evidence));
        out.push_str(&format!("  semantic: {}\n", row.semantic_evidence));
        out.push_str(&format!("  proof: {}\n", row.proof_evidence));
        if !row.notes.is_empty() {
            out.push_str(&format!("  notes: {}\n", row.notes));
        }
    }
    out
}

fn axis_complete(report: &BootstrapParityReport, axis: ParityAxis) -> bool {
    report.axes.iter().any(|summary| summary.axis == axis && summary.complete)
}

fn summarize_axis(axis: ParityAxis, rows: &[ParityRow]) -> AxisSummary {
    let blocking_rows: Vec<String> = rows
        .iter()
        .filter(|row| row.axes.contains(&axis) && row.status.blocks_parity())
        .map(|row| row.id.to_string())
        .collect();
    AxisSummary {
        axis,
        complete: blocking_rows.is_empty(),
        blocking_rows,
    }
}

fn evaluate_stage(project_root: &Path, spec: &StageSpec) -> ParityRow {
    let path = spec.derivation.map(|derivation| project_root.join(BOOTSTRAP_DIR).join(derivation));
    let file_state = path.as_ref().map(|p| inspect_derivation(p));
    let evidence_failure =
        path.as_ref().and_then(|p| validate_stage_evidence(project_root, p, spec.evidence_check).err());
    let status = match (spec.expected_complete, file_state) {
        (_, None) => StageStatus::Blocked,
        (_, Some(FileState::Missing)) => StageStatus::NotStarted,
        (_, Some(FileState::Present)) if evidence_failure.is_some() => StageStatus::Partial,
        (false, Some(FileState::Present)) => StageStatus::Partial,
        (false, Some(FileState::Placeholder)) => StageStatus::Placeholder,
        (true, Some(FileState::Present)) => StageStatus::Complete,
        (true, Some(FileState::Placeholder)) => StageStatus::Placeholder,
        (true, Some(FileState::Unreadable)) => StageStatus::Blocked,
        (false, Some(FileState::Unreadable)) => StageStatus::Blocked,
    };
    let provider_kind = provider_kind_for(spec, status);
    let notes = row_notes(spec, status, path.as_deref(), evidence_failure.as_deref());
    ParityRow {
        id: spec.id,
        title: spec.title,
        axes: spec.axes.to_vec(),
        lineage: spec.lineage,
        derivation: spec.derivation,
        status,
        provider_kind,
        graph_evidence: spec.graph_evidence,
        semantic_evidence: spec.semantic_evidence,
        proof_evidence: spec.proof_evidence,
        notes,
    }
}

fn provider_kind_for(spec: &StageSpec, status: StageStatus) -> ProviderKind {
    if status.blocks_parity() {
        return ProviderKind::Unknown;
    }
    if spec.axes.contains(&ParityAxis::Stagex) {
        ProviderKind::StagexLineage
    } else if spec.axes.contains(&ParityAxis::Guix) {
        ProviderKind::SourceRoot
    } else {
        ProviderKind::LegacyFetch
    }
}

fn row_notes(spec: &StageSpec, status: StageStatus, path: Option<&Path>, evidence_failure: Option<&str>) -> String {
    let mut notes = Vec::new();
    if !spec.notes.is_empty() {
        notes.push(spec.notes.to_string());
    }
    match (status, path) {
        (StageStatus::NotStarted, Some(path)) => notes.push(format!("missing derivation {}", path.display())),
        (StageStatus::Placeholder, Some(path)) => {
            notes.push(format!("placeholder markers found in {}", path.display()))
        }
        (StageStatus::Blocked, Some(path)) => notes.push(format!("could not inspect {}", path.display())),
        (StageStatus::Blocked, None) => {
            notes.push("no derivation is expected; row waits on external proof/evidence".to_string())
        }
        _ => {}
    }
    if let Some(reason) = evidence_failure {
        notes.push(format!("evidence check failed: {reason}"));
    }
    notes.join("; ")
}

fn validate_stage_evidence(project_root: &Path, path: &Path, check: EvidenceCheck) -> Result<(), String> {
    match check {
        EvidenceCheck::None => Ok(()),
        EvidenceCheck::SeedFullSourceRootContract => validate_seed_full_source_root_contract(path),
        EvidenceCheck::BinutilsTccToolTranscript => validate_binutils_tcc_tool_transcript(project_root),
    }
}

fn validate_seed_full_source_root_contract(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    let required = [
        "share/crunch-bootstrap/provider.json",
        "\"provider_id\": \"full-source-v1\"",
        "\"target\": \"x86_64-linux-musl\"",
        "\"dynamic_linker\": \"ld-musl-x86_64.so.1\"",
        "\"source_root\"",
        "\"manifest_digest\"",
        "\"reduction\"",
        "\"retained_tools\"",
        "x86_64-linux-musl-gcc",
        "x86_64-linux-musl-as",
        "x86_64-linux-musl-ld",
        "x86_64-linux-musl-include",
        "libc.a",
    ];
    for needle in required {
        if !content.contains(needle) {
            return Err(format!("seed-full source-root contract missing `{needle}`"));
        }
    }
    let forbidden = [
        "musl.cc/x86_64",
        "LEGACY_MUSL_CC_URL",
        "LEGACY_MUSL_CC_HASH",
        "raw = {",
        "\"raw\"",
    ];
    for needle in forbidden {
        if content.contains(needle) {
            return Err(format!("seed-full source-root contract contains legacy provider marker `{needle}`"));
        }
    }
    Ok(())
}

fn validate_binutils_tcc_tool_transcript(project_root: &Path) -> Result<(), String> {
    let path = project_root.join(BINUTILS_TCC_TOOL_TRANSCRIPT);
    let content = fs::read_to_string(&path).map_err(|err| {
        format!(
            "binutils-tcc tool transcript missing `{}` ({err}); expected JSON fields: schema, derivation, output_path, provider_kind, host_fallback, fallback_markers, and tool_smokes for as/ld/ar/ranlib/nm/objcopy",
            BINUTILS_TCC_TOOL_TRANSCRIPT
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|err| format!("binutils-tcc tool transcript is not valid JSON: {err}"))?;
    require_json_string(&value, "schema", "crunch-binutils-tcc-tool-smoke-v1")?;
    require_json_string(&value, "derivation", "bootstrap/binutils-tcc.ncl")?;
    require_non_empty_json_string(&value, "output_path")?;
    require_non_empty_json_string(&value, "provider_kind")?;
    require_json_bool(&value, "host_fallback", false)?;
    require_empty_array(&value, "fallback_markers")?;
    let tool_smokes = value
        .get("tool_smokes")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "binutils-tcc tool transcript missing object field `tool_smokes`".to_string())?;
    for tool in BINUTILS_TCC_REQUIRED_TOOLS {
        let smoke = tool_smokes
            .get(*tool)
            .and_then(|v| v.as_object())
            .ok_or_else(|| format!("binutils-tcc tool transcript missing `tool_smokes.{tool}`"))?;
        require_object_string(smoke, "path")?;
        require_object_string(smoke, "command")?;
        let exit_status = smoke
            .get("exit_status")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| format!("binutils-tcc transcript `tool_smokes.{tool}.exit_status` must be an integer"))?;
        if exit_status != 0 {
            return Err(format!(
                "binutils-tcc transcript `tool_smokes.{tool}.exit_status` is {exit_status}, expected 0"
            ));
        }
    }
    Ok(())
}

fn require_json_string(value: &serde_json::Value, field: &str, expected: &str) -> Result<(), String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("binutils-tcc transcript missing string field `{field}`"))?;
    if actual != expected {
        return Err(format!("binutils-tcc transcript `{field}` is `{actual}`, expected `{expected}`"));
    }
    Ok(())
}

fn require_non_empty_json_string(value: &serde_json::Value, field: &str) -> Result<(), String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("binutils-tcc transcript missing string field `{field}`"))?;
    if actual.trim().is_empty() {
        return Err(format!("binutils-tcc transcript `{field}` must not be empty"));
    }
    Ok(())
}

fn require_json_bool(value: &serde_json::Value, field: &str, expected: bool) -> Result<(), String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_bool())
        .ok_or_else(|| format!("binutils-tcc transcript missing bool field `{field}`"))?;
    if actual != expected {
        return Err(format!("binutils-tcc transcript `{field}` is {actual}, expected {expected}"));
    }
    Ok(())
}

fn require_empty_array(value: &serde_json::Value, field: &str) -> Result<(), String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("binutils-tcc transcript missing array field `{field}`"))?;
    if !actual.is_empty() {
        return Err(format!("binutils-tcc transcript `{field}` must be empty when host_fallback=false"));
    }
    Ok(())
}

fn require_object_string(object: &serde_json::Map<String, serde_json::Value>, field: &str) -> Result<(), String> {
    let actual = object
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("binutils-tcc transcript smoke missing string field `{field}`"))?;
    if actual.trim().is_empty() {
        return Err(format!("binutils-tcc transcript smoke field `{field}` must not be empty"));
    }
    Ok(())
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum FileState {
    Missing,
    Present,
    Placeholder,
    Unreadable,
}

/// Check if a marker occurs as a standalone word (bounded by non-word
/// characters: non-alphanumeric, non-underscore, non-dash, non-quote,
/// and not a path separator) to avoid false positives from substrings like
/// `stub/atan2.c`, `stub-objc.c`, `placeholders` in comment prose.
/// Also rejects heredoc delimiters (`'STUB'`) and heredoc terminators
/// (`STUB` on a line with only optional whitespace around it).
fn marker_is_standalone(content: &str, marker: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    let search = marker.to_ascii_lowercase();
    let mut start = 0;
    while let Some(pos) = lower[start..].find(&search) {
        let abs = start + pos;
        let end = abs + search.len();
        // Word boundary check: reject if adjacent to alphanumeric, underscore,
        // dot, dash, or forward slash.
        let prev_is_word = abs > 0
            && (lower[..abs].chars().last().unwrap().is_ascii_alphanumeric()
                || lower[..abs].chars().last().unwrap() == '_'
                || lower[..abs].chars().last().unwrap() == '.'
                || lower[..abs].chars().last().unwrap() == '-');
        let next_is_word = end < lower.len()
            && (lower[end..].chars().next().unwrap().is_ascii_alphanumeric()
                || lower[end..].chars().next() == Some('_')
                || lower[end..].chars().next() == Some('/')
                || lower[end..].chars().next() == Some('-'));
        // Reject quote-bounded matches: `'STUB'` is a heredoc delimiter,
        // not a placeholder marker.
        let prev_is_quote = abs > 0 && lower[..abs].chars().last().unwrap() == '\'';
        let next_is_quote = end < lower.len() && lower[end..].chars().next() == Some('\'');
        // Reject heredoc terminators: marker where everything before it
        // on the line is whitespace/newline, and everything after it on
        // the line is whitespace/newline (marker is the sole non-whitespace).
        // Find the start of the current line.
        let line_start = lower[..abs].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_before = &lower[line_start..abs];
        let prev_line_is_blank = line_before.chars().all(|c| c == ' ' || c == '\t');
        let after = &lower[end..];
        let next_line_is_blank = after.chars().take_while(|c| *c != '\n').all(|c| c == ' ' || c == '\t');
        let is_heredoc_terminator = prev_line_is_blank && next_line_is_blank;
        // Reject if any of these are word characters, quote-bounded,
        // or a heredoc terminator.
        if !prev_is_word && !next_is_word && !(prev_is_quote && next_is_quote) && !is_heredoc_terminator {
            return true;
        }
        start = abs + 1;
    }
    false
}

fn inspect_derivation(path: &Path) -> FileState {
    let Ok(content) = fs::read_to_string(path) else {
        return if path.exists() {
            FileState::Unreadable
        } else {
            FileState::Missing
        };
    };
    if PLACEHOLDER_MARKERS.iter().any(|marker| marker_is_standalone(&content, marker)) {
        FileState::Placeholder
    } else {
        FileState::Present
    }
}

const ALL_AXES: &[ParityAxis] = &[ParityAxis::LiveBootstrap, ParityAxis::Guix, ParityAxis::Stagex];
const LIVE_GUIX: &[ParityAxis] = &[ParityAxis::LiveBootstrap, ParityAxis::Guix];
const GUIX_STAGEX: &[ParityAxis] = &[ParityAxis::Guix, ParityAxis::Stagex];
const GUIX_ONLY: &[ParityAxis] = &[ParityAxis::Guix];
const STAGEX_ONLY: &[ParityAxis] = &[ParityAxis::Stagex];

fn parity_stage_specs() -> &'static [StageSpec] {
    &[
        StageSpec {
            id: "seed.hex0",
            title: "Audited hex0 seed",
            axes: ALL_AXES,
            lineage: "stagex",
            derivation: Some("stage0-posix.ncl"),
            expected_complete: true,
            graph_evidence: "stage0-posix derivation present",
            semantic_evidence: "seed size/source audit required",
            proof_evidence: "stagex lineage digest required",
            notes: "root trust is environmental until audited-seed proof is bound",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "stage0.transition-tools",
            title: "M0/M1/hex2/kaem transition tools",
            axes: ALL_AXES,
            lineage: "live-bootstrap",
            derivation: Some("stage0-posix.ncl"),
            expected_complete: true,
            graph_evidence: "stage0-posix derivation present",
            semantic_evidence: "stage0 toolchain smoke required",
            proof_evidence: "stage graph digest required",
            notes: "covers stage0-posix handoff rather than host compiler trust",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "mes",
            title: "Mes bootstrap",
            axes: ALL_AXES,
            lineage: "live-bootstrap",
            derivation: Some("mes.ncl"),
            expected_complete: true,
            graph_evidence: "mes derivation present",
            semantic_evidence: "mes runtime smoke required",
            proof_evidence: "source transcript required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "tinycc.mes",
            title: "TinyCC Mes-linked handoff",
            axes: ALL_AXES,
            lineage: "live-bootstrap",
            derivation: Some("tinycc-mes.ncl"),
            expected_complete: true,
            graph_evidence: "tinycc-mes derivation present",
            semantic_evidence: "version/object smoke required",
            proof_evidence: "source transcript required",
            notes: "Mes runtime defects remain important gap-report details",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "tinycc.0.9.27",
            title: "TinyCC 0.9.27 handoff",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("tinycc.ncl"),
            expected_complete: true,
            graph_evidence: "tinycc derivation present",
            semantic_evidence: "compile/link smoke required",
            proof_evidence: "source transcript required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "tcc-musl-prep",
            title: "TCC musl preparation",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("tcc-musl-prep.ncl"),
            expected_complete: true,
            graph_evidence: "tcc-musl-prep derivation present",
            semantic_evidence: "runtime validation required",
            proof_evidence: "source transcript required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "musl.tcc",
            title: "musl 1.1.24 via TCC",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("musl-1.1.24-tcc.ncl"),
            expected_complete: true,
            graph_evidence: "musl-tcc derivation present",
            semantic_evidence: "libc startup smoke required",
            proof_evidence: "source transcript required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "tcc-musl",
            title: "TCC on musl",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("tcc-musl.ncl"),
            expected_complete: true,
            graph_evidence: "tcc-musl derivation present",
            semantic_evidence: "runtime validation required",
            proof_evidence: "source transcript required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "make.3.82",
            title: "GNU Make 3.82",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("make-tcc.ncl"),
            expected_complete: true,
            graph_evidence: "make-tcc derivation present",
            semantic_evidence: "real recipe smoke required",
            proof_evidence: "source transcript required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "patch.2.5.9",
            title: "patch 2.5.9",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("patch-tcc.ncl"),
            expected_complete: true,
            graph_evidence: "patch-tcc derivation present",
            semantic_evidence: "runtime validation required",
            proof_evidence: "source transcript required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "grep.2.4",
            title: "grep 2.4",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("grep-2.4-musl.ncl"),
            expected_complete: true,
            graph_evidence: "grep derivation present",
            semantic_evidence: "runtime validation required",
            proof_evidence: "source transcript required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "sed.4.0.9",
            title: "sed 4.0.9",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("sed-4.0.9-musl.ncl"),
            expected_complete: true,
            graph_evidence: "sed derivation present",
            semantic_evidence: "runtime validation required",
            proof_evidence: "source transcript required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "archive-tools",
            title: "bzip2/gzip/tar archive tools",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("tar-tcc.ncl"),
            expected_complete: true,
            graph_evidence: "tar plus bzip2/gzip derivations present",
            semantic_evidence: "archive round-trip smokes required",
            proof_evidence: "source transcript required",
            notes: "gap report also checks bzip2/gzip rows through explicit derivations",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "coreutils.5",
            title: "coreutils 5.0",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("coreutils-5.0-musl.ncl"),
            expected_complete: true,
            graph_evidence: "coreutils 5.0 derivation present",
            semantic_evidence: "runtime validation required",
            proof_evidence: "source transcript required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "coreutils.6",
            title: "coreutils 6.10",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("coreutils-6.10-musl.ncl"),
            expected_complete: true,
            graph_evidence: "coreutils 6.10 derivation present",
            semantic_evidence: "runtime validation required",
            proof_evidence: "source transcript required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "parser-generators",
            title: "bison/flex/oyacc parser generators",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("bison-3.4.1-musl.ncl"),
            expected_complete: true,
            graph_evidence: "bison plus flex/oyacc derivations present",
            semantic_evidence: "generator smoke required",
            proof_evidence: "source transcript required",
            notes: "gap report also requires flex and oyacc derivations in repository",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "autotools",
            title: "m4/libtool/autoconf/automake ladder",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("autoconf-2.69.ncl"),
            expected_complete: true,
            graph_evidence: "autotools terminal derivation present",
            semantic_evidence: "configure-generation smokes required",
            proof_evidence: "source transcript required",
            notes: "intermediate autoconf/automake versions remain part of map",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "perl.ladder",
            title: "Perl ladder",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("perl-5.6.2-musl.ncl"),
            expected_complete: true,
            graph_evidence: "Perl terminal derivation present",
            semantic_evidence: "Perl smoke required",
            proof_evidence: "source transcript required",
            notes: "Perl 5.000 through 5.6.2 are mapped",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "math-libs",
            title: "GMP/MPFR/MPC",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("mpc-1.2.1.ncl"),
            expected_complete: true,
            graph_evidence: "MPC terminal derivation present",
            semantic_evidence: "library link smokes required",
            proof_evidence: "source transcript required",
            notes: "MPC upstream heading mismatch remains documented",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "binutils.tcc",
            title: "binutils TCC bridge",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("binutils-tcc.ncl"),
            expected_complete: false,
            graph_evidence: "binutils-tcc derivation present",
            semantic_evidence: "checked binutils-tcc tool transcript required for as/ld/ar/ranlib/nm/objcopy",
            proof_evidence: "source transcript plus no-host-fallback markers required",
            notes: "bridge/omitted-member output must not count as full parity; transcript at bootstrap/evidence/binutils-tcc-tool-smoke.json records schema, derivation, output_path, provider_kind, host_fallback=false, fallback_markers=[], and per-tool smoke exit statuses; row remains partial until native/full-source binutils correctness is proven",
            evidence_check: EvidenceCheck::BinutilsTccToolTranscript,
        },
        StageSpec {
            id: "gcc.4.0",
            title: "GCC 4.0",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("gcc-4.0.ncl"),
            expected_complete: false,
            graph_evidence: "late graph completion recorded",
            semantic_evidence: "bounded libgcc smokes only; native compiler correctness not proven",
            proof_evidence: "source transcript required",
            notes: "pass1 bridge and selected libgcc members are partial progress",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "gcc.4.7",
            title: "GCC 4.7",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("gcc-4.7.ncl"),
            expected_complete: false,
            graph_evidence: "derivation present",
            semantic_evidence: "native correctness evidence required",
            proof_evidence: "source transcript required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "gcc.10",
            title: "GCC 10",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("gcc-10.ncl"),
            expected_complete: false,
            graph_evidence: "derivation present",
            semantic_evidence: "native correctness evidence required",
            proof_evidence: "source transcript required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "full-musl-binutils",
            title: "Full musl/binutils handoff",
            axes: LIVE_GUIX,
            lineage: "guix",
            derivation: Some("binutils-full.ncl"),
            expected_complete: false,
            graph_evidence: "binutils-full plus musl-full derivations present",
            semantic_evidence: "full toolchain smokes required",
            proof_evidence: "source-root proof required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "seed-full",
            title: "Normalized full source seed provider",
            axes: GUIX_ONLY,
            lineage: "guix",
            derivation: Some("seed-full.ncl"),
            expected_complete: true,
            graph_evidence: "seed-full derivation present",
            semantic_evidence: "source-root provider contract validation present",
            proof_evidence: "provider digest/transcript remains required before broader Guix parity",
            notes: "source-root provider contract evidence only; does not satisfy StageX lineage evidence",
            evidence_check: EvidenceCheck::SeedFullSourceRootContract,
        },
        StageSpec {
            id: "seed-full.stagex-lineage",
            title: "StageX lineage normalized seed provider",
            axes: STAGEX_ONLY,
            lineage: "stagex",
            derivation: None,
            expected_complete: false,
            graph_evidence: "StageX lineage provider derivation/proof not yet bound",
            semantic_evidence: "audited lineage provider contract validation required",
            proof_evidence: "lineage provider digest and transcript required",
            notes: "Guix source-root seed-full evidence must not satisfy this StageX row",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "selftest",
            title: "Bootstrap selftest",
            axes: GUIX_STAGEX,
            lineage: "crunch",
            derivation: Some("selftest.ncl"),
            expected_complete: true,
            graph_evidence: "selftest derivation present",
            semantic_evidence: "bootstrap selftest transcript required",
            proof_evidence: "proof bundle digest required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "integration-test",
            title: "Bootstrap integration test",
            axes: GUIX_STAGEX,
            lineage: "crunch",
            derivation: Some("integration-test.ncl"),
            expected_complete: true,
            graph_evidence: "integration-test derivation present",
            semantic_evidence: "integration transcript required",
            proof_evidence: "proof bundle digest required",
            notes: "",
            evidence_check: EvidenceCheck::None,
        },
        StageSpec {
            id: "crunch.self-build",
            title: "Crunch self-build proof",
            axes: GUIX_STAGEX,
            lineage: "crunch",
            derivation: Some("crunch.ncl"),
            expected_complete: false,
            graph_evidence: "crunch derivation present",
            semantic_evidence: "stage1/stage2 binary comparison required",
            proof_evidence: "full proof bundle required",
            notes: "release evidence must bind selected provider kind",
            evidence_check: EvidenceCheck::None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn write_stage(root: &Path, name: &str, content: &str) {
        let bootstrap = root.join(BOOTSTRAP_DIR);
        fs::create_dir_all(&bootstrap).unwrap();
        fs::write(bootstrap.join(name), content).unwrap();
    }

    fn seed_full_spec() -> StageSpec {
        StageSpec {
            id: "seed-full",
            title: "Normalized full source seed provider",
            axes: GUIX_ONLY,
            lineage: "guix",
            derivation: Some("seed-full.ncl"),
            expected_complete: true,
            graph_evidence: "seed-full derivation present",
            semantic_evidence: "source-root provider contract validation present",
            proof_evidence: "provider digest/transcript remains required before broader Guix parity",
            notes: "source-root provider contract evidence only; does not satisfy StageX lineage evidence",
            evidence_check: EvidenceCheck::SeedFullSourceRootContract,
        }
    }

    fn binutils_tcc_spec() -> StageSpec {
        StageSpec {
            id: "binutils.tcc",
            title: "binutils TCC bridge",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("binutils-tcc.ncl"),
            expected_complete: false,
            graph_evidence: "binutils-tcc derivation present",
            semantic_evidence: "checked binutils-tcc tool transcript required for as/ld/ar/ranlib/nm/objcopy",
            proof_evidence: "source transcript plus no-host-fallback markers required",
            notes: "bridge/omitted-member output must not count as full parity; transcript at bootstrap/evidence/binutils-tcc-tool-smoke.json records schema, derivation, output_path, provider_kind, host_fallback=false, fallback_markers=[], and per-tool smoke exit statuses; row remains partial until native/full-source binutils correctness is proven",
            evidence_check: EvidenceCheck::BinutilsTccToolTranscript,
        }
    }

    fn write_binutils_tcc_transcript(root: &Path, extra: &str) {
        let path = root.join(BINUTILS_TCC_TOOL_TRANSCRIPT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            format!(
                r#"{{
  "schema": "crunch-binutils-tcc-tool-smoke-v1",
  "derivation": "bootstrap/binutils-tcc.ncl",
  "output_path": "/crunch/store/example-binutils-2.30-tcc",
  "provider_kind": "source-root",
  "host_fallback": false,
  "fallback_markers": [],
  "tool_smokes": {{
    "as": {{ "path": "/crunch/store/example-binutils-2.30-tcc/bin/as", "command": "as --version", "exit_status": 0 }},
    "ld": {{ "path": "/crunch/store/example-binutils-2.30-tcc/bin/ld", "command": "ld --version", "exit_status": 0 }},
    "ar": {{ "path": "/crunch/store/example-binutils-2.30-tcc/bin/ar", "command": "ar --version", "exit_status": 0 }},
    "ranlib": {{ "path": "/crunch/store/example-binutils-2.30-tcc/bin/ranlib", "command": "ranlib --version", "exit_status": 0 }},
    "nm": {{ "path": "/crunch/store/example-binutils-2.30-tcc/bin/nm", "command": "nm --version", "exit_status": 0 }},
    "objcopy": {{ "path": "/crunch/store/example-binutils-2.30-tcc/bin/objcopy", "command": "objcopy --version", "exit_status": 0 }}
  }}
{extra}}}"#
            ),
        )
        .unwrap();
    }

    fn valid_seed_full_contract() -> &'static str {
        r#"
        share/crunch-bootstrap/provider.json
        "provider_id": "full-source-v1"
        "target": "x86_64-linux-musl"
        "dynamic_linker": "ld-musl-x86_64.so.1"
        "source_root" { "manifest_digest": "pending" }
        "reduction" { "retained_tools": [
          "x86_64-linux-musl-gcc", "x86_64-linux-musl-as", "x86_64-linux-musl-ld",
          "x86_64-linux-musl-include"
        ] }
        libc.a
        "#
    }

    #[test]
    fn missing_derivation_becomes_not_started() {
        let dir = tempdir().unwrap();
        let spec = StageSpec {
            id: "missing",
            title: "Missing",
            axes: &[ParityAxis::LiveBootstrap],
            lineage: "test",
            derivation: Some("missing.ncl"),
            expected_complete: true,
            graph_evidence: "graph",
            semantic_evidence: "semantic",
            proof_evidence: "proof",
            notes: "",
            evidence_check: EvidenceCheck::None,
        };
        let row = evaluate_stage(dir.path(), &spec);
        assert_eq!(row.status, StageStatus::NotStarted);
        assert!(row.notes.contains("missing derivation"));
    }

    #[test]
    fn placeholder_marker_blocks_parity() {
        let dir = tempdir().unwrap();
        write_stage(dir.path(), "placeholder.ncl", "# TODO placeholder\n");
        let spec = StageSpec {
            id: "placeholder",
            title: "Placeholder",
            axes: &[ParityAxis::LiveBootstrap],
            lineage: "test",
            derivation: Some("placeholder.ncl"),
            expected_complete: true,
            graph_evidence: "graph",
            semantic_evidence: "semantic",
            proof_evidence: "proof",
            notes: "",
            evidence_check: EvidenceCheck::None,
        };
        let row = evaluate_stage(dir.path(), &spec);
        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.status.blocks_parity());
    }

    #[test]
    fn axis_summary_reports_blocking_rows() {
        let rows = vec![
            ParityRow {
                id: "complete",
                title: "Complete",
                axes: vec![ParityAxis::LiveBootstrap],
                lineage: "test",
                derivation: None,
                status: StageStatus::Complete,
                provider_kind: ProviderKind::LegacyFetch,
                graph_evidence: "graph",
                semantic_evidence: "semantic",
                proof_evidence: "proof",
                notes: String::new(),
            },
            ParityRow {
                id: "blocked",
                title: "Blocked",
                axes: vec![ParityAxis::LiveBootstrap],
                lineage: "test",
                derivation: None,
                status: StageStatus::Partial,
                provider_kind: ProviderKind::Unknown,
                graph_evidence: "graph",
                semantic_evidence: "semantic",
                proof_evidence: "proof",
                notes: String::new(),
            },
        ];
        let summary = summarize_axis(ParityAxis::LiveBootstrap, &rows);
        assert!(!summary.complete);
        assert_eq!(summary.blocking_rows, vec!["blocked".to_string()]);
    }

    #[test]
    fn seed_full_source_root_contract_completes_guix_row() {
        let dir = tempdir().unwrap();
        write_stage(dir.path(), "seed-full.ncl", valid_seed_full_contract());

        let row = evaluate_stage(dir.path(), &seed_full_spec());

        assert_eq!(row.status, StageStatus::Complete);
        assert_eq!(row.provider_kind, ProviderKind::SourceRoot);
        assert_eq!(row.axes, vec![ParityAxis::Guix]);
        assert!(!row.axes.contains(&ParityAxis::Stagex));
    }

    #[test]
    fn seed_full_legacy_metadata_blocks_guix_row() {
        let dir = tempdir().unwrap();
        write_stage(dir.path(), "seed-full.ncl", &format!("{}\nraw = {{}}\n", valid_seed_full_contract()));

        let row = evaluate_stage(dir.path(), &seed_full_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert_eq!(row.provider_kind, ProviderKind::Unknown);
        assert!(row.notes.contains("legacy provider marker"));
    }

    #[test]
    fn binutils_tcc_without_transcript_remains_partial() {
        let dir = tempdir().unwrap();
        write_stage(dir.path(), "binutils-tcc.ncl", "# real binutils tcc derivation body\n");

        let row = evaluate_stage(dir.path(), &binutils_tcc_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert_eq!(row.provider_kind, ProviderKind::Unknown);
        assert!(row.notes.contains("tool transcript missing"));
        assert!(row.notes.contains(BINUTILS_TCC_TOOL_TRANSCRIPT));
        assert!(row.notes.contains("as/ld/ar/ranlib/nm/objcopy"));
    }

    #[test]
    fn binutils_tcc_valid_transcript_satisfies_evidence_check_but_not_full_parity() {
        let dir = tempdir().unwrap();
        write_stage(dir.path(), "binutils-tcc.ncl", "# real binutils tcc derivation body\n");
        write_binutils_tcc_transcript(dir.path(), "");

        let row = evaluate_stage(dir.path(), &binutils_tcc_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert_eq!(row.provider_kind, ProviderKind::Unknown);
        assert!(!row.notes.contains("evidence check failed"));
    }

    #[test]
    fn binutils_tcc_real_derivation_reports_evidence_backed_partial() {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let report = collect_bootstrap_parity_report(project_root);
        let row = report.rows.iter().find(|row| row.id == "binutils.tcc").unwrap();

        assert_eq!(row.status, StageStatus::Partial);
        assert_eq!(row.provider_kind, ProviderKind::Unknown);
        assert!(row.status.blocks_parity());
        assert!(!row.notes.contains("placeholder markers found"));
        assert!(!row.notes.contains("evidence check failed"));
    }

    #[test]
    fn binutils_tcc_transcript_rejects_host_fallback() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(BINUTILS_TCC_TOOL_TRANSCRIPT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            r#"{
              "schema": "crunch-binutils-tcc-tool-smoke-v1",
              "derivation": "bootstrap/binutils-tcc.ncl",
              "output_path": "/crunch/store/example-binutils-2.30-tcc",
              "provider_kind": "source-root",
              "host_fallback": true,
              "fallback_markers": ["/usr/bin/as"],
              "tool_smokes": {}
            }"#,
        )
        .unwrap();

        let err = validate_binutils_tcc_tool_transcript(dir.path()).unwrap_err();

        assert!(err.contains("host_fallback"));
    }

    #[test]
    fn stagex_lineage_seed_provider_remains_separate_blocker() {
        let dir = tempdir().unwrap();
        write_stage(dir.path(), "seed-full.ncl", valid_seed_full_contract());

        let report = collect_bootstrap_parity_report(dir.path());
        let seed_full = report.rows.iter().find(|row| row.id == "seed-full").unwrap();
        let stagex_seed = report.rows.iter().find(|row| row.id == "seed-full.stagex-lineage").unwrap();
        let stagex = report.axes.iter().find(|axis| axis.axis == ParityAxis::Stagex).unwrap();

        assert_eq!(seed_full.status, StageStatus::Complete);
        assert_eq!(seed_full.axes, vec![ParityAxis::Guix]);
        assert_eq!(stagex_seed.status, StageStatus::Blocked);
        assert!(stagex.blocking_rows.contains(&"seed-full.stagex-lineage".to_string()));
        assert!(!stagex.blocking_rows.contains(&"seed-full".to_string()));
    }

    #[test]
    fn marker_is_standalone_rejects_substring_in_path() {
        // `stub/atan2.c` must NOT trigger "stub"
        assert!(!marker_is_standalone("stub/atan2.c", "stub"));
    }

    #[test]
    fn marker_is_standalone_rejects_plural_word() {
        // `placeholders` must NOT trigger "placeholder"
        assert!(!marker_is_standalone("# printf format placeholders here", "placeholder"));
    }

    #[test]
    fn marker_is_standalone_accepts_standalone_word() {
        assert!(marker_is_standalone("void placeholder(void) {}", "placeholder"));
    }

    #[test]
    fn marker_is_standalone_accepts_comment_marker() {
        assert!(marker_is_standalone("# placeholder marker", "placeholder"));
        assert!(marker_is_standalone("// pass1 bridge", "pass1 bridge"));
    }

    #[test]
    fn marker_is_standalone_accepts_escaped_quote() {
        // `\"stub\"` should match because `"` is non-alphanumeric
        assert!(marker_is_standalone("has a \"stub\" section", "stub"));
    }

    #[test]
    fn tinycc_mes_stub_path_is_false_positive() {
        // Reproduce the tinycc-mes.ncl false positive: stub/atan2.c paths
        // should NOT cause the file to be classified as placeholder
        let content = r#"
          stub/atan2.c stub/bsearch.c
          stub/cos.c stub/ctime.c
        "#;
        assert!(!marker_is_standalone(content, "stub"));
    }

    #[test]
    fn tcc_placeholder_comment_is_false_positive() {
        // Reproduce tinycc.ncl false positive: "placeholders" in comment
        let content = "arguments instead of printing literal `%s:%d placeholders.\n";
        assert!(!marker_is_standalone(content, "placeholder"));
    }

    #[test]
    fn dash_compound_path_rejects_stub() {
        // Reproduce gcc-4.0.ncl false positive: /stub-objc.c path component
        // should NOT cause the file to be classified as placeholder
        let content = "    */stub-objc.c|stub-objc.c) saw_stub_objc_src=1 ;;";
        assert!(!marker_is_standalone(content, "stub"));
    }

    #[test]
    fn heredoc_delimiter_rejects_stub() {
        // Heredoc delimiter `'STUB'` (quote-bounded) is NOT a standalone
        // placeholder marker — shell syntax should not count.
        let content = "      $BB cat > lib/getdate_stub.c << 'STUB'\n#include <time.h>\ntime_t get_date(const char *p, const time_t *now) { return -1; }\nSTUB\n";
        assert!(!marker_is_standalone(content, "stub"));
    }

    #[test]
    fn heredoc_terminator_on_own_line_rejected() {
        // `STUB` at the start of its own line (heredoc terminator) is
        // rejected, but `stub` in `# stub` on the prior line is
        // a standalone word and IS accepted.
        let content = "# stub\nSTUB\n";
        assert!(marker_is_standalone(content, "stub"));
    }

    #[test]
    fn tar_tcc_heredoc_delimiter_and_terminator_rejected() {
        // Reproduce the tar-tcc.ncl false positive: `<< 'STUB'` and
        // `STUB` heredoc delimiter/terminator must NOT cause the
        // file to be classified as having placeholder markers.
        // The `stub` substring inside `getdate_stub.c` is also
        // rejected because it's adjacent to `_` and `.`.
        let content = "      $BB cat > lib/getdate_stub.c << 'STUB'\n#include <time.h>\ntime_t get_date(const char *p, const time_t *now) { return -1; }\nSTUB\n";
        assert!(!marker_is_standalone(content, "stub"));
    }
}
