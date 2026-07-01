use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use crate::fetch_policy::InputFetchPolicy;
use crate::fetch_policy::fetch_policy_compatibility_problems;
use crate::version::SchemaVersion;
use crate::version::parse_version;

pub const MAX_INPUTS: u32 = 4096;
pub const MAX_MIRRORS_PER_INPUT: u32 = 64;
pub const MAX_PATCHES_PER_INPUT: u32 = 256;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectManifest {
    pub version: String,
    pub inputs: Vec<ManifestInput>,
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
    where D: Deserializer<'de> {
        let raw = RawProjectManifest::deserialize(deserializer)?;
        Ok(Self {
            version: raw.version,
            inputs: raw.inputs,
            patches: raw.patches.unwrap_or_else(Vec::new),
        })
    }
}

impl ProjectManifest {
    pub fn schema_version(self) -> Option<SchemaVersion> {
        parse_version(self.version)
    }

    pub fn validate(self) -> Vec<String> {
        let ProjectManifest {
            version,
            inputs,
            patches,
        } = self;
        let mut problems = Vec::with_capacity(inputs.len().saturating_mul(2).saturating_add(patches.len()));

        match parse_version(version.clone()) {
            None => {
                problems.push(format!("invalid manifest version: '{}'", version));
            }
            Some(v) if !v.is_compatible_with(SchemaVersion::CURRENT) => {
                problems.push(format!(
                    "manifest version {} is not compatible with current version {}",
                    v,
                    SchemaVersion::CURRENT
                ));
            }
            _ => {}
        }

        if inputs.len() as u64 > MAX_INPUTS as u64 {
            problems.push(format!("too many inputs: {} (max {MAX_INPUTS})", inputs.len()));
        }

        let mut seen_names = BTreeSet::new();
        for input in &inputs {
            if !seen_names.insert(input.name.as_str()) {
                problems.push(format!("duplicate input name: {}", input.name));
            }
            problems.extend(input.validate());
        }

        let patch_names: BTreeSet<&str> = patches.iter().map(|p| p.name.as_str()).collect();
        for input in &inputs {
            for patch_ref in &input.patches {
                if !patch_names.contains(patch_ref.as_str()) {
                    problems.push(format!("input '{}' references undefined patch '{patch_ref}'", input.name));
                }
            }
        }

        problems
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ManifestInput {
    pub name: String,
    pub kind: InputKind,
    pub hash: HashSpec,
    pub frozen: bool,
    pub mirrors: Vec<String>,
    pub patches: Vec<String>,
    pub fetch_policy: InputFetchPolicy,
}

#[derive(Deserialize)]
struct RawManifestInput {
    name: String,
    kind: InputKind,
    hash: Option<HashSpec>,
    frozen: Option<bool>,
    mirrors: Option<Vec<String>>,
    patches: Option<Vec<String>>,
    fetch_policy: Option<InputFetchPolicy>,
}

impl<'de> Deserialize<'de> for ManifestInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let raw = RawManifestInput::deserialize(deserializer)?;
        Ok(Self {
            name: raw.name,
            kind: raw.kind,
            hash: raw.hash.unwrap_or_default(),
            frozen: raw.frozen.unwrap_or(false),
            mirrors: raw.mirrors.unwrap_or_else(Vec::new),
            patches: raw.patches.unwrap_or_else(Vec::new),
            fetch_policy: raw.fetch_policy.unwrap_or_default(),
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

        problems.extend(fetch_policy_compatibility_problems(self));

        problems
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum InputKind {
    #[serde(rename = "file")]
    File { url: String },
    #[serde(rename = "tarball")]
    Tarball { url: String },
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
    where D: Deserializer<'de> {
        let raw = RawInputKind::deserialize(deserializer)?;
        Ok(match raw {
            RawInputKind::File { url } => Self::File { url },
            RawInputKind::Tarball { url } => Self::Tarball { url },
            RawInputKind::Git { repository, reference } => Self::Git {
                repository,
                reference: reference.unwrap_or_else(GitReference::default),
            },
        })
    }
}

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

impl fmt::Display for HashAlgo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashAlgo::Sha256 => write!(f, "sha256"),
            HashAlgo::Sha512 => write!(f, "sha512"),
            HashAlgo::Blake3 => write!(f, "blake3"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct HashSpec {
    pub algo: HashAlgo,
    pub expected: Option<String>,
}

#[derive(Deserialize)]
struct RawHashSpec {
    algo: Option<HashAlgo>,
    expected: Option<String>,
}

impl<'de> Deserialize<'de> for HashSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let raw = RawHashSpec::deserialize(deserializer)?;
        Ok(Self {
            algo: raw.algo.unwrap_or_default(),
            expected: raw.expected,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchDef {
    pub name: String,
    pub source: PatchSource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PatchSource {
    #[serde(rename = "local")]
    Local { path: String },
    #[serde(rename = "remote")]
    Remote { url: String, hash: HashSpec },
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

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
                    fetch_policy: InputFetchPolicy::GenerationMaterial,
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
                    fetch_policy: InputFetchPolicy::GenerationMaterial,
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
                fetch_policy: InputFetchPolicy::GenerationMaterial,
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
    fn fetch_policy_default_is_generation_material() {
        let json = r#"{
            "name":"src",
            "kind":{"type":"file","url":"https://example.com/src"}
        }"#;
        let input: ManifestInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.fetch_policy, InputFetchPolicy::GenerationMaterial);
    }

    #[test]
    fn build_fetch_policy_with_patches_is_invalid() {
        let mut m = sample_manifest();
        m.inputs[0].fetch_policy = InputFetchPolicy::BuildFetchAction;
        m.inputs[0].patches.push("hello-fix".into());
        let problems = m.validate();
        assert!(problems.iter().any(|problem| problem.contains("build-fetch-action")));
    }

    #[test]
    fn unknown_fetch_policy_string_is_rejected() {
        let json = r#"{
            "name":"src",
            "kind":{"type":"file","url":"https://example.com/src"},
            "fetch_policy":"surprise-network"
        }"#;
        let error = serde_json::from_str::<ManifestInput>(json).unwrap_err();
        assert!(error.to_string().contains("unknown variant"));
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
