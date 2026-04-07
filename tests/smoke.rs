//! End-to-end smoke tests: build derivations and verify outputs on disk.
//!
//! Each test uses `--store <tempdir>` so outputs land in a writable
//! directory without needing a writable /nix/store. Requires bwrap.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

fn crunch_cmd() -> Command {
    Command::cargo_bin("crunch").expect("crunch binary should be built")
}

fn can_build() -> bool {
    Path::new("/nix/store").exists()
        && std::process::Command::new("bwrap")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
}

/// Run `crunch build` with `--store <dir>` and return stdout.
/// Uses a per-call state dir to isolate pathinfo.redb across tests.
/// Pass `state_dir` to share state between calls (e.g., for cache tests).
fn build_ncl(ncl_content: &str, store: &Path) -> String {
    build_ncl_with_state(ncl_content, store, None)
}

fn build_ncl_with_state(ncl_content: &str, store: &Path, state_dir: Option<&Path>) -> String {
    let work = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("test.ncl");
    std::fs::write(&ncl_file, ncl_content).unwrap();

    let mut cmd = crunch_cmd();
    cmd.arg("--store").arg(store);
    if let Some(sd) = state_dir {
        cmd.arg("--state-dir").arg(sd);
    } else {
        // Isolate each build in its own state dir.
        let sd = work.path().join("state");
        cmd.arg("--state-dir").arg(&sd);
    }
    cmd.arg("build")
        .arg("--no-substitute")
        .arg("-I")
        .arg(work.path())
        .arg(&ncl_file);

    let output = cmd.output().expect("should execute");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "build failed (exit {}):\n{stderr}",
        output.status.code().unwrap_or(-1),
    );
    String::from_utf8(output.stdout).unwrap()
}

/// Extract the first output path from crunch build stdout.
/// Stdout lines are like: `/tmp/store/HASH-name` or `/tmp/store/HASH-name (cached)`
fn first_output_path(stdout: &str) -> PathBuf {
    let line = stdout.lines().next().expect("build should print at least one line");
    // Strip any suffix like " (cached)" or " (dev)"
    let path_str = line.split_whitespace().next().unwrap();
    PathBuf::from(path_str)
}

/// Extract all output paths from stdout.
fn all_output_paths(stdout: &str) -> Vec<PathBuf> {
    stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| PathBuf::from(l.split_whitespace().next().unwrap()))
        .collect()
}

// ── Tests ──────────────────────────────────────────────────────────

#[test]
fn smoke_build_flat_file_and_read_content() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let stdout = build_ncl(
        r#"let crunch = import "lib.ncl" in
{
  name = "flat-file",
  builder = "/bin/sh",
  args = ["-c", "echo 'crunch works' > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        store.path(),
    );

    let out = first_output_path(&stdout);
    assert!(out.exists(), "output should exist on disk: {}", out.display());
    let content = std::fs::read_to_string(&out).unwrap();
    assert_eq!(content.trim(), "crunch works");
}

#[test]
fn smoke_build_directory_output_with_structure() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let stdout = build_ncl(
        r#"let crunch = import "lib.ncl" in
{
  name = "dir-output",
  builder = "/bin/sh",
  args = ["-c", "BB=/bin/busybox; $BB mkdir -p $out/bin $out/lib && echo '#!/bin/sh' > $out/bin/run && echo 'libfoo' > $out/lib/foo.txt && $BB chmod +x $out/bin/run"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        store.path(),
    );

    let out = first_output_path(&stdout);
    assert!(out.join("bin/run").exists(), "bin/run should exist");
    assert!(out.join("lib/foo.txt").exists(), "lib/foo.txt should exist");

    let script = std::fs::read_to_string(out.join("bin/run")).unwrap();
    assert_eq!(script.trim(), "#!/bin/sh");

    let lib = std::fs::read_to_string(out.join("lib/foo.txt")).unwrap();
    assert_eq!(lib.trim(), "libfoo");

    // Check executable permission
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(out.join("bin/run")).unwrap().permissions().mode();
        assert!(mode & 0o111 != 0, "bin/run should be executable, mode={mode:o}");
    }
}

#[test]
fn smoke_build_and_run_shell_script() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let stdout = build_ncl(
        r#"let crunch = import "lib.ncl" in
{
  name = "runnable",
  builder = "/bin/sh",
  args = ["-c", "BB=/bin/busybox; $BB mkdir -p $out/bin && $BB printf '#!/bin/sh\necho hello-from-crunch\nexit 0\n' > $out/bin/greet && $BB chmod +x $out/bin/greet"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        store.path(),
    );

    let out = first_output_path(&stdout);
    let greet = out.join("bin/greet");
    assert!(greet.exists(), "greet script should exist");

    // Actually run the built script
    let run = std::process::Command::new(&greet)
        .output()
        .expect("should be able to execute the built script");
    assert!(run.status.success(), "greet should exit 0");
    let run_stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(run_stdout.trim(), "hello-from-crunch");
}

#[test]
fn smoke_build_failure_reports_error() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("fail.ncl");
    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
{
  name = "will-fail",
  builder = "/bin/sh",
  args = ["-c", "echo 'something went wrong' >&2; exit 42"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("--store")
        .arg(store.path())
        .arg("build")
        .arg("--no-substitute")
        .arg("-I")
        .arg(work.path())
        .arg(&ncl_file)
        .output()
        .unwrap();

    assert!(!output.status.success(), "build should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("will-fail") || stderr.contains("build failed"),
        "stderr should mention the derivation name or failure: {stderr}"
    );
}

#[test]
fn smoke_build_cached_on_second_run() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let ncl = r#"let crunch = import "lib.ncl" in
{
  name = "cached-test",
  builder = "/bin/sh",
  args = ["-c", "echo cached > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#;

    // First build — shared state dir so pathinfo.redb persists.
    let stdout1 = build_ncl_with_state(ncl, store.path(), Some(state.path()));
    let path1 = first_output_path(&stdout1);
    assert!(path1.exists());
    // First build should NOT say "(cached)"
    assert!(
        !stdout1.contains("(cached)"),
        "first build should not be cached: {stdout1}"
    );

    // Second build — same store + same state dir → cache hit
    let stdout2 = build_ncl_with_state(ncl, store.path(), Some(state.path()));
    let path2 = first_output_path(&stdout2);
    assert_eq!(path1, path2, "same derivation should produce same path");
    assert!(
        stdout2.contains("(cached)"),
        "second build should be cached: {stdout2}"
    );
}

#[test]
fn smoke_build_ca_derivation() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let stdout = build_ncl(
        r#"let crunch = import "lib.ncl" in
{
  name = "ca-smoke",
  builder = "/bin/sh",
  args = ["-c", "echo content-addressed > $out"],
} | crunch.Derivation"#,
        store.path(),
    );

    let out = first_output_path(&stdout);
    assert!(out.exists(), "CA output should exist: {}", out.display());
    let content = std::fs::read_to_string(&out).unwrap();
    assert_eq!(content.trim(), "content-addressed");
}

#[test]
fn smoke_fetchurl_downloads_and_stores() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    // Spin up a local HTTP server with known content
    let body = b"fetched-by-crunch\n";
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        use std::io::Write;
        // Serve exactly one request
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 4096];
            let _ = std::io::Read::read(&mut stream, &mut buf);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.write_all(body);
        }
    });

    // Compute the expected sha256 of the content for the FOD hash.
    // fetchurl does a flat hash of the downloaded bytes.
    use sha2::{Sha256, Digest};
    use base64::Engine;
    let hash = Sha256::digest(body);
    let sri = format!("sha256-{}", base64::engine::general_purpose::STANDARD.encode(hash));

    let store = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("fetch.ncl");
    std::fs::write(
        &ncl_file,
        format!(
            r#"let crunch = import "lib.ncl" in
crunch.fetchurl {{
  url = "http://{addr}/data.txt",
  hash = "{sri}",
}}"#
        ),
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("--store")
        .arg(store.path())
        .arg("build")
        .arg("--no-substitute")
        .arg("-I")
        .arg(work.path())
        .arg(&ncl_file)
        .output()
        .unwrap();

    server.join().unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "fetchurl build should succeed: {stderr}"
    );

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    let out = first_output_path(&stdout_str);
    assert!(out.exists(), "fetched output should exist: {}", out.display());
    let content = std::fs::read_to_string(&out).unwrap();
    assert_eq!(content, "fetched-by-crunch\n");
}

#[test]
fn smoke_build_multi_derivation_file() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("multi.ncl");
    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
[
  {
    name = "alpha",
    builder = "/bin/sh",
    args = ["-c", "echo alpha-output > $out"],
    addressing_mode = 'input-addressed,
  } | crunch.Derivation,
  {
    name = "beta",
    builder = "/bin/sh",
    args = ["-c", "echo beta-output > $out"],
    addressing_mode = 'input-addressed,
  } | crunch.Derivation,
]"#,
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("--store")
        .arg(store.path())
        .arg("build")
        .arg("--no-substitute")
        .arg("-I")
        .arg(work.path())
        .arg(&ncl_file)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "multi build failed: {stderr}");

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    let paths = all_output_paths(&stdout_str);
    assert_eq!(paths.len(), 2, "should have 2 outputs: {stdout_str}");

    // Both outputs should exist and have distinct content
    let mut contents: Vec<String> = paths
        .iter()
        .map(|p| {
            assert!(p.exists(), "output should exist: {}", p.display());
            std::fs::read_to_string(p).unwrap().trim().to_string()
        })
        .collect();
    contents.sort();
    assert_eq!(contents, vec!["alpha-output", "beta-output"]);
}

#[test]
fn smoke_build_symlink_in_output() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let stdout = build_ncl(
        r#"let crunch = import "lib.ncl" in
{
  name = "with-symlink",
  builder = "/bin/sh",
  args = ["-c", "BB=/bin/busybox; $BB mkdir -p $out/bin && echo '#!/bin/sh' > $out/bin/real && $BB chmod +x $out/bin/real && $BB ln -s real $out/bin/alias"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        store.path(),
    );

    let out = first_output_path(&stdout);
    let alias = out.join("bin/alias");
    assert!(alias.exists(), "symlink should exist");

    #[cfg(unix)]
    {
        let meta = std::fs::symlink_metadata(&alias).unwrap();
        assert!(meta.is_symlink(), "alias should be a symlink");
        let target = std::fs::read_link(&alias).unwrap();
        assert_eq!(target.to_str().unwrap(), "real");
    }
}

#[test]
fn smoke_deterministic_output_path() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let ncl = r#"let crunch = import "lib.ncl" in
{
  name = "deterministic",
  builder = "/bin/sh",
  args = ["-c", "echo stable > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#;

    // Build in two independent store dirs
    let store1 = tempfile::tempdir().unwrap();
    let store2 = tempfile::tempdir().unwrap();

    let stdout1 = build_ncl(ncl, store1.path());
    let stdout2 = build_ncl(ncl, store2.path());

    let path1 = first_output_path(&stdout1);
    let path2 = first_output_path(&stdout2);

    // The filename (hash-name) should be identical even though store dirs differ
    let name1 = path1.file_name().unwrap();
    let name2 = path2.file_name().unwrap();
    assert_eq!(name1, name2, "same derivation should produce same hash-name");
}
