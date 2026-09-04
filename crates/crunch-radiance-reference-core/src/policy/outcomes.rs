pub fn classify_route_convergence(
    route: crate::RadianceRoute,
    left: &crate::RadianceStageObservation,
    right: &crate::RadianceStageObservation,
) -> Result<crate::RadianceRouteConvergence, crate::RadianceReferenceError> {
    if left.route != route || right.route != route {
        return Err(crate::RadianceReferenceError::Convergence);
    }
    if left.stage != crate::RADIANCE_CONVERGENCE_LEFT_STAGE || right.stage != crate::RADIANCE_CONVERGENCE_RIGHT_STAGE {
        return Err(crate::RadianceReferenceError::Convergence);
    }
    let is_equal = left.output_blake3 == right.output_blake3 && left.output_bytes == right.output_bytes;
    let byte_count = if is_equal { left.output_bytes } else { 0 };
    debug_assert_eq!(left.route, right.route);
    debug_assert!(crate::valid_blake3(&left.output_blake3));
    Ok(crate::RadianceRouteConvergence {
        route,
        left_stage: left.stage,
        right_stage: right.stage,
        left_blake3: left.output_blake3.clone(),
        right_blake3: right.output_blake3.clone(),
        byte_count,
        equal: is_equal,
    })
}

pub fn classify_cross_route(
    seed: &crate::RadianceStageObservation,
    c99: &crate::RadianceStageObservation,
) -> Result<crate::RadianceCrossRouteComparison, crate::RadianceReferenceError> {
    if seed.route != crate::RadianceRoute::Seed || c99.route != crate::RadianceRoute::C99 {
        return Err(crate::RadianceReferenceError::Convergence);
    }
    if seed.stage != crate::RADIANCE_CONVERGENCE_RIGHT_STAGE || c99.stage != crate::RADIANCE_CONVERGENCE_RIGHT_STAGE {
        return Err(crate::RadianceReferenceError::Convergence);
    }
    let is_match = seed.output_blake3 == c99.output_blake3 && seed.output_bytes == c99.output_bytes;
    let disposition = if is_match {
        crate::RadianceCrossRouteDisposition::Match
    } else {
        crate::RadianceCrossRouteDisposition::Divergence
    };
    debug_assert!(crate::valid_blake3(&seed.output_blake3));
    debug_assert!(crate::valid_blake3(&c99.output_blake3));
    Ok(crate::RadianceCrossRouteComparison {
        seed_blake3: seed.output_blake3.clone(),
        c99_blake3: c99.output_blake3.clone(),
        seed_bytes: seed.output_bytes,
        c99_bytes: c99.output_bytes,
        disposition,
    })
}

pub(crate) fn validate_route_convergence(
    receipt: &crate::RadianceReferenceReceipt,
    stages: &alloc::collections::BTreeMap<(crate::RadianceRoute, u8), &crate::RadianceStageObservation>,
) -> Result<(), crate::RadianceReferenceError> {
    if receipt.route_convergence.len() != crate::RADIANCE_ROUTE_COUNT {
        return Err(crate::RadianceReferenceError::Convergence);
    }
    for route in [crate::RadianceRoute::Seed, crate::RadianceRoute::C99] {
        let left = stages
            .get(&(route, crate::RADIANCE_CONVERGENCE_LEFT_STAGE))
            .ok_or(crate::RadianceReferenceError::Convergence)?;
        let right = stages
            .get(&(route, crate::RADIANCE_CONVERGENCE_RIGHT_STAGE))
            .ok_or(crate::RadianceReferenceError::Convergence)?;
        let expected = classify_route_convergence(route, left, right)?;
        let actual = receipt
            .route_convergence
            .iter()
            .find(|comparison| comparison.route == route)
            .ok_or(crate::RadianceReferenceError::Convergence)?;
        if actual != &expected || !actual.equal {
            return Err(crate::RadianceReferenceError::Convergence);
        }
    }
    debug_assert_eq!(receipt.route_convergence.len(), crate::RADIANCE_ROUTE_COUNT);
    debug_assert!(receipt.route_convergence.iter().all(|comparison| comparison.equal));
    Ok(())
}

pub(crate) fn validate_cross_route(
    receipt: &crate::RadianceReferenceReceipt,
    stages: &alloc::collections::BTreeMap<(crate::RadianceRoute, u8), &crate::RadianceStageObservation>,
) -> Result<(), crate::RadianceReferenceError> {
    let seed = stages
        .get(&(crate::RadianceRoute::Seed, crate::RADIANCE_CONVERGENCE_RIGHT_STAGE))
        .ok_or(crate::RadianceReferenceError::Convergence)?;
    let c99 = stages
        .get(&(crate::RadianceRoute::C99, crate::RADIANCE_CONVERGENCE_RIGHT_STAGE))
        .ok_or(crate::RadianceReferenceError::Convergence)?;
    let expected = classify_cross_route(seed, c99)?;
    if receipt.cross_route != expected {
        return Err(crate::RadianceReferenceError::Convergence);
    }
    debug_assert_eq!(receipt.cross_route.seed_blake3, seed.output_blake3);
    debug_assert_eq!(receipt.cross_route.c99_blake3, c99.output_blake3);
    Ok(())
}

pub(crate) fn validate_zero_events(
    events: &crate::RadianceZeroEventCounts,
) -> Result<(), crate::RadianceReferenceError> {
    if events.live_fetches != 0 {
        return Err(crate::RadianceReferenceError::ZeroEvents);
    }
    if events.source_fallbacks != 0 {
        return Err(crate::RadianceReferenceError::ZeroEvents);
    }
    if events.substitutions != 0 || events.ambient_discoveries != 0 {
        return Err(crate::RadianceReferenceError::ZeroEvents);
    }
    debug_assert_eq!(events.live_fetches, 0);
    debug_assert_eq!(events.source_fallbacks, 0);
    Ok(())
}

pub(crate) fn validate_publication(
    publication: &[crate::RadiancePublicationArtifact],
    tools: &alloc::collections::BTreeMap<crate::RadianceArtifactRole, &crate::RadianceArtifactObservation>,
    stages: &alloc::collections::BTreeMap<(crate::RadianceRoute, u8), &crate::RadianceStageObservation>,
) -> Result<(), crate::RadianceReferenceError> {
    if publication.len() != crate::RADIANCE_PUBLICATION_COUNT {
        return Err(crate::RadianceReferenceError::Publication);
    }
    let mut roles = alloc::collections::BTreeSet::new();
    for artifact in publication {
        if !roles.insert(artifact.role) {
            return Err(crate::RadianceReferenceError::Publication);
        }
        validate_publication_shape(artifact)?;
        let (digest, byte_count) = publication_source(artifact.role, tools, stages)?;
        if artifact.digest_blake3 != digest || artifact.byte_count != byte_count {
            return Err(crate::RadianceReferenceError::Publication);
        }
    }
    debug_assert_eq!(roles.len(), crate::RADIANCE_PUBLICATION_COUNT);
    debug_assert_eq!(publication.len(), roles.len());
    Ok(())
}

fn validate_publication_shape(
    artifact: &crate::RadiancePublicationArtifact,
) -> Result<(), crate::RadianceReferenceError> {
    validate_text(&artifact.relative_path)?;
    if artifact.relative_path.starts_with('/') || artifact.relative_path.contains("..") {
        return Err(crate::RadianceReferenceError::Publication);
    }
    if !crate::valid_blake3(&artifact.digest_blake3) {
        return Err(crate::RadianceReferenceError::Digest);
    }
    if artifact.byte_count == 0 || artifact.byte_count > crate::RADIANCE_ARTIFACT_BYTES_MAX {
        return Err(crate::RadianceReferenceError::Publication);
    }
    debug_assert!(!artifact.relative_path.is_empty());
    debug_assert!(crate::valid_blake3(&artifact.digest_blake3));
    Ok(())
}

fn publication_source(
    role: crate::RadiancePublicationRole,
    tools: &alloc::collections::BTreeMap<crate::RadianceArtifactRole, &crate::RadianceArtifactObservation>,
    stages: &alloc::collections::BTreeMap<(crate::RadianceRoute, u8), &crate::RadianceStageObservation>,
) -> Result<(alloc::string::String, u64), crate::RadianceReferenceError> {
    match role {
        crate::RadiancePublicationRole::SeedRouteFixedPoint => stage_artifact(stages, crate::RadianceRoute::Seed),
        crate::RadiancePublicationRole::C99RouteFixedPoint => stage_artifact(stages, crate::RadianceRoute::C99),
        crate::RadiancePublicationRole::BootstrapCompiler => {
            tool_artifact(tools, crate::RadianceArtifactRole::BootstrapCompiler)
        }
        crate::RadiancePublicationRole::Emulator => tool_artifact(tools, crate::RadianceArtifactRole::Emulator),
    }
}

fn stage_artifact(
    stages: &alloc::collections::BTreeMap<(crate::RadianceRoute, u8), &crate::RadianceStageObservation>,
    route: crate::RadianceRoute,
) -> Result<(alloc::string::String, u64), crate::RadianceReferenceError> {
    let stage = stages
        .get(&(route, crate::RADIANCE_CONVERGENCE_RIGHT_STAGE))
        .ok_or(crate::RadianceReferenceError::Publication)?;
    debug_assert_eq!(stage.route, route);
    debug_assert_eq!(stage.stage, crate::RADIANCE_CONVERGENCE_RIGHT_STAGE);
    Ok((stage.output_blake3.clone(), stage.output_bytes))
}

fn tool_artifact(
    tools: &alloc::collections::BTreeMap<crate::RadianceArtifactRole, &crate::RadianceArtifactObservation>,
    role: crate::RadianceArtifactRole,
) -> Result<(alloc::string::String, u64), crate::RadianceReferenceError> {
    let tool = tools.get(&role).ok_or(crate::RadianceReferenceError::Publication)?;
    debug_assert_eq!(tool.role, role);
    debug_assert!(tool.byte_count > 0);
    Ok((tool.digest_blake3.clone(), tool.byte_count))
}

pub(crate) fn validate_text(value: &str) -> Result<(), crate::RadianceReferenceError> {
    let text_bytes = u32::try_from(value.len()).map_err(|_| crate::RadianceReferenceError::Text)?;
    if text_bytes == 0 || text_bytes > crate::RADIANCE_TEXT_BYTES_MAX {
        return Err(crate::RadianceReferenceError::Text);
    }
    if value.chars().any(char::is_control) {
        return Err(crate::RadianceReferenceError::Text);
    }
    debug_assert!(text_bytes > 0);
    debug_assert!(!value.is_empty());
    Ok(())
}
