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
pub fn derivation_to_build_request(
    derivation: &Derivation,
    inputs: &BTreeMap<StorePath<String>, Node>,
) -> Result<BuildRequest, crate::Error> {
    // command_args = [builder] ++ arguments, with placeholders replaced
    let mut command_args: Vec<String> = Vec::with_capacity(derivation.arguments.len() + 1);
    command_args.push(derivation.builder.clone());
    for arg in &derivation.arguments {
        command_args.push(replace_placeholders(arg, &derivation.outputs));
    }

    // Environment: start with sandbox defaults, then add derivation env
    let mut env: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for (k, v) in &SANDBOX_ENV_VARS {
        env.insert(k.to_string(), v.as_bytes().to_vec());
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

    Ok(BuildRequest {
        command_args,
        outputs: derivation
            .outputs
            .values()
            .map(|o| {
                let path_str = o.path_str();
                // Strip leading '/' — BuildRequest wants relative paths
                PathBuf::from(&path_str[1..])
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
        inputs_dir: nix_compat::store_path::STORE_DIR[1..].into(),
        constraints,
        working_dir: "build".into(),
        scratch_paths: vec!["build".into(), "nix/store".into()],
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
        let entry = known_paths
            .get_by_drv_path(&drv_path.to_absolute_path())
            .ok_or_else(|| crate::Error::DerivationNotFound {
                path: drv_path.clone(),
            })?;
        for output_name in output_names {
            let output = entry
                .derivation
                .outputs
                .get(output_name)
                .ok_or_else(|| crate::Error::OutputMissing {
                    output: output_name.clone(),
                })?;
            let output_path = output.path.as_ref().ok_or_else(|| {
                crate::Error::OutputNoPath {
                    output: output_name.clone(),
                    drv_name: drv_path.to_string(),
                }
            })?;
            paths.insert(output_path.clone());
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
        let req = derivation_to_build_request(&drv, &BTreeMap::new()).unwrap();

        assert_eq!(req.command_args[0], "/bin/sh");
        assert_eq!(req.command_args[1], "-c");
    }

    #[test]
    fn build_request_has_sandbox_env() {
        let drv = test_derivation();
        let req = derivation_to_build_request(&drv, &BTreeMap::new()).unwrap();

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
        let req = derivation_to_build_request(&drv, &BTreeMap::new()).unwrap();

        assert!(req
            .constraints
            .contains(&BuildConstraints::System("x86_64-linux".to_string())));
        assert!(req.constraints.contains(&BuildConstraints::ProvideBinSh));
    }

    #[test]
    fn build_request_outputs_are_relative() {
        let drv = test_derivation();
        let req = derivation_to_build_request(&drv, &BTreeMap::new()).unwrap();

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
        let req = derivation_to_build_request(&drv, &BTreeMap::new()).unwrap();

        // At least one needle for the output
        assert!(!req.refscan_needles.is_empty());
        // Needles are nixbase32 encoded (32 chars)
        for needle in &req.refscan_needles {
            assert_eq!(needle.len(), 32, "needle should be 32 chars: {needle}");
        }
    }
}
