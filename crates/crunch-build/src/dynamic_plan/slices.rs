//! Pure admission decisions over bounded, shell-supplied subtree observations.
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::NarDigest;
use super::OutputName;
use super::SourceId;
use super::v2::MAX_PLAN_SLICES;
use super::v2::MAX_SLICE_ADMITTED_BYTES;
use super::v2::SliceSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceNodeKind {
    File,
    Directory,
    Symlink,
    Absent,
}

/// The shell supplies one observation for each requested slice, including the
/// node reached (if any) and whether a symlink was encountered during the walk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceTreeFact {
    pub source_id: SourceId,
    pub producer_output: OutputName,
    pub subpath: String,
    pub kind: SliceNodeKind,
    pub traversed_symlink: bool,
    pub observed_nar_blake3: Option<NarDigest>,
    pub nar_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceRejectionKind {
    OutputUndeclared,
    Absent,
    SymlinkTraversal,
    DigestMismatch,
    Limit,
    Conflict,
}

impl SliceRejectionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OutputUndeclared => "slice-output-undeclared",
            Self::Absent => "slice-absent",
            Self::SymlinkTraversal => "slice-symlink-traversal",
            Self::DigestMismatch => "slice-digest-mismatch",
            Self::Limit => "slice-limit",
            Self::Conflict => "slice-conflict",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceRejection {
    pub source_id: SourceId,
    pub kind: SliceRejectionKind,
    pub detail: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedSlice {
    /// Canonical source id chosen for one future content/name publication.
    pub publication_source_id: SourceId,
    pub source_id: SourceId,
    pub producer_output: OutputName,
    pub subpath: String,
    pub store_name: String,
    pub declared_nar_blake3: NarDigest,
    pub observed_nar_blake3: NarDigest,
    pub nar_bytes: u64,
}

// r[impl mantle.dynamic_plan_source_slices.content_admission]
/// Does not publish objects or prove that shell-provided facts are truthful.
/// A rejected plan yields no partial publication plan.
pub fn plan_slices<S: Borrow<SliceSource>>(
    slices: &[S],
    declared_outputs: &BTreeSet<OutputName>,
    facts: &[SliceTreeFact],
) -> Result<Vec<PlannedSlice>, SliceRejection> {
    let first = slices.first().map(Borrow::borrow);
    if slices.len() > MAX_PLAN_SLICES as usize {
        return Err(SliceRejection {
            source_id: first.expect("over-limit slices are non-empty").id.clone(),
            kind: SliceRejectionKind::Limit,
            detail: "slice count",
        });
    }
    if facts.len() > MAX_PLAN_SLICES as usize {
        return Err(reject(facts[0].source_id.clone(), SliceRejectionKind::Limit, "tree fact count"));
    }
    if facts.len() > slices.len() {
        return Err(reject(facts[0].source_id.clone(), SliceRejectionKind::Conflict, "unexpected tree fact"));
    }
    let facts_by_id = facts.iter().map(|fact| (&fact.source_id, fact)).collect::<BTreeMap<_, _>>();
    if facts_by_id.len() != facts.len() {
        return Err(reject(facts[0].source_id.clone(), SliceRejectionKind::Conflict, "duplicate tree fact"));
    }
    let mut ordered = slices.iter().map(Borrow::borrow).collect::<Vec<&SliceSource>>();
    ordered.sort_by(|left, right| left.id.cmp(&right.id));
    let mut planned = Vec::with_capacity(ordered.len());
    let mut publications: BTreeMap<(&str, &NarDigest), &SourceId> = BTreeMap::new();
    let mut total_bytes = 0_u64;
    for slice in ordered {
        if planned.last().is_some_and(|previous: &PlannedSlice| previous.source_id == slice.id) {
            return Err(reject(slice.id.clone(), SliceRejectionKind::Conflict, "duplicate source id"));
        }
        if !declared_outputs.contains(&slice.producer_output) {
            return Err(reject(
                slice.id.clone(),
                SliceRejectionKind::OutputUndeclared,
                "producer output is undeclared",
            ));
        }
        let Some(fact) = facts_by_id.get(&slice.id) else {
            return Err(reject(slice.id.clone(), SliceRejectionKind::Absent, "subtree observation is absent"));
        };
        if fact.producer_output != slice.producer_output || fact.subpath != slice.subpath {
            return Err(reject(
                slice.id.clone(),
                SliceRejectionKind::Conflict,
                "tree fact is bound to another subtree",
            ));
        }
        if fact.traversed_symlink || fact.kind == SliceNodeKind::Symlink {
            return Err(reject(slice.id.clone(), SliceRejectionKind::SymlinkTraversal, "symlink on slice walk"));
        }
        if fact.kind == SliceNodeKind::Absent {
            return Err(reject(slice.id.clone(), SliceRejectionKind::Absent, "subtree is absent"));
        }
        if fact.observed_nar_blake3.as_ref() != Some(&slice.nar_blake3) {
            return Err(reject(slice.id.clone(), SliceRejectionKind::DigestMismatch, "observed NAR BLAKE3 differs"));
        }
        total_bytes = total_bytes
            .checked_add(fact.nar_bytes)
            .ok_or_else(|| reject(slice.id.clone(), SliceRejectionKind::Limit, "admitted NAR bytes overflow"))?;
        if total_bytes > MAX_SLICE_ADMITTED_BYTES {
            return Err(reject(slice.id.clone(), SliceRejectionKind::Limit, "admitted NAR bytes"));
        }
        let publication_source_id =
            publications.entry((slice.store_name.as_str(), &slice.nar_blake3)).or_insert(&slice.id);
        planned.push(PlannedSlice {
            source_id: slice.id.clone(),
            publication_source_id: (**publication_source_id).clone(),
            producer_output: slice.producer_output.clone(),
            subpath: slice.subpath.clone(),
            store_name: slice.store_name.clone(),
            declared_nar_blake3: slice.nar_blake3.clone(),
            observed_nar_blake3: fact.observed_nar_blake3.as_ref().expect("matched digest").clone(),
            nar_bytes: fact.nar_bytes,
        });
    }
    assert_eq!(planned.len(), slices.len());
    assert!(total_bytes <= MAX_SLICE_ADMITTED_BYTES);
    Ok(planned)
}

fn reject(source_id: SourceId, kind: SliceRejectionKind, detail: &'static str) -> SliceRejection {
    SliceRejection {
        source_id,
        kind,
        detail,
    }
}
