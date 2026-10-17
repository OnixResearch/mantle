//! Borrowed decisions over caller-authenticated resource, lease, and receiver facts.
//!
//! This module neither authenticates a worker nor collects an observation. The
//! adapter owns canonical wire encoding, BLAKE3 digests, and durable mutation.

use core::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capacity {
    pub cpu_units: u32,
    pub memory_bytes: u64,
    pub scratch_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapacityShortfall {
    Cpu,
    Memory,
    Scratch,
}

/// Subtract already validated live reservations, including those of the
/// current worker only. The caller maps an underflow to its snapshot verdict.
pub fn remaining_capacity(total: Capacity, used: Capacity) -> Option<Capacity> {
    Some(Capacity {
        cpu_units: total.cpu_units.checked_sub(used.cpu_units)?,
        memory_bytes: total.memory_bytes.checked_sub(used.memory_bytes)?,
        scratch_bytes: total.scratch_bytes.checked_sub(used.scratch_bytes)?,
    })
}

/// The first shortfall has the same CPU → memory → scratch precedence as the
/// existing remote resource admission contract.
pub fn reserve_capacity(available: Capacity, requested: Capacity) -> Result<Capacity, CapacityShortfall> {
    Ok(Capacity {
        cpu_units: available.cpu_units.checked_sub(requested.cpu_units).ok_or(CapacityShortfall::Cpu)?,
        memory_bytes: available.memory_bytes.checked_sub(requested.memory_bytes).ok_or(CapacityShortfall::Memory)?,
        scratch_bytes: available
            .scratch_bytes
            .checked_sub(requested.scratch_bytes)
            .ok_or(CapacityShortfall::Scratch)?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    Exact,
    Compatible,
    Constrained,
}

/// Named accelerator/token quantities are checked by the existing canonical
/// adapter. Their two derived booleans participate without dropping them.
pub fn classify_fit(remaining: Capacity, requested: Capacity, named_empty: bool, named_constrained: bool) -> Fit {
    if remaining.cpu_units == 0 && remaining.memory_bytes == 0 && remaining.scratch_bytes == 0 && named_empty {
        Fit::Exact
    } else if remaining.cpu_units < requested.cpu_units
        || remaining.memory_bytes < requested.memory_bytes
        || remaining.scratch_bytes < requested.scratch_bytes
        || named_constrained
    {
        Fit::Constrained
    } else {
        Fit::Compatible
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaseScope<'a> {
    pub worker_endpoint_id: &'a str,
    pub worker_generation: u64,
    pub job_id: &'a str,
    pub attempt_id: &'a str,
    pub fence_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseMismatch {
    Worker,
    Job,
    StaleFence,
    UnknownFence,
    Attempt,
}

/// This check follows validation of the lease schema and unchanged v1 digest.
/// A restarted worker cannot mutate or preserve its predecessor's lease.
pub fn authorize_lease_scope(lease: LeaseScope<'_>, current: LeaseScope<'_>) -> Result<(), LeaseMismatch> {
    if lease.worker_endpoint_id != current.worker_endpoint_id || lease.worker_generation != current.worker_generation {
        return Err(LeaseMismatch::Worker);
    }
    if lease.job_id != current.job_id {
        return Err(LeaseMismatch::Job);
    }
    if current.fence_generation < lease.fence_generation {
        return Err(LeaseMismatch::StaleFence);
    }
    if current.fence_generation > lease.fence_generation {
        return Err(LeaseMismatch::UnknownFence);
    }
    if lease.attempt_id != current.attempt_id {
        return Err(LeaseMismatch::Attempt);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReceiverDemand {
    pub missing_object_count: u32,
    pub reused_bytes: u64,
    pub missing_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalityClass {
    FullyPresent,
    PartiallyPresent,
    NoVerifiedContent,
    Unverified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferClass {
    None,
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalityDecision {
    pub present_object_count: u32,
    pub missing_object_count: u32,
    pub present_bytes: u64,
    pub missing_bytes: u64,
    pub locality: LocalityClass,
    pub transfer: TransferClass,
}

/// Only `Some` denotes already verified receiver-derived transfer demand.
/// No hint, manifest, or worker claim is promoted into verified presence.
pub fn classify_receiver_locality(
    demanded_object_count: u32,
    demanded_bytes: u64,
    verified: Option<ReceiverDemand>,
) -> Option<LocalityDecision> {
    let Some(demand) = verified else {
        return Some(LocalityDecision {
            present_object_count: 0,
            missing_object_count: demanded_object_count,
            present_bytes: 0,
            missing_bytes: demanded_bytes,
            locality: LocalityClass::Unverified,
            transfer: TransferClass::Large,
        });
    };
    let present_object_count = demanded_object_count.checked_sub(demand.missing_object_count)?;
    if demand.reused_bytes.checked_add(demand.missing_bytes)? != demanded_bytes {
        return None;
    }
    let (locality, transfer) = if demand.missing_object_count == 0 && demand.missing_bytes == 0 {
        (LocalityClass::FullyPresent, TransferClass::None)
    } else if present_object_count == 0 && demand.reused_bytes == 0 {
        (LocalityClass::NoVerifiedContent, TransferClass::Large)
    } else {
        let transfer = match demand.missing_bytes.cmp(&demand.reused_bytes) {
            Ordering::Less => TransferClass::Small,
            Ordering::Equal => TransferClass::Medium,
            Ordering::Greater => TransferClass::Large,
        };
        (LocalityClass::PartiallyPresent, transfer)
    };
    Some(LocalityDecision {
        present_object_count,
        missing_object_count: demand.missing_object_count,
        present_bytes: demand.reused_bytes,
        missing_bytes: demand.missing_bytes,
        locality,
        transfer,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementPreference {
    KnownGraph,
    ResourceFit,
    LocalityTransfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementFacts<'a> {
    pub worker_endpoint_id: &'a str,
    pub resource_fit: u8,
    pub content_locality: u8,
    pub transfer_cost: u8,
}

/// Ordinals are supplied by the adapter's existing public enum ordering.
pub fn compare_placement(
    order: impl IntoIterator<Item = PlacementPreference>,
    left: PlacementFacts<'_>,
    right: PlacementFacts<'_>,
) -> Ordering {
    for field in order {
        let ordering = match field {
            PlacementPreference::KnownGraph => Ordering::Equal,
            PlacementPreference::ResourceFit => right.resource_fit.cmp(&left.resource_fit),
            PlacementPreference::LocalityTransfer => right
                .content_locality
                .cmp(&left.content_locality)
                .then_with(|| right.transfer_cost.cmp(&left.transfer_cost)),
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    left.worker_endpoint_id.cmp(right.worker_endpoint_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_denies_each_shortfall_without_promoting_remaining_fit() {
        let total = Capacity {
            cpu_units: 8,
            memory_bytes: 16,
            scratch_bytes: 32,
        };
        let used = Capacity {
            cpu_units: 2,
            memory_bytes: 4,
            scratch_bytes: 8,
        };
        let available = remaining_capacity(total, used).unwrap();
        assert_eq!(available.cpu_units, 6);
        assert_eq!(
            reserve_capacity(available, Capacity {
                cpu_units: 7,
                memory_bytes: 1,
                scratch_bytes: 1
            }),
            Err(CapacityShortfall::Cpu)
        );
        assert_eq!(
            reserve_capacity(available, Capacity {
                cpu_units: 1,
                memory_bytes: 13,
                scratch_bytes: 1
            }),
            Err(CapacityShortfall::Memory)
        );
        assert_eq!(
            reserve_capacity(available, Capacity {
                cpu_units: 1,
                memory_bytes: 1,
                scratch_bytes: 25
            }),
            Err(CapacityShortfall::Scratch)
        );
        let requested = Capacity {
            cpu_units: 2,
            memory_bytes: 4,
            scratch_bytes: 8,
        };
        let after = reserve_capacity(available, requested).unwrap();
        assert_eq!(classify_fit(after, requested, true, false), Fit::Compatible);
        assert_eq!(
            classify_fit(
                Capacity {
                    cpu_units: 0,
                    memory_bytes: 0,
                    scratch_bytes: 0
                },
                requested,
                true,
                false
            ),
            Fit::Exact
        );
        assert_eq!(
            classify_fit(
                after,
                Capacity {
                    cpu_units: 5,
                    ..requested
                },
                true,
                false
            ),
            Fit::Constrained
        );
        assert_eq!(classify_fit(after, requested, false, true), Fit::Constrained);
        assert_eq!(remaining_capacity(used, total), None);
    }

    #[test]
    fn generation_fence_and_attempt_bind_existing_lease() {
        let lease = LeaseScope {
            worker_endpoint_id: "worker",
            worker_generation: 7,
            job_id: "job",
            attempt_id: "attempt",
            fence_generation: 2,
        };
        assert_eq!(authorize_lease_scope(lease, lease), Ok(()));
        assert_eq!(
            authorize_lease_scope(lease, LeaseScope {
                worker_generation: 8,
                ..lease
            }),
            Err(LeaseMismatch::Worker)
        );
        assert_eq!(
            authorize_lease_scope(lease, LeaseScope {
                fence_generation: 1,
                ..lease
            }),
            Err(LeaseMismatch::StaleFence)
        );
        assert_eq!(
            authorize_lease_scope(lease, LeaseScope {
                fence_generation: 3,
                ..lease
            }),
            Err(LeaseMismatch::UnknownFence)
        );
        assert_eq!(
            authorize_lease_scope(lease, LeaseScope {
                attempt_id: "other",
                ..lease
            }),
            Err(LeaseMismatch::Attempt)
        );
    }

    #[test]
    fn locality_cannot_invent_unverified_or_overflowing_receiver_presence() {
        let unverified = classify_receiver_locality(2, 8, None).unwrap();
        assert_eq!(unverified.locality, LocalityClass::Unverified);
        assert_eq!(unverified.present_bytes, 0);
        let partial = classify_receiver_locality(
            2,
            8,
            Some(ReceiverDemand {
                missing_object_count: 1,
                reused_bytes: 4,
                missing_bytes: 4,
            }),
        )
        .unwrap();
        assert_eq!(partial.locality, LocalityClass::PartiallyPresent);
        assert_eq!(partial.transfer, TransferClass::Medium);
        let full = classify_receiver_locality(
            2,
            8,
            Some(ReceiverDemand {
                missing_object_count: 0,
                reused_bytes: 8,
                missing_bytes: 0,
            }),
        )
        .unwrap();
        let missing = classify_receiver_locality(
            2,
            8,
            Some(ReceiverDemand {
                missing_object_count: 2,
                reused_bytes: 0,
                missing_bytes: 8,
            }),
        )
        .unwrap();
        assert_eq!((full.locality, full.transfer), (LocalityClass::FullyPresent, TransferClass::None));
        assert_eq!((missing.locality, missing.transfer), (LocalityClass::NoVerifiedContent, TransferClass::Large));
        assert_eq!(
            classify_receiver_locality(
                2,
                8,
                Some(ReceiverDemand {
                    missing_object_count: 3,
                    reused_bytes: 4,
                    missing_bytes: 4
                })
            ),
            None
        );
    }

    #[test]
    fn preference_order_preserves_eligibility_independent_ranking() {
        let left = PlacementFacts {
            worker_endpoint_id: "a",
            resource_fit: 2,
            content_locality: 1,
            transfer_cost: 1,
        };
        let right = PlacementFacts {
            worker_endpoint_id: "z",
            resource_fit: 2,
            content_locality: 3,
            transfer_cost: 4,
        };
        let order = [PlacementPreference::ResourceFit, PlacementPreference::LocalityTransfer];
        assert_eq!(compare_placement(order, left, right), Ordering::Greater);
        assert_eq!(compare_placement(order, right, left), Ordering::Less);
        let more_fit = PlacementFacts {
            resource_fit: 3,
            content_locality: 0,
            ..left
        };
        assert_eq!(compare_placement(order, more_fit, right), Ordering::Less);
    }
}
