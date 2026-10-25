// HARDENING-BACKLOG 2026-10-22: pre-existing tigerstyle findings in this file are
// recorded in .cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/
// tigerstyle-backlog-2026-10-22.log and scheduled for the standalone hardening
// pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::too_many_parameters)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use crunch_composition_core::user_profile::Manifest;
use crunch_composition_core::user_profile::Package;
use crunch_composition_core::user_profile::{self as core};
use crunch_store::OutputLookup;
use nix_compat::derivation::Derivation;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::narinfo::fingerprint_with_store_dir;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::StorePathRef;
use snix_store::pathinfoservice::PathInfo;

const CLOSURE_MEMBERS_MAX: usize = 65_536;
const NAR_BYTES_MAX: u64 = 17_592_186_044_416;

pub(super) struct Admission<'a> {
    pub lookup: &'a OutputLookup,
    pub content: &'a crunch_store::BuildServiceStore,
    pub output_dir: &'a Path,
    pub trusted_keys: &'a [VerifyingKey],
}

/// Construct an exact request from an already evaluated derivation and observed
/// output metadata. This is NOT admission; execute_profile rechecks everything.
pub fn package_from_derivation(
    name: String,
    derivation: &Derivation,
    derivation_path: &StorePath<String>,
    output_name: String,
    info: &PathInfo,
    store_prefix: &str,
) -> Result<Package, String> {
    let derivation_aterm = String::from_utf8(derivation.to_aterm_bytes()).map_err(|error| error.to_string())?;
    assert!(!store_prefix.is_empty());
    assert!(!name.is_empty());
    Ok(Package {
        name,
        derivation_path: derivation_path.to_absolute_path_with_prefix(store_prefix),
        derivation_aterm,
        output_name,
        output_path: info.store_path.to_absolute_path_with_prefix(store_prefix),
        nar_sha256: data_encoding::HEXLOWER.encode(&info.nar_sha256),
        nar_size: info.nar_size,
    })
}

pub(super) async fn admit(manifest: &Manifest, authority: &Admission<'_>) -> Result<BTreeMap<String, String>, String> {
    manifest.validate().map_err(|error| error.to_string())?;
    if authority.trusted_keys.is_empty() {
        return Err("profile admission requires explicit trusted public keys".into());
    }
    let mut verified = BTreeSet::new();
    let mut entries = Vec::new();
    for package in &manifest.packages {
        let path = parse_path(&package.output_path, StorePrefix(&manifest.store_prefix))?;
        let info = authority
            .lookup
            .find(&path)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("profile output is not admitted: {}", package.output_path))?;
        verify_package(package, manifest, &info, authority.lookup)?;
        verify_closure(path, &manifest.store_prefix, authority, &mut verified).await?;
        let physical = authority.output_dir.join(info.store_path.to_string());
        if !std::fs::symlink_metadata(&physical).map_err(|error| error.to_string())?.is_dir() {
            return Err(format!("profile package output is not a directory: {}", physical.display()));
        }
        collect_exposed(&physical, &mut entries)?;
    }
    assert!(verified.len() <= CLOSURE_MEMBERS_MAX);
    assert!(entries.len() <= core::ENTRIES_MAX);
    core::compose(entries).map_err(|error| error.to_string())
}

fn verify_package(
    package: &Package,
    manifest: &Manifest,
    info: &PathInfo,
    lookup: &OutputLookup,
) -> Result<(), String> {
    if package.nar_sha256 != data_encoding::HEXLOWER.encode(&info.nar_sha256) || package.nar_size != info.nar_size {
        return Err(format!("profile output NAR identity mismatch: {}", package.output_path));
    }
    let drv_path = parse_path(&package.derivation_path, StorePrefix(&manifest.store_prefix))?;
    let name = drv_path.name().strip_suffix(".drv").ok_or("profile derivation suffix missing")?;
    let derivation = parse_native_projection(package.derivation_aterm.as_bytes(), name)?;
    if derivation.to_aterm_bytes() != package.derivation_aterm.as_bytes() {
        return Err("profile derivation ATerm projection must be canonical".into());
    }
    if derivation.system != manifest.platform {
        return Err("profile derivation platform mismatch".into());
    }
    let calculated = derivation
        .calculate_derivation_path_with_store_dir(name, &manifest.store_prefix)
        .map_err(|error| error.to_string())?;
    if calculated != drv_path {
        return Err("profile derivation identity mismatch".into());
    }
    let output = derivation.outputs.get(&package.output_name).ok_or("profile derivation output name missing")?;
    let actual = match &output.path {
        Some(path) => path.to_absolute_path_with_prefix(&manifest.store_prefix),
        None => lookup
            .resolve_ca_mapping(&package.derivation_path, &package.output_name)
            .map_err(|error| error.to_string())?
            .ok_or("profile CA output has no admitted realization mapping")?,
    };
    if info.deriver.as_ref() != Some(&drv_path) {
        return Err("profile output producer does not match the exact derivation".into());
    }
    if actual != package.output_path {
        return Err("profile derivation/output identity mismatch".into());
    }
    assert_eq!(derivation.system, manifest.platform);
    assert_eq!(actual, package.output_path);
    Ok(())
}

async fn verify_closure(
    root: StorePath<String>,
    prefix: &str,
    authority: &Admission<'_>,
    verified: &mut BTreeSet<String>,
) -> Result<(), String> {
    let mut pending = vec![root];
    while let Some(path) = pending.pop() {
        let logical = path.to_absolute_path_with_prefix(prefix);
        if verified.contains(&logical) {
            continue;
        }
        if verified.len() >= CLOSURE_MEMBERS_MAX {
            return Err("profile closure exceeds member bound".into());
        }
        let info = authority
            .lookup
            .find(&path)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("profile closure member missing: {logical}"))?;
        verify_signature(&info, prefix, authority.trusted_keys)?;
        if !authority.content.has_complete_content(&info).await.map_err(|error| error.to_string())? {
            return Err(format!("profile admitted store content is incomplete: {logical}"));
        }
        let physical = authority.output_dir.join(path.to_string());
        let request = crunch_nar::FilesystemNarRequest::new(
            nix_compat::nixhash::HashAlgo::Sha256,
            crunch_nar::CaseHackPolicy::native(),
            NAR_BYTES_MAX,
        );
        let observation = crunch_nar::observe_path_blocking(physical.clone(), request)
            .await
            .map_err(|error| format!("profile output missing or unreadable {}: {error}", physical.display()))?;
        if observation.nar_size != info.nar_size || observation.digest.digest_as_bytes() != info.nar_sha256 {
            return Err(format!("profile exported output tampered: {logical}"));
        }
        if pending.len().saturating_add(info.references.len()) > CLOSURE_MEMBERS_MAX {
            return Err("profile closure pending references exceed bound".into());
        }
        pending.extend(info.references);
        verified.insert(logical);
    }
    assert!(pending.is_empty());
    assert!(verified.len() <= CLOSURE_MEMBERS_MAX);
    Ok(())
}

fn verify_signature(info: &PathInfo, prefix: &str, keys: &[VerifyingKey]) -> Result<(), String> {
    let path: StorePathRef<'_> = info.store_path.as_ref();
    let references: Vec<_> = info.references.iter().map(|reference| reference.as_ref()).collect();
    let fingerprint = fingerprint_with_store_dir(&path, &info.nar_sha256, info.nar_size, references.iter(), prefix);
    if !info
        .signatures
        .iter()
        .any(|signature| keys.iter().any(|key| key.verify(&fingerprint, &signature.as_ref())))
    {
        return Err(format!("profile output has no trusted signature: {}", info.store_path));
    }
    assert!(!keys.is_empty());
    assert!(!info.signatures.is_empty());
    Ok(())
}

fn collect_exposed(root: &Path, entries: &mut Vec<(String, Option<String>)>) -> Result<(), String> {
    let mut pending = Vec::<(PathBuf, String, u32)>::with_capacity(core::EXPOSED_DIRECTORIES.len());
    for directory in core::EXPOSED_DIRECTORIES {
        let path = root.join(directory);
        match std::fs::symlink_metadata(&path) {
            Ok(_) => pending.push((path, directory.into(), 1)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    while let Some((path, relative, depth)) = pending.pop() {
        if entries.len().saturating_add(pending.len()) >= core::ENTRIES_MAX || depth > 64 {
            return Err("profile output composition exceeds entry/depth bounds".into());
        }
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.is_dir() {
            entries.push((relative.clone(), None));
            for entry in std::fs::read_dir(&path).map_err(|error| error.to_string())? {
                let entry = entry.map_err(|error| error.to_string())?;
                let name = entry.file_name().into_string().map_err(|_| "profile output path is not UTF-8")?;
                if entries.len().saturating_add(pending.len()) >= core::ENTRIES_MAX {
                    return Err("profile composition exceeds entry bound".into());
                }
                pending.push((entry.path(), format!("{relative}/{name}"), depth.saturating_add(1)));
            }
        } else if metadata.is_file() || metadata.file_type().is_symlink() {
            let target = path.to_str().ok_or("profile physical output path is not UTF-8")?.to_string();
            entries.push((relative, Some(target)));
        } else {
            return Err("profile output contains an unsupported filesystem object".into());
        }
    }
    assert!(entries.len() <= core::ENTRIES_MAX);
    assert!(pending.is_empty());
    Ok(())
}

pub(super) fn parse_path(path: &str, prefix: StorePrefix<'_>) -> Result<StorePath<String>, String> {
    StorePath::from_absolute_path_with_prefix(path.as_bytes(), prefix.0).map_err(|error| error.to_string())
}

/// A store prefix carried as its own type, so a path and its prefix are
/// distinguished at every call site.
#[derive(Clone, Copy)]
pub(super) struct StorePrefix<'a>(pub &'a str);

/// The existing Snix parser requires concrete paths. Native CA derivations use
/// the same canonical traditional ATerm with absent paths; reuse the mature Nix
/// parser for that representational case, then validate the native projection.
fn parse_native_projection(bytes: &[u8], name: &str) -> Result<Derivation, String> {
    if let Ok(derivation) = Derivation::from_aterm_bytes(bytes) {
        return Ok(derivation);
    }
    let parsed = nix_derivation::Derivation::from_aterm_bytes(bytes, name)
        .map_err(|error| format!("profile derivation ATerm invalid: {error}"))?;
    let mut derivation = Derivation {
        system: parsed.system().into(),
        builder: parsed.builder().into(),
        arguments: parsed.arguments().to_vec(),
        environment: parsed.environment().iter().map(|(key, value)| (key.clone(), value.clone().into())).collect(),
        input_derivations: BTreeMap::new(),
        input_sources: BTreeSet::new(),
        outputs: BTreeMap::new(),
    };
    let mut has_deferred = false;
    for (output_name, output) in parsed.outputs() {
        let path = match output {
            nix_derivation::Output::InputAddressed { path } => {
                Some(parse_path(&path.to_string(), StorePrefix("/nix/store"))?)
            }
            nix_derivation::Output::Deferred => {
                has_deferred = true;
                None
            }
            _ => return Err("profile derivation cannot be projected to Mantle's native output model".into()),
        };
        derivation
            .outputs
            .insert(output_name.clone(), nix_compat::derivation::Output { path, ca_hash: None });
    }
    if !has_deferred {
        return Err("invalid native profile derivation projection".into());
    }
    for (path, input) in parsed.input_derivations() {
        if !input.dynamic_outputs().is_empty() {
            return Err("profile package requires the already resolved native producer".into());
        }
        derivation
            .input_derivations
            .insert(parse_path(&path.to_string(), StorePrefix("/nix/store"))?, input.outputs().clone());
    }
    for path in parsed.input_sources() {
        derivation.input_sources.insert(parse_path(&path.to_string(), StorePrefix("/nix/store"))?);
    }
    derivation.validate(false).map_err(|error| error.to_string())?;
    if derivation.to_aterm_bytes() != bytes {
        return Err("noncanonical native profile derivation projection".into());
    }
    assert!(has_deferred);
    assert!(!derivation.outputs.is_empty());
    Ok(derivation)
}
