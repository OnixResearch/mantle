use std::fs;
use std::io::Read as _;
use std::io::Seek as _;
use std::os::fd::AsRawFd as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use serde_json::Value;

const TICKET_KEY_ONE: &str = "ticket-key-1:QUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUE";
const TICKET_KEY_TWO: &str = "ticket-key-2:UlJSUlJSUlJSUlJSUlJSUlJSUlJSUlJSUlJSUlJSUlI";
const RESULT_SIGNING_KEY: &str =
    "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";
const SYSTEMD_PROVIDER: &str = "systemd-credential://";
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
const PRIVATE_FILE_MODE: u32 = 0o600;
const LEGACY_DIRECTORY_MODE: u32 = 0o755;
const LEGACY_FILE_MODE: u32 = 0o644;
const EXECUTABLE_FILE_MODE: u32 = 0o755;
const FILE_MODE_MASK: u32 = 0o777;
const LEGACY_CREATED_UNIX_S: u64 = 1;
const LEGACY_EXPIRES_UNIX_S: u64 = 100;
const LEGACY_BUILD_TIME_SECS: u64 = 10;
const LEGACY_UPLOAD_BYTES: u64 = 10;
const CREATE_TTL_SECS: &str = "300";
const CREATE_USES: &str = "2";
const PROVIDER_TIMEOUT_SECS_MAX: u64 = 20;
const FAKE_SOPS_SLEEP_SECS: u64 = 60;
const FAKE_SOPS_OUTPUT_BYTES: usize = 70_000;
const REMOTE_TICKET_INPUT_LIMIT_BYTES: usize = 256;
const TEST_CUSTOM_BUILDER_PROGRAM: &str = "/bin/false";

#[test]
fn ticket_creation_delivers_only_to_fd_and_persists_verifier_only_state() {
    let root = tempfile::tempdir().unwrap();
    let credentials_dir = write_systemd_credentials(root.path(), TICKET_KEY_ONE);
    let issuance_started_unix_s = test_unix_time_now_s();
    let (output, mut delivery_file) =
        create_ticket_command(root.path(), &credentials_dir, SYSTEMD_PROVIDER, "production");
    let issuance_finished_unix_s = test_unix_time_now_s();

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let ticket_id = report["ticket"]["id"].as_str().unwrap();
    let credential = read_delivery(&mut delivery_file);
    let token = credential.split_once(':').unwrap().1;
    let state_path = root.path().join("remote-builders/tickets.json");
    let state = fs::read_to_string(&state_path).unwrap();
    let list = mantle_command()
        .args(["--json", "--state-dir"])
        .arg(root.path())
        .args(["remote", "ticket", "list"])
        .output()
        .unwrap();
    let status = mantle_command()
        .args(["--json", "--state-dir"])
        .arg(root.path())
        .args(["remote", "status"])
        .output()
        .unwrap();

    assert!(credential.starts_with(&format!("{ticket_id}:")));
    let created_unix_s = report["ticket"]["created_unix_s"].as_u64().unwrap();
    assert!(created_unix_s >= issuance_started_unix_s);
    assert!(created_unix_s <= issuance_finished_unix_s);
    assert_eq!(report["delivery"]["target"], "caller-owned-fd");
    assert!(!String::from_utf8_lossy(&output.stdout).contains(token));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(token));
    assert!(!state.contains(token));
    assert!(!state.contains("\"secret\""));
    assert_secret_markers_absent(token, &output.stdout);
    assert_secret_markers_absent(token, &output.stderr);
    assert_secret_markers_absent(token, &list.stdout);
    assert_secret_markers_absent(token, &list.stderr);
    assert_secret_markers_absent(token, &status.stdout);
    assert_secret_markers_absent(token, &status.stderr);
    assert!(state.contains("\"verifier\""));
    assert_eq!(file_mode(&state_path), PRIVATE_FILE_MODE);
    assert_eq!(file_mode(state_path.parent().unwrap()), PRIVATE_DIRECTORY_MODE);
}

#[test]
fn build_ticket_fd_accepts_valid_input_and_rejects_invalid_oversized_and_terminal_inputs() {
    let valid_credential = format!("ticket-1:{}\n", "QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI");
    let mut valid_file = tempfile::tempfile().unwrap();
    use std::io::Write as _;
    valid_file.write_all(valid_credential.as_bytes()).unwrap();
    valid_file.rewind().unwrap();
    inherit_fd(valid_file.as_raw_fd());
    let valid = build_with_ticket_fd(valid_file.as_raw_fd());
    assert!(!valid.status.success());
    assert!(!String::from_utf8_lossy(&valid.stderr).contains("remote-ticket-input"));
    assert!(!String::from_utf8_lossy(&valid.stderr).contains(valid_credential.trim_end()));

    let mut invalid_file = tempfile::tempfile().unwrap();
    invalid_file.write_all(b"invalid-ticket").unwrap();
    invalid_file.rewind().unwrap();
    inherit_fd(invalid_file.as_raw_fd());
    let invalid = build_with_ticket_fd(invalid_file.as_raw_fd());
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("remote-ticket-token-missing-colon"));
    assert!(!String::from_utf8_lossy(&invalid.stderr).contains("invalid-ticket"));

    let mut oversized_file = tempfile::tempfile().unwrap();
    let oversized_bytes = REMOTE_TICKET_INPUT_LIMIT_BYTES.checked_add(1).unwrap();
    oversized_file.write_all(&vec![b'x'; oversized_bytes]).unwrap();
    oversized_file.rewind().unwrap();
    inherit_fd(oversized_file.as_raw_fd());
    let oversized = build_with_ticket_fd(oversized_file.as_raw_fd());
    assert!(!oversized.status.success());
    assert!(String::from_utf8_lossy(&oversized.stderr).contains("remote-ticket-input-size-limit-exceeded"));

    let terminal_fd = unsafe { libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY) };
    assert!(terminal_fd > libc::STDERR_FILENO);
    assert_eq!(unsafe { libc::grantpt(terminal_fd) }, 0);
    assert_eq!(unsafe { libc::unlockpt(terminal_fd) }, 0);
    inherit_fd(terminal_fd);
    let terminal = build_with_ticket_fd(terminal_fd);
    assert_eq!(unsafe { libc::close(terminal_fd) }, 0);
    assert!(!terminal.status.success());
    assert!(String::from_utf8_lossy(&terminal.stderr).contains("remote-ticket-input-terminal-forbidden"));
    assert!(terminal.stdout.is_empty());
}

#[test]
fn rejected_provider_fails_before_ticket_state_or_delivery_changes() {
    let root = tempfile::tempdir().unwrap();
    let credentials_dir = write_systemd_credentials(root.path(), TICKET_KEY_ONE);
    let (output, mut delivery_file) = create_ticket_command(root.path(), &credentials_dir, "env://", "production");

    assert!(!output.status.success());
    assert!(read_delivery(&mut delivery_file).is_empty());
    assert!(!root.path().join("remote-builders/tickets.json").exists());
    assert!(!String::from_utf8_lossy(&output.stderr).contains(TICKET_KEY_ONE));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(RESULT_SIGNING_KEY));
}

#[test]
fn explicit_legacy_migration_invalidates_plaintext_tickets() {
    let root = tempfile::tempdir().unwrap();
    write_legacy_state(root.path());
    let dry_run = mantle_command()
        .args(["--json", "--state-dir"])
        .arg(root.path())
        .args([
            "remote",
            "ticket",
            "migrate-legacy",
            "--invalidate-legacy-tickets",
            "--dry-run",
        ])
        .output()
        .unwrap();
    let legacy_path = root.path().join("remote-builders/tickets.json");
    assert!(dry_run.status.success(), "stderr={}", String::from_utf8_lossy(&dry_run.stderr));
    assert!(fs::read_to_string(&legacy_path).unwrap().contains("legacy-plaintext-token"));

    let executed = mantle_command()
        .args(["--json", "--state-dir"])
        .arg(root.path())
        .args(["remote", "ticket", "migrate-legacy", "--invalidate-legacy-tickets"])
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&executed.stdout).unwrap();
    let migrated = fs::read_to_string(&legacy_path).unwrap();

    assert!(executed.status.success(), "stderr={}", String::from_utf8_lossy(&executed.stderr));
    assert_secret_markers_absent("legacy-plaintext-token", &executed.stdout);
    assert_secret_markers_absent("legacy-plaintext-token", &executed.stderr);
    assert_eq!(report["invalidated_ticket_count"], 1);
    assert_eq!(report["replacement_issuance_required"], true);
    assert!(!migrated.contains("legacy-plaintext-token"));
    assert!(!migrated.contains("\"secret\""));
    assert!(migrated.contains("\"schema_version\": 2"));
    assert_eq!(file_mode(&legacy_path), PRIVATE_FILE_MODE);
    assert_eq!(file_mode(legacy_path.parent().unwrap()), PRIVATE_DIRECTORY_MODE);
}

#[test]
fn key_rotation_invalidates_tickets_from_retired_verifier_key() {
    let root = tempfile::tempdir().unwrap();
    let credentials_dir = write_systemd_credentials(root.path(), TICKET_KEY_ONE);
    let (created, mut delivery_file) =
        create_ticket_command(root.path(), &credentials_dir, SYSTEMD_PROVIDER, "production");
    assert!(created.status.success(), "stderr={}", String::from_utf8_lossy(&created.stderr));
    let credential = read_delivery(&mut delivery_file);
    let ticket_id = credential.split_once(':').unwrap().0;
    write_systemd_credentials(root.path(), TICKET_KEY_TWO);

    let rotated = mantle_command()
        .args(["--json", "--state-dir"])
        .arg(root.path())
        .args(["remote", "ticket", "rotate-keys", "--secret-manifest"])
        .arg(secret_manifest())
        .args(["--secret-profile", "rotation", "--secret-provider", SYSTEMD_PROVIDER])
        .env("CREDENTIALS_DIRECTORY", &credentials_dir)
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&rotated.stdout).unwrap();
    let state: Value =
        serde_json::from_slice(&fs::read(root.path().join("remote-builders/tickets.json")).unwrap()).unwrap();

    assert!(rotated.status.success(), "stderr={}", String::from_utf8_lossy(&rotated.stderr));
    assert_eq!(report["active_ticket_verifier_key_id"], "ticket-key-2");
    assert!(report["invalidated_ticket_ids"].as_array().unwrap().iter().any(|id| id == ticket_id));
    assert_eq!(state["tickets"][ticket_id]["revoked"], true);
    assert!(!state.to_string().contains(credential.split_once(':').unwrap().1));
}

#[test]
#[ignore = "requires a real sops executable in PATH; run with --include-ignored"]
fn real_age_encrypted_sops_bootstrap_and_rotation_use_bounded_worker() {
    let root = tempfile::tempdir().unwrap();
    let provider = real_sops_provider();
    let (mut command, mut delivery_file) = base_create_command(root.path(), &provider, "bootstrap");
    let created = command.output().unwrap();
    assert!(created.status.success(), "stderr={}", String::from_utf8_lossy(&created.stderr));

    let report: Value = serde_json::from_slice(&created.stdout).unwrap();
    let ticket_id = report["ticket"]["id"].as_str().unwrap();
    let credential = read_delivery(&mut delivery_file);
    assert!(credential.starts_with(&format!("{ticket_id}:")));
    let bearer = credential.split_once(':').unwrap().1;
    assert_secret_markers_absent(bearer, &created.stdout);
    assert_secret_markers_absent(bearer, &created.stderr);

    let rotated = mantle_command()
        .args(["--json", "--state-dir"])
        .arg(root.path())
        .args(["remote", "ticket", "rotate-keys", "--secret-manifest"])
        .arg(secret_manifest())
        .args(["--secret-profile", "rotation", "--secret-provider", &provider])
        .output()
        .unwrap();
    assert!(rotated.status.success(), "stderr={}", String::from_utf8_lossy(&rotated.stderr));
    let rotation: Value = serde_json::from_slice(&rotated.stdout).unwrap();
    assert_eq!(rotation["active_ticket_verifier_key_id"], "ticket-key-2");
    assert!(rotation["invalidated_ticket_ids"].as_array().unwrap().iter().any(|id| id == ticket_id));
    let state = fs::read_to_string(root.path().join("remote-builders/tickets.json")).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&state).unwrap()["tickets"][ticket_id]["revoked"], true);
    assert!(!state.contains(bearer));
    for output in [&rotated.stdout, &rotated.stderr] {
        assert_secret_markers_absent(bearer, output);
    }
}

#[test]
fn hanging_sops_provider_is_killed_without_state_or_secret_output() {
    let root = tempfile::tempdir().unwrap();
    let fake_bin = root.path().join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    write_fake_sops(&fake_bin.join("sops"), FakeSopsMode::Sleep);
    let encrypted = root.path().join("bootstrap.enc.json");
    fs::write(&encrypted, b"encrypted-fixture").unwrap();
    let provider = format!("sops://{}?format=json", encrypted.display());
    let (mut command, mut delivery_file) = base_create_command(root.path(), &provider, "bootstrap");
    command.env("PATH", path_with_fake_bin(&fake_bin));
    let started = Instant::now();
    let output = command.output().unwrap();

    assert!(!output.status.success());
    assert!(started.elapsed() <= Duration::from_secs(PROVIDER_TIMEOUT_SECS_MAX));
    assert!(read_delivery(&mut delivery_file).is_empty());
    assert!(!root.path().join("remote-builders/tickets.json").exists());
    for output in [&output.stdout, &output.stderr] {
        assert_secret_markers_absent(TICKET_KEY_ONE, output);
    }
}

#[test]
fn oversized_sops_output_is_rejected_without_state_write() {
    let root = tempfile::tempdir().unwrap();
    let fake_bin = root.path().join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    write_fake_sops(&fake_bin.join("sops"), FakeSopsMode::OversizedOutput);
    let encrypted = root.path().join("bootstrap.enc.json");
    fs::write(&encrypted, b"encrypted-fixture").unwrap();
    let provider = format!("sops://{}?format=json", encrypted.display());
    let (mut command, mut delivery_file) = base_create_command(root.path(), &provider, "bootstrap");
    command.env("PATH", path_with_fake_bin(&fake_bin));
    let output = command.output().unwrap();

    assert!(!output.status.success());
    assert!(read_delivery(&mut delivery_file).is_empty());
    assert!(!root.path().join("remote-builders/tickets.json").exists());
    for output in [&output.stdout, &output.stderr] {
        assert_secret_markers_absent(TICKET_KEY_ONE, output);
    }
}

fn build_with_ticket_fd(ticket_fd: i32) -> std::process::Output {
    mantle_command()
        .args([
            "build",
            "missing-remote-build-input.ncl",
            "--no-substitute",
            "--builder",
            "test-builder",
            "--builder-program",
            TEST_CUSTOM_BUILDER_PROGRAM,
            "--trusted-builder-key",
            "test-builder-key",
            "--ticket-fd",
            &ticket_fd.to_string(),
        ])
        .output()
        .unwrap()
}

fn inherit_fd(fd: i32) {
    assert_eq!(unsafe { libc::fcntl(fd, libc::F_SETFD, 0) }, 0);
}

fn create_ticket_command(
    state_dir: &Path,
    credentials_dir: &Path,
    provider: &str,
    profile: &str,
) -> (std::process::Output, fs::File) {
    let (mut command, delivery_file) = base_create_command(state_dir, provider, profile);
    command.env("CREDENTIALS_DIRECTORY", credentials_dir);
    (command.output().unwrap(), delivery_file)
}

fn base_create_command(state_dir: &Path, provider: &str, profile: &str) -> (Command, fs::File) {
    let delivery_file = tempfile::tempfile().unwrap();
    let delivery_fd = delivery_file.as_raw_fd();
    let clear_cloexec = unsafe { libc::fcntl(delivery_fd, libc::F_SETFD, 0) };
    assert_eq!(clear_cloexec, 0);
    let mut command = mantle_command();
    command
        .args(["--json", "--state-dir"])
        .arg(state_dir)
        .args([
            "remote",
            "ticket",
            "create",
            "--display-name",
            "credential-boundary-test",
            "--ttl-secs",
            CREATE_TTL_SECS,
            "--uses",
            CREATE_USES,
            "--ticket-fd",
            &delivery_fd.to_string(),
            "--secret-manifest",
        ])
        .arg(secret_manifest())
        .args(["--secret-profile", profile, "--secret-provider", provider]);
    (command, delivery_file)
}

fn read_delivery(file: &mut fs::File) -> String {
    file.rewind().unwrap();
    let mut value = String::new();
    file.read_to_string(&mut value).unwrap();
    value.trim_end().to_string()
}

fn write_systemd_credentials(root: &Path, ticket_key: &str) -> PathBuf {
    let credentials_dir = root.join("credentials");
    fs::create_dir_all(&credentials_dir).unwrap();
    fs::write(credentials_dir.join("TICKET_VERIFIER_KEY"), ticket_key).unwrap();
    fs::write(credentials_dir.join("RESULT_SIGNING_KEY"), RESULT_SIGNING_KEY).unwrap();
    credentials_dir
}

fn write_legacy_state(root: &Path) {
    let ticket_dir = root.join("remote-builders");
    fs::create_dir_all(&ticket_dir).unwrap();
    fs::set_permissions(&ticket_dir, fs::Permissions::from_mode(LEGACY_DIRECTORY_MODE)).unwrap();
    let path = ticket_dir.join("tickets.json");
    let legacy = serde_json::json!({
        "tickets": {
            "legacy-ticket": {
                "id": "legacy-ticket",
                "display_name": "legacy",
                "secret": "legacy-plaintext-token",
                "created_unix_s": LEGACY_CREATED_UNIX_S,
                "expires_unix_s": LEGACY_EXPIRES_UNIX_S,
                "uses_remaining": 1,
                "max_build_time_secs": LEGACY_BUILD_TIME_SECS,
                "max_upload_bytes": LEGACY_UPLOAD_BYTES,
                "bound_client_endpoint": null,
                "revoked": false
            }
        }
    });
    fs::write(&path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(LEGACY_FILE_MODE)).unwrap();
}

fn test_unix_time_now_s() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
}

fn mantle_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mantle"))
}

fn secret_manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("secretspec.toml")
}

fn assert_secret_markers_absent(secret: &str, bytes: &[u8]) {
    let rendered = String::from_utf8_lossy(bytes);
    let secret_blake3 = blake3::hash(secret.as_bytes()).to_hex().to_string();
    let verifier_key_blake3 = blake3::hash(TICKET_KEY_ONE.as_bytes()).to_hex().to_string();
    let signing_key_blake3 = blake3::hash(RESULT_SIGNING_KEY.as_bytes()).to_hex().to_string();

    assert!(!rendered.contains(secret));
    assert!(!rendered.contains(&secret_blake3));
    assert!(!rendered.contains(&verifier_key_blake3));
    assert!(!rendered.contains(RESULT_SIGNING_KEY));
    assert!(!rendered.contains(&signing_key_blake3));
}

fn file_mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & FILE_MODE_MASK
}

enum FakeSopsMode {
    Sleep,
    OversizedOutput,
}

fn write_fake_sops(path: &Path, mode: FakeSopsMode) {
    let source = match mode {
        FakeSopsMode::Sleep => format!("#!/bin/sh\nsleep {FAKE_SOPS_SLEEP_SECS}\n"),
        FakeSopsMode::OversizedOutput => format!("#!/bin/sh\nhead -c {FAKE_SOPS_OUTPUT_BYTES} /dev/zero\n"),
    };
    fs::write(path, source).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(EXECUTABLE_FILE_MODE)).unwrap();
}

fn path_with_fake_bin(fake_bin: &Path) -> std::ffi::OsString {
    let mut paths = vec![fake_bin.to_path_buf()];
    paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()));
    std::env::join_paths(paths).unwrap()
}

fn real_sops_provider() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let encrypted = root.join("tests/fixtures/remote-ticket-sops-age.enc.json");
    let age_key = root.join("third_party/secretspec/src/provider/sops/test_fixtures/key.txt");
    format!("sops://{}?format=json&age_key_file={}", encrypted.display(), age_key.display())
}
