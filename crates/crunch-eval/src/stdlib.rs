//! Embedded crunch Nickel stdlib.
//!
//! The `.ncl` files from `lib/` are included at compile time and can be
//! written to a temporary directory for use as an import path, or
//! resolved from the source tree during development.

use std::path::Path;
use std::path::PathBuf;

/// The embedded stdlib files.
const STDLIB_FILES: &[(&str, &str)] = &[
    ("lib.ncl", include_str!("../../../lib/lib.ncl")),
    ("contracts.ncl", include_str!("../../../lib/contracts.ncl")),
    ("derivation.ncl", include_str!("../../../lib/derivation.ncl")),
    ("fetch.ncl", include_str!("../../../lib/fetch.ncl")),
    ("fixed_output.ncl", include_str!("../../../lib/fixed_output.ncl")),
    ("helpers.ncl", include_str!("../../../lib/helpers.ncl")),
    ("project.ncl", include_str!("../../../lib/project.ncl")),
    ("project_outputs.ncl", include_str!("../../../lib/project_outputs.ncl")),
    ("seed.ncl", include_str!("../../../lib/seed.ncl")),
];

/// Write the embedded stdlib to a directory. Returns the path that
/// should be added to the Nickel import path.
///
/// If `dir` is `None`, writes to a temporary directory under
/// `$XDG_CACHE_HOME/crunch/stdlib/` (or `/tmp/crunch-stdlib/`).
pub fn write_stdlib(dir: Option<&Path>) -> Result<PathBuf, std::io::Error> {
    let target = match dir {
        Some(d) => d.to_path_buf(),
        None => {
            let cache = std::env::var("XDG_CACHE_HOME").map(PathBuf::from).unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home).join(".cache")
            });
            cache.join("crunch").join("stdlib")
        }
    };

    std::fs::create_dir_all(&target)?;

    for (name, contents) in STDLIB_FILES {
        let path = target.join(name);
        // Only write if content changed (avoid unnecessary FS writes)
        let needs_write = match std::fs::read_to_string(&path) {
            Ok(existing) => existing != *contents,
            Err(_) => true,
        };
        if needs_write {
            std::fs::write(&path, contents)?;
        }
    }

    Ok(target)
}

/// Return the stdlib directory from the source tree, if it exists.
/// Useful during development — avoids needing to embed/extract.
pub fn source_stdlib_dir() -> Option<PathBuf> {
    // Look for lib/ relative to the workspace root
    let candidates = [
        PathBuf::from("lib"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lib"),
    ];
    for candidate in &candidates {
        let resolved = candidate.canonicalize().ok().filter(|p| p.join("lib.ncl").exists());
        if let Some(path) = resolved {
            return Some(path);
        }
    }
    None
}

fn force_embedded_stdlib_from_env() -> bool {
    match std::env::var("CRUNCH_FORCE_EMBEDDED_STDLIB") {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            normalized == "1" || normalized == "true" || normalized == "yes"
        }
        Err(_) => false,
    }
}

/// Get the stdlib import path — tries source tree first, falls back to
/// writing embedded files. Set `CRUNCH_FORCE_EMBEDDED_STDLIB=1` to force the
/// embedded path even in a source checkout.
pub fn stdlib_import_path() -> Result<PathBuf, std::io::Error> {
    if !force_embedded_stdlib_from_env()
        && let Some(source_dir) = source_stdlib_dir()
    {
        return Ok(source_dir);
    }
    write_stdlib(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    static STDLIB_ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn write_stdlib_creates_all_files() {
        let dir = tempfile::tempdir().unwrap();
        let result = write_stdlib(Some(dir.path())).unwrap();
        assert_eq!(result, dir.path());

        for (name, _contents) in STDLIB_FILES {
            let path = dir.path().join(name);
            assert!(path.exists(), "stdlib file missing: {name}");
        }
    }

    #[test]
    fn write_stdlib_content_matches_embedded() {
        let dir = tempfile::tempdir().unwrap();
        write_stdlib(Some(dir.path())).unwrap();

        for (name, expected) in STDLIB_FILES {
            let actual = std::fs::read_to_string(dir.path().join(name)).unwrap();
            assert_eq!(actual, *expected, "content mismatch for {name}");
        }
    }

    #[test]
    fn write_stdlib_skips_rewrite_when_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        write_stdlib(Some(dir.path())).unwrap();

        let mtime_before = std::fs::metadata(dir.path().join("lib.ncl")).unwrap().modified().unwrap();

        // Small sleep to ensure mtime granularity
        std::thread::sleep(std::time::Duration::from_millis(50));
        write_stdlib(Some(dir.path())).unwrap();

        let mtime_after = std::fs::metadata(dir.path().join("lib.ncl")).unwrap().modified().unwrap();

        assert_eq!(mtime_before, mtime_after, "file should not be rewritten");
    }

    #[test]
    fn stdlib_import_path_returns_dir_with_lib_ncl() {
        let dir = stdlib_import_path().unwrap();
        assert!(dir.join("lib.ncl").exists());
    }

    #[test]
    fn stdlib_is_importable() {
        let stdlib_dir = stdlib_import_path().unwrap();
        let import_paths = vec![stdlib_dir.into_os_string()];

        let expr = crate::evaluate_str(r#"let lib = import "lib.ncl" in "ok""#, &import_paths).unwrap();
        assert_eq!(expr.as_str(), Some("ok"));
    }

    #[test]
    fn embedded_stdlib_matches_repo_lib_directory() {
        let repo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lib");
        let expected: std::collections::BTreeSet<String> = std::fs::read_dir(&repo_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".ncl"))
            .collect();
        let actual: std::collections::BTreeSet<String> =
            STDLIB_FILES.iter().map(|(name, _)| (*name).to_string()).collect();

        assert_eq!(actual, expected);
        assert!(actual.contains("fetch.ncl"));
    }

    #[test]
    fn written_embedded_stdlib_is_importable() {
        let dir = tempfile::tempdir().unwrap();
        let stdlib_dir = write_stdlib(Some(dir.path())).unwrap();
        let import_paths = vec![stdlib_dir.into_os_string()];

        let expr = crate::evaluate_str(r#"let lib = import "lib.ncl" in "ok""#, &import_paths).unwrap();
        assert_eq!(expr.as_str(), Some("ok"));
        assert!(dir.path().join("fetch.ncl").exists());
    }

    #[test]
    fn stdlib_import_path_can_force_embedded_copy() {
        let _guard = STDLIB_ENV_MUTEX.lock().unwrap();
        // SAFETY: test process serializes all environment mutation through
        // STDLIB_ENV_MUTEX, and this test restores the variable before exit.
        unsafe {
            std::env::set_var("CRUNCH_FORCE_EMBEDDED_STDLIB", "1");
        }

        let path = stdlib_import_path().unwrap();
        let repo_dir = source_stdlib_dir().unwrap();

        // SAFETY: same serialization + restoration argument as above.
        unsafe {
            std::env::remove_var("CRUNCH_FORCE_EMBEDDED_STDLIB");
        }

        assert_ne!(path, repo_dir);
        assert!(path.join("fetch.ncl").exists());
    }
}
