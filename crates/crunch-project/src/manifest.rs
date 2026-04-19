//! Project manifest types.
//!
//! The manifest (`crunch-project.ncl`) is the human-edited file. This
//! module defines the Rust-side representation after Nickel evaluation
//! and deserialization. The types here are pure data — no I/O, no eval.

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use crate::version::SchemaVersion;
use crate::version::parse_version;

/// Maximum number of inputs in a single manifest.
pub const MAX_INPUTS: u32 = 4096;

/// Maximum number of mirrors per input.
pub const MAX_MIRRORS_PER_INPUT: u32 = 64;

/// Maximum number of patches per input.
pub const MAX_PATCHES_PER_INPUT: u32 = 256;

/// A project manifest deserialized from `crunch-project.ncl`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectManifest {
    /// Schema version of the manifest format (as string for Nickel compat).
    pub version: String,

    /// Named project inputs.
    pub inputs: Vec<ManifestInput>,

    /// Global patch definitions.
    pub patches: Vec<PatchDef>,
}

#[derive(Deserialize)]
struct RawProjectManifest {
    version: String,
    inputs: Vec<ManifestInput>,
    patches: Option<Vec<PatchDef>>,
}

impl<'de> Deserialize<'de> for ProjectManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawProjectManifest::deserialize(deserializer)?;
        Ok(Self {
            version: raw.version,
            inputs: raw.inputs,
            patches: raw.patches.unwrap_or_else(Vec::new),
        })
    }
}

impl ProjectManifest {
    /// Validate internal consistency.
    ///
    /// Returns a list of problems. Empty vec = valid.
    /// Parse and return the schema version, or None if malformed.
    pub fn schema_version(&self) -> Option<SchemaVersion> {
        parse_version(&self.version)
    }

    /// Validate internal consistency.
    ///
    /// Returns a list of problems. Empty vec = valid.
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::with_capacity(self.inputs.len().saturating_mul(2).saturating_add(self.patches.len()));

        // Validate version string
        match parse_version(&self.version) {
            None => {
                problems.push(format!("invalid manifest version: '{}'", self.version));
            }
            Some(v) if !v.is_compatible_with(&SchemaVersion::CURRENT) => {
                problems.push(format!(
                    "manifest version {} is not compatible with current version {}",
                    v,
                    SchemaVersion::CURRENT
                ));
            }
            _ => {}
        }

        if self.inputs.len() as u64 > MAX_INPUTS as u64 {
            problems.push(format!("too many inputs: {} (max {MAX_INPUTS})", self.inputs.len()));
        }

        // Check for duplicate input names
        let mut seen_names = std::collections::HashSet::new();
        for input in &self.inputs {
            if !seen_names.insert(&input.name) {
                problems.push(format!("duplicate input name: {}", input.name));
            }
            problems.extend(input.validate());
        }

        // Check that patch references point to defined patches
        let patch_names: std::collections::HashSet<&str> = self.patches.iter().map(|p| p.name.as_str()).collect();
        for input in &self.inputs {
            for patch_ref in &input.patches {
                if !patch_names.contains(patch_ref.as_str()) {
                    problems.push(format!("input '{}' references undefined patch '{patch_ref}'", input.name));
                }
            }
        }

        problems
    }
}

/// A named input in the project manifest.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ManifestInput {
    /// Unique name for this input (used as key in the lockfile).
    pub name: String,

    /// What kind of source this input is.
    pub kind: InputKind,

    /// Hash specification (algorithm + optional expected hash).
    pub hash: HashSpec,

    /// Whether this input is frozen (skip on refresh).
    pub frozen: bool,

    /// Mirror URLs for this input.
    pub mirrors: Vec<String>,

    /// Names of patches to apply to this input.
    pub patches: Vec<String>,
}

#[derive(Deserialize)]
struct RawManifestInput {
    name: String,
    kind: InputKind,
    hash: Option<HashSpec>,
    frozen: Option<bool>,
    mirrors: Option<Vec<String>>,
    patches: Option<Vec<String>>,
}

impl<'de> Deserialize<'de> for ManifestInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawManifestInput::deserialize(deserializer)?;
        Ok(Self {
            name: raw.name,
            kind: raw.kind,
            hash: raw.hash.unwrap_or_default(),
            frozen: raw.frozen.unwrap_or(false),
            mirrors: raw.mirrors.unwrap_or_else(Vec::new),
            patches: raw.patches.unwrap_or_else(Vec::new),
        })
    }
}

impl ManifestInput {
    fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();

        if self.name.is_empty() {
            problems.push("input name must not be empty".into());
        }

        if self.mirrors.len() as u64 > MAX_MIRRORS_PER_INPUT as u64 {
            problems.push(format!(
                "input '{}': too many mirrors: {} (max {MAX_MIRRORS_PER_INPUT})",
                self.name,
                self.mirrors.len()
            ));
        }

        if self.patches.len() as u64 > MAX_PATCHES_PER_INPUT as u64 {
            problems.push(format!(
                "input '{}': too many patches: {} (max {MAX_PATCHES_PER_INPUT})",
                self.name,
                self.patches.len()
            ));
        }

        problems
    }
}

/// The kind of a project input.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum InputKind {
    /// A single file download.
    #[serde(rename = "file")]
    File { url: String },

    /// A tarball to fetch and unpack.
    #[serde(rename = "tarball")]
    Tarball { url: String },

    /// A git repository.
    #[serde(rename = "git")]
    Git {
        repository: String,
        reference: GitReference,
    },
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum RawInputKind {
    #[serde(rename = "file")]
    File { url: String },
    #[serde(rename = "tarball")]
    Tarball { url: String },
    #[serde(rename = "git")]
    Git {
        repository: String,
        reference: Option<GitReference>,
    },
}

impl<'de> Deserialize<'de> for InputKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawInputKind::deserialize(deserializer)?;
        Ok(match raw {
            RawInputKind::File { url } => Self::File { url },
            RawInputKind::Tarball { url } => Self::Tarball { url },
            RawInputKind::Git {
                repository,
                reference,
            } => Self::Git {
                repository,
                reference: reference.unwrap_or_else(GitReference::default),
            },
        })
    }
}

/// A git reference specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "ref_type", content = "ref_value")]
pub enum GitReference {
    #[serde(rename = "branch")]
    Branch(String),
    #[serde(rename = "tag")]
    Tag(String),
    #[serde(rename = "rev")]
    Rev(String),
}

impl Default for GitReference {
    fn default() -> Self {
        GitReference::Branch("main".into())
    }
}

/// Hash algorithm specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum HashAlgo {
    #[default]
    #[serde(rename = "sha256")]
    Sha256,
    #[serde(rename = "sha512")]
    Sha512,
    #[serde(rename = "blake3")]
    Blake3,
}

impl std::fmt::Display for HashAlgo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashAlgo::Sha256 => write!(f, "sha256"),
            HashAlgo::Sha512 => write!(f, "sha512"),
            HashAlgo::Blake3 => write!(f, "blake3"),
        }
    }
}

/// Hash specification: algorithm + optional expected hash value.
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct HashSpec {
    pub algo: HashAlgo,

    /// Expected hash in SRI format (e.g. "sha256-..."). None means
    /// not yet computed (will be filled on first refresh).
    pub expected: Option<String>,
}

#[derive(Deserialize)]
struct RawHashSpec {
    algo: Option<HashAlgo>,
    expected: Option<String>,
}

impl<'de> Deserialize<'de> for HashSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawHashSpec::deserialize(deserializer)?;
        Ok(Self {
            algo: raw.algo.unwrap_or_default(),
            expected: raw.expected,
        })
    }
}

/// A global patch definition in the manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchDef {
    /// Unique name for this patch.
    pub name: String,

    /// Where the patch comes from.
    pub source: PatchSource,
}

/// Source of a patch file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PatchSource {
    /// A local path relative to the project root.
    #[serde(rename = "local")]
    Local { path: String },

    /// A remote URL with hash verification.
    #[serde(rename = "remote")]
    Remote { url: String, hash: HashSpec },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> ProjectManifest {
        ProjectManifest {
            version: "1.0.0".into(),
            inputs: vec![
                ManifestInput {
                    name: "nixpkgs".into(),
                    kind: InputKind::Git {
                        repository: "https://github.com/NixOS/nixpkgs.git".into(),
                        reference: GitReference::Branch("nixos-unstable".into()),
                    },
                    hash: HashSpec::default(),
                    frozen: false,
                    mirrors: vec!["https://mirrors.tuna.tsinghua.edu.cn/git/nixpkgs.git".into()],
                    patches: vec![],
                },
                ManifestInput {
                    name: "hello-src".into(),
                    kind: InputKind::Tarball {
                        url: "https://ftp.gnu.org/gnu/hello/hello-2.12.1.tar.gz".into(),
                    },
                    hash: HashSpec {
                        algo: HashAlgo::Sha256,
                        expected: Some("sha256-jZkUKv2SV28wsM18tCqNxoCZmLxdYH2Idh9RLibH2yA=".into()),
                    },
                    frozen: true,
                    mirrors: vec![],
                    patches: vec!["hello-fix".into()],
                },
            ],
            patches: vec![PatchDef {
                name: "hello-fix".into(),
                source: PatchSource::Local {
                    path: "patches/hello-fix.patch".into(),
                },
            }],
        }
    }

    #[test]
    fn manifest_validates_clean() {
        let m = sample_manifest();
        let problems = m.validate();
        assert!(problems.is_empty(), "unexpected problems: {problems:?}");
    }

    #[test]
    fn manifest_detects_duplicate_names() {
        let mut m = sample_manifest();
        m.inputs.push(m.inputs[0].clone());
        let problems = m.validate();
        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains("duplicate input name"));
    }

    #[test]
    fn manifest_detects_undefined_patch() {
        let mut m = sample_manifest();
        m.inputs[0].patches.push("nonexistent".into());
        let problems = m.validate();
        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains("undefined patch"));
    }

    #[test]
    fn manifest_serde_roundtrip() {
        let m = sample_manifest();
        let json = serde_json::to_string_pretty(&m).unwrap();
        let parsed: ProjectManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(m, parsed);
    }

    #[test]
    fn empty_name_is_invalid() {
        let m = ProjectManifest {
            version: "1.0.0".into(),
            inputs: vec![ManifestInput {
                name: "".into(),
                kind: InputKind::File {
                    url: "https://example.com/f".into(),
                },
                hash: HashSpec::default(),
                frozen: false,
                mirrors: vec![],
                patches: vec![],
            }],
            patches: vec![],
        };
        let problems = m.validate();
        assert!(!problems.is_empty());
        assert!(problems[0].contains("must not be empty"));
    }

    #[test]
    fn git_reference_default_is_main() {
        let r = GitReference::default();
        assert_eq!(r, GitReference::Branch("main".into()));
    }

    #[test]
    fn manifest_rejects_bad_version() {
        let m = ProjectManifest {
            version: "garbage".into(),
            inputs: vec![],
            patches: vec![],
        };
        let problems = m.validate();
        assert!(!problems.is_empty());
        assert!(problems[0].contains("invalid manifest version"));
    }

    #[test]
    fn manifest_rejects_incompatible_version() {
        let m = ProjectManifest {
            version: "99.0.0".into(),
            inputs: vec![],
            patches: vec![],
        };
        let problems = m.validate();
        assert!(!problems.is_empty());
        assert!(problems[0].contains("not compatible"));
    }

    #[test]
    fn manifest_accepts_compatible_version() {
        let m = ProjectManifest {
            version: "1.1.0".into(),
            inputs: vec![],
            patches: vec![],
        };
        let problems = m.validate();
        assert!(problems.is_empty());
    }

    #[test]
    fn schema_version_accessor() {
        let m = sample_manifest();
        let v = m.schema_version().unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
    }
}
