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
    BinutilsTccToolTranscript,
    RealSelfBuildProof,
    StagexLineageProviderReceipt,
    Gcc40PlaceholderInventory,
    Gcc47CxxProviderContract,
    Gcc10ProviderContract,
    FullMuslBinutilsProviderContract,
}

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
    let evidence_result = validate_stage_evidence(project_root, path.as_deref(), spec.evidence_check);
    let evidence_failure = evidence_result.as_ref().err().map(String::as_str);
    let proof_details = evidence_result.as_ref().ok().and_then(|validation| validation.proof_details.clone());
    let status = match (spec.expected_complete, file_state) {
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
    };
    let provider_kind = provider_kind_for(spec, status, proof_details.as_ref());
    let notes = row_notes(spec, status, path.as_deref(), evidence_failure, proof_details.as_ref());
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
        proof_details,
        notes,
    }
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
            "real self-build proof evidence accepted for provider_kind={} release_id={} proof_digest={} verify_status={}; bounded claim: {}",
            details.selected_provider_kind,
            details.release_id,
            details.deterministic_proof_digest_blake3,
            details.verify_status,
            details.bounded_claim
        ));
    }
    if let Some(reason) = evidence_failure {
        notes.push(format!("evidence check failed: {reason}"));
    }
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
        EvidenceCheck::BinutilsTccToolTranscript => {
            validate_binutils_tcc_tool_transcript(project_root).map(|()| EvidenceValidation::empty())
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

    require_provider_contract_string(
        &receipt,
        "full musl/binutils provider contract",
        "schema",
        "mantle-full-musl-binutils-provider-contract-v1",
    )?;
    require_provider_contract_string(
        &receipt,
        "full musl/binutils provider contract",
        "musl_derivation",
        "bootstrap/musl-full.ncl",
    )?;
    require_provider_contract_string(
        &receipt,
        "full musl/binutils provider contract",
        "binutils_derivation",
        "bootstrap/binutils-full.ncl",
    )?;
    require_provider_contract_string(&receipt, "full musl/binutils provider contract", "status", "contract-only")?;
    require_provider_contract_string(
        &receipt,
        "full musl/binutils provider contract",
        "parity_effect",
        "evidence-backed partial; does not prove full musl/binutils correctness",
    )?;

    validate_contract_markers(
        &receipt,
        "musl_required_markers",
        "full musl/binutils provider contract musl",
        &project_root.join("bootstrap/musl-full.ncl"),
        "bootstrap/musl-full.ncl",
    )?;
    validate_contract_markers(
        &receipt,
        "binutils_required_markers",
        "full musl/binutils provider contract binutils",
        &project_root.join("bootstrap/binutils-full.ncl"),
        "bootstrap/binutils-full.ncl",
    )?;
    Ok(())
}

fn validate_contract_markers(
    receipt: &serde_json::Value,
    field: &str,
    label: &str,
    derivation_path: &Path,
    derivation_label: &str,
) -> Result<(), String> {
    let derivation_content = fs::read_to_string(derivation_path)
        .map_err(|err| format!("read {label} derivation {}: {err}", derivation_path.display()))?;
    let markers = receipt
        .get(field)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{label} missing array field `{field}`"))?;
    if markers.is_empty() {
        return Err(format!("{label} `{field}` must not be empty"));
    }
    for marker in markers {
        let marker = marker.as_str().ok_or_else(|| format!("{label} marker must be a string"))?;
        if marker.trim().is_empty() {
            return Err(format!("{label} marker must not be empty"));
        }
        if !derivation_content.contains(marker) {
            return Err(format!("{label} marker not found in {derivation_label}: `{marker}`"));
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
    require_gcc47_contract_string(&receipt, "schema", "mantle-gcc47-cxx-provider-contract-v1")?;
    require_gcc47_contract_string(&receipt, "derivation", "bootstrap/gcc-4.7.ncl")?;
    require_gcc47_contract_string(&receipt, "status", "contract-only")?;
    require_gcc47_contract_string(
        &receipt,
        "parity_effect",
        "evidence-backed partial; does not prove native/full GCC 4.7 correctness",
    )?;

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
    require_provider_contract_string(&receipt, spec.label, "schema", spec.schema)?;
    require_provider_contract_string(&receipt, spec.label, "derivation", spec.derivation)?;
    require_provider_contract_string(&receipt, spec.label, "status", "contract-only")?;
    require_provider_contract_string(&receipt, spec.label, "parity_effect", spec.parity_effect)?;

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
    Ok(())
}

fn require_provider_contract_string(
    value: &serde_json::Value,
    label: &str,
    field: &str,
    expected: &str,
) -> Result<(), String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{label} missing string field `{field}`"))?;
    if actual != expected {
        return Err(format!("{label} `{field}` is `{actual}`, expected `{expected}`"));
    }
    Ok(())
}

fn require_gcc47_contract_string(value: &serde_json::Value, field: &str, expected: &str) -> Result<(), String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("GCC 4.7 C++ provider contract missing string field `{field}`"))?;
    if actual != expected {
        return Err(format!("GCC 4.7 C++ provider contract `{field}` is `{actual}`, expected `{expected}`"));
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
    require_gcc40_inventory_string(&receipt, "schema", "mantle-gcc40-placeholder-inventory-v1")?;
    require_gcc40_inventory_string(&receipt, "derivation", "bootstrap/gcc-4.0.ncl")?;
    require_gcc40_inventory_string(&receipt, "status", "inventory-only")?;
    let actual_content = fs::read_to_string(derivation_path)
        .map_err(|err| format!("read GCC 4.0 derivation {}: {err}", derivation_path.display()))?;
    let actual = collect_placeholder_marker_occurrences(&actual_content);
    let receipt_markers = receipt
        .get("markers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "GCC 4.0 placeholder inventory missing array field `markers`".to_string())?;
    let marker_count = receipt
        .get("marker_count")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "GCC 4.0 placeholder inventory missing integer field `marker_count`".to_string())?;
    if marker_count as usize != receipt_markers.len() {
        return Err(format!(
            "GCC 4.0 placeholder inventory marker_count={} does not match markers length {}",
            marker_count,
            receipt_markers.len()
        ));
    }
    let mut expected = Vec::new();
    for marker in receipt_markers {
        let line = marker
            .get("line")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "GCC 4.0 placeholder inventory marker missing integer `line`".to_string())?;
        let marker_name = marker
            .get("marker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "GCC 4.0 placeholder inventory marker missing string `marker`".to_string())?;
        let classification = marker
            .get("classification")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "GCC 4.0 placeholder inventory marker missing string `classification`".to_string())?;
        if classification.trim().is_empty() {
            return Err("GCC 4.0 placeholder inventory marker classification must not be empty".to_string());
        }
        expected.push(PlaceholderMarkerOccurrence {
            line: line as usize,
            marker: marker_name.to_string(),
        });
    }
    if actual != expected {
        return Err(format!(
            "GCC 4.0 placeholder inventory drift: expected {}, recomputed {}",
            format_marker_occurrences(&expected),
            format_marker_occurrences(&actual)
        ));
    }
    validate_gcc40_native_boundary_receipt(project_root, &actual_content)?;
    validate_gcc40_native_cc1_arithmetic_receipt(project_root, &actual_content)?;
    validate_gcc40_native_generator_receipt(project_root, &actual_content)?;
    validate_gcc40_native_demangle_receipt(project_root, &actual_content)?;
    validate_gcc40_native_cc1_build_frontier_receipt(project_root, &actual_content)?;
    Ok(())
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
    require_gcc40_boundary_string(&value, "schema", "mantle-gcc40-native-boundary-v1")?;
    require_gcc40_boundary_string(&value, "derivation", "bootstrap/gcc-4.0.ncl")?;
    require_gcc40_boundary_string(&value, "status", "boundary-only")?;
    require_gcc40_boundary_string(&value, "boundary", "native-gcc-make-to-pass1-bridge")?;

    let native_attempt = require_gcc40_boundary_object(&value, "native_attempt")?;
    for field in ["command_marker", "diagnostic_marker"] {
        let marker = require_gcc40_boundary_object_string(native_attempt, field)?;
        require_gcc40_derivation_marker(derivation_content, marker)?;
    }

    let installed_bridge_markers = value
        .get("installed_bridge_markers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "GCC 4.0 native boundary receipt missing array field `installed_bridge_markers`".to_string())?;
    if installed_bridge_markers.is_empty() {
        return Err("GCC 4.0 native boundary receipt `installed_bridge_markers` must not be empty".to_string());
    }
    for marker in installed_bridge_markers {
        let marker = marker
            .as_str()
            .ok_or_else(|| "GCC 4.0 native boundary receipt marker must be a string".to_string())?;
        require_gcc40_derivation_marker(derivation_content, marker)?;
    }

    let last_log = require_gcc40_boundary_object(&value, "last_observed_build_log")?;
    require_gcc40_boundary_object_string(last_log, "status")?;
    let log_markers = last_log.get("boundary_log_markers").and_then(|v| v.as_array()).ok_or_else(|| {
        "GCC 4.0 native boundary receipt missing array field `last_observed_build_log.boundary_log_markers`".to_string()
    })?;
    if log_markers.is_empty() {
        return Err("GCC 4.0 native boundary receipt `last_observed_build_log.boundary_log_markers` must not be empty"
            .to_string());
    }

    let native_frontier = require_gcc40_boundary_object(&value, "native_frontier")?;
    require_gcc40_boundary_object_string(native_frontier, "status")?;
    let frontier_blockers = native_frontier
        .get("blockers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "GCC 4.0 native boundary receipt missing array field `native_frontier.blockers`".to_string())?;
    if frontier_blockers.is_empty() {
        return Err("GCC 4.0 native boundary receipt `native_frontier.blockers` must not be empty".to_string());
    }
    for blocker in frontier_blockers {
        let blocker = blocker.as_object().ok_or_else(|| {
            "GCC 4.0 native boundary receipt `native_frontier.blockers` entries must be objects".to_string()
        })?;
        require_gcc40_boundary_object_string(blocker, "id")?;
        let marker = require_gcc40_boundary_object_string(blocker, "derivation_marker")?;
        require_gcc40_boundary_object_string(blocker, "frontier")?;
        require_gcc40_derivation_marker(derivation_content, marker)?;
    }

    require_gcc40_boundary_string(
        &value,
        "parity_effect",
        "evidence-backed partial; does not prove native gcc.4.0 correctness",
    )?;
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
    require_gcc40_boundary_string(&value, "schema", "mantle-gcc40-native-cc1-build-frontier-v1")?;
    require_gcc40_boundary_string(&value, "derivation", "bootstrap/gcc-4.0.ncl")?;
    require_gcc40_boundary_string(&value, "status", "frontier-only")?;
    require_gcc40_boundary_string(
        &value,
        "parity_effect",
        "evidence-backed partial; does not prove native/full GCC 4.0 correctness",
    )?;
    let native_attempt = require_gcc40_boundary_object(&value, "native_attempt")?;
    for field in ["make_marker", "diagnostic_marker", "pass1_bridge_marker"] {
        let marker = require_gcc40_boundary_object_string(native_attempt, field)?;
        require_gcc40_derivation_marker(derivation_content, marker)?;
    }
    let source_frontier = value.get("source_frontier_markers").and_then(|v| v.as_array()).ok_or_else(|| {
        "GCC 4.0 native cc1 build-frontier receipt missing array field `source_frontier_markers`".to_string()
    })?;
    if source_frontier.is_empty() {
        return Err("GCC 4.0 native cc1 build-frontier receipt `source_frontier_markers` must not be empty".to_string());
    }
    for marker in source_frontier {
        let marker = marker
            .as_str()
            .ok_or_else(|| "GCC 4.0 native cc1 build-frontier source frontier marker must be a string".to_string())?;
        require_gcc40_derivation_marker(derivation_content, marker)?;
    }
    let reduction = require_gcc40_boundary_object(&value, "source_frontier_reduction")?;
    let reduction_schema = require_gcc40_boundary_object_string(reduction, "schema")?;
    if reduction_schema != "mantle-gcc40-native-cc1-source-frontier-reduction-v7" {
        return Err(format!(
            "GCC 4.0 native cc1 source-frontier reduction schema is `{reduction_schema}`, expected `mantle-gcc40-native-cc1-source-frontier-reduction-v7`"
        ));
    }
    for field in [
        "prior_frontier",
        "attempted_probe",
        "observed_frontier",
        "retirement_condition",
    ] {
        let value = require_gcc40_boundary_object_string(reduction, field)?;
        if value.trim().is_empty() {
            return Err(format!("GCC 4.0 native cc1 source-frontier reduction field `{field}` must not be empty"));
        }
    }
    let observed_result = require_gcc40_boundary_object_string(reduction, "observed_result")?;
    if observed_result != "narrowed-stable-blocker" {
        return Err(format!(
            "GCC 4.0 native cc1 source-frontier reduction observed_result is `{observed_result}`, expected `narrowed-stable-blocker`"
        ));
    }
    let observed_frontier = require_gcc40_boundary_object_string(reduction, "observed_frontier")?;
    for required_fragment in [
        "c-parse.o compile",
        "six-undef autohost probes pass",
        "cparse_undef6_inc_tm",
        "cparse_undef6_insn_modes_lines_20 fails",
        "cparse_undef6_insn_modes_lines_40 passes",
        "cparse_undef6_machmode_lines_20 fails",
        "cparse_undef6_machmode_lines_40 passes",
        "cparse_undef6_tree_lines_28 passes",
        "cparse_undef6_tree_lines_36 fails",
        "cparse_undef6_machmode_lines_120 passes",
        "cparse_undef6_tree_lines_80 passes",
        "cparse_undef6_tree_lines_120 passes",
        "cparse_undef6_tree_lines_160 passes",
        "cparse_undef6_tree_lines_220 passes",
        "cparse_undef6_tree_builtin_empty passes",
        "cparse_undef6_tree_builtin_complex_arith_only passes",
        "make: *** [c-parse.o] Error 1",
        "cparse_make_cparse_o_rc=2",
        "cparse_make_cparse_o_tail",
        "captured real c-parse.o make-error boundary",
    ] {
        if !observed_frontier.contains(required_fragment) {
            return Err(format!(
                "GCC 4.0 native cc1 source-frontier reduction observed_frontier missing required v7 fragment `{required_fragment}`"
            ));
        }
    }
    for stale_fragment in [
        "remaining runtime frontier is the copied fd_bad branch",
        "forcing the copied fd_bad branch false reaches fdopen pre/post markers",
        "autohost_defines_84_system fails while autohost_defines_84_skip84_system",
        "autohost_defines_97_undef_need64_gid_system segfaults",
        "this narrows the diagnostic frontier to generated-header prefix/balance behavior",
        "remaining real c-parse.o/full-header source-build boundary",
    ] {
        if observed_frontier.contains(stale_fragment) {
            return Err(format!(
                "GCC 4.0 native cc1 source-frontier reduction observed_frontier contains stale pre-v7 frontier fragment `{stale_fragment}`"
            ));
        }
    }
    let probe_marker = require_gcc40_boundary_object_string(reduction, "probe_marker")?;
    require_gcc40_derivation_marker(derivation_content, probe_marker)?;
    require_gcc40_boundary_object_string(reduction, "non_claim")?;
    require_gcc40_boundary_object_string(reduction, "diagnostic_derivation")?;
    let diagnostic_content = fs::read_to_string(project_root.join(GCC40_CPARSE_DIAGNOSTIC_DERIVATION)).map_err(|err| {
        format!(
            "GCC 4.0 native cc1 source-frontier diagnostic derivation missing `{}` ({err}); expected checked c-parse/decl0 diagnostic markers",
            GCC40_CPARSE_DIAGNOSTIC_DERIVATION
        )
    })?;
    let diagnostic_markers = reduction.get("diagnostic_markers").and_then(|v| v.as_array()).ok_or_else(|| {
        "GCC 4.0 native cc1 source-frontier reduction missing array field `diagnostic_markers`".to_string()
    })?;
    if diagnostic_markers.is_empty() {
        return Err("GCC 4.0 native cc1 source-frontier reduction `diagnostic_markers` must not be empty".to_string());
    }
    for marker in diagnostic_markers {
        let marker = marker
            .as_str()
            .ok_or_else(|| "GCC 4.0 native cc1 source-frontier diagnostic marker must be a string".to_string())?;
        if !diagnostic_content.contains(marker) {
            return Err(format!(
                "GCC 4.0 native cc1 source-frontier diagnostic marker `{marker}` missing from `{}`",
                GCC40_CPARSE_DIAGNOSTIC_DERIVATION
            ));
        }
    }

    let retirement = require_gcc40_boundary_object(&value, "retirement_condition")?;
    let replacement = require_gcc40_boundary_object_string(retirement, "replacement_evidence")?;
    if replacement.trim().is_empty() {
        return Err("GCC 4.0 native cc1 build-frontier retirement replacement_evidence must not be empty".to_string());
    }
    require_gcc40_boundary_object_string(retirement, "non_claim")?;
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
    require_gcc40_arithmetic_string(&value, "schema", "mantle-gcc40-native-cc1-arithmetic-v7")?;
    require_gcc40_arithmetic_string(&value, "derivation", "bootstrap/gcc-4.0.ncl")?;
    require_gcc40_arithmetic_string(&value, "status", "bounded-native-slice")?;
    require_gcc40_arithmetic_string(&value, "selected_slice", "pointer-deref-v7")?;
    require_gcc40_arithmetic_string(
        &value,
        "parity_effect",
        "evidence-backed partial; does not prove native/full GCC 4.0 correctness",
    )?;

    let smoke = value
        .get("smoke")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "GCC 4.0 native cc1 arithmetic receipt missing object field `smoke`".to_string())?;
    validate_gcc40_cc1_smoke(
        smoke,
        &[
            "int mantle_gcc40_pointer_slice(int x)",
            "int value = x;",
            "int *slot = &value;",
            "*slot = x + 4;",
            "return *slot + value;",
        ],
        "MANTLE-GCC40-NATIVE-CC1-POINTER-DEREF-SLICE-V7",
    )?;

    let regressions = value
        .get("regressions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "GCC 4.0 native cc1 arithmetic receipt missing array field `regressions`".to_string())?;
    let arithmetic_regression = regressions
        .iter()
        .find(|entry| entry.get("slice").and_then(|v| v.as_str()) == Some("arithmetic-control-flow-v1"))
        .and_then(|entry| entry.as_object())
        .ok_or_else(|| {
            "GCC 4.0 native cc1 arithmetic receipt missing arithmetic-control-flow-v1 regression".to_string()
        })?;
    validate_gcc40_cc1_smoke(
        arithmetic_regression,
        &["int mantle_gcc40_arith_slice", "return y > 7 ? y - 3 : y + 3;"],
        "MANTLE-GCC40-NATIVE-CC1-ARITHMETIC-SLICE-V1",
    )?;

    let logical_regression = regressions
        .iter()
        .find(|entry| entry.get("slice").and_then(|v| v.as_str()) == Some("logical-boolean-control-flow-v2"))
        .and_then(|entry| entry.as_object())
        .ok_or_else(|| {
            "GCC 4.0 native cc1 arithmetic receipt missing logical-boolean-control-flow-v2 regression".to_string()
        })?;
    validate_gcc40_cc1_smoke(
        logical_regression,
        &["int mantle_gcc40_logic_slice", "if ((x > 0 && y > 0) || x == y)"],
        "MANTLE-GCC40-NATIVE-CC1-LOGICAL-SLICE-V2",
    )?;

    let local_vars_regression = regressions
        .iter()
        .find(|entry| entry.get("slice").and_then(|v| v.as_str()) == Some("local-variable-assignment-v3"))
        .and_then(|entry| entry.as_object())
        .ok_or_else(|| {
            "GCC 4.0 native cc1 arithmetic receipt missing local-variable-assignment-v3 regression".to_string()
        })?;
    validate_gcc40_cc1_smoke(
        local_vars_regression,
        &["int mantle_gcc40_local_vars_slice", "int y = x + 1;", "y = y * 3;"],
        "MANTLE-GCC40-NATIVE-CC1-LOCAL-VARS-SLICE-V3",
    )?;

    let function_call_regression = regressions
        .iter()
        .find(|entry| entry.get("slice").and_then(|v| v.as_str()) == Some("function-call-v4"))
        .and_then(|entry| entry.as_object())
        .ok_or_else(|| "GCC 4.0 native cc1 arithmetic receipt missing function-call-v4 regression".to_string())?;
    validate_gcc40_cc1_smoke(
        function_call_regression,
        &[
            "int mantle_gcc40_helper(int x)",
            "int mantle_gcc40_function_call_slice(int x)",
            "return mantle_gcc40_helper(x) + 1;",
        ],
        "MANTLE-GCC40-NATIVE-CC1-FUNCTION-CALL-SLICE-V4",
    )?;

    let array_index_regression = regressions
        .iter()
        .find(|entry| entry.get("slice").and_then(|v| v.as_str()) == Some("array-index-v5"))
        .and_then(|entry| entry.as_object())
        .ok_or_else(|| "GCC 4.0 native cc1 arithmetic receipt missing array-index-v5 regression".to_string())?;
    validate_gcc40_cc1_smoke(
        array_index_regression,
        &[
            "int mantle_gcc40_array_slice(int x)",
            "int values[2];",
            "values[0] = x;",
            "return values[0] + values[1];",
        ],
        "MANTLE-GCC40-NATIVE-CC1-ARRAY-INDEX-SLICE-V5",
    )?;

    let struct_field_regression = regressions
        .iter()
        .find(|entry| entry.get("slice").and_then(|v| v.as_str()) == Some("struct-field-v6"))
        .and_then(|entry| entry.as_object())
        .ok_or_else(|| "GCC 4.0 native cc1 arithmetic receipt missing struct-field-v6 regression".to_string())?;
    validate_gcc40_cc1_smoke(
        struct_field_regression,
        &[
            "struct mantle_gcc40_pair",
            "struct mantle_gcc40_pair pair;",
            "pair.left = x;",
            "return pair.left + pair.right;",
        ],
        "MANTLE-GCC40-NATIVE-CC1-STRUCT-FIELD-SLICE-V6",
    )?;

    let no_delegation = value.get("no_tinycc_delegation").and_then(|v| v.as_object()).ok_or_else(|| {
        "GCC 4.0 native cc1 arithmetic receipt missing object field `no_tinycc_delegation`".to_string()
    })?;
    let marker = require_gcc40_arithmetic_object_string(no_delegation, "derivation_marker")?;
    require_gcc40_derivation_marker(derivation_content, marker)?;
    let regression_markers = no_delegation.get("regression_markers").and_then(|v| v.as_array()).ok_or_else(|| {
        "GCC 4.0 native cc1 arithmetic receipt missing array field `no_tinycc_delegation.regression_markers`"
            .to_string()
    })?;
    for regression_marker in regression_markers {
        let regression_marker = regression_marker
            .as_str()
            .ok_or_else(|| "GCC 4.0 native cc1 arithmetic receipt regression markers must be strings".to_string())?;
        require_gcc40_derivation_marker(derivation_content, regression_marker)?;
    }
    let forbidden = no_delegation.get("forbidden_markers").and_then(|v| v.as_array()).ok_or_else(|| {
        "GCC 4.0 native cc1 arithmetic receipt missing array field `no_tinycc_delegation.forbidden_markers`".to_string()
    })?;
    if forbidden.is_empty() {
        return Err("GCC 4.0 native cc1 arithmetic receipt forbidden marker list must not be empty".to_string());
    }
    for forbidden_marker in forbidden {
        let forbidden_marker = forbidden_marker
            .as_str()
            .ok_or_else(|| "GCC 4.0 native cc1 arithmetic receipt forbidden markers must be strings".to_string())?;
        if forbidden_marker.trim().is_empty() {
            return Err("GCC 4.0 native cc1 arithmetic receipt forbidden markers must not be empty".to_string());
        }
        for transcript in [
            require_gcc40_arithmetic_object_string(smoke, "transcript")?,
            require_gcc40_arithmetic_object_string(arithmetic_regression, "transcript")?,
            require_gcc40_arithmetic_object_string(logical_regression, "transcript")?,
            require_gcc40_arithmetic_object_string(local_vars_regression, "transcript")?,
            require_gcc40_arithmetic_object_string(function_call_regression, "transcript")?,
            require_gcc40_arithmetic_object_string(array_index_regression, "transcript")?,
            require_gcc40_arithmetic_object_string(struct_field_regression, "transcript")?,
        ] {
            if transcript.contains(forbidden_marker) {
                return Err(format!(
                    "GCC 4.0 native cc1 arithmetic receipt transcript contains forbidden TinyCC delegation marker `{forbidden_marker}`"
                ));
            }
        }
    }

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
    let actual_digest = blake3::hash(transcript.as_bytes()).to_hex().to_string();
    if expected_digest != actual_digest {
        return Err(format!(
            "GCC 4.0 native cc1 arithmetic receipt transcript digest is `{expected_digest}`, recomputed `{actual_digest}`"
        ));
    }
    let expected_output_digest = require_gcc40_arithmetic_object_string(smoke, "output_digest_blake3")?;
    if expected_output_digest.len() != 64 || !expected_output_digest.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err("GCC 4.0 native cc1 arithmetic receipt output digest must be a BLAKE3 hex digest".to_string());
    }
    Ok(())
}

fn validate_gcc40_native_generator_receipt(project_root: &Path, derivation_content: &str) -> Result<(), String> {
    let path = project_root.join(GCC40_NATIVE_GENERATOR_RECEIPT);
    let content = fs::read_to_string(&path).map_err(|err| {
        format!(
            "GCC 4.0 native generator receipt missing `{}` ({err}); expected checked bounded genattrtab/genoutput receipt",
            GCC40_NATIVE_GENERATOR_RECEIPT
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|err| format!("GCC 4.0 native generator receipt is not valid JSON: {err}"))?;
    require_gcc40_generator_string(&value, "schema", "mantle-gcc40-native-generator-slice-v2")?;
    require_gcc40_generator_string(&value, "derivation", "bootstrap/gcc-4.0.ncl")?;
    require_gcc40_generator_string(&value, "status", "bounded-native-generator-slices")?;
    require_gcc40_generator_string(
        &value,
        "parity_effect",
        "evidence-backed partial; does not prove native/full GCC 4.0 generator correctness",
    )?;

    let selected = value
        .get("selected_generators")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "GCC 4.0 native generator receipt missing array field `selected_generators`".to_string())?;
    let selected = selected
        .iter()
        .map(|v| {
            v.as_str()
                .ok_or_else(|| "GCC 4.0 native generator receipt selected generators must be strings".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    for required in ["genattrtab", "genoutput"] {
        if !selected.contains(&required) {
            return Err(format!("GCC 4.0 native generator receipt selected generators missing `{required}`"));
        }
    }

    let outputs = value
        .get("bounded_outputs")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "GCC 4.0 native generator receipt missing object field `bounded_outputs`".to_string())?;
    validate_gcc40_generator_output(outputs, derivation_content, "genattrtab", &[
        "gcc40_genattrtab_bounded_output_slice",
        "HAVE_ATTR_enabled",
    ])?;
    validate_gcc40_generator_output(outputs, derivation_content, "genoutput", &[
        "gcc40_genoutput_bounded_output_slice",
        "GCC40_GENOUTPUT_BOUNDED",
    ])?;

    let forbidden = value.get("forbidden_boundary_markers").and_then(|v| v.as_array()).ok_or_else(|| {
        "GCC 4.0 native generator receipt missing array field `forbidden_boundary_markers`".to_string()
    })?;
    if forbidden.is_empty() {
        return Err("GCC 4.0 native generator receipt forbidden boundary marker list must not be empty".to_string());
    }
    for forbidden_marker in forbidden {
        let forbidden_marker = forbidden_marker
            .as_str()
            .ok_or_else(|| "GCC 4.0 native generator receipt forbidden boundary markers must be strings".to_string())?;
        if forbidden_marker.trim().is_empty() {
            return Err("GCC 4.0 native generator receipt forbidden boundary markers must not be empty".to_string());
        }
        if derivation_content.contains(forbidden_marker) {
            return Err(format!(
                "GCC 4.0 native generator receipt derivation still contains forbidden empty-boundary marker `{forbidden_marker}`"
            ));
        }
    }

    Ok(())
}

fn validate_gcc40_generator_output(
    outputs: &serde_json::Map<String, serde_json::Value>,
    derivation_content: &str,
    generator: &str,
    required_fragments: &[&str],
) -> Result<(), String> {
    let output = outputs
        .get(generator)
        .and_then(|v| v.as_object())
        .ok_or_else(|| format!("GCC 4.0 native generator receipt missing bounded output `{generator}`"))?;
    let marker = require_gcc40_generator_object_string(output, "derivation_marker")?;
    require_gcc40_derivation_marker(derivation_content, marker)?;
    let contract = require_gcc40_generator_object_string(output, "contract")?;
    if !contract.contains(generator) || !contract.contains("bounded") {
        return Err(format!("GCC 4.0 native generator receipt contract must describe a bounded {generator} output"));
    }
    let fragment = require_gcc40_generator_object_string(output, "output_program_fragment")?;
    for required in required_fragments {
        if !fragment.contains(required) {
            return Err(format!(
                "GCC 4.0 native generator receipt {generator} output fragment missing required bounded fragment `{required}`"
            ));
        }
    }
    let expected_output_digest = require_gcc40_generator_object_string(output, "output_digest_blake3")?;
    let actual_output_digest = blake3::hash(fragment.as_bytes()).to_hex().to_string();
    if expected_output_digest != actual_output_digest {
        return Err(format!(
            "GCC 4.0 native generator receipt {generator} output digest is `{expected_output_digest}`, recomputed `{actual_output_digest}`"
        ));
    }
    let transcript = require_gcc40_generator_object_string(output, "transcript")?;
    let expected_digest = require_gcc40_generator_object_string(output, "transcript_digest_blake3")?;
    let actual_digest = blake3::hash(transcript.as_bytes()).to_hex().to_string();
    if expected_digest != actual_digest {
        return Err(format!(
            "GCC 4.0 native generator receipt {generator} transcript digest is `{expected_digest}`, recomputed `{actual_digest}`"
        ));
    }
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
    require_gcc40_demangle_string(&value, "schema", "mantle-gcc40-native-demangle-slice-v5")?;
    require_gcc40_demangle_string(&value, "derivation", "bootstrap/gcc-4.0.ncl")?;
    require_gcc40_demangle_string(&value, "status", "bounded-native-demangle-slice")?;
    require_gcc40_demangle_string(&value, "selected_shape", "single-long-arg-itanium-v5")?;
    require_gcc40_demangle_string(
        &value,
        "parity_effect",
        "evidence-backed partial; does not prove native/full GCC 4.0 demangler correctness",
    )?;

    let contract = value
        .get("bounded_contract")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "GCC 4.0 native demangle receipt missing object field `bounded_contract`".to_string())?;
    require_gcc40_demangle_named_io(contract, "accepted_inputs", "_ZN3foo3bar3bazEl", "foo::bar::baz(long)")?;
    require_gcc40_demangle_named_io(contract, "long_regressions", "_ZN3foo3barEl", "foo::bar(long)")?;
    require_gcc40_demangle_named_io(contract, "flat_long_regressions", "_Z3fool", "foo(long)")?;
    require_gcc40_demangle_named_io(contract, "char_regressions", "_ZN3foo3bar3bazEc", "foo::bar::baz(char)")?;
    require_gcc40_demangle_named_io(contract, "char_regressions", "_ZN3foo3barEc", "foo::bar(char)")?;
    require_gcc40_demangle_named_io(contract, "flat_char_regressions", "_Z3fooc", "foo(char)")?;
    require_gcc40_demangle_named_io(contract, "int_regressions", "_ZN3foo3bar3bazEi", "foo::bar::baz(int)")?;
    require_gcc40_demangle_named_io(contract, "int_regressions", "_ZN3foo3barEi", "foo::bar(int)")?;
    require_gcc40_demangle_named_io(contract, "flat_int_regressions", "_Z3fooi", "foo(int)")?;
    require_gcc40_demangle_named_io(contract, "zero_arg_regressions", "_ZN3foo3bar3bazEv", "foo::bar::baz()")?;
    require_gcc40_demangle_named_io(contract, "nested_regressions", "_ZN3foo3barEv", "foo::bar()")?;
    require_gcc40_demangle_named_io(contract, "flat_regressions", "_Z3foov", "foo()")?;
    let rejected = contract.get("rejected_inputs").and_then(|v| v.as_array()).ok_or_else(|| {
        "GCC 4.0 native demangle receipt missing array field `bounded_contract.rejected_inputs`".to_string()
    })?;
    for required in [
        "_ZN3foo3bar3bazEf",
        "_ZN3foo3bar3bazEx",
        "_ZN3foo3bar3baz3quxEl",
        "_ZN3foo3bar3baz3quxEc",
        "_ZN3foo3bar3baz3quxEi",
        "_ZN3foo3bar3baz3quxEv",
        "_ZN3fooE",
        "not_mangled",
    ] {
        if !rejected.iter().any(|v| v.as_str() == Some(required)) {
            return Err(format!("GCC 4.0 native demangle receipt rejected inputs missing `{required}`"));
        }
    }
    let non_claim = require_gcc40_demangle_object_string(contract, "non_claim")?;
    if !non_claim.contains("single-long") || !non_claim.contains("in scope") {
        return Err("GCC 4.0 native demangle receipt non-claim must bound the supported shape".to_string());
    }

    let markers = value
        .get("source_markers")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "GCC 4.0 native demangle receipt missing object field `source_markers`".to_string())?;
    for field in ["cplus_demangle_marker", "cp_demangle_marker"] {
        let marker = require_gcc40_demangle_object_string(markers, field)?;
        require_gcc40_derivation_marker(derivation_content, marker)?;
    }

    let smoke = value
        .get("smoke")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "GCC 4.0 native demangle receipt missing object field `smoke`".to_string())?;
    let transcript = require_gcc40_demangle_object_string(smoke, "transcript")?;
    for required in [
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
    ] {
        if !transcript.contains(required) {
            return Err(format!("GCC 4.0 native demangle receipt transcript missing `{required}`"));
        }
    }
    let expected_digest = require_gcc40_demangle_object_string(smoke, "transcript_digest_blake3")?;
    let actual_digest = blake3::hash(transcript.as_bytes()).to_hex().to_string();
    if expected_digest != actual_digest {
        return Err(format!(
            "GCC 4.0 native demangle receipt transcript digest is `{expected_digest}`, recomputed `{actual_digest}`"
        ));
    }
    require_gcc40_demangle_object_string(smoke, "output_digest_blake3")?;

    let forbidden = value
        .get("forbidden_stale_markers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "GCC 4.0 native demangle receipt missing array field `forbidden_stale_markers`".to_string())?;
    if forbidden.is_empty() {
        return Err("GCC 4.0 native demangle receipt forbidden stale marker list must not be empty".to_string());
    }
    for marker in forbidden {
        let marker = marker
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

    Ok(())
}

fn require_gcc40_demangle_named_io(
    contract: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    expected_mangled: &str,
    expected_demangled: &str,
) -> Result<(), String> {
    let entries = contract
        .get(field)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("GCC 4.0 native demangle receipt missing array field `bounded_contract.{field}`"))?;
    if entries.iter().any(|entry| {
        entry.get("mangled").and_then(|v| v.as_str()) == Some(expected_mangled)
            && entry.get("demangled").and_then(|v| v.as_str()) == Some(expected_demangled)
    }) {
        Ok(())
    } else {
        Err(format!(
            "GCC 4.0 native demangle receipt `{field}` missing `{expected_mangled} -> {expected_demangled}`"
        ))
    }
}

fn require_gcc40_demangle_string<'a>(
    value: &'a serde_json::Value,
    field: &str,
    expected: &str,
) -> Result<&'a str, String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("GCC 4.0 native demangle receipt missing string field `{field}`"))?;
    if actual != expected {
        return Err(format!("GCC 4.0 native demangle receipt `{field}` is `{actual}`, expected `{expected}`"));
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
    field: &str,
    expected: &str,
) -> Result<&'a str, String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("GCC 4.0 native generator receipt missing string field `{field}`"))?;
    if actual != expected {
        return Err(format!("GCC 4.0 native generator receipt `{field}` is `{actual}`, expected `{expected}`"));
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
    field: &str,
    expected: &str,
) -> Result<&'a str, String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("GCC 4.0 native cc1 arithmetic receipt missing string field `{field}`"))?;
    if actual != expected {
        return Err(format!("GCC 4.0 native cc1 arithmetic receipt `{field}` is `{actual}`, expected `{expected}`"));
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

fn require_gcc40_derivation_marker(content: &str, marker: &str) -> Result<(), String> {
    if marker.trim().is_empty() {
        return Err("GCC 4.0 native boundary receipt marker must not be empty".to_string());
    }
    if !content.contains(marker) {
        return Err(format!("GCC 4.0 native boundary receipt marker not found in derivation: {marker}"));
    }
    Ok(())
}

fn require_gcc40_boundary_string<'a>(
    value: &'a serde_json::Value,
    field: &str,
    expected: &str,
) -> Result<&'a str, String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("GCC 4.0 native boundary receipt missing string field `{field}`"))?;
    if actual != expected {
        return Err(format!("GCC 4.0 native boundary receipt `{field}` is `{actual}`, expected `{expected}`"));
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
    field: &str,
    expected: &str,
) -> Result<&'a str, String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("GCC 4.0 placeholder inventory missing string field `{field}`"))?;
    if actual != expected {
        return Err(format!("GCC 4.0 placeholder inventory `{field}` is `{actual}`, expected `{expected}`"));
    }
    Ok(actual)
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct PlaceholderMarkerOccurrence {
    line: usize,
    marker: String,
}

fn collect_placeholder_marker_occurrences(content: &str) -> Vec<PlaceholderMarkerOccurrence> {
    let mut occurrences = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        for marker in PLACEHOLDER_MARKERS {
            if marker_is_standalone(line, marker) {
                occurrences.push(PlaceholderMarkerOccurrence {
                    line: idx + 1,
                    marker: (*marker).to_string(),
                });
            }
        }
    }
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
            "StageX lineage provider receipt missing `{}` ({err}); expected schema, provider_kind=stagex-lineage, lineage_receipt_status=scaffold-only, digest fields, and fallback_events=[]",
            STAGEX_LINEAGE_PROVIDER_RECEIPT
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|err| format!("StageX lineage provider receipt is not valid JSON: {err}"))?;
    require_stagex_json_string(&value, "schema", "mantle-stagex-lineage-provider-receipt-v1")?;
    require_stagex_json_string(&value, "provider_kind", "stagex-lineage")?;
    require_stagex_json_string(&value, "lineage_receipt_status", "scaffold-only")?;
    for field in [
        "audited_seed_digest",
        "lineage_manifest_digest",
        "stage_graph_digest",
        "normalized_provider_digest",
    ] {
        let digest = require_stagex_non_empty_string(&value, field)?;
        validate_lower_hex_digest(digest, field)?;
    }
    require_stagex_empty_array(&value, "fallback_events")?;
    Ok(())
}

fn require_stagex_json_string<'a>(
    value: &'a serde_json::Value,
    field: &str,
    expected: &str,
) -> Result<&'a str, String> {
    let actual = require_stagex_non_empty_string(value, field)?;
    if actual != expected {
        return Err(format!("StageX lineage provider receipt `{field}` is `{actual}`, expected `{expected}`"));
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

fn validate_lower_hex_digest(digest: &str, field: &str) -> Result<(), String> {
    if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return Err(format!("StageX lineage provider receipt `{field}` must be a 64-character lowercase hex digest"));
    }
    Ok(())
}

fn require_stagex_empty_array(value: &serde_json::Value, field: &str) -> Result<(), String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("StageX lineage provider receipt missing array field `{field}`"))?;
    if !actual.is_empty() {
        return Err(format!("StageX lineage provider receipt `{field}` must be empty for scaffold-only evidence"));
    }
    Ok(())
}

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

    require_real_proof_string(&value, "schema", "mantle-real-self-build-proof-parity-evidence-v1")?;
    let release_id = require_real_proof_non_empty_string(&value, "release_id")?.to_string();
    let selected_provider_kind = parse_provider_kind(
        require_real_proof_non_empty_string(&value, "selected_provider_kind")?,
        "selected_provider_kind",
    )?;
    let provider_kind_linkage = require_real_proof_object(&value, "provider_kind_linkage")?;
    let deterministic_proof = require_real_proof_object(&value, "deterministic_proof")?;
    let verify_receipt = require_real_proof_object(&value, "verify_receipt")?;
    let sandbox_evidence = require_real_proof_object(&value, "sandbox_evidence")?;
    let summary = require_real_proof_object(&value, "summary")?;

    require_real_proof_object_string(provider_kind_linkage, "receipt_path", SELF_BUILD_PROVIDER_KIND_LINKAGE_RECEIPT)?;
    require_real_proof_object_provider_kind(provider_kind_linkage, "selected_provider_kind", selected_provider_kind)?;

    require_real_proof_object_string(deterministic_proof, "workflow", "mantle-deterministic-proof-receipt-v1")?;
    require_real_proof_object_string(deterministic_proof, "verdict", "self-rebuild-match")?;
    require_real_proof_object_provider_kind(deterministic_proof, "selected_provider_kind", selected_provider_kind)?;
    let deterministic_proof_digest_blake3 =
        require_real_proof_object_digest(deterministic_proof, "digest_blake3")?.to_string();
    let source_digest = require_real_proof_object_digest(deterministic_proof, "source_blake3")?;
    let vendor_digest = require_real_proof_object_digest(deterministic_proof, "vendor_blake3")?;

    let roots = deterministic_proof.get("clean_rebuild_roots").and_then(|v| v.as_array()).ok_or_else(|| {
        "real self-build proof parity receipt missing array field `deterministic_proof.clean_rebuild_roots`".to_string()
    })?;
    if roots.len() != 2 {
        return Err(format!(
            "real self-build proof parity receipt `deterministic_proof.clean_rebuild_roots` has length {}, expected 2",
            roots.len()
        ));
    }
    let first_root = roots[0]
        .as_str()
        .ok_or_else(|| "real self-build proof clean rebuild root must be a string".to_string())?;
    let second_root = roots[1]
        .as_str()
        .ok_or_else(|| "real self-build proof clean rebuild root must be a string".to_string())?;
    if first_root.trim().is_empty() || second_root.trim().is_empty() {
        return Err("real self-build proof clean rebuild roots must not be empty".to_string());
    }
    if first_root == second_root {
        return Err("real self-build proof clean rebuild roots must be distinct".to_string());
    }
    let artifacts = deterministic_proof.get("artifact_digests_blake3").and_then(|v| v.as_array()).ok_or_else(|| {
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
        validate_blake3_digest(digest, "deterministic_proof.artifact_digests_blake3")?;
    }

    let sandbox_evidence_digest_blake3 =
        require_real_proof_object_digest(sandbox_evidence, "digest_blake3")?.to_string();
    let sandbox_profile_identity =
        require_real_proof_object_non_empty_string(sandbox_evidence, "profile_identity")?.to_string();
    if !sandbox_profile_identity.starts_with("mantle-proof-sandbox-v1:") {
        return Err(format!(
            "real self-build proof sandbox profile `{sandbox_profile_identity}` is unsupported; expected mantle-proof-sandbox-v1:*"
        ));
    }

    let verify_receipt_digest_blake3 = require_real_proof_object_digest(verify_receipt, "digest_blake3")?.to_string();
    require_real_proof_object_string(verify_receipt, "deterministic_release_status", "eligible")?;
    require_real_proof_object_string(verify_receipt, "proof_digest_blake3", &deterministic_proof_digest_blake3)?;
    require_real_proof_object_string(
        verify_receipt,
        "sandbox_evidence_digest_blake3",
        &sandbox_evidence_digest_blake3,
    )?;

    let summary_json_digest_blake3 = require_real_proof_object_digest(summary, "json_digest_blake3")?.to_string();
    let summary_markdown_digest_blake3 =
        require_real_proof_object_digest(summary, "markdown_digest_blake3")?.to_string();
    require_real_proof_object_string(summary, "release_id", &release_id)?;
    require_real_proof_object_provider_kind(summary, "selected_provider_kind", selected_provider_kind)?;
    require_real_proof_object_string(summary, "verdict", "self-rebuild-match")?;
    require_real_proof_object_string(summary, "verify_status", "eligible")?;
    require_real_proof_object_string(summary, "proof_digest_blake3", &deterministic_proof_digest_blake3)?;
    require_real_proof_object_string(summary, "sandbox_evidence_digest_blake3", &sandbox_evidence_digest_blake3)?;
    let bounded_claim = require_real_proof_object_non_empty_string(summary, "bounded_claim")?.to_string();
    if !bounded_claim.contains("rebuilt twice") || !bounded_claim.contains("recorded inputs") {
        return Err("real self-build proof bounded claim must state rebuilt-twice recorded-input scope".to_string());
    }
    let non_claims = summary
        .get("non_claims")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "real self-build proof summary missing array field `non_claims`".to_string())?;
    if !non_claims
        .iter()
        .any(|claim| claim.as_str().unwrap_or("").contains("full bootstrap reproducibility"))
    {
        return Err("real self-build proof summary non_claims must reject full bootstrap reproducibility".to_string());
    }

    require_real_proof_string(&value, "source_blake3", source_digest)?;
    require_real_proof_string(&value, "vendor_blake3", vendor_digest)?;

    Ok(EvidenceValidation::with_proof(ParityProofDetails {
        schema: "mantle-real-self-build-proof-parity-evidence-v1".to_string(),
        release_id,
        selected_provider_kind,
        evidence_digest_blake3,
        deterministic_proof_digest_blake3,
        sandbox_evidence_digest_blake3,
        verify_receipt_digest_blake3,
        summary_json_digest_blake3,
        summary_markdown_digest_blake3,
        verdict: "self-rebuild-match".to_string(),
        verify_status: "eligible".to_string(),
        sandbox_profile_identity,
        bounded_claim,
    }))
}

fn parse_provider_kind(kind: &str, field: &str) -> Result<ProviderKind, String> {
    match kind {
        "legacy-fetch" => Ok(ProviderKind::LegacyFetch),
        "source-root" => Ok(ProviderKind::SourceRoot),
        "stagex-lineage" => Ok(ProviderKind::StagexLineage),
        other => Err(format!("real self-build proof parity receipt `{field}` has unknown provider kind `{other}`")),
    }
}

fn require_real_proof_string<'a>(value: &'a serde_json::Value, field: &str, expected: &str) -> Result<&'a str, String> {
    let actual = require_real_proof_non_empty_string(value, field)?;
    if actual != expected {
        return Err(format!("real self-build proof parity receipt `{field}` is `{actual}`, expected `{expected}`"));
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
    field: &str,
    expected: &str,
) -> Result<&'a str, String> {
    let actual = require_real_proof_object_non_empty_string(object, field)?;
    if actual != expected {
        return Err(format!("real self-build proof parity receipt `{field}` is `{actual}`, expected `{expected}`"));
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
    validate_blake3_digest(digest, field)?;
    Ok(digest)
}

fn require_real_proof_object_provider_kind(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    expected: ProviderKind,
) -> Result<(), String> {
    let actual = parse_provider_kind(require_real_proof_object_non_empty_string(object, field)?, field)?;
    if actual != expected {
        return Err(format!(
            "real self-build proof parity receipt provider kind mismatch in `{field}`: {actual}, expected {expected}"
        ));
    }
    Ok(())
}

fn validate_blake3_digest(digest: &str, field: &str) -> Result<(), String> {
    if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return Err(format!(
            "real self-build proof parity receipt `{field}` must be a 64-character lowercase BLAKE3 digest"
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

    require_self_build_json_string(&value, "schema", "mantle-self-build-provider-kind-linkage-v1")?;
    let proof_identity = require_self_build_object(&value, "proof_identity")?;
    let proof_linkage = require_self_build_object(&value, "proof_linkage")?;
    let prerequisites = require_self_build_object(&value, "prerequisites")?;
    let proof_identity_kind = require_self_build_object_string(proof_identity, "selected_provider_kind")?;
    let proof_linkage_kind = require_self_build_object_string(proof_linkage, "selected_provider_kind")?;
    let prerequisites_kind = require_self_build_object_string(prerequisites, "provider_kind")?;

    validate_closed_provider_kind(proof_identity_kind, "proof_identity.selected_provider_kind")?;
    validate_closed_provider_kind(proof_linkage_kind, "proof_linkage.selected_provider_kind")?;
    validate_closed_provider_kind(prerequisites_kind, "prerequisites.provider_kind")?;
    if proof_identity_kind != proof_linkage_kind || proof_identity_kind != prerequisites_kind {
        return Err(format!(
            "crunch self-build provider-kind linkage mismatch: proof_identity.selected_provider_kind={proof_identity_kind}, proof_linkage.selected_provider_kind={proof_linkage_kind}, prerequisites.provider_kind={prerequisites_kind}"
        ));
    }
    Ok(())
}

fn validate_closed_provider_kind(kind: &str, field: &str) -> Result<(), String> {
    match kind {
        "legacy-fetch" | "source-root" | "stagex-lineage" => Ok(()),
        other => Err(format!("crunch self-build provider-kind linkage `{field}` has unknown provider kind `{other}`")),
    }
}

fn require_self_build_json_string<'a>(
    value: &'a serde_json::Value,
    field: &str,
    expected: &str,
) -> Result<&'a str, String> {
    let actual = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("crunch self-build provider-kind linkage missing string field `{field}`"))?;
    if actual != expected {
        return Err(format!("crunch self-build provider-kind linkage `{field}` is `{actual}`, expected `{expected}`"));
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
    require_json_string(&value, "schema", "mantle-binutils-tcc-tool-smoke-v1")?;
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
            semantic_evidence: "bounded libgcc/driver/cc1 arithmetic+logical+local-vars+function-call+array-index+struct-field+pointer-deref and generator boundary smokes only; native compiler correctness not proven",
            proof_evidence: "source transcript, placeholder inventory, native-boundary receipt, native-cc1 slice receipt, native-generator receipt, and native-cc1 build/source-frontier receipt required",
            notes: "pass1 bridge and selected bounded semantics are partial progress; checked placeholder inventory at bootstrap/evidence/gcc-4.0-placeholder-inventory.json records remaining marker debt, native-boundary receipt at bootstrap/evidence/gcc-4.0-native-boundary.json records the intentional bridge boundary, native-cc1 receipt at bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json records no-TinyCC-delegation arithmetic, logical/control-flow, local-variable, helper-call, array-index, struct-field, and pointer-deref slices, native-generator receipt at bootstrap/evidence/gcc-4.0-native-generator-slice.json records bounded genattrtab and genoutput slices, and native-cc1 build/source-frontier receipt at bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json records source-build frontier markers plus the bounded unchanged c-parse/gengtype-yacc probe, while remaining native generator/compiler correctness is still unproven",
            evidence_check: EvidenceCheck::Gcc40PlaceholderInventory,
        },
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
        },
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
            proof_evidence: "source-root proof and checked full musl/binutils provider contract required",
            notes: "checked receipt at bootstrap/evidence/full-musl-binutils-provider-contract.json validates the musl 1.2.5 and binutils 2.41 configure/build/install/smoke contract; row remains partial until full toolchain correctness and source-root proof are proven",
            evidence_check: EvidenceCheck::FullMuslBinutilsProviderContract,
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
            notes: "Guix source-root seed-full evidence must not satisfy this StageX row; checked scaffold receipt at bootstrap/evidence/stagex-lineage-provider-receipt.json records provider_kind=stagex-lineage but does not prove audited lineage",
            evidence_check: EvidenceCheck::StagexLineageProviderReceipt,
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
            notes: "release evidence must bind selected provider kind; checked receipt at bootstrap/evidence/crunch-self-build-provider-kind-linkage.json validates proof_identity/proof_linkage/prerequisites provider-kind equality but does not prove full self-build",
            evidence_check: EvidenceCheck::RealSelfBuildProof,
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

    fn gcc40_spec() -> StageSpec {
        StageSpec {
            id: "gcc.4.0",
            title: "GCC 4.0",
            axes: LIVE_GUIX,
            lineage: "live-bootstrap",
            derivation: Some("gcc-4.0.ncl"),
            expected_complete: false,
            graph_evidence: "late graph completion recorded",
            semantic_evidence: "bounded libgcc/driver/cc1 arithmetic+logical+local-vars+function-call+array-index+struct-field+pointer-deref and generator boundary smokes only; native compiler correctness not proven",
            proof_evidence: "source transcript, placeholder inventory, native-boundary receipt, native-cc1 slice receipt, native-generator receipt, and native-cc1 build/source-frontier receipt required",
            notes: "pass1 bridge and selected bounded semantics are partial progress; checked placeholder inventory at bootstrap/evidence/gcc-4.0-placeholder-inventory.json records remaining marker debt, native-boundary receipt at bootstrap/evidence/gcc-4.0-native-boundary.json records the intentional bridge boundary, native-cc1 receipt at bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json records no-TinyCC-delegation arithmetic, logical/control-flow, local-variable, helper-call, array-index, struct-field, and pointer-deref slices, native-generator receipt at bootstrap/evidence/gcc-4.0-native-generator-slice.json records bounded genattrtab and genoutput slices, and native-cc1 build/source-frontier receipt at bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json records source-build frontier markers plus the bounded unchanged c-parse/gengtype-yacc probe, while remaining native generator/compiler correctness is still unproven",
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
            expected_complete: false,
            graph_evidence: "StageX lineage provider derivation/proof not yet bound",
            semantic_evidence: "audited lineage provider contract validation required",
            proof_evidence: "lineage provider digest and transcript required",
            notes: "Guix source-root seed-full evidence must not satisfy this StageX row; checked scaffold receipt at bootstrap/evidence/stagex-lineage-provider-receipt.json records provider_kind=stagex-lineage but does not prove audited lineage",
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
            path,
            format!(
                r#"{{
  "schema": "mantle-stagex-lineage-provider-receipt-v1",
  "provider_kind": "{provider_kind}",
  "lineage_receipt_status": "{status}",
  "audited_seed_digest": "{audited_seed_digest}",
  "lineage_manifest_digest": "2222222222222222222222222222222222222222222222222222222222222222",
  "stage_graph_digest": "3333333333333333333333333333333333333333333333333333333333333333",
  "normalized_provider_digest": "4444444444444444444444444444444444444444444444444444444444444444",
  "fallback_events": {fallback_events}
}}"#
            ),
        )
        .unwrap();
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
    "workflow": "mantle-deterministic-proof-receipt-v1",
    "verdict": "self-rebuild-match",
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
    "bounded_claim": "This artifact rebuilt twice from the recorded inputs under the recorded sandbox and matched.",
    "non_claims": ["full bootstrap reproducibility"]
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
      "id": "libiberty-demangle-single-long-arg",
      "receipt": "bootstrap/evidence/gcc-4.0-native-demangle-slice.json",
      "derivation_marker": "gcc40_cplus_demangle_long_arg_itanium_v5_boundary"
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
        "derivation_marker": "gcc40_cplus_demangle_long_arg_itanium_v5_boundary",
        "frontier": "libiberty demangling has checked bounded flat, two-component nested, selected three-component nested zero-argument, selected single-int, selected single-char, and selected single-long Itanium semantic slices; full native cp-demangle remains pending"
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
        let mut content = format!(
            r#"{{
  "schema": "mantle-gcc40-native-generator-slice-v2",
  "derivation": "bootstrap/gcc-4.0.ncl",
  "status": "bounded-native-generator-slices",
  "selected_generators": ["genattrtab", "genoutput"],
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
    }}
  }},
  "forbidden_boundary_markers": [
    "Crunch GCC 4.0 empty-attrtab source boundary: native genattrtab promotion pending.",
    "gcc40_genattrtab_empty_attrtab_source_boundary",
    "Crunch GCC 4.0 empty-output source boundary: native genoutput promotion pending.",
    "gcc40_genoutput_empty_output_source_boundary"
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
            blake3::hash(genoutput_fragment.as_bytes()).to_hex()
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
selected_shape: single-long-arg-itanium-v5
input: _ZN3foo3bar3bazEl
output: foo::bar::baz(long)
long_regression: _ZN3foo3barEl -> foo::bar(long)
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
negative: _ZN3foo3bar3baz3quxEl -> <null>
negative: _ZN3foo3bar3baz3quxEc -> <null>
negative: _ZN3foo3bar3baz3quxEi -> <null>
negative: _ZN3foo3bar3baz3quxEv -> <null>
exit_status: 0
stdout: <empty>
stderr: <empty>
non_claim: full native cp-demangle and GCC 4.0 correctness pending
";
        let mut content = format!(
            r#"{{
  "schema": "mantle-gcc40-native-demangle-slice-v5",
  "derivation": "bootstrap/gcc-4.0.ncl",
  "status": "bounded-native-demangle-slice",
  "selected_shape": "single-long-arg-itanium-v5",
  "bounded_contract": {{
    "accepted_inputs": [{{ "mangled": "_ZN3foo3bar3bazEl", "demangled": "foo::bar::baz(long)" }}],
    "long_regressions": [{{ "mangled": "_ZN3foo3barEl", "demangled": "foo::bar(long)" }}],
    "flat_long_regressions": [{{ "mangled": "_Z3fool", "demangled": "foo(long)" }}],
    "char_regressions": [{{ "mangled": "_ZN3foo3bar3bazEc", "demangled": "foo::bar::baz(char)" }}, {{ "mangled": "_ZN3foo3barEc", "demangled": "foo::bar(char)" }}],
    "flat_char_regressions": [{{ "mangled": "_Z3fooc", "demangled": "foo(char)" }}],
    "int_regressions": [{{ "mangled": "_ZN3foo3bar3bazEi", "demangled": "foo::bar::baz(int)" }}, {{ "mangled": "_ZN3foo3barEi", "demangled": "foo::bar(int)" }}],
    "flat_int_regressions": [{{ "mangled": "_Z3fooi", "demangled": "foo(int)" }}],
    "zero_arg_regressions": [{{ "mangled": "_ZN3foo3bar3bazEv", "demangled": "foo::bar::baz()" }}],
    "nested_regressions": [{{ "mangled": "_ZN3foo3barEv", "demangled": "foo::bar()" }}],
    "flat_regressions": [{{ "mangled": "_Z3foov", "demangled": "foo()" }}],
    "rejected_inputs": ["_ZN3foo3bar3bazEf", "_ZN3foo3bar3bazEx", "_ZN3foo3bar3baz3quxEl", "_ZN3foo3bar3baz3quxEc", "_ZN3foo3bar3baz3quxEi", "_ZN3foo3bar3baz3quxEv", "_ZN3fooE", "not_mangled"],
    "non_claim": "only flat, two-component nested, and selected three-component nested zero-argument, single-int, single-char, or single-long Itanium function names are in scope"
  }},
  "source_markers": {{
    "cplus_demangle_marker": "gcc40_cplus_demangle_long_arg_itanium_v5_boundary",
    "cp_demangle_marker": "gcc40_cp_demangle_long_arg_itanium_v5_boundary"
  }},
  "smoke": {{
    "transcript": {},
    "transcript_digest_blake3": "{}",
    "output_digest_blake3": "{}"
  }},
  "forbidden_stale_markers": [
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
            blake3::hash(b"foo::bar::baz(long)\n").to_hex()
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
        let mut content = r#"{
  "schema": "mantle-gcc40-native-cc1-build-frontier-v1",
  "derivation": "bootstrap/gcc-4.0.ncl",
  "status": "frontier-only",
  "native_attempt": {
    "make_marker": "make -j1 -C gcc gengtype-yacc.c CC=tcc AR=\"$BINUTILS/bin/ar\" RANLIB=\"$BINUTILS/bin/ranlib\" MAKEINFO=true 2>&1 || true",
    "diagnostic_marker": "MANTLE: gcc-4.0 native cc1 build reached TinyCC/Mes source boundary; installing pass1 bridge",
    "pass1_bridge_marker": "the validated TinyCC handoff. This remains a bridge until native cc1 builds."
  },
  "source_frontier_markers": [
    "under the c-parse flags deterministically segfaults TinyCC after that",
    "rewrites before system.h trigger deterministic TinyCC segfaults.",
    "through TinyCC/Mes diagnostics and segfault; seed the same inert files"
  ],
  "source_frontier_reduction": {
    "schema": "mantle-gcc40-native-cc1-source-frontier-reduction-v7",
    "prior_frontier": "source-frontier v4 narrowed the c-parse diagnostic to the auto-host.h macro window; six targeted undefines for NEED_64BIT_HOST_WIDE_INT, gid_t, inline, rlim_t, ssize_t, and uid_t advance through the autohost/system.h boundary",
    "attempted_probe": "bounded c-parse generated-header probe from bootstrap/diag-gcc40-c-parse-boundary.ncl using the six-undef autohost configuration and representative insn-modes.h, machmode.h, and tree.h prefix/include probes",
    "probe_marker": "MANTLE-GCC40-NATIVE-CC1-SOURCE-FRONTIER-REDUCTION-V1: bounded gengtype-yacc probe records unchanged TinyCC/Mes c-parse boundary after generated-header seeds.",
    "observed_result": "narrowed-stable-blocker",
    "observed_frontier": "the focused c-parse make attempt reaches the real c-parse.o compile and still fails with make: *** [c-parse.o] Error 1 after generated config.h normalization; six-undef autohost probes pass through autohost_defines_120_undef_need64_gid_inline_rlim_ssize_uid_system, autohost_full_undef_need64_gid_inline_rlim_ssize_uid_system, cparse_config_undef6_system, cparse_undef6_inc_system, cparse_undef6_inc_coretypes, and cparse_undef6_inc_tm; generated-header sweep probes show cparse_undef6_insn_modes_include_only passes, cparse_undef6_insn_modes_lines_20 fails while cparse_undef6_insn_modes_lines_40 passes, cparse_undef6_machmode_lines_20 fails while cparse_undef6_machmode_lines_40 passes and cparse_undef6_machmode_lines_60 passes, cparse_undef6_machmode_lines_80 passes, cparse_undef6_machmode_lines_100 passes, and cparse_undef6_machmode_lines_120 passes; cparse_undef6_tree_lines_28 passes while cparse_undef6_tree_lines_36 fails, but cparse_undef6_tree_lines_80 passes, cparse_undef6_tree_lines_120 passes, cparse_undef6_tree_lines_160 passes, cparse_undef6_tree_lines_166 passes, cparse_undef6_tree_lines_177 passes, cparse_undef6_tree_lines_180 passes, cparse_undef6_tree_lines_207 passes, cparse_undef6_tree_lines_212 passes, and cparse_undef6_tree_lines_220 passes; cparse_undef6_tree_builtin_empty passes and cparse_undef6_tree_builtin_complex_arith_only passes; the diagnostic capture records cparse_make_cparse_o_rc=2, cparse_make_cparse_o_tail, repeated In file included from diagnostics, and make: *** [c-parse.o] Error 1; this narrows the diagnostic frontier to the captured real c-parse.o make-error boundary but still does not prove the native GCC 4.0 c-parse/cc1 source build",
    "retirement_condition": "replace when the diagnostic handoff advances beyond the full generated-header sweep and real c-parse.o failure to a native GCC 4.0 c-parse/cc1 source-build step without pass1 fallback",
    "diagnostic_derivation": "bootstrap/diag-gcc40-c-parse-boundary.ncl",
    "diagnostic_markers": [
      "make_autohost_define_undef6_probe autohost_defines_120_undef_need64_gid_inline_rlim_ssize_uid_system 120 NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t ssize_t uid_t",
      "make_autohost_full_undef6_probe autohost_full_undef_need64_gid_inline_rlim_ssize_uid_system NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t ssize_t uid_t",
      "try_cparse_variant cparse_config_undef6_system /tmp/cparse_config_undef6_system.c",
      "make_include_probe_config_undef6 cparse_undef6_inc_system 'system.h'",
      "make_include_probe_config_undef6 cparse_undef6_inc_coretypes 'system.h coretypes.h'",
      "make_include_probe_config_undef6 cparse_undef6_inc_tm 'system.h coretypes.h tm.h'",
      "make_insn_modes_manual_config_undef6 cparse_undef6_insn_modes_include_only '#include \"insn-modes.h\"'",
      "make_insn_modes_header_prefix_config_undef6 cparse_undef6_insn_modes_lines_20 20",
      "make_insn_modes_header_prefix_config_undef6 cparse_undef6_insn_modes_lines_40 40",
      "make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_20 20",
      "make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_40 40",
      "make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_60 60",
      "make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_80 80",
      "make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_100 100",
      "make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_120 120",
      "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_28 28",
      "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_36 36",
      "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_80 80",
      "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_120 120",
      "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_160 160",
      "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_166 166",
      "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_177 177",
      "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_180 180",
      "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_207 207",
      "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_212 212",
      "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_220 220",
      "make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_empty 'enum built_in_function {",
      "make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_complex_arith_only",
      "m=/tmp/gcc40-cparse-make.log",
      "diag-cparse-frontier: cparse_make_cparse_o_rc=$rc",
      "diag-cparse-frontier: cparse_make_cparse_o_tail",
      "ERROR: c-parse.o unexpectedly compiled",
      "exit \"$rc\""
    ],
    "non_claim": "diagnostic frontier evidence only; does not prove native GCC 4.0 compiler correctness"
  },
  "retirement_condition": {
    "replacement_evidence": "retire when GCC 4.0 native cc1 source build evidence supersedes the pass1 bridge boundary",
    "non_claim": "frontier-only receipt; does not prove native GCC 4.0 compiler correctness"
  },
  "parity_effect": "evidence-backed partial; does not prove native/full GCC 4.0 correctness"
}
"#
        .to_string();
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
            "make_autohost_define_undef6_probe autohost_defines_120_undef_need64_gid_inline_rlim_ssize_uid_system 120 NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t ssize_t uid_t\\n",
            "make_autohost_full_undef6_probe autohost_full_undef_need64_gid_inline_rlim_ssize_uid_system NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t ssize_t uid_t\\n",
            "try_cparse_variant cparse_config_undef6_system /tmp/cparse_config_undef6_system.c\\n",
            "make_include_probe_config_undef6 cparse_undef6_inc_system 'system.h'\\n",
            "make_include_probe_config_undef6 cparse_undef6_inc_coretypes 'system.h coretypes.h'\\n",
            "make_include_probe_config_undef6 cparse_undef6_inc_tm 'system.h coretypes.h tm.h'\\n",
            "make_insn_modes_manual_config_undef6 cparse_undef6_insn_modes_include_only '#include \"insn-modes.h\"'\\n",
            "make_insn_modes_header_prefix_config_undef6 cparse_undef6_insn_modes_lines_20 20\\n",
            "make_insn_modes_header_prefix_config_undef6 cparse_undef6_insn_modes_lines_40 40\\n",
            "make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_20 20\\n",
            "make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_40 40\\n",
            "make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_60 60\\n",
            "make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_80 80\\n",
            "make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_100 100\\n",
            "make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_120 120\\n",
            "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_28 28\\n",
            "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_36 36\\n",
            "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_80 80\\n",
            "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_120 120\\n",
            "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_160 160\\n",
            "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_166 166\\n",
            "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_177 177\\n",
            "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_180 180\\n",
            "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_207 207\\n",
            "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_212 212\\n",
            "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_220 220\\n",
            "make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_empty 'enum built_in_function {\\n",
            "make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_complex_arith_only\\n",
            "m=/tmp/gcc40-cparse-make.log\\n",
            "diag-cparse-frontier: cparse_make_cparse_o_rc=$rc\\n",
            "diag-cparse-frontier: cparse_make_cparse_o_tail\\n",
            "make: *** [c-parse.o] Error 1\\n",
            "ERROR: c-parse.o unexpectedly compiled\\n",
            "exit \"$rc\"\\n",
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
            "gcc40_cplus_demangle_long_arg_itanium_v5_boundary\n",
            "gcc40_cp_demangle_long_arg_itanium_v5_boundary\n",
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
            "gcc40_cplus_demangle_long_arg_itanium_v5_boundary\n",
            "gcc40_cp_demangle_long_arg_itanium_v5_boundary\n",
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
            "gcc40_cplus_demangle_long_arg_itanium_v5_boundary\n",
            "gcc40_cp_demangle_long_arg_itanium_v5_boundary\n",
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
            "gcc40_cplus_demangle_long_arg_itanium_v5_boundary\n",
            "gcc40_cp_demangle_long_arg_itanium_v5_boundary\n",
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
            "gcc40_cplus_demangle_long_arg_itanium_v5_boundary\n",
            "gcc40_cp_demangle_long_arg_itanium_v5_boundary\n",
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
            "gcc40_cplus_demangle_long_arg_itanium_v5_boundary\n",
            "gcc40_cp_demangle_long_arg_itanium_v5_boundary\n",
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
            "gcc40_cplus_demangle_long_arg_itanium_v5_boundary\n",
            "gcc40_cp_demangle_long_arg_itanium_v5_boundary\n",
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
        write_gcc40_native_generator_receipt(dir.path(), "\"genattrtab\", \"genoutput\"=>\"genattrtab\", \"genemit\"");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("selected generators missing `genoutput`"));
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
            "gcc40_cplus_demangle_long_arg_itanium_v5_boundary\n",
            "gcc40_cp_demangle_long_arg_itanium_v5_boundary\n",
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
            "mantle-gcc40-native-generator-slice-v2=>mantle-gcc40-native-generator-slice-v0",
        );

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("expected `mantle-gcc40-native-generator-slice-v2`"));
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
            "gcc40_cplus_demangle_long_arg_itanium_v5_boundary\n",
            "gcc40_cp_demangle_long_arg_itanium_v5_boundary\n",
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
        write_gcc40_native_demangle_receipt(dir.path(), "71504a07=>00000000");

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
        write_gcc40_native_demangle_receipt(dir.path(), "single-long-arg-itanium-v5=>operator-name-itanium-v1");

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("expected `single-long-arg-itanium-v5`"));
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
            "mantle-gcc40-native-demangle-slice-v5=>mantle-gcc40-native-demangle-slice-v0",
        );

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("expected `mantle-gcc40-native-demangle-slice-v5`"));
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
            "mantle-gcc40-native-cc1-source-frontier-reduction-v7=>mantle-gcc40-native-cc1-source-frontier-reduction-v6",
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
            "cparse_make_cparse_o_rc=2=>cparse_make_cparse_o_rc=1",
        );

        let row = evaluate_stage(dir.path(), &gcc40_spec());

        assert_eq!(row.status, StageStatus::Placeholder);
        assert!(row.notes.contains("observed_frontier missing required v7 fragment"), "{}", row.notes);
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
            "make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_220 220=>make_tree_header_balanced_config_undef6 missing_tree_lines_220",
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
    fn self_build_without_provider_kind_linkage_receipt_remains_partial() {
        let dir = tempdir().unwrap();
        write_stage(dir.path(), "crunch.ncl", "# real crunch derivation body\n");

        let row = evaluate_stage(dir.path(), &self_build_spec());

        assert_eq!(row.status, StageStatus::Partial);
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
        assert!(row.notes.contains("real self-build proof evidence accepted"));
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
            "mantle-deterministic-proof-receipt-v1=>mantle-deterministic-proof-receipt-v2",
        );

        let err = validate_real_self_build_proof_parity_evidence(dir.path()).unwrap_err();

        assert!(err.contains("workflow"));
        assert!(err.contains("mantle-deterministic-proof-receipt-v1"));
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
    fn self_build_real_derivation_reports_real_proof_backed_partial_without_completing_axes() {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let report = collect_bootstrap_parity_report(project_root);
        let row = report.rows.iter().find(|row| row.id == "crunch.self-build").unwrap();

        assert_eq!(row.status, StageStatus::Partial);
        assert_eq!(row.provider_kind, ProviderKind::SourceRoot);
        assert!(row.status.blocks_parity());
        assert!(row.proof_details.is_some());
        assert!(!row.notes.contains("provider-kind linkage receipt missing"));
        assert!(!row.notes.contains("evidence check failed"));
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
    fn stagex_lineage_scaffold_receipt_is_partial_not_complete() {
        let dir = tempdir().unwrap();
        write_stagex_lineage_receipt(
            dir.path(),
            "stagex-lineage",
            "scaffold-only",
            "1111111111111111111111111111111111111111111111111111111111111111",
            "[]",
        );

        let row = evaluate_stage(dir.path(), &stagex_lineage_spec());

        assert_eq!(row.status, StageStatus::Partial);
        assert_eq!(row.provider_kind, ProviderKind::Unknown);
        assert!(row.status.blocks_parity());
        assert!(!row.notes.contains("evidence check failed"));
    }

    #[test]
    fn stagex_lineage_receipt_rejects_source_root_provider() {
        let dir = tempdir().unwrap();
        write_stagex_lineage_receipt(
            dir.path(),
            "source-root",
            "scaffold-only",
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
        write_stagex_lineage_receipt(dir.path(), "stagex-lineage", "scaffold-only", "ABC", "[]");

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
            "scaffold-only",
            "1111111111111111111111111111111111111111111111111111111111111111",
            r#"["host-bwrap"]"#,
        );

        let err = validate_stagex_lineage_provider_receipt(dir.path()).unwrap_err();

        assert!(err.contains("fallback_events"));
        assert!(err.contains("must be empty"));
    }

    #[test]
    fn gcc40_real_derivation_reports_inventory_backed_partial() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));

        let row = evaluate_stage(root, &gcc40_spec());

        assert_eq!(row.status, StageStatus::Partial, "{}", row.notes);
        assert!(row.status.blocks_parity());
        assert!(!row.notes.contains("evidence check failed"));
        assert!(row.notes.contains("checked placeholder inventory"));
    }

    #[test]
    fn gcc40_real_derivation_contains_bcmp_semantic_smoke() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("int __gcc_bcmp(const unsigned char *lhs"));
        assert!(content.contains("if (__gcc_bcmp(a, b, 4) != 0) return 1;"));
        assert!(content.contains("if (__gcc_bcmp(a, c, 4) == 0) return 2;"));
        assert!(content.contains("if (__gcc_bcmp(a, c, 2) != 0) return 3;"));
        assert!(content.contains("ERROR: __gcc_bcmp semantic smoke failed"));
    }

    #[test]
    fn gcc40_real_derivation_contains_driver_query_smoke() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("-dumpversion)"));
        assert!(content.contains("echo '4.0.4'"));
        assert!(content.contains("-dumpmachine)"));
        assert!(content.contains("echo 'x86_64-unknown-linux-musl'"));
        assert!(content.contains("-print-libgcc-file-name)"));
        assert!(content.contains("driver_libgcc=$(\"$out/bin/gcc\" -print-libgcc-file-name)"));
        assert!(content.contains("ERROR: gcc -print-libgcc-file-name path missing"));
        assert!(content.contains("ERROR: gcc -print-search-dirs missing install dir"));
    }

    #[test]
    fn gcc40_real_derivation_contains_cc1_object_smoke() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("cc1 object boundary"));
        assert!(content.contains("exec \"$TCC/bin/tcc\" -c"));
        assert!(content.contains("-quiet /tmp/cc1-smoke.c -o /tmp/cc1-smoke.o"));
        assert!(content.contains("ERROR: cc1 object smoke failed"));
        assert!(content.contains("ERROR: cc1 object smoke output missing"));
    }

    #[test]
    fn gcc40_real_derivation_contains_generator_header_boundary_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("empty-machine constants boundary"));
        assert!(content.contains("empty-machine flags boundary"));
        assert!(content.contains("ERROR: insn-constants boundary guard missing"));
        assert!(content.contains("ERROR: insn-flags boundary guard missing"));
        assert!(content.contains("bootstrap genconstants s[t]ub"));
        assert!(content.contains("bootstrap genflags s[t]ub"));
        assert!(!content.contains("bootstrap genconstants stub: no md constants required"));
        assert!(!content.contains("bootstrap genflags stub: generator-only boundary bridge"));
    }

    #[test]
    fn gcc40_real_derivation_contains_gencheck_boundary_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("gcc40_gencheck_disabled_tree_check_boundary"));
        assert!(content.contains("disabled-tree-checking boundary"));
        assert!(content.contains("ERROR: gencheck boundary executable missing"));
        assert!(content.contains("ERROR: gencheck boundary guard missing"));
        assert!(content.contains("ERROR: gencheck boundary marker missing"));
        assert!(content.contains("bootstrap gencheck s[t]ub"));
        assert!(!content.contains("bootstrap gencheck stub"));
        assert!(!content.contains("gcc40_gencheck_bootstrap_stub"));
    }

    #[test]
    fn gcc40_real_derivation_contains_genpreds_boundary_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("gcc40_genpreds_empty_predicate_object_boundary"));
        assert!(content.contains("gcc40_genpreds_empty_predicate_source_boundary"));
        assert!(content.contains("empty-predicate header boundary"));
        assert!(content.contains("empty-predicate source boundary"));
        assert!(content.contains("Normalize this seam to the same checked"));
        assert!(content.contains("GENPREDS_BOUNDARY_SCRIPT"));
        assert!(content.contains("ERROR: genpreds boundary executable missing"));
        assert!(content.contains("ERROR: genpreds header boundary guard missing"));
        assert!(content.contains("ERROR: genpreds source boundary symbol missing"));
        assert!(content.contains("bootstrap genpreds s[t]ub"));
        assert!(!content.contains("bootstrap genpreds header stub"));
        assert!(!content.contains("bootstrap genpreds source stub"));
    }

    #[test]
    fn gcc40_real_derivation_contains_genattr_boundary_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("gcc40_genattr_empty_attribute_object_boundary"));
        assert!(content.contains("empty-attribute header boundary"));
        assert!(content.contains("ERROR: genattr boundary executable missing"));
        assert!(content.contains("ERROR: genattr boundary guard missing"));
        assert!(content.contains("ERROR: genattr enabled attribute boundary missing"));
        assert!(content.contains("bootstrap genattr s[t]ub"));
        assert!(!content.contains("gcc40_genattr_bootstrap_stub"));
        assert!(!content.contains("bootstrap genattr stub"));
    }

    #[test]
    fn gcc40_real_derivation_contains_genemit_boundary_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("gcc40_genemit_empty_emit_source_boundary"));
        assert!(content.contains("empty-emit source boundary"));
        assert!(content.contains("ERROR: genemit boundary executable missing"));
        assert!(content.contains("ERROR: genemit source boundary symbol missing"));
        assert!(content.contains("ERROR: genemit source boundary marker missing"));
        assert!(content.contains("genemit_bootstrap_s[t]ub"));
        assert!(!content.contains("void genemit_bootstrap_stub"));
    }

    #[test]
    fn gcc40_real_derivation_contains_genrecog_boundary_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("gcc40_genrecog_empty_recognition_source_boundary"));
        assert!(content.contains("empty-recognition source boundary"));
        assert!(content.contains("ERROR: genrecog boundary executable missing"));
        assert!(content.contains("ERROR: genrecog source boundary symbol missing"));
        assert!(content.contains("ERROR: genrecog source boundary marker missing"));
        assert!(content.contains("generated_bootstrap_s[t]ub"));
        assert!(!content.contains("genrecog|build/genrecog|*/build/genextract"));
    }

    #[test]
    fn gcc40_real_derivation_contains_genextract_boundary_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("gcc40_genextract_empty_extraction_source_boundary"));
        assert!(content.contains("empty-extraction source boundary"));
        assert!(content.contains("ERROR: genextract boundary executable missing"));
        assert!(content.contains("ERROR: genextract source boundary symbol missing"));
        assert!(content.contains("ERROR: genextract source boundary marker missing"));
        assert!(content.contains("generated_bootstrap_s[t]ub"));
        assert!(!content.contains("genextract|build/genextract|*/build/genpeep"));
    }

    #[test]
    fn gcc40_real_derivation_contains_genpeep_boundary_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("gcc40_genpeep_empty_peephole_source_boundary"));
        assert!(content.contains("empty-peephole source boundary"));
        assert!(content.contains("ERROR: genpeep boundary executable missing"));
        assert!(content.contains("ERROR: genpeep source boundary symbol missing"));
        assert!(content.contains("ERROR: genpeep source boundary marker missing"));
        assert!(content.contains("generated_bootstrap_s[t]ub"));
        assert!(!content.contains("genpeep|build/genpeep|*/build/genopinit"));
    }

    #[test]
    fn gcc40_real_derivation_contains_genopinit_boundary_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("gcc40_genopinit_empty_opinit_source_boundary"));
        assert!(content.contains("empty-opinit source boundary"));
        assert!(content.contains("ERROR: genopinit boundary executable missing"));
        assert!(content.contains("ERROR: genopinit source boundary symbol missing"));
        assert!(content.contains("ERROR: genopinit source boundary marker missing"));
        assert!(content.contains("generated_bootstrap_s[t]ub"));
        assert!(!content.contains("genopinit|build/genopinit|*/build/genoutput"));
    }

    #[test]
    fn gcc40_real_derivation_contains_genoutput_boundary_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("gcc40_genoutput_bounded_output_slice"));
        assert!(content.contains("native genoutput bounded output slice"));
        assert!(content.contains("ERROR: genoutput boundary executable missing"));
        assert!(content.contains("ERROR: genoutput bounded output symbol missing"));
        assert!(content.contains("ERROR: genoutput bounded output marker missing"));
        assert!(content.contains("ERROR: genoutput bounded output constant missing"));
        assert!(content.contains("generated_bootstrap_s[t]ub"));
        assert!(!content.contains("genoutput|build/genoutput|*/build/genattrtab"));
    }

    #[test]
    fn gcc40_real_derivation_contains_genattrtab_boundary_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("gcc40_genattrtab_bounded_output_slice"));
        assert!(content.contains("native genattrtab bounded output slice"));
        assert!(content.contains("ERROR: genattrtab boundary executable missing"));
        assert!(content.contains("ERROR: genattrtab bounded output symbol missing"));
        assert!(content.contains("ERROR: genattrtab bounded output marker missing"));
        assert!(content.contains("ERROR: genattrtab bounded enabled attribute missing"));
        assert!(content.contains("generated_bootstrap_s[t]ub"));
        assert!(!content.contains("void generated_bootstrap_stub(void) { }"));
    }

    #[test]
    fn gcc40_real_derivation_contains_demangle_semantic_boundary_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("gcc40_cplus_demangle_long_arg_itanium_v5_boundary"));
        assert!(content.contains("gcc40_cp_demangle_long_arg_itanium_v5_boundary"));
        assert!(content.contains("cplus_demangle bounded single-long argument semantic smoke"));
        assert!(content.contains("_Z3foov"));
        assert!(content.contains("foo()"));
        assert!(content.contains("_ZN3foo3barEv"));
        assert!(content.contains("foo::bar()"));
        assert!(content.contains("_ZN3foo3bar3bazEv"));
        assert!(content.contains("foo::bar::baz()"));
        assert!(content.contains("_Z3fooi"));
        assert!(content.contains("foo(int)"));
        assert!(content.contains("_ZN3foo3barEi"));
        assert!(content.contains("foo::bar(int)"));
        assert!(content.contains("_ZN3foo3bar3bazEc"));
        assert!(content.contains("foo::bar::baz(char)"));
        assert!(content.contains("_ZN3foo3bar3bazEi"));
        assert!(content.contains("foo::bar::baz(int)"));
        assert!(content.contains("_ZN3foo3bar3bazEf"));
        assert!(content.contains("_ZN3foo3bar3baz3quxEi"));
        assert!(content.contains("_ZN3foo3bar3baz3quxEv"));
        assert!(content.contains("bounded Itanium single-long argument function semantic slice"));
        assert!(!content.contains("gcc40_cp_demangle_disabled_boundary"));
        assert!(!content.contains("libiberty_cp_demangle_bootstrap_stub"));
    }

    #[test]
    fn gcc40_real_derivation_contains_gengtype_boundary_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content = fs::read_to_string(root.join("bootstrap/gcc-4.0.ncl")).unwrap();

        assert!(content.contains("empty-GTY header boundary"));
        assert!(content.contains("empty-GTY descriptor boundary"));
        assert!(content.contains("ERROR: gengtype boundary executable missing"));
        assert!(content.contains("ERROR: gengtype boundary header missing"));
        assert!(content.contains("ERROR: gengtype descriptor boundary marker missing"));
        assert!(content.contains("bootstrap gengtype s[t]ub"));
        assert!(!content.contains("bootstrap gengtype stub"));
    }

    #[test]
    fn stagex_lineage_real_receipt_reports_evidence_backed_partial() {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let report = collect_bootstrap_parity_report(project_root);
        let row = report.rows.iter().find(|row| row.id == "seed-full.stagex-lineage").unwrap();

        assert_eq!(row.status, StageStatus::Partial);
        assert_eq!(row.provider_kind, ProviderKind::Unknown);
        assert!(row.status.blocks_parity());
        assert!(!row.notes.contains("evidence check failed"));
        let stagex = report.axes.iter().find(|axis| axis.axis == ParityAxis::Stagex).unwrap();
        assert!(stagex.blocking_rows.contains(&"seed-full.stagex-lineage".to_string()));
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
