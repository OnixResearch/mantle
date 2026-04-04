//! Convert `CrunchDerivation` → `nix_compat::Derivation` with BLAKE3 store paths.

use std::collections::{BTreeMap, BTreeSet};

use bstr::BString;
use nix_compat::derivation::{Derivation, Output};
use nix_compat::nixhash::{CAHash, HashAlgo, NixHash};
use nix_compat::store_path::StorePath;

use crate::error::Error;
use crate::known_paths::KnownPaths;
use crate::types::{CrunchDerivation, FixedOutput, Input};

/// Convert a `CrunchDerivation` into a `nix_compat::Derivation` with
/// computed BLAKE3 store paths.
///
/// Recursively converts all `Input::Derivation` entries, registering
/// them in `known_paths` for dedup and HDM lookup.
///
/// Returns `(drv_store_path, derivation)`.
pub fn convert(
    drv: &CrunchDerivation,
    known_paths: &mut KnownPaths,
) -> Result<(StorePath<String>, Derivation), Error> {
    let identity = derivation_identity(drv);

    // Cycle detection
    if !known_paths.begin_conversion(&identity) {
        return Err(Error::CircularDependency(drv.name.clone()));
    }

    let result = convert_inner(drv, known_paths);

    known_paths.end_conversion(&identity);
    result
}

fn convert_inner(
    drv: &CrunchDerivation,
    known_paths: &mut KnownPaths,
) -> Result<(StorePath<String>, Derivation), Error> {
    // 1. Process inputs: resolve sources and recursively convert derivation inputs
    let mut input_derivations: BTreeMap<StorePath<String>, BTreeSet<String>> = BTreeMap::new();
    let mut input_sources: BTreeSet<StorePath<String>> = BTreeSet::new();

    for input in &drv.inputs {
        match input {
            Input::Source(path_str) => {
                let store_path = parse_store_path(path_str)?;
                input_sources.insert(store_path);
            }
            Input::Derivation(nested_drv) => {
                let (nested_drv_path, _nested_nix_drv) = convert(nested_drv, known_paths)?;
                // Reference all outputs of the nested derivation
                let output_names: BTreeSet<String> =
                    nested_drv.outputs.iter().cloned().collect();
                input_derivations.insert(nested_drv_path, output_names);
            }
        }
    }

    // 2. Parse fixed-output if present
    let ca_hash = drv.fixed_output.as_ref().map(parse_fixed_output).transpose()?;

    // 3. Build the nix_compat::Derivation struct (without output paths yet)
    let mut outputs = BTreeMap::new();
    for output_name in &drv.outputs {
        outputs.insert(
            output_name.clone(),
            Output {
                path: None,
                ca_hash: if output_name == "out" { ca_hash.clone() } else { None },
            },
        );
    }

    // Environment: start with user-provided env, leave output path slots empty
    let mut environment: BTreeMap<String, BString> = BTreeMap::new();
    for (k, v) in &drv.env {
        environment.insert(k.clone(), v.as_bytes().into());
    }

    // Auto-populate standard env entries
    environment.insert("system".to_string(), drv.system.as_bytes().into());
    environment.insert("builder".to_string(), drv.builder.as_bytes().into());
    environment.insert("name".to_string(), drv.name.as_bytes().into());

    // Reserve empty slots for output paths (filled after path computation)
    for output_name in &drv.outputs {
        environment
            .entry(output_name.clone())
            .or_insert_with(|| "".into());
    }

    let mut nix_drv = Derivation {
        arguments: drv.args.clone(),
        builder: drv.builder.clone(),
        environment,
        input_derivations,
        input_sources,
        outputs,
        system: drv.system.clone(),
    };

    // 4. Compute hash_derivation_modulo
    let hdm = nix_drv.hash_derivation_modulo(|parent_drv_path| {
        known_paths
            .get_hdm_by_drv_path(&parent_drv_path.to_absolute_path())
            .unwrap_or_else(|| {
                panic!(
                    "BUG: parent derivation {} not in KnownPaths during HDM computation",
                    parent_drv_path
                )
            })
    });

    // 5. Compute output paths and fill them into the derivation
    nix_drv.calculate_output_paths(&drv.name, &hdm)?;

    // 6. Compute the .drv store path
    let drv_path = nix_drv.calculate_derivation_path(&drv.name)?;

    // 7. Compute ATerm hash for dedup key
    let aterm_bytes = nix_drv.to_aterm_bytes();
    let aterm_hash = *blake3::hash(&aterm_bytes).as_bytes();

    // 8. Register in KnownPaths
    known_paths.insert(aterm_hash, drv_path.clone(), hdm, nix_drv.clone());

    Ok((drv_path, nix_drv))
}

/// Parse a store path string into a `StorePath`.
fn parse_store_path(s: &str) -> Result<StorePath<String>, Error> {
    StorePath::from_absolute_path(s.as_bytes())
        .map_err(|_| Error::InvalidStorePath(s.to_string()))
}

/// Parse a `FixedOutput` into a `CAHash`.
fn parse_fixed_output(fo: &FixedOutput) -> Result<CAHash, Error> {
    let algo: HashAlgo = fo
        .algo
        .parse()
        .map_err(|_| Error::UnknownHashAlgo(fo.algo.clone()))?;

    // Try SRI format first ("sha256-..."), then hex
    let nix_hash = if fo.hash.contains('-') {
        NixHash::from_sri(&fo.hash).map_err(|e| Error::InvalidHash(format!("{}: {e}", fo.hash)))?
    } else {
        let digest_bytes = data_encoding::HEXLOWER
            .decode(fo.hash.as_bytes())
            .map_err(|e| Error::InvalidHash(format!("{}: {e}", fo.hash)))?;
        NixHash::from_algo_and_digest(algo, &digest_bytes)
            .map_err(|e| Error::InvalidHash(format!("{}: {e}", fo.hash)))?
    };

    match fo.mode.as_str() {
        "flat" => Ok(CAHash::Flat(nix_hash)),
        "recursive" => Ok(CAHash::Nar(nix_hash)),
        other => Err(Error::InvalidHash(format!("unknown hash mode: {other}"))),
    }
}

/// An opaque identity for cycle detection.
/// Two CrunchDerivations with the same identity are considered the same
/// derivation for cycle-detection purposes.
fn derivation_identity(drv: &CrunchDerivation) -> String {
    // Use name + builder + system as a simple identity.
    // The ATerm hash is the true identity but isn't available before conversion.
    format!("{}:{}:{}", drv.name, drv.builder, drv.system)
}
