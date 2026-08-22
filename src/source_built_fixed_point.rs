//! Pure plan and validation core for the source-built Mantle fixed-point proof.
//!
//! The imperative proof shell observes files, trees, and empty output roots. This
//! module accepts only those observations. It does not read files, inspect the
//! environment, start processes, or publish evidence.

use std::collections::BTreeMap;
use std::fmt;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const SOURCE_BUILT_FIXED_POINT_PLAN_SCHEMA: &str = "mantle-source-built-fixed-point-plan-v3";
pub(crate) const SOURCE_BUILT_FIXED_POINT_PROOF_WORKFLOW: &str = "mantle-deterministic-proof-receipt-v2";
pub(crate) const SOURCE_BUILT_FIXED_POINT_PROVIDER_KIND: &str = "full-source";
pub(crate) const SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX: u64 = 4_096;
pub(crate) const STAGEX_TRANSITION_STAGE_ID: &str = "stagex-transition";
pub(crate) const STAGEX_PROVIDER_STAGE_ID: &str = "stagex-provider-publication";
pub(crate) const FULL_SOURCE_NATIVE_STAGE_ID: &str = "full-source-native-provider";
pub(crate) const FULL_SOURCE_RUST_STAGE_ID: &str = "full-source-rust-provider";
pub(crate) const MANTLE_STAGE1_STAGE_ID: &str = "mantle-stage1";
pub(crate) const MANTLE_STAGE2_STAGE_ID: &str = "mantle-stage2";
const SOURCE_AUTHORITY_DIGEST_CONTEXT: &str = "mantle-source-built-fixed-point-source-authority-v1";
const PLAN_DIGEST_CONTEXT: &str = "mantle-source-built-fixed-point-plan-v3";
const BUILD_EFFECT_POLICY_VERSION: &str = "mantle-build-effects-v1";
const REQUIRED_SOURCE_ROLE_COUNT: u32 = 8;
const EXPECTED_STAGE_COUNT: u32 = 6;
const EXPECTED_RUN_COUNT: u32 = 2;
const BLAKE3_HEX_LENGTH: usize = 64;
const TEXT_BYTES_MAX: usize = 4_096;
const ELAPSED_SECONDS_MAX: u64 = 86_400;
const DISK_BYTES_MAX: u64 = 1_099_511_627_776;
const EXEC_EVENT_COUNT_MAX: u32 = 262_144;
const SOURCE_RECORD_COUNT_MAX: u32 = 65_536;
const MANTLE_STAGE2_INDEX: usize = 5;

const REQUIRED_SOURCE_ROLES: [SourceAuthorityRole; REQUIRED_SOURCE_ROLE_COUNT as usize] = [
    SourceAuthorityRole::StagexSeed,
    SourceAuthorityRole::StagexLineage,
    SourceAuthorityRole::StagexSourceBundle,
    SourceAuthorityRole::NativeSourceBundle,
    SourceAuthorityRole::RustSourceArchiveSet,
    SourceAuthorityRole::ProviderRecipeProjection,
    SourceAuthorityRole::MantleSource,
    SourceAuthorityRole::VendorInputs,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SourceAuthorityRole {
    StagexSeed,
    StagexLineage,
    StagexSourceBundle,
    NativeSourceBundle,
    RustSourceArchiveSet,
    ProviderRecipeProjection,
    MantleSource,
    VendorInputs,
    ImportedNativeProvider,
    ImportedRustProvider,
    PriorMantleOutput,
}

impl SourceAuthorityRole {
    fn is_forbidden_output_authority(self) -> bool {
        matches!(self, Self::ImportedNativeProvider | Self::ImportedRustProvider | Self::PriorMantleOutput)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SourceContentKind {
    RegularFile,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct SourceAuthorityInput {
    pub(crate) id: String,
    pub(crate) role: SourceAuthorityRole,
    pub(crate) kind: SourceContentKind,
    pub(crate) digest_blake3: String,
    pub(crate) size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct InitialOutputAuthorityState {
    pub(crate) stagex_transition_entries: u32,
    pub(crate) native_provider_entries: u32,
    pub(crate) rust_provider_entries: u32,
    pub(crate) mantle_output_entries: u32,
}

impl InitialOutputAuthorityState {
    fn is_empty(&self) -> bool {
        self.stagex_transition_entries == 0
            && self.native_provider_entries == 0
            && self.rust_provider_entries == 0
            && self.mantle_output_entries == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProofHermeticityMode {
    Strict,
    Practical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SourceBuiltFixedPointPolicies {
    pub(crate) expected_native_provider_digest_blake3: String,
    pub(crate) closure_policy_digest_blake3: String,
    pub(crate) hermeticity_policy_digest_blake3: String,
    pub(crate) protected_execution_policy_digest_blake3: String,
    pub(crate) effect_policy_digest_blake3: String,
    pub(crate) normalization_policy_digest_blake3: String,
    pub(crate) hermeticity_mode: ProofHermeticityMode,
    pub(crate) live_fetch_allowed: bool,
    pub(crate) cargo_invocation_allowed: bool,
    pub(crate) ambient_discovery_allowed: bool,
    pub(crate) fallback_allowed: bool,
    pub(crate) provider_cache_completion_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SourceBuiltFixedPointResourceBounds {
    pub(crate) elapsed_seconds_max: u64,
    pub(crate) disk_bytes_max: u64,
    pub(crate) open_file_descriptors_max: u64,
    pub(crate) protected_exec_events_max: u32,
    pub(crate) source_records_max: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceBuiltFixedPointPlanInput {
    pub(crate) proof_id: String,
    pub(crate) logical_store_prefix: String,
    pub(crate) source_inputs: Vec<SourceAuthorityInput>,
    pub(crate) initial_output_authority: InitialOutputAuthorityState,
    pub(crate) policies: SourceBuiltFixedPointPolicies,
    pub(crate) resource_bounds: SourceBuiltFixedPointResourceBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProofPolicyRole {
    Closure,
    Hermeticity,
    ProtectedExecution,
    Effect,
    Normalization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProofOutputRole {
    StagexTransition,
    StagexProvider,
    FullSourceNativeProvider,
    FullSourceRustProvider,
    MantleStage1,
    MantleStage2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "authority")]
pub(crate) enum StageAuthorityInput {
    Source { role: SourceAuthorityRole },
    Policy { role: ProofPolicyRole },
    Output { role: ProofOutputRole },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProofOrchestrator {
    HostMantle,
    Stage1Mantle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProofStageKind {
    StagexTransition,
    StagexProviderPublication,
    FullSourceNativeProvider,
    FullSourceRustProvider,
    MantleStage1,
    MantleStage2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SourceBuiltFixedPointStagePlan {
    pub(crate) stage_id: String,
    pub(crate) kind: ProofStageKind,
    pub(crate) orchestrator: ProofOrchestrator,
    pub(crate) inputs: Vec<StageAuthorityInput>,
    pub(crate) output: ProofOutputRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SourceBuiltFixedPointReceiptContract {
    pub(crate) schema: String,
    pub(crate) workflow_version: String,
    pub(crate) selected_provider_kind: String,
    pub(crate) source_blake3: String,
    pub(crate) vendor_blake3: String,
    pub(crate) logical_store_prefix: String,
    pub(crate) effect_policy_version: String,
    pub(crate) run_ids: Vec<String>,
    pub(crate) derivation_identity_source: String,
    pub(crate) toolchain_provider_identity_source: String,
    pub(crate) toolchain_stage_roots_source: String,
    pub(crate) physical_store_isolation: String,
    pub(crate) declared_effects: Vec<String>,
    pub(crate) normalized_execution_envelope: Vec<String>,
    pub(crate) sandbox_profile_identities: Vec<String>,
    pub(crate) require_content_bound_rebuild_descriptor: bool,
    pub(crate) require_rebuild_authority_plan: bool,
    pub(crate) require_matching_output_digests: bool,
    pub(crate) require_zero_authority_violations: bool,
    pub(crate) require_zero_hermeticity_events: bool,
    pub(crate) require_zero_substitutions: bool,
    pub(crate) require_zero_live_fetches: bool,
    pub(crate) require_zero_cargo_invocations: bool,
    pub(crate) require_zero_fallback_events: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SourceBuiltFixedPointPlan {
    pub(crate) schema: String,
    pub(crate) proof_id: String,
    pub(crate) logical_store_prefix: String,
    pub(crate) source_authority_digest_blake3: String,
    pub(crate) source_inputs: Vec<SourceAuthorityInput>,
    pub(crate) policies: SourceBuiltFixedPointPolicies,
    pub(crate) resource_bounds: SourceBuiltFixedPointResourceBounds,
    pub(crate) stages: Vec<SourceBuiltFixedPointStagePlan>,
    pub(crate) receipt_contract: SourceBuiltFixedPointReceiptContract,
    pub(crate) non_claims: Vec<String>,
    pub(crate) plan_digest_blake3: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceBuiltFixedPointPlanErrorKind {
    InvalidText,
    InvalidDigest,
    InvalidSourceAuthority,
    NonEmptyOutputAuthority,
    InvalidPolicy,
    InvalidResourceBounds,
    InvalidPlan,
    Serialization,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceBuiltFixedPointPlanError {
    pub(crate) kind: SourceBuiltFixedPointPlanErrorKind,
    pub(crate) message: String,
}

impl fmt::Display for SourceBuiltFixedPointPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for SourceBuiltFixedPointPlanError {}

// r[impl bootstrap_inventory.source_built_mantle_fixed_point]
pub(crate) fn plan_source_built_fixed_point(
    mut input: SourceBuiltFixedPointPlanInput,
) -> Result<SourceBuiltFixedPointPlan, SourceBuiltFixedPointPlanError> {
    validate_text("proof_id", &input.proof_id)?;
    validate_store_prefix(&input.logical_store_prefix)?;
    validate_source_inputs(&input.source_inputs)?;
    validate_initial_output_authority(&input.initial_output_authority)?;
    validate_policies(&input.policies)?;
    validate_resource_bounds(&input.resource_bounds)?;
    input.source_inputs.sort();
    let source_authority_digest_blake3 = digest_payload(SOURCE_AUTHORITY_DIGEST_CONTEXT, &input.source_inputs)?;
    let vendor_blake3 = source_input_for_role(&input.source_inputs, SourceAuthorityRole::VendorInputs)?
        .digest_blake3
        .clone();
    let mut plan = SourceBuiltFixedPointPlan {
        schema: SOURCE_BUILT_FIXED_POINT_PLAN_SCHEMA.to_string(),
        proof_id: input.proof_id,
        logical_store_prefix: input.logical_store_prefix.clone(),
        source_authority_digest_blake3: source_authority_digest_blake3.clone(),
        source_inputs: input.source_inputs,
        policies: input.policies,
        resource_bounds: input.resource_bounds,
        stages: expected_stage_plans(),
        receipt_contract: receipt_contract(
            &input.logical_store_prefix,
            &source_authority_digest_blake3,
            &vendor_blake3,
        ),
        non_claims: fixed_point_non_claims(),
        plan_digest_blake3: String::new(),
    };
    plan.plan_digest_blake3 = plan_digest_blake3(&plan)?;
    validate_source_built_fixed_point_plan(&plan)?;
    debug_assert_eq!(plan.stages.len(), EXPECTED_STAGE_COUNT as usize);
    debug_assert_eq!(plan.receipt_contract.run_ids.len(), EXPECTED_RUN_COUNT as usize);
    Ok(plan)
}

pub(crate) fn validate_source_built_fixed_point_plan(
    plan: &SourceBuiltFixedPointPlan,
) -> Result<(), SourceBuiltFixedPointPlanError> {
    if plan.schema != SOURCE_BUILT_FIXED_POINT_PLAN_SCHEMA {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidPlan,
            format!("fixed-point plan schema must be {SOURCE_BUILT_FIXED_POINT_PLAN_SCHEMA}"),
        ));
    }
    validate_text("proof_id", &plan.proof_id)?;
    validate_store_prefix(&plan.logical_store_prefix)?;
    validate_digest("source_authority_digest_blake3", &plan.source_authority_digest_blake3)?;
    validate_source_inputs(&plan.source_inputs)?;
    validate_policies(&plan.policies)?;
    validate_resource_bounds(&plan.resource_bounds)?;
    validate_stage_plans(&plan.stages)?;
    validate_receipt_contract(plan)?;
    validate_non_claims(&plan.non_claims)?;
    validate_digest("plan_digest_blake3", &plan.plan_digest_blake3)?;
    let expected_digest = plan_digest_blake3(plan)?;
    if plan.plan_digest_blake3 != expected_digest {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidPlan,
            "fixed-point plan digest does not match canonical plan content".to_string(),
        ));
    }
    debug_assert_eq!(plan.schema, SOURCE_BUILT_FIXED_POINT_PLAN_SCHEMA);
    debug_assert!(!plan.non_claims.is_empty());
    Ok(())
}

fn validate_source_inputs(inputs: &[SourceAuthorityInput]) -> Result<(), SourceBuiltFixedPointPlanError> {
    let input_count = u32::try_from(inputs.len()).map_err(|_| {
        plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidSourceAuthority,
            "source authority input count exceeds u32".to_string(),
        )
    })?;
    if input_count != REQUIRED_SOURCE_ROLE_COUNT {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidSourceAuthority,
            format!("source authority must contain exactly {REQUIRED_SOURCE_ROLE_COUNT} inputs, got {input_count}"),
        ));
    }
    let mut roles = BTreeMap::new();
    let mut ids = BTreeMap::new();
    for input in inputs {
        validate_source_input(input)?;
        if input.role.is_forbidden_output_authority() {
            return Err(plan_error(
                SourceBuiltFixedPointPlanErrorKind::InvalidSourceAuthority,
                format!("source authority input '{}' is a forbidden provider or prior-output authority", input.id),
            ));
        }
        if roles.insert(input.role, input.id.as_str()).is_some() {
            return Err(plan_error(
                SourceBuiltFixedPointPlanErrorKind::InvalidSourceAuthority,
                format!("source authority duplicates role {:?}", input.role),
            ));
        }
        if ids.insert(input.id.as_str(), input.role).is_some() {
            return Err(plan_error(
                SourceBuiltFixedPointPlanErrorKind::InvalidSourceAuthority,
                format!("source authority duplicates input id '{}'", input.id),
            ));
        }
    }
    for role in REQUIRED_SOURCE_ROLES {
        if !roles.contains_key(&role) {
            return Err(plan_error(
                SourceBuiltFixedPointPlanErrorKind::InvalidSourceAuthority,
                format!("source authority is missing required role {role:?}"),
            ));
        }
    }
    debug_assert_eq!(roles.len(), REQUIRED_SOURCE_ROLE_COUNT as usize);
    debug_assert_eq!(ids.len(), REQUIRED_SOURCE_ROLE_COUNT as usize);
    Ok(())
}

fn validate_source_input(input: &SourceAuthorityInput) -> Result<(), SourceBuiltFixedPointPlanError> {
    validate_text("source input id", &input.id)?;
    validate_digest("source input digest_blake3", &input.digest_blake3)?;
    if input.size_bytes == 0 {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidSourceAuthority,
            format!("source authority input '{}' is empty", input.id),
        ));
    }
    let expected_kind = expected_content_kind(input.role);
    if input.kind != expected_kind {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidSourceAuthority,
            format!("source authority input '{}' has kind {:?}, expected {expected_kind:?}", input.id, input.kind),
        ));
    }
    debug_assert!(!input.id.is_empty());
    debug_assert_eq!(input.digest_blake3.len(), BLAKE3_HEX_LENGTH);
    Ok(())
}

fn expected_content_kind(role: SourceAuthorityRole) -> SourceContentKind {
    match role {
        SourceAuthorityRole::StagexSeed
        | SourceAuthorityRole::StagexLineage
        | SourceAuthorityRole::StagexSourceBundle
        | SourceAuthorityRole::NativeSourceBundle => SourceContentKind::RegularFile,
        SourceAuthorityRole::RustSourceArchiveSet
        | SourceAuthorityRole::ProviderRecipeProjection
        | SourceAuthorityRole::MantleSource
        | SourceAuthorityRole::VendorInputs
        | SourceAuthorityRole::ImportedNativeProvider
        | SourceAuthorityRole::ImportedRustProvider
        | SourceAuthorityRole::PriorMantleOutput => SourceContentKind::Directory,
    }
}

fn validate_initial_output_authority(
    state: &InitialOutputAuthorityState,
) -> Result<(), SourceBuiltFixedPointPlanError> {
    if !state.is_empty() {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::NonEmptyOutputAuthority,
            "proof root must start with empty transition, provider, Rust-provider, and Mantle output authorities"
                .to_string(),
        ));
    }
    debug_assert_eq!(state.stagex_transition_entries, 0);
    debug_assert_eq!(state.mantle_output_entries, 0);
    Ok(())
}

fn validate_policies(policies: &SourceBuiltFixedPointPolicies) -> Result<(), SourceBuiltFixedPointPlanError> {
    for (name, digest) in [
        ("expected native provider", &policies.expected_native_provider_digest_blake3),
        ("closure policy", &policies.closure_policy_digest_blake3),
        ("hermeticity policy", &policies.hermeticity_policy_digest_blake3),
        ("protected execution policy", &policies.protected_execution_policy_digest_blake3),
        ("effect policy", &policies.effect_policy_digest_blake3),
        ("normalization policy", &policies.normalization_policy_digest_blake3),
    ] {
        validate_digest(name, digest)?;
    }
    let strict = policies.hermeticity_mode == ProofHermeticityMode::Strict;
    let forbidden_effect = policies.live_fetch_allowed
        || policies.cargo_invocation_allowed
        || policies.ambient_discovery_allowed
        || policies.fallback_allowed
        || policies.provider_cache_completion_allowed;
    if !strict || forbidden_effect {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidPolicy,
            "source-built fixed-point proof requires strict hermeticity and forbids fetch, Cargo, ambient discovery, fallback, and provider-cache completion".to_string(),
        ));
    }
    debug_assert!(strict);
    debug_assert!(!forbidden_effect);
    Ok(())
}

fn validate_resource_bounds(
    bounds: &SourceBuiltFixedPointResourceBounds,
) -> Result<(), SourceBuiltFixedPointPlanError> {
    let elapsed_valid = bounds.elapsed_seconds_max > 0 && bounds.elapsed_seconds_max <= ELAPSED_SECONDS_MAX;
    let disk_valid = bounds.disk_bytes_max > 0 && bounds.disk_bytes_max <= DISK_BYTES_MAX;
    let open_files_valid = bounds.open_file_descriptors_max > 0
        && bounds.open_file_descriptors_max <= SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
    let exec_events_valid =
        bounds.protected_exec_events_max > 0 && bounds.protected_exec_events_max <= EXEC_EVENT_COUNT_MAX;
    let source_records_valid = bounds.source_records_max > 0 && bounds.source_records_max <= SOURCE_RECORD_COUNT_MAX;
    if !elapsed_valid || !disk_valid || !open_files_valid || !exec_events_valid || !source_records_valid {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidResourceBounds,
            "source-built fixed-point resource bounds are zero or exceed the accepted maxima".to_string(),
        ));
    }
    debug_assert!(bounds.elapsed_seconds_max <= ELAPSED_SECONDS_MAX);
    debug_assert!(bounds.open_file_descriptors_max <= SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX);
    debug_assert!(bounds.source_records_max <= SOURCE_RECORD_COUNT_MAX);
    Ok(())
}

fn validate_stage_plans(stages: &[SourceBuiltFixedPointStagePlan]) -> Result<(), SourceBuiltFixedPointPlanError> {
    let count = u32::try_from(stages.len()).map_err(|_| {
        plan_error(SourceBuiltFixedPointPlanErrorKind::InvalidPlan, "fixed-point stage count exceeds u32".to_string())
    })?;
    if count != EXPECTED_STAGE_COUNT {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidPlan,
            format!("fixed-point plan must contain {EXPECTED_STAGE_COUNT} stages, got {count}"),
        ));
    }
    if stages != expected_stage_plans() {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidPlan,
            "fixed-point stages do not match the required construction and orchestration sequence".to_string(),
        ));
    }
    debug_assert_eq!(stages[MANTLE_STAGE2_INDEX].orchestrator, ProofOrchestrator::Stage1Mantle);
    debug_assert_eq!(stages[MANTLE_STAGE2_INDEX].output, ProofOutputRole::MantleStage2);
    Ok(())
}

fn validate_receipt_contract(plan: &SourceBuiltFixedPointPlan) -> Result<(), SourceBuiltFixedPointPlanError> {
    let contract = &plan.receipt_contract;
    let vendor_digest = source_input_for_role(&plan.source_inputs, SourceAuthorityRole::VendorInputs)?
        .digest_blake3
        .as_str();
    let expected = receipt_contract(&plan.logical_store_prefix, &plan.source_authority_digest_blake3, vendor_digest);
    if contract != &expected {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidPlan,
            "fixed-point v2 receipt contract does not match the source and policy plan".to_string(),
        ));
    }
    debug_assert_eq!(contract.schema, SOURCE_BUILT_FIXED_POINT_PROOF_WORKFLOW);
    debug_assert!(contract.require_rebuild_authority_plan);
    Ok(())
}

fn validate_non_claims(non_claims: &[String]) -> Result<(), SourceBuiltFixedPointPlanError> {
    let expected = fixed_point_non_claims();
    if non_claims != expected {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidPlan,
            "fixed-point non-claims do not match the bounded proof contract".to_string(),
        ));
    }
    debug_assert!(!non_claims.is_empty());
    debug_assert!(non_claims.iter().all(|claim| !claim.trim().is_empty()));
    Ok(())
}

fn source_input_for_role(
    inputs: &[SourceAuthorityInput],
    role: SourceAuthorityRole,
) -> Result<&SourceAuthorityInput, SourceBuiltFixedPointPlanError> {
    let mut matches = inputs.iter().filter(|input| input.role == role);
    let input = matches.next().ok_or_else(|| {
        plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidSourceAuthority,
            format!("source authority is missing required role {role:?}"),
        )
    })?;
    if matches.next().is_some() {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidSourceAuthority,
            format!("source authority duplicates required role {role:?}"),
        ));
    }
    Ok(input)
}

pub(crate) fn expected_stage_plans() -> Vec<SourceBuiltFixedPointStagePlan> {
    let stages = vec![
        stagex_transition_stage(),
        stagex_provider_stage(),
        full_source_native_stage(),
        full_source_rust_stage(),
        mantle_stage1_plan(),
        mantle_stage2_plan(),
    ];
    debug_assert_eq!(stages.len(), EXPECTED_STAGE_COUNT as usize);
    debug_assert_eq!(stages[MANTLE_STAGE2_INDEX].orchestrator, ProofOrchestrator::Stage1Mantle);
    stages
}

fn stagex_transition_stage() -> SourceBuiltFixedPointStagePlan {
    stage_plan(
        STAGEX_TRANSITION_STAGE_ID,
        ProofStageKind::StagexTransition,
        ProofOrchestrator::HostMantle,
        vec![
            source_authority(SourceAuthorityRole::StagexSeed),
            source_authority(SourceAuthorityRole::StagexLineage),
            source_authority(SourceAuthorityRole::StagexSourceBundle),
            source_authority(SourceAuthorityRole::NativeSourceBundle),
            source_authority(SourceAuthorityRole::ProviderRecipeProjection),
            policy_authority(ProofPolicyRole::ProtectedExecution),
        ],
        ProofOutputRole::StagexTransition,
    )
}

fn stagex_provider_stage() -> SourceBuiltFixedPointStagePlan {
    stage_plan(
        STAGEX_PROVIDER_STAGE_ID,
        ProofStageKind::StagexProviderPublication,
        ProofOrchestrator::HostMantle,
        vec![
            output_authority(ProofOutputRole::StagexTransition),
            source_authority(SourceAuthorityRole::StagexLineage),
            policy_authority(ProofPolicyRole::ProtectedExecution),
            policy_authority(ProofPolicyRole::Hermeticity),
        ],
        ProofOutputRole::StagexProvider,
    )
}

fn full_source_native_stage() -> SourceBuiltFixedPointStagePlan {
    stage_plan(
        FULL_SOURCE_NATIVE_STAGE_ID,
        ProofStageKind::FullSourceNativeProvider,
        ProofOrchestrator::HostMantle,
        vec![
            output_authority(ProofOutputRole::StagexProvider),
            source_authority(SourceAuthorityRole::NativeSourceBundle),
            source_authority(SourceAuthorityRole::ProviderRecipeProjection),
            policy_authority(ProofPolicyRole::Closure),
            policy_authority(ProofPolicyRole::Hermeticity),
            policy_authority(ProofPolicyRole::ProtectedExecution),
            policy_authority(ProofPolicyRole::Effect),
        ],
        ProofOutputRole::FullSourceNativeProvider,
    )
}

fn full_source_rust_stage() -> SourceBuiltFixedPointStagePlan {
    stage_plan(
        FULL_SOURCE_RUST_STAGE_ID,
        ProofStageKind::FullSourceRustProvider,
        ProofOrchestrator::HostMantle,
        vec![
            output_authority(ProofOutputRole::FullSourceNativeProvider),
            source_authority(SourceAuthorityRole::RustSourceArchiveSet),
            source_authority(SourceAuthorityRole::ProviderRecipeProjection),
            policy_authority(ProofPolicyRole::Closure),
            policy_authority(ProofPolicyRole::Hermeticity),
            policy_authority(ProofPolicyRole::ProtectedExecution),
            policy_authority(ProofPolicyRole::Effect),
        ],
        ProofOutputRole::FullSourceRustProvider,
    )
}

fn mantle_stage1_plan() -> SourceBuiltFixedPointStagePlan {
    stage_plan(
        MANTLE_STAGE1_STAGE_ID,
        ProofStageKind::MantleStage1,
        ProofOrchestrator::HostMantle,
        mantle_build_inputs(false),
        ProofOutputRole::MantleStage1,
    )
}

fn mantle_stage2_plan() -> SourceBuiltFixedPointStagePlan {
    stage_plan(
        MANTLE_STAGE2_STAGE_ID,
        ProofStageKind::MantleStage2,
        ProofOrchestrator::Stage1Mantle,
        mantle_build_inputs(true),
        ProofOutputRole::MantleStage2,
    )
}

fn mantle_build_inputs(include_stage1: bool) -> Vec<StageAuthorityInput> {
    let mut inputs = vec![
        output_authority(ProofOutputRole::FullSourceNativeProvider),
        output_authority(ProofOutputRole::FullSourceRustProvider),
    ];
    if include_stage1 {
        inputs.push(output_authority(ProofOutputRole::MantleStage1));
    }
    inputs.extend([
        source_authority(SourceAuthorityRole::MantleSource),
        source_authority(SourceAuthorityRole::VendorInputs),
        policy_authority(ProofPolicyRole::Closure),
        policy_authority(ProofPolicyRole::Hermeticity),
        policy_authority(ProofPolicyRole::ProtectedExecution),
        policy_authority(ProofPolicyRole::Effect),
        policy_authority(ProofPolicyRole::Normalization),
    ]);
    debug_assert!(inputs.contains(&output_authority(ProofOutputRole::FullSourceRustProvider)));
    debug_assert_eq!(inputs.contains(&output_authority(ProofOutputRole::MantleStage1)), include_stage1);
    inputs
}

fn stage_plan(
    stage_id: &str,
    kind: ProofStageKind,
    orchestrator: ProofOrchestrator,
    inputs: Vec<StageAuthorityInput>,
    output: ProofOutputRole,
) -> SourceBuiltFixedPointStagePlan {
    debug_assert!(!stage_id.is_empty());
    debug_assert!(!inputs.is_empty());
    SourceBuiltFixedPointStagePlan {
        stage_id: stage_id.to_string(),
        kind,
        orchestrator,
        inputs,
        output,
    }
}

fn source_authority(role: SourceAuthorityRole) -> StageAuthorityInput {
    StageAuthorityInput::Source { role }
}

fn output_authority(role: ProofOutputRole) -> StageAuthorityInput {
    StageAuthorityInput::Output { role }
}

fn policy_authority(role: ProofPolicyRole) -> StageAuthorityInput {
    StageAuthorityInput::Policy { role }
}

fn receipt_contract(
    logical_store_prefix: &str,
    source_authority_digest_blake3: &str,
    vendor_blake3: &str,
) -> SourceBuiltFixedPointReceiptContract {
    let contract = SourceBuiltFixedPointReceiptContract {
        schema: SOURCE_BUILT_FIXED_POINT_PROOF_WORKFLOW.to_string(),
        workflow_version: SOURCE_BUILT_FIXED_POINT_PROOF_WORKFLOW.to_string(),
        selected_provider_kind: SOURCE_BUILT_FIXED_POINT_PROVIDER_KIND.to_string(),
        source_blake3: source_authority_digest_blake3.to_string(),
        vendor_blake3: vendor_blake3.to_string(),
        logical_store_prefix: logical_store_prefix.to_string(),
        effect_policy_version: BUILD_EFFECT_POLICY_VERSION.to_string(),
        run_ids: vec!["stage1".to_string(), "stage2".to_string()],
        derivation_identity_source: "cargo-free-native-topology-plan".to_string(),
        toolchain_provider_identity_source: "full-source-native-provider-output".to_string(),
        toolchain_stage_roots_source: "provider-construction-stage-receipts".to_string(),
        physical_store_isolation: "fresh-proof-root-per-run".to_string(),
        declared_effects: vec![
            "read-store".to_string(),
            "write-store".to_string(),
            "execute-process".to_string(),
        ],
        normalized_execution_envelope: vec![
            "cargo-forbidden".to_string(),
            "live-fetch-forbidden".to_string(),
            "open-file-descriptors-bounded".to_string(),
            "strict-hermeticity".to_string(),
            "stage1-orchestrates-stage2".to_string(),
        ],
        sandbox_profile_identities: vec![
            "mantle-proof-sandbox-v1:stage1".to_string(),
            "mantle-proof-sandbox-v1:stage2".to_string(),
        ],
        require_content_bound_rebuild_descriptor: true,
        require_rebuild_authority_plan: true,
        require_matching_output_digests: true,
        require_zero_authority_violations: true,
        require_zero_hermeticity_events: true,
        require_zero_substitutions: true,
        require_zero_live_fetches: true,
        require_zero_cargo_invocations: true,
        require_zero_fallback_events: true,
    };
    debug_assert_eq!(contract.run_ids.len(), EXPECTED_RUN_COUNT as usize);
    debug_assert!(contract.require_content_bound_rebuild_descriptor);
    contract
}

fn fixed_point_non_claims() -> Vec<String> {
    vec![
        "not-compiler-correctness".to_string(),
        "not-bootstrap-seed-correctness".to_string(),
        "not-kernel-isolation".to_string(),
        "not-independent-rebuild-agreement".to_string(),
        "not-release-reproducibility".to_string(),
        "not-deployment-success".to_string(),
        "not-full-cargo-compatibility".to_string(),
    ]
}

fn plan_digest_blake3(plan: &SourceBuiltFixedPointPlan) -> Result<String, SourceBuiltFixedPointPlanError> {
    let mut payload = plan.clone();
    payload.plan_digest_blake3.clear();
    digest_payload(PLAN_DIGEST_CONTEXT, &payload)
}

fn digest_payload<T: Serialize>(context: &str, value: &T) -> Result<String, SourceBuiltFixedPointPlanError> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        plan_error(
            SourceBuiltFixedPointPlanErrorKind::Serialization,
            format!("serializing source-built fixed-point plan payload: {error}"),
        )
    })?;
    let mut hasher = blake3::Hasher::new_derive_key(context);
    hasher.update(&bytes);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert!(!bytes.is_empty());
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    Ok(digest)
}

fn validate_text(name: &str, value: &str) -> Result<(), SourceBuiltFixedPointPlanError> {
    let is_valid = !value.trim().is_empty() && value.len() <= TEXT_BYTES_MAX && !value.chars().any(char::is_control);
    if !is_valid {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidText,
            format!("{name} must be nonempty, bounded UTF-8 text without control characters"),
        ));
    }
    debug_assert!(!value.trim().is_empty());
    debug_assert!(value.len() <= TEXT_BYTES_MAX);
    Ok(())
}

fn validate_store_prefix(prefix: &str) -> Result<(), SourceBuiltFixedPointPlanError> {
    validate_text("logical_store_prefix", prefix)?;
    let has_parent_component = prefix.split('/').any(|component| component == "..");
    if !prefix.starts_with('/') || prefix == "/" || prefix.ends_with('/') || has_parent_component {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidText,
            "logical_store_prefix must be an absolute non-root path without a trailing slash or parent component"
                .to_string(),
        ));
    }
    debug_assert!(prefix.starts_with('/'));
    debug_assert!(!prefix.ends_with('/'));
    Ok(())
}

fn validate_digest(name: &str, digest: &str) -> Result<(), SourceBuiltFixedPointPlanError> {
    let valid = digest.len() == BLAKE3_HEX_LENGTH
        && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid {
        return Err(plan_error(
            SourceBuiltFixedPointPlanErrorKind::InvalidDigest,
            format!("{name} must be 64 lowercase hexadecimal BLAKE3 characters"),
        ));
    }
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    Ok(())
}

fn plan_error(kind: SourceBuiltFixedPointPlanErrorKind, message: String) -> SourceBuiltFixedPointPlanError {
    SourceBuiltFixedPointPlanError { kind, message }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ELAPSED_SECONDS_MAX: u64 = 43_200;
    const TEST_DISK_BYTES_MAX: u64 = 536_870_912_000;
    const TEST_OPEN_FILE_DESCRIPTORS_MAX: u64 = SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX / 2;
    const TEST_EXEC_EVENT_COUNT_MAX: u32 = 131_072;
    const TEST_SOURCE_RECORD_COUNT_MAX: u32 = 32_768;

    #[test]
    fn plans_complete_source_built_fixed_point_authority() {
        let plan = plan_source_built_fixed_point(valid_input()).unwrap();

        assert_eq!(plan.schema, SOURCE_BUILT_FIXED_POINT_PLAN_SCHEMA);
        assert_eq!(plan.stages.len(), EXPECTED_STAGE_COUNT as usize);
        assert_eq!(plan.stages[MANTLE_STAGE2_INDEX].orchestrator, ProofOrchestrator::Stage1Mantle);
        assert_eq!(plan.receipt_contract.schema, SOURCE_BUILT_FIXED_POINT_PROOF_WORKFLOW);
        assert!(plan.receipt_contract.require_rebuild_authority_plan);
        assert_eq!(plan.plan_digest_blake3.len(), BLAKE3_HEX_LENGTH);
        assert!(validate_source_built_fixed_point_plan(&plan).is_ok());
    }

    #[test]
    fn plan_is_stable_across_source_input_order() {
        let first = plan_source_built_fixed_point(valid_input()).unwrap();
        let mut reversed = valid_input();
        reversed.source_inputs.reverse();
        let second = plan_source_built_fixed_point(reversed).unwrap();

        assert_eq!(first.source_authority_digest_blake3, second.source_authority_digest_blake3);
        assert_eq!(first.plan_digest_blake3, second.plan_digest_blake3);
        assert_eq!(first.source_inputs, second.source_inputs);
    }

    #[test]
    fn rejects_missing_required_source_role() {
        let mut input = valid_input();
        input.source_inputs.retain(|entry| entry.role != SourceAuthorityRole::VendorInputs);
        let error = plan_source_built_fixed_point(input).unwrap_err();

        assert_eq!(error.kind, SourceBuiltFixedPointPlanErrorKind::InvalidSourceAuthority);
        assert!(error.message.contains("exactly"));
        assert!(error.message.contains("inputs"));
    }

    #[test]
    fn rejects_imported_provider_as_source_authority() {
        let mut input = valid_input();
        let provider = input
            .source_inputs
            .iter_mut()
            .find(|entry| entry.role == SourceAuthorityRole::VendorInputs)
            .unwrap();
        provider.role = SourceAuthorityRole::ImportedRustProvider;
        let error = plan_source_built_fixed_point(input).unwrap_err();

        assert_eq!(error.kind, SourceBuiltFixedPointPlanErrorKind::InvalidSourceAuthority);
        assert!(error.message.contains("forbidden provider"));
        assert!(error.message.contains("prior-output authority"));
    }

    #[test]
    fn rejects_nonempty_initial_output_authority() {
        let mut input = valid_input();
        input.initial_output_authority.native_provider_entries = 1;
        let error = plan_source_built_fixed_point(input).unwrap_err();

        assert_eq!(error.kind, SourceBuiltFixedPointPlanErrorKind::NonEmptyOutputAuthority);
        assert!(error.message.contains("must start with empty"));
        assert!(error.message.contains("provider"));
    }

    #[test]
    fn rejects_invalid_expected_native_provider_digest() {
        let mut input = valid_input();
        input.policies.expected_native_provider_digest_blake3 = "not-a-digest".to_string();

        let error = plan_source_built_fixed_point(input).unwrap_err();

        assert!(error.to_string().contains("expected native provider"));
        assert!(error.to_string().contains("lowercase hexadecimal BLAKE3"));
    }

    #[test]
    fn rejects_practical_or_fallback_policy() {
        let mut practical = valid_input();
        practical.policies.hermeticity_mode = ProofHermeticityMode::Practical;
        let practical_error = plan_source_built_fixed_point(practical).unwrap_err();
        let mut fallback = valid_input();
        fallback.policies.fallback_allowed = true;
        let fallback_error = plan_source_built_fixed_point(fallback).unwrap_err();

        assert_eq!(practical_error.kind, SourceBuiltFixedPointPlanErrorKind::InvalidPolicy);
        assert_eq!(fallback_error.kind, SourceBuiltFixedPointPlanErrorKind::InvalidPolicy);
        assert!(practical_error.message.contains("strict hermeticity"));
        assert!(fallback_error.message.contains("fallback"));
    }

    #[test]
    fn rejects_out_of_range_resource_bounds() {
        let mut elapsed = valid_input();
        elapsed.resource_bounds.elapsed_seconds_max = ELAPSED_SECONDS_MAX.saturating_add(1);
        let elapsed_error = plan_source_built_fixed_point(elapsed).unwrap_err();
        let mut open_files = valid_input();
        open_files.resource_bounds.open_file_descriptors_max =
            SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX.saturating_add(1);
        let open_files_error = plan_source_built_fixed_point(open_files).unwrap_err();

        assert_eq!(elapsed_error.kind, SourceBuiltFixedPointPlanErrorKind::InvalidResourceBounds);
        assert_eq!(open_files_error.kind, SourceBuiltFixedPointPlanErrorKind::InvalidResourceBounds);
        assert!(elapsed_error.message.contains("resource bounds"));
        assert!(open_files_error.message.contains("maxima"));
    }

    #[test]
    fn rejects_host_orchestration_for_stage2() {
        let mut plan = plan_source_built_fixed_point(valid_input()).unwrap();
        plan.stages[MANTLE_STAGE2_INDEX].orchestrator = ProofOrchestrator::HostMantle;
        plan.plan_digest_blake3 = plan_digest_blake3(&plan).unwrap();
        let error = validate_source_built_fixed_point_plan(&plan).unwrap_err();

        assert_eq!(error.kind, SourceBuiltFixedPointPlanErrorKind::InvalidPlan);
        assert!(error.message.contains("construction and orchestration sequence"));
        assert_ne!(plan.stages[MANTLE_STAGE2_INDEX], expected_stage_plans()[MANTLE_STAGE2_INDEX]);
    }

    #[test]
    fn rejects_stale_plan_digest() {
        let mut plan = plan_source_built_fixed_point(valid_input()).unwrap();
        plan.proof_id.push_str("-changed");
        let error = validate_source_built_fixed_point_plan(&plan).unwrap_err();

        assert_eq!(error.kind, SourceBuiltFixedPointPlanErrorKind::InvalidPlan);
        assert!(error.message.contains("plan digest"));
        assert_ne!(plan.plan_digest_blake3, plan_digest_blake3(&plan).unwrap());
    }

    fn valid_input() -> SourceBuiltFixedPointPlanInput {
        SourceBuiltFixedPointPlanInput {
            proof_id: "source-built-mantle-fixed-point-test".to_string(),
            logical_store_prefix: "/mantle/store".to_string(),
            source_inputs: vec![
                source_input("stagex-seed", SourceAuthorityRole::StagexSeed, SourceContentKind::RegularFile, 'a'),
                source_input("stagex-lineage", SourceAuthorityRole::StagexLineage, SourceContentKind::RegularFile, 'b'),
                source_input(
                    "stagex-source-bundle",
                    SourceAuthorityRole::StagexSourceBundle,
                    SourceContentKind::RegularFile,
                    'c',
                ),
                source_input(
                    "native-source-bundle",
                    SourceAuthorityRole::NativeSourceBundle,
                    SourceContentKind::RegularFile,
                    'd',
                ),
                source_input(
                    "rust-source-archives",
                    SourceAuthorityRole::RustSourceArchiveSet,
                    SourceContentKind::Directory,
                    'e',
                ),
                source_input(
                    "provider-recipe-projection",
                    SourceAuthorityRole::ProviderRecipeProjection,
                    SourceContentKind::Directory,
                    '8',
                ),
                source_input("mantle-source", SourceAuthorityRole::MantleSource, SourceContentKind::Directory, 'f'),
                source_input("vendor-inputs", SourceAuthorityRole::VendorInputs, SourceContentKind::Directory, '7'),
            ],
            initial_output_authority: InitialOutputAuthorityState {
                stagex_transition_entries: 0,
                native_provider_entries: 0,
                rust_provider_entries: 0,
                mantle_output_entries: 0,
            },
            policies: SourceBuiltFixedPointPolicies {
                expected_native_provider_digest_blake3: digest('0'),
                closure_policy_digest_blake3: digest('1'),
                hermeticity_policy_digest_blake3: digest('2'),
                protected_execution_policy_digest_blake3: digest('3'),
                effect_policy_digest_blake3: digest('4'),
                normalization_policy_digest_blake3: digest('5'),
                hermeticity_mode: ProofHermeticityMode::Strict,
                live_fetch_allowed: false,
                cargo_invocation_allowed: false,
                ambient_discovery_allowed: false,
                fallback_allowed: false,
                provider_cache_completion_allowed: false,
            },
            resource_bounds: SourceBuiltFixedPointResourceBounds {
                elapsed_seconds_max: TEST_ELAPSED_SECONDS_MAX,
                disk_bytes_max: TEST_DISK_BYTES_MAX,
                open_file_descriptors_max: TEST_OPEN_FILE_DESCRIPTORS_MAX,
                protected_exec_events_max: TEST_EXEC_EVENT_COUNT_MAX,
                source_records_max: TEST_SOURCE_RECORD_COUNT_MAX,
            },
        }
    }

    fn source_input(
        id: &str,
        role: SourceAuthorityRole,
        kind: SourceContentKind,
        digest_character: char,
    ) -> SourceAuthorityInput {
        SourceAuthorityInput {
            id: id.to_string(),
            role,
            kind,
            digest_blake3: digest(digest_character),
            size_bytes: 1,
        }
    }

    fn digest(character: char) -> String {
        character.to_string().repeat(BLAKE3_HEX_LENGTH)
    }
}
