//! Every graph with up to three nodes, every root set and every reclaimable
//! set, checked against an independent breadth-first oracle written without
//! the spec functions (which Cargo builds erase).
use super::*;

fn oracle_live(n: usize, edges: &[(usize, usize)], roots: &[usize]) -> Vec<bool> {
    let mut live = vec![false; n];
    let mut queue: Vec<usize> = roots.to_vec();
    while let Some(node) = queue.pop() {
        if live[node] {
            continue;
        }
        live[node] = true;
        for &(from, to) in edges {
            if from == node && !live[to] {
                queue.push(to);
            }
        }
    }
    live
}

fn subsets<T: Copy>(items: &[T]) -> Vec<Vec<T>> {
    (0..1u32 << items.len())
        .map(|mask| items.iter().enumerate().filter(|(i, _)| mask & (1 << i) != 0).map(|(_, &item)| item).collect())
        .collect()
}

// r[verify reclaim-plan.no-live-candidate]
// r[verify reclaim-plan.no-dangling-reference]
// r[verify reclaim-plan.no-leak]
// r[verify reclaim-plan.planner]
// r[verify reclaim-plan.evidence]
#[test]
fn every_small_graph_plans_exactly_the_unreachable_reclaimable_nodes() {
    let mut planned_count = 0u32;
    let mut rejected_count = 0u32;
    for n in 0..=3usize {
        let all_edges: Vec<(usize, usize)> = (0..n).flat_map(|u| (0..n).map(move |v| (u, v))).collect();
        let nodes: Vec<usize> = (0..n).collect();
        for edges in subsets(&all_edges) {
            for roots in subsets(&nodes) {
                for mask in 0..1u32 << n {
                    let reclaimable: Vec<bool> = (0..n).map(|v| mask & (1 << v) != 0).collect();
                    let protected_closed = edges.iter().all(|&(u, v)| reclaimable[u] || !reclaimable[v]);
                    let result = plan(n, &edges, &roots, &reclaimable);
                    if !protected_closed {
                        assert_eq!(result.status, PROTECTED_REFERENCES_RECLAIMABLE);
                        rejected_count += 1;
                        continue;
                    }
                    assert_eq!(result.status, PLANNED, "admissible input hit {}", result.status);
                    let live = oracle_live(n, &edges, &roots);
                    assert_eq!(result.retained, live);
                    for v in 0..n {
                        assert_eq!(result.candidate[v], reclaimable[v] && !live[v]);
                    }
                    for &(u, v) in &edges {
                        if !result.candidate[u] {
                            assert!(!result.candidate[v], "surviving {u} references removed {v}");
                        }
                    }
                    assert!(check_plan(n, &edges, &roots, &reclaimable, &result.retained, &result.candidate));
                    // Any single-bit corruption of a correct plan is either still a
                    // valid plan (over-retention) or rejected; it never removes a live node.
                    for v in 0..n {
                        let mut retained = result.retained.clone();
                        retained[v] = !retained[v];
                        let candidate: Vec<bool> = (0..n).map(|k| reclaimable[k] && !retained[k]).collect();
                        if check_plan(n, &edges, &roots, &reclaimable, &retained, &candidate) {
                            for k in 0..n {
                                assert!(!(candidate[k] && live[k]), "checker accepted removal of live node {k}");
                            }
                        }
                    }
                    planned_count += 1;
                }
            }
        }
    }
    assert_eq!(planned_count + rejected_count, 1 + 2 * 2 * 2 + 16 * 4 * 4 + 512 * 8 * 8);
}
