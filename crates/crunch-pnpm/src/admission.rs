//! Lock admission deliberately leaves dependency, peer and optional resolution to pinned pnpm.
use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use crunch_pnpm_core::Package;
use crunch_pnpm_core::Request;
use serde_json::Value;
use sha2::Digest;
use sha2::Sha512;

use crate::files::Result;
use crate::files::{self};

/// Ambient configuration, cache and hook entries forbidden anywhere in the source tree.
const FORBIDDEN_SOURCE_ENTRIES: [&str; 6] = [
    "node_modules",
    ".pnpmfile.cjs",
    ".pnpmfile.js",
    ".npmrc",
    ".pnpm-store",
    ".git",
];
/// Lifecycle hooks that must never run from an admitted manifest.
const LIFECYCLE_HOOKS: [&str; 6] = [
    "preinstall",
    "install",
    "postinstall",
    "prepare",
    "prepublish",
    "prepublishOnly",
];
/// Deepest lock or importer nesting admitted.
const MAX_LINK_DEPTH: u32 = 64;
/// Largest number of entries admitted in one package archive.
const MAX_ARCHIVE_ENTRIES: u32 = 100_000;
/// Chunk size of the bounded archive read loop.
const ARCHIVE_CHUNK_BYTES: usize = 16 * 1024;
/// Read iterations: the input byte bound over the chunk size, plus the terminating empty read.
const MAX_ARCHIVE_CHUNKS: u64 = 32_769;
/// Working-set size of the explicit traversal stacks.
const STACK_CAPACITY: usize = 64;

pub fn admit(request: &Request, source: &Path, mirror: &Path) -> Result<Value> {
    let lock: Value = serde_yaml::from_slice(&files::document(&source.join("pnpm-lock.yaml"))?)?;
    check_lock_header(&lock, request)?;
    let root = files::json(&source.join("package.json"))?;
    check_root_manifest(&root)?;
    check_manifest(&root, &BTreeSet::new())?;
    inspect_source(source)?;
    check_workspace(&root, &lock, source)?;
    check_packages(request, &lock, mirror)?;
    check_targets(request, &lock, source)?;
    check_importers(request, &lock, source)?;
    check_links(&lock, 0)?;
    Ok(lock)
}

/// The lockfile version and package inventory admitted for this request.
fn check_lock_header(lock: &Value, request: &Request) -> Result<()> {
    if lock["lockfileVersion"].as_str() != Some("9.0") {
        return Err("requires quoted lockfileVersion '9.0'".into());
    }
    let inventory = lock["packages"].as_object().ok_or("lock has no package inventory")?;
    if inventory.len() != request.packages.len() {
        return Err("package inventory differs from admitted mirror".into());
    }
    debug_assert_eq!(inventory.len(), request.packages.len());
    Ok(())
}

/// The root manifest must pin the exact pnpm release this shell runs.
fn check_root_manifest(root: &Value) -> Result<()> {
    if root["packageManager"] != "pnpm@9.15.9" {
        return Err("root packageManager must pin pnpm@9.15.9".into());
    }
    Ok(())
}

/// Workspace configuration and patched dependencies admitted from the root manifest.
fn check_workspace(root: &Value, lock: &Value, source: &Path) -> Result<()> {
    let workspace: Value = serde_yaml::from_slice(&files::document(&source.join("pnpm-workspace.yaml"))?)?;
    if workspace.as_object().ok_or("workspace must be a record")?.keys().any(|k| k != "packages") {
        return Err("workspace configuration supports only explicit packages globs".into());
    }
    check_patches(root, lock, source)
}

/// The patched dependency inventory must match the lock exactly.
fn check_patches(root: &Value, lock: &Value, source: &Path) -> Result<()> {
    if let Some(patches) = root["pnpm"]["patchedDependencies"].as_object() {
        if patches.len() != lock["patchedDependencies"].as_object().ok_or("missing locked patches")?.len() {
            return Err("stale patch inventory".into());
        }
        for (key, value) in patches {
            let path = value.as_str().ok_or("patch path must be a string")?;
            files::bytes(&files::child(source, path)?)?;
            if lock["patchedDependencies"][key]["path"] != *value {
                return Err("stale patch path".into());
            }
        }
    } else if lock["patchedDependencies"].as_object().is_some_and(|v| !v.is_empty()) {
        return Err("locked patches missing from root manifest".into());
    }
    Ok(())
}

/// Every declared package must be locked, integrity pinned and expansion bounded.
fn check_packages(request: &Request, lock: &Value, mirror: &Path) -> Result<()> {
    let inventory = lock["packages"].as_object().ok_or("lock has no package inventory")?;
    let mut expanded_bytes = 0_u64;
    for package in &request.packages {
        let entry = inventory.get(&package.lock_key).ok_or("missing locked package")?;
        check_resolution(package, entry)?;
        check_tarball_locator(package, entry)?;
        expanded_bytes = expanded_bytes
            .checked_add(inspect_archive(package, &files::child(mirror, &package.archive)?)?)
            .ok_or("package expansion overflow")?;
        if expanded_bytes > crunch_pnpm_core::MAX_INPUT_BYTES {
            return Err("total package expansion bound exceeded".into());
        }
    }
    debug_assert!(expanded_bytes <= crunch_pnpm_core::MAX_INPUT_BYTES);
    debug_assert_eq!(inventory.len(), request.packages.len());
    Ok(())
}

/// A locked package must resolve by integrity only and must not need a native build.
fn check_resolution(package: &Package, entry: &Value) -> Result<()> {
    if entry["requiresBuild"] == true {
        return Err("dependency lifecycle/native builds are unsupported; provide prebuilt integrity archives".into());
    }
    let resolution = entry["resolution"].as_object().ok_or("missing resolution")?;
    let is_integrity_only = !resolution.keys().any(|k| k != "integrity" && k != "tarball");
    let is_integrity_admitted = entry["resolution"]["integrity"].as_str() == Some(package.integrity.as_str());
    if !is_integrity_only || !is_integrity_admitted {
        return Err("unsupported or drifting package resolution (integrity tarballs only)".into());
    }
    Ok(())
}

/// Tarball locators must be credential free and Git archives must bind the admitted revision.
fn check_tarball_locator(package: &Package, entry: &Value) -> Result<()> {
    if let Some(url) = entry["resolution"]["tarball"].as_str()
        && (!url.starts_with("https://") || url.contains(['@', '?', '#']))
    {
        return Err("tarball locators must be credential-free HTTPS URLs".into());
    }
    if let Some(revision) = &package.git_revision {
        let url = entry["resolution"]["tarball"].as_str().ok_or("Git archives require explicit tarball URL")?;
        let is_bound_revision = url
            .split('/')
            .any(|segment| segment == revision.as_str() || segment.strip_suffix(".tar.gz") == Some(revision.as_str()));
        if !is_bound_revision {
            return Err("Git archive URL must bind admitted revision as an exact path segment".into());
        }
    }
    Ok(())
}

/// Every declared target must be a locked importer whose scripts are declared.
fn check_targets(request: &Request, lock: &Value, source: &Path) -> Result<()> {
    for target in &request.targets {
        let manifest = files::json(&files::child(source, &target.workspace)?.join("package.json"))?;
        let allowed = target.scripts.iter().map(String::as_str).collect();
        check_manifest(&manifest, &allowed)?;
        for script in &target.scripts {
            if manifest["scripts"][script].as_str().is_none() {
                return Err("declared script missing from workspace".into());
            }
        }
        if lock["importers"].get(&target.workspace).is_none() {
            return Err("target is not a locked workspace importer".into());
        }
    }
    Ok(())
}

/// Every locked importer must stay inside the source and declare no lifecycle hooks.
fn check_importers(request: &Request, lock: &Value, source: &Path) -> Result<()> {
    // Validate all importer manifests, not only selected targets. No unbounded install hooks.
    let locked = lock["importers"].as_object().ok_or("missing importers")?;
    let mut checked = 0_usize;
    for (path, importer) in locked {
        let directory = if path == "." {
            source.to_path_buf()
        } else {
            files::child(source, path)?
        };
        validate_links(importer, &directory, source, 0)?;
        if path == "." {
            continue;
        }
        let manifest = files::json(&files::child(source, path)?.join("package.json"))?;
        let allowed = request
            .targets
            .iter()
            .find(|t| &t.workspace == path)
            .map(|t| t.scripts.iter().map(String::as_str).collect())
            .unwrap_or_default();
        check_manifest(&manifest, &allowed)?;
        checked = checked.saturating_add(1);
    }
    debug_assert!(lock["importers"].is_object());
    debug_assert!(checked <= locked.len());
    Ok(())
}

/// Lifecycle hooks are forbidden and only patched dependencies may be configured.
fn check_manifest(manifest: &Value, allowed: &BTreeSet<&str>) -> Result<()> {
    check_lifecycle_hooks(manifest, allowed)?;
    check_pnpm_configuration(manifest)
}

/// A manifest may only declare a lifecycle hook the request explicitly allows.
fn check_lifecycle_hooks(manifest: &Value, allowed: &BTreeSet<&str>) -> Result<()> {
    for hook in LIFECYCLE_HOOKS {
        let is_declared = manifest["scripts"].get(hook).is_some();
        if is_declared && !allowed.contains(hook) {
            return Err(format!("prohibited lifecycle script: {hook}").into());
        }
    }
    Ok(())
}

/// Only the patched dependency configuration is supported under `pnpm`.
fn check_pnpm_configuration(manifest: &Value) -> Result<()> {
    if let Some(config) = manifest["pnpm"].as_object()
        && config.keys().any(|key| key != "patchedDependencies")
    {
        return Err("only patchedDependencies pnpm configuration is supported".into());
    }
    Ok(())
}

/// Ambient configuration, cache and hook entries are forbidden anywhere in the source tree.
fn inspect_source(path: &Path) -> Result<()> {
    let mut pending = Vec::with_capacity(STACK_CAPACITY);
    push_children(&mut pending, path)?;
    while let Some(current) = pending.pop() {
        let name = current.file_name().and_then(|entry| entry.to_str()).unwrap_or_default();
        if FORBIDDEN_SOURCE_ENTRIES.contains(&name) {
            return Err(format!("ambient configuration/cache/hook forbidden: {}", current.display()).into());
        }
        if current.is_dir() {
            push_children(&mut pending, &current)?;
        }
    }
    Ok(())
}

/// Push the entries of a directory in reverse so the stack visits them in read order.
fn push_children(pending: &mut Vec<PathBuf>, dir: &Path) -> Result<()> {
    let mut entries = Vec::with_capacity(STACK_CAPACITY);
    for entry in std::fs::read_dir(dir)? {
        entries.push(entry?.path());
    }
    entries.reverse();
    pending.extend(entries);
    Ok(())
}

/// Raw Git and file resolutions are forbidden and workspace links must stay relative.
fn check_links(value: &Value, depth: u32) -> Result<()> {
    let mut pending = Vec::with_capacity(STACK_CAPACITY);
    pending.push((value, depth));
    while let Some((current, current_depth)) = pending.pop() {
        if current_depth > MAX_LINK_DEPTH {
            return Err("lock nesting bound exceeded".into());
        }
        check_link_value(current)?;
        match current {
            Value::Object(object) => push_values(&mut pending, object.values(), current_depth),
            Value::Array(array) => push_values(&mut pending, array.iter(), current_depth),
            _ => (),
        }
    }
    Ok(())
}

/// A locked locator may not be a raw Git or file reference and a workspace link stays relative.
fn check_link_value(value: &Value) -> Result<()> {
    let Value::String(text) = value else {
        return Ok(());
    };
    if text.starts_with("file:") || text.starts_with("git+") || text.starts_with("git:") {
        return Err("raw Git/file resolutions unsupported; use integrity-pinned archives or workspace links".into());
    }
    if let Some(member) = text.strip_prefix("link:") {
        // pnpm importer-relative ../sibling links are valid, but must remain in source.
        if member.starts_with('/') || member.contains('\\') {
            return Err("unsafe workspace link".into());
        }
    }
    Ok(())
}

/// Workspace links must resolve inside the source and name a readable manifest.
fn validate_links(value: &Value, importer: &Path, root: &Path, depth: u32) -> Result<()> {
    let mut pending = Vec::with_capacity(STACK_CAPACITY);
    pending.push((value, depth));
    while let Some((current, current_depth)) = pending.pop() {
        if current_depth > MAX_LINK_DEPTH {
            return Err("link nesting bound exceeded".into());
        }
        validate_link_value(current, importer, root)?;
        match current {
            Value::Object(object) => push_values(&mut pending, object.values(), current_depth),
            Value::Array(array) => push_values(&mut pending, array.iter(), current_depth),
            _ => (),
        }
    }
    Ok(())
}

/// A workspace link must resolve inside the source and name a readable manifest.
fn validate_link_value(value: &Value, importer: &Path, root: &Path) -> Result<()> {
    let Value::String(text) = value else {
        return Ok(());
    };
    let Some(member) = text.strip_prefix("link:") else {
        return Ok(());
    };
    let resolved = std::fs::canonicalize(importer.join(member))?;
    if !resolved.starts_with(root) {
        return Err("workspace link escapes source".into());
    }
    files::json(&resolved.join("package.json"))?;
    Ok(())
}

/// Push nested values in reverse so the stack visits them in declaration order.
fn push_values<'a>(pending: &mut Vec<(&'a Value, u32)>, values: impl Iterator<Item = &'a Value>, depth: u32) {
    let mut children = Vec::with_capacity(STACK_CAPACITY);
    for value in values {
        children.push((value, depth.saturating_add(1)));
    }
    children.reverse();
    pending.extend(children);
}

/// Verify the archive identity, its expanded size and its embedded manifest.
fn inspect_archive(package: &Package, path: &Path) -> Result<u64> {
    let mut file = std::fs::File::open(path)?;
    let mut blake = blake3::Hasher::new();
    let mut sha = Sha512::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; ARCHIVE_CHUNK_BYTES];
    for _ in 0..MAX_ARCHIVE_CHUNKS {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total = total.checked_add(count as u64).ok_or("archive byte overflow")?;
        if total > crunch_pnpm_core::MAX_INPUT_BYTES {
            return Err("archive byte bound exceeded".into());
        }
        blake.update(&buffer[..count]);
        sha.update(&buffer[..count]);
    }
    if blake3::Hasher::finalize(&blake).to_hex().as_str() != package.blake3.as_str()
        || format!("sha512-{}", STANDARD.encode(sha.finalize())) != package.integrity
    {
        return Err("archive integrity drift".into());
    }
    let expanded = inspect_archive_entries(path)?;
    debug_assert!(total <= crunch_pnpm_core::MAX_INPUT_BYTES);
    debug_assert!(expanded <= crunch_pnpm_core::MAX_INPUT_BYTES);
    Ok(expanded)
}

/// Every archive entry must be a regular file or directory under the `package/` root.
fn inspect_archive_entries(path: &Path) -> Result<u64> {
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(std::fs::File::open(path)?));
    let mut manifest = None;
    let mut entry_count = 0_u32;
    let mut expanded = 0_u64;
    for entry in archive.entries()? {
        entry_count = entry_count.saturating_add(1);
        if entry_count > MAX_ARCHIVE_ENTRIES {
            return Err("archive entry bound exceeded".into());
        }
        let mut entry = entry?;
        let kind = entry.header().entry_type();
        if !kind.is_file() && !kind.is_dir() {
            return Err("archive symlinks, hardlinks and special files prohibited".into());
        }
        let path = entry.path()?.into_owned();
        let path = path.to_str().ok_or("archive non-UTF8 path")?.trim_end_matches('/');
        let path = crunch_pnpm_core::member(path.to_owned())?;
        if path != "package" && !path.starts_with("package/") {
            return Err("tarball must have package/ root".into());
        }
        expanded = expanded.checked_add(entry.size()).ok_or("archive size overflow")?;
        if expanded > crunch_pnpm_core::MAX_INPUT_BYTES {
            return Err("expanded archive bound exceeded".into());
        }
        if path == "package/package.json" {
            let mut data = Vec::new();
            entry.by_ref().take(1024 * 1024).read_to_end(&mut data)?;
            manifest = Some(serde_json::from_slice::<Value>(&data)?);
        }
    }
    check_manifest(&manifest.ok_or("archive missing package.json")?, &BTreeSet::new())?;
    debug_assert!(entry_count > 0);
    debug_assert!(expanded <= crunch_pnpm_core::MAX_INPUT_BYTES);
    Ok(expanded)
}
