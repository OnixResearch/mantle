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
}

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
    let evidence_failure = path.as_ref().and_then(|p| validate_stage_evidence(p, spec.evidence_check).err());
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

fn validate_stage_evidence(path: &Path, check: EvidenceCheck) -> Result<(), String> {
    match check {
        EvidenceCheck::None => Ok(()),
        EvidenceCheck::SeedFullSourceRootContract => validate_seed_full_source_root_contract(path),
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum FileState {
    Missing,
    Present,
    Placeholder,
    Unreadable,
}

fn inspect_derivation(path: &Path) -> FileState {
    let Ok(content) = fs::read_to_string(path) else {
        return if path.exists() {
            FileState::Unreadable
        } else {
            FileState::Missing
        };
    };
    let lower = content.to_ascii_lowercase();
    if PLACEHOLDER_MARKERS.iter().any(|marker| lower.contains(&marker.to_ascii_lowercase())) {
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
            semantic_evidence: "assembler/linker smokes required",
            proof_evidence: "source transcript required",
            notes: "placeholder/bridge output must not count as full parity",
            evidence_check: EvidenceCheck::None,
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
}
