use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use crunch_pipeline::BuildConfig;
use crunch_pipeline::build;

fn sha256_sri(bytes: &[u8]) -> String {
    use sha2::Digest;

    let digest: [u8; 32] = sha2::Sha256::digest(bytes).into();
    format!("sha256-{}", data_encoding::BASE64.encode(&digest))
}

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
    let stdlib = crunch_eval::stdlib::stdlib_import_path().expect("stdlib import path should resolve");
    vec![stdlib.into(), repo_root().into_os_string()]
}

fn build_config(file: PathBuf, output_dir: &Path, state_dir: &Path) -> BuildConfig {
    let keypair = crunch_build::load_keypair(
        "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
    )
    .unwrap();
    let trusted_keys = crunch_build::build_trusted_keys(&keypair, None);
    BuildConfig {
        file,
        import_paths: import_paths(),
        output_dir: output_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        store_dir: nix_compat::store_path::STORE_DIR.to_string(),
        verbose: false,
        max_jobs: 2,
        substituter_url: None,
        hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
        keypair,
        trusted_keys,
        trust_unsigned: false,
    }
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

    let mut config = build_config(ncl_file, output_dir.path(), state_dir.path());
    config.max_jobs = 1;

    let result = build(&config).await.unwrap();
    assert!(result.failed.is_empty(), "pipeline failures: {:?}", result.failed);
    assert!(result.fod_mismatches.is_empty(), "unexpected FOD mismatches");
    assert_eq!(result.hermeticity_mode, crunch_pipeline::HermeticityMode::Practical);
    assert!(result.hermeticity_audit_events.is_empty(), "unexpected hermeticity audit events");
    assert_eq!(result.outcomes.len(), 1);

    let outcome = &result.outcomes[0];
    let path_info = outcome.outputs.get("out").expect("out output should exist");
    let output_path = path_info.store_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap());
    let output_path = PathBuf::from(output_path);

    assert!(output_path.exists(), "output should exist on disk: {}", output_path.display());
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert_eq!(content.trim(), "pipeline works");
}

#[tokio::test]
async fn pipeline_reports_fod_mismatch_without_aborting_other_roots() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let good_src = work.path().join("good.txt");
    let bad_src = work.path().join("bad.txt");
    let ncl_file = work.path().join("fetches.ncl");

    let good_content = b"good pipeline fetch";
    let bad_content = b"bad pipeline fetch";
    let wrong_hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

    std::fs::write(&good_src, good_content).unwrap();
    std::fs::write(&bad_src, bad_content).unwrap();
    std::fs::write(
        &ncl_file,
        format!(
            r#"let crunch = import "lib.ncl" in
{{
  good = crunch.fetchurl {{
    name = "good-src",
    url = "file://{}",
    hash = "{}",
  }},
  bad = crunch.fetchurl {{
    name = "bad-src",
    url = "file://{}",
    hash = "{}",
  }},
}}"#,
            good_src.display(),
            sha256_sri(good_content),
            bad_src.display(),
            wrong_hash,
        ),
    )
    .unwrap();

    let config = build_config(ncl_file, output_dir.path(), state_dir.path());
    let result = build(&config).await.unwrap();

    assert_eq!(result.outcomes.len(), 1, "successful roots: {:?}", result.outcomes);
    assert_eq!(result.failed.len(), 1, "failed roots: {:?}", result.failed);
    assert_eq!(result.fod_mismatches.len(), 1, "FOD mismatches: {:?}", result.fod_mismatches);

    let mismatch = &result.fod_mismatches[0];
    assert_eq!(mismatch.name, "bad-src");
    assert_eq!(mismatch.expected_sri, wrong_hash);
    assert_ne!(mismatch.actual_sri, wrong_hash);
    assert!(mismatch.actual_sri.starts_with("sha256-"));
    assert!(result.failed[0].error.contains("FOD hash mismatch"));
    assert!(result.root_labels.values().any(|label| label == "good"));
    assert!(result.root_labels.values().any(|label| label == "bad"));

    let good_outcome = &result.outcomes[0];
    let good_output = good_outcome.outputs.get("out").expect("good root should have out");
    let good_path =
        PathBuf::from(good_output.store_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap()));
    assert!(good_path.exists(), "good fetch should land on disk: {}", good_path.display());
    assert_eq!(std::fs::read(&good_path).unwrap(), good_content);
}

#[tokio::test]
async fn pipeline_normalizes_runtime_environment_and_umask() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("env-envelope.ncl");

    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
{
  name = "pipeline-env-envelope",
  builder = "/bin/sh",
  args = ["-c", "set -eu; [ \"$HOME\" = \"/homeless-shelter\" ]; [ \"$PATH\" = \"/path-not-set\" ]; [ \"$USER\" = \"nixbld\" ]; [ \"$TZ\" = \"UTC\" ]; [ \"$LANG\" = \"C\" ]; [ \"$TMPDIR\" = \"/build\" ]; [ \"$SHELL\" = \"/bin/sh\" ]; [ -x \"$SHELL\" ]; [ -x /bin/sh ]; printf '%s\n' \"$HOME\" \"$PATH\" \"$USER\" \"$TZ\" \"$LANG\" \"$TMPDIR\" \"$SHELL\" \"$(umask)\" > \"$out\""],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
    )
    .unwrap();

    let mut config = build_config(ncl_file, output_dir.path(), state_dir.path());
    config.hermeticity_mode = crunch_pipeline::HermeticityMode::Strict;

    let result = build(&config).await.unwrap();

    assert_eq!(result.hermeticity_mode, crunch_pipeline::HermeticityMode::Strict);
    assert!(result.failed.is_empty(), "pipeline failures: {:?}", result.failed);
    assert!(result.hermeticity_audit_events.is_empty(), "unexpected hermeticity audit events");
    assert_eq!(result.outcomes.len(), 1);

    let outcome = &result.outcomes[0];
    let output_path = PathBuf::from(
        outcome.outputs["out"].store_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap()),
    );
    let lines: Vec<String> = std::fs::read_to_string(&output_path).unwrap().lines().map(ToOwned::to_owned).collect();

    assert_eq!(lines, vec![
        "/homeless-shelter".to_string(),
        "/path-not-set".to_string(),
        "nixbld".to_string(),
        "UTC".to_string(),
        "C".to_string(),
        "/build".to_string(),
        "/bin/sh".to_string(),
        "0022".to_string(),
    ]);
}

#[test]
fn pipeline_host_ambient_state_does_not_leak_into_strict_build() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let current_exe = std::env::current_exe().unwrap();
    let current_path = std::env::var_os("PATH").unwrap_or_default();
    let poisoned_path =
        std::env::join_paths([PathBuf::from("/tmp/hostile-path").into_os_string(), current_path]).unwrap();

    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("umask 077 && exec \"$1\" --exact --test-threads=1 \"$2\"")
        .arg("sh")
        .arg(&current_exe)
        .arg("pipeline_normalizes_runtime_environment_and_umask")
        .env("HOME", "/tmp/hostile-home")
        .env("PATH", &poisoned_path)
        .env("USER", "hostile-user")
        .env("LOGNAME", "hostile-logname")
        .env("TZ", "America/New_York")
        .env("LANG", "en_US.UTF-8")
        .env("LC_ALL", "en_US.UTF-8")
        .env("TMPDIR", "/tmp/hostile-tmpdir")
        .env("TEMP", "/tmp/hostile-temp")
        .env("TMP", "/tmp/hostile-tmp")
        .env("TEMPDIR", "/tmp/hostile-tempdir")
        .env("SHELL", "/tmp/hostile-shell")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "hostile child run failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn pipeline_practical_mode_audits_environment_override() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("env-override-practical.ncl");

    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
{
  name = "pipeline-env-override-practical",
  builder = "/bin/sh",
  args = ["-c", "set -eu; [ \"$PATH\" = \"/override/bin\" ]; printf '%s' \"$PATH\" > \"$out\""],
  env = { PATH = "/override/bin" },
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
    )
    .unwrap();

    let result = build(&build_config(ncl_file, output_dir.path(), state_dir.path())).await.unwrap();

    assert!(result.failed.is_empty(), "pipeline failures: {:?}", result.failed);
    assert_eq!(result.outcomes.len(), 1);
    assert_eq!(result.hermeticity_audit_events.len(), 1);
    assert_eq!(result.hermeticity_audit_events[0].kind, crunch_pipeline::HermeticityAuditKind::EnvironmentOverride);
    assert!(result.hermeticity_audit_events[0].detail.contains("PATH"));

    let outcome = &result.outcomes[0];
    let output_path = PathBuf::from(
        outcome.outputs["out"].store_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap()),
    );
    assert_eq!(std::fs::read_to_string(&output_path).unwrap(), "/override/bin");
}

#[tokio::test]
async fn pipeline_strict_mode_rejects_environment_override() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("env-override-strict.ncl");

    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
{
  name = "pipeline-env-override-strict",
  builder = "/bin/sh",
  args = ["-c", "set -eu; printf '%s' \"$PATH\" > \"$out\""],
  env = { PATH = "/override/bin" },
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
    )
    .unwrap();

    let mut config = build_config(ncl_file, output_dir.path(), state_dir.path());
    config.hermeticity_mode = crunch_pipeline::HermeticityMode::Strict;

    let result = build(&config).await.unwrap();

    assert_eq!(result.hermeticity_mode, crunch_pipeline::HermeticityMode::Strict);
    assert!(result.outcomes.is_empty(), "strict mode should reject before build: {:?}", result.outcomes);
    assert_eq!(result.failed.len(), 1, "strict-mode failures: {:?}", result.failed);
    assert!(
        result.hermeticity_audit_events.is_empty(),
        "strict rejection should not downgrade to an audit event"
    );
    assert!(result.failed[0].error.contains("unsafe sandbox environment override"));
    assert!(result.failed[0].error.contains("PATH"));
}
