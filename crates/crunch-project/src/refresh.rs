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
use crate::lock::LockEntry;
use crate::lock::LockedHash;
use crate::lock::LockedKind;
use crate::lock::LockedPatch;
use crate::lock::LockedPatchSource;
use crate::lock::Lockfile;
use crate::manifest::GitReference;
use crate::manifest::HashAlgo;
use crate::manifest::InputKind;
use crate::manifest::ManifestInput;
use crate::manifest::PatchDef;
use crate::manifest::PatchSource;
use crate::manifest::ProjectManifest;

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

/// How content should be hashed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashResolutionMode {
    /// Hash raw bytes exactly as downloaded or read from disk.
    Flat,
    /// Hash the NAR serialization of the unpacked or checked-out tree.
    Recursive,
}

/// A refresh or stale-check failure tied to a named item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshFailure {
    pub name: String,
    pub reason: String,
}

/// Result of a stale check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleReport {
    /// Input names whose resolved state differs from the current lock.
    pub stale: Vec<String>,
    /// Inputs that could not be checked.
    pub failed: Vec<RefreshFailure>,
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
    /// couldn't be resolved, or `Err` on hard failures.
    fn resolve_git_rev(&self, repository: &str, reference: &GitReference) -> Result<Option<String>, Error>;

    /// Compute the hash of downloaded URL content.
    ///
    /// `mode` distinguishes flat file hashing from recursive tree hashing.
    fn hash_url_content(&self, url: &str, algo: &HashAlgo, mode: HashResolutionMode) -> Result<Option<String>, Error>;

    /// Compute the recursive hash of a git checkout at a concrete revision.
    fn hash_git_checkout(&self, repository: &str, rev: &str, algo: &HashAlgo) -> Result<Option<String>, Error> {
        let _ = (repository, rev, algo);
        Ok(None)
    }

    /// Compute the hash of a local file's content.
    ///
    /// Used for local patches. Returns `Ok(Some(sri_hash))` if the file
    /// was hashed, `Ok(None)` if unavailable.
    fn hash_local_file(&self, path: &str, algo: &HashAlgo) -> Result<Option<String>, Error> {
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

    inputs_to_refresh.iter().map(|input| refresh_one(input, lock, resolver)).collect()
}

/// Select which inputs to refresh based on the `selected` list.
fn select_inputs<'a>(manifest: &'a ProjectManifest, selected: &[String]) -> Vec<&'a ManifestInput> {
    if selected.is_empty() {
        manifest.inputs.iter().collect()
    } else {
        let selected_set: std::collections::HashSet<&str> = selected.iter().map(|s| s.as_str()).collect();
        manifest.inputs.iter().filter(|i| selected_set.contains(i.name.as_str())).collect()
    }
}

/// Refresh a single input.
fn refresh_one(input: &ManifestInput, lock: &Lockfile, resolver: &dyn RefreshResolver) -> RefreshOutcome {
    assert!(!input.name.is_empty(), "input name must not be empty");
    assert!(
        input.mirrors.len() as u64 <= crate::manifest::MAX_MIRRORS_PER_INPUT as u64,
        "mirror count must stay within manifest limit"
    );
    if input.frozen {
        return RefreshOutcome::Frozen {
            name: input.name.clone(),
        };
    }

    match resolve_input(input, resolver) {
        Ok(entry) => {
            // Compare with existing lock entry
            if let Some(existing) = lock.inputs.get(&input.name)
                && existing == &entry
            {
                return RefreshOutcome::Unchanged {
                    name: input.name.clone(),
                };
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

fn require_resolution(value: Option<String>, what: &str) -> Result<String, Error> {
    let resolved = value.ok_or_else(|| Error::Manifest(format!("unable to resolve {what}")))?;
    if resolved.is_empty() {
        return Err(Error::Manifest(format!("resolver returned empty {what}")));
    }
    Ok(resolved)
}

/// Resolve a manifest input to a lock entry.
fn resolve_input(input: &ManifestInput, resolver: &dyn RefreshResolver) -> Result<LockEntry, Error> {
    assert!(!input.name.is_empty(), "input name must not be empty");
    assert!(
        input.patches.len() as u64 <= crate::manifest::MAX_PATCHES_PER_INPUT as u64,
        "patch count must stay within manifest limit"
    );
    let (kind, hash) = match &input.kind {
        InputKind::File { url } => {
            let hash_value = require_resolution(
                resolver.hash_url_content(url, &input.hash.algo, HashResolutionMode::Flat)?,
                &format!("flat hash for {}", input.name),
            )?;
            (LockedKind::File { url: url.clone() }, LockedHash {
                algo: input.hash.algo.clone(),
                value: hash_value,
            })
        }
        InputKind::Tarball { url } => {
            let hash_value = require_resolution(
                resolver.hash_url_content(url, &input.hash.algo, HashResolutionMode::Recursive)?,
                &format!("tarball tree hash for {}", input.name),
            )?;
            (LockedKind::Tarball { url: url.clone() }, LockedHash {
                algo: input.hash.algo.clone(),
                value: hash_value,
            })
        }
        InputKind::Git { repository, reference } => {
            let rev = require_resolution(
                resolver.resolve_git_rev(repository, reference)?,
                &format!("git revision for {}", input.name),
            )?;
            let ref_name = match reference {
                GitReference::Branch(b) => Some(b.clone()),
                GitReference::Tag(t) => Some(t.clone()),
                GitReference::Rev(_) => None,
            };
            let hash_value = require_resolution(
                resolver.hash_git_checkout(repository, &rev, &input.hash.algo)?,
                &format!("git tree hash for {}", input.name),
            )?;
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
    /// Patch-level failures encountered while resolving lock data.
    pub failures: Vec<RefreshFailure>,
}

impl ApplyResult {
    /// Whether anything changed (inputs or patches).
    pub fn has_changes(&self) -> bool {
        self.inputs_changed > 0 || self.patches_changed
    }
}

pub(crate) struct PatchResolutionResult {
    changed: bool,
    failures: Vec<RefreshFailure>,
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
    assert!(
        lock.inputs.len() as u64 <= crate::manifest::MAX_INPUTS as u64,
        "lock inputs must stay within manifest limit"
    );
    let mut new_lock = lock.clone();
    let mut inputs_changed: u32 = 0;
    for outcome in outcomes {
        if let RefreshOutcome::Updated(resolved) = outcome {
            new_lock.inputs.insert(resolved.name.clone(), resolved.entry.clone());
            inputs_changed = inputs_changed.saturating_add(1);
        }
    }

    let patch_result = resolve_patches_into_lock(manifest, &mut new_lock, resolver);
    let reverted = revert_failed_patch_inputs(lock, &mut new_lock, &patch_result.failures);
    inputs_changed = inputs_changed.saturating_sub(reverted);
    let is_orphan_changed = remove_orphaned_patches(&mut new_lock);

    ApplyResult {
        lock: new_lock,
        inputs_changed,
        patches_changed: patch_result.changed || is_orphan_changed,
        failures: patch_result.failures,
    }
}

/// Resolve manifest patch definitions into locked patches.
///
/// For each patch name referenced by any lock entry, find the
/// manifest's `PatchDef` and lock it (compute hash, record source).
/// Re-resolves patches whose manifest definition changed (different
/// source path/url).
pub(crate) fn resolve_patches_into_lock(
    manifest: &ProjectManifest,
    lock: &mut Lockfile,
    resolver: &dyn RefreshResolver,
) -> PatchResolutionResult {
    assert!(
        manifest.patches.len() as u64 <= crate::lock::MAX_LOCKED_PATCHES as u64,
        "manifest patch count must stay within lock limit"
    );
    assert!(
        lock.inputs.len() as u64 <= crate::manifest::MAX_INPUTS as u64,
        "lock inputs must stay within manifest limit"
    );
    let mut is_changed = false;
    let mut failures = Vec::with_capacity(manifest.patches.len());
    let needed = collect_needed_patches(lock);

    let defs: std::collections::HashMap<&str, &PatchDef> =
        manifest.patches.iter().map(|d| (d.name.as_str(), d)).collect();

    for name in &needed {
        let def = match defs.get(name.as_str()) {
            Some(d) => d,
            None => continue,
        };
        let needs_resolve = match lock.patches.get(name) {
            None => true,
            Some(existing) => !patch_matches_def(existing, def),
        };
        if !needs_resolve {
            continue;
        }
        match resolve_patch(def, resolver) {
            Ok(locked) => {
                lock.patches.insert(name.clone(), locked);
                is_changed = true;
            }
            Err(err) => failures.push(RefreshFailure {
                name: name.clone(),
                reason: err.to_string(),
            }),
        }
    }

    let is_orphan_changed = remove_orphaned_patches(lock);
    PatchResolutionResult {
        changed: is_changed || is_orphan_changed,
        failures,
    }
}

fn collect_needed_patches(lock: &Lockfile) -> std::collections::BTreeSet<String> {
    let mut needed: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
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
    assert!(
        old_lock.inputs.len() as u64 <= crate::manifest::MAX_INPUTS as u64,
        "old lock input count must stay within manifest limit"
    );
    assert!(
        new_lock.inputs.len() as u64 <= crate::manifest::MAX_INPUTS as u64,
        "new lock input count must stay within manifest limit"
    );
    if failures.is_empty() {
        return 0;
    }
    let failed_names: std::collections::HashSet<&str> = failures.iter().map(|failure| failure.name.as_str()).collect();
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

/// Check if a locked patch still matches its manifest definition.
///
/// Compares source type, path/url, AND hash algo/expected for remote
/// patches. Any mismatch means the locked patch is stale.
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
            // Algo change means the locked hash is under a different
            // algorithm than what the manifest now requests.
            if locked.hash.algo != def_hash.algo {
                return false;
            }
            // If the manifest declares an expected hash and it differs
            // from what we locked, the definition changed.
            if let Some(ref expected) = def_hash.expected
                && locked.hash.value != *expected
            {
                return false;
            }
            true
        }
        _ => false, // type changed (local <-> remote)
    }
}

/// Resolve a single manifest patch definition to a locked patch.
fn resolve_patch(def: &PatchDef, resolver: &dyn RefreshResolver) -> Result<LockedPatch, Error> {
    match &def.source {
        PatchSource::Local { path } => {
            let hash_value = require_resolution(
                resolver.hash_local_file(path, &HashAlgo::Sha256)?,
                &format!("local patch hash for {path}"),
            )?;
            Ok(LockedPatch {
                source: LockedPatchSource::Local { path: path.clone() },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: hash_value,
                },
            })
        }
        PatchSource::Remote { url, hash } => {
            let hash_value = require_resolution(
                resolver.hash_url_content(url, &hash.algo, HashResolutionMode::Flat)?,
                &format!("remote patch hash for {url}"),
            )?;
            Ok(LockedPatch {
                source: LockedPatchSource::Remote { url: url.clone() },
                hash: LockedHash {
                    algo: hash.algo.clone(),
                    value: hash_value,
                },
            })
        }
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

/// List inputs that are stale (would change on refresh) without
/// mutating anything.
pub fn list_stale(manifest: &ProjectManifest, lock: &Lockfile, resolver: &dyn RefreshResolver) -> StaleReport {
    let outcomes = refresh_inputs(manifest, lock, &[], resolver);
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::lock::*;
    use crate::manifest::*;
    use crate::version::SchemaVersion;

    /// A test resolver that returns fixed values.
    struct MockResolver {
        git_rev: Option<String>,
        git_hash: Option<String>,
        url_hash: Option<String>,
    }

    impl RefreshResolver for MockResolver {
        fn resolve_git_rev(&self, _repository: &str, _reference: &GitReference) -> Result<Option<String>, Error> {
            Ok(self.git_rev.clone())
        }

        fn hash_url_content(
            &self,
            _url: &str,
            _algo: &HashAlgo,
            _mode: HashResolutionMode,
        ) -> Result<Option<String>, Error> {
            Ok(self.url_hash.clone())
        }

        fn hash_git_checkout(&self, _repository: &str, _rev: &str, _algo: &HashAlgo) -> Result<Option<String>, Error> {
            Ok(self.git_hash.clone())
        }
    }

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
            git_hash: None,
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
            git_rev: Some("0123456789abcdef0123456789abcdef01234567".into()),
            git_hash: Some("sha256-git-tree=".into()),
            url_hash: None,
        };

        let outcomes = refresh_inputs(&m, &empty_lock(), &[], &resolver);
        assert_eq!(outcomes.len(), 1);
        match &outcomes[0] {
            RefreshOutcome::Updated(r) => {
                assert_eq!(r.name, "nixpkgs");
                if let LockedKind::Git { rev, ref_name, .. } = &r.entry.kind {
                    assert_eq!(rev, "0123456789abcdef0123456789abcdef01234567");
                    assert_eq!(ref_name.as_deref(), Some("main"));
                    assert_eq!(r.entry.hash.value, "sha256-git-tree=");
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
            git_hash: None,
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
            git_hash: None,
            url_hash: Some("sha256-same=".into()),
        };

        let m = manifest_with(vec![file_input("data", "https://example.com/data.bin")]);

        // Create a lock that already has the same resolved state
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
            git_hash: None,
            url_hash: Some("sha256-new=".into()),
        };

        let outcomes = refresh_inputs(&m, &empty_lock(), &["a".into(), "c".into()], &resolver);
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
            git_hash: None,
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
        fn resolve_git_rev(&self, _: &str, _: &GitReference) -> Result<Option<String>, Error> {
            Ok(None)
        }
        fn hash_url_content(&self, _: &str, _: &HashAlgo, _: HashResolutionMode) -> Result<Option<String>, Error> {
            Ok(self.url_hash.clone())
        }
        fn hash_local_file(&self, _: &str, _: &HashAlgo) -> Result<Option<String>, Error> {
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
            git_hash: None,
            url_hash: Some("sha256-h=".into()),
        };
        // Lock starts with a stale patch that no input references
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
        let outcomes = refresh_inputs(&m, &lock, &[], &resolver);
        let result = apply_outcomes(&m, &lock, &outcomes, &resolver);

        // Orphaned patch should be removed
        assert!(!result.lock.patches.contains_key("old-patch"));
        assert!(result.patches_changed);
    }

    #[test]
    fn resolve_patch_failure_is_reported_and_input_is_reverted() {
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
            git_hash: None,
            url_hash: Some("sha256-pkghash=".into()),
        };
        let lock = empty_lock();
        let outcomes = refresh_inputs(&m, &lock, &[], &resolver);
        let result = apply_outcomes(&m, &lock, &outcomes, &resolver);

        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].name, "nohash");
        assert!(!result.lock.patches.contains_key("nohash"));
        assert!(!result.lock.inputs.contains_key("pkg"));
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
        lock.inputs.insert("pkg".into(), LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/pkg".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-pkghash=".into(),
            },
            patches: vec!["fix1".into()],
            mirrors: vec![],
        });
        lock.patches.insert("fix1".into(), LockedPatch {
            source: LockedPatchSource::Local {
                path: "patches/fix1.patch".into(), // old path
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-oldhash=".into(),
            },
        });

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
        lock.inputs.insert("pkg".into(), LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/pkg".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-h=".into(),
            },
            patches: vec!["newpatch".into()],
            mirrors: vec![],
        });

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
    fn remote_patch_algo_change_triggers_relock() {
        let resolver = PatchResolver {
            url_hash: Some("blake3-newhash=".into()),
            local_hash: None,
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
                patches: vec!["rpatch".into()],
            }],
            patches: vec![PatchDef {
                name: "rpatch".into(),
                source: PatchSource::Remote {
                    url: "https://example.com/fix.patch".into(),
                    hash: HashSpec {
                        algo: HashAlgo::Blake3, // changed from sha256
                        expected: None,
                    },
                },
            }],
        };

        // Lock has the same URL but sha256 algo
        let mut lock = empty_lock();
        lock.inputs.insert("pkg".into(), LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/pkg".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-pkghash=".into(),
            },
            patches: vec!["rpatch".into()],
            mirrors: vec![],
        });
        lock.patches.insert("rpatch".into(), LockedPatch {
            source: LockedPatchSource::Remote {
                url: "https://example.com/fix.patch".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256, // old algo
                value: "sha256-oldhash=".into(),
            },
        });

        let outcomes = refresh_inputs(&m, &lock, &[], &resolver);
        let result = apply_outcomes(&m, &lock, &outcomes, &resolver);

        // Patch must be re-resolved with blake3 algo
        let locked = &result.lock.patches["rpatch"];
        assert_eq!(locked.hash.algo, HashAlgo::Blake3);
        assert_eq!(locked.hash.value, "blake3-newhash=");
        assert!(result.patches_changed);
    }

    #[test]
    fn remote_patch_expected_hash_change_triggers_relock() {
        let resolver = PatchResolver {
            url_hash: Some("sha256-newexpected=".into()),
            local_hash: None,
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
                patches: vec!["rpatch".into()],
            }],
            patches: vec![PatchDef {
                name: "rpatch".into(),
                source: PatchSource::Remote {
                    url: "https://example.com/fix.patch".into(), // same URL
                    hash: HashSpec {
                        algo: HashAlgo::Sha256,
                        expected: Some("sha256-newexpected=".into()), // changed expected
                    },
                },
            }],
        };

        // Lock has the same URL and algo, but a different hash value
        let mut lock = empty_lock();
        lock.inputs.insert("pkg".into(), LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/pkg".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-pkghash=".into(),
            },
            patches: vec!["rpatch".into()],
            mirrors: vec![],
        });
        lock.patches.insert("rpatch".into(), LockedPatch {
            source: LockedPatchSource::Remote {
                url: "https://example.com/fix.patch".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-oldhash=".into(), // differs from new expected
            },
        });

        let outcomes = refresh_inputs(&m, &lock, &[], &resolver);
        let result = apply_outcomes(&m, &lock, &outcomes, &resolver);

        // Patch must be re-resolved; expected hash from manifest takes priority
        let locked = &result.lock.patches["rpatch"];
        assert_eq!(locked.hash.value, "sha256-newexpected=");
        assert!(result.patches_changed);
    }

    #[test]
    fn remote_patch_same_url_same_hash_not_relocked() {
        let resolver = PatchResolver {
            url_hash: Some("sha256-resolved=".into()),
            local_hash: None,
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
                patches: vec!["rpatch".into()],
            }],
            patches: vec![PatchDef {
                name: "rpatch".into(),
                source: PatchSource::Remote {
                    url: "https://example.com/fix.patch".into(),
                    hash: HashSpec {
                        algo: HashAlgo::Sha256,
                        expected: None, // no explicit expected
                    },
                },
            }],
        };

        // Lock already has the correct entry
        let mut lock = empty_lock();
        lock.inputs.insert("pkg".into(), LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/pkg".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-pkghash=".into(),
            },
            patches: vec!["rpatch".into()],
            mirrors: vec![],
        });
        lock.patches.insert("rpatch".into(), LockedPatch {
            source: LockedPatchSource::Remote {
                url: "https://example.com/fix.patch".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-existing=".into(),
            },
        });

        let outcomes = refresh_inputs(&m, &lock, &[], &resolver);
        let result = apply_outcomes(&m, &lock, &outcomes, &resolver);

        // Nothing should change
        assert!(!result.patches_changed);
        assert_eq!(result.lock.patches["rpatch"].hash.value, "sha256-existing=");
    }

    #[test]
    fn list_stale_separates_changed_and_failed_inputs() {
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

        // Resolver returns "sha256-current=" for all, so "fresh" is unchanged
        // but "stale" is missing from lock -> updated
        struct MixedResolver;
        impl RefreshResolver for MixedResolver {
            fn resolve_git_rev(&self, _: &str, _: &GitReference) -> Result<Option<String>, Error> {
                Ok(None)
            }

            fn hash_url_content(
                &self,
                url: &str,
                _: &HashAlgo,
                _: HashResolutionMode,
            ) -> Result<Option<String>, Error> {
                if url.ends_with("fresh") || url.ends_with("stale") {
                    return Ok(Some("sha256-current=".into()));
                }
                Err(Error::Manifest("network down".into()))
            }
        }

        let broken_input = file_input("broken", "https://example.com/broken");
        let m = manifest_with(vec![
            file_input("fresh", "https://example.com/fresh"),
            file_input("stale", "https://example.com/stale"),
            broken_input,
        ]);

        let stale = list_stale(&m, &lock, &MixedResolver);
        assert_eq!(stale.stale, vec!["stale"]);
        assert_eq!(stale.failed.len(), 1);
        assert_eq!(stale.failed[0].name, "broken");
        assert!(stale.failed[0].reason.contains("network down"));
    }

    #[test]
    fn resolver_failure_produces_failed_outcome() {
        struct FailingResolver;
        impl RefreshResolver for FailingResolver {
            fn resolve_git_rev(&self, _: &str, _: &GitReference) -> Result<Option<String>, Error> {
                Err(Error::Manifest("network down".into()))
            }
            fn hash_url_content(&self, _: &str, _: &HashAlgo, _: HashResolutionMode) -> Result<Option<String>, Error> {
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
