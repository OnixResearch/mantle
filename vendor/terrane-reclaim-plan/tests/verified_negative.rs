use super::*;

fn assert_rejected(result: ReclaimPlan, status: u8) {
    assert_eq!(result.status, status);
    assert!(result.retained.is_empty());
    assert!(result.candidate.is_empty());
}

// r[verify reclaim-plan.admission]
#[test]
fn malformed_inputs_reject_with_their_class_and_no_plan() {
    assert_rejected(plan(2, &vec![], &vec![], &vec![true]), LENGTH_MISMATCH);
    assert_rejected(plan(2, &vec![], &vec![2], &vec![true, true]), ROOT_OUT_OF_RANGE);
    assert_rejected(plan(2, &vec![(0, 2)], &vec![0], &vec![true, true]), EDGE_OUT_OF_RANGE);
    assert_rejected(plan(2, &vec![(5, 0)], &vec![0], &vec![true, true]), EDGE_OUT_OF_RANGE);
}

// r[verify reclaim-plan.protected-closure]
#[test]
fn protected_node_referencing_a_reclaimable_node_rejects() {
    assert_rejected(plan(2, &vec![(0, 1)], &vec![], &vec![false, true]), PROTECTED_REFERENCES_RECLAIMABLE);
    // The reverse direction is allowed: a reclaimable node may reference a protected one.
    assert_eq!(plan(2, &vec![(1, 0)], &vec![], &vec![false, true]).status, PLANNED);
}

// r[verify reclaim-plan.validity]
#[test]
fn checker_rejects_corrupted_plans() {
    let edges = vec![(0, 1)];
    let roots = vec![0];
    let reclaimable = vec![true, true, true];
    let good = plan(3, &edges, &roots, &reclaimable);
    assert!(check_plan(3, &edges, &roots, &reclaimable, &good.retained, &good.candidate));
    // Dropping a reachable node from retention breaks closure.
    assert!(!check_plan(3, &edges, &roots, &reclaimable, &vec![true, false, false], &vec![false, true, true]));
    // Dropping a root breaks root retention.
    assert!(!check_plan(3, &edges, &roots, &reclaimable, &vec![false, false, false], &vec![true, true, true]));
    // Candidates must match exactly.
    assert!(!check_plan(3, &edges, &roots, &reclaimable, &good.retained, &vec![false, false, false]));
    assert!(!check_plan(3, &edges, &roots, &reclaimable, &good.retained, &vec![true, false, true]));
    // Wrong lengths.
    assert!(!check_plan(3, &edges, &roots, &reclaimable, &vec![true, true], &good.candidate));
}
