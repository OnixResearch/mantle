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
const DERIVATION_FILE_EXTENSION: &str = "ncl";
const SINGLE_FILE_ROOT_COUNT: usize = 1;

pub(crate) struct DerivationFileResolver {
    root_dir: PathBuf,
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
        assert!(DERIVATION_FILE_COUNT_MAX > 1);
        debug_assert!(!root_dir.as_os_str().is_empty());
        Ok(Self {
            root_dir,
            import_paths: import_paths.to_vec(),
            resolved: BTreeMap::new(),
            visiting: BTreeSet::new(),
            resolved_file_count: 0,
        })
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
        let path = resolve_reference_path(&self.root_dir, owner_file, reference)?;
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
        let removed = self.visiting.remove(&path);
        assert!(removed, "visited derivation file must be removed explicitly");
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

fn resolve_reference_path(root_dir: &Path, owner_file: &Path, reference: &str) -> Result<PathBuf, Error> {
    validate_reference_text(reference)?;
    let owner_dir = owner_file
        .parent()
        .ok_or_else(|| Error::Eval(format!("derivation-file owner has no parent: {}", owner_file.display())))?;
    let candidate = owner_dir.join(reference);
    let canonical = candidate
        .canonicalize()
        .map_err(|error| Error::Eval(format!("resolving derivation-file input {}: {error}", candidate.display())))?;
    if !canonical.starts_with(root_dir) {
        return Err(Error::Eval(format!(
            "derivation-file input escapes root {}: {}",
            root_dir.display(),
            canonical.display()
        )));
    }
    if !canonical.is_file() {
        return Err(Error::Eval(format!("derivation-file input is not a file: {}", canonical.display())));
    }
    assert!(canonical.starts_with(root_dir));
    debug_assert_eq!(canonical.extension().and_then(|extension| extension.to_str()), Some(DERIVATION_FILE_EXTENSION));
    Ok(canonical)
}

fn validate_reference_text(reference: &str) -> Result<(), Error> {
    let path = Path::new(reference);
    let normalized_components = path.components().all(|component| matches!(component, Component::Normal(_)));
    let expected_extension =
        path.extension().and_then(|extension| extension.to_str()) == Some(DERIVATION_FILE_EXTENSION);
    if reference.is_empty()
        || reference.len() > DERIVATION_FILE_PATH_BYTES_MAX
        || path.is_absolute()
        || !normalized_components
        || !expected_extension
    {
        return Err(Error::Eval(format!(
            "derivation-file input must be a bounded normalized relative .ncl path: {reference}"
        )));
    }
    assert!(!reference.is_empty());
    debug_assert!(reference.len() <= DERIVATION_FILE_PATH_BYTES_MAX);
    Ok(())
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
    assert!(DERIVATION_FILE_DEPTH_MAX > 1);
    debug_assert!(depth <= DERIVATION_FILE_DEPTH_MAX);
    Ok(())
}

fn ensure_file_capacity(file_count: u32) -> Result<(), Error> {
    if file_count >= DERIVATION_FILE_COUNT_MAX {
        return Err(Error::Eval(format!("derivation-file count exceeds bounded maximum {DERIVATION_FILE_COUNT_MAX}")));
    }
    assert!(DERIVATION_FILE_COUNT_MAX > 1);
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
        crunch_glue::convert(&derivation, &mut cache).unwrap();
        assert_eq!(cache.drain_pending().len(), EXPECTED_PENDING_DERIVATION_COUNT);
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
