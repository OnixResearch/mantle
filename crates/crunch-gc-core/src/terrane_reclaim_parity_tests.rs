//! Executable parity between Mantle's `plan_gc` and the imported, unmodified
//! Terrane reclaim-plan kernel, plus the kernel's verified checker applied to
//! Mantle's own output. Parity is agreement on enumerated cases, not a theorem
//! about Mantle source; the checker's safety theorems apply to each checked
//! output individually.
//!
//! Non-claims: root and reference completeness, plan identity, byte totals,
//! the store shell's replan-before-delete and removal execution, and inputs
//! beyond the enumerated space.

extern crate std;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use super::*;

// Exact upstream bytes are refreshed only by the repository-owned importer.
// The producer's lint policy allows `&Vec` parameters; the bytes stay unchanged.
#[rustfmt::skip]
#[allow(clippy::ptr_arg)]
#[path = "../../../vendor/terrane-reclaim-plan/verified.rs"]
mod terrane;

/// Kernel inputs: node count, references, roots, and reclaimable flags.
type KernelInput = (usize, Vec<(usize, usize)>, Vec<usize>, Vec<bool>);

const NODE_IDS: [&str; 3] = ["/store/a", "/store/b", "/store/c"];
const MISSING_ID: &str = "/store/missing";

#[derive(Clone, Copy, Debug)]
enum Variant {
    Plain,
    MissingRoot,
    DanglingReference,
}

struct Case {
    request: GcPlanRequest,
    n: usize,
    edges: Vec<(usize, usize)>,
    roots: Vec<usize>,
    reclaimable: Vec<bool>,
}

/// Projects a request built from `NODE_IDS[..n]` into kernel indices. A path
/// that names no entry maps to index `n`, which the kernel rejects as out of
/// range, matching Mantle's missing-root and missing-reference classes.
fn project(request: &GcPlanRequest) -> KernelInput {
    let mut entries: Vec<&GcEntry> = request.entries.iter().collect();
    entries.sort_by(|left, right| left.path_id.cmp(&right.path_id));
    let n = entries.len();
    let index: BTreeMap<&str, usize> =
        entries.iter().enumerate().map(|(position, entry)| (entry.path_id.as_str(), position)).collect();
    let lookup = |path_id: &str| index.get(path_id).copied().unwrap_or(n);
    let mut edges = Vec::new();
    for entry in &entries {
        let owner = lookup(&entry.path_id);
        let mut references = entry.references.clone();
        references.sort();
        references.dedup();
        for reference in &references {
            edges.push((owner, lookup(reference)));
        }
    }
    let mut roots: Vec<usize> = request.roots.iter().map(|root| lookup(root)).collect();
    roots.sort_unstable();
    roots.dedup();
    let reclaimable = entries.iter().map(|entry| entry.ownership == GcOwnership::Overlay).collect();
    (n, edges, roots, reclaimable)
}

fn build_case(n: usize, edges: &[(usize, usize)], roots: &[usize], overlay_mask: u32, variant: Variant) -> Case {
    let mut entries: Vec<GcEntry> = (0..n)
        .map(|node| GcEntry {
            path_id: NODE_IDS[node].to_string(),
            references: edges
                .iter()
                .filter(|(from, _)| *from == node)
                .map(|(_, to)| NODE_IDS[*to].to_string())
                .collect(),
            declared_nar_bytes: 1,
            ownership: if overlay_mask & (1 << node) != 0 {
                GcOwnership::Overlay
            } else {
                GcOwnership::Base { layer_index: 0 }
            },
        })
        .collect();
    let mut root_ids: Vec<String> = roots.iter().map(|root| NODE_IDS[*root].to_string()).collect();
    match variant {
        Variant::Plain => {}
        Variant::MissingRoot => root_ids.push(MISSING_ID.to_string()),
        Variant::DanglingReference => {
            if let Some(first) = entries.first_mut() {
                first.references.push(MISSING_ID.to_string());
            }
        }
    }
    let request = GcPlanRequest {
        roots: root_ids,
        entries,
        execution_mode: GcExecutionMode::DryRun,
    };
    let (n, edges, roots, reclaimable) = project(&request);
    Case {
        request,
        n,
        edges,
        roots,
        reclaimable,
    }
}

fn subsets<T: Copy>(items: &[T]) -> Vec<Vec<T>> {
    (0..1u32 << items.len())
        .map(|mask| items.iter().enumerate().filter(|(i, _)| mask & (1 << i) != 0).map(|(_, &item)| item).collect())
        .collect()
}

fn path_ids_where(n: usize, flags: &[bool]) -> Vec<String> {
    (0..n).filter(|&node| flags[node]).map(|node| NODE_IDS[node].to_string()).collect()
}

fn indices_of(path_ids: &[String], n: usize) -> Vec<bool> {
    (0..n).map(|node| path_ids.iter().any(|path_id| path_id == NODE_IDS[node])).collect()
}

/// For each node, the roots whose single-root kernel plan retains it.
fn kernel_retaining_roots(case: &Case) -> Vec<(String, Vec<String>)> {
    let mut by_node: Vec<Vec<String>> = vec![Vec::new(); case.n];
    for &root in &case.roots {
        let single = terrane::plan(case.n, &case.edges, &vec![root], &case.reclaimable);
        assert_eq!(single.status, terrane::PLANNED, "single-root plan");
        for (node, roots) in by_node.iter_mut().enumerate() {
            if single.retained[node] {
                roots.push(NODE_IDS[root].to_string());
            }
        }
    }
    by_node
        .into_iter()
        .enumerate()
        .filter(|(_, roots)| !roots.is_empty())
        .map(|(node, roots)| (NODE_IDS[node].to_string(), roots))
        .collect()
}

fn kernel_violations(case: &Case) -> (bool, bool, bool) {
    let root_out = case.roots.iter().any(|&root| root >= case.n);
    let edge_out = case.edges.iter().any(|&(from, to)| from >= case.n || to >= case.n);
    let protected = case
        .edges
        .iter()
        .any(|&(from, to)| from < case.n && to < case.n && !case.reclaimable[from] && case.reclaimable[to]);
    (root_out, edge_out, protected)
}

fn compare(case: &Case, context: &str) -> bool {
    let mantle = plan_gc(case.request.clone());
    let kernel = terrane::plan(case.n, &case.edges, &case.roots, &case.reclaimable);
    let (root_out, edge_out, protected) = kernel_violations(case);
    match mantle {
        Ok(plan) => {
            assert_eq!(kernel.status, terrane::PLANNED, "Mantle planned, kernel rejected: {context}");
            assert_eq!(plan.retained_path_ids, path_ids_where(case.n, &kernel.retained), "retained: {context}");
            assert_eq!(plan.candidate_path_ids, path_ids_where(case.n, &kernel.candidate), "candidates: {context}");
            let intents: Vec<String> = plan.mutation_intents.iter().map(|intent| intent.path_id.clone()).collect();
            assert_eq!(intents, plan.candidate_path_ids, "intents: {context}");
            let retaining: Vec<(String, Vec<String>)> =
                plan.retaining_roots.iter().map(|entry| (entry.path_id.clone(), entry.root_ids.clone())).collect();
            assert_eq!(retaining, kernel_retaining_roots(case), "retaining roots: {context}");
            let protected_ids: Vec<bool> = case.reclaimable.iter().map(|reclaimable| !reclaimable).collect();
            assert_eq!(plan.base_path_ids, path_ids_where(case.n, &protected_ids), "base: {context}");
            // The verified checker accepts Mantle's own output, so the kernel's
            // no-live-candidate and no-dangling-reference theorems apply to it.
            let retained = indices_of(&plan.retained_path_ids, case.n);
            let candidate = indices_of(&plan.candidate_path_ids, case.n);
            assert!(
                terrane::check_plan(case.n, &case.edges, &case.roots, &case.reclaimable, &retained, &candidate),
                "checker rejected Mantle's plan: {context}"
            );
            true
        }
        Err(error) => {
            assert_ne!(kernel.status, terrane::PLANNED, "kernel planned, Mantle rejected with {error:?}: {context}");
            match error {
                GcPlanError::MissingRoot { .. } => assert!(root_out, "{context}"),
                GcPlanError::MissingReference { .. } => assert!(edge_out, "{context}"),
                GcPlanError::BaseToOverlayReference { .. } => assert!(protected, "{context}"),
                other => panic!("unexpected Mantle rejection {other:?}: {context}"),
            }
            let classes = [root_out, edge_out, protected].iter().filter(|class| **class).count();
            if classes == 1 {
                let expected = if root_out {
                    terrane::ROOT_OUT_OF_RANGE
                } else if edge_out {
                    terrane::EDGE_OUT_OF_RANGE
                } else {
                    terrane::PROTECTED_REFERENCES_RECLAIMABLE
                };
                assert_eq!(kernel.status, expected, "rejection class: {context}");
            }
            false
        }
    }
}

// r[verify mantle.gc_plan.planning_parity]
// r[verify mantle.gc_plan.checked_output]
#[test]
fn plan_gc_matches_the_terrane_kernel_on_every_small_store() {
    let mut planned = 0u32;
    let mut rejected = 0u32;
    for n in 0..=3usize {
        let all_edges: Vec<(usize, usize)> = (0..n).flat_map(|from| (0..n).map(move |to| (from, to))).collect();
        let nodes: Vec<usize> = (0..n).collect();
        for edges in subsets(&all_edges) {
            for roots in subsets(&nodes) {
                for overlay_mask in 0..1u32 << n {
                    for variant in [Variant::Plain, Variant::MissingRoot, Variant::DanglingReference] {
                        if matches!(variant, Variant::DanglingReference) && n == 0 {
                            continue;
                        }
                        let case = build_case(n, &edges, &roots, overlay_mask, variant);
                        let context =
                            format!("n={n} edges={edges:?} roots={roots:?} overlay={overlay_mask:#b} {variant:?}");
                        if compare(&case, &context) {
                            planned += 1;
                        } else {
                            rejected += 1;
                        }
                    }
                }
            }
        }
    }
    let base_cases = 1 + 2 * 2 * 2 + 16 * 4 * 4 + 512 * 8 * 8;
    assert_eq!(planned + rejected, 3 * base_cases - 1);
    assert!(planned > 0 && rejected > 0);
}

// r[verify mantle.gc_plan.planning_parity]
#[test]
fn duplicate_references_and_unsorted_input_project_identically() {
    let request = GcPlanRequest {
        roots: vec![NODE_IDS[1].to_string(), NODE_IDS[1].to_string()],
        entries: vec![
            GcEntry {
                path_id: NODE_IDS[1].to_string(),
                references: vec![NODE_IDS[0].to_string(), NODE_IDS[0].to_string()],
                declared_nar_bytes: 1,
                ownership: GcOwnership::Overlay,
            },
            GcEntry {
                path_id: NODE_IDS[0].to_string(),
                references: Vec::new(),
                declared_nar_bytes: 1,
                ownership: GcOwnership::Overlay,
            },
            GcEntry {
                path_id: NODE_IDS[2].to_string(),
                references: Vec::new(),
                declared_nar_bytes: 1,
                ownership: GcOwnership::Overlay,
            },
        ],
        execution_mode: GcExecutionMode::Execute,
    };
    let (n, edges, roots, reclaimable) = project(&request);
    let case = Case {
        request,
        n,
        edges,
        roots,
        reclaimable,
    };
    assert!(compare(&case, "unsorted duplicates"));
}
