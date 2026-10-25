//! Exact, resolver-free user package profile composition policy.
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const MANIFEST_SCHEMA: &str = "mantle-user-profile-v1";
pub const GENERATION_SCHEMA: &str = "mantle-user-profile-generation-v1";
pub const PACKAGES_MAX: usize = 64;
pub const ENTRIES_MAX: usize = 65_536;
pub const ATERM_BYTES_MAX: usize = 65_536;
pub const EXPOSED_DIRECTORIES: [&str; 6] = ["bin", "lib", "libexec", "share", "include", "etc"];

/// Manifest transitions and forest conflicts, without pre-rendered diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    SchemaOrPackageBound,
    UnsupportedPlatform,
    StorePrefix,
    DuplicateSlot(String),
    ReplaceRequired(String),
    AlreadyInstalled,
    MissingReplacement,
    SlotNotInstalled(String),
    PackageName,
    PackageAterm,
    PackageStorePrefix,
    PackageStoreMember,
    PackageDerivation,
    PackageNarHash,
    EntryBound,
    CompositionPath(String),
    LeafDirectoryConflict(String),
    AbsoluteLeafTarget,
    LeafConflict(String),
}

impl core::fmt::Display for ProfileError {
    fn fmt(&self, output: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SchemaOrPackageBound => output.write_str("unsupported profile schema or too many packages"),
            Self::UnsupportedPlatform => output.write_str("user profiles support x86_64-linux and aarch64-linux only"),
            Self::StorePrefix => output.write_str("profile requires a normalized absolute store prefix"),
            Self::DuplicateSlot(name) => write!(output, "duplicate package slot: {name}"),
            Self::ReplaceRequired(name) => write!(output, "package slot {name} exists; explicit replace required"),
            Self::AlreadyInstalled => output.write_str("package is already installed exactly"),
            Self::MissingReplacement => output.write_str("replace requires an existing package slot"),
            Self::SlotNotInstalled(name) => write!(output, "package slot not installed: {name}"),
            Self::PackageName => output.write_str("package slot and output name require 1..64 ASCII identifier bytes"),
            Self::PackageAterm => output.write_str("package ATerm is empty or exceeds 65536 bytes"),
            Self::PackageStorePrefix => output.write_str("package path has wrong store prefix"),
            Self::PackageStoreMember => output.write_str("package path is not a direct store member"),
            Self::PackageDerivation => output.write_str("package requires an exact derivation and nonzero NAR size"),
            Self::PackageNarHash => output.write_str("package NAR SHA256 must be 64 lowercase hex bytes"),
            Self::EntryBound => output.write_str("profile composition exceeds entry bound"),
            Self::CompositionPath(path) => write!(output, "invalid profile composition path: {path}"),
            Self::LeafDirectoryConflict(path) => write!(output, "profile leaf/directory conflict: {path}"),
            Self::AbsoluteLeafTarget => output.write_str("profile leaf target must be absolute"),
            Self::LeafConflict(path) => write!(output, "profile leaf conflict: {path}"),
        }
    }
}

impl core::error::Error for ProfileError {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ConflictPolicy {
    MergeDirectoriesRejectLeavesV1,
}

/// Package names are local slots, not resolver selectors. ATerm and output
/// identities are exact; the adapter authenticates these against admitted data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub name: String,
    pub derivation_path: String,
    pub derivation_aterm: String,
    pub output_name: String,
    pub output_path: String,
    pub nar_sha256: String,
    pub nar_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub platform: String,
    pub store_prefix: String,
    pub conflict_policy: ConflictPolicy,
    pub packages: Vec<Package>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Generation {
    pub schema: String,
    pub number: u64,
    pub parent: Option<u64>,
    pub manifest: Manifest,
    /// Relative exposed leaf -> absolute physical admitted output leaf.
    pub links: BTreeMap<String, String>,
}

impl Manifest {
    pub fn empty(platform: String, store_prefix: String) -> Self {
        assert!(!platform.is_empty());
        assert!(!store_prefix.is_empty());
        Self {
            schema: MANIFEST_SCHEMA.to_string(),
            platform,
            store_prefix,
            conflict_policy: ConflictPolicy::MergeDirectoriesRejectLeavesV1,
            packages: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.schema != MANIFEST_SCHEMA || self.packages.len() > PACKAGES_MAX {
            return Err(ProfileError::SchemaOrPackageBound);
        }
        if !matches!(self.platform.as_str(), "x86_64-linux" | "aarch64-linux") {
            return Err(ProfileError::UnsupportedPlatform);
        }
        if !self.store_prefix.starts_with('/') || self.store_prefix.ends_with('/') {
            return Err(ProfileError::StorePrefix);
        }
        let mut names = BTreeSet::new();
        for package in &self.packages {
            validate_package(package, &self.store_prefix)?;
            if !names.insert(&package.name) {
                return Err(ProfileError::DuplicateSlot(package.name.clone()));
            }
        }
        assert!(self.packages.len() <= PACKAGES_MAX);
        assert_eq!(self.schema, MANIFEST_SCHEMA);
        Ok(())
    }

    pub fn install(&self, package: Package, replace: bool) -> Result<Self, ProfileError> {
        self.validate()?;
        validate_package(&package, &self.store_prefix)?;
        let mut next = self.clone();
        if let Some(index) = next.packages.iter().position(|old| old.name == package.name) {
            if !replace {
                return Err(ProfileError::ReplaceRequired(package.name));
            }
            if next.packages[index] == package {
                return Err(ProfileError::AlreadyInstalled);
            }
            next.packages[index] = package;
        } else {
            if replace {
                return Err(ProfileError::MissingReplacement);
            }
            next.packages.push(package);
        }
        next.packages.sort_by(|left, right| left.name.cmp(&right.name));
        next.validate()?;
        assert_eq!(next.platform, self.platform);
        assert_eq!(next.store_prefix, self.store_prefix);
        Ok(next)
    }

    pub fn remove(&self, name: &str) -> Result<Self, ProfileError> {
        self.validate()?;
        let mut next = self.clone();
        let Some(index) = next.packages.iter().position(|package| package.name == name) else {
            return Err(ProfileError::SlotNotInstalled(name.to_string()));
        };
        next.packages.remove(index);
        assert!(next.packages.len() < self.packages.len());
        assert!(!next.packages.iter().any(|package| package.name == name));
        Ok(next)
    }
}

fn validate_package(package: &Package, prefix: &str) -> Result<(), ProfileError> {
    if !valid_name(&package.name) || !valid_name(&package.output_name) {
        return Err(ProfileError::PackageName);
    }
    if package.derivation_aterm.is_empty() || package.derivation_aterm.len() > ATERM_BYTES_MAX {
        return Err(ProfileError::PackageAterm);
    }
    let under = format!("{prefix}/");
    for path in [&package.derivation_path, &package.output_path] {
        let Some(base) = path.strip_prefix(&under) else {
            return Err(ProfileError::PackageStorePrefix);
        };
        let is_reserved = base == "." || base == "..";
        if base.is_empty() || base.contains('/') || is_reserved {
            return Err(ProfileError::PackageStoreMember);
        }
    }
    if !package.derivation_path.ends_with(".drv") || package.nar_size == 0 {
        return Err(ProfileError::PackageDerivation);
    }
    if package.nar_sha256.len() != 64
        || !package.nar_sha256.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ProfileError::PackageNarHash);
    }
    assert!(!package.name.is_empty());
    assert!(!package.output_path.is_empty());
    Ok(())
}

fn valid_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 {
        return false;
    }
    if name == "." || name == ".." {
        return false;
    }
    name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

const PATH_PARTS_MAX: usize = 64;
const PATH_BYTES_MAX: usize = 4096;

/// Reject leaf/leaf and leaf/directory ambiguity even for identical contents.
pub fn compose(entries: Vec<(String, Option<String>)>) -> Result<BTreeMap<String, String>, ProfileError> {
    if entries.len() > ENTRIES_MAX {
        return Err(ProfileError::EntryBound);
    }
    let mut directories = BTreeSet::new();
    let mut links = BTreeMap::new();
    for (path, target) in entries {
        let parts: Vec<&str> = path.split('/').collect();
        let is_exposed = EXPOSED_DIRECTORIES.contains(&parts[0]);
        let has_invalid_part = parts.iter().any(|part| part.is_empty() || *part == "." || *part == "..");
        if parts.len() > PATH_PARTS_MAX || path.len() > PATH_BYTES_MAX {
            return Err(ProfileError::CompositionPath(path));
        }
        if !is_exposed || has_invalid_part {
            return Err(ProfileError::CompositionPath(path));
        }
        let mut ancestor = String::with_capacity(path.len());
        for part in parts.iter().take(parts.len().saturating_sub(1)) {
            if !ancestor.is_empty() {
                ancestor.push('/');
            }
            ancestor.push_str(part);
            if links.contains_key(&ancestor) {
                return Err(ProfileError::LeafDirectoryConflict(ancestor));
            }
            directories.insert(ancestor.clone());
        }
        match target {
            None => {
                if links.contains_key(&path) {
                    return Err(ProfileError::LeafDirectoryConflict(path));
                }
                directories.insert(path);
            }
            Some(target) => {
                if !target.starts_with('/') {
                    return Err(ProfileError::AbsoluteLeafTarget);
                }
                if directories.contains(&path) || links.contains_key(&path) {
                    return Err(ProfileError::LeafConflict(path));
                }
                if links.len() >= ENTRIES_MAX {
                    return Err(ProfileError::EntryBound);
                }
                links.insert(path, target);
            }
        }
    }
    assert!(links.len() <= ENTRIES_MAX);
    assert!(links.keys().all(|path| !path.starts_with('/')));
    Ok(links)
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn rejects_identical_binary_and_directory_leaf_conflicts() {
        assert!(
            compose(vec![
                ("bin/tool".into(), Some("/store/a/bin/tool".into())),
                ("bin/tool".into(), Some("/store/a/bin/tool".into()))
            ])
            .is_err()
        );
        assert!(
            compose(vec![
                ("share/doc".into(), Some("/store/a/share/doc".into())),
                ("share/doc/readme".into(), Some("/store/b/share/doc/readme".into()))
            ])
            .is_err()
        );
        assert!(
            compose(vec![
                ("share/doc/readme".into(), Some("/store/b/share/doc/readme".into())),
                ("share/doc".into(), Some("/store/a/share/doc".into()))
            ])
            .is_err()
        );
    }

    #[test]
    fn directories_merge_without_binary_precedence() {
        let links = compose(vec![
            ("bin".into(), None),
            ("bin/a".into(), Some("/store/a/bin/a".into())),
            ("bin".into(), None),
            ("bin/b".into(), Some("/store/b/bin/b".into())),
        ])
        .unwrap();
        assert_eq!(links["bin/a"], "/store/a/bin/a");
        assert_eq!(links["bin/b"], "/store/b/bin/b");
    }
}
