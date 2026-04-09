//! Self-hosting proof: stage0 -> stage1 -> stage2.
//!
//! This test is expensive (~30 min) and requires:
//!   - bwrap on PATH
//!   - git, cargo, tar, xz, cp on PATH
//!   - ~4 GiB free disk in /tmp
//!   - Internet access (for initial bootstrap fetch)
//!
//! Run with:
//!   ./scripts/prove-self-hosting.sh
//!
//! The helper prepares PATH, compiler/linker lookup, pkg-config, openssl,
//! and sandbox-shell discovery before invoking the ignored proof test.
//!
//! The test:
//! 1. Runs `crunch self-build` (stage0) using the checkout binary
//! 2. Finds the stage1 binary in the output store
//! 3. Invalidates the prior `*-crunch` output
//! 4. Runs `stage1/bin/crunch self-build` (stage2) with a fresh state dir
//! 5. Verifies stage2 produced a working binary
//! 6. Checks that stage2 used crunch-built bwrap (not host fallback)

mod audit_support;

use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Output;
use std::process::Stdio;
use std::thread;
use std::thread::JoinHandle;

use assert_cmd::cargo::cargo_bin;
use audit_support::AuditArtifact;
use audit_support::write_command_audit;

const MAX_DIAGNOSTIC_LINES: u32 = 60;
const MAX_DIAGNOSTIC_ENTRIES: u32 = 64;
const CONTROLLED_FAILURE_ENV: &str = "CRUNCH_SELF_HOSTING_CONTROLLED_FAILURE";

struct StageEvidence {
    stage_name: String,
    output: Output,
    stderr: String,
    audit_dir: PathBuf,
    diagnostics_file: PathBuf,
    stdout_file: PathBuf,
    stderr_file: PathBuf,
}

struct CapturedStageOutput {
    output: Output,
    stdout_file: PathBuf,
    stderr_file: PathBuf,
}

fn stage_stream_file(proof_dir: &Path, stage_name: &str, stream_name: &str) -> PathBuf {
    assert!(proof_dir.exists(), "proof dir must exist");
    assert!(!stage_name.is_empty(), "stage name must not be empty");
    assert!(matches!(stream_name, "stdout" | "stderr"), "unexpected stream name: {stream_name}");

    proof_dir.join(format!("{stage_name}-{stream_name}.txt"))
}

fn write_stage_stream_files(proof_dir: &Path, stage_name: &str, output: &Output) -> (PathBuf, PathBuf) {
    let stdout_file = stage_stream_file(proof_dir, stage_name, "stdout");
    let stderr_file = stage_stream_file(proof_dir, stage_name, "stderr");
    std::fs::write(&stdout_file, &output.stdout).unwrap_or_else(|err| {
        panic!(
            "{stage_name} stdout file write failed: {err}\n\
             path: {}\nproof_dir: {}",
            stdout_file.display(),
            proof_dir.display(),
        )
    });
    std::fs::write(&stderr_file, &output.stderr).unwrap_or_else(|err| {
        panic!(
            "{stage_name} stderr file write failed: {err}\n\
             path: {}\nproof_dir: {}",
            stderr_file.display(),
            proof_dir.display(),
        )
    });
    (stdout_file, stderr_file)
}

fn copy_stream_to_parent_and_file<R: Read>(mut reader: R, mut file: File, is_stderr: bool) -> io::Result<Vec<u8>> {
    let mut captured = Vec::new();
    let mut buffer = [0_u8; 4096];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let chunk = &buffer[..bytes_read];
        file.write_all(chunk)?;
        if is_stderr {
            let mut sink = io::stderr();
            sink.write_all(chunk)?;
            sink.flush()?;
        } else {
            let mut sink = io::stdout();
            sink.write_all(chunk)?;
            sink.flush()?;
        }
        captured.extend_from_slice(chunk);
    }

    file.flush()?;
    Ok(captured)
}

fn spawn_stream_capture<R: Read + Send + 'static>(
    reader: R,
    file: File,
    is_stderr: bool,
) -> JoinHandle<io::Result<Vec<u8>>> {
    thread::spawn(move || copy_stream_to_parent_and_file(reader, file, is_stderr))
}

fn join_stream_capture(
    handle: JoinHandle<io::Result<Vec<u8>>>,
    stage_name: &str,
    stream_name: &str,
    proof_dir: &Path,
) -> Vec<u8> {
    assert!(!stage_name.is_empty(), "stage name must not be empty");
    assert!(matches!(stream_name, "stdout" | "stderr"), "unexpected stream name: {stream_name}");

    match handle.join() {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(err)) => panic!(
            "{stage_name} {stream_name} capture failed: {err}\n\
             proof_dir: {}\n\
             Check {stage_name}-{stream_name}.txt in proof_dir for partial output.",
            proof_dir.display(),
        ),
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

fn run_command_live(
    proof_dir: &Path,
    stage_name: &str,
    command: &mut std::process::Command,
) -> io::Result<CapturedStageOutput> {
    assert!(proof_dir.exists(), "proof dir must exist");
    assert!(!stage_name.is_empty(), "stage name must not be empty");

    let stdout_file = stage_stream_file(proof_dir, stage_name, "stdout");
    let stderr_file = stage_stream_file(proof_dir, stage_name, "stderr");
    let stdout_writer = File::create(&stdout_file)?;
    let stderr_writer = File::create(&stderr_file)?;

    eprintln!("{stage_name} stdout: {}", stdout_file.display());
    eprintln!("{stage_name} stderr: {}", stderr_file.display());

    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let stdout_reader = child.stdout.take().ok_or_else(|| io::Error::other("child stdout must be piped"))?;
    let stderr_reader = child.stderr.take().ok_or_else(|| io::Error::other("child stderr must be piped"))?;

    let stdout_handle = spawn_stream_capture(stdout_reader, stdout_writer, false);
    let stderr_handle = spawn_stream_capture(stderr_reader, stderr_writer, true);
    let status = child.wait()?;

    let stdout = join_stream_capture(stdout_handle, stage_name, "stdout", proof_dir);
    let stderr = join_stream_capture(stderr_handle, stage_name, "stderr", proof_dir);
    Ok(CapturedStageOutput {
        output: Output { status, stdout, stderr },
        stdout_file,
        stderr_file,
    })
}

fn pre_stage_context(stage_name: &str, command: &[String], store_dir: &Path, state_dir: &Path) -> String {
    assert!(!stage_name.is_empty(), "stage name must not be empty");
    assert!(!command.is_empty(), "stage command must not be empty");

    let logs_dir = state_dir.join("logs");
    let mut out = String::with_capacity(4096);
    out.push_str(&format!("stage: {stage_name}\n"));
    out.push_str(&format!("command: {}\n", command.join(" ")));
    out.push_str(&format!("store_dir: {}\n", store_dir.display()));
    out.push_str(&format!("state_dir: {}\n", state_dir.display()));
    out.push_str(&format!("logs_dir: {}\n", logs_dir.display()));
    append_section(&mut out, "store entries", &list_dir_entries(store_dir, MAX_DIAGNOSTIC_ENTRIES));
    append_section(&mut out, "log entries", &list_dir_entries(&logs_dir, MAX_DIAGNOSTIC_ENTRIES));
    out
}

/// Check prerequisites for the proof.
fn self_build_prereq_error() -> Option<String> {
    // Need the helper's core runtime tools on PATH.
    let tools = ["bwrap", "git", "cargo", "tar", "xz", "cp", "chmod"];
    for tool in tools {
        if std::process::Command::new(tool).arg("--version").output().is_err() {
            return Some(format!(
                "{tool} not on PATH. Run ./scripts/prove-self-hosting.sh --check for the full environment probe."
            ));
        }
    }
    // Need the crunch source tree (Cargo.toml + bootstrap/ in cwd or parents).
    let cwd = std::env::current_dir().unwrap();
    if !cwd.join("Cargo.toml").exists() || !cwd.join("bootstrap").exists() {
        return Some("not in crunch source tree".to_string());
    }
    None
}

/// Find the crunch binary in an output store (`*-crunch/bin/crunch`).
fn find_crunch_binary(store: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(store).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-crunch") {
            let binary = entry.path().join("bin").join("crunch");
            if binary.exists() {
                return Some(binary);
            }
        }
    }
    None
}

/// Remove all `*-crunch` output directories from a store.
fn remove_crunch_outputs(store: &Path) -> u32 {
    let mut removed: u32 = 0;
    let entries = match std::fs::read_dir(store) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-crunch") {
            if std::fs::remove_dir_all(entry.path()).is_ok() {
                removed += 1;
            }
        }
    }
    removed
}

/// Scan store for `*-bwrap/bin/bwrap` and return the path if found.
fn find_bwrap_on_disk(store: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(store).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-bwrap") {
            let bin = entry.path().join("bin").join("bwrap");
            if bin.exists() {
                return Some(bin);
            }
        }
    }
    None
}

/// Scan store for `*-busybox/bin/busybox` and return the path if found.
fn find_busybox_on_disk(store: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(store).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-busybox") {
            let bin = entry.path().join("bin").join("busybox");
            if bin.exists() {
                return Some(bin);
            }
        }
    }
    None
}

/// Check a file is executable (unix).
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
}

/// Parse proof lines from stderr.
fn extract_proof_field<'a>(stderr: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("self-build-proof: {key}=");
    for line in stderr.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            return Some(rest);
        }
    }
    None
}

fn extract_optional_path_field(stderr: &str, key: &str) -> Option<PathBuf> {
    let value = extract_proof_field(stderr, key)?;
    if value.is_empty() {
        return None;
    }
    if value == "none" {
        return None;
    }
    Some(PathBuf::from(value))
}

fn extract_bwrap_binary_path(stderr: &str) -> Option<PathBuf> {
    let value = extract_proof_field(stderr, "bwrap-source")?;
    let (_, path) = value.split_once(':')?;
    if path.is_empty() {
        return None;
    }
    Some(PathBuf::from(path))
}

fn append_section(out: &mut String, title: &str, lines: &[String]) {
    assert!(!title.is_empty(), "section title must not be empty");
    assert!(!lines.is_empty(), "section {title} must have at least one line");

    out.push_str(title);
    out.push_str(":\n");
    for line in lines {
        out.push_str("  ");
        out.push_str(line);
        out.push('\n');
    }
}

fn tail_lines(text: &str, max_lines: u32) -> Vec<String> {
    assert!(max_lines > 0, "max_lines must be positive");

    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return vec!["<none>".to_string()];
    }

    let line_count = u32::try_from(lines.len()).unwrap_or(u32::MAX);
    let start = line_count.saturating_sub(max_lines);
    let mut tail = Vec::new();
    if start > 0 {
        tail.push(format!("... {start} earlier lines omitted ..."));
    }
    let start_index = usize::try_from(start).unwrap_or(usize::MAX);
    for line in lines.into_iter().skip(start_index) {
        tail.push(line.to_string());
    }
    tail
}

fn list_dir_entries(dir: &Path, max_entries: u32) -> Vec<String> {
    assert!(max_entries > 0, "max_entries must be positive");

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => return vec![format!("<unavailable: {err}>")],
    };

    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    if names.is_empty() {
        return vec!["<empty>".to_string()];
    }

    let name_count = u32::try_from(names.len()).unwrap_or(u32::MAX);
    if name_count > max_entries {
        let keep = usize::try_from(max_entries).unwrap_or(usize::MAX);
        let omitted = name_count.saturating_sub(max_entries);
        names.truncate(keep);
        names.push(format!("... {omitted} more entries omitted ..."));
    }
    names
}

fn proof_lines(stderr: &str, max_lines: u32) -> Vec<String> {
    assert!(max_lines > 0, "max_lines must be positive");

    let mut lines: Vec<String> = stderr
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("self-build-proof:"))
        .map(str::to_string)
        .collect();
    if lines.is_empty() {
        return vec!["<none>".to_string()];
    }

    let line_count = u32::try_from(lines.len()).unwrap_or(u32::MAX);
    if line_count > max_lines {
        let keep = usize::try_from(max_lines).unwrap_or(usize::MAX);
        let omitted = line_count.saturating_sub(max_lines);
        lines.truncate(keep);
        lines.push(format!("... {omitted} more proof lines omitted ..."));
    }
    lines
}

fn render_stage_diagnostics(
    stage_name: &str,
    command: &[String],
    output: &Output,
    store_dir: &Path,
    state_dir: &Path,
) -> String {
    assert!(!stage_name.is_empty(), "stage name must not be empty");
    assert!(!command.is_empty(), "stage command must not be empty");

    let logs_dir = state_dir.join("logs");
    let exit_code = output.status.code().map(|code| code.to_string()).unwrap_or_else(|| "signal".to_string());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut out = String::with_capacity(8192);
    out.push_str(&format!("stage: {stage_name}\n"));
    out.push_str(&format!("exit_code: {exit_code}\n"));
    out.push_str(&format!("command: {}\n", command.join(" ")));
    out.push_str(&format!("store_dir: {}\n", store_dir.display()));
    out.push_str(&format!("state_dir: {}\n", state_dir.display()));
    out.push_str(&format!("logs_dir: {}\n", logs_dir.display()));
    append_section(&mut out, "proof lines", &proof_lines(&stderr, MAX_DIAGNOSTIC_LINES));
    append_section(&mut out, "store entries", &list_dir_entries(store_dir, MAX_DIAGNOSTIC_ENTRIES));
    append_section(&mut out, "log entries", &list_dir_entries(&logs_dir, MAX_DIAGNOSTIC_ENTRIES));
    append_section(&mut out, "stderr tail", &tail_lines(&stderr, MAX_DIAGNOSTIC_LINES));
    append_section(&mut out, "stdout tail", &tail_lines(&stdout, MAX_DIAGNOSTIC_LINES / 2));
    out
}

fn write_stage_diagnostics_file(proof_dir: &Path, stage_name: &str, diagnostics: &str) -> PathBuf {
    assert!(proof_dir.exists(), "proof dir must exist");
    assert!(!stage_name.is_empty(), "stage name must not be empty");

    let path = proof_dir.join(format!("{stage_name}-diagnostics.txt"));
    std::fs::write(&path, diagnostics).unwrap_or_else(|err| {
        panic!(
            "{stage_name} diagnostics write failed: {err}\n\
             path: {}\nproof_dir: {}",
            path.display(),
            proof_dir.display(),
        )
    });
    path
}

fn record_stage_evidence(
    proof_dir: &Path,
    stage_name: &str,
    command: Vec<String>,
    output: Output,
    store_dir: &Path,
    state_dir: &Path,
) -> StageEvidence {
    assert!(proof_dir.exists(), "proof dir must exist");
    assert!(!stage_name.is_empty(), "stage name must not be empty");
    assert!(!command.is_empty(), "stage command must not be empty");

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let (stdout_file, stderr_file) = write_stage_stream_files(proof_dir, stage_name, &output);
    let diagnostics = render_stage_diagnostics(stage_name, &command, &output, store_dir, state_dir);
    let diagnostics_file = write_stage_diagnostics_file(proof_dir, stage_name, &diagnostics);
    let logs_dir = state_dir.join("logs");
    let pathinfo_db = state_dir.join("pathinfo.redb");
    let output_binary = extract_optional_path_field(&stderr, "output-binary");
    let busybox_binary = extract_optional_path_field(&stderr, "busybox-path");
    let bwrap_binary = extract_bwrap_binary_path(&stderr);

    let mut audit_artifacts = vec![
        AuditArtifact {
            label: "logs-dir",
            path: &logs_dir,
        },
        AuditArtifact {
            label: "pathinfo-db",
            path: &pathinfo_db,
        },
        AuditArtifact {
            label: "diagnostics",
            path: &diagnostics_file,
        },
        AuditArtifact {
            label: "stdout-capture",
            path: &stdout_file,
        },
        AuditArtifact {
            label: "stderr-capture",
            path: &stderr_file,
        },
    ];
    if let Some(path) = output_binary.as_ref() {
        audit_artifacts.push(AuditArtifact {
            label: "output-binary",
            path,
        });
    }
    if let Some(path) = busybox_binary.as_ref() {
        audit_artifacts.push(AuditArtifact {
            label: "busybox-binary",
            path,
        });
    }
    if let Some(path) = bwrap_binary.as_ref() {
        audit_artifacts.push(AuditArtifact {
            label: "bwrap-binary",
            path,
        });
    }

    let cwd = std::env::current_dir().unwrap_or_else(|err| {
        panic!(
            "{stage_name} cwd lookup failed: {err}\n{}",
            pre_stage_context(stage_name, &command, store_dir, state_dir),
        )
    });
    let audit_dir = write_command_audit(
        "self-hosting",
        stage_name,
        &cwd,
        &command,
        &output,
        &audit_artifacts,
        &[
            ("CRUNCH_STATE_DIR", state_dir.display().to_string()),
            ("CRUNCH_STORE_DIR", store_dir.display().to_string()),
        ],
    )
    .unwrap_or_else(|err| {
        panic!(
            "{stage_name} audit bundle write failed: {err}\n\
             diagnostics: {}\n{}",
            diagnostics_file.display(),
            pre_stage_context(stage_name, &command, store_dir, state_dir),
        )
    });

    StageEvidence {
        stage_name: stage_name.to_string(),
        output,
        stderr,
        audit_dir,
        diagnostics_file,
        stdout_file,
        stderr_file,
    }
}

fn stage_context(stage: &StageEvidence) -> String {
    let mut out = String::with_capacity(10240);
    out.push_str(&format!("audit bundle: {}\n", stage.audit_dir.display()));
    out.push_str(&format!("diagnostics: {}\n", stage.diagnostics_file.display()));
    out.push_str(&format!("stdout: {}\n", stage.stdout_file.display()));
    out.push_str(&format!("stderr: {}\n", stage.stderr_file.display()));
    match std::fs::read_to_string(&stage.diagnostics_file) {
        Ok(snapshot) => {
            out.push_str("saved diagnostics snapshot:\n");
            out.push_str(&snapshot);
            if !snapshot.ends_with('\n') {
                out.push('\n');
            }
        }
        Err(err) => {
            out.push_str("saved diagnostics snapshot:\n");
            out.push_str(&format!("<unavailable: {err}>\n"));
        }
    }
    out
}

fn assert_stage_success(stage: &StageEvidence) {
    let exit_code = stage.output.status.code().unwrap_or(-1);
    assert!(
        stage.output.status.success(),
        "{} failed (exit {}).\n{}",
        stage.stage_name,
        exit_code,
        stage_context(stage),
    );
}

#[test]
fn render_stage_diagnostics_includes_proof_lines_and_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let store = tmp.path().join("store");
    let state = tmp.path().join("state");
    let logs = state.join("logs");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::create_dir_all(&logs).unwrap();
    std::fs::write(logs.join("stage.log"), "boom").unwrap();

    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("printf 'stdout-line\n'; printf 'self-build-proof: key=value\nstderr-line\n' >&2; exit 7")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(7));

    let text =
        render_stage_diagnostics("stage-x", &["crunch".to_string(), "self-build".to_string()], &output, &store, &state);

    assert!(text.contains("stage: stage-x"));
    assert!(text.contains("command: crunch self-build"));
    assert!(text.contains("self-build-proof: key=value"));
    assert!(text.contains("store entries:"));
    assert!(text.contains("log entries:"));
    assert!(text.contains("stderr tail:"));
}

#[test]
fn run_command_live_writes_stage_stream_files() {
    let proof_dir = tempfile::tempdir().unwrap();
    let mut command = std::process::Command::new("/bin/sh");
    command
        .arg("-c")
        .arg("printf 'stdout-1\n'; printf 'stderr-1\n' >&2; sleep 0.05; printf 'stdout-2\n'; printf 'stderr-2\n' >&2");

    let captured = run_command_live(proof_dir.path(), "live-stage", &mut command).unwrap();
    let stdout = String::from_utf8_lossy(&captured.output.stdout);
    let stderr = String::from_utf8_lossy(&captured.output.stderr);

    assert!(captured.output.status.success(), "live command should succeed");
    assert_eq!(stdout, "stdout-1\nstdout-2\n");
    assert_eq!(stderr, "stderr-1\nstderr-2\n");
    assert_eq!(std::fs::read(&captured.stdout_file).unwrap(), captured.output.stdout);
    assert_eq!(std::fs::read(&captured.stderr_file).unwrap(), captured.output.stderr);
}

#[test]
fn record_stage_evidence_writes_audit_and_diagnostics_paths() {
    let proof_dir = tempfile::tempdir().unwrap();
    let store = proof_dir.path().join("store");
    let state = proof_dir.path().join("state");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    let output_binary = proof_dir.path().join("out");
    std::fs::write(&output_binary, b"binary").unwrap();

    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("printf 'self-build-proof: output-binary={}\\n' >&2", output_binary.display()))
        .output()
        .unwrap();
    assert!(output.status.success());

    let evidence = record_stage_evidence(
        proof_dir.path(),
        "stage-y",
        vec!["crunch".to_string(), "self-build".to_string()],
        output,
        &store,
        &state,
    );
    let context = stage_context(&evidence);
    let meta_json = std::fs::read_to_string(evidence.audit_dir.join("meta.json")).unwrap();

    assert!(evidence.audit_dir.exists(), "audit dir should exist on disk");
    assert!(evidence.diagnostics_file.exists(), "diagnostics file should exist on disk");
    assert!(evidence.stdout_file.exists(), "stdout file should exist on disk");
    assert!(evidence.stderr_file.exists(), "stderr file should exist on disk");
    assert!(context.contains(&evidence.audit_dir.display().to_string()));
    assert!(context.contains(&evidence.diagnostics_file.display().to_string()));
    assert!(context.contains(&evidence.stdout_file.display().to_string()));
    assert!(context.contains(&evidence.stderr_file.display().to_string()));
    assert!(context.contains("saved diagnostics snapshot:"));
    assert!(meta_json.contains("\"output-binary\""));
    assert!(meta_json.contains("\"stdout-capture\""));
    assert!(meta_json.contains("\"stderr-capture\""));
    assert!(!meta_json.contains("\"store-dir\""));
    assert!(!meta_json.contains("\"state-dir\""));
}

/// Regression: a populated store directory must NOT be hashed as an
/// audit artifact. The prior bug hashed the entire store/ tree,
/// which exceeded `MAX_AUDIT_ENTRIES` (100,000) on real self-build
/// stores and panicked between stage0 and stage2.
///
/// This test creates a store with enough structure to be representative
/// and verifies that `record_stage_evidence` succeeds without including
/// store-dir or state-dir as audit artifacts.
#[test]
fn record_stage_evidence_with_populated_store_avoids_audit_limit() {
    let proof_dir = tempfile::tempdir().unwrap();
    let store = proof_dir.path().join("store");
    let state = proof_dir.path().join("state");
    let logs = state.join("logs");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::create_dir_all(&logs).unwrap();

    // Simulate a self-build store: 200 hash-prefixed directories, each
    // with a bin/ subdir and a fake binary. Total entries: 200 * 3 = 600
    // (dir + bin/ + bin/file). A real store has 10k+ entries.
    let entry_count: u32 = 200;
    for i in 0..entry_count {
        let name = format!("abcdef1234567890-bootstrap-tool-{i}");
        let bin_dir = store.join(&name).join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::write(bin_dir.join("tool"), format!("fake-{i}")).unwrap();
    }
    // Also populate logs.
    for i in 0..50_u32 {
        std::fs::write(logs.join(format!("build-{i}.log")), format!("log-{i}")).unwrap();
    }
    // Write a pathinfo.redb stand-in.
    std::fs::write(state.join("pathinfo.redb"), b"fake-redb").unwrap();

    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("printf 'self-build-proof: output-binary=/tmp/fake\n' >&2")
        .output()
        .unwrap();
    assert!(output.status.success());

    // This must NOT panic. If store/ were an audit artifact, the
    // recursive hash would process 600+ entries (and on a real store,
    // 100k+, triggering the MAX_AUDIT_ENTRIES assert).
    let evidence = record_stage_evidence(
        proof_dir.path(),
        "populated-store",
        vec!["crunch".to_string(), "self-build".to_string()],
        output,
        &store,
        &state,
    );

    let meta_json = std::fs::read_to_string(evidence.audit_dir.join("meta.json")).unwrap();
    // Store and state dirs must NOT appear as artifact labels.
    // (The store path may appear in the env section — that's fine.)
    assert!(!meta_json.contains("\"store-dir\""));
    assert!(!meta_json.contains("\"state-dir\""));
    // But selected artifacts (logs-dir, pathinfo-db, diagnostics, captures) must appear.
    assert!(meta_json.contains("\"logs-dir\""));
    assert!(meta_json.contains("\"pathinfo-db\""));
    assert!(meta_json.contains("\"diagnostics\""));
    assert!(meta_json.contains("\"stdout-capture\""));
    assert!(meta_json.contains("\"stderr-capture\""));
}

/// Regression: audit bundle hashing enforces MAX_AUDIT_DEPTH (64).
/// A directory tree deeper than the limit must trigger a panic, not
/// silently skip or corrupt the hash.
#[test]
fn audit_hashing_rejects_excessively_deep_trees() {
    let tmp = tempfile::tempdir().unwrap();
    // Build a path 70 levels deep (exceeds MAX_AUDIT_DEPTH of 64).
    let depth: u32 = 70;
    let mut deep_path = tmp.path().to_path_buf();
    for i in 0..depth {
        deep_path = deep_path.join(format!("d{i}"));
    }
    std::fs::create_dir_all(&deep_path).unwrap();
    std::fs::write(deep_path.join("leaf.txt"), b"deep").unwrap();

    let dummy_output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("true")
        .output()
        .unwrap();

    let result = std::panic::catch_unwind(|| {
        write_command_audit(
            "depth-limit",
            "deep",
            tmp.path(),
            &["test".to_string()],
            &dummy_output,
            &[AuditArtifact {
                label: "deep-tree",
                path: tmp.path(),
            }],
            &[],
        )
    });

    assert!(result.is_err(), "audit hashing must panic on trees deeper than MAX_AUDIT_DEPTH");
}

#[test]
fn stage_context_reads_saved_snapshot_from_disk() {
    let proof_dir = tempfile::tempdir().unwrap();
    let store = proof_dir.path().join("store");
    let state = proof_dir.path().join("state");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::create_dir_all(&state).unwrap();

    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("printf 'self-build-proof: output-binary=/tmp/out\n' >&2")
        .output()
        .unwrap();
    assert!(output.status.success());

    let evidence = record_stage_evidence(
        proof_dir.path(),
        "stage-z",
        vec!["crunch".to_string(), "self-build".to_string()],
        output,
        &store,
        &state,
    );
    std::fs::write(&evidence.diagnostics_file, "saved snapshot marker\n").unwrap();

    let context = stage_context(&evidence);
    assert!(context.contains("saved snapshot marker"));
    assert!(
        !context.contains("stage: stage-z"),
        "context should come from saved file, not recompute diagnostics"
    );
}

fn panic_with_controlled_breadcrumbs() -> ! {
    let proof_dir = tempfile::tempdir().unwrap();
    let store = proof_dir.path().join("store");
    let state = proof_dir.path().join("state");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::create_dir_all(&state).unwrap();

    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("printf 'self-build-proof: output-binary=/tmp/fake-out\ncontrolled stderr\n' >&2")
        .output()
        .unwrap();
    assert!(output.status.success());

    let evidence = record_stage_evidence(
        proof_dir.path(),
        "controlled-failure",
        vec!["crunch".to_string(), "self-build".to_string()],
        output,
        &store,
        &state,
    );

    panic!("controlled failure for breadcrumb verification\n{}", stage_context(&evidence));
}

#[test]
fn self_hosting_controlled_failure_reports_breadcrumbs_trigger() {
    if std::env::var_os(CONTROLLED_FAILURE_ENV).is_none() {
        return;
    }

    panic_with_controlled_breadcrumbs();
}

#[test]
fn self_hosting_controlled_failure_reports_breadcrumbs() {
    let current_exe = std::env::current_exe().unwrap();
    let output = std::process::Command::new(&current_exe)
        .env(CONTROLLED_FAILURE_ENV, "1")
        .arg("self_hosting_controlled_failure_reports_breadcrumbs_trigger")
        .arg("--exact")
        .arg("--nocapture")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    assert!(!output.status.success(), "controlled child run should fail to expose breadcrumbs");
    assert!(combined.contains("controlled failure for breadcrumb verification"));
    assert!(combined.contains("audit bundle:"));
    assert!(combined.contains("diagnostics:"));
    assert!(combined.contains("saved diagnostics snapshot:"));
    assert!(combined.contains("self-build-proof: output-binary=/tmp/fake-out"));
}

#[test]
#[ignore]
fn self_hosting_stage0_stage1_stage2() {
    if let Some(err) = self_build_prereq_error() {
        panic!("self-hosting proof prerequisites not met: {err}");
    }

    let proof_dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir for proof: {err}"));
    eprintln!("proof dir: {}", proof_dir.path().display());
    let store = proof_dir.path().join("store");
    std::fs::create_dir_all(&store).unwrap();

    // The store starts empty. Verify no stale outputs exist.
    assert!(find_bwrap_on_disk(&store).is_none(), "fresh store must not contain bwrap",);
    assert!(find_busybox_on_disk(&store).is_none(), "fresh store must not contain busybox",);
    assert!(find_crunch_binary(&store).is_none(), "fresh store must not contain crunch",);

    // ── Stage 0: checkout binary builds stage1 ──────────────────

    eprintln!("\n=== PROOF: Stage 0 (checkout -> stage1) ===\n");

    let stage0_state = proof_dir.path().join("state0");
    std::fs::create_dir_all(&stage0_state).unwrap();

    let stage0_command = vec![
        "crunch".to_string(),
        "--verbose".to_string(),
        "--log-level".to_string(),
        "info".to_string(),
        "--store".to_string(),
        store.display().to_string(),
        "--state-dir".to_string(),
        stage0_state.display().to_string(),
        "--nix-compat".to_string(),
        "self-build".to_string(),
        "--no-substitute".to_string(),
        "-j".to_string(),
        "4".to_string(),
    ];
    eprintln!("stage0 store: {}", store.display());
    eprintln!("stage0 state: {}", stage0_state.display());
    let crunch_bin = cargo_bin("crunch");
    let mut stage0_process = std::process::Command::new(&crunch_bin);
    stage0_process.args(&stage0_command[1..]);
    let stage0 = run_command_live(proof_dir.path(), "stage0", &mut stage0_process).unwrap_or_else(|err| {
        panic!(
            "stage0 should execute: {err}\n{}",
            pre_stage_context("stage0", &stage0_command, &store, &stage0_state),
        )
    });
    assert_eq!(stage0.stdout_file, stage_stream_file(proof_dir.path(), "stage0", "stdout"));
    assert_eq!(stage0.stderr_file, stage_stream_file(proof_dir.path(), "stage0", "stderr"));
    let stage0_evidence =
        record_stage_evidence(proof_dir.path(), "stage0", stage0_command, stage0.output, &store, &stage0_state);

    eprintln!("stage0 audit: {}", stage0_evidence.audit_dir.display());
    eprintln!("stage0 diagnostics: {}", stage0_evidence.diagnostics_file.display());
    eprintln!("stage0 stdout: {}", stage0_evidence.stdout_file.display());
    eprintln!("stage0 stderr: {}", stage0_evidence.stderr_file.display());
    assert_stage_success(&stage0_evidence);

    // Find the stage1 binary.
    let stage1_binary = find_crunch_binary(&store);
    assert!(
        stage1_binary.is_some(),
        "stage0 should produce *-crunch/bin/crunch.\n{}",
        stage_context(&stage0_evidence),
    );
    let stage1_binary = stage1_binary.unwrap();
    eprintln!("stage1 binary: {}", stage1_binary.display());

    // Verify stage1 runs.
    let stage1_help = std::process::Command::new(&stage1_binary)
        .arg("--help")
        .output()
        .unwrap_or_else(|err| panic!("stage1 binary should run: {err}\n{}", stage_context(&stage0_evidence)));
    let stage1_help_stderr = String::from_utf8_lossy(&stage1_help.stderr);
    assert!(
        stage1_help.status.success(),
        "stage1 --help failed.\n{}\nhelp stderr:\n{}",
        stage_context(&stage0_evidence),
        stage1_help_stderr,
    );

    // ── Verify bootstrap tools on disk (spec requirement) ─────

    let bwrap_bin = find_bwrap_on_disk(&store);
    assert!(
        bwrap_bin.is_some(),
        "*-bwrap/bin/bwrap must exist on disk after stage0.\n{}",
        stage_context(&stage0_evidence),
    );
    let bwrap_bin = bwrap_bin.unwrap();
    #[cfg(unix)]
    assert!(
        is_executable(&bwrap_bin),
        "bwrap binary must be executable: {}\n{}",
        bwrap_bin.display(),
        stage_context(&stage0_evidence),
    );
    eprintln!("bwrap on disk: {}", bwrap_bin.display());

    let busybox_bin = find_busybox_on_disk(&store);
    assert!(
        busybox_bin.is_some(),
        "*-busybox/bin/busybox must exist on disk after stage0.\n{}",
        stage_context(&stage0_evidence),
    );
    let busybox_bin = busybox_bin.unwrap();
    #[cfg(unix)]
    assert!(
        is_executable(&busybox_bin),
        "busybox binary must be executable: {}\n{}",
        busybox_bin.display(),
        stage_context(&stage0_evidence),
    );
    eprintln!("busybox on disk: {}", busybox_bin.display());

    // ── Prepare for Stage 2 ─────────────────────────────────────

    // The store has the root output (*-crunch) on disk.
    // Intermediate bootstrap deps (bwrap, busybox, gcc, etc.) are in
    // castore/PathInfo but NOT exported to the --store directory
    // (only root outputs get exported to disk).
    //
    // Copy the stage1 binary out before invalidation, since it lives
    // inside the *-crunch directory we're about to remove.

    let stage1_copy = proof_dir.path().join("stage1-crunch");
    std::fs::copy(&stage1_binary, &stage1_copy)
        .unwrap_or_else(|err| panic!("copy stage1 binary out of store: {err}\n{}", stage_context(&stage0_evidence)));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&stage1_copy, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let stage1_binary = stage1_copy;

    // Remove *-crunch so stage2 must rebuild the final binary.
    // Stage2 reuses the castore/PathInfo cache for intermediate deps.
    let removed = remove_crunch_outputs(&store);
    assert!(removed >= 1, "should have removed at least 1 *-crunch dir");
    assert!(find_crunch_binary(&store).is_none(), "crunch output should be gone after invalidation",);

    // Fresh state dir so pathinfo.redb doesn't give a false cache hit
    // on the final crunch output.
    let stage2_state = proof_dir.path().join("state2");
    std::fs::create_dir_all(&stage2_state).unwrap();

    // ── Stage 2: stage1 binary rebuilds crunch ──────────────────

    eprintln!("\n=== PROOF: Stage 2 (stage1 -> stage2) ===\n");

    let stage2_command = vec![
        stage1_binary.display().to_string(),
        "--verbose".to_string(),
        "--log-level".to_string(),
        "info".to_string(),
        "--store".to_string(),
        store.display().to_string(),
        "--state-dir".to_string(),
        stage2_state.display().to_string(),
        "--nix-compat".to_string(),
        "self-build".to_string(),
        "--no-substitute".to_string(),
        "-j".to_string(),
        "4".to_string(),
    ];
    eprintln!("stage2 store: {}", store.display());
    eprintln!("stage2 state: {}", stage2_state.display());
    let mut stage2_process = std::process::Command::new(&stage1_binary);
    stage2_process.args(&stage2_command[1..]);
    let stage2 = run_command_live(proof_dir.path(), "stage2", &mut stage2_process).unwrap_or_else(|err| {
        panic!(
            "stage2 should execute: {err}\n{}",
            pre_stage_context("stage2", &stage2_command, &store, &stage2_state),
        )
    });
    assert_eq!(stage2.stdout_file, stage_stream_file(proof_dir.path(), "stage2", "stdout"));
    assert_eq!(stage2.stderr_file, stage_stream_file(proof_dir.path(), "stage2", "stderr"));
    let stage2_evidence =
        record_stage_evidence(proof_dir.path(), "stage2", stage2_command, stage2.output, &store, &stage2_state);

    eprintln!("stage2 audit: {}", stage2_evidence.audit_dir.display());
    eprintln!("stage2 diagnostics: {}", stage2_evidence.diagnostics_file.display());
    eprintln!("stage2 stdout: {}", stage2_evidence.stdout_file.display());
    eprintln!("stage2 stderr: {}", stage2_evidence.stderr_file.display());
    assert_stage_success(&stage2_evidence);

    // ── Verify stage2 output ────────────────────────────────────

    let stage2_binary = find_crunch_binary(&store);
    assert!(
        stage2_binary.is_some(),
        "stage2 should produce *-crunch/bin/crunch.\n{}",
        stage_context(&stage2_evidence),
    );
    let stage2_binary = stage2_binary.unwrap();
    eprintln!("stage2 binary: {}", stage2_binary.display());

    // Stage2 binary should run.
    let stage2_help = std::process::Command::new(&stage2_binary)
        .arg("--help")
        .output()
        .unwrap_or_else(|err| panic!("stage2 binary should run: {err}\n{}", stage_context(&stage2_evidence)));
    let stage2_help_stderr = String::from_utf8_lossy(&stage2_help.stderr);
    assert!(
        stage2_help.status.success(),
        "stage2 --help failed.\n{}\nhelp stderr:\n{}",
        stage_context(&stage2_evidence),
        stage2_help_stderr,
    );
    let stage2_stdout = String::from_utf8_lossy(&stage2_help.stdout);
    assert!(
        stage2_stdout.contains("crunch"),
        "stage2 --help should mention crunch.\n{}",
        stage_context(&stage2_evidence),
    );

    // ── Verify proof markers ────────────────────────────────────

    // Stage2 was driven by stage1 binary, not the checkout binary.
    let s2_invoking = extract_proof_field(&stage2_evidence.stderr, "invoking-binary");
    assert!(
        s2_invoking.is_some(),
        "stage2 should emit invoking-binary proof line.\n{}",
        stage_context(&stage2_evidence),
    );
    let s2_invoking_path = PathBuf::from(s2_invoking.unwrap());
    // The invoking binary MUST be the stage1 binary we found earlier.
    // current_exe() may resolve symlinks or return a different
    // representation, so canonicalize both before comparing.
    let stage1_canonical = std::fs::canonicalize(&stage1_binary).unwrap_or_else(|_| stage1_binary.clone());
    let invoking_canonical = std::fs::canonicalize(&s2_invoking_path).unwrap_or_else(|_| s2_invoking_path.clone());
    assert_eq!(
        invoking_canonical,
        stage1_canonical,
        "stage2 invoking binary must be the stage1 binary.\n\
         invoking: {}\n\
         stage1:   {}\n{}",
        invoking_canonical.display(),
        stage1_canonical.display(),
        stage_context(&stage2_evidence),
    );

    // Stage2 MUST use crunch-built bwrap. The self-build pipeline
    // now exports bwrap and busybox as separate root builds (step 2/4)
    // so they land on disk in the --store directory.
    let s2_bwrap = extract_proof_field(&stage2_evidence.stderr, "bwrap-source");
    assert!(
        s2_bwrap.is_some(),
        "stage2 should emit bwrap-source proof line.\n{}",
        stage_context(&stage2_evidence),
    );
    let bwrap_val = s2_bwrap.unwrap();
    assert!(
        bwrap_val.starts_with("crunch-built:"),
        "stage2 bwrap must be crunch-built, got: {bwrap_val}.\n{}",
        stage_context(&stage2_evidence),
    );

    // Stage2 MUST find crunch-built busybox on disk.
    let s2_busybox = extract_proof_field(&stage2_evidence.stderr, "busybox-path");
    assert!(
        s2_busybox.is_some(),
        "stage2 should emit busybox-path proof line.\n{}",
        stage_context(&stage2_evidence),
    );
    assert_ne!(
        s2_busybox.unwrap(),
        "none",
        "stage2 must have a crunch-built busybox, not none.\n{}",
        stage_context(&stage2_evidence),
    );

    // Output binary recorded.
    let s2_output = extract_proof_field(&stage2_evidence.stderr, "output-binary");
    assert!(
        s2_output.is_some(),
        "stage2 should emit output-binary proof line.\n{}",
        stage_context(&stage2_evidence),
    );

    eprintln!("\n=== PROOF PASSED ===");
    eprintln!("stage1: {}", stage1_binary.display());
    eprintln!("stage2: {}", stage2_binary.display());
    eprintln!("bwrap:  {bwrap_val}");
    eprintln!("busybox: {}", s2_busybox.unwrap());
}
