//! Host-side bridge for remote application observations. Fenced execution keeps
//! using crunch_remote_app and its existing core effect sessions.

pub use mantle_application_contract::remote_execution::RemoteOperation;
pub use mantle_application_contract::remote_execution::RemoteOperationFacts;
pub use mantle_application_contract::remote_execution::RemoteOperationPlan;

pub fn plan(operation: RemoteOperation) -> Result<RemoteOperationPlan, crate::RunError> {
    mantle_application_contract::remote_execution::plan_remote_operation(operation)
        .map_err(|error| crate::RunError::Internal(format!("remote effect plan: {}", error.code())))
}

pub fn classify(plan: &RemoteOperationPlan, facts: RemoteOperationFacts<'_>) -> Result<(), crate::RunError> {
    match mantle_application_contract::remote_execution::classify_remote_operation(plan, facts) {
        mantle_application_contract::ApplicationOutcome::Completed => Ok(()),
        outcome => Err(crate::RunError::Internal(format!("{}: {outcome:?}", plan.operation.effect_id()))),
    }
}

/// Read-back is done while the caller still holds the ticket mutation lock.
/// Compare every durable record, including revocation, verifier and limits.
pub fn classify_ticket_state(
    plan: &RemoteOperationPlan,
    expected: &crate::remote_credentials::RemoteTicketState,
    observed: &crate::remote_credentials::RemoteTicketState,
) -> Result<(), crate::RunError> {
    let mismatched_records =
        expected.tickets.iter().filter(|(id, record)| observed.tickets.get(*id) != Some(*record)).count()
            + usize::from(expected.schema_version != observed.schema_version)
            + usize::from(expected.invalidated_legacy_ticket_ids != observed.invalidated_legacy_ticket_ids);
    classify(plan, RemoteOperationFacts::TicketState {
        expected_sequence: expected.next_ticket_sequence,
        observed_sequence: observed.next_ticket_sequence,
        expected_records: expected.tickets.len(),
        observed_records: observed.tickets.len(),
        mismatched_records,
    })
}

pub fn query_readiness(
    socket: &std::path::Path,
) -> Result<super::live_state::daemon::ReadinessQueryReport, crate::RunError> {
    let plan = plan(RemoteOperation::ReadinessQuery)?;
    let report = super::live_state::daemon::query_readiness(socket)
        .map_err(|error| crate::RunError::Internal(format!("querying remote readiness: {error}")))?;
    let invalid_components = report
        .components
        .iter()
        .enumerate()
        .filter(|(index, row)| {
            row.id.is_empty()
                || row.generation == Some(0)
                || (row.ready && !row.blocked_by.is_empty())
                || report.components[..*index].iter().any(|prior| prior.id == row.id)
        })
        .count();
    classify(&plan, RemoteOperationFacts::Readiness {
        schema: &report.schema,
        expected_schema: crunch_service_readiness_core::READINESS_SCHEMA,
        classification: &report.classification,
        expected_classification: crunch_service_readiness_core::COORDINATION_CLASSIFICATION,
        evidence_eligible: report.evidence_eligible,
        invalid_components,
    })?;
    Ok(report)
}

pub fn classify_fixture_response(
    plan: &RemoteOperationPlan,
    request: &super::ConcreteBuildRequest,
    builder_key: &String,
    response: &super::RemoteBuilderFrameResponse,
) -> Result<(), crate::RunError> {
    let admission =
        super::validate_remote_builder_response_output_import(request, std::slice::from_ref(builder_key), response)
            .map_err(crate::RunError::Internal)?;
    classify(plan, RemoteOperationFacts::Outputs {
        expected: request.expected_outputs.len(),
        admitted: admission.outputs.len(),
        unevidenced: admission
            .outputs
            .iter()
            .filter(|output| {
                output.logical_path.is_empty()
                    || output.path_info_signing_key_id.is_empty()
                    || !super::is_blake3_hex_digest(&output.artifact_attestation_digest_blake3)
            })
            .count(),
    })
}
