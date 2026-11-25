use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;

use serde::Deserialize;
use serde::Serialize;

pub const ORACLE_SCHEMA_VERSION: &str = "mantle-store-oracle-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationKind {
    InvalidCacheHitDiscovered,
    IncompleteIndexExposed,
    NoClobberBroken,
    ProtectedRootLost,
    UnsignedTrustMutated,
    AcknowledgedDurabilityLost,
    StaleGcPlanMutatedStore,
    FencedMutationNotBlocked,
    NegativeMutantUnreachedOrUncaught,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepClassification {
    Passed,
    ExpectedRejection { code: String },
    IncompleteOrBlocked { reason: String },
    HarnessError { detail: String },
    SafetyViolation { kind: ViolationKind, detail: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPlanSnapshot {
    pub plan_id: String,
    pub root_snapshot: BTreeSet<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct StoreRefModel {
    pub objects_present: BTreeSet<u8>,
    pub dag_refs: BTreeMap<u8, BTreeSet<u8>>,
    pub active_pins: BTreeMap<(u8, u8, u8), ()>,
    pub registered_roots: BTreeSet<u8>,
    pub unsigned_objects: BTreeSet<u8>,
    pub action_records: BTreeMap<u8, String>,
    pub last_plan: Option<ModelPlanSnapshot>,
    pub acknowledged_durable: BTreeSet<u8>,
}

impl StoreRefModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reachable_live_objects(&self) -> BTreeSet<u8> {
        let mut reachable = BTreeSet::new();
        let mut queue = VecDeque::new();
        for &(object_idx, _, _) in self.active_pins.keys() {
            if self.objects_present.contains(&object_idx) {
                queue.push_back(object_idx);
            }
        }
        for &object_idx in &self.registered_roots {
            if self.objects_present.contains(&object_idx) {
                queue.push_back(object_idx);
            }
        }
        let max_steps = 128;
        for _ in 0..max_steps {
            let Some(idx) = queue.pop_front() else {
                break;
            };
            if !reachable.insert(idx) {
                continue;
            }
            if let Some(deps) = self.dag_refs.get(&idx) {
                for &dep in deps {
                    if self.objects_present.contains(&dep) && !reachable.contains(&dep) {
                        queue.push_back(dep);
                    }
                }
            }
        }
        reachable
    }

    pub fn apply_gc_collection(&mut self) {
        let live = self.reachable_live_objects();
        self.objects_present.retain(|idx| live.contains(idx));
        self.unsigned_objects.retain(|idx| live.contains(idx));
        self.acknowledged_durable.retain(|idx| live.contains(idx));
        self.action_records.retain(|idx, _| live.contains(idx));
    }
}
