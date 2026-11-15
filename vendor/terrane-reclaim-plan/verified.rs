//! A finite, verified reclaim-planning kernel over an indexed reference graph.
//!
//! Nodes are `0..n`. `edges` are references `(owner, target)`, `roots` name
//! retained starting nodes, and `reclaimable[v]` says whether node `v` may be
//! removed at all. A plan marks every node it retains and every node it would
//! remove (a candidate).
//!
//! `check_plan` validates any plan and is proved equal to `plan_valid_spec`.
//! From validity alone, `no_live_candidate` proves no candidate is reachable
//! from a root, and `no_dangling_reference` proves no surviving node references
//! a removed one. `plan` builds a plan by fixpoint iteration and is proved to
//! return only valid plans whose retained nodes are all reachable, so every
//! unreachable reclaimable node is a candidate, and to always plan admissible
//! input.
//!
//! A planned result retains exactly the live nodes and is the least valid plan,
//! so planned results are unique. Adding roots or references, or protecting
//! more nodes, never adds a candidate, and a planned result depends only on the
//! sets of roots and references, not their order or multiplicity.
//!
//! Non-claims: that the caller's roots and edges are complete and current, the
//! mapping from store paths to indices, plan identities, byte totals, and the
//! execution of removals.

use vstd::prelude::*;
// Explicit so the kernel also compiles inside `no_std` consumers' test builds.
#[allow(unused_imports)]
use std::vec;
#[allow(unused_imports)]
use std::vec::Vec;

verus! {

pub const MAX_NODES: usize = 1_000_000;
pub const MAX_EDGES: usize = 4_000_000;
pub const MAX_ROOTS: usize = 65_536;

pub const PLANNED: u8 = 0;
pub const LENGTH_MISMATCH: u8 = 1;
pub const TOO_LARGE: u8 = 2;
pub const ROOT_OUT_OF_RANGE: u8 = 3;
pub const EDGE_OUT_OF_RANGE: u8 = 4;
pub const PROTECTED_REFERENCES_RECLAIMABLE: u8 = 5;
pub const ROUND_LIMIT: u8 = 6;

// ── Reachability ────────────────────────────────────────────────────────

pub open spec fn has_edge_spec(edges: Seq<(usize, usize)>, u: int, v: int) -> bool {
    exists|i: int| #![trigger edges[i]] 0 <= i < edges.len() && edges[i].0 as int == u && edges[i].1 as int == v
}

/// `dst` is reachable from `src` in at most `fuel` reference steps.
pub open spec fn reachable_fuel(edges: Seq<(usize, usize)>, src: int, dst: int, fuel: nat) -> bool
    decreases fuel,
{
    if src == dst {
        true
    } else if fuel == 0 {
        false
    } else {
        exists|mid: int| #![trigger has_edge_spec(edges, src, mid)]
            has_edge_spec(edges, src, mid) && reachable_fuel(edges, mid, dst, (fuel - 1) as nat)
    }
}

// r[impl reclaim-plan.reachability]
/// `v` is reachable from some root.
pub open spec fn live_spec(edges: Seq<(usize, usize)>, roots: Seq<usize>, v: int) -> bool {
    exists|r: int, fuel: nat| #![trigger roots[r], reachable_fuel(edges, roots[r] as int, v, fuel)]
        0 <= r < roots.len() && reachable_fuel(edges, roots[r] as int, v, fuel)
}

// ── Plan validity ───────────────────────────────────────────────────────

pub open spec fn roots_in_range_spec(n: int, roots: Seq<usize>) -> bool {
    forall|r: int| #![trigger roots[r]] 0 <= r < roots.len() ==> (roots[r] as int) < n
}

pub open spec fn edges_in_range_spec(n: int, edges: Seq<(usize, usize)>) -> bool {
    forall|i: int| #![trigger edges[i]] 0 <= i < edges.len() ==> (edges[i].0 as int) < n && (edges[i].1 as int) < n
}

// r[impl reclaim-plan.protected-closure]
/// No protected (non-reclaimable) node references a reclaimable node.
pub open spec fn protected_closed_spec(edges: Seq<(usize, usize)>, reclaimable: Seq<bool>) -> bool {
    forall|i: int| #![trigger edges[i]] 0 <= i < edges.len() ==>
        (!reclaimable[edges[i].0 as int] ==> !reclaimable[edges[i].1 as int])
}

// r[impl reclaim-plan.admission]
/// The inputs a plan may be built for.
pub open spec fn admissible_spec(n: int, edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>) -> bool {
    reclaimable.len() == n && n <= MAX_NODES && edges.len() <= MAX_EDGES && roots.len() <= MAX_ROOTS
        && roots_in_range_spec(n, roots) && edges_in_range_spec(n, edges)
        && protected_closed_spec(edges, reclaimable)
}

pub open spec fn roots_retained_spec(roots: Seq<usize>, retained: Seq<bool>) -> bool {
    forall|r: int| #![trigger roots[r]] 0 <= r < roots.len() ==> retained[roots[r] as int]
}

pub open spec fn closed_spec(edges: Seq<(usize, usize)>, retained: Seq<bool>) -> bool {
    forall|i: int| #![trigger edges[i]] 0 <= i < edges.len() ==>
        (retained[edges[i].0 as int] ==> retained[edges[i].1 as int])
}

pub open spec fn candidates_exact_spec(reclaimable: Seq<bool>, retained: Seq<bool>, candidate: Seq<bool>) -> bool {
    forall|v: int| #![trigger candidate[v]] 0 <= v < candidate.len() ==>
        candidate[v] == (reclaimable[v] && !retained[v])
}

// r[impl reclaim-plan.validity]
// r[impl reclaim-plan.no-live-candidate]
// r[impl reclaim-plan.no-dangling-reference]
/// A plan is valid when the inputs are admissible, every root is retained,
/// retention is closed under references, and the candidates are exactly the
/// reclaimable nodes that are not retained.
pub open spec fn plan_valid_spec(
    n: int, edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>,
    retained: Seq<bool>, candidate: Seq<bool>,
) -> bool {
    admissible_spec(n, edges, roots, reclaimable) && retained.len() == n && candidate.len() == n
        && roots_retained_spec(roots, retained) && closed_spec(edges, retained)
        && candidates_exact_spec(reclaimable, retained, candidate)
}

// ── Lemmas ──────────────────────────────────────────────────────────────

/// A retained set closed under references contains everything reachable from
/// any of its members.
pub proof fn closed_set_contains_reachable(
    n: int, edges: Seq<(usize, usize)>, retained: Seq<bool>, src: int, dst: int, fuel: nat,
)
    requires
        retained.len() == n, edges_in_range_spec(n, edges), closed_spec(edges, retained),
        0 <= src < n, retained[src], reachable_fuel(edges, src, dst, fuel),
    ensures retained[dst],
    decreases fuel,
{
    if src != dst {
        let mid = choose|mid: int| #![trigger has_edge_spec(edges, src, mid)]
            has_edge_spec(edges, src, mid) && reachable_fuel(edges, mid, dst, (fuel - 1) as nat);
        let i = choose|i: int| #![trigger edges[i]]
            0 <= i < edges.len() && edges[i].0 as int == src && edges[i].1 as int == mid;
        assert(retained[edges[i].0 as int] ==> retained[edges[i].1 as int]);
        closed_set_contains_reachable(n, edges, retained, mid, dst, (fuel - 1) as nat);
    }
}

/// Appending one reference to a path reaches one step further.
pub proof fn reachable_extend(edges: Seq<(usize, usize)>, src: int, u: int, v: int, fuel: nat)
    requires reachable_fuel(edges, src, u, fuel), has_edge_spec(edges, u, v),
    ensures reachable_fuel(edges, src, v, fuel + 1),
    decreases fuel,
{
    if src == v {
    } else if src == u {
        assert(reachable_fuel(edges, v, v, 0));
        assert(has_edge_spec(edges, src, v) && reachable_fuel(edges, v, v, fuel));
    } else {
        let mid = choose|mid: int| #![trigger has_edge_spec(edges, src, mid)]
            has_edge_spec(edges, src, mid) && reachable_fuel(edges, mid, u, (fuel - 1) as nat);
        reachable_extend(edges, mid, u, v, (fuel - 1) as nat);
        assert(has_edge_spec(edges, src, mid) && reachable_fuel(edges, mid, v, fuel));
    }
}

// ── Safety theorems over any valid plan ─────────────────────────────────

// r[verify reclaim-plan.no-live-candidate]
// r[verify reclaim-plan.reachability]
/// No candidate is reachable from any root.
pub proof fn no_live_candidate(
    n: int, edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>,
    retained: Seq<bool>, candidate: Seq<bool>, v: int,
)
    requires plan_valid_spec(n, edges, roots, reclaimable, retained, candidate), 0 <= v < n, candidate[v],
    ensures !live_spec(edges, roots, v),
{
    if live_spec(edges, roots, v) {
        let (r, fuel) = choose|r: int, fuel: nat| #![trigger roots[r], reachable_fuel(edges, roots[r] as int, v, fuel)]
            0 <= r < roots.len() && reachable_fuel(edges, roots[r] as int, v, fuel);
        assert(retained[roots[r] as int]);
        closed_set_contains_reachable(n, edges, retained, roots[r] as int, v, fuel);
        assert(candidate[v] == (reclaimable[v] && !retained[v]));
    }
}

// r[verify reclaim-plan.no-dangling-reference]
/// Removing every candidate leaves no surviving node referencing a removed one.
pub proof fn no_dangling_reference(
    n: int, edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>,
    retained: Seq<bool>, candidate: Seq<bool>, i: int,
)
    requires plan_valid_spec(n, edges, roots, reclaimable, retained, candidate), 0 <= i < edges.len(),
        !candidate[edges[i].0 as int],
    ensures !candidate[edges[i].1 as int],
{
    let u = edges[i].0 as int;
    let v = edges[i].1 as int;
    assert(candidate[u] == (reclaimable[u] && !retained[u]));
    assert(candidate[v] == (reclaimable[v] && !retained[v]));
    assert(retained[u] ==> retained[v]);
    assert(!reclaimable[u] ==> !reclaimable[v]);
}

// r[verify reclaim-plan.protected-closure]
/// A protected node is never a candidate.
pub proof fn protected_never_candidate(
    n: int, edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>,
    retained: Seq<bool>, candidate: Seq<bool>, v: int,
)
    requires plan_valid_spec(n, edges, roots, reclaimable, retained, candidate), 0 <= v < n, !reclaimable[v],
    ensures !candidate[v],
{
    assert(candidate[v] == (reclaimable[v] && !retained[v]));
}

// ── Checker ─────────────────────────────────────────────────────────────

fn roots_in_range(n: usize, roots: &Vec<usize>) -> (ok: bool)
    ensures ok == roots_in_range_spec(n as int, roots@),
{
    let mut r: usize = 0;
    while r < roots.len()
        invariant 0 <= r <= roots.len(), forall|j: int| #![trigger roots@[j]] 0 <= j < r ==> (roots@[j] as int) < n as int,
        decreases roots.len() - r,
    {
        if roots[r] >= n {
            return false;
        }
        r += 1;
    }
    true
}

fn edges_in_range(n: usize, edges: &Vec<(usize, usize)>) -> (ok: bool)
    ensures ok == edges_in_range_spec(n as int, edges@),
{
    let mut i: usize = 0;
    while i < edges.len()
        invariant 0 <= i <= edges.len(),
            forall|j: int| #![trigger edges@[j]] 0 <= j < i ==> (edges@[j].0 as int) < n as int && (edges@[j].1 as int) < n as int,
        decreases edges.len() - i,
    {
        if edges[i].0 >= n || edges[i].1 >= n {
            return false;
        }
        i += 1;
    }
    true
}

fn protected_closed(edges: &Vec<(usize, usize)>, reclaimable: &Vec<bool>) -> (ok: bool)
    requires edges_in_range_spec(reclaimable@.len() as int, edges@),
    ensures ok == protected_closed_spec(edges@, reclaimable@),
{
    let mut i: usize = 0;
    while i < edges.len()
        invariant 0 <= i <= edges.len(), edges_in_range_spec(reclaimable@.len() as int, edges@),
            forall|j: int| #![trigger edges@[j]] 0 <= j < i ==>
                (!reclaimable@[edges@[j].0 as int] ==> !reclaimable@[edges@[j].1 as int]),
        decreases edges.len() - i,
    {
        let (u, v) = edges[i];
        if !reclaimable[u] && reclaimable[v] {
            return false;
        }
        i += 1;
    }
    true
}

// r[impl reclaim-plan.admission]
pub fn admissible(n: usize, edges: &Vec<(usize, usize)>, roots: &Vec<usize>, reclaimable: &Vec<bool>) -> (ok: bool)
    ensures ok == admissible_spec(n as int, edges@, roots@, reclaimable@),
{
    if reclaimable.len() != n || n > MAX_NODES || edges.len() > MAX_EDGES || roots.len() > MAX_ROOTS {
        return false;
    }
    if !roots_in_range(n, roots) || !edges_in_range(n, edges) {
        return false;
    }
    protected_closed(edges, reclaimable)
}

// r[impl reclaim-plan.validity]
/// Validates any plan against its inputs.
pub fn check_plan(
    n: usize, edges: &Vec<(usize, usize)>, roots: &Vec<usize>, reclaimable: &Vec<bool>,
    retained: &Vec<bool>, candidate: &Vec<bool>,
) -> (valid: bool)
    ensures valid == plan_valid_spec(n as int, edges@, roots@, reclaimable@, retained@, candidate@),
{
    if !admissible(n, edges, roots, reclaimable) || retained.len() != n || candidate.len() != n {
        return false;
    }
    let mut r: usize = 0;
    while r < roots.len()
        invariant 0 <= r <= roots.len(), roots_in_range_spec(n as int, roots@), retained@.len() == n,
            forall|j: int| #![trigger roots@[j]] 0 <= j < r ==> retained@[roots@[j] as int],
        decreases roots.len() - r,
    {
        if !retained[roots[r]] {
            return false;
        }
        r += 1;
    }
    let mut i: usize = 0;
    while i < edges.len()
        invariant 0 <= i <= edges.len(), edges_in_range_spec(n as int, edges@), retained@.len() == n,
            forall|j: int| #![trigger edges@[j]] 0 <= j < i ==>
                (retained@[edges@[j].0 as int] ==> retained@[edges@[j].1 as int]),
        decreases edges.len() - i,
    {
        let (u, v) = edges[i];
        if retained[u] && !retained[v] {
            return false;
        }
        i += 1;
    }
    let mut v: usize = 0;
    while v < n
        invariant 0 <= v <= n, candidate@.len() == n, retained@.len() == n, reclaimable@.len() == n,
            forall|k: int| #![trigger candidate@[k]] 0 <= k < v ==> candidate@[k] == (reclaimable@[k] && !retained@[k]),
        decreases n - v,
    {
        if candidate[v] != (reclaimable[v] && !retained[v]) {
            return false;
        }
        v += 1;
    }
    true
}

// ── Counting ────────────────────────────────────────────────────────────

/// Number of retained nodes.
pub open spec fn count_true(s: Seq<bool>) -> nat
    decreases s.len(),
{
    if s.len() == 0 {
        0
    } else {
        (if s[s.len() - 1] { 1nat } else { 0nat }) + count_true(s.subrange(0, s.len() - 1))
    }
}

pub proof fn count_true_bounded(s: Seq<bool>)
    ensures count_true(s) <= s.len(),
    decreases s.len(),
{
    if s.len() > 0 {
        count_true_bounded(s.subrange(0, s.len() - 1));
    }
}

/// Retaining one more node raises the count by exactly one.
pub proof fn count_true_set(s: Seq<bool>, i: int)
    requires 0 <= i < s.len(), !s[i],
    ensures count_true(s.update(i, true)) == count_true(s) + 1,
    decreases s.len(),
{
    let t = s.update(i, true);
    let last = s.len() - 1;
    if i == last {
        assert(t.subrange(0, last) =~= s.subrange(0, last));
    } else {
        assert(t.subrange(0, last) =~= s.subrange(0, last).update(i, true));
        count_true_set(s.subrange(0, last), i);
    }
}

// ── Planner ─────────────────────────────────────────────────────────────

pub struct ReclaimPlan {
    pub status: u8,
    pub retained: Vec<bool>,
    pub candidate: Vec<bool>,
}

// r[impl reclaim-plan.no-leak]
/// Every retained node is reachable from a root.
// r[impl reclaim-plan.exact-retention]
// r[impl reclaim-plan.least-plan]
pub open spec fn retained_sound_spec(edges: Seq<(usize, usize)>, roots: Seq<usize>, retained: Seq<bool>) -> bool {
    forall|v: int| #![trigger retained[v]] 0 <= v < retained.len() && retained[v] ==> live_spec(edges, roots, v)
}

fn first_rejection(n: usize, edges: &Vec<(usize, usize)>, roots: &Vec<usize>, reclaimable: &Vec<bool>) -> (status: u8)
    ensures
        (status == PLANNED) == admissible_spec(n as int, edges@, roots@, reclaimable@),
        status == LENGTH_MISMATCH ==> reclaimable@.len() != n,
        status == TOO_LARGE ==> (n > MAX_NODES || edges@.len() > MAX_EDGES || roots@.len() > MAX_ROOTS),
        status == ROOT_OUT_OF_RANGE ==> !roots_in_range_spec(n as int, roots@),
        status == EDGE_OUT_OF_RANGE ==> !edges_in_range_spec(n as int, edges@),
        status == PROTECTED_REFERENCES_RECLAIMABLE ==> !protected_closed_spec(edges@, reclaimable@),
        status <= PROTECTED_REFERENCES_RECLAIMABLE,
{
    if reclaimable.len() != n {
        return LENGTH_MISMATCH;
    }
    if n > MAX_NODES || edges.len() > MAX_EDGES || roots.len() > MAX_ROOTS {
        return TOO_LARGE;
    }
    if !roots_in_range(n, roots) {
        return ROOT_OUT_OF_RANGE;
    }
    if !edges_in_range(n, edges) {
        return EDGE_OUT_OF_RANGE;
    }
    if !protected_closed(edges, reclaimable) {
        return PROTECTED_REFERENCES_RECLAIMABLE;
    }
    PLANNED
}

fn empty_plan(status: u8) -> (plan: ReclaimPlan)
    ensures plan.status == status, plan.retained@.len() == 0, plan.candidate@.len() == 0,
{
    ReclaimPlan { status, retained: Vec::new(), candidate: Vec::new() }
}

// r[impl reclaim-plan.planner]
/// Builds a plan by fixpoint iteration. A `PLANNED` result is valid and sound;
/// any other status names the rejected input class and carries no plan.
pub fn plan(n: usize, edges: &Vec<(usize, usize)>, roots: &Vec<usize>, reclaimable: &Vec<bool>) -> (result: ReclaimPlan)
    ensures
        result.status == PLANNED ==> plan_valid_spec(n as int, edges@, roots@, reclaimable@, result.retained@, result.candidate@)
            && retained_sound_spec(edges@, roots@, result.retained@),
        result.status != PLANNED ==> result.retained@.len() == 0 && result.candidate@.len() == 0,
        !admissible_spec(n as int, edges@, roots@, reclaimable@) ==> result.status != PLANNED,
        admissible_spec(n as int, edges@, roots@, reclaimable@) ==> result.status == PLANNED,
{
    let status = first_rejection(n, edges, roots, reclaimable);
    if status != PLANNED {
        return empty_plan(status);
    }
    let ghost e = edges@;
    let ghost rs = roots@;

    let mut retained: Vec<bool> = Vec::new();
    let mut k: usize = 0;
    while k < n
        invariant 0 <= k <= n, retained@.len() == k, forall|j: int| #![trigger retained@[j]] 0 <= j < k ==> !retained@[j],
        decreases n - k,
    {
        retained.push(false);
        k += 1;
    }

    let mut r: usize = 0;
    while r < roots.len()
        invariant 0 <= r <= roots.len(), retained@.len() == n, roots_in_range_spec(n as int, rs), rs == roots@, e == edges@,
            forall|j: int| #![trigger rs[j]] 0 <= j < r ==> retained@[rs[j] as int],
            retained_sound_spec(e, rs, retained@),
        decreases roots.len() - r,
    {
        let root = roots[r];
        proof {
            assert(reachable_fuel(e, rs[r as int] as int, root as int, 0));
            assert(live_spec(e, rs, root as int));
        }
        retained.set(root, true);
        r += 1;
    }

    let mut rounds: usize = 0;
    let mut changed = true;
    while changed && rounds <= n
        invariant
            retained@.len() == n, rs == roots@, e == edges@, n <= MAX_NODES,
            edges_in_range_spec(n as int, e), roots_in_range_spec(n as int, rs),
            roots_retained_spec(rs, retained@),
            retained_sound_spec(e, rs, retained@),
            !changed ==> closed_spec(e, retained@),
            changed ==> count_true(retained@) >= rounds,
            rounds <= n + 1,
        decreases n + 1 - rounds,
    {
        changed = false;
        let mut i: usize = 0;
        let ghost start = retained@;
        let ghost start_count = count_true(retained@);
        while i < edges.len()
            invariant
                0 <= i <= edges.len(), retained@.len() == n, rs == roots@, e == edges@,
                edges_in_range_spec(n as int, e), roots_in_range_spec(n as int, rs),
                roots_retained_spec(rs, retained@),
                retained_sound_spec(e, rs, retained@),
                !changed ==> retained@ == start,
                !changed ==> forall|j: int| #![trigger e[j]] 0 <= j < i ==>
                    (retained@[e[j].0 as int] ==> retained@[e[j].1 as int]),
                start_count == count_true(start),
                count_true(retained@) >= start_count,
                changed ==> count_true(retained@) >= start_count + 1,
            decreases edges.len() - i,
        {
            let (u, v) = edges[i];
            if retained[u] && !retained[v] {
                proof {
                    assert(e[i as int] == (u, v));
                    assert(has_edge_spec(e, u as int, v as int));
                    assert(live_spec(e, rs, u as int));
                    let (ro, fuel) = choose|ro: int, fuel: nat| #![trigger rs[ro], reachable_fuel(e, rs[ro] as int, u as int, fuel)]
                        0 <= ro < rs.len() && reachable_fuel(e, rs[ro] as int, u as int, fuel);
                    reachable_extend(e, rs[ro] as int, u as int, v as int, fuel);
                    assert(reachable_fuel(e, rs[ro] as int, v as int, fuel + 1));
                    assert(live_spec(e, rs, v as int));
                }
                let ghost before = retained@;
                retained.set(v, true);
                proof {
                    count_true_set(before, v as int);
                    assert(retained@ == before.update(v as int, true));
                    assert forall|j: int| #![trigger rs[j]] 0 <= j < rs.len() implies retained@[rs[j] as int] by {
                        assert(before[rs[j] as int]);
                    }
                }
                changed = true;
            }
            i += 1;
        }
        rounds += 1;
    }
    proof {
        count_true_bounded(retained@);
        assert(!changed);
    }
    if changed {
        return empty_plan(ROUND_LIMIT);
    }

    let mut candidate: Vec<bool> = Vec::new();
    let mut v: usize = 0;
    while v < n
        invariant 0 <= v <= n, candidate@.len() == v, retained@.len() == n, reclaimable@.len() == n,
            forall|k: int| #![trigger candidate@[k]] 0 <= k < v ==> candidate@[k] == (reclaimable@[k] && !retained@[k]),
        decreases n - v,
    {
        candidate.push(reclaimable[v] && !retained[v]);
        v += 1;
    }
    ReclaimPlan { status: PLANNED, retained, candidate }
}

// r[verify reclaim-plan.no-leak]
/// A planned result never leaks: every reclaimable node that no root reaches
/// is a candidate.
pub proof fn planned_result_reclaims_every_unreachable_node(
    n: int, edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>,
    retained: Seq<bool>, candidate: Seq<bool>, v: int,
)
    requires plan_valid_spec(n, edges, roots, reclaimable, retained, candidate),
        retained_sound_spec(edges, roots, retained),
        0 <= v < n, reclaimable[v], !live_spec(edges, roots, v),
    ensures candidate[v],
{
    assert(retained[v] ==> live_spec(edges, roots, v));
    assert(candidate[v] == (reclaimable[v] && !retained[v]));
}

// ── Optimality, determinism, monotonicity, and set invariance ───────────

// r[impl reclaim-plan.monotone]
// r[impl reclaim-plan.set-invariance]
/// Every reference in `a` is also a reference in `b`.
pub open spec fn edges_subset_spec(a: Seq<(usize, usize)>, b: Seq<(usize, usize)>) -> bool {
    forall|i: int| #![trigger a[i]] 0 <= i < a.len() ==> has_edge_spec(b, a[i].0 as int, a[i].1 as int)
}

// r[impl reclaim-plan.monotone]
// r[impl reclaim-plan.set-invariance]
/// Every root in `a` is also a root in `b`.
pub open spec fn roots_subset_spec(a: Seq<usize>, b: Seq<usize>) -> bool {
    forall|r: int| #![trigger a[r]] 0 <= r < a.len() ==> exists|s: int| #![trigger b[s]] 0 <= s < b.len() && b[s] == a[r]
}

// r[impl reclaim-plan.monotone]
/// Every node `b` may reclaim, `a` may also reclaim.
pub open spec fn reclaimable_subset_spec(b: Seq<bool>, a: Seq<bool>) -> bool {
    b.len() == a.len() && forall|v: int| #![trigger b[v]] 0 <= v < b.len() && b[v] ==> a[v]
}

/// More references never shorten what is reachable.
pub proof fn reachable_edges_mono(
    edges: Seq<(usize, usize)>, edges2: Seq<(usize, usize)>, src: int, dst: int, fuel: nat,
)
    requires edges_subset_spec(edges, edges2), reachable_fuel(edges, src, dst, fuel),
    ensures reachable_fuel(edges2, src, dst, fuel),
    decreases fuel,
{
    if src != dst {
        let mid = choose|mid: int| #![trigger has_edge_spec(edges, src, mid)]
            has_edge_spec(edges, src, mid) && reachable_fuel(edges, mid, dst, (fuel - 1) as nat);
        let i = choose|i: int| #![trigger edges[i]]
            0 <= i < edges.len() && edges[i].0 as int == src && edges[i].1 as int == mid;
        assert(has_edge_spec(edges2, edges[i].0 as int, edges[i].1 as int));
        reachable_edges_mono(edges, edges2, mid, dst, (fuel - 1) as nat);
        assert(has_edge_spec(edges2, src, mid) && reachable_fuel(edges2, mid, dst, (fuel - 1) as nat));
    }
}

/// More roots and more references never make a live node dead.
pub proof fn live_mono(
    edges: Seq<(usize, usize)>, edges2: Seq<(usize, usize)>, roots: Seq<usize>, roots2: Seq<usize>, v: int,
)
    requires edges_subset_spec(edges, edges2), roots_subset_spec(roots, roots2), live_spec(edges, roots, v),
    ensures live_spec(edges2, roots2, v),
{
    let (r, fuel) = choose|r: int, fuel: nat| #![trigger roots[r], reachable_fuel(edges, roots[r] as int, v, fuel)]
        0 <= r < roots.len() && reachable_fuel(edges, roots[r] as int, v, fuel);
    let s = choose|s: int| #![trigger roots2[s]] 0 <= s < roots2.len() && roots2[s] == roots[r];
    reachable_edges_mono(edges, edges2, roots[r] as int, v, fuel);
    assert(0 <= s < roots2.len() && reachable_fuel(edges2, roots2[s] as int, v, fuel));
}

/// Any valid plan retains every live node.
pub proof fn valid_plan_retains_live(
    n: int, edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>,
    retained: Seq<bool>, candidate: Seq<bool>, v: int,
)
    requires plan_valid_spec(n, edges, roots, reclaimable, retained, candidate), live_spec(edges, roots, v),
    ensures retained[v],
{
    let (r, fuel) = choose|r: int, fuel: nat| #![trigger roots[r], reachable_fuel(edges, roots[r] as int, v, fuel)]
        0 <= r < roots.len() && reachable_fuel(edges, roots[r] as int, v, fuel);
    assert(retained[roots[r] as int]);
    closed_set_contains_reachable(n, edges, retained, roots[r] as int, v, fuel);
}

// r[verify reclaim-plan.exact-retention]
/// A planned result retains exactly the live nodes.
pub proof fn planned_result_retains_exactly_live(
    n: int, edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>,
    retained: Seq<bool>, candidate: Seq<bool>, v: int,
)
    requires plan_valid_spec(n, edges, roots, reclaimable, retained, candidate),
        retained_sound_spec(edges, roots, retained), 0 <= v < n,
    ensures retained[v] == live_spec(edges, roots, v),
        candidate[v] == (reclaimable[v] && !live_spec(edges, roots, v)),
{
    if live_spec(edges, roots, v) {
        valid_plan_retains_live(n, edges, roots, reclaimable, retained, candidate, v);
    }
    assert(retained[v] ==> live_spec(edges, roots, v));
    assert(candidate[v] == (reclaimable[v] && !retained[v]));
}

// r[verify reclaim-plan.least-plan]
/// A planned result is the least valid plan: it retains no more, and
/// reclaims no less, than any other valid plan for the same input.
pub proof fn planned_result_is_least_valid_plan(
    n: int, edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>,
    retained: Seq<bool>, candidate: Seq<bool>, other_retained: Seq<bool>, other_candidate: Seq<bool>, v: int,
)
    requires plan_valid_spec(n, edges, roots, reclaimable, retained, candidate),
        retained_sound_spec(edges, roots, retained),
        plan_valid_spec(n, edges, roots, reclaimable, other_retained, other_candidate), 0 <= v < n,
    ensures retained[v] ==> other_retained[v], other_candidate[v] ==> candidate[v],
{
    if retained[v] {
        valid_plan_retains_live(n, edges, roots, reclaimable, other_retained, other_candidate, v);
    }
    assert(candidate[v] == (reclaimable[v] && !retained[v]));
    assert(other_candidate[v] == (reclaimable[v] && !other_retained[v]));
}

// r[verify reclaim-plan.least-plan]
/// Planned results are determined by their input: any two are equal.
pub proof fn planned_results_are_unique(
    n: int, edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>,
    retained: Seq<bool>, candidate: Seq<bool>, other_retained: Seq<bool>, other_candidate: Seq<bool>,
)
    requires plan_valid_spec(n, edges, roots, reclaimable, retained, candidate),
        retained_sound_spec(edges, roots, retained),
        plan_valid_spec(n, edges, roots, reclaimable, other_retained, other_candidate),
        retained_sound_spec(edges, roots, other_retained),
    ensures retained =~= other_retained, candidate =~= other_candidate,
{
    assert forall|v: int| 0 <= v < n implies retained[v] == other_retained[v] && candidate[v] == other_candidate[v] by {
        planned_result_is_least_valid_plan(n, edges, roots, reclaimable, retained, candidate,
            other_retained, other_candidate, v);
        planned_result_is_least_valid_plan(n, edges, roots, reclaimable, other_retained, other_candidate,
            retained, candidate, v);
    }
}

// r[verify reclaim-plan.monotone]
/// Adding roots or references, or protecting more nodes, never adds a
/// candidate: every candidate of a valid plan for the larger input is a
/// candidate of the planned result for the smaller one.
pub proof fn stronger_input_never_adds_candidates(
    n: int, edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>,
    retained: Seq<bool>, candidate: Seq<bool>,
    edges2: Seq<(usize, usize)>, roots2: Seq<usize>, reclaimable2: Seq<bool>,
    retained2: Seq<bool>, candidate2: Seq<bool>, v: int,
)
    requires plan_valid_spec(n, edges, roots, reclaimable, retained, candidate),
        retained_sound_spec(edges, roots, retained),
        plan_valid_spec(n, edges2, roots2, reclaimable2, retained2, candidate2),
        edges_subset_spec(edges, edges2), roots_subset_spec(roots, roots2),
        reclaimable_subset_spec(reclaimable2, reclaimable),
        0 <= v < n, candidate2[v],
    ensures candidate[v],
{
    assert(candidate2[v] == (reclaimable2[v] && !retained2[v]));
    if retained[v] {
        live_mono(edges, edges2, roots, roots2, v);
        valid_plan_retains_live(n, edges2, roots2, reclaimable2, retained2, candidate2, v);
    }
    assert(reclaimable[v]);
    assert(candidate[v] == (reclaimable[v] && !retained[v]));
}

// r[verify reclaim-plan.set-invariance]
/// A planned result depends only on the sets of roots and references:
/// reordering or deduplicating either, within the bounds, keeps it valid and
/// sound.
pub proof fn planned_result_depends_only_on_sets(
    n: int, edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>,
    retained: Seq<bool>, candidate: Seq<bool>, edges2: Seq<(usize, usize)>, roots2: Seq<usize>,
)
    requires plan_valid_spec(n, edges, roots, reclaimable, retained, candidate),
        retained_sound_spec(edges, roots, retained),
        edges_subset_spec(edges, edges2), edges_subset_spec(edges2, edges),
        roots_subset_spec(roots, roots2), roots_subset_spec(roots2, roots),
        edges2.len() <= MAX_EDGES, roots2.len() <= MAX_ROOTS,
    ensures plan_valid_spec(n, edges2, roots2, reclaimable, retained, candidate),
        retained_sound_spec(edges2, roots2, retained),
{
    assert forall|s: int| #![trigger roots2[s]] 0 <= s < roots2.len()
        implies (roots2[s] as int) < n && retained[roots2[s] as int] by {
        let r = choose|r: int| #![trigger roots[r]] 0 <= r < roots.len() && roots[r] == roots2[s];
        assert(retained[roots[r] as int]);
    }
    assert forall|j: int| #![trigger edges2[j]] 0 <= j < edges2.len() implies
        (edges2[j].0 as int) < n && (edges2[j].1 as int) < n
        && (!reclaimable[edges2[j].0 as int] ==> !reclaimable[edges2[j].1 as int])
        && (retained[edges2[j].0 as int] ==> retained[edges2[j].1 as int]) by {
        assert(has_edge_spec(edges, edges2[j].0 as int, edges2[j].1 as int));
        let i = choose|i: int| #![trigger edges[i]]
            0 <= i < edges.len() && edges[i].0 as int == edges2[j].0 as int && edges[i].1 as int == edges2[j].1 as int;
        assert(edges[i].0 == edges2[j].0 && edges[i].1 == edges2[j].1);
    }
    assert forall|v: int| #![trigger retained[v]] 0 <= v < retained.len() && retained[v]
        implies live_spec(edges2, roots2, v) by {
        live_mono(edges, edges2, roots, roots2, v);
    }
}

} // verus!

#[cfg(test)]
#[path = "tests/verified_positive.rs"]
mod verified_positive;
#[cfg(test)]
#[path = "tests/verified_negative.rs"]
mod verified_negative;
#[cfg(test)]
#[path = "tests/verified_exhaustive.rs"]
mod verified_exhaustive;
