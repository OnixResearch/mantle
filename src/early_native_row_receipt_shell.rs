use std::collections::BTreeMap;
use std::fs;
use std::path::Component;
use std::path::Path;

use crunch_attestation::ArtifactAttestation;
use crunch_attestation::Canonicalize;
use serde::Deserialize;

use crate::early_native_row_receipt::EarlyNativeRowReceipt;
use crate::early_native_row_receipt::FallbackRecord;
use crate::early_native_row_receipt::ObservedAcceptance;
use crate::early_native_row_receipt::ObservedArtifact;
use crate::early_native_row_receipt::ObservedPredecessor;
use crate::early_native_row_receipt::RowExpectation;
use crate::early_native_row_receipt::TrustRecord;
use crate::early_native_row_receipt::parse_receipt;
use crate::early_native_row_receipt::validate_receipt;

const ARTIFACT_EVIDENCE_SCHEMA: &str = "mantle-early-native-artifact-evidence-v1";
const PREDECESSOR_ARTIFACT_KIND: &str = "artifact";
const ACCEPTANCE_EVIDENCE_SCHEMA: &str = "mantle-early-native-row-acceptance-v1";
const BINUTILS_RECEIPT_PATH: &str = "bootstrap/evidence/early-native-binutils-row-v2.json";
const BINUTILS_ARTIFACT_EVIDENCE_PATH: &str = "bootstrap/evidence/early-native-binutils-artifact.json";
const BINUTILS_ACCEPTANCE_EVIDENCE_PATH: &str = "bootstrap/evidence/early-native-binutils-acceptance-v1.json";
const BINUTILS_SUCCESS_MARKER: &str = "source-built TCC-era binutils matrix passed";
const GCC40_RECEIPT_PATH: &str = "bootstrap/evidence/early-native-gcc40-row-v2.json";
const GCC40_ARTIFACT_EVIDENCE_PATH: &str = "bootstrap/evidence/early-native-gcc40-artifact.json";
const GCC40_ACCEPTANCE_EVIDENCE_PATH: &str = "bootstrap/evidence/early-native-gcc40-acceptance-v1.json";
const GCC40_SUCCESS_MARKER: &str = "GCC 4.0.4 regenerated C/C++ early-native row behavior matrix passed";
const NAR_SHA256_PREFIX: &str = "nar-sha256:";

const BINUTILS_SOURCES: &[&str] = &[
    "bootstrap/binutils-tcc.ncl",
    "bootstrap/bison-2.3-musl.ncl",
    "bootstrap/diffutils-2.7-musl.ncl",
    "bootstrap/flex-2.6.4-musl.ncl",
    "bootstrap/gawk-3.0.4-musl.ncl",
    "bootstrap/m4-1.4.7-musl.ncl",
    "bootstrap/musl-1.1.24-native.ncl",
    "bootstrap/tcc-musl-native.ncl",
    "bootstrap/tcc-musl-v2.ncl",
];
const BINUTILS_PREDECESSORS: &[&str] = &[
    "bison-2.3",
    "flex-2.6.4",
    "gawk-3.0.4",
    "m4-1.4.7",
    "native-musl",
    "tcc-v2",
];
const BINUTILS_PREDECESSOR_EVIDENCE: &[&str] = &[
    "bootstrap/evidence/early-native-predecessors/bison-2.3.json",
    "bootstrap/evidence/early-native-predecessors/flex-2.6.4.json",
    "bootstrap/evidence/early-native-predecessors/gawk-3.0.4.json",
    "bootstrap/evidence/early-native-predecessors/m4-1.4.7.json",
    "bootstrap/evidence/early-native-predecessors/native-musl.json",
    "bootstrap/evidence/early-native-predecessors/tcc-v2.json",
];
const BINUTILS_GENERATED: &[&str] = &[
    "bfd-two-pass-headers",
    "binutils-parser-scanner-set",
    "gas-parser-set",
    "ld-parser-scanner-set",
];
const BINUTILS_POSITIVE: &[&str] = &[
    "archive-roundtrip",
    "assemble-object",
    "link-execute",
    "nm-symbols",
    "objcopy-copy",
    "object-format-readelf",
    "ranlib-index",
    "relocation-link",
];
const BINUTILS_REJECTION: &[&str] = &[
    "malformed-archive-rejected",
    "malformed-assembly-rejected",
    "malformed-link-input-rejected",
    "malformed-object-rejected",
    "missing-archive-member-rejected",
];

const GCC40_SOURCES: &[&str] = &[
    "bootstrap/gcc-4.0.ncl",
    "bootstrap/gcc-4.0-musl-cxx.ncl",
    "bootstrap/gcc-4.0-musl-pass3.ncl",
    "bootstrap/gcc-4.0-musl-pass4.ncl",
    "bootstrap/gcc-4.0-musl-pass5.ncl",
    "bootstrap/gcc-4.0-musl-pass6.ncl",
    "bootstrap/gcc-4.0-native.ncl",
    "bootstrap/musl-1.1.24-gcc.ncl",
    "bootstrap/musl-1.1.24-gcc-pass4.ncl",
    "bootstrap/musl-1.1.24-gcc-pass5.ncl",
    "bootstrap/binutils-2.30-gcc.ncl",
    "bootstrap/binutils-tcc.ncl",
    "bootstrap/bison-3.4.1-gcc.ncl",
    "bootstrap/flex-2.6.4-gcc.ncl",
    "bootstrap/m4-1.4.7-gcc.ncl",
    "bootstrap/autoconf-2.13-gcc.ncl",
    "bootstrap/autoconf-2.59-gcc.ncl",
    "bootstrap/perl-5.6.2-configured-gcc.ncl",
];
const GCC40_PREDECESSORS: &[&str] = &[
    "early-binutils",
    "gcc-built-binutils",
    "gcc-pass6",
    "musl-pass5",
    "native-musl",
    "native-tcc",
];
const GCC40_PREDECESSOR_EVIDENCE: &[&str] = &[
    "bootstrap/evidence/early-native-predecessors/early-binutils.json",
    "bootstrap/evidence/early-native-predecessors/gcc-built-binutils.json",
    "bootstrap/evidence/early-native-predecessors/gcc-pass6.json",
    "bootstrap/evidence/early-native-predecessors/musl-pass5.json",
    "bootstrap/evidence/early-native-predecessors/native-musl.json",
    "bootstrap/evidence/early-native-predecessors/native-tcc.json",
];
const GCC40_GENERATED: &[&str] = &[
    "c-parser-bison-3.4.1",
    "gcc-configure-autoconf-2.59",
    "gengtype-parser-bison-3.4.1",
    "gengtype-scanner-flex-2.6.4",
    "top-level-configure-autoconf-2.13",
];
const GCC40_POSITIVE: &[&str] = &[
    "c-driver-static-runtime",
    "cc1-and-cc1plus-present",
    "cxx-exception-rtti-allocation-runtime",
    "demangler-runtime",
    "generator-artifact-set",
    "libgcc-exception-runtime-present",
    "relocated-c-driver-runtime",
];
const GCC40_REJECTION: &[&str] = &[
    "configure-bridge-compiler-use-scan-clean",
    "fabricated-object-scan-clean",
    "malformed-c-rejected",
    "malformed-cxx-rejected",
    "release-generated-substitution-scan-clean",
    "wrapper-delegation-scan-clean",
];
const REQUIRED_NON_CLAIMS: &[&str] = &[
    "general compiler or binutils correctness",
    "provider or seed admission",
    "whole-bootstrap correctness",
];

#[derive(Debug, Deserialize)]
struct ArtifactEvidenceEnvelope {
    schema: String,
    row_id: String,
    digest: String,
    attestation: ArtifactAttestation,
}

#[derive(Debug, Deserialize)]
struct PredecessorArtifactEnvelope {
    kind: String,
    digest: String,
    attestation: ArtifactAttestation,
}

#[derive(Debug, Deserialize)]
struct AcceptanceEvidenceEnvelope {
    schema: String,
    row_id: String,
    logical_path: String,
    success_marker: String,
    positive: Vec<String>,
    rejection: Vec<String>,
    trust: TrustRecord,
    fallback: FallbackRecord,
}

pub(crate) fn validate_binutils_row(project_root: &Path) -> Result<(), String> {
    validate_row(project_root, BINUTILS_RECEIPT_PATH, &binutils_expectation())
}

pub(crate) fn validate_gcc40_row(project_root: &Path) -> Result<(), String> {
    validate_row(project_root, GCC40_RECEIPT_PATH, &gcc40_expectation())
}

fn validate_row(project_root: &Path, receipt_path: &str, expectation: &RowExpectation) -> Result<(), String> {
    let receipt_text = read_relative(project_root, receipt_path, "early-native row receipt")?;
    let receipt = parse_receipt(&receipt_text)?;
    require_expected_artifact_path(&receipt, expectation)?;
    let observed_sources = observe_sources(project_root, &receipt, expectation)?;
    let observed_predecessors = observe_predecessors(project_root, expectation)?;
    let observed_acceptance = observe_acceptance(project_root, expectation)?;
    let observed_artifact = observe_artifact(project_root, &receipt.output.artifact_evidence_path)?;
    validate_receipt(
        &receipt,
        expectation,
        &observed_sources,
        &observed_predecessors,
        &observed_acceptance,
        &observed_artifact,
    )?;
    assert_eq!(receipt.row_id, expectation.row_id);
    assert_eq!(observed_sources.len(), expectation.source_paths.len());
    assert_eq!(observed_predecessors.len(), expectation.predecessor_roles.len());
    assert_eq!(observed_acceptance.row_id, expectation.row_id);
    Ok(())
}

fn require_expected_artifact_path(receipt: &EarlyNativeRowReceipt, expectation: &RowExpectation) -> Result<(), String> {
    if receipt.output.artifact_evidence_path != expectation.artifact_evidence_path {
        return Err(format!(
            "artifact evidence path is `{}`, expected `{}`",
            receipt.output.artifact_evidence_path, expectation.artifact_evidence_path
        ));
    }
    validate_relative_path(&receipt.output.artifact_evidence_path)?;
    assert_eq!(receipt.output.artifact_evidence_path, expectation.artifact_evidence_path);
    Ok(())
}

fn observe_sources(
    project_root: &Path,
    receipt: &EarlyNativeRowReceipt,
    expectation: &RowExpectation,
) -> Result<BTreeMap<String, String>, String> {
    let expected = expectation.source_paths.iter().copied().collect::<std::collections::BTreeSet<_>>();
    let actual = receipt
        .source_records
        .iter()
        .map(|record| record.path.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if actual != expected {
        return Err("receipt source paths do not match the row contract".to_string());
    }
    let mut observed = BTreeMap::new();
    for source_path in expectation.source_paths {
        let bytes = read_relative_bytes(project_root, source_path, "row source")?;
        observed.insert((*source_path).to_string(), blake3::hash(&bytes).to_hex().to_string());
    }
    assert_eq!(observed.len(), expectation.source_paths.len());
    assert!(expectation.source_paths.iter().all(|path| observed.contains_key(*path)));
    Ok(observed)
}

fn observe_predecessors(
    project_root: &Path,
    expectation: &RowExpectation,
) -> Result<BTreeMap<String, ObservedPredecessor>, String> {
    if expectation.predecessor_roles.len() != expectation.predecessor_evidence_paths.len() {
        return Err("predecessor role and evidence path contracts have different lengths".to_string());
    }
    let mut observed = BTreeMap::new();
    for (role, path) in expectation.predecessor_roles.iter().zip(expectation.predecessor_evidence_paths) {
        let text = read_relative(project_root, path, "predecessor artifact evidence")?;
        let envelope: PredecessorArtifactEnvelope =
            serde_json::from_str(&text).map_err(|error| format!("parse `{path}`: {error}"))?;
        if envelope.kind != PREDECESSOR_ARTIFACT_KIND {
            return Err(format!(
                "predecessor `{role}` artifact kind is `{}`, expected `{PREDECESSOR_ARTIFACT_KIND}`",
                envelope.kind
            ));
        }
        let computed_digest = envelope
            .attestation
            .canonical_digest()
            .map_err(|error| format!("canonicalize predecessor evidence `{path}`: {error}"))?
            .to_hex();
        if computed_digest != envelope.digest {
            return Err(format!(
                "predecessor `{role}` artifact evidence digest mismatch: recorded `{}`, computed `{computed_digest}`",
                envelope.digest
            ));
        }
        let predecessor = ObservedPredecessor {
            role: (*role).to_string(),
            logical_path: envelope.attestation.facts.logical_path,
            artifact_attestation_blake3: computed_digest,
        };
        if observed.insert((*role).to_string(), predecessor).is_some() {
            return Err(format!("duplicate observed predecessor role `{role}`"));
        }
    }
    assert_eq!(observed.len(), expectation.predecessor_roles.len());
    assert!(observed.values().all(|item| item.logical_path.starts_with("/mantle/store/")));
    Ok(observed)
}

fn observe_acceptance(project_root: &Path, expectation: &RowExpectation) -> Result<ObservedAcceptance, String> {
    let path = expectation.acceptance_evidence_path;
    let text = read_relative(project_root, path, "early-native acceptance evidence")?;
    let envelope: AcceptanceEvidenceEnvelope =
        serde_json::from_str(&text).map_err(|error| format!("parse `{path}`: {error}"))?;
    if envelope.schema != ACCEPTANCE_EVIDENCE_SCHEMA {
        return Err(format!(
            "acceptance evidence schema is `{}`, expected `{ACCEPTANCE_EVIDENCE_SCHEMA}`",
            envelope.schema
        ));
    }
    let positive = envelope.positive.into_iter().collect::<std::collections::BTreeSet<_>>();
    let rejection = envelope.rejection.into_iter().collect::<std::collections::BTreeSet<_>>();
    if positive.len() != expectation.positive_matrix.len() {
        return Err("acceptance positive matrix contains duplicates or omissions".to_string());
    }
    if rejection.len() != expectation.rejection_matrix.len() {
        return Err("acceptance rejection matrix contains duplicates or omissions".to_string());
    }
    validate_final_source_policy(project_root, expectation)?;
    assert!(!positive.is_empty());
    assert!(!rejection.is_empty());
    Ok(ObservedAcceptance {
        row_id: envelope.row_id,
        logical_path: envelope.logical_path,
        success_marker: envelope.success_marker,
        positive,
        rejection,
        trust: envelope.trust,
        fallback: envelope.fallback,
    })
}

fn validate_final_source_policy(project_root: &Path, expectation: &RowExpectation) -> Result<(), String> {
    let (path, required, forbidden): (&str, &[&str], &[&str]) = match expectation.row_id {
        "binutils.tcc" => (
            "bootstrap/binutils-tcc.ncl",
            &[
                "source-built TCC-era binutils matrix passed",
                "CONFIGURE_PROBE_INVOCATION_COUNT_MAX=4096",
                "CONFIGURE_PROBE_SOURCE_BYTES_MAX=65536",
                "delegates to predecessor TinyCC",
            ],
            &["command -v ", "/usr/bin/", "exec \"$TCC/bin/tcc\"", "NON_ADMISSION"],
        ),
        "gcc.4.0" => (
            "bootstrap/gcc-4.0-musl-cxx.ncl",
            &[
                "early-native-gcc40-row.txt",
                "accepted malformed C",
                "accepted malformed C++",
                "for generator in genattrtab genoutput genemit genrecog genextract gengtype",
            ],
            &["$STAGE0/bin:$PATH", "exec \"$TCC/bin/tcc\"", "NON_ADMISSION"],
        ),
        other => return Err(format!("no final-source policy exists for early-native row `{other}`")),
    };
    let source = read_relative(project_root, path, "early-native final source policy input")?;
    for marker in required {
        if !source.contains(marker) {
            return Err(format!("final source `{path}` is missing required policy marker `{marker}`"));
        }
    }
    for marker in forbidden {
        if source.contains(marker) {
            return Err(format!("final source `{path}` contains forbidden policy marker `{marker}`"));
        }
    }
    assert!(required.iter().all(|marker| source.contains(marker)));
    assert!(forbidden.iter().all(|marker| !source.contains(marker)));
    Ok(())
}

fn observe_artifact(project_root: &Path, path: &str) -> Result<ObservedArtifact, String> {
    let text = read_relative(project_root, path, "early-native artifact evidence")?;
    let envelope: ArtifactEvidenceEnvelope =
        serde_json::from_str(&text).map_err(|error| format!("parse `{path}`: {error}"))?;
    if envelope.schema != ARTIFACT_EVIDENCE_SCHEMA {
        return Err(format!(
            "artifact evidence schema is `{}`, expected `{ARTIFACT_EVIDENCE_SCHEMA}`",
            envelope.schema
        ));
    }
    let computed_digest = envelope
        .attestation
        .canonical_digest()
        .map_err(|error| format!("canonicalize artifact evidence `{path}`: {error}"))?
        .to_hex();
    if computed_digest != envelope.digest {
        return Err(format!(
            "artifact evidence digest mismatch: recorded `{}`, computed `{computed_digest}`",
            envelope.digest
        ));
    }
    let nar_sha256 = envelope
        .attestation
        .facts
        .content_digest
        .strip_prefix(NAR_SHA256_PREFIX)
        .ok_or_else(|| format!("artifact evidence content digest must start with `{NAR_SHA256_PREFIX}`"))?
        .to_string();
    assert_eq!(computed_digest, envelope.digest);
    assert!(envelope.attestation.facts.logical_path.starts_with("/mantle/store/"));
    Ok(ObservedArtifact {
        row_id: envelope.row_id,
        logical_path: envelope.attestation.facts.logical_path,
        artifact_attestation_blake3: computed_digest,
        nar_sha256,
    })
}

fn read_relative(project_root: &Path, path: &str, label: &str) -> Result<String, String> {
    let bytes = read_relative_bytes(project_root, path, label)?;
    String::from_utf8(bytes).map_err(|error| format!("{label} `{path}` is not UTF-8: {error}"))
}

fn read_relative_bytes(project_root: &Path, path: &str, label: &str) -> Result<Vec<u8>, String> {
    validate_relative_path(path)?;
    let full_path = project_root.join(path);
    let bytes = fs::read(&full_path).map_err(|error| format!("read {label} `{}`: {error}", full_path.display()))?;
    if bytes.is_empty() {
        return Err(format!("{label} `{}` must not be empty", full_path.display()));
    }
    assert!(!bytes.is_empty());
    assert!(full_path.starts_with(project_root));
    Ok(bytes)
}

fn validate_relative_path(path: &str) -> Result<(), String> {
    let candidate = Path::new(path);
    let invalid = candidate.is_absolute()
        || candidate
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)));
    if invalid || path.trim().is_empty() {
        return Err(format!("evidence path `{path}` must be a non-empty project-relative path without traversal"));
    }
    assert!(!candidate.is_absolute());
    assert!(!path.trim().is_empty());
    Ok(())
}

fn binutils_expectation() -> RowExpectation {
    RowExpectation {
        row_id: "binutils.tcc",
        derivation: "bootstrap/binutils-tcc.ncl",
        artifact_evidence_path: BINUTILS_ARTIFACT_EVIDENCE_PATH,
        acceptance_evidence_path: BINUTILS_ACCEPTANCE_EVIDENCE_PATH,
        success_marker: BINUTILS_SUCCESS_MARKER,
        source_paths: BINUTILS_SOURCES,
        predecessor_roles: BINUTILS_PREDECESSORS,
        predecessor_evidence_paths: BINUTILS_PREDECESSOR_EVIDENCE,
        generated_artifacts: BINUTILS_GENERATED,
        positive_matrix: BINUTILS_POSITIVE,
        rejection_matrix: BINUTILS_REJECTION,
        required_non_claims: REQUIRED_NON_CLAIMS,
    }
}

fn gcc40_expectation() -> RowExpectation {
    RowExpectation {
        row_id: "gcc.4.0",
        derivation: "bootstrap/gcc-4.0.ncl",
        artifact_evidence_path: GCC40_ARTIFACT_EVIDENCE_PATH,
        acceptance_evidence_path: GCC40_ACCEPTANCE_EVIDENCE_PATH,
        success_marker: GCC40_SUCCESS_MARKER,
        source_paths: GCC40_SOURCES,
        predecessor_roles: GCC40_PREDECESSORS,
        predecessor_evidence_paths: GCC40_PREDECESSOR_EVIDENCE,
        generated_artifacts: GCC40_GENERATED,
        positive_matrix: GCC40_POSITIVE,
        rejection_matrix: GCC40_REJECTION,
        required_non_claims: REQUIRED_NON_CLAIMS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_contracts_have_distinct_receipts_and_artifacts() {
        let binutils = binutils_expectation();
        let gcc40 = gcc40_expectation();
        assert_ne!(BINUTILS_RECEIPT_PATH, GCC40_RECEIPT_PATH);
        assert_ne!(binutils.artifact_evidence_path, gcc40.artifact_evidence_path);
        assert_ne!(binutils.row_id, gcc40.row_id);
    }

    #[test]
    fn traversal_paths_are_rejected() {
        let error = validate_relative_path("bootstrap/../outside").unwrap_err();
        assert!(error.contains("without traversal"));
    }
}
