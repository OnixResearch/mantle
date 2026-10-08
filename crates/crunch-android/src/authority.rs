//! Trust-bound same-run extraction admission. A signed PathInfo does not sign
//! `deriver`; the exact derivation edge is the fresh `BuildOutcome`, not that
//! unauthenticated metadata field. Cached outcomes are refused.

use std::collections::HashMap;
use std::path::Path;

use crunch_build::BuildOutcome;
use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use crunch_glue::Input;
use crunch_store::OutputLookup;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::nixhash::CAHash;
use nix_compat::nixhash::NixHash;
use nix_compat::store_path::StorePath;

use super::BoundRuntime;
use super::Error;
use super::ExtractedTrees;
use super::ExtractionDerivation;
use super::PreparedApk;
use super::store_path;

fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidInput(message.into())
}

fn conversion(
    extraction: &ExtractionDerivation,
    prefix: &str,
) -> Result<(StorePath<String>, nix_compat::derivation::Derivation), Error> {
    let derivation = CrunchDerivation {
        name: extraction.name.clone(),
        builder: extraction.builder.clone(),
        system: extraction.system.clone(),
        args: extraction.args.clone(),
        outputs: extraction.outputs.clone(),
        dynamic_plan_outputs: Vec::new(),
        env: extraction.env.clone().into_iter().collect::<HashMap<_, _>>(),
        inputs: extraction.inputs.iter().cloned().map(Input::Source).collect(),
        fixed_output: None,
        addressing_mode: extraction.addressing_mode.clone(),
        provenance: None,
    };
    let mut cache = ConversionCache::new(prefix);
    crunch_glue::convert(&derivation, &mut cache).map_err(|error| invalid(format!("extraction conversion: {error}")))
}

fn trusted(path_info: &snix_store::path_info::PathInfo, keys: &[VerifyingKey], prefix: &str) -> bool {
    !keys.is_empty() && crunch_build::verify_pathinfo_signatures_with_store_dir(path_info, keys, prefix).is_trusted()
}

fn hash_sha256(hex: &str, label: &str) -> Result<[u8; 32], Error> {
    if hex.len() != 64 {
        return Err(invalid(format!("{label} digest length")));
    }
    let mut digest = [0_u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)
            .map_err(|_| invalid(format!("{label} digest encoding")))?;
    }
    Ok(digest)
}

fn flat_sha256(hex: &str) -> Result<CAHash, Error> {
    Ok(CAHash::Flat(NixHash::Sha256(hash_sha256(hex, "FOD")?)))
}

async fn verify_one(
    extraction: &ExtractionDerivation,
    outcome: &BuildOutcome,
    lookup: &OutputLookup,
    trusted_keys: &[VerifyingKey],
    prefix: &Path,
) -> Result<String, Error> {
    let logical = prefix.to_str().ok_or_else(|| invalid("non-UTF8 store prefix"))?;
    if outcome.cached {
        return Err(invalid("cached extraction has no authenticated same-run recipe edge"));
    }
    let (expected_drv, converted) = conversion(extraction, logical)?;
    if outcome.drv_path != expected_drv {
        return Err(invalid("extraction derivation identity drift"));
    }
    let archive = StorePath::<String>::from_absolute_path_with_prefix(extraction.inputs[1].as_bytes(), logical)
        .map_err(|_| invalid("unbound FOD archive input"))?;
    let utility = StorePath::<String>::from_absolute_path_with_prefix(extraction.inputs[0].as_bytes(), logical)
        .map_err(|_| invalid("unbound extraction utility input"))?;
    if converted.input_sources.len() != 2
        || !converted.input_sources.contains(&archive)
        || !converted.input_sources.contains(&utility)
    {
        return Err(invalid("extraction input identity drift"));
    }
    let archive_info = lookup
        .find(&archive)
        .await
        .map_err(|error| invalid(error.to_string()))?
        .ok_or_else(|| invalid("FOD archive PathInfo missing"))?;
    if archive_info.ca != Some(flat_sha256(&extraction.archive_sha256)?)
        || !trusted(&archive_info, trusted_keys, logical)
    {
        return Err(invalid("FOD archive identity or trust mismatch"));
    }
    let utility_info = lookup
        .find(&utility)
        .await
        .map_err(|error| invalid(error.to_string()))?
        .ok_or_else(|| invalid("extraction utility PathInfo missing"))?;
    if !trusted(&utility_info, trusted_keys, logical) {
        return Err(invalid("untrusted extraction utility"));
    }
    let fresh = outcome.outputs.get("out").ok_or_else(|| invalid("same-run extraction output absent"))?;
    let stored = lookup
        .find(&fresh.store_path)
        .await
        .map_err(|error| invalid(error.to_string()))?
        .ok_or_else(|| invalid("extraction output PathInfo missing"))?;
    if stored.store_path != fresh.store_path
        || stored.node != fresh.node
        || stored.nar_sha256 != fresh.nar_sha256
        || stored.nar_size != fresh.nar_size
        || stored.references != fresh.references
        || stored.ca != fresh.ca
        || !trusted(&stored, trusted_keys, logical)
    {
        return Err(invalid("same-run output differs from authenticated store content"));
    }
    let tree_path = prefix.join(stored.store_path.to_string());
    let text = store_path(&tree_path, prefix)?;
    let meta = std::fs::symlink_metadata(&tree_path)?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err(invalid("extraction output is absent or redirected"));
    }
    let mut unpack_root = tree_path;
    for component in extraction.unpack_root.split('/').filter(|component| !component.is_empty()) {
        unpack_root.push(component);
        let metadata = std::fs::symlink_metadata(&unpack_root)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(invalid("extraction root is missing or redirected"));
        }
    }
    Ok(text)
}

async fn verify_runtime(
    runtime: &BoundRuntime,
    lookup: &OutputLookup,
    trusted_keys: &[VerifyingKey],
    prefix: &Path,
) -> Result<(), Error> {
    let logical = prefix.to_str().ok_or_else(|| invalid("non-UTF8 store prefix"))?;
    let path = store_path(&runtime.store_path, prefix)?;
    let key = StorePath::<String>::from_absolute_path_with_prefix(path.as_bytes(), logical)
        .map_err(|_| invalid("unbound runtime store root"))?;
    let info = lookup
        .find(&key)
        .await
        .map_err(|error| invalid(error.to_string()))?
        .ok_or_else(|| invalid("runtime PathInfo missing"))?;
    if info.store_path != key
        || info.nar_sha256 != hash_sha256(&runtime.identity.nar_sha256, "runtime")?
        || info.nar_size != runtime.identity.nar_size
        || !trusted(&info, trusted_keys, logical)
    {
        return Err(invalid("runtime content identity or trust mismatch"));
    }
    let metadata = std::fs::symlink_metadata(&runtime.store_path)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(invalid("runtime tree is absent or redirected"));
    }
    Ok(())
}

/// Admit only *fresh outcomes returned by the same invocation of the real
/// builder*. `trusted_keys` are independently configured store trust keys.
/// A persisted PathInfo (including its unsigned `deriver`) alone is not proof
/// of execution. Archive/utility PathInfo and the returned output are checked
/// against selected store state and its signatures before any SDK stage.
/// This first boundary uses one logical/physical store prefix.
pub async fn verified_same_run_extractions(
    prepared: &PreparedApk,
    outcomes: &[BuildOutcome; 3],
    lookup: &OutputLookup,
    trusted_keys: &[VerifyingKey],
    store_prefix: &Path,
) -> Result<ExtractedTrees, Error> {
    let [build, jdk, platform] = prepared.extractions(store_prefix)?;
    verify_runtime(&prepared.inputs.runtime_glibc, lookup, trusted_keys, store_prefix).await?;
    verify_runtime(&prepared.inputs.runtime_libgcc, lookup, trusted_keys, store_prefix).await?;
    Ok(ExtractedTrees {
        build: verify_one(&build, &outcomes[0], lookup, trusted_keys, store_prefix).await?,
        jdk: verify_one(&jdk, &outcomes[1], lookup, trusted_keys, store_prefix).await?,
        platform: verify_one(&platform, &outcomes[2], lookup, trusted_keys, store_prefix).await?,
    })
}
