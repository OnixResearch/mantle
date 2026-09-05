use super::*;
use crate::source_built_fixed_point_checkpoint::*;
use crate::source_built_fixed_point_checkpoint_shell::CheckpointPayloadSource;
use crate::source_built_fixed_point_checkpoint_shell::ProviderCheckpointLimits;

pub(crate) enum ProviderPrefix<'a> {
    Transition(&'a Path),
    Stagex(&'a Path, &'a crate::stagex_provider::StagexProviderPublicationReport),
    Native(&'a NativeProviderPrefix),
    Complete(&'a ConstructedProviders),
}

pub(crate) fn publish_dev_provider_boundary(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    prefix: ProviderPrefix<'_>,
) -> Result<(), RunError> {
    if !options.dev_resume {
        return Ok(());
    }
    let cache = options
        .dev_provider_cache
        .ok_or_else(|| proof_error("dev prefix publication requires its cache".to_string()))?;
    let store = super::super::dev_resume::checkpoint_store(cache);
    let limits = crate::source_built_fixed_point_checkpoint_shell::provider_checkpoint_limits(options.disk_bytes_max);
    let (observations, sources) = prefix_material(prepared, &prefix, limits)?;
    let publication = crate::source_built_fixed_point_checkpoint_shell::publish_dev_provider_prefix(
        &store,
        &prepared.plan,
        observations,
        &sources,
        STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST,
        limits,
    )?;
    let admitted =
        admit_dev_checkpoint_identity(&store, &prepared.plan, &publication.checkpoint_digest_blake3, limits)?
            .ok_or_else(|| proof_error("published dev prefix disappeared".to_string()))?;
    super::super::dev_resume::publish_provider_prefix_manifest(options, prepared, &admitted)
}

fn prefix_material(
    prepared: &PreparedAttempt,
    prefix: &ProviderPrefix<'_>,
    limits: ProviderCheckpointLimits,
) -> Result<(Vec<ProviderCheckpointStageObservation>, Vec<CheckpointPayloadSource>), RunError> {
    if let ProviderPrefix::Complete(providers) = prefix {
        let mut sources = provider_checkpoint_payload_sources(providers)?.to_vec();
        normalize_stagex_destination(&mut sources);
        return Ok((provider_checkpoint_stage_observations(prepared, providers, limits)?, sources));
    }
    let (transition, stagex, native) = match prefix {
        ProviderPrefix::Transition(path) => (*path, None, None),
        ProviderPrefix::Stagex(path, report) => (*path, Some(*report), None),
        ProviderPrefix::Native(native) => (
            native.stagex_transition_execution_dir.as_path(),
            Some(&native.stagex_provider_report),
            Some(*native),
        ),
        ProviderPrefix::Complete(_) => unreachable!("complete prefix returns before partial material"),
    };
    let producer = super::super::dev_resume::current_executable_digest()?;
    let mut observations = Vec::with_capacity(PROVIDER_CHECKPOINT_STAGE_COUNT);
    let mut sources = Vec::with_capacity(PROVIDER_CHECKPOINT_PAYLOAD_COUNT);
    observations.push(transition_observation(transition, &producer, limits)?);
    sources.push(checkpoint_payload_source(
        PAYLOAD_STAGEX_TRANSITION,
        transition,
        Path::new(CHECKPOINT_STAGEX_TRANSITION_PATH),
        CheckpointPayloadKind::PreservedTree,
    ));
    if let Some(stagex) = stagex {
        observations.push(stagex_observation(stagex, &producer)?);
        sources.push(checkpoint_payload_source(
            PAYLOAD_STAGEX_PROVIDER,
            &stagex.output_path,
            &Path::new(CHECKPOINT_STAGEX_PROVIDER_PREFIX).join(STAGEX_PROVIDER_STORE_BASENAME),
            CheckpointPayloadKind::Directory,
        ));
    }
    if let Some(native) = native {
        observations.push(native_observation(native, &producer)?);
        sources.extend(native_sources(native)?);
    }
    assert!(!observations.is_empty());
    assert!(observations.len() < PROVIDER_CHECKPOINT_STAGE_COUNT);
    Ok((observations, sources))
}

fn normalize_stagex_destination(sources: &mut [CheckpointPayloadSource]) {
    for source in sources {
        if source.payload_id == PAYLOAD_STAGEX_PROVIDER {
            source.relative_path = Path::new(CHECKPOINT_STAGEX_PROVIDER_PREFIX).join(STAGEX_PROVIDER_STORE_BASENAME);
        }
    }
}

fn transition_observation(
    path: &Path,
    producer: &str,
    limits: ProviderCheckpointLimits,
) -> Result<ProviderCheckpointStageObservation, RunError> {
    let identity = crate::preserved_evidence_tree::hash_preserved_evidence_tree(path, limits.preserved_tree)?;
    let evidence = checkpoint_evidence_digest(&[
        ("transition-plan", path.join(STAGEX_TRANSITION_PLAN_FILE)),
        ("transition-report", path.join(STAGEX_TRANSITION_REPORT_FILE)),
        ("transition-audit", path.join(STAGEX_TRANSITION_AUDIT_FILE)),
    ])?;
    Ok(provider_stage_observation(
        ProofOutputRole::StagexTransition,
        &identity.digest_blake3,
        &identity.digest_blake3,
        &identity.digest_blake3,
        &evidence,
        producer,
    ))
}

fn stagex_observation(
    report: &crate::stagex_provider::StagexProviderPublicationReport,
    producer: &str,
) -> Result<ProviderCheckpointStageObservation, RunError> {
    let payload = crate::release_tree_copy::hash_directory_tree(&report.output_path)?.1;
    let evidence = checkpoint_evidence_digest(&[
        ("stagex-receipt", report.receipt_path.clone()),
        (
            "stagex-validation",
            report.output_path.join(crate::stagex_provider::PROVIDER_VALIDATION_RELATIVE_PATH),
        ),
    ])?;
    Ok(provider_stage_observation(
        ProofOutputRole::StagexProvider,
        &report.final_bundle_digest_blake3,
        &report.normalized_provider_digest_blake3,
        &payload,
        &evidence,
        producer,
    ))
}

fn native_observation(
    prefix: &NativeProviderPrefix,
    producer: &str,
) -> Result<ProviderCheckpointStageObservation, RunError> {
    let payload = crate::release_tree_copy::hash_directory_tree(&prefix.native_provider.output.path)?.1;
    let trust = prefix
        .native_action_trust
        .as_ref()
        .ok_or_else(|| proof_error("dev native prefix lacks action evidence".to_string()))?;
    let evidence = checkpoint_evidence_digest(&[
        ("native-transcript", prefix.native_provider.transcript_path.clone()),
        ("native-admission", prefix.native_admission_report_path.clone()),
        ("native-action-plan", trust.plan_path.clone()),
        ("native-action-reconciliation", trust.reconciliation_path.clone()),
    ])?;
    Ok(provider_stage_observation(
        ProofOutputRole::FullSourceNativeProvider,
        &prefix.native_admission.output_digest_blake3,
        &prefix.native_admission.output_digest_blake3,
        &payload,
        &evidence,
        producer,
    ))
}

fn native_sources(prefix: &NativeProviderPrefix) -> Result<Vec<CheckpointPayloadSource>, RunError> {
    let trust = prefix
        .native_action_trust
        .as_ref()
        .ok_or_else(|| proof_error("dev native prefix lacks action evidence".to_string()))?;
    let native_basename = required_utf8_basename(&prefix.native_provider.output.path, "native provider")?;
    let mut sources = Vec::with_capacity(PROVIDER_CHECKPOINT_PAYLOAD_COUNT);
    sources.push(checkpoint_payload_source(
        PAYLOAD_NATIVE_PROVIDER,
        &prefix.native_provider.output.path,
        &Path::new(CHECKPOINT_NATIVE_PROVIDER_PREFIX).join(native_basename),
        CheckpointPayloadKind::Directory,
    ));
    for (id, source, relative) in [
        (PAYLOAD_NATIVE_ADMISSION, &prefix.native_admission_report_path, CHECKPOINT_NATIVE_ADMISSION_PATH),
        (
            PAYLOAD_NATIVE_TRANSCRIPT,
            &prefix.native_provider.transcript_path,
            CHECKPOINT_NATIVE_TRANSCRIPT_PATH,
        ),
        (PAYLOAD_NATIVE_ACTION_PLAN, &trust.plan_path, CHECKPOINT_NATIVE_ACTION_PLAN_PATH),
        (
            PAYLOAD_NATIVE_ACTION_RECONCILIATION,
            &trust.reconciliation_path,
            CHECKPOINT_NATIVE_ACTION_RECONCILIATION_PATH,
        ),
    ] {
        sources.push(checkpoint_payload_source(id, source, Path::new(relative), CheckpointPayloadKind::RegularFile));
    }
    for (name, id) in [
        ("make", PAYLOAD_RUST_HOST_MAKE),
        ("cmake", PAYLOAD_RUST_HOST_CMAKE),
        ("python", PAYLOAD_RUST_HOST_PYTHON),
        ("perl", PAYLOAD_RUST_HOST_PERL),
        ("busybox", PAYLOAD_RUST_HOST_BUSYBOX),
        ("linux-headers", PAYLOAD_RUST_HOST_LINUX_HEADERS),
    ] {
        let tool = prefix
            .rust_host_tools
            .get(name)
            .ok_or_else(|| proof_error(format!("dev native prefix is missing {name}")))?;
        sources.push(checkpoint_payload_source(
            id,
            &tool.output.path,
            &Path::new(CHECKPOINT_RUST_HOST_TOOL_PREFIX).join(name),
            CheckpointPayloadKind::Directory,
        ));
    }
    sources.push(checkpoint_payload_source(
        PAYLOAD_RUST_HOST_EVIDENCE,
        &prefix.rust_host_tool_evidence_dir,
        Path::new(CHECKPOINT_RUST_HOST_EVIDENCE_PATH),
        CheckpointPayloadKind::PreservedTree,
    ));
    assert!(!sources.is_empty());
    assert!(sources.len() < PROVIDER_CHECKPOINT_PAYLOAD_COUNT);
    Ok(sources)
}
