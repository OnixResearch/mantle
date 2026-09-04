use super::*;

const CHECKPOINT_ORIGIN_TOOLCHAIN_CLOSURE_FILE: &str = "source-built-toolchain-closure.json";
const CHECKPOINT_CLOSURE_RELOCATION_REPORT_FILE: &str = "provider-checkpoint-closure-relocation.json";
const CHECKPOINT_CLOSURE_RELOCATION_REPORT_SCHEMA: &str = "mantle-source-built-checkpoint-closure-relocation-v1";
const CHECKPOINT_RUST_BINDING_RELOCATION_REPORT_FILE: &str = "provider-checkpoint-rust-binding-relocation.json";
const CHECKPOINT_RUST_BINDING_RELOCATION_REPORT_SCHEMA: &str =
    "mantle-source-built-checkpoint-rust-binding-relocation-v1";
const IMPORTED_NATIVE_PREFIX_DIR: &str = "imported-native-prefix";
const IMPORTED_NATIVE_ORIGIN_REVALIDATION_FILE: &str = "imported-native-provider-origin-revalidation.json";
const IMPORTED_NATIVE_MATERIALIZATION_REPORT_FILE: &str = "imported-native-provider-materialization.json";
const IMPORTED_NATIVE_MATERIALIZATION_REPORT_SCHEMA: &str = "mantle-source-built-native-provider-materialization-v1";

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

pub(super) fn restore_native_provider_prefix_attempt(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    import_root: &Path,
) -> Result<NativeProviderPrefix, RunError> {
    let imported_plan = read_imported_attempt_plan(import_root)?;
    validate_imported_attempt_status(import_root, &imported_plan)?;
    validate_imported_provider_authority(&prepared.plan, &imported_plan, import_root)?;
    let prefix = imported_native_provider_prefix(options, prepared, import_root)?;
    let report = serde_json::json!({
        "schema": "mantle-source-built-native-prefix-reuse-v1",
        "status": "complete",
        "current_plan_digest_blake3": prepared.plan.plan_digest_blake3,
        "imported_plan_digest_blake3": imported_plan.plan_digest_blake3,
        "imported_attempt": import_root,
        "non_claim": "native-prefix reuse validates StageX, native, host-tool, and action evidence; it does not claim Rust-provider completion",
    });
    write_json_create_new(&prepared.staging_dir.join("native-prefix-reuse.json"), &report)?;
    Ok(prefix)
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

fn imported_native_provider_prefix(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    import_root: &Path,
) -> Result<NativeProviderPrefix, RunError> {
    let imported_admission_path = import_root.join(NATIVE_ADMISSION_REPORT_FILE);
    let imported_admission: crate::full_source_provider::FullSourceProviderAdmissionReport =
        serde_json::from_slice(&fs::read(&imported_admission_path).map_err(|error| {
            proof_error(format!("reading imported native admission {}: {error}", imported_admission_path.display()))
        })?)
        .map_err(|error| proof_error(format!("parsing imported native admission: {error}")))?;
    let native_basename = required_utf8_basename(&imported_admission.provider_path, "imported native provider")?;
    let origin_native_path = import_root.join(NATIVE_STORE_DIR).join(native_basename);
    let native_source_manifest = import_root.join(NATIVE_SOURCE_MANIFEST_FILE);
    let (native_path, native_admission, revalidation_path) = materialize_imported_native_provider(
        options,
        prepared,
        &origin_native_path,
        &native_source_manifest,
        native_basename,
    )?;
    let native_transcript = import_root.join(TRANSCRIPTS_DIR).join(format!("{NATIVE_PROVIDER_ID}.json"));
    let native_provider = imported_native_observation(&native_path, &native_transcript, &revalidation_path)?;
    let stagex_path = import_root.join(NATIVE_STORE_DIR).join(STAGEX_PROVIDER_STORE_BASENAME);
    let stagex_provider_report = imported_stagex_provider_report(&stagex_path)?;
    let rust_host_tools = imported_host_tool_observations(import_root)?;
    let rust_host_tool_evidence_dir = import_root.join(HOST_TOOLS_EVIDENCE_DIR);
    let host_tool_manifest_path = rust_host_tool_evidence_dir
        .join(crate::full_source_rust_binding_shell::FULL_SOURCE_RUST_HOST_TOOL_MANIFEST_FILE);
    let _host_tool_observation = crate::full_source_rust_binding_shell::observe_full_source_rust_host_tools(
        &host_tool_manifest_path,
        &native_admission.output_digest_blake3,
    )
    .map_err(|error| proof_error(format!("validating imported Rust host-tool evidence: {error}")))?;
    let native_action_trust = imported_native_action_trust(import_root)?;
    Ok(NativeProviderPrefix {
        stagex_transition_execution_dir: import_root.join(STAGEX_TRANSITION_EXECUTION_DIR),
        stagex_provider_report,
        native_provider,
        native_action_trust: Some(native_action_trust),
        native_admission,
        native_admission_report_path: revalidation_path,
        rust_host_tools,
        rust_host_tool_evidence_dir,
        rust_host_tool_manifest_path: host_tool_manifest_path,
    })
}

fn materialize_imported_native_provider(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    origin_native_path: &Path,
    native_source_manifest: &Path,
    native_basename: &str,
) -> Result<(PathBuf, crate::full_source_provider::FullSourceProviderAdmissionReport, PathBuf), RunError> {
    debug_assert!(origin_native_path.is_absolute());
    debug_assert!(prepared.staging_dir.is_absolute());
    let source_closure_blake3 = crate::source_bundle::read_source_bundle(native_source_manifest)?.manifest_blake3;
    let origin_revalidation_path = prepared.staging_dir.join(IMPORTED_NATIVE_ORIGIN_REVALIDATION_FILE);
    let origin_admission = crate::full_source_provider::cmd_admit_full_source_provider(
        origin_native_path,
        options.expected_native_provider_blake3,
        native_source_manifest,
        &source_closure_blake3,
        &origin_revalidation_path,
        false,
    )?;
    let materialized_root = prepared.staging_dir.join(IMPORTED_NATIVE_PREFIX_DIR).join(NATIVE_STORE_DIR);
    let materialized_path = materialized_root.join(native_basename);
    let materialized_identity = copy_imported_native_provider_tree(origin_native_path, &materialized_path)?;
    if materialized_identity.1 != origin_admission.output_digest_blake3 {
        return Err(proof_error("isolated imported native provider copy changed provider identity".to_string()));
    }
    let revalidation_path = prepared.staging_dir.join("imported-native-provider-revalidation.json");
    let admission = crate::full_source_provider::cmd_admit_full_source_provider(
        &materialized_path,
        options.expected_native_provider_blake3,
        native_source_manifest,
        &source_closure_blake3,
        &revalidation_path,
        false,
    )?;
    let origin_identity = crate::release_tree_copy::hash_directory_tree(origin_native_path)
        .map_err(|error| proof_error(format!("rehashing imported native provider origin: {error}")))?;
    if origin_identity.1 != origin_admission.output_digest_blake3 {
        return Err(proof_error("imported native provider origin changed during isolated materialization".to_string()));
    }
    write_imported_native_materialization_report(
        prepared,
        origin_native_path,
        &materialized_path,
        &origin_revalidation_path,
        &revalidation_path,
        &admission.output_digest_blake3,
    )?;
    Ok((materialized_path, admission, revalidation_path))
}

fn copy_imported_native_provider_tree(origin: &Path, materialized: &Path) -> Result<(u64, String), RunError> {
    debug_assert!(!origin.as_os_str().is_empty());
    debug_assert_ne!(origin, materialized);
    crate::release_tree_copy::copy_directory_tree(origin, materialized)
        .map_err(|error| proof_error(format!("copying imported native provider into isolated staging: {error}")))?;
    crate::release_tree_copy::hash_directory_tree(materialized)
        .map_err(|error| proof_error(format!("hashing isolated imported native provider: {error}")))
}

fn write_imported_native_materialization_report(
    prepared: &PreparedAttempt,
    origin: &Path,
    materialized: &Path,
    origin_revalidation: &Path,
    materialized_revalidation: &Path,
    output_digest_blake3: &str,
) -> Result<(), RunError> {
    assert_eq!(output_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    debug_assert_ne!(origin, materialized);
    let report = serde_json::json!({
        "schema": IMPORTED_NATIVE_MATERIALIZATION_REPORT_SCHEMA,
        "status": "complete",
        "origin": origin,
        "origin_revalidation": origin_revalidation,
        "materialized": materialized,
        "materialized_revalidation": materialized_revalidation,
        "output_digest_blake3": output_digest_blake3,
        "copy_semantics": "bounded-no-follow-byte-copy",
        "non_claim": "isolated materialization preserves validated provider identity without repeating provider construction",
    });
    write_json_create_new(&prepared.staging_dir.join(IMPORTED_NATIVE_MATERIALIZATION_REPORT_FILE), &report)
}

fn imported_host_tool_observations(import_root: &Path) -> Result<BTreeMap<String, BuildObservation>, RunError> {
    let mut observations = BTreeMap::new();
    for label in ["make", "linux-headers", "busybox", "cmake", "python", "perl"] {
        let transcript_path = import_root.join(TRANSCRIPTS_DIR).join(format!("{label}.json"));
        let bytes = fs::read(&transcript_path)
            .map_err(|error| proof_error(format!("reading imported host-tool transcript {label}: {error}")))?;
        let report: BuildJsonReport = serde_json::from_slice(&bytes)
            .map_err(|error| proof_error(format!("parsing imported host-tool transcript {label}: {error}")))?;
        let output = require_single_build_output(label, report, false)?;
        if !output.path.is_dir() || !output.artifact_attestation.path.is_file() {
            return Err(proof_error(format!("imported host-tool evidence is incomplete for {label}")));
        }
        let transcript_digest_blake3 = crate::protected_exec::blake3_file_hex(&transcript_path)
            .map_err(|error| proof_error(format!("hashing imported host-tool transcript {label}: {error}")))?;
        let observation = BuildObservation {
            output,
            transcript_path,
            transcript_digest_blake3,
            action_trust: None,
        };
        if observations.insert(label.to_string(), observation).is_some() {
            return Err(proof_error(format!("duplicate imported host-tool label {label}")));
        }
    }
    assert_eq!(observations.len(), 6);
    Ok(observations)
}

fn imported_native_action_trust(import_root: &Path) -> Result<NativeBuildActionTrustEvidence, RunError> {
    let plan_path = import_root.join(NATIVE_ACTION_PLAN_FILE);
    let reconciliation_path = import_root.join(NATIVE_ACTION_RECONCILIATION_FILE);
    let plan: crate::source_built_derivation_action_plan::EagerDerivationActionPlan =
        serde_json::from_slice(&fs::read(&plan_path).map_err(|error| {
            proof_error(format!("reading imported native action plan {}: {error}", plan_path.display()))
        })?)
        .map_err(|error| proof_error(format!("parsing imported native action plan: {error}")))?;
    let reconciliation: crate::source_built_derivation_action_plan::EagerDerivationReconciliation =
        serde_json::from_slice(&fs::read(&reconciliation_path).map_err(|error| {
            proof_error(format!(
                "reading imported native action reconciliation {}: {error}",
                reconciliation_path.display()
            ))
        })?)
        .map_err(|error| proof_error(format!("parsing imported native action reconciliation: {error}")))?;
    validate_native_build_reconciliation("imported-native-prefix", &reconciliation)?;
    if plan.plan_digest_blake3 != reconciliation.action_plan_digest_blake3 {
        return Err(proof_error("imported native action reconciliation does not bind its plan".to_string()));
    }
    assert!(reconciliation.is_complete());
    Ok(NativeBuildActionTrustEvidence {
        plan_path,
        reconciliation_path,
        plan,
        reconciliation,
    })
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
        native_action_trust: None,
        native_admission,
        native_admission_report_path: imported_admission_path,
        rust_provider: crate::rust_source_provider::RustSourceProviderMaterialization {
            output_path: rust_path,
            recipe_digest_blake3,
            metadata_path: rust_validation.metadata_path,
            metadata_digest_blake3: rust_validation.metadata_digest_blake3,
            action_trust: None,
        },
        rust_host_tools: BTreeMap::new(),
        rust_host_tool_evidence_dir: import_root.join(HOST_TOOLS_EVIDENCE_DIR),
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
        action_trust: None,
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

pub(super) enum DevProviderResume {
    Transition {
        execution_dir: PathBuf,
    },
    Stagex {
        execution_dir: PathBuf,
        provider_report: crate::stagex_provider::StagexProviderPublicationReport,
    },
    Native(Box<NativeProviderPrefix>),
    Complete(Box<ConstructedProviders>),
}

pub(super) fn restore_dev_provider_stage(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    checkpoint_store: &Path,
    completed_stage: crunch_dev_resume_core::ResumeStage,
) -> Result<Option<DevProviderResume>, RunError> {
    let limits = crate::source_built_fixed_point_checkpoint_shell::provider_checkpoint_limits(options.disk_bytes_max);
    let Some(admitted) = crate::source_built_fixed_point_checkpoint_shell::admit_dev_provider_checkpoint_store(
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
    let paths = RestoredProviderPaths::new(prepared, stagex_basename, native_basename)?;
    let requests = dev_provider_restore_requests(&paths, completed_stage)?;
    let restored =
        crate::source_built_fixed_point_checkpoint_shell::restore_dev_provider_checkpoint(admitted, &requests, limits)?;
    let result = match completed_stage {
        crunch_dev_resume_core::ResumeStage::StagexTransition => Ok(DevProviderResume::Transition {
            execution_dir: paths.stagex_transition,
        }),
        crunch_dev_resume_core::ResumeStage::StagexProvider => {
            restored_stagex_provider_report(&paths.stagex_provider, &restored).map(|provider_report| {
                DevProviderResume::Stagex {
                    execution_dir: paths.stagex_transition,
                    provider_report,
                }
            })
        }
        crunch_dev_resume_core::ResumeStage::FullSourceNativeProvider => {
            validate_restored_native_prefix(options, prepared, &paths, &restored)
                .map(|native| DevProviderResume::Native(Box::new(native)))
        }
        crunch_dev_resume_core::ResumeStage::FullSourceRustProvider
        | crunch_dev_resume_core::ResumeStage::MantleStage1
        | crunch_dev_resume_core::ResumeStage::MantleStage2 => {
            validate_restored_provider_checkpoint(options, prepared, paths, restored)
                .map(|providers| DevProviderResume::Complete(Box::new(providers)))
        }
    };
    match result {
        Ok(resume) => Ok(Some(resume)),
        Err(error) => {
            let mut destinations = requests.iter().map(|request| request.destination_path.clone()).collect::<Vec<_>>();
            destinations.push(prepared.staging_dir.join(CHECKPOINT_ORIGIN_EVIDENCE_DIR));
            if let Err(cleanup) = crate::source_built_fixed_point_checkpoint_shell::cleanup_restores(&destinations) {
                return Err(proof_error(format!("{error}; dev resume cleanup failed: {cleanup}")));
            }
            Err(error)
        }
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct RustBindingRelocationObservation {
    origin_binding_digest_blake3: String,
    relocated_binding_digest_blake3: String,
    origin_sysroot_digest_blake3: String,
    relocated_sysroot_digest_blake3: String,
}

struct RestoredProviderPaths {
    stagex_transition: PathBuf,
    stagex_provider: PathBuf,
    native_provider: PathBuf,
    rust_provider: PathBuf,
    rust_host_tools: BTreeMap<String, PathBuf>,
    rust_host_evidence: PathBuf,
    rust_action_trust: PathBuf,
    origin_rust_binding: PathBuf,
    origin_native_admission: PathBuf,
    origin_native_transcript: PathBuf,
    origin_native_action_plan: PathBuf,
    origin_native_action_reconciliation: PathBuf,
    origin_toolchain_closure: PathBuf,
    toolchain_closure: PathBuf,
}

impl RestoredProviderPaths {
    fn new(prepared: &PreparedAttempt, stagex_basename: &str, native_basename: &str) -> Result<Self, RunError> {
        let origin = prepared.staging_dir.join(CHECKPOINT_ORIGIN_EVIDENCE_DIR);
        fs::create_dir(&origin).map_err(|error| {
            proof_error(format!("creating checkpoint origin evidence {}: {error}", origin.display()))
        })?;
        let rust_host_tools_root = prepared.staging_dir.join("rust-host-tools");
        let rust_host_tools = ["make", "cmake", "python", "perl", "busybox", "linux-headers"]
            .into_iter()
            .map(|name| (name.to_string(), rust_host_tools_root.join(name)))
            .collect();
        Ok(Self {
            stagex_transition: prepared.staging_dir.join(STAGEX_TRANSITION_EXECUTION_DIR),
            stagex_provider: prepared.native_store_dir.join(stagex_basename),
            native_provider: prepared.native_store_dir.join(native_basename),
            rust_provider: prepared.staging_dir.join(RUST_PROVIDER_DIR),
            rust_host_tools,
            rust_host_evidence: prepared.staging_dir.join(HOST_TOOLS_EVIDENCE_DIR),
            rust_action_trust: origin.join("rust-provider-action-trust"),
            origin_rust_binding: origin.join("full-source-rust-binding.json"),
            origin_native_admission: origin.join("native-admission.json"),
            origin_native_transcript: origin.join("native-provider.json"),
            origin_native_action_plan: origin.join("native-provider-action-plan.json"),
            origin_native_action_reconciliation: origin.join("native-provider-action-reconciliation.json"),
            origin_toolchain_closure: origin.join(CHECKPOINT_ORIGIN_TOOLCHAIN_CLOSURE_FILE),
            toolchain_closure: prepared.staging_dir.join(TOOLCHAIN_CLOSURE_FILE),
        })
    }

    fn rust_host_tool(&self, name: &str) -> &Path {
        self.rust_host_tools
            .get(name)
            .map(PathBuf::as_path)
            .expect("restored Rust host-tool paths are constructed from a fixed inventory")
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
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_ACTION_PLAN,
                &self.origin_native_action_plan,
                CheckpointPayloadKind::RegularFile,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_ACTION_RECONCILIATION,
                &self.origin_native_action_reconciliation,
                CheckpointPayloadKind::RegularFile,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_HOST_MAKE,
                self.rust_host_tool("make"),
                CheckpointPayloadKind::Directory,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_HOST_CMAKE,
                self.rust_host_tool("cmake"),
                CheckpointPayloadKind::Directory,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_HOST_PYTHON,
                self.rust_host_tool("python"),
                CheckpointPayloadKind::Directory,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_HOST_PERL,
                self.rust_host_tool("perl"),
                CheckpointPayloadKind::Directory,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_HOST_BUSYBOX,
                self.rust_host_tool("busybox"),
                CheckpointPayloadKind::Directory,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_HOST_LINUX_HEADERS,
                self.rust_host_tool("linux-headers"),
                CheckpointPayloadKind::Directory,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_HOST_EVIDENCE,
                &self.rust_host_evidence,
                CheckpointPayloadKind::PreservedTree,
            ),
            checkpoint_restore(
                crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_ACTION_TRUST,
                &self.rust_action_trust,
                CheckpointPayloadKind::PreservedTree,
            ),
        ]
    }
}

fn dev_provider_restore_requests(
    paths: &RestoredProviderPaths,
    completed_stage: crunch_dev_resume_core::ResumeStage,
) -> Result<Vec<crate::source_built_fixed_point_checkpoint_shell::CheckpointPayloadRestore>, RunError> {
    let all = paths.requests();
    let selected = all
        .into_iter()
        .filter(|request| dev_stage_requires_payload(completed_stage, &request.payload_id))
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Err(proof_error("dev resume selected no provider checkpoint payloads".to_string()));
    }
    debug_assert!(selected.len() <= crate::source_built_fixed_point_checkpoint::PROVIDER_CHECKPOINT_PAYLOAD_COUNT);
    Ok(selected)
}

fn dev_stage_requires_payload(stage: crunch_dev_resume_core::ResumeStage, payload_id: &str) -> bool {
    use crate::source_built_fixed_point_checkpoint::*;
    match stage {
        crunch_dev_resume_core::ResumeStage::StagexTransition => payload_id == PAYLOAD_STAGEX_TRANSITION,
        crunch_dev_resume_core::ResumeStage::StagexProvider => {
            matches!(payload_id, PAYLOAD_STAGEX_TRANSITION | PAYLOAD_STAGEX_PROVIDER)
        }
        crunch_dev_resume_core::ResumeStage::FullSourceNativeProvider => {
            !matches!(payload_id, PAYLOAD_RUST_PROVIDER | PAYLOAD_TOOLCHAIN_CLOSURE | PAYLOAD_RUST_ACTION_TRUST)
        }
        crunch_dev_resume_core::ResumeStage::FullSourceRustProvider
        | crunch_dev_resume_core::ResumeStage::MantleStage1
        | crunch_dev_resume_core::ResumeStage::MantleStage2 => true,
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

fn validate_restored_native_prefix(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    paths: &RestoredProviderPaths,
    restored: &crate::source_built_fixed_point_checkpoint_shell::RestoredProviderCheckpoint,
) -> Result<NativeProviderPrefix, RunError> {
    let stagex_logical =
        register_adopted_provider(&paths.stagex_provider, &prepared.native_store_dir, &prepared.native_state_dir)?;
    if stagex_logical != STAGEX_PROVIDER_LOGICAL_PATH {
        return Err(proof_error("dev resume StageX provider logical path mismatch".to_string()));
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
    let host_manifest = paths
        .rust_host_evidence
        .join(crate::full_source_rust_binding_shell::FULL_SOURCE_RUST_HOST_TOOL_MANIFEST_FILE);
    let host_observation = crate::full_source_rust_binding_shell::observe_full_source_rust_host_tools(
        &host_manifest,
        &native_admission.output_digest_blake3,
    )
    .map_err(|error| proof_error(format!("validating resumed Rust host-tool evidence: {error}")))?;
    let rust_host_tools = resumed_host_tool_observations(&host_observation.manifest)?;
    let native_provider = restored_native_provider_observation(
        prepared,
        &paths.native_provider,
        &native_logical,
        &paths.origin_native_admission,
        restored,
    )?;
    let prefix = NativeProviderPrefix {
        stagex_transition_execution_dir: paths.stagex_transition.clone(),
        stagex_provider_report: restored_stagex_provider_report(&paths.stagex_provider, restored)?,
        native_provider,
        native_action_trust: Some(restored_native_action_trust(paths)?),
        native_admission,
        native_admission_report_path,
        rust_host_tools,
        rust_host_tool_evidence_dir: paths.rust_host_evidence.clone(),
        rust_host_tool_manifest_path: host_manifest,
    };
    debug_assert!(prefix.stagex_transition_execution_dir.is_dir());
    debug_assert!(!prefix.rust_host_tools.is_empty());
    Ok(prefix)
}

fn resumed_host_tool_observations(
    manifest: &crate::full_source_rust_binding::FullSourceRustHostToolManifest,
) -> Result<BTreeMap<String, BuildObservation>, RunError> {
    let mut observations = BTreeMap::new();
    for tool in &manifest.tools {
        let label = rust_host_tool_name(tool.role).to_string();
        let observation = resumed_build_observation(
            &label,
            &tool.path,
            &tool.construction_receipt_path,
            &tool.construction_receipt_digest_blake3,
        )?;
        if observations.insert(label, observation).is_some() {
            return Err(proof_error("resumed host-tool manifest has a duplicate role".to_string()));
        }
    }
    for support in &manifest.support_inputs {
        if support.id != "linux-headers" {
            return Err(proof_error(format!("unsupported resumed host support input {}", support.id)));
        }
        let observation = resumed_build_observation(
            "linux-headers",
            &support.path,
            &support.attestation_path,
            &support.attestation_digest_blake3,
        )?;
        if observations.insert("linux-headers".to_string(), observation).is_some() {
            return Err(proof_error("resumed Linux headers input is duplicated".to_string()));
        }
    }
    debug_assert!(!observations.is_empty());
    Ok(observations)
}

fn resumed_build_observation(
    label: &str,
    path: &str,
    attestation_path: &str,
    attestation_digest_blake3: &str,
) -> Result<BuildObservation, RunError> {
    let output_path = PathBuf::from(path);
    let evidence_path = PathBuf::from(attestation_path);
    if !output_path.is_dir() || !evidence_path.is_file() {
        return Err(proof_error(format!("resumed host input is missing for {label}")));
    }
    Ok(BuildObservation {
        output: BuildJsonOutput {
            name: BUILD_OUTPUT_NAME.to_string(),
            path: output_path,
            artifact_attestation: BuildJsonAttestationReference {
                logical_path: path.to_string(),
                path: evidence_path.clone(),
            },
        },
        transcript_path: evidence_path,
        transcript_digest_blake3: attestation_digest_blake3.to_string(),
        action_trust: None,
    })
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
    let rust_binding_relocation = relocate_restored_rust_binding(&paths)?;
    let origin_rust_action_trust = restored_rust_provider_action_trust(&paths)?;
    let rust_action_trust = crate::source_built_rust_provider_action::validate_rust_provider_action_evidence(
        &paths.rust_provider.join(crate::rust_source_provider::RUST_PROVIDER_ACTION_EVIDENCE_RELATIVE_PATH),
    )
    .map_err(|error| proof_error(format!("validating embedded restored Rust action evidence: {error}")))?;
    if origin_rust_action_trust.plan_digest_blake3 != rust_action_trust.plan_digest_blake3
        || origin_rust_action_trust.reconciliation_digest_blake3 != rust_action_trust.reconciliation_digest_blake3
    {
        return Err(proof_error(
            "restored Rust action evidence differs between origin payload and embedded provider".to_string(),
        ));
    }
    let rust_validation = crate::rust_source_provider::validate_materialized_rust_source_provider(&paths.rust_provider)
        .map_err(|error| proof_error(format!("validating restored Rust provider: {error}")))?;
    let recipe_digest = crate::protected_exec::blake3_file_hex(&prepared.source_root.join(RUST_RECIPE_NCL))
        .map_err(|error| proof_error(format!("hashing restored Rust provider recipe: {error}")))?;
    let native_action_trust = restored_native_action_trust(&paths)?;
    materialize_relocated_toolchain_closure(prepared, &paths, &restored, &rust_binding_relocation)?;
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
        native_action_trust: Some(native_action_trust),
        native_admission,
        native_admission_report_path,
        rust_provider: crate::rust_source_provider::RustSourceProviderMaterialization {
            output_path: paths.rust_provider,
            recipe_digest_blake3: recipe_digest,
            metadata_path: rust_validation.metadata_path,
            metadata_digest_blake3: rust_validation.metadata_digest_blake3,
            action_trust: Some(rust_action_trust),
        },
        rust_host_tools: BTreeMap::new(),
        rust_host_tool_evidence_dir: paths.rust_host_evidence,
        toolchain_closure_path: paths.toolchain_closure,
        provider_checkpoint: Some(restored),
    })
}

fn restored_rust_provider_action_trust(
    paths: &RestoredProviderPaths,
) -> Result<crate::source_built_rust_provider_action::RustProviderActionEvidence, RunError> {
    crate::source_built_rust_provider_action::validate_rust_provider_action_evidence(&paths.rust_action_trust)
        .map_err(|error| proof_error(format!("validating restored Rust provider action evidence: {error}")))
}

fn relocate_restored_rust_binding(paths: &RestoredProviderPaths) -> Result<RustBindingRelocationObservation, RunError> {
    let origin_sysroot_digest_blake3 =
        crate::native_toolchain_closure::toolchain_closure_path_blake3(&paths.rust_provider)
            .map_err(|error| proof_error(format!("hashing checkpoint-origin Rust sysroot: {error}")))?;
    let binding_path = paths.rust_provider.join(RUST_PROVIDER_BINDING_RECEIPT_RELATIVE);
    let origin_bytes = fs::read(&binding_path)
        .map_err(|error| proof_error(format!("reading restored Rust binding {}: {error}", binding_path.display())))?;
    write_bytes_create_new(&paths.origin_rust_binding, &origin_bytes)?;
    let mut binding =
        serde_json::from_slice::<crate::full_source_rust_binding::FullSourceRustProviderBindingReceipt>(&origin_bytes)
            .map_err(|error| {
                proof_error(format!("parsing restored Rust binding {}: {error}", binding_path.display()))
            })?;
    let origin_rust_root = origin_rust_provider_root(&binding)?;
    for artifact in &mut binding.rust_artifacts {
        artifact.path = relocate_bound_path(&origin_rust_root, &paths.rust_provider, &artifact.path, "Rust artifact")?;
    }
    for receipt in &mut binding.rust_build_receipts {
        receipt.path = relocate_bound_path(&origin_rust_root, &paths.rust_provider, &receipt.path, "Rust receipt")?;
    }
    for artifact in &mut binding.native_artifacts {
        let relative = required_native_artifact_relative_path(artifact.role, &artifact.path)?;
        let relocated = paths.native_provider.join(relative);
        require_relocated_binding_path(&relocated, "native artifact")?;
        artifact.path = relative.to_string();
    }
    for tool in &mut binding.host_tools {
        let name = rust_host_tool_name(tool.role);
        let relative =
            crate::full_source_rust_binding_shell::full_source_rust_host_tool_executable_relative_path(tool.role)
                .ok_or_else(|| {
                    proof_error(format!("restored Rust binding has unsupported host-tool role {:?}", tool.role))
                })?;
        let executable = paths.rust_host_tool(name).join(relative);
        require_relocated_binding_path(&executable, "host tool")?;
        let executable_path = path_to_utf8(&executable, "host tool")?;
        let (receipt_path, receipt_digest_blake3) =
            relocate_host_tool_receipt(&paths.rust_host_evidence, &tool.construction_receipt_path, &executable_path)?;
        tool.path = executable_path;
        tool.construction_receipt_path = receipt_path;
        tool.construction_receipt_digest_blake3 = receipt_digest_blake3;
    }
    for input in &mut binding.host_support_inputs {
        if input.id != "linux-headers" {
            return Err(proof_error(format!("restored Rust binding has unsupported host support input {}", input.id)));
        }
        let headers = paths.rust_host_tool("linux-headers");
        require_relocated_binding_path(headers, "Linux headers")?;
        input.path = path_to_utf8(headers, "Linux headers")?;
        input.attestation_path =
            relocate_evidence_file(&paths.rust_host_evidence, &input.attestation_path, "Linux headers attestation")?;
    }
    let relocated_bytes = crate::full_source_rust_binding::canonical_full_source_rust_binding_bytes(&binding)
        .map_err(|error| proof_error(format!("serializing relocated Rust binding: {error}")))?;
    fs::write(&binding_path, &relocated_bytes)
        .map_err(|error| proof_error(format!("writing relocated Rust binding {}: {error}", binding_path.display())))?;
    let observation =
        observe_rust_binding_relocation(paths, origin_sysroot_digest_blake3, &origin_bytes, &relocated_bytes)?;
    write_rust_binding_relocation_report(paths, &binding, &binding_path, &observation)?;
    Ok(observation)
}

fn observe_rust_binding_relocation(
    paths: &RestoredProviderPaths,
    origin_sysroot_digest_blake3: String,
    origin_bytes: &[u8],
    relocated_bytes: &[u8],
) -> Result<RustBindingRelocationObservation, RunError> {
    let relocated_sysroot_digest_blake3 =
        crate::native_toolchain_closure::toolchain_closure_path_blake3(&paths.rust_provider)
            .map_err(|error| proof_error(format!("hashing relocated Rust sysroot: {error}")))?;
    if origin_sysroot_digest_blake3 == relocated_sysroot_digest_blake3 {
        return Err(proof_error(
            "restored Rust binding relocation did not change the Rust sysroot identity".to_string(),
        ));
    }
    let origin_binding_digest_blake3 = blake3::hash(origin_bytes).to_hex().to_string();
    let relocated_binding_digest_blake3 = blake3::hash(relocated_bytes).to_hex().to_string();
    if origin_binding_digest_blake3 == relocated_binding_digest_blake3 {
        return Err(proof_error("restored Rust binding relocation did not change the binding identity".to_string()));
    }
    assert_eq!(origin_binding_digest_blake3.len(), blake3::OUT_LEN * 2);
    assert_eq!(relocated_binding_digest_blake3.len(), blake3::OUT_LEN * 2);
    Ok(RustBindingRelocationObservation {
        origin_binding_digest_blake3,
        relocated_binding_digest_blake3,
        origin_sysroot_digest_blake3,
        relocated_sysroot_digest_blake3,
    })
}

fn write_rust_binding_relocation_report(
    paths: &RestoredProviderPaths,
    binding: &crate::full_source_rust_binding::FullSourceRustProviderBindingReceipt,
    binding_path: &Path,
    observation: &RustBindingRelocationObservation,
) -> Result<(), RunError> {
    let report = serde_json::json!({
        "schema": CHECKPOINT_RUST_BINDING_RELOCATION_REPORT_SCHEMA,
        "origin_binding": {
            "path": paths.origin_rust_binding,
            "digest_blake3": &observation.origin_binding_digest_blake3,
        },
        "relocated_binding": {
            "path": binding_path,
            "digest_blake3": &observation.relocated_binding_digest_blake3,
        },
        "rust_sysroot": {
            "origin_digest_blake3": &observation.origin_sysroot_digest_blake3,
            "relocated_digest_blake3": &observation.relocated_sysroot_digest_blake3,
        },
        "host_tool_count": binding.host_tools.len(),
        "native_artifact_count": binding.native_artifacts.len(),
        "rust_artifact_count": binding.rust_artifacts.len(),
        "non_claim": "binding relocation changes only checkpoint-origin absolute authority paths and does not repeat provider execution",
    });
    write_json_create_new(
        &paths
            .rust_provider
            .parent()
            .unwrap_or(Path::new("."))
            .join(CHECKPOINT_RUST_BINDING_RELOCATION_REPORT_FILE),
        &report,
    )
}

fn origin_rust_provider_root(
    binding: &crate::full_source_rust_binding::FullSourceRustProviderBindingReceipt,
) -> Result<PathBuf, RunError> {
    let rustc = binding
        .rust_artifacts
        .iter()
        .find(|artifact| artifact.role == crate::source_toolchain_closure::RustProviderRole::Rustc)
        .ok_or_else(|| proof_error("restored Rust binding lacks rustc artifact authority".to_string()))?;
    let rustc_path = Path::new(&rustc.path);
    if !rustc_path.ends_with(RUST_PROVIDER_RUSTC_RELATIVE) {
        return Err(proof_error(format!(
            "restored Rust binding rustc path has unexpected shape: {}",
            rustc_path.display()
        )));
    }
    rustc_path
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| proof_error("restored Rust binding rustc path has no provider root".to_string()))
}

fn required_native_artifact_relative_path(
    role: crate::full_source_rust_binding::FullSourceNativeArtifactRole,
    origin_path: &str,
) -> Result<&'static str, RunError> {
    let matches = crate::full_source_rust_binding::required_full_source_native_artifacts()
        .iter()
        .filter(|(relative, candidate_role)| *candidate_role == role && Path::new(origin_path).ends_with(relative))
        .map(|(relative, _role)| *relative)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [relative] => Ok(*relative),
        _ => Err(proof_error(format!(
            "restored native artifact path has {} matching authorities: {origin_path}",
            matches.len()
        ))),
    }
}

fn relocate_bound_path(
    origin_root: &Path,
    current_root: &Path,
    origin_path: &str,
    label: &str,
) -> Result<String, RunError> {
    let relative = Path::new(origin_path)
        .strip_prefix(origin_root)
        .map_err(|_| proof_error(format!("{label} is outside the origin provider root: {origin_path}")))?;
    if relative.components().any(|component| !matches!(component, std::path::Component::Normal(_))) {
        return Err(proof_error(format!("{label} relative path is unsafe: {}", relative.display())));
    }
    let relocated = current_root.join(relative);
    require_relocated_binding_path(&relocated, label)?;
    path_to_utf8(&relocated, label)
}

fn relocate_host_tool_receipt(
    evidence_root: &Path,
    origin_path: &str,
    executable_path: &str,
) -> Result<(String, String), RunError> {
    let receipt_path =
        PathBuf::from(relocate_evidence_file(evidence_root, origin_path, "host-tool construction receipt")?);
    let bytes = fs::read(&receipt_path)
        .map_err(|error| proof_error(format!("reading host-tool receipt {}: {error}", receipt_path.display())))?;
    let mut receipt =
        serde_json::from_slice::<crate::full_source_rust_binding::FullSourceRustHostToolConstructionReceipt>(&bytes)
            .map_err(|error| proof_error(format!("parsing host-tool receipt {}: {error}", receipt_path.display())))?;
    receipt.executable_path = executable_path.to_string();
    receipt.artifact_attestation_path =
        relocate_evidence_file(evidence_root, &receipt.artifact_attestation_path, "host-tool artifact attestation")?;
    let mut relocated = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| proof_error(format!("serializing relocated host-tool receipt: {error}")))?;
    relocated.push(b'\n');
    fs::write(&receipt_path, &relocated)
        .map_err(|error| proof_error(format!("writing host-tool receipt {}: {error}", receipt_path.display())))?;
    let digest = blake3::hash(&relocated).to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    assert!(receipt_path.is_file());
    Ok((path_to_utf8(&receipt_path, "host-tool construction receipt")?, digest))
}

fn relocate_evidence_file(evidence_root: &Path, origin_path: &str, label: &str) -> Result<String, RunError> {
    let name = Path::new(origin_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| proof_error(format!("{label} has no UTF-8 basename: {origin_path}")))?;
    let relocated = evidence_root.join(name);
    if !relocated.is_file() {
        return Err(proof_error(format!("relocated {label} is missing: {}", relocated.display())));
    }
    path_to_utf8(&relocated, label)
}

fn require_relocated_binding_path(path: &Path, label: &str) -> Result<(), RunError> {
    if !path.exists() {
        return Err(proof_error(format!("relocated {label} is missing: {}", path.display())));
    }
    Ok(())
}

fn path_to_utf8(path: &Path, label: &str) -> Result<String, RunError> {
    path.to_str()
        .map(str::to_string)
        .ok_or_else(|| proof_error(format!("relocated {label} path is not UTF-8: {}", path.display())))
}

fn rust_host_tool_name(role: crate::full_source_rust_binding::FullSourceRustHostToolRole) -> &'static str {
    use crate::full_source_rust_binding::FullSourceRustHostToolRole;
    match role {
        FullSourceRustHostToolRole::Make => "make",
        FullSourceRustHostToolRole::Cmake => "cmake",
        FullSourceRustHostToolRole::Python => "python",
        FullSourceRustHostToolRole::Perl => "perl",
        FullSourceRustHostToolRole::Busybox => "busybox",
    }
}

fn restored_native_action_trust(paths: &RestoredProviderPaths) -> Result<NativeBuildActionTrustEvidence, RunError> {
    let plan: crate::source_built_derivation_action_plan::EagerDerivationActionPlan = serde_json::from_slice(
        &fs::read(&paths.origin_native_action_plan)
            .map_err(|error| proof_error(format!("reading restored native action plan: {error}")))?,
    )
    .map_err(|error| proof_error(format!("parsing restored native action plan: {error}")))?;
    crate::source_built_derivation_action_plan::validate_eager_derivation_action_plan(&plan)
        .map_err(|error| proof_error(format!("validating restored native action plan: {error}")))?;
    let reconciliation: crate::source_built_derivation_action_plan::EagerDerivationReconciliation =
        serde_json::from_slice(
            &fs::read(&paths.origin_native_action_reconciliation)
                .map_err(|error| proof_error(format!("reading restored native action reconciliation: {error}")))?,
        )
        .map_err(|error| proof_error(format!("parsing restored native action reconciliation: {error}")))?;
    crate::source_built_derivation_action_plan::require_complete_eager_derivation_reconciliation(&reconciliation)
        .map_err(|error| proof_error(format!("validating restored native action reconciliation: {error}")))?;
    if reconciliation.action_plan_digest_blake3 != plan.plan_digest_blake3
        || reconciliation.planned_action_count != plan.action_count
    {
        return Err(proof_error("restored native action plan and reconciliation linkage mismatch".to_string()));
    }
    assert!(reconciliation.is_complete());
    debug_assert_eq!(reconciliation.matched_action_count, plan.action_count);
    Ok(NativeBuildActionTrustEvidence {
        plan_path: paths.origin_native_action_plan.clone(),
        reconciliation_path: paths.origin_native_action_reconciliation.clone(),
        plan,
        reconciliation,
    })
}

fn materialize_relocated_toolchain_closure(
    prepared: &PreparedAttempt,
    paths: &RestoredProviderPaths,
    restored: &crate::source_built_fixed_point_checkpoint_shell::RestoredProviderCheckpoint,
    rust_binding: &RustBindingRelocationObservation,
) -> Result<(), RunError> {
    crate::native_toolchain_closure::cmd_materialize_native_toolchain_closure(NativeToolchainClosureOptions {
        rust_source_provider: &paths.rust_provider,
        host_root: &paths.native_provider,
        target_root: &paths.native_provider,
        output: &paths.toolchain_closure,
    })?;
    let origin = read_toolchain_closure_manifest(&paths.origin_toolchain_closure)?;
    let relocated = read_toolchain_closure_manifest(&paths.toolchain_closure)?;
    let rust_sysroot = crate::source_toolchain_closure::ToolchainClosureRustSysrootRelocation {
        origin_content_digest_blake3: &rust_binding.origin_sysroot_digest_blake3,
        relocated_content_digest_blake3: &rust_binding.relocated_sysroot_digest_blake3,
    };
    let validation = crate::source_toolchain_closure::validate_relocated_toolchain_closure_with_rust_sysroot(
        &origin,
        &relocated,
        rust_sysroot,
    )
    .map_err(|error| proof_error(format!("validating restored toolchain closure relocation: {error}")))?;
    let required_count = crate::source_toolchain_closure::required_native_closure_member_names().len();
    if validation.member_count != required_count {
        return Err(proof_error(format!(
            "restored toolchain closure relocation has {} members, expected {required_count}",
            validation.member_count
        )));
    }
    write_toolchain_closure_relocation_report(prepared, paths, restored, rust_binding, &validation)
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
    rust_binding: &RustBindingRelocationObservation,
    validation: &crate::source_toolchain_closure::ToolchainClosureRelocationValidation,
) -> Result<(), RunError> {
    let origin_digest = crate::protected_exec::blake3_file_hex(&paths.origin_toolchain_closure)
        .map_err(|error| proof_error(format!("hashing checkpoint origin closure: {error}")))?;
    let relocated_digest = crate::protected_exec::blake3_file_hex(&paths.toolchain_closure)
        .map_err(|error| proof_error(format!("hashing relocated checkpoint closure: {error}")))?;
    let rust_binding_report = prepared.staging_dir.join(CHECKPOINT_RUST_BINDING_RELOCATION_REPORT_FILE);
    let rust_binding_report_digest = crate::protected_exec::blake3_file_hex(&rust_binding_report)
        .map_err(|error| proof_error(format!("hashing checkpoint Rust binding relocation report: {error}")))?;
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
        "rust_sysroot_relocation": {
            "origin_digest_blake3": &rust_binding.origin_sysroot_digest_blake3,
            "relocated_digest_blake3": &rust_binding.relocated_sysroot_digest_blake3,
            "binding_report": rust_binding_report,
            "binding_report_digest_blake3": rust_binding_report_digest,
        },
        "member_count": validation.member_count,
        "non_claim": "closure relocation changes absolute provider roots plus one receipt-bound Rust sysroot binding identity; it does not repeat provider execution or permit other member changes",
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
        action_trust: None,
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

pub(super) fn publish_dev_resume_provider_checkpoint(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    providers: &ConstructedProviders,
    checkpoint_store: &Path,
) -> Result<crate::source_built_fixed_point_checkpoint_shell::PublishedProviderCheckpoint, RunError> {
    let limits = crate::source_built_fixed_point_checkpoint_shell::provider_checkpoint_limits(options.disk_bytes_max);
    let stage_observations = provider_checkpoint_stage_observations(prepared, providers, limits)?;
    let payload_sources = provider_checkpoint_payload_sources(providers)?;
    crate::source_built_fixed_point_checkpoint_shell::publish_dev_provider_checkpoint(
        checkpoint_store,
        &prepared.plan,
        stage_observations,
        &payload_sources,
        STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST,
        limits,
    )
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

pub(super) fn provider_checkpoint_payload_sources(
    providers: &ConstructedProviders,
) -> Result<
    [crate::source_built_fixed_point_checkpoint_shell::CheckpointPayloadSource;
        crate::source_built_fixed_point_checkpoint::PROVIDER_CHECKPOINT_PAYLOAD_COUNT],
    RunError,
> {
    use crate::source_built_fixed_point_checkpoint::CheckpointPayloadKind;
    let stagex_basename = required_utf8_basename(&providers.stagex_provider_report.output_path, "StageX provider")?;
    let native_basename = required_utf8_basename(&providers.native_provider.output.path, "native provider")?;
    let native_action_trust = providers
        .native_action_trust
        .as_ref()
        .ok_or_else(|| proof_error("promoted checkpoint requires complete native action trust".to_string()))?;
    let rust_action_trust =
        providers.rust_provider.action_trust.as_ref().ok_or_else(|| {
            proof_error("promoted checkpoint requires complete Rust provider action trust".to_string())
        })?;
    let rust_action_dir = rust_action_trust
        .plan_path
        .parent()
        .ok_or_else(|| proof_error("Rust provider action plan has no parent directory".to_string()))?;
    if rust_action_trust.audit_path.parent() != Some(rust_action_dir)
        || rust_action_trust.reconciliation_path.parent() != Some(rust_action_dir)
    {
        return Err(proof_error("Rust provider action evidence does not share one directory".to_string()));
    }
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
        checkpoint_payload_source(
            crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_ACTION_PLAN,
            &native_action_trust.plan_path,
            Path::new(CHECKPOINT_NATIVE_ACTION_PLAN_PATH),
            CheckpointPayloadKind::RegularFile,
        ),
        checkpoint_payload_source(
            crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_ACTION_RECONCILIATION,
            &native_action_trust.reconciliation_path,
            Path::new(CHECKPOINT_NATIVE_ACTION_RECONCILIATION_PATH),
            CheckpointPayloadKind::RegularFile,
        ),
        rust_host_tool_payload(
            providers,
            "make",
            crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_HOST_MAKE,
            CheckpointPayloadKind::Directory,
        )?,
        rust_host_tool_payload(
            providers,
            "cmake",
            crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_HOST_CMAKE,
            CheckpointPayloadKind::Directory,
        )?,
        rust_host_tool_payload(
            providers,
            "python",
            crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_HOST_PYTHON,
            CheckpointPayloadKind::Directory,
        )?,
        rust_host_tool_payload(
            providers,
            "perl",
            crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_HOST_PERL,
            CheckpointPayloadKind::Directory,
        )?,
        rust_host_tool_payload(
            providers,
            "busybox",
            crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_HOST_BUSYBOX,
            CheckpointPayloadKind::Directory,
        )?,
        rust_host_tool_payload(
            providers,
            "linux-headers",
            crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_HOST_LINUX_HEADERS,
            CheckpointPayloadKind::Directory,
        )?,
        checkpoint_payload_source(
            crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_HOST_EVIDENCE,
            &providers.rust_host_tool_evidence_dir,
            Path::new(CHECKPOINT_RUST_HOST_EVIDENCE_PATH),
            CheckpointPayloadKind::PreservedTree,
        ),
        checkpoint_payload_source(
            crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_ACTION_TRUST,
            rust_action_dir,
            Path::new(CHECKPOINT_RUST_ACTION_TRUST_PATH),
            CheckpointPayloadKind::PreservedTree,
        ),
    ])
}

fn rust_host_tool_payload(
    providers: &ConstructedProviders,
    name: &str,
    payload_id: &str,
    kind: crate::source_built_fixed_point_checkpoint::CheckpointPayloadKind,
) -> Result<crate::source_built_fixed_point_checkpoint_shell::CheckpointPayloadSource, RunError> {
    let observation = providers
        .rust_host_tools
        .get(name)
        .ok_or_else(|| proof_error(format!("promoted checkpoint is missing Rust host tool {name}")))?;
    Ok(checkpoint_payload_source(
        payload_id,
        &observation.output.path,
        &Path::new(CHECKPOINT_RUST_HOST_TOOL_PREFIX).join(name),
        kind,
    ))
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

pub(super) fn provider_checkpoint_stage_observations(
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
        ("transition-plan", providers.stagex_transition_execution_dir.join(STAGEX_TRANSITION_PLAN_FILE)),
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
    let native_action_trust = providers
        .native_action_trust
        .as_ref()
        .ok_or_else(|| proof_error("native checkpoint evidence lacks action trust".to_string()))?;
    let native_provider = checkpoint_evidence_digest(&[
        ("native-transcript", providers.native_provider.transcript_path.clone()),
        ("native-admission", providers.native_admission_report_path.clone()),
        ("native-action-plan", native_action_trust.plan_path.clone()),
        ("native-action-reconciliation", native_action_trust.reconciliation_path.clone()),
    ])?;
    let rust_action_trust = providers
        .rust_provider
        .action_trust
        .as_ref()
        .ok_or_else(|| proof_error("Rust provider checkpoint evidence lacks action trust".to_string()))?;
    let rust_provider = checkpoint_evidence_digest(&[
        ("rust-build-receipt", providers.rust_provider.output_path.join(RUST_PROVIDER_BUILD_RECEIPT_RELATIVE)),
        (
            "rust-binding-receipt",
            providers.rust_provider.output_path.join(RUST_PROVIDER_BINDING_RECEIPT_RELATIVE),
        ),
        ("rust-metadata", providers.rust_provider.metadata_path.clone()),
        ("rust-action-plan", rust_action_trust.plan_path.clone()),
        ("rust-action-audit", rust_action_trust.audit_path.clone()),
        ("rust-action-reconciliation", rust_action_trust.reconciliation_path.clone()),
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

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

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
            rust_host_tools: ["make", "cmake", "python", "perl", "busybox", "linux-headers"]
                .into_iter()
                .map(|name| (name.to_string(), PathBuf::from(format!("/proof/rust-host-tools/{name}"))))
                .collect(),
            rust_host_evidence: PathBuf::from("/proof/rust-host-evidence"),
            rust_action_trust: PathBuf::from("/proof/origin/rust-provider-action-trust"),
            origin_rust_binding: PathBuf::from("/proof/origin/full-source-rust-binding.json"),
            origin_native_admission: PathBuf::from("/proof/origin/native-admission.json"),
            origin_native_transcript: PathBuf::from("/proof/origin/native-provider.json"),
            origin_native_action_plan: PathBuf::from("/proof/origin/native-provider-action-plan.json"),
            origin_native_action_reconciliation: PathBuf::from(
                "/proof/origin/native-provider-action-reconciliation.json",
            ),
            origin_toolchain_closure: PathBuf::from("/proof/origin/source-built-toolchain-closure.json"),
            toolchain_closure: PathBuf::from("/proof/source-built-toolchain-closure.json"),
        };

        let requests = paths.requests();
        let closure = requests
            .iter()
            .find(|request| request.payload_id == crate::source_built_fixed_point_checkpoint::PAYLOAD_TOOLCHAIN_CLOSURE)
            .expect("toolchain closure restore request");
        let action_plan = requests
            .iter()
            .find(|request| {
                request.payload_id == crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_ACTION_PLAN
            })
            .expect("native action-plan restore request");
        let action_reconciliation = requests
            .iter()
            .find(|request| {
                request.payload_id == crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_ACTION_RECONCILIATION
            })
            .expect("native action-reconciliation restore request");

        assert_eq!(closure.destination_path, paths.origin_toolchain_closure);
        assert_ne!(closure.destination_path, paths.toolchain_closure);
        assert_eq!(closure.kind, crate::source_built_fixed_point_checkpoint::CheckpointPayloadKind::RegularFile);
        assert_eq!(action_plan.destination_path, paths.origin_native_action_plan);
        assert_eq!(action_reconciliation.destination_path, paths.origin_native_action_reconciliation);
    }

    #[test]
    fn restored_rust_binding_relocates_provider_host_and_native_authority() {
        use crate::full_source_rust_binding::FullSourceNativeArtifactBinding;
        use crate::full_source_rust_binding::FullSourceNativeArtifactRole;
        use crate::full_source_rust_binding::FullSourceNativeProviderAdmissionIdentity;
        use crate::full_source_rust_binding::FullSourceRustArtifactBinding;
        use crate::full_source_rust_binding::FullSourceRustHostSupportInputBinding;
        use crate::full_source_rust_binding::FullSourceRustHostToolBinding;
        use crate::full_source_rust_binding::FullSourceRustHostToolConstructionReceipt;
        use crate::full_source_rust_binding::FullSourceRustHostToolRole;
        use crate::full_source_rust_binding::FullSourceRustProviderBindingReceipt;
        use crate::full_source_rust_binding::FullSourceRustReceiptBinding;
        use crate::source_toolchain_closure::RustProviderRole;
        use crate::source_toolchain_closure::ToolchainBuildReceiptKind;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let rust_provider = root.join("rust-provider");
        let native_provider = root.join("native-provider");
        let host_root = root.join("rust-host-tools/busybox");
        let headers_root = root.join("rust-host-tools/linux-headers");
        let host_evidence = root.join("rust-host-evidence");
        let rust_action = root.join("origin/rust-action");
        let origin_dir = root.join("origin");
        for path in [
            rust_provider.join("bin"),
            rust_provider.join("share/mantle-rust-provider/receipts"),
            native_provider.join("bin"),
            host_root.join("bin"),
            headers_root.clone(),
            host_evidence.clone(),
            rust_action.clone(),
            origin_dir.clone(),
        ] {
            fs::create_dir_all(path).unwrap();
        }
        fs::write(rust_provider.join(RUST_PROVIDER_RUSTC_RELATIVE), b"rustc").unwrap();
        fs::write(rust_provider.join(RUST_PROVIDER_BUILD_RECEIPT_RELATIVE), b"{}\n").unwrap();
        let native_relative = "bin/x86_64-linux-musl-gcc";
        fs::write(native_provider.join(native_relative), b"gcc").unwrap();
        let busybox_relative =
            crate::full_source_rust_binding_shell::full_source_rust_host_tool_executable_relative_path(
                FullSourceRustHostToolRole::Busybox,
            )
            .unwrap();
        fs::create_dir_all(host_root.join(busybox_relative).parent().unwrap()).unwrap();
        fs::write(host_root.join(busybox_relative), b"busybox").unwrap();
        let attestation_path = host_evidence.join("busybox-attestation.json");
        fs::write(&attestation_path, b"attestation").unwrap();
        let construction_path = host_evidence.join("busybox-construction.json");
        let construction = FullSourceRustHostToolConstructionReceipt {
            schema: crate::full_source_rust_binding::FULL_SOURCE_RUST_HOST_TOOL_RECEIPT_SCHEMA.to_string(),
            receipt_id: "busybox-construction".to_string(),
            role: FullSourceRustHostToolRole::Busybox,
            source_policy: "authenticated-offline-only".to_string(),
            source_id: "busybox-source".to_string(),
            source_url: "https://example.invalid/busybox".to_string(),
            source_sha256_hex: DIGEST_A.to_string(),
            native_provider_output_digest_blake3: DIGEST_A.to_string(),
            executable_path: "/origin/host/busybox/bin/busybox".to_string(),
            executable_digest_blake3: DIGEST_A.to_string(),
            artifact_attestation_path: "/origin/evidence/busybox-attestation.json".to_string(),
            artifact_attestation_digest_blake3: DIGEST_A.to_string(),
            artifact_attestation_file_digest_blake3: DIGEST_A.to_string(),
            positive_checks: vec!["smoke".to_string()],
            rejection_checks: vec!["negative".to_string()],
            dependency_digests_blake3: BTreeMap::new(),
            ambient_tool_discovery: false,
            fallback_events: Vec::new(),
            environmental_assumptions: Vec::new(),
        };
        let mut construction_bytes = serde_json::to_vec_pretty(&construction).unwrap();
        construction_bytes.push(b'\n');
        fs::write(&construction_path, &construction_bytes).unwrap();
        let headers_attestation = host_evidence.join("linux-headers-attestation.json");
        fs::write(&headers_attestation, b"headers-attestation").unwrap();
        let binding = FullSourceRustProviderBindingReceipt {
            schema: crate::full_source_rust_binding::FULL_SOURCE_RUST_BINDING_SCHEMA.to_string(),
            receipt_id: "binding".to_string(),
            rust_provider_id: "rust-provider".to_string(),
            host_triple: "x86_64-unknown-linux-musl".to_string(),
            target_triple: "x86_64-unknown-linux-musl".to_string(),
            source_policy: "authenticated-offline-only".to_string(),
            ambient_tool_discovery: false,
            rust_provider_policy_digest_blake3: DIGEST_A.to_string(),
            host_tool_manifest_digest_blake3: DIGEST_A.to_string(),
            host_tools: vec![FullSourceRustHostToolBinding {
                role: FullSourceRustHostToolRole::Busybox,
                path: "/origin/host/busybox/bin/busybox".to_string(),
                content_digest_blake3: DIGEST_A.to_string(),
                source_id: "busybox-source".to_string(),
                construction_receipt_path: "/origin/evidence/busybox-construction.json".to_string(),
                construction_receipt_digest_blake3: blake3::hash(&construction_bytes).to_hex().to_string(),
            }],
            host_support_inputs: vec![FullSourceRustHostSupportInputBinding {
                id: "linux-headers".to_string(),
                path: "/origin/host/linux-headers".to_string(),
                content_digest_blake3: DIGEST_A.to_string(),
                source_id: "linux-6.6".to_string(),
                attestation_path: "/origin/evidence/linux-headers-attestation.json".to_string(),
                attestation_digest_blake3: DIGEST_A.to_string(),
            }],
            native_provider: FullSourceNativeProviderAdmissionIdentity {
                schema: "mantle-full-source-provider-admission-v2".to_string(),
                status: "admitted".to_string(),
                provider_id: "full-source-v1".to_string(),
                provider_target: "x86_64-linux-musl".to_string(),
                compiler_target: "x86_64-unknown-linux-musl".to_string(),
                admission_report_digest_blake3: DIGEST_A.to_string(),
                metadata_digest_blake3: DIGEST_A.to_string(),
                output_digest_blake3: DIGEST_A.to_string(),
                expected_output_digest_blake3: DIGEST_A.to_string(),
                source_closure_manifest_blake3: DIGEST_A.to_string(),
                expected_source_closure_manifest_blake3: DIGEST_A.to_string(),
                source_closure_record_count: 1,
            },
            native_artifacts: vec![FullSourceNativeArtifactBinding {
                role: FullSourceNativeArtifactRole::CCompiler,
                path: format!("/origin/native/{native_relative}"),
                content_digest_blake3: DIGEST_A.to_string(),
            }],
            rust_source_ids: vec!["rust-source".to_string()],
            rust_build_receipts: vec![FullSourceRustReceiptBinding {
                id: "build".to_string(),
                kind: ToolchainBuildReceiptKind::MantleRustTopology,
                name: "build".to_string(),
                path: format!("/origin/rust/{RUST_PROVIDER_BUILD_RECEIPT_RELATIVE}"),
                digest_blake3: DIGEST_A.to_string(),
            }],
            rust_artifacts: vec![FullSourceRustArtifactBinding {
                role: RustProviderRole::Rustc,
                name: "rustc".to_string(),
                path: format!("/origin/rust/{RUST_PROVIDER_RUSTC_RELATIVE}"),
                content_digest_blake3: DIGEST_A.to_string(),
                source_id: "rust-source".to_string(),
                build_receipt_id: "build".to_string(),
            }],
            fallback_events: Vec::new(),
            seed_exceptions: Vec::new(),
        };
        let binding_path = rust_provider.join(RUST_PROVIDER_BINDING_RECEIPT_RELATIVE);
        fs::write(
            &binding_path,
            crate::full_source_rust_binding::canonical_full_source_rust_binding_bytes(&binding).unwrap(),
        )
        .unwrap();
        let paths = RestoredProviderPaths {
            stagex_transition: root.join("stagex-transition"),
            stagex_provider: root.join("stagex-provider"),
            native_provider: native_provider.clone(),
            rust_provider: rust_provider.clone(),
            rust_host_tools: [
                ("busybox".to_string(), host_root),
                ("linux-headers".to_string(), headers_root),
            ]
            .into_iter()
            .collect(),
            rust_host_evidence: host_evidence,
            rust_action_trust: rust_action,
            origin_rust_binding: origin_dir.join("binding.json"),
            origin_native_admission: origin_dir.join("native-admission.json"),
            origin_native_transcript: origin_dir.join("native-provider.json"),
            origin_native_action_plan: origin_dir.join("native-plan.json"),
            origin_native_action_reconciliation: origin_dir.join("native-reconciliation.json"),
            origin_toolchain_closure: origin_dir.join("closure.json"),
            toolchain_closure: root.join("closure.json"),
        };

        let observation = relocate_restored_rust_binding(&paths).unwrap();
        let relocated: FullSourceRustProviderBindingReceipt =
            serde_json::from_slice(&fs::read(&binding_path).unwrap()).unwrap();
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join(CHECKPOINT_RUST_BINDING_RELOCATION_REPORT_FILE)).unwrap())
                .unwrap();
        let unchanged_error = observe_rust_binding_relocation(
            &paths,
            observation.relocated_sysroot_digest_blake3.clone(),
            b"same binding",
            b"same binding",
        )
        .unwrap_err();
        let unchanged_binding_error =
            observe_rust_binding_relocation(&paths, DIGEST_A.to_string(), b"same binding", b"same binding")
                .unwrap_err();

        assert!(paths.origin_rust_binding.is_file());
        assert_ne!(observation.origin_binding_digest_blake3, observation.relocated_binding_digest_blake3);
        assert_ne!(observation.origin_sysroot_digest_blake3, observation.relocated_sysroot_digest_blake3);
        assert_eq!(observation.origin_sysroot_digest_blake3.len(), blake3::OUT_LEN * 2);
        assert_eq!(observation.relocated_sysroot_digest_blake3.len(), blake3::OUT_LEN * 2);
        assert!(unchanged_error.to_string().contains("did not change the Rust sysroot identity"));
        assert!(unchanged_binding_error.to_string().contains("did not change the binding identity"));
        assert_eq!(
            report.pointer("/rust_sysroot/origin_digest_blake3").and_then(serde_json::Value::as_str),
            Some(observation.origin_sysroot_digest_blake3.as_str())
        );
        assert_eq!(
            report.pointer("/rust_sysroot/relocated_digest_blake3").and_then(serde_json::Value::as_str),
            Some(observation.relocated_sysroot_digest_blake3.as_str())
        );
        assert_eq!(
            relocated.rust_artifacts[0].path,
            rust_provider.join(RUST_PROVIDER_RUSTC_RELATIVE).display().to_string()
        );
        assert_eq!(relocated.native_artifacts[0].path, native_relative);
        assert_eq!(
            relocated.host_tools[0].path,
            paths.rust_host_tool("busybox").join(busybox_relative).display().to_string()
        );
        assert_ne!(
            relocated.host_tools[0].construction_receipt_digest_blake3,
            binding.host_tools[0].construction_receipt_digest_blake3
        );
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

    #[cfg(unix)]
    #[test]
    fn imported_native_provider_copy_is_identity_preserving_and_inode_isolated() {
        use std::os::unix::fs::MetadataExt as _;
        use std::os::unix::fs::PermissionsExt as _;

        const PERMISSION_BITS_MASK: u32 = 0o777;
        const READ_ONLY_FILE_MODE: u32 = 0o444;
        const WRITABLE_FILE_MODE: u32 = 0o644;
        const PAYLOAD: &[u8] = b"provider payload\n";
        const CHANGED_PAYLOAD: &[u8] = b"changed payload\n";

        let temp = tempfile::tempdir().expect("temporary native provider roots");
        let origin = temp.path().join("origin");
        let materialized = temp.path().join("materialized");
        fs::create_dir(&origin).expect("origin provider directory");
        let origin_file = origin.join("payload");
        fs::write(&origin_file, PAYLOAD).expect("origin provider payload");
        fs::set_permissions(&origin_file, fs::Permissions::from_mode(READ_ONLY_FILE_MODE))
            .expect("read-only origin payload");
        let origin_identity = crate::release_tree_copy::hash_directory_tree(&origin).expect("origin identity");

        let materialized_identity =
            copy_imported_native_provider_tree(&origin, &materialized).expect("isolated provider copy");

        let materialized_file = materialized.join("payload");
        assert_eq!(materialized_identity, origin_identity);
        assert_ne!(fs::metadata(&origin_file).unwrap().ino(), fs::metadata(&materialized_file).unwrap().ino());
        fs::set_permissions(&materialized_file, fs::Permissions::from_mode(WRITABLE_FILE_MODE))
            .expect("writable materialized payload");
        fs::write(&materialized_file, CHANGED_PAYLOAD).expect("change materialized payload");
        assert_eq!(fs::read(&origin_file).unwrap(), PAYLOAD);
        assert_eq!(
            fs::metadata(&origin_file).unwrap().permissions().mode() & PERMISSION_BITS_MASK,
            READ_ONLY_FILE_MODE
        );
    }

    #[test]
    fn imported_native_provider_copy_rejects_a_nonempty_destination() {
        const ORIGIN_PAYLOAD: &[u8] = b"origin\n";
        const DESTINATION_PAYLOAD: &[u8] = b"destination\n";

        let temp = tempfile::tempdir().expect("temporary native provider roots");
        let origin = temp.path().join("origin");
        let materialized = temp.path().join("materialized");
        fs::create_dir(&origin).expect("origin provider directory");
        fs::create_dir(&materialized).expect("materialized provider directory");
        fs::write(origin.join("payload"), ORIGIN_PAYLOAD).expect("origin provider payload");
        fs::write(materialized.join("existing"), DESTINATION_PAYLOAD).expect("existing destination payload");

        let error = copy_imported_native_provider_tree(&origin, &materialized)
            .expect_err("nonempty destination must fail closed");

        assert!(error.to_string().contains("destination"));
        assert_eq!(fs::read(origin.join("payload")).unwrap(), ORIGIN_PAYLOAD);
        assert_eq!(fs::read(materialized.join("existing")).unwrap(), DESTINATION_PAYLOAD);
    }
}
