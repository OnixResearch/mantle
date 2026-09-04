use alloc::string::ToString;

const REFERENCE_RECEIPT_DOMAIN: &[u8] = b"mantle.radiance-reference.receipt.v1";

pub fn seal_radiance_reference_receipt(
    draft: crate::RadianceReferenceReceiptDraft,
) -> Result<crate::RadianceReferenceReceipt, crate::RadianceReferenceError> {
    let mut receipt = crate::RadianceReferenceReceipt {
        schema: crate::RADIANCE_REFERENCE_RECEIPT_SCHEMA.to_string(),
        source_cohort: draft.source_cohort,
        source_bundle_blake3: draft.source_bundle_blake3,
        source_state_blake3: draft.source_state_blake3,
        plan: draft.plan,
        tools: draft.tools,
        stages: draft.stages,
        route_convergence: draft.route_convergence,
        cross_route: draft.cross_route,
        zero_events: draft.zero_events,
        publication: draft.publication,
        protected_execution_audit_blake3: draft.protected_execution_audit_blake3,
        receipt_blake3: alloc::string::String::new(),
        non_claim: crate::RADIANCE_REFERENCE_NON_CLAIM.to_string(),
    };
    validate_fields(&receipt)?;
    receipt.receipt_blake3 = identity(&receipt)?;
    validate_radiance_reference_receipt(&receipt)?;
    debug_assert!(crate::valid_blake3(&receipt.receipt_blake3));
    debug_assert_eq!(receipt.stages.len(), crate::RADIANCE_STAGE_COUNT);
    Ok(receipt)
}

pub fn validate_radiance_reference_receipt(
    receipt: &crate::RadianceReferenceReceipt,
) -> Result<(), crate::RadianceReferenceError> {
    if receipt.schema != crate::RADIANCE_REFERENCE_RECEIPT_SCHEMA {
        return Err(crate::RadianceReferenceError::Schema);
    }
    if receipt.non_claim != crate::RADIANCE_REFERENCE_NON_CLAIM {
        return Err(crate::RadianceReferenceError::NonClaim);
    }
    validate_fields(receipt)?;
    let expected = identity(receipt)?;
    if receipt.receipt_blake3 != expected {
        return Err(crate::RadianceReferenceError::Canonicalization);
    }
    debug_assert!(crate::valid_blake3(&receipt.receipt_blake3));
    debug_assert_eq!(receipt.route_convergence.len(), crate::RADIANCE_ROUTE_COUNT);
    Ok(())
}

fn validate_fields(receipt: &crate::RadianceReferenceReceipt) -> Result<(), crate::RadianceReferenceError> {
    crate::validate_source_cohort(receipt.source_cohort.clone())?;
    crate::validate_radiance_reference_plan(&receipt.plan)?;
    validate_identities(receipt)?;
    let tools = super::lineage::validate_tools(&receipt.tools)?;
    let stages = super::lineage::validate_stages(receipt, &tools)?;
    super::outcomes::validate_route_convergence(receipt, &stages)?;
    super::outcomes::validate_cross_route(receipt, &stages)?;
    super::outcomes::validate_zero_events(&receipt.zero_events)?;
    super::outcomes::validate_publication(&receipt.publication, &tools, &stages)?;
    debug_assert_eq!(receipt.source_cohort.cohort_blake3, receipt.plan.source_cohort_blake3);
    debug_assert_eq!(receipt.source_bundle_blake3, receipt.plan.source_bundle_blake3);
    Ok(())
}

fn validate_identities(receipt: &crate::RadianceReferenceReceipt) -> Result<(), crate::RadianceReferenceError> {
    let digests = [
        receipt.source_bundle_blake3.as_str(),
        receipt.source_state_blake3.as_str(),
        receipt.protected_execution_audit_blake3.as_str(),
    ];
    if digests.iter().any(|value| !crate::valid_blake3(value)) {
        return Err(crate::RadianceReferenceError::Digest);
    }
    if receipt.source_cohort.cohort_blake3 != receipt.plan.source_cohort_blake3 {
        return Err(crate::RadianceReferenceError::SourceIdentity);
    }
    if receipt.source_bundle_blake3 != receipt.plan.source_bundle_blake3 {
        return Err(crate::RadianceReferenceError::Plan);
    }
    debug_assert_eq!(digests.len(), 3);
    debug_assert!(digests.iter().all(|value| crate::valid_blake3(value)));
    Ok(())
}

fn identity(receipt: &crate::RadianceReferenceReceipt) -> Result<alloc::string::String, crate::RadianceReferenceError> {
    let mut candidate = receipt.clone();
    candidate.receipt_blake3.clear();
    let digest = crate::canonical_blake3(REFERENCE_RECEIPT_DOMAIN, &candidate)?;
    debug_assert!(crate::valid_blake3(&digest));
    debug_assert!(candidate.receipt_blake3.is_empty());
    Ok(digest)
}
