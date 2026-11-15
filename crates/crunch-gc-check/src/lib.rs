#![feature(register_tool)]
#![register_tool(tigerstyle)]
//! Runtime check of `crunch_gc_core::plan_gc` results with the imported,
//! unmodified Terrane reclaim-plan kernel (ADR 0158).
//!
//! [`GcCheck::prepare`] projects a [`GcPlanRequest`] into kernel indices
//! before `plan_gc` consumes it, and [`GcCheck::check`] maps the resulting
//! plan back onto those indices and runs the kernel's `check_plan`. The kernel
//! proves that a plan it accepts keeps every root, is closed under references,
//! and makes exactly the unretained reclaimable entries candidates, so no
//! entry reachable from a root, and no base entry, is a candidate.
//!
//! Non-claims: this projection is not verified; the kernel is compiled with
//! Mantle's `vstd`, which erases its ghost code, and is not re-verified under
//! that cohort; root and reference completeness, and anything the store
//! deletes without consulting the checked plan, are outside the check.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crunch_gc_core::GcOwnership;
use crunch_gc_core::GcPlan;
use crunch_gc_core::GcPlanRequest;

// r[impl mantle.gc_plan.kernel_import]
// Exact upstream bytes are refreshed only by the repository-owned importer.
// The producer's lint policy allows `&Vec` parameters, and the planner and
// admission helpers are unused here; the bytes stay unchanged.
#[rustfmt::skip]
#[allow(clippy::ptr_arg, dead_code, unused_imports)]
#[path = "../../../vendor/terrane-reclaim-plan/verified.rs"]
mod kernel;

#[cfg(test)]
#[path = "parity_tests.rs"]
mod parity_tests;

/// Why a plan was refused. Every variant blocks deletion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GcCheckError {
    /// The request names the same entry twice, so indices are ambiguous.
    DuplicateEntry { path_id: String },
    /// The plan retains or deletes a path the request does not contain.
    UnknownPlannedPath { path_id: String },
    /// The plan lists the same path twice in one set.
    DuplicatePlannedPath { path_id: String },
    /// The deletion intents do not name exactly the candidates.
    IntentsDifferFromCandidates,
    /// The verified checker rejected the plan.
    KernelRejected,
}

/// A request projected into kernel indices, ready to check a plan.
#[derive(Clone, Debug)]
pub struct GcCheck {
    index: BTreeMap<String, usize>,
    n: usize,
    edges: Vec<(usize, usize)>,
    roots: Vec<usize>,
    reclaimable: Vec<bool>,
}

impl GcCheck {
    /// Projects `request`: entries sorted by path identifier, references and
    /// roots deduplicated, overlay entries reclaimable, and any path that
    /// names no entry mapped to `n`, which the kernel rejects as out of range.
    pub fn prepare(request: &GcPlanRequest) -> Result<Self, GcCheckError> {
        let mut index = BTreeMap::new();
        for entry in &request.entries {
            if index.insert(entry.path_id.clone(), 0).is_some() {
                return Err(GcCheckError::DuplicateEntry {
                    path_id: entry.path_id.clone(),
                });
            }
        }
        for (position, slot) in index.values_mut().enumerate() {
            *slot = position;
        }
        let n = index.len();
        debug_assert_eq!(n, request.entries.len());
        let lookup = |path_id: &str| index.get(path_id).copied().unwrap_or(n);
        let mut reclaimable = vec![false; n];
        let mut edges = Vec::new();
        for entry in &request.entries {
            let owner = lookup(&entry.path_id);
            reclaimable[owner] = entry.ownership == GcOwnership::Overlay;
            let targets: BTreeSet<usize> = entry.references.iter().map(|reference| lookup(reference)).collect();
            edges.extend(targets.into_iter().map(|target| (owner, target)));
        }
        edges.sort_unstable();
        let roots: BTreeSet<usize> = request.roots.iter().map(|root| lookup(root)).collect();
        Ok(Self {
            index,
            n,
            edges,
            roots: roots.into_iter().collect(),
            reclaimable,
        })
    }

    // r[impl mantle.gc_plan.runtime_check]
    /// Accepts `plan` only if the verified kernel accepts its retained and
    /// candidate sets and its deletion intents name exactly its candidates.
    pub fn check(&self, plan: &GcPlan) -> Result<(), GcCheckError> {
        let retained = self.mask(&plan.retained_path_ids)?;
        let candidate = self.mask(&plan.candidate_path_ids)?;
        let intents: Vec<&str> = plan.mutation_intents.iter().map(|intent| intent.path_id.as_str()).collect();
        let candidates: Vec<&str> = plan.candidate_path_ids.iter().map(String::as_str).collect();
        if intents != candidates {
            return Err(GcCheckError::IntentsDifferFromCandidates);
        }
        if kernel::check_plan(self.n, &self.edges, &self.roots, &self.reclaimable, &retained, &candidate) {
            Ok(())
        } else {
            Err(GcCheckError::KernelRejected)
        }
    }

    fn mask(&self, path_ids: &[String]) -> Result<Vec<bool>, GcCheckError> {
        let mut mask = vec![false; self.n];
        for path_id in path_ids {
            let Some(&position) = self.index.get(path_id) else {
                return Err(GcCheckError::UnknownPlannedPath {
                    path_id: path_id.clone(),
                });
            };
            if mask[position] {
                return Err(GcCheckError::DuplicatePlannedPath {
                    path_id: path_id.clone(),
                });
            }
            mask[position] = true;
        }
        Ok(mask)
    }
}
