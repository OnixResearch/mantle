//! Refresh and stale detection for project inputs.
//!
//! The refresh module resolves new revisions/hashes for project inputs
//! and produces updated lock entries. Actual fetching (download, hash
//! verification) stays in the build pipeline. This module resolves
//! *metadata* — URL expansion, git rev resolution via ls-remote, and
//! hash recording.
//!
//! `RefreshResolver` is a trait that callers implement to provide
//! network-dependent resolution. The pure refresh logic lives here;
//! the I/O lives in the caller.

use crate::error::Error;
use crate::lock::{
    LockEntry, LockedHash, LockedKind, LockedPatch, LockedPatchSource, Lockfile,
};
use crate::manifest::{
    GitReference, HashAlgo, InputKind, ManifestInput, PatchDef, PatchSource,
    ProjectManifest,
};

/// Maximum number of inputs to refresh in a single call.
const MAX_REFRESH_BATCH: u32 = 256;

/// Outcome of resolving a single input.
#[derive(Debug, Clone)]
pub struct ResolvedInput {
    /// The input name.
    pub name: String,
    /// The resolved lock entry.
    pub entry: LockEntry,
}

/// Outcome of refreshing a single input.
#[derive(Debug, Clone)]
pub enum RefreshOutcome {
    /// Input was updated with new resolved state.
    Updated(ResolvedInput),
    /// Input is unchanged (lock entry matches current resolution).
    Unchanged { name: String },
    /// Input was skipped because it is frozen.
    Frozen { name: String },
    /// Resolution failed for this input.
    Failed { name: String, reason: String },
}

impl RefreshOutcome {
    pub fn name(&self) -> &str {
        match self {
            RefreshOutcome::Updated(r) => &r.name,
            RefreshOutcome::Unchanged { name } => name,
            RefreshOutcome::Frozen { name } => name,
            RefreshOutcome::Failed { name, .. } => name,
        }
    }
}

/// Trait for resolving input metadata from external sources.
///
/// Callers implement this to provide git rev lookups, URL content
/// hashing, etc. The trait is intentionally synchronous — async
/// callers can block or wrap.
pub trait RefreshResolver {
    /// Resolve a git reference to a concrete revision hash.
    ///
    /// Returns `Ok(Some(rev))` if resolved, `Ok(None)` if the reference
    /// couldn't be resolved (repo unreachable, ref doesn't exist), or
    /// `Err` on hard failures.
    fn resolve_git_rev(
        &self,
        repository: &str,
        reference: &GitReference,
    ) -> Result<Option<String>, Error>;

    /// Compute the hash of a URL's content.
    ///
    /// Returns `Ok(Some(sri_hash))` if the content was fetched and
    /// hashed, `Ok(None)` if unavailable, or `Err` on hard failures.
    fn hash_url_content(
        &self,
        url: &str,
        algo: &HashAlgo,
    ) -> Result<Option<String>, Error>;

    /// Compute the hash of a local file's content.
    ///
    /// Used for local patches. Returns `Ok(Some(sri_hash))` if the file
    /// was hashed, `Ok(None)` if unavailable.
    fn hash_local_file(
        &self,
        path: &str,
        algo: &HashAlgo,
    ) -> Result<Option<String>, Error> {
        // Default: not available. Callers override for real I/O.
        let _ = (path, algo);
        Ok(None)
    }
}

/// Refresh selected inputs in a manifest.
///
/// `selected` is a list of input names to refresh. If empty, refreshes
/// all non-frozen inputs. Returns one outcome per input processed.
pub fn refresh_inputs(
    manifest: &ProjectManifest,
    lock: &Lockfile,
    selected: &[String],
    resolver: &dyn RefreshResolver,
) -> Vec<RefreshOutcome> {
    let inputs_to_refresh = select_inputs(manifest, selected);
    assert!(
        inputs_to_refresh.len() as u64 <= MAX_REFRESH_BATCH as u64,
        "too many inputs to refresh: {}",
        inputs_to_refresh.len()
    );

    inputs_to_refresh
        .iter()
        .map(|input| refresh_one(input, lock, resolver))
        .collect()
}

/// Select which inputs to refresh based on the `selected` list.
fn select_inputs<'a>(
    manifest: &'a ProjectManifest,
    selected: &[String],
) -> Vec<&'a ManifestInput> {
    if selected.is_empty() {
        manifest
            .inputs
            .iter()
            .collect()
    } else {
        let selected_set: std::collections::HashSet<&str> =
            selected.iter().map(|s| s.as_str()).collect();
        manifest
            .inputs
            .iter()
            .filter(|i| selected_set.contains(i.name.as_str()))
            .collect()
    }
}

/// Refresh a single input.
fn refresh_one(
    input: &ManifestInput,
    lock: &Lockfile,
    resolver: &dyn RefreshResolver,
) -> RefreshOutcome {
    if input.frozen {
        return RefreshOutcome::Frozen {
            name: input.name.clone(),
        };
    }

    match resolve_input(input, resolver) {
        Ok(entry) => {
            // Compare with existing lock entry
            if let Some(existing) = lock.inputs.get(&input.name) {
                if existing == &entry {
                    return RefreshOutcome::Unchanged {
                        name: input.name.clone(),
                    };
                }
            }
            RefreshOutcome::Updated(ResolvedInput {
                name: input.name.clone(),
                entry,
            })
        }
        Err(e) => RefreshOutcome::Failed {
            name: input.name.clone(),
            reason: e.to_string(),
        },
    }
}

/// Resolve a manifest input to a lock entry.
fn resolve_input(
    input: &ManifestInput,
    resolver: &dyn RefreshResolver,
) -> Result<LockEntry, Error> {
    let (kind, hash) = match &input.kind {
        InputKind::File { url } => {
            let hash_value = resolver
                .hash_url_content(url, &input.hash.algo)?
                .or_else(|| input.hash.expected.clone())
                .unwrap_or_default();
            (
                LockedKind::File { url: url.clone() },
                LockedHash {
                    algo: input.hash.algo.clone(),
                    value: hash_value,
                },
            )
        }
        InputKind::Tarball { url } => {
            let hash_value = resolver
                .hash_url_content(url, &input.hash.algo)?
                .or_else(|| input.hash.expected.clone())
                .unwrap_or_default();
            (
                LockedKind::Tarball { url: url.clone() },
                LockedHash {
                    algo: input.hash.algo.clone(),
                    value: hash_value,
                },
            )
        }
        InputKind::Git {
            repository,
            reference,
        } => {
            let rev = resolver
                .resolve_git_rev(repository, reference)?
                .unwrap_or_default();
            let ref_name = match reference {
                GitReference::Branch(b) => Some(b.clone()),
                GitReference::Tag(t) => Some(t.clone()),
                GitReference::Rev(_) => None,
            };
            // For git, hash comes from the expected value or stays empty
            // until the build pipeline fetches and hashes the content.
            let hash_value = input.hash.expected.clone().unwrap_or_default();
            (
                LockedKind::Git {
                    repository: repository.clone(),
                    rev,
                    ref_name,
                },
                LockedHash {
                    algo: input.hash.algo.clone(),
                    value: hash_value,
                },
            )
        }
    };

    Ok(LockEntry {
        kind,
        hash,
        patches: input.patches.clone(),
        mirrors: input.mirrors.clone(),
    })
}

/// Result of applying refresh outcomes.
pub struct ApplyResult {
    /// The updated lockfile.
    pub lock: Lockfile,
    /// Number of input entries that were updated.
    pub inputs_changed: u32,
    /// Whether patch lock data was added, changed, or removed.
    pub patches_changed: bool,
}

impl ApplyResult {
    /// Whether anything changed (inputs or patches).
    pub fn has_changes(&self) -> bool {
        self.inputs_changed > 0 || self.patches_changed
    }
}

/// Apply refresh outcomes to a lockfile, producing a new lockfile.
///
/// Updates input entries AND resolves manifest patches into
/// `Lockfile.patches`. Patch definitions referenced by any input
/// are resolved from the manifest and locked.
pub fn apply_outcomes(
    manifest: &ProjectManifest,
    lock: &Lockfile,
    outcomes: &[RefreshOutcome],
    resolver: &dyn RefreshResolver,
) -> ApplyResult {
    let mut new_lock = lock.clone();
    let mut inputs_changed: u32 = 0;
    for outcome in outcomes {
        if let RefreshOutcome::Updated(resolved) = outcome {
            new_lock
                .inputs
                .insert(resolved.name.clone(), resolved.entry.clone());
            inputs_changed = inputs_changed.saturating_add(1);
        }
    }

    // Resolve all patches referenced by any input entry.
    let patches_changed = resolve_patches_into_lock(manifest, &mut new_lock, resolver);

    ApplyResult {
        lock: new_lock,
        inputs_changed,
        patches_changed,
    }
}

/// Resolve manifest patch definitions into locked patches.
///
/// For each patch name referenced by any lock entry, find the
/// manifest's `PatchDef` and lock it (compute hash, record source).
/// Re-resolves patches whose manifest definition changed (different
/// source path/url). Returns true if any patch was added or changed.
pub fn resolve_patches_into_lock(
    manifest: &ProjectManifest,
    lock: &mut Lockfile,
    resolver: &dyn RefreshResolver,
) -> bool {
    let mut changed = false;

    // Collect all patch names referenced by any lock entry.
    let mut needed: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for entry in lock.inputs.values() {
        for p in &entry.patches {
            needed.insert(p.clone());
        }
    }

    // Build index of manifest patch definitions.
    let defs: std::collections::HashMap<&str, &PatchDef> = manifest
        .patches
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();

    for name in &needed {
        let def = match defs.get(name.as_str()) {
            Some(d) => d,
            None => continue, // manifest doesn't define this patch
        };

        // Check if existing locked patch matches the manifest definition.
        let needs_resolve = match lock.patches.get(name) {
            None => true, // not locked yet
            Some(existing) => !patch_matches_def(existing, def),
        };

        if needs_resolve {
            if let Some(locked) = resolve_patch(def, resolver) {
                lock.patches.insert(name.clone(), locked);
                changed = true;
            }
        }
    }

    // Remove orphaned locked patches (no longer referenced).
    let before_len = lock.patches.len() as u32;
    lock.patches.retain(|name, _| needed.contains(name));
    if lock.patches.len() as u32 != before_len {
        changed = true;
    }

    changed
}

/// Check if a locked patch still matches its manifest definition.
fn patch_matches_def(locked: &LockedPatch, def: &PatchDef) -> bool {
    match (&locked.source, &def.source) {
        (
            LockedPatchSource::Local { path: locked_path },
            PatchSource::Local { path: def_path },
        ) => locked_path == def_path,
        (
            LockedPatchSource::Remote { url: locked_url },
            PatchSource::Remote { url: def_url, .. },
        ) => locked_url == def_url,
        _ => false, // type changed (local <-> remote)
    }
}

/// Resolve a single manifest patch definition to a locked patch.
///
/// Returns `None` if the hash cannot be resolved (resolver returns
/// None and no expected hash is available). Never produces a
/// `LockedPatch` with an empty hash value.
fn resolve_patch(
    def: &PatchDef,
    resolver: &dyn RefreshResolver,
) -> Option<LockedPatch> {
    match &def.source {
        PatchSource::Local { path } => {
            let hash_value = resolver
                .hash_local_file(path, &HashAlgo::Sha256)
                .ok()
                .flatten()?;
            assert!(!hash_value.is_empty(), "resolver returned empty hash");
            Some(LockedPatch {
                source: LockedPatchSource::Local {
                    path: path.clone(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: hash_value,
                },
            })
        }
        PatchSource::Remote { url, hash } => {
            let hash_value = resolver
                .hash_url_content(url, &hash.algo)
                .ok()
                .flatten()
                .or_else(|| hash.expected.clone())?;
            assert!(!hash_value.is_empty(), "resolver returned empty hash");
            Some(LockedPatch {
                source: LockedPatchSource::Remote {
                    url: url.clone(),
                },
                hash: LockedHash {
                    algo: hash.algo.clone(),
                    value: hash_value,
                },
            })
        }
    }
}

/// List inputs that are stale (would change on refresh) without
/// mutating anything.
///
/// Returns names of inputs whose resolved state differs from their
/// current lock entry.
pub fn list_stale(
    manifest: &ProjectManifest,
    lock: &Lockfile,
    resolver: &dyn RefreshResolver,
) -> Vec<String> {
    let outcomes = refresh_inputs(manifest, lock, &[], resolver);
    outcomes
        .iter()
        .filter_map(|o| match o {
            RefreshOutcome::Updated(r) => Some(r.name.clone()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::*;
    use crate::manifest::*;
    use crate::version::SchemaVersion;
    use std::collections::BTreeMap;

    /// A test resolver that returns fixed values.
    struct MockResolver {
        git_rev: Option<String>,
        url_hash: Option<String>,
    }

    impl RefreshResolver for MockResolver {
        fn resolve_git_rev(
            &self,
            _repository: &str,
            _reference: &GitReference,
        ) -> Result<Option<String>, Error> {
            Ok(self.git_rev.clone())
        }

        fn hash_url_content(
            &self,
            _url: &str,
            _algo: &HashAlgo,
        ) -> Result<Option<String>, Error> {
            Ok(self.url_hash.clone())
        }
    }

    fn file_input(name: &str, url: &str) -> ManifestInput {
        ManifestInput {
            name: name.into(),
            kind: InputKind::File {
                url: url.into(),
            },
            hash: HashSpec::default(),
            frozen: false,
            mirrors: vec![],
            patches: vec![],
        }
    }

    fn git_input(name: &str, repo: &str) -> ManifestInput {
        ManifestInput {
            name: name.into(),
            kind: InputKind::Git {
                repository: repo.into(),
                reference: GitReference::Branch("main".into()),
            },
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

    #[test]
    fn refresh_file_input_new() {
        let m = manifest_with(vec![file_input("data", "https://example.com/data.bin")]);
        let resolver = MockResolver {
            git_rev: None,
            url_hash: Some("sha256-abc=".into()),
        };

        let outcomes = refresh_inputs(&m, &empty_lock(), &[], &resolver);
        assert_eq!(outcomes.len(), 1);
        match &outcomes[0] {
            RefreshOutcome::Updated(r) => {
                assert_eq!(r.name, "data");
                assert_eq!(r.entry.hash.value, "sha256-abc=");
            }
            other => panic!("expected Updated, got {other:?}"),
        }
    }

    #[test]
    fn refresh_git_input_resolves_rev() {
        let m = manifest_with(vec![git_input("nixpkgs", "https://github.com/NixOS/nixpkgs.git")]);
        let resolver = MockResolver {
            git_rev: Some("deadbeef123".into()),
            url_hash: None,
        };

        let outcomes = refresh_inputs(&m, &empty_lock(), &[], &resolver);
        assert_eq!(outcomes.len(), 1);
        match &outcomes[0] {
            RefreshOutcome::Updated(r) => {
                assert_eq!(r.name, "nixpkgs");
                if let LockedKind::Git { rev, ref_name, .. } = &r.entry.kind {
                    assert_eq!(rev, "deadbeef123");
                    assert_eq!(ref_name.as_deref(), Some("main"));
                } else {
                    panic!("expected Git kind");
                }
            }
            other => panic!("expected Updated, got {other:?}"),
        }
    }

    #[test]
    fn refresh_frozen_input_skipped() {
        let mut input = file_input("frozen", "https://example.com/f");
        input.frozen = true;
        let m = manifest_with(vec![input]);
        let resolver = MockResolver {
            git_rev: None,
            url_hash: Some("sha256-new=".into()),
        };

        let outcomes = refresh_inputs(&m, &empty_lock(), &[], &resolver);
        assert_eq!(outcomes.len(), 1);
        assert!(matches!(&outcomes[0], RefreshOutcome::Frozen { name } if name == "frozen"));
    }

    #[test]
    fn refresh_unchanged_input() {
        let resolver = MockResolver {
            git_rev: None,
            url_hash: Some("sha256-same=".into()),
        };

        let m = manifest_with(vec![file_input("data", "https://example.com/data.bin")]);

        // Create a lock that already has the same resolved state
        let mut lock = empty_lock();
        lock.inputs.insert(
            "data".into(),
            LockEntry {
                kind: LockedKind::File {
                    url: "https://example.com/data.bin".into(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-same=".into(),
                },
                patches: vec![],
                mirrors: vec![],
            },
        );

        let outcomes = refresh_inputs(&m, &lock, &[], &resolver);
        assert_eq!(outcomes.len(), 1);
        assert!(matches!(&outcomes[0], RefreshOutcome::Unchanged { .. }));
    }

    #[test]
    fn refresh_selected_inputs_only() {
        let m = manifest_with(vec![
            file_input("a", "https://example.com/a"),
            file_input("b", "https://example.com/b"),
            file_input("c", "https://example.com/c"),
        ]);
        let resolver = MockResolver {
            git_rev: None,
            url_hash: Some("sha256-new=".into()),
        };

        let outcomes = refresh_inputs(
            &m,
            &empty_lock(),
            &["a".into(), "c".into()],
            &resolver,
        );
        assert_eq!(outcomes.len(), 2);
        assert_eq!(outcomes[0].name(), "a");
        assert_eq!(outcomes[1].name(), "c");
    }

    #[test]
    fn apply_outcomes_updates_lock() {
        let m = manifest_with(vec![]);
        let lock = empty_lock();
        let resolver = MockResolver {
            git_rev: None,
            url_hash: None,
        };
        let outcomes = vec![RefreshOutcome::Updated(ResolvedInput {
            name: "new".into(),
            entry: LockEntry {
                kind: LockedKind::File {
                    url: "https://example.com/new".into(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-new=".into(),
                },
                patches: vec![],
                mirrors: vec![],
            },
        })];

        let result = apply_outcomes(&m, &lock, &outcomes, &resolver);
        assert!(result.lock.inputs.contains_key("new"));
        assert_eq!(result.lock.inputs["new"].hash.value, "sha256-new=");
        assert_eq!(result.inputs_changed, 1);
        assert!(result.has_changes());
    }

    /// A resolver that also hashes local files (for patch tests).
    struct PatchResolver {
        url_hash: Option<String>,
        local_hash: Option<String>,
    }

    impl RefreshResolver for PatchResolver {
        fn resolve_git_rev(
            &self,
            _: &str,
            _: &GitReference,
        ) -> Result<Option<String>, Error> {
            Ok(None)
        }
        fn hash_url_content(
            &self,
            _: &str,
            _: &HashAlgo,
        ) -> Result<Option<String>, Error> {
            Ok(self.url_hash.clone())
        }
        fn hash_local_file(
            &self,
            _: &str,
            _: &HashAlgo,
        ) -> Result<Option<String>, Error> {
            Ok(self.local_hash.clone())
        }
    }

    #[test]
    fn apply_outcomes_resolves_patches() {
        let m = ProjectManifest {
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
        let resolver = PatchResolver {
            url_hash: Some("sha256-pkghash=".into()),
            local_hash: Some("sha256-patchhash=".into()),
        };
        let lock = empty_lock();
        let outcomes = refresh_inputs(&m, &lock, &[], &resolver);
        let result = apply_outcomes(&m, &lock, &outcomes, &resolver);

        // The lock entry should reference the patch
        assert_eq!(result.lock.inputs["pkg"].patches, vec!["fix1"]);
        // The locked patches map should have the patch resolved
        assert!(result.lock.patches.contains_key("fix1"));
        let locked_patch = &result.lock.patches["fix1"];
        assert!(matches!(
            &locked_patch.source,
            LockedPatchSource::Local { path } if path == "patches/fix1.patch"
        ));
        // Hash must be non-empty
        assert!(!locked_patch.hash.value.is_empty());
        assert_eq!(locked_patch.hash.value, "sha256-patchhash=");
        // Lockfile must be valid
        assert!(result.lock.validate().is_empty(), "lockfile invalid: {:?}", result.lock.validate());
        assert!(result.patches_changed);
    }

    #[test]
    fn apply_outcomes_removes_orphaned_patches() {
        let m = manifest_with(vec![file_input("pkg", "https://example.com/pkg")]);
        let resolver = MockResolver {
            git_rev: None,
            url_hash: Some("sha256-h=".into()),
        };
        // Lock starts with a stale patch that no input references
        let mut lock = empty_lock();
        lock.patches.insert(
            "old-patch".into(),
            LockedPatch {
                source: LockedPatchSource::Local {
                    path: "old.patch".into(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-old=".into(),
                },
            },
        );
        let outcomes = refresh_inputs(&m, &lock, &[], &resolver);
        let result = apply_outcomes(&m, &lock, &outcomes, &resolver);

        // Orphaned patch should be removed
        assert!(!result.lock.patches.contains_key("old-patch"));
        assert!(result.patches_changed);
    }

    #[test]
    fn resolve_patch_returns_none_when_hash_unavailable() {
        // MockResolver doesn't implement hash_local_file (default returns None)
        // So local patches cannot be resolved -> resolve_patch returns None
        let m = ProjectManifest {
            version: "1.0.0".into(),
            inputs: vec![ManifestInput {
                name: "pkg".into(),
                kind: InputKind::File {
                    url: "https://example.com/pkg".into(),
                },
                hash: HashSpec::default(),
                frozen: false,
                mirrors: vec![],
                patches: vec!["nohash".into()],
            }],
            patches: vec![PatchDef {
                name: "nohash".into(),
                source: PatchSource::Local {
                    path: "patches/nohash.patch".into(),
                },
            }],
        };
        let resolver = MockResolver {
            git_rev: None,
            url_hash: Some("sha256-pkghash=".into()),
        };
        let lock = empty_lock();
        let outcomes = refresh_inputs(&m, &lock, &[], &resolver);
        let result = apply_outcomes(&m, &lock, &outcomes, &resolver);

        // Patch should NOT be in the lock (hash unavailable)
        assert!(!result.lock.patches.contains_key("nohash"));
    }

    #[test]
    fn patch_definition_change_triggers_relock() {
        let resolver = PatchResolver {
            url_hash: Some("sha256-pkghash=".into()),
            local_hash: Some("sha256-newhash=".into()),
        };

        let m = ProjectManifest {
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
                    path: "patches/fix1-v2.patch".into(), // changed path
                },
            }],
        };

        // Lock has the patch at the old path
        let mut lock = empty_lock();
        lock.inputs.insert(
            "pkg".into(),
            LockEntry {
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
        );
        lock.patches.insert(
            "fix1".into(),
            LockedPatch {
                source: LockedPatchSource::Local {
                    path: "patches/fix1.patch".into(), // old path
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-oldhash=".into(),
                },
            },
        );

        let outcomes = refresh_inputs(&m, &lock, &[], &resolver);
        let result = apply_outcomes(&m, &lock, &outcomes, &resolver);

        // Patch should be re-resolved with the new path and hash
        let locked = &result.lock.patches["fix1"];
        assert!(matches!(
            &locked.source,
            LockedPatchSource::Local { path } if path == "patches/fix1-v2.patch"
        ));
        assert_eq!(locked.hash.value, "sha256-newhash=");
        assert!(result.patches_changed);
    }

    #[test]
    fn patch_only_change_has_changes_true() {
        // Inputs are unchanged, but a new patch definition is added.
        let resolver = PatchResolver {
            url_hash: Some("sha256-h=".into()),
            local_hash: Some("sha256-ph=".into()),
        };

        let m = ProjectManifest {
            version: "1.0.0".into(),
            inputs: vec![ManifestInput {
                name: "pkg".into(),
                kind: InputKind::File {
                    url: "https://example.com/pkg".into(),
                },
                hash: HashSpec::default(),
                frozen: false,
                mirrors: vec![],
                patches: vec!["newpatch".into()],
            }],
            patches: vec![PatchDef {
                name: "newpatch".into(),
                source: PatchSource::Local {
                    path: "patches/new.patch".into(),
                },
            }],
        };

        // Lock has the input already, but no patches locked
        let mut lock = empty_lock();
        lock.inputs.insert(
            "pkg".into(),
            LockEntry {
                kind: LockedKind::File {
                    url: "https://example.com/pkg".into(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-h=".into(),
                },
                patches: vec!["newpatch".into()],
                mirrors: vec![],
            },
        );

        let outcomes = refresh_inputs(&m, &lock, &[], &resolver);
        // Input itself is unchanged
        assert!(matches!(&outcomes[0], RefreshOutcome::Unchanged { .. }));

        let result = apply_outcomes(&m, &lock, &outcomes, &resolver);
        // But patches changed
        assert!(result.patches_changed);
        assert_eq!(result.inputs_changed, 0);
        assert!(result.has_changes()); // critical: this must be true
        assert!(result.lock.patches.contains_key("newpatch"));
    }

    #[test]
    fn list_stale_returns_changed_only() {
        let m = manifest_with(vec![
            file_input("fresh", "https://example.com/fresh"),
            file_input("stale", "https://example.com/stale"),
        ]);

        let mut lock = empty_lock();
        lock.inputs.insert(
            "fresh".into(),
            LockEntry {
                kind: LockedKind::File {
                    url: "https://example.com/fresh".into(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-current=".into(),
                },
                patches: vec![],
                mirrors: vec![],
            },
        );

        // Resolver returns "sha256-current=" for all, so "fresh" is unchanged
        // but "stale" is missing from lock -> updated
        let resolver = MockResolver {
            git_rev: None,
            url_hash: Some("sha256-current=".into()),
        };

        let stale = list_stale(&m, &lock, &resolver);
        assert_eq!(stale, vec!["stale"]);
    }

    #[test]
    fn resolver_failure_produces_failed_outcome() {
        struct FailingResolver;
        impl RefreshResolver for FailingResolver {
            fn resolve_git_rev(
                &self,
                _: &str,
                _: &GitReference,
            ) -> Result<Option<String>, Error> {
                Err(Error::Manifest("network down".into()))
            }
            fn hash_url_content(
                &self,
                _: &str,
                _: &HashAlgo,
            ) -> Result<Option<String>, Error> {
                Err(Error::Manifest("network down".into()))
            }
        }

        let m = manifest_with(vec![file_input("broken", "https://example.com/broken")]);
        let outcomes = refresh_inputs(&m, &empty_lock(), &[], &FailingResolver);
        assert_eq!(outcomes.len(), 1);
        match &outcomes[0] {
            RefreshOutcome::Failed { name, reason } => {
                assert_eq!(name, "broken");
                assert!(reason.contains("network down"));
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }
}
