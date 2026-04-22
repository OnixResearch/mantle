use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::LockEntry;
use crate::LockedPatch;
use crate::LockedPatchSource;
use crate::Lockfile;
use crate::MAX_INPUTS;
use crate::MAX_LOCKED_PATCHES;
use crate::ManifestInput;
use crate::PatchDef;
use crate::PatchSource;
use crate::ProjectManifest;

const MAX_REFRESH_BATCH: u32 = 256;

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedInput {
    pub name: String,
    pub entry: LockEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HashResolutionMode {
    Flat,
    Recursive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshFailure {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleReport {
    pub stale: Vec<String>,
    pub failed: Vec<RefreshFailure>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RefreshOutcome {
    Updated(ResolvedInput),
    Unchanged { name: String },
    Frozen { name: String },
    Failed { name: String, reason: String },
}

impl RefreshOutcome {
    pub fn name(&self) -> &str {
        match self {
            RefreshOutcome::Updated(resolved) => &resolved.name,
            RefreshOutcome::Unchanged { name } => name,
            RefreshOutcome::Frozen { name } => name,
            RefreshOutcome::Failed { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedInputState {
    Resolved(ResolvedInput),
    Failed(RefreshFailure),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefreshInputsPlanRequest {
    pub manifest: ProjectManifest,
    pub selected: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefreshInputsRequest {
    pub manifest: ProjectManifest,
    pub lock: Lockfile,
    pub selected: Vec<String>,
    pub resolutions: Vec<ResolvedInputState>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatchResolutionPlanRequest {
    pub manifest: ProjectManifest,
    pub lock: Lockfile,
    pub outcomes: Vec<RefreshOutcome>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatchResolution {
    Resolved { name: String, patch: LockedPatch },
    Failed { name: String, reason: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplyOutcomesRequest {
    pub manifest: ProjectManifest,
    pub lock: Lockfile,
    pub outcomes: Vec<RefreshOutcome>,
    pub patch_resolutions: Vec<PatchResolution>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplyResult {
    pub lock: Lockfile,
    pub inputs_changed: u32,
    pub patches_changed: bool,
    pub failures: Vec<RefreshFailure>,
}

impl ApplyResult {
    pub fn has_changes(&self) -> bool {
        self.inputs_changed > 0 || self.patches_changed
    }
}

pub fn plan_refresh_inputs(request: RefreshInputsPlanRequest) -> Vec<ManifestInput> {
    select_inputs(&request.manifest, &request.selected).into_iter().cloned().collect()
}

pub fn refresh_inputs(request: RefreshInputsRequest) -> Vec<RefreshOutcome> {
    let inputs_to_refresh = select_inputs(&request.manifest, &request.selected);
    assert!(
        inputs_to_refresh.len() as u64 <= MAX_REFRESH_BATCH as u64,
        "too many inputs to refresh: {}",
        inputs_to_refresh.len()
    );
    let resolution_map = input_resolution_map(request.resolutions);

    inputs_to_refresh
        .into_iter()
        .map(|input| refresh_one(input, &request.lock, &resolution_map))
        .collect()
}

pub fn list_stale(request: RefreshInputsRequest) -> StaleReport {
    let outcomes = refresh_inputs(request);
    let stale = outcomes
        .iter()
        .filter_map(|outcome| match outcome {
            RefreshOutcome::Updated(resolved) => Some(resolved.name.clone()),
            _ => None,
        })
        .collect();
    let failed = refresh_failures_from_outcomes(&outcomes);
    StaleReport { stale, failed }
}

pub fn plan_patch_resolutions(request: PatchResolutionPlanRequest) -> Vec<PatchDef> {
    let mut staged_lock = request.lock;
    apply_updated_outcomes(&mut staged_lock, &request.outcomes);
    compute_needed_patch_resolutions(&request.manifest, &staged_lock)
}

pub fn apply_outcomes(request: ApplyOutcomesRequest) -> ApplyResult {
    assert!(request.lock.inputs.len() as u64 <= MAX_INPUTS as u64, "lock inputs must stay within manifest limit");
    let mut new_lock = request.lock.clone();
    let inputs_changed = apply_updated_outcomes(&mut new_lock, &request.outcomes);

    let planned_patches = compute_needed_patch_resolutions(&request.manifest, &new_lock);
    let patch_resolution_map = patch_resolution_map(request.patch_resolutions);
    let mut patch_failures = Vec::new();
    let mut patch_changed = false;
    for patch_def in planned_patches {
        let patch_name = patch_def.name.clone();
        match patch_resolution_map.get(patch_name.as_str()) {
            Some(PatchResolution::Resolved { patch, .. }) => {
                new_lock.patches.insert(patch_name, patch.clone());
                patch_changed = true;
            }
            Some(PatchResolution::Failed { reason, .. }) => patch_failures.push(RefreshFailure {
                name: patch_name,
                reason: reason.clone(),
            }),
            None => patch_failures.push(RefreshFailure {
                name: patch_name,
                reason: "missing patch resolution".to_string(),
            }),
        }
    }

    let reverted = revert_failed_patch_inputs(&request.lock, &mut new_lock, &patch_failures);
    let inputs_changed = inputs_changed.saturating_sub(reverted);
    let orphan_changed = remove_orphaned_patches(&mut new_lock);

    ApplyResult {
        lock: new_lock,
        inputs_changed,
        patches_changed: patch_changed || orphan_changed,
        failures: patch_failures,
    }
}

fn select_inputs<'a>(manifest: &'a ProjectManifest, selected: &[String]) -> Vec<&'a ManifestInput> {
    if selected.is_empty() {
        return manifest.inputs.iter().collect();
    }
    let selected_set: BTreeSet<&str> = selected.iter().map(|value| value.as_str()).collect();
    manifest.inputs.iter().filter(|input| selected_set.contains(input.name.as_str())).collect()
}

fn input_resolution_map(resolutions: Vec<ResolvedInputState>) -> BTreeMap<String, ResolvedInputState> {
    let mut map = BTreeMap::new();
    for resolution in resolutions {
        let key = match &resolution {
            ResolvedInputState::Resolved(resolved) => resolved.name.clone(),
            ResolvedInputState::Failed(failure) => failure.name.clone(),
        };
        map.insert(key, resolution);
    }
    map
}

fn refresh_one(
    input: &ManifestInput,
    lock: &Lockfile,
    resolutions: &BTreeMap<String, ResolvedInputState>,
) -> RefreshOutcome {
    assert!(!input.name.is_empty(), "input name must not be empty");
    assert!(
        input.mirrors.len() as u64 <= crate::MAX_MIRRORS_PER_INPUT as u64,
        "mirror count must stay within manifest limit"
    );
    if input.frozen {
        return RefreshOutcome::Frozen {
            name: input.name.clone(),
        };
    }

    match resolutions.get(input.name.as_str()) {
        Some(ResolvedInputState::Resolved(resolved)) => {
            if let Some(existing) = lock.inputs.get(&input.name)
                && existing == &resolved.entry
            {
                return RefreshOutcome::Unchanged {
                    name: input.name.clone(),
                };
            }
            RefreshOutcome::Updated(resolved.clone())
        }
        Some(ResolvedInputState::Failed(failure)) => RefreshOutcome::Failed {
            name: failure.name.clone(),
            reason: failure.reason.clone(),
        },
        None => RefreshOutcome::Failed {
            name: input.name.clone(),
            reason: alloc::format!("missing resolution for {}", input.name),
        },
    }
}

fn apply_updated_outcomes(lock: &mut Lockfile, outcomes: &[RefreshOutcome]) -> u32 {
    let mut inputs_changed: u32 = 0;
    for outcome in outcomes {
        if let RefreshOutcome::Updated(resolved) = outcome {
            lock.inputs.insert(resolved.name.clone(), resolved.entry.clone());
            inputs_changed = inputs_changed.saturating_add(1);
        }
    }
    inputs_changed
}

fn compute_needed_patch_resolutions(manifest: &ProjectManifest, lock: &Lockfile) -> Vec<PatchDef> {
    assert!(
        manifest.patches.len() as u64 <= MAX_LOCKED_PATCHES as u64,
        "manifest patch count must stay within lock limit"
    );
    let defs: BTreeMap<&str, &PatchDef> = manifest.patches.iter().map(|def| (def.name.as_str(), def)).collect();
    collect_needed_patches(lock)
        .into_iter()
        .filter_map(|name| {
            let def = defs.get(name.as_str())?;
            let needs_resolve = match lock.patches.get(name.as_str()) {
                None => true,
                Some(existing) => !patch_matches_def(existing, def),
            };
            if needs_resolve { Some((*def).clone()) } else { None }
        })
        .collect()
}

fn patch_resolution_map(resolutions: Vec<PatchResolution>) -> BTreeMap<String, PatchResolution> {
    let mut map = BTreeMap::new();
    for resolution in resolutions {
        let key = match &resolution {
            PatchResolution::Resolved { name, .. } => name.clone(),
            PatchResolution::Failed { name, .. } => name.clone(),
        };
        map.insert(key, resolution);
    }
    map
}

fn collect_needed_patches(lock: &Lockfile) -> BTreeSet<String> {
    let mut needed = BTreeSet::new();
    for entry in lock.inputs.values() {
        for patch_name in &entry.patches {
            needed.insert(patch_name.clone());
        }
    }
    needed
}

fn remove_orphaned_patches(lock: &mut Lockfile) -> bool {
    let needed = collect_needed_patches(lock);
    let before_len = lock.patches.len() as u32;
    lock.patches.retain(|name, _| needed.contains(name));
    lock.patches.len() as u32 != before_len
}

fn revert_failed_patch_inputs(old_lock: &Lockfile, new_lock: &mut Lockfile, failures: &[RefreshFailure]) -> u32 {
    if failures.is_empty() {
        return 0;
    }
    let failed_names: BTreeSet<&str> = failures.iter().map(|failure| failure.name.as_str()).collect();
    let impacted_inputs: Vec<String> = new_lock
        .inputs
        .iter()
        .filter(|(_, entry)| entry.patches.iter().any(|patch_name| failed_names.contains(patch_name.as_str())))
        .map(|(name, _)| name.clone())
        .collect();
    let mut reverted: u32 = 0;
    for name in impacted_inputs {
        match old_lock.inputs.get(&name) {
            Some(previous) => {
                let current = new_lock.inputs.get(&name);
                if current != Some(previous) {
                    new_lock.inputs.insert(name, previous.clone());
                    reverted = reverted.saturating_add(1);
                }
            }
            None => {
                if new_lock.inputs.remove(&name).is_some() {
                    reverted = reverted.saturating_add(1);
                }
            }
        }
    }
    reverted
}

fn patch_matches_def(locked: &LockedPatch, def: &PatchDef) -> bool {
    match (&locked.source, &def.source) {
        (LockedPatchSource::Local { path: locked_path }, PatchSource::Local { path: def_path }) => {
            locked_path == def_path
        }
        (
            LockedPatchSource::Remote { url: locked_url },
            PatchSource::Remote {
                url: def_url,
                hash: def_hash,
            },
        ) => {
            if locked_url != def_url {
                return false;
            }
            if locked.hash.algo != def_hash.algo {
                return false;
            }
            if let Some(expected) = &def_hash.expected
                && locked.hash.value != *expected
            {
                return false;
            }
            true
        }
        _ => false,
    }
}

fn refresh_failures_from_outcomes(outcomes: &[RefreshOutcome]) -> Vec<RefreshFailure> {
    outcomes
        .iter()
        .filter_map(|outcome| match outcome {
            RefreshOutcome::Failed { name, reason } => Some(RefreshFailure {
                name: name.clone(),
                reason: reason.clone(),
            }),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use alloc::vec;

    use super::*;
    use crate::HashAlgo;
    use crate::HashSpec;
    use crate::InputKind;
    use crate::LockedHash;
    use crate::LockedKind;
    use crate::ManifestInput;
    use crate::PatchSource;
    use crate::SchemaVersion;

    fn file_input(name: &str, url: &str) -> ManifestInput {
        ManifestInput {
            name: name.into(),
            kind: InputKind::File { url: url.into() },
            hash: HashSpec::default(),
            frozen: false,
            mirrors: vec![],
            patches: vec![],
        }
    }

    fn manifest_with(inputs: Vec<ManifestInput>) -> ProjectManifest {
        ProjectManifest {
            version: "1.0.0".into(),
            inputs,
            patches: vec![],
        }
    }

    fn empty_lock() -> Lockfile {
        Lockfile {
            version: SchemaVersion::CURRENT,
            inputs: BTreeMap::new(),
            patches: BTreeMap::new(),
        }
    }

    fn resolved_file(name: &str, url: &str, hash: &str) -> ResolvedInputState {
        ResolvedInputState::Resolved(ResolvedInput {
            name: name.to_string(),
            entry: LockEntry {
                kind: LockedKind::File { url: url.to_string() },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: hash.to_string(),
                },
                patches: vec![],
                mirrors: vec![],
            },
        })
    }

    #[test]
    fn refresh_inputs_marks_resolved_entry_updated() {
        let manifest = manifest_with(vec![file_input("data", "https://example.com/data.bin")]);
        let outcomes = refresh_inputs(RefreshInputsRequest {
            manifest,
            lock: empty_lock(),
            selected: Vec::new(),
            resolutions: vec![resolved_file("data", "https://example.com/data.bin", "sha256-abc=")],
        });

        assert_eq!(outcomes.len(), 1);
        match &outcomes[0] {
            RefreshOutcome::Updated(resolved) => {
                assert_eq!(resolved.name, "data");
                assert_eq!(resolved.entry.hash.value, "sha256-abc=");
            }
            other => panic!("expected Updated, got {other:?}"),
        }
    }

    #[test]
    fn refresh_inputs_marks_matching_entry_unchanged() {
        let manifest = manifest_with(vec![file_input("data", "https://example.com/data.bin")]);
        let mut lock = empty_lock();
        lock.inputs.insert("data".into(), LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/data.bin".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-same=".into(),
            },
            patches: vec![],
            mirrors: vec![],
        });

        let outcomes = refresh_inputs(RefreshInputsRequest {
            manifest,
            lock,
            selected: Vec::new(),
            resolutions: vec![resolved_file("data", "https://example.com/data.bin", "sha256-same=")],
        });
        assert!(matches!(&outcomes[0], RefreshOutcome::Unchanged { .. }));
    }

    #[test]
    fn refresh_inputs_marks_frozen_without_resolution() {
        let mut input = file_input("frozen", "https://example.com/f");
        input.frozen = true;
        let outcomes = refresh_inputs(RefreshInputsRequest {
            manifest: manifest_with(vec![input]),
            lock: empty_lock(),
            selected: Vec::new(),
            resolutions: Vec::new(),
        });
        assert!(matches!(&outcomes[0], RefreshOutcome::Frozen { name } if name == "frozen"));
    }

    #[test]
    fn plan_refresh_inputs_filters_selected_inputs() {
        let manifest = manifest_with(vec![
            file_input("a", "https://example.com/a"),
            file_input("b", "https://example.com/b"),
            file_input("c", "https://example.com/c"),
        ]);
        let plan = plan_refresh_inputs(RefreshInputsPlanRequest {
            manifest,
            selected: vec!["a".into(), "c".into()],
        });
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].name, "a");
        assert_eq!(plan[1].name, "c");
    }

    #[test]
    fn apply_outcomes_updates_lock_and_resolves_patches() {
        let manifest = ProjectManifest {
            version: "1.0.0".into(),
            inputs: vec![ManifestInput {
                name: "pkg".into(),
                kind: InputKind::File {
                    url: "https://example.com/pkg".into(),
                },
                hash: HashSpec::default(),
                frozen: false,
                mirrors: vec![],
                patches: vec!["fix1".into()],
            }],
            patches: vec![PatchDef {
                name: "fix1".into(),
                source: PatchSource::Local {
                    path: "patches/fix1.patch".into(),
                },
            }],
        };
        let outcomes = vec![RefreshOutcome::Updated(ResolvedInput {
            name: "pkg".into(),
            entry: LockEntry {
                kind: LockedKind::File {
                    url: "https://example.com/pkg".into(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-pkghash=".into(),
                },
                patches: vec!["fix1".into()],
                mirrors: vec![],
            },
        })];
        let patch_plan = plan_patch_resolutions(PatchResolutionPlanRequest {
            manifest: manifest.clone(),
            lock: empty_lock(),
            outcomes: outcomes.clone(),
        });
        assert_eq!(patch_plan.len(), 1);
        let result = apply_outcomes(ApplyOutcomesRequest {
            manifest,
            lock: empty_lock(),
            outcomes,
            patch_resolutions: vec![PatchResolution::Resolved {
                name: "fix1".into(),
                patch: LockedPatch {
                    source: LockedPatchSource::Local {
                        path: "patches/fix1.patch".into(),
                    },
                    hash: LockedHash {
                        algo: HashAlgo::Sha256,
                        value: "sha256-patchhash=".into(),
                    },
                },
            }],
        });

        assert!(result.lock.inputs.contains_key("pkg"));
        assert!(result.lock.patches.contains_key("fix1"));
        assert!(result.patches_changed);
        assert_eq!(result.inputs_changed, 1);
    }

    #[test]
    fn apply_outcomes_reverts_input_when_patch_resolution_fails() {
        let manifest = ProjectManifest {
            version: "1.0.0".into(),
            inputs: vec![ManifestInput {
                name: "pkg".into(),
                kind: InputKind::File {
                    url: "https://example.com/pkg".into(),
                },
                hash: HashSpec::default(),
                frozen: false,
                mirrors: vec![],
                patches: vec!["bad".into()],
            }],
            patches: vec![PatchDef {
                name: "bad".into(),
                source: PatchSource::Local {
                    path: "patches/bad.patch".into(),
                },
            }],
        };
        let outcomes = vec![RefreshOutcome::Updated(ResolvedInput {
            name: "pkg".into(),
            entry: LockEntry {
                kind: LockedKind::File {
                    url: "https://example.com/pkg".into(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-pkghash=".into(),
                },
                patches: vec!["bad".into()],
                mirrors: vec![],
            },
        })];

        let result = apply_outcomes(ApplyOutcomesRequest {
            manifest,
            lock: empty_lock(),
            outcomes,
            patch_resolutions: vec![PatchResolution::Failed {
                name: "bad".into(),
                reason: "hash failed".into(),
            }],
        });

        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].name, "bad");
        assert!(!result.lock.inputs.contains_key("pkg"));
        assert!(!result.lock.patches.contains_key("bad"));
    }

    #[test]
    fn apply_outcomes_removes_orphaned_patches() {
        let manifest = manifest_with(vec![file_input("pkg", "https://example.com/pkg")]);
        let mut lock = empty_lock();
        lock.patches.insert("old-patch".into(), LockedPatch {
            source: LockedPatchSource::Local {
                path: "old.patch".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-old=".into(),
            },
        });
        let result = apply_outcomes(ApplyOutcomesRequest {
            manifest,
            lock,
            outcomes: vec![RefreshOutcome::Updated(ResolvedInput {
                name: "pkg".into(),
                entry: LockEntry {
                    kind: LockedKind::File {
                        url: "https://example.com/pkg".into(),
                    },
                    hash: LockedHash {
                        algo: HashAlgo::Sha256,
                        value: "sha256-h=".into(),
                    },
                    patches: vec![],
                    mirrors: vec![],
                },
            })],
            patch_resolutions: Vec::new(),
        });

        assert!(!result.lock.patches.contains_key("old-patch"));
        assert!(result.patches_changed);
    }

    #[test]
    fn list_stale_separates_changed_and_failed_inputs() {
        let manifest = manifest_with(vec![
            file_input("fresh", "https://example.com/fresh"),
            file_input("stale", "https://example.com/stale"),
            file_input("broken", "https://example.com/broken"),
        ]);
        let mut lock = empty_lock();
        lock.inputs.insert("fresh".into(), LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/fresh".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-current=".into(),
            },
            patches: vec![],
            mirrors: vec![],
        });
        let stale = list_stale(RefreshInputsRequest {
            manifest,
            lock,
            selected: Vec::new(),
            resolutions: vec![
                resolved_file("fresh", "https://example.com/fresh", "sha256-current="),
                resolved_file("stale", "https://example.com/stale", "sha256-current="),
                ResolvedInputState::Failed(RefreshFailure {
                    name: "broken".into(),
                    reason: "network down".into(),
                }),
            ],
        });
        assert_eq!(stale.stale, vec!["stale"]);
        assert_eq!(stale.failed.len(), 1);
        assert_eq!(stale.failed[0].name, "broken");
    }
}
