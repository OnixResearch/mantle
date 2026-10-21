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

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != MANIFEST_SCHEMA || self.packages.len() > PACKAGES_MAX {
            return Err("unsupported profile schema or too many packages".to_string());
        }
        if !matches!(self.platform.as_str(), "x86_64-linux" | "aarch64-linux") {
            return Err("user profiles support x86_64-linux and aarch64-linux only".to_string());
        }
        if !self.store_prefix.starts_with('/') || self.store_prefix.ends_with('/') {
            return Err("profile requires a normalized absolute store prefix".to_string());
        }
        let mut names = BTreeSet::new();
        for package in &self.packages {
            validate_package(package, &self.store_prefix)?;
            if !names.insert(&package.name) {
                return Err(format!("duplicate package slot: {}", package.name));
            }
        }
        assert!(self.packages.len() <= PACKAGES_MAX);
        assert_eq!(self.schema, MANIFEST_SCHEMA);
        Ok(())
    }

    pub fn install(&self, package: Package, replace: bool) -> Result<Self, String> {
        self.validate()?;
        validate_package(&package, &self.store_prefix)?;
        let mut next = self.clone();
        if let Some(index) = next.packages.iter().position(|old| old.name == package.name) {
            if !replace {
                return Err(format!("package slot {} exists; explicit replace required", package.name));
            }
            if next.packages[index] == package {
                return Err("package is already installed exactly".to_string());
            }
            next.packages[index] = package;
        } else {
            if replace {
                return Err("replace requires an existing package slot".to_string());
            }
            next.packages.push(package);
        }
        next.packages.sort_by(|left, right| left.name.cmp(&right.name));
        next.validate()?;
        assert_eq!(next.platform, self.platform);
        assert_eq!(next.store_prefix, self.store_prefix);
        Ok(next)
    }

    pub fn remove(&self, name: &str) -> Result<Self, String> {
        self.validate()?;
        let mut next = self.clone();
        let Some(index) = next.packages.iter().position(|package| package.name == name) else {
            return Err(format!("package slot not installed: {name}"));
        };
        next.packages.remove(index);
        assert!(next.packages.len() < self.packages.len());
        assert!(!next.packages.iter().any(|package| package.name == name));
        Ok(next)
    }
}

fn validate_package(package: &Package, prefix: &str) -> Result<(), String> {
    if !valid_name(&package.name) || !valid_name(&package.output_name) {
        return Err("package slot and output name require 1..64 ASCII identifier bytes".to_string());
    }
    if package.derivation_aterm.is_empty() || package.derivation_aterm.len() > ATERM_BYTES_MAX {
        return Err("package ATerm is empty or exceeds 65536 bytes".to_string());
    }
    let under = format!("{prefix}/");
    for path in [&package.derivation_path, &package.output_path] {
        let Some(base) = path.strip_prefix(&under) else {
            return Err("package path has wrong store prefix".to_string());
        };
        if base.is_empty() || base.contains('/') || base == "." || base == ".." {
            return Err("package path is not a direct store member".to_string());
        }
    }
    if !package.derivation_path.ends_with(".drv") || package.nar_size == 0 {
        return Err("package requires an exact derivation and nonzero NAR size".to_string());
    }
    if package.nar_sha256.len() != 64
        || !package.nar_sha256.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("package NAR SHA256 must be 64 lowercase hex bytes".to_string());
    }
    assert!(!package.name.is_empty());
    assert!(!package.output_path.is_empty());
    Ok(())
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        && name != "."
        && name != ".."
}

/// Reject leaf/leaf and leaf/directory ambiguity even for identical contents.
pub fn compose(entries: Vec<(String, Option<String>)>) -> Result<BTreeMap<String, String>, String> {
    if entries.len() > ENTRIES_MAX {
        return Err("profile composition exceeds entry bound".to_string());
    }
    let mut directories = BTreeSet::new();
    let mut links = BTreeMap::new();
    for (path, target) in entries {
        let parts: Vec<_> = path.split('/').collect();
        if parts.len() > 64
            || path.len() > 4096
            || !EXPOSED_DIRECTORIES.contains(&parts[0])
            || parts.iter().any(|part| part.is_empty() || *part == "." || *part == "..")
        {
            return Err(format!("invalid profile composition path: {path}"));
        }
        let mut ancestor = String::new();
        for part in parts.iter().take(parts.len().saturating_sub(1)) {
            if !ancestor.is_empty() {
                ancestor.push('/');
            }
            ancestor.push_str(part);
            if links.contains_key(&ancestor) {
                return Err(format!("profile leaf/directory conflict: {ancestor}"));
            }
            directories.insert(ancestor.clone());
        }
        match target {
            None => {
                if links.contains_key(&path) {
                    return Err(format!("profile leaf/directory conflict: {path}"));
                }
                directories.insert(path);
            }
            Some(target) => {
                if !target.starts_with('/') {
                    return Err("profile leaf target must be absolute".to_string());
                }
                if directories.contains(&path) || links.contains_key(&path) {
                    return Err(format!("profile leaf conflict: {path}"));
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
