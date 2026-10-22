//! Read-only native derivation preflight for proof-scoped action planning.
//!
//! This enumerates derivations, not the child processes of shell builders. A
//! caller must still lower every builder's command topology and generated-file
//! publications before constructing an action adapter. Neither a store path nor
//! this inventory grants executable authority or records execution evidence.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;

use crate::Error;
use crate::derivation_file::DerivationFileResolver;
use crate::map_eval_error;

const ROOT_FILES_MAX: usize = 64;
const DERIVATIONS_MAX: usize = 16_384;

/// One selected root, retaining the physical recipe separately from logical
/// derivation identity. The physical path is never executable authority.
pub struct NativeDerivationRoot {
    pub recipe: PathBuf,
    pub label: String,
    pub drv_path: StorePath<String>,
}

/// Exact converted recipe, including builder, arguments, environment, selected
/// input outputs, source inputs, and declared outputs. Generated child tools are
/// intentionally not inferred from an output directory.
pub struct NativePlannedDerivation {
    pub drv_path: StorePath<String>,
    pub hash_derivation_modulo: [u8; 32],
    pub derivation: Derivation,
    pub content_addressed: bool,
}

pub struct NativeDerivationClosure {
    pub roots: Vec<NativeDerivationRoot>,
    /// Canonical logical-path order; dependency edges remain in each recipe.
    pub derivations: Vec<NativePlannedDerivation>,
}

/// The mutable planning state for one native preflight closure.
struct NativeClosureState<'a> {
    store_prefix: &'a str,
    cache: ConversionCache,
    roots: Vec<NativeDerivationRoot>,
    derivations: BTreeMap<String, NativePlannedDerivation>,
    resolvers: BTreeMap<PathBuf, DerivationFileResolver>,
}

/// Evaluate the complete static derivation closure without opening a store,
/// starting a worker, fetching sources, or running any recipe. Use this before
/// the first proof action, with the same imported source tree and logical prefix
/// later used by execution. Dynamic-plan outputs are not a static closure.
pub fn plan_derivation_closure(
    files: &[PathBuf],
    import_paths: &[OsString],
    store_prefix: &str,
) -> Result<NativeDerivationClosure, Error> {
    if files.is_empty() || files.len() > ROOT_FILES_MAX {
        return Err(Error::Convert(format!("native preflight requires 1..={ROOT_FILES_MAX} root files")));
    }
    if !store_prefix.starts_with('/')
        || store_prefix[1..].split('/').any(|component| matches!(component, "" | "." | ".."))
    {
        return Err(Error::Convert("native preflight requires a normalized absolute store prefix".into()));
    }
    let mut state = NativeClosureState {
        store_prefix,
        cache: ConversionCache::new(store_prefix),
        roots: Vec::new(),
        derivations: BTreeMap::new(),
        resolvers: BTreeMap::new(),
    };
    for file in files {
        plan_recipe_roots(&mut state, file, import_paths)?;
    }
    validate_dependencies(&state.derivations, store_prefix)?;
    Ok(NativeDerivationClosure {
        roots: state.roots,
        derivations: state.derivations.into_values().collect(),
    })
}

/// Convert one recipe's selected roots and the derivations they introduce.
fn plan_recipe_roots(state: &mut NativeClosureState<'_>, file: &Path, import_paths: &[OsString]) -> Result<(), Error> {
    let recipe = file
        .canonicalize()
        .map_err(|error| Error::Eval(format!("canonicalizing native preflight recipe {}: {error}", file.display())))?;
    let selected = crunch_eval::evaluate_and_extract_named_roots::<CrunchDerivation>(&recipe, import_paths)
        .map_err(map_eval_error)?;
    if selected.is_empty() {
        return Err(Error::Convert(format!("native preflight recipe has no roots: {}", recipe.display())));
    }
    debug_assert!(!selected.is_empty(), "a planned recipe contributes at least one selected root");
    let directory = recipe
        .parent()
        .ok_or_else(|| Error::Eval(format!("native preflight recipe has no parent: {}", recipe.display())))?;
    let resolver = match state.resolvers.entry(directory.to_path_buf()) {
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(DerivationFileResolver::new(&recipe, import_paths)?)
        }
        std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
    };
    for (label, mut derivation) in selected {
        resolver.resolve_root_inputs(&recipe, &mut derivation, &mut state.cache)?;
        let (drv_path, _) = crunch_glue::convert(&derivation, &mut state.cache)
            .map_err(|error| Error::Convert(format!("native preflight {label}: {error}")))?;
        if state.roots.len() >= DERIVATIONS_MAX {
            return Err(Error::Convert("native preflight root count exceeds derivation bound".into()));
        }
        state.roots.push(NativeDerivationRoot {
            recipe: recipe.clone(),
            label,
            drv_path,
        });
        drain_pending_derivations(&mut state.cache, &mut state.derivations, state.store_prefix)?;
    }
    Ok(())
}

/// Move every derivation the conversion cache just produced into the closure.
fn drain_pending_derivations(
    cache: &mut ConversionCache,
    derivations: &mut BTreeMap<String, NativePlannedDerivation>,
    store_prefix: &str,
) -> Result<(), Error> {
    for (drv_path, hash_derivation_modulo, derivation, content_addressed, dynamic_outputs, _) in cache.drain_pending() {
        let key = drv_path.to_absolute_path_with_prefix(store_prefix);
        if !dynamic_outputs.is_empty() {
            return Err(Error::Convert(format!("native preflight cannot enumerate dynamic plan outputs: {key}")));
        }
        if derivations.len() >= DERIVATIONS_MAX {
            return Err(Error::Convert("native preflight derivation count exceeds bound".into()));
        }
        match derivations.entry(key) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(NativePlannedDerivation {
                    drv_path,
                    hash_derivation_modulo,
                    derivation,
                    content_addressed,
                });
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                return Err(Error::Convert(format!("native preflight duplicate derivation identity: {}", entry.key())));
            }
        }
    }
    Ok(())
}

fn validate_dependencies(
    derivations: &BTreeMap<String, NativePlannedDerivation>,
    store_prefix: &str,
) -> Result<(), Error> {
    for (key, planned) in derivations {
        for (dependency, outputs) in &planned.derivation.input_derivations {
            let dependency_key = dependency.to_absolute_path_with_prefix(store_prefix);
            let producer = derivations.get(&dependency_key).ok_or_else(|| {
                Error::Convert(format!("native preflight missing producer {dependency_key} required by {key}"))
            })?;
            for output in outputs {
                if !producer.derivation.outputs.contains_key(output) {
                    return Err(Error::Convert(format!(
                        "native preflight missing output {output} from {dependency_key} required by {key}"
                    )));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_and_host_tool_roots_share_the_same_planned_native_producer() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
        let files = [
            "seed-full-toolchain.ncl",
            "make-4.4.1-gcc10.ncl",
            "linux-headers-6.6-gcc10.ncl",
            "busybox-1.37.0-gcc10.ncl",
            "cmake-3.31.8-gcc10.ncl",
            "python-3.13.5-gcc10.ncl",
            "perl-5.10.1-gcc10.ncl",
        ]
        .map(|name| workspace.join("bootstrap").join(name));
        let imports = [
            workspace.join("lib").into_os_string(),
            workspace.join("bootstrap").into_os_string(),
        ];
        let closure = plan_derivation_closure(&files, &imports, "/mantle/store").unwrap();
        let provider = &closure.roots[0].drv_path;
        for root in &closure.roots[1..] {
            let host_tool = closure.derivations.iter().find(|node| node.drv_path == root.drv_path).unwrap();
            assert_eq!(
                host_tool.derivation.input_derivations.get(provider),
                Some(&std::collections::BTreeSet::from(["out".to_string()]))
            );
        }
        assert_eq!(closure.derivations.iter().filter(|node| &node.drv_path == provider).count(), 1);
    }

    #[test]
    fn enumerates_transitive_diamond_without_running_builders() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = |name: &str, text: &str| {
            let path = dir.path().join(name);
            std::fs::write(&path, text).unwrap();
            path
        };
        recipe("leaf.ncl", r#"{ name = "leaf", builder = "/no-such-native-preflight-builder" }"#);
        recipe(
            "left.ncl",
            r#"{ name = "left", builder = "/no-such-native-preflight-builder", inputs = [{ derivation_file = "leaf.ncl" }] }"#,
        );
        recipe(
            "right.ncl",
            r#"{ name = "right", builder = "/no-such-native-preflight-builder", inputs = [{ derivation_file = "leaf.ncl" }] }"#,
        );
        let root = recipe(
            "root.ncl",
            r#"{ name = "root", builder = "/no-such-native-preflight-builder", inputs = [{ derivation_file = "left.ncl" }, { derivation_file = "right.ncl" }] }"#,
        );
        let closure = plan_derivation_closure(&[root], &[], "/mantle/store").unwrap();
        let names = closure.derivations.iter().map(|node| node.drv_path.to_string()).collect::<Vec<_>>();
        assert_eq!(names.len(), 4);
        for name in ["leaf", "left", "right", "root"] {
            assert_eq!(names.iter().filter(|path| path.ends_with(&format!("-{name}.drv"))).count(), 1);
        }
        let root = closure.derivations.iter().find(|node| node.drv_path == closure.roots[0].drv_path).unwrap();
        assert_eq!(root.derivation.input_derivations.len(), 2);
        assert!(
            closure
                .derivations
                .iter()
                .all(|node| node.derivation.builder == "/no-such-native-preflight-builder")
        );
    }

    #[test]
    fn rejects_missing_dependency_and_escaping_reference_before_execution() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root.ncl");
        for reference in ["missing.ncl", "../outside.ncl"] {
            std::fs::write(
                &root,
                format!(
                    r#"{{ name = "root", builder = "/bin/false", inputs = [{{ derivation_file = "{reference}" }}] }}"#
                ),
            )
            .unwrap();
            assert!(plan_derivation_closure(std::slice::from_ref(&root), &[], "/mantle/store").is_err());
        }
    }

    #[test]
    fn rejects_unenumerated_dynamic_children_and_missing_selected_output() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root.ncl");
        std::fs::write(&root, r#"{ name = "dynamic", builder = "/bin/false", outputs = ["out", "plan"], dynamic_plan_outputs = ["plan"] }"#).unwrap();
        let error = plan_derivation_closure(std::slice::from_ref(&root), &[], "/mantle/store").err().unwrap();
        assert!(error.to_string().contains("dynamic plan outputs"));
        std::fs::write(dir.path().join("dep.ncl"), r#"{ name = "dep", builder = "/bin/false" }"#).unwrap();
        std::fs::write(&root, r#"{ name = "root", builder = "/bin/false", inputs = [{ derivation_file = "dep.ncl", output = "missing" }] }"#).unwrap();
        let error = plan_derivation_closure(&[root], &[], "/mantle/store").err().unwrap();
        assert!(error.to_string().contains("missing"));
    }

    #[test]
    fn changed_recipe_cannot_reuse_a_previous_derivation_identity() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root.ncl");
        let mut previous = None;
        for argument in ["first", "second"] {
            std::fs::write(&root, format!(r#"{{ name = "root", builder = "/bin/false", args = ["{argument}"] }}"#))
                .unwrap();
            let closure = plan_derivation_closure(std::slice::from_ref(&root), &[], "/mantle/store").unwrap();
            let node = closure.derivations.into_iter().next().unwrap();
            if let Some(previous) = previous {
                assert_ne!(node.hash_derivation_modulo, previous);
            }
            previous = Some(node.hash_derivation_modulo);
            assert_eq!(node.derivation.arguments, vec![argument]);
        }
    }

    #[test]
    fn rejects_invalid_root_and_store_authority() {
        assert!(plan_derivation_closure(&[], &[], "/mantle/store").is_err());
        for prefix in [
            "relative",
            "/mantle/store/",
            "/mantle/../store",
            "/mantle/./store",
            "/mantle//store",
        ] {
            assert!(plan_derivation_closure(&[PathBuf::from("not-opened.ncl")], &[], prefix).is_err());
        }
    }
}
