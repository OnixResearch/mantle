//! Lock admission deliberately leaves dependency, peer and optional resolution to pinned pnpm.
use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use crunch_pnpm_core::Package;
use crunch_pnpm_core::Request;
use serde_json::Value;
use sha2::Digest;
use sha2::Sha512;

use crate::files::Result;
use crate::files::{self};

pub fn admit(request: &Request, source: &Path, mirror: &Path) -> Result<Value> {
    let lock: Value = serde_yaml::from_slice(&files::document(&source.join("pnpm-lock.yaml"))?)?;
    if lock["lockfileVersion"].as_str() != Some("9.0") {
        return Err("requires quoted lockfileVersion '9.0'".into());
    }
    let inventory = lock["packages"].as_object().ok_or("lock has no package inventory")?;
    if inventory.len() != request.packages.len() {
        return Err("package inventory differs from admitted mirror".into());
    }
    let root = files::json(&source.join("package.json"))?;
    if root["packageManager"] != "pnpm@9.15.9" {
        return Err("root packageManager must pin pnpm@9.15.9".into());
    }
    check_manifest(&root, &BTreeSet::new())?;
    inspect_source(source)?;
    let workspace: Value = serde_yaml::from_slice(&files::document(&source.join("pnpm-workspace.yaml"))?)?;
    if workspace.as_object().ok_or("workspace must be a record")?.keys().any(|k| k != "packages") {
        return Err("workspace configuration supports only explicit packages globs".into());
    }
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
    let mut expanded_bytes = 0_u64;
    for package in &request.packages {
        let entry = inventory.get(&package.lock_key).ok_or("missing locked package")?;
        if entry["requiresBuild"] == true {
            return Err(
                "dependency lifecycle/native builds are unsupported; provide prebuilt integrity archives".into()
            );
        }
        let resolution = entry["resolution"].as_object().ok_or("missing resolution")?;
        if resolution.keys().any(|k| k != "integrity" && k != "tarball")
            || entry["resolution"]["integrity"].as_str() != Some(package.integrity.as_str())
        {
            return Err("unsupported or drifting package resolution (integrity tarballs only)".into());
        }
        if let Some(url) = entry["resolution"]["tarball"].as_str()
            && (!url.starts_with("https://") || url.contains(['@', '?', '#']))
        {
            return Err("tarball locators must be credential-free HTTPS URLs".into());
        }
        if let Some(revision) = &package.git_revision {
            let url = entry["resolution"]["tarball"].as_str().ok_or("Git archives require explicit tarball URL")?;
            if !url.split('/').any(|segment| {
                segment == revision.as_str() || segment.strip_suffix(".tar.gz") == Some(revision.as_str())
            }) {
                return Err("Git archive URL must bind admitted revision as an exact path segment".into());
            }
        }
        expanded_bytes = expanded_bytes
            .checked_add(inspect_archive(package, &files::child(mirror, &package.archive)?)?)
            .ok_or("package expansion overflow")?;
        if expanded_bytes > crunch_pnpm_core::MAX_INPUT_BYTES {
            return Err("total package expansion bound exceeded".into());
        }
    }
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
    // Validate all importer manifests, not only selected targets. No unbounded install hooks.
    for (path, importer) in lock["importers"].as_object().ok_or("missing importers")? {
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
    }
    check_links(&lock, 0)?;
    Ok(lock)
}

fn inspect_source(path: &Path) -> Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let name = entry.file_name();
            if [
                "node_modules",
                ".pnpmfile.cjs",
                ".pnpmfile.js",
                ".npmrc",
                ".pnpm-store",
                ".git",
            ]
            .iter()
            .any(|v| name == *v)
            {
                return Err(format!("ambient configuration/cache/hook forbidden: {}", entry.path().display()).into());
            }
            inspect_source(&entry.path())?;
        }
    }
    Ok(())
}
fn check_manifest(manifest: &Value, allowed: &BTreeSet<&str>) -> Result<()> {
    for hook in [
        "preinstall",
        "install",
        "postinstall",
        "prepare",
        "prepublish",
        "prepublishOnly",
    ] {
        if manifest["scripts"].get(hook).is_some() && !allowed.contains(hook) {
            return Err(format!("prohibited lifecycle script: {hook}").into());
        }
    }
    if let Some(config) = manifest["pnpm"].as_object()
        && config.keys().any(|key| key != "patchedDependencies")
    {
        return Err("only patchedDependencies pnpm configuration is supported".into());
    }
    Ok(())
}
fn check_links(value: &Value, depth: u32) -> Result<()> {
    if depth > 64 {
        return Err("lock nesting bound exceeded".into());
    }
    match value {
        Value::String(s) if s.starts_with("file:") || s.starts_with("git+") || s.starts_with("git:") => {
            return Err("raw Git/file resolutions unsupported; use integrity-pinned archives or workspace links".into());
        }
        Value::String(s) if s.starts_with("link:") => {
            let member = &s[5..];
            // pnpm importer-relative ../sibling links are valid, but must remain in source.
            if member.starts_with('/') || member.contains('\\') {
                return Err("unsafe workspace link".into());
            }
        }
        Value::Object(object) => {
            for value in object.values() {
                check_links(value, depth + 1)?;
            }
        }
        Value::Array(array) => {
            for value in array {
                check_links(value, depth + 1)?;
            }
        }
        _ => (),
    }
    Ok(())
}
fn validate_links(value: &Value, importer: &Path, root: &Path, depth: u32) -> Result<()> {
    if depth > 64 {
        return Err("link nesting bound exceeded".into());
    }
    match value {
        Value::String(value) if value.starts_with("link:") => {
            let resolved = std::fs::canonicalize(importer.join(&value[5..]))?;
            if !resolved.starts_with(root) {
                return Err("workspace link escapes source".into());
            }
            files::json(&resolved.join("package.json"))?;
        }
        Value::Object(object) => {
            for value in object.values() {
                validate_links(value, importer, root, depth + 1)?;
            }
        }
        Value::Array(array) => {
            for value in array {
                validate_links(value, importer, root, depth + 1)?;
            }
        }
        _ => (),
    }
    Ok(())
}
fn inspect_archive(package: &Package, path: &Path) -> Result<u64> {
    let mut file = std::fs::File::open(path)?;
    let mut blake = blake3::Hasher::new();
    let mut sha = Sha512::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; 16 * 1024];
    loop {
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
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(std::fs::File::open(path)?));
    let mut manifest = None;
    let mut count = 0_u32;
    let mut expanded = 0_u64;
    for entry in archive.entries()? {
        count += 1;
        if count > 100_000 {
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
    Ok(expanded)
}
