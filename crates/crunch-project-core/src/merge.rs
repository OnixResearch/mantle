use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::lock::Lockfile;
use crate::manifest::InputKind;
use crate::manifest::ManifestInput;
use crate::manifest::ProjectManifest;

const MAX_ISSUES: u32 = 512;

#[derive(Debug, Clone, PartialEq)]
pub struct MergeIssue {
    pub severity: Severity,
    pub input_name: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct MergeReport {
    pub issues: Vec<MergeIssue>,
}

impl MergeReport {
    pub fn has_errors(self) -> bool {
        self.issues.iter().any(|issue| issue.severity == Severity::Error)
    }

    pub fn is_clean(self) -> bool {
        self.issues.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterInputsResult {
    pub found: Vec<ManifestInput>,
    pub missing: Vec<String>,
}

pub fn check_manifest_lock(manifest: ProjectManifest, lock: Lockfile) -> MergeReport {
    let mut issues = Vec::new();

    check_missing_entries(&manifest, &lock, &mut issues);
    check_orphaned_entries(&manifest, &lock, &mut issues);
    check_kind_consistency(&manifest, &lock, &mut issues);
    check_patch_consistency(&manifest, &lock, &mut issues);
    check_frozen_inputs(&manifest, &lock, &mut issues);

    MergeReport { issues }
}

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

fn check_orphaned_entries(manifest: &ProjectManifest, lock: &Lockfile, issues: &mut Vec<MergeIssue>) {
    let manifest_names: BTreeSet<&str> = manifest.inputs.iter().map(|input| input.name.as_str()).collect();

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

pub fn inputs_needing_refresh(manifest: ProjectManifest, lock: Lockfile) -> Vec<String> {
    let mut result = Vec::with_capacity(manifest.inputs.len());

    for input in &manifest.inputs {
        if input.frozen {
            continue;
        }
        if !lock.inputs.contains_key(&input.name) {
            result.push(input.name.clone());
            continue;
        }
        if let Some(entry) = lock.inputs.get(&input.name)
            && !kinds_compatible(&input.kind, &entry.kind)
        {
            result.push(input.name.clone());
        }
    }

    result
}

pub fn filter_inputs(manifest: ProjectManifest, selected: Vec<String>) -> FilterInputsResult {
    let mut found = Vec::with_capacity(selected.len());
    let mut missing = Vec::with_capacity(selected.len());

    let manifest_names: BTreeMap<&str, &ManifestInput> =
        manifest.inputs.iter().map(|input| (input.name.as_str(), input)).collect();

    for name in selected {
        match manifest_names.get(name.as_str()) {
            Some(input) => found.push((*input).clone()),
            None => missing.push(name),
        }
    }

    FilterInputsResult { found, missing }
}

pub fn orphaned_lock_entries(manifest: ProjectManifest, lock: Lockfile) -> Vec<String> {
    let manifest_names: BTreeSet<&str> = manifest.inputs.iter().map(|input| input.name.as_str()).collect();

    lock.inputs.keys().filter(|name| !manifest_names.contains(name.as_str())).cloned().collect()
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use alloc::string::ToString;
    use alloc::vec;

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
        let mut inputs = BTreeMap::new();
        for (name, entry) in entries {
            inputs.insert(name.to_string(), entry);
        }
        Lockfile {
            version: SchemaVersion::CURRENT,
            inputs,
            patches: BTreeMap::new(),
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
            fetch_policy: crate::InputFetchPolicy::GenerationMaterial,
            freshness: None,
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
            fetch_policy: crate::InputFetchPolicy::GenerationMaterial,
            freshness: None,
        }
    }

    #[test]
    fn clean_merge() {
        let manifest = manifest_with(vec![file_input("foo")]);
        let lock = lock_with(vec![("foo", file_lock_entry())]);
        let report = check_manifest_lock(manifest, lock);
        assert!(report.clone().is_clean());
        assert!(!report.has_errors());
    }

    #[test]
    fn missing_lock_entry() {
        let manifest = manifest_with(vec![file_input("foo"), file_input("bar")]);
        let lock = lock_with(vec![("foo", file_lock_entry())]);
        let report = check_manifest_lock(manifest, lock);
        assert!(report.clone().has_errors());
        assert_eq!(report.issues.len(), 1);
        assert!(report.issues[0].message.contains("bar"));
        assert!(report.issues[0].message.contains("no lock entry"));
    }

    #[test]
    fn orphaned_lock_entry() {
        let manifest = manifest_with(vec![file_input("foo")]);
        let lock = lock_with(vec![("foo", file_lock_entry()), ("stale", file_lock_entry())]);
        let report = check_manifest_lock(manifest, lock);
        assert!(!report.clone().has_errors());
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0].severity, Severity::Warning);
        assert!(report.issues[0].message.contains("orphaned"));
    }

    #[test]
    fn kind_mismatch() {
        let manifest = manifest_with(vec![file_input("foo")]);
        let lock = lock_with(vec![("foo", LockEntry {
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
            fetch_policy: crate::InputFetchPolicy::GenerationMaterial,
            freshness: None,
        })]);
        let report = check_manifest_lock(manifest, lock);
        assert!(report.clone().has_errors());
        assert!(report.issues[0].message.contains("does not match"));
    }

    #[test]
    fn frozen_without_lock() {
        let mut input = file_input("frozen-thing");
        input.frozen = true;
        let manifest = manifest_with(vec![input]);
        let lock = lock_with(vec![]);
        let report = check_manifest_lock(manifest, lock);
        assert!(report.clone().has_errors());
        assert!(!report.issues.is_empty());
    }

    #[test]
    fn inputs_needing_refresh_skips_frozen() {
        let mut input = file_input("frozen-thing");
        input.frozen = true;
        let manifest = manifest_with(vec![input, file_input("unlocked")]);
        let lock = lock_with(vec![]);
        let needs = inputs_needing_refresh(manifest, lock);
        assert_eq!(needs, vec!["unlocked"]);
    }

    #[test]
    fn inputs_needing_refresh_includes_missing() {
        let manifest = manifest_with(vec![file_input("a"), file_input("b")]);
        let lock = lock_with(vec![("a", file_lock_entry())]);
        let needs = inputs_needing_refresh(manifest, lock);
        assert_eq!(needs, vec!["b"]);
    }

    #[test]
    fn filter_inputs_finds_existing() {
        let manifest = manifest_with(vec![file_input("a"), file_input("b")]);
        let result = filter_inputs(manifest, vec!["a".into()]);
        assert_eq!(result.found.len(), 1);
        assert_eq!(result.found[0].name, "a");
        assert!(result.missing.is_empty());
    }

    #[test]
    fn filter_inputs_reports_missing() {
        let manifest = manifest_with(vec![file_input("a")]);
        let result = filter_inputs(manifest, vec!["a".into(), "z".into()]);
        assert_eq!(result.found.len(), 1);
        assert_eq!(result.missing, vec!["z"]);
    }

    #[test]
    fn orphaned_entries_detected() {
        let manifest = manifest_with(vec![file_input("a")]);
        let lock = lock_with(vec![("a", file_lock_entry()), ("b", file_lock_entry())]);
        let orphans = orphaned_lock_entries(manifest, lock);
        assert_eq!(orphans, vec!["b"]);
    }

    #[test]
    fn empty_manifest_and_lock_is_clean() {
        let manifest = manifest_with(vec![]);
        let lock = lock_with(vec![]);
        let report = check_manifest_lock(manifest, lock);
        assert!(report.is_clean());
    }
}
