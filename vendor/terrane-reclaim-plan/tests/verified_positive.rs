use super::*;

// r[verify reclaim-plan.planner]
#[test]
fn chain_keeps_everything_a_root_reaches_and_reclaims_the_rest() {
    // 0 -> 1 -> 2, 3 unreachable, 4 protected and unreachable.
    let edges = vec![(0, 1), (1, 2)];
    let roots = vec![0];
    let reclaimable = vec![true, true, true, true, false];
    let planned = plan(5, &edges, &roots, &reclaimable);
    assert_eq!(planned.status, PLANNED);
    assert_eq!(planned.retained, vec![true, true, true, false, false]);
    assert_eq!(planned.candidate, vec![false, false, false, true, false]);
    assert!(check_plan(5, &edges, &roots, &reclaimable, &planned.retained, &planned.candidate));
}

// r[verify reclaim-plan.planner]
#[test]
fn cycles_and_back_references_reach_a_fixpoint() {
    // Edges listed in reverse order force several propagation rounds.
    let edges = vec![(3, 0), (2, 3), (1, 2), (0, 1)];
    let roots = vec![1];
    let reclaimable = vec![true; 4];
    let planned = plan(4, &edges, &roots, &reclaimable);
    assert_eq!(planned.status, PLANNED);
    assert_eq!(planned.retained, vec![true; 4]);
    assert_eq!(planned.candidate, vec![false; 4]);
}

// r[verify reclaim-plan.validity]
#[test]
fn empty_store_and_rootless_store_plan_cleanly() {
    let empty = plan(0, &vec![], &vec![], &vec![]);
    assert_eq!(empty.status, PLANNED);
    assert!(empty.retained.is_empty() && empty.candidate.is_empty());
    let rootless = plan(2, &vec![(0, 1)], &vec![], &vec![true, true]);
    assert_eq!(rootless.status, PLANNED);
    assert_eq!(rootless.candidate, vec![true, true]);
}
