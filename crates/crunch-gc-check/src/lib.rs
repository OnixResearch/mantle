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
//! The mapping from path identifiers to indices is computed here but
//! validated by the kernel's `project_keys` and `key_mask`, so the kernel's
//! key-level theorems apply: no listed candidate path is reachable from a
//! root path, and every reachable path is listed as retained.
//!
//! Non-claims: the kernel is compiled with Mantle's `vstd`, which erases its
//! ghost code, and is not re-verified under that cohort; that each entry's
//! path, references, and ownership are passed together (one loop below);
//! root and reference completeness; and anything the store deletes without
//! consulting the checked plan.

use std::collections::BTreeMap;

use crunch_gc_core::GcOwnership;
use crunch_gc_core::GcPlan;
use crunch_gc_core::GcPlanRequest;

// r[impl mantle.gc_plan.kernel_import]
// Exact upstream bytes are refreshed only by the repository-owned importer.
// The producer's lint policy allows `&Vec` parameters and `x = x + 1`, and
// the planner and admission helpers are unused here; the bytes stay
// unchanged.
#[rustfmt::skip]
#[allow(clippy::ptr_arg, clippy::assign_op_pattern, dead_code, unused_imports)]
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
    /// The verified projection refused the request: a root or reference
    /// names no entry, or a computed index names the wrong entry.
    ProjectionRejected,
    /// The verified mask check refused a planned path list: a path is not
    /// an entry, is listed twice, or its computed index names another entry.
    PlannedPathsRejected,
    /// The deletion intents do not name exactly the candidates.
    IntentsDifferFromCandidates,
    /// The verified checker rejected the plan.
    KernelRejected,
}

/// A request projected into kernel indices by the verified projection, ready
/// to check a plan.
#[derive(Clone, Debug)]
pub struct GcCheck {
    index: BTreeMap<String, usize>,
    keys: Vec<Vec<u8>>,
    n: usize,
    edges: Vec<(usize, usize)>,
    roots: Vec<usize>,
    reclaimable: Vec<bool>,
}

impl GcCheck {
    /// Projects `request`. Entries are sorted by path identifier and indexed
    /// by an unverified lookup; the kernel's `project_keys` then validates
    /// that every index names the entry whose path bytes it claims and builds
    /// the edges itself, so the lookup is checked, not trusted.
    pub fn prepare(request: &GcPlanRequest) -> Result<Self, GcCheckError> {
        let mut entries: Vec<&crunch_gc_core::GcEntry> = request.entries.iter().collect();
        entries.sort_by(|left, right| left.path_id.cmp(&right.path_id));
        let mut index = BTreeMap::new();
        for (position, entry) in entries.iter().enumerate() {
            if index.insert(entry.path_id.clone(), position).is_some() {
                return Err(GcCheckError::DuplicateEntry {
                    path_id: entry.path_id.clone(),
                });
            }
        }
        let n = entries.len();
        let lookup = |path_id: &str| index.get(path_id).copied().unwrap_or(n);
        let keys: Vec<Vec<u8>> = entries.iter().map(|entry| entry.path_id.as_bytes().to_vec()).collect();
        let references: Vec<Vec<Vec<u8>>> = entries
            .iter()
            .map(|entry| entry.references.iter().map(|reference| reference.as_bytes().to_vec()).collect())
            .collect();
        let reference_index: Vec<Vec<usize>> = entries
            .iter()
            .map(|entry| entry.references.iter().map(|reference| lookup(reference)).collect())
            .collect();
        let root_keys: Vec<Vec<u8>> = request.roots.iter().map(|root| root.as_bytes().to_vec()).collect();
        let roots: Vec<usize> = request.roots.iter().map(|root| lookup(root)).collect();
        let reclaimable = entries.iter().map(|entry| entry.ownership == GcOwnership::Overlay).collect();
        let projection = kernel::project_keys(&keys, &references, &reference_index, &root_keys, &roots);
        if !projection.ok {
            return Err(GcCheckError::ProjectionRejected);
        }
        Ok(Self {
            index,
            keys,
            n,
            edges: projection.edges,
            roots,
            reclaimable,
        })
    }

    // r[impl mantle.gc_plan.runtime_check]
    /// Accepts `plan` only if the verified mask check accepts its retained
    /// and candidate path lists, its deletion intents name exactly its
    /// candidates, and the verified checker accepts the plan.
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
        let paths: Vec<Vec<u8>> = path_ids.iter().map(|path_id| path_id.as_bytes().to_vec()).collect();
        let positions: Vec<usize> =
            path_ids.iter().map(|path_id| self.index.get(path_id).copied().unwrap_or(self.n)).collect();
        kernel::key_mask(&self.keys, &paths, &positions).ok_or(GcCheckError::PlannedPathsRejected)
    }
}
