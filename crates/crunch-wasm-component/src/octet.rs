use std::path::Path;

use crunch_wasm_component_core::Blake3Identity;
use crunch_wasm_component_core::BoundedComponentClaim;
use crunch_wasm_component_core::ComponentStageKind;
use crunch_wasm_component_core::ComponentStageStatus;
use crunch_wasm_component_core::OctetValidationBinding;
use crunch_wasm_component_core::StoreObject;
use crunch_wasm_component_core::ValidationDecision;
use serde::Deserialize;

use crate::Error;
use crate::OctetValidationEvidence;
use crate::files::write_new;
use crate::preflight::PreparedPipeline;
use crate::reporting::ExecutionState;
use crate::stages::StageWorkspace;
use crate::stages::invoke;
use crate::stages::publish_evidence_artifact;

const OCTET_RECEIPT_FILE: &str = "evidence-rail-receipt.json";
const OCTET_SUMMARY_FILE: &str = "summary.txt";
const OCTET_PROVENANCE_FILE: &str = "provenance.jsonl";
const OCTET_ARTIFACT_FILE: &str = "wasm-artifact.bin";
const OCTET_VERIFICATION_FILE: &str = "octet-artifact-verification.json";
const MAX_OCTET_RECEIPT_BYTES: u64 = 16 * 1024 * 1024;
const OCTET_RECEIPT_SCHEMA: &str = "octet-evidence-rail-receipt/v1";
const OCTET_RAIL_ID: &str = "wasm-artifact";
const OCTET_CHECKER_ID: &str = "cargo-octet";
const OCTET_HANDOFF_ROLE: &str = "mantle-exact-byte";
const OCTET_VERIFICATION_ROLE: &str = "recorded_only";
const OCTET_EXACT_IDENTITY_ROLE: &str = "exact-bytes";

pub(crate) struct OctetStageAdmission {
    pub binding: OctetValidationBinding,
    pub evidence: OctetValidationEvidence,
}

#[derive(Debug, Clone, Deserialize)]
struct OctetReceipt {
    schema_version: String,
    rail_id: String,
    checker_id: String,
    status: String,
    profile: Option<String>,
    diagnostics: Vec<String>,
    wasm_artifact: Option<OctetWasmArtifact>,
}

#[derive(Debug, Clone, Deserialize)]
struct OctetWasmArtifact {
    profile_id: String,
    profile_identity: String,
    registry_identity: String,
    cohort_identity: String,
    exact_artifact_identity: String,
    exact_identity_role: String,
    input_artifact: String,
    handoff_role: String,
    handoff_parent_identities: Vec<String>,
    verification_role: String,
}

#[derive(Debug, Clone, Deserialize)]
struct OctetVerificationReport {
    status: String,
    diagnostics: Vec<String>,
}

struct OctetExpected<'a> {
    artifact: &'a StoreObject,
    profile_id: &'a str,
    profile_blake3: &'a Blake3Identity,
    config_blake3: &'a Blake3Identity,
    cohort_blake3: &'a Blake3Identity,
    report_blake3: &'a Blake3Identity,
}

pub(crate) fn run_octet_validation(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    artifact_path: &Path,
    artifact: &StoreObject,
) -> Result<Option<OctetStageAdmission>, Error> {
    let evidence_root = workspace.root.join("octet-final");
    let config_path = prepared.toolchain.octet_config_path()?;
    let parent_identity = format!("mantle:{}", artifact.digest_blake3.clone().into_hex());
    let args = vec![
        "evidence".to_string(),
        "collect".to_string(),
        "--rail".to_string(),
        OCTET_RAIL_ID.to_string(),
        "--input".to_string(),
        artifact_path.display().to_string(),
        "--config".to_string(),
        config_path.display().to_string(),
        "--profile".to_string(),
        prepared.toolchain.manifest.octet.profile_id.clone(),
        "--parent-artifact".to_string(),
        parent_identity,
        "--artifact-dir".to_string(),
        evidence_root.display().to_string(),
        "--output-format".to_string(),
        "json".to_string(),
    ];
    let collect = invoke(prepared, workspace, "octet-validation", "cargo-octet", args, None)?;
    state.add_receipt(collect.receipt)?;
    let receipt_path = evidence_root.join(OCTET_RECEIPT_FILE);
    let receipt_bytes =
        crate::files::read_source_file_bounded(&receipt_path, MAX_OCTET_RECEIPT_BYTES, "Octet Wasm artifact receipt")?;
    let receipt: OctetReceipt = serde_json::from_slice(&receipt_bytes)
        .map_err(|error| Error::Invalid(format!("parsing exact Octet Wasm artifact receipt: {error}")))?;
    let verify = verify_octet_bundle(prepared, workspace, state, &evidence_root)?;
    let published = publish_octet_evidence(workspace, state, &evidence_root, &verify)?;
    let expected = OctetExpected {
        artifact,
        profile_id: &prepared.toolchain.manifest.octet.profile_id,
        profile_blake3: &prepared.toolchain.manifest.octet.profile_identity_blake3,
        config_blake3: &prepared.toolchain.manifest.octet.config_digest_blake3,
        cohort_blake3: &prepared.toolchain.manifest.octet.wasm_tools_cohort_identity_blake3,
        report_blake3: &published.receipt.digest_blake3,
    };
    let binding = validate_octet_receipt(&receipt, &expected, collect.success, verify.valid);
    let admitted = binding.is_ok();
    state.push_stage(
        "octet-validation",
        ComponentStageKind::OctetValidation,
        if admitted {
            ComponentStageStatus::Succeeded
        } else {
            ComponentStageStatus::Denied
        },
        Some(published.receipt.clone()),
        Some(prepared.toolchain.tool_digest("cargo-octet")?),
        Some(prepared.toolchain.manifest.octet.profile_identity_blake3.clone()),
        if admitted {
            vec![BoundedComponentClaim::OctetReportBound]
        } else {
            Vec::new()
        },
    )?;
    let binding = match binding {
        Ok(binding) => binding,
        Err(message) => {
            state.block("octet-report-not-admitted", "octet-validation", message);
            return Ok(None);
        }
    };
    let evidence = OctetValidationEvidence {
        stage_key: "octet-validation".to_string(),
        artifact: artifact.clone(),
        receipt: published.receipt,
        verification_report: published.verification,
        profile_blake3: prepared.toolchain.manifest.octet.profile_identity_blake3.clone(),
        cohort_blake3: prepared.toolchain.manifest.octet.wasm_tools_cohort_identity_blake3.clone(),
        source_repository: prepared.toolchain.manifest.octet.source_repository.clone(),
        source_revision: prepared.toolchain.manifest.octet.source_revision.clone(),
        package_name: prepared.toolchain.manifest.octet.package_name.clone(),
        package_version: prepared.toolchain.manifest.octet.package_version.clone(),
        config_blake3: prepared.toolchain.manifest.octet.config_digest_blake3.clone(),
        decision: "passed".to_string(),
    };
    state.add_octet_validation(evidence.clone())?;
    debug_assert_eq!(binding.decision, ValidationDecision::Pass);
    debug_assert_eq!(evidence.artifact.digest_blake3, binding.artifact_blake3);
    Ok(Some(OctetStageAdmission { binding, evidence }))
}

struct PublishedOctetEvidence {
    receipt: StoreObject,
    verification: StoreObject,
}

struct VerifiedOctetBundle {
    bytes: Vec<u8>,
    valid: bool,
}

fn verify_octet_bundle(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    evidence_root: &Path,
) -> Result<VerifiedOctetBundle, Error> {
    let args = vec![
        "artifact".to_string(),
        "verify".to_string(),
        "--artifact-dir".to_string(),
        evidence_root.display().to_string(),
        "--output-format".to_string(),
        "json".to_string(),
    ];
    let run = invoke(prepared, workspace, "octet-artifact-verification", "cargo-octet", args, None)?;
    state.add_receipt(run.receipt)?;
    let report: OctetVerificationReport = serde_json::from_slice(&run.stdout)
        .map_err(|error| Error::Invalid(format!("parsing Octet artifact verification report: {error}")))?;
    let valid = run.success && report.status == "valid" && report.diagnostics.is_empty();
    debug_assert_eq!(valid, run.success && report.status == "valid" && report.diagnostics.is_empty());
    debug_assert!(!run.stdout.is_empty());
    Ok(VerifiedOctetBundle {
        bytes: run.stdout,
        valid,
    })
}

fn publish_octet_evidence(
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    evidence_root: &Path,
    verification: &VerifiedOctetBundle,
) -> Result<PublishedOctetEvidence, Error> {
    let files = [
        (OCTET_RECEIPT_FILE, "octet-evidence-rail-receipt.json", "octet-receipt"),
        (OCTET_SUMMARY_FILE, "octet-summary.txt", "octet-summary"),
        (OCTET_PROVENANCE_FILE, "octet-provenance.jsonl", "octet-provenance"),
        (OCTET_ARTIFACT_FILE, "octet-wasm-artifact.bin", "octet-exact-artifact"),
    ];
    let mut receipt = None;
    for (source_name, published_name, role) in files {
        let object = publish_evidence_artifact(&evidence_root.join(source_name), published_name, workspace)?;
        state.add_artifact(role, &object)?;
        if source_name == OCTET_RECEIPT_FILE {
            receipt = Some(object);
        }
    }
    let verification_path = workspace.root.join(OCTET_VERIFICATION_FILE);
    write_new(&verification_path, &verification.bytes)?;
    let verification_object = publish_evidence_artifact(&verification_path, OCTET_VERIFICATION_FILE, workspace)?;
    state.add_artifact("octet-artifact-verification", &verification_object)?;
    let receipt = receipt.ok_or_else(|| Error::Invalid("Octet receipt publication was omitted".to_string()))?;
    debug_assert!(receipt.size_bytes > 0);
    debug_assert!(verification_object.size_bytes > 0);
    Ok(PublishedOctetEvidence {
        receipt,
        verification: verification_object,
    })
}

fn validate_octet_receipt(
    receipt: &OctetReceipt,
    expected: &OctetExpected<'_>,
    collect_success: bool,
    verification_valid: bool,
) -> Result<OctetValidationBinding, String> {
    let details = receipt
        .wasm_artifact
        .as_ref()
        .ok_or_else(|| "Octet receipt omitted wasm_artifact details".to_string())?;
    let artifact_blake3 = parse_prefixed_blake3(&details.exact_artifact_identity, "exact artifact")?;
    let profile_blake3 = parse_prefixed_blake3(&details.profile_identity, "profile")?;
    let config_blake3 = parse_prefixed_blake3(&details.registry_identity, "configuration")?;
    let cohort_blake3 = parse_prefixed_blake3(&details.cohort_identity, "cohort")?;
    let parent = format!("mantle:{}", expected.artifact.digest_blake3.clone().into_hex());
    let identity_matches = artifact_blake3 == expected.artifact.digest_blake3
        && profile_blake3 == *expected.profile_blake3
        && config_blake3 == *expected.config_blake3
        && cohort_blake3 == *expected.cohort_blake3;
    let contract_matches = receipt.schema_version == OCTET_RECEIPT_SCHEMA
        && receipt.rail_id == OCTET_RAIL_ID
        && receipt.checker_id == OCTET_CHECKER_ID
        && receipt.profile.as_deref() == Some(expected.profile_id)
        && details.profile_id == expected.profile_id
        && details.exact_identity_role == OCTET_EXACT_IDENTITY_ROLE
        && details.input_artifact == OCTET_ARTIFACT_FILE
        && details.handoff_role == OCTET_HANDOFF_ROLE
        && details.handoff_parent_identities == vec![parent]
        && details.verification_role == OCTET_VERIFICATION_ROLE;
    let decision_passed =
        collect_success && verification_valid && receipt.status == "passed" && receipt.diagnostics.is_empty();
    if !identity_matches || !contract_matches || !decision_passed {
        return Err(
            "Octet independently returned a non-pass, unverifiable, or identity-mismatched report; exact Octet evidence was retained without reinterpreting its findings"
                .to_string(),
        );
    }
    debug_assert!(identity_matches);
    debug_assert!(contract_matches);
    Ok(OctetValidationBinding {
        artifact_blake3,
        profile_blake3,
        cohort_blake3,
        report_blake3: expected.report_blake3.clone(),
        decision: ValidationDecision::Pass,
    })
}

fn parse_prefixed_blake3(value: &str, label: &str) -> Result<Blake3Identity, String> {
    let hex = value
        .strip_prefix("b3:")
        .ok_or_else(|| format!("Octet {label} identity lacks the b3: role prefix"))?;
    Blake3Identity::parse(hex.to_string()).map_err(|error| format!("invalid Octet {label} BLAKE3 identity: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OCTET_PROFILE_ID;

    fn digest(seed: u8) -> Blake3Identity {
        Blake3Identity::from_slice(&[seed])
    }

    fn expected_object() -> StoreObject {
        StoreObject {
            logical_path: "/mantle/store/component.wasm".to_string(),
            digest_blake3: digest(1),
            size_bytes: 1,
        }
    }

    fn receipt_for(expected: &OctetExpected<'_>) -> OctetReceipt {
        let prefixed = |identity: &Blake3Identity| format!("b3:{}", identity.clone().into_hex());
        OctetReceipt {
            schema_version: OCTET_RECEIPT_SCHEMA.to_string(),
            rail_id: OCTET_RAIL_ID.to_string(),
            checker_id: OCTET_CHECKER_ID.to_string(),
            status: "passed".to_string(),
            profile: Some(expected.profile_id.to_string()),
            diagnostics: Vec::new(),
            wasm_artifact: Some(OctetWasmArtifact {
                profile_id: expected.profile_id.to_string(),
                profile_identity: prefixed(expected.profile_blake3),
                registry_identity: prefixed(expected.config_blake3),
                cohort_identity: prefixed(expected.cohort_blake3),
                exact_artifact_identity: prefixed(&expected.artifact.digest_blake3),
                exact_identity_role: OCTET_EXACT_IDENTITY_ROLE.to_string(),
                input_artifact: OCTET_ARTIFACT_FILE.to_string(),
                handoff_role: OCTET_HANDOFF_ROLE.to_string(),
                handoff_parent_identities: vec![format!(
                    "mantle:{}",
                    expected.artifact.digest_blake3.clone().into_hex()
                )],
                verification_role: OCTET_VERIFICATION_ROLE.to_string(),
            }),
        }
    }

    #[test]
    fn exact_octet_receipt_is_bound_without_reinterpreting_findings() {
        let artifact = expected_object();
        let profile = digest(2);
        let config = digest(3);
        let cohort = digest(4);
        let expected = OctetExpected {
            artifact: &artifact,
            profile_id: OCTET_PROFILE_ID,
            profile_blake3: &profile,
            config_blake3: &config,
            cohort_blake3: &cohort,
            report_blake3: &config,
        };
        let result = validate_octet_receipt(&receipt_for(&expected), &expected, true, true).unwrap();

        assert_eq!(result.artifact_blake3, artifact.digest_blake3);
        assert_eq!(result.profile_blake3, profile);
        assert_eq!(result.decision, ValidationDecision::Pass);
    }

    #[test]
    fn mismatched_or_non_pass_octet_receipt_is_denied() {
        let artifact = expected_object();
        let profile = digest(2);
        let config = digest(3);
        let cohort = digest(4);
        let expected = OctetExpected {
            artifact: &artifact,
            profile_id: OCTET_PROFILE_ID,
            profile_blake3: &profile,
            config_blake3: &config,
            cohort_blake3: &cohort,
            report_blake3: &config,
        };
        let mut mismatched = receipt_for(&expected);
        mismatched.wasm_artifact.as_mut().unwrap().exact_artifact_identity = format!("b3:{}", digest(9).into_hex());
        let mismatch = validate_octet_receipt(&mismatched, &expected, true, true);
        let non_pass = validate_octet_receipt(&receipt_for(&expected), &expected, false, true);

        assert!(mismatch.is_err());
        assert!(non_pass.is_err());
    }
}
