use proptest::prelude::*;
use serde::Deserialize;
use serde::Serialize;

pub const HISTORY_SCHEMA_VERSION: &str = "mantle-store-history-v1";
pub const MAX_OBJECTS: u8 = 6;
pub const MAX_STEPS_PROPTEST: usize = 10;
pub const MAX_PARTICIPANTS: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreProfile {
    SnixSingle,
    CasitaSingle,
}

impl StoreProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SnixSingle => "snix-single",
            Self::CasitaSingle => "casita-single",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustAdmissionMode {
    SignedTrusted,
    SignedUntrusted,
    Unsigned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FaultInjectionMode {
    None,
    RecordOnlyInterruptedPublish,
    DanglingIndexWithoutRecord,
    ConflictingRecordBytes,
    HttpIndexPut500,
    AbruptChildAbort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GcExecutionKind {
    DryRunPlanOnly,
    ExecuteLatestPlan,
    ExecuteStalePlanAfterRootChange,
    ExecuteUnguarded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReopenKind {
    InProcessReopen,
    FreshChildProcessReopen,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreOp {
    PublishSignedOutput {
        object_idx: u8,
        is_publish_action_result: bool,
        is_register_root: bool,
        fault: FaultInjectionMode,
    },
    LookupAndProbeActionResult {
        object_idx: u8,
        is_verify_export: bool,
    },
    ImportCacheDir {
        object_idx: u8,
        dep_indices: Vec<u8>,
        trust_mode: TrustAdmissionMode,
    },
    PinRoot {
        object_idx: u8,
        owner_idx: u8,
        reason_idx: u8,
    },
    UnpinRoot {
        object_idx: u8,
        owner_idx: u8,
        reason_idx: u8,
    },
    RunGc {
        kind: GcExecutionKind,
    },
    RecoverCasitaGc,
    Reopen {
        kind: ReopenKind,
    },
    CompetingProcessMutation {
        is_wait_for_lock: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryProgramV1 {
    pub schema_version: String,
    pub profile: StoreProfile,
    pub seed: u64,
    pub steps: Vec<StoreOp>,
}

fn arb_trust_mode() -> impl Strategy<Value = TrustAdmissionMode> {
    prop_oneof![
        Just(TrustAdmissionMode::SignedTrusted),
        Just(TrustAdmissionMode::SignedUntrusted),
        Just(TrustAdmissionMode::Unsigned),
    ]
}

fn arb_fault_mode() -> impl Strategy<Value = FaultInjectionMode> {
    prop_oneof![
        4 => Just(FaultInjectionMode::None),
        1 => Just(FaultInjectionMode::RecordOnlyInterruptedPublish),
        1 => Just(FaultInjectionMode::DanglingIndexWithoutRecord),
        1 => Just(FaultInjectionMode::ConflictingRecordBytes),
        1 => Just(FaultInjectionMode::HttpIndexPut500),
        1 => Just(FaultInjectionMode::AbruptChildAbort),
    ]
}

fn arb_gc_kind() -> impl Strategy<Value = GcExecutionKind> {
    prop_oneof![
        Just(GcExecutionKind::DryRunPlanOnly),
        Just(GcExecutionKind::ExecuteLatestPlan),
        Just(GcExecutionKind::ExecuteStalePlanAfterRootChange),
        Just(GcExecutionKind::ExecuteUnguarded),
    ]
}

fn canonical_deps_for_object(object_idx: u8) -> Vec<u8> {
    match object_idx {
        0 => Vec::new(),
        1 => vec![0],
        2 => Vec::new(),
        3 => vec![0, 2],
        4 => vec![1, 3],
        _ => Vec::new(),
    }
}

fn arb_store_op() -> impl Strategy<Value = StoreOp> {
    prop_oneof![
        3 => (0..MAX_OBJECTS, any::<bool>(), any::<bool>(), arb_fault_mode()).prop_map(
            |(object_idx, is_publish_action_result, is_register_root, fault)| StoreOp::PublishSignedOutput {
                object_idx,
                is_publish_action_result,
                is_register_root,
                fault,
            },
        ),
        2 => (0..MAX_OBJECTS, any::<bool>()).prop_map(|(object_idx, is_verify_export)| {
            StoreOp::LookupAndProbeActionResult {
                object_idx,
                is_verify_export,
            }
        }),
        2 => (0..MAX_OBJECTS, arb_trust_mode()).prop_map(|(object_idx, trust_mode)| StoreOp::ImportCacheDir {
            object_idx,
            dep_indices: canonical_deps_for_object(object_idx),
            trust_mode,
        }),
        2 => (0..MAX_OBJECTS, 0..MAX_PARTICIPANTS, 0..2_u8).prop_map(|(object_idx, owner_idx, reason_idx)| {
            StoreOp::PinRoot {
                object_idx,
                owner_idx,
                reason_idx,
            }
        }),
        2 => (0..MAX_OBJECTS, 0..MAX_PARTICIPANTS, 0..2_u8).prop_map(|(object_idx, owner_idx, reason_idx)| {
            StoreOp::UnpinRoot {
                object_idx,
                owner_idx,
                reason_idx,
            }
        }),
        2 => arb_gc_kind().prop_map(|kind| StoreOp::RunGc { kind }),
        1 => Just(StoreOp::RecoverCasitaGc),
        2 => prop_oneof![
            Just(ReopenKind::InProcessReopen),
            Just(ReopenKind::FreshChildProcessReopen),
        ]
        .prop_map(|kind| StoreOp::Reopen { kind }),
        1 => any::<bool>().prop_map(|is_wait_for_lock| StoreOp::CompetingProcessMutation { is_wait_for_lock }),
    ]
}

pub fn arb_history_program(profile: StoreProfile) -> impl Strategy<Value = HistoryProgramV1> {
    (any::<u64>().no_shrink(), proptest::collection::vec(arb_store_op(), 1..=MAX_STEPS_PROPTEST)).prop_map(
        move |(seed, steps)| HistoryProgramV1 {
            schema_version: HISTORY_SCHEMA_VERSION.to_string(),
            profile,
            seed,
            steps,
        },
    )
}
