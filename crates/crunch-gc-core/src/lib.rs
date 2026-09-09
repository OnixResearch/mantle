#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]
//! Pure, bounded garbage-collection decisions over normalized store facts.
//!
//! This crate does not inspect a filesystem, query a service, delete data, or
//! publish reports. A standard-library shell supplies complete normalized
//! facts and executes the returned mutation intents.

extern crate alloc;

pub mod retention;

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

pub const MAX_GC_ENTRIES: usize = 1_000_000;
pub const MAX_GC_ROOTS: usize = 65_536;
pub const MAX_GC_REFERENCES: usize = 4_000_000;
pub const MAX_REFERENCES_PER_ENTRY: usize = 65_536;
pub const MAX_RECLAIM_OBSERVATIONS: usize = 4_000_000;
pub const MAX_STORE_PATH_ID_BYTES: usize = 4_096;
pub const GC_PLAN_ID_BYTES: usize = blake3::OUT_LEN;

const GC_PLAN_DOMAIN: &[u8] = b"mantle.gc.plan.v2";
const ROOTS_FIELD: &[u8] = b"roots";
const ENTRIES_FIELD: &[u8] = b"entries";
const RETAINED_FIELD: &[u8] = b"retained";
const CANDIDATES_FIELD: &[u8] = b"candidates";
const BASES_FIELD: &[u8] = b"bases";
const FIELD_SEPARATOR: u8 = 0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcExecutionMode {
    DryRun,
    Execute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcMutationDisposition {
    ReportOnly,
    Execute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcMutationKind {
    RemoveStorePath,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GcOwnership {
    Overlay,
    Base { layer_index: usize },
}

impl GcOwnership {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Overlay => "overlay",
            Self::Base { .. } => "base",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GcEntry {
    pub path_id: String,
    pub references: Vec<String>,
    pub declared_nar_bytes: u64,
    pub ownership: GcOwnership,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GcPlanRequest {
    pub roots: Vec<String>,
    pub entries: Vec<GcEntry>,
    pub execution_mode: GcExecutionMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GcMutationIntent {
    pub path_id: String,
    pub kind: GcMutationKind,
    pub disposition: GcMutationDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GcPlanId([u8; GC_PLAN_ID_BYTES]);

impl GcPlanId {
    #[must_use]
    pub const fn into_bytes(self) -> [u8; GC_PLAN_ID_BYTES] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GcReclaimSummary {
    pub candidate_count: usize,
    pub declared_nar_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GcObservedReclaimSummary {
    pub observation_count: usize,
    pub reclaimable_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GcRetainingRoots {
    pub path_id: String,
    pub root_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GcPlan {
    pub plan_id: GcPlanId,
    pub roots: Vec<String>,
    pub retained_path_ids: Vec<String>,
    pub retaining_roots: Vec<GcRetainingRoots>,
    pub base_path_ids: Vec<String>,
    pub candidate_path_ids: Vec<String>,
    pub reclaim_summary: GcReclaimSummary,
    pub mutation_disposition: GcMutationDisposition,
    pub mutation_intents: Vec<GcMutationIntent>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GcReportDecision {
    pub plan_id: GcPlanId,
    pub retained_path_count: usize,
    pub candidate_path_count: usize,
    pub declared_reclaimable_bytes: u64,
    pub mutation_disposition: GcMutationDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GcPlanError {
    TooManyRoots {
        actual: usize,
        maximum: usize,
    },
    TooManyEntries {
        actual: usize,
        maximum: usize,
    },
    TooManyReferences {
        actual: usize,
        maximum: usize,
    },
    TooManyEntryReferences {
        path_id: String,
        actual: usize,
        maximum: usize,
    },
    InvalidPathId {
        path_id: String,
    },
    DuplicateEntry {
        path_id: String,
    },
    MissingRoot {
        path_id: String,
    },
    MissingReference {
        owner_path_id: String,
        reference_path_id: String,
    },
    BaseToOverlayReference {
        owner_path_id: String,
        reference_path_id: String,
        base_layer_index: usize,
    },
    ReclaimSizeOverflow,
    TooManyReclaimObservations {
        actual: usize,
        maximum: usize,
    },
    IdentityEncodingOverflow,
}

pub fn summarize_reclaim_observations(observed_sizes_bytes: Vec<u64>) -> Result<GcObservedReclaimSummary, GcPlanError> {
    if observed_sizes_bytes.len() > MAX_RECLAIM_OBSERVATIONS {
        return Err(GcPlanError::TooManyReclaimObservations {
            actual: observed_sizes_bytes.len(),
            maximum: MAX_RECLAIM_OBSERVATIONS,
        });
    }
    let mut reclaimable_bytes = 0_u64;
    for observed_size_bytes in &observed_sizes_bytes {
        reclaimable_bytes =
            reclaimable_bytes.checked_add(*observed_size_bytes).ok_or(GcPlanError::ReclaimSizeOverflow)?;
    }
    debug_assert!(observed_sizes_bytes.len() <= MAX_RECLAIM_OBSERVATIONS);
    debug_assert!(!observed_sizes_bytes.is_empty() || reclaimable_bytes == 0);
    Ok(GcObservedReclaimSummary {
        observation_count: observed_sizes_bytes.len(),
        reclaimable_bytes,
    })
}

#[must_use]
pub fn report_decision(plan: GcPlan) -> GcReportDecision {
    GcReportDecision {
        plan_id: plan.plan_id,
        retained_path_count: plan.retained_path_ids.len(),
        candidate_path_count: plan.candidate_path_ids.len(),
        declared_reclaimable_bytes: plan.reclaim_summary.declared_nar_bytes,
        mutation_disposition: plan.mutation_disposition,
    }
}

pub fn plan_gc(request: GcPlanRequest) -> Result<GcPlan, GcPlanError> {
    validate_request_bounds(&request)?;
    let roots = normalize_roots(request.roots)?;
    let entries = normalize_entries(request.entries)?;
    let entries_by_id = index_entries(&entries)?;
    validate_links(&roots, &entries, &entries_by_id)?;

    let retained = compute_reachable(&roots, &entries_by_id)?;
    let candidate_path_ids = entries
        .iter()
        .filter(|entry| entry.ownership == GcOwnership::Overlay && !retained.contains(entry.path_id.as_str()))
        .map(|entry| entry.path_id.clone())
        .collect::<Vec<_>>();
    let base_path_ids = entries
        .iter()
        .filter(|entry| matches!(entry.ownership, GcOwnership::Base { .. }))
        .map(|entry| entry.path_id.clone())
        .collect::<Vec<_>>();
    let retained_path_ids = retained.into_iter().collect::<Vec<_>>();
    let retaining_roots = compute_retaining_roots(&roots, &entries_by_id)?;
    let reclaim_summary = summarize_reclaim(&candidate_path_ids, &entries_by_id)?;
    let mutation_disposition = disposition_for_mode(request.execution_mode);
    let mutation_intents = candidate_path_ids
        .iter()
        .map(|path_id| GcMutationIntent {
            path_id: path_id.clone(),
            kind: GcMutationKind::RemoveStorePath,
            disposition: mutation_disposition,
        })
        .collect::<Vec<_>>();
    let plan_id = compute_plan_id(&roots, &entries, &retained_path_ids, &base_path_ids, &candidate_path_ids)?;

    debug_assert_eq!(candidate_path_ids.len(), mutation_intents.len());
    debug_assert_eq!(reclaim_summary.candidate_count, candidate_path_ids.len());
    Ok(GcPlan {
        plan_id,
        roots,
        retained_path_ids,
        retaining_roots,
        base_path_ids,
        candidate_path_ids,
        reclaim_summary,
        mutation_disposition,
        mutation_intents,
    })
}

fn validate_request_bounds(request: &GcPlanRequest) -> Result<(), GcPlanError> {
    if request.roots.len() > MAX_GC_ROOTS {
        return Err(GcPlanError::TooManyRoots {
            actual: request.roots.len(),
            maximum: MAX_GC_ROOTS,
        });
    }
    if request.entries.len() > MAX_GC_ENTRIES {
        return Err(GcPlanError::TooManyEntries {
            actual: request.entries.len(),
            maximum: MAX_GC_ENTRIES,
        });
    }
    let mut reference_count = 0_usize;
    for entry in &request.entries {
        if entry.references.len() > MAX_REFERENCES_PER_ENTRY {
            return Err(GcPlanError::TooManyEntryReferences {
                path_id: entry.path_id.clone(),
                actual: entry.references.len(),
                maximum: MAX_REFERENCES_PER_ENTRY,
            });
        }
        reference_count =
            reference_count.checked_add(entry.references.len()).ok_or(GcPlanError::TooManyReferences {
                actual: usize::MAX,
                maximum: MAX_GC_REFERENCES,
            })?;
        if reference_count > MAX_GC_REFERENCES {
            return Err(GcPlanError::TooManyReferences {
                actual: reference_count,
                maximum: MAX_GC_REFERENCES,
            });
        }
    }
    debug_assert!(request.roots.len() <= MAX_GC_ROOTS);
    debug_assert!(reference_count <= MAX_GC_REFERENCES);
    Ok(())
}

fn normalize_roots(mut roots: Vec<String>) -> Result<Vec<String>, GcPlanError> {
    for root in &roots {
        validate_path_id(root)?;
    }
    roots.sort();
    roots.dedup();
    Ok(roots)
}

fn normalize_entries(mut entries: Vec<GcEntry>) -> Result<Vec<GcEntry>, GcPlanError> {
    for entry in &mut entries {
        validate_path_id(&entry.path_id)?;
        for reference in &entry.references {
            validate_path_id(reference)?;
        }
        entry.references.sort();
        entry.references.dedup();
    }
    entries.sort_by(|left, right| left.path_id.cmp(&right.path_id));
    debug_assert!(entries.windows(2).all(|pair| pair[0].path_id <= pair[1].path_id));
    debug_assert!(entries.iter().all(|entry| entry.references.windows(2).all(|pair| pair[0] < pair[1])));
    for adjacent in entries.windows(2) {
        let [left, right] = adjacent else {
            continue;
        };
        if left.path_id == right.path_id {
            return Err(GcPlanError::DuplicateEntry {
                path_id: left.path_id.clone(),
            });
        }
    }
    Ok(entries)
}

fn validate_path_id(path_id: &str) -> Result<(), GcPlanError> {
    let has_valid_segments =
        path_id.split('/').skip(1).all(|segment| !segment.is_empty() && segment != "." && segment != "..");
    let is_valid = !path_id.is_empty()
        && path_id.len() <= MAX_STORE_PATH_ID_BYTES
        && path_id.starts_with('/')
        && !path_id.ends_with('/')
        && !path_id.chars().any(char::is_control)
        && has_valid_segments;
    if !is_valid {
        return Err(GcPlanError::InvalidPathId {
            path_id: String::from(path_id),
        });
    }
    Ok(())
}

fn index_entries(entries: &[GcEntry]) -> Result<BTreeMap<&str, &GcEntry>, GcPlanError> {
    let mut entries_by_id = BTreeMap::new();
    for entry in entries {
        if entries_by_id.insert(entry.path_id.as_str(), entry).is_some() {
            return Err(GcPlanError::DuplicateEntry {
                path_id: entry.path_id.clone(),
            });
        }
    }
    debug_assert_eq!(entries_by_id.len(), entries.len());
    Ok(entries_by_id)
}

fn validate_links(
    roots: &[String],
    entries: &[GcEntry],
    entries_by_id: &BTreeMap<&str, &GcEntry>,
) -> Result<(), GcPlanError> {
    debug_assert!(entries_by_id.len() >= entries.len());
    debug_assert!(roots.len() <= entries_by_id.len());
    for root in roots {
        if !entries_by_id.contains_key(root.as_str()) {
            return Err(GcPlanError::MissingRoot { path_id: root.clone() });
        }
    }
    for entry in entries {
        for reference in &entry.references {
            let Some(target) = entries_by_id.get(reference.as_str()) else {
                return Err(GcPlanError::MissingReference {
                    owner_path_id: entry.path_id.clone(),
                    reference_path_id: reference.clone(),
                });
            };
            if let GcOwnership::Base { layer_index } = entry.ownership
                && target.ownership == GcOwnership::Overlay
            {
                return Err(GcPlanError::BaseToOverlayReference {
                    owner_path_id: entry.path_id.clone(),
                    reference_path_id: reference.clone(),
                    base_layer_index: layer_index,
                });
            }
        }
    }
    Ok(())
}

fn compute_reachable(
    roots: &[String],
    entries_by_id: &BTreeMap<&str, &GcEntry>,
) -> Result<BTreeSet<String>, GcPlanError> {
    let mut retained = BTreeSet::new();
    let mut queue = roots.to_vec();
    let mut queue_index = 0_usize;
    while queue_index < queue.len() {
        let path_id = &queue[queue_index];
        queue_index += 1;
        if !retained.insert(path_id.clone()) {
            continue;
        }
        let entry = entries_by_id.get(path_id.as_str()).ok_or_else(|| GcPlanError::MissingRoot {
            path_id: path_id.clone(),
        })?;
        queue.extend(entry.references.iter().cloned());
    }
    debug_assert!(retained.len() <= entries_by_id.len());
    Ok(retained)
}

fn compute_retaining_roots(
    roots: &[String],
    entries_by_id: &BTreeMap<&str, &GcEntry>,
) -> Result<Vec<GcRetainingRoots>, GcPlanError> {
    debug_assert!(!roots.is_empty() || entries_by_id.is_empty());
    let mut roots_by_path = BTreeMap::<String, BTreeSet<String>>::new();
    for root in roots {
        let mut visited = BTreeSet::new();
        let mut queue = vec![root.clone()];
        let mut queue_index = 0_usize;
        while queue_index < queue.len() {
            let path_id = &queue[queue_index];
            queue_index += 1;
            if !visited.insert(path_id.clone()) {
                continue;
            }
            roots_by_path.entry(path_id.clone()).or_default().insert(root.clone());
            debug_assert!(roots_by_path.len() <= entries_by_id.len());
            let entry = entries_by_id.get(path_id.as_str()).ok_or_else(|| GcPlanError::MissingRoot {
                path_id: path_id.clone(),
            })?;
            queue.extend(entry.references.iter().cloned());
        }
    }
    debug_assert!(roots_by_path.len() <= entries_by_id.len());
    Ok(roots_by_path
        .into_iter()
        .map(|(path_id, root_ids)| GcRetainingRoots {
            path_id,
            root_ids: root_ids.into_iter().collect(),
        })
        .collect())
}

fn summarize_reclaim(
    candidate_path_ids: &[String],
    entries_by_id: &BTreeMap<&str, &GcEntry>,
) -> Result<GcReclaimSummary, GcPlanError> {
    let mut declared_nar_bytes = 0_u64;
    for path_id in candidate_path_ids {
        let entry = entries_by_id.get(path_id.as_str()).ok_or_else(|| GcPlanError::MissingRoot {
            path_id: path_id.clone(),
        })?;
        declared_nar_bytes =
            declared_nar_bytes.checked_add(entry.declared_nar_bytes).ok_or(GcPlanError::ReclaimSizeOverflow)?;
    }
    Ok(GcReclaimSummary {
        candidate_count: candidate_path_ids.len(),
        declared_nar_bytes,
    })
}

const fn disposition_for_mode(mode: GcExecutionMode) -> GcMutationDisposition {
    match mode {
        GcExecutionMode::DryRun => GcMutationDisposition::ReportOnly,
        GcExecutionMode::Execute => GcMutationDisposition::Execute,
    }
}

fn compute_plan_id(
    roots: &[String],
    entries: &[GcEntry],
    retained_path_ids: &[String],
    base_path_ids: &[String],
    candidate_path_ids: &[String],
) -> Result<GcPlanId, GcPlanError> {
    debug_assert!(roots.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(entries.windows(2).all(|pair| pair[0].path_id < pair[1].path_id));
    let classified_count = retained_path_ids
        .len()
        .checked_add(candidate_path_ids.len())
        .ok_or(GcPlanError::IdentityEncodingOverflow)?;
    debug_assert!(classified_count <= entries.len());
    let mut hasher = blake3::Hasher::new();
    hasher.update(GC_PLAN_DOMAIN);
    hash_field_label(&mut hasher, ROOTS_FIELD);
    hash_strings(&mut hasher, roots)?;
    hash_field_label(&mut hasher, ENTRIES_FIELD);
    let entry_count = u64::try_from(entries.len()).map_err(|_| GcPlanError::IdentityEncodingOverflow)?;
    hasher.update(&entry_count.to_be_bytes());
    for entry in entries {
        hash_string(&mut hasher, &entry.path_id)?;
        hash_strings(&mut hasher, &entry.references)?;
        hasher.update(&entry.declared_nar_bytes.to_be_bytes());
        hash_string(&mut hasher, entry.ownership.as_str())?;
        let layer_index = match entry.ownership {
            GcOwnership::Overlay => 0,
            GcOwnership::Base { layer_index } => layer_index,
        };
        let encoded_layer = u64::try_from(layer_index).map_err(|_| GcPlanError::IdentityEncodingOverflow)?;
        hasher.update(&encoded_layer.to_be_bytes());
    }
    hash_field_label(&mut hasher, RETAINED_FIELD);
    hash_strings(&mut hasher, retained_path_ids)?;
    hash_field_label(&mut hasher, BASES_FIELD);
    hash_strings(&mut hasher, base_path_ids)?;
    hash_field_label(&mut hasher, CANDIDATES_FIELD);
    hash_strings(&mut hasher, candidate_path_ids)?;
    Ok(GcPlanId(*hasher.finalize().as_bytes()))
}

fn hash_field_label(hasher: &mut blake3::Hasher, label: &[u8]) {
    hasher.update(label);
    hasher.update(&[FIELD_SEPARATOR]);
}

fn hash_strings(hasher: &mut blake3::Hasher, values: &[String]) -> Result<(), GcPlanError> {
    let count = u64::try_from(values.len()).map_err(|_| GcPlanError::IdentityEncodingOverflow)?;
    hasher.update(&count.to_be_bytes());
    for value in values {
        hash_string(hasher, value)?;
    }
    Ok(())
}

fn hash_string(hasher: &mut blake3::Hasher, value: &str) -> Result<(), GcPlanError> {
    let byte_count = u64::try_from(value.len()).map_err(|_| GcPlanError::IdentityEncodingOverflow)?;
    hasher.update(&byte_count.to_be_bytes());
    hasher.update(value.as_bytes());
    hasher.update(&[FIELD_SEPARATOR]);
    Ok(())
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    const ROOT: &str = "/nix/store/aaaaaaaa-root";
    const CHILD: &str = "/nix/store/bbbbbbbb-child";
    const DEAD: &str = "/nix/store/cccccccc-dead";
    const MISSING: &str = "/nix/store/dddddddd-missing";
    const ROOT_BYTES: u64 = 10;
    const CHILD_BYTES: u64 = 20;
    const DEAD_BYTES: u64 = 30;
    const EXPECTED_RETAINED: usize = 2;
    const EXPECTED_CANDIDATES: usize = 1;

    fn entry(path_id: &str, references: &[&str], declared_nar_bytes: u64) -> GcEntry {
        GcEntry {
            path_id: path_id.to_string(),
            references: references.iter().map(|value| (*value).to_string()).collect(),
            declared_nar_bytes,
            ownership: GcOwnership::Overlay,
        }
    }

    fn base_entry(path_id: &str, references: &[&str], declared_nar_bytes: u64) -> GcEntry {
        let mut value = entry(path_id, references, declared_nar_bytes);
        value.ownership = GcOwnership::Base { layer_index: 1 };
        value
    }

    fn request(mode: GcExecutionMode) -> GcPlanRequest {
        GcPlanRequest {
            roots: vec![ROOT.to_string()],
            entries: vec![
                entry(DEAD, &[], DEAD_BYTES),
                entry(ROOT, &[CHILD], ROOT_BYTES),
                entry(CHILD, &[], CHILD_BYTES),
            ],
            execution_mode: mode,
        }
    }

    #[test]
    fn reachability_and_candidates_are_canonical() {
        let plan = plan_gc(request(GcExecutionMode::Execute)).expect("valid graph must plan");
        assert_eq!(plan.retained_path_ids, vec![ROOT.to_string(), CHILD.to_string()]);
        assert_eq!(plan.candidate_path_ids, vec![DEAD.to_string()]);
        assert_eq!(plan.reclaim_summary.declared_nar_bytes, DEAD_BYTES);
        assert_eq!(plan.retained_path_ids.len(), EXPECTED_RETAINED);
        assert_eq!(plan.candidate_path_ids.len(), EXPECTED_CANDIDATES);
        assert_eq!(plan.mutation_intents[0].disposition, GcMutationDisposition::Execute);
    }

    #[test]
    fn shuffled_facts_produce_the_same_plan_identity() {
        let expected = plan_gc(request(GcExecutionMode::Execute)).expect("valid graph must plan");
        let mut shuffled = request(GcExecutionMode::Execute);
        shuffled.entries.reverse();
        shuffled.roots.push(ROOT.to_string());
        let actual = plan_gc(shuffled).expect("equivalent graph must plan");
        assert_eq!(actual, expected);
    }

    #[test]
    fn overlapping_roots_and_cycles_terminate_without_duplicate_retention() {
        let cycle = GcPlanRequest {
            roots: vec![ROOT.to_string(), CHILD.to_string()],
            entries: vec![
                entry(ROOT, &[CHILD], ROOT_BYTES),
                entry(CHILD, &[ROOT], CHILD_BYTES),
                entry(DEAD, &[], DEAD_BYTES),
            ],
            execution_mode: GcExecutionMode::DryRun,
        };
        let plan = plan_gc(cycle).expect("bounded cycle must plan");
        assert_eq!(plan.retained_path_ids, vec![ROOT.to_string(), CHILD.to_string()]);
        assert_eq!(plan.candidate_path_ids, vec![DEAD.to_string()]);
    }

    #[test]
    fn dry_run_preserves_candidates_without_execution_authority() {
        let dry_run = plan_gc(request(GcExecutionMode::DryRun)).expect("dry run must plan");
        let execute = plan_gc(request(GcExecutionMode::Execute)).expect("execution must plan");
        assert_eq!(dry_run.candidate_path_ids, execute.candidate_path_ids);
        assert_eq!(dry_run.mutation_intents[0].disposition, GcMutationDisposition::ReportOnly);
        assert_eq!(dry_run.plan_id, execute.plan_id);
    }

    #[test]
    fn empty_execute_plan_keeps_execution_disposition() {
        let plan = plan_gc(GcPlanRequest {
            roots: vec![],
            entries: vec![],
            execution_mode: GcExecutionMode::Execute,
        })
        .expect("empty store must plan");
        assert!(plan.mutation_intents.is_empty());
        assert_eq!(report_decision(plan).mutation_disposition, GcMutationDisposition::Execute);
    }

    #[test]
    fn missing_reference_is_rejected() {
        let mut invalid = request(GcExecutionMode::DryRun);
        invalid.entries[1].references = vec![MISSING.to_string()];
        assert_eq!(
            plan_gc(invalid),
            Err(GcPlanError::MissingReference {
                owner_path_id: ROOT.to_string(),
                reference_path_id: MISSING.to_string(),
            })
        );
    }

    #[test]
    fn duplicate_identity_is_rejected() {
        let mut invalid = request(GcExecutionMode::DryRun);
        invalid.entries.push(entry(ROOT, &[], ROOT_BYTES));
        assert_eq!(
            plan_gc(invalid),
            Err(GcPlanError::DuplicateEntry {
                path_id: ROOT.to_string(),
            })
        );
    }

    #[test]
    fn missing_root_is_rejected() {
        let mut invalid = request(GcExecutionMode::DryRun);
        invalid.roots = vec![MISSING.to_string()];
        assert_eq!(
            plan_gc(invalid),
            Err(GcPlanError::MissingRoot {
                path_id: MISSING.to_string(),
            })
        );
    }

    #[test]
    fn observed_reclaim_summary_uses_checked_arithmetic() {
        let summary =
            summarize_reclaim_observations(vec![ROOT_BYTES, CHILD_BYTES]).expect("bounded observations must summarize");
        assert_eq!(summary.observation_count, EXPECTED_RETAINED);
        assert_eq!(summary.reclaimable_bytes, ROOT_BYTES + CHILD_BYTES);

        assert_eq!(summarize_reclaim_observations(vec![u64::MAX, 1]), Err(GcPlanError::ReclaimSizeOverflow));
    }

    #[test]
    fn noncanonical_path_identity_is_rejected() {
        let invalid_path = "/nix/store/../escape";
        let invalid = GcPlanRequest {
            roots: vec![],
            entries: vec![entry(invalid_path, &[], ROOT_BYTES)],
            execution_mode: GcExecutionMode::DryRun,
        };
        assert_eq!(
            plan_gc(invalid),
            Err(GcPlanError::InvalidPathId {
                path_id: invalid_path.to_string(),
            })
        );
    }

    #[test]
    fn reclaim_overflow_is_rejected() {
        let first = "/nix/store/eeeeeeee-first";
        let second = "/nix/store/ffffffff-second";
        let invalid = GcPlanRequest {
            roots: vec![],
            entries: vec![entry(first, &[], u64::MAX), entry(second, &[], 1)],
            execution_mode: GcExecutionMode::DryRun,
        };
        assert_eq!(plan_gc(invalid), Err(GcPlanError::ReclaimSizeOverflow));
    }

    #[test]
    fn overlay_root_can_reach_base_without_base_mutation_intent() {
        let plan = plan_gc(GcPlanRequest {
            roots: vec![ROOT.to_string()],
            entries: vec![entry(ROOT, &[CHILD], ROOT_BYTES), base_entry(CHILD, &[], CHILD_BYTES)],
            execution_mode: GcExecutionMode::Execute,
        })
        .expect("overlay-to-base reachability must plan");
        assert_eq!(plan.retained_path_ids, vec![ROOT.to_string(), CHILD.to_string()]);
        assert_eq!(plan.base_path_ids, vec![CHILD.to_string()]);
        assert!(plan.candidate_path_ids.is_empty());
        assert!(plan.mutation_intents.is_empty());
    }

    #[test]
    fn unreachable_base_is_reported_but_never_collected() {
        let plan = plan_gc(GcPlanRequest {
            roots: vec![],
            entries: vec![entry(DEAD, &[], DEAD_BYTES), base_entry(CHILD, &[], CHILD_BYTES)],
            execution_mode: GcExecutionMode::Execute,
        })
        .expect("unreachable base fact must remain external");
        assert_eq!(plan.base_path_ids, vec![CHILD.to_string()]);
        assert_eq!(plan.candidate_path_ids, vec![DEAD.to_string()]);
        assert_eq!(plan.mutation_intents.len(), EXPECTED_CANDIDATES);
    }

    #[test]
    fn base_to_overlay_reference_is_rejected() {
        let invalid = GcPlanRequest {
            roots: vec![CHILD.to_string()],
            entries: vec![base_entry(CHILD, &[ROOT], CHILD_BYTES), entry(ROOT, &[], ROOT_BYTES)],
            execution_mode: GcExecutionMode::DryRun,
        };
        assert_eq!(
            plan_gc(invalid),
            Err(GcPlanError::BaseToOverlayReference {
                owner_path_id: CHILD.to_string(),
                reference_path_id: ROOT.to_string(),
                base_layer_index: 1,
            })
        );
    }
}
