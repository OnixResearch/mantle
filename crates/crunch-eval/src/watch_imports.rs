//! Files whose changes can invalidate one file-backed Nickel evaluation.
//!
//! Walk the parsed import expressions, not text matches: comments and string
//! literals must not create phantom dependencies. Each candidate is selected
//! in Nickel's importer-directory-then-import-path order. This scan is a
//! watch hint, not evaluation authority: a failed scan never admits goals.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use nickel_lang_core::cache::CacheHub;
use nickel_lang_core::cache::InputFormat;
use nickel_lang_core::cache::SourcePath;
use nickel_lang_core::cache::normalize_path;
use nickel_lang_core::parser::ErrorTolerantParserCompat;
use nickel_lang_core::parser::grammar::TermParser;
use nickel_lang_core::parser::lexer::Lexer;
use nickel_lang_core::position::PosTable;
use nickel_lang_core::term::Import;
use nickel_lang_core::term::RecRecordData;
use nickel_lang_core::term::Term;
use nickel_lang_core::term::record::Field;
use nickel_lang_core::term::record::RecordAttrs;
use nickel_lang_core::traverse::Traverse;
use nickel_lang_core::traverse::TraverseControl;

use crate::Error;

const MAX_WATCH_FILES: usize = 256;
const MAX_WATCH_FILE_BYTES: usize = 2_097_152;
const MAX_WATCH_TOTAL_BYTES: usize = 16_777_216;
/// Record attributes of a manifest that carries no record annotation.
const PLAIN_RECORD_ATTRS: RecordAttrs = RecordAttrs {
    open: false,
    frozen: false,
};

/// Return a stable, sorted set of the root and transitive on-disk imports.
///
/// Bounds prevent an import fanout from making a polling watch consume
/// unbounded memory or open arbitrarily many sources. The caller must keep
/// the previous admitted source set and goals when this returns an error.
/// In particular, missing imports are not silently removed from the watch.
/// Every watch evaluation failure should be retried at the bounded poll rate
/// so a newly created missing import can recover without another root edit.
///
/// This scan alone cannot establish a stable source snapshot across edits;
/// the caller must confirm the observed fingerprints around goal admission.
/// Non-Nickel imports are watched as files, but have no Nickel child imports.
///
/// The file paths deliberately use Nickel's lexical normalization, not
/// `canonicalize`: the Nickel source cache doesn't resolve symlinks.
///
/// # Errors
///
/// Returns an error if any candidate cannot be read, parsed, or bounded.
/// The caller must not retract the last admitted goals on that error.
///
/// # Panics
///
/// Never intentionally panics on malformed source.
///
/// # Examples
///
/// The return value includes the root source even without imports.
pub fn source_dependencies(root: &Path, import_paths: &[OsString]) -> Result<Vec<PathBuf>, Error> {
    let root = normalize_path(root).map_err(Error::Io)?;
    let scope = crate::import_paths_for_file(&root, import_paths);
    source_dependencies_in_scope(&root, &scope)
}

pub(crate) fn source_dependencies_in_scope(root: &Path, import_paths: &[OsString]) -> Result<Vec<PathBuf>, Error> {
    let root = normalize_path(root).map_err(Error::Io)?;
    let mut pending = vec![(root, InputFormat::Nickel)];
    let mut observed = BTreeSet::new();
    let mut total_bytes = 0_usize;
    let mut cache = CacheHub::new();

    while let Some((path, format)) = pending.pop() {
        if !observed.insert(path.clone()) {
            continue;
        }
        if observed.len() > MAX_WATCH_FILES {
            return Err(boundary("too many watched Nickel source files"));
        }
        let remaining = MAX_WATCH_TOTAL_BYTES.saturating_sub(total_bytes);
        let read_limit_bytes = MAX_WATCH_FILE_BYTES.min(remaining);
        let mut bytes = Vec::new();
        let file_read_limit_bytes =
            u64::try_from(read_limit_bytes).map_err(|_| boundary("watched Nickel read limit cannot fit u64"))?;
        std::fs::File::open(&path)?.take(file_read_limit_bytes.saturating_add(1)).read_to_end(&mut bytes)?;
        if bytes.len() > read_limit_bytes {
            return Err(boundary("watched Nickel sources exceed byte limit"));
        }
        total_bytes = total_bytes
            .checked_add(bytes.len())
            .ok_or_else(|| boundary("watched Nickel sources exceed byte limit"))?;
        if format != InputFormat::Nickel {
            continue;
        }
        let source = String::from_utf8(bytes).map_err(|_| boundary("watched Nickel source is not UTF-8"))?;
        let id = cache.replace_string(SourcePath::Path(path.clone(), format), source);
        let parsed = TermParser::new()
            .parse_strict_compat(&mut PosTable::new(), id, Lexer::new(cache.sources.source(id)))
            .map_err(|_| boundary("could not parse watched Nickel source"))?;
        let mut discovered_paths = Vec::new();
        enum ScanStop {
            ImportBudget,
            PackageImport,
        }
        let stop = parsed.traverse_ref(
            &mut |value: &nickel_lang_core::eval::value::NickelValue, _: &()| {
                match value.as_term() {
                    Some(Term::Import(Import::Path { path, format })) => {
                        if discovered_paths.len() >= MAX_WATCH_FILES {
                            return TraverseControl::Return(ScanStop::ImportBudget);
                        }
                        discovered_paths.push((path.clone(), *format));
                    }
                    Some(Term::Import(Import::Package { .. })) => {
                        return TraverseControl::Return(ScanStop::PackageImport);
                    }
                    _ => {}
                }
                TraverseControl::Continue
            },
            &(),
        );
        match stop {
            Some(ScanStop::ImportBudget) => return Err(boundary("too many watched Nickel imports")),
            Some(ScanStop::PackageImport) => {
                return Err(boundary("watched Nickel package imports need an explicit package map"));
            }
            None => {}
        }
        enqueue_resolved_imports(&path, discovered_paths, import_paths, &mut pending)?;
    }

    debug_assert!(observed.len() <= MAX_WATCH_FILES);
    debug_assert!(total_bytes <= MAX_WATCH_TOTAL_BYTES);
    Ok(observed.into_iter().collect())
}

fn enqueue_resolved_imports(
    path: &Path,
    discovered_paths: Vec<(OsString, InputFormat)>,
    import_paths: &[OsString],
    pending: &mut Vec<(PathBuf, InputFormat)>,
) -> Result<(), Error> {
    for (import, format) in discovered_paths {
        let parent = path.parent().ok_or_else(|| boundary("watched source has no parent directory"))?;
        let found = std::iter::once(parent.to_path_buf())
            .chain(import_paths.iter().map(PathBuf::from))
            .find_map(|base| {
                let candidate = base.join(&import);
                std::fs::File::open(&candidate).ok().map(|_| candidate)
            })
            .ok_or_else(|| boundary("watched Nickel import is missing or unreadable"))?;
        pending.push((normalize_path(found).map_err(Error::Io)?, format));
    }
    Ok(())
}
fn boundary(message: &str) -> Error {
    Error::Boundary(message.to_string())
}

/// Attribute only independent, unannotated direct-import roots. Everything else
/// requires fresh evaluation; a syntactic label is not a dependency proof.
pub(crate) fn independent_import_roots(
    source: &str,
    root: &Path,
    scope: &[OsString],
) -> Result<Option<std::collections::BTreeMap<String, Vec<PathBuf>>>, Error> {
    let mut cache = CacheHub::new();
    let id = cache.replace_string(SourcePath::Query, source.to_string());
    let parsed = TermParser::new()
        .parse_strict_compat(&mut PosTable::new(), id, Lexer::new(cache.sources.source(id)))
        .map_err(|_| boundary("could not parse watched Nickel source"))?;
    let Some(Term::RecRecord(data)) = parsed.as_term() else {
        return Ok(None);
    };
    if !is_plain_manifest(data) {
        return Ok(None);
    }
    let mut collected = Vec::with_capacity(data.record.fields.len());
    for (label, field) in &data.record.fields {
        let Some(dependencies) = independent_root_dependencies(field, root, scope)? else {
            return Ok(None);
        };
        collected.push((label.label().to_string(), dependencies));
    }
    assert_eq!(collected.len(), data.record.fields.len());
    debug_assert!(collected.iter().all(|(_, dependencies)| !dependencies.is_empty()));
    Ok(Some(collected.into_iter().collect()))
}

/// Whether a manifest is a plain record that can carry independent roots.
fn is_plain_manifest(data: &RecRecordData) -> bool {
    if !data.includes.is_empty() || !data.dyn_fields.is_empty() {
        return false;
    }
    if data.record.attrs != PLAIN_RECORD_ATTRS {
        return false;
    }
    if data.record.sealed_tail.is_some() {
        return false;
    }
    if data.record.fields.keys().any(|key| key.label() == "name") {
        return false;
    }
    true
}

/// The import path of a manifest field, when the field is a plain Nickel import.
fn plain_import_path(field: &Field) -> Option<OsString> {
    if !field.metadata.is_empty() || !field.pending_contracts.is_empty() {
        return None;
    }
    let Some(Term::Import(Import::Path {
        path,
        format: InputFormat::Nickel,
    })) = field.value.as_ref().and_then(|value| value.as_term())
    else {
        return None;
    };
    Some(path.clone())
}

/// The dependencies of one manifest field, when that field is an independent
/// import whose own sources do not reach back into the manifest.
fn independent_root_dependencies(
    field: &Field,
    root: &Path,
    scope: &[OsString],
) -> Result<Option<Vec<PathBuf>>, Error> {
    let Some(import_path) = plain_import_path(field) else {
        return Ok(None);
    };
    let mut pending = Vec::new();
    enqueue_resolved_imports(root, vec![(import_path, InputFormat::Nickel)], scope, &mut pending)?;
    let (path, _) = pending.pop().ok_or_else(|| boundary("root import did not resolve"))?;
    let dependencies = source_dependencies_in_scope(&path, scope)?;
    // An import of this manifest can introduce root/sibling recursion.
    if dependencies.iter().any(|path| path == root) {
        return Ok(None);
    }
    Ok(Some(dependencies))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_transitive_imports_and_ignores_comments_and_strings() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("main.ncl");
        let mid = dir.path().join("mid.ncl");
        let leaf = dir.path().join("leaf.ncl");
        std::fs::write(&root, "# import \"ghost.ncl\"\nlet x = import \"mid.ncl\" in x").unwrap();
        std::fs::write(&mid, "let x = import \"leaf.ncl\" in x").unwrap();
        std::fs::write(&leaf, "\"import \\\"ghost.ncl\\\"\"").unwrap();
        assert_eq!(source_dependencies(&root, &[]).unwrap(), vec![leaf, root, mid]);
    }

    #[test]
    fn import_search_prefers_importer_parent_and_rejects_missing_or_broken_imports() {
        let root_dir = tempfile::tempdir().unwrap();
        let extra_dir = tempfile::tempdir().unwrap();
        let root = root_dir.path().join("main.ncl");
        let local = root_dir.path().join("dep.ncl");
        let extra = extra_dir.path().join("dep.ncl");
        let paths = [extra_dir.path().as_os_str().to_owned()];
        std::fs::write(&root, "import \"dep.ncl\"").unwrap();
        assert!(source_dependencies(&root, &paths).is_err());
        std::fs::write(&extra, "1").unwrap();
        let mut expected = vec![root.clone(), extra];
        expected.sort();
        assert_eq!(source_dependencies(&root, &paths).unwrap(), expected);
        std::fs::write(&local, "syntax error: ??").unwrap();
        assert!(source_dependencies(&root, &paths).is_err());
        std::fs::write(&local, "1").unwrap();
        assert_eq!(source_dependencies(&root, &paths).unwrap(), vec![local, root]);
    }

    #[test]
    fn cyclic_imports_do_not_revisit_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("main.ncl");
        let second = dir.path().join("second.ncl");
        std::fs::write(&root, "import \"second.ncl\"").unwrap();
        std::fs::write(&second, "import \"main.ncl\"").unwrap();
        assert_eq!(source_dependencies(&root, &[]).unwrap(), vec![root, second]);
    }
}
