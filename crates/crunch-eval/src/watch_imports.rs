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
use nickel_lang_core::term::Term;
use nickel_lang_core::traverse::Traverse;
use nickel_lang_core::traverse::TraverseControl;

use crate::Error;

const MAX_WATCH_FILES: usize = 256;
const MAX_WATCH_FILE_BYTES: usize = 2 * 1024 * 1024;
const MAX_WATCH_TOTAL_BYTES: usize = 16 * 1024 * 1024;

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
        let read_limit = MAX_WATCH_FILE_BYTES.min(remaining);
        let mut bytes = Vec::new();
        std::fs::File::open(&path)?
            .take(u64::try_from(read_limit).unwrap_or(u64::MAX).saturating_add(1))
            .read_to_end(&mut bytes)?;
        if bytes.len() > read_limit {
            return Err(boundary("watched Nickel sources exceed byte limit"));
        }
        total_bytes += bytes.len();
        if format != InputFormat::Nickel {
            continue;
        }
        let source = String::from_utf8(bytes).map_err(|_| boundary("watched Nickel source is not UTF-8"))?;
        let id = cache.replace_string(SourcePath::Path(path.clone(), format), source);
        let parsed = TermParser::new()
            .parse_strict_compat(&mut PosTable::new(), id, Lexer::new(cache.sources.source(id)))
            .map_err(|_| boundary("could not parse watched Nickel source"))?;
        let mut imports = Vec::new();
        enum ScanStop {
            ImportBudget,
            PackageImport,
        }
        let stop = parsed.traverse_ref(
            &mut |value: &nickel_lang_core::eval::value::NickelValue, _: &()| {
                match value.as_term() {
                    Some(Term::Import(Import::Path { path, format })) => {
                        if imports.len() >= MAX_WATCH_FILES {
                            return TraverseControl::Return(ScanStop::ImportBudget);
                        }
                        imports.push((path.clone(), *format));
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
        for (import, format) in imports {
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
    }

    Ok(observed.into_iter().collect())
}

fn boundary(message: &str) -> Error {
    Error::Boundary(message.to_string())
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
