use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;

pub const RECEIPT_BUNDLE_FORMAT: &str = "mantle-build-receipt-bundle-v1";
pub const RECEIPT_BUNDLE_VERSION: u32 = 1;
pub const RECEIPT_BUNDLE_NON_CLAIM: &str =
    "receipt bundle evidence proves only matched receipt, attestation, graph, and trust-basis facts";
pub const MAX_RECEIPT_RECORDS: usize = 65_536;
pub const MAX_RECEIPT_FIELD_BYTES: usize = 4_096;

const RECORD_SPEC_SEPARATOR: char = ':';
const RECEIPT_STATE_DIR: &str = "receipt-bundles";
const RECEIPT_RECORDS_DIR: &str = "records";
const TEMP_FILE_EXTENSION: &str = "tmp";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReceiptRecordKind {
    SourceRef,
    ActionRef,
    SandboxReport,
    NetworkPolicy,
    ReferenceScan,
    OutputRef,
    ArtifactAttestation,
    ClosureAttestation,
    SemanticGraphEdge,
    TrustBasis,
    Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClaimStrengthRequest {
    Diagnostic,
    StrongActionCorrectness,
    ReleaseFacing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptRecord {
    pub kind: ReceiptRecordKind,
    pub identity: String,
    pub digest: String,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustSnapshot {
    pub policy_hash: String,
    pub public_key_digests: Vec<String>,
    pub revocation_ref: Option<String>,
    pub valid_after_unix_s: u64,
    pub valid_before_unix_s: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptBundle {
    pub format: String,
    pub version: u32,
    pub store_prefix: String,
    pub claim_strength: ClaimStrengthRequest,
    pub trust_snapshot: TrustSnapshot,
    pub records: Vec<ReceiptRecord>,
    pub bundle_blake3: String,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReceiptBundleReport {
    pub format: &'static str,
    pub store_prefix: String,
    pub claim_strength: ClaimStrengthRequest,
    pub evidence_complete: bool,
    pub missing_evidence: Vec<ReceiptRecordKind>,
    pub record_count: u32,
    pub bundle_blake3: String,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReceiptImportReport {
    pub imported: bool,
    pub idempotent: bool,
    pub bundle_blake3: String,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptRecordSpec {
    pub kind: ReceiptRecordKind,
    pub identity: String,
    pub digest: String,
}

impl std::str::FromStr for ReceiptRecordKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "source-ref" => Ok(Self::SourceRef),
            "action-ref" => Ok(Self::ActionRef),
            "sandbox-report" => Ok(Self::SandboxReport),
            "network-policy" => Ok(Self::NetworkPolicy),
            "reference-scan" => Ok(Self::ReferenceScan),
            "output-ref" => Ok(Self::OutputRef),
            "artifact-attestation" => Ok(Self::ArtifactAttestation),
            "closure-attestation" => Ok(Self::ClosureAttestation),
            "semantic-graph-edge" => Ok(Self::SemanticGraphEdge),
            "trust-basis" => Ok(Self::TrustBasis),
            "signature" => Ok(Self::Signature),
            other => Err(format!("unsupported receipt record kind '{other}'")),
        }
    }
}

pub fn parse_receipt_record_spec(raw: &str) -> Result<ReceiptRecordSpec, RunError> {
    let parts = raw.splitn(3, RECORD_SPEC_SEPARATOR).collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(RunError::Internal(format!("receipt record spec must be kind:identity:digest, got '{raw}'")));
    }
    let kind = parts[0].parse::<ReceiptRecordKind>().map_err(RunError::Internal)?;
    validate_field("identity", parts[1])?;
    validate_digest(parts[2])?;
    Ok(ReceiptRecordSpec {
        kind,
        identity: parts[1].to_string(),
        digest: parts[2].to_string(),
    })
}

pub fn build_receipt_bundle(
    specs: &[ReceiptRecordSpec],
    store_prefix: &str,
    policy_hash: &str,
    claim_strength: ClaimStrengthRequest,
) -> Result<ReceiptBundle, RunError> {
    if specs.is_empty() {
        return Err(RunError::Internal("receipt bundle requires at least one --record".to_string()));
    }
    if specs.len() > MAX_RECEIPT_RECORDS {
        return Err(RunError::Internal(format!("receipt record count exceeds {MAX_RECEIPT_RECORDS}")));
    }
    validate_field("policy_hash", policy_hash)?;
    if !store_prefix.starts_with('/') {
        return Err(RunError::Internal(format!("store prefix must be absolute: {store_prefix}")));
    }
    let mut records = specs.iter().map(record_from_spec).collect::<Vec<_>>();
    records.sort_by(|left, right| record_key(left).cmp(&record_key(right)));
    reject_duplicate_records(&records)?;
    let mut bundle = ReceiptBundle {
        format: RECEIPT_BUNDLE_FORMAT.to_string(),
        version: RECEIPT_BUNDLE_VERSION,
        store_prefix: store_prefix.to_string(),
        claim_strength,
        trust_snapshot: TrustSnapshot {
            policy_hash: policy_hash.to_string(),
            public_key_digests: Vec::new(),
            revocation_ref: None,
            valid_after_unix_s: 0,
            valid_before_unix_s: u64::MAX,
        },
        records,
        bundle_blake3: String::new(),
        non_claim: RECEIPT_BUNDLE_NON_CLAIM.to_string(),
    };
    bundle.bundle_blake3 = digest_bundle_without_digest(&bundle)?;
    Ok(bundle)
}

pub fn validate_receipt_bundle(bundle: &ReceiptBundle) -> Result<ReceiptBundleReport, RunError> {
    if bundle.format != RECEIPT_BUNDLE_FORMAT {
        return Err(RunError::Internal(format!("unsupported receipt bundle format {}", bundle.format)));
    }
    if bundle.version != RECEIPT_BUNDLE_VERSION {
        return Err(RunError::Internal(format!("unsupported receipt bundle version {}", bundle.version)));
    }
    if !bundle.store_prefix.starts_with('/') {
        return Err(RunError::Internal(format!(
            "receipt bundle store prefix is not absolute: {}",
            bundle.store_prefix
        )));
    }
    reject_duplicate_records(&bundle.records)?;
    let expected = digest_bundle_without_digest(bundle)?;
    if expected != bundle.bundle_blake3 {
        return Err(RunError::Internal("receipt bundle digest mismatch".to_string()));
    }
    let missing_evidence = classify_missing_evidence(bundle);
    Ok(ReceiptBundleReport {
        format: RECEIPT_BUNDLE_FORMAT,
        store_prefix: bundle.store_prefix.clone(),
        claim_strength: bundle.claim_strength.clone(),
        evidence_complete: missing_evidence.is_empty(),
        missing_evidence,
        record_count: checked_u32(bundle.records.len(), "receipt record count")?,
        bundle_blake3: bundle.bundle_blake3.clone(),
        non_claim: RECEIPT_BUNDLE_NON_CLAIM,
    })
}

pub fn write_receipt_bundle(path: &Path, bundle: &ReceiptBundle) -> Result<(), RunError> {
    validate_receipt_bundle(bundle)?;
    let rendered = serde_json::to_string_pretty(bundle)
        .map_err(|err| RunError::Internal(format!("serializing receipt bundle: {err}")))?;
    fs::write(path, format!("{rendered}\n"))
        .map_err(|err| RunError::Internal(format!("writing receipt bundle {}: {err}", path.display())))
}

pub fn read_receipt_bundle(path: &Path) -> Result<ReceiptBundle, RunError> {
    let bytes = fs::read(path)
        .map_err(|err| RunError::Internal(format!("reading receipt bundle {}: {err}", path.display())))?;
    let bundle = serde_json::from_slice::<ReceiptBundle>(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing receipt bundle {}: {err}", path.display())))?;
    validate_receipt_bundle(&bundle)?;
    Ok(bundle)
}

pub fn import_receipt_bundle(bundle: &ReceiptBundle, state_dir: &Path) -> Result<ReceiptImportReport, RunError> {
    validate_receipt_bundle(bundle)?;
    let records_dir = state_dir.join(RECEIPT_STATE_DIR).join(RECEIPT_RECORDS_DIR);
    fs::create_dir_all(&records_dir)
        .map_err(|err| RunError::Internal(format!("creating receipt records dir {}: {err}", records_dir.display())))?;
    let target = records_dir.join(format!("{}.json", bundle.bundle_blake3));
    if target.exists() {
        let existing = read_receipt_bundle(&target)?;
        if &existing != bundle {
            return Err(RunError::Internal(format!(
                "conflicting receipt bundle already exists for {}",
                bundle.bundle_blake3
            )));
        }
        return Ok(ReceiptImportReport {
            imported: false,
            idempotent: true,
            bundle_blake3: bundle.bundle_blake3.clone(),
            non_claim: RECEIPT_BUNDLE_NON_CLAIM,
        });
    }
    let tmp = target.with_extension(TEMP_FILE_EXTENSION);
    write_receipt_bundle(&tmp, bundle)?;
    fs::rename(&tmp, &target)
        .map_err(|err| RunError::Internal(format!("committing receipt bundle {}: {err}", target.display())))?;
    Ok(ReceiptImportReport {
        imported: true,
        idempotent: false,
        bundle_blake3: bundle.bundle_blake3.clone(),
        non_claim: RECEIPT_BUNDLE_NON_CLAIM,
    })
}

fn record_from_spec(spec: &ReceiptRecordSpec) -> ReceiptRecord {
    ReceiptRecord {
        kind: spec.kind.clone(),
        identity: spec.identity.clone(),
        digest: spec.digest.clone(),
        metadata: BTreeMap::new(),
    }
}

fn classify_missing_evidence(bundle: &ReceiptBundle) -> Vec<ReceiptRecordKind> {
    if bundle.claim_strength == ClaimStrengthRequest::Diagnostic {
        return Vec::new();
    }
    let present = bundle.records.iter().map(|record| record.kind.clone()).collect::<BTreeSet<_>>();
    let required = [
        ReceiptRecordKind::SourceRef,
        ReceiptRecordKind::ActionRef,
        ReceiptRecordKind::SandboxReport,
        ReceiptRecordKind::NetworkPolicy,
        ReceiptRecordKind::ReferenceScan,
        ReceiptRecordKind::OutputRef,
        ReceiptRecordKind::ArtifactAttestation,
        ReceiptRecordKind::TrustBasis,
        ReceiptRecordKind::Signature,
    ];
    required.iter().filter(|kind| !present.contains(*kind)).cloned().collect::<Vec<_>>()
}

fn reject_duplicate_records(records: &[ReceiptRecord]) -> Result<(), RunError> {
    let mut seen = BTreeSet::new();
    for record in records {
        validate_field("identity", &record.identity)?;
        validate_digest(&record.digest)?;
        let key = record_key(record);
        if !seen.insert(key.clone()) {
            return Err(RunError::Internal(format!("duplicate receipt record {key}")));
        }
    }
    Ok(())
}

fn record_key(record: &ReceiptRecord) -> String {
    format!("{:?}:{}", record.kind, record.identity)
}

fn digest_bundle_without_digest(bundle: &ReceiptBundle) -> Result<String, RunError> {
    let mut clone = bundle.clone();
    clone.bundle_blake3.clear();
    let encoded = serde_json::to_vec(&clone)
        .map_err(|err| RunError::Internal(format!("serializing receipt bundle for digest: {err}")))?;
    Ok(blake3::hash(&encoded).to_hex().to_string())
}

fn validate_field(label: &str, value: &str) -> Result<(), RunError> {
    if value.is_empty() || value.len() > MAX_RECEIPT_FIELD_BYTES || value.contains('\0') || value.contains("..") {
        return Err(RunError::Internal(format!("invalid receipt {label}: '{value}'")));
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<(), RunError> {
    validate_field("digest", value)?;
    let digest_like = value.starts_with("blake3:") || value.starts_with("sha256:") || value.len() == 64;
    if !digest_like {
        return Err(RunError::Internal(format!("receipt digest must be blake3:/sha256:/hex, got '{value}'")));
    }
    Ok(())
}

fn checked_u32(count: usize, label: &str) -> Result<u32, RunError> {
    u32::try_from(count).map_err(|_| RunError::Internal(format!("{label} does not fit in u32: {count}")))
}

pub fn render_json(value: &impl Serialize) -> Result<String, RunError> {
    serde_json::to_string_pretty(value)
        .map_err(|err| RunError::Internal(format!("serializing receipt bundle report: {err}")))
}

pub fn cmd_receipt(
    action: crate::ReceiptAction,
    state_dir: &Path,
    store_prefix: &str,
    json_output: bool,
) -> Result<(), RunError> {
    match action {
        crate::ReceiptAction::Bundle { action } => cmd_receipt_bundle(action, state_dir, store_prefix, json_output),
    }
}

fn cmd_receipt_bundle(
    action: crate::ReceiptBundleAction,
    state_dir: &Path,
    store_prefix: &str,
    json_output: bool,
) -> Result<(), RunError> {
    match action {
        crate::ReceiptBundleAction::Export {
            records,
            to,
            policy_hash,
            strong,
        } => {
            let bundle = build_from_cli_records(&records, store_prefix, &policy_hash, strong)?;
            write_receipt_bundle(&to, &bundle)?;
            print_report(&validate_receipt_bundle(&bundle)?, json_output)
        }
        crate::ReceiptBundleAction::List { from } | crate::ReceiptBundleAction::Verify { from } => {
            let bundle = read_receipt_bundle(&from)?;
            print_report(&validate_receipt_bundle(&bundle)?, json_output)
        }
        crate::ReceiptBundleAction::Import { from } => {
            let bundle = read_receipt_bundle(&from)?;
            let report = import_receipt_bundle(&bundle, state_dir)?;
            print_import(&report, json_output)
        }
    }
}

fn build_from_cli_records(
    records: &[String],
    store_prefix: &str,
    policy_hash: &str,
    strong: bool,
) -> Result<ReceiptBundle, RunError> {
    let specs = records.iter().map(|record| parse_receipt_record_spec(record)).collect::<Result<Vec<_>, _>>()?;
    let claim = if strong {
        ClaimStrengthRequest::StrongActionCorrectness
    } else {
        ClaimStrengthRequest::Diagnostic
    };
    build_receipt_bundle(&specs, store_prefix, policy_hash, claim)
}

fn print_report(report: &ReceiptBundleReport, json_output: bool) -> Result<(), RunError> {
    if json_output {
        println!("{}", render_json(report)?);
        return Ok(());
    }
    println!(
        "format={} store_prefix={} records={} complete={} bundle_blake3={}",
        report.format, report.store_prefix, report.record_count, report.evidence_complete, report.bundle_blake3
    );
    if !report.missing_evidence.is_empty() {
        println!("missing_evidence={:?}", report.missing_evidence);
    }
    eprintln!("non_claim={}", report.non_claim);
    Ok(())
}

fn print_import(report: &ReceiptImportReport, json_output: bool) -> Result<(), RunError> {
    if json_output {
        println!("{}", render_json(report)?);
        return Ok(());
    }
    println!(
        "RECEIPT_IMPORT bundle_blake3={} imported={} idempotent={}",
        report.bundle_blake3, report.imported, report.idempotent
    );
    eprintln!("non_claim={}", report.non_claim);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_specs() -> Vec<ReceiptRecordSpec> {
        [
            ReceiptRecordKind::SourceRef,
            ReceiptRecordKind::ActionRef,
            ReceiptRecordKind::SandboxReport,
            ReceiptRecordKind::NetworkPolicy,
            ReceiptRecordKind::ReferenceScan,
            ReceiptRecordKind::OutputRef,
            ReceiptRecordKind::ArtifactAttestation,
            ReceiptRecordKind::TrustBasis,
            ReceiptRecordKind::Signature,
        ]
        .iter()
        .enumerate()
        .map(|(index, kind)| ReceiptRecordSpec {
            kind: kind.clone(),
            identity: format!("record-{index}"),
            digest: format!("blake3:{:064x}", index + 1),
        })
        .collect()
    }

    #[test]
    fn complete_bundle_supports_strong_claim_classification() {
        let bundle = build_receipt_bundle(
            &complete_specs(),
            "/mantle/store",
            "policy-v1",
            ClaimStrengthRequest::StrongActionCorrectness,
        )
        .unwrap();
        let report = validate_receipt_bundle(&bundle).unwrap();
        assert!(report.evidence_complete);
        assert!(report.missing_evidence.is_empty());
    }

    #[test]
    fn partial_bundle_is_diagnostic_for_strong_claim() {
        let specs = vec![ReceiptRecordSpec {
            kind: ReceiptRecordKind::OutputRef,
            identity: "out".to_string(),
            digest: "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        }];
        let bundle =
            build_receipt_bundle(&specs, "/mantle/store", "policy-v1", ClaimStrengthRequest::StrongActionCorrectness)
                .unwrap();
        let report = validate_receipt_bundle(&bundle).unwrap();
        assert!(!report.evidence_complete);
        assert!(report.missing_evidence.contains(&ReceiptRecordKind::SourceRef));
    }

    #[test]
    fn duplicate_receipt_records_fail_closed() {
        let specs = vec![
            ReceiptRecordSpec {
                kind: ReceiptRecordKind::OutputRef,
                identity: "out".to_string(),
                digest: "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            },
            ReceiptRecordSpec {
                kind: ReceiptRecordKind::OutputRef,
                identity: "out".to_string(),
                digest: "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            },
        ];
        let err =
            build_receipt_bundle(&specs, "/mantle/store", "policy-v1", ClaimStrengthRequest::Diagnostic).unwrap_err();
        assert!(err.to_string().contains("duplicate receipt record"));
    }

    #[test]
    fn receipt_import_is_idempotent_and_conflict_checked() {
        let temp = tempfile::tempdir().unwrap();
        let bundle =
            build_receipt_bundle(&complete_specs(), "/mantle/store", "policy-v1", ClaimStrengthRequest::Diagnostic)
                .unwrap();
        let first = import_receipt_bundle(&bundle, temp.path()).unwrap();
        assert!(first.imported);
        let second = import_receipt_bundle(&bundle, temp.path()).unwrap();
        assert!(second.idempotent);
    }

    #[test]
    fn malformed_record_spec_is_rejected() {
        let err = parse_receipt_record_spec("output-ref:only-two-parts").unwrap_err();
        assert!(err.to_string().contains("kind:identity:digest"));
    }
}
