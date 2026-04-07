use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crunch_pipeline::{BuildConfig, build};

fn can_build() -> bool {
    Path::new("/nix/store").exists()
        && std::process::Command::new("bwrap")
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should exist")
}

fn import_paths() -> Vec<OsString> {
    let stdlib = crunch_eval::stdlib::stdlib_import_path()
        .expect("stdlib import path should resolve");
    vec![stdlib.into(), repo_root().into_os_string()]
}

#[tokio::test]
async fn pipeline_builds_trivial_derivation_end_to_end() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("test.ncl");

    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
{
  name = "pipeline-e2e",
  builder = "/bin/sh",
  args = ["-c", "echo 'pipeline works' > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
    )
    .unwrap();

    let config = BuildConfig {
        file: ncl_file,
        import_paths: import_paths(),
        output_dir: output_dir.path().to_path_buf(),
        state_dir: state_dir.path().to_path_buf(),
        store_dir: nix_compat::store_path::STORE_DIR.to_string(),
        verbose: false,
        max_jobs: 1,
        substituter_url: None,
    };

    let result = build(&config).await.unwrap();
    assert!(result.failed.is_empty(), "pipeline failures: {:?}", result.failed);
    assert!(result.fod_mismatches.is_empty(), "unexpected FOD mismatches");
    assert_eq!(result.outcomes.len(), 1);

    let outcome = &result.outcomes[0];
    let path_info = outcome.outputs.get("out").expect("out output should exist");
    let output_path = path_info
        .store_path
        .to_absolute_path_with_prefix(output_dir.path().to_str().unwrap());
    let output_path = PathBuf::from(output_path);

    assert!(output_path.exists(), "output should exist on disk: {}", output_path.display());
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert_eq!(content.trim(), "pipeline works");
}
