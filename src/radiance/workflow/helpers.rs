const SOURCE_STATE_DOMAIN: &[u8] = b"mantle.radiance-reference.source-state.v1";

pub(super) struct SourceStateIdentityInput<'a> {
    pub source_bundle_blake3: &'a str,
    pub cohort_blake3: &'a str,
}

pub(super) fn read_cohort(
    path: &std::path::Path,
) -> Result<crunch_radiance_reference_core::RadianceSourceCohortWire, crate::errors::RunError> {
    let bytes = std::fs::read(path).map_err(|error| {
        crate::errors::RunError::Internal(format!("reading Radiance source cohort {}: {error}", path.display()))
    })?;
    let cohort = serde_json::from_slice(&bytes)
        .map_err(|error| crate::errors::RunError::Internal(format!("parsing Radiance source cohort: {error}")))?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(path.is_file());
    Ok(cohort)
}

pub(super) fn validate_manifest_binding(
    manifest: &crate::source_bundle::SourceBundleManifest,
    cohort: &crunch_radiance_reference_core::RadianceSourceCohortWire,
) -> Result<(), crate::errors::RunError> {
    for member in &cohort.members {
        let role = crate::radiance::profile::role_label(member.role);
        let matches = manifest.records.iter().filter(|record| record.identity == role).collect::<Vec<_>>();
        let [record] = matches.as_slice() else {
            return Err(crate::errors::RunError::Internal(format!(
                "Radiance source bundle role {role} is missing or duplicated"
            )));
        };
        let observation = crate::source_bundle::admitted_source_observation(record)?.ok_or_else(|| {
            crate::errors::RunError::Internal(format!("Radiance source bundle role {role} lacks an observation"))
        })?;
        if observation != member.observation {
            return Err(crate::errors::RunError::Internal(format!(
                "Radiance source bundle role {role} does not match its cohort"
            )));
        }
    }
    if manifest.records.len() != cohort.members.len() {
        return Err(crate::errors::RunError::Internal(
            "Radiance source bundle contains records outside the admitted cohort".to_string(),
        ));
    }
    debug_assert_eq!(manifest.records.len(), crunch_radiance_reference_core::RADIANCE_SOURCE_COUNT);
    debug_assert_eq!(manifest.records.len(), cohort.members.len());
    Ok(())
}

pub(super) fn create_root(output: &std::path::Path) -> Result<(), crate::errors::RunError> {
    let parent = output
        .parent()
        .ok_or_else(|| crate::errors::RunError::Internal("Radiance proof output has no parent".to_string()))?;
    if !parent.is_dir() {
        return Err(crate::errors::RunError::Internal(format!(
            "Radiance proof output parent is missing: {}",
            parent.display()
        )));
    }
    std::fs::create_dir(output).map_err(|error| {
        crate::errors::RunError::Internal(format!("creating Radiance proof output {}: {error}", output.display()))
    })?;
    debug_assert!(output.is_dir());
    debug_assert!(parent.is_dir());
    Ok(())
}

pub(super) fn materialize(
    manifest: &crate::source_bundle::SourceBundleManifest,
    root: &std::path::Path,
) -> Result<Vec<(String, std::path::PathBuf)>, crate::errors::RunError> {
    std::fs::create_dir(root).map_err(|error| {
        crate::errors::RunError::Internal(format!("creating Radiance source root {}: {error}", root.display()))
    })?;
    let mut sources = Vec::with_capacity(manifest.records.len());
    for record in &manifest.records {
        crate::radiance::source::role_from_label(&record.identity)?;
        let destination = root.join(&record.identity);
        crate::source_bundle::materialize_source_record_payload(record, &destination)?;
        sources.push((record.identity.clone(), destination));
    }
    debug_assert_eq!(sources.len(), manifest.records.len());
    debug_assert!(sources.iter().all(|(_, path)| path.is_dir()));
    Ok(sources)
}

pub(super) fn source_path<'a>(
    sources: &'a [(String, std::path::PathBuf)],
    role: &str,
) -> Result<&'a std::path::Path, crate::errors::RunError> {
    let matches = sources.iter().filter(|(candidate, _)| candidate == role).collect::<Vec<_>>();
    let [selected] = matches.as_slice() else {
        return Err(crate::errors::RunError::Internal(format!(
            "materialized Radiance source role {role} is missing or duplicated"
        )));
    };
    debug_assert_eq!(selected.0, role);
    debug_assert!(selected.1.is_dir());
    Ok(selected.1.as_path())
}

pub(super) fn source_state_identity(input: SourceStateIdentityInput<'_>) -> Result<String, crate::errors::RunError> {
    let framed = format!(
        "{}\0{}\0{}",
        String::from_utf8_lossy(SOURCE_STATE_DOMAIN),
        input.source_bundle_blake3,
        input.cohort_blake3
    );
    let digest = blake3::hash(framed.as_bytes()).to_hex().to_string();
    if !valid_blake3_hex(&digest) {
        return Err(crate::errors::RunError::Internal("Radiance source state identity is invalid".to_string()));
    }
    debug_assert!(!input.source_bundle_blake3.is_empty());
    debug_assert!(!input.cohort_blake3.is_empty());
    Ok(digest)
}

pub(super) fn route_convergence(
    stages: &[crunch_radiance_reference_core::RadianceStageObservation],
) -> Result<Vec<crunch_radiance_reference_core::RadianceRouteConvergence>, crate::errors::RunError> {
    let mut comparisons = Vec::with_capacity(crunch_radiance_reference_core::RADIANCE_ROUTE_COUNT);
    for route in [
        crunch_radiance_reference_core::RadianceRoute::Seed,
        crunch_radiance_reference_core::RadianceRoute::C99,
    ] {
        let left = stage(stages, route, crunch_radiance_reference_core::RADIANCE_CONVERGENCE_LEFT_STAGE)?;
        let right = stage(stages, route, crunch_radiance_reference_core::RADIANCE_CONVERGENCE_RIGHT_STAGE)?;
        comparisons.push(crunch_radiance_reference_core::classify_route_convergence(route, left, right).map_err(
            |error| crate::errors::RunError::Internal(format!("classifying Radiance route convergence: {error:?}")),
        )?);
    }
    debug_assert_eq!(comparisons.len(), crunch_radiance_reference_core::RADIANCE_ROUTE_COUNT);
    debug_assert!(comparisons.iter().all(|comparison| comparison.equal));
    Ok(comparisons)
}

pub(super) fn final_stage_pair(
    stages: &[crunch_radiance_reference_core::RadianceStageObservation],
) -> Result<
    (
        &crunch_radiance_reference_core::RadianceStageObservation,
        &crunch_radiance_reference_core::RadianceStageObservation,
    ),
    crate::errors::RunError,
> {
    let seed = stage(
        stages,
        crunch_radiance_reference_core::RadianceRoute::Seed,
        crunch_radiance_reference_core::RADIANCE_CONVERGENCE_RIGHT_STAGE,
    )?;
    let c99 = stage(
        stages,
        crunch_radiance_reference_core::RadianceRoute::C99,
        crunch_radiance_reference_core::RADIANCE_CONVERGENCE_RIGHT_STAGE,
    )?;
    debug_assert_eq!(seed.route, crunch_radiance_reference_core::RadianceRoute::Seed);
    debug_assert_eq!(c99.route, crunch_radiance_reference_core::RadianceRoute::C99);
    Ok((seed, c99))
}

fn stage(
    stages: &[crunch_radiance_reference_core::RadianceStageObservation],
    route: crunch_radiance_reference_core::RadianceRoute,
    stage_number: u8,
) -> Result<&crunch_radiance_reference_core::RadianceStageObservation, crate::errors::RunError> {
    let matches = stages
        .iter()
        .filter(|candidate| candidate.route == route && candidate.stage == stage_number)
        .collect::<Vec<_>>();
    let [selected] = matches.as_slice() else {
        return Err(crate::errors::RunError::Internal("Radiance stage is missing or duplicated".to_string()));
    };
    debug_assert_eq!(selected.route, route);
    debug_assert_eq!(selected.stage, stage_number);
    Ok(*selected)
}

pub(super) fn verify_published(
    output: &std::path::Path,
    receipt: &crunch_radiance_reference_core::RadianceReferenceReceipt,
) -> Result<(), crate::errors::RunError> {
    for artifact in &receipt.publication {
        let path = output.join(publication_path(&artifact.relative_path));
        let (digest, byte_count) = crate::radiance::source::digest_file(&path, "Radiance published artifact")?;
        if digest != artifact.digest_blake3 || byte_count != artifact.byte_count {
            return Err(crate::errors::RunError::Internal(format!(
                "Radiance published artifact drifted: {}",
                artifact.relative_path
            )));
        }
    }
    debug_assert_eq!(receipt.publication.len(), crunch_radiance_reference_core::RADIANCE_PUBLICATION_COUNT);
    debug_assert!(receipt.publication.iter().all(|artifact| artifact.byte_count > 0));
    Ok(())
}

pub(super) fn publication_path(value: &str) -> &std::path::Path {
    std::path::Path::new(value)
}

pub(in crate::radiance) fn valid_blake3_hex(value: &str) -> bool {
    value.len() == crunch_radiance_reference_core::BLAKE3_HEX_CHARS
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn write_import_summary(
    output: &std::path::Path,
    source_ingest_result: &crate::source_bundle::SourceBundleImportReport,
) -> Result<(), crate::errors::RunError> {
    let bytes = crate::radiance::source::canonical_pretty_json(source_ingest_result, "Radiance source import report")?;
    let path = output.join("evidence/source-import-report.json");
    crate::source_bundle::publish_immutable_source_bytes(&path, &bytes, "Radiance source import report")?;
    debug_assert!(path.is_file());
    debug_assert!(!bytes.is_empty());
    Ok(())
}
