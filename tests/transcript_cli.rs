#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command as ProcessCommand;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn crunch() -> Command {
    Command::cargo_bin("crunch").unwrap()
}

fn output_path(dir: &TempDir, name: &str) -> std::path::PathBuf {
    dir.path().join(format!("{name}.output.json"))
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

#[cfg(unix)]
fn write_fake_mantle(path: &Path) {
    write_file(
        path,
        r#"#!/bin/sh
set -eu
printf 'args:%s\n' "$*"
printf 'tmp:%s\n' "$MANTLE_TRANSCRIPT_TMP"
case " $* " in
  *" fail "*)
    printf 'expected failure marker\n' >&2
    exit 42
    ;;
  *" check-marker "*)
    test -f "$MANTLE_TRANSCRIPT_TMP/marker.txt"
    printf 'hidden setup marker present\n'
    ;;
  *" json "*)
    printf '{"schema":"crunch-doctor-report-v1","ok":true}\n'
    ;;
  *" cwd "*)
    cat local-input.txt
    ;;
  *)
    printf 'success marker\n'
    ;;
esac
"#,
    );
    let mut permissions = std::fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).unwrap();
}

#[cfg(unix)]
#[test]
fn transcript_run_passes_success_and_injects_isolated_state_and_writes_output() {
    let dir = TempDir::new().unwrap();
    let fake = dir.path().join("fake-mantle");
    write_fake_mantle(&fake);
    let transcript = dir.path().join("success.md");
    let output = output_path(&dir, "success");
    write_file(
        &transcript,
        r#"```mantle
mantle ok
```
```expect
--store
--state-dir
success marker
```
"#,
    );

    crunch()
        .args(["transcript", "run"])
        .arg(&transcript)
        .arg("--output")
        .arg(&output)
        .arg("--mantle-bin")
        .arg(&fake)
        .assert()
        .success()
        .stdout(predicate::str::contains("transcript ok: 1 visible step(s)"));
    let evidence = std::fs::read_to_string(output).unwrap();
    assert!(evidence.contains("mantle-transcript-output-v1"));
    assert!(evidence.contains("success marker"));
}

#[cfg(unix)]
#[test]
fn transcript_run_passes_expected_failure_when_diagnostic_matches() {
    let dir = TempDir::new().unwrap();
    let fake = dir.path().join("fake-mantle");
    write_fake_mantle(&fake);
    let transcript = dir.path().join("expected-error.md");
    let output = output_path(&dir, "expected-error");
    write_file(
        &transcript,
        r#"```mantle:error
mantle fail
```
```expect
expected failure marker
```
```mantle
mantle ok
```
```expect
success marker
```
"#,
    );

    crunch()
        .args(["transcript", "run"])
        .arg(&transcript)
        .arg("--output")
        .arg(&output)
        .arg("--mantle-bin")
        .arg(&fake)
        .assert()
        .success();
}

#[cfg(unix)]
#[test]
fn transcript_run_rejects_expected_failure_without_expectation() {
    let dir = TempDir::new().unwrap();
    let fake = dir.path().join("fake-mantle");
    write_fake_mantle(&fake);
    let transcript = dir.path().join("expected-error-without-expect.md");
    let output = output_path(&dir, "expected-error-without-expect");
    write_file(
        &transcript,
        r#"```mantle:error
mantle fail
```
"#,
    );

    crunch()
        .args(["transcript", "run"])
        .arg(&transcript)
        .arg("--output")
        .arg(&output)
        .arg("--mantle-bin")
        .arg(&fake)
        .assert()
        .failure()
        .stderr(predicate::str::contains("mantle:error block must be followed"));
}

#[cfg(unix)]
#[test]
fn transcript_run_rejects_empty_expect_block() {
    let dir = TempDir::new().unwrap();
    let fake = dir.path().join("fake-mantle");
    write_fake_mantle(&fake);
    let transcript = dir.path().join("empty-expect.md");
    let output = output_path(&dir, "empty-expect");
    write_file(
        &transcript,
        r#"```mantle:error
mantle fail
```
```expect
```
"#,
    );

    crunch()
        .args(["transcript", "run"])
        .arg(&transcript)
        .arg("--output")
        .arg(&output)
        .arg("--mantle-bin")
        .arg(&fake)
        .assert()
        .failure()
        .stderr(predicate::str::contains("expect block must contain"));
}

#[cfg(unix)]
#[test]
fn transcript_run_rejects_empty_expect_json_block() {
    let dir = TempDir::new().unwrap();
    let fake = dir.path().join("fake-mantle");
    write_fake_mantle(&fake);
    let transcript = dir.path().join("empty-expect-json.md");
    let output = output_path(&dir, "empty-expect-json");
    write_file(
        &transcript,
        r#"```mantle
mantle json
```
```expect:json
```
"#,
    );

    crunch()
        .args(["transcript", "run"])
        .arg(&transcript)
        .arg("--output")
        .arg(&output)
        .arg("--mantle-bin")
        .arg(&fake)
        .assert()
        .failure()
        .stderr(predicate::str::contains("expect:json block must contain"));
}

#[cfg(unix)]
#[test]
fn transcript_run_executes_hidden_setup_before_visible_command() {
    let dir = TempDir::new().unwrap();
    let fake = dir.path().join("fake-mantle");
    write_fake_mantle(&fake);
    let transcript = dir.path().join("hidden-setup.md");
    let output = output_path(&dir, "hidden-setup");
    write_file(
        &transcript,
        r#"```setup:hide
printf marker > "$MANTLE_TRANSCRIPT_TMP/marker.txt"
printf 'hidden setup output\n'
```
```mantle
mantle check-marker
```
```expect
hidden setup marker present
```
"#,
    );

    crunch()
        .args(["transcript", "run"])
        .arg(&transcript)
        .arg("--output")
        .arg(&output)
        .arg("--mantle-bin")
        .arg(&fake)
        .assert()
        .success();
    let evidence = std::fs::read_to_string(output).unwrap();
    assert!(evidence.contains("hidden_setups"));
    assert!(evidence.contains("hidden setup output"));
}

#[cfg(unix)]
#[test]
fn transcript_run_runs_cleanup_even_after_failure() {
    let dir = TempDir::new().unwrap();
    let fake = dir.path().join("fake-mantle");
    write_fake_mantle(&fake);
    let cleanup_marker = dir.path().join("cleanup-ran");
    let transcript = dir.path().join("cleanup.md");
    let output = output_path(&dir, "cleanup");
    write_file(
        &transcript,
        &format!(
            r#"```cleanup:hide
touch {}
```
```mantle
mantle fail
```
```expect
not reached
```
"#,
            cleanup_marker.display()
        ),
    );

    crunch()
        .args(["transcript", "run"])
        .arg(&transcript)
        .arg("--output")
        .arg(&output)
        .arg("--mantle-bin")
        .arg(&fake)
        .assert()
        .failure();
    assert!(cleanup_marker.exists(), "cleanup must run after transcript failure");
    assert!(output.exists(), "failed transcript still writes evidence artifact");
}

#[cfg(unix)]
#[test]
fn transcript_run_uses_transcript_directory_and_ignores_prose_fences() {
    let dir = TempDir::new().unwrap();
    let fake = dir.path().join("fake-mantle");
    write_fake_mantle(&fake);
    let subdir = dir.path().join("doc");
    std::fs::create_dir_all(&subdir).unwrap();
    write_file(&subdir.join("local-input.txt"), "local file from transcript dir\n");
    let transcript = subdir.join("cwd.md");
    let output = output_path(&dir, "cwd");
    write_file(
        &transcript,
        r#"```sh
echo this is prose, not a transcript block
```
```mantle
mantle cwd
```
```expect
local file from transcript dir
```
"#,
    );

    crunch()
        .args(["transcript", "run"])
        .arg(&transcript)
        .arg("--output")
        .arg(&output)
        .arg("--mantle-bin")
        .arg(&fake)
        .assert()
        .success();
}

#[cfg(unix)]
#[test]
fn transcript_run_rejects_in_place_without_explicit_allowance_before_command_execution() {
    let dir = TempDir::new().unwrap();
    let invoked = dir.path().join("invoked");
    let fake = dir.path().join("fake-mantle");
    write_file(&fake, &format!("#!/bin/sh\ntouch {}\necho should-not-run\n", invoked.display()));
    let mut permissions = std::fs::metadata(&fake).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&fake, permissions).unwrap();
    let transcript = dir.path().join("in-place.md");
    let output = output_path(&dir, "in-place");
    write_file(
        &transcript,
        r#"```transcript:options
{"in_place": true, "reason": "operator repro"}
```
```mantle
mantle ok
```
```expect
should-not-run
```
"#,
    );

    crunch()
        .args(["transcript", "run"])
        .arg(&transcript)
        .arg("--output")
        .arg(&output)
        .arg("--mantle-bin")
        .arg(&fake)
        .assert()
        .failure()
        .stderr(predicate::str::contains("in_place=true"));
    assert!(!invoked.exists(), "in-place rejection must happen before command execution");
}

#[cfg(unix)]
#[test]
fn transcript_run_supports_json_expectations() {
    let dir = TempDir::new().unwrap();
    let fake = dir.path().join("fake-mantle");
    write_fake_mantle(&fake);
    let transcript = dir.path().join("json.md");
    let output = output_path(&dir, "json");
    write_file(
        &transcript,
        r#"```mantle
mantle json
```
```expect:json
schema = crunch-doctor-report-v1
ok = true
```
"#,
    );

    crunch()
        .args(["transcript", "run"])
        .arg(&transcript)
        .arg("--output")
        .arg(&output)
        .arg("--mantle-bin")
        .arg(&fake)
        .assert()
        .success();
}

#[cfg(unix)]
#[test]
fn checked_fast_fixtures_parse_through_transcript_runner() {
    let dir = TempDir::new().unwrap();
    let fake = dir.path().join("fake-mantle");
    write_fake_mantle(&fake);
    let output = output_path(&dir, "fixture");
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let status = ProcessCommand::new(env!("CARGO_BIN_EXE_crunch"))
        .args(["transcript", "run", "tests/fixtures/transcripts/fast/doctor-success.md"])
        .arg("--output")
        .arg(&output)
        .arg("--mantle-bin")
        .arg(&fake)
        .current_dir(repo_root)
        .status()
        .unwrap();
    assert!(status.success(), "doctor-success fixture must execute through the runner");
}
