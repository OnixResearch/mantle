use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use crunch_glue::Input;
use crunch_glue::ResolvedDerivationRef;

use crate::Error;
use crate::map_eval_error;

const DERIVATION_FILE_DEPTH_MAX: u32 = 128;
const DERIVATION_FILE_COUNT_MAX: u32 = 4_096;
const DERIVATION_FILE_PATH_BYTES_MAX: usize = 4_096;
const DERIVATION_FILE_IMPORT_ROOT_COUNT_MAX: usize = 256;
const DERIVATION_FILE_EXTENSION: &str = "ncl";
const SINGLE_FILE_ROOT_COUNT: usize = 1;

pub(crate) struct DerivationFileResolver {
    root_dir: PathBuf,
    import_authority_catalog: Vec<PathBuf>,
    import_paths: Vec<OsString>,
    resolved: BTreeMap<PathBuf, ResolvedDerivationRef>,
    visiting: BTreeSet<PathBuf>,
    resolved_file_count: u32,
}

impl DerivationFileResolver {
    pub(crate) fn new(root_file: &Path, import_paths: &[OsString]) -> Result<Self, Error> {
        let root_parent = root_file
            .parent()
            .ok_or_else(|| Error::Eval(format!("derivation-file root has no parent: {}", root_file.display())))?;
        let root_dir = root_parent.canonicalize().map_err(|error| {
            Error::Eval(format!("canonicalizing derivation-file root {}: {error}", root_parent.display()))
        })?;
        if !root_dir.is_dir() {
            return Err(Error::Eval(format!("derivation-file root is not a directory: {}", root_dir.display())));
        }
        const { assert!(DERIVATION_FILE_COUNT_MAX > 1) };
        debug_assert!(!root_dir.as_os_str().is_empty());
        let resolver = Self {
            root_dir,
            import_authority_catalog: canonical_import_roots(import_paths)?,
            import_paths: import_paths.to_vec(),
            resolved: BTreeMap::new(),
            visiting: BTreeSet::new(),
            resolved_file_count: 0,
        };
        assert!(resolver.import_authority_catalog.len() <= DERIVATION_FILE_IMPORT_ROOT_COUNT_MAX);
        Ok(resolver)
    }

    pub(crate) fn resolve_root_inputs(
        &mut self,
        root_file: &Path,
        derivation: &mut CrunchDerivation,
        cache: &mut ConversionCache,
    ) -> Result<(), Error> {
        let canonical_root = root_file
            .canonicalize()
            .map_err(|error| Error::Eval(format!("canonicalizing root derivation {}: {error}", root_file.display())))?;
        self.resolve_inline_inputs(&canonical_root, derivation, cache, 0)?;
        assert!(self.resolved_file_count <= DERIVATION_FILE_COUNT_MAX);
        debug_assert!(self.visiting.is_empty());
        Ok(())
    }

    fn resolve_inline_inputs(
        &mut self,
        owner_file: &Path,
        derivation: &mut CrunchDerivation,
        cache: &mut ConversionCache,
        depth: u32,
    ) -> Result<(), Error> {
        ensure_depth(depth)?;
        let inputs = std::mem::take(&mut derivation.inputs);
        let mut resolved_inputs = Vec::with_capacity(inputs.len());
        for input in inputs {
            resolved_inputs.push(self.resolve_input(owner_file, input, cache, depth)?);
        }
        derivation.inputs = resolved_inputs;
        assert!(derivation.inputs.len() <= derivation.inputs.capacity());
        debug_assert!(derivation.inputs.iter().all(|input| !matches!(input, Input::DerivationFile(_))));
        Ok(())
    }

    fn resolve_input(
        &mut self,
        owner_file: &Path,
        input: Input,
        cache: &mut ConversionCache,
        depth: u32,
    ) -> Result<Input, Error> {
        match input {
            Input::DerivationFile(reference) => {
                let mut resolved = self.resolve_file(owner_file, &reference.path, cache, depth.saturating_add(1))?;
                select_requested_output(&reference.path, reference.output.as_deref(), &mut resolved)?;
                Ok(Input::ResolvedDerivation(resolved))
            }
            Input::Derivation(mut nested) => {
                self.resolve_inline_inputs(owner_file, &mut nested, cache, depth.saturating_add(1))?;
                Ok(Input::Derivation(nested))
            }
            Input::OutputSelection(mut selection) => {
                self.resolve_inline_inputs(owner_file, &mut selection.drv, cache, depth.saturating_add(1))?;
                Ok(Input::OutputSelection(selection))
            }
            Input::PlanOutput(mut reference) => {
                self.resolve_inline_inputs(owner_file, &mut reference.producer, cache, depth.saturating_add(1))?;
                Ok(Input::PlanOutput(reference))
            }
            Input::Source(_) | Input::ResolvedDerivation(_) => Ok(input),
        }
    }

    fn resolve_file(
        &mut self,
        owner_file: &Path,
        reference: &str,
        cache: &mut ConversionCache,
        depth: u32,
    ) -> Result<ResolvedDerivationRef, Error> {
        ensure_depth(depth)?;
        let path = resolve_reference_path(&self.root_dir, &self.import_authority_catalog, owner_file, reference)?;
        if let Some(resolved) = self.resolved.get(&path) {
            return Ok(resolved.clone());
        }
        ensure_file_capacity(self.resolved_file_count)?;
        if !self.visiting.insert(path.clone()) {
            return Err(Error::Eval(format!("derivation-file cycle detected at {}", path.display())));
        }
        self.resolved_file_count = self.resolved_file_count.saturating_add(1);
        tracing::info!(
            path = %path.display(),
            depth,
            resolved_file_count = self.resolved_file_count,
            "resolving lazy derivation-file input"
        );
        let result = self.load_resolve_and_convert_file(&path, cache, depth);
        let was_removed = self.visiting.remove(&path);
        assert!(was_removed, "visited derivation file must be removed explicitly");
        let resolved = result?;
        tracing::info!(
            path = %path.display(),
            drv_path = %resolved.drv_path,
            "resolved lazy derivation-file input"
        );
        self.resolved.insert(path, resolved.clone());
        debug_assert!(self.resolved_file_count <= DERIVATION_FILE_COUNT_MAX);
        Ok(resolved)
    }

    fn load_resolve_and_convert_file(
        &mut self,
        path: &Path,
        cache: &mut ConversionCache,
        depth: u32,
    ) -> Result<ResolvedDerivationRef, Error> {
        let mut roots = crunch_eval::evaluate_and_extract_named_roots::<CrunchDerivation>(path, &self.import_paths)
            .map_err(map_eval_error)?;
        if roots.len() != SINGLE_FILE_ROOT_COUNT {
            return Err(Error::Deserialize(format!(
                "derivation-file input {} must evaluate to exactly one derivation root, observed {}",
                path.display(),
                roots.len()
            )));
        }
        let (_, mut derivation) = roots.pop().ok_or_else(|| {
            Error::Deserialize(format!("derivation-file input {} returned no derivation", path.display()))
        })?;
        self.resolve_inline_inputs(path, &mut derivation, cache, depth.saturating_add(1))?;
        let outputs = derivation.outputs.clone();
        let (drv_path, _) = crunch_glue::convert(&derivation, cache)
            .map_err(|error| Error::Convert(format!("derivation-file {}: {error}", path.display())))?;
        let drv_path = drv_path.to_absolute_path_with_prefix(cache.store_dir());
        assert!(!outputs.is_empty());
        debug_assert!(drv_path.starts_with('/'));
        Ok(ResolvedDerivationRef { drv_path, outputs })
    }
}

fn canonical_import_roots(import_paths: &[OsString]) -> Result<Vec<PathBuf>, Error> {
    if import_paths.len() > DERIVATION_FILE_IMPORT_ROOT_COUNT_MAX {
        return Err(Error::Eval(format!(
            "derivation-file import root count exceeds bounded maximum {DERIVATION_FILE_IMPORT_ROOT_COUNT_MAX}"
        )));
    }
    let mut roots = BTreeSet::new();
    for import_path in import_paths {
        let path = PathBuf::from(import_path);
        if path.is_dir() {
            let canonical = path.canonicalize().map_err(|error| {
                Error::Eval(format!("canonicalizing derivation-file import root {}: {error}", path.display()))
            })?;
            roots.insert(canonical);
        }
    }
    assert!(roots.len() <= DERIVATION_FILE_IMPORT_ROOT_COUNT_MAX);
    Ok(roots.into_iter().collect())
}

fn resolve_reference_path(
    root_dir: &Path,
    import_authority_catalog: &[PathBuf],
    owner_file: &Path,
    reference: &str,
) -> Result<PathBuf, Error> {
    assert!(!root_dir.as_os_str().is_empty());
    assert!(!owner_file.as_os_str().is_empty());
    validate_reference_text(reference)?;
    let owner_dir = owner_file
        .parent()
        .ok_or_else(|| Error::Eval(format!("derivation-file owner has no parent: {}", owner_file.display())))?;
    let owner_root = admitted_owner_root(root_dir, import_authority_catalog, owner_dir)?;
    let mut candidates = vec![(owner_root, owner_dir.join(reference))];
    candidates.extend(import_authority_catalog.iter().map(|root| (root.as_path(), root.join(reference))));
    for (admitted_root, candidate) in candidates {
        match resolve_admitted_candidate(admitted_root, &candidate)? {
            Some(canonical) => return Ok(canonical),
            None => continue,
        }
    }
    Err(Error::Eval(format!(
        "resolving derivation-file input {} from owner {} and {} import roots: no such file",
        reference,
        owner_file.display(),
        import_authority_catalog.len()
    )))
}

fn admitted_owner_root<'a>(
    root_dir: &'a Path,
    import_authority_catalog: &'a [PathBuf],
    owner_dir: &Path,
) -> Result<&'a Path, Error> {
    if owner_dir.starts_with(root_dir) {
        return Ok(root_dir);
    }
    import_authority_catalog
        .iter()
        .find(|root| owner_dir.starts_with(root))
        .map(PathBuf::as_path)
        .ok_or_else(|| Error::Eval(format!("derivation-file owner escapes admitted roots: {}", owner_dir.display())))
}

fn resolve_admitted_candidate(admitted_root: &Path, candidate: &Path) -> Result<Option<PathBuf>, Error> {
    let canonical = match candidate.canonicalize() {
        Ok(canonical) => canonical,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(Error::Eval(format!("resolving derivation-file input {}: {error}", candidate.display())));
        }
    };
    if !canonical.starts_with(admitted_root) {
        return Err(Error::Eval(format!(
            "derivation-file input escapes admitted root {}: {}",
            admitted_root.display(),
            canonical.display()
        )));
    }
    if !canonical.is_file() {
        return Err(Error::Eval(format!("derivation-file input is not a file: {}", canonical.display())));
    }
    assert!(canonical.starts_with(admitted_root));
    debug_assert_eq!(canonical.extension().and_then(|extension| extension.to_str()), Some(DERIVATION_FILE_EXTENSION));
    Ok(Some(canonical))
}

fn validate_reference_text(reference: &str) -> Result<(), Error> {
    if reference.is_empty() {
        return Err(invalid_reference_error(reference));
    }
    if reference.len() > DERIVATION_FILE_PATH_BYTES_MAX {
        return Err(invalid_reference_error(reference));
    }
    let path = Path::new(reference);
    if path.is_absolute() {
        return Err(invalid_reference_error(reference));
    }
    let has_normal_components = path.components().all(|component| matches!(component, Component::Normal(_)));
    if !has_normal_components {
        return Err(invalid_reference_error(reference));
    }
    let has_expected_extension =
        path.extension().and_then(|extension| extension.to_str()) == Some(DERIVATION_FILE_EXTENSION);
    if !has_expected_extension {
        return Err(invalid_reference_error(reference));
    }
    assert!(!reference.is_empty());
    debug_assert!(reference.len() <= DERIVATION_FILE_PATH_BYTES_MAX);
    Ok(())
}

fn invalid_reference_error(reference: &str) -> Error {
    Error::Eval(format!("derivation-file input must be a bounded normalized relative .ncl path: {reference}"))
}

fn select_requested_output(
    reference_path: &str,
    requested_output: Option<&str>,
    resolved: &mut ResolvedDerivationRef,
) -> Result<(), Error> {
    let Some(requested_output) = requested_output else {
        return Ok(());
    };
    if requested_output.is_empty() || !resolved.outputs.iter().any(|output| output == requested_output) {
        return Err(Error::Deserialize(format!(
            "derivation-file input {reference_path} selects unavailable output {requested_output}; available: {}",
            resolved.outputs.join(",")
        )));
    }
    resolved.outputs = vec![requested_output.to_string()];
    assert_eq!(resolved.outputs.len(), 1);
    debug_assert_eq!(resolved.outputs.first().map(String::as_str), Some(requested_output));
    Ok(())
}

fn ensure_depth(depth: u32) -> Result<(), Error> {
    if depth > DERIVATION_FILE_DEPTH_MAX {
        return Err(Error::Eval(format!("derivation-file depth exceeds bounded maximum {DERIVATION_FILE_DEPTH_MAX}")));
    }
    const { assert!(DERIVATION_FILE_DEPTH_MAX > 1) };
    debug_assert!(depth <= DERIVATION_FILE_DEPTH_MAX);
    Ok(())
}

fn ensure_file_capacity(file_count: u32) -> Result<(), Error> {
    if file_count >= DERIVATION_FILE_COUNT_MAX {
        return Err(Error::Eval(format!("derivation-file count exceeds bounded maximum {DERIVATION_FILE_COUNT_MAX}")));
    }
    const { assert!(DERIVATION_FILE_COUNT_MAX > 1) };
    debug_assert!(file_count < DERIVATION_FILE_COUNT_MAX);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_PENDING_DERIVATION_COUNT: usize = 2;

    fn write_fixture(path: &Path, text: &str) {
        std::fs::write(path, text).unwrap();
        assert!(path.is_file());
        assert!(!text.is_empty());
    }

    fn assert_stagex_cutover(pending_paths: &[String]) {
        let forbidden_legacy_names = [
            "mes",
            "tinycc-0.9.26",
            "tinycc-0.9.27",
            "patch-2.5.9-tcc",
            "gzip-1.2.4-tcc",
            "tar-1.12-tcc",
            "sed-4.0.9-tcc",
            "tcc-0.9.27-musl-prep",
            "bzip2-1.0.8-tcc",
            "coreutils-5.0-tcc",
            "oyacc-6.6-tcc",
            "bash-2.05b-tcc",
            "musl-1.1.24-tcc",
            "tcc-0.9.27-musl",
            "musl-1.1.24-tcc-musl",
            "tcc-0.9.27-musl-v2",
            "tcc-0.9.27-musl-selfhost",
            "tcc-0.9.27-musl-native-runtime",
            "musl-1.1.24-native-candidate",
            "sed-4.0.9-musl",
            "m4-1.4.7-musl",
            "grep-2.4-musl",
            "diffutils-2.7-musl",
            "gawk-3.0.4-musl",
            "bison-2.3-musl",
            "flex-2.6.4-musl",
            "binutils-2.30-tcc-source-v1",
        ];
        assert!(pending_paths.iter().any(|path| path.ends_with("-stage0-posix.drv")));
        assert!(pending_paths.iter().any(|path| path.ends_with("-make-3.82-tcc.drv")));
        for forbidden_name in forbidden_legacy_names {
            let forbidden_suffix = format!("-{forbidden_name}.drv");
            assert!(
                !pending_paths.iter().any(|path| path.ends_with(&forbidden_suffix)),
                "post-StageX graph retained forbidden legacy derivation {forbidden_name}: {pending_paths:?}"
            );
        }
    }

    #[test]
    fn reference_text_accepts_normalized_relative_nickel_path() {
        assert!(validate_reference_text("dep.ncl").is_ok());
        assert!(validate_reference_text("nested/dep.ncl").is_ok());
    }

    #[test]
    fn reference_text_rejects_absolute_parent_and_non_nickel_paths() {
        for invalid in ["/tmp/dep.ncl", "../dep.ncl", "./dep.ncl", "dep.json", ""] {
            let error = validate_reference_text(invalid).unwrap_err().to_string();
            assert!(error.contains("normalized relative .ncl path"));
            assert!(error.contains(invalid));
        }
    }

    #[test]
    fn resolver_converts_lazy_file_edge_without_embedding_dependency() {
        const EXPECTED_INPUT_DERIVATION_COUNT: usize = 1;
        let dir = tempfile::tempdir().unwrap();
        let dep = dir.path().join("dep.ncl");
        let root = dir.path().join("root.ncl");
        write_fixture(&dep, r#"{ name = "dep", builder = "/bin/sh" }"#);
        write_fixture(&root, r#"{ name = "root", builder = "/bin/sh", inputs = [{ derivation_file = "dep.ncl" }] }"#);
        let mut roots = crunch_eval::evaluate_and_extract_named_roots::<CrunchDerivation>(&root, &[]).unwrap();
        let (_, mut derivation) = roots.pop().unwrap();
        let mut cache = ConversionCache::default();
        let mut resolver = DerivationFileResolver::new(&root, &[]).unwrap();
        resolver.resolve_root_inputs(&root, &mut derivation, &mut cache).unwrap();
        assert!(matches!(derivation.inputs.as_slice(), [Input::ResolvedDerivation(_)]));
        let (_, converted) = crunch_glue::convert(&derivation, &mut cache).unwrap();
        assert_eq!(converted.input_derivations.len(), EXPECTED_INPUT_DERIVATION_COUNT);
        assert_eq!(cache.drain_pending().len(), EXPECTED_PENDING_DERIVATION_COUNT);
    }

    #[test]
    fn full_source_linux_headers_preserve_all_derivation_file_edges() {
        const EXPECTED_INPUT_DERIVATION_COUNT: usize = 3;
        const STORE_PREFIX: &str = "/mantle/store";
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
        let root = workspace.join("bootstrap/linux-headers-6.6-gcc10.ncl");
        let import_paths = vec![
            workspace.join("lib").into_os_string(),
            workspace.join("bootstrap").into_os_string(),
        ];
        let mut roots =
            crunch_eval::evaluate_and_extract_named_roots::<CrunchDerivation>(&root, &import_paths).unwrap();
        assert_eq!(roots.len(), SINGLE_FILE_ROOT_COUNT);
        let (_, mut derivation) = roots.pop().unwrap();
        let mut cache = ConversionCache::new(STORE_PREFIX);
        let mut resolver = DerivationFileResolver::new(&root, &import_paths).unwrap();
        resolver.resolve_root_inputs(&root, &mut derivation, &mut cache).unwrap();

        let (_, converted) = crunch_glue::convert(&derivation, &mut cache).unwrap();
        let dependency_names = converted.input_derivations.keys().map(ToString::to_string).collect::<Vec<_>>();

        assert_eq!(converted.input_derivations.len(), EXPECTED_INPUT_DERIVATION_COUNT);
        for required_name in [
            "full-source-seed-toolchain",
            "make-4.4.1-full-source-gcc10-v1",
            "linux-6.6-src",
        ] {
            assert!(
                dependency_names.iter().any(|name| name.contains(required_name)),
                "converted Linux-header dependencies: {dependency_names:?}"
            );
        }
        assert!(!dependency_names.iter().any(|name| name.contains("linux-headers-6.6-full-source-gcc10-v3")));
    }

    #[test]
    fn full_source_graph_cuts_legacy_edges_at_stagex_sources() {
        const PENDING_DERIVATION_COUNT_MIN: usize = 50;
        const STORE_PREFIX: &str = "/mantle/store";
        const STAGEX_PROVIDER: &str =
            "/mantle/store/snzd91n8dv6l21xa89vml67229n9svkg-mantle-stagex-intermediate-provider";
        const STAGEX_TRANSITION: &str = "/mantle/store/ki5gkg5d6si77dl5k4mav4s6x9s8l25r-mantle-stagex-transition";
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
        let root = workspace.join("bootstrap/seed-full-toolchain.ncl");
        let import_paths = vec![
            workspace.join("lib").into_os_string(),
            workspace.join("bootstrap").into_os_string(),
        ];
        let mut roots =
            crunch_eval::evaluate_and_extract_named_roots::<CrunchDerivation>(&root, &import_paths).unwrap();
        assert_eq!(roots.len(), SINGLE_FILE_ROOT_COUNT);
        let (_, mut derivation) = roots.pop().unwrap();
        let mut cache = ConversionCache::new(STORE_PREFIX);
        let mut resolver = DerivationFileResolver::new(&root, &import_paths).unwrap();
        resolver.resolve_root_inputs(&root, &mut derivation, &mut cache).unwrap();
        crunch_glue::convert(&derivation, &mut cache).unwrap();
        let pending = cache.drain_pending();
        let pending_paths = pending.iter().map(|entry| entry.0.to_string()).collect::<Vec<_>>();
        let gcc40 = pending
            .iter()
            .find(|entry| entry.0.to_string().contains("gcc-4.0.4-native-gas-v45"))
            .expect("resolved GCC 4.0 native derivation");
        let input_sources = gcc40
            .2
            .input_sources
            .iter()
            .map(|source| source.to_absolute_path_with_prefix(STORE_PREFIX))
            .collect::<Vec<_>>();

        assert!(pending.len() >= PENDING_DERIVATION_COUNT_MIN);
        assert_stagex_cutover(&pending_paths);
        assert!(
            input_sources.iter().any(|source| source == STAGEX_PROVIDER),
            "GCC 4.0 input sources: {input_sources:?}"
        );
        assert!(
            input_sources.iter().any(|source| source == STAGEX_TRANSITION),
            "GCC 4.0 input sources: {input_sources:?}"
        );
    }

    #[test]
    fn resolver_falls_back_to_explicit_import_root() {
        let root_dir = tempfile::tempdir().unwrap();
        let import_dir = tempfile::tempdir().unwrap();
        let root = root_dir.path().join("root.ncl");
        let dep = import_dir.path().join("dep.ncl");
        write_fixture(&root, r#"{ name = "root", builder = "/bin/sh" }"#);
        write_fixture(&dep, r#"{ name = "dep", builder = "/bin/sh" }"#);
        let import_paths = vec![import_dir.path().as_os_str().to_owned()];
        let resolver = DerivationFileResolver::new(&root, &import_paths).unwrap();
        let resolved =
            resolve_reference_path(&resolver.root_dir, &resolver.import_authority_catalog, &root, "dep.ncl").unwrap();
        assert_eq!(resolved, dep.canonicalize().unwrap());
        assert!(resolved.starts_with(import_dir.path()));
    }

    #[test]
    fn resolver_preserves_imported_derivation_file_authority() {
        let root_dir = tempfile::tempdir().unwrap();
        let import_dir = tempfile::tempdir().unwrap();
        let root = root_dir.path().join("root.ncl");
        let seed = import_dir.path().join("seed.ncl");
        let dep = import_dir.path().join("dep.ncl");
        write_fixture(&root, r#"let seed = import "seed.ncl" in seed.toolchain"#);
        write_fixture(
            &seed,
            r#"{ toolchain = { name = "toolchain", builder = "/bin/sh", inputs = [{ derivation_file = "dep.ncl" }] } }"#,
        );
        write_fixture(&dep, r#"{ name = "dep", builder = "/bin/sh" }"#);
        let import_paths = vec![import_dir.path().as_os_str().to_owned()];
        let mut roots =
            crunch_eval::evaluate_and_extract_named_roots::<CrunchDerivation>(&root, &import_paths).unwrap();
        let (_, mut derivation) = roots.pop().unwrap();
        let mut cache = ConversionCache::default();
        let mut resolver = DerivationFileResolver::new(&root, &import_paths).unwrap();
        resolver.resolve_root_inputs(&root, &mut derivation, &mut cache).unwrap();
        assert!(matches!(derivation.inputs.as_slice(), [Input::ResolvedDerivation(_)]));
        assert_eq!(cache.drain_pending().len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn resolver_rejects_import_root_symlink_escape() {
        use std::os::unix::fs::symlink;

        let root_dir = tempfile::tempdir().unwrap();
        let import_dir = tempfile::tempdir().unwrap();
        let outside_dir = tempfile::tempdir().unwrap();
        let root = root_dir.path().join("root.ncl");
        let outside = outside_dir.path().join("dep.ncl");
        write_fixture(&root, r#"{ name = "root", builder = "/bin/sh" }"#);
        write_fixture(&outside, r#"{ name = "outside", builder = "/bin/sh" }"#);
        symlink(&outside, import_dir.path().join("dep.ncl")).unwrap();
        let import_paths = vec![import_dir.path().as_os_str().to_owned()];
        let resolver = DerivationFileResolver::new(&root, &import_paths).unwrap();
        let error = resolve_reference_path(&resolver.root_dir, &resolver.import_authority_catalog, &root, "dep.ncl")
            .unwrap_err();
        assert!(error.to_string().contains("escapes admitted root"));
        assert!(error.to_string().contains(&outside.display().to_string()));
    }

    #[test]
    fn selected_output_accepts_available_and_rejects_missing_name() {
        let mut resolved = ResolvedDerivationRef {
            drv_path: "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-dep.drv".to_string(),
            outputs: vec!["out".to_string(), "dev".to_string()],
        };
        select_requested_output("dep.ncl", Some("dev"), &mut resolved).unwrap();
        assert_eq!(resolved.outputs, vec!["dev"]);

        let error = select_requested_output("dep.ncl", Some("missing"), &mut resolved).unwrap_err();
        assert!(error.to_string().contains("unavailable output missing"));
        assert!(error.to_string().contains("available: dev"));
    }

    #[test]
    fn resolver_rejects_file_cycle() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("first.ncl");
        let second = dir.path().join("second.ncl");
        write_fixture(
            &first,
            r#"{ name = "first", builder = "/bin/sh", inputs = [{ derivation_file = "second.ncl" }] }"#,
        );
        write_fixture(
            &second,
            r#"{ name = "second", builder = "/bin/sh", inputs = [{ derivation_file = "first.ncl" }] }"#,
        );
        let mut roots = crunch_eval::evaluate_and_extract_named_roots::<CrunchDerivation>(&first, &[]).unwrap();
        let (_, mut derivation) = roots.pop().unwrap();
        let mut cache = ConversionCache::default();
        let mut resolver = DerivationFileResolver::new(&first, &[]).unwrap();
        let error = resolver.resolve_root_inputs(&first, &mut derivation, &mut cache).unwrap_err();
        assert!(error.to_string().contains("cycle detected"));
        assert!(resolver.visiting.is_empty());
    }
}
