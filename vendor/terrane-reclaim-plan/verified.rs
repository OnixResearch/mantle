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
//! `project_keys` validates a consumer's mapping from byte-string keys to
//! indices instead of trusting it, and `key_mask` validates its plan path
//! lists, so the safety theorems also hold over keys: no listed candidate path
//! is reachable from a root path, and every reachable path is listed retained.
//!
//! Non-claims: that the caller's roots and references are complete and
//! current, that the caller passes each entry's key, references, and flag
//! together, plan identities, byte totals, and the execution of removals.

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
        let r = choose|r: int| #![trigger roots[r]] 0 <= r < roots.len()
            && exists|fuel: nat| #![trigger reachable_fuel(edges, roots[r] as int, v, fuel)] reachable_fuel(edges, roots[r] as int, v, fuel);
        let fuel = choose|fuel: nat| #![trigger reachable_fuel(edges, roots[r] as int, v, fuel)] reachable_fuel(edges, roots[r] as int, v, fuel);
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
/// The number of `true` entries among the first `k` of `s`.
pub open spec fn count_true_upto(s: Seq<bool>, k: int) -> nat
    decreases k,
{
    if k <= 0 {
        0
    } else {
        (if s[k - 1] { 1nat } else { 0nat }) + count_true_upto(s, k - 1)
    }
}

pub open spec fn count_true(s: Seq<bool>) -> nat {
    count_true_upto(s, s.len() as int)
}

proof fn count_true_upto_bounded(s: Seq<bool>, k: int)
    requires 0 <= k <= s.len(),
    ensures count_true_upto(s, k) <= k,
    decreases k,
{
    if k > 0 {
        count_true_upto_bounded(s, k - 1);
    }
}

pub proof fn count_true_bounded(s: Seq<bool>)
    ensures count_true(s) <= s.len(),
{
    count_true_upto_bounded(s, s.len() as int);
}

proof fn count_true_upto_set(s: Seq<bool>, i: int, k: int)
    requires 0 <= i < s.len(), !s[i], 0 <= k <= s.len(),
    ensures count_true_upto(s.update(i, true), k) == count_true_upto(s, k) + (if i < k { 1nat } else { 0nat }),
    decreases k,
{
    if k > 0 {
        count_true_upto_set(s, i, k - 1);
    }
}

/// Retaining one more node raises the count by exactly one.
pub proof fn count_true_set(s: Seq<bool>, i: int)
    requires 0 <= i < s.len(), !s[i],
    ensures count_true(s.update(i, true)) == count_true(s) + 1,
{
    count_true_upto_set(s, i, s.len() as int);
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
                    let ro = choose|ro: int| #![trigger rs[ro]] 0 <= ro < rs.len()
                        && exists|fuel: nat| #![trigger reachable_fuel(e, rs[ro] as int, u as int, fuel)] reachable_fuel(e, rs[ro] as int, u as int, fuel);
                    let fuel = choose|fuel: nat| #![trigger reachable_fuel(e, rs[ro] as int, u as int, fuel)] reachable_fuel(e, rs[ro] as int, u as int, fuel);
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
    let r = choose|r: int| #![trigger roots[r]] 0 <= r < roots.len()
        && exists|fuel: nat| #![trigger reachable_fuel(edges, roots[r] as int, v, fuel)] reachable_fuel(edges, roots[r] as int, v, fuel);
    let fuel = choose|fuel: nat| #![trigger reachable_fuel(edges, roots[r] as int, v, fuel)] reachable_fuel(edges, roots[r] as int, v, fuel);
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
    let r = choose|r: int| #![trigger roots[r]] 0 <= r < roots.len()
        && exists|fuel: nat| #![trigger reachable_fuel(edges, roots[r] as int, v, fuel)] reachable_fuel(edges, roots[r] as int, v, fuel);
    let fuel = choose|fuel: nat| #![trigger reachable_fuel(edges, roots[r] as int, v, fuel)] reachable_fuel(edges, roots[r] as int, v, fuel);
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


// ── Key projection: path identifiers to indices ─────────────────────────
//
// A consumer names nodes by byte-string keys and maps them to indices with
// its own (unverified) lookup. `project_keys` validates that mapping instead
// of trusting it: keys must be strictly sorted, so distinct, and every
// reference and root index must name an entry whose key equals the
// referenced bytes. `key_live_matches_index_live` then shows reachability
// over indices is exactly reachability over keys.

/// Keys `a` and `b` agree on their first `p` bytes.
pub open spec fn prefix_eq_spec(a: Seq<u8>, b: Seq<u8>, p: int) -> bool {
    forall|k: int| #![trigger a[k], b[k]] 0 <= k < p ==> a[k] == b[k]
}

/// `p` is where `a` first falls below `b`: either `a` ends there and `b`
/// does not, or both continue and `a`'s byte is smaller.
pub open spec fn key_lt_at_spec(a: Seq<u8>, b: Seq<u8>, p: int) -> bool {
    0 <= p <= a.len() && p <= b.len() && prefix_eq_spec(a, b, p)
        && ((p == a.len() && p < b.len()) || (p < a.len() && p < b.len() && a[p] < b[p]))
}

// r[impl reclaim-plan.key-projection]
/// Lexicographic order on byte strings.
pub open spec fn key_lt_spec(a: Seq<u8>, b: Seq<u8>) -> bool {
    exists|p: int| #![trigger key_lt_at_spec(a, b, p)] key_lt_at_spec(a, b, p)
}

pub proof fn key_lt_irreflexive(a: Seq<u8>)
    ensures !key_lt_spec(a, a),
{
}

pub proof fn key_lt_transitive(a: Seq<u8>, b: Seq<u8>, c: Seq<u8>)
    requires key_lt_spec(a, b), key_lt_spec(b, c),
    ensures key_lt_spec(a, c),
{
    let p = choose|p: int| #![trigger key_lt_at_spec(a, b, p)] key_lt_at_spec(a, b, p);
    let q = choose|q: int| #![trigger key_lt_at_spec(b, c, q)] key_lt_at_spec(b, c, q);
    if p < q {
        assert(forall|k: int| #![trigger a[k], c[k]] 0 <= k < p ==> a[k] == b[k] && b[k] == c[k]);
        if p < a.len() {
            assert(b[p] == c[p]);
        }
        assert(key_lt_at_spec(a, c, p));
    } else if q < p {
        assert(forall|k: int| #![trigger a[k], c[k]] 0 <= k < q ==> a[k] == b[k] && b[k] == c[k]);
        assert(a[q] == b[q]);
        assert(key_lt_at_spec(a, c, q));
    } else {
        assert(forall|k: int| #![trigger a[k], c[k]] 0 <= k < p ==> a[k] == b[k] && b[k] == c[k]);
        assert(key_lt_at_spec(a, c, p));
    }
}

pub proof fn key_lt_implies_ne(a: Seq<u8>, b: Seq<u8>)
    requires key_lt_spec(a, b),
    ensures a != b,
{
    if a == b {
        key_lt_irreflexive(a);
    }
}

/// Every pair of keys is in strictly increasing order, so keys are distinct.
pub open spec fn keys_sorted_spec(keys: Seq<Vec<u8>>) -> bool {
    forall|i: int, j: int| #![trigger keys[i], keys[j]] 0 <= i < j < keys.len() ==> key_lt_spec(keys[i]@, keys[j]@)
}

pub proof fn sorted_keys_distinct(keys: Seq<Vec<u8>>, i: int, j: int)
    requires keys_sorted_spec(keys), 0 <= i < keys.len(), 0 <= j < keys.len(), keys[i]@ == keys[j]@,
    ensures i == j,
{
    if i < j {
        key_lt_implies_ne(keys[i]@, keys[j]@);
    } else if j < i {
        key_lt_implies_ne(keys[j]@, keys[i]@);
    }
}

/// `b` is reachable from `a` over keys in at most `fuel` reference steps.
pub open spec fn key_reach_fuel(keys: Seq<Vec<u8>>, refs: Seq<Vec<Vec<u8>>>, a: Seq<u8>, b: Seq<u8>, fuel: nat) -> bool
    decreases fuel,
{
    if a == b {
        true
    } else if fuel == 0 {
        false
    } else {
        exists|i: int, j: int| #![trigger keys[i], refs[i][j]]
            0 <= i < keys.len() && i < refs.len() && keys[i]@ == a && 0 <= j < refs[i].len()
            && key_reach_fuel(keys, refs, refs[i][j]@, b, (fuel - 1) as nat)
    }
}

// r[impl reclaim-plan.key-projection]
/// `b` is reachable from some root key.
pub open spec fn key_live_spec(keys: Seq<Vec<u8>>, refs: Seq<Vec<Vec<u8>>>, root_keys: Seq<Vec<u8>>, b: Seq<u8>) -> bool {
    exists|r: int, fuel: nat| #![trigger root_keys[r], key_reach_fuel(keys, refs, root_keys[r]@, b, fuel)]
        0 <= r < root_keys.len() && key_reach_fuel(keys, refs, root_keys[r]@, b, fuel)
}

// r[impl reclaim-plan.key-projection]
/// The index edges are exactly the key references: each edge names a
/// reference of its owner, and each reference is an edge to the entry whose
/// key it names.
pub open spec fn edges_match_refs_spec(keys: Seq<Vec<u8>>, refs: Seq<Vec<Vec<u8>>>, edges: Seq<(usize, usize)>) -> bool {
    &&& refs.len() == keys.len()
    &&& (forall|k: int| #![trigger edges[k]] 0 <= k < edges.len() ==> edge_names_ref_spec(keys, refs, edges[k]))
    &&& (forall|i: int, j: int| #![trigger ref_resolved_spec(keys, refs, edges, i, j)]
        0 <= i < refs.len() && 0 <= j < refs[i].len() ==> ref_resolved_spec(keys, refs, edges, i, j))
}

// r[impl reclaim-plan.key-projection]
/// The index roots are exactly the root keys.
pub open spec fn roots_match_keys_spec(keys: Seq<Vec<u8>>, root_keys: Seq<Vec<u8>>, roots: Seq<usize>) -> bool {
    &&& roots.len() == root_keys.len()
    &&& forall|r: int| #![trigger roots[r]] 0 <= r < roots.len() ==>
        (roots[r] as int) < keys.len() && keys[roots[r] as int]@ == root_keys[r]@
}

pub open spec fn projection_valid_spec(
    keys: Seq<Vec<u8>>, refs: Seq<Vec<Vec<u8>>>, root_keys: Seq<Vec<u8>>,
    edges: Seq<(usize, usize)>, roots: Seq<usize>,
) -> bool {
    keys_sorted_spec(keys) && edges_match_refs_spec(keys, refs, edges) && roots_match_keys_spec(keys, root_keys, roots)
}

/// Index reachability implies key reachability.
pub proof fn index_reach_implies_key_reach(
    keys: Seq<Vec<u8>>, refs: Seq<Vec<Vec<u8>>>, edges: Seq<(usize, usize)>, s: int, t: int, fuel: nat,
)
    requires edges_match_refs_spec(keys, refs, edges), 0 <= s < keys.len(), 0 <= t < keys.len(),
        reachable_fuel(edges, s, t, fuel),
    ensures key_reach_fuel(keys, refs, keys[s]@, keys[t]@, fuel),
    decreases fuel,
{
    if s != t && keys[s]@ != keys[t]@ {
        let mid = choose|mid: int| #![trigger has_edge_spec(edges, s, mid)]
            has_edge_spec(edges, s, mid) && reachable_fuel(edges, mid, t, (fuel - 1) as nat);
        let k = choose|k: int| #![trigger edges[k]] 0 <= k < edges.len() && edges[k].0 as int == s && edges[k].1 as int == mid;
        assert(edge_names_ref_spec(keys, refs, edges[k]));
        let j = choose|j: int| #![trigger refs[edges[k].0 as int][j]]
            0 <= j < refs[edges[k].0 as int].len() && refs[edges[k].0 as int][j]@ == keys[edges[k].1 as int]@;
        index_reach_implies_key_reach(keys, refs, edges, mid, t, (fuel - 1) as nat);
        assert(refs[s][j]@ == keys[mid]@ && key_reach_fuel(keys, refs, refs[s][j]@, keys[t]@, (fuel - 1) as nat)) by {
            assert(keys[s] == keys[edges[k].0 as int] && refs[s][j] == refs[edges[k].0 as int][j]);
        }
    }
}

/// Key reachability from an entry's key reaches an entry by index.
pub proof fn key_reach_implies_index_reach(
    keys: Seq<Vec<u8>>, refs: Seq<Vec<Vec<u8>>>, edges: Seq<(usize, usize)>, s: int, b: Seq<u8>, fuel: nat,
) -> (t: int)
    requires keys_sorted_spec(keys), edges_match_refs_spec(keys, refs, edges), 0 <= s < keys.len(),
        key_reach_fuel(keys, refs, keys[s]@, b, fuel),
    ensures 0 <= t < keys.len(), keys[t]@ == b, reachable_fuel(edges, s, t, fuel),
    decreases fuel,
{
    if keys[s]@ == b {
        assert(reachable_fuel(edges, s, s, fuel));
        s
    } else {
        let i = choose|i: int| #![trigger keys[i]] 0 <= i < keys.len() && i < refs.len() && keys[i]@ == keys[s]@
            && exists|j: int| #![trigger refs[i][j]] 0 <= j < refs[i].len() && key_reach_fuel(keys, refs, refs[i][j]@, b, (fuel - 1) as nat);
        let j = choose|j: int| #![trigger refs[i][j]] 0 <= j < refs[i].len() && key_reach_fuel(keys, refs, refs[i][j]@, b, (fuel - 1) as nat);
        sorted_keys_distinct(keys, i, s);
        assert(ref_resolved_spec(keys, refs, edges, i, j));
        let m = choose|t: int| #![trigger keys[t]] 0 <= t < keys.len() && keys[t]@ == refs[i][j]@ && has_edge_spec(edges, i, t);
        let t = key_reach_implies_index_reach(keys, refs, edges, m, b, (fuel - 1) as nat);
        assert(has_edge_spec(edges, s, m) && reachable_fuel(edges, m, t, (fuel - 1) as nat));
        t
    }
}

// r[verify reclaim-plan.key-projection]
/// Under a valid projection, a node is live over indices exactly when its key
/// is live over keys.
pub proof fn key_live_matches_index_live(
    keys: Seq<Vec<u8>>, refs: Seq<Vec<Vec<u8>>>, root_keys: Seq<Vec<u8>>,
    edges: Seq<(usize, usize)>, roots: Seq<usize>, v: int,
)
    requires projection_valid_spec(keys, refs, root_keys, edges, roots), 0 <= v < keys.len(),
    ensures live_spec(edges, roots, v) == key_live_spec(keys, refs, root_keys, keys[v]@),
{
    if live_spec(edges, roots, v) {
        let r = choose|r: int| #![trigger roots[r]] 0 <= r < roots.len()
            && exists|fuel: nat| #![trigger reachable_fuel(edges, roots[r] as int, v, fuel)] reachable_fuel(edges, roots[r] as int, v, fuel);
        let fuel = choose|fuel: nat| #![trigger reachable_fuel(edges, roots[r] as int, v, fuel)] reachable_fuel(edges, roots[r] as int, v, fuel);
        index_reach_implies_key_reach(keys, refs, edges, roots[r] as int, v, fuel);
        assert(key_reach_fuel(keys, refs, root_keys[r]@, keys[v]@, fuel));
    }
    if key_live_spec(keys, refs, root_keys, keys[v]@) {
        let r = choose|r: int| #![trigger root_keys[r]] 0 <= r < root_keys.len()
            && exists|fuel: nat| #![trigger key_reach_fuel(keys, refs, root_keys[r]@, keys[v]@, fuel)] key_reach_fuel(keys, refs, root_keys[r]@, keys[v]@, fuel);
        let fuel = choose|fuel: nat| #![trigger key_reach_fuel(keys, refs, root_keys[r]@, keys[v]@, fuel)] key_reach_fuel(keys, refs, root_keys[r]@, keys[v]@, fuel);
        assert(keys[roots[r] as int]@ == root_keys[r]@);
        let t = key_reach_implies_index_reach(keys, refs, edges, roots[r] as int, keys[v]@, fuel);
        sorted_keys_distinct(keys, t, v);
        assert(0 <= r < roots.len() && reachable_fuel(edges, roots[r] as int, v, fuel));
    }
}

// r[verify reclaim-plan.key-projection]
/// End to end: under a valid projection, no candidate of a valid plan has a
/// key reachable from a root key.
pub proof fn no_live_key_is_candidate(
    keys: Seq<Vec<u8>>, refs: Seq<Vec<Vec<u8>>>, root_keys: Seq<Vec<u8>>,
    edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>,
    retained: Seq<bool>, candidate: Seq<bool>, v: int,
)
    requires projection_valid_spec(keys, refs, root_keys, edges, roots),
        plan_valid_spec(keys.len() as int, edges, roots, reclaimable, retained, candidate),
        0 <= v < keys.len(), candidate[v],
    ensures !key_live_spec(keys, refs, root_keys, keys[v]@),
{
    no_live_candidate(keys.len() as int, edges, roots, reclaimable, retained, candidate, v);
    key_live_matches_index_live(keys, refs, root_keys, edges, roots, v);
}

fn key_lt(a: &Vec<u8>, b: &Vec<u8>) -> (lt: bool)
    ensures lt == key_lt_spec(a@, b@),
{
    let mut i: usize = 0;
    let mut same = true;
    while same && i < a.len() && i < b.len()
        invariant 0 <= i <= a.len(), i <= b.len(), prefix_eq_spec(a@, b@, i as int),
            !same ==> i < a.len() && i < b.len() && a@[i as int] != b@[i as int],
        decreases a.len() - i + (if same { 1int } else { 0int }),
    {
        if a[i] == b[i] {
            i = i + 1;
        } else {
            same = false;
        }
    }
    let lt = (i == a.len() && i < b.len()) || (i < a.len() && i < b.len() && a[i] < b[i]);
    proof {
        if lt {
            assert(key_lt_at_spec(a@, b@, i as int));
        }
        if key_lt_spec(a@, b@) {
            let p = choose|p: int| #![trigger key_lt_at_spec(a@, b@, p)] key_lt_at_spec(a@, b@, p);
            if p < i as int {
                assert(a@[p] == b@[p]);
            } else if p > i as int {
                assert(a@[i as int] == b@[i as int]);
            }
        }
    }
    lt
}

fn key_eq(a: &Vec<u8>, b: &Vec<u8>) -> (eq: bool)
    ensures eq == (a@ == b@),
{
    if a.len() != b.len() {
        return false;
    }
    let mut i: usize = 0;
    while i < a.len()
        invariant 0 <= i <= a.len(), a.len() == b.len(), prefix_eq_spec(a@, b@, i as int),
        decreases a.len() - i,
    {
        if a[i] != b[i] {
            assert(a@[i as int] != b@[i as int]);
            return false;
        }
        i = i + 1;
    }
    assert(a@ =~= b@);
    true
}

fn keys_sorted(keys: &Vec<Vec<u8>>) -> (ok: bool)
    ensures ok == keys_sorted_spec(keys@),
{
    if keys.len() < 2 {
        return true;
    }
    let mut i: usize = 1;
    while i < keys.len()
        invariant 1 <= i <= keys.len(),
            forall|a: int, b: int| #![trigger keys@[a], keys@[b]] 0 <= a < b < i ==> key_lt_spec(keys@[a]@, keys@[b]@),
        decreases keys.len() - i,
    {
        if !key_lt(&keys[i - 1], &keys[i]) {
            assert(!key_lt_spec(keys@[i - 1]@, keys@[i as int]@));
            return false;
        }
        proof {
            assert forall|a: int, b: int| #![trigger keys@[a], keys@[b]] 0 <= a < b < i + 1
                implies key_lt_spec(keys@[a]@, keys@[b]@) by {
                if b == i && a < i - 1 {
                    key_lt_transitive(keys@[a]@, keys@[i - 1]@, keys@[i as int]@);
                }
            }
        }
        i = i + 1;
    }
    true
}

/// Edge `e` names a reference of its owner by the target's key.
pub open spec fn edge_names_ref_spec(keys: Seq<Vec<u8>>, refs: Seq<Vec<Vec<u8>>>, e: (usize, usize)) -> bool {
    (e.0 as int) < keys.len() && (e.1 as int) < keys.len() && (e.0 as int) < refs.len()
        && exists|q: int| #![trigger refs[e.0 as int][q]] 0 <= q < refs[e.0 as int].len() && refs[e.0 as int][q]@ == keys[e.1 as int]@
}

/// Reference `j` of entry `i` resolves to an edge to the entry with its key.
pub open spec fn ref_resolved_spec(keys: Seq<Vec<u8>>, refs: Seq<Vec<Vec<u8>>>, edges: Seq<(usize, usize)>, i: int, j: int) -> bool {
    exists|t: int| #![trigger keys[t]] 0 <= t < keys.len() && keys[t]@ == refs[i][j]@ && has_edge_spec(edges, i, t)
}

pub open spec fn prefix_of_spec(a: Seq<(usize, usize)>, b: Seq<(usize, usize)>) -> bool {
    a.len() <= b.len() && forall|k: int| #![trigger a[k]] 0 <= k < a.len() ==> b[k] == a[k]
}

proof fn ref_resolved_extend(keys: Seq<Vec<u8>>, refs: Seq<Vec<Vec<u8>>>, a: Seq<(usize, usize)>, b: Seq<(usize, usize)>, i: int, j: int)
    requires ref_resolved_spec(keys, refs, a, i, j), prefix_of_spec(a, b),
    ensures ref_resolved_spec(keys, refs, b, i, j),
{
    let t = choose|t: int| #![trigger keys[t]] 0 <= t < keys.len() && keys[t]@ == refs[i][j]@ && has_edge_spec(a, i, t);
    let k = choose|k: int| #![trigger a[k]] 0 <= k < a.len() && a[k].0 as int == i && a[k].1 as int == t;
    assert(b[k] == a[k]);
    assert(has_edge_spec(b, i, t));
}

/// Edges after projecting one entry's references, and whether every
/// reference resolved.
pub struct RowProjection {
    pub ok: bool,
    pub edges: Vec<(usize, usize)>,
}

/// Appends one edge per reference of entry `i` to `edges`, refusing any
/// reference whose index does not name an entry with the referenced key.
fn project_row(keys: &Vec<Vec<u8>>, references: &Vec<Vec<Vec<u8>>>, i: usize, index_row: &Vec<usize>, input: Vec<(usize, usize)>) -> (result: RowProjection)
    requires i < keys.len(), references.len() == keys.len(), index_row.len() == references@[i as int].len(),
        forall|k: int| #![trigger input@[k]] 0 <= k < input@.len() ==> edge_names_ref_spec(keys@, references@, input@[k]),
    ensures
        prefix_of_spec(input@, result.edges@),
        forall|k: int| #![trigger result.edges@[k]] 0 <= k < result.edges@.len() ==> edge_names_ref_spec(keys@, references@, result.edges@[k]),
        result.ok ==> forall|q: int| #![trigger ref_resolved_spec(keys@, references@, result.edges@, i as int, q)]
            0 <= q < references@[i as int].len() ==> ref_resolved_spec(keys@, references@, result.edges@, i as int, q),
{
    let mut edges = input;
    let row = &references[i];
    let mut j: usize = 0;
    while j < row.len()
        invariant 0 <= j <= row.len(), i < keys.len(), references.len() == keys.len(),
            row == references@[i as int], index_row.len() == row.len(),
            prefix_of_spec(input@, edges@),
            forall|k: int| #![trigger edges@[k]] 0 <= k < edges@.len() ==> edge_names_ref_spec(keys@, references@, edges@[k]),
            forall|q: int| #![trigger ref_resolved_spec(keys@, references@, edges@, i as int, q)] 0 <= q < j ==> ref_resolved_spec(keys@, references@, edges@, i as int, q),
        decreases row.len() - j,
    {
        let t = index_row[j];
        if t >= keys.len() || !key_eq(&keys[t], &row[j]) {
            return RowProjection { ok: false, edges };
        }
        let ghost prev = edges@;
        edges.push((i, t));
        proof {
            assert(prefix_of_spec(prev, edges@));
            let e = edges@[prev.len() as int];
            assert(e == (i, t));
            assert(references@[e.0 as int][j as int]@ == keys@[e.1 as int]@);
            assert(0 <= j < references@[e.0 as int].len());
            assert(edge_names_ref_spec(keys@, references@, e));
            assert forall|k: int| #![trigger edges@[k]] 0 <= k < edges@.len() implies edge_names_ref_spec(keys@, references@, edges@[k]) by {
                if k < prev.len() {
                    assert(edges@[k] == prev[k]);
                }
            }
            assert forall|q: int| #![trigger ref_resolved_spec(keys@, references@, edges@, i as int, q)] 0 <= q < j + 1 implies
                ref_resolved_spec(keys@, references@, edges@, i as int, q) by {
                if q < j {
                    assert(ref_resolved_spec(keys@, references@, prev, i as int, q));
                    ref_resolved_extend(keys@, references@, prev, edges@, i as int, q);
                } else {
                    assert(edges@[prev.len() as int] == (i, t));
                    assert(has_edge_spec(edges@, i as int, t as int));
                }
            }
            assert forall|k: int| #![trigger input@[k]] 0 <= k < input@.len() implies edges@[k] == input@[k] by {
                assert(prev[k] == input@[k]);
            }
        }
        j = j + 1;
    }
    RowProjection { ok: true, edges }
}

/// A validated projection, or the reason it was refused.
pub struct KeyProjection {
    pub ok: bool,
    pub edges: Vec<(usize, usize)>,
}

// r[impl reclaim-plan.key-projection]
/// Validates a consumer's key-to-index mapping and builds the index edges.
/// `reference_index[i][j]` must name the entry whose key equals
/// `references[i][j]`, and `roots[r]` the entry whose key equals
/// `root_keys[r]`. Any unresolved or mismatched reference or root, unsorted
/// or duplicate keys, or a shape mismatch refuses the projection.
pub fn project_keys(
    keys: &Vec<Vec<u8>>, references: &Vec<Vec<Vec<u8>>>, reference_index: &Vec<Vec<usize>>,
    root_keys: &Vec<Vec<u8>>, roots: &Vec<usize>,
) -> (result: KeyProjection)
    ensures result.ok ==> projection_valid_spec(keys@, references@, root_keys@, result.edges@, roots@),
{
    let n = keys.len();
    if references.len() != n || reference_index.len() != n || roots.len() != root_keys.len() {
        return KeyProjection { ok: false, edges: Vec::new() };
    }
    if !keys_sorted(keys) {
        return KeyProjection { ok: false, edges: Vec::new() };
    }
    let mut r: usize = 0;
    while r < roots.len()
        invariant 0 <= r <= roots.len(), roots.len() == root_keys.len(), n == keys.len(),
            forall|q: int| #![trigger roots@[q]] 0 <= q < r ==>
                (roots@[q] as int) < keys.len() && keys@[roots@[q] as int]@ == root_keys@[q]@,
        decreases roots.len() - r,
    {
        if roots[r] >= n || !key_eq(&keys[roots[r]], &root_keys[r]) {
            return KeyProjection { ok: false, edges: Vec::new() };
        }
        r = r + 1;
    }
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut i: usize = 0;
    while i < n
        invariant 0 <= i <= n, n == keys.len(), references.len() == n, reference_index.len() == n,
            keys_sorted_spec(keys@), roots_match_keys_spec(keys@, root_keys@, roots@),
            forall|k: int| #![trigger edges@[k]] 0 <= k < edges@.len() ==> edge_names_ref_spec(keys@, references@, edges@[k]),
            forall|a: int, q: int| #![trigger ref_resolved_spec(keys@, references@, edges@, a, q)]
                0 <= a < i && 0 <= q < references@[a].len() ==> ref_resolved_spec(keys@, references@, edges@, a, q),
        decreases n - i,
    {
        if reference_index[i].len() != references[i].len() {
            return KeyProjection { ok: false, edges: Vec::new() };
        }
        let ghost prev = edges@;
        let row = project_row(keys, references, i, &reference_index[i], edges);
        if !row.ok {
            return KeyProjection { ok: false, edges: Vec::new() };
        }
        edges = row.edges;
        proof {
            assert forall|a: int, q: int| #![trigger ref_resolved_spec(keys@, references@, edges@, a, q)]
                0 <= a < i + 1 && 0 <= q < references@[a].len() implies ref_resolved_spec(keys@, references@, edges@, a, q) by {
                if a < i {
                    assert(ref_resolved_spec(keys@, references@, prev, a, q));
                    ref_resolved_extend(keys@, references@, prev, edges@, a, q);
                }
            }
        }
        i = i + 1;
    }
    proof {
        assert forall|a: int, q: int| #![trigger ref_resolved_spec(keys@, references@, edges@, a, q)]
            0 <= a < references@.len() && 0 <= q < references@[a].len() implies ref_resolved_spec(keys@, references@, edges@, a, q) by {
        }
    }
    KeyProjection { ok: true, edges }
}

/// `mask` marks exactly the keys that `paths` lists.
pub open spec fn mask_matches_paths_spec(keys: Seq<Vec<u8>>, paths: Seq<Vec<u8>>, mask: Seq<bool>) -> bool {
    &&& mask.len() == keys.len()
    &&& (forall|t: int| #![trigger mask[t]] 0 <= t < keys.len() && mask[t] ==>
        exists|s: int| #![trigger paths[s]] 0 <= s < paths.len() && paths[s]@ == keys[t]@)
    &&& (forall|s: int| #![trigger paths[s]] 0 <= s < paths.len() ==> key_marked_spec(keys, mask, paths[s]@))
}

/// Some key equal to `p` is marked in `mask`.
pub open spec fn key_marked_spec(keys: Seq<Vec<u8>>, mask: Seq<bool>, p: Seq<u8>) -> bool {
    exists|t: int| #![trigger keys[t]] 0 <= t < keys.len() && t < mask.len() && mask[t] && keys[t]@ == p
}

/// A plan's path list as a mask over keys, or `None` when a path is not a
/// key, is listed twice, or its index does not name it.
pub fn key_mask(keys: &Vec<Vec<u8>>, paths: &Vec<Vec<u8>>, index: &Vec<usize>) -> (mask: Option<Vec<bool>>)
    ensures
        mask.is_some() ==> mask_matches_paths_spec(keys@, paths@, mask.unwrap()@),
{
    let n = keys.len();
    if index.len() != paths.len() {
        return None;
    }
    let mut m: Vec<bool> = Vec::new();
    let mut k: usize = 0;
    while k < n
        invariant 0 <= k <= n, n == keys.len(), m@.len() == k, forall|t: int| #![trigger m@[t]] 0 <= t < k ==> !m@[t],
        decreases n - k,
    {
        m.push(false);
        k = k + 1;
    }
    let mut s: usize = 0;
    while s < paths.len()
        invariant 0 <= s <= paths.len(), index.len() == paths.len(), n == keys.len(), m@.len() == n,
            forall|t: int| #![trigger m@[t]] 0 <= t < n && m@[t] ==>
                exists|q: int| #![trigger paths@[q]] 0 <= q < s && paths@[q]@ == keys@[t]@,
            forall|q: int| #![trigger paths@[q]] 0 <= q < s ==>
                exists|t: int| #![trigger keys@[t]] 0 <= t < n && m@[t] && keys@[t]@ == paths@[q]@,
        decreases paths.len() - s,
    {
        let t = index[s];
        if t >= n || m[t] || !key_eq(&keys[t], &paths[s]) {
            return None;
        }
        let ghost prev = m@;
        m.set(t, true);
        proof {
            assert forall|u: int| #![trigger m@[u]] 0 <= u < n && m@[u] implies
                exists|q: int| #![trigger paths@[q]] 0 <= q < s + 1 && paths@[q]@ == keys@[u]@ by {
                if u == t as int {
                    assert(paths@[s as int]@ == keys@[u]@);
                } else {
                    assert(m@[u] == prev[u]);
                }
            }
            assert forall|q: int| #![trigger paths@[q]] 0 <= q < s + 1 implies
                exists|u: int| #![trigger keys@[u]] 0 <= u < n && m@[u] && keys@[u]@ == paths@[q]@ by {
                if q == s as int {
                    assert(m@[t as int] && keys@[t as int]@ == paths@[q]@);
                } else {
                    let u = choose|u: int| #![trigger keys@[u]] 0 <= u < n && prev[u] && keys@[u]@ == paths@[q]@;
                    assert(m@[u]);
                }
            }
        }
        s = s + 1;
    }
    assert forall|q: int| #![trigger paths@[q]] 0 <= q < paths@.len() implies key_marked_spec(keys@, m@, paths@[q]@) by {
        let t = choose|t: int| #![trigger keys@[t]] 0 <= t < n && m@[t] && keys@[t]@ == paths@[q]@;
        assert(0 <= t < keys@.len() && t < m@.len() && m@[t] && keys@[t]@ == paths@[q]@);
    }
    Some(m)
}

// r[verify reclaim-plan.key-projection]
/// End to end over paths: under a valid projection, a valid plan, and masks
/// that match the consumer's path lists, no listed candidate path is
/// reachable from a root path.
pub proof fn no_listed_candidate_path_is_live(
    keys: Seq<Vec<u8>>, refs: Seq<Vec<Vec<u8>>>, root_keys: Seq<Vec<u8>>,
    edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>,
    retained: Seq<bool>, candidate: Seq<bool>, candidate_paths: Seq<Vec<u8>>, s: int,
)
    requires projection_valid_spec(keys, refs, root_keys, edges, roots),
        plan_valid_spec(keys.len() as int, edges, roots, reclaimable, retained, candidate),
        mask_matches_paths_spec(keys, candidate_paths, candidate), 0 <= s < candidate_paths.len(),
    ensures !key_live_spec(keys, refs, root_keys, candidate_paths[s]@),
{
    assert(key_marked_spec(keys, candidate, candidate_paths[s]@));
    let t = choose|t: int| #![trigger keys[t]]
        0 <= t < keys.len() && t < candidate.len() && candidate[t] && keys[t]@ == candidate_paths[s]@;
    no_live_key_is_candidate(keys, refs, root_keys, edges, roots, reclaimable, retained, candidate, t);
}

// r[verify reclaim-plan.key-projection]
/// End to end over paths: every entry whose path is reachable from a root
/// path appears in the consumer's retained path list.
pub proof fn every_live_path_is_listed_retained(
    keys: Seq<Vec<u8>>, refs: Seq<Vec<Vec<u8>>>, root_keys: Seq<Vec<u8>>,
    edges: Seq<(usize, usize)>, roots: Seq<usize>, reclaimable: Seq<bool>,
    retained: Seq<bool>, candidate: Seq<bool>, retained_paths: Seq<Vec<u8>>, v: int,
)
    requires projection_valid_spec(keys, refs, root_keys, edges, roots),
        plan_valid_spec(keys.len() as int, edges, roots, reclaimable, retained, candidate),
        mask_matches_paths_spec(keys, retained_paths, retained), 0 <= v < keys.len(),
        key_live_spec(keys, refs, root_keys, keys[v]@),
    ensures exists|s: int| #![trigger retained_paths[s]] 0 <= s < retained_paths.len() && retained_paths[s]@ == keys[v]@,
{
    key_live_matches_index_live(keys, refs, root_keys, edges, roots, v);
    valid_plan_retains_live(keys.len() as int, edges, roots, reclaimable, retained, candidate, v);
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
