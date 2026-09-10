//! Facade over the pure implementation core.
//!
//! Every structural and canonical-identity decision delegates to
//! `crunch-wasm-component-core::verify_materialization_bundle`. This module
//! only projects the decision into the bounded consumer report and never
//! re-implements validation.

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crunch_wasm_component_core::Blake3Identity;
use crunch_wasm_component_core::MaterializationBundle;
use crunch_wasm_component_core::REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM;
use crunch_wasm_component_core::REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM;
use crunch_wasm_component_core::StoreObject;

use crate::contract::ConsumerLayer;
use crate::contract::ConsumerVerificationReport;
use crate::contract::ConsumerVerificationStatus;

/// Consumer report schema identifier.
pub const CONSUMER_REPORT_SCHEMA: &str = "mantle-wasm-consumer-verification-report-v1";

/// Fixed verifier non-claims. Reports can never extend or weaken these.
pub const CONSUMER_VERIFIER_NON_CLAIMS: [&str; 6] = [
    REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM,
    REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM,
    "not-source-trust",
    "not-compiler-correctness",
    "not-component-behavior",
    "not-runtime-isolation",
];

/// Fixed member-count bound for one declared bundle member list.
const MAX_DECLARED_MEMBERS: usize = 8_192;

/// Estimated member count for initial reservation.
const DECLARED_MEMBERS_RESERVATION: usize = 16;

/// Run the structural verification layer over an in-memory bundle.
pub fn verify_consumer_bundle(bundle: MaterializationBundle) -> ConsumerVerificationReport {
    let bundle_schema = bundle.schema.clone();
    let bundle_identity = Some(bundle.bundle_identity_blake3.to_string());
    let runtime_profile = Some(bundle.expected_runtime_profile_blake3.to_string());
    let result = crunch_wasm_component_core::verify_materialization_bundle(bundle);
    let structural_blockers: Vec<String> = result.blockers.iter().map(|blocker| blocker.code.clone()).collect();
    let is_accepted = result.bundle.is_some();
    let status = if is_accepted {
        ConsumerVerificationStatus::Verified
    } else {
        ConsumerVerificationStatus::Rejected
    };
    let layers_completed = if is_accepted {
        vec![ConsumerLayer::Structural]
    } else {
        Vec::new()
    };
    debug_assert_eq!(structural_blockers.len(), result.blockers.len());
    debug_assert!(!is_accepted || layers_completed.len() == 1);
    ConsumerVerificationReport {
        schema: String::from(CONSUMER_REPORT_SCHEMA),
        bundle_schema,
        bundle_identity_blake3: bundle_identity,
        expected_runtime_profile_blake3: runtime_profile,
        members: Vec::new(),
        layers_completed,
        blockers: structural_blockers,
        status,
        non_claims: non_claims_owned(),
    }
}

/// Fixed non-claims as an owned vector for report payloads.
pub fn non_claims_owned() -> Vec<String> {
    CONSUMER_VERIFIER_NON_CLAIMS.iter().map(|claim| String::from(*claim)).collect()
}

/// Declared store objects with their stable report roles, in bundle order.
pub fn declared_member_objects(bundle: &MaterializationBundle) -> Vec<(&'static str, &StoreObject)> {
    fn push_member<'a>(
        members: &mut Vec<(&'static str, &'a StoreObject)>,
        role: &'static str,
        object: &'a StoreObject,
        count: &mut usize,
    ) {
        assert!(*count < MAX_DECLARED_MEMBERS, "declared member bound exceeded");
        *count += 1;
        members.push((role, object));
    }

    let mut members: Vec<(&'static str, &StoreObject)> = Vec::with_capacity(DECLARED_MEMBERS_RESERVATION);
    let mut member_count = 0_usize;
    push_member(&mut members, "source-closure", &bundle.source_closure, &mut member_count);
    push_member(&mut members, "lock", &bundle.lock, &mut member_count);
    push_member(&mut members, "final-portable", &bundle.final_portable, &mut member_count);
    for input in &bundle.wit_inputs {
        push_member(&mut members, "wit-input", input, &mut member_count);
    }
    for package in &bundle.package_inputs {
        push_member(&mut members, "package-object", &package.object, &mut member_count);
    }
    for receipt in &bundle.stage_receipts {
        push_member(&mut members, "stage-receipt", &receipt.receipt, &mut member_count);
        if let Some(artifact) = &receipt.artifact {
            push_member(&mut members, "stage-artifact", artifact, &mut member_count);
        }
    }
    debug_assert!(!members.is_empty());
    members
}

/// Recompute a [`Blake3Identity`] from raw bytes; shared by the shell.
pub fn identity_from_bytes(bytes: &[u8]) -> Blake3Identity {
    Blake3Identity::from_slice(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_claims_are_fixed_and_complete() {
        let claims = non_claims_owned();
        assert_eq!(claims.len(), CONSUMER_VERIFIER_NON_CLAIMS.len());
        assert!(claims.iter().all(|claim| !claim.contains("proves")));
        assert!(claims.contains(&String::from(REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM)));
        assert!(claims.contains(&String::from(REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM)));
    }
}
