//! Translate `nix_compat::Derivation` → `snix_build::BuildRequest`.
//!
//! Adapted from snix-glue's `derivation_into_build_request`, simplified:
//! no structured_attrs, no passAsFile (those are Nix-isms that crunch
//! doesn't need in v0).

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::PathBuf;

use bstr::BString;
use bytes::Bytes;
use nix_compat::derivation::{Derivation, Output};
use nix_compat::store_path::hash_placeholder;
use nix_compat::{nixbase32, store_path::StorePath};
use snix_build::buildservice::{BuildConstraints, BuildRequest, EnvVar};
use snix_castore::Node;

use crunch_glue::KnownPaths;

/// Environment variables that crunch sets in every sandbox build,
/// matching Nix's sandbox conventions for compatibility with build
/// scripts that expect them.
const SANDBOX_ENV_VARS: [(&str, &str); 12] = [
    ("HOME", "/homeless-shelter"),
    ("NIX_BUILD_CORES", "0"),
    ("NIX_BUILD_TOP", "/build"),
    ("NIX_LOG_FD", "2"),
    ("NIX_STORE", "/nix/store"),
    ("PATH", "/path-not-set"),
    ("PWD", "/build"),
    ("TEMP", "/build"),
    ("TEMPDIR", "/build"),
    ("TERM", "xterm-256color"),
    ("TMP", "/build"),
    ("TMPDIR", "/build"),
];

/// Translate a `Derivation` into a `BuildRequest`.
///
/// `inputs` maps store path → castore Node for every input that must be
/// visible in the sandbox. The caller resolves these from the store
/// before calling this function.
///
/// `known_paths` is used to look up nested derivation outputs when
/// resolving `input_derivations`.
/// Compile-time: sandbox env vars must not be empty.
const _: () = assert!(SANDBOX_ENV_VARS.len() > 0);

pub fn derivation_to_build_request(
    derivation: &Derivation,
    inputs: &BTreeMap<StorePath<String>, Node>,
    store_dir: &str,
) -> Result<BuildRequest, crate::Error> {
    // Tiger Style: assert preconditions.
    debug_assert!(!derivation.builder.is_empty(), "builder must not be empty");
    debug_assert!(!derivation.outputs.is_empty(), "must have at least one output");
    debug_assert!(!store_dir.is_empty(), "store_dir must not be empty");
    debug_assert!(store_dir.starts_with('/'), "store_dir must be absolute path");

    // command_args = [builder] ++ arguments, with placeholders replaced
    let mut command_args: Vec<String> = Vec::with_capacity(derivation.arguments.len() + 1);
    command_args.push(derivation.builder.clone());
    for arg in &derivation.arguments {
        command_args.push(replace_placeholders(arg, &derivation.outputs));
    }

    // Environment: start with sandbox defaults, then add derivation env.
    // NIX_STORE uses the configured store dir.
    let mut env: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for (k, v) in &SANDBOX_ENV_VARS {
        if *k == "NIX_STORE" {
            env.insert(k.to_string(), store_dir.as_bytes().to_vec());
        } else {
            env.insert(k.to_string(), v.as_bytes().to_vec());
        }
    }
    for (k, v) in &derivation.environment {
        let replaced = replace_placeholders_bstr(v, &derivation.outputs);
        env.insert(
            k.clone(),
            Vec::from(replaced),
        );
    }

    // Constraints
    let mut constraints = HashSet::from([
        BuildConstraints::System(derivation.system.clone()),
        BuildConstraints::ProvideBinSh,
    ]);

    // FODs get network access
    let is_fod = derivation.outputs.len() == 1
        && derivation
            .outputs
            .get("out")
            .is_some_and(|o| o.is_fixed());
    if is_fod {
        constraints.insert(BuildConstraints::NetworkAccess);
    }

    // Refscan needles: output digests first, then input digests
    let refscan_needles: Vec<String> = derivation
        .outputs
        .values()
        .filter_map(|o| o.path.as_ref())
        .map(|p| nixbase32::encode(p.digest()))
        .chain(inputs.keys().map(|p| nixbase32::encode(p.digest())))
        .collect();

    // Tiger Style: assert command_args has at least the builder.
    debug_assert!(!command_args.is_empty());
    debug_assert!(!env.is_empty(), "environment must include sandbox vars");

    Ok(BuildRequest {
        command_args,
        outputs: derivation
            .outputs
            .iter()
            .map(|(output_name, o)| {
                let path_str = o.path_str();
                if path_str.is_empty() {
                    // CA derivation: use the placeholder path (from env)
                    // as the sandbox output location.
                    let placeholder = derivation.environment
                        .get(output_name)
                        .map(|v| String::from_utf8_lossy(v).to_string())
                        .unwrap_or_default();
                    if placeholder.starts_with('/') {
                        PathBuf::from(&placeholder[1..])
                    } else {
                        PathBuf::from(&placeholder)
                    }
                } else {
                    // Strip leading '/' — BuildRequest wants relative paths
                    PathBuf::from(&path_str[1..])
                }
            })
            .collect(),
        environment_vars: env
            .into_iter()
            .map(|(key, value)| EnvVar {
                key,
                value: Bytes::from(value),
            })
            .collect(),
        inputs: inputs
            .iter()
            .map(|(path, node)| {
                (
                    path.to_string()
                        .as_str()
                        .try_into()
                        .expect("store path basename must be valid PathComponent"),
                    node.clone(),
                )
            })
            .collect(),
        inputs_dir: store_dir[1..].into(),
        constraints,
        working_dir: "build".into(),
        scratch_paths: vec!["build".into(), store_dir[1..].into()],
        additional_files: vec![],
        refscan_needles,
    })
}

/// Collect all store paths that must be visible in the sandbox.
///
/// For `input_sources`, the store path itself is needed.
/// For `input_derivations`, we need the output paths of each referenced
/// derivation output.
pub fn collect_input_paths(
    derivation: &Derivation,
    known_paths: &KnownPaths,
) -> Result<BTreeSet<StorePath<String>>, crate::Error> {
    let mut paths = BTreeSet::new();

    // Source inputs
    for source in &derivation.input_sources {
        paths.insert(source.clone());
    }

    // Derivation input outputs
    for (drv_path, output_names) in &derivation.input_derivations {
        let drv_abs = drv_path.to_absolute_path_with_prefix(known_paths.store_dir());
        // Verify the derivation is in KnownPaths
        if known_paths.get_by_drv_path(&drv_abs).is_none() {
            return Err(crate::Error::DerivationNotFound {
                path: drv_path.clone(),
            });
        }
        for output_name in output_names {
            // Use get_output_path which handles both input-addressed
            // (reads from derivation.outputs[].path) and content-addressed
            // (reads from resolved_outputs).
            let output_path = known_paths
                .get_output_path(&drv_abs, output_name)
                .ok_or_else(|| crate::Error::OutputNoPath {
                    output: output_name.clone(),
                    drv_name: drv_path.to_string(),
                })?;
            paths.insert(output_path);
        }
    }

    Ok(paths)
}

/// Replace `hash_placeholder(outputName)` strings with actual output paths.
fn replace_placeholders(s: &str, outputs: &BTreeMap<String, Output>) -> String {
    let mut result = s.to_owned();
    for (name, output) in outputs {
        if let Some(path) = output.path.as_ref() {
            let placeholder = hash_placeholder(name.as_str());
            result = result.replace(&placeholder, &path.to_absolute_path());
        }
    }
    result
}

/// Replace placeholders in a BString.
fn replace_placeholders_bstr(s: &BString, outputs: &BTreeMap<String, Output>) -> BString {
    use bstr::ByteSlice;
    let mut result = s.clone();
    for (name, output) in outputs {
        if let Some(path) = output.path.as_ref() {
            let placeholder = hash_placeholder(name.as_str());
            result = result
                .replace(placeholder.as_bytes(), path.to_absolute_path().as_bytes())
                .into();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix_compat::derivation::Derivation;
    use std::collections::BTreeMap;

    // ── Helper: build a derivation and register in KnownPaths ──

    fn make_drv_with_name(name: &str) -> Derivation {
        let mut outputs = BTreeMap::new();
        outputs.insert(
            "out".to_string(),
            Output { path: None, ca_hash: None },
        );
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), name.into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());

        let mut drv = Derivation {
            arguments: vec!["-c".into(), "echo > $out".into()],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };
        let hdm = drv.hash_derivation_modulo(|_| panic!("no parent"));
        drv.calculate_output_paths(name, &hdm).unwrap();
        let _ = drv.calculate_derivation_path(name).unwrap();
        drv
    }

    fn register_drv(name: &str, kp: &mut KnownPaths) -> (StorePath<String>, Derivation) {
        let drv = make_drv_with_name(name);
        let aterm_hash = nix_compat::derivation::CAHash::Nar(
            nix_compat::nixhash::NixHash::Sha256([0; 32]),
        );
        // Use drv name bytes as a unique fake aterm hash
        let mut fake_hash = [0u8; 32];
        for (i, b) in name.bytes().enumerate().take(32) {
            fake_hash[i] = b;
        }
        let hdm = drv.hash_derivation_modulo(|_| panic!("no parent"));
        let drv_path = drv.calculate_derivation_path(name).unwrap();
        kp.insert(fake_hash, drv_path.clone(), hdm, drv.clone());
        (drv_path, drv)
    }

    /// Construct a minimal Derivation for testing.
    fn test_derivation() -> Derivation {
        let mut outputs = BTreeMap::new();
        outputs.insert(
            "out".to_string(),
            Output {
                path: None,
                ca_hash: None,
            },
        );

        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), "test".into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());

        let mut drv = Derivation {
            arguments: vec!["-c".to_string(), "echo hello > $out".to_string()],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };

        // Compute paths so outputs have real values
        let hdm = drv.hash_derivation_modulo(|_| {
            panic!("no parent derivations")
        });
        drv.calculate_output_paths("test", &hdm).unwrap();
        let _ = drv.calculate_derivation_path("test").unwrap();

        drv
    }

    #[test]
    fn build_request_has_correct_builder() {
        let drv = test_derivation();
        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store").unwrap();

        assert_eq!(req.command_args[0], "/bin/sh");
        assert_eq!(req.command_args[1], "-c");
    }

    #[test]
    fn build_request_has_sandbox_env() {
        let drv = test_derivation();
        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store").unwrap();

        let env_map: BTreeMap<&str, &[u8]> = req
            .environment_vars
            .iter()
            .map(|e| (e.key.as_str(), e.value.as_ref()))
            .collect();

        assert_eq!(*env_map.get("HOME").unwrap(), &b"/homeless-shelter"[..]);
        assert_eq!(*env_map.get("TMPDIR").unwrap(), &b"/build"[..]);
    }

    #[test]
    fn build_request_has_system_constraint() {
        let drv = test_derivation();
        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store").unwrap();

        assert!(req
            .constraints
            .contains(&BuildConstraints::System("x86_64-linux".to_string())));
        assert!(req.constraints.contains(&BuildConstraints::ProvideBinSh));
    }

    #[test]
    fn build_request_outputs_are_relative() {
        let drv = test_derivation();
        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store").unwrap();

        for output in &req.outputs {
            assert!(
                !output.starts_with("/"),
                "output path must be relative: {output:?}"
            );
            assert!(
                output.starts_with("nix/store"),
                "output must be under nix/store: {output:?}"
            );
        }
    }

    #[test]
    fn build_request_has_refscan_needles() {
        let drv = test_derivation();
        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store").unwrap();

        // At least one needle for the output
        assert!(!req.refscan_needles.is_empty());
        // Needles are nixbase32 encoded (32 chars)
        for needle in &req.refscan_needles {
            assert_eq!(needle.len(), 32, "needle should be 32 chars: {needle}");
        }
    }

    // ── Phase 1: replace_placeholders tests ────────────────────

    #[test]
    fn replace_placeholders_substitutes_output_path() {
        let drv = test_derivation();
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let placeholder = hash_placeholder("out");
        let input = format!("echo hello > {placeholder}");

        let result = replace_placeholders(&input, &drv.outputs);
        assert!(!result.contains(&placeholder), "placeholder should be gone");
        assert!(
            result.contains(&out_path.to_absolute_path()),
            "should contain output path: {result}"
        );
    }

    #[test]
    fn replace_placeholders_noop_without_placeholder() {
        let drv = test_derivation();
        let input = "echo hello world";
        let result = replace_placeholders(input, &drv.outputs);
        assert_eq!(result, input);
    }

    #[test]
    fn replace_placeholders_multi_output() {
        // Build a fresh derivation with both outputs before computing paths
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output { path: None, ca_hash: None });
        outputs.insert("dev".to_string(), Output { path: None, ca_hash: None });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), "test".into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());
        environment.insert("dev".to_string(), "".into());
        let mut drv = Derivation {
            arguments: vec!["-c".into(), "echo > $out".into()],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };
        let hdm = drv.hash_derivation_modulo(|_| panic!("no parent"));
        drv.calculate_output_paths("test", &hdm).unwrap();

        let ph_out = hash_placeholder("out");
        let ph_dev = hash_placeholder("dev");
        let input = format!("install -D {ph_out}/bin/x {ph_dev}/include/x.h");

        let result = replace_placeholders(&input, &drv.outputs);
        assert!(!result.contains(&ph_out));
        assert!(!result.contains(&ph_dev));
        let out_path = drv.outputs["out"].path.as_ref().unwrap().to_absolute_path();
        let dev_path = drv.outputs["dev"].path.as_ref().unwrap().to_absolute_path();
        assert!(result.contains(&out_path));
        assert!(result.contains(&dev_path));
    }

    #[test]
    fn replace_placeholders_bstr_matches_string_variant() {
        let drv = test_derivation();
        let placeholder = hash_placeholder("out");
        let input = format!("echo > {placeholder}");

        let str_result = replace_placeholders(&input, &drv.outputs);
        let bstr_result = replace_placeholders_bstr(&BString::from(input.as_bytes()), &drv.outputs);
        assert_eq!(str_result.as_bytes(), bstr_result.as_ref() as &[u8]);
    }

    // ── Phase 1: collect_input_paths tests ────────────────────

    #[test]
    fn collect_inputs_source_only() {
        let mut drv = test_derivation();
        let source: StorePath<String> = StorePath::from_absolute_path(
            "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash-5.2".as_bytes()
        ).unwrap();
        drv.input_sources.insert(source.clone());

        let kp = KnownPaths::default();
        let paths = collect_input_paths(&drv, &kp).unwrap();
        assert!(paths.contains(&source));
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn collect_inputs_derivation_only() {
        let mut kp = KnownPaths::default();
        let (dep_drv_path, dep_drv) = register_drv("dep", &mut kp);

        let mut parent = test_derivation();
        let mut dep_outputs = BTreeSet::new();
        dep_outputs.insert("out".to_string());
        parent.input_derivations.insert(dep_drv_path.clone(), dep_outputs);

        let paths = collect_input_paths(&parent, &kp).unwrap();
        let dep_out = dep_drv.outputs["out"].path.as_ref().unwrap();
        assert!(paths.contains(dep_out), "should contain dep's output path");
    }

    #[test]
    fn collect_inputs_mixed() {
        let mut kp = KnownPaths::default();
        let (dep_drv_path, dep_drv) = register_drv("mixdep", &mut kp);

        let source: StorePath<String> = StorePath::from_absolute_path(
            "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-src".as_bytes()
        ).unwrap();

        let mut parent = test_derivation();
        parent.input_sources.insert(source.clone());
        let mut dep_outputs = BTreeSet::new();
        dep_outputs.insert("out".to_string());
        parent.input_derivations.insert(dep_drv_path, dep_outputs);

        let paths = collect_input_paths(&parent, &kp).unwrap();
        assert!(paths.contains(&source));
        let dep_out = dep_drv.outputs["out"].path.as_ref().unwrap();
        assert!(paths.contains(dep_out));
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn collect_inputs_missing_drv_returns_error() {
        let kp = KnownPaths::default();
        let mut parent = test_derivation();
        let fake_drv: StorePath<String> = StorePath::from_absolute_path(
            "/nix/store/cccccccccccccccccccccccccccccccc-missing.drv".as_bytes()
        ).unwrap();
        let mut outputs = BTreeSet::new();
        outputs.insert("out".to_string());
        parent.input_derivations.insert(fake_drv, outputs);

        let err = collect_input_paths(&parent, &kp).unwrap_err();
        assert!(matches!(err, crate::Error::DerivationNotFound { .. }));
    }
}
