#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProofDisposition {
    Match,
    Divergence,
    UnexpectedOutput,
}

// machine-artifact-public: radiance.radiance-reference-proof-report
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProofReport {
    pub schema: String,
    pub disposition: ProofDisposition,
    pub receipt_blake3: String,
    pub source_bundle_blake3: String,
    pub source_cohort_blake3: String,
    pub plan_blake3: String,
    pub protected_execution_audit_blake3: String,
    pub fixed_point_blake3: String,
    pub fixed_point_bytes: u64,
    pub output_path: String,
    pub proof_time_network_requests: u32,
    pub proof_success: bool,
    pub non_claim: String,
}

pub(super) struct BuildInput<'a> {
    pub receipt: &'a crunch_radiance_reference_core::RadianceReferenceReceipt,
    pub source_bundle_blake3: &'a str,
    pub audit_blake3: &'a str,
    pub output: &'a std::path::Path,
    pub disposition: ProofDisposition,
}

pub(super) fn disposition(
    receipt: &crunch_radiance_reference_core::RadianceReferenceReceipt,
    profile: &crate::radiance::profile::Definition,
) -> ProofDisposition {
    if receipt.cross_route.disposition == crunch_radiance_reference_core::RadianceCrossRouteDisposition::Divergence {
        return ProofDisposition::Divergence;
    }
    let is_expected_output = receipt.cross_route.seed_blake3 == profile.expected_fixed_point_blake3
        && receipt.cross_route.seed_bytes == profile.expected_fixed_point_bytes;
    if is_expected_output {
        ProofDisposition::Match
    } else {
        ProofDisposition::UnexpectedOutput
    }
}

pub(super) fn build(input: BuildInput<'_>) -> Result<ProofReport, crate::errors::RunError> {
    let is_proof_success = input.disposition == ProofDisposition::Match;
    let proof_result = ProofReport {
        schema: "mantle-radiance-reference-proof-report-v1".to_string(),
        disposition: input.disposition,
        receipt_blake3: input.receipt.receipt_blake3.clone(),
        source_bundle_blake3: input.source_bundle_blake3.to_string(),
        source_cohort_blake3: input.receipt.source_cohort.cohort_blake3.clone(),
        plan_blake3: input.receipt.plan.plan_blake3.clone(),
        protected_execution_audit_blake3: input.audit_blake3.to_string(),
        fixed_point_blake3: input.receipt.cross_route.seed_blake3.clone(),
        fixed_point_bytes: input.receipt.cross_route.seed_bytes,
        output_path: input.output.display().to_string(),
        proof_time_network_requests: input.receipt.zero_events.live_fetches,
        proof_success: is_proof_success,
        non_claim: input.receipt.non_claim.clone(),
    };
    if !super::helpers::valid_blake3_hex(&proof_result.receipt_blake3) {
        return Err(crate::errors::RunError::Internal(
            "Radiance proof report has an invalid receipt identity".to_string(),
        ));
    }
    debug_assert!(!proof_result.output_path.is_empty());
    debug_assert_eq!(proof_result.proof_success, proof_result.disposition == ProofDisposition::Match);
    Ok(proof_result)
}

pub(in crate::radiance) fn print_prepare(
    report: &crate::radiance::source::PrepareReport,
    is_json: bool,
) -> Result<(), crate::errors::RunError> {
    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(report).map_err(|error| crate::errors::RunError::Internal(format!(
                "serializing Radiance prepare report: {error}"
            )))?
        );
    } else {
        println!("Prepared Radiance offline source bundle: {}", report.source_bundle_path);
        println!("Source bundle BLAKE3: {}", report.source_bundle_blake3);
        println!("Source cohort BLAKE3: {}", report.source_cohort_blake3);
        println!("Proof-time network required: {}", report.network_required_for_proof);
    }
    debug_assert_eq!(
        usize::try_from(report.source_count).ok(),
        Some(crunch_radiance_reference_core::RADIANCE_SOURCE_COUNT)
    );
    debug_assert!(!report.source_bundle_blake3.is_empty());
    Ok(())
}

pub(in crate::radiance) fn print_proof(report: &ProofReport, is_json: bool) -> Result<(), crate::errors::RunError> {
    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(report).map_err(|error| crate::errors::RunError::Internal(format!(
                "serializing Radiance proof report: {error}"
            )))?
        );
    } else {
        println!("Radiance reference disposition: {:?}", report.disposition);
        println!("Receipt BLAKE3: {}", report.receipt_blake3);
        println!("Fixed-point BLAKE3: {}", report.fixed_point_blake3);
        println!("Fixed-point bytes: {}", report.fixed_point_bytes);
        println!("Proof output: {}", report.output_path);
    }
    debug_assert!(!report.receipt_blake3.is_empty());
    debug_assert_eq!(report.proof_time_network_requests, 0);
    Ok(())
}
