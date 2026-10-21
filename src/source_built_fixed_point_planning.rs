//! Construction-only proof planning, compiled by the binary and regression tests.
//! Receipt verification uses the shared contracts without observing empty output roots
//! or manufacturing a new action plan from supplied evidence.

use serde::Deserialize;
use serde::Serialize;

use crate::source_built_fixed_point::*;
use crate::source_built_fixed_point_action_trust::ACTION_TRUST_SCHEMA;
use crate::source_built_fixed_point_action_trust::ActionTrustPlan;
use crate::source_built_fixed_point_action_trust::StageActionAdapter;
use crate::source_built_fixed_point_action_trust::action_plan_digest;
use crate::source_built_fixed_point_action_trust::action_root_identity;
use crate::source_built_fixed_point_action_trust::validate_action_graph;

const SOURCE_AUTHORITY_DIGEST_CONTEXT: &str = "mantle-source-built-fixed-point-source-authority-v1";

/// Construction has an initial-empty precondition that receipt validation does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SourceBuiltFixedPointPlanningError {
    NonEmptyOutputAuthority,
    InvalidPlan(SourceBuiltFixedPointPlanError),
}

impl std::fmt::Display for SourceBuiltFixedPointPlanningError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonEmptyOutputAuthority => formatter.write_str(
                "proof root must start with empty transition, provider, Rust-provider, and Mantle output authorities",
            ),
            Self::InvalidPlan(error) => std::fmt::Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for SourceBuiltFixedPointPlanningError {}

impl From<SourceBuiltFixedPointPlanError> for SourceBuiltFixedPointPlanningError {
    fn from(error: SourceBuiltFixedPointPlanError) -> Self {
        Self::InvalidPlan(error)
    }
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceBuiltFixedPointPlanInput {
    pub(crate) proof_id: String,
    pub(crate) logical_store_prefix: String,
    pub(crate) source_inputs: Vec<SourceAuthorityInput>,
    pub(crate) initial_output_authority: InitialOutputAuthorityState,
    pub(crate) policies: SourceBuiltFixedPointPolicies,
    pub(crate) resource_bounds: SourceBuiltFixedPointResourceBounds,
}
// r[impl bootstrap_inventory.source_built_mantle_fixed_point]
pub(crate) fn plan_source_built_fixed_point(
    mut input: SourceBuiltFixedPointPlanInput,
) -> Result<SourceBuiltFixedPointPlan, SourceBuiltFixedPointPlanningError> {
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
fn validate_initial_output_authority(
    state: &InitialOutputAuthorityState,
) -> Result<(), SourceBuiltFixedPointPlanningError> {
    if !state.is_empty() {
        return Err(SourceBuiltFixedPointPlanningError::NonEmptyOutputAuthority);
    }
    debug_assert_eq!(state.stagex_transition_entries, 0);
    debug_assert_eq!(state.mantle_output_entries, 0);
    Ok(())
}
// r[impl bootstrap_inventory.source_built_mantle_fixed_point]
pub(crate) fn plan_actions(
    proof: &SourceBuiltFixedPointPlan,
    adapters: Vec<StageActionAdapter>,
) -> Result<ActionTrustPlan, String> {
    validate_source_built_fixed_point_plan(proof).map_err(|error| error.to_string())?;
    let proof_root_identity_blake3 = action_root_identity(proof)?;
    let mut plan = ActionTrustPlan {
        schema: ACTION_TRUST_SCHEMA.into(),
        proof_plan_digest_blake3: proof.plan_digest_blake3.clone(),
        proof_root_identity_blake3,
        adapters,
        digest_blake3: String::new(),
    };
    validate_action_graph(proof, &plan)?;
    plan.digest_blake3 = action_plan_digest(&plan)?;
    Ok(plan)
}
