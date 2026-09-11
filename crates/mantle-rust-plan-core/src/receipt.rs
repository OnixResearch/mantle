//! Receipt preimages for admitted plans.

use alloc::string::String;
use alloc::vec::Vec;

use serde::Serialize;

use crate::digest::Blake3Digest;
use crate::digest::domain_digest;
use crate::plan::RustPlan;

/// Domain tag for receipt preimage identities.
const RECEIPT_DOMAIN: &[u8] = b"mantle-rust-plan-receipt-v1\0";

/// Receipt preimage schema identifier.
pub const RECEIPT_PREIMAGE_SCHEMA: &str = "mantle-rust-plan-receipt-preimage-v1";

/// Canonical receipt preimage for one plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanReceiptPreimage {
    pub schema: String,
    pub plan_schema: String,
    pub plan_blake3: Blake3Digest,
    pub package_keys: Vec<String>,
    pub unit_count: u32,
    pub effect_count: u32,
    pub preimage_blake3: Blake3Digest,
}

#[derive(Serialize)]
struct ReceiptIdentityInput<'a> {
    schema: &'a str,
    plan_schema: &'a str,
    plan_blake3: &'a Blake3Digest,
    package_keys: &'a [String],
    unit_count: u32,
    effect_count: u32,
}

/// Build the canonical receipt preimage for a completed plan.
pub fn build_receipt_preimage(plan: &RustPlan) -> Option<PlanReceiptPreimage> {
    if plan.outcome != crate::plan::PlanOutcome::Completed {
        return None;
    }
    let mut package_keys: Vec<String> = plan.units.iter().map(|unit| unit.package_key.clone()).collect();
    package_keys.sort();
    package_keys.dedup();
    let unit_count = u32::try_from(plan.units.len()).ok()?;
    let effect_count = u32::try_from(plan.effects.len()).ok()?;
    let identity_input = ReceiptIdentityInput {
        schema: RECEIPT_PREIMAGE_SCHEMA,
        plan_schema: crate::model::PLAN_SCHEMA,
        plan_blake3: &plan.plan_blake3,
        package_keys: &package_keys,
        unit_count,
        effect_count,
    };
    let preimage = domain_digest(RECEIPT_DOMAIN, &identity_input).ok()?;
    debug_assert_eq!(unit_count, effect_count);
    debug_assert!(!package_keys.is_empty());
    Some(PlanReceiptPreimage {
        schema: String::from(RECEIPT_PREIMAGE_SCHEMA),
        plan_schema: String::from(crate::model::PLAN_SCHEMA),
        plan_blake3: plan.plan_blake3.clone(),
        package_keys,
        unit_count,
        effect_count,
        preimage_blake3: preimage,
    })
}
