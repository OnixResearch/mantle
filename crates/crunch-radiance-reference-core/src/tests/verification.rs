use alloc::string::ToString;

#[test]
fn matching_routes_produce_an_admitted_receipt() {
    let receipt = super::receipt(super::DIGEST_8);
    assert_eq!(receipt.cross_route.disposition, crate::RadianceCrossRouteDisposition::Match);
    assert!(receipt.route_convergence.iter().all(|comparison| comparison.equal));
    assert_eq!(receipt.publication.len(), crate::RADIANCE_PUBLICATION_COUNT);
    assert!(crate::validate_radiance_reference_receipt(&receipt).is_ok());
}

#[test]
fn link_runtime_tool_observations_are_required_and_bounded() {
    let complete = super::receipt(super::DIGEST_8);
    let mut missing = complete.clone();
    missing.tools.retain(|tool| tool.role != crate::RadianceArtifactRole::HostCrtInputs);
    let mut receipt_with_oversized_runtime_bytes = complete.clone();
    let runtime = receipt_with_oversized_runtime_bytes
        .tools
        .iter_mut()
        .find(|tool| tool.role == crate::RadianceArtifactRole::HostLibgccInputs)
        .unwrap();
    runtime.byte_count = crate::RADIANCE_LINK_RUNTIME_BYTES_MAX.saturating_add(1);
    assert!(complete.tools.iter().any(|tool| tool.role == crate::RadianceArtifactRole::HostCompilerDriver));
    assert_eq!(crate::validate_radiance_reference_receipt(&missing), Err(crate::RadianceReferenceError::Tool));
    assert_eq!(
        crate::validate_radiance_reference_receipt(&receipt_with_oversized_runtime_bytes),
        Err(crate::RadianceReferenceError::Tool)
    );
}

#[test]
fn divergent_route_fixed_points_remain_valid_bounded_evidence() {
    let receipt = super::receipt(super::DIGEST_9);
    assert_eq!(receipt.cross_route.disposition, crate::RadianceCrossRouteDisposition::Divergence);
    assert!(receipt.route_convergence.iter().all(|comparison| comparison.equal));
    assert_ne!(receipt.cross_route.seed_blake3, receipt.cross_route.c99_blake3);
    assert!(crate::validate_radiance_reference_receipt(&receipt).is_ok());
}

#[test]
fn route_nonconvergence_rejects_receipt() {
    let mut receipt = super::receipt(super::DIGEST_8);
    receipt.stages[5].output_blake3 = super::DIGEST_9.to_string();
    receipt.receipt_blake3.clear();
    assert_eq!(
        crate::validate_radiance_reference_receipt(&receipt),
        Err(crate::RadianceReferenceError::Convergence)
    );
    assert_ne!(receipt.stages[4].output_blake3, receipt.stages[5].output_blake3);
}

#[test]
fn predecessor_or_launcher_substitution_rejects_lineage() {
    let mut predecessor = super::receipt(super::DIGEST_8);
    predecessor.stages[1].predecessor_blake3 = super::DIGEST_5.to_string();
    let mut launcher = super::receipt(super::DIGEST_8);
    launcher.stages[3].launcher_blake3 = super::DIGEST_4.to_string();
    assert_eq!(
        crate::validate_radiance_reference_receipt(&predecessor),
        Err(crate::RadianceReferenceError::Lineage)
    );
    assert_eq!(crate::validate_radiance_reference_receipt(&launcher), Err(crate::RadianceReferenceError::Lineage));
}

#[test]
fn denied_execution_and_nonzero_exit_reject_receipt() {
    let mut denied = super::receipt(super::DIGEST_8);
    denied.stages[0].denied_event_count = 1;
    let mut failed = super::receipt(super::DIGEST_8);
    failed.stages[0].exit_code = 1;
    assert_eq!(crate::validate_radiance_reference_receipt(&denied), Err(crate::RadianceReferenceError::Execution));
    assert_eq!(crate::validate_radiance_reference_receipt(&failed), Err(crate::RadianceReferenceError::Execution));
}

#[test]
fn live_fetch_fallback_substitution_and_ambient_discovery_reject() {
    let mut cases = [
        super::receipt(super::DIGEST_8),
        super::receipt(super::DIGEST_8),
        super::receipt(super::DIGEST_8),
        super::receipt(super::DIGEST_8),
    ];
    cases[0].zero_events.live_fetches = 1;
    cases[1].zero_events.source_fallbacks = 1;
    cases[2].zero_events.substitutions = 1;
    cases[3].zero_events.ambient_discoveries = 1;
    assert!(cases.iter().all(|candidate| {
        crate::validate_radiance_reference_receipt(candidate) == Err(crate::RadianceReferenceError::ZeroEvents)
    }));
    assert_eq!(cases.len(), 4);
}

#[test]
fn publication_path_digest_and_bytes_are_exact() {
    let mut path = super::receipt(super::DIGEST_8);
    path.publication[0].relative_path = "../escape".to_string();
    let mut digest = super::receipt(super::DIGEST_8);
    digest.publication[0].digest_blake3 = super::DIGEST_9.to_string();
    let mut bytes = super::receipt(super::DIGEST_8);
    bytes.publication[0].byte_count = super::ARTIFACT_BYTES.saturating_add(1);
    assert_eq!(crate::validate_radiance_reference_receipt(&path), Err(crate::RadianceReferenceError::Publication));
    assert_eq!(crate::validate_radiance_reference_receipt(&digest), Err(crate::RadianceReferenceError::Publication));
    assert_eq!(crate::validate_radiance_reference_receipt(&bytes), Err(crate::RadianceReferenceError::Publication));
}

#[test]
fn receipt_identity_is_tamper_evident() {
    let mut candidate = super::receipt(super::DIGEST_8);
    candidate.receipt_blake3 = super::DIGEST_1.to_string();
    assert_eq!(
        crate::validate_radiance_reference_receipt(&candidate),
        Err(crate::RadianceReferenceError::Canonicalization)
    );
    assert_ne!(candidate.receipt_blake3, super::receipt(super::DIGEST_8).receipt_blake3);
}

#[test]
fn correctness_and_seed_trust_fields_cannot_enter_wire_receipt() {
    let baseline = serde_json::to_value(super::receipt(super::DIGEST_8)).unwrap();
    let forbidden_fields = ["compiler_correctness", "seed_trusted"];
    for field in forbidden_fields {
        let mut value = baseline.clone();
        value.as_object_mut().unwrap().insert(field.to_string(), serde_json::json!(true));
        let error = serde_json::from_value::<crate::RadianceReferenceReceipt>(value).unwrap_err();
        assert!(error.to_string().contains("unknown field"));
        assert!(error.to_string().contains(field));
    }
}
