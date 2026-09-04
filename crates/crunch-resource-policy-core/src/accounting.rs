use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::LEDGER_DOMAIN;
use crate::LedgerMutationDisposition;
use crate::MAX_CHARGE_UNITS;
use crate::MAX_USAGE_RECORDS;
use crate::MAX_WINDOW_SECS;
use crate::RESOURCE_POLICY_NON_CLAIM;
use crate::ResourcePolicyError;
use crate::USAGE_LEDGER_SCHEMA;
use crate::USAGE_LEDGER_SUMMARY_SCHEMA;
use crate::USAGE_RECONCILIATION_SCHEMA;
use crate::USAGE_RESERVATION_SCHEMA;
use crate::UsageAmounts;
use crate::UsageByScope;
use crate::UsageLedgerMutationPlan;
use crate::UsageLedgerState;
use crate::UsageLedgerSummary;
use crate::UsageReconciliation;
use crate::UsageReconciliationKind;
use crate::UsageReconciliationRequest;
use crate::UsageReservation;
use crate::UsageReservationRequest;
use crate::UsageScopeSummary;
use crate::UsageUnitSchedule;
use crate::bounded_count;
use crate::canonical_blake3;
use crate::validate_digest;
use crate::validate_id;

#[derive(Clone, Copy, Debug)]
struct CeilDivision {
    dividend: u64,
    divisor: u64,
}

pub fn plan_usage_reservation(
    mut state: UsageLedgerState,
    request: UsageReservationRequest,
) -> Result<UsageLedgerMutationPlan, ResourcePolicyError> {
    validate_usage_state(&state)?;
    validate_reservation_request(&request)?;
    let prior_state_blake3 = usage_ledger_identity(&state)?;
    let reservation = build_reservation(&request)?;
    if let Some(existing) = state.reservations.iter().find(|existing| existing.attempt_id == request.attempt_id) {
        if existing != &reservation {
            return Err(ResourcePolicyError::ReservationConflict);
        }
        return reuse_reservation_plan(state, prior_state_blake3, reservation);
    }
    require_quota_capacity(&state, &request)?;
    state.reservations.push(reservation.clone());
    state.reservations.sort_by(|left, right| left.attempt_id.cmp(&right.attempt_id));
    validate_usage_state(&state)?;
    let next_state_blake3 = usage_ledger_identity(&state)?;
    debug_assert_ne!(prior_state_blake3, next_state_blake3);
    debug_assert!(state.reservations.iter().any(|row| row == &reservation));
    Ok(UsageLedgerMutationPlan {
        disposition: LedgerMutationDisposition::Add,
        prior_state_blake3,
        next_state_blake3,
        next_state: state,
        reservation: Some(reservation),
        reconciliation: None,
        quota_granted: true,
        reason_code: "usage-reservation-added".to_string(),
    })
}

pub fn plan_usage_reconciliation(
    mut state: UsageLedgerState,
    request: UsageReconciliationRequest,
) -> Result<UsageLedgerMutationPlan, ResourcePolicyError> {
    validate_usage_state(&state)?;
    validate_reconciliation_request(&request)?;
    let prior_state_blake3 = usage_ledger_identity(&state)?;
    let reservation = state
        .reservations
        .iter()
        .find(|reservation| reservation.attempt_id == request.attempt_id)
        .cloned()
        .ok_or(ResourcePolicyError::ReservationMissing)?;
    let reconciliation = build_reconciliation(&reservation, &request)?;
    if let Some(existing) = state.reconciliations.iter().find(|existing| existing.attempt_id == request.attempt_id) {
        if existing != &reconciliation {
            return Err(ResourcePolicyError::ReconciliationConflict);
        }
        return reuse_reconciliation_plan(state, prior_state_blake3, reconciliation);
    }
    state.reconciliations.push(reconciliation.clone());
    state.reconciliations.sort_by(|left, right| left.attempt_id.cmp(&right.attempt_id));
    validate_usage_state(&state)?;
    let next_state_blake3 = usage_ledger_identity(&state)?;
    debug_assert_ne!(prior_state_blake3, next_state_blake3);
    debug_assert!(state.reconciliations.iter().any(|row| row == &reconciliation));
    Ok(UsageLedgerMutationPlan {
        disposition: LedgerMutationDisposition::Add,
        prior_state_blake3,
        next_state_blake3,
        next_state: state,
        reservation: None,
        reconciliation: Some(reconciliation),
        quota_granted: false,
        reason_code: "usage-reconciliation-added".to_string(),
    })
}

pub fn usage_ledger_identity(state: &UsageLedgerState) -> Result<String, ResourcePolicyError> {
    validate_usage_state(state)?;
    let identity = canonical_blake3(LEDGER_DOMAIN, state)?;
    debug_assert!(crate::valid_blake3(&identity));
    debug_assert_eq!(state.schema, USAGE_LEDGER_SCHEMA);
    Ok(identity)
}

pub fn usage_totals_by_project(state: UsageLedgerState) -> Result<UsageByScope, ResourcePolicyError> {
    validate_usage_state(&state)?;
    let totals = usage_totals(&state, true)?;
    debug_assert!(totals.len() <= state.reservations.len());
    debug_assert!(totals.values().all(|value| *value <= MAX_CHARGE_UNITS));
    Ok(totals)
}

pub fn usage_totals_by_account(state: UsageLedgerState) -> Result<UsageByScope, ResourcePolicyError> {
    validate_usage_state(&state)?;
    let totals = usage_totals(&state, false)?;
    debug_assert!(totals.len() <= state.reservations.len());
    debug_assert!(totals.values().all(|value| *value <= MAX_CHARGE_UNITS));
    Ok(totals)
}

pub fn summarize_usage_ledger(state: UsageLedgerState) -> Result<UsageLedgerSummary, ResourcePolicyError> {
    validate_usage_state(&state)?;
    let state_blake3 = usage_ledger_identity(&state)?;
    let reservation_count =
        u32::try_from(state.reservations.len()).map_err(|_| ResourcePolicyError::TooManyUsageRecords)?;
    let reconciliation_count =
        u32::try_from(state.reconciliations.len()).map_err(|_| ResourcePolicyError::TooManyUsageRecords)?;
    let project_units = scope_summaries(&state, true)?;
    let account_units = scope_summaries(&state, false)?;
    let mut reason_codes = state.reconciliations.iter().map(|row| row.reason_code.clone()).collect::<Vec<_>>();
    reason_codes.sort();
    reason_codes.dedup();
    let summary = UsageLedgerSummary {
        schema: USAGE_LEDGER_SUMMARY_SCHEMA.to_string(),
        state_blake3,
        reservation_count,
        reconciliation_count,
        project_units,
        account_units,
        reason_codes,
        non_claim: RESOURCE_POLICY_NON_CLAIM.to_string(),
    };
    debug_assert_eq!(usize::try_from(summary.reservation_count).ok(), Some(state.reservations.len()));
    debug_assert!(crate::valid_blake3(&summary.state_blake3));
    Ok(summary)
}

fn validate_usage_state(state: &UsageLedgerState) -> Result<(), ResourcePolicyError> {
    if state.schema != USAGE_LEDGER_SCHEMA {
        return Err(ResourcePolicyError::InvalidSchema);
    }
    bounded_count(state.reservations.len(), MAX_USAGE_RECORDS, ResourcePolicyError::TooManyUsageRecords)?;
    bounded_count(state.reconciliations.len(), MAX_USAGE_RECORDS, ResourcePolicyError::TooManyUsageRecords)?;
    let mut reservation_attempts = BTreeSet::new();
    for reservation in &state.reservations {
        validate_reservation(reservation)?;
        if !reservation_attempts.insert(reservation.attempt_id.as_str()) {
            return Err(ResourcePolicyError::DuplicateIdentity);
        }
    }
    let mut reconciliation_attempts = BTreeSet::new();
    for reconciliation in &state.reconciliations {
        validate_reconciliation(reconciliation)?;
        if !reconciliation_attempts.insert(reconciliation.attempt_id.as_str())
            || !reservation_attempts.contains(reconciliation.attempt_id.as_str())
        {
            return Err(ResourcePolicyError::DuplicateIdentity);
        }
        let reservation = state
            .reservations
            .iter()
            .find(|row| row.attempt_id == reconciliation.attempt_id)
            .ok_or(ResourcePolicyError::ReservationMissing)?;
        let accounted = reconciliation
            .charged_units
            .checked_add(reconciliation.released_units)
            .ok_or(ResourcePolicyError::ArithmeticOverflow)?;
        if reconciliation.reservation_blake3 != reservation.reservation_blake3
            || accounted != reservation.reserved_units
        {
            return Err(ResourcePolicyError::ReconciliationConflict);
        }
    }
    debug_assert!(reservation_attempts.len() == state.reservations.len());
    debug_assert!(reconciliation_attempts.len() == state.reconciliations.len());
    Ok(())
}

fn validate_reservation_request(request: &UsageReservationRequest) -> Result<(), ResourcePolicyError> {
    validate_id(&request.attempt_id)?;
    validate_id(&request.project_id)?;
    validate_id(&request.account_id)?;
    validate_id(&request.policy_id)?;
    validate_schedule(&request.schedule)?;
    let window_secs = request
        .window_end_unix_s
        .checked_sub(request.window_start_unix_s)
        .ok_or(ResourcePolicyError::InvalidBounds)?;
    if window_secs == 0 || window_secs > MAX_WINDOW_SECS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if request.reserved_units == 0 || request.reserved_units > MAX_CHARGE_UNITS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if request.project_limit_units == 0 || request.project_limit_units > MAX_CHARGE_UNITS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if request.account_limit_units == 0 || request.account_limit_units > MAX_CHARGE_UNITS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(request.window_end_unix_s > request.window_start_unix_s);
    debug_assert!(request.reserved_units <= MAX_CHARGE_UNITS);
    Ok(())
}

fn validate_reconciliation_request(request: &UsageReconciliationRequest) -> Result<(), ResourcePolicyError> {
    validate_id(&request.attempt_id)?;
    validate_id(&request.policy_id)?;
    validate_schedule(&request.schedule)?;
    if request.kind == UsageReconciliationKind::Completed && request.observed.is_none() {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if let Some(observed) = request.observed {
        validate_usage_amounts(observed)?;
    }
    debug_assert!(!request.attempt_id.is_empty());
    debug_assert!(request.kind != UsageReconciliationKind::Completed || request.observed.is_some());
    Ok(())
}

fn validate_schedule(schedule: &UsageUnitSchedule) -> Result<(), ResourcePolicyError> {
    validate_id(&schedule.schedule_id)?;
    if schedule.cpu_ms_per_unit == 0 {
        return Err(ResourcePolicyError::InvalidUsageSchedule);
    }
    if schedule.memory_byte_ms_per_unit == 0 {
        return Err(ResourcePolicyError::InvalidUsageSchedule);
    }
    if schedule.transfer_bytes_per_unit == 0 {
        return Err(ResourcePolicyError::InvalidUsageSchedule);
    }
    if schedule.charge_units_max == 0 || schedule.charge_units_max > MAX_CHARGE_UNITS {
        return Err(ResourcePolicyError::InvalidUsageSchedule);
    }
    debug_assert!(schedule.cpu_ms_per_unit > 0);
    debug_assert!(schedule.charge_units_max <= MAX_CHARGE_UNITS);
    Ok(())
}

fn validate_usage_amounts(amounts: UsageAmounts) -> Result<(), ResourcePolicyError> {
    if amounts.cpu_ms > crate::MAX_MEASUREMENT_MILLISECONDS
        || amounts.memory_byte_ms > crate::MAX_MEASUREMENT_BYTES
        || amounts.transfer_bytes > crate::MAX_MEASUREMENT_BYTES
    {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(amounts.cpu_ms <= crate::MAX_MEASUREMENT_MILLISECONDS);
    debug_assert!(amounts.transfer_bytes <= crate::MAX_MEASUREMENT_BYTES);
    Ok(())
}

fn build_reservation(request: &UsageReservationRequest) -> Result<UsageReservation, ResourcePolicyError> {
    let mut reservation = UsageReservation {
        schema: USAGE_RESERVATION_SCHEMA.to_string(),
        reservation_blake3: String::new(),
        attempt_id: request.attempt_id.clone(),
        project_id: request.project_id.clone(),
        account_id: request.account_id.clone(),
        window_start_unix_s: request.window_start_unix_s,
        window_end_unix_s: request.window_end_unix_s,
        reserved_units: request.reserved_units,
        policy_id: request.policy_id.clone(),
        schedule_id: request.schedule.schedule_id.clone(),
    };
    reservation.reservation_blake3 = reservation_identity(&reservation)?;
    debug_assert!(crate::valid_blake3(&reservation.reservation_blake3));
    debug_assert_eq!(reservation.schema, USAGE_RESERVATION_SCHEMA);
    Ok(reservation)
}

fn validate_reservation(reservation: &UsageReservation) -> Result<(), ResourcePolicyError> {
    if reservation.schema != USAGE_RESERVATION_SCHEMA {
        return Err(ResourcePolicyError::InvalidSchema);
    }
    validate_digest(&reservation.reservation_blake3)?;
    validate_id(&reservation.attempt_id)?;
    validate_id(&reservation.project_id)?;
    validate_id(&reservation.account_id)?;
    validate_id(&reservation.policy_id)?;
    validate_id(&reservation.schedule_id)?;
    if reservation.reserved_units == 0 || reservation.reserved_units > MAX_CHARGE_UNITS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if reservation.window_end_unix_s <= reservation.window_start_unix_s {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if reservation_identity(reservation)? != reservation.reservation_blake3 {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(reservation.window_end_unix_s > reservation.window_start_unix_s);
    debug_assert!(reservation.reserved_units > 0);
    Ok(())
}

fn build_reconciliation(
    reservation: &UsageReservation,
    request: &UsageReconciliationRequest,
) -> Result<UsageReconciliation, ResourcePolicyError> {
    if reservation.policy_id != request.policy_id || reservation.schedule_id != request.schedule.schedule_id {
        return Err(ResourcePolicyError::ReconciliationConflict);
    }
    let (charged_units, reason_code) = reconciliation_charge(reservation, request)?;
    let released_units = reservation
        .reserved_units
        .checked_sub(charged_units)
        .ok_or(ResourcePolicyError::ArithmeticOverflow)?;
    let mut reconciliation = UsageReconciliation {
        schema: USAGE_RECONCILIATION_SCHEMA.to_string(),
        reconciliation_blake3: String::new(),
        reservation_blake3: reservation.reservation_blake3.clone(),
        attempt_id: reservation.attempt_id.clone(),
        kind: request.kind,
        charged_units,
        released_units,
        policy_id: request.policy_id.clone(),
        schedule_id: request.schedule.schedule_id.clone(),
        reason_code: reason_code.to_string(),
    };
    reconciliation.reconciliation_blake3 = reconciliation_identity(&reconciliation)?;
    debug_assert_eq!(charged_units.checked_add(released_units), Some(reservation.reserved_units));
    debug_assert!(crate::valid_blake3(&reconciliation.reconciliation_blake3));
    Ok(reconciliation)
}

fn reconciliation_charge(
    reservation: &UsageReservation,
    request: &UsageReconciliationRequest,
) -> Result<(u64, &'static str), ResourcePolicyError> {
    match request.kind {
        UsageReconciliationKind::Completed => {
            let observed = request.observed.ok_or(ResourcePolicyError::InvalidBounds)?;
            let measured = charge_units(observed, &request.schedule)?;
            Ok((measured.min(reservation.reserved_units), "usage-completed-measured"))
        }
        UsageReconciliationKind::Cancelled => match request.observed {
            Some(observed) => Ok((
                charge_units(observed, &request.schedule)?.min(reservation.reserved_units),
                "usage-cancelled-measured",
            )),
            None => Ok((0, "usage-cancelled-released")),
        },
        UsageReconciliationKind::WorkerLost => {
            Ok((reservation.reserved_units, "usage-worker-lost-reservation-retained"))
        }
        UsageReconciliationKind::PartialObservation => match request.observed {
            Some(observed) => Ok((
                charge_units(observed, &request.schedule)?.min(reservation.reserved_units),
                "usage-partial-measured",
            )),
            None => Ok((reservation.reserved_units, "usage-partial-reservation-retained")),
        },
    }
}

fn charge_units(amounts: UsageAmounts, schedule: &UsageUnitSchedule) -> Result<u64, ResourcePolicyError> {
    validate_usage_amounts(amounts)?;
    validate_schedule(schedule)?;
    let cpu = ceil_div(CeilDivision {
        dividend: amounts.cpu_ms,
        divisor: schedule.cpu_ms_per_unit,
    })?;
    let memory = ceil_div(CeilDivision {
        dividend: amounts.memory_byte_ms,
        divisor: schedule.memory_byte_ms_per_unit,
    })?;
    let transfer = ceil_div(CeilDivision {
        dividend: amounts.transfer_bytes,
        divisor: schedule.transfer_bytes_per_unit,
    })?;
    let total = cpu
        .checked_add(memory)
        .and_then(|subtotal| subtotal.checked_add(transfer))
        .ok_or(ResourcePolicyError::ArithmeticOverflow)?;
    if total > schedule.charge_units_max {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(total >= cpu);
    debug_assert!(total <= schedule.charge_units_max);
    Ok(total)
}

fn ceil_div(input: CeilDivision) -> Result<u64, ResourcePolicyError> {
    if input.divisor == 0 {
        return Err(ResourcePolicyError::InvalidUsageSchedule);
    }
    if input.dividend == 0 {
        return Ok(0);
    }
    let divisor_units = input.divisor;
    assert!(divisor_units != 0, "validated usage divisor must remain nonzero");
    let adjusted = input
        .dividend
        .checked_add(divisor_units.saturating_sub(1))
        .ok_or(ResourcePolicyError::ArithmeticOverflow)?;
    let quotient = adjusted / divisor_units;
    debug_assert!(quotient > 0);
    debug_assert!(quotient <= input.dividend);
    Ok(quotient)
}

fn validate_reconciliation(reconciliation: &UsageReconciliation) -> Result<(), ResourcePolicyError> {
    if reconciliation.schema != USAGE_RECONCILIATION_SCHEMA {
        return Err(ResourcePolicyError::InvalidSchema);
    }
    validate_digest(&reconciliation.reconciliation_blake3)?;
    validate_digest(&reconciliation.reservation_blake3)?;
    validate_id(&reconciliation.attempt_id)?;
    validate_id(&reconciliation.policy_id)?;
    validate_id(&reconciliation.schedule_id)?;
    validate_id(&reconciliation.reason_code)?;
    let total = reconciliation
        .charged_units
        .checked_add(reconciliation.released_units)
        .ok_or(ResourcePolicyError::ArithmeticOverflow)?;
    if total == 0
        || total > MAX_CHARGE_UNITS
        || reconciliation_identity(reconciliation)? != reconciliation.reconciliation_blake3
    {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(total > 0);
    debug_assert!(total <= MAX_CHARGE_UNITS);
    Ok(())
}

fn require_quota_capacity(
    state: &UsageLedgerState,
    request: &UsageReservationRequest,
) -> Result<(), ResourcePolicyError> {
    let project_used = scope_usage(state, &request.project_id, true)?;
    let account_used = scope_usage(state, &request.account_id, false)?;
    let project_after =
        project_used.checked_add(request.reserved_units).ok_or(ResourcePolicyError::ArithmeticOverflow)?;
    let account_after =
        account_used.checked_add(request.reserved_units).ok_or(ResourcePolicyError::ArithmeticOverflow)?;
    if project_after > request.project_limit_units || account_after > request.account_limit_units {
        return Err(ResourcePolicyError::QuotaDenied);
    }
    debug_assert!(project_after <= request.project_limit_units);
    debug_assert!(account_after <= request.account_limit_units);
    Ok(())
}

fn scope_summaries(state: &UsageLedgerState, project: bool) -> Result<Vec<UsageScopeSummary>, ResourcePolicyError> {
    let mut values = BTreeMap::<String, (u64, u64)>::new();
    for reservation in &state.reservations {
        let scope = if project {
            reservation.project_id.clone()
        } else {
            reservation.account_id.clone()
        };
        if !values.contains_key(&scope) && values.len() >= state.reservations.len() {
            return Err(ResourcePolicyError::TooManyUsageRecords);
        }
        let entry = values.entry(scope).or_insert((0, 0));
        match state.reconciliations.iter().find(|row| row.attempt_id == reservation.attempt_id) {
            Some(reconciliation) => {
                entry.1 =
                    entry.1.checked_add(reconciliation.charged_units).ok_or(ResourcePolicyError::ArithmeticOverflow)?;
            }
            None => {
                entry.0 =
                    entry.0.checked_add(reservation.reserved_units).ok_or(ResourcePolicyError::ArithmeticOverflow)?;
            }
        }
    }
    let summaries = values
        .into_iter()
        .map(|(scope_id, (active_reserved_units, charged_units))| UsageScopeSummary {
            scope_id,
            active_reserved_units,
            charged_units,
        })
        .collect::<Vec<_>>();
    debug_assert!(summaries.len() <= state.reservations.len());
    debug_assert!(summaries.iter().all(|row| row.active_reserved_units <= MAX_CHARGE_UNITS));
    Ok(summaries)
}

fn usage_totals(state: &UsageLedgerState, project: bool) -> Result<UsageByScope, ResourcePolicyError> {
    let mut totals = UsageByScope::new();
    for reservation in &state.reservations {
        let scope = if project {
            reservation.project_id.clone()
        } else {
            reservation.account_id.clone()
        };
        let amount = accounted_units(state, reservation);
        let current = totals.get(&scope).copied().unwrap_or(0);
        let next = current.checked_add(amount).ok_or(ResourcePolicyError::ArithmeticOverflow)?;
        if next > MAX_CHARGE_UNITS {
            return Err(ResourcePolicyError::InvalidBounds);
        }
        if !totals.contains_key(&scope) && totals.len() >= state.reservations.len() {
            return Err(ResourcePolicyError::TooManyUsageRecords);
        }
        totals.insert(scope, next);
    }
    debug_assert!(totals.len() <= state.reservations.len());
    debug_assert!(totals.values().all(|value| *value <= MAX_CHARGE_UNITS));
    Ok(totals)
}

fn scope_usage(state: &UsageLedgerState, scope: &str, project: bool) -> Result<u64, ResourcePolicyError> {
    let totals = usage_totals(state, project)?;
    let used = totals.get(scope).copied().unwrap_or(0);
    debug_assert!(used <= MAX_CHARGE_UNITS);
    debug_assert!(scope.is_empty() || totals.len() <= state.reservations.len());
    Ok(used)
}

fn accounted_units(state: &UsageLedgerState, reservation: &UsageReservation) -> u64 {
    state
        .reconciliations
        .iter()
        .find(|row| row.attempt_id == reservation.attempt_id)
        .map_or(reservation.reserved_units, |row| row.charged_units)
}

fn reuse_reservation_plan(
    state: UsageLedgerState,
    identity: String,
    reservation: UsageReservation,
) -> Result<UsageLedgerMutationPlan, ResourcePolicyError> {
    debug_assert!(state.reservations.iter().any(|row| row == &reservation));
    debug_assert!(crate::valid_blake3(&identity));
    Ok(UsageLedgerMutationPlan {
        disposition: LedgerMutationDisposition::Reuse,
        prior_state_blake3: identity.clone(),
        next_state_blake3: identity,
        next_state: state,
        reservation: Some(reservation),
        reconciliation: None,
        quota_granted: true,
        reason_code: "usage-reservation-reused".to_string(),
    })
}

fn reuse_reconciliation_plan(
    state: UsageLedgerState,
    identity: String,
    reconciliation: UsageReconciliation,
) -> Result<UsageLedgerMutationPlan, ResourcePolicyError> {
    debug_assert!(state.reconciliations.iter().any(|row| row == &reconciliation));
    debug_assert!(crate::valid_blake3(&identity));
    Ok(UsageLedgerMutationPlan {
        disposition: LedgerMutationDisposition::Reuse,
        prior_state_blake3: identity.clone(),
        next_state_blake3: identity,
        next_state: state,
        reservation: None,
        reconciliation: Some(reconciliation),
        quota_granted: false,
        reason_code: "usage-reconciliation-reused".to_string(),
    })
}

fn reservation_identity(reservation: &UsageReservation) -> Result<String, ResourcePolicyError> {
    let mut material = reservation.clone();
    material.reservation_blake3.clear();
    canonical_blake3(b"mantle.resource-policy.usage-reservation.v1", &material)
}

fn reconciliation_identity(reconciliation: &UsageReconciliation) -> Result<String, ResourcePolicyError> {
    let mut material = reconciliation.clone();
    material.reconciliation_blake3.clear();
    canonical_blake3(b"mantle.resource-policy.usage-reconciliation.v1", &material)
}
