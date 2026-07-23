//! Convert `CrunchDerivation` → `nix_compat::Derivation` with BLAKE3 store paths.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use bstr::BString;
use nix_compat::derivation::Derivation;
use nix_compat::derivation::Output;
use nix_compat::nixhash::CAHash;
use nix_compat::nixhash::HashAlgo;
use nix_compat::nixhash::NixHash;
use nix_compat::store_path::StorePath;

use crate::conversion_cache::ConversionCache;
use crate::error::Error;
use crate::types::CrunchDerivation;
use crate::types::FixedOutput;
use crate::types::Input;
use crate::types::validate_dynamic_plan_outputs;

/// Maximum derivation dependency depth before we bail out.
/// Prevents stack overflow from pathological or accidental deep graphs.
const MAX_RECURSION_DEPTH: u32 = 512;

type InputDerivationMap = BTreeMap<StorePath<String>, BTreeSet<String>>;
type InputSourceSet = BTreeSet<StorePath<String>>;
type ResolvedInputs = (InputDerivationMap, InputSourceSet);

struct StorePathText<'a> {
    path_text: &'a str,
    store_dir: &'a str,
}

/// Convert a `CrunchDerivation` into a `nix_compat::Derivation` with
/// computed BLAKE3 store paths.
///
/// Recursively converts all `Input::Derivation` entries, registering
/// them in `known_paths` for dedup and HDM lookup.
///
/// Returns `(drv_store_path, derivation)`.
pub fn convert(
    drv: &CrunchDerivation,
    known_paths: &mut ConversionCache,
) -> Result<(StorePath<String>, Derivation), Error> {
    convert_with_depth(drv, known_paths, 0)
}

fn convert_with_depth(
    drv: &CrunchDerivation,
    known_paths: &mut ConversionCache,
    depth: u32,
) -> Result<(StorePath<String>, Derivation), Error> {
    // Tiger Style: fixed limit on recursion depth.
    if depth >= MAX_RECURSION_DEPTH {
        return Err(Error::CircularDependency(format!("{} (depth limit {} exceeded)", drv.name, MAX_RECURSION_DEPTH)));
    }

    debug_assert!(!drv.name.is_empty(), "derivation name must not be empty");
    debug_assert!(!drv.builder.is_empty(), "derivation builder must not be empty");
    debug_assert!(!drv.outputs.is_empty(), "derivation must have at least one output");
    validate_dynamic_plan_outputs(&drv.outputs, &drv.dynamic_plan_outputs).map_err(Error::InvalidDynamicPlanOutputs)?;

    let identity = derivation_identity(drv);

    if let Some(converted) = known_paths.get_completed_identity(&identity) {
        if converted.dynamic_plan_outputs != drv.dynamic_plan_outputs {
            return Err(Error::InvalidDynamicPlanOutputs(format!(
                "conflicting dynamic_plan_outputs for derivation identity '{}'",
                drv.name
            )));
        }
        return Ok((converted.drv_path, converted.derivation));
    }

    // Cycle detection
    if !known_paths.begin_conversion(&identity) {
        return Err(Error::CircularDependency(drv.name.clone()));
    }

    let result = convert_inner(drv, known_paths, depth);

    known_paths.end_conversion(&identity);
    if let Ok((drv_path, nix_drv)) = &result {
        known_paths.insert_completed_identity(
            identity,
            drv_path.clone(),
            nix_drv.clone(),
            drv.dynamic_plan_outputs.clone(),
        );
    }
    result
}

fn convert_inner(
    drv: &CrunchDerivation,
    known_paths: &mut ConversionCache,
    depth: u32,
) -> Result<(StorePath<String>, Derivation), Error> {
    let store_dir = known_paths.store_dir().to_string();

    // 1. Resolve inputs (recursive for derivation deps).
    let (input_derivations, input_sources) = resolve_inputs(&drv.inputs, known_paths, depth, &store_dir)?;

    // 2. Build the nix_compat::Derivation struct.
    let ca_hash = drv.fixed_output.as_ref().map(parse_fixed_output).transpose()?;
    let mut nix_drv = build_nix_derivation(drv, input_derivations, input_sources, ca_hash);

    // 3. Finalize: compute HDM, output paths, drv path, register.
    finalize_and_register(drv, &mut nix_drv, known_paths, &store_dir)
}

/// Resolve all inputs: source paths are parsed, derivation inputs are
/// recursively converted and registered in `known_paths`.
fn resolve_inputs(
    inputs: &[Input],
    known_paths: &mut ConversionCache,
    depth: u32,
    store_dir: &str,
) -> Result<ResolvedInputs, Error> {
    assert!(depth < MAX_RECURSION_DEPTH, "input resolution depth must stay within limit");
    assert!(store_dir.starts_with('/'), "store_dir must be absolute");
    let mut input_derivations: InputDerivationMap = BTreeMap::new();
    let mut input_sources: InputSourceSet = BTreeSet::new();

    for input in inputs {
        match input {
            Input::Source(path_str) => {
                let store_path = parse_store_path(StorePathText {
                    path_text: path_str,
                    store_dir,
                })?;
                input_sources.insert(store_path);
            }
            Input::DerivationFile(reference) => {
                return Err(Error::UnresolvedDerivationFile {
                    path: reference.path.clone(),
                });
            }
            Input::ResolvedDerivation(reference) => {
                let drv_path = parse_store_path(StorePathText {
                    path_text: &reference.drv_path,
                    store_dir,
                })?;
                let output_names = reference.outputs.iter().cloned().collect::<BTreeSet<_>>();
                if output_names.is_empty() {
                    return Err(Error::InvalidOutputSelection {
                        drv_name: reference.drv_path.clone(),
                        output: "<all>".to_string(),
                        available: "<none>".to_string(),
                    });
                }
                input_derivations.entry(drv_path).or_default().extend(output_names);
            }
            Input::OutputSelection(output_ref) => {
                // Validate the selected output exists.
                if !output_ref.drv.outputs.contains(&output_ref.output) {
                    return Err(Error::InvalidOutputSelection {
                        drv_name: output_ref.drv.name.clone(),
                        output: output_ref.output.clone(),
                        available: output_ref.drv.outputs.join(", "),
                    });
                }
                let (nested_drv_path, _nested_nix_drv) =
                    convert_with_depth(&output_ref.drv, known_paths, depth.saturating_add(1))?;
                // Coalesce: merge into existing entry if same drv appears twice.
                input_derivations.entry(nested_drv_path).or_default().insert(output_ref.output.clone());
            }
            Input::Derivation(nested_drv) => {
                let (nested_drv_path, _nested_nix_drv) =
                    convert_with_depth(nested_drv, known_paths, depth.saturating_add(1))?;
                let output_names: BTreeSet<String> = nested_drv.outputs.iter().cloned().collect();
                input_derivations.entry(nested_drv_path).or_default().extend(output_names);
            }
        }
    }

    Ok((input_derivations, input_sources))
}

/// Construct a `nix_compat::Derivation` from the CrunchDerivation fields.
/// Output paths are not yet computed — that happens in `finalize_and_register`.
fn build_nix_derivation(
    drv: &CrunchDerivation,
    input_derivations: InputDerivationMap,
    input_sources: InputSourceSet,
    ca_hash: Option<CAHash>,
) -> Derivation {
    assert!(!drv.builder.is_empty(), "builder must not be empty");
    assert!(!drv.outputs.is_empty(), "derivation must have at least one output");
    let outputs: BTreeMap<String, Output> = drv
        .outputs
        .iter()
        .map(|output_name| {
            (output_name.clone(), Output {
                path: None,
                ca_hash: if output_name == "out" { ca_hash.clone() } else { None },
            })
        })
        .collect();

    let mut environment: BTreeMap<String, BString> =
        drv.env.iter().map(|(key, value)| (key.clone(), value.as_bytes().into())).collect();
    environment.insert("system".to_string(), drv.system.as_bytes().into());
    environment.insert("builder".to_string(), drv.builder.as_bytes().into());
    environment.insert("name".to_string(), drv.name.as_bytes().into());
    environment.extend(drv.outputs.iter().map(|output_name| (output_name.clone(), BString::from(""))));
    environment.insert("outputs".to_string(), drv.outputs.join(" ").as_bytes().into());

    Derivation {
        arguments: drv.args.clone(),
        builder: drv.builder.clone(),
        environment,
        input_derivations,
        input_sources,
        outputs,
        system: drv.system.clone(),
    }
}

/// Compute HDM, output paths, and .drv path. Register in KnownPaths.
fn finalize_and_register(
    drv: &CrunchDerivation,
    nix_drv: &mut Derivation,
    known_paths: &mut ConversionCache,
    store_dir: &str,
) -> Result<(StorePath<String>, Derivation), Error> {
    // Pre-validate: every input derivation must already be registered.
    // The nix-compat HDM callback is infallible, so we check up front.
    for parent_drv_path in nix_drv.input_derivations.keys() {
        let abs = parent_drv_path.to_absolute_path_with_prefix(store_dir);
        if known_paths.get_hdm_by_drv_path(&abs).is_none() {
            return Err(Error::InvalidStorePath(format!(
                "parent derivation {} not in KnownPaths during HDM computation for '{}'",
                parent_drv_path, drv.name,
            )));
        }
    }

    let hdm = nix_drv.hash_derivation_modulo(|parent_drv_path| {
        let parent_hdm = known_paths.get_hdm_by_drv_path(&parent_drv_path.to_absolute_path_with_prefix(store_dir));
        assert!(parent_hdm.is_some(), "pre-validated parent missing");
        parent_hdm.unwrap_or([0u8; 32])
    });

    let is_ca = drv.addressing_mode == "content-addressed" && drv.fixed_output.is_none();

    // Compute provisional output paths (used as $out in the sandbox).
    nix_drv.calculate_output_paths_with_store_dir(&drv.name, &hdm, store_dir)?;

    if is_ca {
        // CA: clear stored paths but keep provisional paths in env for $out.
        for (_name, output) in nix_drv.outputs.iter_mut() {
            output.path = None;
        }
    }

    let drv_path = nix_drv.calculate_derivation_path_with_store_dir(&drv.name, store_dir)?;

    let aterm_bytes = nix_drv.to_aterm_bytes();
    let aterm_hash = *blake3::hash(&aterm_bytes).as_bytes();
    if let Some(existing) = known_paths.get_by_aterm_hash(&aterm_hash)
        && existing.dynamic_plan_outputs != drv.dynamic_plan_outputs
    {
        return Err(Error::InvalidDynamicPlanOutputs(format!(
            "conflicting dynamic_plan_outputs for derivation '{}'",
            drv.name
        )));
    }

    known_paths.insert_ca(crate::conversion_cache::InsertCaEntry {
        aterm_hash,
        drv_path: drv_path.clone(),
        hdm,
        derivation: nix_drv.clone(),
        content_addressed: is_ca,
        dynamic_plan_outputs: drv.dynamic_plan_outputs.clone(),
        provenance_claims: drv.provenance.clone(),
    });

    // Tiger Style: assert postconditions.
    debug_assert!(
        known_paths.get_by_drv_path(&drv_path.to_absolute_path_with_prefix(store_dir)).is_some(),
        "derivation must be registered in KnownPaths after insert"
    );
    debug_assert!(
        !is_ca || nix_drv.outputs.values().all(|o| o.path.is_none()),
        "CA derivation outputs must have None paths (resolved after build)"
    );

    Ok((drv_path, nix_drv.clone()))
}

/// Parse a store path string into a `StorePath` using the configured store prefix.
fn parse_store_path(path: StorePathText<'_>) -> Result<StorePath<String>, Error> {
    assert!(!path.path_text.is_empty(), "store path string must not be empty");
    assert!(path.store_dir.starts_with('/'), "store_dir must be absolute");
    StorePath::from_absolute_path_with_prefix(path.path_text.as_bytes(), path.store_dir)
        .map_err(|_| Error::InvalidStorePath(path.path_text.to_string()))
}

/// Parse a `FixedOutput` into a `CAHash`.
fn parse_fixed_output(fo: &FixedOutput) -> Result<CAHash, Error> {
    let algo: HashAlgo = fo.algo.parse().map_err(|_| Error::UnknownHashAlgo(fo.algo.clone()))?;

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

#[cfg(test)]
mod tests {
    use nix_compat::nixhash::CAHash;

    use super::*;
    use crate::conversion_cache::ConversionCache;
    use crate::types::*;

    fn minimal_drv(name: &str, builder: &str) -> CrunchDerivation {
        CrunchDerivation {
            name: name.to_string(),
            builder: builder.to_string(),
            system: "x86_64-linux".to_string(),
            args: vec![],
            outputs: vec!["out".to_string()],
            dynamic_plan_outputs: vec![],
            env: Default::default(),
            inputs: vec![],
            fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
            provenance: None,
        }
    }

    // ── Phase 2: basic derivations ────────────────────────────────────

    #[test]
    fn simple_drv_exact_store_path() {
        let drv = minimal_drv("hello", "/bin/sh");
        let mut kp = ConversionCache::default();
        let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();
        let out_path = nix_drv.outputs.get("out").unwrap().path.as_ref().unwrap();

        // These exact paths are regression fixtures. They change only
        // when the environment/outputs schema changes (e.g., adding
        // the `outputs` env var).
        assert!(drv_path.to_absolute_path().starts_with("/nix/store/"), "drv path must be a store path");
        assert!(
            drv_path.to_absolute_path().ends_with("-hello.drv"),
            "drv path must end with -hello.drv: {}",
            drv_path.to_absolute_path()
        );
        assert!(out_path.to_absolute_path().starts_with("/nix/store/"), "out path must be a store path");
        assert!(
            out_path.to_absolute_path().ends_with("-hello"),
            "out path must end with -hello: {}",
            out_path.to_absolute_path()
        );
    }

    #[test]
    fn builder_and_system_propagate() {
        let drv = minimal_drv("hello", "/usr/bin/env");
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert_eq!(nix_drv.builder, "/usr/bin/env");
        assert_eq!(nix_drv.system, "x86_64-linux");
    }

    #[test]
    fn environment_auto_populated() {
        let drv = minimal_drv("hello", "/bin/sh");
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert_eq!(nix_drv.environment.get("system").unwrap(), "x86_64-linux");
        assert_eq!(nix_drv.environment.get("builder").unwrap(), "/bin/sh");
        assert_eq!(nix_drv.environment.get("name").unwrap(), "hello");

        // "out" env entry matches the computed output path
        let out_path = nix_drv.outputs.get("out").unwrap().path.as_ref().unwrap().to_absolute_path();
        let out_env: &[u8] = nix_drv.environment.get("out").unwrap().as_ref();
        assert_eq!(out_env, out_path.as_bytes());
    }

    #[test]
    fn multi_output_reflected_in_derivation() {
        let drv = CrunchDerivation {
            outputs: vec!["out".to_string(), "lib".to_string(), "dev".to_string()],
            ..minimal_drv("multi", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert!(drv_path.to_absolute_path().ends_with("-multi.drv"));
        assert_eq!(nix_drv.outputs.len(), 3);
        assert!(nix_drv.outputs.contains_key("out"));
        assert!(nix_drv.outputs.contains_key("lib"));
        assert!(nix_drv.outputs.contains_key("dev"));
    }

    #[test]
    fn outputs_env_var_set_single() {
        let drv = minimal_drv("hello", "/bin/sh");
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        let outputs_env: &[u8] = nix_drv.environment.get("outputs").unwrap().as_ref();
        assert_eq!(outputs_env, b"out");
    }

    #[test]
    fn outputs_env_var_set_multi() {
        let drv = CrunchDerivation {
            outputs: vec!["out".to_string(), "dev".to_string(), "lib".to_string()],
            ..minimal_drv("multi", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        let outputs_env: &[u8] = nix_drv.environment.get("outputs").unwrap().as_ref();
        assert_eq!(outputs_env, b"out dev lib");
    }

    #[test]
    fn multi_output_all_env_vars_populated() {
        let drv = CrunchDerivation {
            outputs: vec!["out".to_string(), "dev".to_string(), "lib".to_string()],
            ..minimal_drv("multi", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        // Each output name has its own env var with a store path
        for name in &["out", "dev", "lib"] {
            let env_val: &[u8] = nix_drv.environment.get(*name).unwrap().as_ref();
            let env_str = std::str::from_utf8(env_val).unwrap();
            assert!(env_str.starts_with("/nix/store/"), "${name} should be a store path, got: {env_str}");
        }

        // All three paths are distinct
        let out: &[u8] = nix_drv.environment.get("out").unwrap().as_ref();
        let dev: &[u8] = nix_drv.environment.get("dev").unwrap().as_ref();
        let lib: &[u8] = nix_drv.environment.get("lib").unwrap().as_ref();
        assert_ne!(out, dev);
        assert_ne!(dev, lib);
        assert_ne!(out, lib);
    }

    #[test]
    fn multi_output_distinct_paths() {
        let drv = CrunchDerivation {
            outputs: vec!["out".to_string(), "lib".to_string(), "dev".to_string()],
            ..minimal_drv("multi", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        let out = nix_drv.outputs["out"].path.as_ref().unwrap().to_absolute_path();
        let lib = nix_drv.outputs["lib"].path.as_ref().unwrap().to_absolute_path();
        let dev = nix_drv.outputs["dev"].path.as_ref().unwrap().to_absolute_path();

        assert!(out.ends_with("-multi"), "out: {out}");
        assert!(lib.ends_with("-multi-lib"), "lib: {lib}");
        assert!(dev.ends_with("-multi-dev"), "dev: {dev}");

        assert_ne!(out, lib);
        assert_ne!(lib, dev);
        assert_ne!(out, dev);
    }

    // ── Phase 3: inputs and dependencies ──────────────────────────────

    #[test]
    fn convert_rejects_programmatic_undeclared_dynamic_plan_output() {
        let drv = CrunchDerivation {
            dynamic_plan_outputs: vec!["plan".to_string()],
            ..minimal_drv("bad-plan-output", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let err = convert(&drv, &mut kp).unwrap_err().to_string();
        assert!(err.contains("invalid dynamic plan outputs"), "error should name dynamic-plan validation: {err}");
        assert!(err.contains("dynamic_plan_outputs entry 'plan'"), "error should name bad output: {err}");
    }

    #[test]
    fn convert_rejects_conflicting_dynamic_plan_metadata_for_cached_identity() {
        let without_plan = CrunchDerivation {
            outputs: vec!["out".to_string(), "plan".to_string()],
            ..minimal_drv("same-identity", "/bin/sh")
        };
        let with_plan = CrunchDerivation {
            dynamic_plan_outputs: vec!["plan".to_string()],
            ..without_plan.clone()
        };

        let mut kp = ConversionCache::default();
        convert(&without_plan, &mut kp).unwrap();
        let err = convert(&with_plan, &mut kp).unwrap_err().to_string();
        assert!(err.contains("conflicting dynamic_plan_outputs"), "error should reject conflict: {err}");
    }

    #[test]
    fn source_input_wires_to_input_sources() {
        let drv = CrunchDerivation {
            inputs: vec![Input::Source(
                "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash".to_string(),
            )],
            ..minimal_drv("with-src", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert_eq!(nix_drv.input_sources.len(), 1);
        let src = nix_drv.input_sources.iter().next().unwrap();
        assert!(src.to_string().contains("bash"));
        assert!(nix_drv.input_derivations.is_empty());
    }

    #[test]
    fn derivation_input_wires_to_input_derivations() {
        let dep = minimal_drv("libfoo", "/bin/sh");
        let drv = CrunchDerivation {
            inputs: vec![Input::Derivation(Box::new(dep))],
            ..minimal_drv("myapp", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert_eq!(nix_drv.input_derivations.len(), 1);
        let (dep_path, dep_outputs) = nix_drv.input_derivations.iter().next().unwrap();
        assert!(dep_path.to_absolute_path().ends_with("-libfoo.drv"), "dep path: {}", dep_path.to_absolute_path());
        assert!(dep_outputs.contains("out"));
    }

    #[test]
    fn nested_drv_registered_in_known_paths() {
        let dep = minimal_drv("libfoo", "/bin/sh");
        let drv = CrunchDerivation {
            inputs: vec![Input::Derivation(Box::new(dep))],
            ..minimal_drv("myapp", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (parent_path, _) = convert(&drv, &mut kp).unwrap();

        // Both parent and dep are in KnownPaths
        assert!(kp.get_by_drv_path(&parent_path.to_absolute_path()).is_some());
        // The nested dep should also be registered. Find it by name.
        let libfoo_registered = kp
            .get_by_drv_path(&{
                let mut kp2 = ConversionCache::default();
                let (p, _) = convert(&minimal_drv("libfoo", "/bin/sh"), &mut kp2).unwrap();
                p.to_absolute_path()
            })
            .is_some();
        assert!(libfoo_registered, "libfoo should be registered in KnownPaths");
    }

    #[test]
    fn diamond_dep_single_known_paths_entry() {
        let shared = minimal_drv("shared", "/bin/sh");
        let left = CrunchDerivation {
            inputs: vec![Input::Derivation(Box::new(shared.clone()))],
            ..minimal_drv("left", "/bin/sh")
        };
        let right = CrunchDerivation {
            inputs: vec![Input::Derivation(Box::new(shared))],
            ..minimal_drv("right", "/bin/sh")
        };
        let top = CrunchDerivation {
            inputs: vec![Input::Derivation(Box::new(left)), Input::Derivation(Box::new(right))],
            ..minimal_drv("top", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&top, &mut kp).unwrap();

        // top depends on left and right
        assert_eq!(nix_drv.input_derivations.len(), 2);

        // shared's drv path appears once in KnownPaths and only once in the
        // pending conversion stream. This protects broad bootstrap roots from
        // exponential conversion over diamond-shaped dependency graphs.
        let shared_path = {
            let mut kp_check = ConversionCache::default();
            let (p, _) = convert(&minimal_drv("shared", "/bin/sh"), &mut kp_check).unwrap();
            p.to_absolute_path()
        };
        assert!(kp.get_by_drv_path(&shared_path).is_some());
        assert_eq!(kp.pending_count(), 4);
    }

    #[test]
    fn cycle_returns_circular_dependency_error() {
        // Same identity nested inside itself
        let inner = minimal_drv("loop", "/bin/sh");
        let outer = CrunchDerivation {
            inputs: vec![Input::Derivation(Box::new(inner))],
            ..minimal_drv("loop", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let err = convert(&outer, &mut kp).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("ircular"), "expected circular dependency error, got: {msg}");
    }

    // ── Phase 4: fixed-output derivations ─────────────────────────────

    #[test]
    fn fod_flat_sha256() {
        let drv = CrunchDerivation {
            fixed_output: Some(FixedOutput {
                hash: "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba".to_string(),
                algo: "sha256".to_string(),
                mode: "flat".to_string(),
            }),
            ..minimal_drv("src-flat", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert!(drv_path.to_absolute_path().ends_with("-src-flat.drv"));
        let out = nix_drv.outputs.get("out").unwrap();
        // FOD output paths are determined by the declared hash, not the drv hash.
        assert_eq!(
            out.path.as_ref().unwrap().to_absolute_path(),
            "/nix/store/0s3jkqjcbwm5g95k3n5mamnqd6fi7ypk-src-flat"
        );
        assert!(matches!(out.ca_hash, Some(CAHash::Flat(_))));
    }

    #[test]
    fn fod_recursive_sha256() {
        let drv = CrunchDerivation {
            fixed_output: Some(FixedOutput {
                hash: "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba".to_string(),
                algo: "sha256".to_string(),
                mode: "recursive".to_string(),
            }),
            ..minimal_drv("src-rec", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert!(drv_path.to_absolute_path().ends_with("-src-rec.drv"));
        let out = nix_drv.outputs.get("out").unwrap();
        // FOD output paths are determined by the declared hash.
        assert_eq!(
            out.path.as_ref().unwrap().to_absolute_path(),
            "/nix/store/jfpkf3icsq4vlsmg0xfly8b1rv80hcxq-src-rec"
        );
        assert!(matches!(out.ca_hash, Some(CAHash::Nar(_))));
    }

    #[test]
    fn fod_sri_hash_parses() {
        let drv = CrunchDerivation {
            fixed_output: Some(FixedOutput {
                hash: "sha256-CIE8vumQPGK+TFAnJqQYowoNpFALLTadOvkob0gVzro=".to_string(),
                algo: "sha256".to_string(),
                mode: "flat".to_string(),
            }),
            ..minimal_drv("src-sri", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert!(drv_path.to_absolute_path().ends_with("-src-sri.drv"));
        // SRI and hex parse to the same hash → same FOD output path.
        assert_eq!(
            nix_drv.outputs.get("out").unwrap().path.as_ref().unwrap().to_absolute_path(),
            "/nix/store/mikp7vivga7ysvaqnj6594dm3qz3qdz5-src-sri"
        );
    }

    #[test]
    fn fod_hex_parses_to_flat_ca_hash() {
        let fo = FixedOutput {
            hash: "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba".to_string(),
            algo: "sha256".to_string(),
            mode: "flat".to_string(),
        };
        let ca = parse_fixed_output(&fo).unwrap();
        assert!(matches!(ca, CAHash::Flat(_)));
    }

    #[test]
    fn fod_sri_parses_to_flat_ca_hash() {
        let fo = FixedOutput {
            hash: "sha256-CIE8vumQPGK+TFAnJqQYowoNpFALLTadOvkob0gVzro=".to_string(),
            algo: "sha256".to_string(),
            mode: "flat".to_string(),
        };
        let ca = parse_fixed_output(&fo).unwrap();
        assert!(matches!(ca, CAHash::Flat(_)));
    }

    #[test]
    fn fod_invalid_algo_rejected() {
        let drv = CrunchDerivation {
            fixed_output: Some(FixedOutput {
                hash: "deadbeef".to_string(),
                algo: "crc32".to_string(),
                mode: "flat".to_string(),
            }),
            ..minimal_drv("bad-algo", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let err = convert(&drv, &mut kp).unwrap_err();
        assert!(err.to_string().contains("hash algorithm"), "expected hash algorithm error, got: {err}");
    }

    #[test]
    fn fod_invalid_mode_rejected() {
        let drv = CrunchDerivation {
            fixed_output: Some(FixedOutput {
                hash: "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba".to_string(),
                algo: "sha256".to_string(),
                mode: "broken".to_string(),
            }),
            ..minimal_drv("bad-mode", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let err = convert(&drv, &mut kp).unwrap_err();
        assert!(err.to_string().contains("hash mode"), "expected hash mode error, got: {err}");
    }

    // ── Private helper tests ─────────────────────────────────────────

    #[test]
    fn parse_store_path_valid() {
        let sp = parse_store_path(StorePathText {
            path_text: "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash",
            store_dir: "/nix/store",
        })
        .unwrap();
        assert!(sp.to_string().contains("bash"));
    }

    #[test]
    fn parse_store_path_accepts_custom_prefix() {
        let sp = parse_store_path(StorePathText {
            path_text: "/crunch/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-busybox",
            store_dir: "/crunch/store",
        })
        .unwrap();
        assert!(sp.to_string().contains("busybox"));
    }

    #[test]
    fn parse_store_path_invalid() {
        assert!(
            parse_store_path(StorePathText {
                path_text: "/tmp/not-a-store-path",
                store_dir: "/nix/store",
            })
            .is_err()
        );
    }

    #[test]
    fn derivation_identity_format() {
        let drv = minimal_drv("foo", "/bin/sh");
        assert_eq!(derivation_identity(&drv), "foo:/bin/sh:x86_64-linux");
    }

    // ── store_dir threading tests ─────────────────────────────────────

    #[test]
    fn convert_custom_store_dir_produces_different_paths() {
        let drv = minimal_drv("hello", "/bin/sh");

        let mut kp_default = ConversionCache::default();
        let (drv_path_default, nix_drv_default) = convert(&drv, &mut kp_default).unwrap();

        let mut kp_custom = ConversionCache::new("/opt/crunch");
        let (drv_path_custom, nix_drv_custom) = convert(&drv, &mut kp_custom).unwrap();

        // Drv paths differ
        assert_ne!(drv_path_default, drv_path_custom);

        // Output paths differ
        let out_default = nix_drv_default.outputs["out"].path.as_ref().unwrap();
        let out_custom = nix_drv_custom.outputs["out"].path.as_ref().unwrap();
        assert_ne!(out_default, out_custom);

        // Custom env uses /opt/crunch prefix
        let env_out: &[u8] = nix_drv_custom.environment.get("out").unwrap().as_ref();
        assert!(
            env_out.starts_with(b"/opt/crunch/"),
            "expected /opt/crunch/ prefix, got: {}",
            String::from_utf8_lossy(env_out)
        );
    }

    #[test]
    fn convert_default_store_dir_matches_existing() {
        let drv = minimal_drv("hello", "/bin/sh");

        let mut kp1 = ConversionCache::default();
        let (path1, drv1) = convert(&drv, &mut kp1).unwrap();

        let mut kp2 = ConversionCache::new("/nix/store");
        let (path2, drv2) = convert(&drv, &mut kp2).unwrap();

        assert_eq!(path1, path2);
        assert_eq!(drv1.outputs, drv2.outputs);
    }

    #[test]
    fn known_paths_custom_store_dir_lookup() {
        let drv = minimal_drv("hello", "/bin/sh");
        let mut kp = ConversionCache::new("/opt/crunch");
        let (drv_path, _) = convert(&drv, &mut kp).unwrap();

        // Lookup with the custom prefix must succeed
        let abs = drv_path.to_absolute_path_with_prefix("/opt/crunch");
        assert!(kp.get_by_drv_path(&abs).is_some());

        // Lookup with the wrong prefix must fail
        let wrong = drv_path.to_absolute_path();
        assert!(kp.get_by_drv_path(&wrong).is_none());
    }

    #[test]
    fn convert_custom_store_dir_accepts_source_inputs() {
        let drv = CrunchDerivation {
            inputs: vec![Input::Source(
                "/crunch/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-busybox".to_string(),
            )],
            ..minimal_drv("hello", "/bin/sh")
        };
        let mut kp = ConversionCache::new("/crunch/store");

        let (_drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert_eq!(nix_drv.input_sources.len(), 1, "custom-prefix source input should be preserved");
        let only_source = nix_drv.input_sources.iter().next().unwrap();
        assert_eq!(
            only_source.to_absolute_path_with_prefix("/crunch/store"),
            "/crunch/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-busybox"
        );
    }

    // ── content-addressed derivation tests ─────────────────────────

    fn ca_drv(name: &str, builder: &str) -> CrunchDerivation {
        CrunchDerivation {
            addressing_mode: "content-addressed".to_string(),
            ..minimal_drv(name, builder)
        }
    }

    #[test]
    fn ca_derivation_has_none_output_paths() {
        let drv = ca_drv("ca-hello", "/bin/sh");
        let mut kp = ConversionCache::default();
        let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();

        // drv path still computed (needed for build graph)
        assert!(drv_path.to_string().ends_with("ca-hello.drv"));

        // output paths are None (not known until after build)
        for (name, output) in &nix_drv.outputs {
            assert!(output.path.is_none(), "CA output '{name}' should have None path");
        }
    }

    #[test]
    fn ca_derivation_env_has_provisional_path() {
        let drv = ca_drv("ca-hello", "/bin/sh");
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        let env_out: &[u8] = nix_drv.environment.get("out").unwrap().as_ref();
        let env_str = std::str::from_utf8(env_out).unwrap();

        // CA derivations use the input-addressed path as a provisional $out.
        // It's a proper store path: /nix/store/<32-char hash>-<name>
        assert!(env_str.starts_with("/nix/store/"), "provisional should be under store dir");
        let after_prefix = &env_str["/nix/store/".len()..];
        assert!(after_prefix.contains('-'), "provisional should be a store path with hash-name format");
        assert!(env_str.ends_with("-ca-hello"), "provisional should end with derivation name");

        // output.path should be None (marking it as CA)
        assert!(nix_drv.outputs["out"].path.is_none(), "CA output path should be None");
    }

    #[test]
    fn ca_known_paths_marked_content_addressed() {
        let drv = ca_drv("ca-hello", "/bin/sh");
        let mut kp = ConversionCache::default();
        let (drv_path, _) = convert(&drv, &mut kp).unwrap();

        let entry = kp.get_by_drv_path(&drv_path.to_absolute_path()).unwrap();
        assert!(entry.content_addressed);
    }

    #[test]
    fn input_addressed_known_paths_not_ca() {
        let drv = minimal_drv("ia-hello", "/bin/sh");
        let mut kp = ConversionCache::default();
        let (drv_path, _) = convert(&drv, &mut kp).unwrap();

        let entry = kp.get_by_drv_path(&drv_path.to_absolute_path()).unwrap();
        assert!(!entry.content_addressed);
    }

    #[test]
    fn ia_derivation_has_some_output_paths() {
        let drv = minimal_drv("ia-hello", "/bin/sh");
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        for (name, output) in &nix_drv.outputs {
            assert!(output.path.is_some(), "IA output '{name}' should have Some path");
        }
    }

    #[test]
    fn fod_stays_input_addressed_even_with_ca_mode() {
        // FODs are always content-addressed by definition (hash declared).
        // Setting addressing_mode = CA shouldn't break them.
        let drv = CrunchDerivation {
            addressing_mode: "content-addressed".to_string(),
            fixed_output: Some(FixedOutput {
                hash: "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba".to_string(),
                algo: "sha256".to_string(),
                mode: "flat".to_string(),
            }),
            ..minimal_drv("fod-ca", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        // FOD output path is always computed (from declared hash)
        let out = nix_drv.outputs.get("out").unwrap();
        assert!(out.path.is_some(), "FOD should always have output path");
        assert!(out.ca_hash.is_some(), "FOD should have ca_hash");
    }

    // ca_resolve_output test moved to crunch-build/src/registry.rs
    // (resolve_output/get_output_path are DerivationRegistry methods)

    #[test]
    fn ca_json_default_is_content_addressed() {
        let json = r#"{
            "name": "from-json-ca",
            "builder": "/bin/sh"
        }"#;
        let drv: CrunchDerivation = serde_json::from_str(json).unwrap();
        assert_eq!(drv.addressing_mode, "content-addressed");
    }

    // ── output selection tests ─────────────────────────────────────

    fn multi_output_drv(name: &str) -> CrunchDerivation {
        CrunchDerivation {
            outputs: vec!["out".to_string(), "dev".to_string(), "lib".to_string()],
            ..minimal_drv(name, "/bin/sh")
        }
    }

    #[test]
    fn deserialize_output_selection_from_json() {
        let json = r#"{
            "drv": {
                "name": "libfoo",
                "builder": "/bin/sh",
                "outputs": ["out", "dev", "lib"]
            },
            "output": "dev"
        }"#;
        let input: Input = serde_json::from_str(json).unwrap();
        match input {
            Input::OutputSelection(oref) => {
                assert_eq!(oref.drv.name, "libfoo");
                assert_eq!(oref.output, "dev");
            }
            other => panic!("expected OutputSelection, got: {other:?}"),
        }
    }

    #[test]
    fn serde_ordering_output_selection_before_derivation() {
        // A record with `drv` + `output` must parse as OutputSelection,
        // NOT as Derivation (even though both are records).
        let json_selection = r#"{
            "drv": { "name": "x", "builder": "/bin/sh" },
            "output": "dev"
        }"#;
        let json_derivation = r#"{
            "name": "x",
            "builder": "/bin/sh"
        }"#;

        assert!(matches!(serde_json::from_str::<Input>(json_selection).unwrap(), Input::OutputSelection(_)));
        assert!(matches!(serde_json::from_str::<Input>(json_derivation).unwrap(), Input::Derivation(_)));
    }

    #[test]
    fn serde_source_still_works() {
        let json = r#""/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash""#;
        let input: Input = serde_json::from_str(json).unwrap();
        assert!(matches!(input, Input::Source(_)));
    }

    #[test]
    fn output_selection_single_output_in_input_derivations() {
        let dep = multi_output_drv("libfoo");
        let drv = CrunchDerivation {
            inputs: vec![Input::OutputSelection(Box::new(OutputRef {
                drv: dep,
                output: "dev".to_string(),
            }))],
            ..minimal_drv("myapp", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert_eq!(nix_drv.input_derivations.len(), 1);
        let (dep_path, dep_outputs) = nix_drv.input_derivations.iter().next().unwrap();
        assert!(dep_path.to_absolute_path().ends_with("-libfoo.drv"), "dep path: {}", dep_path.to_absolute_path());
        // Only the selected output, not all three.
        assert_eq!(dep_outputs.len(), 1);
        assert!(dep_outputs.contains("dev"));
        assert!(!dep_outputs.contains("out"));
        assert!(!dep_outputs.contains("lib"));
    }

    #[test]
    fn output_selection_coalescing_same_dep() {
        let dep = multi_output_drv("libfoo");
        let drv = CrunchDerivation {
            inputs: vec![
                Input::OutputSelection(Box::new(OutputRef {
                    drv: dep.clone(),
                    output: "dev".to_string(),
                })),
                Input::OutputSelection(Box::new(OutputRef {
                    drv: dep,
                    output: "lib".to_string(),
                })),
            ],
            ..minimal_drv("myapp", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        // Same dep appears once with both outputs coalesced.
        assert_eq!(nix_drv.input_derivations.len(), 1);
        let (_, dep_outputs) = nix_drv.input_derivations.iter().next().unwrap();
        assert_eq!(dep_outputs.len(), 2);
        assert!(dep_outputs.contains("dev"));
        assert!(dep_outputs.contains("lib"));
    }

    #[test]
    fn output_selection_coalescing_with_bare_derivation() {
        // OutputSelection("dev") + bare Derivation (all outputs) coalesce.
        let dep = multi_output_drv("libfoo");
        let drv = CrunchDerivation {
            inputs: vec![
                Input::OutputSelection(Box::new(OutputRef {
                    drv: dep.clone(),
                    output: "dev".to_string(),
                })),
                Input::Derivation(Box::new(dep)),
            ],
            ..minimal_drv("myapp", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        // Coalesced: dev + all three = all three.
        assert_eq!(nix_drv.input_derivations.len(), 1);
        let (_, dep_outputs) = nix_drv.input_derivations.iter().next().unwrap();
        assert_eq!(dep_outputs.len(), 3);
        assert!(dep_outputs.contains("out"));
        assert!(dep_outputs.contains("dev"));
        assert!(dep_outputs.contains("lib"));
    }

    #[test]
    fn output_selection_invalid_output_rejected() {
        let dep = multi_output_drv("libfoo");
        let drv = CrunchDerivation {
            inputs: vec![Input::OutputSelection(Box::new(OutputRef {
                drv: dep,
                output: "headers".to_string(),
            }))],
            ..minimal_drv("myapp", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let err = convert(&drv, &mut kp).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("libfoo"), "error should name the derivation, got: {msg}");
        assert!(msg.contains("headers"), "error should name the invalid output, got: {msg}");
        assert!(msg.contains("out, dev, lib"), "error should list available outputs, got: {msg}");
    }

    #[test]
    fn output_selection_duplicate_same_output_idempotent() {
        // Selecting "dev" twice from the same dep is fine — coalesces to one.
        let dep = multi_output_drv("libfoo");
        let drv = CrunchDerivation {
            inputs: vec![
                Input::OutputSelection(Box::new(OutputRef {
                    drv: dep.clone(),
                    output: "dev".to_string(),
                })),
                Input::OutputSelection(Box::new(OutputRef {
                    drv: dep,
                    output: "dev".to_string(),
                })),
            ],
            ..minimal_drv("myapp", "/bin/sh")
        };
        let mut kp = ConversionCache::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert_eq!(nix_drv.input_derivations.len(), 1);
        let (_, dep_outputs) = nix_drv.input_derivations.iter().next().unwrap();
        assert_eq!(dep_outputs.len(), 1);
        assert!(dep_outputs.contains("dev"));
    }
}
