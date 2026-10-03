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
    validate_slice_fact_counts(slices, facts)?;
    let facts_by_id = facts.iter().map(|fact| (&fact.source_id, fact)).collect::<BTreeMap<_, _>>();
    if let Some(first) = facts.first()
        && facts_by_id.len() != facts.len()
    {
        return Err(reject(first.source_id.clone(), SliceRejectionKind::Conflict, "duplicate tree fact"));
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
        let Some(observed_digest) = fact.observed_nar_blake3.as_ref() else {
            return Err(reject(slice.id.clone(), SliceRejectionKind::DigestMismatch, "observed NAR BLAKE3 differs"));
        };
        if observed_digest != &slice.nar_blake3 {
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
            observed_nar_blake3: observed_digest.clone(),
            nar_bytes: fact.nar_bytes,
        });
    }
    assert_eq!(planned.len(), slices.len());
    assert!(total_bytes <= MAX_SLICE_ADMITTED_BYTES);
    Ok(planned)
}

fn validate_slice_fact_counts<S: Borrow<SliceSource>>(
    slices: &[S],
    facts: &[SliceTreeFact],
) -> Result<(), SliceRejection> {
    if let Some(first) = slices.first().map(Borrow::borrow)
        && !matches!(u32::try_from(slices.len()), Ok(count) if count <= MAX_PLAN_SLICES)
    {
        return Err(reject(first.id.clone(), SliceRejectionKind::Limit, "slice count"));
    }
    if let Some(first) = facts.first() {
        if !matches!(u32::try_from(facts.len()), Ok(count) if count <= MAX_PLAN_SLICES) {
            return Err(reject(first.source_id.clone(), SliceRejectionKind::Limit, "tree fact count"));
        }
        if facts.len() > slices.len() {
            return Err(reject(first.source_id.clone(), SliceRejectionKind::Conflict, "unexpected tree fact"));
        }
    }
    Ok(())
}

fn reject(source_id: SourceId, kind: SliceRejectionKind, detail: &'static str) -> SliceRejection {
    SliceRejection {
        source_id,
        kind,
        detail,
    }
}
