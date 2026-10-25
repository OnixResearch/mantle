//! Atomic project/account accounting decisions. A pending reconciliation keeps
//! the entire reservation: cancellation, worker loss and missing data never
//! mint capacity. The shell persists this combined ledger in one transaction.
use alloc::collections::BTreeMap;
use alloc::string::String;

use crate::attempt::RemoteAttemptId;
use crate::attempt::RemoteFenceGeneration;
use crate::attempt::RemoteJobId;
use crate::resource_policy::valid_id;
pub const MAX_RESERVATIONS: usize = 4096;
pub const LEDGER_SCHEMA: &str = "mantle-resource-quota-v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QuotaError {
    Invalid,
    Full,
    Overflow,
    Denied,
    Conflict,
    UnknownAttempt,
    StoreUnavailable,
    StoreUncertain,
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct UnitSchedule {
    pub version: u32,
    pub unit: String,
    pub window_id: String,
    pub window_start_s: u64,
    pub window_end_s: u64,
    pub cpu_ms_per_unit: u64,
    pub memory_byte_ms_per_unit: u64,
    pub executor_output_bytes_per_unit: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct Reservation {
    pub job: RemoteJobId,
    pub attempt: RemoteAttemptId,
    pub fence: RemoteFenceGeneration,
    pub project: String,
    pub account: String,
    pub request_digest: String,
    pub units: u64,
    pub schedule: UnitSchedule,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PendingReason {
    CancelledWithoutFinalUsage,
    WorkerLost,
    PartialObservation,
    UntrustedObservation,
    UsageExceedsReservation,
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Reconciliation {
    Reserved,
    Pending(PendingReason),
    Settled { event_digest: String, charged_units: u64 },
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct Entry {
    pub reservation: Reservation,
    pub reconciliation: Reconciliation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct ObservationRecord {
    pub observation: crate::resource_policy::Observation,
    /// Digest of a verified usage report, or explicitly unknown coordinator
    /// terminal event. This reference is not itself signer authority.
    pub usage_event_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct ResourceContext {
    pub declared: crate::resource_policy::Quantities,
    pub selected: crate::resource_policy::Quantities,
    pub features: crate::resource_policy::Features,
    pub machine_class: String,
    pub action_family: String,
    pub policy_version: u32,
    pub predecessor: Option<RemoteAttemptId>,
}
impl ResourceContext {
    fn valid(&self) -> bool {
        self.declared.valid()
            && self.selected.fits(self.declared)
            && self.features.valid()
            && valid_id(&self.machine_class)
            && valid_id(&self.action_family)
            && self.policy_version != 0
    }
    fn matches(&self, observation: &crate::resource_policy::Observation) -> bool {
        self.declared == observation.declared
            && self.selected == observation.selected
            && self.features == observation.features
            && self.machine_class == observation.machine_class
            && self.action_family == observation.action_family
            && self.policy_version == observation.policy_version
            && self.predecessor == observation.predecessor
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct Ledger {
    pub schema: String,
    pub revision: u64,
    pub entries: BTreeMap<String, Entry>,
    pub observations: BTreeMap<String, ObservationRecord>,
    pub resource_contexts: BTreeMap<String, ResourceContext>,
}
impl Default for Ledger {
    fn default() -> Self {
        Self {
            schema: String::from(LEDGER_SCHEMA),
            revision: 0,
            entries: BTreeMap::new(),
            observations: BTreeMap::new(),
            resource_contexts: BTreeMap::new(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub project_units: u64,
    pub account_units: u64,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Summary {
    pub project_reserved: u64,
    pub project_charged: u64,
    pub account_reserved: u64,
    pub account_charged: u64,
}
fn reservation_key(r: &Reservation) -> &str {
    r.attempt.as_str()
}
/// One recorded observation bound to its reservation context and usage digest.
fn observation_is_admissible(
    observation: &crate::resource_policy::Observation,
    usage_event_digest: &str,
    context: Option<&ResourceContext>,
) -> bool {
    let has_valid_features = observation.features.valid();
    let has_valid_family = valid_id(&observation.action_family);
    let has_valid_class = valid_id(&observation.machine_class);
    let has_valid_collector = valid_id(&observation.collector_version);
    let has_valid_declared = observation.declared.valid();
    let has_declared_fit = observation.selected.fits(observation.declared);
    let has_valid_digest = crate::output::is_blake3_hex_digest(usage_event_digest);
    let has_matching_context = context.is_some_and(|recorded| recorded.matches(observation));
    has_valid_features
        && has_valid_family
        && has_valid_class
        && has_valid_collector
        && has_valid_declared
        && has_declared_fit
        && has_valid_digest
        && has_matching_context
}
fn schedule_is_valid(schedule: &UnitSchedule) -> bool {
    let has_version = schedule.version != 0;
    let has_valid_unit = valid_id(&schedule.unit);
    let has_valid_window = valid_id(&schedule.window_id);
    let has_ordered_window = schedule.window_start_s < schedule.window_end_s;
    let has_units = schedule.cpu_ms_per_unit != 0
        && schedule.memory_byte_ms_per_unit != 0
        && schedule.executor_output_bytes_per_unit != 0;
    has_version && has_valid_unit && has_valid_window && has_ordered_window && has_units
}
fn validate_reservation(r: &Reservation) -> Result<(), QuotaError> {
    let has_valid_scope = valid_id(&r.project) && valid_id(&r.account);
    let has_valid_attempt = valid_id(r.job.as_str()) && valid_id(r.attempt.as_str()) && r.fence.get() != 0;
    let has_valid_digest = crate::output::is_blake3_hex_digest(&r.request_digest);
    let has_valid_schedule = schedule_is_valid(&r.schedule);
    let has_units = r.units != 0;
    let is_valid = has_units && has_valid_scope && has_valid_attempt && has_valid_digest && has_valid_schedule;
    if !is_valid {
        return Err(QuotaError::Invalid);
    }
    debug_assert!(r.units > 0);
    debug_assert!(r.schedule.window_start_s < r.schedule.window_end_s);
    Ok(())
}
impl Ledger {
    pub fn record_resource_context(
        &mut self,
        reservation: &Reservation,
        context: ResourceContext,
    ) -> Result<(), QuotaError> {
        self.validate()?;
        let key = reservation.attempt.as_str();
        if !context.valid() || self.entries.get(key).is_none_or(|entry| entry.reservation != *reservation) {
            return Err(QuotaError::Conflict);
        }
        if let Some(existing) = self.resource_contexts.get(key) {
            return if existing == &context {
                Ok(())
            } else {
                Err(QuotaError::Conflict)
            };
        }
        let revision = self.revision.checked_add(1).ok_or(QuotaError::Overflow)?;
        self.resource_contexts.insert(String::from(key), context);
        self.revision = revision;
        Ok(())
    }
    /// Idempotent terminal observation bound to an already admitted reservation.
    /// The shell authenticates collector facts before invoking this pure mutation.
    pub fn record_observation(
        &mut self,
        reservation: &Reservation,
        record: ObservationRecord,
    ) -> Result<(), QuotaError> {
        self.validate()?;
        let key = reservation.attempt.as_str();
        let admitted = self.entries.get(key).ok_or(QuotaError::UnknownAttempt)?;
        let has_matching_reservation = admitted.reservation == *reservation;
        let has_matching_attempt = record.observation.attempt_id == reservation.attempt;
        let has_matching_schema = record.observation.schema == crate::resource_policy::OBSERVATION_SCHEMA;
        let context = self.resource_contexts.get(key);
        let has_admissible_observation =
            observation_is_admissible(&record.observation, &record.usage_event_digest, context);
        let is_admissible =
            has_matching_reservation && has_matching_attempt && has_matching_schema && has_admissible_observation;
        if !is_admissible {
            return Err(QuotaError::Conflict);
        }
        if let Some(existing) = self.observations.get(key) {
            if existing.usage_event_digest == record.usage_event_digest {
                return if existing == &record {
                    Ok(())
                } else {
                    Err(QuotaError::Conflict)
                };
            }
            if existing.observation.trusted_collector || !record.observation.trusted_collector {
                return Err(QuotaError::Conflict);
            }
        }
        if !self.observations.contains_key(key) && self.observations.len() == crate::resource_policy::MAX_HISTORY {
            return Err(QuotaError::Full);
        }
        let revision = self.revision.checked_add(1).ok_or(QuotaError::Overflow)?;
        self.observations.insert(String::from(key), record);
        self.revision = revision;
        Ok(())
    }
    pub fn validate(&self) -> Result<(), QuotaError> {
        if self.schema != LEDGER_SCHEMA
            || self.entries.len() > MAX_RESERVATIONS
            || self.observations.len() > crate::resource_policy::MAX_HISTORY
        {
            return Err(QuotaError::Invalid);
        }
        for (key, entry) in &self.entries {
            validate_reservation(&entry.reservation)?;
            if key.as_str() != reservation_key(&entry.reservation) {
                return Err(QuotaError::Invalid);
            }
            if matches!(
                &entry.reconciliation,
                Reconciliation::Settled { event_digest, charged_units }
                    if !crate::output::is_blake3_hex_digest(event_digest)
                        || *charged_units > entry.reservation.units
            ) {
                return Err(QuotaError::Invalid);
            }
        }
        for (key, context) in &self.resource_contexts {
            if !self.entries.contains_key(key) || !context.valid() {
                return Err(QuotaError::Invalid);
            }
        }
        for (key, record) in &self.observations {
            let observation = &record.observation;
            let has_entry = self.entries.contains_key(key);
            let has_matching_attempt = observation.attempt_id.as_str() == key;
            let has_matching_schema = observation.schema == crate::resource_policy::OBSERVATION_SCHEMA;
            let context = self.resource_contexts.get(key);
            let has_admissible_observation =
                observation_is_admissible(observation, &record.usage_event_digest, context);
            let is_admissible = has_entry && has_matching_attempt && has_matching_schema && has_admissible_observation;
            if !is_admissible {
                return Err(QuotaError::Invalid);
            }
        }
        Ok(())
    }
    pub fn summary(&self, r: &Reservation) -> Result<Summary, QuotaError> {
        self.validate()?;
        let mut out = Summary::default();
        for entry in self.entries.values() {
            let old = &entry.reservation;
            if old.schedule.window_id != r.schedule.window_id {
                continue;
            }
            if old.schedule != r.schedule {
                return Err(QuotaError::Conflict);
            }
            let (reserved, charged) = match entry.reconciliation {
                Reconciliation::Settled { charged_units, .. } => (0, charged_units),
                _ => (old.units, 0),
            };
            if old.project == r.project {
                out.project_reserved = out.project_reserved.checked_add(reserved).ok_or(QuotaError::Overflow)?;
                out.project_charged = out.project_charged.checked_add(charged).ok_or(QuotaError::Overflow)?;
            }
            if old.account == r.account {
                out.account_reserved = out.account_reserved.checked_add(reserved).ok_or(QuotaError::Overflow)?;
                out.account_charged = out.account_charged.checked_add(charged).ok_or(QuotaError::Overflow)?;
            }
        }
        Ok(out)
    }
    // r[impl remote_builds.usage_reservation_and_reconciliation]
    pub fn reserve(&mut self, reservation: Reservation, limits: Limits) -> Result<Entry, QuotaError> {
        validate_reservation(&reservation)?;
        self.validate()?;
        let key = reservation_key(&reservation);
        if let Some(existing) = self.entries.get(key) {
            return if existing.reservation == reservation {
                Ok(existing.clone())
            } else {
                Err(QuotaError::Conflict)
            };
        }
        if self.entries.len() == MAX_RESERVATIONS {
            return Err(QuotaError::Full);
        }
        let s = self.summary(&reservation)?;
        let project = s
            .project_reserved
            .checked_add(s.project_charged)
            .and_then(|n| n.checked_add(reservation.units))
            .ok_or(QuotaError::Overflow)?;
        let account = s
            .account_reserved
            .checked_add(s.account_charged)
            .and_then(|n| n.checked_add(reservation.units))
            .ok_or(QuotaError::Overflow)?;
        if project > limits.project_units || account > limits.account_units {
            return Err(QuotaError::Denied);
        }
        let next_revision = self.revision.checked_add(1).ok_or(QuotaError::Overflow)?;
        let key = String::from(key);
        let entry = Entry {
            reservation,
            reconciliation: Reconciliation::Reserved,
        };
        self.entries.insert(key, entry.clone());
        self.revision = next_revision;
        Ok(entry)
    }
    pub fn reconcile(&mut self, reservation: &Reservation, event: UsageEvent) -> Result<Entry, QuotaError> {
        self.validate()?;
        let key = reservation_key(reservation);
        let existing = self.entries.get(key).ok_or(QuotaError::UnknownAttempt)?;
        if existing.reservation != *reservation {
            return Err(QuotaError::Conflict);
        }
        let next = event.reconciliation(&reservation.schedule, reservation.units)?;
        if let Reconciliation::Settled { .. } = existing.reconciliation {
            return if existing.reconciliation == next {
                Ok(existing.clone())
            } else {
                Err(QuotaError::Conflict)
            };
        }
        if existing.reconciliation == next {
            return Ok(existing.clone());
        }
        let revision = self.revision.checked_add(1).ok_or(QuotaError::Overflow)?;
        let entry = self.entries.get_mut(key).ok_or(QuotaError::UnknownAttempt)?;
        entry.reconciliation = next;
        let output = entry.clone();
        self.revision = revision;
        Ok(output)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Usage {
    pub cpu_ms: u64,
    pub memory_byte_ms: u64,
    /// Observed executor stdout + stderr, not network or output NAR transfer.
    pub executor_output_bytes: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsageEvent {
    Complete {
        event_digest: String,
        usage: Usage,
        trusted_and_complete: bool,
    },
    /// Only authenticated confirmation that execution never began can release zero.
    CancelledBeforeExecution {
        event_digest: String,
        verified_never_started: bool,
    },
    Cancelled,
    WorkerLost,
    Partial,
}
/// One nonzero per-unit divisor taken from an admitted schedule.
#[derive(Clone, Copy)]
struct UnitDivisor(u64);

fn ceil_units(value: u64, divisor: UnitDivisor) -> Result<u64, QuotaError> {
    let divisor = divisor.0;
    value
        .checked_div(divisor)
        .and_then(|n| n.checked_add(u64::from(!value.is_multiple_of(divisor))))
        .ok_or(QuotaError::Overflow)
}
impl UsageEvent {
    fn reconciliation(self, schedule: &UnitSchedule, reserved: u64) -> Result<Reconciliation, QuotaError> {
        let (event_digest, charge) = match self {
            Self::Complete {
                event_digest,
                usage,
                trusted_and_complete: true,
            } => {
                let charge = ceil_units(usage.cpu_ms, UnitDivisor(schedule.cpu_ms_per_unit))?
                    .checked_add(ceil_units(usage.memory_byte_ms, UnitDivisor(schedule.memory_byte_ms_per_unit))?)
                    .and_then(|n| {
                        n.checked_add(
                            ceil_units(
                                usage.executor_output_bytes,
                                UnitDivisor(schedule.executor_output_bytes_per_unit),
                            )
                            .ok()?,
                        )
                    })
                    .ok_or(QuotaError::Overflow)?;
                (event_digest, charge)
            }
            Self::CancelledBeforeExecution {
                event_digest,
                verified_never_started: true,
            } => (event_digest, 0),
            Self::Complete { .. } => return Ok(Reconciliation::Pending(PendingReason::UntrustedObservation)),
            Self::CancelledBeforeExecution { .. } | Self::Cancelled => {
                return Ok(Reconciliation::Pending(PendingReason::CancelledWithoutFinalUsage));
            }
            Self::WorkerLost => return Ok(Reconciliation::Pending(PendingReason::WorkerLost)),
            Self::Partial => return Ok(Reconciliation::Pending(PendingReason::PartialObservation)),
        };
        if !crate::output::is_blake3_hex_digest(&event_digest) {
            return Err(QuotaError::Invalid);
        }
        if charge > reserved {
            return Ok(Reconciliation::Pending(PendingReason::UsageExceedsReservation));
        }
        Ok(Reconciliation::Settled {
            event_digest,
            charged_units: charge,
        })
    }
}
