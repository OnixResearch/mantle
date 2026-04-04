//! Embedded crunch Nickel stdlib.
//!
//! The `.ncl` files from `lib/` are included at compile time and can be
//! written to a temporary directory for use as an import path, or
//! resolved from the source tree during development.

use std::path::{Path, PathBuf};

/// The embedded stdlib files.
const STDLIB_FILES: &[(&str, &str)] = &[
    ("lib.ncl", include_str!("../../../lib/lib.ncl")),
    ("contracts.ncl", include_str!("../../../lib/contracts.ncl")),
    ("derivation.ncl", include_str!("../../../lib/derivation.ncl")),
    ("fixed_output.ncl", include_str!("../../../lib/fixed_output.ncl")),
    ("helpers.ncl", include_str!("../../../lib/helpers.ncl")),
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
            let cache = std::env::var("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
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
        let resolved = candidate
            .canonicalize()
            .ok()
            .filter(|p| p.join("lib.ncl").exists());
        if let Some(path) = resolved {
            return Some(path);
        }
    }
    None
}

/// Get the stdlib import path — tries source tree first, falls back to
/// writing embedded files.
pub fn stdlib_import_path() -> Result<PathBuf, std::io::Error> {
    if let Some(source_dir) = source_stdlib_dir() {
        return Ok(source_dir);
    }
    write_stdlib(None)
}
