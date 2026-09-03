#[derive(Debug, Clone)]
pub struct RemoteCandidateProjection {
    pub command: crunch_remote_core::RemoteCommand,
    pub configured: bool,
    pub credential_present: bool,
    pub capabilities_match: bool,
    pub source_inputs_ready: bool,
    pub output_trusted: bool,
    pub requires_network: bool,
    pub upload_classes: Vec<crunch_build_planning_core::UploadClass>,
    pub upload_object_count: u32,
    pub upload_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteProjectionError {
    RemoteIdentity(crunch_remote_core::RemoteCoreError),
    UploadSummary(crunch_build_planning_core::RoutePlanError),
}

pub fn project_remote_candidate(
    projection: RemoteCandidateProjection,
) -> Result<crunch_build_planning_core::RemoteCandidateObservation, RemoteProjectionError> {
    let candidate_identity_blake3 = crunch_remote_core::remote_command_identity(projection.command)
        .map_err(RemoteProjectionError::RemoteIdentity)?;
    let upload_summary = crunch_build_planning_core::UploadSummary::try_new(
        projection.upload_classes,
        projection.upload_object_count,
        projection.upload_bytes,
    )
    .map_err(RemoteProjectionError::UploadSummary)?;
    debug_assert_eq!(candidate_identity_blake3.len(), crunch_remote_core::BLAKE3_HEX_LENGTH);
    debug_assert!(upload_summary.object_count <= projection.upload_object_count);
    Ok(crunch_build_planning_core::RemoteCandidateObservation {
        candidate_identity_blake3,
        configured: projection.configured,
        credential_present: projection.credential_present,
        capabilities_match: projection.capabilities_match,
        source_inputs_ready: projection.source_inputs_ready,
        output_trusted: projection.output_trusted,
        requires_network: projection.requires_network,
        upload_summary,
    })
}
