use crate::ClosureFixture;
use crate::ReceiverManifest;
use crate::TransferPlan;
use crate::model::core_closure_fixture;
use crate::model::core_receiver_manifest;
use crate::model::transfer_plan_from_core;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    StorePrefixMismatch {
        sender_prefix: String,
        receiver_prefix: String,
    },
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StorePrefixMismatch {
                sender_prefix,
                receiver_prefix,
            } => write!(f, "store prefix mismatch: sender={sender_prefix} receiver={receiver_prefix}"),
        }
    }
}

impl std::error::Error for PlanError {}

impl From<crunch_delta_core::PlanError> for PlanError {
    fn from(value: crunch_delta_core::PlanError) -> Self {
        match value {
            crunch_delta_core::PlanError::StorePrefixMismatch {
                sender_prefix,
                receiver_prefix,
            } => Self::StorePrefixMismatch {
                sender_prefix,
                receiver_prefix,
            },
        }
    }
}

pub fn plan_transfer(sender: &ClosureFixture, receiver: &ReceiverManifest) -> Result<TransferPlan, PlanError> {
    let core_plan = crunch_delta_core::plan_transfer(core_closure_fixture(sender), core_receiver_manifest(receiver))
        .map_err(PlanError::from)?;
    Ok(transfer_plan_from_core(core_plan))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NegotiationOffer;
    use crate::bench_suite;
    use crate::chunk_profile_v1;
    use crate::chunk_profile_wire_v1;
    use crate::model::core_closure_fixture;
    use crate::model::core_receiver_manifest;
    use crate::model::new_receiver_manifest;
    use crate::negotiation::matches_runtime_profile;

    #[test]
    fn prefix_mismatch_is_rejected() {
        let suite = bench_suite();
        let case = &suite.cases[0];
        let mut receiver = case.receiver.clone();
        receiver.store_prefix = "/nix/store".to_owned();
        let err = plan_transfer(&case.sender, &receiver).expect_err("prefix mismatch must fail");
        assert_eq!(err, PlanError::StorePrefixMismatch {
            sender_prefix: "/crunch/store".to_owned(),
            receiver_prefix: "/nix/store".to_owned(),
        });
    }

    #[test]
    fn planner_matches_fixed_suite_targets() {
        let suite = bench_suite();
        let mut total = 0u64;
        for case in &suite.cases {
            let plan = plan_transfer(&case.sender, &case.receiver).expect("coarse plan");
            assert_eq!(plan.transferred_bytes, case.expected_coarse_bytes, "case {}", case.name);
            total = total.saturating_add(plan.transferred_bytes);
        }
        assert_eq!(total, suite.coarse_transfer_bytes_total());
    }

    #[test]
    fn whole_output_case_reuses_entire_output() {
        let suite = bench_suite();
        let case = suite.cases.iter().find(|case| case.name == "whole-output-hit").unwrap();
        let plan = plan_transfer(&case.sender, &case.receiver).unwrap();
        assert_eq!(plan.transferred_bytes, 0);
        assert_eq!(plan.tally.reused_outputs, 1);
    }

    #[test]
    fn chunk_case_sends_only_missing_chunk() {
        let suite = bench_suite();
        let case = suite.cases.iter().find(|case| case.name == "chunk-hit").unwrap();
        let plan = plan_transfer(&case.sender, &case.receiver).unwrap();
        assert_eq!(plan.transferred_bytes, 262_144);
        assert_eq!(plan.tally.reused_chunks, 7);
        assert_eq!(plan.tally.sent_chunks, 1);
    }

    #[test]
    fn cross_output_case_sends_shared_blob_once() {
        let suite = bench_suite();
        let case = suite.cases.iter().find(|case| case.name == "cross-output").unwrap();
        let plan = plan_transfer(&case.sender, &case.receiver).unwrap();
        assert_eq!(plan.transferred_bytes, 139_264);
        assert_eq!(plan.tally.sent_blobs, 3);
        assert_eq!(plan.tally.reused_blobs, 1);
    }

    #[test]
    fn borrowed_facade_accepts_std_receiver_manifest() {
        let suite = bench_suite();
        let case = &suite.cases[0];
        let empty_receiver = new_receiver_manifest(&case.sender.store_prefix);
        let plan = plan_transfer(&case.sender, &empty_receiver).expect("borrowed facade should plan");
        assert!(plan.full_transfer_bytes >= plan.transferred_bytes);
        assert_eq!(empty_receiver.store_prefix, case.sender.store_prefix);
    }

    #[test]
    fn delta_facade_reexports_core_planner_types() {
        let suite = bench_suite();
        let case = &suite.cases[0];
        let facade_plan = plan_transfer(&case.sender, &case.receiver).expect("facade should return transfer plan");
        let core_plan = crunch_delta_core::plan_transfer(
            core_closure_fixture(&case.sender),
            core_receiver_manifest(&case.receiver),
        )
        .expect("core planner should accept converted facade inputs");
        let offer = NegotiationOffer::protocol_v1();
        let profile = chunk_profile_wire_v1();

        assert_eq!(facade_plan.transferred_bytes, core_plan.transferred_bytes);
        assert_eq!(facade_plan.full_transfer_bytes, core_plan.full_transfer_bytes);
        assert_eq!(offer.supported_chunk_profiles, vec![profile]);
        assert!(matches_runtime_profile(profile, &chunk_profile_v1()));
    }

    #[test]
    fn plan_error_keeps_std_error_in_facade() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<PlanError>();
    }
}
