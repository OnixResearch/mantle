use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use crate::error::Error;
use crate::fetch_policy::InputFetchPolicy;
use crate::freshness::LockedFreshnessValue;
use crate::manifest::HashAlgo;
use crate::version::SchemaVersion;

const MAX_LOCK_ENTRIES: u32 = 4096;
pub const MAX_LOCKED_PATCHES: u32 = 1024;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Lockfile {
    pub version: SchemaVersion,
    pub inputs: BTreeMap<String, LockEntry>,
    pub patches: BTreeMap<String, LockedPatch>,
}

#[derive(Deserialize)]
struct RawLockfile {
    version: SchemaVersion,
    inputs: BTreeMap<String, LockEntry>,
    patches: Option<BTreeMap<String, LockedPatch>>,
}

impl<'de> Deserialize<'de> for Lockfile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let raw = RawLockfile::deserialize(deserializer)?;
        Ok(Self {
            version: raw.version,
            inputs: raw.inputs,
            patches: raw.patches.unwrap_or_else(BTreeMap::new),
        })
    }
}

impl Lockfile {
    pub fn new() -> Self {
        Self {
            version: SchemaVersion::CURRENT,
            inputs: BTreeMap::new(),
            patches: BTreeMap::new(),
        }
    }

    pub fn to_json(self) -> Result<String, Error> {
        serde_json::to_string_pretty(&self).map_err(|err| Error::Lockfile(format!("JSON serialization: {err}")))
    }

    pub fn from_json(s: String) -> Result<Self, Error> {
        serde_json::from_str(&s).map_err(|err| Error::Lockfile(format!("JSON parse: {err}")))
    }

    pub fn validate(self) -> Vec<String> {
        let Lockfile {
            version: _,
            inputs,
            patches,
        } = self;
        let mut problems = Vec::with_capacity(inputs.len().saturating_add(patches.len()));

        if inputs.len() as u64 > MAX_LOCK_ENTRIES as u64 {
            problems.push(format!("too many lock entries: {} (max {MAX_LOCK_ENTRIES})", inputs.len()));
        }

        if patches.len() as u64 > MAX_LOCKED_PATCHES as u64 {
            problems.push(format!("too many locked patches: {} (max {MAX_LOCKED_PATCHES})", patches.len()));
        }

        for (name, entry) in &inputs {
            if name.is_empty() {
                problems.push("lock entry with empty name".into());
            }
            if entry.hash.value.is_empty() {
                problems.push(format!("lock entry '{name}': hash value is empty"));
            }
            if let Some(freshness) = &entry.freshness {
                if freshness.input_name != *name {
                    problems.push(format!("lock entry '{name}': freshness input name mismatch"));
                }
                if freshness.value_digest.is_empty() {
                    problems.push(format!("lock entry '{name}': freshness value digest is empty"));
                }
            }
            for patch_name in &entry.patches {
                if !patches.contains_key(patch_name) {
                    problems.push(format!("lock entry '{name}' references unlocked patch '{patch_name}'"));
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LockEntry {
    pub kind: LockedKind,
    pub hash: LockedHash,
    pub patches: Vec<String>,
    pub mirrors: Vec<String>,
    pub fetch_policy: InputFetchPolicy,
    pub freshness: Option<LockedFreshnessValue>,
}

#[derive(Deserialize)]
struct RawLockEntry {
    kind: LockedKind,
    hash: LockedHash,
    patches: Option<Vec<String>>,
    mirrors: Option<Vec<String>>,
    fetch_policy: Option<InputFetchPolicy>,
    freshness: Option<LockedFreshnessValue>,
}

impl<'de> Deserialize<'de> for LockEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let raw = RawLockEntry::deserialize(deserializer)?;
        Ok(Self {
            kind: raw.kind,
            hash: raw.hash,
            patches: raw.patches.unwrap_or_else(Vec::new),
            mirrors: raw.mirrors.unwrap_or_else(Vec::new),
            fetch_policy: raw.fetch_policy.unwrap_or_default(),
            freshness: raw.freshness,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum LockedKind {
    #[serde(rename = "file")]
    File { url: String },
    #[serde(rename = "tarball")]
    Tarball { url: String },
    #[serde(rename = "git")]
    Git {
        repository: String,
        rev: String,
        ref_name: Option<String>,
    },
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum RawLockedKind {
    #[serde(rename = "file")]
    File { url: String },
    #[serde(rename = "tarball")]
    Tarball { url: String },
    #[serde(rename = "git")]
    Git {
        repository: String,
        rev: String,
        ref_name: Option<String>,
    },
}

impl<'de> Deserialize<'de> for LockedKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let raw = RawLockedKind::deserialize(deserializer)?;
        Ok(match raw {
            RawLockedKind::File { url } => Self::File { url },
            RawLockedKind::Tarball { url } => Self::Tarball { url },
            RawLockedKind::Git {
                repository,
                rev,
                ref_name,
            } => Self::Git {
                repository,
                rev,
                ref_name,
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedHash {
    pub algo: HashAlgo,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedPatch {
    pub source: LockedPatchSource,
    pub hash: LockedHash,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LockedPatchSource {
    #[serde(rename = "local")]
    Local { path: String },
    #[serde(rename = "remote")]
    Remote { url: String },
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn sample_lockfile() -> Lockfile {
        let mut inputs = BTreeMap::new();
        inputs.insert("nixpkgs".into(), LockEntry {
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
            mirrors: vec!["https://mirrors.tuna.tsinghua.edu.cn/git/nixpkgs.git".into()],
            fetch_policy: InputFetchPolicy::GenerationMaterial,
            freshness: None,
        });
        inputs.insert("hello-src".into(), LockEntry {
            kind: LockedKind::Tarball {
                url: "https://ftp.gnu.org/gnu/hello/hello-2.12.1.tar.gz".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-jZkUKv2SV28wsM18tCqNxoCZmLxdYH2Idh9RLibH2yA=".into(),
            },
            patches: vec!["hello-fix".into()],
            mirrors: vec![],
            fetch_policy: InputFetchPolicy::GenerationMaterial,
            freshness: None,
        });

        let mut patches = BTreeMap::new();
        patches.insert("hello-fix".into(), LockedPatch {
            source: LockedPatchSource::Local {
                path: "patches/hello-fix.patch".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-patchhashvalue123=".into(),
            },
        });

        Lockfile {
            version: SchemaVersion::CURRENT,
            inputs,
            patches,
        }
    }

    #[test]
    fn lockfile_json_roundtrip() {
        let lock = sample_lockfile();
        let json = lock.clone().to_json().unwrap();
        let parsed = Lockfile::from_json(json).unwrap();
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
            fetch_policy: InputFetchPolicy::BuildFetchAction,
            freshness: None,
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
