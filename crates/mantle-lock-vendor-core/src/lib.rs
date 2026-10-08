#![no_std]

//! Bounded Cargo.lock-to-fixed-output-fetch planning. No filesystem, clock, or network access.
//! The shell must verify fetched archive SHA-256 against `Artifact::sha256` before unpacking,
//! generate Cargo's per-file checksum manifest, and publish the assembled tree atomically.

extern crate alloc;
pub mod shared_table;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cmp::Ordering;

use shared_table::HashMode;
use shared_table::SharedHashTable;

pub const MAX_LOCK_BYTES: u32 = 4 * 1024 * 1024;
pub const MAX_ARTIFACTS: u32 = 4096;
pub const MAX_PLAN_BYTES: u32 = 2 * 1024 * 1024;
const MAX_FIELD_BYTES: u32 = 1024;
const MAX_PLAN_ENTRY_OVERHEAD_BYTES: usize = 192;
const REGISTRY_SOURCE: &str = "registry+https://github.com/rust-lang/crates.io-index";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub lock_bytes: u32,
    pub artifacts: u32,
    pub plan_bytes: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            lock_bytes: MAX_LOCK_BYTES,
            artifacts: MAX_ARTIFACTS,
            plan_bytes: MAX_PLAN_BYTES,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenialCode {
    LockBound,
    ArtifactBound,
    PlanBound,
    InvalidUtf8,
    InvalidLock,
    InvalidField,
    DuplicateField,
    DuplicateIdentity,
    ContradictoryEntry,
    MissingHash,
    UnsupportedSource,
    PathEscape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Denial {
    pub code: DenialCode,
    pub artifact: Option<String>,
    pub detail: String,
}

fn deny(code: DenialCode, artifact: Option<&str>, detail: impl Into<String>) -> Denial {
    Denial {
        code,
        artifact: artifact.map(ToString::to_string),
        detail: detail.into(),
    }
}

/// One fixed-output fetch request. Registry archives use the literal lock
/// SHA-256; hash-less Git sources require a pinned recursive hash table entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub name: String,
    pub version: String,
    pub sha256: String,
    pub hash_mode: HashMode,
    pub fetch_url: String,
    pub git_rev: Option<String>,
    /// Exact Cargo.lock source identity consumed from the shared hash table.
    pub shared_hash_identity: Option<String>,
    pub relative_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VendorPlan {
    /// Sorted by Cargo package identity, independent of lock entry order.
    pub artifacts: Vec<Artifact>,
    /// Conservative size budget: artifact strings plus bounded record overhead.
    pub planned_bytes: u32,
}

impl VendorPlan {
    /// Project only the pinned Git rows this plan consumes. The caller must
    /// bind these canonical bytes, not the full mutable table, to its input.
    pub fn selected_shared_hashes(&self, table: &SharedHashTable) -> Result<SharedHashTable, Denial> {
        for artifact in &self.artifacts {
            let Some(identity) = artifact.shared_hash_identity.as_deref() else {
                continue;
            };
            let entry = table.get(identity).ok_or_else(|| {
                deny(DenialCode::MissingHash, Some(&artifact.name), format!("missing shared hash {identity}"))
            })?;
            if entry.mode != artifact.hash_mode || entry.sha256 != artifact.sha256 {
                return Err(deny(
                    DenialCode::ContradictoryEntry,
                    Some(&artifact.name),
                    format!("shared hash changed for {identity}"),
                ));
            }
        }
        table.subset_from_iter(self.artifacts.iter().filter_map(|artifact| artifact.shared_hash_identity.as_deref()))
    }
}

#[derive(Default)]
struct Package<'a> {
    name: Option<&'a str>,
    version: Option<&'a str>,
    source: Option<&'a str>,
    checksum: Option<&'a str>,
}

impl<'a> Package<'a> {
    fn set(&mut self, field: &str, value: &'a str) -> Result<(), Denial> {
        let slot = match field {
            "name" => &mut self.name,
            "version" => &mut self.version,
            "source" => &mut self.source,
            "checksum" => &mut self.checksum,
            _ => return Err(deny(DenialCode::InvalidField, self.name, format!("unknown package field {field}"))),
        };
        if slot.is_some() {
            return Err(deny(DenialCode::DuplicateField, self.name, format!("duplicate {field}")));
        }
        if value.len() > MAX_FIELD_BYTES as usize || value.is_empty() {
            return Err(deny(DenialCode::InvalidField, self.name, format!("invalid {field} length")));
        }
        *slot = Some(value);
        Ok(())
    }
}

fn quoted(value: &str) -> Option<&str> {
    let inner = value.strip_prefix('"')?.strip_suffix('"')?;
    if inner.contains(['"', '\\', '\n', '\r']) {
        return None;
    }
    Some(inner)
}

fn safe_component(component: &str, is_version: bool) -> bool {
    !component.is_empty()
        && component.len() <= 128
        && component.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.') || (is_version && byte == b'+')
        })
        && component != "."
        && component != ".."
}

fn valid_version_identifier(value: &str, prerelease: bool) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        && !(prerelease && value.len() > 1 && value.starts_with('0') && value.bytes().all(|byte| byte.is_ascii_digit()))
}

fn version_number(value: &str) -> Option<u64> {
    if value.len() > 1 && value.starts_with('0') {
        return None;
    }
    value.parse().ok()
}

fn version_parts(version: &str) -> Option<([u64; 3], Option<&str>)> {
    let (without_build, build) = version.split_once('+').map_or((version, None), |(head, tail)| (head, Some(tail)));
    if build.is_some_and(|value| value.split('.').any(|part| !valid_version_identifier(part, false))) {
        return None;
    }
    let (numbers, prerelease) =
        without_build.split_once('-').map_or((without_build, None), |(head, tail)| (head, Some(tail)));
    if prerelease.is_some_and(|value| value.split('.').any(|part| !valid_version_identifier(part, true))) {
        return None;
    }
    let mut parts = numbers.split('.');
    let parsed = [
        version_number(parts.next()?)?,
        version_number(parts.next()?)?,
        version_number(parts.next()?)?,
    ];
    if parts.next().is_some() {
        return None;
    }
    Some((parsed, prerelease))
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    let (left_numbers, left_pre) = version_parts(left).expect("admitted Cargo version");
    let (right_numbers, right_pre) = version_parts(right).expect("admitted Cargo version");
    match left_numbers.cmp(&right_numbers) {
        Ordering::Equal => {}
        other => return other,
    }
    match (left_pre, right_pre) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(left), Some(right)) => {
            let mut left_parts = left.split('.');
            let mut right_parts = right.split('.');
            loop {
                match (left_parts.next(), right_parts.next()) {
                    (None, None) => return Ordering::Equal,
                    (None, Some(_)) => return Ordering::Less,
                    (Some(_), None) => return Ordering::Greater,
                    (Some(l), Some(r)) => {
                        let order = match (l.parse::<u64>(), r.parse::<u64>()) {
                            (Ok(l), Ok(r)) => l.cmp(&r),
                            (Ok(_), Err(_)) => Ordering::Less,
                            (Err(_), Ok(_)) => Ordering::Greater,
                            (Err(_), Err(_)) => l.cmp(r),
                        };
                        if order != Ordering::Equal {
                            return order;
                        }
                    }
                }
            }
        }
    }
}

/// Validate a relative assembly destination independently of lock parsing.
/// This is checked again by the shell before creating each filesystem entry.
pub fn validate_layout_path(path: &str) -> Result<(), Denial> {
    if path.is_empty() || path.len() > MAX_FIELD_BYTES as usize || path.starts_with('/') || path.contains('\\') {
        return Err(deny(DenialCode::PathEscape, Some(path), "layout path must be relative and bounded"));
    }
    if path.split('/').any(|part| part.is_empty() || part == "." || part == ".." || part.contains(':')) {
        return Err(deny(DenialCode::PathEscape, Some(path), "layout path has an unsafe component"));
    }
    Ok(())
}

fn git_fetch_source<'a>(source: &'a str, name: &str) -> Result<(&'a str, &'a str), Denial> {
    let git = source
        .strip_prefix("git+")
        .ok_or_else(|| deny(DenialCode::UnsupportedSource, Some(name), "not a Git source"))?;
    let (url_and_query, rev) = git
        .rsplit_once('#')
        .ok_or_else(|| deny(DenialCode::InvalidField, Some(name), "Git source lacks pinned revision"))?;
    if rev.len() != 40 && rev.len() != 64 {
        return Err(deny(DenialCode::InvalidField, Some(name), "Git revision must be 40 or 64 lowercase hex digits"));
    }
    if !rev.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err(deny(DenialCode::InvalidField, Some(name), "Git revision must be lowercase hex"));
    }
    let url = url_and_query.split_once('?').map_or(url_and_query, |(head, _)| head);
    if !(url.starts_with("https://") || url.starts_with("ssh://")) || url.contains(['\t', '\\']) {
        return Err(deny(DenialCode::UnsupportedSource, Some(name), "Git transport must be HTTPS or SSH"));
    }
    Ok((url, rev))
}

fn admit_package<'a>(
    package: Package<'a>,
    artifacts: &mut BTreeMap<(&'a str, &'a str), Artifact>,
    plan_bytes: &mut u32,
    limits: Limits,
    table: Option<&SharedHashTable>,
) -> Result<(), Denial> {
    let name = package.name.ok_or_else(|| deny(DenialCode::InvalidLock, None, "package missing name"))?;
    let version = package
        .version
        .ok_or_else(|| deny(DenialCode::InvalidLock, Some(name), "package missing version"))?;
    if !safe_component(name, false) || !safe_component(version, true) {
        return Err(deny(DenialCode::PathEscape, Some(name), format!("invalid name/version: {name} {version}")));
    }
    if version_parts(version).is_none() {
        return Err(deny(DenialCode::InvalidField, Some(name), format!("invalid Cargo version {version}")));
    }
    let Some(source) = package.source else {
        if package.checksum.is_some() {
            return Err(deny(DenialCode::ContradictoryEntry, Some(name), "local package claims remote checksum"));
        }
        return Ok(());
    };
    let (checksum, fetch_url, git_rev, hash_mode, shared_hash_identity) = if source.starts_with("git+") {
        if package.checksum.is_some() {
            return Err(deny(
                DenialCode::ContradictoryEntry,
                Some(name),
                "Git lock entry cannot override its shared source hash",
            ));
        }
        let (url, rev) = git_fetch_source(source, name)?;
        let identity = format!("cargo/{name}@{version}/{source}");
        let entry = table.and_then(|table| table.get(&identity)).ok_or_else(|| {
            deny(
                DenialCode::MissingHash,
                Some(name),
                format!("{name} {version} needs pinned shared hash: {identity}"),
            )
        })?;
        if entry.mode != HashMode::RecursiveSha256 {
            return Err(deny(DenialCode::InvalidField, Some(name), "Git source requires recursive SHA-256"));
        }
        let reviewed_url = entry.transport_url.as_deref().unwrap_or(url);
        if !reviewed_url.starts_with("https://") {
            return Err(deny(DenialCode::UnsupportedSource, Some(name), "Git source has no reviewed HTTPS transport"));
        }
        (
            entry.sha256.as_str(),
            reviewed_url.to_string(),
            Some(rev.to_string()),
            HashMode::RecursiveSha256,
            Some(identity),
        )
    } else {
        if source != REGISTRY_SOURCE {
            return Err(deny(DenialCode::UnsupportedSource, Some(name), format!("unsupported fetch source {source}")));
        }
        let checksum = package
            .checksum
            .ok_or_else(|| deny(DenialCode::MissingHash, Some(name), format!("{name} {version} has no SHA-256")))?;
        let url = format!("https://static.crates.io/crates/{name}/{name}-{version}.crate");
        (checksum, url, None, HashMode::FlatSha256, None)
    };
    if checksum.len() != 64 || !checksum.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err(deny(DenialCode::InvalidField, Some(name), "checksum must be lowercase 64-digit SHA-256"));
    }
    let key = (name, version);
    if let Some(previous) = artifacts.get(&key) {
        let code = if previous.sha256 == checksum {
            DenialCode::DuplicateIdentity
        } else {
            DenialCode::ContradictoryEntry
        };
        return Err(deny(
            code,
            Some(name),
            format!("duplicate {name} {version}: {} versus {checksum}", previous.sha256),
        ));
    }
    let artifact_limit = limits.artifacts.min(MAX_ARTIFACTS);
    if artifacts.len() >= artifact_limit as usize {
        return Err(deny(DenialCode::ArtifactBound, Some(name), format!("artifact count exceeds {artifact_limit}")));
    }
    let layout_bytes = name.len() + 1 + version.len();
    let entry_bytes = name.len()
        + version.len()
        + checksum.len()
        + layout_bytes
        + fetch_url.len()
        + git_rev.as_ref().map_or(0, String::len)
        + shared_hash_identity.as_ref().map_or(0, String::len)
        + MAX_PLAN_ENTRY_OVERHEAD_BYTES;
    let new_bytes = (*plan_bytes as u64).saturating_add(entry_bytes as u64);
    let plan_limit = limits.plan_bytes.min(MAX_PLAN_BYTES);
    if new_bytes > plan_limit as u64 {
        return Err(deny(DenialCode::PlanBound, Some(name), format!("plan bytes exceed {plan_limit}")));
    }
    *plan_bytes = new_bytes as u32;
    artifacts.insert(key, Artifact {
        name: name.to_string(),
        version: version.to_string(),
        sha256: checksum.to_string(),
        hash_mode,
        fetch_url,
        git_rev,
        shared_hash_identity,
        relative_path: String::new(),
    });
    Ok(())
}

// r[impl mantle.lock_vendor_fetch.producer_derivation]
// r[impl mantle.lock_vendor_fetch.lock_hash_reuse]
// r[impl mantle.lock_vendor_fetch.negative_controls]
/// Parse the supported Cargo.lock v3/v4 grammar into immutable fetch/layout decisions.
/// Unknown package fields and non-literal identity fields fail closed; dependencies are
/// ignored because the lock's package inventory, not graph edges, defines the vendor set.
pub fn plan_cargo_vendor(lock: &[u8], limits: Limits) -> Result<VendorPlan, Denial> {
    plan_cargo_vendor_with_table(lock, limits, None)
}

/// Only a selected shared-hash subset, never the whole mutable table, may
/// contribute to a package producer's derivation input identity.
pub fn plan_cargo_vendor_with_table(
    lock: &[u8],
    limits: Limits,
    table: Option<&SharedHashTable>,
) -> Result<VendorPlan, Denial> {
    if lock.len() > limits.lock_bytes as usize || lock.len() > MAX_LOCK_BYTES as usize {
        return Err(deny(
            DenialCode::LockBound,
            None,
            format!("lock bytes exceed {}", limits.lock_bytes.min(MAX_LOCK_BYTES)),
        ));
    }
    let text =
        core::str::from_utf8(lock).map_err(|_| deny(DenialCode::InvalidUtf8, None, "Cargo.lock is not UTF-8"))?;
    let mut package: Option<Package<'_>> = None;
    let mut artifacts = BTreeMap::new();
    let mut plan_bytes = 0u32;
    let mut in_dependencies = false;
    let mut version_seen = false;
    let mut ignored_section = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if in_dependencies {
            if line == "]" {
                in_dependencies = false;
                continue;
            }
            let dependency = line.strip_suffix(',').unwrap_or(line);
            if quoted(dependency).is_none_or(|value| value.is_empty() || value.len() > MAX_FIELD_BYTES as usize) {
                return Err(deny(
                    DenialCode::InvalidLock,
                    package.as_ref().and_then(|p| p.name),
                    "invalid dependencies list",
                ));
            }
            continue;
        }
        if line == "[[package]]" {
            if let Some(previous) = package.take() {
                admit_package(previous, &mut artifacts, &mut plan_bytes, limits, table)?;
            }
            package = Some(Package::default());
            ignored_section = false;
            continue;
        }
        if line.starts_with('[') {
            if let Some(previous) = package.take() {
                admit_package(previous, &mut artifacts, &mut plan_bytes, limits, table)?;
            }
            if line != "[metadata]" && line != "[[patch.unused]]" {
                return Err(deny(DenialCode::InvalidLock, None, format!("unsupported section {line}")));
            }
            ignored_section = true;
            continue;
        }
        if ignored_section {
            continue;
        }
        let (field, value) = line.split_once('=').map(|(key, value)| (key.trim(), value.trim())).ok_or_else(|| {
            deny(DenialCode::InvalidLock, package.as_ref().and_then(|p| p.name), "invalid lock assignment")
        })?;
        if let Some(current) = package.as_mut() {
            if field == "dependencies" {
                if value == "[" {
                    in_dependencies = true;
                } else if value != "[]" {
                    return Err(deny(DenialCode::InvalidLock, current.name, "invalid dependencies array"));
                }
                continue;
            }
            let value = quoted(value).ok_or_else(|| {
                deny(DenialCode::InvalidField, current.name, format!("{field} must be a literal string"))
            })?;
            current.set(field, value)?;
        } else if field == "version" && (value == "3" || value == "4") && !version_seen {
            version_seen = true;
        } else {
            return Err(deny(DenialCode::InvalidLock, None, format!("unsupported Cargo.lock header {field}")));
        }
    }
    if in_dependencies {
        return Err(deny(DenialCode::InvalidLock, package.as_ref().and_then(|p| p.name), "unterminated dependencies"));
    }
    if let Some(previous) = package {
        admit_package(previous, &mut artifacts, &mut plan_bytes, limits, table)?;
    }
    if !version_seen {
        return Err(deny(DenialCode::InvalidLock, None, "Cargo.lock requires version 3 or 4"));
    }
    let mut ordered: Vec<Artifact> = artifacts.into_values().collect();
    let mut index = 0;
    while index < ordered.len() {
        let first = index;
        while index < ordered.len() && ordered[index].name == ordered[first].name {
            index += 1;
        }
        let mut highest = first;
        for candidate in first + 1..index {
            if compare_versions(&ordered[candidate].version, &ordered[highest].version) == Ordering::Greater {
                highest = candidate;
            }
        }
        for (offset, artifact) in ordered.iter_mut().enumerate().take(index).skip(first) {
            if offset == highest {
                artifact.relative_path.clone_from(&artifact.name);
            } else {
                artifact.relative_path = format!("{}-{}", artifact.name, artifact.version);
            }
            validate_layout_path(&artifact.relative_path)?;
        }
    }
    let mut paths = alloc::collections::BTreeSet::new();
    for artifact in &ordered {
        if !paths.insert(artifact.relative_path.as_str()) {
            return Err(deny(
                DenialCode::ContradictoryEntry,
                Some(&artifact.name),
                format!("vendor layout collision at {}", artifact.relative_path),
            ));
        }
    }
    Ok(VendorPlan {
        artifacts: ordered,
        planned_bytes: plan_bytes,
    })
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    const SHA_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SHA_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn lock(entries: &str) -> String {
        format!("version = 4\n{entries}")
    }
    fn package(name: &str, sha: &str) -> String {
        format!(
            "[[package]]\nname = \"{name}\"\nversion = \"1.2.0\"\nsource = \"{REGISTRY_SOURCE}\"\nchecksum = \"{sha}\"\n"
        )
    }
    fn code(text: &str, limits: Limits) -> DenialCode {
        plan_cargo_vendor(text.as_bytes(), limits).unwrap_err().code
    }

    #[test]
    fn two_locked_fetches_have_exact_hashes_urls_and_cargo_layout() {
        let text = lock(&format!("{}{}", package("zeta", SHA_B), package("alpha", SHA_A)));
        let plan = plan_cargo_vendor(text.as_bytes(), Limits::default()).unwrap();
        assert_eq!(plan.artifacts.len(), 2);
        assert_eq!(plan.artifacts.iter().map(|a| a.relative_path.as_str()).collect::<Vec<_>>(), vec!["alpha", "zeta"]);
        assert_eq!(plan.artifacts[0].sha256, SHA_A);
        assert_eq!(plan.artifacts[1].sha256, SHA_B);
        assert_eq!(plan.artifacts[0].fetch_url, "https://static.crates.io/crates/alpha/alpha-1.2.0.crate");
        assert!(plan.artifacts.iter().all(|artifact| artifact.shared_hash_identity.is_none()));
        let unrelated = SharedHashTable::parse(format!("{}\ncargo/gitdep@1.0.0/git+https://example.test/source#0123456789abcdef0123456789abcdef01234567\trecursive-sha256\t{SHA_A}\n", shared_table::TABLE_HEADER).as_bytes()).unwrap();
        assert_eq!(
            plan.selected_shared_hashes(&unrelated).unwrap().render(),
            format!("{}\n", shared_table::TABLE_HEADER)
        );
    }

    #[test]
    fn multiple_versions_get_cargo_vendor_suffixes_and_local_package_is_not_fetched() {
        let alternate = package("alpha", SHA_B).replace("1.2.0", "2.0.0");
        let text = lock(&format!(
            "{}{}{}[[package]]\nname = \"local\"\nversion = \"0.1.0\"\n",
            package("alpha", SHA_A),
            alternate,
            package("beta", SHA_A)
        ));
        let plan = plan_cargo_vendor(text.as_bytes(), Limits::default()).unwrap();
        assert_eq!(plan.artifacts.len(), 3);
        assert_eq!(plan.artifacts.iter().map(|artifact| artifact.relative_path.as_str()).collect::<Vec<_>>(), vec![
            "alpha-1.2.0",
            "alpha",
            "beta"
        ]);
    }

    #[test]
    fn cargo_vendor_default_uses_numeric_semver_and_stable_over_prerelease() {
        let old = package("rustix", SHA_A).replace("1.2.0", "9.0.0");
        let newest = package("rustix", SHA_B).replace("1.2.0", "10.0.0-pre.7");
        let stable = package("rustix", SHA_A).replace("1.2.0", "10.0.0");
        let plan = plan_cargo_vendor(lock(&format!("{old}{newest}{stable}")).as_bytes(), Limits::default()).unwrap();
        assert_eq!(plan.artifacts.iter().map(|artifact| artifact.relative_path.as_str()).collect::<Vec<_>>(), vec![
            "rustix",
            "rustix-10.0.0-pre.7",
            "rustix-9.0.0"
        ]);
    }

    #[test]
    fn git_source_uses_only_pinned_recursive_hash_and_revision() {
        const REV: &str = "0123456789abcdef0123456789abcdef01234567";
        let source = format!("git+https://example.test/mono.git?rev={REV}#{REV}");
        let text = lock(&format!("[[package]]\nname = \"gitdep\"\nversion = \"1.0.0\"\nsource = \"{source}\"\n"));
        let identity = format!("cargo/gitdep@1.0.0/{source}");
        let table = SharedHashTable::parse(
            format!("{}\n{identity}\trecursive-sha256\t{SHA_A}\n", shared_table::TABLE_HEADER).as_bytes(),
        )
        .unwrap();
        let plan = plan_cargo_vendor_with_table(text.as_bytes(), Limits::default(), Some(&table)).unwrap();
        assert_eq!(plan.artifacts.len(), 1);
        let artifact = &plan.artifacts[0];
        assert_eq!(artifact.hash_mode, HashMode::RecursiveSha256);
        assert_eq!(artifact.sha256, SHA_A);
        assert_eq!(artifact.fetch_url, "https://example.test/mono.git");
        assert_eq!(artifact.git_rev.as_deref(), Some(REV));
        assert_eq!(artifact.relative_path, "gitdep");
        assert_eq!(artifact.shared_hash_identity.as_deref(), Some(identity.as_str()));
        assert_eq!(plan.selected_shared_hashes(&table).unwrap().render(), table.render());
        let unrelated = format!("cargo/unrelated@1.0.0/git+https://example.test/else#{REV}");
        let extended = SharedHashTable::parse(
            format!(
                "{}\n{identity}\trecursive-sha256\t{SHA_A}\n{unrelated}\trecursive-sha256\t{SHA_B}\n",
                shared_table::TABLE_HEADER
            )
            .as_bytes(),
        )
        .unwrap();
        assert_eq!(plan.selected_shared_hashes(&extended).unwrap().render(), table.render());
        let changed = SharedHashTable::parse(
            format!("{}\n{identity}\trecursive-sha256\t{SHA_B}\n", shared_table::TABLE_HEADER).as_bytes(),
        )
        .unwrap();
        assert_eq!(plan.selected_shared_hashes(&changed).unwrap_err().code, DenialCode::ContradictoryEntry);
        let empty = SharedHashTable::parse(format!("{}\n", shared_table::TABLE_HEADER).as_bytes()).unwrap();
        assert_eq!(plan.selected_shared_hashes(&empty).unwrap_err().code, DenialCode::MissingHash);

        let wrong_mode = SharedHashTable::parse(
            format!("{}\n{identity}\tflat-sha256\t{SHA_A}\n", shared_table::TABLE_HEADER).as_bytes(),
        )
        .unwrap();
        assert_eq!(
            plan_cargo_vendor_with_table(text.as_bytes(), Limits::default(), Some(&wrong_mode))
                .unwrap_err()
                .code,
            DenialCode::InvalidField
        );
        let wrong_revision = text.replace(REV, "bad");
        assert_eq!(
            plan_cargo_vendor_with_table(wrong_revision.as_bytes(), Limits::default(), Some(&table))
                .unwrap_err()
                .code,
            DenialCode::InvalidField
        );
    }

    #[test]
    fn reviewed_git_transport_preserves_lock_identity_and_rejects_undeclared_ssh() {
        const REV: &str = "0123456789abcdef0123456789abcdef01234567";
        let source = format!("git+ssh://git@github.com/example/repo.git?rev={REV}#{REV}");
        let lock = lock(&format!("[[package]]\nname = \"gitdep\"\nversion = \"1.0.0\"\nsource = \"{source}\"\n"));
        let identity = format!("cargo/gitdep@1.0.0/{source}");
        let unreviewed = SharedHashTable::parse(
            format!("{}\n{identity}\trecursive-sha256\t{SHA_A}\n", shared_table::TABLE_HEADER).as_bytes(),
        )
        .unwrap();
        assert_eq!(
            plan_cargo_vendor_with_table(lock.as_bytes(), Limits::default(), Some(&unreviewed))
                .unwrap_err()
                .code,
            DenialCode::UnsupportedSource
        );
        let reviewed = SharedHashTable::parse(
            format!(
                "{}\n{identity}\trecursive-sha256\t{SHA_A}\thttps://github.com/example/repo.git\n",
                shared_table::TABLE_HEADER
            )
            .as_bytes(),
        )
        .unwrap();
        let artifact = plan_cargo_vendor_with_table(lock.as_bytes(), Limits::default(), Some(&reviewed))
            .unwrap()
            .artifacts
            .remove(0);
        assert_eq!(artifact.fetch_url, "https://github.com/example/repo.git");
        assert_eq!(artifact.git_rev.as_deref(), Some(REV));
        assert_eq!(artifact.shared_hash_identity.as_deref(), Some(identity.as_str()));
    }

    #[test]
    fn rejects_bounded_and_hashless_inputs_without_emitting_plan() {
        let text = lock(&package("alpha", SHA_A));
        assert_eq!(
            code(&text, Limits {
                lock_bytes: 8,
                ..Limits::default()
            }),
            DenialCode::LockBound
        );
        assert_eq!(
            code(&text, Limits {
                artifacts: 0,
                ..Limits::default()
            }),
            DenialCode::ArtifactBound
        );
        assert_eq!(
            code(&text, Limits {
                plan_bytes: 1,
                ..Limits::default()
            }),
            DenialCode::PlanBound
        );
        let missing = lock(
            "[[package]]\nname = \"gitdep\"\nversion = \"1.0.0\"\nsource = \"git+https://example.test/repo#0123456789abcdef0123456789abcdef01234567\"\n",
        );
        let denial = plan_cargo_vendor(missing.as_bytes(), Limits::default()).unwrap_err();
        assert_eq!(denial.code, DenialCode::MissingHash);
        assert_eq!(denial.artifact.as_deref(), Some("gitdep"));
    }

    #[test]
    fn absolute_limits_apply_even_if_caller_supplies_larger_limits() {
        let large = vec![b' '; MAX_LOCK_BYTES as usize + 1];
        let permissive = Limits {
            lock_bytes: u32::MAX,
            artifacts: u32::MAX,
            plan_bytes: u32::MAX,
        };
        assert_eq!(plan_cargo_vendor(&large, permissive).unwrap_err().code, DenialCode::LockBound);

        let mut many = String::from("version = 4\n");
        for index in 0..=MAX_ARTIFACTS {
            many.push_str(&package(&format!("crate{index}"), SHA_A));
        }
        assert_eq!(code(&many, permissive), DenialCode::ArtifactBound);
    }

    #[test]
    fn rejects_path_escape_duplicate_and_contradictory_entries() {
        assert_eq!(validate_layout_path("../escape").unwrap_err().code, DenialCode::PathEscape);
        assert_eq!(validate_layout_path("package//file").unwrap_err().code, DenialCode::PathEscape);
        assert_eq!(validate_layout_path("package\\file").unwrap_err().code, DenialCode::PathEscape);
        assert_eq!(code(&lock(&package("../escape", SHA_A)), Limits::default()), DenialCode::PathEscape);
        let same = lock(&format!("{}{}", package("alpha", SHA_A), package("alpha", SHA_A)));
        assert_eq!(code(&same, Limits::default()), DenialCode::DuplicateIdentity);
        let conflict = lock(&format!("{}{}", package("alpha", SHA_A), package("alpha", SHA_B)));
        assert_eq!(code(&conflict, Limits::default()), DenialCode::ContradictoryEntry);
        let duplicate_field = lock(&format!("{}checksum = \"{SHA_B}\"\n", package("alpha", SHA_A)));
        assert_eq!(code(&duplicate_field, Limits::default()), DenialCode::DuplicateField);
        let older = package("foo", SHA_A).replace("1.2.0", "1.0.0");
        let newer = package("foo", SHA_B).replace("1.2.0", "2.0.0");
        let colliding = package("foo-1.0.0", SHA_A);
        assert_eq!(
            code(&lock(&format!("{older}{newer}{colliding}")), Limits::default()),
            DenialCode::ContradictoryEntry
        );
    }

    #[test]
    fn rejects_untrusted_grammar_instead_of_guessing_source_or_hash() {
        let bad_hash = lock(&package("alpha", "ABCD"));
        assert_eq!(code(&bad_hash, Limits::default()), DenialCode::InvalidField);
        let bad_source = lock(&package("alpha", SHA_A).replace(REGISTRY_SOURCE, "registry+https://host.invalid/index"));
        assert_eq!(code(&bad_source, Limits::default()), DenialCode::UnsupportedSource);
        let bad_version = lock(&package("alpha", SHA_A).replace("1.2.0", "01.2.0"));
        assert_eq!(code(&bad_version, Limits::default()), DenialCode::InvalidField);
        let local_hash = lock(
            "[[package]]\nname = \"local\"\nversion = \"0.1.0\"\nchecksum = \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\n",
        );
        assert_eq!(code(&local_hash, Limits::default()), DenialCode::ContradictoryEntry);
        let malformed = lock(&format!("{}dependencies = [\n\"unterminated,\n]\n", package("alpha", SHA_A)));
        assert_eq!(code(&malformed, Limits::default()), DenialCode::InvalidLock);
    }
}
