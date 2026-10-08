//! Embedded crunch Nickel stdlib.
//!
//! The `.ncl` files from `lib/` are included at compile time and can be
//! written to a temporary directory for use as an import path, or
//! resolved from the source tree during development.

use std::path::Path;
use std::path::PathBuf;

const MAX_STDLIB_SEARCH_ANCESTORS: usize = 12;

/// The embedded stdlib files.
const STDLIB_FILES: &[(&str, &str)] = &[
    ("lib.ncl", include_str!("../../../lib/lib.ncl")),
    ("android.ncl", include_str!("../../../lib/android.ncl")),
    ("android/sources.ncl", include_str!("../../../lib/android/sources.ncl")),
    ("artifact-auth-cutover-receipt.ncl", include_str!("../../../lib/artifact-auth-cutover-receipt.ncl")),
    (
        "artifact-source-migration-receipt.ncl",
        include_str!("../../../lib/artifact-source-migration-receipt.ncl"),
    ),
    ("contracts.ncl", include_str!("../../../lib/contracts.ncl")),
    (
        "durable-file-publication-adoption-receipt.ncl",
        include_str!("../../../lib/durable-file-publication-adoption-receipt.ncl"),
    ),
    ("derivation.ncl", include_str!("../../../lib/derivation.ncl")),
    ("fetch.ncl", include_str!("../../../lib/fetch.ncl")),
    ("fixed_output.ncl", include_str!("../../../lib/fixed_output.ncl")),
    ("helpers.ncl", include_str!("../../../lib/helpers.ncl")),
    ("kernelscript_experiment.ncl", include_str!("../../../lib/kernelscript_experiment.ncl")),
    ("offline_cargo.ncl", include_str!("../../../lib/offline_cargo.ncl")),
    ("oci_registry_trust.ncl", include_str!("../../../lib/oci_registry_trust.ncl")),
    ("project.ncl", include_str!("../../../lib/project.ncl")),
    ("project_outputs.ncl", include_str!("../../../lib/project_outputs.ncl")),
    ("seed.ncl", include_str!("../../../lib/seed.ncl")),
    ("remote-builders.ncl", include_str!("../../../lib/remote-builders.ncl")),
    ("wasm_component.ncl", include_str!("../../../lib/wasm_component.ncl")),
    ("scheduling.ncl", include_str!("../../../lib/scheduling.ncl")),
];

/// Write the embedded stdlib to a directory. Returns the path that
/// should be added to the Nickel import path.
///
/// If `dir` is `None`, writes to a temporary directory under
/// `$XDG_CACHE_HOME/crunch/stdlib/` (or `/tmp/crunch-stdlib/`).
pub fn write_stdlib(dir: Option<&Path>) -> Result<PathBuf, std::io::Error> {
    assert!(!STDLIB_FILES.is_empty(), "embedded stdlib must not be empty");
    assert!(
        STDLIB_FILES.iter().all(|(name, _contents)| !name.is_empty()),
        "embedded stdlib file names must not be empty"
    );

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
        if name.contains('/')
            && let Some(parent) = path.parent()
        {
            std::fs::create_dir_all(parent)?;
        }
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

fn source_stdlib_candidates(current_dir: Option<&Path>, current_exe: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(dir) = current_dir {
        for ancestor in dir.ancestors().take(MAX_STDLIB_SEARCH_ANCESTORS) {
            candidates.push(ancestor.join("lib"));
        }
    }
    if let Some(exe) = current_exe
        && let Some(exe_dir) = exe.parent()
    {
        for ancestor in exe_dir.ancestors().take(MAX_STDLIB_SEARCH_ANCESTORS) {
            candidates.push(ancestor.join("lib"));
        }
    }
    candidates
}

/// Return the stdlib directory from the source tree, if it exists.
/// Useful during development — avoids needing to embed/extract.
pub fn source_stdlib_dir() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok();
    let current_exe = std::env::current_exe().ok();
    let candidates = source_stdlib_candidates(current_dir.as_deref(), current_exe.as_deref());
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
    fn source_stdlib_candidates_include_cwd_and_exe_ancestors() {
        let cwd = Path::new("/workspace/project");
        let exe = Path::new("/workspace/project/target/debug/mantle");
        let candidates = source_stdlib_candidates(Some(cwd), Some(exe));

        assert!(candidates.contains(&PathBuf::from("/workspace/project/lib")));
        assert!(candidates.contains(&PathBuf::from("/workspace/project/target/debug/lib")));
        assert!(candidates.contains(&PathBuf::from("/workspace/lib")));
    }

    #[test]
    fn source_stdlib_candidates_empty_without_inputs() {
        let candidates = source_stdlib_candidates(None, None);

        assert!(candidates.is_empty());
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
        let mut expected: std::collections::BTreeSet<String> = std::fs::read_dir(&repo_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".ncl"))
            .collect();
        for entry in std::fs::read_dir(repo_dir.join("android")).unwrap() {
            let name = entry.unwrap().file_name().to_string_lossy().into_owned();
            if name.ends_with(".ncl") {
                expected.insert(format!("android/{name}"));
            }
        }
        let actual: std::collections::BTreeSet<String> =
            STDLIB_FILES.iter().map(|(name, _)| (*name).to_string()).collect();

        assert_eq!(actual, expected);
        assert!(actual.contains("fetch.ncl"));
        assert!(actual.contains("kernelscript_experiment.ncl"));
        assert!(actual.contains("oci_registry_trust.ncl"));
        assert!(actual.contains("scheduling.ncl"));
    }

    #[test]
    fn written_embedded_stdlib_is_importable() {
        let dir = tempfile::tempdir().unwrap();
        let stdlib_dir = write_stdlib(Some(dir.path())).unwrap();
        let import_paths = vec![stdlib_dir.into_os_string()];

        let expr = crate::evaluate_str(r#"let lib = import "lib.ncl" in "ok""#, &import_paths).unwrap();
        assert_eq!(expr.as_str(), Some("ok"));
        assert!(dir.path().join("fetch.ncl").exists());
        assert!(dir.path().join("kernelscript_experiment.ncl").exists());
        assert!(dir.path().join("oci_registry_trust.ncl").exists());
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
        assert!(path.join("android/sources.ncl").exists());
        let paths = vec![path.into_os_string()];
        let source: crunch_glue::CrunchDerivation = crate::evaluate_str(
            r#"let mantle = import "lib.ncl" in
               let [jdk, _, _, _] = mantle.AndroidSources.cohort in
               mantle.AndroidSources.source jdk"#,
            &paths,
        )
        .unwrap()
        .to_serde()
        .unwrap();
        assert_eq!(source.builder, "builtin:fetchurl");
        assert_eq!(
            source.env.get("url").unwrap(),
            "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.17%2B10/OpenJDK17U-jdk_x64_linux_hotspot_17.0.17_10.tar.gz"
        );
        assert_eq!(source.fixed_output.unwrap().hash, "sha256-mS+W55lQdax2NrsajeUrDGHXHtMTf6/JeauWtKt43XU=");
    }

    #[test]
    fn embedded_android_source_record_identities_match_canonical_blake3() {
        let dir = tempfile::tempdir().unwrap();
        let stdlib_dir = write_stdlib(Some(dir.path())).unwrap();
        let records = crate::evaluate_str(
            r#"let android = import "android.ncl" in
               android.cohort |> std.array.map (fun source => {
                 component = source.component,
                 normalized = android.normalized source,
                 identity = source.record_blake3,
               })"#,
            &[stdlib_dir.into_os_string()],
        )
        .unwrap();
        let pins: Vec<serde_json::Value> = records.to_serde().unwrap();
        assert_eq!(pins.len(), 4);
        for pin in pins {
            let normalized = pin["normalized"].as_str().unwrap();
            let declared = pin["identity"].as_str().unwrap();
            assert_eq!(
                blake3::hash(normalized.as_bytes()).to_hex().as_str(),
                declared,
                "pinned BLAKE3 must match normalized {}",
                pin["component"]
            );
        }
    }
}
