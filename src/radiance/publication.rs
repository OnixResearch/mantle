const SEED_FIXED_POINT_PATH: &str = "artifacts/seed-route-fixed-point.rv64";
const C99_FIXED_POINT_PATH: &str = "artifacts/c99-route-fixed-point.rv64";
const BOOTSTRAP_COMPILER_PATH: &str = "artifacts/radiance.s0";
const EMULATOR_PATH: &str = "artifacts/emulator";
pub(super) const PROTECTED_AUDIT_PATH: &str = "evidence/protected-execution-audit.json";
pub(super) const RECEIPT_PATH: &str = "receipt.json";
const EXECUTABLE_TOOL_ROLE_COUNT: usize = 5;
const POLICY_DECISION_ALLOWED: &str = "allowed";

pub(super) fn publish_artifacts(
    output_root: &std::path::Path,
    stages: &crate::radiance::runtime::execution::RunOutputs,
    tools: &crate::radiance::runtime::native::BuildOutputs,
) -> Result<Vec<crunch_radiance_reference_core::RadiancePublicationArtifact>, crate::errors::RunError> {
    let bindings = [
        (
            crunch_radiance_reference_core::RadiancePublicationRole::SeedRouteFixedPoint,
            SEED_FIXED_POINT_PATH,
            stages.seed_final.as_path(),
        ),
        (
            crunch_radiance_reference_core::RadiancePublicationRole::C99RouteFixedPoint,
            C99_FIXED_POINT_PATH,
            stages.c99_final.as_path(),
        ),
        (
            crunch_radiance_reference_core::RadiancePublicationRole::BootstrapCompiler,
            BOOTSTRAP_COMPILER_PATH,
            tools.bootstrap_compiler.as_path(),
        ),
        (
            crunch_radiance_reference_core::RadiancePublicationRole::Emulator,
            EMULATOR_PATH,
            tools.emulator.as_path(),
        ),
    ];
    let artifacts_dir = output_root.join("artifacts");
    std::fs::create_dir_all(&artifacts_dir).map_err(|error| {
        crate::errors::RunError::Internal(format!(
            "creating Radiance artifact directory {}: {error}",
            artifacts_dir.display()
        ))
    })?;
    let mut publication = Vec::with_capacity(crunch_radiance_reference_core::RADIANCE_PUBLICATION_COUNT);
    for (role, relative_path, source) in bindings {
        let bytes = std::fs::read(source).map_err(|error| {
            crate::errors::RunError::Internal(format!(
                "reading Radiance publication source {}: {error}",
                source.display()
            ))
        })?;
        let destination = output_root.join(relative_path);
        crate::source_bundle::publish_immutable_source_bytes(&destination, &bytes, "Radiance proof artifact")?;
        let (digest_blake3, byte_count) =
            crate::radiance::source::digest_file(&destination, "published Radiance proof artifact")?;
        publication.push(crunch_radiance_reference_core::RadiancePublicationArtifact {
            role,
            relative_path: relative_path.to_string(),
            digest_blake3,
            byte_count,
        });
    }
    debug_assert_eq!(publication.len(), crunch_radiance_reference_core::RADIANCE_PUBLICATION_COUNT);
    debug_assert!(publication.iter().all(|artifact| artifact.byte_count > 0));
    Ok(publication)
}

pub(super) fn publish_protected_audit(
    output_root: &std::path::Path,
    stages: &crate::radiance::runtime::execution::RunOutputs,
) -> Result<String, crate::errors::RunError> {
    let evidence_dir = output_root.join("evidence");
    std::fs::create_dir_all(&evidence_dir).map_err(|error| {
        crate::errors::RunError::Internal(format!(
            "creating Radiance evidence directory {}: {error}",
            evidence_dir.display()
        ))
    })?;
    let audit = crate::radiance::runtime::AuditView {
        schema: crate::radiance::runtime::PROTECTED_AUDIT_SCHEMA,
        network_policy: crate::radiance::runtime::PROTECTED_NETWORK_POLICY,
        events: &stages.raw_audit_events,
    };
    let bytes = crate::radiance::source::canonical_pretty_json(&audit, "Radiance protected-execution audit")?;
    let destination = output_root.join(PROTECTED_AUDIT_PATH);
    crate::source_bundle::publish_immutable_source_bytes(&destination, &bytes, "Radiance protected-execution audit")?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    debug_assert_eq!(digest.len(), crunch_radiance_reference_core::BLAKE3_HEX_CHARS);
    debug_assert!(destination.is_file());
    Ok(digest)
}

pub(super) fn validate_protected_audit_bytes(
    bytes: &[u8],
    receipt: &crunch_radiance_reference_core::RadianceReferenceReceipt,
) -> Result<(), crate::errors::RunError> {
    let audit: crate::radiance::runtime::AuditRecord = serde_json::from_slice(bytes)
        .map_err(|error| crate::errors::RunError::Internal(format!("parsing Radiance protected audit: {error}")))?;
    if audit.schema != crate::radiance::runtime::PROTECTED_AUDIT_SCHEMA
        || audit.network_policy != crate::radiance::runtime::PROTECTED_NETWORK_POLICY
    {
        return Err(crate::errors::RunError::Internal(
            "Radiance protected audit policy identity is invalid".to_string(),
        ));
    }
    let executable_roles = [
        crunch_radiance_reference_core::RadianceArtifactRole::HostCompilerLauncher,
        crunch_radiance_reference_core::RadianceArtifactRole::HostCompilerDriver,
        crunch_radiance_reference_core::RadianceArtifactRole::HostLinker,
        crunch_radiance_reference_core::RadianceArtifactRole::BootstrapCompiler,
        crunch_radiance_reference_core::RadianceArtifactRole::Emulator,
    ];
    let executable_digests = executable_roles
        .iter()
        .map(|role| {
            receipt
                .tools
                .iter()
                .find(|tool| tool.role == *role)
                .map(|tool| tool.digest_blake3.as_str())
                .ok_or_else(|| {
                    crate::errors::RunError::Internal(format!("Radiance receipt omits executable tool role {role:?}"))
                })
        })
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    let audit_events_max = usize::try_from(crunch_radiance_reference_core::RADIANCE_AUDIT_EVENTS_MAX)
        .map_err(|_| crate::errors::RunError::Internal("Radiance audit event bound does not fit usize".to_string()))?;
    let is_event_set_admitted = !audit.events.is_empty()
        && audit.events.len() <= audit_events_max
        && audit.events.iter().all(|event| {
            event.policy_decision == POLICY_DECISION_ALLOWED && executable_digests.contains(event.digest_hex.as_str())
        });
    let is_every_tool_observed =
        executable_digests.iter().all(|digest| audit.events.iter().any(|event| event.digest_hex == *digest));
    if !is_event_set_admitted || !is_every_tool_observed || executable_digests.len() != EXECUTABLE_TOOL_ROLE_COUNT {
        return Err(crate::errors::RunError::Internal(
            "Radiance protected audit does not match the receipt executable identities".to_string(),
        ));
    }
    debug_assert!(audit.events.iter().all(|event| event.policy_decision == POLICY_DECISION_ALLOWED));
    debug_assert_eq!(executable_digests.len(), EXECUTABLE_TOOL_ROLE_COUNT);
    Ok(())
}

pub(super) fn publish_receipt_bytes(
    output_root: &std::path::Path,
    bytes: &[u8],
) -> Result<(), crate::errors::RunError> {
    let destination = output_root.join(RECEIPT_PATH);
    crate::source_bundle::publish_immutable_source_bytes(&destination, bytes, "Radiance reference receipt")?;
    debug_assert!(destination.is_file());
    debug_assert!(!bytes.is_empty());
    Ok(())
}
