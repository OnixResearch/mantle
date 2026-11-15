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

fn protected_closed(edges: &[(usize, usize)], reclaimable: &[bool]) -> bool {
    edges.iter().all(|&(u, v)| reclaimable[u] || !reclaimable[v])
}

// r[verify reclaim-plan.least-plan]
// r[verify reclaim-plan.exact-retention]
// r[verify reclaim-plan.evidence]
#[test]
fn planned_result_is_the_least_of_every_valid_plan() {
    let mut valid_plans = 0u32;
    for n in 0..=3usize {
        let all_edges: Vec<(usize, usize)> = (0..n).flat_map(|u| (0..n).map(move |v| (u, v))).collect();
        let nodes: Vec<usize> = (0..n).collect();
        for edges in subsets(&all_edges) {
            for roots in subsets(&nodes) {
                for mask in 0..1u32 << n {
                    let reclaimable: Vec<bool> = (0..n).map(|v| mask & (1 << v) != 0).collect();
                    if !protected_closed(&edges, &reclaimable) {
                        continue;
                    }
                    let planned = plan(n, &edges, &roots, &reclaimable);
                    assert_eq!(planned.status, PLANNED);
                    for retained_mask in 0..1u32 << n {
                        let retained: Vec<bool> = (0..n).map(|v| retained_mask & (1 << v) != 0).collect();
                        let candidate: Vec<bool> = (0..n).map(|v| reclaimable[v] && !retained[v]).collect();
                        if !check_plan(n, &edges, &roots, &reclaimable, &retained, &candidate) {
                            continue;
                        }
                        valid_plans += 1;
                        for v in 0..n {
                            assert!(!planned.retained[v] || retained[v], "planned retains {v} beyond a valid plan");
                            assert!(!candidate[v] || planned.candidate[v], "valid plan reclaims {v}, planned does not");
                        }
                    }
                }
            }
        }
    }
    // Pinned so a narrowed enumeration cannot pass silently.
    assert_eq!(valid_plans, 27_437);
}

// r[verify reclaim-plan.monotone]
// r[verify reclaim-plan.evidence]
#[test]
fn adding_a_root_reference_or_protection_never_adds_a_candidate() {
    for n in 0..=3usize {
        let all_edges: Vec<(usize, usize)> = (0..n).flat_map(|u| (0..n).map(move |v| (u, v))).collect();
        let nodes: Vec<usize> = (0..n).collect();
        for edges in subsets(&all_edges) {
            for roots in subsets(&nodes) {
                for mask in 0..1u32 << n {
                    let reclaimable: Vec<bool> = (0..n).map(|v| mask & (1 << v) != 0).collect();
                    if !protected_closed(&edges, &reclaimable) {
                        continue;
                    }
                    let base = plan(n, &edges, &roots, &reclaimable);
                    let assert_no_new_candidate = |other: &ReclaimPlan, what: &str| {
                        if other.status == PLANNED {
                            for v in 0..n {
                                assert!(!other.candidate[v] || base.candidate[v], "{what} added candidate {v}");
                            }
                        }
                    };
                    for &root in &nodes {
                        let mut more_roots = roots.clone();
                        more_roots.push(root);
                        assert_no_new_candidate(&plan(n, &edges, &more_roots, &reclaimable), "root");
                    }
                    for &edge in &all_edges {
                        let mut more_edges = edges.clone();
                        more_edges.push(edge);
                        assert_no_new_candidate(&plan(n, &more_edges, &roots, &reclaimable), "reference");
                    }
                    for v in 0..n {
                        let mut more_protected = reclaimable.clone();
                        more_protected[v] = false;
                        assert_no_new_candidate(&plan(n, &edges, &roots, &more_protected), "protection");
                    }
                }
            }
        }
    }
}

// r[verify reclaim-plan.set-invariance]
// r[verify reclaim-plan.evidence]
#[test]
fn reordering_and_duplicating_roots_and_references_keep_the_plan() {
    for n in 0..=3usize {
        let all_edges: Vec<(usize, usize)> = (0..n).flat_map(|u| (0..n).map(move |v| (u, v))).collect();
        let nodes: Vec<usize> = (0..n).collect();
        for edges in subsets(&all_edges) {
            for roots in subsets(&nodes) {
                for mask in 0..1u32 << n {
                    let reclaimable: Vec<bool> = (0..n).map(|v| mask & (1 << v) != 0).collect();
                    if !protected_closed(&edges, &reclaimable) {
                        continue;
                    }
                    let base = plan(n, &edges, &roots, &reclaimable);
                    let mut shuffled_edges: Vec<(usize, usize)> = edges.iter().rev().copied().collect();
                    shuffled_edges.extend(edges.iter().copied());
                    let mut shuffled_roots: Vec<usize> = roots.iter().rev().copied().collect();
                    shuffled_roots.extend(roots.iter().copied());
                    let other = plan(n, &shuffled_edges, &shuffled_roots, &reclaimable);
                    assert_eq!(other.status, PLANNED);
                    assert_eq!(other.retained, base.retained);
                    assert_eq!(other.candidate, base.candidate);
                    assert!(check_plan(n, &shuffled_edges, &shuffled_roots, &reclaimable, &base.retained, &base.candidate));
                }
            }
        }
    }
}

// Keys chosen so byte order differs from length order and one is a prefix of another.
const KEYS: [&[u8]; 3] = [b"/a", b"/a-b", b"/b"];

fn keyed(nodes: &[usize]) -> Vec<Vec<u8>> {
    nodes.iter().map(|&node| KEYS[node].to_vec()).collect()
}

// r[verify reclaim-plan.key-projection]
// r[verify reclaim-plan.evidence]
#[test]
fn projected_keys_plan_exactly_the_unreachable_reclaimable_keys() {
    let mut planned_count = 0u32;
    for n in 0..=3usize {
        let all_edges: Vec<(usize, usize)> = (0..n).flat_map(|u| (0..n).map(move |v| (u, v))).collect();
        let nodes: Vec<usize> = (0..n).collect();
        let keys = keyed(&nodes);
        for edges in subsets(&all_edges) {
            let references: Vec<Vec<Vec<u8>>> =
                (0..n).map(|u| edges.iter().filter(|e| e.0 == u).map(|e| KEYS[e.1].to_vec()).collect()).collect();
            let reference_index: Vec<Vec<usize>> =
                (0..n).map(|u| edges.iter().filter(|e| e.0 == u).map(|e| e.1).collect()).collect();
            for roots in subsets(&nodes) {
                let root_keys = keyed(&roots);
                let projection = project_keys(&keys, &references, &reference_index, &root_keys, &roots);
                assert!(projection.ok);
                for mask in 0..1u32 << n {
                    let reclaimable: Vec<bool> = (0..n).map(|v| mask & (1 << v) != 0).collect();
                    if !protected_closed(&projection.edges, &reclaimable) {
                        continue;
                    }
                    let result = plan(n, &projection.edges, &roots, &reclaimable);
                    assert_eq!(result.status, PLANNED);
                    assert_eq!(result.retained, oracle_live(n, &edges, &roots));
                    let retained_paths: Vec<Vec<u8>> = (0..n).filter(|&v| result.retained[v]).map(|v| KEYS[v].to_vec()).collect();
                    let retained_index: Vec<usize> = (0..n).filter(|&v| result.retained[v]).collect();
                    assert_eq!(key_mask(&keys, &retained_paths, &retained_index), Some(result.retained.clone()));
                    planned_count += 1;
                }
            }
        }
    }
    assert!(planned_count > 0);
}

// r[verify reclaim-plan.key-projection]
#[test]
fn projection_refuses_bad_mappings() {
    let keys = keyed(&[0, 1, 2]);
    let references = vec![vec![KEYS[1].to_vec()], vec![], vec![KEYS[0].to_vec()]];
    let good_index = vec![vec![1], vec![], vec![0]];
    let root_keys = keyed(&[0]);
    assert!(project_keys(&keys, &references, &good_index, &root_keys, &vec![0]).ok);
    // A reference index naming the wrong entry.
    assert!(!project_keys(&keys, &references, &vec![vec![2], vec![], vec![0]], &root_keys, &vec![0]).ok);
    // A reference index out of range, as for an unknown path.
    assert!(!project_keys(&keys, &references, &vec![vec![3], vec![], vec![0]], &root_keys, &vec![0]).ok);
    // A root index naming the wrong entry, or out of range.
    assert!(!project_keys(&keys, &references, &good_index, &root_keys, &vec![1]).ok);
    assert!(!project_keys(&keys, &references, &good_index, &root_keys, &vec![3]).ok);
    // Unsorted and duplicate keys.
    assert!(!project_keys(&keyed(&[1, 0, 2]), &references, &good_index, &root_keys, &vec![1]).ok);
    assert!(!project_keys(&keyed(&[0, 0, 2]), &references, &good_index, &root_keys, &vec![0]).ok);
    // Shape mismatches.
    assert!(!project_keys(&keys, &references, &vec![vec![1], vec![]], &root_keys, &vec![0]).ok);
    assert!(!project_keys(&keys, &references, &vec![vec![1, 0], vec![], vec![0]], &root_keys, &vec![0]).ok);
    assert!(!project_keys(&keys, &references, &good_index, &root_keys, &vec![]).ok);
    // Masks: wrong index, out of range, duplicate, and a path that is no key.
    assert_eq!(key_mask(&keys, &keyed(&[1]), &vec![1]), Some(vec![false, true, false]));
    assert_eq!(key_mask(&keys, &keyed(&[1]), &vec![2]), None);
    assert_eq!(key_mask(&keys, &keyed(&[1]), &vec![3]), None);
    assert_eq!(key_mask(&keys, &keyed(&[1, 1]), &vec![1, 1]), None);
    assert_eq!(key_mask(&keys, &vec![b"/c".to_vec()], &vec![2]), None);
    assert_eq!(key_mask(&keys, &keyed(&[1]), &vec![]), None);
}
