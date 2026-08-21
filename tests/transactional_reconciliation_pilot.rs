//! Product-boundary pilot for immutable Mantle GC plans.

const ROOT: &str = "/mantle/store/aaaaaaaa-root";
const DEAD: &str = "/mantle/store/bbbbbbbb-dead";
const ROOT_BYTES: u64 = 11;
const DEAD_BYTES: u64 = 13;
const POLICY_BYTE: u8 = 17;
const ATTEMPT_BYTE: u8 = 19;
const CHANGED_BYTE: u8 = 23;
const DIGEST_BYTES: usize = 32;
const OBSERVED_REVISION: u64 = 5;
const OPERATION_GENERATION: u64 = 1;
const OPERATION_BOUND: u32 = 1;
const PREREQUISITE_BOUND: u32 = 1;

struct MantleBlake3;

impl transactional_reconciliation_core::Blake3IdentityDeriver for MantleBlake3 {
    fn derive_identity(
        &self,
        framed_bytes: &[u8],
    ) -> Result<transactional_reconciliation_core::Identity, transactional_reconciliation_core::CoreError> {
        transactional_reconciliation_core::Identity::new(*blake3::hash(framed_bytes).as_bytes())
    }
}

#[derive(Debug)]
enum TestError {
    Gc,
    Core,
}

impl From<crunch_gc_core::GcPlanError> for TestError {
    fn from(_error: crunch_gc_core::GcPlanError) -> Self {
        Self::Gc
    }
}

impl From<transactional_reconciliation_core::CoreError> for TestError {
    fn from(_error: transactional_reconciliation_core::CoreError) -> Self {
        Self::Core
    }
}

fn entry(path_id: &str, declared_nar_bytes: u64) -> crunch_gc_core::GcEntry {
    crunch_gc_core::GcEntry {
        path_id: path_id.to_string(),
        references: Vec::new(),
        declared_nar_bytes,
        ownership: crunch_gc_core::GcOwnership::Overlay,
    }
}

fn gc_plan() -> Result<crunch_gc_core::GcPlan, crunch_gc_core::GcPlanError> {
    crunch_gc_core::plan_gc(crunch_gc_core::GcPlanRequest {
        roots: vec![ROOT.to_string()],
        entries: vec![entry(ROOT, ROOT_BYTES), entry(DEAD, DEAD_BYTES)],
        execution_mode: crunch_gc_core::GcExecutionMode::Execute,
    })
}

fn identity(
    byte: u8,
) -> Result<transactional_reconciliation_core::Identity, transactional_reconciliation_core::CoreError> {
    transactional_reconciliation_core::Identity::new([byte; DIGEST_BYTES])
}

#[test]
fn exact_gc_plan_is_current_and_changed_plan_is_stale() -> Result<(), TestError> {
    let gc = gc_plan()?;
    let desired = transactional_reconciliation_core::Identity::new(gc.plan_id.into_bytes())?;
    let policy = identity(POLICY_BYTE)?;
    let revision = transactional_reconciliation_core::Revision::observed(OBSERVED_REVISION);
    let operation = transactional_reconciliation_core::OperationDraft::new(
        desired,
        desired,
        transactional_reconciliation_core::Generation::observed(OPERATION_GENERATION),
        policy,
    );
    let plan = transactional_reconciliation_core::build_plan(
        &MantleBlake3,
        transactional_reconciliation_core::Limits::new(
            transactional_reconciliation_core::Bound::new(OPERATION_BOUND)?,
            transactional_reconciliation_core::Bound::new(PREREQUISITE_BOUND)?,
        ),
        transactional_reconciliation_core::PlanningInput::new(revision, desired, policy, Vec::new(), vec![operation]),
    )?;
    let current = transactional_reconciliation_core::CurrentFacts::new(revision, desired, policy, Vec::new());
    let changed =
        transactional_reconciliation_core::CurrentFacts::new(revision, identity(CHANGED_BYTE)?, policy, Vec::new());
    let operation_identity = plan.operations()[0].idempotency_identity();
    let reservation =
        transactional_reconciliation_core::reserve_attempt(&plan, operation_identity, identity(ATTEMPT_BYTE)?)?;
    let dispatch = transactional_reconciliation_core::admit_dispatch(
        &plan,
        &current,
        reservation,
        transactional_reconciliation_core::ReservationObservation::Durable(reservation),
    )?;

    assert_eq!(dispatch.plan_identity(), plan.identity());
    assert_eq!(
        transactional_reconciliation_core::check_freshness(&plan, &changed),
        Err(transactional_reconciliation_core::FreshnessError::DesiredState)
    );
    Ok(())
}
