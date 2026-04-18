//! Merge and validate manifest + lock state.
//!
//! Pure functions that compare a `ProjectManifest` against a `Lockfile`
//! to detect missing entries, stale inputs, orphaned lock entries,
//! and inconsistencies. No I/O, no network.

use crate::lock::Lockfile;
use crate::manifest::InputKind;
use crate::manifest::ManifestInput;
use crate::manifest::ProjectManifest;

/// Maximum number of validation issues before we stop collecting.
const MAX_ISSUES: u32 = 512;

/// A validation issue found during merge checking.
#[derive(Debug, Clone, PartialEq)]
pub struct MergeIssue {
    pub severity: Severity,
    pub input_name: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Must be fixed before builds work.
    Error,
    /// Should be fixed but builds may still work.
    Warning,
}

/// Result of comparing manifest against lockfile.
#[derive(Debug, Clone)]
pub struct MergeReport {
    pub issues: Vec<MergeIssue>,
}

impl MergeReport {
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| i.severity == Severity::Error)
    }

    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Compare a manifest against a lockfile and report issues.
///
/// Checks:
/// 1. Every manifest input has a corresponding lock entry.
/// 2. No orphaned lock entries exist (locked but not in manifest).
/// 3. Input kinds match between manifest and lock.
/// 4. Patch references are consistent.
/// 5. Frozen inputs have lock entries with hashes.
pub fn check_manifest_lock(manifest: &ProjectManifest, lock: &Lockfile) -> MergeReport {
    let mut issues = Vec::new();

    check_missing_entries(manifest, lock, &mut issues);
    check_orphaned_entries(manifest, lock, &mut issues);
    check_kind_consistency(manifest, lock, &mut issues);
    check_patch_consistency(manifest, lock, &mut issues);
    check_frozen_inputs(manifest, lock, &mut issues);

    MergeReport { issues }
}

/// Find manifest inputs that have no lock entry.
fn check_missing_entries(manifest: &ProjectManifest, lock: &Lockfile, issues: &mut Vec<MergeIssue>) {
    for input in &manifest.inputs {
        if issues.len() as u64 >= MAX_ISSUES as u64 {
            break;
        }
        if !lock.inputs.contains_key(&input.name) {
            issues.push(MergeIssue {
                severity: Severity::Error,
                input_name: Some(input.name.clone()),
                message: format!("input '{}' is in the manifest but has no lock entry", input.name),
            });
        }
    }
}

/// Find lock entries that have no corresponding manifest input.
fn check_orphaned_entries(manifest: &ProjectManifest, lock: &Lockfile, issues: &mut Vec<MergeIssue>) {
    let manifest_names: std::collections::HashSet<&str> = manifest.inputs.iter().map(|i| i.name.as_str()).collect();

    for name in lock.inputs.keys() {
        if issues.len() as u64 >= MAX_ISSUES as u64 {
            break;
        }
        if !manifest_names.contains(name.as_str()) {
            issues.push(MergeIssue {
                severity: Severity::Warning,
                input_name: Some(name.clone()),
                message: format!("lock entry '{name}' has no corresponding manifest input (orphaned)"),
            });
        }
    }
}

/// Check that input kinds in manifest and lock are compatible.
fn check_kind_consistency(manifest: &ProjectManifest, lock: &Lockfile, issues: &mut Vec<MergeIssue>) {
    for input in &manifest.inputs {
        if issues.len() as u64 >= MAX_ISSUES as u64 {
            break;
        }
        if let Some(entry) = lock.inputs.get(&input.name)
            && !kinds_compatible(&input.kind, &entry.kind)
        {
            issues.push(MergeIssue {
                severity: Severity::Error,
                input_name: Some(input.name.clone()),
                message: format!(
                    "input '{}': manifest kind ({}) does not match lock kind ({})",
                    input.name,
                    kind_label(&input.kind),
                    locked_kind_label(&entry.kind),
                ),
            });
        }
    }
}

/// Check that patch references are consistent between manifest and lock.
fn check_patch_consistency(manifest: &ProjectManifest, lock: &Lockfile, issues: &mut Vec<MergeIssue>) {
    for input in &manifest.inputs {
        if issues.len() as u64 >= MAX_ISSUES as u64 {
            break;
        }
        if let Some(entry) = lock.inputs.get(&input.name)
            && input.patches != entry.patches
        {
            issues.push(MergeIssue {
                severity: Severity::Warning,
                input_name: Some(input.name.clone()),
                message: format!(
                    "input '{}': manifest patches {:?} differ from lock patches {:?}",
                    input.name, input.patches, entry.patches,
                ),
            });
        }
    }
}

/// Frozen inputs must have lock entries with non-empty hashes.
fn check_frozen_inputs(manifest: &ProjectManifest, lock: &Lockfile, issues: &mut Vec<MergeIssue>) {
    for input in &manifest.inputs {
        if !input.frozen {
            continue;
        }
        if issues.len() as u64 >= MAX_ISSUES as u64 {
            break;
        }
        match lock.inputs.get(&input.name) {
            None => {
                issues.push(MergeIssue {
                    severity: Severity::Error,
                    input_name: Some(input.name.clone()),
                    message: format!("input '{}' is frozen but has no lock entry", input.name),
                });
            }
            Some(entry) if entry.hash.value.is_empty() => {
                issues.push(MergeIssue {
                    severity: Severity::Error,
                    input_name: Some(input.name.clone()),
                    message: format!("input '{}' is frozen but lock entry has no hash", input.name),
                });
            }
            _ => {}
        }
    }
}

/// Check if a manifest kind is compatible with a locked kind.
fn kinds_compatible(manifest: &InputKind, locked: &crate::lock::LockedKind) -> bool {
    use crate::lock::LockedKind;
    matches!(
        (manifest, locked),
        (InputKind::File { .. }, LockedKind::File { .. })
            | (InputKind::Tarball { .. }, LockedKind::Tarball { .. })
            | (InputKind::Git { .. }, LockedKind::Git { .. })
    )
}

fn kind_label(kind: &InputKind) -> &'static str {
    match kind {
        InputKind::File { .. } => "file",
        InputKind::Tarball { .. } => "tarball",
        InputKind::Git { .. } => "git",
    }
}

fn locked_kind_label(kind: &crate::lock::LockedKind) -> &'static str {
    use crate::lock::LockedKind;
    match kind {
        LockedKind::File { .. } => "file",
        LockedKind::Tarball { .. } => "tarball",
        LockedKind::Git { .. } => "git",
    }
}

/// Determine which manifest inputs need refreshing.
///
/// Returns names of inputs that have no lock entry or whose lock entry
/// is stale. Frozen inputs are excluded.
pub fn inputs_needing_refresh(manifest: &ProjectManifest, lock: &Lockfile) -> Vec<String> {
    let mut result = Vec::new();

    for input in &manifest.inputs {
        if input.frozen {
            continue;
        }
        if !lock.inputs.contains_key(&input.name) {
            result.push(input.name.clone());
            continue;
        }
        // If the kind changed, needs refresh
        if let Some(entry) = lock.inputs.get(&input.name)
            && !kinds_compatible(&input.kind, &entry.kind)
        {
            result.push(input.name.clone());
        }
    }

    result
}

/// Filter manifest inputs to only those named in `selected`.
///
/// Returns an error message for any name in `selected` that does not
/// exist in the manifest.
pub fn filter_inputs<'a>(manifest: &'a ProjectManifest, selected: &[String]) -> (Vec<&'a ManifestInput>, Vec<String>) {
    let mut found = Vec::new();
    let mut not_found = Vec::new();

    let manifest_names: std::collections::HashMap<&str, &ManifestInput> =
        manifest.inputs.iter().map(|i| (i.name.as_str(), i)).collect();

    for name in selected {
        match manifest_names.get(name.as_str()) {
            Some(input) => found.push(*input),
            None => not_found.push(name.clone()),
        }
    }

    (found, not_found)
}

/// Compute which lock entries would be removed if they're not in the manifest.
pub fn orphaned_lock_entries(manifest: &ProjectManifest, lock: &Lockfile) -> Vec<String> {
    let manifest_names: std::collections::HashSet<&str> = manifest.inputs.iter().map(|i| i.name.as_str()).collect();

    lock.inputs.keys().filter(|name| !manifest_names.contains(name.as_str())).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::*;
    use crate::manifest::*;
    use crate::version::SchemaVersion;

    fn manifest_with(inputs: Vec<ManifestInput>) -> ProjectManifest {
        ProjectManifest {
            version: "1.0.0".into(),
            inputs,
            patches: vec![],
        }
    }

    fn lock_with(entries: Vec<(&str, LockEntry)>) -> Lockfile {
        let mut inputs = std::collections::BTreeMap::new();
        for (name, entry) in entries {
            inputs.insert(name.to_string(), entry);
        }
        Lockfile {
            version: SchemaVersion::CURRENT,
            inputs,
            patches: std::collections::BTreeMap::new(),
        }
    }

    fn file_input(name: &str) -> ManifestInput {
        ManifestInput {
            name: name.into(),
            kind: InputKind::File {
                url: format!("https://example.com/{name}"),
            },
            hash: HashSpec::default(),
            frozen: false,
            mirrors: vec![],
            patches: vec![],
        }
    }

    fn file_lock_entry() -> LockEntry {
        LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/file".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-abc123=".into(),
            },
            patches: vec![],
            mirrors: vec![],
        }
    }

    #[test]
    fn clean_merge() {
        let m = manifest_with(vec![file_input("foo")]);
        let l = lock_with(vec![("foo", file_lock_entry())]);
        let report = check_manifest_lock(&m, &l);
        assert!(report.is_clean());
        assert!(!report.has_errors());
    }

    #[test]
    fn missing_lock_entry() {
        let m = manifest_with(vec![file_input("foo"), file_input("bar")]);
        let l = lock_with(vec![("foo", file_lock_entry())]);
        let report = check_manifest_lock(&m, &l);
        assert!(report.has_errors());
        assert_eq!(report.issues.len(), 1);
        assert!(report.issues[0].message.contains("bar"));
        assert!(report.issues[0].message.contains("no lock entry"));
    }

    #[test]
    fn orphaned_lock_entry() {
        let m = manifest_with(vec![file_input("foo")]);
        let l = lock_with(vec![("foo", file_lock_entry()), ("stale", file_lock_entry())]);
        let report = check_manifest_lock(&m, &l);
        assert!(!report.has_errors()); // warnings only
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0].severity, Severity::Warning);
        assert!(report.issues[0].message.contains("orphaned"));
    }

    #[test]
    fn kind_mismatch() {
        let m = manifest_with(vec![file_input("foo")]);
        let l = lock_with(vec![("foo", LockEntry {
            kind: LockedKind::Git {
                repository: "https://example.com/repo.git".into(),
                rev: "abc".into(),
                ref_name: None,
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-abc=".into(),
            },
            patches: vec![],
            mirrors: vec![],
        })]);
        let report = check_manifest_lock(&m, &l);
        assert!(report.has_errors());
        assert!(report.issues[0].message.contains("does not match"));
    }

    #[test]
    fn frozen_without_lock() {
        let mut input = file_input("frozen-thing");
        input.frozen = true;
        let m = manifest_with(vec![input]);
        let l = lock_with(vec![]);
        let report = check_manifest_lock(&m, &l);
        assert!(report.has_errors());
        // Two errors: missing lock entry + frozen without lock
        assert!(!report.issues.is_empty());
    }

    #[test]
    fn inputs_needing_refresh_skips_frozen() {
        let mut input = file_input("frozen-thing");
        input.frozen = true;
        let m = manifest_with(vec![input, file_input("unlocked")]);
        let l = lock_with(vec![]);
        let needs = inputs_needing_refresh(&m, &l);
        assert_eq!(needs, vec!["unlocked"]);
    }

    #[test]
    fn inputs_needing_refresh_includes_missing() {
        let m = manifest_with(vec![file_input("a"), file_input("b")]);
        let l = lock_with(vec![("a", file_lock_entry())]);
        let needs = inputs_needing_refresh(&m, &l);
        assert_eq!(needs, vec!["b"]);
    }

    #[test]
    fn filter_inputs_finds_existing() {
        let m = manifest_with(vec![file_input("a"), file_input("b")]);
        let (found, missing) = filter_inputs(&m, &["a".into()]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "a");
        assert!(missing.is_empty());
    }

    #[test]
    fn filter_inputs_reports_missing() {
        let m = manifest_with(vec![file_input("a")]);
        let (found, missing) = filter_inputs(&m, &["a".into(), "z".into()]);
        assert_eq!(found.len(), 1);
        assert_eq!(missing, vec!["z"]);
    }

    #[test]
    fn orphaned_entries_detected() {
        let m = manifest_with(vec![file_input("a")]);
        let l = lock_with(vec![("a", file_lock_entry()), ("b", file_lock_entry())]);
        let orphans = orphaned_lock_entries(&m, &l);
        assert_eq!(orphans, vec!["b"]);
    }

    #[test]
    fn empty_manifest_and_lock_is_clean() {
        let m = manifest_with(vec![]);
        let l = lock_with(vec![]);
        let report = check_manifest_lock(&m, &l);
        assert!(report.is_clean());
    }
}
