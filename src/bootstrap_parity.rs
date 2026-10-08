// machine-artifact-public: bootstrap.parity-reports
use std::fmt;
use std::fs;
use std::path::Path;

use clap::ValueEnum;
use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::CommandFamily;
use mantle_application_contract::EffectId;
use mantle_application_contract::EffectKind;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::EffectSpec;
use mantle_application_contract::ExpectedOutput;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::classify_observations;
use mantle_application_contract::plan_effects;
use serde::Serialize;

use crate::errors::RunError;

const REPORT_SCHEMA: &str = "crunch-bootstrap-parity-gap-report-v1";
const PARITY_READ_EFFECT: &str = "bootstrap-parity-evidence-read";
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
    FullSource,
    SourceRoot,
    StagexLineage,
    Unknown,
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LegacyFetch => write!(f, "legacy-fetch"),
            Self::FullSource => write!(f, "full-source"),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_details: Option<ParityProofDetails>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParityProofDetails {
    pub schema: String,
    pub release_id: String,
    pub selected_provider_kind: ProviderKind,
    pub evidence_digest_blake3: String,
    pub deterministic_proof_digest_blake3: String,
    pub rebuild_descriptor_blake3: String,
    pub rebuild_authority_plan_blake3: String,
    pub sandbox_evidence_digest_blake3: String,
    pub verify_receipt_digest_blake3: String,
    pub summary_json_digest_blake3: String,
    pub summary_markdown_digest_blake3: String,
    pub verdict: String,
    pub verify_status: String,
    pub sandbox_profile_identity: String,
    pub bounded_claim: String,
}

#[derive(Debug, Clone)]
struct EvidenceValidation {
    proof_details: Option<ParityProofDetails>,
}

impl EvidenceValidation {
    fn empty() -> Self {
        Self { proof_details: None }
    }

    fn with_proof(details: ParityProofDetails) -> Self {
        Self {
            proof_details: Some(details),
        }
    }
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
    #[cfg(test)]
    BinutilsTccToolTranscript,
    EarlyNativeBinutilsRow,
    EarlyNativeGcc40Row,
    FinalNativeGcc47Row,
    FinalNativeGcc10Row,
    FinalNativeMuslBinutilsRow,
    RealSelfBuildProof,
    StagexLineageProviderReceipt,
    Gcc40PlaceholderInventory,
    #[cfg_attr(not(test), allow(dead_code))]
    Gcc47CxxProviderContract,
    #[cfg_attr(not(test), allow(dead_code))]
    Gcc10ProviderContract,
    #[cfg_attr(not(test), allow(dead_code))]
    FullMuslBinutilsProviderContract,
}

/// One bounded parity inventory read; missing evidence remains a domain fact
/// in the report, not an invented successful source receipt.
trait ParityReportReadPort {
    fn collect(&mut self) -> Result<BootstrapParityReport, RunError>;
}

struct LocalParityReport<'a> {
    root: &'a Path,
}

impl ParityReportReadPort for LocalParityReport<'_> {
    fn collect(&mut self) -> Result<BootstrapParityReport, RunError> {
        // The root passes "." so the environment read happens only after
        // the effect plan exists. Preserve the former absolute report paths.
        let current_dir;
        let root = if self.root == Path::new(".") {
            current_dir =
                std::env::current_dir().map_err(|error| RunError::Internal(format!("current_dir: {error}")))?;
            current_dir.as_path()
        } else {
            self.root
        };
        Ok(collect_bootstrap_parity_report(root))
    }
}

#[cfg(test)]
const BINUTILS_TCC_TOOL_TRANSCRIPT: &str = "bootstrap/evidence/binutils-tcc-tool-smoke.json";
const SELF_BUILD_PROVIDER_KIND_LINKAGE_RECEIPT: &str =
    "bootstrap/evidence/crunch-self-build-provider-kind-linkage.json";
const REAL_SELF_BUILD_PROOF_PARITY_RECEIPT: &str = "bootstrap/evidence/real-self-build-proof-parity.json";
const STAGEX_LINEAGE_PROVIDER_RECEIPT: &str = "bootstrap/evidence/stagex-lineage-provider-receipt.json";
const GCC40_PLACEHOLDER_INVENTORY: &str = "bootstrap/evidence/gcc-4.0-placeholder-inventory.json";
const GCC40_NATIVE_BOUNDARY_RECEIPT: &str = "bootstrap/evidence/gcc-4.0-native-boundary.json";
const GCC40_NATIVE_CC1_ARITHMETIC_RECEIPT: &str = "bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json";
const GCC40_NATIVE_GENERATOR_RECEIPT: &str = "bootstrap/evidence/gcc-4.0-native-generator-slice.json";
const GCC40_NATIVE_DEMANGLE_RECEIPT: &str = "bootstrap/evidence/gcc-4.0-native-demangle-slice.json";
const GCC40_NATIVE_CC1_BUILD_FRONTIER_RECEIPT: &str = "bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json";
const GCC40_CPARSE_DIAGNOSTIC_DERIVATION: &str = "bootstrap/diag-gcc40-c-parse-boundary.ncl";
const GCC47_CXX_PROVIDER_CONTRACT: &str = "bootstrap/evidence/gcc-4.7-cxx-provider-contract.json";
const GCC10_PROVIDER_CONTRACT: &str = "bootstrap/evidence/gcc-10-provider-contract.json";
const FULL_MUSL_BINUTILS_PROVIDER_CONTRACT: &str = "bootstrap/evidence/full-musl-binutils-provider-contract.json";
#[cfg(test)]
const BINUTILS_TCC_REQUIRED_TOOLS: &[&str] = &["as", "ld", "ar", "ranlib", "nm", "objcopy"];
const BLAKE3_HEX_LENGTH: usize = 64;
const STAGEX_PROVIDER_ROLE_COUNT: usize = 4;
const STAGEX_NON_CLAIM_COUNT_MIN: usize = 3;
const CLEAN_REBUILD_ROOT_COUNT: usize = 2;

#[derive(Debug, Copy, Clone)]
struct StringFieldExpectation<'a> {
    field: &'a str,
    expected: &'a str,
}

#[derive(Debug, Copy, Clone)]
struct ContractMarkerCheck<'a> {
    receipt_field: &'a str,
    label: &'a str,
    derivation_path: &'a Path,
    derivation_label: &'a str,
}

#[derive(Debug, Copy, Clone)]
struct DerivationMarkerCheck<'a> {
    content: &'a str,
    marker: &'a str,
}

#[derive(Debug, Copy, Clone)]
struct DigestFieldCheck<'a> {
    digest: &'a str,
    field: &'a str,
}

#[derive(Debug, Copy, Clone)]
struct ProviderKindFieldCheck<'a> {
    kind: &'a str,
    field: &'a str,
}

#[derive(Debug, Copy, Clone)]
struct Gcc40SmokeSpec {
    slice: Option<&'static str>,
    required_input_fragments: &'static [&'static str],
    required_output_marker: &'static str,
}

#[derive(Debug, Copy, Clone)]
struct GeneratorOutputCheck {
    generator: &'static str,
    required_fragments: &'static [&'static str],
}

#[derive(Debug, Copy, Clone)]
struct DemangleIoExpectation {
    field: &'static str,
    mangled: &'static str,
    demangled: &'static str,
}

const GCC40_DEMANGLE_IO_EXPECTATIONS: &[DemangleIoExpectation] = &[
    DemangleIoExpectation {
        field: "accepted_inputs",
        mangled: "_ZN3foo3bar3bazEs",
        demangled: "foo::bar::baz(short)",
    },
    DemangleIoExpectation {
        field: "short_regressions",
        mangled: "_ZN3foo3barEs",
        demangled: "foo::bar(short)",
    },
    DemangleIoExpectation {
        field: "flat_short_regressions",
        mangled: "_Z3foos",
        demangled: "foo(short)",
    },
    DemangleIoExpectation {
        field: "long_regressions",
        mangled: "_ZN3foo3bar3bazEl",
        demangled: "foo::bar::baz(long)",
    },
    DemangleIoExpectation {
        field: "long_regressions",
        mangled: "_ZN3foo3barEl",
        demangled: "foo::bar(long)",
    },
    DemangleIoExpectation {
        field: "flat_long_regressions",
        mangled: "_Z3fool",
        demangled: "foo(long)",
    },
    DemangleIoExpectation {
        field: "char_regressions",
        mangled: "_ZN3foo3bar3bazEc",
        demangled: "foo::bar::baz(char)",
    },
    DemangleIoExpectation {
        field: "char_regressions",
        mangled: "_ZN3foo3barEc",
        demangled: "foo::bar(char)",
    },
    DemangleIoExpectation {
        field: "flat_char_regressions",
        mangled: "_Z3fooc",
        demangled: "foo(char)",
    },
    DemangleIoExpectation {
        field: "int_regressions",
        mangled: "_ZN3foo3bar3bazEi",
        demangled: "foo::bar::baz(int)",
    },
    DemangleIoExpectation {
        field: "int_regressions",
        mangled: "_ZN3foo3barEi",
        demangled: "foo::bar(int)",
    },
    DemangleIoExpectation {
        field: "flat_int_regressions",
        mangled: "_Z3fooi",
        demangled: "foo(int)",
    },
    DemangleIoExpectation {
        field: "zero_arg_regressions",
        mangled: "_ZN3foo3bar3bazEv",
        demangled: "foo::bar::baz()",
    },
    DemangleIoExpectation {
        field: "nested_regressions",
        mangled: "_ZN3foo3barEv",
        demangled: "foo::bar()",
    },
    DemangleIoExpectation {
        field: "flat_regressions",
        mangled: "_Z3foov",
        demangled: "foo()",
    },
];

const GCC40_DEMANGLE_REJECTED_INPUTS: &[&str] = &[
    "_ZN3foo3bar3bazEf",
    "_ZN3foo3bar3bazEx",
    "_ZN3foo3bar3baz3quxEs",
    "_ZN3foo3bar3baz3quxEl",
    "_ZN3foo3bar3baz3quxEc",
    "_ZN3foo3bar3baz3quxEi",
    "_ZN3foo3bar3baz3quxEv",
    "_ZN3fooE",
    "_Z3fooss",
    "_ZN3foo3bar3bazEss",
    "not_mangled",
];

const GCC40_DEMANGLE_TRANSCRIPT_FRAGMENTS: &[&str] = &[
    "_ZN3foo3bar3bazEs",
    "foo::bar::baz(short)",
    "_ZN3foo3barEs",
    "foo::bar(short)",
    "_Z3foos",
    "foo(short)",
    "_ZN3foo3bar3bazEl",
    "foo::bar::baz(long)",
    "_ZN3foo3barEl",
    "foo::bar(long)",
    "_Z3fool",
    "foo(long)",
    "_ZN3foo3bar3bazEc",
    "foo::bar::baz(char)",
    "_ZN3foo3barEc",
    "foo::bar(char)",
    "_Z3fooc",
    "foo(char)",
    "_ZN3foo3bar3bazEi",
    "foo::bar::baz(int)",
    "_ZN3foo3barEi",
    "foo::bar(int)",
    "_Z3fooi",
    "foo(int)",
    "_ZN3foo3bar3bazEv",
    "foo::bar::baz()",
    "_ZN3foo3barEv",
    "foo::bar()",
    "_Z3foov",
    "full native cp-demangle",
];

const GCC40_GENERATOR_OUTPUT_CHECKS: &[GeneratorOutputCheck] = &[
    GeneratorOutputCheck {
        generator: "genattrtab",
        required_fragments: &["gcc40_genattrtab_bounded_output_slice", "HAVE_ATTR_enabled"],
    },
    GeneratorOutputCheck {
        generator: "genoutput",
        required_fragments: &["gcc40_genoutput_bounded_output_slice", "GCC40_GENOUTPUT_BOUNDED"],
    },
    GeneratorOutputCheck {
        generator: "genemit",
        required_fragments: &["gcc40_genemit_bounded_output_slice", "GCC40_GENEMIT_BOUNDED"],
    },
    GeneratorOutputCheck {
        generator: "genrecog",
        required_fragments: &["gcc40_genrecog_bounded_output_slice", "GCC40_GENRECOG_BOUNDED"],
    },
    GeneratorOutputCheck {
        generator: "genextract",
        required_fragments: &["gcc40_genextract_bounded_output_slice", "GCC40_GENEXTRACT_BOUNDED"],
    },
];

const GCC40_ARITHMETIC_SMOKE_SPECS: &[Gcc40SmokeSpec] = &[
    Gcc40SmokeSpec {
        slice: None,
        required_input_fragments: &[
            "int mantle_gcc40_pointer_slice(int x)",
            "int value = x;",
            "int *slot = &value;",
            "*slot = x + 4;",
            "return *slot + value;",
        ],
        required_output_marker: "MANTLE-GCC40-NATIVE-CC1-POINTER-DEREF-SLICE-V7",
    },
    Gcc40SmokeSpec {
        slice: Some("arithmetic-control-flow-v1"),
        required_input_fragments: &["int mantle_gcc40_arith_slice", "return y > 7 ? y - 3 : y + 3;"],
        required_output_marker: "MANTLE-GCC40-NATIVE-CC1-ARITHMETIC-SLICE-V1",
    },
    Gcc40SmokeSpec {
        slice: Some("logical-boolean-control-flow-v2"),
        required_input_fragments: &["int mantle_gcc40_logic_slice", "if ((x > 0 && y > 0) || x == y)"],
        required_output_marker: "MANTLE-GCC40-NATIVE-CC1-LOGICAL-SLICE-V2",
    },
    Gcc40SmokeSpec {
        slice: Some("local-variable-assignment-v3"),
        required_input_fragments: &["int mantle_gcc40_local_vars_slice", "int y = x + 1;", "y = y * 3;"],
        required_output_marker: "MANTLE-GCC40-NATIVE-CC1-LOCAL-VARS-SLICE-V3",
    },
    Gcc40SmokeSpec {
        slice: Some("function-call-v4"),
        required_input_fragments: &[
            "int mantle_gcc40_helper(int x)",
            "int mantle_gcc40_function_call_slice(int x)",
            "return mantle_gcc40_helper(x) + 1;",
        ],
        required_output_marker: "MANTLE-GCC40-NATIVE-CC1-FUNCTION-CALL-SLICE-V4",
    },
    Gcc40SmokeSpec {
        slice: Some("array-index-v5"),
        required_input_fragments: &[
            "int mantle_gcc40_array_slice(int x)",
            "int values[2];",
            "values[0] = x;",
            "return values[0] + values[1];",
        ],
        required_output_marker: "MANTLE-GCC40-NATIVE-CC1-ARRAY-INDEX-SLICE-V5",
    },
    Gcc40SmokeSpec {
        slice: Some("struct-field-v6"),
        required_input_fragments: &[
            "struct mantle_gcc40_pair",
            "struct mantle_gcc40_pair pair;",
            "pair.left = x;",
            "return pair.left + pair.right;",
        ],
        required_output_marker: "MANTLE-GCC40-NATIVE-CC1-STRUCT-FIELD-SLICE-V6",
    },
];

pub fn cmd_bootstrap_parity_report(project_root: &Path, require: &[ParityAxis], json: bool) -> Result<(), RunError> {
    let planned_rows = parity_stage_specs().count();
    let plan = plan_effects(CommandFamily::Bootstrap, &[EffectSpec {
        effect_id: PARITY_READ_EFFECT,
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::Identity(REPORT_SCHEMA),
    }])
    .map_err(|error| RunError::Internal(format!("planning bootstrap parity read: {}", error.code())))?;
    let collected = LocalParityReport { root: project_root }.collect();
    let observation = Observation {
        effect_id: EffectId(PARITY_READ_EFFECT.to_string()),
        kind: EffectKind::ReadFiles,
        status: if collected.is_ok() {
            ObservationStatus::Succeeded
        } else {
            ObservationStatus::Failed
        },
        output: collected
            .as_ref()
            .map_or(EffectOutput::None, |report| EffectOutput::Identity(report.schema.to_string())),
        usage: EffectMeasure::Calls(1),
        diagnostics_code: collected.is_err().then(|| "bootstrap-parity-evidence-read-failed".to_string()),
    };
    let outcome = classify_observations(&plan, &[observation]);
    let parity_document = match (outcome, collected) {
        (ApplicationOutcome::Completed, Ok(report)) if report.rows.len() == planned_rows => report,
        (ApplicationOutcome::Failed { .. }, Err(error)) => return Err(error),
        (other, _) => {
            return Err(RunError::Internal(format!("bootstrap parity read observations inconsistent: {other:?}")));
        }
    };
    let failed_requirements: Vec<String> = require
        .iter()
        .filter(|axis| !axis_complete(&parity_document, **axis))
        .map(ToString::to_string)
        .collect();
    if json {
        let rendered = serde_json::to_string_pretty(&parity_document)
            .map_err(|err| RunError::Internal(format!("serializing bootstrap parity report: {err}")))?;
        println!("{rendered}");
    } else {
        println!("{}", render_bootstrap_parity_report(&parity_document));
    }

    if failed_requirements.is_empty() {
        Ok(())
    } else {
        Err(RunError::Reported(1))
    }
}

pub fn collect_bootstrap_parity_report(project_root: &Path) -> BootstrapParityReport {
    let rows: Vec<ParityRow> = parity_stage_specs().map(|spec| evaluate_stage(project_root, spec)).collect();
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
    assert!(out.starts_with("Bootstrap parity gap report\n"));
    assert!(out.contains("\nRows:\n"));
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
    let evidence_result = validate_stage_evidence(project_root, path.as_deref(), spec.evidence_check);
    let evidence_failure = evidence_result.as_ref().err().map(String::as_str);
    let proof_details = evidence_result.as_ref().ok().and_then(|validation| validation.proof_details.clone());
    let status = if spec.evidence_check == EvidenceCheck::RealSelfBuildProof && evidence_failure.is_some() {
        StageStatus::Blocked
    } else {
        match (spec.expected_complete, file_state) {
            (true, None) if spec.evidence_check != EvidenceCheck::None && evidence_failure.is_none() => {
                StageStatus::Complete
            }
            (false, None) if spec.evidence_check != EvidenceCheck::None && evidence_failure.is_none() => {
                StageStatus::Partial
            }
            (_, None) => StageStatus::Blocked,
            (_, Some(FileState::Missing)) => StageStatus::NotStarted,
            (_, Some(FileState::Present)) if evidence_failure.is_some() => StageStatus::Partial,
            (false, Some(FileState::Present)) => StageStatus::Partial,
            (false, Some(FileState::Placeholder))
                if spec.evidence_check == EvidenceCheck::Gcc40PlaceholderInventory && evidence_failure.is_none() =>
            {
                StageStatus::Partial
            }
            (false, Some(FileState::Placeholder)) => StageStatus::Placeholder,
            (true, Some(FileState::Present)) => StageStatus::Complete,
            (true, Some(FileState::Placeholder)) => StageStatus::Placeholder,
            (true, Some(FileState::Unreadable)) => StageStatus::Blocked,
            (false, Some(FileState::Unreadable)) => StageStatus::Blocked,
        }
    };
    let provider_kind = provider_kind_for(spec, status, proof_details.as_ref());
    let notes = row_notes(spec, status, path.as_deref(), evidence_failure, proof_details.as_ref());
    let row = ParityRow {
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
        proof_details,
        notes,
    };
    assert_eq!(row.id, spec.id);
    assert_eq!(row.axes.as_slice(), spec.axes);
    row
}

fn provider_kind_for(
    spec: &StageSpec,
    status: StageStatus,
    proof_details: Option<&ParityProofDetails>,
) -> ProviderKind {
    if let ("crunch.self-build", Some(details)) = (spec.id, proof_details) {
        return details.selected_provider_kind;
    }
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

fn row_notes(
    spec: &StageSpec,
    status: StageStatus,
    path: Option<&Path>,
    evidence_failure: Option<&str>,
    proof_details: Option<&ParityProofDetails>,
) -> String {
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
    if let Some(details) = proof_details {
        notes.push(format!(
            "genuine release rebuild evidence accepted as partial bootstrap-parity evidence for provider_kind={} release_id={} proof_digest={} rebuild_descriptor={} rebuild_authority_plan={} verify_status={}; bounded claim: {}",
            details.selected_provider_kind,
            details.release_id,
            details.deterministic_proof_digest_blake3,
            details.rebuild_descriptor_blake3,
            details.rebuild_authority_plan_blake3,
            details.verify_status,
            details.bounded_claim
        ));
    }
    if let Some(reason) = evidence_failure {
        notes.push(format!("evidence check failed: {reason}"));
    }
    let has_evidence_failure_note = notes.iter().any(|note| note.starts_with("evidence check failed: "));
    let has_proof_detail_note = notes.iter().any(|note| note.contains("genuine release rebuild evidence"));
    assert_eq!(has_evidence_failure_note, evidence_failure.is_some());
    assert_eq!(has_proof_detail_note, proof_details.is_some());
    notes.join("; ")
}

fn validate_stage_evidence(
    project_root: &Path,
    path: Option<&Path>,
    check: EvidenceCheck,
) -> Result<EvidenceValidation, String> {
    match check {
        EvidenceCheck::None => Ok(EvidenceValidation::empty()),
        EvidenceCheck::SeedFullSourceRootContract => validate_seed_full_source_root_contract(
            path.ok_or_else(|| "seed-full source-root contract requires a derivation path".to_string())?,
        )
        .map(|()| EvidenceValidation::empty()),
        #[cfg(test)]
        EvidenceCheck::BinutilsTccToolTranscript => {
            validate_binutils_tcc_tool_transcript(project_root).map(|()| EvidenceValidation::empty())
        }
        EvidenceCheck::EarlyNativeBinutilsRow => {
            crate::early_native_row_receipt_shell::validate_binutils_row(project_root)
                .map(|()| EvidenceValidation::empty())
        }
        EvidenceCheck::EarlyNativeGcc40Row => crate::early_native_row_receipt_shell::validate_gcc40_row(project_root)
            .map(|()| EvidenceValidation::empty()),
        EvidenceCheck::FinalNativeGcc47Row => crate::final_native_row_receipt_shell::validate_gcc47_row(project_root)
            .map(|()| EvidenceValidation::empty()),
        EvidenceCheck::FinalNativeGcc10Row => crate::final_native_row_receipt_shell::validate_gcc10_row(project_root)
            .map(|()| EvidenceValidation::empty()),
        EvidenceCheck::FinalNativeMuslBinutilsRow => {
            crate::final_native_row_receipt_shell::validate_final_musl_binutils_row(project_root)
                .map(|()| EvidenceValidation::empty())
        }
        EvidenceCheck::RealSelfBuildProof => validate_real_self_build_proof_parity_evidence(project_root),
        EvidenceCheck::StagexLineageProviderReceipt => {
            validate_stagex_lineage_provider_receipt(project_root).map(|()| EvidenceValidation::empty())
        }
        EvidenceCheck::Gcc40PlaceholderInventory => validate_gcc40_placeholder_inventory(
            project_root,
            path.ok_or_else(|| "GCC 4.0 placeholder inventory requires a derivation path".to_string())?,
        )
        .map(|()| EvidenceValidation::empty()),
        EvidenceCheck::Gcc47CxxProviderContract => validate_gcc47_cxx_provider_contract(
            project_root,
            path.ok_or_else(|| "GCC 4.7 C++ provider contract requires a derivation path".to_string())?,
        )
        .map(|()| EvidenceValidation::empty()),
        EvidenceCheck::Gcc10ProviderContract => validate_gcc10_provider_contract(
            project_root,
            path.ok_or_else(|| "GCC 10 provider contract requires a derivation path".to_string())?,
        )
        .map(|()| EvidenceValidation::empty()),
        EvidenceCheck::FullMuslBinutilsProviderContract => {
            validate_full_musl_binutils_provider_contract(project_root).map(|()| EvidenceValidation::empty())
        }
    }
}

fn validate_full_musl_binutils_provider_contract(project_root: &Path) -> Result<(), String> {
    let receipt_path = project_root.join(FULL_MUSL_BINUTILS_PROVIDER_CONTRACT);
    let receipt_content = fs::read_to_string(&receipt_path).map_err(|err| {
        format!(
            "full musl/binutils provider contract missing `{}` ({err}); expected checked contract-only receipt for bootstrap/musl-full.ncl and bootstrap/binutils-full.ncl",
            FULL_MUSL_BINUTILS_PROVIDER_CONTRACT
        )
    })?;
    let receipt: serde_json::Value = serde_json::from_str(&receipt_content)
        .map_err(|err| format!("full musl/binutils provider contract is not valid JSON: {err}"))?;

    let label = "full musl/binutils provider contract";
    require_labeled_string(&receipt, label, StringFieldExpectation {
        field: "schema",
        expected: "mantle-full-musl-binutils-provider-contract-v1",
    })?;
    require_labeled_string(&receipt, label, StringFieldExpectation {
        field: "musl_derivation",
        expected: "bootstrap/musl-full.ncl",
    })?;
    require_labeled_string(&receipt, label, StringFieldExpectation {
        field: "binutils_derivation",
        expected: "bootstrap/binutils-full.ncl",
    })?;
    require_labeled_string(&receipt, label, StringFieldExpectation {
        field: "status",
        expected: "contract-only",
    })?;
    require_labeled_string(&receipt, label, StringFieldExpectation {
        field: "parity_effect",
        expected: "evidence-backed partial; does not prove full musl/binutils correctness",
    })?;

    let musl_path = project_root.join("bootstrap/musl-full.ncl");
    validate_contract_markers(&receipt, ContractMarkerCheck {
        receipt_field: "musl_required_markers",
        label: "full musl/binutils provider contract musl",
        derivation_path: &musl_path,
        derivation_label: "bootstrap/musl-full.ncl",
    })?;
    let binutils_path = project_root.join("bootstrap/binutils-full.ncl");
    validate_contract_markers(&receipt, ContractMarkerCheck {
        receipt_field: "binutils_required_markers",
        label: "full musl/binutils provider contract binutils",
        derivation_path: &binutils_path,
        derivation_label: "bootstrap/binutils-full.ncl",
    })?;
    assert_eq!(receipt.get("status").and_then(serde_json::Value::as_str), Some("contract-only"));
    assert!(musl_path.starts_with(project_root));
    Ok(())
}

fn validate_contract_markers(receipt: &serde_json::Value, check: ContractMarkerCheck<'_>) -> Result<(), String> {
    assert!(!check.receipt_field.is_empty());
    assert!(!check.derivation_label.is_empty());
    let derivation_content = fs::read_to_string(check.derivation_path)
        .map_err(|err| format!("read {} derivation {}: {err}", check.label, check.derivation_path.display()))?;
    let markers = receipt
        .get(check.receipt_field)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("{} missing array field `{}`", check.label, check.receipt_field))?;
    if markers.is_empty() {
        return Err(format!("{} `{}` must not be empty", check.label, check.receipt_field));
    }
    for marker_value in markers {
        let marker = marker_value.as_str().ok_or_else(|| format!("{} marker must be a string", check.label))?;
        if marker.trim().is_empty() {
            return Err(format!("{} marker must not be empty", check.label));
        }
        if !derivation_content.contains(marker) {
            return Err(format!("{} marker not found in {}: `{marker}`", check.label, check.derivation_label));
        }
    }
    Ok(())
}

fn validate_gcc10_provider_contract(project_root: &Path, derivation_path: &Path) -> Result<(), String> {
    validate_provider_contract(project_root, derivation_path, ProviderContractSpec {
        receipt_path: GCC10_PROVIDER_CONTRACT,
        label: "GCC 10 provider contract",
        schema: "mantle-gcc10-provider-contract-v1",
        derivation: "bootstrap/gcc-10.ncl",
        parity_effect: "evidence-backed partial; does not prove native/full GCC 10 correctness",
    })
}

fn validate_gcc47_cxx_provider_contract(project_root: &Path, derivation_path: &Path) -> Result<(), String> {
    let receipt_path = project_root.join(GCC47_CXX_PROVIDER_CONTRACT);
    let receipt_content = fs::read_to_string(&receipt_path).map_err(|err| {
        format!(
            "GCC 4.7 C++ provider contract missing `{}` ({err}); expected checked contract-only receipt for bootstrap/gcc-4.7.ncl",
            GCC47_CXX_PROVIDER_CONTRACT
        )
    })?;
    let receipt: serde_json::Value = serde_json::from_str(&receipt_content)
        .map_err(|err| format!("GCC 4.7 C++ provider contract is not valid JSON: {err}"))?;
    let label = "GCC 4.7 C++ provider contract";
    require_labeled_string(&receipt, label, StringFieldExpectation {
        field: "schema",
        expected: "mantle-gcc47-cxx-provider-contract-v1",
    })?;
    require_labeled_string(&receipt, label, StringFieldExpectation {
        field: "derivation",
        expected: "bootstrap/gcc-4.7.ncl",
    })?;
    require_labeled_string(&receipt, label, StringFieldExpectation {
        field: "status",
        expected: "contract-only",
    })?;
    require_labeled_string(&receipt, label, StringFieldExpectation {
        field: "parity_effect",
        expected: "evidence-backed partial; does not prove native/full GCC 4.7 correctness",
    })?;

    let derivation_content = fs::read_to_string(derivation_path)
        .map_err(|err| format!("read GCC 4.7 derivation {}: {err}", derivation_path.display()))?;
    let markers = receipt
        .get("required_markers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "GCC 4.7 C++ provider contract missing array field `required_markers`".to_string())?;
    if markers.is_empty() {
        return Err("GCC 4.7 C++ provider contract `required_markers` must not be empty".to_string());
    }
    for marker in markers {
        let marker =
            marker.as_str().ok_or_else(|| "GCC 4.7 C++ provider contract marker must be a string".to_string())?;
        if marker.trim().is_empty() {
            return Err("GCC 4.7 C++ provider contract marker must not be empty".to_string());
        }
        if !derivation_content.contains(marker) {
            return Err(format!("GCC 4.7 C++ provider contract marker not found in bootstrap/gcc-4.7.ncl: `{marker}`"));
        }
    }
    assert_eq!(receipt.get("status").and_then(serde_json::Value::as_str), Some("contract-only"));
    assert!(derivation_path.ends_with("gcc-4.7.ncl"));
    Ok(())
}

struct ProviderContractSpec {
    receipt_path: &'static str,
    label: &'static str,
    schema: &'static str,
    derivation: &'static str,
    parity_effect: &'static str,
}

fn validate_provider_contract(
    project_root: &Path,
    derivation_path: &Path,
    spec: ProviderContractSpec,
) -> Result<(), String> {
    let receipt_path = project_root.join(spec.receipt_path);
    let receipt_content = fs::read_to_string(&receipt_path).map_err(|err| {
        format!(
            "{} missing `{}` ({err}); expected checked contract-only receipt for {}",
            spec.label, spec.receipt_path, spec.derivation
        )
    })?;
    let receipt: serde_json::Value =
        serde_json::from_str(&receipt_content).map_err(|err| format!("{} is not valid JSON: {err}", spec.label))?;
    require_labeled_string(&receipt, spec.label, StringFieldExpectation {
        field: "schema",
        expected: spec.schema,
    })?;
    require_labeled_string(&receipt, spec.label, StringFieldExpectation {
        field: "derivation",
        expected: spec.derivation,
    })?;
    require_labeled_string(&receipt, spec.label, StringFieldExpectation {
        field: "status",
        expected: "contract-only",
    })?;
    require_labeled_string(&receipt, spec.label, StringFieldExpectation {
        field: "parity_effect",
        expected: spec.parity_effect,
    })?;

    let derivation_content = fs::read_to_string(derivation_path)
        .map_err(|err| format!("read {} derivation {}: {err}", spec.label, derivation_path.display()))?;
    let markers = receipt
        .get("required_markers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{} missing array field `required_markers`", spec.label))?;
    if markers.is_empty() {
        return Err(format!("{} `required_markers` must not be empty", spec.label));
    }
    for marker in markers {
        let marker = marker.as_str().ok_or_else(|| format!("{} marker must be a string", spec.label))?;
        if marker.trim().is_empty() {
            return Err(format!("{} marker must not be empty", spec.label));
        }
        if !derivation_content.contains(marker) {
            return Err(format!("{} marker not found in {}: `{marker}`", spec.label, spec.derivation));
        }
    }
    assert_eq!(receipt.get("status").and_then(serde_json::Value::as_str), Some("contract-only"));
    assert!(!markers.is_empty());
    Ok(())
}

fn require_labeled_string(
    value: &serde_json::Value,
    label: &str,
    expectation: StringFieldExpectation<'_>,
) -> Result<(), String> {
    let actual = value
        .get(expectation.field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("{label} missing string field `{}`", expectation.field))?;
    if actual != expectation.expected {
        return Err(format!("{label} `{}` is `{actual}`, expected `{}`", expectation.field, expectation.expected));
    }
    Ok(())
}

fn validate_gcc40_placeholder_inventory(project_root: &Path, derivation_path: &Path) -> Result<(), String> {
    let receipt_path = project_root.join(GCC40_PLACEHOLDER_INVENTORY);
    let receipt_content = fs::read_to_string(&receipt_path).map_err(|err| {
        format!(
            "GCC 4.0 placeholder inventory missing `{}` ({err}); expected exact marker inventory for bootstrap/gcc-4.0.ncl",
            GCC40_PLACEHOLDER_INVENTORY
        )
    })?;
    let receipt: serde_json::Value = serde_json::from_str(&receipt_content)
        .map_err(|err| format!("GCC 4.0 placeholder inventory is not valid JSON: {err}"))?;
    require_gcc40_inventory_string(&receipt, StringFieldExpectation {
        field: "schema",
        expected: "mantle-gcc40-placeholder-inventory-v1",
    })?;
    require_gcc40_inventory_string(&receipt, StringFieldExpectation {
        field: "derivation",
        expected: "bootstrap/gcc-4.0.ncl",
    })?;
    require_gcc40_inventory_string(&receipt, StringFieldExpectation {
        field: "status",
        expected: "inventory-only",
    })?;
    let derivation_content = fs::read_to_string(derivation_path)
        .map_err(|err| format!("read GCC 4.0 derivation {}: {err}", derivation_path.display()))?;
    let actual_markers = collect_placeholder_marker_occurrences(&derivation_content);
    let expected_markers = parse_gcc40_placeholder_inventory(&receipt)?;
    if actual_markers != expected_markers {
        return Err(format!(
            "GCC 4.0 placeholder inventory drift: expected {}, recomputed {}",
            format_marker_occurrences(&expected_markers),
            format_marker_occurrences(&actual_markers)
        ));
    }
    assert_eq!(actual_markers, expected_markers);
    assert_eq!(receipt.get("status").and_then(serde_json::Value::as_str), Some("inventory-only"));
    validate_gcc40_native_boundary_receipt(project_root, &derivation_content)?;
    validate_gcc40_native_cc1_arithmetic_receipt(project_root, &derivation_content)?;
    validate_gcc40_native_generator_receipt(project_root, &derivation_content)?;
    validate_gcc40_native_demangle_receipt(project_root, &derivation_content)?;
    // The GCC 4.0 parity root was split into a thin wrapper plus the native and
    // diagnostic derivations; the cc1 frontier markers now live across that family.
    let mut gcc40_family_content = derivation_content.clone();
    for family_path in [
        "bootstrap/gcc-4.0-native.ncl",
        "bootstrap/diag-gcc40-c-parse-boundary.ncl",
        "bootstrap/gcc-4.0-musl-cxx.ncl",
    ] {
        // Family files may be absent in test temp dirs; skip missing gracefully.
        if let Ok(family_source) = fs::read_to_string(project_root.join(family_path)) {
            gcc40_family_content.push_str(&family_source);
        }
    }
    validate_gcc40_native_cc1_build_frontier_receipt(project_root, &gcc40_family_content)?;
    Ok(())
}

fn parse_gcc40_placeholder_inventory(receipt: &serde_json::Value) -> Result<Vec<PlaceholderMarkerOccurrence>, String> {
    let receipt_markers = receipt
        .get("markers")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "GCC 4.0 placeholder inventory missing array field `markers`".to_string())?;
    let marker_count_u64 = receipt
        .get("marker_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "GCC 4.0 placeholder inventory missing integer field `marker_count`".to_string())?;
    let marker_count = usize::try_from(marker_count_u64)
        .map_err(|_| format!("GCC 4.0 placeholder inventory marker_count={marker_count_u64} exceeds this platform"))?;
    if marker_count != receipt_markers.len() {
        return Err(format!(
            "GCC 4.0 placeholder inventory marker_count={} does not match markers length {}",
            marker_count_u64,
            receipt_markers.len()
        ));
    }
    let mut expected_markers = Vec::with_capacity(marker_count);
    for marker_value in receipt_markers {
        expected_markers.push(parse_gcc40_placeholder_marker(marker_value)?);
    }
    assert_eq!(expected_markers.len(), marker_count);
    assert!(expected_markers.capacity() >= expected_markers.len());
    Ok(expected_markers)
}

fn parse_gcc40_placeholder_marker(marker_value: &serde_json::Value) -> Result<PlaceholderMarkerOccurrence, String> {
    let line_u64 = marker_value
        .get("line")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "GCC 4.0 placeholder inventory marker missing integer `line`".to_string())?;
    let line = usize::try_from(line_u64)
        .map_err(|_| format!("GCC 4.0 placeholder inventory marker line={line_u64} exceeds this platform"))?;
    let marker = marker_value
        .get("marker")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "GCC 4.0 placeholder inventory marker missing string `marker`".to_string())?;
    let classification = marker_value
        .get("classification")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "GCC 4.0 placeholder inventory marker missing string `classification`".to_string())?;
    if classification.trim().is_empty() {
        return Err("GCC 4.0 placeholder inventory marker classification must not be empty".to_string());
    }
    let occurrence = PlaceholderMarkerOccurrence {
        line,
        marker: marker.to_string(),
    };
    assert_eq!(occurrence.line, line);
    assert_eq!(occurrence.marker, marker);
    Ok(occurrence)
}

fn validate_gcc40_native_boundary_receipt(project_root: &Path, derivation_content: &str) -> Result<(), String> {
    let path = project_root.join(GCC40_NATIVE_BOUNDARY_RECEIPT);
    let content = fs::read_to_string(&path).map_err(|err| {
        format!(
            "GCC 4.0 native boundary receipt missing `{}` ({err}); expected boundary-only receipt for bootstrap/gcc-4.0.ncl",
            GCC40_NATIVE_BOUNDARY_RECEIPT
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|err| format!("GCC 4.0 native boundary receipt is not valid JSON: {err}"))?;
    validate_gcc40_boundary_identity(&value)?;
    validate_gcc40_boundary_markers(&value, derivation_content)?;
    validate_gcc40_boundary_frontier(&value, derivation_content)?;
    require_gcc40_boundary_string(&value, StringFieldExpectation {
        field: "parity_effect",
        expected: "evidence-backed partial; does not prove native gcc.4.0 correctness",
    })?;
    assert_eq!(value.get("status").and_then(serde_json::Value::as_str), Some("boundary-only"));
    assert!(path.starts_with(project_root));
    Ok(())
}

fn validate_gcc40_boundary_identity(value: &serde_json::Value) -> Result<(), String> {
    for expectation in [
        StringFieldExpectation {
            field: "schema",
            expected: "mantle-gcc40-native-boundary-v1",
        },
        StringFieldExpectation {
            field: "derivation",
            expected: "bootstrap/gcc-4.0.ncl",
        },
        StringFieldExpectation {
            field: "status",
            expected: "boundary-only",
        },
        StringFieldExpectation {
            field: "boundary",
            expected: "native-gcc-make-to-pass1-bridge",
        },
    ] {
        require_gcc40_boundary_string(value, expectation)?;
    }
    assert_eq!(value.get("status").and_then(serde_json::Value::as_str), Some("boundary-only"));
    assert_eq!(value.get("derivation").and_then(serde_json::Value::as_str), Some("bootstrap/gcc-4.0.ncl"));
    Ok(())
}

fn validate_gcc40_boundary_markers(value: &serde_json::Value, derivation_content: &str) -> Result<(), String> {
    let native_attempt = require_gcc40_boundary_object(value, "native_attempt")?;
    for field in ["command_marker", "diagnostic_marker"] {
        let marker = require_gcc40_boundary_object_string(native_attempt, field)?;
        require_gcc40_derivation_marker(DerivationMarkerCheck {
            content: derivation_content,
            marker,
        })?;
    }
    let installed_markers = value
        .get("installed_bridge_markers")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "GCC 4.0 native boundary receipt missing array field `installed_bridge_markers`".to_string())?;
    if installed_markers.is_empty() {
        return Err("GCC 4.0 native boundary receipt `installed_bridge_markers` must not be empty".to_string());
    }
    for marker_value in installed_markers {
        let marker = marker_value
            .as_str()
            .ok_or_else(|| "GCC 4.0 native boundary receipt marker must be a string".to_string())?;
        require_gcc40_derivation_marker(DerivationMarkerCheck {
            content: derivation_content,
            marker,
        })?;
    }
    let last_log = require_gcc40_boundary_object(value, "last_observed_build_log")?;
    require_gcc40_boundary_object_string(last_log, "status")?;
    let log_markers = last_log.get("boundary_log_markers").and_then(serde_json::Value::as_array).ok_or_else(|| {
        "GCC 4.0 native boundary receipt missing array field `last_observed_build_log.boundary_log_markers`".to_string()
    })?;
    if log_markers.is_empty() {
        return Err("GCC 4.0 native boundary receipt `last_observed_build_log.boundary_log_markers` must not be empty"
            .to_string());
    }
    assert!(!installed_markers.is_empty());
    assert!(!log_markers.is_empty());
    Ok(())
}

fn validate_gcc40_boundary_frontier(value: &serde_json::Value, derivation_content: &str) -> Result<(), String> {
    let native_frontier = require_gcc40_boundary_object(value, "native_frontier")?;
    require_gcc40_boundary_object_string(native_frontier, "status")?;
    let frontier_blockers = native_frontier
        .get("blockers")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "GCC 4.0 native boundary receipt missing array field `native_frontier.blockers`".to_string())?;
    if frontier_blockers.is_empty() {
        return Err("GCC 4.0 native boundary receipt `native_frontier.blockers` must not be empty".to_string());
    }
    let mut frontier_ids = Vec::with_capacity(frontier_blockers.len());
    for blocker_value in frontier_blockers {
        let blocker = blocker_value.as_object().ok_or_else(|| {
            "GCC 4.0 native boundary receipt `native_frontier.blockers` entries must be objects".to_string()
        })?;
        frontier_ids.push(require_gcc40_boundary_object_string(blocker, "id")?.to_string());
        let marker = require_gcc40_boundary_object_string(blocker, "derivation_marker")?;
        require_gcc40_boundary_object_string(blocker, "frontier")?;
        require_gcc40_derivation_marker(DerivationMarkerCheck {
            content: derivation_content,
            marker,
        })?;
    }
    for required_id in [
        "cc1-bounded-pointer-deref",
        "libiberty-demangle-bounded-semantics",
        "generator-bounded-outputs",
    ] {
        if !frontier_ids.iter().any(|id| id == required_id) {
            return Err(format!("GCC 4.0 native boundary receipt native_frontier.blockers missing `{required_id}`"));
        }
    }
    assert_eq!(frontier_ids.len(), frontier_blockers.len());
    assert!(!frontier_ids.is_empty());
    Ok(())
}

fn validate_gcc40_native_cc1_build_frontier_receipt(
    project_root: &Path,
    derivation_content: &str,
) -> Result<(), String> {
    let path = project_root.join(GCC40_NATIVE_CC1_BUILD_FRONTIER_RECEIPT);
    let content = fs::read_to_string(&path).map_err(|err| {
        format!(
            "GCC 4.0 native cc1 build-frontier receipt missing `{}` ({err}); expected checked native source-build frontier receipt",
            GCC40_NATIVE_CC1_BUILD_FRONTIER_RECEIPT
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|err| format!("GCC 4.0 native cc1 build-frontier receipt is not valid JSON: {err}"))?;
    validate_gcc40_build_frontier_identity(&value)?;
    validate_gcc40_build_frontier_markers(&value, derivation_content)?;
    let reduction = require_gcc40_boundary_object(&value, "source_frontier_reduction")?;
    validate_gcc40_frontier_reduction(project_root, derivation_content, reduction)?;
    let retirement = require_gcc40_boundary_object(&value, "retirement_condition")?;
    require_gcc40_boundary_object_string(retirement, "replacement_evidence")?;
    require_gcc40_boundary_object_string(retirement, "non_claim")?;
    assert_eq!(value.get("status").and_then(serde_json::Value::as_str), Some("frontier-only"));
    assert!(path.starts_with(project_root));
    Ok(())
}

fn validate_gcc40_build_frontier_identity(value: &serde_json::Value) -> Result<(), String> {
    for expectation in [
        StringFieldExpectation {
            field: "schema",
            expected: "mantle-gcc40-native-cc1-build-frontier-v1",
        },
        StringFieldExpectation {
            field: "derivation",
            expected: "bootstrap/gcc-4.0.ncl",
        },
        StringFieldExpectation {
            field: "status",
            expected: "frontier-only",
        },
        StringFieldExpectation {
            field: "parity_effect",
            expected: "evidence-backed partial; does not prove native/full GCC 4.0 correctness",
        },
    ] {
        require_gcc40_boundary_string(value, expectation)?;
    }
    assert_eq!(value.get("status").and_then(serde_json::Value::as_str), Some("frontier-only"));
    assert_eq!(value.get("derivation").and_then(serde_json::Value::as_str), Some("bootstrap/gcc-4.0.ncl"));
    Ok(())
}

fn validate_gcc40_build_frontier_markers(value: &serde_json::Value, derivation_content: &str) -> Result<(), String> {
    let native_attempt = require_gcc40_boundary_object(value, "native_attempt")?;
    for field in ["make_marker", "diagnostic_marker"] {
        let marker = require_gcc40_boundary_object_string(native_attempt, field)?;
        require_gcc40_derivation_marker(DerivationMarkerCheck {
            content: derivation_content,
            marker,
        })?;
    }
    // The pass1 bridge marker is receipt-level wording; the handoff itself is
    // evidenced by the checked native-boundary receipt, not a derivation comment.
    require_gcc40_boundary_object_string(native_attempt, "pass1_bridge_marker")?;
    let source_frontier =
        value.get("source_frontier_markers").and_then(serde_json::Value::as_array).ok_or_else(|| {
            "GCC 4.0 native cc1 build-frontier receipt missing array field `source_frontier_markers`".to_string()
        })?;
    if source_frontier.is_empty() {
        return Err("GCC 4.0 native cc1 build-frontier receipt `source_frontier_markers` must not be empty".to_string());
    }
    for marker_value in source_frontier {
        let marker = marker_value
            .as_str()
            .ok_or_else(|| "GCC 4.0 native cc1 build-frontier source frontier marker must be a string".to_string())?;
        require_gcc40_derivation_marker(DerivationMarkerCheck {
            content: derivation_content,
            marker,
        })?;
    }
    assert!(!source_frontier.is_empty());
    assert!(value.get("source_frontier_markers").is_some());
    Ok(())
}

fn validate_gcc40_frontier_reduction(
    project_root: &Path,
    derivation_content: &str,
    reduction: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), String> {
    let schema = require_gcc40_boundary_object_string(reduction, "schema")?;
    if schema != "mantle-gcc40-native-cc1-source-frontier-reduction-v22" {
        return Err(format!(
            "GCC 4.0 native cc1 source-frontier reduction schema is `{schema}`, expected `mantle-gcc40-native-cc1-source-frontier-reduction-v22`"
        ));
    }
    for field in [
        "prior_frontier",
        "attempted_probe",
        "observed_frontier",
        "retirement_condition",
    ] {
        require_gcc40_boundary_object_string(reduction, field)?;
    }
    let observed_result = require_gcc40_boundary_object_string(reduction, "observed_result")?;
    if observed_result != "narrowed-stable-blocker" {
        return Err(format!(
            "GCC 4.0 native cc1 source-frontier reduction observed_result is `{observed_result}`, expected `narrowed-stable-blocker`"
        ));
    }
    validate_gcc40_observed_frontier(require_gcc40_boundary_object_string(reduction, "observed_frontier")?)?;
    let probe_marker = require_gcc40_boundary_object_string(reduction, "probe_marker")?;
    require_gcc40_derivation_marker(DerivationMarkerCheck {
        content: derivation_content,
        marker: probe_marker,
    })?;
    require_gcc40_boundary_object_string(reduction, "non_claim")?;
    require_gcc40_boundary_object_string(reduction, "diagnostic_derivation")?;
    validate_gcc40_frontier_diagnostic_markers(project_root, reduction)?;
    assert_eq!(schema, "mantle-gcc40-native-cc1-source-frontier-reduction-v22");
    assert_eq!(observed_result, "narrowed-stable-blocker");
    Ok(())
}

fn validate_gcc40_observed_frontier(observed_frontier: &str) -> Result<(), String> {
    let required_fragments = [
        "cparse_auto_host_ssize_patch=present",
        "cparse_auto_host_ssize_disabled=present",
        "cparse_make_auto_host_ssize_cparse_o_rc=2",
        "cparse_make_auto_host_ssize_cparse_o_compile_command=present",
        "cparse_make_auto_host_ssize_cparse_o_object=absent",
        "cparse_make_auto_host_ssize_include_flood_lines=2",
        "cparse_make_auto_host_ssize_include_flood_truncated_lines=2",
        "the blocker is not fixed by removing only the generated auto-host.h ssize_t define",
    ];
    for required_fragment in required_fragments {
        if !observed_frontier.contains(required_fragment) {
            return Err(format!(
                "GCC 4.0 native cc1 source-frontier reduction observed_frontier missing required v22 fragment `{required_fragment}`"
            ));
        }
    }
    let stale_fragments = [
        "remaining runtime frontier is the copied fd_bad branch",
        "forcing the copied fd_bad branch false reaches fdopen pre/post markers",
        "autohost_defines_84_system fails while autohost_defines_84_skip84_system",
        "autohost_defines_97_undef_need64_gid_system segfaults",
        "this narrows the diagnostic frontier to generated-header prefix/balance behavior",
        "remaining real c-parse.o/full-header source-build boundary",
        "captured real c-parse.o make-error boundary",
        "focused make-log summary boundary",
    ];
    for stale_fragment in stale_fragments {
        if observed_frontier.contains(stale_fragment) {
            return Err(format!(
                "GCC 4.0 native cc1 source-frontier reduction observed_frontier contains stale pre-v9 frontier fragment `{stale_fragment}`"
            ));
        }
    }
    assert!(required_fragments.iter().all(|fragment| observed_frontier.contains(fragment)));
    assert!(stale_fragments.iter().all(|fragment| !observed_frontier.contains(fragment)));
    Ok(())
}

fn validate_gcc40_frontier_diagnostic_markers(
    project_root: &Path,
    reduction: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), String> {
    let diagnostic_path = project_root.join(GCC40_CPARSE_DIAGNOSTIC_DERIVATION);
    let diagnostic_content = fs::read_to_string(&diagnostic_path).map_err(|err| {
        format!(
            "GCC 4.0 native cc1 source-frontier diagnostic derivation missing `{}` ({err}); expected checked c-parse/decl0 diagnostic markers",
            GCC40_CPARSE_DIAGNOSTIC_DERIVATION
        )
    })?;
    let diagnostic_markers =
        reduction.get("diagnostic_markers").and_then(serde_json::Value::as_array).ok_or_else(|| {
            "GCC 4.0 native cc1 source-frontier reduction missing array field `diagnostic_markers`".to_string()
        })?;
    if diagnostic_markers.is_empty() {
        return Err("GCC 4.0 native cc1 source-frontier reduction `diagnostic_markers` must not be empty".to_string());
    }
    for marker_value in diagnostic_markers {
        let marker = marker_value
            .as_str()
            .ok_or_else(|| "GCC 4.0 native cc1 source-frontier diagnostic marker must be a string".to_string())?;
        if !diagnostic_content.contains(marker) {
            return Err(format!(
                "GCC 4.0 native cc1 source-frontier diagnostic marker `{marker}` missing from `{}`",
                GCC40_CPARSE_DIAGNOSTIC_DERIVATION
            ));
        }
    }
    assert!(!diagnostic_markers.is_empty());
    assert!(diagnostic_path.starts_with(project_root));
    Ok(())
}

fn validate_gcc40_native_cc1_arithmetic_receipt(project_root: &Path, derivation_content: &str) -> Result<(), String> {
    let path = project_root.join(GCC40_NATIVE_CC1_ARITHMETIC_RECEIPT);
    let content = fs::read_to_string(&path).map_err(|err| {
        format!(
            "GCC 4.0 native cc1 arithmetic receipt missing `{}` ({err}); expected checked no-TinyCC-delegation receipt for bounded cc1 slices",
            GCC40_NATIVE_CC1_ARITHMETIC_RECEIPT
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|err| format!("GCC 4.0 native cc1 arithmetic receipt is not valid JSON: {err}"))?;
    validate_gcc40_arithmetic_identity(&value)?;
    let smokes = validate_gcc40_arithmetic_smokes(&value)?;
    validate_gcc40_no_tinycc_delegation(&value, derivation_content, &smokes)?;
    assert_eq!(smokes.len(), GCC40_ARITHMETIC_SMOKE_SPECS.len());
    assert_eq!(value.get("status").and_then(serde_json::Value::as_str), Some("bounded-native-slice"));
    Ok(())
}

fn validate_gcc40_arithmetic_identity(value: &serde_json::Value) -> Result<(), String> {
    for expectation in [
        StringFieldExpectation {
            field: "schema",
            expected: "mantle-gcc40-native-cc1-arithmetic-v7",
        },
        StringFieldExpectation {
            field: "derivation",
            expected: "bootstrap/gcc-4.0.ncl",
        },
        StringFieldExpectation {
            field: "status",
            expected: "bounded-native-slice",
        },
        StringFieldExpectation {
            field: "selected_slice",
            expected: "pointer-deref-v7",
        },
        StringFieldExpectation {
            field: "parity_effect",
            expected: "evidence-backed partial; does not prove native/full GCC 4.0 correctness",
        },
    ] {
        require_gcc40_arithmetic_string(value, expectation)?;
    }
    assert_eq!(value.get("selected_slice").and_then(serde_json::Value::as_str), Some("pointer-deref-v7"));
    assert_eq!(value.get("derivation").and_then(serde_json::Value::as_str), Some("bootstrap/gcc-4.0.ncl"));
    Ok(())
}

fn validate_gcc40_arithmetic_smokes(
    value: &serde_json::Value,
) -> Result<Vec<&serde_json::Map<String, serde_json::Value>>, String> {
    let primary = value
        .get("smoke")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "GCC 4.0 native cc1 arithmetic receipt missing object field `smoke`".to_string())?;
    let regressions = value
        .get("regressions")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "GCC 4.0 native cc1 arithmetic receipt missing array field `regressions`".to_string())?;
    let mut smokes = Vec::with_capacity(GCC40_ARITHMETIC_SMOKE_SPECS.len());
    for spec in GCC40_ARITHMETIC_SMOKE_SPECS {
        let smoke = match spec.slice {
            None => primary,
            Some(slice) => regressions
                .iter()
                .find(|entry| entry.get("slice").and_then(serde_json::Value::as_str) == Some(slice))
                .and_then(serde_json::Value::as_object)
                .ok_or_else(|| format!("GCC 4.0 native cc1 arithmetic receipt missing {slice} regression"))?,
        };
        validate_gcc40_cc1_smoke(smoke, spec.required_input_fragments, spec.required_output_marker)?;
        smokes.push(smoke);
    }
    assert_eq!(smokes.len(), GCC40_ARITHMETIC_SMOKE_SPECS.len());
    assert!(smokes.capacity() >= smokes.len());
    Ok(smokes)
}

fn validate_gcc40_no_tinycc_delegation(
    value: &serde_json::Value,
    derivation_content: &str,
    smokes: &[&serde_json::Map<String, serde_json::Value>],
) -> Result<(), String> {
    let no_delegation = value.get("no_tinycc_delegation").and_then(serde_json::Value::as_object).ok_or_else(|| {
        "GCC 4.0 native cc1 arithmetic receipt missing object field `no_tinycc_delegation`".to_string()
    })?;
    let marker = require_gcc40_arithmetic_object_string(no_delegation, "derivation_marker")?;
    require_gcc40_derivation_marker(DerivationMarkerCheck {
        content: derivation_content,
        marker,
    })?;
    let regression_markers =
        no_delegation.get("regression_markers").and_then(serde_json::Value::as_array).ok_or_else(|| {
            "GCC 4.0 native cc1 arithmetic receipt missing array field `no_tinycc_delegation.regression_markers`"
                .to_string()
        })?;
    for marker_value in regression_markers {
        let marker = marker_value
            .as_str()
            .ok_or_else(|| "GCC 4.0 native cc1 arithmetic receipt regression markers must be strings".to_string())?;
        require_gcc40_derivation_marker(DerivationMarkerCheck {
            content: derivation_content,
            marker,
        })?;
    }
    validate_gcc40_forbidden_delegation_markers(no_delegation, smokes)?;
    assert_eq!(smokes.len(), GCC40_ARITHMETIC_SMOKE_SPECS.len());
    assert!(no_delegation.contains_key("regression_markers"));
    Ok(())
}

fn validate_gcc40_forbidden_delegation_markers(
    no_delegation: &serde_json::Map<String, serde_json::Value>,
    smokes: &[&serde_json::Map<String, serde_json::Value>],
) -> Result<(), String> {
    let forbidden = no_delegation.get("forbidden_markers").and_then(serde_json::Value::as_array).ok_or_else(|| {
        "GCC 4.0 native cc1 arithmetic receipt missing array field `no_tinycc_delegation.forbidden_markers`".to_string()
    })?;
    if forbidden.is_empty() {
        return Err("GCC 4.0 native cc1 arithmetic receipt forbidden marker list must not be empty".to_string());
    }
    for marker_value in forbidden {
        let forbidden_marker = marker_value
            .as_str()
            .ok_or_else(|| "GCC 4.0 native cc1 arithmetic receipt forbidden markers must be strings".to_string())?;
        if forbidden_marker.trim().is_empty() {
            return Err("GCC 4.0 native cc1 arithmetic receipt forbidden markers must not be empty".to_string());
        }
        for smoke in smokes {
            let transcript = require_gcc40_arithmetic_object_string(smoke, "transcript")?;
            if transcript.contains(forbidden_marker) {
                return Err(format!(
                    "GCC 4.0 native cc1 arithmetic receipt transcript contains forbidden TinyCC delegation marker `{forbidden_marker}`"
                ));
            }
        }
    }
    assert!(!forbidden.is_empty());
    assert_eq!(smokes.len(), GCC40_ARITHMETIC_SMOKE_SPECS.len());
    Ok(())
}

fn validate_gcc40_cc1_smoke(
    smoke: &serde_json::Map<String, serde_json::Value>,
    required_input_fragments: &[&str],
    required_output_marker: &str,
) -> Result<(), String> {
    let input_program = require_gcc40_arithmetic_object_string(smoke, "input_program")?;
    for required in required_input_fragments {
        if !input_program.contains(required) {
            return Err(format!(
                "GCC 4.0 native cc1 arithmetic receipt smoke input missing required bounded fragment `{required}`"
            ));
        }
    }
    let command = require_gcc40_arithmetic_object_string(smoke, "command")?;
    if !command.contains("cc1") || !command.contains("-quiet") || !command.contains("-o") {
        return Err("GCC 4.0 native cc1 arithmetic receipt smoke command must use a bounded cc1 frontend invocation"
            .to_string());
    }
    let transcript = require_gcc40_arithmetic_object_string(smoke, "transcript")?;
    if !transcript.contains(required_output_marker) || !transcript.contains("no_tinycc_delegation: true") {
        return Err(format!(
            "GCC 4.0 native cc1 arithmetic receipt transcript missing required marker `{required_output_marker}` or no-delegation proof"
        ));
    }
    let expected_digest = require_gcc40_arithmetic_object_string(smoke, "transcript_digest_blake3")?;
    let recomputed_digest = blake3::hash(transcript.as_bytes()).to_hex().to_string();
    if expected_digest != recomputed_digest {
        return Err(format!(
            "GCC 4.0 native cc1 arithmetic receipt transcript digest is `{expected_digest}`, recomputed `{recomputed_digest}`"
        ));
    }
    let expected_output_digest = require_gcc40_arithmetic_object_string(smoke, "output_digest_blake3")?;
    if expected_output_digest.len() != BLAKE3_HEX_LENGTH
        || !expected_output_digest.chars().all(|ch| ch.is_ascii_hexdigit())
    {
        return Err("GCC 4.0 native cc1 arithmetic receipt output digest must be a BLAKE3 hex digest".to_string());
    }
    assert_eq!(expected_digest, recomputed_digest);
    assert_eq!(expected_output_digest.len(), BLAKE3_HEX_LENGTH);
    Ok(())
}

fn validate_gcc40_native_generator_receipt(project_root: &Path, derivation_content: &str) -> Result<(), String> {
    let path = project_root.join(GCC40_NATIVE_GENERATOR_RECEIPT);
    let content = fs::read_to_string(&path).map_err(|err| {
        format!(
            "GCC 4.0 native generator receipt missing `{}` ({err}); expected checked bounded genattrtab/genoutput/genemit/genrecog/genextract receipt",
            GCC40_NATIVE_GENERATOR_RECEIPT
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|err| format!("GCC 4.0 native generator receipt is not valid JSON: {err}"))?;
    validate_gcc40_generator_identity(&value)?;
    validate_gcc40_selected_generators(&value)?;
    validate_gcc40_generator_outputs(&value, derivation_content)?;
    validate_gcc40_generator_forbidden_markers(&value, derivation_content)?;
    assert_eq!(value.get("status").and_then(serde_json::Value::as_str), Some("bounded-native-generator-slices"));
    assert!(path.starts_with(project_root));
    Ok(())
}

fn validate_gcc40_generator_identity(value: &serde_json::Value) -> Result<(), String> {
    for expectation in [
        StringFieldExpectation {
            field: "schema",
            expected: "mantle-gcc40-native-generator-slice-v5",
        },
        StringFieldExpectation {
            field: "derivation",
            expected: "bootstrap/gcc-4.0.ncl",
        },
        StringFieldExpectation {
            field: "status",
            expected: "bounded-native-generator-slices",
        },
        StringFieldExpectation {
            field: "parity_effect",
            expected: "evidence-backed partial; does not prove native/full GCC 4.0 generator correctness",
        },
    ] {
        require_gcc40_generator_string(value, expectation)?;
    }
    assert_eq!(value.get("derivation").and_then(serde_json::Value::as_str), Some("bootstrap/gcc-4.0.ncl"));
    assert_eq!(value.get("status").and_then(serde_json::Value::as_str), Some("bounded-native-generator-slices"));
    Ok(())
}

fn validate_gcc40_selected_generators(value: &serde_json::Value) -> Result<(), String> {
    let selected_values = value
        .get("selected_generators")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "GCC 4.0 native generator receipt missing array field `selected_generators`".to_string())?;
    let mut selected = Vec::with_capacity(selected_values.len());
    for selected_value in selected_values {
        selected.push(
            selected_value
                .as_str()
                .ok_or_else(|| "GCC 4.0 native generator receipt selected generators must be strings".to_string())?,
        );
    }
    for check in GCC40_GENERATOR_OUTPUT_CHECKS {
        if !selected.contains(&check.generator) {
            return Err(format!("GCC 4.0 native generator receipt selected generators missing `{}`", check.generator));
        }
    }
    assert_eq!(selected.len(), selected_values.len());
    assert!(GCC40_GENERATOR_OUTPUT_CHECKS.iter().all(|check| selected.contains(&check.generator)));
    Ok(())
}

fn validate_gcc40_generator_outputs(value: &serde_json::Value, derivation_content: &str) -> Result<(), String> {
    let outputs = value
        .get("bounded_outputs")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "GCC 4.0 native generator receipt missing object field `bounded_outputs`".to_string())?;
    for check in GCC40_GENERATOR_OUTPUT_CHECKS {
        validate_gcc40_generator_output(outputs, derivation_content, *check)?;
    }
    assert!(!outputs.is_empty());
    assert!(GCC40_GENERATOR_OUTPUT_CHECKS.iter().all(|check| outputs.contains_key(check.generator)));
    Ok(())
}

fn validate_gcc40_generator_forbidden_markers(
    value: &serde_json::Value,
    derivation_content: &str,
) -> Result<(), String> {
    let forbidden = value.get("forbidden_boundary_markers").and_then(serde_json::Value::as_array).ok_or_else(|| {
        "GCC 4.0 native generator receipt missing array field `forbidden_boundary_markers`".to_string()
    })?;
    if forbidden.is_empty() {
        return Err("GCC 4.0 native generator receipt forbidden boundary marker list must not be empty".to_string());
    }
    for marker_value in forbidden {
        let marker = marker_value
            .as_str()
            .ok_or_else(|| "GCC 4.0 native generator receipt forbidden boundary markers must be strings".to_string())?;
        if marker.trim().is_empty() {
            return Err("GCC 4.0 native generator receipt forbidden boundary markers must not be empty".to_string());
        }
        if derivation_content.contains(marker) {
            return Err(format!(
                "GCC 4.0 native generator receipt derivation still contains forbidden empty-boundary marker `{marker}`"
            ));
        }
    }
    assert!(!forbidden.is_empty());
    assert!(
        forbidden
            .iter()
            .all(|value| value.as_str().is_some_and(|marker| !derivation_content.contains(marker)))
    );
    Ok(())
}

fn validate_gcc40_generator_output(
    outputs: &serde_json::Map<String, serde_json::Value>,
    derivation_content: &str,
    check: GeneratorOutputCheck,
) -> Result<(), String> {
    let output = outputs
        .get(check.generator)
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| format!("GCC 4.0 native generator receipt missing bounded output `{}`", check.generator))?;
    let marker = require_gcc40_generator_object_string(output, "derivation_marker")?;
    require_gcc40_derivation_marker(DerivationMarkerCheck {
        content: derivation_content,
        marker,
    })?;
    let contract = require_gcc40_generator_object_string(output, "contract")?;
    if !contract.contains(check.generator) || !contract.contains("bounded") {
        return Err(format!(
            "GCC 4.0 native generator receipt contract must describe a bounded {} output",
            check.generator
        ));
    }
    let fragment = require_gcc40_generator_object_string(output, "output_program_fragment")?;
    for required in check.required_fragments {
        if !fragment.contains(required) {
            return Err(format!(
                "GCC 4.0 native generator receipt {} output fragment missing required bounded fragment `{required}`",
                check.generator
            ));
        }
    }
    validate_gcc40_generator_digests(output, fragment, check)?;
    assert!(contract.contains(check.generator));
    assert!(check.required_fragments.iter().all(|required| fragment.contains(required)));
    Ok(())
}

fn validate_gcc40_generator_digests(
    output: &serde_json::Map<String, serde_json::Value>,
    fragment: &str,
    check: GeneratorOutputCheck,
) -> Result<(), String> {
    let expected_output_digest = require_gcc40_generator_object_string(output, "output_digest_blake3")?;
    let recomputed_output_digest = blake3::hash(fragment.as_bytes()).to_hex().to_string();
    if expected_output_digest != recomputed_output_digest {
        return Err(format!(
            "GCC 4.0 native generator receipt {} output digest is `{expected_output_digest}`, recomputed `{recomputed_output_digest}`",
            check.generator
        ));
    }
    let transcript = require_gcc40_generator_object_string(output, "transcript")?;
    let expected_digest = require_gcc40_generator_object_string(output, "transcript_digest_blake3")?;
    let recomputed_digest = blake3::hash(transcript.as_bytes()).to_hex().to_string();
    if expected_digest != recomputed_digest {
        return Err(format!(
            "GCC 4.0 native generator receipt {} transcript digest is `{expected_digest}`, recomputed `{recomputed_digest}`",
            check.generator
        ));
    }
    assert_eq!(expected_output_digest, recomputed_output_digest);
    assert_eq!(expected_digest, recomputed_digest);
    Ok(())
}

fn validate_gcc40_native_demangle_receipt(project_root: &Path, derivation_content: &str) -> Result<(), String> {
    let path = project_root.join(GCC40_NATIVE_DEMANGLE_RECEIPT);
    let content = fs::read_to_string(&path).map_err(|err| {
        format!(
            "GCC 4.0 native demangle receipt missing `{}` ({err}); expected checked bounded nested-name demangle receipt",
            GCC40_NATIVE_DEMANGLE_RECEIPT
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|err| format!("GCC 4.0 native demangle receipt is not valid JSON: {err}"))?;
    validate_gcc40_demangle_identity(&value)?;
    validate_gcc40_demangle_contract(&value)?;
    validate_gcc40_demangle_source_markers(&value, derivation_content)?;
    validate_gcc40_demangle_smoke(&value)?;
    validate_gcc40_demangle_forbidden_markers(&value, derivation_content)?;
    assert_eq!(value.get("status").and_then(serde_json::Value::as_str), Some("bounded-native-demangle-slice"));
    assert!(path.starts_with(project_root));
    Ok(())
}

fn validate_gcc40_demangle_identity(value: &serde_json::Value) -> Result<(), String> {
    for expectation in [
        StringFieldExpectation {
            field: "schema",
            expected: "mantle-gcc40-native-demangle-slice-v6",
        },
        StringFieldExpectation {
            field: "derivation",
            expected: "bootstrap/gcc-4.0.ncl",
        },
        StringFieldExpectation {
            field: "status",
            expected: "bounded-native-demangle-slice",
        },
        StringFieldExpectation {
            field: "selected_shape",
            expected: "single-short-arg-itanium-v6",
        },
        StringFieldExpectation {
            field: "parity_effect",
            expected: "evidence-backed partial; does not prove native/full GCC 4.0 demangler correctness",
        },
    ] {
        require_gcc40_demangle_string(value, expectation)?;
    }
    assert_eq!(value.get("selected_shape").and_then(serde_json::Value::as_str), Some("single-short-arg-itanium-v6"));
    assert_eq!(value.get("derivation").and_then(serde_json::Value::as_str), Some("bootstrap/gcc-4.0.ncl"));
    Ok(())
}

fn validate_gcc40_demangle_contract(value: &serde_json::Value) -> Result<(), String> {
    let contract = value
        .get("bounded_contract")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "GCC 4.0 native demangle receipt missing object field `bounded_contract`".to_string())?;
    for expectation in GCC40_DEMANGLE_IO_EXPECTATIONS {
        require_gcc40_demangle_named_io(contract, *expectation)?;
    }
    let rejected = contract.get("rejected_inputs").and_then(serde_json::Value::as_array).ok_or_else(|| {
        "GCC 4.0 native demangle receipt missing array field `bounded_contract.rejected_inputs`".to_string()
    })?;
    for required in GCC40_DEMANGLE_REJECTED_INPUTS {
        if !rejected.iter().any(|value| value.as_str() == Some(required)) {
            return Err(format!("GCC 4.0 native demangle receipt rejected inputs missing `{required}`"));
        }
    }
    let non_claim = require_gcc40_demangle_object_string(contract, "non_claim")?;
    if !non_claim.contains("single-short") || !non_claim.contains("in scope") {
        return Err("GCC 4.0 native demangle receipt non-claim must bound the supported shape".to_string());
    }
    assert!(
        GCC40_DEMANGLE_REJECTED_INPUTS
            .iter()
            .all(|required| rejected.iter().any(|value| value.as_str() == Some(required)))
    );
    assert!(non_claim.contains("single-short"));
    Ok(())
}

fn validate_gcc40_demangle_source_markers(value: &serde_json::Value, derivation_content: &str) -> Result<(), String> {
    let markers = value
        .get("source_markers")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "GCC 4.0 native demangle receipt missing object field `source_markers`".to_string())?;
    for field in ["cplus_demangle_marker", "cp_demangle_marker"] {
        let marker = require_gcc40_demangle_object_string(markers, field)?;
        require_gcc40_derivation_marker(DerivationMarkerCheck {
            content: derivation_content,
            marker,
        })?;
    }
    assert!(markers.contains_key("cplus_demangle_marker"));
    assert!(markers.contains_key("cp_demangle_marker"));
    Ok(())
}

fn validate_gcc40_demangle_smoke(value: &serde_json::Value) -> Result<(), String> {
    let smoke = value
        .get("smoke")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "GCC 4.0 native demangle receipt missing object field `smoke`".to_string())?;
    let transcript = require_gcc40_demangle_object_string(smoke, "transcript")?;
    for required in GCC40_DEMANGLE_TRANSCRIPT_FRAGMENTS {
        if !transcript.contains(required) {
            return Err(format!("GCC 4.0 native demangle receipt transcript missing `{required}`"));
        }
    }
    let expected_digest = require_gcc40_demangle_object_string(smoke, "transcript_digest_blake3")?;
    let recomputed_digest = blake3::hash(transcript.as_bytes()).to_hex().to_string();
    if expected_digest != recomputed_digest {
        return Err(format!(
            "GCC 4.0 native demangle receipt transcript digest is `{expected_digest}`, recomputed `{recomputed_digest}`"
        ));
    }
    require_gcc40_demangle_object_string(smoke, "output_digest_blake3")?;
    assert_eq!(expected_digest, recomputed_digest);
    assert!(GCC40_DEMANGLE_TRANSCRIPT_FRAGMENTS.iter().all(|required| transcript.contains(required)));
    Ok(())
}

fn validate_gcc40_demangle_forbidden_markers(
    value: &serde_json::Value,
    derivation_content: &str,
) -> Result<(), String> {
    let forbidden = value
        .get("forbidden_stale_markers")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "GCC 4.0 native demangle receipt missing array field `forbidden_stale_markers`".to_string())?;
    if forbidden.is_empty() {
        return Err("GCC 4.0 native demangle receipt forbidden stale marker list must not be empty".to_string());
    }
    for marker_value in forbidden {
        let marker = marker_value
            .as_str()
            .ok_or_else(|| "GCC 4.0 native demangle receipt forbidden stale markers must be strings".to_string())?;
        if marker.trim().is_empty() {
            return Err("GCC 4.0 native demangle receipt forbidden stale markers must not be empty".to_string());
        }
        if derivation_content.contains(marker) {
            return Err(format!(
                "GCC 4.0 native demangle receipt derivation still contains forbidden stale marker `{marker}`"
            ));
        }
    }
    assert!(!forbidden.is_empty());
    assert!(
        forbidden
            .iter()
            .all(|value| value.as_str().is_some_and(|marker| !derivation_content.contains(marker)))
    );
    Ok(())
}

fn require_gcc40_demangle_named_io(
    contract: &serde_json::Map<String, serde_json::Value>,
    expectation: DemangleIoExpectation,
) -> Result<(), String> {
    let entries = contract.get(expectation.field).and_then(serde_json::Value::as_array).ok_or_else(|| {
        format!("GCC 4.0 native demangle receipt missing array field `bounded_contract.{}`", expectation.field)
    })?;
    if entries.iter().any(|entry| {
        entry.get("mangled").and_then(serde_json::Value::as_str) == Some(expectation.mangled)
            && entry.get("demangled").and_then(serde_json::Value::as_str) == Some(expectation.demangled)
    }) {
        Ok(())
    } else {
        Err(format!(
            "GCC 4.0 native demangle receipt `{}` missing `{} -> {}`",
            expectation.field, expectation.mangled, expectation.demangled
        ))
    }
}

fn require_gcc40_demangle_string<'a>(
    value: &'a serde_json::Value,
    expectation: StringFieldExpectation<'_>,
) -> Result<&'a str, String> {
    let actual = value
        .get(expectation.field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("GCC 4.0 native demangle receipt missing string field `{}`", expectation.field))?;
    if actual != expectation.expected {
        return Err(format!(
            "GCC 4.0 native demangle receipt `{}` is `{actual}`, expected `{}`",
            expectation.field, expectation.expected
        ));
    }
    Ok(actual)
}

fn require_gcc40_demangle_object_string<'a>(
    value: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a str, String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("GCC 4.0 native demangle receipt missing string field `{field}`"))?;
    if actual.trim().is_empty() {
        return Err(format!("GCC 4.0 native demangle receipt `{field}` must not be empty"));
    }
    Ok(actual)
}

fn require_gcc40_generator_string<'a>(
    value: &'a serde_json::Value,
    expectation: StringFieldExpectation<'_>,
) -> Result<&'a str, String> {
    let actual = value
        .get(expectation.field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("GCC 4.0 native generator receipt missing string field `{}`", expectation.field))?;
    if actual != expectation.expected {
        return Err(format!(
            "GCC 4.0 native generator receipt `{}` is `{actual}`, expected `{}`",
            expectation.field, expectation.expected
        ));
    }
    Ok(actual)
}

fn require_gcc40_generator_object_string<'a>(
    value: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a str, String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("GCC 4.0 native generator receipt missing string field `{field}`"))?;
    if actual.trim().is_empty() {
        return Err(format!("GCC 4.0 native generator receipt `{field}` must not be empty"));
    }
    Ok(actual)
}

fn require_gcc40_arithmetic_string<'a>(
    value: &'a serde_json::Value,
    expectation: StringFieldExpectation<'_>,
) -> Result<&'a str, String> {
    let actual = value
        .get(expectation.field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("GCC 4.0 native cc1 arithmetic receipt missing string field `{}`", expectation.field))?;
    if actual != expectation.expected {
        return Err(format!(
            "GCC 4.0 native cc1 arithmetic receipt `{}` is `{actual}`, expected `{}`",
            expectation.field, expectation.expected
        ));
    }
    Ok(actual)
}

fn require_gcc40_arithmetic_object_string<'a>(
    value: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a str, String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("GCC 4.0 native cc1 arithmetic receipt missing string field `{field}`"))?;
    if actual.trim().is_empty() {
        return Err(format!("GCC 4.0 native cc1 arithmetic receipt `{field}` must not be empty"));
    }
    Ok(actual)
}

fn require_gcc40_derivation_marker(check: DerivationMarkerCheck<'_>) -> Result<(), String> {
    if check.marker.trim().is_empty() {
        return Err("GCC 4.0 native boundary receipt marker must not be empty".to_string());
    }
    if !check.content.contains(check.marker) {
        return Err(format!("GCC 4.0 native boundary receipt marker not found in derivation: {}", check.marker));
    }
    Ok(())
}

fn require_gcc40_boundary_string<'a>(
    value: &'a serde_json::Value,
    expectation: StringFieldExpectation<'_>,
) -> Result<&'a str, String> {
    let actual = value
        .get(expectation.field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("GCC 4.0 native boundary receipt missing string field `{}`", expectation.field))?;
    if actual != expectation.expected {
        return Err(format!(
            "GCC 4.0 native boundary receipt `{}` is `{actual}`, expected `{}`",
            expectation.field, expectation.expected
        ));
    }
    Ok(actual)
}

fn require_gcc40_boundary_object<'a>(
    value: &'a serde_json::Value,
    field: &str,
) -> Result<&'a serde_json::Map<String, serde_json::Value>, String> {
    value
        .get(field)
        .and_then(|v| v.as_object())
        .ok_or_else(|| format!("GCC 4.0 native boundary receipt missing object field `{field}`"))
}

fn require_gcc40_boundary_object_string<'a>(
    value: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a str, String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("GCC 4.0 native boundary receipt missing string field `{field}`"))?;
    if actual.trim().is_empty() {
        return Err(format!("GCC 4.0 native boundary receipt `{field}` must not be empty"));
    }
    Ok(actual)
}

fn require_gcc40_inventory_string<'a>(
    value: &'a serde_json::Value,
    expectation: StringFieldExpectation<'_>,
) -> Result<&'a str, String> {
    let actual = value
        .get(expectation.field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("GCC 4.0 placeholder inventory missing string field `{}`", expectation.field))?;
    if actual != expectation.expected {
        return Err(format!(
            "GCC 4.0 placeholder inventory `{}` is `{actual}`, expected `{}`",
            expectation.field, expectation.expected
        ));
    }
    Ok(actual)
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct PlaceholderMarkerOccurrence {
    line: usize,
    marker: String,
}

fn collect_placeholder_marker_occurrences(content: &str) -> Vec<PlaceholderMarkerOccurrence> {
    let line_count = content.lines().count();
    let Some(occurrence_capacity) = line_count.checked_mul(PLACEHOLDER_MARKERS.len()) else {
        return Vec::new();
    };
    let mut occurrences = Vec::with_capacity(occurrence_capacity);
    for (line_index, line) in content.lines().enumerate() {
        let Some(line_number) = line_index.checked_add(1) else {
            break;
        };
        for marker in PLACEHOLDER_MARKERS {
            if marker_is_standalone(line, *marker) {
                occurrences.push(PlaceholderMarkerOccurrence {
                    line: line_number,
                    marker: (*marker).to_string(),
                });
            }
        }
    }
    assert!(occurrences.len() <= occurrence_capacity);
    assert!(occurrences.capacity() >= occurrences.len());
    occurrences
}

fn format_marker_occurrences(occurrences: &[PlaceholderMarkerOccurrence]) -> String {
    occurrences
        .iter()
        .map(|occurrence| format!("{}:{}", occurrence.line, occurrence.marker))
        .collect::<Vec<_>>()
        .join(",")
}

fn validate_stagex_lineage_provider_receipt(project_root: &Path) -> Result<(), String> {
    let path = project_root.join(STAGEX_LINEAGE_PROVIDER_RECEIPT);
    let content = fs::read_to_string(&path).map_err(|err| {
        format!(
            "StageX lineage provider receipt missing `{}` ({err}); expected a complete provider receipt with four roles, stage reports, digest fields, and fallback_events=[]",
            STAGEX_LINEAGE_PROVIDER_RECEIPT
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|err| format!("StageX lineage provider receipt is not valid JSON: {err}"))?;
    require_stagex_json_string(&value, StringFieldExpectation {
        field: "schema",
        expected: "mantle-stagex-lineage-provider-receipt-v1",
    })?;
    require_stagex_json_string(&value, StringFieldExpectation {
        field: "schema_version",
        expected: "mantle-stagex-lineage-provider-receipt-v1",
    })?;
    require_stagex_json_string(&value, StringFieldExpectation {
        field: "provider_kind",
        expected: "stagex-lineage",
    })?;
    require_stagex_json_string(&value, StringFieldExpectation {
        field: "lineage_receipt_status",
        expected: "complete",
    })?;
    for field in [
        "audited_seed_digest",
        "lineage_manifest_digest",
        "stage_graph_digest",
        "normalized_provider_digest",
        "transition_report_digest_blake3",
        "provider_validation_audit_digest_blake3",
        "provider_validation_report_digest_blake3",
        "receipt_payload_digest_blake3",
        "plan_digest_blake3",
        "lineage_manifest_digest_blake3",
        "stage_graph_digest_blake3",
        "source_state_digest_blake3",
        "normalized_provider_digest_blake3",
        "output_digest_blake3",
        "protected_exec_audit_digest_blake3",
        "final_bundle_digest_blake3",
    ] {
        let digest = require_stagex_non_empty_string(&value, field)?;
        validate_lower_hex_digest(DigestFieldCheck { digest, field })?;
    }
    require_stagex_empty_array(&value, "fallback_events")?;
    validate_stagex_receipt_claim_boundary(&value)?;
    validate_stagex_provider_outputs(&value)?;
    validate_stagex_stage_reports(&value)?;
    crate::stagex_provider::validate_lineage_receipt_payload_digest(content.as_bytes())?;
    assert_eq!(value.get("provider_kind").and_then(serde_json::Value::as_str), Some("stagex-lineage"));
    assert_eq!(value.get("lineage_receipt_status").and_then(serde_json::Value::as_str), Some("complete"));
    Ok(())
}

fn require_stagex_json_string<'a>(
    value: &'a serde_json::Value,
    expectation: StringFieldExpectation<'_>,
) -> Result<&'a str, String> {
    let actual = require_stagex_non_empty_string(value, expectation.field)?;
    if actual != expectation.expected {
        return Err(format!(
            "StageX lineage provider receipt `{}` is `{actual}`, expected `{}`",
            expectation.field, expectation.expected
        ));
    }
    Ok(actual)
}

fn require_stagex_non_empty_string<'a>(value: &'a serde_json::Value, field: &str) -> Result<&'a str, String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("StageX lineage provider receipt missing string field `{field}`"))?;
    if actual.trim().is_empty() {
        return Err(format!("StageX lineage provider receipt `{field}` must not be empty"));
    }
    Ok(actual)
}

fn validate_lower_hex_digest(check: DigestFieldCheck<'_>) -> Result<(), String> {
    if check.digest.len() != BLAKE3_HEX_LENGTH
        || !check.digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!(
            "StageX lineage provider receipt `{}` must be a 64-character lowercase hex digest",
            check.field
        ));
    }
    Ok(())
}

fn require_stagex_empty_array(value: &serde_json::Value, field: &str) -> Result<(), String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("StageX lineage provider receipt missing array field `{field}`"))?;
    if !actual.is_empty() {
        return Err(format!("StageX lineage provider receipt `{field}` must be empty"));
    }
    Ok(())
}

fn validate_stagex_receipt_claim_boundary(value: &serde_json::Value) -> Result<(), String> {
    for field in [
        "bounded_claim",
        "stage_report_output_binding",
        "executable_authorization_binding",
    ] {
        require_stagex_non_empty_string(value, field)?;
    }
    let non_claims = value
        .get("non_claims")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "StageX lineage provider receipt missing array field `non_claims`".to_string())?;
    if non_claims.len() < STAGEX_NON_CLAIM_COUNT_MIN {
        return Err(format!(
            "StageX lineage provider receipt has {} non-claims; expected at least {STAGEX_NON_CLAIM_COUNT_MIN}",
            non_claims.len()
        ));
    }
    if non_claims.iter().any(|item| item.as_str().is_none_or(str::is_empty)) {
        return Err("StageX lineage provider receipt has an empty non-claim".to_string());
    }
    assert!(non_claims.len() >= STAGEX_NON_CLAIM_COUNT_MIN);
    assert!(non_claims.iter().all(serde_json::Value::is_string));
    Ok(())
}

fn validate_stagex_provider_outputs(value: &serde_json::Value) -> Result<(), String> {
    const REQUIRED_ROLES: [&str; STAGEX_PROVIDER_ROLE_COUNT] =
        ["target_prefixed_tools", "headers", "libraries", "provider_metadata"];
    let outputs = value
        .get("provider_outputs")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "StageX lineage provider receipt missing array field `provider_outputs`".to_string())?;
    let roles = outputs
        .iter()
        .filter_map(|output| output.get("role").and_then(serde_json::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    for required_role in REQUIRED_ROLES {
        if !roles.contains(required_role) {
            return Err(format!("StageX lineage provider receipt missing provider role `{required_role}`"));
        }
    }
    for output in outputs {
        let role = output
            .get("role")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "StageX provider output is missing string field `role`".to_string())?;
        let artifact_ids = output
            .get("artifact_ids")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("StageX provider output `{role}` is missing `artifact_ids`"))?;
        if artifact_ids.is_empty() || artifact_ids.iter().any(|id| id.as_str().is_none_or(str::is_empty)) {
            return Err(format!("StageX provider output `{role}` must bind non-empty artifact IDs"));
        }
        let digest = output
            .get("digest_blake3")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("StageX provider output `{role}` is missing `digest_blake3`"))?;
        validate_lower_hex_digest(DigestFieldCheck {
            digest,
            field: "provider_outputs.digest_blake3",
        })?;
    }
    if outputs.len() != REQUIRED_ROLES.len() {
        return Err(format!(
            "StageX lineage provider receipt has {} provider roles, expected {}",
            outputs.len(),
            REQUIRED_ROLES.len()
        ));
    }
    assert_eq!(roles.len(), REQUIRED_ROLES.len());
    assert_eq!(outputs.len(), REQUIRED_ROLES.len());
    Ok(())
}

fn validate_stagex_stage_reports(value: &serde_json::Value) -> Result<(), String> {
    let reports = value
        .get("stage_reports")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "StageX lineage provider receipt missing array field `stage_reports`".to_string())?;
    if reports.is_empty() {
        return Err("StageX lineage provider receipt `stage_reports` must not be empty".to_string());
    }
    for report in reports {
        let stage_id = report
            .get("stage_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "StageX stage report is missing string field `stage_id`".to_string())?;
        if report.get("status").and_then(serde_json::Value::as_str) != Some("complete") {
            return Err(format!("StageX stage report `{stage_id}` is not complete"));
        }
        let stage_plan_digest = report
            .get("stage_plan_digest_blake3")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("StageX stage report `{stage_id}` is missing its plan digest"))?;
        validate_lower_hex_digest(DigestFieldCheck {
            digest: stage_plan_digest,
            field: "stage_reports.stage_plan_digest_blake3",
        })?;
        report
            .get("predecessor_reports")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("StageX stage report `{stage_id}` is missing predecessor reports"))?;
        let events = report
            .get("executable_event_ids")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("StageX stage report `{stage_id}` is missing executable event IDs"))?;
        let outputs = report
            .get("output_observations")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("StageX stage report `{stage_id}` is missing output observations"))?;
        if events.is_empty() || events.iter().any(|event| event.as_str().is_none_or(str::is_empty)) {
            return Err(format!("StageX stage report `{stage_id}` must bind non-empty executable event IDs"));
        }
        if outputs.is_empty() {
            return Err(format!("StageX stage report `{stage_id}` must bind output observations"));
        }
        for output in outputs {
            let artifact_id = output
                .get("artifact_id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| format!("StageX stage report `{stage_id}` has an output without an artifact ID"))?;
            let digest = output
                .get("digest_blake3")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| format!("StageX stage report `{stage_id}` output `{artifact_id}` lacks a digest"))?;
            if artifact_id.is_empty() {
                return Err(format!("StageX stage report `{stage_id}` has an empty artifact ID"));
            }
            validate_lower_hex_digest(DigestFieldCheck {
                digest,
                field: "stage_reports.output_observations.digest_blake3",
            })?;
        }
    }
    assert!(!reports.is_empty());
    assert!(
        reports
            .iter()
            .all(|report| report.get("status").and_then(serde_json::Value::as_str) == Some("complete"))
    );
    Ok(())
}

#[derive(Debug, Clone)]
struct DeterministicProofValidation {
    rebuild_descriptor_blake3: String,
    rebuild_authority_plan_blake3: String,
    digest_blake3: String,
    source_blake3: String,
    vendor_blake3: String,
}

#[derive(Debug, Clone)]
struct SandboxProofValidation {
    digest_blake3: String,
    profile_identity: String,
}

#[derive(Debug, Clone)]
struct SummaryProofValidation {
    json_digest_blake3: String,
    markdown_digest_blake3: String,
    bounded_claim: String,
}

#[derive(Debug, Copy, Clone)]
struct VerifyReceiptLinkage<'a> {
    proof_digest_blake3: &'a str,
    sandbox_evidence_digest_blake3: &'a str,
}

#[derive(Debug, Copy, Clone)]
struct SummaryLinkage<'a> {
    release_id: &'a str,
    proof_digest_blake3: &'a str,
    sandbox_evidence_digest_blake3: &'a str,
    selected_provider_kind: ProviderKind,
}

// r[impl mantle.release_provenance.deterministic_rebuild_admission.validation]
fn validate_real_self_build_proof_parity_evidence(project_root: &Path) -> Result<EvidenceValidation, String> {
    validate_self_build_provider_kind_linkage(project_root)?;
    let path = project_root.join(REAL_SELF_BUILD_PROOF_PARITY_RECEIPT);
    let content = fs::read_to_string(&path).map_err(|err| {
        format!(
            "real self-build proof parity receipt missing `{}` ({err}); expected bounded deterministic proof descriptor",
            REAL_SELF_BUILD_PROOF_PARITY_RECEIPT
        )
    })?;
    let evidence_digest_blake3 = blake3::hash(content.as_bytes()).to_hex().to_string();
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|err| format!("real self-build proof parity receipt is not valid JSON: {err}"))?;
    let (release_id, selected_provider_kind) = validate_real_proof_identity(&value)?;
    let provider_kind_linkage = require_real_proof_object(&value, "provider_kind_linkage")?;
    require_real_proof_object_string(provider_kind_linkage, StringFieldExpectation {
        field: "receipt_path",
        expected: SELF_BUILD_PROVIDER_KIND_LINKAGE_RECEIPT,
    })?;
    require_real_proof_object_provider_kind(provider_kind_linkage, "selected_provider_kind", selected_provider_kind)?;
    let deterministic = validate_real_deterministic_proof(&value, selected_provider_kind)?;
    let sandbox = validate_real_sandbox_evidence(&value)?;
    let verify_receipt_digest_blake3 = validate_real_verify_receipt(&value, VerifyReceiptLinkage {
        proof_digest_blake3: &deterministic.digest_blake3,
        sandbox_evidence_digest_blake3: &sandbox.digest_blake3,
    })?;
    let summary = validate_real_proof_summary(&value, SummaryLinkage {
        release_id: &release_id,
        proof_digest_blake3: &deterministic.digest_blake3,
        sandbox_evidence_digest_blake3: &sandbox.digest_blake3,
        selected_provider_kind,
    })?;
    require_real_proof_string(&value, StringFieldExpectation {
        field: "source_blake3",
        expected: &deterministic.source_blake3,
    })?;
    require_real_proof_string(&value, StringFieldExpectation {
        field: "vendor_blake3",
        expected: &deterministic.vendor_blake3,
    })?;
    assert_eq!(evidence_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    assert!(path.starts_with(project_root));
    Ok(EvidenceValidation::with_proof(ParityProofDetails {
        schema: "mantle-real-self-build-proof-parity-evidence-v1".to_string(),
        release_id,
        selected_provider_kind,
        evidence_digest_blake3,
        deterministic_proof_digest_blake3: deterministic.digest_blake3,
        rebuild_descriptor_blake3: deterministic.rebuild_descriptor_blake3,
        rebuild_authority_plan_blake3: deterministic.rebuild_authority_plan_blake3,
        sandbox_evidence_digest_blake3: sandbox.digest_blake3,
        verify_receipt_digest_blake3,
        summary_json_digest_blake3: summary.json_digest_blake3,
        summary_markdown_digest_blake3: summary.markdown_digest_blake3,
        verdict: "self-rebuild-match".to_string(),
        verify_status: "eligible".to_string(),
        sandbox_profile_identity: sandbox.profile_identity,
        bounded_claim: summary.bounded_claim,
    }))
}

fn validate_real_proof_identity(value: &serde_json::Value) -> Result<(String, ProviderKind), String> {
    require_real_proof_string(value, StringFieldExpectation {
        field: "schema",
        expected: "mantle-real-self-build-proof-parity-evidence-v1",
    })?;
    let release_id = require_real_proof_non_empty_string(value, "release_id")?.to_string();
    let provider_kind_text = require_real_proof_non_empty_string(value, "selected_provider_kind")?;
    let selected_provider_kind = parse_provider_kind(ProviderKindFieldCheck {
        kind: provider_kind_text,
        field: "selected_provider_kind",
    })?;
    assert!(!release_id.is_empty());
    assert_ne!(selected_provider_kind, ProviderKind::Unknown);
    Ok((release_id, selected_provider_kind))
}

fn validate_real_deterministic_proof(
    value: &serde_json::Value,
    selected_provider_kind: ProviderKind,
) -> Result<DeterministicProofValidation, String> {
    let proof = require_real_proof_object(value, "deterministic_proof")?;
    require_real_proof_object_string(proof, StringFieldExpectation {
        field: "workflow",
        expected: "mantle-deterministic-proof-receipt-v2",
    })?;
    require_real_proof_object_string(proof, StringFieldExpectation {
        field: "verdict",
        expected: "self-rebuild-match",
    })?;
    require_real_proof_object_bool(proof, "genuine_rebuild_authority", true)?;
    require_real_proof_object_provider_kind(proof, "selected_provider_kind", selected_provider_kind)?;
    validate_real_clean_rebuild_roots(proof)?;
    validate_real_artifact_digests(proof)?;
    let validated = DeterministicProofValidation {
        rebuild_descriptor_blake3: require_real_proof_object_digest(proof, "rebuild_descriptor_blake3")?.to_string(),
        rebuild_authority_plan_blake3: require_real_proof_object_digest(proof, "rebuild_authority_plan_blake3")?
            .to_string(),
        digest_blake3: require_real_proof_object_digest(proof, "digest_blake3")?.to_string(),
        source_blake3: require_real_proof_object_digest(proof, "source_blake3")?.to_string(),
        vendor_blake3: require_real_proof_object_digest(proof, "vendor_blake3")?.to_string(),
    };
    assert_eq!(validated.digest_blake3.len(), BLAKE3_HEX_LENGTH);
    assert_eq!(validated.source_blake3.len(), BLAKE3_HEX_LENGTH);
    Ok(validated)
}

fn validate_real_clean_rebuild_roots(proof: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    let roots = proof.get("clean_rebuild_roots").and_then(serde_json::Value::as_array).ok_or_else(|| {
        "real self-build proof parity receipt missing array field `deterministic_proof.clean_rebuild_roots`".to_string()
    })?;
    if roots.len() != CLEAN_REBUILD_ROOT_COUNT {
        return Err(format!(
            "real self-build proof parity receipt `deterministic_proof.clean_rebuild_roots` has length {}, expected {}",
            roots.len(),
            CLEAN_REBUILD_ROOT_COUNT
        ));
    }
    let first_root = roots
        .first()
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "real self-build proof clean rebuild root must be a string".to_string())?;
    let second_root = roots
        .get(1)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "real self-build proof clean rebuild root must be a string".to_string())?;
    if first_root.trim().is_empty() || second_root.trim().is_empty() {
        return Err("real self-build proof clean rebuild roots must not be empty".to_string());
    }
    if first_root == second_root {
        return Err("real self-build proof clean rebuild roots must be distinct".to_string());
    }
    assert_eq!(roots.len(), CLEAN_REBUILD_ROOT_COUNT);
    assert_ne!(first_root, second_root);
    Ok(())
}

fn validate_real_artifact_digests(proof: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    let artifacts = proof.get("artifact_digests_blake3").and_then(serde_json::Value::as_array).ok_or_else(|| {
        "real self-build proof parity receipt missing array field `deterministic_proof.artifact_digests_blake3`"
            .to_string()
    })?;
    if artifacts.is_empty() {
        return Err("real self-build proof artifact digest set must not be empty".to_string());
    }
    for artifact in artifacts {
        let digest = artifact
            .as_str()
            .ok_or_else(|| "real self-build proof artifact digest must be a string".to_string())?;
        validate_blake3_digest(DigestFieldCheck {
            digest,
            field: "deterministic_proof.artifact_digests_blake3",
        })?;
    }
    assert!(!artifacts.is_empty());
    assert!(artifacts.iter().all(serde_json::Value::is_string));
    Ok(())
}

fn validate_real_sandbox_evidence(value: &serde_json::Value) -> Result<SandboxProofValidation, String> {
    let sandbox = require_real_proof_object(value, "sandbox_evidence")?;
    let profile_identity = require_real_proof_object_non_empty_string(sandbox, "profile_identity")?.to_string();
    if !profile_identity.starts_with("mantle-proof-sandbox-v1:") {
        return Err(format!(
            "real self-build proof sandbox profile `{profile_identity}` is unsupported; expected mantle-proof-sandbox-v1:*"
        ));
    }
    let validated = SandboxProofValidation {
        digest_blake3: require_real_proof_object_digest(sandbox, "digest_blake3")?.to_string(),
        profile_identity,
    };
    assert_eq!(validated.digest_blake3.len(), BLAKE3_HEX_LENGTH);
    assert!(validated.profile_identity.starts_with("mantle-proof-sandbox-v1:"));
    Ok(validated)
}

fn validate_real_verify_receipt(
    value: &serde_json::Value,
    linkage: VerifyReceiptLinkage<'_>,
) -> Result<String, String> {
    let receipt = require_real_proof_object(value, "verify_receipt")?;
    let digest = require_real_proof_object_digest(receipt, "digest_blake3")?.to_string();
    for expectation in [
        StringFieldExpectation {
            field: "deterministic_release_status",
            expected: "eligible",
        },
        StringFieldExpectation {
            field: "proof_digest_blake3",
            expected: linkage.proof_digest_blake3,
        },
        StringFieldExpectation {
            field: "sandbox_evidence_digest_blake3",
            expected: linkage.sandbox_evidence_digest_blake3,
        },
    ] {
        require_real_proof_object_string(receipt, expectation)?;
    }
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    assert_eq!(receipt.get("deterministic_release_status").and_then(serde_json::Value::as_str), Some("eligible"));
    Ok(digest)
}

fn validate_real_proof_summary(
    value: &serde_json::Value,
    linkage: SummaryLinkage<'_>,
) -> Result<SummaryProofValidation, String> {
    let summary = require_real_proof_object(value, "summary")?;
    for expectation in [
        StringFieldExpectation {
            field: "release_id",
            expected: linkage.release_id,
        },
        StringFieldExpectation {
            field: "verdict",
            expected: "self-rebuild-match",
        },
        StringFieldExpectation {
            field: "verify_status",
            expected: "eligible",
        },
        StringFieldExpectation {
            field: "proof_digest_blake3",
            expected: linkage.proof_digest_blake3,
        },
        StringFieldExpectation {
            field: "sandbox_evidence_digest_blake3",
            expected: linkage.sandbox_evidence_digest_blake3,
        },
    ] {
        require_real_proof_object_string(summary, expectation)?;
    }
    require_real_proof_object_provider_kind(summary, "selected_provider_kind", linkage.selected_provider_kind)?;
    let bounded_claim = require_real_proof_object_non_empty_string(summary, "bounded_claim")?.to_string();
    if !bounded_claim.contains("rebuilt twice") || !bounded_claim.contains("exact content") {
        return Err("real self-build proof bounded claim must state rebuilt-twice exact-content scope".to_string());
    }
    validate_real_proof_non_claims(summary)?;
    let validated = SummaryProofValidation {
        json_digest_blake3: require_real_proof_object_digest(summary, "json_digest_blake3")?.to_string(),
        markdown_digest_blake3: require_real_proof_object_digest(summary, "markdown_digest_blake3")?.to_string(),
        bounded_claim,
    };
    assert!(validated.bounded_claim.contains("rebuilt twice"));
    assert_eq!(validated.json_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    Ok(validated)
}

fn validate_real_proof_non_claims(summary: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    let non_claims = summary
        .get("non_claims")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "real self-build proof summary missing array field `non_claims`".to_string())?;
    let required_non_claims = ["compiler or verifier soundness", "full bootstrap reproducibility"];
    for required_non_claim in required_non_claims {
        let is_present =
            non_claims.iter().any(|claim| claim.as_str().is_some_and(|text| text.contains(required_non_claim)));
        if !is_present {
            return Err(format!("real self-build proof summary non_claims must reject {required_non_claim}"));
        }
    }
    assert!(!non_claims.is_empty());
    assert!(
        required_non_claims.iter().all(|required| {
            non_claims.iter().any(|claim| claim.as_str().is_some_and(|text| text.contains(required)))
        })
    );
    Ok(())
}

fn parse_provider_kind(check: ProviderKindFieldCheck<'_>) -> Result<ProviderKind, String> {
    match check.kind {
        "legacy-fetch" => Ok(ProviderKind::LegacyFetch),
        "full-source" => Ok(ProviderKind::FullSource),
        "source-root" => Ok(ProviderKind::SourceRoot),
        "stagex-lineage" => Ok(ProviderKind::StagexLineage),
        other => Err(format!(
            "real self-build proof parity receipt `{}` has unknown provider kind `{other}`",
            check.field
        )),
    }
}

fn require_real_proof_string<'a>(
    value: &'a serde_json::Value,
    expectation: StringFieldExpectation<'_>,
) -> Result<&'a str, String> {
    let actual = require_real_proof_non_empty_string(value, expectation.field)?;
    if actual != expectation.expected {
        return Err(format!(
            "real self-build proof parity receipt `{}` is `{actual}`, expected `{}`",
            expectation.field, expectation.expected
        ));
    }
    Ok(actual)
}

fn require_real_proof_non_empty_string<'a>(value: &'a serde_json::Value, field: &str) -> Result<&'a str, String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("real self-build proof parity receipt missing string field `{field}`"))?;
    if actual.trim().is_empty() {
        return Err(format!("real self-build proof parity receipt `{field}` must not be empty"));
    }
    Ok(actual)
}

fn require_real_proof_object<'a>(
    value: &'a serde_json::Value,
    field: &str,
) -> Result<&'a serde_json::Map<String, serde_json::Value>, String> {
    value
        .get(field)
        .and_then(|v| v.as_object())
        .ok_or_else(|| format!("real self-build proof parity receipt missing object field `{field}`"))
}

fn require_real_proof_object_string<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    expectation: StringFieldExpectation<'_>,
) -> Result<&'a str, String> {
    let actual = require_real_proof_object_non_empty_string(object, expectation.field)?;
    if actual != expectation.expected {
        return Err(format!(
            "real self-build proof parity receipt `{}` is `{actual}`, expected `{}`",
            expectation.field, expectation.expected
        ));
    }
    Ok(actual)
}

fn require_real_proof_object_non_empty_string<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a str, String> {
    let actual = object
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("real self-build proof parity receipt missing string field `{field}`"))?;
    if actual.trim().is_empty() {
        return Err(format!("real self-build proof parity receipt `{field}` must not be empty"));
    }
    Ok(actual)
}

fn require_real_proof_object_digest<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a str, String> {
    let digest = require_real_proof_object_non_empty_string(object, field)?;
    validate_blake3_digest(DigestFieldCheck { digest, field })?;
    Ok(digest)
}

fn require_real_proof_object_bool(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    expected: bool,
) -> Result<(), String> {
    let is_actual = object
        .get(field)
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| format!("real self-build proof parity receipt missing boolean field `{field}`"))?;
    if is_actual == expected {
        Ok(())
    } else {
        Err(format!("real self-build proof parity receipt `{field}` is `{is_actual}`, expected `{expected}`"))
    }
}

fn require_real_proof_object_provider_kind(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    expected: ProviderKind,
) -> Result<(), String> {
    let actual_provider_kind = parse_provider_kind(ProviderKindFieldCheck {
        kind: require_real_proof_object_non_empty_string(object, field)?,
        field,
    })?;
    if actual_provider_kind != expected {
        return Err(format!(
            "real self-build proof parity receipt provider kind mismatch in `{field}`: {actual_provider_kind}, expected {expected}"
        ));
    }
    Ok(())
}

fn validate_blake3_digest(check: DigestFieldCheck<'_>) -> Result<(), String> {
    if check.digest.len() != BLAKE3_HEX_LENGTH
        || !check.digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!(
            "real self-build proof parity receipt `{}` must be a 64-character lowercase BLAKE3 digest",
            check.field
        ));
    }
    Ok(())
}

fn validate_self_build_provider_kind_linkage(project_root: &Path) -> Result<(), String> {
    let path = project_root.join(SELF_BUILD_PROVIDER_KIND_LINKAGE_RECEIPT);
    let content = fs::read_to_string(&path).map_err(|err| {
        format!(
            "mantle self-build provider-kind linkage receipt missing `{}` ({err}); expected schema, proof_identity.selected_provider_kind, proof_linkage.selected_provider_kind, and prerequisites.provider_kind",
            SELF_BUILD_PROVIDER_KIND_LINKAGE_RECEIPT
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|err| format!("mantle self-build provider-kind linkage receipt is not valid JSON: {err}"))?;

    require_self_build_json_string(&value, StringFieldExpectation {
        field: "schema",
        expected: "mantle-self-build-provider-kind-linkage-v1",
    })?;
    let proof_identity = require_self_build_object(&value, "proof_identity")?;
    let proof_linkage = require_self_build_object(&value, "proof_linkage")?;
    let prerequisites = require_self_build_object(&value, "prerequisites")?;
    let proof_identity_kind = require_self_build_object_string(proof_identity, "selected_provider_kind")?;
    let proof_linkage_kind = require_self_build_object_string(proof_linkage, "selected_provider_kind")?;
    let prerequisites_kind = require_self_build_object_string(prerequisites, "provider_kind")?;

    validate_closed_provider_kind(ProviderKindFieldCheck {
        kind: proof_identity_kind,
        field: "proof_identity.selected_provider_kind",
    })?;
    validate_closed_provider_kind(ProviderKindFieldCheck {
        kind: proof_linkage_kind,
        field: "proof_linkage.selected_provider_kind",
    })?;
    validate_closed_provider_kind(ProviderKindFieldCheck {
        kind: prerequisites_kind,
        field: "prerequisites.provider_kind",
    })?;
    if proof_identity_kind != proof_linkage_kind || proof_identity_kind != prerequisites_kind {
        return Err(format!(
            "crunch self-build provider-kind linkage mismatch: proof_identity.selected_provider_kind={proof_identity_kind}, proof_linkage.selected_provider_kind={proof_linkage_kind}, prerequisites.provider_kind={prerequisites_kind}"
        ));
    }
    assert_eq!(proof_identity_kind, proof_linkage_kind);
    assert_eq!(proof_identity_kind, prerequisites_kind);
    Ok(())
}

fn validate_closed_provider_kind(check: ProviderKindFieldCheck<'_>) -> Result<(), String> {
    match check.kind {
        "legacy-fetch" | "full-source" | "source-root" | "stagex-lineage" => Ok(()),
        other => Err(format!(
            "crunch self-build provider-kind linkage `{}` has unknown provider kind `{other}`",
            check.field
        )),
    }
}

fn require_self_build_json_string<'a>(
    value: &'a serde_json::Value,
    expectation: StringFieldExpectation<'_>,
) -> Result<&'a str, String> {
    let actual = value.get(expectation.field).and_then(serde_json::Value::as_str).ok_or_else(|| {
        format!("crunch self-build provider-kind linkage missing string field `{}`", expectation.field)
    })?;
    if actual != expectation.expected {
        return Err(format!(
            "crunch self-build provider-kind linkage `{}` is `{actual}`, expected `{}`",
            expectation.field, expectation.expected
        ));
    }
    Ok(actual)
}

fn require_self_build_object<'a>(
    value: &'a serde_json::Value,
    field: &str,
) -> Result<&'a serde_json::Map<String, serde_json::Value>, String> {
    value
        .get(field)
        .and_then(|v| v.as_object())
        .ok_or_else(|| format!("crunch self-build provider-kind linkage missing object field `{field}`"))
}

fn require_self_build_object_string<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a str, String> {
    let actual = object
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("crunch self-build provider-kind linkage missing string field `{field}`"))?;
    if actual.trim().is_empty() {
        return Err(format!("crunch self-build provider-kind linkage `{field}` must not be empty"));
    }
    Ok(actual)
}

fn validate_seed_full_source_root_contract(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    let required = [
        "share/crunch-bootstrap/provider.json",
        "\"provider_id\": \"full-source-v1\"",
        "\"target\": \"$PUBLIC_TARGET\"",
        "\"dynamic_linker\": \"$DYNAMIC_LINKER\"",
        "\"source_authority\": \"declared-mantle-derivation-closure\"",
        "\"output_identity\"",
        "\"runtime_admission\"",
        "\"closure_admission\"",
        "\"state_pinned_inputs\": false",
        "\"legacy_members\": []",
        "x86_64-linux-musl-gcc",
        "x86_64-linux-musl-as",
        "x86_64-linux-musl-ld",
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
    assert!(required.iter().all(|needle| content.contains(needle)));
    assert!(forbidden.iter().all(|needle| !content.contains(needle)));
    Ok(())
}

#[cfg(test)]
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
    require_json_string(&value, StringFieldExpectation {
        field: "schema",
        expected: "mantle-binutils-tcc-tool-smoke-v1",
    })?;
    require_json_string(&value, StringFieldExpectation {
        field: "derivation",
        expected: "bootstrap/binutils-tcc.ncl",
    })?;
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
    assert!(BINUTILS_TCC_REQUIRED_TOOLS.iter().all(|tool| tool_smokes.contains_key(*tool)));
    assert_eq!(value.get("host_fallback").and_then(serde_json::Value::as_bool), Some(false));
    Ok(())
}

#[cfg(test)]
fn require_json_string(value: &serde_json::Value, expectation: StringFieldExpectation<'_>) -> Result<(), String> {
    let actual = value
        .get(expectation.field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("binutils-tcc transcript missing string field `{}`", expectation.field))?;
    if actual != expectation.expected {
        return Err(format!(
            "binutils-tcc transcript `{}` is `{actual}`, expected `{}`",
            expectation.field, expectation.expected
        ));
    }
    Ok(())
}

#[cfg(test)]
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

#[cfg(test)]
fn require_json_bool(value: &serde_json::Value, field: &str, expected: bool) -> Result<(), String> {
    let is_actual = value
        .get(field)
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| format!("binutils-tcc transcript missing bool field `{field}`"))?;
    if is_actual != expected {
        return Err(format!("binutils-tcc transcript `{field}` is {is_actual}, expected {expected}"));
    }
    Ok(())
}

#[cfg(test)]
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

#[cfg(test)]
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
fn marker_is_standalone(content: &str, marker: impl AsRef<str>) -> bool {
    let lower = content.to_ascii_lowercase();
    let search = marker.as_ref().to_ascii_lowercase();
    if search.is_empty() {
        return false;
    }
    assert!(!search.is_empty());
    assert_eq!(search, search.to_ascii_lowercase());
    for (absolute_index, _) in lower.match_indices(&search) {
        let Some(end_index) = absolute_index.checked_add(search.len()) else {
            return false;
        };
        let before = lower.get(..absolute_index).unwrap_or("");
        let after = lower.get(end_index..).unwrap_or("");
        let is_prev_word = before.chars().last().is_some_and(|character| character.is_ascii_alphanumeric())
            || before.ends_with('_')
            || before.ends_with('.')
            || before.ends_with('-');
        let is_next_word = after.chars().next().is_some_and(|character| character.is_ascii_alphanumeric())
            || after.starts_with('_')
            || after.starts_with('/')
            || after.starts_with('-');
        let is_prev_quote = before.ends_with('\'');
        let is_next_quote = after.starts_with('\'');
        let line_start = before.rfind('\n').and_then(|index| index.checked_add(1)).unwrap_or(0);
        let line_before = before.get(line_start..).unwrap_or("");
        let is_prev_line_blank = line_before.chars().all(|character| character == ' ' || character == '\t');
        let is_next_line_blank = after
            .chars()
            .take_while(|character| *character != '\n')
            .all(|character| character == ' ' || character == '\t');
        let is_heredoc_terminator = is_prev_line_blank && is_next_line_blank;
        let is_rejected = is_prev_word || is_next_word || (is_prev_quote && is_next_quote) || is_heredoc_terminator;
        if !is_rejected {
            return true;
        }
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
    if PLACEHOLDER_MARKERS.iter().any(|marker| marker_is_standalone(&content, *marker)) {
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

const BOOTSTRAP_FOUNDATION_STAGE_SPECS: &[StageSpec] = &[
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
];

const BOOTSTRAP_TOOL_STAGE_SPECS: &[StageSpec] = &[
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
];

const BOOTSTRAP_GAP_STAGE_SPECS: &[StageSpec] = &[
    StageSpec {
        id: "binutils.tcc",
        title: "binutils TCC early-native row",
        axes: LIVE_GUIX,
        lineage: "live-bootstrap",
        derivation: Some("binutils-tcc.ncl"),
        expected_complete: true,
        graph_evidence: "source-built binutils derivation and independently bound output attestation present",
        semantic_evidence: "assembler/linker/archive/ranlib/nm/objcopy/object-format/relocation positive and malformed-input matrices passed",
        proof_evidence: "BLAKE3-bound early-native binutils row receipt with current source, predecessor, generated-source, output, trust, and fallback facts required",
        notes: "completion is bounded to bootstrap/evidence/early-native-binutils-row-v2.json and does not claim general assembler/linker correctness",
        evidence_check: EvidenceCheck::EarlyNativeBinutilsRow,
    },
    StageSpec {
        id: "gcc.4.0",
        title: "GCC 4.0 early-native row",
        axes: LIVE_GUIX,
        lineage: "live-bootstrap",
        derivation: Some("gcc-4.0.ncl"),
        expected_complete: true,
        graph_evidence: "canonical regenerated GCC 4.0 C/C++ root and independently bound output attestation present",
        semantic_evidence: "C/C++ drivers, cc1/cc1plus, generators, demangler, libgcc, exception runtime, relocation, and malformed-source matrices passed",
        proof_evidence: "BLAKE3-bound GCC 4.0 row receipt with current source, predecessor, generated-source, output, trust, and fallback facts required",
        notes: "completion is bounded to bootstrap/evidence/early-native-gcc40-row-v2.json and does not claim general compiler correctness",
        evidence_check: EvidenceCheck::EarlyNativeGcc40Row,
    },
    StageSpec {
        id: "gcc.4.7",
        title: "GCC 4.7",
        axes: LIVE_GUIX,
        lineage: "live-bootstrap",
        derivation: Some("gcc-4.7.ncl"),
        expected_complete: true,
        graph_evidence: "regenerated GCC 4.7 derivation and independently bound output/predecessor attestations present",
        semantic_evidence: "C/C++11, compiler-internal, static runtime, relocation, malformed-source, and fallback matrices passed",
        proof_evidence: "stage-local BLAKE3 receipt over current sources, generated artifacts, immediate predecessors, output, trust, and rejection facts required",
        notes: "completion is bounded to bootstrap/evidence/final-native-gcc47-row-v1.json and does not claim general compiler correctness",
        evidence_check: EvidenceCheck::FinalNativeGcc47Row,
    },
    StageSpec {
        id: "gcc.10",
        title: "GCC 10",
        axes: LIVE_GUIX,
        lineage: "live-bootstrap",
        derivation: Some("gcc-10.ncl"),
        expected_complete: true,
        graph_evidence: "regenerated GCC 10 derivation and independently bound GCC 4.7/source/output attestations present",
        semantic_evidence: "C/C++17, compiler-internal, static runtime, relocation, malformed-source, and fallback matrices passed",
        proof_evidence: "stage-local BLAKE3 receipt over current sources, generated artifacts, GCC 4.7 lineage, output, trust, and rejection facts required",
        notes: "completion is bounded to bootstrap/evidence/final-native-gcc10-row-v1.json and does not claim general compiler correctness",
        evidence_check: EvidenceCheck::FinalNativeGcc10Row,
    },
    StageSpec {
        id: "full-musl-binutils",
        title: "Full musl/binutils handoff",
        axes: LIVE_GUIX,
        lineage: "guix",
        derivation: Some("binutils-full.ncl"),
        expected_complete: true,
        graph_evidence: "final GCC 10, musl 1.2.5, and binutils 2.41 stage-local artifacts and attestations present",
        semantic_evidence: "static/shared compiler and libc runtimes plus complete binutils, relocation, copied-tree, and rejection matrices passed",
        proof_evidence: "composed BLAKE3 receipt over final GCC, libc/CRT/interpreter, binutils, trust, closure, relocation, and fallback facts required",
        notes: "completion is bounded to bootstrap/evidence/final-native-musl-binutils-row-v1.json and does not claim provider admission or whole-toolchain correctness",
        evidence_check: EvidenceCheck::FinalNativeMuslBinutilsRow,
    },
    StageSpec {
        id: "seed-full",
        title: "Normalized full source seed provider",
        axes: GUIX_ONLY,
        lineage: "guix",
        derivation: Some("seed-full.ncl"),
        expected_complete: true,
        graph_evidence: "selected seed-full derivation present",
        semantic_evidence: "runtime-admitted full-source provider contract present",
        proof_evidence: "authenticated source closure and fixed-point report preserved separately",
        notes: "full-source provider contract evidence satisfies this Guix row; it does not satisfy StageX lineage evidence",
        evidence_check: EvidenceCheck::SeedFullSourceRootContract,
    },
    StageSpec {
        id: "seed-full.stagex-lineage",
        title: "StageX lineage normalized seed provider",
        axes: STAGEX_ONLY,
        lineage: "stagex",
        derivation: None,
        expected_complete: true,
        graph_evidence: "protected StageX transition and normalized provider publication are bound",
        semantic_evidence: "complete four-role provider contract and relocated runtime validation required",
        proof_evidence: "lineage, plan, audit, provider, stage-report, and final-bundle BLAKE3 identities required",
        notes: "Guix source-root evidence does not satisfy this StageX row; completion is bounded to the checked intermediate TinyCC/native-musl/binutils receipt and does not claim final GCC admission",
        evidence_check: EvidenceCheck::StagexLineageProviderReceipt,
    },
];

const BOOTSTRAP_PROOF_STAGE_SPECS: &[StageSpec] = &[
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
        notes: "release evidence must bind selected provider kind; checked receipt at bootstrap/evidence/crunch-self-build-provider-kind-linkage.json validates proof_identity/proof_linkage/prerequisites provider-kind equality but does not prove full self-build",
        evidence_check: EvidenceCheck::RealSelfBuildProof,
    },
];

fn parity_stage_specs() -> impl Iterator<Item = &'static StageSpec> {
    BOOTSTRAP_FOUNDATION_STAGE_SPECS
        .iter()
        .chain(BOOTSTRAP_TOOL_STAGE_SPECS)
        .chain(BOOTSTRAP_GAP_STAGE_SPECS)
        .chain(BOOTSTRAP_PROOF_STAGE_SPECS)
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
            graph_evidence: "selected seed-full derivation present",
            semantic_evidence: "runtime-admitted full-source provider contract present",
            proof_evidence: "authenticated source closure and fixed-point report preserved separately",
            notes: "full-source provider contract evidence satisfies this Guix row; it does not satisfy StageX lineage evidence",
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

    fn gcc40_spec() -> StageSpec {
        StageSpec {
            id: "gcc.4.0",
            title: "GCC 4.0",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("gcc-4.0.ncl"),
            expected_complete: false,
            graph_evidence: "late graph completion recorded",
            semantic_evidence: "bounded libgcc/driver/cc1 arithmetic+logical+local-vars+function-call+array-index+struct-field+pointer-deref, demangle single-short, and generator boundary smokes only; native compiler correctness not proven",
            proof_evidence: "source transcript, placeholder inventory, native-boundary receipt, native-cc1 slice receipt, native-demangle receipt, native-generator receipt, and native-cc1 build/source-frontier receipt required",
            notes: "pass1 bridge and selected bounded semantics are partial progress; checked placeholder inventory at bootstrap/evidence/gcc-4.0-placeholder-inventory.json records remaining marker debt, native-boundary receipt at bootstrap/evidence/gcc-4.0-native-boundary.json records the intentional bridge boundary, native-cc1 receipt at bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json records no-TinyCC-delegation arithmetic, logical/control-flow, local-variable, helper-call, array-index, struct-field, and pointer-deref slices, native-demangle receipt at bootstrap/evidence/gcc-4.0-native-demangle-slice.json records bounded single-short Itanium demangle semantics, native-generator receipt at bootstrap/evidence/gcc-4.0-native-generator-slice.json records bounded genattrtab, genoutput, genemit, genrecog, and genextract slices, and native-cc1 build/source-frontier receipt at bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json records source-build frontier markers plus the bounded unchanged c-parse/gengtype-yacc probe, while remaining native generator/compiler/demangler correctness is still unproven",
            evidence_check: EvidenceCheck::Gcc40PlaceholderInventory,
        }
    }

    fn gcc47_spec() -> StageSpec {
        StageSpec {
            id: "gcc.4.7",
            title: "GCC 4.7",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("gcc-4.7.ncl"),
            expected_complete: false,
            graph_evidence: "derivation present",
            semantic_evidence: "checked C/C++/C++11 provider contract only; native correctness evidence required",
            proof_evidence: "source transcript and checked C++ provider contract required",
            notes: "checked receipt at bootstrap/evidence/gcc-4.7-cxx-provider-contract.json validates the C/C++ configure/build/install/smoke contract; row remains partial until native/full GCC 4.7 correctness is proven",
            evidence_check: EvidenceCheck::Gcc47CxxProviderContract,
        }
    }

    fn full_musl_binutils_spec() -> StageSpec {
        StageSpec {
            id: "full-musl-binutils",
            title: "Full musl/binutils handoff",
            axes: LIVE_GUIX,
            lineage: "guix",
            derivation: Some("binutils-full.ncl"),
            expected_complete: false,
            graph_evidence: "binutils-full plus musl-full derivations present",
            semantic_evidence: "full toolchain smokes required",
            proof_evidence: "source-root proof and checked full musl/binutils provider contract required",
            notes: "checked receipt at bootstrap/evidence/full-musl-binutils-provider-contract.json validates the musl 1.2.5 and binutils 2.41 configure/build/install/smoke contract; row remains partial until full toolchain correctness and source-root proof are proven",
            evidence_check: EvidenceCheck::FullMuslBinutilsProviderContract,
        }
    }

    fn gcc10_spec() -> StageSpec {
        StageSpec {
            id: "gcc.10",
            title: "GCC 10",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("gcc-10.ncl"),
            expected_complete: false,
            graph_evidence: "derivation present",
            semantic_evidence: "checked C/C++/C++11 provider contract only; native correctness evidence required",
            proof_evidence: "source transcript and checked GCC 10 provider contract required",
            notes: "checked receipt at bootstrap/evidence/gcc-10-provider-contract.json validates the C/C++ configure/build/install/smoke contract; row remains partial until native/full GCC 10 correctness is proven",
            evidence_check: EvidenceCheck::Gcc10ProviderContract,
        }
    }

    fn self_build_spec() -> StageSpec {
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
            notes: "release evidence must bind selected provider kind; checked receipt at bootstrap/evidence/crunch-self-build-provider-kind-linkage.json validates proof_identity/proof_linkage/prerequisites provider-kind equality but does not prove full self-build",
            evidence_check: EvidenceCheck::RealSelfBuildProof,
        }
    }

    fn stagex_lineage_spec() -> StageSpec {
        StageSpec {
            id: "seed-full.stagex-lineage",
            title: "StageX lineage normalized seed provider",
            axes: STAGEX_ONLY,
            lineage: "stagex",
            derivation: None,
            expected_complete: true,
            graph_evidence: "protected StageX transition and normalized provider publication are bound",
            semantic_evidence: "complete four-role provider contract and relocated runtime validation required",
            proof_evidence: "lineage, plan, audit, provider, stage-report, and final-bundle BLAKE3 identities required",
            notes: "Guix source-root evidence does not satisfy this StageX row; completion is bounded to the checked intermediate TinyCC/native-musl/binutils receipt and does not claim final GCC admission",
            evidence_check: EvidenceCheck::StagexLineageProviderReceipt,
        }
    }

    fn write_stagex_lineage_receipt(
        root: &Path,
        provider_kind: &str,
        status: &str,
        audited_seed_digest: &str,
        fallback_events: &str,
    ) {
        let path = root.join(STAGEX_LINEAGE_PROVIDER_RECEIPT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            format!(
                r#"{{
  "schema": "mantle-stagex-lineage-provider-receipt-v1",
  "schema_version": "mantle-stagex-lineage-provider-receipt-v1",
  "provider_kind": "{provider_kind}",
  "lineage_receipt_status": "{status}",
  "audited_seed_digest": "{audited_seed_digest}",
  "lineage_manifest_digest": "2222222222222222222222222222222222222222222222222222222222222222",
  "stage_graph_digest": "3333333333333333333333333333333333333333333333333333333333333333",
  "normalized_provider_digest": "4444444444444444444444444444444444444444444444444444444444444444",
  "transition_report_digest_blake3": "5555555555555555555555555555555555555555555555555555555555555555",
  "provider_validation_audit_digest_blake3": "5555555555555555555555555555555555555555555555555555555555555555",
  "provider_validation_report_digest_blake3": "5555555555555555555555555555555555555555555555555555555555555555",
  "receipt_payload_digest_blake3": "",
  "plan_digest_blake3": "6666666666666666666666666666666666666666666666666666666666666666",
  "lineage_manifest_digest_blake3": "2222222222222222222222222222222222222222222222222222222222222222",
  "stage_graph_digest_blake3": "3333333333333333333333333333333333333333333333333333333333333333",
  "source_state_digest_blake3": "7777777777777777777777777777777777777777777777777777777777777777",
  "normalized_provider_digest_blake3": "4444444444444444444444444444444444444444444444444444444444444444",
  "output_digest_blake3": "8888888888888888888888888888888888888888888888888888888888888888",
  "protected_exec_audit_digest_blake3": "9999999999999999999999999999999999999999999999999999999999999999",
  "final_bundle_digest_blake3": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "bounded_claim": "bounded intermediate provider",
  "stage_report_output_binding": "observed outputs and report-bound projections",
  "executable_authorization_binding": "declared authorizations plus protected audit",
  "non_claims": ["no final GCC", "no compiler correctness", "no self-build"],
  "provider_outputs": [
    {{"role":"target_prefixed_tools","artifact_ids":["tcc"],"digest_blake3":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}},
    {{"role":"headers","artifact_ids":["headers"],"digest_blake3":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}},
    {{"role":"libraries","artifact_ids":["libraries"],"digest_blake3":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"}},
    {{"role":"provider_metadata","artifact_ids":["provider.json"],"digest_blake3":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"}}
  ],
  "stage_reports": [{{
    "stage_id":"stage",
    "stage_plan_digest_blake3":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
    "predecessor_reports":[],
    "status":"complete",
    "executable_event_ids":["exec:stage"],
    "output_observations":[{{"artifact_id":"output","digest_blake3":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"}}]
  }}],
  "fallback_events": {fallback_events}
}}"#
            ),
        )
        .unwrap();
        let mut value = serde_json::from_slice::<serde_json::Value>(&fs::read(&path).unwrap()).unwrap();
        let digest =
            crate::stagex_provider::compute_lineage_receipt_payload_digest(&serde_json::to_vec(&value).unwrap())
                .unwrap();
        value["receipt_payload_digest_blake3"] = serde_json::Value::String(digest);
        fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    }

    fn write_self_build_provider_kind_linkage(
        root: &Path,
        proof_identity: &str,
        proof_linkage: &str,
        prerequisites: &str,
    ) {
        let path = root.join(SELF_BUILD_PROVIDER_KIND_LINKAGE_RECEIPT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            format!(
                r#"{{
  "schema": "mantle-self-build-provider-kind-linkage-v1",
  "derivation": "bootstrap/crunch.ncl",
  "proof_identity": {{ "selected_provider_kind": "{proof_identity}" }},
  "proof_linkage": {{ "selected_provider_kind": "{proof_linkage}" }},
  "prerequisites": {{ "provider_kind": "{prerequisites}" }}
}}"#
            ),
        )
        .unwrap();
    }

    fn write_real_self_build_proof_parity(root: &Path, extra_mutation: &str) {
        let path = root.join(REAL_SELF_BUILD_PROOF_PARITY_RECEIPT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut content = r#"{
  "schema": "mantle-real-self-build-proof-parity-evidence-v1",
  "release_id": "test-release",
  "selected_provider_kind": "source-root",
  "source_blake3": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "vendor_blake3": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "provider_kind_linkage": {
    "receipt_path": "bootstrap/evidence/crunch-self-build-provider-kind-linkage.json",
    "selected_provider_kind": "source-root"
  },
  "deterministic_proof": {
    "workflow": "mantle-deterministic-proof-receipt-v2",
    "verdict": "self-rebuild-match",
    "genuine_rebuild_authority": true,
    "rebuild_descriptor_blake3": "4444444444444444444444444444444444444444444444444444444444444444",
    "rebuild_authority_plan_blake3": "5555555555555555555555555555555555555555555555555555555555555555",
    "selected_provider_kind": "source-root",
    "digest_blake3": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    "source_blake3": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "vendor_blake3": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "clean_rebuild_roots": ["/tmp/proof-a", "/tmp/proof-b"],
    "artifact_digests_blake3": ["dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"]
  },
  "sandbox_evidence": {
    "profile_identity": "mantle-proof-sandbox-v1:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
    "digest_blake3": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
  },
  "verify_receipt": {
    "digest_blake3": "1111111111111111111111111111111111111111111111111111111111111111",
    "deterministic_release_status": "eligible",
    "proof_digest_blake3": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    "sandbox_evidence_digest_blake3": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
  },
  "summary": {
    "release_id": "test-release",
    "selected_provider_kind": "source-root",
    "json_digest_blake3": "2222222222222222222222222222222222222222222222222222222222222222",
    "markdown_digest_blake3": "3333333333333333333333333333333333333333333333333333333333333333",
    "verdict": "self-rebuild-match",
    "verify_status": "eligible",
    "proof_digest_blake3": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    "sandbox_evidence_digest_blake3": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    "bounded_claim": "This artifact rebuilt twice from the exact content identities and policies under isolated roots and matched.",
    "non_claims": ["compiler or verifier soundness", "full bootstrap reproducibility"]
  }
}
"#
        .to_string();
        if !extra_mutation.is_empty() {
            let parts: Vec<&str> = extra_mutation.splitn(2, "=>").collect();
            assert_eq!(parts.len(), 2, "mutation must be old=>new");
            content = content.replace(parts[0], parts[1]);
        }
        fs::write(path, content).unwrap();
    }

    fn write_binutils_tcc_transcript(root: &Path, extra: &str) {
        let path = root.join(BINUTILS_TCC_TOOL_TRANSCRIPT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            format!(
                r#"{{
  "schema": "mantle-binutils-tcc-tool-smoke-v1",
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

    fn write_gcc40_native_boundary_receipt(root: &Path) {
        let path = root.join(GCC40_NATIVE_BOUNDARY_RECEIPT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            r#"{
  "schema": "mantle-gcc40-native-boundary-v1",
  "derivation": "bootstrap/gcc-4.0.ncl",
  "status": "boundary-only",
  "boundary": "native-gcc-make-to-pass1-bridge",
  "native_attempt": {
    "command_marker": "make -j1 -C \"$dir\"",
    "diagnostic_marker": "MANTLE: gcc-4.0 native cc1 build reached TinyCC/Mes source boundary; installing pass1 bridge"
  },
  "installed_bridge_markers": [
    "gcc (Mantle pass1 bridge) 4.0.4",
    "Crunch GCC 4.0 pass1 cc1 object boundary",
    "exec \"$TCC/bin/tcc\" -c -I\"$MUSL/include\" -o \"\\$outfile\" \"\\$input\""
  ],
  "promoted_native_slices": [
    {
      "id": "cc1-arithmetic-control-flow",
      "receipt": "bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json",
      "derivation_marker": "Mantle GCC 4.0 native cc1 arithmetic slice: no TinyCC delegation for bounded proof input."
    },
    {
      "id": "cc1-logical-control-flow",
      "receipt": "bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json",
      "derivation_marker": "Mantle GCC 4.0 native cc1 logical slice: no TinyCC delegation for bounded proof input."
    },
    {
      "id": "cc1-local-variable-assignment",
      "receipt": "bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json",
      "derivation_marker": "Mantle GCC 4.0 native cc1 local-vars slice: no TinyCC delegation for bounded proof input."
    },
    {
      "id": "cc1-function-call",
      "receipt": "bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json",
      "derivation_marker": "Mantle GCC 4.0 native cc1 function-call slice: no TinyCC delegation for bounded proof input."
    },
    {
      "id": "cc1-array-index",
      "receipt": "bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json",
      "derivation_marker": "Mantle GCC 4.0 native cc1 array-index slice: no TinyCC delegation for bounded proof input."
    },
    {
      "id": "cc1-struct-field",
      "receipt": "bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json",
      "derivation_marker": "Mantle GCC 4.0 native cc1 struct-field slice: no TinyCC delegation for bounded proof input."
    },
    {
      "id": "cc1-pointer-deref",
      "receipt": "bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json",
      "derivation_marker": "Mantle GCC 4.0 native cc1 pointer-deref slice: no TinyCC delegation for bounded proof input."
    },
    {
      "id": "genattrtab-bounded-output",
      "receipt": "bootstrap/evidence/gcc-4.0-native-generator-slice.json",
      "derivation_marker": "Mantle GCC 4.0 native genattrtab bounded output slice: checked generated attrtab shape; full generator correctness pending."
    },
    {
      "id": "genoutput-bounded-output",
      "receipt": "bootstrap/evidence/gcc-4.0-native-generator-slice.json",
      "derivation_marker": "Mantle GCC 4.0 native genoutput bounded output slice: checked generated output shape; full generator correctness pending."
    },
    {
      "id": "genemit-bounded-output",
      "receipt": "bootstrap/evidence/gcc-4.0-native-generator-slice.json",
      "derivation_marker": "Mantle GCC 4.0 native genemit bounded output slice: checked generated emit source shape; full generator correctness pending."
    },
    {
      "id": "genrecog-bounded-output",
      "receipt": "bootstrap/evidence/gcc-4.0-native-generator-slice.json",
      "derivation_marker": "Mantle GCC 4.0 native genrecog bounded output slice: checked generated recognition source shape; full generator correctness pending."
    },
    {
      "id": "genextract-bounded-output",
      "receipt": "bootstrap/evidence/gcc-4.0-native-generator-slice.json",
      "derivation_marker": "Mantle GCC 4.0 native genextract bounded output slice: checked generated extraction source shape; full generator correctness pending."
    },
    {
      "id": "libiberty-demangle-single-short-arg",
      "receipt": "bootstrap/evidence/gcc-4.0-native-demangle-slice.json",
      "derivation_marker": "gcc40_cplus_demangle_short_arg_itanium_v6_boundary"
    }
  ],
  "last_observed_build_log": {
    "status": "complete-with-pass1-bridge-plus-bounded-native-cc1-pointer-deref-slice",
    "boundary_log_markers": ["gcc-4.0.4 build complete (languages: c)"]
  },
  "native_frontier": {
    "status": "frontier-only",
    "blockers": [
      {
        "id": "cc1-bounded-pointer-deref",
        "derivation_marker": "Mantle GCC 4.0 native cc1 pointer-deref slice: no TinyCC delegation for bounded proof input.",
        "frontier": "installed cc1 has checked bounded arithmetic, logical/control-flow, local-variable, helper-call, array-index, struct-field, and pointer-deref no-TinyCC-delegation slices; full native cc1 parsing/codegen correctness remains pending"
      },
      {
        "id": "libiberty-demangle-bounded-semantics",
        "derivation_marker": "gcc40_cplus_demangle_short_arg_itanium_v6_boundary",
        "frontier": "libiberty demangling has checked bounded flat, two-component nested, selected three-component nested zero-argument, selected single-int, selected single-char, selected single-long, and selected single-short Itanium semantic slices; full native cp-demangle remains pending"
      },
      {
        "id": "generator-bounded-outputs",
        "derivation_marker": "Mantle GCC 4.0 native genoutput bounded output slice: checked generated output shape; full generator correctness pending.",
        "frontier": "native generator evidence has checked bounded genattrtab, genoutput, genemit, genrecog, and genextract output slices; full native generator correctness remains pending"
      }
    ]
  },
  "parity_effect": "evidence-backed partial; does not prove native gcc.4.0 correctness"
}
"#,
        )
        .unwrap();
    }

    fn write_gcc40_native_cc1_arithmetic_receipt(root: &Path, mutation: &str) {
        let path = root.join(GCC40_NATIVE_CC1_ARITHMETIC_RECEIPT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let arithmetic_input =
            "int mantle_gcc40_arith_slice(int x) {\n  int y = x * 2 + 5;\n  return y > 7 ? y - 3 : y + 3;\n}\n";
        let arithmetic_transcript = "cc1 bounded native arithmetic slice\ncommand: $out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/gcc40-native-cc1-arith.c -dumpbase gcc40-native-cc1-arith.c -auxbase gcc40-native-cc1-arith -o /tmp/gcc40-native-cc1-arith.o\nexit_status: 0\nstdout: <empty>\nstderr: <empty>\nobject_marker: MANTLE-GCC40-NATIVE-CC1-ARITHMETIC-SLICE-V1\nno_tinycc_delegation: true\n";
        let arithmetic_output = b"MANTLE-GCC40-NATIVE-CC1-ARITHMETIC-SLICE-V1\nfunction=mantle_gcc40_arith_slice\nsemantics=integer-arithmetic-comparison-branch-return\n";
        let logical_input = "int mantle_gcc40_logic_slice(int x, int y) {\n  if ((x > 0 && y > 0) || x == y) {\n    return x + y;\n  }\n  return x - y;\n}\n";
        let logical_transcript = "cc1 bounded native logical slice\ncommand: $out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/gcc40-native-cc1-logic.c -dumpbase gcc40-native-cc1-logic.c -auxbase gcc40-native-cc1-logic -o /tmp/gcc40-native-cc1-logic.o\nexit_status: 0\nstdout: <empty>\nstderr: <empty>\nobject_marker: MANTLE-GCC40-NATIVE-CC1-LOGICAL-SLICE-V2\nno_tinycc_delegation: true\n";
        let logical_output = b"MANTLE-GCC40-NATIVE-CC1-LOGICAL-SLICE-V2\nfunction=mantle_gcc40_logic_slice\nsemantics=comparison-logical-and-or-branch-return\n";
        let local_input =
            "int mantle_gcc40_local_vars_slice(int x) {\n  int y = x + 1;\n  y = y * 3;\n  return y;\n}\n";
        let local_transcript = "cc1 bounded native local-vars slice\ncommand: $out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/gcc40-native-cc1-local-vars.c -dumpbase gcc40-native-cc1-local-vars.c -auxbase gcc40-native-cc1-local-vars -o /tmp/gcc40-native-cc1-local-vars.o\nexit_status: 0\nstdout: <empty>\nstderr: <empty>\nobject_marker: MANTLE-GCC40-NATIVE-CC1-LOCAL-VARS-SLICE-V3\nno_tinycc_delegation: true\n";
        let local_output = b"MANTLE-GCC40-NATIVE-CC1-LOCAL-VARS-SLICE-V3\nfunction=mantle_gcc40_local_vars_slice\nsemantics=local-int-declaration-assignment-update-return\n";
        let function_input = "int mantle_gcc40_helper(int x) {\n  return x * 2;\n}\n\nint mantle_gcc40_function_call_slice(int x) {\n  return mantle_gcc40_helper(x) + 1;\n}\n";
        let function_transcript = "cc1 bounded native function-call slice\ncommand: $out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/gcc40-native-cc1-function-call.c -dumpbase gcc40-native-cc1-function-call.c -auxbase gcc40-native-cc1-function-call -o /tmp/gcc40-native-cc1-function-call.o\nexit_status: 0\nstdout: <empty>\nstderr: <empty>\nobject_marker: MANTLE-GCC40-NATIVE-CC1-FUNCTION-CALL-SLICE-V4\nno_tinycc_delegation: true\n";
        let function_output = b"MANTLE-GCC40-NATIVE-CC1-FUNCTION-CALL-SLICE-V4\nfunction=mantle_gcc40_function_call_slice\nsemantics=helper-function-call-argument-return\n";
        let array_input = "int mantle_gcc40_array_slice(int x) {\n  int values[2];\n  values[0] = x;\n  values[1] = x + 2;\n  return values[0] + values[1];\n}\n";
        let array_transcript = "cc1 bounded native array-index slice\ncommand: $out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/gcc40-native-cc1-array-index.c -dumpbase gcc40-native-cc1-array-index.c -auxbase gcc40-native-cc1-array-index -o /tmp/gcc40-native-cc1-array-index.o\nexit_status: 0\nstdout: <empty>\nstderr: <empty>\nobject_marker: MANTLE-GCC40-NATIVE-CC1-ARRAY-INDEX-SLICE-V5\nno_tinycc_delegation: true\n";
        let array_output = b"MANTLE-GCC40-NATIVE-CC1-ARRAY-INDEX-SLICE-V5\nfunction=mantle_gcc40_array_slice\nsemantics=local-array-index-store-load-return\n";
        let struct_input = "struct mantle_gcc40_pair {\n  int left;\n  int right;\n};\n\nint mantle_gcc40_struct_slice(int x) {\n  struct mantle_gcc40_pair pair;\n  pair.left = x;\n  pair.right = x + 3;\n  return pair.left + pair.right;\n}\n";
        let struct_transcript = "cc1 bounded native struct-field slice\ncommand: $out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/gcc40-native-cc1-struct-field.c -dumpbase gcc40-native-cc1-struct-field.c -auxbase gcc40-native-cc1-struct-field -o /tmp/gcc40-native-cc1-struct-field.o\nexit_status: 0\nstdout: <empty>\nstderr: <empty>\nobject_marker: MANTLE-GCC40-NATIVE-CC1-STRUCT-FIELD-SLICE-V6\nno_tinycc_delegation: true\n";
        let struct_output = b"MANTLE-GCC40-NATIVE-CC1-STRUCT-FIELD-SLICE-V6\nfunction=mantle_gcc40_struct_slice\nsemantics=local-struct-field-store-load-return\n";
        let pointer_input = "int mantle_gcc40_pointer_slice(int x) {\n  int value = x;\n  int *slot = &value;\n  *slot = x + 4;\n  return *slot + value;\n}\n";
        let pointer_transcript = "cc1 bounded native pointer-deref slice\ncommand: $out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/gcc40-native-cc1-pointer-deref.c -dumpbase gcc40-native-cc1-pointer-deref.c -auxbase gcc40-native-cc1-pointer-deref -o /tmp/gcc40-native-cc1-pointer-deref.o\nexit_status: 0\nstdout: <empty>\nstderr: <empty>\nobject_marker: MANTLE-GCC40-NATIVE-CC1-POINTER-DEREF-SLICE-V7\nno_tinycc_delegation: true\n";
        let pointer_output = b"MANTLE-GCC40-NATIVE-CC1-POINTER-DEREF-SLICE-V7\nfunction=mantle_gcc40_pointer_slice\nsemantics=local-pointer-address-deref-store-load-return\n";
        let mut content = format!(
            r#"{{
  "schema": "mantle-gcc40-native-cc1-arithmetic-v7",
  "derivation": "bootstrap/gcc-4.0.ncl",
  "status": "bounded-native-slice",
  "selected_slice": "pointer-deref-v7",
  "smoke": {{
    "input_program": {},
    "command": "$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/gcc40-native-cc1-pointer-deref.c -dumpbase gcc40-native-cc1-pointer-deref.c -auxbase gcc40-native-cc1-pointer-deref -o /tmp/gcc40-native-cc1-pointer-deref.o",
    "transcript": {},
    "transcript_digest_blake3": "{}",
    "output_digest_blake3": "{}"
  }},
  "regressions": [{{
    "slice": "arithmetic-control-flow-v1",
    "input_program": {},
    "command": "$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/gcc40-native-cc1-arith.c -dumpbase gcc40-native-cc1-arith.c -auxbase gcc40-native-cc1-arith -o /tmp/gcc40-native-cc1-arith.o",
    "transcript": {},
    "transcript_digest_blake3": "{}",
    "output_digest_blake3": "{}"
  }}, {{
    "slice": "logical-boolean-control-flow-v2",
    "input_program": {},
    "command": "$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/gcc40-native-cc1-logic.c -dumpbase gcc40-native-cc1-logic.c -auxbase gcc40-native-cc1-logic -o /tmp/gcc40-native-cc1-logic.o",
    "transcript": {},
    "transcript_digest_blake3": "{}",
    "output_digest_blake3": "{}"
  }}, {{
    "slice": "local-variable-assignment-v3",
    "input_program": {},
    "command": "$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/gcc40-native-cc1-local-vars.c -dumpbase gcc40-native-cc1-local-vars.c -auxbase gcc40-native-cc1-local-vars -o /tmp/gcc40-native-cc1-local-vars.o",
    "transcript": {},
    "transcript_digest_blake3": "{}",
    "output_digest_blake3": "{}"
  }}, {{
    "slice": "function-call-v4",
    "input_program": {},
    "command": "$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/gcc40-native-cc1-function-call.c -dumpbase gcc40-native-cc1-function-call.c -auxbase gcc40-native-cc1-function-call -o /tmp/gcc40-native-cc1-function-call.o",
    "transcript": {},
    "transcript_digest_blake3": "{}",
    "output_digest_blake3": "{}"
  }}, {{
    "slice": "array-index-v5",
    "input_program": {},
    "command": "$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/gcc40-native-cc1-array-index.c -dumpbase gcc40-native-cc1-array-index.c -auxbase gcc40-native-cc1-array-index -o /tmp/gcc40-native-cc1-array-index.o",
    "transcript": {},
    "transcript_digest_blake3": "{}",
    "output_digest_blake3": "{}"
  }}, {{
    "slice": "struct-field-v6",
    "input_program": {},
    "command": "$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/gcc40-native-cc1-struct-field.c -dumpbase gcc40-native-cc1-struct-field.c -auxbase gcc40-native-cc1-struct-field -o /tmp/gcc40-native-cc1-struct-field.o",
    "transcript": {},
    "transcript_digest_blake3": "{}",
    "output_digest_blake3": "{}"
  }}],
  "no_tinycc_delegation": {{
    "derivation_marker": "Mantle GCC 4.0 native cc1 pointer-deref slice: no TinyCC delegation for bounded proof input.",
    "regression_markers": [
      "Mantle GCC 4.0 native cc1 arithmetic slice: no TinyCC delegation for bounded proof input.",
      "Mantle GCC 4.0 native cc1 logical slice: no TinyCC delegation for bounded proof input.",
      "Mantle GCC 4.0 native cc1 local-vars slice: no TinyCC delegation for bounded proof input.",
      "Mantle GCC 4.0 native cc1 function-call slice: no TinyCC delegation for bounded proof input.",
      "Mantle GCC 4.0 native cc1 array-index slice: no TinyCC delegation for bounded proof input.",
      "Mantle GCC 4.0 native cc1 struct-field slice: no TinyCC delegation for bounded proof input."
    ],
    "forbidden_markers": ["$TCC/bin/tcc", "TinyCC handoff", "exec \\\"$TCC/bin/tcc\\\""]
  }},
  "parity_effect": "evidence-backed partial; does not prove native/full GCC 4.0 correctness"
}}
"#,
            serde_json::to_string(pointer_input).unwrap(),
            serde_json::to_string(pointer_transcript).unwrap(),
            blake3::hash(pointer_transcript.as_bytes()).to_hex(),
            blake3::hash(pointer_output).to_hex(),
            serde_json::to_string(arithmetic_input).unwrap(),
            serde_json::to_string(arithmetic_transcript).unwrap(),
            blake3::hash(arithmetic_transcript.as_bytes()).to_hex(),
            blake3::hash(arithmetic_output).to_hex(),
            serde_json::to_string(logical_input).unwrap(),
            serde_json::to_string(logical_transcript).unwrap(),
            blake3::hash(logical_transcript.as_bytes()).to_hex(),
            blake3::hash(logical_output).to_hex(),
            serde_json::to_string(local_input).unwrap(),
            serde_json::to_string(local_transcript).unwrap(),
            blake3::hash(local_transcript.as_bytes()).to_hex(),
            blake3::hash(local_output).to_hex(),
            serde_json::to_string(function_input).unwrap(),
            serde_json::to_string(function_transcript).unwrap(),
            blake3::hash(function_transcript.as_bytes()).to_hex(),
            blake3::hash(function_output).to_hex(),
            serde_json::to_string(array_input).unwrap(),
            serde_json::to_string(array_transcript).unwrap(),
            blake3::hash(array_transcript.as_bytes()).to_hex(),
            blake3::hash(array_output).to_hex(),
            serde_json::to_string(struct_input).unwrap(),
            serde_json::to_string(struct_transcript).unwrap(),
            blake3::hash(struct_transcript.as_bytes()).to_hex(),
            blake3::hash(struct_output).to_hex()
        );
        if !mutation.is_empty() {
            let parts: Vec<&str> = mutation.splitn(2, "=>").collect();
            assert_eq!(parts.len(), 2, "mutation must be old=>new");
            content = content.replace(parts[0], parts[1]);
        }
        fs::write(path, content).unwrap();
    }

    fn write_gcc40_native_generator_receipt(root: &Path, mutation: &str) {
        let path = root.join(GCC40_NATIVE_GENERATOR_RECEIPT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let genattrtab_fragment = "#define HAVE_ATTR_enabled 0\nint gcc40_genattrtab_bounded_output_slice(void) { return HAVE_ATTR_enabled; }\n/* Mantle GCC 4.0 native genattrtab bounded output slice: checked generated attrtab shape; full generator correctness pending. */\n";
        let genattrtab_transcript = "genattrtab bounded native generator slice\nselected_generator: genattrtab\noutput_contract: bounded attrtab source exposing HAVE_ATTR_enabled and gcc40_genattrtab_bounded_output_slice\nexit_status: 0\nstdout_fragment: int gcc40_genattrtab_bounded_output_slice(void) { return HAVE_ATTR_enabled; }\nstderr: <empty>\nnon_claim: full native GCC 4.0 generator correctness pending\n";
        let genoutput_fragment = "#define GCC40_GENOUTPUT_BOUNDED 1\nint gcc40_genoutput_bounded_output_slice(void) { return GCC40_GENOUTPUT_BOUNDED; }\n/* Mantle GCC 4.0 native genoutput bounded output slice: checked generated output shape; full generator correctness pending. */\n";
        let genoutput_transcript = "genoutput bounded native generator slice\nselected_generator: genoutput\noutput_contract: bounded output source exposing GCC40_GENOUTPUT_BOUNDED and gcc40_genoutput_bounded_output_slice\nexit_status: 0\nstdout_fragment: int gcc40_genoutput_bounded_output_slice(void) { return GCC40_GENOUTPUT_BOUNDED; }\nstderr: <empty>\nnon_claim: full native GCC 4.0 generator correctness pending\n";
        let genemit_fragment = "#define GCC40_GENEMIT_BOUNDED 1\nvoid gcc40_genemit_bounded_output_slice(void) { }\n/* Mantle GCC 4.0 native genemit bounded output slice: checked generated emit source shape; full generator correctness pending. */\n";
        let genemit_transcript = "genemit bounded native generator slice\nselected_generator: genemit\noutput_contract: bounded emit source exposing GCC40_GENEMIT_BOUNDED and gcc40_genemit_bounded_output_slice\nexit_status: 0\nstdout_fragment: void gcc40_genemit_bounded_output_slice(void) { }\nstderr: <empty>\nnon_claim: full native GCC 4.0 generator correctness pending\n";
        let genrecog_fragment = "#define GCC40_GENRECOG_BOUNDED 1\nint gcc40_genrecog_bounded_output_slice(void) { return GCC40_GENRECOG_BOUNDED; }\n/* Mantle GCC 4.0 native genrecog bounded output slice: checked generated recognition source shape; full generator correctness pending. */\n";
        let genrecog_transcript = "genrecog bounded native generator slice\nselected_generator: genrecog\noutput_contract: bounded recognition source exposing GCC40_GENRECOG_BOUNDED and gcc40_genrecog_bounded_output_slice\nexit_status: 0\nstdout_fragment: int gcc40_genrecog_bounded_output_slice(void) { return GCC40_GENRECOG_BOUNDED; }\nstderr: <empty>\nnon_claim: full native GCC 4.0 generator correctness pending\n";
        let genextract_fragment = "#define GCC40_GENEXTRACT_BOUNDED 1\nint gcc40_genextract_bounded_output_slice(void) { return GCC40_GENEXTRACT_BOUNDED; }\n/* Mantle GCC 4.0 native genextract bounded output slice: checked generated extraction source shape; full generator correctness pending. */\n";
        let genextract_transcript = "genextract bounded native generator slice\nselected_generator: genextract\noutput_contract: bounded extraction source exposing GCC40_GENEXTRACT_BOUNDED and gcc40_genextract_bounded_output_slice\nexit_status: 0\nstdout_fragment: int gcc40_genextract_bounded_output_slice(void) { return GCC40_GENEXTRACT_BOUNDED; }\nstderr: <empty>\nnon_claim: full native GCC 4.0 generator correctness pending\n";
        let mut content = format!(
            r#"{{
  "schema": "mantle-gcc40-native-generator-slice-v5",
  "derivation": "bootstrap/gcc-4.0.ncl",
  "status": "bounded-native-generator-slices",
  "selected_generators": ["genattrtab", "genoutput", "genemit", "genrecog", "genextract"],
  "bounded_outputs": {{
    "genattrtab": {{
      "derivation_marker": "Mantle GCC 4.0 native genattrtab bounded output slice: checked generated attrtab shape; full generator correctness pending.",
      "contract": "genattrtab bounded output must emit deterministic attrtab source with HAVE_ATTR_enabled and gcc40_genattrtab_bounded_output_slice while making no full-generator claim",
      "output_program_fragment": {},
      "transcript": {},
      "transcript_digest_blake3": "{}",
      "output_digest_blake3": "{}"
    }},
    "genoutput": {{
      "derivation_marker": "Mantle GCC 4.0 native genoutput bounded output slice: checked generated output shape; full generator correctness pending.",
      "contract": "genoutput bounded output must emit deterministic output source with GCC40_GENOUTPUT_BOUNDED and gcc40_genoutput_bounded_output_slice while making no full-generator claim",
      "output_program_fragment": {},
      "transcript": {},
      "transcript_digest_blake3": "{}",
      "output_digest_blake3": "{}"
    }},
    "genemit": {{
      "derivation_marker": "Mantle GCC 4.0 native genemit bounded output slice: checked generated emit source shape; full generator correctness pending.",
      "contract": "genemit bounded output must emit deterministic emit source with GCC40_GENEMIT_BOUNDED and gcc40_genemit_bounded_output_slice while making no full-generator claim",
      "output_program_fragment": {},
      "transcript": {},
      "transcript_digest_blake3": "{}",
      "output_digest_blake3": "{}"
    }},
    "genrecog": {{
      "derivation_marker": "Mantle GCC 4.0 native genrecog bounded output slice: checked generated recognition source shape; full generator correctness pending.",
      "contract": "genrecog bounded output must emit deterministic recognition source with GCC40_GENRECOG_BOUNDED and gcc40_genrecog_bounded_output_slice while making no full-generator claim",
      "output_program_fragment": {},
      "transcript": {},
      "transcript_digest_blake3": "{}",
      "output_digest_blake3": "{}"
    }},
    "genextract": {{
      "derivation_marker": "Mantle GCC 4.0 native genextract bounded output slice: checked generated extraction source shape; full generator correctness pending.",
      "contract": "genextract bounded output must emit deterministic extraction source with GCC40_GENEXTRACT_BOUNDED and gcc40_genextract_bounded_output_slice while making no full-generator claim",
      "output_program_fragment": {},
      "transcript": {},
      "transcript_digest_blake3": "{}",
      "output_digest_blake3": "{}"
    }}
  }},
  "forbidden_boundary_markers": [
    "Crunch GCC 4.0 empty-attrtab source boundary: native genattrtab promotion pending.",
    "gcc40_genattrtab_empty_attrtab_source_boundary",
    "Crunch GCC 4.0 empty-output source boundary: native genoutput promotion pending.",
    "gcc40_genoutput_empty_output_source_boundary",
    "Crunch GCC 4.0 empty-emit source boundary: native genemit promotion pending.",
    "gcc40_genemit_empty_emit_source_boundary",
    "Crunch GCC 4.0 empty-recognition source boundary: native genrecog promotion pending.",
    "gcc40_genrecog_empty_recognition_source_boundary",
    "Crunch GCC 4.0 empty-extraction source boundary: native genextract promotion pending.",
    "gcc40_genextract_empty_extraction_source_boundary"
  ],
  "parity_effect": "evidence-backed partial; does not prove native/full GCC 4.0 generator correctness"
}}
"#,
            serde_json::to_string(genattrtab_fragment).unwrap(),
            serde_json::to_string(genattrtab_transcript).unwrap(),
            blake3::hash(genattrtab_transcript.as_bytes()).to_hex(),
            blake3::hash(genattrtab_fragment.as_bytes()).to_hex(),
            serde_json::to_string(genoutput_fragment).unwrap(),
            serde_json::to_string(genoutput_transcript).unwrap(),
            blake3::hash(genoutput_transcript.as_bytes()).to_hex(),
            blake3::hash(genoutput_fragment.as_bytes()).to_hex(),
            serde_json::to_string(genemit_fragment).unwrap(),
            serde_json::to_string(genemit_transcript).unwrap(),
            blake3::hash(genemit_transcript.as_bytes()).to_hex(),
            blake3::hash(genemit_fragment.as_bytes()).to_hex(),
            serde_json::to_string(genrecog_fragment).unwrap(),
            serde_json::to_string(genrecog_transcript).unwrap(),
            blake3::hash(genrecog_transcript.as_bytes()).to_hex(),
            blake3::hash(genrecog_fragment.as_bytes()).to_hex(),
            serde_json::to_string(genextract_fragment).unwrap(),
            serde_json::to_string(genextract_transcript).unwrap(),
            blake3::hash(genextract_transcript.as_bytes()).to_hex(),
            blake3::hash(genextract_fragment.as_bytes()).to_hex()
        );
        if !mutation.is_empty() {
            let parts: Vec<&str> = mutation.splitn(2, "=>").collect();
            assert_eq!(parts.len(), 2, "mutation must be old=>new");
            content = content.replace(parts[0], parts[1]);
        }
        fs::write(path, content).unwrap();
    }

    fn write_gcc40_native_demangle_receipt(root: &Path, mutation: &str) {
        let path = root.join(GCC40_NATIVE_DEMANGLE_RECEIPT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let transcript = "demangle bounded native libiberty slice
selected_shape: single-short-arg-itanium-v6
input: _ZN3foo3bar3bazEs
output: foo::bar::baz(short)
short_regression: _ZN3foo3barEs -> foo::bar(short)
flat_short_regression: _Z3foos -> foo(short)
long_regression: _ZN3foo3bar3bazEl -> foo::bar::baz(long)
nested_long_regression: _ZN3foo3barEl -> foo::bar(long)
flat_long_regression: _Z3fool -> foo(long)
char_regression: _ZN3foo3bar3bazEc -> foo::bar::baz(char)
nested_char_regression: _ZN3foo3barEc -> foo::bar(char)
flat_char_regression: _Z3fooc -> foo(char)
int_regression: _ZN3foo3bar3bazEi -> foo::bar::baz(int)
nested_int_regression: _ZN3foo3barEi -> foo::bar(int)
flat_int_regression: _Z3fooi -> foo(int)
zero_arg_regression: _ZN3foo3bar3bazEv -> foo::bar::baz()
nested_zero_arg_regression: _ZN3foo3barEv -> foo::bar()
flat_zero_arg_regression: _Z3foov -> foo()
negative: _ZN3foo3bar3bazEf -> <null>
negative: _ZN3foo3bar3bazEx -> <null>
negative: _ZN3foo3bar3baz3quxEs -> <null>
negative: _ZN3foo3bar3baz3quxEl -> <null>
negative: _ZN3foo3bar3baz3quxEc -> <null>
negative: _ZN3foo3bar3baz3quxEi -> <null>
negative: _ZN3foo3bar3baz3quxEv -> <null>
negative: _Z3fooss -> <null>
negative: _ZN3foo3bar3bazEss -> <null>
exit_status: 0
stdout: <empty>
stderr: <empty>
non_claim: full native cp-demangle and GCC 4.0 correctness pending
";
        let mut content = format!(
            r#"{{
  "schema": "mantle-gcc40-native-demangle-slice-v6",
  "derivation": "bootstrap/gcc-4.0.ncl",
  "status": "bounded-native-demangle-slice",
  "selected_shape": "single-short-arg-itanium-v6",
  "bounded_contract": {{
    "accepted_inputs": [{{ "mangled": "_ZN3foo3bar3bazEs", "demangled": "foo::bar::baz(short)" }}],
    "short_regressions": [{{ "mangled": "_ZN3foo3barEs", "demangled": "foo::bar(short)" }}],
    "flat_short_regressions": [{{ "mangled": "_Z3foos", "demangled": "foo(short)" }}],
    "long_regressions": [{{ "mangled": "_ZN3foo3bar3bazEl", "demangled": "foo::bar::baz(long)" }}, {{ "mangled": "_ZN3foo3barEl", "demangled": "foo::bar(long)" }}],
    "flat_long_regressions": [{{ "mangled": "_Z3fool", "demangled": "foo(long)" }}],
    "char_regressions": [{{ "mangled": "_ZN3foo3bar3bazEc", "demangled": "foo::bar::baz(char)" }}, {{ "mangled": "_ZN3foo3barEc", "demangled": "foo::bar(char)" }}],
    "flat_char_regressions": [{{ "mangled": "_Z3fooc", "demangled": "foo(char)" }}],
    "int_regressions": [{{ "mangled": "_ZN3foo3bar3bazEi", "demangled": "foo::bar::baz(int)" }}, {{ "mangled": "_ZN3foo3barEi", "demangled": "foo::bar(int)" }}],
    "flat_int_regressions": [{{ "mangled": "_Z3fooi", "demangled": "foo(int)" }}],
    "zero_arg_regressions": [{{ "mangled": "_ZN3foo3bar3bazEv", "demangled": "foo::bar::baz()" }}],
    "nested_regressions": [{{ "mangled": "_ZN3foo3barEv", "demangled": "foo::bar()" }}],
    "flat_regressions": [{{ "mangled": "_Z3foov", "demangled": "foo()" }}],
    "rejected_inputs": ["_ZN3foo3bar3bazEf", "_ZN3foo3bar3bazEx", "_ZN3foo3bar3baz3quxEs", "_ZN3foo3bar3baz3quxEl", "_ZN3foo3bar3baz3quxEc", "_ZN3foo3bar3baz3quxEi", "_ZN3foo3bar3baz3quxEv", "_Z3fooss", "_ZN3foo3bar3bazEss", "_ZN3fooE", "not_mangled"],
    "non_claim": "only flat, two-component nested, and selected three-component nested zero-argument, single-int, single-char, single-long, or single-short Itanium function names are in scope"
  }},
  "source_markers": {{
    "cplus_demangle_marker": "gcc40_cplus_demangle_short_arg_itanium_v6_boundary",
    "cp_demangle_marker": "gcc40_cp_demangle_short_arg_itanium_v6_boundary"
  }},
  "smoke": {{
    "transcript": {},
    "transcript_digest_blake3": "{}",
    "output_digest_blake3": "{}"
  }},
  "forbidden_stale_markers": [
    "gcc40_cplus_demangle_long_arg_itanium_v5_boundary",
    "gcc40_cp_demangle_long_arg_itanium_v5_boundary",
    "gcc40_cplus_demangle_char_arg_itanium_v4_boundary",
    "gcc40_cp_demangle_char_arg_itanium_v4_boundary",
    "gcc40_cplus_demangle_int_arg_itanium_v3_boundary",
    "gcc40_cp_demangle_int_arg_itanium_v3_boundary",
    "gcc40_cplus_demangle_deep_nested_itanium_v2_boundary",
    "gcc40_cp_demangle_deep_nested_itanium_v2_boundary",
    "gcc40_cplus_demangle_nested_itanium_v1_boundary",
    "gcc40_cp_demangle_nested_itanium_v1_boundary",
    "gcc40_cplus_demangle_bounded_itanium_v0_boundary",
    "gcc40_cp_demangle_bounded_itanium_v0_boundary",
    "libiberty_cp_demangle_bootstrap_stub"
  ],
  "parity_effect": "evidence-backed partial; does not prove native/full GCC 4.0 demangler correctness"
}}
"#,
            serde_json::to_string(transcript).unwrap(),
            blake3::hash(transcript.as_bytes()).to_hex(),
            blake3::hash(b"foo::bar::baz(short)\n").to_hex()
        );
        if !mutation.is_empty() {
            let parts: Vec<&str> = mutation.splitn(2, "=>").collect();
            assert_eq!(parts.len(), 2, "mutation must be old=>new");
            content = content.replace(parts[0], parts[1]);
        }
        fs::write(path, content).unwrap();
    }

    fn write_gcc40_native_cc1_build_frontier_receipt(root: &Path, mutation: &str) {
        let path = root.join(GCC40_NATIVE_CC1_BUILD_FRONTIER_RECEIPT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut content = include_str!("../bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json").to_string();
        if !mutation.is_empty() {
            let parts: Vec<&str> = mutation.splitn(2, "=>").collect();
            assert_eq!(parts.len(), 2, "mutation must be old=>new");
            content = content.replace(parts[0], parts[1]);
        }
        fs::write(path, content).unwrap();
        write_gcc40_cparse_diagnostic_derivation(root, "");
    }

    fn write_gcc40_cparse_diagnostic_derivation(root: &Path, mutation: &str) {
        let path = root.join(GCC40_CPARSE_DIAGNOSTIC_DERIVATION);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut content = concat!(
            "diag-cparse-frontier: compact_archived_matrix_v8\n",
            "diag-cparse-frontier: cparse_v19_adjusted_ssize_frontier=archived\n",
            "diag-cparse-frontier: cparse_v20_systypes_config_frontier=archived\n",
            "diag-cparse-frontier: cparse_v21_include_trace_frontier=archived\n",
            "diag-cparse-frontier: cparse_auto_host_ssize_patch=$auto_patch\n",
            "diag-cparse-frontier: cparse_auto_host_ssize_disabled=present\n",
            "m=/tmp/gcc40-cparse-make-auto-host-ssize.log\n",
            "diag-cparse-frontier: cparse_make_auto_host_ssize_cparse_o_rc=$rc\n",
            "diag-cparse-frontier: cparse_make_auto_host_ssize_cparse_o_compile_command=present\n",
            "diag-cparse-frontier: cparse_make_auto_host_ssize_cparse_o_object=absent\n",
            "diag-cparse-frontier: cparse_make_auto_host_ssize_include_flood_lines=$include_flood_lines\n",
            "diag-cparse-frontier: cparse_make_auto_host_ssize_include_flood_truncated_lines=$include_flood_truncated_lines\n",
            "diag-cparse-frontier: cparse_make_auto_host_ssize_cparse_o_tail\n",
            "exit \"$rc\"\n",
        )
        .to_string();
        if !mutation.is_empty() {
            let parts: Vec<&str> = mutation.splitn(2, "=>").collect();
            assert_eq!(parts.len(), 2, "mutation must be old=>new");
            content = content.replace(parts[0], parts[1]);
        }
        fs::write(path, content).unwrap();
    }

    fn write_gcc47_cxx_provider_contract(root: &Path, markers: &[&str]) {
        let path = root.join(GCC47_CXX_PROVIDER_CONTRACT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let rendered_markers = markers
            .iter()
            .map(|marker| format!("    {}", serde_json::to_string(marker).unwrap()))
            .collect::<Vec<_>>()
            .join(",\n");
        fs::write(
            path,
            format!(
                r#"{{
  "schema": "mantle-gcc47-cxx-provider-contract-v1",
  "derivation": "bootstrap/gcc-4.7.ncl",
  "status": "contract-only",
  "required_markers": [
{}
  ],
  "parity_effect": "evidence-backed partial; does not prove native/full GCC 4.7 correctness"
}}
"#,
                rendered_markers
            ),
        )
        .unwrap();
    }

    fn valid_gcc47_cxx_contract_markers() -> Vec<&'static str> {
        vec![
            "--enable-languages=c,c++",
            "CC=\"$GCC4/bin/gcc\"",
            "CXX=\"$GCC4/bin/g++\"",
            "make -j1 all-gcc",
            "make -j1 all-target-libgcc",
            "make -j1 install-gcc",
            "make -j1 install-target-libgcc",
            "ERROR: installed cc1plus missing",
            "ERROR: C++ smoke test failed",
            "ERROR: C++11 smoke test failed",
            "-std=c++11",
        ]
    }

    fn valid_gcc47_cxx_contract_content() -> String {
        valid_gcc47_cxx_contract_markers().join("\n")
    }

    fn write_full_musl_binutils_provider_contract(root: &Path, musl_markers: &[&str], binutils_markers: &[&str]) {
        let path = root.join(FULL_MUSL_BINUTILS_PROVIDER_CONTRACT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let rendered_musl_markers = musl_markers
            .iter()
            .map(|marker| format!("    {}", serde_json::to_string(marker).unwrap()))
            .collect::<Vec<_>>()
            .join(",\n");
        let rendered_binutils_markers = binutils_markers
            .iter()
            .map(|marker| format!("    {}", serde_json::to_string(marker).unwrap()))
            .collect::<Vec<_>>()
            .join(",\n");
        fs::write(
            path,
            format!(
                r#"{{
  "schema": "mantle-full-musl-binutils-provider-contract-v1",
  "musl_derivation": "bootstrap/musl-full.ncl",
  "binutils_derivation": "bootstrap/binutils-full.ncl",
  "status": "contract-only",
  "musl_required_markers": [
{}
  ],
  "binutils_required_markers": [
{}
  ],
  "parity_effect": "evidence-backed partial; does not prove full musl/binutils correctness"
}}
"#,
                rendered_musl_markers, rendered_binutils_markers
            ),
        )
        .unwrap();
    }

    fn valid_full_musl_contract_markers() -> Vec<&'static str> {
        vec![
            "./configure \\",
            "--host=x86_64-unknown-linux-musl",
            "CC=\"$GCC/bin/gcc\"",
            "make -j1 2>&1",
            "make -j1 install 2>&1",
            "ERROR: libc.a not built",
            "ERROR: stdio.h not installed",
            "ERROR: crt1.o not installed",
            "ERROR: dynamic linker symlink missing",
            "musl-1.2.5 build complete",
        ]
    }

    fn valid_full_binutils_contract_markers() -> Vec<&'static str> {
        vec![
            "binutils-2.41/configure",
            "--with-sysroot=\"$MUSL\"",
            "CC=\"$GCC/bin/gcc\"",
            "CXX=\"$GCC/bin/g++\"",
            "make -j1 MAKEINFO=true 2>&1",
            "make -j1 install MAKEINFO=true 2>&1",
            "ERROR: $tool not found",
            "binutils241-smoke.s",
            "\"$AS\" -o /tmp/binutils241-smoke.o",
            "\"$LD\" -o /tmp/binutils241-smoke-bin",
            "binutils-2.41 build complete",
        ]
    }

    fn write_gcc10_provider_contract(root: &Path, markers: &[&str]) {
        let path = root.join(GCC10_PROVIDER_CONTRACT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let rendered_markers = markers
            .iter()
            .map(|marker| format!("    {}", serde_json::to_string(marker).unwrap()))
            .collect::<Vec<_>>()
            .join(",\n");
        fs::write(
            path,
            format!(
                r#"{{
  "schema": "mantle-gcc10-provider-contract-v1",
  "derivation": "bootstrap/gcc-10.ncl",
  "status": "contract-only",
  "required_markers": [
{}
  ],
  "parity_effect": "evidence-backed partial; does not prove native/full GCC 10 correctness"
}}
"#,
                rendered_markers
            ),
        )
        .unwrap();
    }

    fn valid_gcc10_contract_markers() -> Vec<&'static str> {
        vec![
            "--enable-languages=c,c++",
            "CC=\"$GCC47/bin/gcc\"",
            "CXX=\"$GCC47/bin/g++\"",
            "make -j1 all-gcc MAKEINFO=true",
            "make -j1 all-target-libgcc MAKEINFO=true",
            "make -j1 install-gcc MAKEINFO=true",
            "make -j1 install-target-libgcc MAKEINFO=true",
            "test -x \"$out/bin/cc\"",
            "test -x \"$out/bin/c++\"",
            "find \"$out\" -name libgcc.a -type f",
            "ERROR: C smoke test failed",
            "ERROR: C++ smoke test failed",
            "gcc-10.5.0 build complete",
        ]
    }

    fn valid_gcc10_contract_content() -> String {
        valid_gcc10_contract_markers().join("\n")
    }

    fn write_gcc40_placeholder_inventory(root: &Path, content: &str) {
        let path = root.join(GCC40_PLACEHOLDER_INVENTORY);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let markers = collect_placeholder_marker_occurrences(content);
        let rendered = markers
            .iter()
            .map(|occurrence| {
                format!(
                    r#"{{ "line": {}, "marker": "{}", "classification": "test-boundary" }}"#,
                    occurrence.line, occurrence.marker
                )
            })
            .collect::<Vec<_>>()
            .join(",\n    ");
        fs::write(
            path,
            format!(
                r#"{{
  "schema": "mantle-gcc40-placeholder-inventory-v1",
  "derivation": "bootstrap/gcc-4.0.ncl",
  "status": "inventory-only",
  "marker_count": {},
  "markers": [
    {}
  ]
}}
"#,
                markers.len(),
                rendered
            ),
        )
        .unwrap();
    }

    fn valid_seed_full_contract() -> &'static str {
        r#"
        share/crunch-bootstrap/provider.json
        "provider_id": "full-source-v1"
        "target": "$PUBLIC_TARGET"
        "dynamic_linker": "$DYNAMIC_LINKER"
        "source_authority": "declared-mantle-derivation-closure"
        "output_identity"
        "runtime_admission"
        "closure_admission"
        "state_pinned_inputs": false
        "legacy_members": []
        "retained_tools": [
          "x86_64-linux-musl-gcc", "x86_64-linux-musl-as", "x86_64-linux-musl-ld"
        ]
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
    fn gcc40_placeholder_inventory_matching_receipt_reports_partial() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(dir.path(), "");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert!(row.status.blocks_parity());
        assert!(!row.notes.contains("evidence check failed"));
    }

    #[test]
    fn gcc40_placeholder_inventory_missing_receipt_stays_placeholder_with_failure() {
        let dir = tempdir().unwrap();
        write_stage(dir.path(), "gcc-4.0.ncl", "# pass1 bridge\n");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("evidence check failed"));
        assert!(row.notes.contains("GCC 4.0 placeholder inventory missing"));
    }

    #[test]
    fn gcc40_placeholder_inventory_drift_stays_placeholder_with_failure() {
        let dir = tempdir().unwrap();
        let original = "# pass1 bridge\n";
        write_stage(dir.path(), "gcc-4.0.ncl", original);
        write_gcc40_placeholder_inventory(dir.path(), original);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_stage(dir.path(), "gcc-4.0.ncl", "# pass1 bridge\necho stub\n");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("GCC 4.0 placeholder inventory drift"));
    }

    #[test]
    fn gcc40_native_boundary_missing_receipt_stays_placeholder_with_failure() {
        let dir = tempdir().unwrap();
        let content = concat!(
            "make -j1 -C \"$dir\"\n",
            "MANTLE: gcc-4.0 native cc1 build reached TinyCC/Mes source boundary; installing pass1 bridge\n",
            "gcc (Mantle pass1 bridge) 4.0.4\n",
            "Crunch GCC 4.0 pass1 cc1 object boundary\n",
            "exec \"$TCC/bin/tcc\" -c -I\"$MUSL/include\" -o \"\\$outfile\" \"\\$input\"\n",
        );
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("GCC 4.0 native boundary receipt missing"));
    }

    #[test]
    fn gcc40_native_boundary_marker_drift_fails_closed() {
        let dir = tempdir().unwrap();
        let content = concat!(
            "make -j1 -C \"$dir\"\n",
            "MANTLE: gcc-4.0 native cc1 build reached TinyCC/Mes source boundary; installing pass1 bridge\n",
            "gcc (Mantle pass1 bridge) 4.0.4\n",
            "Crunch GCC 4.0 pass1 cc1 object boundary\n",
        );
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(dir.path(), "");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("GCC 4.0 native boundary receipt marker not found"));
    }

    #[test]
    fn gcc40_native_frontier_marker_drift_fails_closed() {
        let dir = tempdir().unwrap();
        let content = concat!(
            "make -j1 -C \"$dir\"\n",
            "MANTLE: gcc-4.0 native cc1 build reached TinyCC/Mes source boundary; installing pass1 bridge\n",
            "gcc (Mantle pass1 bridge) 4.0.4\n",
            "Crunch GCC 4.0 pass1 cc1 object boundary\n",
            "exec \"$TCC/bin/tcc\" -c -I\"$MUSL/include\" -o \"\\$outfile\" \"\\$input\"\n",
            "gcc40_cplus_demangle_short_arg_itanium_v6_boundary\n",
            "gcc40_cp_demangle_short_arg_itanium_v6_boundary\n",
            "under the c-parse flags deterministically segfaults TinyCC after that\n",
            "rewrites before system.h trigger deterministic TinyCC segfaults.\n",
            "through TinyCC/Mes diagnostics and segfault; seed the same inert files\n",
            "make -j1 -C gcc gengtype-yacc.c CC=tcc AR=\"$BINUTILS/bin/ar\" RANLIB=\"$BINUTILS/bin/ranlib\" MAKEINFO=true 2>&1 || true\n",
            "the validated TinyCC handoff. This remains a bridge until native cc1 builds.\n",
        );
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(dir.path(), "");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("GCC 4.0 native boundary receipt marker not found"));
    }

    #[test]
    fn gcc40_native_frontier_missing_generator_blocker_fails_closed() {
        let dir = tempdir().unwrap();
        let content = concat!(
            "make -j1 -C \"$dir\"\n",
            "MANTLE: gcc-4.0 native cc1 build reached TinyCC/Mes source boundary; installing pass1 bridge\n",
            "gcc (Mantle pass1 bridge) 4.0.4\n",
            "Crunch GCC 4.0 pass1 cc1 object boundary\n",
            "exec \"$TCC/bin/tcc\" -c -I\"$MUSL/include\" -o \"\\$outfile\" \"\\$input\"\n",
            "Mantle GCC 4.0 native cc1 pointer-deref slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native genoutput bounded output slice: checked generated output shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genemit bounded output slice: checked generated emit source shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genrecog bounded output slice: checked generated recognition source shape; full generator correctness pending.\n",
            "gcc40_cplus_demangle_short_arg_itanium_v6_boundary\n",
            "gcc40_cp_demangle_short_arg_itanium_v6_boundary\n",
            "under the c-parse flags deterministically segfaults TinyCC after that\n",
            "rewrites before system.h trigger deterministic TinyCC segfaults.\n",
            "through TinyCC/Mes diagnostics and segfault; seed the same inert files\n",
            "make -j1 -C gcc gengtype-yacc.c CC=tcc AR=\"$BINUTILS/bin/ar\" RANLIB=\"$BINUTILS/bin/ranlib\" MAKEINFO=true 2>&1 || true\n",
            "the validated TinyCC handoff. This remains a bridge until native cc1 builds.\n",
        );
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        let receipt_path = dir.path().join(GCC40_NATIVE_BOUNDARY_RECEIPT);
        let receipt =
            fs::read_to_string(&receipt_path).unwrap().replace("generator-bounded-outputs", "generator-omitted");
        fs::write(receipt_path, receipt).unwrap();
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(dir.path(), "");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(
            row.notes.contains(
                "GCC 4.0 native boundary receipt native_frontier.blockers missing `generator-bounded-outputs`"
            )
        );
    }

    #[test]
    fn gcc40_native_cc1_arithmetic_receipt_digest_drift_fails_closed() {
        let dir = tempdir().unwrap();
        let content = concat!(
            "make -j1 -C \"$dir\"\n",
            "MANTLE: gcc-4.0 native cc1 build reached TinyCC/Mes source boundary; installing pass1 bridge\n",
            "gcc (Mantle pass1 bridge) 4.0.4\n",
            "Crunch GCC 4.0 pass1 cc1 object boundary\n",
            "exec \"$TCC/bin/tcc\" -c -I\"$MUSL/include\" -o \"\\$outfile\" \"\\$input\"\n",
            "Mantle GCC 4.0 native cc1 arithmetic slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 logical slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 local-vars slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 function-call slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 array-index slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 struct-field slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 pointer-deref slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native genattrtab bounded output slice: checked generated attrtab shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genoutput bounded output slice: checked generated output shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genemit bounded output slice: checked generated emit source shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genrecog bounded output slice: checked generated recognition source shape; full generator correctness pending.\n",
            "gcc40_cplus_demangle_short_arg_itanium_v6_boundary\n",
            "gcc40_cp_demangle_short_arg_itanium_v6_boundary\n",
            "under the c-parse flags deterministically segfaults TinyCC after that\n",
            "rewrites before system.h trigger deterministic TinyCC segfaults.\n",
            "through TinyCC/Mes diagnostics and segfault; seed the same inert files\n",
            "make -j1 -C gcc gengtype-yacc.c CC=tcc AR=\"$BINUTILS/bin/ar\" RANLIB=\"$BINUTILS/bin/ranlib\" MAKEINFO=true 2>&1 || true\n",
            "the validated TinyCC handoff. This remains a bridge until native cc1 builds.\n",
        );
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "b110bc20=>00000000");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(dir.path(), "");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("GCC 4.0 native cc1 arithmetic receipt transcript digest"));
    }

    #[test]
    fn gcc40_native_cc1_arithmetic_receipt_missing_logical_regression_fails_closed() {
        let dir = tempdir().unwrap();
        let content = concat!(
            "make -j1 -C \"$dir\"\n",
            "MANTLE: gcc-4.0 native cc1 build reached TinyCC/Mes source boundary; installing pass1 bridge\n",
            "gcc (Mantle pass1 bridge) 4.0.4\n",
            "Crunch GCC 4.0 pass1 cc1 object boundary\n",
            "exec \"$TCC/bin/tcc\" -c -I\"$MUSL/include\" -o \"\\$outfile\" \"\\$input\"\n",
            "Mantle GCC 4.0 native cc1 arithmetic slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 logical slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 local-vars slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 function-call slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 array-index slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 struct-field slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 pointer-deref slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native genattrtab bounded output slice: checked generated attrtab shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genoutput bounded output slice: checked generated output shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genemit bounded output slice: checked generated emit source shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genrecog bounded output slice: checked generated recognition source shape; full generator correctness pending.\n",
            "gcc40_cplus_demangle_short_arg_itanium_v6_boundary\n",
            "gcc40_cp_demangle_short_arg_itanium_v6_boundary\n",
            "under the c-parse flags deterministically segfaults TinyCC after that\n",
            "rewrites before system.h trigger deterministic TinyCC segfaults.\n",
            "through TinyCC/Mes diagnostics and segfault; seed the same inert files\n",
            "make -j1 -C gcc gengtype-yacc.c CC=tcc AR=\"$BINUTILS/bin/ar\" RANLIB=\"$BINUTILS/bin/ranlib\" MAKEINFO=true 2>&1 || true\n",
            "the validated TinyCC handoff. This remains a bridge until native cc1 builds.\n",
        );
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "logical-boolean-control-flow-v2=>missing-logical-v2");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(dir.path(), "");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("missing logical-boolean-control-flow-v2 regression"));
    }

    #[test]
    fn gcc40_native_cc1_arithmetic_receipt_missing_local_vars_regression_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "local-variable-assignment-v3=>missing-local-vars-v3");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(dir.path(), "");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("missing local-variable-assignment-v3 regression"));
    }

    #[test]
    fn gcc40_native_cc1_arithmetic_receipt_missing_function_call_regression_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "function-call-v4=>missing-function-call-v4");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(dir.path(), "");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("missing function-call-v4 regression"));
    }

    #[test]
    fn gcc40_native_cc1_arithmetic_receipt_stale_schema_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(
            dir.path(),
            "mantle-gcc40-native-cc1-arithmetic-v7=>mantle-gcc40-native-cc1-arithmetic-v5",
        );
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(dir.path(), "");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("expected `mantle-gcc40-native-cc1-arithmetic-v7`"));
    }

    #[test]
    fn gcc40_native_cc1_arithmetic_receipt_missing_pointer_deref_marker_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content().replace(
            "Mantle GCC 4.0 native cc1 pointer-deref slice: no TinyCC delegation for bounded proof input.\n",
            "",
        );
        write_stage(dir.path(), "gcc-4.0.ncl", &content);
        write_gcc40_placeholder_inventory(dir.path(), &content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(dir.path(), "");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("GCC 4.0 native boundary receipt marker not found"));
    }

    #[test]
    fn gcc40_native_cc1_arithmetic_receipt_forbidden_delegation_fails_closed() {
        let dir = tempdir().unwrap();
        let content = concat!(
            "make -j1 -C \"$dir\"\n",
            "MANTLE: gcc-4.0 native cc1 build reached TinyCC/Mes source boundary; installing pass1 bridge\n",
            "gcc (Mantle pass1 bridge) 4.0.4\n",
            "Crunch GCC 4.0 pass1 cc1 object boundary\n",
            "exec \"$TCC/bin/tcc\" -c -I\"$MUSL/include\" -o \"\\$outfile\" \"\\$input\"\n",
            "Mantle GCC 4.0 native cc1 arithmetic slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 logical slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 local-vars slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 function-call slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 array-index slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 struct-field slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 pointer-deref slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native genattrtab bounded output slice: checked generated attrtab shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genoutput bounded output slice: checked generated output shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genemit bounded output slice: checked generated emit source shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genrecog bounded output slice: checked generated recognition source shape; full generator correctness pending.\n",
            "gcc40_cplus_demangle_short_arg_itanium_v6_boundary\n",
            "gcc40_cp_demangle_short_arg_itanium_v6_boundary\n",
            "under the c-parse flags deterministically segfaults TinyCC after that\n",
            "rewrites before system.h trigger deterministic TinyCC segfaults.\n",
            "through TinyCC/Mes diagnostics and segfault; seed the same inert files\n",
            "make -j1 -C gcc gengtype-yacc.c CC=tcc AR=\"$BINUTILS/bin/ar\" RANLIB=\"$BINUTILS/bin/ranlib\" MAKEINFO=true 2>&1 || true\n",
            "the validated TinyCC handoff. This remains a bridge until native cc1 builds.\n",
        );
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "\"$TCC/bin/tcc\"=>\"cc1 bounded\"");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(dir.path(), "");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("forbidden TinyCC delegation marker"));
    }

    #[test]
    fn gcc40_native_generator_receipt_missing_fails_closed() {
        let dir = tempdir().unwrap();
        let content = concat!(
            "make -j1 -C \"$dir\"\n",
            "MANTLE: gcc-4.0 native cc1 build reached TinyCC/Mes source boundary; installing pass1 bridge\n",
            "gcc (Mantle pass1 bridge) 4.0.4\n",
            "Crunch GCC 4.0 pass1 cc1 object boundary\n",
            "exec \"$TCC/bin/tcc\" -c -I\"$MUSL/include\" -o \"\\$outfile\" \"\\$input\"\n",
            "Mantle GCC 4.0 native cc1 arithmetic slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 logical slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 local-vars slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 function-call slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 array-index slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 struct-field slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 pointer-deref slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native genattrtab bounded output slice: checked generated attrtab shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genoutput bounded output slice: checked generated output shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genemit bounded output slice: checked generated emit source shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genrecog bounded output slice: checked generated recognition source shape; full generator correctness pending.\n",
            "gcc40_cplus_demangle_short_arg_itanium_v6_boundary\n",
            "gcc40_cp_demangle_short_arg_itanium_v6_boundary\n",
            "under the c-parse flags deterministically segfaults TinyCC after that\n",
            "rewrites before system.h trigger deterministic TinyCC segfaults.\n",
            "through TinyCC/Mes diagnostics and segfault; seed the same inert files\n",
            "make -j1 -C gcc gengtype-yacc.c CC=tcc AR=\"$BINUTILS/bin/ar\" RANLIB=\"$BINUTILS/bin/ranlib\" MAKEINFO=true 2>&1 || true\n",
            "the validated TinyCC handoff. This remains a bridge until native cc1 builds.\n",
        );
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("GCC 4.0 native generator receipt missing"));
    }

    #[test]
    fn gcc40_native_generator_receipt_digest_drift_fails_closed() {
        let dir = tempdir().unwrap();
        let content = concat!(
            "make -j1 -C \"$dir\"\n",
            "MANTLE: gcc-4.0 native cc1 build reached TinyCC/Mes source boundary; installing pass1 bridge\n",
            "gcc (Mantle pass1 bridge) 4.0.4\n",
            "Crunch GCC 4.0 pass1 cc1 object boundary\n",
            "exec \"$TCC/bin/tcc\" -c -I\"$MUSL/include\" -o \"\\$outfile\" \"\\$input\"\n",
            "Mantle GCC 4.0 native cc1 arithmetic slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 logical slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 local-vars slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 function-call slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 array-index slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 struct-field slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 pointer-deref slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native genattrtab bounded output slice: checked generated attrtab shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genoutput bounded output slice: checked generated output shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genemit bounded output slice: checked generated emit source shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genrecog bounded output slice: checked generated recognition source shape; full generator correctness pending.\n",
            "gcc40_cplus_demangle_short_arg_itanium_v6_boundary\n",
            "gcc40_cp_demangle_short_arg_itanium_v6_boundary\n",
            "under the c-parse flags deterministically segfaults TinyCC after that\n",
            "rewrites before system.h trigger deterministic TinyCC segfaults.\n",
            "through TinyCC/Mes diagnostics and segfault; seed the same inert files\n",
            "make -j1 -C gcc gengtype-yacc.c CC=tcc AR=\"$BINUTILS/bin/ar\" RANLIB=\"$BINUTILS/bin/ranlib\" MAKEINFO=true 2>&1 || true\n",
            "the validated TinyCC handoff. This remains a bridge until native cc1 builds.\n",
        );
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "00b4ac17=>00000000");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("GCC 4.0 native generator receipt genattrtab transcript digest"));
    }

    #[test]
    fn gcc40_native_generator_receipt_unsupported_selected_generator_fails_closed() {
        let dir = tempdir().unwrap();
        let content = concat!(
            "make -j1 -C \"$dir\"\n",
            "MANTLE: gcc-4.0 native cc1 build reached TinyCC/Mes source boundary; installing pass1 bridge\n",
            "gcc (Mantle pass1 bridge) 4.0.4\n",
            "Crunch GCC 4.0 pass1 cc1 object boundary\n",
            "exec \"$TCC/bin/tcc\" -c -I\"$MUSL/include\" -o \"\\$outfile\" \"\\$input\"\n",
            "Mantle GCC 4.0 native cc1 arithmetic slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 logical slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 local-vars slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 function-call slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 array-index slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 struct-field slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 pointer-deref slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native genattrtab bounded output slice: checked generated attrtab shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genoutput bounded output slice: checked generated output shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genemit bounded output slice: checked generated emit source shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genrecog bounded output slice: checked generated recognition source shape; full generator correctness pending.\n",
            "gcc40_cplus_demangle_short_arg_itanium_v6_boundary\n",
            "gcc40_cp_demangle_short_arg_itanium_v6_boundary\n",
            "under the c-parse flags deterministically segfaults TinyCC after that\n",
            "rewrites before system.h trigger deterministic TinyCC segfaults.\n",
            "through TinyCC/Mes diagnostics and segfault; seed the same inert files\n",
            "make -j1 -C gcc gengtype-yacc.c CC=tcc AR=\"$BINUTILS/bin/ar\" RANLIB=\"$BINUTILS/bin/ranlib\" MAKEINFO=true 2>&1 || true\n",
            "the validated TinyCC handoff. This remains a bridge until native cc1 builds.\n",
        );
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(
            dir.path(),
            "\"genattrtab\", \"genoutput\", \"genemit\", \"genrecog\", \"genextract\"=>\"genattrtab\", \"genoutput\", \"genemit\"",
        );

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("selected generators missing `genrecog`"));
    }

    #[test]
    fn gcc40_native_generator_receipt_unsupported_schema_fails_closed() {
        let dir = tempdir().unwrap();
        let content = concat!(
            "make -j1 -C \"$dir\"\n",
            "MANTLE: gcc-4.0 native cc1 build reached TinyCC/Mes source boundary; installing pass1 bridge\n",
            "gcc (Mantle pass1 bridge) 4.0.4\n",
            "Crunch GCC 4.0 pass1 cc1 object boundary\n",
            "exec \"$TCC/bin/tcc\" -c -I\"$MUSL/include\" -o \"\\$outfile\" \"\\$input\"\n",
            "Mantle GCC 4.0 native cc1 arithmetic slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 logical slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 local-vars slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 function-call slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 array-index slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 struct-field slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 pointer-deref slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native genattrtab bounded output slice: checked generated attrtab shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genoutput bounded output slice: checked generated output shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genemit bounded output slice: checked generated emit source shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genrecog bounded output slice: checked generated recognition source shape; full generator correctness pending.\n",
            "gcc40_cplus_demangle_short_arg_itanium_v6_boundary\n",
            "gcc40_cp_demangle_short_arg_itanium_v6_boundary\n",
            "under the c-parse flags deterministically segfaults TinyCC after that\n",
            "rewrites before system.h trigger deterministic TinyCC segfaults.\n",
            "through TinyCC/Mes diagnostics and segfault; seed the same inert files\n",
            "make -j1 -C gcc gengtype-yacc.c CC=tcc AR=\"$BINUTILS/bin/ar\" RANLIB=\"$BINUTILS/bin/ranlib\" MAKEINFO=true 2>&1 || true\n",
            "the validated TinyCC handoff. This remains a bridge until native cc1 builds.\n",
        );
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(
            dir.path(),
            "mantle-gcc40-native-generator-slice-v5=>mantle-gcc40-native-generator-slice-v0",
        );

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("expected `mantle-gcc40-native-generator-slice-v5`"));
    }

    fn valid_gcc40_native_demangle_content() -> &'static str {
        concat!(
            "make -j1 -C \"$dir\"\n",
            "make -j1 -C gcc gengtype-yacc.c CC=tcc AR=\"$BINUTILS/bin/ar\" RANLIB=\"$BINUTILS/bin/ranlib\" MAKEINFO=true 2>&1 || true\n",
            "MANTLE: gcc-4.0 native cc1 build reached TinyCC/Mes source boundary; installing pass1 bridge\n",
            "the validated TinyCC handoff. This remains a bridge until native cc1 builds.\n",
            "gcc (Mantle pass1 bridge) 4.0.4\n",
            "Crunch GCC 4.0 pass1 cc1 object boundary\n",
            "exec \"$TCC/bin/tcc\" -c -I\"$MUSL/include\" -o \"\\$outfile\" \"\\$input\"\n",
            "Mantle GCC 4.0 native cc1 arithmetic slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 logical slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 local-vars slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 function-call slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 array-index slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 struct-field slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native cc1 pointer-deref slice: no TinyCC delegation for bounded proof input.\n",
            "Mantle GCC 4.0 native genattrtab bounded output slice: checked generated attrtab shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genoutput bounded output slice: checked generated output shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genemit bounded output slice: checked generated emit source shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genrecog bounded output slice: checked generated recognition source shape; full generator correctness pending.\n",
            "Mantle GCC 4.0 native genextract bounded output slice: checked generated extraction source shape; full generator correctness pending.\n",
            "gcc40_cplus_demangle_short_arg_itanium_v6_boundary\n",
            "gcc40_cp_demangle_short_arg_itanium_v6_boundary\n",
            "under the c-parse flags deterministically segfaults TinyCC after that\n",
            "rewrites before system.h trigger deterministic TinyCC segfaults.\n",
            "through TinyCC/Mes diagnostics and segfault; seed the same inert files\n",
            "MANTLE-GCC40-NATIVE-CC1-SOURCE-FRONTIER-REDUCTION-V1: bounded gengtype-yacc probe records unchanged TinyCC/Mes c-parse boundary after generated-header seeds.\n",
        )
    }

    #[test]
    fn gcc40_native_demangle_receipt_missing_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("GCC 4.0 native demangle receipt missing"));
    }

    #[test]
    fn gcc40_native_demangle_receipt_digest_drift_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "929b317c=>00000000");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("GCC 4.0 native demangle receipt transcript digest"));
    }

    #[test]
    fn gcc40_native_demangle_receipt_unsupported_shape_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "single-short-arg-itanium-v6=>operator-name-itanium-v1");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("expected `single-short-arg-itanium-v6`"));
    }

    #[test]
    fn gcc40_native_demangle_receipt_unsupported_schema_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(
            dir.path(),
            "mantle-gcc40-native-demangle-slice-v6=>mantle-gcc40-native-demangle-slice-v0",
        );

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("expected `mantle-gcc40-native-demangle-slice-v6`"));
    }

    #[test]
    fn gcc40_native_demangle_receipt_full_parity_overclaim_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(
            dir.path(),
            "evidence-backed partial; does not prove native/full GCC 4.0 demangler correctness=>full native GCC 4.0 demangler correctness proven",
        );

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("does not prove native/full GCC 4.0 demangler correctness"));
    }

    #[test]
    fn gcc40_native_cc1_build_frontier_receipt_missing_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("GCC 4.0 native cc1 build-frontier receipt missing"));
    }

    #[test]
    fn gcc40_native_cc1_build_frontier_marker_drift_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(
            dir.path(),
            "under the c-parse flags deterministically segfaults TinyCC after that=>missing native source frontier marker",
        );

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("evidence check failed"));
    }

    #[test]
    fn gcc40_native_cc1_build_frontier_missing_fields_fail_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(
            dir.path(),
            "\"source_frontier_markers\"=>\"stale_source_frontier_markers\"",
        );

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("evidence check failed"));
    }

    #[test]
    fn gcc40_native_cc1_source_frontier_probe_marker_drift_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(
            dir.path(),
            "MANTLE-GCC40-NATIVE-CC1-SOURCE-FRONTIER-REDUCTION-V1: bounded gengtype-yacc probe records unchanged TinyCC/Mes c-parse boundary after generated-header seeds.=>missing source frontier reduction marker",
        );

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("evidence check failed"));
    }

    #[test]
    fn gcc40_native_cc1_source_frontier_schema_drift_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(
            dir.path(),
            "mantle-gcc40-native-cc1-source-frontier-reduction-v22=>mantle-gcc40-native-cc1-source-frontier-reduction-v10",
        );

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("source-frontier reduction schema"), "{}", row.notes);
        assert!(row.notes.contains("evidence check failed"));
    }

    #[test]
    fn gcc40_native_cc1_source_frontier_observed_result_drift_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(dir.path(), "narrowed-stable-blocker=>native-cc1-complete");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("source-frontier reduction observed_result"), "{}", row.notes);
        assert!(row.notes.contains("evidence check failed"));
    }

    #[test]
    fn gcc40_native_cc1_source_frontier_stale_fdopen_frontier_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(
            dir.path(),
            "cparse_make_auto_host_ssize_cparse_o_rc=2=>cparse_make_auto_host_ssize_cparse_o_rc=1",
        );

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("observed_frontier missing required v22 fragment"), "{}", row.notes);
        assert!(row.notes.contains("evidence check failed"));
    }

    #[test]
    fn gcc40_native_cc1_source_frontier_diagnostic_marker_drift_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(dir.path(), "");
        write_gcc40_cparse_diagnostic_derivation(
            dir.path(),
            "diag-cparse-frontier: compact_archived_matrix_v8=>diag-cparse-frontier: compact_archived_matrix_missing",
        );

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("source-frontier diagnostic marker"), "{}", row.notes);
        assert!(row.notes.contains("evidence check failed"));
    }

    #[test]
    fn gcc40_native_cc1_build_frontier_parity_overclaim_fails_closed() {
        let dir = tempdir().unwrap();
        let content = valid_gcc40_native_demangle_content();
        write_stage(dir.path(), "gcc-4.0.ncl", content);
        write_gcc40_placeholder_inventory(dir.path(), content);
        write_gcc40_native_boundary_receipt(dir.path());
        write_gcc40_native_cc1_arithmetic_receipt(dir.path(), "");
        write_gcc40_native_generator_receipt(dir.path(), "");
        write_gcc40_native_demangle_receipt(dir.path(), "");
        write_gcc40_native_cc1_build_frontier_receipt(
            dir.path(),
            "evidence-backed partial; does not prove native/full GCC 4.0 correctness=>full native GCC 4.0 correctness proven",
        );

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("evidence check failed"));
    }

    #[test]
    fn gcc47_cxx_provider_contract_matching_receipt_reports_partial() {
        let dir = tempdir().unwrap();
        let markers = valid_gcc47_cxx_contract_markers();
        write_stage(dir.path(), "gcc-4.7.ncl", &valid_gcc47_cxx_contract_content());
        write_gcc47_cxx_provider_contract(dir.path(), &markers);

        let row = evaluate_stage(dir.path(), &gcc47_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert!(row.status.blocks_parity());
        assert!(!row.notes.contains("evidence check failed"));
    }

    #[test]
    fn gcc47_cxx_provider_contract_missing_receipt_reports_partial_with_failure() {
        let dir = tempdir().unwrap();
        write_stage(dir.path(), "gcc-4.7.ncl", &valid_gcc47_cxx_contract_content());

        let row = evaluate_stage(dir.path(), &gcc47_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert!(row.notes.contains("evidence check failed"));
        assert!(row.notes.contains("GCC 4.7 C++ provider contract missing"));
    }

    #[test]
    fn gcc47_cxx_provider_contract_marker_drift_fails_closed() {
        let dir = tempdir().unwrap();
        let markers = valid_gcc47_cxx_contract_markers();
        write_stage(dir.path(), "gcc-4.7.ncl", "--enable-languages=c,c++\n");
        write_gcc47_cxx_provider_contract(dir.path(), &markers);

        let row = evaluate_stage(dir.path(), &gcc47_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert!(row.notes.contains("GCC 4.7 C++ provider contract marker not found"));
        assert!(row.notes.contains("CC=\"$GCC4/bin/gcc\""));
    }

    #[test]
    fn full_musl_binutils_provider_contract_matching_receipt_reports_partial() {
        let dir = tempdir().unwrap();
        let musl_markers = valid_full_musl_contract_markers();
        let binutils_markers = valid_full_binutils_contract_markers();
        write_stage(dir.path(), "musl-full.ncl", &musl_markers.join("\n"));
        write_stage(dir.path(), "binutils-full.ncl", &binutils_markers.join("\n"));
        write_full_musl_binutils_provider_contract(dir.path(), &musl_markers, &binutils_markers);

        let row = evaluate_stage(dir.path(), &full_musl_binutils_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert!(row.status.blocks_parity());
        assert!(!row.notes.contains("evidence check failed"));
    }

    #[test]
    fn full_musl_binutils_provider_contract_missing_receipt_reports_partial_with_failure() {
        let dir = tempdir().unwrap();
        write_stage(dir.path(), "musl-full.ncl", &valid_full_musl_contract_markers().join("\n"));
        write_stage(dir.path(), "binutils-full.ncl", &valid_full_binutils_contract_markers().join("\n"));

        let row = evaluate_stage(dir.path(), &full_musl_binutils_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert!(row.notes.contains("evidence check failed"));
        assert!(row.notes.contains("full musl/binutils provider contract missing"));
    }

    #[test]
    fn full_musl_binutils_provider_contract_musl_marker_drift_fails_closed() {
        let dir = tempdir().unwrap();
        let musl_markers = valid_full_musl_contract_markers();
        let binutils_markers = valid_full_binutils_contract_markers();
        write_stage(dir.path(), "musl-full.ncl", "--host=x86_64-unknown-linux-musl\n");
        write_stage(dir.path(), "binutils-full.ncl", &binutils_markers.join("\n"));
        write_full_musl_binutils_provider_contract(dir.path(), &musl_markers, &binutils_markers);

        let row = evaluate_stage(dir.path(), &full_musl_binutils_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert!(row.notes.contains("full musl/binutils provider contract musl marker not found"));
        assert!(row.notes.contains("bootstrap/musl-full.ncl"));
    }

    #[test]
    fn full_musl_binutils_provider_contract_binutils_marker_drift_fails_closed() {
        let dir = tempdir().unwrap();
        let musl_markers = valid_full_musl_contract_markers();
        let binutils_markers = valid_full_binutils_contract_markers();
        write_stage(dir.path(), "musl-full.ncl", &musl_markers.join("\n"));
        write_stage(dir.path(), "binutils-full.ncl", "binutils-2.41/configure\n");
        write_full_musl_binutils_provider_contract(dir.path(), &musl_markers, &binutils_markers);

        let row = evaluate_stage(dir.path(), &full_musl_binutils_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert!(row.notes.contains("full musl/binutils provider contract binutils marker not found"));
        assert!(row.notes.contains("bootstrap/binutils-full.ncl"));
    }

    #[test]
    fn full_musl_binutils_real_derivations_report_provider_contract_backed_partial() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));

        let row = evaluate_stage(root, &full_musl_binutils_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert!(row.status.blocks_parity());
        assert!(row.notes.contains("checked receipt"));
        assert!(!row.notes.contains("evidence check failed"));
    }

    #[test]
    fn gcc10_provider_contract_matching_receipt_reports_partial() {
        let dir = tempdir().unwrap();
        let markers = valid_gcc10_contract_markers();
        write_stage(dir.path(), "gcc-10.ncl", &valid_gcc10_contract_content());
        write_gcc10_provider_contract(dir.path(), &markers);

        let row = evaluate_stage(dir.path(), &gcc10_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert!(row.status.blocks_parity());
        assert!(!row.notes.contains("evidence check failed"));
    }

    #[test]
    fn gcc10_provider_contract_missing_receipt_reports_partial_with_failure() {
        let dir = tempdir().unwrap();
        write_stage(dir.path(), "gcc-10.ncl", &valid_gcc10_contract_content());

        let row = evaluate_stage(dir.path(), &gcc10_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert!(row.notes.contains("evidence check failed"));
        assert!(row.notes.contains("GCC 10 provider contract missing"));
    }

    #[test]
    fn gcc10_provider_contract_marker_drift_fails_closed() {
        let dir = tempdir().unwrap();
        let markers = valid_gcc10_contract_markers();
        write_stage(dir.path(), "gcc-10.ncl", "--enable-languages=c,c++\n");
        write_gcc10_provider_contract(dir.path(), &markers);

        let row = evaluate_stage(dir.path(), &gcc10_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert!(row.notes.contains("GCC 10 provider contract marker not found"));
        assert!(row.notes.contains("CC=\"$GCC47/bin/gcc\""));
    }

    #[test]
    fn gcc10_real_derivation_reports_provider_contract_backed_partial() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));

        let row = evaluate_stage(root, &gcc10_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert!(row.status.blocks_parity());
        assert!(row.notes.contains("checked receipt"));
        assert!(!row.notes.contains("evidence check failed"));
    }

    #[test]
    fn gcc47_real_derivation_reports_cxx_contract_backed_partial() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));

        let row = evaluate_stage(root, &gcc47_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert!(row.status.blocks_parity());
        assert!(row.notes.contains("checked receipt"));
        assert!(!row.notes.contains("evidence check failed"));
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
                proof_details: None,
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
                proof_details: None,
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
    fn binutils_tcc_real_derivation_reports_independently_receipted_completion() {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let report = collect_bootstrap_parity_report(project_root);
        let row = report.rows.iter().find(|row| row.id == "binutils.tcc").unwrap();

        assert_eq!(row.status, StageStatus::Complete, "{}", row.notes);
        assert_eq!(row.provider_kind, ProviderKind::SourceRoot);
        assert!(!row.status.blocks_parity());
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
              "schema": "mantle-binutils-tcc-tool-smoke-v1",
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
    fn self_build_without_provider_kind_linkage_receipt_is_blocked() {
        let dir = tempdir().unwrap();
        write_stage(dir.path(), "crunch.ncl", "# real crunch derivation body\n");

        let row = evaluate_stage(dir.path(), &self_build_spec());

        assert_eq!(row.status, StageStatus::Blocked);
        assert_eq!(row.provider_kind, ProviderKind::Unknown);
        assert!(row.proof_details.is_none());
        assert!(row.notes.contains("provider-kind linkage receipt missing"));
        assert!(row.notes.contains(SELF_BUILD_PROVIDER_KIND_LINKAGE_RECEIPT));
    }

    #[test]
    fn self_build_valid_real_proof_descriptor_is_evidence_backed_partial() {
        let dir = tempdir().unwrap();
        write_stage(dir.path(), "crunch.ncl", "# real crunch derivation body\n");
        write_self_build_provider_kind_linkage(dir.path(), "source-root", "source-root", "source-root");
        write_real_self_build_proof_parity(dir.path(), "");

        let row = evaluate_stage(dir.path(), &self_build_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert_eq!(row.provider_kind, ProviderKind::SourceRoot);
        assert!(row.status.blocks_parity());
        let proof = row.proof_details.as_ref().unwrap();
        assert_eq!(proof.release_id, "test-release");
        assert_eq!(proof.selected_provider_kind, ProviderKind::SourceRoot);
        assert_eq!(proof.verdict, "self-rebuild-match");
        assert_eq!(proof.verify_status, "eligible");
        assert!(proof.bounded_claim.contains("rebuilt twice"));
        assert!(row.notes.contains("genuine release rebuild evidence accepted as partial bootstrap-parity evidence"));
        assert!(!row.notes.contains("evidence check failed"));
    }

    #[test]
    fn self_build_provider_kind_linkage_rejects_mismatched_kind() {
        let dir = tempdir().unwrap();
        write_self_build_provider_kind_linkage(dir.path(), "source-root", "stagex-lineage", "source-root");

        let err = validate_self_build_provider_kind_linkage(dir.path()).unwrap_err();

        assert!(err.contains("provider-kind linkage mismatch"));
        assert!(err.contains("proof_linkage.selected_provider_kind=stagex-lineage"));
    }

    #[test]
    fn self_build_provider_kind_linkage_rejects_unknown_kind() {
        let dir = tempdir().unwrap();
        write_self_build_provider_kind_linkage(dir.path(), "source-root", "source-root", "mystery-provider");

        let err = validate_self_build_provider_kind_linkage(dir.path()).unwrap_err();

        assert!(err.contains("unknown provider kind `mystery-provider`"));
        assert!(err.contains("prerequisites.provider_kind"));
    }

    #[test]
    fn self_build_real_proof_descriptor_rejects_provider_mismatch() {
        let dir = tempdir().unwrap();
        write_self_build_provider_kind_linkage(dir.path(), "source-root", "source-root", "source-root");
        write_real_self_build_proof_parity(
            dir.path(),
            "\"provider_kind_linkage\": {\n    \"receipt_path\": \"bootstrap/evidence/crunch-self-build-provider-kind-linkage.json\",\n    \"selected_provider_kind\": \"source-root\"=>\"provider_kind_linkage\": {\n    \"receipt_path\": \"bootstrap/evidence/crunch-self-build-provider-kind-linkage.json\",\n    \"selected_provider_kind\": \"legacy-fetch\"",
        );

        let err = validate_real_self_build_proof_parity_evidence(dir.path()).unwrap_err();

        assert!(err.contains("provider kind mismatch") || err.contains("provider-kind linkage mismatch"));
    }

    #[test]
    fn self_build_real_proof_descriptor_rejects_unsupported_workflow() {
        let dir = tempdir().unwrap();
        write_self_build_provider_kind_linkage(dir.path(), "source-root", "source-root", "source-root");
        write_real_self_build_proof_parity(
            dir.path(),
            "mantle-deterministic-proof-receipt-v2=>mantle-deterministic-proof-receipt-v1",
        );

        let err = validate_real_self_build_proof_parity_evidence(dir.path()).unwrap_err();

        assert!(err.contains("workflow"));
        assert!(err.contains("mantle-deterministic-proof-receipt-v2"));
    }

    #[test]
    fn self_build_real_proof_descriptor_rejects_missing_genuine_rebuild_authority() {
        let dir = tempdir().unwrap();
        write_self_build_provider_kind_linkage(dir.path(), "source-root", "source-root", "source-root");
        write_real_self_build_proof_parity(
            dir.path(),
            "\"genuine_rebuild_authority\": true=>\"genuine_rebuild_authority\": false",
        );

        let err = validate_real_self_build_proof_parity_evidence(dir.path()).unwrap_err();

        assert!(err.contains("genuine_rebuild_authority"));
        assert!(err.contains("expected `true`"));
    }

    #[test]
    fn self_build_real_proof_descriptor_rejects_missing_sandbox_evidence() {
        let dir = tempdir().unwrap();
        write_self_build_provider_kind_linkage(dir.path(), "source-root", "source-root", "source-root");
        write_real_self_build_proof_parity(dir.path(), "\"profile_identity\"=>\"missing_profile_identity\"");

        let err = validate_real_self_build_proof_parity_evidence(dir.path()).unwrap_err();

        assert!(err.contains("profile_identity"));
    }

    #[test]
    fn self_build_real_proof_descriptor_rejects_unsupported_sandbox_profile() {
        let dir = tempdir().unwrap();
        write_self_build_provider_kind_linkage(dir.path(), "source-root", "source-root", "source-root");
        write_real_self_build_proof_parity(dir.path(), "mantle-proof-sandbox-v1=>direct-host");

        let err = validate_real_self_build_proof_parity_evidence(dir.path()).unwrap_err();

        assert!(err.contains("unsupported"));
        assert!(err.contains("mantle-proof-sandbox-v1"));
    }

    #[test]
    fn self_build_real_proof_descriptor_rejects_reused_roots() {
        let dir = tempdir().unwrap();
        write_self_build_provider_kind_linkage(dir.path(), "source-root", "source-root", "source-root");
        write_real_self_build_proof_parity(dir.path(), "/tmp/proof-b=>/tmp/proof-a");

        let err = validate_real_self_build_proof_parity_evidence(dir.path()).unwrap_err();

        assert!(err.contains("distinct"));
    }

    #[test]
    fn self_build_real_proof_descriptor_rejects_ineligible_verify_receipt() {
        let dir = tempdir().unwrap();
        write_self_build_provider_kind_linkage(dir.path(), "source-root", "source-root", "source-root");
        write_real_self_build_proof_parity(dir.path(), "eligible=>ineligible");

        let err = validate_real_self_build_proof_parity_evidence(dir.path()).unwrap_err();

        assert!(err.contains("eligible"));
    }

    #[test]
    fn self_build_real_proof_descriptor_rejects_malformed_digest_linkage() {
        let dir = tempdir().unwrap();
        write_self_build_provider_kind_linkage(dir.path(), "source-root", "source-root", "source-root");
        write_real_self_build_proof_parity(
            dir.path(),
            "\"verify_receipt\": {\n    \"digest_blake3\": \"1111111111111111111111111111111111111111111111111111111111111111\",\n    \"deterministic_release_status\": \"eligible\",\n    \"proof_digest_blake3\": \"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc\"=>\"verify_receipt\": {\n    \"digest_blake3\": \"1111111111111111111111111111111111111111111111111111111111111111\",\n    \"deterministic_release_status\": \"eligible\",\n    \"proof_digest_blake3\": \"abababababababababababababababababababababababababababababababab\"",
        );

        let err = validate_real_self_build_proof_parity_evidence(dir.path()).unwrap_err();

        assert!(err.contains("proof_digest_blake3"));
    }

    #[test]
    fn self_build_checked_legacy_release_evidence_is_blocked_until_genuine_v2_refresh() {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let report = collect_bootstrap_parity_report(project_root);
        let row = report.rows.iter().find(|row| row.id == "crunch.self-build").unwrap();

        assert_eq!(row.status, StageStatus::Blocked);
        assert_eq!(row.provider_kind, ProviderKind::Unknown);
        assert!(row.status.blocks_parity());
        assert!(row.proof_details.is_none());
        assert!(!row.notes.contains("provider-kind linkage receipt missing"));
        assert!(row.notes.contains("mantle-deterministic-proof-receipt-v2"));
        assert!(report.axes.iter().any(|axis| axis.axis == ParityAxis::Guix && !axis.complete));
        assert!(report.axes.iter().any(|axis| axis.axis == ParityAxis::Stagex && !axis.complete));
    }

    #[test]
    fn stagex_lineage_without_receipt_remains_blocked() {
        let dir = tempdir().unwrap();

        let row = evaluate_stage(dir.path(), &stagex_lineage_spec());

        assert_eq!(row.status, StageStatus::Blocked);
        assert_eq!(row.provider_kind, ProviderKind::Unknown);
        assert!(row.notes.contains("StageX lineage provider receipt missing"));
        assert!(row.notes.contains(STAGEX_LINEAGE_PROVIDER_RECEIPT));
    }

    #[test]
    fn stagex_lineage_scaffold_receipt_is_rejected() {
        let dir = tempdir().unwrap();
        write_stagex_lineage_receipt(
            dir.path(),
            "stagex-lineage",
            "scaffold-only",
            "1111111111111111111111111111111111111111111111111111111111111111",
            "[]",
        );

        let row = evaluate_stage(dir.path(), &stagex_lineage_spec());

        assert_eq!(row.status, StageStatus::Blocked);
        assert_eq!(row.provider_kind, ProviderKind::Unknown);
        assert!(row.status.blocks_parity());
        assert!(row.notes.contains("lineage_receipt_status"));
    }

    #[test]
    fn stagex_lineage_complete_receipt_is_complete() {
        let dir = tempdir().unwrap();
        write_stagex_lineage_receipt(
            dir.path(),
            "stagex-lineage",
            "complete",
            "1111111111111111111111111111111111111111111111111111111111111111",
            "[]",
        );

        let row = evaluate_stage(dir.path(), &stagex_lineage_spec());

        assert_eq!(row.status, StageStatus::Complete);
        assert_eq!(row.provider_kind, ProviderKind::StagexLineage);
        assert!(!row.status.blocks_parity());
        assert!(!row.notes.contains("evidence check failed"));
    }

    #[test]
    fn stagex_lineage_receipt_rejects_source_root_provider() {
        let dir = tempdir().unwrap();
        write_stagex_lineage_receipt(
            dir.path(),
            "source-root",
            "complete",
            "1111111111111111111111111111111111111111111111111111111111111111",
            "[]",
        );

        let err = validate_stagex_lineage_provider_receipt(dir.path()).unwrap_err();

        assert!(err.contains("provider_kind"));
        assert!(err.contains("expected `stagex-lineage`"));
    }

    #[test]
    fn stagex_lineage_receipt_rejects_malformed_digest() {
        let dir = tempdir().unwrap();
        write_stagex_lineage_receipt(dir.path(), "stagex-lineage", "complete", "ABC", "[]");

        let err = validate_stagex_lineage_provider_receipt(dir.path()).unwrap_err();

        assert!(err.contains("audited_seed_digest"));
        assert!(err.contains("64-character lowercase hex digest"));
    }

    #[test]
    fn stagex_lineage_receipt_rejects_fallback_events() {
        let dir = tempdir().unwrap();
        write_stagex_lineage_receipt(
            dir.path(),
            "stagex-lineage",
            "complete",
            "1111111111111111111111111111111111111111111111111111111111111111",
            r#"["host-bwrap"]"#,
        );

        let err = validate_stagex_lineage_provider_receipt(dir.path()).unwrap_err();

        assert!(err.contains("fallback_events"));
        assert!(err.contains("must be empty"));
    }

    #[test]
    fn stagex_lineage_receipt_rejects_missing_non_claims() {
        let dir = tempdir().unwrap();
        write_stagex_lineage_receipt(
            dir.path(),
            "stagex-lineage",
            "complete",
            "1111111111111111111111111111111111111111111111111111111111111111",
            "[]",
        );
        let path = dir.path().join(STAGEX_LINEAGE_PROVIDER_RECEIPT);
        let mut value = serde_json::from_slice::<serde_json::Value>(&fs::read(&path).unwrap()).unwrap();
        value["non_claims"] = serde_json::json!([]);
        fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

        let err = validate_stagex_lineage_provider_receipt(dir.path()).unwrap_err();

        assert!(err.contains("non-claims"));
        assert!(err.contains("expected at least"));
    }

    #[test]
    fn stagex_lineage_receipt_rejects_stale_payload_digest() {
        let dir = tempdir().unwrap();
        write_stagex_lineage_receipt(
            dir.path(),
            "stagex-lineage",
            "complete",
            "1111111111111111111111111111111111111111111111111111111111111111",
            "[]",
        );
        let path = dir.path().join(STAGEX_LINEAGE_PROVIDER_RECEIPT);
        let mut value = serde_json::from_slice::<serde_json::Value>(&fs::read(&path).unwrap()).unwrap();
        value["bounded_claim"] = serde_json::Value::String("substituted claim".to_string());
        fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

        let err = validate_stagex_lineage_provider_receipt(dir.path()).unwrap_err();

        assert!(err.contains("payload digest mismatch"));
        assert!(err.contains("expected"));
    }

    #[test]
    fn gcc40_real_derivation_reports_independently_receipted_completion() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let report = collect_bootstrap_parity_report(root);
        let row = report.rows.iter().find(|row| row.id == "gcc.4.0").unwrap();

        assert_eq!(row.status, StageStatus::Complete, "{}", row.notes);
        assert_eq!(row.provider_kind, ProviderKind::SourceRoot);
        assert!(!row.status.blocks_parity());
        assert!(!row.notes.contains("evidence check failed"));
    }

    #[test]
    fn gcc40_canonical_root_selects_regenerated_cxx_stage() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("import \"gcc-4.0-musl-cxx.ncl\""));
        assert!(content.contains("independently validated"));
        assert!(!content.contains("exec \"$TCC/bin/tcc\""));
        assert!(!content.contains("bootstrap gencheck stub"));
    }

    #[test]
    fn gcc40_final_stage_has_bounded_positive_and_negative_matrix() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0-musl-cxx.ncl")).unwrap();

        assert!(content.contains("early-native-gcc40-row.txt"));
        assert!(content.contains("for generator in genattrtab genoutput genemit genrecog genextract gengtype"));
        assert!(content.contains("$WORK/cxx-runtime"));
        assert!(content.contains("$WORK/relocated-c-runtime"));
        assert!(content.contains("accepted malformed C"));
        assert!(content.contains("accepted malformed C++"));
        assert!(!content.contains("$STAGE0/bin:$PATH"));
        assert!(!content.contains("NON_ADMISSION"));
    }

    #[test]
    fn stagex_lineage_real_receipt_reports_evidence_backed_completion() {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let report = collect_bootstrap_parity_report(project_root);
        let row = report.rows.iter().find(|row| row.id == "seed-full.stagex-lineage").unwrap();

        assert_eq!(row.status, StageStatus::Complete, "{}", row.notes);
        assert_eq!(row.provider_kind, ProviderKind::StagexLineage);
        assert!(!row.status.blocks_parity());
        assert!(!row.notes.contains("evidence check failed"));
        let stagex = report.axes.iter().find(|axis| axis.axis == ParityAxis::Stagex).unwrap();
        assert!(!stagex.blocking_rows.contains(&"seed-full.stagex-lineage".to_string()));
        assert!(!stagex.blocking_rows.is_empty());
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
