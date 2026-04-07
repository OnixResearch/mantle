//! Project lockfile types and JSON serialization.
//!
//! The lockfile (`crunch.lock`) is machine-edited JSON. It records the
//! resolved state of every project input: concrete URLs, verified hashes,
//! selected mirrors, patch hashes, and the schema version.
//!
//! All types are pure data with serde derives. No I/O.

use crate::manifest::HashAlgo;
use crate::version::SchemaVersion;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Maximum number of entries in a lockfile.
const MAX_LOCK_ENTRIES: u32 = 4096;

/// Maximum number of locked patches.
const MAX_LOCKED_PATCHES: u32 = 1024;

/// A resolved lockfile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lockfile {
    /// Schema version.
    pub version: SchemaVersion,

    /// Resolved inputs keyed by name.
    pub inputs: BTreeMap<String, LockEntry>,

    /// Resolved patches keyed by name.
    #[serde(default)]
    pub patches: BTreeMap<String, LockedPatch>,
}

impl Lockfile {
    /// Create a new empty lockfile at the current schema version.
    pub fn new() -> Self {
        Self {
            version: SchemaVersion::CURRENT,
            inputs: BTreeMap::new(),
            patches: BTreeMap::new(),
        }
    }

    /// Serialize to a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from a JSON string.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Validate internal consistency.
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();

        if self.inputs.len() as u64 > MAX_LOCK_ENTRIES as u64 {
            problems.push(format!(
                "too many lock entries: {} (max {MAX_LOCK_ENTRIES})",
                self.inputs.len()
            ));
        }

        if self.patches.len() as u64 > MAX_LOCKED_PATCHES as u64 {
            problems.push(format!(
                "too many locked patches: {} (max {MAX_LOCKED_PATCHES})",
                self.patches.len()
            ));
        }

        for (name, entry) in &self.inputs {
            if name.is_empty() {
                problems.push("lock entry with empty name".into());
            }
            if entry.hash.value.is_empty() {
                problems.push(format!("lock entry '{name}': hash value is empty"));
            }
            // Patch references should correspond to locked patches
            for patch_name in &entry.patches {
                if !self.patches.contains_key(patch_name) {
                    problems.push(format!(
                        "lock entry '{name}' references unlocked patch '{patch_name}'"
                    ));
                }
            }
        }

        problems
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self::new()
    }
}

/// A resolved input in the lockfile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockEntry {
    /// Resolved input kind with concrete URLs.
    pub kind: LockedKind,

    /// Verified hash.
    pub hash: LockedHash,

    /// Names of patches applied to this input.
    #[serde(default)]
    pub patches: Vec<String>,

    /// Mirror URLs that were resolved for this input.
    #[serde(default)]
    pub mirrors: Vec<String>,
}

/// Resolved input kind with all template variables expanded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LockedKind {
    /// A single file at a resolved URL.
    #[serde(rename = "file")]
    File { url: String },

    /// A tarball at a resolved URL.
    #[serde(rename = "tarball")]
    Tarball { url: String },

    /// A git repository at a specific revision.
    #[serde(rename = "git")]
    Git {
        repository: String,
        /// The resolved revision hash (full SHA).
        rev: String,
        /// The reference that was resolved to get this rev
        /// (informational, for display).
        #[serde(default)]
        ref_name: Option<String>,
    },
}

/// A verified hash in the lockfile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedHash {
    pub algo: HashAlgo,
    /// Hash value in SRI format (e.g. "sha256-abc123...=").
    pub value: String,
}

/// A resolved patch with its own hash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedPatch {
    /// Where the patch was fetched from.
    pub source: LockedPatchSource,

    /// Hash of the patch file content.
    pub hash: LockedHash,
}

/// Source of a locked patch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LockedPatchSource {
    /// Local path (relative to project root).
    #[serde(rename = "local")]
    Local { path: String },

    /// Remote URL.
    #[serde(rename = "remote")]
    Remote { url: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn sample_lockfile() -> Lockfile {
        let mut inputs = BTreeMap::new();
        inputs.insert(
            "nixpkgs".into(),
            LockEntry {
                kind: LockedKind::Git {
                    repository: "https://github.com/NixOS/nixpkgs.git".into(),
                    rev: "abc123def456789".into(),
                    ref_name: Some("nixos-unstable".into()),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into(),
                },
                patches: vec![],
                mirrors: vec![
                    "https://mirrors.tuna.tsinghua.edu.cn/git/nixpkgs.git".into(),
                ],
            },
        );
        inputs.insert(
            "hello-src".into(),
            LockEntry {
                kind: LockedKind::Tarball {
                    url: "https://ftp.gnu.org/gnu/hello/hello-2.12.1.tar.gz".into(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-jZkUKv2SV28wsM18tCqNxoCZmLxdYH2Idh9RLibH2yA=".into(),
                },
                patches: vec!["hello-fix".into()],
                mirrors: vec![],
            },
        );

        let mut patches = BTreeMap::new();
        patches.insert(
            "hello-fix".into(),
            LockedPatch {
                source: LockedPatchSource::Local {
                    path: "patches/hello-fix.patch".into(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-patchhashvalue123=".into(),
                },
            },
        );

        Lockfile {
            version: SchemaVersion::CURRENT,
            inputs,
            patches,
        }
    }

    #[test]
    fn lockfile_json_roundtrip() {
        let lock = sample_lockfile();
        let json = lock.to_json().unwrap();
        let parsed = Lockfile::from_json(&json).unwrap();
        assert_eq!(lock, parsed);
    }

    #[test]
    fn lockfile_validates_clean() {
        let lock = sample_lockfile();
        let problems = lock.validate();
        assert!(problems.is_empty(), "unexpected: {problems:?}");
    }

    #[test]
    fn lockfile_detects_empty_hash() {
        let mut lock = sample_lockfile();
        lock.inputs.get_mut("nixpkgs").unwrap().hash.value = String::new();
        let problems = lock.validate();
        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains("hash value is empty"));
    }

    #[test]
    fn lockfile_detects_unlocked_patch() {
        let mut lock = sample_lockfile();
        lock.patches.clear();
        let problems = lock.validate();
        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains("unlocked patch"));
    }

    #[test]
    fn empty_lockfile_is_valid() {
        let lock = Lockfile::new();
        let problems = lock.validate();
        assert!(problems.is_empty());
    }

    #[test]
    fn lockfile_default_version_is_current() {
        let lock = Lockfile::new();
        assert_eq!(lock.version, SchemaVersion::CURRENT);
    }

    #[test]
    fn lockfile_json_stability() {
        // Verify that key ordering is stable (BTreeMap) and version
        // serializes as a string.
        let lock = Lockfile::new();
        let json = lock.to_json().unwrap();
        assert!(json.contains("\"version\": \"1.0.0\""));
        assert!(json.contains("\"inputs\": {}"));
    }

    #[test]
    fn lockfile_entry_with_all_fields() {
        let entry = LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/file.txt".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Blake3,
                value: "blake3-somehashvalue=".into(),
            },
            patches: vec!["p1".into(), "p2".into()],
            mirrors: vec!["https://mirror1.example.com/file.txt".into()],
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: LockEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, parsed);
    }

    #[test]
    fn locked_kind_git_serde() {
        let kind = LockedKind::Git {
            repository: "https://github.com/user/repo.git".into(),
            rev: "deadbeef".into(),
            ref_name: Some("main".into()),
        };
        let json = serde_json::to_string(&kind).unwrap();
        assert!(json.contains("\"type\":\"git\""));
        assert!(json.contains("\"rev\":\"deadbeef\""));
        let parsed: LockedKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, parsed);
    }

    #[test]
    fn locked_patch_source_variants() {
        let local = LockedPatchSource::Local {
            path: "./p.patch".into(),
        };
        let remote = LockedPatchSource::Remote {
            url: "https://example.com/p.patch".into(),
        };

        let local_json = serde_json::to_string(&local).unwrap();
        let remote_json = serde_json::to_string(&remote).unwrap();

        assert!(local_json.contains("\"type\":\"local\""));
        assert!(remote_json.contains("\"type\":\"remote\""));

        let local_parsed: LockedPatchSource = serde_json::from_str(&local_json).unwrap();
        let remote_parsed: LockedPatchSource = serde_json::from_str(&remote_json).unwrap();
        assert_eq!(local, local_parsed);
        assert_eq!(remote, remote_parsed);
    }
}
