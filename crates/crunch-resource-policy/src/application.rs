#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourcePolicyFailure {
    Core(crunch_resource_policy_core::ResourcePolicyError),
    Port {
        error: crate::ResourcePolicyPortError,
        quota_granted: bool,
    },
    Valence {
        issue_codes: Vec<String>,
    },
}

impl From<crunch_resource_policy_core::ResourcePolicyError> for ResourcePolicyFailure {
    fn from(error: crunch_resource_policy_core::ResourcePolicyError) -> Self {
        Self::Core(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DurableUsageOutcome {
    pub plan: crunch_resource_policy_core::UsageLedgerMutationPlan,
    pub durable: bool,
}

pub fn reserve_usage(
    port: &mut dyn crate::UsageLedgerPort,
    request: crunch_resource_policy_core::UsageReservationRequest,
) -> Result<DurableUsageOutcome, ResourcePolicyFailure> {
    let state = load_ledger(port)?;
    let plan = crunch_resource_policy_core::plan_usage_reservation(state, request)?;
    commit_plan(port, plan)
}

pub fn reconcile_usage(
    port: &mut dyn crate::UsageLedgerPort,
    request: crunch_resource_policy_core::UsageReconciliationRequest,
) -> Result<DurableUsageOutcome, ResourcePolicyFailure> {
    let state = load_ledger(port)?;
    let plan = crunch_resource_policy_core::plan_usage_reconciliation(state, request)?;
    commit_plan(port, plan)
}

pub fn publish_valence_evidence(
    port: &mut dyn crate::ResourcePolicyEvidencePort,
    bundle: valence_core::build_service::BuildServiceBundle,
) -> Result<valence_core::build_service::BuildServiceReport, ResourcePolicyFailure> {
    let valence_validation = valence_core::build_service::validate_build_service_bundle(&bundle);
    if !valence_validation.valid {
        let issue_codes =
            valence_validation.issues.iter().map(|issue| format!("{}:{}", issue.field, issue.code)).collect();
        return Err(ResourcePolicyFailure::Valence { issue_codes });
    }
    port.publish_valence_bundle(&bundle).map_err(|error| port_failure(error, false))?;
    debug_assert!(valence_validation.valid);
    debug_assert!(valence_validation.issues.is_empty());
    Ok(valence_validation)
}

fn load_ledger(
    port: &mut dyn crate::UsageLedgerPort,
) -> Result<crunch_resource_policy_core::UsageLedgerState, ResourcePolicyFailure> {
    let state = port.load_usage_ledger().map_err(|error| port_failure(error, false))?;
    crunch_resource_policy_core::usage_ledger_identity(&state)?;
    debug_assert_eq!(state.schema, crunch_resource_policy_core::USAGE_LEDGER_SCHEMA);
    debug_assert!(
        u32::try_from(state.reservations.len())
            .is_ok_and(|count| count <= crunch_resource_policy_core::MAX_USAGE_RECORDS)
    );
    Ok(state)
}

fn commit_plan(
    port: &mut dyn crate::UsageLedgerPort,
    plan: crunch_resource_policy_core::UsageLedgerMutationPlan,
) -> Result<DurableUsageOutcome, ResourcePolicyFailure> {
    if plan.disposition == crunch_resource_policy_core::LedgerMutationDisposition::Reuse {
        debug_assert_eq!(plan.prior_state_blake3, plan.next_state_blake3);
        debug_assert!(
            plan.reservation.is_some() || plan.reconciliation.is_some(),
            "a reused ledger plan must identify its existing record"
        );
        return Ok(DurableUsageOutcome { plan, durable: true });
    }
    port.compare_and_commit_usage_ledger(&plan.prior_state_blake3, &plan.next_state)
        .map_err(|error| port_failure(error, false))?;
    debug_assert_ne!(plan.prior_state_blake3, plan.next_state_blake3);
    debug_assert_eq!(plan.disposition, crunch_resource_policy_core::LedgerMutationDisposition::Add);
    Ok(DurableUsageOutcome { plan, durable: true })
}

fn port_failure(error: crate::ResourcePolicyPortError, quota_granted: bool) -> ResourcePolicyFailure {
    debug_assert!(!error.code.is_empty());
    debug_assert!(!quota_granted, "failed external mutations cannot grant quota");
    ResourcePolicyFailure::Port { error, quota_granted }
}
