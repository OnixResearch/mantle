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
    let store_dir = known_paths.store_dir().to_string();
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
            .get_hdm_by_drv_path(&parent_drv_path.to_absolute_path_with_prefix(&store_dir))
            .unwrap_or_else(|| {
                panic!(
                    "BUG: parent derivation {} not in KnownPaths during HDM computation",
                    parent_drv_path
                )
            })
    });

    let is_ca = drv.addressing_mode == "content-addressed" && drv.fixed_output.is_none();

    // 5. Compute output paths.
    //    Input-addressed (and FODs): compute now, fill into derivation.
    //    Content-addressed: output paths are unknown until after the build.
    //    Set environment to placeholders so the builder has a $out to write to.
    if is_ca {
        for (output_name, output) in nix_drv.outputs.iter_mut() {
            assert!(output.path.is_none());
            let placeholder = nix_compat::store_path::hash_placeholder(output_name);
            nix_drv.environment.insert(output_name.clone(), placeholder.into());
        }
    } else {
        nix_drv.calculate_output_paths_with_store_dir(&drv.name, &hdm, &store_dir)?;
    }

    // 6. Compute the .drv store path
    let drv_path = nix_drv.calculate_derivation_path_with_store_dir(&drv.name, &store_dir)?;

    // 7. Compute ATerm hash for dedup key
    let aterm_bytes = nix_drv.to_aterm_bytes();
    let aterm_hash = *blake3::hash(&aterm_bytes).as_bytes();

    // 8. Register in KnownPaths
    known_paths.insert_ca(aterm_hash, drv_path.clone(), hdm, nix_drv.clone(), is_ca);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::known_paths::KnownPaths;
    use crate::types::*;
    use nix_compat::nixhash::CAHash;

    fn minimal_drv(name: &str, builder: &str) -> CrunchDerivation {
        CrunchDerivation {
            name: name.to_string(),
            builder: builder.to_string(),
            system: "x86_64-linux".to_string(),
            args: vec![],
            outputs: vec!["out".to_string()],
            env: Default::default(),
            inputs: vec![],
            fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
        }
    }

    // ── Phase 2: basic derivations ────────────────────────────────────

    #[test]
    fn simple_drv_exact_store_path() {
        let drv = minimal_drv("hello", "/bin/sh");
        let mut kp = KnownPaths::default();
        let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();
        let out_path = nix_drv.outputs.get("out").unwrap().path.as_ref().unwrap();

        assert_eq!(
            drv_path.to_absolute_path(),
            "/nix/store/hyvs2ylkzjddglrw4vd0kzc2dgdypwhg-hello.drv"
        );
        assert_eq!(
            out_path.to_absolute_path(),
            "/nix/store/acsr0icqcmd1yx786z9ia4l1lvg2fw5c-hello"
        );
    }

    #[test]
    fn builder_and_system_propagate() {
        let drv = minimal_drv("hello", "/usr/bin/env");
        let mut kp = KnownPaths::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert_eq!(nix_drv.builder, "/usr/bin/env");
        assert_eq!(nix_drv.system, "x86_64-linux");
    }

    #[test]
    fn environment_auto_populated() {
        let drv = minimal_drv("hello", "/bin/sh");
        let mut kp = KnownPaths::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert_eq!(nix_drv.environment.get("system").unwrap(), "x86_64-linux");
        assert_eq!(nix_drv.environment.get("builder").unwrap(), "/bin/sh");
        assert_eq!(nix_drv.environment.get("name").unwrap(), "hello");

        // "out" env entry matches the computed output path
        let out_path = nix_drv
            .outputs
            .get("out")
            .unwrap()
            .path
            .as_ref()
            .unwrap()
            .to_absolute_path();
        let out_env: &[u8] = nix_drv.environment.get("out").unwrap().as_ref();
        assert_eq!(out_env, out_path.as_bytes());
    }

    #[test]
    fn multi_output_reflected_in_derivation() {
        let drv = CrunchDerivation {
            outputs: vec!["out".to_string(), "lib".to_string(), "dev".to_string()],
            ..minimal_drv("multi", "/bin/sh")
        };
        let mut kp = KnownPaths::default();
        let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert_eq!(
            drv_path.to_absolute_path(),
            "/nix/store/dw47932whb0073k1m8j1lq3q8v507fsg-multi.drv"
        );
        assert_eq!(nix_drv.outputs.len(), 3);
        assert!(nix_drv.outputs.contains_key("out"));
        assert!(nix_drv.outputs.contains_key("lib"));
        assert!(nix_drv.outputs.contains_key("dev"));
    }

    #[test]
    fn multi_output_distinct_paths() {
        let drv = CrunchDerivation {
            outputs: vec!["out".to_string(), "lib".to_string(), "dev".to_string()],
            ..minimal_drv("multi", "/bin/sh")
        };
        let mut kp = KnownPaths::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        let out = nix_drv.outputs["out"].path.as_ref().unwrap().to_absolute_path();
        let lib = nix_drv.outputs["lib"].path.as_ref().unwrap().to_absolute_path();
        let dev = nix_drv.outputs["dev"].path.as_ref().unwrap().to_absolute_path();

        assert_eq!(out, "/nix/store/87bra95jlhk67vvw4zfz8q4df850drfg-multi");
        assert_eq!(lib, "/nix/store/q82x493bnlma6xah3bxgz4ap55f1i6bd-multi-lib");
        assert_eq!(dev, "/nix/store/73pmhlffls3p3pyyr2cp7g53vxsbn4ar-multi-dev");

        assert_ne!(out, lib);
        assert_ne!(lib, dev);
        assert_ne!(out, dev);
    }

    // ── Phase 3: inputs and dependencies ──────────────────────────────

    #[test]
    fn source_input_wires_to_input_sources() {
        let drv = CrunchDerivation {
            inputs: vec![Input::Source(
                "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash".to_string(),
            )],
            ..minimal_drv("with-src", "/bin/sh")
        };
        let mut kp = KnownPaths::default();
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
        let mut kp = KnownPaths::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert_eq!(nix_drv.input_derivations.len(), 1);
        let (dep_path, dep_outputs) = nix_drv.input_derivations.iter().next().unwrap();
        assert_eq!(
            dep_path.to_absolute_path(),
            "/nix/store/zqnanhvcp038ycgaw5zxccng3b1gm136-libfoo.drv"
        );
        assert!(dep_outputs.contains("out"));
    }

    #[test]
    fn nested_drv_registered_in_known_paths() {
        let dep = minimal_drv("libfoo", "/bin/sh");
        let drv = CrunchDerivation {
            inputs: vec![Input::Derivation(Box::new(dep))],
            ..minimal_drv("myapp", "/bin/sh")
        };
        let mut kp = KnownPaths::default();
        let (parent_path, _) = convert(&drv, &mut kp).unwrap();

        // Both parent and dep are in KnownPaths
        assert!(kp.get_by_drv_path(&parent_path.to_absolute_path()).is_some());
        assert!(kp
            .get_by_drv_path("/nix/store/zqnanhvcp038ycgaw5zxccng3b1gm136-libfoo.drv")
            .is_some());
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
            inputs: vec![
                Input::Derivation(Box::new(left)),
                Input::Derivation(Box::new(right)),
            ],
            ..minimal_drv("top", "/bin/sh")
        };
        let mut kp = KnownPaths::default();
        let (_, nix_drv) = convert(&top, &mut kp).unwrap();

        // top depends on left and right
        assert_eq!(nix_drv.input_derivations.len(), 2);

        // shared's drv path appears once in KnownPaths (dedup via identity)
        let shared_path = {
            let mut kp_check = KnownPaths::default();
            let (p, _) = convert(&minimal_drv("shared", "/bin/sh"), &mut kp_check).unwrap();
            p.to_absolute_path()
        };
        assert!(kp.get_by_drv_path(&shared_path).is_some());
    }

    #[test]
    fn cycle_returns_circular_dependency_error() {
        // Same identity nested inside itself
        let inner = minimal_drv("loop", "/bin/sh");
        let outer = CrunchDerivation {
            inputs: vec![Input::Derivation(Box::new(inner))],
            ..minimal_drv("loop", "/bin/sh")
        };
        let mut kp = KnownPaths::default();
        let err = convert(&outer, &mut kp).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("ircular"),
            "expected circular dependency error, got: {msg}"
        );
    }

    // ── Phase 4: fixed-output derivations ─────────────────────────────

    #[test]
    fn fod_flat_sha256() {
        let drv = CrunchDerivation {
            fixed_output: Some(FixedOutput {
                hash: "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba"
                    .to_string(),
                algo: "sha256".to_string(),
                mode: "flat".to_string(),
            }),
            ..minimal_drv("src-flat", "/bin/sh")
        };
        let mut kp = KnownPaths::default();
        let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert_eq!(
            drv_path.to_absolute_path(),
            "/nix/store/wg3bgqpvdksabf8knkj8przj71jq2jas-src-flat.drv"
        );
        let out = nix_drv.outputs.get("out").unwrap();
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
                hash: "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba"
                    .to_string(),
                algo: "sha256".to_string(),
                mode: "recursive".to_string(),
            }),
            ..minimal_drv("src-rec", "/bin/sh")
        };
        let mut kp = KnownPaths::default();
        let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert_eq!(
            drv_path.to_absolute_path(),
            "/nix/store/gpbxkgfx2xjhq2ybi1jwsps6nd23lj7y-src-rec.drv"
        );
        let out = nix_drv.outputs.get("out").unwrap();
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
        let mut kp = KnownPaths::default();
        let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();

        assert_eq!(
            drv_path.to_absolute_path(),
            "/nix/store/qppxjry1mz95f8r35nx4l4dbn0yiq84g-src-sri.drv"
        );
        assert_eq!(
            nix_drv
                .outputs
                .get("out")
                .unwrap()
                .path
                .as_ref()
                .unwrap()
                .to_absolute_path(),
            "/nix/store/mikp7vivga7ysvaqnj6594dm3qz3qdz5-src-sri"
        );
    }

    #[test]
    fn fod_hex_parses_to_flat_ca_hash() {
        let fo = FixedOutput {
            hash: "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba"
                .to_string(),
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
        let mut kp = KnownPaths::default();
        let err = convert(&drv, &mut kp).unwrap_err();
        assert!(
            err.to_string().contains("hash algorithm"),
            "expected hash algorithm error, got: {err}"
        );
    }

    #[test]
    fn fod_invalid_mode_rejected() {
        let drv = CrunchDerivation {
            fixed_output: Some(FixedOutput {
                hash: "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba"
                    .to_string(),
                algo: "sha256".to_string(),
                mode: "broken".to_string(),
            }),
            ..minimal_drv("bad-mode", "/bin/sh")
        };
        let mut kp = KnownPaths::default();
        let err = convert(&drv, &mut kp).unwrap_err();
        assert!(
            err.to_string().contains("hash mode"),
            "expected hash mode error, got: {err}"
        );
    }

    // ── Private helper tests ─────────────────────────────────────────

    #[test]
    fn parse_store_path_valid() {
        let sp = parse_store_path("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash").unwrap();
        assert!(sp.to_string().contains("bash"));
    }

    #[test]
    fn parse_store_path_invalid() {
        assert!(parse_store_path("/tmp/not-a-store-path").is_err());
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

        let mut kp_default = KnownPaths::default();
        let (drv_path_default, nix_drv_default) = convert(&drv, &mut kp_default).unwrap();

        let mut kp_custom = KnownPaths::new("/opt/crunch");
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

        let mut kp1 = KnownPaths::default();
        let (path1, drv1) = convert(&drv, &mut kp1).unwrap();

        let mut kp2 = KnownPaths::new("/nix/store");
        let (path2, drv2) = convert(&drv, &mut kp2).unwrap();

        assert_eq!(path1, path2);
        assert_eq!(drv1.outputs, drv2.outputs);
    }

    #[test]
    fn known_paths_custom_store_dir_lookup() {
        let drv = minimal_drv("hello", "/bin/sh");
        let mut kp = KnownPaths::new("/opt/crunch");
        let (drv_path, _) = convert(&drv, &mut kp).unwrap();

        // Lookup with the custom prefix must succeed
        let abs = drv_path.to_absolute_path_with_prefix("/opt/crunch");
        assert!(kp.get_by_drv_path(&abs).is_some());

        // Lookup with the wrong prefix must fail
        let wrong = drv_path.to_absolute_path();
        assert!(kp.get_by_drv_path(&wrong).is_none());
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
        let mut kp = KnownPaths::default();
        let (drv_path, nix_drv) = convert(&drv, &mut kp).unwrap();

        // drv path still computed (needed for build graph)
        assert!(drv_path.to_string().ends_with("ca-hello.drv"));

        // output paths are None (not known until after build)
        for (name, output) in &nix_drv.outputs {
            assert!(
                output.path.is_none(),
                "CA output '{name}' should have None path"
            );
        }
    }

    #[test]
    fn ca_derivation_env_has_placeholders() {
        let drv = ca_drv("ca-hello", "/bin/sh");
        let mut kp = KnownPaths::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        let env_out: &[u8] = nix_drv.environment.get("out").unwrap().as_ref();
        let env_str = std::str::from_utf8(env_out).unwrap();

        // Placeholder starts with / and is a nixbase32-encoded hash
        assert!(env_str.starts_with('/'), "placeholder should start with /");
        assert!(env_str.len() > 1, "placeholder should not be empty");
        // It should NOT start with /nix/store (that's the input-addressed path)
        assert!(!env_str.starts_with("/nix/store"), "CA env should be placeholder, not store path");
    }

    #[test]
    fn ca_known_paths_marked_content_addressed() {
        let drv = ca_drv("ca-hello", "/bin/sh");
        let mut kp = KnownPaths::default();
        let (drv_path, _) = convert(&drv, &mut kp).unwrap();

        let entry = kp.get_by_drv_path(&drv_path.to_absolute_path()).unwrap();
        assert!(entry.content_addressed);
    }

    #[test]
    fn input_addressed_known_paths_not_ca() {
        let drv = minimal_drv("ia-hello", "/bin/sh");
        let mut kp = KnownPaths::default();
        let (drv_path, _) = convert(&drv, &mut kp).unwrap();

        let entry = kp.get_by_drv_path(&drv_path.to_absolute_path()).unwrap();
        assert!(!entry.content_addressed);
    }

    #[test]
    fn ia_derivation_has_some_output_paths() {
        let drv = minimal_drv("ia-hello", "/bin/sh");
        let mut kp = KnownPaths::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        for (name, output) in &nix_drv.outputs {
            assert!(
                output.path.is_some(),
                "IA output '{name}' should have Some path"
            );
        }
    }

    #[test]
    fn fod_stays_input_addressed_even_with_ca_mode() {
        // FODs are always content-addressed by definition (hash declared).
        // Setting addressing_mode = CA shouldn't break them.
        let drv = CrunchDerivation {
            addressing_mode: "content-addressed".to_string(),
            fixed_output: Some(FixedOutput {
                hash: "08813cbee9903c62be4c5027726a418a300da4500b2d369d3af9286f4815ceba"
                    .to_string(),
                algo: "sha256".to_string(),
                mode: "flat".to_string(),
            }),
            ..minimal_drv("fod-ca", "/bin/sh")
        };
        let mut kp = KnownPaths::default();
        let (_, nix_drv) = convert(&drv, &mut kp).unwrap();

        // FOD output path is always computed (from declared hash)
        let out = nix_drv.outputs.get("out").unwrap();
        assert!(out.path.is_some(), "FOD should always have output path");
        assert!(out.ca_hash.is_some(), "FOD should have ca_hash");
    }

    #[test]
    fn ca_resolve_output_updates_known_paths() {
        let drv = ca_drv("ca-resolve", "/bin/sh");
        let mut kp = KnownPaths::default();
        let (drv_path, _) = convert(&drv, &mut kp).unwrap();
        let drv_abs = drv_path.to_absolute_path();

        // Before resolve: no output path
        assert!(kp.get_output_path(&drv_abs, "out").is_none());

        // Simulate post-build resolution
        let final_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed(
            "ca-resolve", [0xbb; 20]
        ).unwrap();
        kp.resolve_output(&drv_abs, "out", final_path.clone());

        // After resolve: output path available
        assert_eq!(kp.get_output_path(&drv_abs, "out").unwrap(), final_path);
    }

    #[test]
    fn ca_json_default_is_content_addressed() {
        let json = r#"{
            "name": "from-json-ca",
            "builder": "/bin/sh"
        }"#;
        let drv: CrunchDerivation = serde_json::from_str(json).unwrap();
        assert_eq!(drv.addressing_mode, "content-addressed");
    }
}
