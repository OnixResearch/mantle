use super::*;

const CHECKPOINT_ORIGIN_TOOLCHAIN_CLOSURE_FILE: &str = "source-built-toolchain-closure.json";
const CHECKPOINT_CLOSURE_RELOCATION_REPORT_FILE: &str = "provider-checkpoint-closure-relocation.json";
const CHECKPOINT_CLOSURE_RELOCATION_REPORT_SCHEMA: &str = "mantle-source-built-checkpoint-closure-relocation-v1";

pub(super) fn import_provider_checkpoint_attempt(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    import_root: &Path,
) -> Result<(), RunError> {
    let imported_plan = read_imported_attempt_plan(import_root)?;
    validate_imported_attempt_status(import_root, &imported_plan)?;
    validate_imported_provider_authority(&prepared.plan, &imported_plan, import_root)?;
    let providers = imported_constructed_providers(options, prepared, import_root)?;
    publish_constructed_provider_checkpoint(options, prepared, &providers)?;
    write_checkpoint_import_report(prepared, import_root, &imported_plan)
}

fn read_imported_attempt_plan(import_root: &Path) -> Result<SourceBuiltFixedPointPlan, RunError> {
    let path = import_root.join(PLAN_FILE);
    let bytes = fs::read(&path)
        .map_err(|error| proof_error(format!("reading imported attempt plan {}: {error}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| proof_error(format!("parsing imported attempt plan {}: {error}", path.display())))
}

pub(super) fn validate_imported_attempt_status(
    import_root: &Path,
    plan: &SourceBuiltFixedPointPlan,
) -> Result<(), RunError> {
    let path = import_root.join(ATTEMPT_STATUS_FILE);
    let bytes = fs::read(&path)
        .map_err(|error| proof_error(format!("reading imported attempt status {}: {error}", path.display())))?;
    let status: ImportedAttemptStatus = serde_json::from_slice(&bytes)
        .map_err(|error| proof_error(format!("parsing imported attempt status {}: {error}", path.display())))?;
    if status.status == PROOF_STATUS_RUNNING {
        return Err(proof_error("cannot import a running proof attempt".to_string()));
    }
    if !matches!(status.status.as_str(), PROOF_STATUS_FAILED | PROOF_STATUS_COMPLETE) {
        return Err(proof_error(format!("imported attempt has unsupported status {}", status.status)));
    }
    if status.plan_digest_blake3.as_deref() != Some(plan.plan_digest_blake3.as_str()) {
        return Err(proof_error("imported attempt status does not bind its plan digest".to_string()));
    }
    assert_ne!(status.status, PROOF_STATUS_RUNNING);
    Ok(())
}

pub(super) fn validate_imported_provider_authority(
    current: &SourceBuiltFixedPointPlan,
    imported: &SourceBuiltFixedPointPlan,
    import_root: &Path,
) -> Result<(), RunError> {
    let is_supported_schema =
        imported.schema == current.schema || imported.schema == LEGACY_PROVIDER_CHECKPOINT_IMPORT_PLAN_SCHEMA;
    if !is_supported_schema
        || current.logical_store_prefix != imported.logical_store_prefix
        || current.resource_bounds != imported.resource_bounds
        || !provider_policies_match(current, imported)
    {
        return Err(proof_error("imported provider plan policy or resource authority mismatch".to_string()));
    }
    validate_imported_provider_sources(current, imported, import_root)?;
    for (current_stage, imported_stage) in current
        .stages
        .iter()
        .take(crate::source_built_fixed_point_checkpoint::PROVIDER_CHECKPOINT_STAGE_COUNT)
        .zip(imported.stages.iter())
    {
        if current_stage.stage_id != imported_stage.stage_id || current_stage.output != imported_stage.output {
            return Err(proof_error("imported provider stage order or output role mismatch".to_string()));
        }
    }
    assert_eq!(current.logical_store_prefix, imported.logical_store_prefix);
    Ok(())
}

fn provider_policies_match(current: &SourceBuiltFixedPointPlan, imported: &SourceBuiltFixedPointPlan) -> bool {
    let left = &current.policies;
    let right = &imported.policies;
    left.expected_native_provider_digest_blake3 == right.expected_native_provider_digest_blake3
        && left.closure_policy_digest_blake3 == right.closure_policy_digest_blake3
        && left.hermeticity_policy_digest_blake3 == right.hermeticity_policy_digest_blake3
        && left.protected_execution_policy_digest_blake3 == right.protected_execution_policy_digest_blake3
        && left.effect_policy_digest_blake3 == right.effect_policy_digest_blake3
        && left.hermeticity_mode == right.hermeticity_mode
        && !right.live_fetch_allowed
        && !right.cargo_invocation_allowed
        && !right.ambient_discovery_allowed
        && !right.fallback_allowed
        && !right.provider_cache_completion_allowed
}

fn validate_imported_provider_sources(
    current: &SourceBuiltFixedPointPlan,
    imported: &SourceBuiltFixedPointPlan,
    import_root: &Path,
) -> Result<(), RunError> {
    let roles = [
        SourceAuthorityRole::StagexSeed,
        SourceAuthorityRole::StagexLineage,
        SourceAuthorityRole::StagexSourceBundle,
        SourceAuthorityRole::NativeSourceBundle,
        SourceAuthorityRole::RustSourceArchiveSet,
    ];
    for role in roles {
        let current_input = plan_source_input(current, role)?;
        let imported_input = plan_source_input(imported, role)?;
        if current_input.kind != imported_input.kind
            || current_input.digest_blake3 != imported_input.digest_blake3
            || current_input.size_bytes != imported_input.size_bytes
        {
            return Err(proof_error(format!("imported provider source authority mismatch for {role:?}")));
        }
    }
    let current_projection = plan_source_input(current, SourceAuthorityRole::ProviderRecipeProjection)?;
    let imported_projection = imported
        .source_inputs
        .iter()
        .find(|input| input.role == SourceAuthorityRole::ProviderRecipeProjection)
        .map(|input| (input.size_bytes, input.digest_blake3.clone()))
        .map_or_else(|| hash_provider_recipe_projection(&import_root.join(INPUTS_DIR).join(SOURCE_ROOT_DIR)), Ok)?;
    if current_projection.size_bytes != imported_projection.0
        || current_projection.digest_blake3 != imported_projection.1
    {
        return Err(proof_error("imported provider recipe projection mismatch".to_string()));
    }
    assert_eq!(roles.len(), IMPORTED_PROVIDER_SOURCE_ROLE_COUNT);
    Ok(())
}

fn plan_source_input(
    plan: &SourceBuiltFixedPointPlan,
    role: SourceAuthorityRole,
) -> Result<&SourceAuthorityInput, RunError> {
    let mut matches = plan.source_inputs.iter().filter(|input| input.role == role);
    let input = matches.next().ok_or_else(|| proof_error(format!("plan is missing source authority {role:?}")))?;
    if matches.next().is_some() {
        return Err(proof_error(format!("plan duplicates source authority {role:?}")));
    }
    Ok(input)
}

fn imported_constructed_providers(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    import_root: &Path,
) -> Result<ConstructedProviders, RunError> {
    let imported_admission_path = import_root.join(NATIVE_ADMISSION_REPORT_FILE);
    let imported_admission: crate::full_source_provider::FullSourceProviderAdmissionReport =
        serde_json::from_slice(&fs::read(&imported_admission_path).map_err(|error| {
            proof_error(format!("reading imported native admission {}: {error}", imported_admission_path.display()))
        })?)
        .map_err(|error| proof_error(format!("parsing imported native admission: {error}")))?;
    let native_basename = required_utf8_basename(&imported_admission.provider_path, "imported native provider")?;
    let native_path = import_root.join(NATIVE_STORE_DIR).join(native_basename);
    let native_source_manifest = import_root.join(NATIVE_SOURCE_MANIFEST_FILE);
    let revalidation_path = prepared.staging_dir.join("imported-native-provider-revalidation.json");
    let native_admission = crate::full_source_provider::cmd_admit_full_source_provider(
        &native_path,
        options.expected_native_provider_blake3,
        &native_source_manifest,
        &crate::source_bundle::read_source_bundle(&native_source_manifest)?.manifest_blake3,
        &revalidation_path,
        false,
    )?;
    let native_transcript = import_root.join(TRANSCRIPTS_DIR).join(format!("{NATIVE_PROVIDER_ID}.json"));
    let native_provider = imported_native_observation(&native_path, &native_transcript, &imported_admission_path)?;
    let stagex_path = import_root.join(NATIVE_STORE_DIR).join(STAGEX_PROVIDER_STORE_BASENAME);
    let stagex_provider_report = imported_stagex_provider_report(&stagex_path)?;
    let rust_path = import_root.join(RUST_PROVIDER_DIR);
    let rust_validation = crate::rust_source_provider::validate_materialized_rust_source_provider(&rust_path)
        .map_err(|error| proof_error(format!("validating imported Rust provider: {error}")))?;
    let recipe_digest_blake3 = crate::protected_exec::blake3_file_hex(
        &import_root.join(INPUTS_DIR).join(SOURCE_ROOT_DIR).join(RUST_RECIPE_NCL),
    )
    .map_err(|error| proof_error(format!("hashing imported Rust recipe: {error}")))?;
    Ok(ConstructedProviders {
        stagex_transition_execution_dir: import_root.join(STAGEX_TRANSITION_EXECUTION_DIR),
        stagex_provider_report,
        native_provider,
        native_admission,
        native_admission_report_path: imported_admission_path,
        rust_provider: crate::rust_source_provider::RustSourceProviderMaterialization {
            output_path: rust_path,
            recipe_digest_blake3,
            metadata_path: rust_validation.metadata_path,
            metadata_digest_blake3: rust_validation.metadata_digest_blake3,
        },
        toolchain_closure_path: import_root.join(TOOLCHAIN_CLOSURE_FILE),
        provider_checkpoint: None,
    })
}

fn imported_native_observation(
    native_path: &Path,
    transcript_path: &Path,
    admission_path: &Path,
) -> Result<BuildObservation, RunError> {
    if !native_path.is_dir() || !transcript_path.is_file() || !admission_path.is_file() {
        return Err(proof_error("imported native provider evidence is incomplete".to_string()));
    }
    let basename = required_utf8_basename(native_path, "imported native provider")?;
    let transcript_digest_blake3 = crate::protected_exec::blake3_file_hex(transcript_path)
        .map_err(|error| proof_error(format!("hashing imported native transcript: {error}")))?;
    Ok(BuildObservation {
        output: BuildJsonOutput {
            name: BUILD_OUTPUT_NAME.to_string(),
            path: native_path.to_path_buf(),
            artifact_attestation: BuildJsonAttestationReference {
                logical_path: format!("{LOGICAL_STORE_PREFIX}/{basename}"),
                path: admission_path.to_path_buf(),
            },
        },
        transcript_path: transcript_path.to_path_buf(),
        transcript_digest_blake3,
    })
}

fn imported_stagex_provider_report(
    stagex_path: &Path,
) -> Result<crate::stagex_provider::StagexProviderPublicationReport, RunError> {
    if !stagex_path.is_dir() {
        return Err(proof_error(format!("imported StageX provider is missing: {}", stagex_path.display())));
    }
    let receipt_path = stagex_path.join(crate::stagex_provider::PROVIDER_RECEIPT_RELATIVE_PATH);
    let validation_path = stagex_path.join(crate::stagex_provider::PROVIDER_VALIDATION_RELATIVE_PATH);
    let output_digest_blake3 = crate::release_tree_copy::hash_directory_tree(stagex_path)?.1;
    let validation_digest_blake3 = crate::protected_exec::blake3_file_hex(&validation_path)
        .map_err(|error| proof_error(format!("hashing imported StageX validation: {error}")))?;
    if !receipt_path.is_file() {
        return Err(proof_error(format!("imported StageX provider receipt is missing: {}", receipt_path.display())));
    }
    Ok(crate::stagex_provider::StagexProviderPublicationReport {
        schema: "mantle-stagex-provider-publication-v1",
        provider_kind: "stagex-intermediate-provider",
        output_path: stagex_path.to_path_buf(),
        receipt_path,
        normalized_provider_digest_blake3: STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST.to_string(),
        output_digest_blake3: output_digest_blake3.clone(),
        final_bundle_digest_blake3: output_digest_blake3,
        provider_validation_audit_digest_blake3: validation_digest_blake3.clone(),
        provider_validation_report_digest_blake3: validation_digest_blake3,
    })
}

fn write_checkpoint_import_report(
    prepared: &PreparedAttempt,
    import_root: &Path,
    imported_plan: &SourceBuiltFixedPointPlan,
) -> Result<(), RunError> {
    let checkpoint_transcript = prepared.transcripts_dir.join(PROVIDER_CHECKPOINT_TRANSCRIPT_FILE);
    let report = serde_json::json!({
        "schema": "mantle-source-built-provider-checkpoint-import-v1",
        "status": "complete",
        "current_plan_digest_blake3": prepared.plan.plan_digest_blake3,
        "imported_plan_digest_blake3": imported_plan.plan_digest_blake3,
        "imported_attempt": import_root,
        "checkpoint_publication_transcript": checkpoint_transcript,
        "non_claim": "checkpoint import validates completed provider stages; it does not execute them again",
    });
    write_json_create_new(&prepared.staging_dir.join(PROVIDER_CHECKPOINT_IMPORT_REPORT_FILE), &report)
}

pub(super) fn print_checkpoint_import_completion(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
) -> Result<(), RunError> {
    let report_path = prepared.staging_dir.join(PROVIDER_CHECKPOINT_IMPORT_REPORT_FILE);
    let bytes = fs::read(&report_path)
        .map_err(|error| proof_error(format!("reading checkpoint import report {}: {error}", report_path.display())))?;
    if options.json {
        println!("{}", String::from_utf8_lossy(&bytes));
    } else {
        eprintln!("source-built provider checkpoint imported");
        eprintln!("  report: {}", report_path.display());
        eprintln!("  preserved_attempt: {}", prepared.staging_dir.display());
    }
    Ok(())
}

pub(super) fn restore_constructed_provider_checkpoint(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
) -> Result<Option<ConstructedProviders>, RunError> {
    let Some(checkpoint_store) = options.proof_checkpoint_store else {
        return Ok(None);
    };
    let limits = crate::source_built_fixed_point_checkpoint_shell::provider_checkpoint_limits(options.disk_bytes_max);
    let Some(admitted) = crate::source_built_fixed_point_checkpoint_shell::admit_provider_checkpoint_store(
        checkpoint_store,
        &prepared.plan,
        STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST,
        limits,
    )?
    else {
        return Ok(None);
    };
    let stagex_basename = checkpoint_payload_basename(
        &admitted.manifest,
        crate::source_built_fixed_point_checkpoint::PAYLOAD_STAGEX_PROVIDER,
    )?;
    let native_basename = checkpoint_payload_basename(
        &admitted.manifest,
        crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_PROVIDER,
    )?;
    let restored_paths = RestoredProviderPaths::new(prepared, stagex_basename, native_basename)?;
    let restores = restored_paths.requests();
    let restored =
        crate::source_built_fixed_point_checkpoint_shell::restore_provider_checkpoint(admitted, &restores, limits)?;
    let providers = validate_restored_provider_checkpoint(options, prepared, restored_paths, restored)?;
    Ok(Some(providers))
}

struct RestoredProviderPaths {
    stagex_transition: PathBuf,
    stagex_provider: PathBuf,
    native_provider: PathBuf,
    rust_provider: PathBuf,
    origin_native_admission: PathBuf,
    origin_native_transcript: PathBuf,
    origin_toolchain_closure: PathBuf,
    toolchain_closure: PathBuf,
}

impl RestoredProviderPaths {
    fn new(prepared: &PreparedAttempt, stagex_basename: &str, native_basename: &str) -> Result<Self, RunError> {
        let origin = prepared.staging_dir.join(CHECKPOINT_ORIGIN_EVIDENCE_DIR);
        fs::create_dir(&origin).map_err(|error| {
            proof_error(format!("creating checkpoint origin evidence {}: {error}", origin.display()))
        })?;
        Ok(Self {
            stagex_transition: prepared.staging_dir.join(STAGEX_TRANSITION_EXECUTION_DIR),
            stagex_provider: prepared.native_store_dir.join(stagex_basename),
            native_provider: prepared.native_store_dir.join(native_basename),
            rust_provider: prepared.staging_dir.join(RUST_PROVIDER_DIR),
            origin_native_admission: origin.join("native-admission.json"),
            origin_native_transcript: origin.join("native-provider.json"),
            origin_toolchain_closure: origin.join(CHECKPOINT_ORIGIN_TOOLCHAIN_CLOSURE_FILE),
            toolchain_closure: prepared.staging_dir.join(TOOLCHAIN_CLOSURE_FILE),
        })
    }

    fn requests(
        &self,
    ) -> [crate::source_built_fixed_point_checkpoint_shell::CheckpointPayloadRestore;
        crate::source_built_fixed_point_checkpoint::PROVIDER_CHECKPOINT_PAYLOAD_COUNT] {
        use crate::source_built_fixed_point_checkpoint::CheckpointPayloadKind;
        [
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_STAGEX_TRANSITION,
                &self.stagex_transition,
                CheckpointPayloadKind::PreservedTree,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_STAGEX_PROVIDER,
                &self.stagex_provider,
                CheckpointPayloadKind::Directory,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_PROVIDER,
                &self.native_provider,
                CheckpointPayloadKind::Directory,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_PROVIDER,
                &self.rust_provider,
                CheckpointPayloadKind::Directory,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_ADMISSION,
                &self.origin_native_admission,
                CheckpointPayloadKind::RegularFile,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_TRANSCRIPT,
                &self.origin_native_transcript,
                CheckpointPayloadKind::RegularFile,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_TOOLCHAIN_CLOSURE,
                &self.origin_toolchain_closure,
                CheckpointPayloadKind::RegularFile,
            ),
        ]
    }
}

fn checkpoint_restore(
    payload_id: &str,
    destination_path: &Path,
    kind: crate::source_built_fixed_point_checkpoint::CheckpointPayloadKind,
) -> crate::source_built_fixed_point_checkpoint_shell::CheckpointPayloadRestore {
    crate::source_built_fixed_point_checkpoint_shell::CheckpointPayloadRestore {
        payload_id: payload_id.to_string(),
        destination_path: destination_path.to_path_buf(),
        kind,
    }
}

fn checkpoint_payload_basename<'a>(
    manifest: &'a crate::source_built_fixed_point_checkpoint::ProviderCheckpointManifest,
    payload_id: &str,
) -> Result<&'a str, RunError> {
    let payload = manifest
        .payloads
        .iter()
        .find(|payload| payload.payload_id == payload_id)
        .ok_or_else(|| proof_error(format!("checkpoint manifest is missing payload {payload_id}")))?;
    Path::new(&payload.relative_path)
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| proof_error(format!("checkpoint payload {payload_id} has no UTF-8 basename")))
}

fn validate_restored_provider_checkpoint(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    paths: RestoredProviderPaths,
    restored: crate::source_built_fixed_point_checkpoint_shell::RestoredProviderCheckpoint,
) -> Result<ConstructedProviders, RunError> {
    let stagex_logical =
        register_adopted_provider(&paths.stagex_provider, &prepared.native_store_dir, &prepared.native_state_dir)?;
    if stagex_logical != STAGEX_PROVIDER_LOGICAL_PATH {
        return Err(proof_error(format!(
            "restored StageX provider logical path mismatch: expected {STAGEX_PROVIDER_LOGICAL_PATH}, observed {stagex_logical}"
        )));
    }
    let native_logical =
        register_adopted_provider(&paths.native_provider, &prepared.native_store_dir, &prepared.native_state_dir)?;
    let native_admission_report_path = prepared.staging_dir.join(NATIVE_ADMISSION_REPORT_FILE);
    let native_admission = crate::full_source_provider::cmd_admit_full_source_provider(
        &paths.native_provider,
        options.expected_native_provider_blake3,
        &prepared.native_source_manifest,
        &crate::source_bundle::read_source_bundle(&prepared.native_source_manifest)?.manifest_blake3,
        &native_admission_report_path,
        false,
    )?;
    let rust_validation = crate::rust_source_provider::validate_materialized_rust_source_provider(&paths.rust_provider)
        .map_err(|error| proof_error(format!("validating restored Rust provider: {error}")))?;
    let recipe_digest = crate::protected_exec::blake3_file_hex(&prepared.source_root.join(RUST_RECIPE_NCL))
        .map_err(|error| proof_error(format!("hashing restored Rust provider recipe: {error}")))?;
    materialize_relocated_toolchain_closure(prepared, &paths, &restored)?;
    let native_provider = restored_native_provider_observation(
        prepared,
        &paths.native_provider,
        &native_logical,
        &paths.origin_native_admission,
        &restored,
    )?;
    let stagex_provider_report = restored_stagex_provider_report(&paths.stagex_provider, &restored)?;
    write_checkpoint_restore_transcript(prepared, &restored)?;
    Ok(ConstructedProviders {
        stagex_transition_execution_dir: paths.stagex_transition,
        stagex_provider_report,
        native_provider,
        native_admission,
        native_admission_report_path,
        rust_provider: crate::rust_source_provider::RustSourceProviderMaterialization {
            output_path: paths.rust_provider,
            recipe_digest_blake3: recipe_digest,
            metadata_path: rust_validation.metadata_path,
            metadata_digest_blake3: rust_validation.metadata_digest_blake3,
        },
        toolchain_closure_path: paths.toolchain_closure,
        provider_checkpoint: Some(restored),
    })
}

fn materialize_relocated_toolchain_closure(
    prepared: &PreparedAttempt,
    paths: &RestoredProviderPaths,
    restored: &crate::source_built_fixed_point_checkpoint_shell::RestoredProviderCheckpoint,
) -> Result<(), RunError> {
    crate::native_toolchain_closure::cmd_materialize_native_toolchain_closure(NativeToolchainClosureOptions {
        rust_source_provider: &paths.rust_provider,
        host_root: &paths.native_provider,
        target_root: &paths.native_provider,
        output: &paths.toolchain_closure,
    })?;
    let origin = read_toolchain_closure_manifest(&paths.origin_toolchain_closure)?;
    let relocated = read_toolchain_closure_manifest(&paths.toolchain_closure)?;
    let validation = crate::source_toolchain_closure::validate_relocated_toolchain_closure(&origin, &relocated)
        .map_err(|error| proof_error(format!("validating restored toolchain closure relocation: {error}")))?;
    let required_count = crate::source_toolchain_closure::required_native_closure_member_names().len();
    if validation.member_count != required_count {
        return Err(proof_error(format!(
            "restored toolchain closure relocation has {} members, expected {required_count}",
            validation.member_count
        )));
    }
    write_toolchain_closure_relocation_report(prepared, paths, restored, &validation)
}

fn read_toolchain_closure_manifest(
    path: &Path,
) -> Result<crate::source_toolchain_closure::ToolchainClosureManifest, RunError> {
    let bytes = fs::read(path)
        .map_err(|error| proof_error(format!("reading toolchain closure {}: {error}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| proof_error(format!("parsing toolchain closure {}: {error}", path.display())))
}

fn write_toolchain_closure_relocation_report(
    prepared: &PreparedAttempt,
    paths: &RestoredProviderPaths,
    restored: &crate::source_built_fixed_point_checkpoint_shell::RestoredProviderCheckpoint,
    validation: &crate::source_toolchain_closure::ToolchainClosureRelocationValidation,
) -> Result<(), RunError> {
    let origin_digest = crate::protected_exec::blake3_file_hex(&paths.origin_toolchain_closure)
        .map_err(|error| proof_error(format!("hashing checkpoint origin closure: {error}")))?;
    let relocated_digest = crate::protected_exec::blake3_file_hex(&paths.toolchain_closure)
        .map_err(|error| proof_error(format!("hashing relocated checkpoint closure: {error}")))?;
    let report = serde_json::json!({
        "schema": CHECKPOINT_CLOSURE_RELOCATION_REPORT_SCHEMA,
        "checkpoint_digest_blake3": restored.admission.checkpoint_digest_blake3,
        "origin_closure": {
            "path": paths.origin_toolchain_closure,
            "digest_blake3": origin_digest,
            "policy_digest_blake3": validation.origin_policy_digest_blake3,
        },
        "relocated_closure": {
            "path": paths.toolchain_closure,
            "digest_blake3": relocated_digest,
            "policy_digest_blake3": validation.relocated_policy_digest_blake3,
        },
        "member_count": validation.member_count,
        "non_claim": "closure relocation changes only absolute provider roots and does not repeat provider execution",
    });
    write_json_create_new(&prepared.staging_dir.join(CHECKPOINT_CLOSURE_RELOCATION_REPORT_FILE), &report)
}

fn restored_native_provider_observation(
    prepared: &PreparedAttempt,
    native_provider_path: &Path,
    native_logical_path: &str,
    origin_admission_path: &Path,
    restored: &crate::source_built_fixed_point_checkpoint_shell::RestoredProviderCheckpoint,
) -> Result<BuildObservation, RunError> {
    let transcript_path = prepared.transcripts_dir.join("native-provider.checkpoint-restored.txt");
    let text = format!(
        "checkpoint-restored native_provider={} logical={} checkpoint_digest_blake3={} origin_admission={}\n",
        native_provider_path.display(),
        native_logical_path,
        restored.admission.checkpoint_digest_blake3,
        origin_admission_path.display(),
    );
    write_bytes_create_new(&transcript_path, text.as_bytes())?;
    let transcript_digest_blake3 = crate::protected_exec::blake3_file_hex(&transcript_path)
        .map_err(|error| proof_error(format!("hashing restored native transcript: {error}")))?;
    Ok(BuildObservation {
        output: BuildJsonOutput {
            name: BUILD_OUTPUT_NAME.to_string(),
            path: native_provider_path.to_path_buf(),
            artifact_attestation: BuildJsonAttestationReference {
                logical_path: native_logical_path.to_string(),
                path: origin_admission_path.to_path_buf(),
            },
        },
        transcript_path,
        transcript_digest_blake3,
    })
}

fn restored_stagex_provider_report(
    stagex_provider_path: &Path,
    restored: &crate::source_built_fixed_point_checkpoint_shell::RestoredProviderCheckpoint,
) -> Result<crate::stagex_provider::StagexProviderPublicationReport, RunError> {
    let record = restored
        .manifest
        .stages
        .iter()
        .find(|stage| stage.output_role == ProofOutputRole::StagexProvider)
        .ok_or_else(|| proof_error("checkpoint is missing StageX provider stage evidence".to_string()))?;
    let validation = stagex_provider_path.join(crate::stagex_provider::PROVIDER_VALIDATION_RELATIVE_PATH);
    let validation_digest = crate::protected_exec::blake3_file_hex(&validation)
        .map_err(|error| proof_error(format!("hashing restored StageX validation: {error}")))?;
    Ok(crate::stagex_provider::StagexProviderPublicationReport {
        schema: "mantle-stagex-provider-publication-v1",
        provider_kind: "stagex-intermediate-provider",
        output_path: stagex_provider_path.to_path_buf(),
        receipt_path: stagex_provider_path.join(crate::stagex_provider::PROVIDER_RECEIPT_RELATIVE_PATH),
        normalized_provider_digest_blake3: record.semantic_output_digest_blake3.clone(),
        output_digest_blake3: record.output_digest_blake3.clone(),
        final_bundle_digest_blake3: record.output_digest_blake3.clone(),
        provider_validation_audit_digest_blake3: validation_digest.clone(),
        provider_validation_report_digest_blake3: validation_digest,
    })
}

fn write_checkpoint_restore_transcript(
    prepared: &PreparedAttempt,
    restored: &crate::source_built_fixed_point_checkpoint_shell::RestoredProviderCheckpoint,
) -> Result<(), RunError> {
    let transcript = prepared.transcripts_dir.join(PROVIDER_CHECKPOINT_RESTORE_TRANSCRIPT_FILE);
    let text = format!(
        "schema=mantle-source-built-provider-checkpoint-restore-v1\ncheckpoint_digest_blake3={}\nlookup_key_blake3={}\ncheckpoint_root={}\nrestored_stage_count={}\n",
        restored.admission.checkpoint_digest_blake3,
        restored.admission.lookup_key_blake3,
        restored.checkpoint_root.display(),
        restored.admission.completed_stage_count,
    );
    write_bytes_create_new(&transcript, text.as_bytes())
}

pub(super) fn publish_constructed_provider_checkpoint(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    providers: &ConstructedProviders,
) -> Result<(), RunError> {
    let checkpoint_store = options
        .proof_checkpoint_store
        .ok_or_else(|| proof_error("proof checkpoint store is unset during publication".to_string()))?;
    let limits = crate::source_built_fixed_point_checkpoint_shell::provider_checkpoint_limits(options.disk_bytes_max);
    let stage_observations = provider_checkpoint_stage_observations(prepared, providers, limits)?;
    let payload_sources = provider_checkpoint_payload_sources(providers)?;
    let published = crate::source_built_fixed_point_checkpoint_shell::publish_provider_checkpoint(
        checkpoint_store,
        &prepared.plan,
        stage_observations,
        &payload_sources,
        STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST,
        limits,
    )?;
    write_checkpoint_publication_transcript(prepared, &published)
}

fn provider_checkpoint_payload_sources(
    providers: &ConstructedProviders,
) -> Result<
    [crate::source_built_fixed_point_checkpoint_shell::CheckpointPayloadSource;
        crate::source_built_fixed_point_checkpoint::PROVIDER_CHECKPOINT_PAYLOAD_COUNT],
    RunError,
> {
    use crate::source_built_fixed_point_checkpoint::CheckpointPayloadKind;
    let stagex_basename = required_utf8_basename(&providers.stagex_provider_report.output_path, "StageX provider")?;
    let native_basename = required_utf8_basename(&providers.native_provider.output.path, "native provider")?;
    let stagex_relative = PathBuf::from(CHECKPOINT_STAGEX_PROVIDER_PREFIX).join(stagex_basename);
    let native_relative = PathBuf::from(CHECKPOINT_NATIVE_PROVIDER_PREFIX).join(native_basename);
    Ok([
        checkpoint_payload_source(
            crate::source_built_fixed_point_checkpoint::PAYLOAD_STAGEX_TRANSITION,
            &providers.stagex_transition_execution_dir,
            Path::new(CHECKPOINT_STAGEX_TRANSITION_PATH),
            CheckpointPayloadKind::PreservedTree,
        ),
        checkpoint_payload_source(
            crate::source_built_fixed_point_checkpoint::PAYLOAD_STAGEX_PROVIDER,
            &providers.stagex_provider_report.output_path,
            &stagex_relative,
            CheckpointPayloadKind::Directory,
        ),
        checkpoint_payload_source(
            crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_PROVIDER,
            &providers.native_provider.output.path,
            &native_relative,
            CheckpointPayloadKind::Directory,
        ),
        checkpoint_payload_source(
            crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_PROVIDER,
            &providers.rust_provider.output_path,
            Path::new(CHECKPOINT_RUST_PROVIDER_PATH),
            CheckpointPayloadKind::Directory,
        ),
        checkpoint_payload_source(
            crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_ADMISSION,
            &providers.native_admission_report_path,
            Path::new(CHECKPOINT_NATIVE_ADMISSION_PATH),
            CheckpointPayloadKind::RegularFile,
        ),
        checkpoint_payload_source(
            crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_TRANSCRIPT,
            &providers.native_provider.transcript_path,
            Path::new(CHECKPOINT_NATIVE_TRANSCRIPT_PATH),
            CheckpointPayloadKind::RegularFile,
        ),
        checkpoint_payload_source(
            crate::source_built_fixed_point_checkpoint::PAYLOAD_TOOLCHAIN_CLOSURE,
            &providers.toolchain_closure_path,
            Path::new(CHECKPOINT_TOOLCHAIN_CLOSURE_PATH),
            CheckpointPayloadKind::RegularFile,
        ),
    ])
}

fn checkpoint_payload_source(
    payload_id: &str,
    source_path: &Path,
    relative_path: &Path,
    kind: crate::source_built_fixed_point_checkpoint::CheckpointPayloadKind,
) -> crate::source_built_fixed_point_checkpoint_shell::CheckpointPayloadSource {
    crate::source_built_fixed_point_checkpoint_shell::CheckpointPayloadSource {
        payload_id: payload_id.to_string(),
        source_path: source_path.to_path_buf(),
        relative_path: relative_path.to_path_buf(),
        kind,
    }
}

fn provider_checkpoint_stage_observations(
    prepared: &PreparedAttempt,
    providers: &ConstructedProviders,
    limits: crate::source_built_fixed_point_checkpoint_shell::ProviderCheckpointLimits,
) -> Result<Vec<crate::source_built_fixed_point_checkpoint::ProviderCheckpointStageObservation>, RunError> {
    let current_executable = std::env::current_exe()
        .map_err(|error| proof_error(format!("resolving checkpoint producer executable: {error}")))?;
    let producer_digest = crate::protected_exec::blake3_file_hex(&current_executable)
        .map_err(|error| proof_error(format!("hashing checkpoint producer executable: {error}")))?;
    let transition = crate::preserved_evidence_tree::hash_preserved_evidence_tree(
        &providers.stagex_transition_execution_dir,
        limits.preserved_tree,
    )?;
    let stagex_payload_digest =
        crate::release_tree_copy::hash_directory_tree(&providers.stagex_provider_report.output_path)?.1;
    let native_payload_digest =
        crate::release_tree_copy::hash_directory_tree(&providers.native_provider.output.path)?.1;
    let rust_provider_digest = crate::release_tree_copy::hash_directory_tree(&providers.rust_provider.output_path)?.1;
    let evidence = provider_checkpoint_evidence_digests(providers)?;
    let observations = vec![
        provider_stage_observation(
            ProofOutputRole::StagexTransition,
            &transition.digest_blake3,
            &transition.digest_blake3,
            &transition.digest_blake3,
            &evidence.transition,
            &producer_digest,
        ),
        provider_stage_observation(
            ProofOutputRole::StagexProvider,
            &providers.stagex_provider_report.final_bundle_digest_blake3,
            &providers.stagex_provider_report.normalized_provider_digest_blake3,
            &stagex_payload_digest,
            &evidence.stagex_provider,
            &producer_digest,
        ),
        provider_stage_observation(
            ProofOutputRole::FullSourceNativeProvider,
            &providers.native_admission.output_digest_blake3,
            &providers.native_admission.output_digest_blake3,
            &native_payload_digest,
            &evidence.native_provider,
            &producer_digest,
        ),
        provider_stage_observation(
            ProofOutputRole::FullSourceRustProvider,
            &rust_provider_digest,
            &rust_provider_digest,
            &rust_provider_digest,
            &evidence.rust_provider,
            &producer_digest,
        ),
    ];
    assert_eq!(observations.len(), crate::source_built_fixed_point_checkpoint::PROVIDER_CHECKPOINT_STAGE_COUNT);
    debug_assert_eq!(prepared.plan.stages.len(), EXPECTED_STAGE_COUNT);
    Ok(observations)
}

struct ProviderCheckpointEvidenceDigests {
    transition: String,
    stagex_provider: String,
    native_provider: String,
    rust_provider: String,
}

fn provider_checkpoint_evidence_digests(
    providers: &ConstructedProviders,
) -> Result<ProviderCheckpointEvidenceDigests, RunError> {
    let transition = checkpoint_evidence_digest(&[
        ("transition-report", providers.stagex_transition_execution_dir.join(STAGEX_TRANSITION_REPORT_FILE)),
        ("transition-audit", providers.stagex_transition_execution_dir.join(STAGEX_TRANSITION_AUDIT_FILE)),
    ])?;
    let stagex_validation = providers
        .stagex_provider_report
        .output_path
        .join(crate::stagex_provider::PROVIDER_VALIDATION_RELATIVE_PATH);
    let stagex_provider = checkpoint_evidence_digest(&[
        ("stagex-receipt", providers.stagex_provider_report.receipt_path.clone()),
        ("stagex-validation", stagex_validation),
    ])?;
    let native_provider = checkpoint_evidence_digest(&[
        ("native-transcript", providers.native_provider.transcript_path.clone()),
        ("native-admission", providers.native_admission_report_path.clone()),
    ])?;
    let rust_provider = checkpoint_evidence_digest(&[
        ("rust-build-receipt", providers.rust_provider.output_path.join(RUST_PROVIDER_BUILD_RECEIPT_RELATIVE)),
        (
            "rust-binding-receipt",
            providers.rust_provider.output_path.join(RUST_PROVIDER_BINDING_RECEIPT_RELATIVE),
        ),
        ("rust-metadata", providers.rust_provider.metadata_path.clone()),
    ])?;
    Ok(ProviderCheckpointEvidenceDigests {
        transition,
        stagex_provider,
        native_provider,
        rust_provider,
    })
}

fn provider_stage_observation(
    output_role: ProofOutputRole,
    output_digest_blake3: &str,
    semantic_output_digest_blake3: &str,
    payload_digest_blake3: &str,
    execution_evidence_digest_blake3: &str,
    producer_executable_digest_blake3: &str,
) -> crate::source_built_fixed_point_checkpoint::ProviderCheckpointStageObservation {
    crate::source_built_fixed_point_checkpoint::ProviderCheckpointStageObservation {
        output_role,
        output_digest_blake3: output_digest_blake3.to_string(),
        semantic_output_digest_blake3: semantic_output_digest_blake3.to_string(),
        payload_digest_blake3: payload_digest_blake3.to_string(),
        execution_evidence_digest_blake3: execution_evidence_digest_blake3.to_string(),
        producer_executable_digest_blake3: producer_executable_digest_blake3.to_string(),
    }
}

fn checkpoint_evidence_digest(files: &[(&str, PathBuf)]) -> Result<String, RunError> {
    let mut hasher = blake3::Hasher::new_derive_key(CHECKPOINT_EXECUTION_EVIDENCE_CONTEXT);
    for (label, path) in files {
        if label.is_empty() {
            return Err(proof_error("checkpoint evidence label is empty".to_string()));
        }
        let digest = crate::protected_exec::blake3_file_hex(path)
            .map_err(|error| proof_error(format!("hashing checkpoint evidence {}: {error}", path.display())))?;
        hasher.update(label.as_bytes());
        hasher.update(b"\0");
        hasher.update(digest.as_bytes());
        hasher.update(b"\n");
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert!(!files.is_empty());
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    Ok(digest)
}

fn required_utf8_basename<'a>(path: &'a Path, label: &str) -> Result<&'a str, RunError> {
    path.file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| proof_error(format!("{label} path has no UTF-8 basename: {}", path.display())))
}

fn write_checkpoint_publication_transcript(
    prepared: &PreparedAttempt,
    published: &crate::source_built_fixed_point_checkpoint_shell::PublishedProviderCheckpoint,
) -> Result<(), RunError> {
    let transcript = prepared.transcripts_dir.join(PROVIDER_CHECKPOINT_TRANSCRIPT_FILE);
    let text = format!(
        "schema=mantle-source-built-provider-checkpoint-publication-v1\ndisposition={:?}\nlookup_key_blake3={}\ncheckpoint_digest_blake3={}\ncheckpoint_root={}\nmanifest_path={}\n",
        published.disposition,
        published.lookup_key_blake3,
        published.checkpoint_digest_blake3,
        published.checkpoint_root.display(),
        published.manifest_path.display(),
    );
    write_bytes_create_new(&transcript, text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imported_stagex_provider_report_uses_the_published_receipt_path() {
        let temp = tempfile::tempdir().expect("temporary provider root");
        let provider_root = temp.path().join("stagex-provider");
        let receipt_path = provider_root.join(crate::stagex_provider::PROVIDER_RECEIPT_RELATIVE_PATH);
        let validation_path = provider_root.join(crate::stagex_provider::PROVIDER_VALIDATION_RELATIVE_PATH);
        fs::create_dir_all(receipt_path.parent().expect("receipt parent")).expect("provider evidence directory");
        fs::write(&receipt_path, b"{}\n").expect("provider receipt");
        fs::write(&validation_path, b"{}\n").expect("provider validation");

        let report = imported_stagex_provider_report(&provider_root).expect("imported provider report");

        assert_eq!(report.receipt_path, receipt_path);
        assert!(report.receipt_path.is_file());
        assert_ne!(report.receipt_path, provider_root.join("provider-receipt.json"));
    }

    #[test]
    fn restored_paths_keep_the_checkpoint_closure_as_origin_evidence() {
        let paths = RestoredProviderPaths {
            stagex_transition: PathBuf::from("/proof/stagex-transition"),
            stagex_provider: PathBuf::from("/proof/native-store/stagex"),
            native_provider: PathBuf::from("/proof/native-store/native"),
            rust_provider: PathBuf::from("/proof/rust-provider"),
            origin_native_admission: PathBuf::from("/proof/origin/native-admission.json"),
            origin_native_transcript: PathBuf::from("/proof/origin/native-provider.json"),
            origin_toolchain_closure: PathBuf::from("/proof/origin/source-built-toolchain-closure.json"),
            toolchain_closure: PathBuf::from("/proof/source-built-toolchain-closure.json"),
        };

        let requests = paths.requests();
        let closure = requests
            .iter()
            .find(|request| request.payload_id == crate::source_built_fixed_point_checkpoint::PAYLOAD_TOOLCHAIN_CLOSURE)
            .expect("toolchain closure restore request");

        assert_eq!(closure.destination_path, paths.origin_toolchain_closure);
        assert_ne!(closure.destination_path, paths.toolchain_closure);
        assert_eq!(closure.kind, crate::source_built_fixed_point_checkpoint::CheckpointPayloadKind::RegularFile);
    }

    #[test]
    fn imported_stagex_provider_report_rejects_the_legacy_root_receipt_path() {
        let temp = tempfile::tempdir().expect("temporary provider root");
        let provider_root = temp.path().join("stagex-provider");
        let validation_path = provider_root.join(crate::stagex_provider::PROVIDER_VALIDATION_RELATIVE_PATH);
        fs::create_dir_all(validation_path.parent().expect("validation parent")).expect("provider evidence directory");
        fs::write(provider_root.join("provider-receipt.json"), b"{}\n").expect("legacy root receipt");
        fs::write(&validation_path, b"{}\n").expect("provider validation");

        let error = imported_stagex_provider_report(&provider_root).expect_err("canonical receipt must be required");
        let message = error.to_string();

        assert!(message.contains("imported StageX provider receipt is missing"));
        assert!(message.contains(crate::stagex_provider::PROVIDER_RECEIPT_RELATIVE_PATH));
        assert!(!provider_root.join(crate::stagex_provider::PROVIDER_RECEIPT_RELATIVE_PATH).exists());
    }
}
