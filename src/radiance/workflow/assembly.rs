pub(super) struct Input<'request, 'paths> {
    pub operator: &'request super::ProofRequest<'paths>,
    pub profile: &'request crate::radiance::profile::Definition,
    pub manifest: &'request crate::source_bundle::SourceBundleManifest,
    pub cohort: crunch_radiance_reference_core::RadianceSourceCohortWire,
    pub materialized: &'request [(String, std::path::PathBuf)],
    pub scratch: &'request std::path::Path,
}

pub(super) struct Output {
    pub receipt: crunch_radiance_reference_core::RadianceReferenceReceipt,
    pub protected_audit_blake3: String,
}

pub(super) fn execute_graph(input: Input<'_, '_>) -> Result<Output, crate::errors::RunError> {
    let radiance_source = super::helpers::source_path(input.materialized, "radiance")?;
    let bootstrap_source = super::helpers::source_path(input.materialized, "bootstrap-compiler")?;
    let emulator_source = super::helpers::source_path(input.materialized, "emulator")?;
    let seed = radiance_source.join(super::SEED_RELATIVE_PATH);
    let tools = crate::radiance::runtime::native::build_tools(crate::radiance::runtime::native::BuildRequest {
        cc: input.operator.cc,
        cc_driver: input.operator.cc_driver,
        linker: input.operator.linker,
        crt_dir: input.operator.crt_dir,
        libgcc_dir: input.operator.libgcc_dir,
        bootstrap_source,
        emulator_source,
        tools_dir: &input.scratch.join("tools"),
        profile: input.profile,
    })?;
    let mut tool_observations = tools.tool_observations.clone();
    tool_observations.push(crate::radiance::runtime::native::observe_artifact(
        &seed,
        crunch_radiance_reference_core::RadianceArtifactRole::Seed,
        "Radiance seed",
    )?);
    let plan = crunch_radiance_reference_core::plan_radiance_reference(
        input.cohort.cohort_blake3.clone(),
        input.manifest.manifest_blake3.clone(),
    )
    .map_err(|error| crate::errors::RunError::Internal(format!("planning Radiance reference proof: {error:?}")))?;
    let mut stages =
        crate::radiance::runtime::execution::run_stages(crate::radiance::runtime::execution::RunRequest {
            plan: &plan,
            profile: input.profile,
            radiance_source,
            seed: &seed,
            native_tools: &tools,
            stages_dir: &input.scratch.join("stages"),
        })?;
    let mut native_events = tools.protected_audit_events.clone();
    native_events
        .try_reserve(stages.raw_audit_events.len())
        .map_err(|error| crate::errors::RunError::Internal(format!("reserving Radiance audit events: {error}")))?;
    native_events.extend(stages.raw_audit_events);
    stages.raw_audit_events = native_events;
    debug_assert_eq!(tool_observations.len(), crunch_radiance_reference_core::RADIANCE_TOOL_COUNT);
    debug_assert_eq!(stages.stages.len(), crunch_radiance_reference_core::RADIANCE_STAGE_COUNT);
    seal_graph(input, plan, tool_observations, stages, tools)
}

fn seal_graph(
    input: Input<'_, '_>,
    plan: crunch_radiance_reference_core::RadianceReferencePlan,
    tools: Vec<crunch_radiance_reference_core::RadianceArtifactObservation>,
    stages: crate::radiance::runtime::execution::RunOutputs,
    native_tools: crate::radiance::runtime::native::BuildOutputs,
) -> Result<Output, crate::errors::RunError> {
    let route_convergence = super::helpers::route_convergence(&stages.stages)?;
    let (seed_final, c99_final) = super::helpers::final_stage_pair(&stages.stages)?;
    let cross_route = crunch_radiance_reference_core::classify_cross_route(seed_final, c99_final).map_err(|error| {
        crate::errors::RunError::Internal(format!("classifying Radiance cross-route result: {error:?}"))
    })?;
    let publication = crate::radiance::publication::publish_artifacts(input.operator.output, &stages, &native_tools)?;
    let protected_audit_blake3 = crate::radiance::publication::publish_protected_audit(input.operator.output, &stages)?;
    let source_state_blake3 = super::helpers::source_state_identity(super::helpers::SourceStateIdentityInput {
        source_bundle_blake3: &input.manifest.manifest_blake3,
        cohort_blake3: &input.cohort.cohort_blake3,
    })?;
    let receipt = crunch_radiance_reference_core::seal_radiance_reference_receipt(
        crunch_radiance_reference_core::RadianceReferenceReceiptDraft {
            source_cohort: input.cohort,
            source_bundle_blake3: input.manifest.manifest_blake3.clone(),
            source_state_blake3,
            plan,
            tools,
            stages: stages.stages,
            route_convergence,
            cross_route,
            zero_events: crunch_radiance_reference_core::RadianceZeroEventCounts {
                live_fetches: 0,
                source_fallbacks: 0,
                substitutions: 0,
                ambient_discoveries: 0,
            },
            publication,
            protected_execution_audit_blake3: protected_audit_blake3.clone(),
        },
    )
    .map_err(|error| crate::errors::RunError::Internal(format!("sealing Radiance reference receipt: {error:?}")))?;
    debug_assert_eq!(receipt.zero_events.live_fetches, 0);
    debug_assert_eq!(receipt.tools.len(), crunch_radiance_reference_core::RADIANCE_TOOL_COUNT);
    Ok(Output {
        receipt,
        protected_audit_blake3,
    })
}
