pub(crate) fn validate_tools(
    tools: &[crate::RadianceArtifactObservation],
) -> Result<
    alloc::collections::BTreeMap<crate::RadianceArtifactRole, &crate::RadianceArtifactObservation>,
    crate::RadianceReferenceError,
> {
    if tools.len() != crate::RADIANCE_TOOL_COUNT {
        return Err(crate::RadianceReferenceError::Tool);
    }
    let expected = alloc::collections::BTreeSet::from([
        crate::RadianceArtifactRole::HostCompilerLauncher,
        crate::RadianceArtifactRole::HostCompilerDriver,
        crate::RadianceArtifactRole::HostLinker,
        crate::RadianceArtifactRole::HostCrtInputs,
        crate::RadianceArtifactRole::HostLibgccInputs,
        crate::RadianceArtifactRole::BootstrapCompiler,
        crate::RadianceArtifactRole::Emulator,
        crate::RadianceArtifactRole::Seed,
    ]);
    let mut by_role = alloc::collections::BTreeMap::new();
    for tool in tools {
        validate_artifact(tool)?;
        if by_role.insert(tool.role, tool).is_some() {
            return Err(crate::RadianceReferenceError::Tool);
        }
    }
    if by_role.keys().copied().collect::<alloc::collections::BTreeSet<_>>() != expected {
        return Err(crate::RadianceReferenceError::Tool);
    }
    debug_assert_eq!(by_role.len(), crate::RADIANCE_TOOL_COUNT);
    debug_assert_eq!(expected.len(), crate::RADIANCE_TOOL_COUNT);
    Ok(by_role)
}

fn validate_artifact(artifact: &crate::RadianceArtifactObservation) -> Result<(), crate::RadianceReferenceError> {
    if !crate::valid_blake3(&artifact.digest_blake3) {
        return Err(crate::RadianceReferenceError::Digest);
    }
    let byte_count_max = match artifact.role {
        crate::RadianceArtifactRole::HostCrtInputs | crate::RadianceArtifactRole::HostLibgccInputs => {
            crate::RADIANCE_LINK_RUNTIME_BYTES_MAX
        }
        _ => crate::RADIANCE_ARTIFACT_BYTES_MAX,
    };
    if artifact.byte_count == 0 || artifact.byte_count > byte_count_max {
        return Err(crate::RadianceReferenceError::Tool);
    }
    debug_assert!(artifact.byte_count > 0);
    debug_assert!(artifact.byte_count <= byte_count_max);
    Ok(())
}

pub(crate) fn validate_stages<'a>(
    receipt: &'a crate::RadianceReferenceReceipt,
    tools: &alloc::collections::BTreeMap<crate::RadianceArtifactRole, &crate::RadianceArtifactObservation>,
) -> Result<
    alloc::collections::BTreeMap<(crate::RadianceRoute, u8), &'a crate::RadianceStageObservation>,
    crate::RadianceReferenceError,
> {
    if receipt.stages.len() != crate::RADIANCE_STAGE_COUNT {
        return Err(crate::RadianceReferenceError::Stage);
    }
    let mut stages = alloc::collections::BTreeMap::new();
    for observation in &receipt.stages {
        validate_stage_shape(observation)?;
        if stages.insert((observation.route, observation.stage), observation).is_some() {
            return Err(crate::RadianceReferenceError::Stage);
        }
    }
    for planned in &receipt.plan.stages {
        let observed = stages.get(&(planned.route, planned.stage)).ok_or(crate::RadianceReferenceError::Stage)?;
        validate_stage_against_plan(planned, observed, tools, &stages)?;
    }
    debug_assert_eq!(stages.len(), crate::RADIANCE_STAGE_COUNT);
    debug_assert_eq!(receipt.plan.stages.len(), receipt.stages.len());
    Ok(stages)
}

fn validate_stage_shape(stage: &crate::RadianceStageObservation) -> Result<(), crate::RadianceReferenceError> {
    let is_stage_in_range = (crate::RADIANCE_FIRST_STAGE..=crate::RADIANCE_STAGES_PER_ROUTE).contains(&stage.stage);
    if !is_stage_in_range {
        return Err(crate::RadianceReferenceError::Stage);
    }
    let digests = [
        stage.launcher_blake3.as_str(),
        stage.predecessor_blake3.as_str(),
        stage.output_blake3.as_str(),
        stage.stdout_blake3.as_str(),
        stage.stderr_blake3.as_str(),
        stage.protected_audit_blake3.as_str(),
    ];
    if digests.iter().any(|value| !crate::valid_blake3(value)) {
        return Err(crate::RadianceReferenceError::Digest);
    }
    if stage.output_bytes == 0 || stage.output_bytes > crate::RADIANCE_ARTIFACT_BYTES_MAX {
        return Err(crate::RadianceReferenceError::Stage);
    }
    if stage.exit_code != 0 || stage.denied_event_count != 0 {
        return Err(crate::RadianceReferenceError::Execution);
    }
    if stage.protected_audit_event_count == 0 || stage.protected_audit_event_count > crate::RADIANCE_AUDIT_EVENTS_MAX {
        return Err(crate::RadianceReferenceError::Execution);
    }
    super::outcomes::validate_text(&stage.output_role)?;
    debug_assert!(is_stage_in_range);
    debug_assert!(digests.iter().all(|value| crate::valid_blake3(value)));
    Ok(())
}

fn validate_stage_against_plan(
    planned: &crate::RadianceStagePlan,
    observed: &crate::RadianceStageObservation,
    tools: &alloc::collections::BTreeMap<crate::RadianceArtifactRole, &crate::RadianceArtifactObservation>,
    stages: &alloc::collections::BTreeMap<(crate::RadianceRoute, u8), &crate::RadianceStageObservation>,
) -> Result<(), crate::RadianceReferenceError> {
    if observed.launch_kind != planned.launch_kind || observed.output_role != planned.output_role {
        return Err(crate::RadianceReferenceError::Stage);
    }
    let launcher_role = match planned.launch_kind {
        crate::RadianceLaunchKind::NativeBootstrap => crate::RadianceArtifactRole::BootstrapCompiler,
        crate::RadianceLaunchKind::EmulatedCompiler => crate::RadianceArtifactRole::Emulator,
    };
    let launcher = tools.get(&launcher_role).ok_or(crate::RadianceReferenceError::Tool)?;
    if observed.launcher_blake3 != launcher.digest_blake3 {
        return Err(crate::RadianceReferenceError::Lineage);
    }
    let expected_predecessor = expected_predecessor_digest(planned, tools, stages)?;
    if observed.predecessor_blake3 != expected_predecessor {
        return Err(crate::RadianceReferenceError::Lineage);
    }
    debug_assert_eq!(observed.route, planned.route);
    debug_assert_eq!(observed.stage, planned.stage);
    Ok(())
}

fn expected_predecessor_digest(
    planned: &crate::RadianceStagePlan,
    tools: &alloc::collections::BTreeMap<crate::RadianceArtifactRole, &crate::RadianceArtifactObservation>,
    stages: &alloc::collections::BTreeMap<(crate::RadianceRoute, u8), &crate::RadianceStageObservation>,
) -> Result<alloc::string::String, crate::RadianceReferenceError> {
    if planned.stage == crate::RADIANCE_FIRST_STAGE {
        let role = match planned.route {
            crate::RadianceRoute::Seed => crate::RadianceArtifactRole::Seed,
            crate::RadianceRoute::C99 => crate::RadianceArtifactRole::BootstrapCompiler,
        };
        return tools
            .get(&role)
            .map(|artifact| artifact.digest_blake3.clone())
            .ok_or(crate::RadianceReferenceError::Lineage);
    }
    let prior_stage = planned.stage.checked_sub(1).ok_or(crate::RadianceReferenceError::Lineage)?;
    let prior = stages.get(&(planned.route, prior_stage)).ok_or(crate::RadianceReferenceError::Lineage)?;
    debug_assert!(prior.stage < planned.stage);
    debug_assert_eq!(prior.route, planned.route);
    Ok(prior.output_blake3.clone())
}
