//! Pure, bounded machine-wide compile admission. Time is supplied by the shell.
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::string::ToString;

use serde::Deserialize;
use serde::Serialize;

pub const SLOT_POLICY_SCHEMA: &str = "mantle-compile-slot-policy-v1";
pub const SLOT_RECEIPT_SCHEMA: &str = "mantle-compile-slot-receipt-v1";
pub const MAX_SLOTS: u32 = 4_096;
pub const MAX_QUEUE: u32 = 65_536;
pub const MAX_BUILD_ID_BYTES: usize = 256;
pub const MAX_WAIT_TIMEOUT_MS: u64 = 3_600_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotFairness {
    RoundRobin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlotPolicy {
    pub schema: String,
    pub total_slots: u32,
    pub per_build_max: u32,
    pub total_queue: u32,
    pub per_build_queue: u32,
    pub wait_timeout_ms: u64,
    pub fairness: SlotFairness,
}

impl SlotPolicy {
    pub fn validate(&self) -> Result<(), SlotRejectReason> {
        if self.schema != SLOT_POLICY_SCHEMA {
            return Err(SlotRejectReason::InvalidPolicy);
        }
        if self.total_slots == 0 || self.total_slots > MAX_SLOTS {
            return Err(SlotRejectReason::InvalidPolicy);
        }
        if self.per_build_max == 0 || self.per_build_max > self.total_slots {
            return Err(SlotRejectReason::InvalidPolicy);
        }
        if self.total_queue == 0 || self.total_queue > MAX_QUEUE {
            return Err(SlotRejectReason::InvalidPolicy);
        }
        if self.per_build_queue == 0 || self.per_build_queue > self.total_queue {
            return Err(SlotRejectReason::InvalidPolicy);
        }
        if self.wait_timeout_ms == 0 || self.wait_timeout_ms > MAX_WAIT_TIMEOUT_MS {
            return Err(SlotRejectReason::InvalidPolicy);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotRejectReason {
    InvalidPolicy,
    InvalidBuildId,
    DuplicateRequest,
    TotalQueueFull,
    BuildQueueFull,
    WaitTimeout,
    Cancelled,
    UnknownRequest,
    RequestIdExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotDecision {
    Queued,
    Granted,
    Released,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlotRequest {
    pub request_id: u64,
    pub build_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlotReceipt {
    pub schema: String,
    pub request_id: u64,
    pub build_id: String,
    pub decision: SlotDecision,
    pub reason: Option<SlotRejectReason>,
    pub fairness: SlotFairness,
    pub observed_ms: u64,
    pub waited_ms: u64,
    pub active_slots: u32,
    pub queued_requests: u32,
    pub total_slots: u32,
    pub per_build_max: u32,
    pub total_queue: u32,
    pub per_build_queue: u32,
    pub wait_timeout_ms: u64,
}

impl SlotReceipt {
    /// Validate an untrusted authority grant before starting a leased child.
    /// Scheduling receipts cannot enlarge the client's declared protocol bounds.
    pub fn is_grant_for(&self, build_id: &str) -> bool {
        self.schema == SLOT_RECEIPT_SCHEMA
            && self.request_id != 0
            && self.build_id == build_id
            && !build_id.is_empty()
            && build_id.len() <= MAX_BUILD_ID_BYTES
            && self.decision == SlotDecision::Granted
            && self.reason.is_none()
            && self.total_slots > 0
            && self.total_slots <= MAX_SLOTS
            && self.per_build_max > 0
            && self.per_build_max <= self.total_slots
            && self.total_queue > 0
            && self.total_queue <= MAX_QUEUE
            && self.per_build_queue > 0
            && self.per_build_queue <= self.total_queue
            && self.wait_timeout_ms > 0
            && self.wait_timeout_ms <= MAX_WAIT_TIMEOUT_MS
            && self.waited_ms < self.wait_timeout_ms
            && self.waited_ms <= self.observed_ms
            && self.active_slots > 0
            && self.active_slots <= self.total_slots
            && self.queued_requests <= self.total_queue
    }
}

#[derive(Debug)]
struct Pending {
    request: SlotRequest,
    admitted_ms: u64,
}

#[derive(Debug)]
struct Reservation {
    request: SlotRequest,
    admitted_ms: u64,
    granted_ms: u64,
}

/// The identity and timing facts a slot receipt reports.
struct SlotReceiptFacts {
    request_id: u64,
    build_id: String,
    decision: SlotDecision,
    reason: Option<SlotRejectReason>,
    observed_ms: u64,
    waited_ms: u64,
}

/// No completed-request history is retained. Occupancy is bounded by policy.
#[derive(Debug)]
pub struct SlotState {
    policy: SlotPolicy,
    queue: VecDeque<Pending>,
    active: VecDeque<Reservation>,
    build_order: VecDeque<String>,
    last_build: Option<String>,
}

// r[impl mantle.compile_slots.bounded_fairness]
impl SlotState {
    pub fn new(policy: SlotPolicy) -> Result<Self, SlotRejectReason> {
        policy.validate()?;
        Ok(Self {
            policy,
            queue: VecDeque::new(),
            active: VecDeque::new(),
            build_order: VecDeque::new(),
            last_build: None,
        })
    }

    pub fn policy(&self) -> &SlotPolicy {
        &self.policy
    }

    pub fn active_count(&self) -> u32 {
        self.active.len() as u32
    }

    pub fn queued_count(&self) -> u32 {
        self.queue.len() as u32
    }

    pub fn rejection(&self, request: &SlotRequest, now_ms: u64, reason: SlotRejectReason) -> SlotReceipt {
        self.receipt(request, SlotDecision::Rejected, Some(reason), now_ms, 0)
    }

    pub fn admit(&mut self, request: SlotRequest, now_ms: u64) -> SlotReceipt {
        if let Some(reason) = self.admission_rejection(&request) {
            return self.rejection(&request, now_ms, reason);
        }
        if !self.build_order.contains(&request.build_id) {
            let index = if self.build_order.back() == self.last_build.as_ref() {
                self.build_order.len().saturating_sub(1)
            } else {
                self.build_order.len()
            };
            self.build_order.insert(index, request.build_id.clone());
        }
        let request_id = request.request_id;
        let build_id = request.build_id.clone();
        self.queue.push_back(Pending {
            request,
            admitted_ms: now_ms,
        });
        assert!(self.queued_count() <= self.policy.total_queue);
        assert!(self.active_count() <= self.policy.total_slots);
        self.receipt_fields(SlotReceiptFacts {
            request_id,
            build_id,
            decision: SlotDecision::Queued,
            reason: None,
            observed_ms: now_ms,
            waited_ms: 0,
        })
    }

    /// Reject admissions that duplicate a live request or exceed the queue
    /// bounds of the policy.
    fn admission_rejection(&self, request: &SlotRequest) -> Option<SlotRejectReason> {
        if request.build_id.is_empty() || request.build_id.len() > MAX_BUILD_ID_BYTES {
            return Some(SlotRejectReason::InvalidBuildId);
        }
        let is_duplicate = self.queue.iter().any(|p| p.request.request_id == request.request_id)
            || self.active.iter().any(|p| p.request.request_id == request.request_id);
        if is_duplicate {
            return Some(SlotRejectReason::DuplicateRequest);
        }
        if self.queued_count() >= self.policy.total_queue {
            return Some(SlotRejectReason::TotalQueueFull);
        }
        let queued_for_build = self.queue.iter().filter(|p| p.request.build_id == request.build_id).count() as u64;
        if queued_for_build >= u64::from(self.policy.per_build_queue) {
            return Some(SlotRejectReason::BuildQueueFull);
        }
        None
    }

    /// Round-robin among eligible builds, FIFO within each build. Expired
    /// requests cannot reserve even when their connection has not yet woken.
    pub fn reserve(&mut self, now_ms: u64) -> Option<SlotReceipt> {
        if self.active_count() >= self.policy.total_slots {
            return None;
        }
        let unexpired = |pending: &Pending| now_ms.saturating_sub(pending.admitted_ms) < self.policy.wait_timeout_ms;
        let build_index = self.build_order.iter().position(|build| {
            let active = self.active.iter().filter(|r| r.request.build_id == *build).count() as u64;
            active < u64::from(self.policy.per_build_max)
                && self.queue.iter().any(|pending| pending.request.build_id == *build && unexpired(pending))
        })?;
        let next_build = &self.build_order[build_index];
        let index = self.queue.iter().position(|p| p.request.build_id == *next_build && unexpired(p))?;
        self.build_order.rotate_left(build_index.saturating_add(1));
        let pending = self.queue.remove(index)?;
        self.last_build = Some(pending.request.build_id.clone());
        let waited_ms = now_ms.saturating_sub(pending.admitted_ms);
        let request_id = pending.request.request_id;
        let build_id = pending.request.build_id.clone();
        self.active.push_back(Reservation {
            request: pending.request,
            admitted_ms: pending.admitted_ms,
            granted_ms: now_ms,
        });
        assert!(self.active_count() <= self.policy.total_slots);
        assert!(self.queued_count() < self.policy.total_queue);
        Some(self.receipt_fields(SlotReceiptFacts {
            request_id,
            build_id,
            decision: SlotDecision::Granted,
            reason: None,
            observed_ms: now_ms,
            waited_ms,
        }))
    }

    pub fn grant(&self, request_id: u64) -> Option<SlotReceipt> {
        let active = self.active.iter().find(|r| r.request.request_id == request_id)?;
        Some(self.receipt(
            &active.request,
            SlotDecision::Granted,
            None,
            active.granted_ms,
            active.granted_ms.saturating_sub(active.admitted_ms),
        ))
    }

    /// The shell MUST establish actual child completion before calling this.
    pub fn release(&mut self, request_id: u64, now_ms: u64) -> Option<SlotReceipt> {
        let index = self.active.iter().position(|r| r.request.request_id == request_id)?;
        let active = self.active.remove(index)?;
        self.remove_idle_build(&active.request.build_id);
        assert!(self.active_count() < self.policy.total_slots);
        assert!(self.queued_count() <= self.policy.total_queue);
        Some(self.receipt(
            &active.request,
            SlotDecision::Released,
            None,
            now_ms,
            active.granted_ms.saturating_sub(active.admitted_ms),
        ))
    }

    /// Only pending requests can be cancelled. Reservations require release.
    pub fn cancel(&mut self, request_id: u64, now_ms: u64, reason: SlotRejectReason) -> Option<SlotReceipt> {
        let index = self.queue.iter().position(|r| r.request.request_id == request_id)?;
        let pending = self.queue.remove(index)?;
        self.remove_idle_build(&pending.request.build_id);
        Some(self.receipt(
            &pending.request,
            SlotDecision::Cancelled,
            Some(reason),
            now_ms,
            now_ms.saturating_sub(pending.admitted_ms),
        ))
    }

    fn remove_idle_build(&mut self, build_id: &str) {
        if !self.queue.iter().any(|p| p.request.build_id == build_id)
            && !self.active.iter().any(|p| p.request.build_id == build_id)
            && let Some(index) = self.build_order.iter().position(|build| build == build_id)
        {
            self.build_order.remove(index);
        }
    }

    fn receipt(
        &self,
        request: &SlotRequest,
        decision: SlotDecision,
        reason: Option<SlotRejectReason>,
        observed_ms: u64,
        waited_ms: u64,
    ) -> SlotReceipt {
        self.receipt_fields(SlotReceiptFacts {
            request_id: request.request_id,
            build_id: request.build_id.clone(),
            decision,
            reason,
            observed_ms,
            waited_ms,
        })
    }

    /// Build a receipt from already owned identity fields, so callers that
    /// consumed their request do not have to look it up again.
    fn receipt_fields(&self, facts: SlotReceiptFacts) -> SlotReceipt {
        SlotReceipt {
            schema: SLOT_RECEIPT_SCHEMA.to_string(),
            request_id: facts.request_id,
            build_id: facts.build_id,
            decision: facts.decision,
            reason: facts.reason,
            fairness: self.policy.fairness,
            observed_ms: facts.observed_ms,
            waited_ms: facts.waited_ms,
            active_slots: self.active_count(),
            queued_requests: self.queued_count(),
            total_slots: self.policy.total_slots,
            per_build_max: self.policy.per_build_max,
            total_queue: self.policy.total_queue,
            per_build_queue: self.policy.per_build_queue,
            wait_timeout_ms: self.policy.wait_timeout_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> SlotPolicy {
        SlotPolicy {
            schema: SLOT_POLICY_SCHEMA.to_string(),
            total_slots: 1,
            per_build_max: 1,
            total_queue: 4,
            per_build_queue: 3,
            wait_timeout_ms: 100,
            fairness: SlotFairness::RoundRobin,
        }
    }

    fn request(request_id: u64, build_id: &str) -> SlotRequest {
        SlotRequest {
            request_id,
            build_id: build_id.to_string(),
        }
    }

    #[test]
    fn grants_require_matching_identity_and_possible_policy_state() {
        let mut state = SlotState::new(policy()).unwrap();
        state.admit(request(1, "consumer"), 10);
        let grant = state.reserve(12).unwrap();
        assert!(grant.is_grant_for("consumer"));
        assert!(!grant.is_grant_for("other-build"));
        let corruptions: [fn(&mut SlotReceipt); 12] = [
            |r| r.request_id = 0,
            |r| r.schema.clear(),
            |r| r.reason = Some(SlotRejectReason::WaitTimeout),
            |r| r.total_slots = MAX_SLOTS + 1,
            |r| r.per_build_max = r.total_slots + 1,
            |r| r.total_queue = MAX_QUEUE + 1,
            |r| r.per_build_queue = r.total_queue + 1,
            |r| r.wait_timeout_ms = MAX_WAIT_TIMEOUT_MS + 1,
            |r| r.waited_ms = r.wait_timeout_ms,
            |r| r.active_slots = 0,
            |r| r.active_slots = r.total_slots + 1,
            |r| r.queued_requests = r.total_queue + 1,
        ];
        for corrupt in corruptions {
            let mut invalid = grant.clone();
            corrupt(&mut invalid);
            assert!(!invalid.is_grant_for("consumer"), "accepted invalid grant: {invalid:?}");
        }
        assert!(!state.release(1, 13).unwrap().is_grant_for("consumer"));
    }
    #[test]
    fn monopolizing_build_yields_to_other_waiting_build() {
        let mut state = SlotState::new(policy()).unwrap();
        state.admit(request(1, "large"), 0);
        assert_eq!(state.reserve(0).unwrap().request_id, 1);
        state.admit(request(2, "large"), 1);
        state.admit(request(3, "small"), 2);
        assert!(state.reserve(2).is_none());
        state.release(1, 3).unwrap();
        assert_eq!(state.reserve(3).unwrap().request_id, 3);
        state.release(3, 4).unwrap();
        assert_eq!(state.reserve(4).unwrap().request_id, 2);
    }

    #[test]
    fn three_builds_rotate_without_fifo_interleaving_bias() {
        let mut state = SlotState::new(policy()).unwrap();
        state.admit(request(1, "a"), 0);
        state.admit(request(2, "a"), 0);
        state.admit(request(3, "b"), 0);
        state.admit(request(4, "c"), 0);
        for expected in [1, 3, 4, 2] {
            let receipt = state.reserve(1).unwrap();
            assert_eq!(receipt.request_id, expected);
            state.release(expected, 2).unwrap();
        }
    }

    #[test]
    fn queue_bounds_timeout_and_cancellation_are_explicit() {
        let mut state = SlotState::new(policy()).unwrap();
        for id in 1..=3 {
            state.admit(request(id, "a"), 0);
        }
        assert_eq!(state.admit(request(4, "a"), 0).reason, Some(SlotRejectReason::BuildQueueFull));
        state.admit(request(4, "b"), 0);
        assert_eq!(state.admit(request(5, "c"), 0).reason, Some(SlotRejectReason::TotalQueueFull));
        assert!(state.reserve(100).is_none());
        let cancelled = state.cancel(1, 100, SlotRejectReason::WaitTimeout).unwrap();
        assert_eq!(cancelled.waited_ms, 100);
        assert_eq!(cancelled.queued_requests, 3);
        assert!(state.cancel(1, 100, SlotRejectReason::Cancelled).is_none());
    }

    #[test]
    fn per_build_cap_and_exactly_once_release() {
        let mut p = policy();
        p.total_slots = 2;
        let mut state = SlotState::new(p).unwrap();
        state.admit(request(1, "a"), 0);
        state.admit(request(2, "a"), 0);
        state.admit(request(3, "b"), 0);
        assert_eq!(state.reserve(0).unwrap().request_id, 1);
        assert_eq!(state.reserve(0).unwrap().request_id, 3);
        assert!(state.cancel(1, 1, SlotRejectReason::Cancelled).is_none());
        assert!(state.release(1, 1).is_some());
        assert!(state.release(1, 1).is_none());
        assert_eq!(state.reserve(1).unwrap().request_id, 2);
        assert_eq!(state.active_count(), 2);
    }
}
