use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use serde::Deserialize;
use serde::Serialize;
use zeroize::Zeroize as _;

use crate::errors::RunError;
use crate::remote_credentials::TicketVerifierKey;

pub const REMOTE_SECRET_SCOPE: &str = "mantle-remote";
pub const REMOTE_SECRET_PRODUCTION_PROFILE: &str = "production";
pub const REMOTE_SECRET_BOOTSTRAP_PROFILE: &str = "bootstrap";
pub const REMOTE_SECRET_ROTATION_PROFILE: &str = "rotation";
pub const REMOTE_SYSTEMD_CREDENTIAL_PROVIDER: &str = "systemd-credential://";
pub const TICKET_VERIFIER_KEY_SECRET: &str = "TICKET_VERIFIER_KEY";
pub const RESULT_SIGNING_KEY_SECRET: &str = "RESULT_SIGNING_KEY";

const REMOTE_SECRET_WORKER_ENV: &str = "MANTLE_REMOTE_SECRET_WORKER";
const REMOTE_SECRET_WORKER_ENV_VALUE: &str = "1";
const REMOTE_SECRET_WORKER_SUBCOMMAND: &str = "remote-secret-worker";
const REMOTE_SECRET_WORKER_TIMEOUT_SECS: u64 = 15;
const REMOTE_SECRET_WORKER_POLL_MS: u64 = 10;
const REMOTE_SECRET_WORKER_STDOUT_BYTES_MAX: usize = 65_536;
const REMOTE_SECRET_WORKER_STDERR_BYTES_MAX: usize = 16_384;
const REMOTE_SECRET_VALUE_BYTES_MAX: usize = 4_096;
const REMOTE_SECRET_MANIFEST_BYTES_MAX: u64 = 65_536;
const REMOTE_SECRET_WORKER_MEMORY_MIB: u64 = 512;
const BYTES_PER_MIB: u64 = 1_048_576;
const REMOTE_SECRET_WORKER_MEMORY_BYTES: u64 = REMOTE_SECRET_WORKER_MEMORY_MIB * BYTES_PER_MIB;
const REMOTE_SECRET_WORKER_CPU_SECS: u64 = 10;
const PIPE_BUFFER_BYTES: usize = 8_192;
const WORKER_TERMINATION_WAIT_MS: u64 = 100;
const PROFILE_COUNT: usize = 3;
const REQUIRED_SECRET_COUNT: usize = 2;

pub struct RemoteServiceKeys {
    pub verifier_key: Arc<TicketVerifierKey>,
    pub result_signing_key: crunch_build::KeyPair,
}

impl std::fmt::Debug for RemoteServiceKeys {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RemoteServiceKeys")
            .field("verifier_key_id", &self.verifier_key.id())
            .field(
                "result_signing_key_id",
                &self.result_signing_key.verifying_key.name(),
            )
            .field("key_material", &"<redacted>")
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteServiceSecretRequest {
    pub manifest_path: PathBuf,
    pub profile: String,
    pub provider: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoteServiceKeyWire {
    ticket_verifier_key: String,
    result_signing_key: String,
}

impl Drop for RemoteServiceKeyWire {
    fn drop(&mut self) {
        self.ticket_verifier_key.zeroize();
        self.result_signing_key.zeroize();
    }
}

pub fn resolve_remote_service_keys_bounded(
    request: &RemoteServiceSecretRequest,
) -> Result<RemoteServiceKeys, RunError> {
    validate_remote_secret_request(request).map_err(secret_error)?;
    let current_exe = std::env::current_exe().map_err(|_| secret_error("worker-executable-unavailable"))?;
    let mut command = Command::new(&current_exe);
    command
        .arg(REMOTE_SECRET_WORKER_SUBCOMMAND)
        .arg("--manifest")
        .arg(&request.manifest_path)
        .arg("--profile")
        .arg(&request.profile)
        .arg("--provider")
        .arg(&request.provider)
        .env(REMOTE_SECRET_WORKER_ENV, REMOTE_SECRET_WORKER_ENV_VALUE)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_secret_worker_limits(&mut command);
    let child = command
        .spawn()
        .map_err(|_| secret_error("worker-spawn-failed"))?;
    let output = collect_secret_worker_output(child)?;
    if output.stdout_exceeded || output.stderr_exceeded {
        return Err(secret_error("worker-output-limit-exceeded"));
    }
    if !output.status_success {
        return Err(secret_error("worker-provider-failed"));
    }
    let wire: RemoteServiceKeyWire =
        serde_json::from_slice(&output.stdout).map_err(|_| secret_error("worker-response-malformed"))?;
    parse_remote_service_keys(&wire).map_err(secret_error)
}

pub fn run_remote_secret_worker(
    manifest_path: &Path,
    profile: &str,
    provider: &str,
) -> Result<(), RunError> {
    if std::env::var(REMOTE_SECRET_WORKER_ENV).as_deref() != Ok(REMOTE_SECRET_WORKER_ENV_VALUE) {
        return Err(secret_error("worker-invocation-rejected"));
    }
    let request = RemoteServiceSecretRequest {
        manifest_path: manifest_path.to_path_buf(),
        profile: profile.to_string(),
        provider: provider.to_string(),
    };
    validate_remote_secret_request(&request).map_err(secret_error)?;
    let wire = resolve_remote_service_key_wire(&request).map_err(secret_error)?;
    let stdout = std::io::stdout();
    let mut locked = stdout.lock();
    serde_json::to_writer(&mut locked, &wire).map_err(|_| secret_error("worker-response-serialization-failed"))?;
    locked
        .flush()
        .map_err(|_| secret_error("worker-response-write-failed"))?;
    Ok(())
}

fn resolve_remote_service_key_wire(
    request: &RemoteServiceSecretRequest,
) -> Result<RemoteServiceKeyWire, &'static str> {
    validate_remote_secret_manifest(&request.manifest_path, &request.profile)?;
    let mut secrets = secretspec::Secrets::load_from(&request.manifest_path)
        .map_err(|_| "manifest-load-failed")?;
    secrets.set_provider(request.provider.clone());
    secrets.set_profile(request.profile.clone());
    secrets.set_scope(REMOTE_SECRET_SCOPE);
    secrets.set_ignore_ambient_scope(true);
    let secrets = secrets.with_reason("Mantle remote service key resolution");
    let mut resolved = secrets.resolve().map_err(|_| "provider-resolution-failed")?;
    if !resolved.is_ok() {
        return Err("required-secret-missing");
    }
    if resolved.profile != request.profile || resolved.scope.as_deref() != Some(REMOTE_SECRET_SCOPE) {
        return Err("resolution-boundary-mismatch");
    }
    if resolved.secrets.len() != REQUIRED_SECRET_COUNT {
        return Err("resolution-secret-count-mismatch");
    }
    let ticket_verifier_key = take_inline_secret(&mut resolved.secrets, TICKET_VERIFIER_KEY_SECRET)?;
    let result_signing_key = take_inline_secret(&mut resolved.secrets, RESULT_SIGNING_KEY_SECRET)?;
    let wire = RemoteServiceKeyWire {
        ticket_verifier_key,
        result_signing_key,
    };
    parse_remote_service_keys(&wire)?;
    Ok(wire)
}

fn take_inline_secret(
    secrets: &mut std::collections::BTreeMap<String, secretspec::ResolvedSecret>,
    name: &str,
) -> Result<String, &'static str> {
    let secret = secrets.remove(name).ok_or("required-secret-missing")?;
    if secret.as_path || secret.path.is_some() {
        return Err("secret-path-materialization-forbidden");
    }
    let value = secret.value.ok_or("required-secret-value-missing")?;
    if value.is_empty() || value.len() > REMOTE_SECRET_VALUE_BYTES_MAX {
        return Err("secret-value-size-invalid");
    }
    Ok(value)
}

fn parse_remote_service_keys(wire: &RemoteServiceKeyWire) -> Result<RemoteServiceKeys, &'static str> {
    let verifier_key = TicketVerifierKey::parse(&wire.ticket_verifier_key)
        .map_err(|_| "ticket-verifier-key-invalid")?;
    let result_signing_key = crunch_build::load_keypair(&wire.result_signing_key)
        .map_err(|_| "result-signing-key-invalid")?;
    Ok(RemoteServiceKeys {
        verifier_key: Arc::new(verifier_key),
        result_signing_key,
    })
}

fn validate_remote_secret_request(request: &RemoteServiceSecretRequest) -> Result<(), &'static str> {
    let valid_profile = [
        REMOTE_SECRET_PRODUCTION_PROFILE,
        REMOTE_SECRET_BOOTSTRAP_PROFILE,
        REMOTE_SECRET_ROTATION_PROFILE,
    ]
    .contains(&request.profile.as_str());
    if !valid_profile {
        return Err("profile-not-allowed");
    }
    if request.provider == REMOTE_SYSTEMD_CREDENTIAL_PROVIDER {
        if request.profile == REMOTE_SECRET_BOOTSTRAP_PROFILE {
            return Err("bootstrap-profile-requires-sops");
        }
        return Ok(());
    }
    if request.provider.starts_with("sops://") {
        if request.profile == REMOTE_SECRET_PRODUCTION_PROFILE {
            return Err("production-profile-requires-systemd-credential");
        }
        return Ok(());
    }
    Err("provider-not-allowed")
}

fn validate_remote_secret_manifest(path: &Path, profile: &str) -> Result<(), &'static str> {
    let metadata = fs::metadata(path).map_err(|_| "manifest-inspection-failed")?;
    if !metadata.is_file() || metadata.len() > REMOTE_SECRET_MANIFEST_BYTES_MAX {
        return Err("manifest-boundary-invalid");
    }
    let source = fs::read_to_string(path).map_err(|_| "manifest-read-failed")?;
    let value: toml::Value = toml::from_str(&source).map_err(|_| "manifest-parse-failed")?;
    let root = value.as_table().ok_or("manifest-root-invalid")?;
    let project = root
        .get("project")
        .and_then(toml::Value::as_table)
        .ok_or("manifest-project-missing")?;
    if project.get("name").and_then(toml::Value::as_str) != Some("mantle") {
        return Err("manifest-project-invalid");
    }
    let profiles = root
        .get("profiles")
        .and_then(toml::Value::as_table)
        .ok_or("manifest-profiles-missing")?;
    if profiles.len() != PROFILE_COUNT {
        return Err("manifest-profile-count-invalid");
    }
    let selected = profiles
        .get(profile)
        .and_then(toml::Value::as_table)
        .ok_or("manifest-profile-missing")?;
    validate_selected_secret_declarations(selected)?;
    let scope = root
        .get("scopes")
        .and_then(toml::Value::as_table)
        .and_then(|scopes| scopes.get(REMOTE_SECRET_SCOPE))
        .and_then(toml::Value::as_table)
        .ok_or("manifest-scope-missing")?;
    let members = scope
        .get("secrets")
        .and_then(toml::Value::as_array)
        .ok_or("manifest-scope-members-invalid")?;
    let names = members
        .iter()
        .map(toml::Value::as_str)
        .collect::<Option<Vec<_>>>()
        .ok_or("manifest-scope-members-invalid")?;
    if names != [TICKET_VERIFIER_KEY_SECRET, RESULT_SIGNING_KEY_SECRET] {
        return Err("manifest-scope-members-invalid");
    }
    if contains_forbidden_secret_storage_field(&value, None) {
        return Err("manifest-secret-write-or-cache-forbidden");
    }
    Ok(())
}

fn validate_selected_secret_declarations(
    selected: &toml::map::Map<String, toml::Value>,
) -> Result<(), &'static str> {
    if selected.len() != REQUIRED_SECRET_COUNT {
        return Err("manifest-secret-count-invalid");
    }
    for name in [TICKET_VERIFIER_KEY_SECRET, RESULT_SIGNING_KEY_SECRET] {
        let declaration = selected
            .get(name)
            .and_then(toml::Value::as_table)
            .ok_or("manifest-secret-declaration-missing")?;
        if declaration.get("required").and_then(toml::Value::as_bool) != Some(true) {
            return Err("manifest-secret-not-required");
        }
        if declaration
            .get("description")
            .and_then(toml::Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Err("manifest-secret-description-missing");
        }
    }
    Ok(())
}

fn contains_forbidden_secret_storage_field(value: &toml::Value, parent_key: Option<&str>) -> bool {
    match value {
        toml::Value::Table(table) => table.iter().any(|(key, nested)| {
            let forbidden = matches!(
                key.as_str(),
                "default" | "generate" | "as_path" | "cache" | "credentials"
            );
            let provider_cache = parent_key == Some("providers") && key == "cache";
            forbidden || provider_cache || contains_forbidden_secret_storage_field(nested, Some(key))
        }),
        toml::Value::Array(values) => values
            .iter()
            .any(|nested| contains_forbidden_secret_storage_field(nested, parent_key)),
        _ => false,
    }
}

struct SecretWorkerOutput {
    stdout: zeroize::Zeroizing<Vec<u8>>,
    stdout_exceeded: bool,
    stderr_exceeded: bool,
    status_success: bool,
}

impl SecretWorkerOutput {
    fn zeroize_retained_stdout(&mut self) {
        self.stdout.zeroize();
    }
}

impl Drop for SecretWorkerOutput {
    fn drop(&mut self) {
        self.zeroize_retained_stdout();
    }
}

fn collect_secret_worker_output(mut child: std::process::Child) -> Result<SecretWorkerOutput, RunError> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| secret_error("worker-stdout-missing"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| secret_error("worker-stderr-missing"))?;
    let stdout_reader = thread::spawn(move || drain_bounded_pipe(stdout, REMOTE_SECRET_WORKER_STDOUT_BYTES_MAX));
    let stderr_reader = thread::spawn(move || drain_bounded_pipe(stderr, REMOTE_SECRET_WORKER_STDERR_BYTES_MAX));
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(REMOTE_SECRET_WORKER_TIMEOUT_SECS))
        .ok_or_else(|| secret_error("worker-deadline-overflow"))?;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|_| secret_error("worker-wait-failed"))?
        {
            break status;
        }
        if Instant::now() >= deadline {
            terminate_secret_worker_tree(&mut child)?;
            return Err(secret_error("worker-timeout"));
        }
        thread::sleep(Duration::from_millis(REMOTE_SECRET_WORKER_POLL_MS));
    };
    terminate_secret_worker_descendants(child.id())?;
    let _reaped_status = child
        .wait()
        .map_err(|_| secret_error("worker-reap-failed"))?;
    let (stdout, stdout_exceeded) = stdout_reader
        .join()
        .map_err(|_| secret_error("worker-stdout-reader-failed"))?
        .map_err(|_| secret_error("worker-stdout-read-failed"))?;
    let (_stderr, stderr_exceeded) = stderr_reader
        .join()
        .map_err(|_| secret_error("worker-stderr-reader-failed"))?
        .map_err(|_| secret_error("worker-stderr-read-failed"))?;
    Ok(SecretWorkerOutput {
        stdout,
        stdout_exceeded,
        stderr_exceeded,
        status_success: status.success(),
    })
}

fn drain_bounded_pipe(
    mut reader: impl std::io::Read,
    retained_bytes_max: usize,
) -> std::io::Result<(zeroize::Zeroizing<Vec<u8>>, bool)> {
    let mut retained = zeroize::Zeroizing::new(Vec::with_capacity(retained_bytes_max));
    let mut exceeded = false;
    let mut buffer = zeroize::Zeroizing::new([0_u8; PIPE_BUFFER_BYTES]);
    loop {
        let read_bytes = reader.read(&mut *buffer)?;
        if read_bytes == 0 {
            break;
        }
        let remaining = retained_bytes_max.saturating_sub(retained.len());
        let retained_now = remaining.min(read_bytes);
        retained.extend_from_slice(&buffer[..retained_now]);
        exceeded |= retained_now < read_bytes;
    }
    Ok((retained, exceeded))
}

#[cfg(unix)]
fn configure_secret_worker_limits(command: &mut Command) {
    use std::os::unix::process::CommandExt as _;

    // SAFETY: The closure calls only async-signal-safe libc functions before exec.
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let memory_limit = libc::rlimit {
                rlim_cur: REMOTE_SECRET_WORKER_MEMORY_BYTES,
                rlim_max: REMOTE_SECRET_WORKER_MEMORY_BYTES,
            };
            if libc::setrlimit(libc::RLIMIT_AS, &memory_limit) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let cpu_limit = libc::rlimit {
                rlim_cur: REMOTE_SECRET_WORKER_CPU_SECS,
                rlim_max: REMOTE_SECRET_WORKER_CPU_SECS,
            };
            if libc::setrlimit(libc::RLIMIT_CPU, &cpu_limit) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

#[cfg(not(unix))]
fn configure_secret_worker_limits(_command: &mut Command) {}

#[cfg(unix)]
fn terminate_secret_worker_tree(child: &mut std::process::Child) -> Result<(), RunError> {
    terminate_secret_worker_descendants(child.id())?;
    thread::sleep(Duration::from_millis(WORKER_TERMINATION_WAIT_MS));
    child
        .wait()
        .map_err(|_| secret_error("worker-reap-failed"))?;
    Ok(())
}

#[cfg(not(unix))]
fn terminate_secret_worker_tree(child: &mut std::process::Child) -> Result<(), RunError> {
    child
        .kill()
        .map_err(|_| secret_error("worker-termination-failed"))?;
    child
        .wait()
        .map_err(|_| secret_error("worker-reap-failed"))?;
    Ok(())
}

#[cfg(unix)]
fn terminate_secret_worker_descendants(pid: u32) -> Result<(), RunError> {
    let process_group = i32::try_from(pid).map_err(|_| secret_error("worker-pid-invalid"))?;
    let result = unsafe { libc::kill(-process_group, libc::SIGKILL) };
    if result == 0 {
        return Ok(());
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        return Ok(());
    }
    Err(secret_error("worker-termination-failed"))
}

#[cfg(not(unix))]
fn terminate_secret_worker_descendants(_pid: u32) -> Result<(), RunError> {
    Ok(())
}

fn secret_error(category: &str) -> RunError {
    RunError::Internal(format!("remote-service-secret-{category}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;

    const TEST_SIGNING_KEY: &str =
        "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";

    fn valid_manifest() -> String {
        format!(
            r#"[project]
name = "mantle"
revision = "1.0"
require_reason = false

[profiles.production]
{TICKET_VERIFIER_KEY_SECRET} = {{ description = "Ticket verifier key", required = true }}
{RESULT_SIGNING_KEY_SECRET} = {{ description = "Result signing key", required = true }}

[profiles.bootstrap]
{TICKET_VERIFIER_KEY_SECRET} = {{ description = "Ticket verifier key", required = true }}
{RESULT_SIGNING_KEY_SECRET} = {{ description = "Result signing key", required = true }}

[profiles.rotation]
{TICKET_VERIFIER_KEY_SECRET} = {{ description = "Ticket verifier key", required = true }}
{RESULT_SIGNING_KEY_SECRET} = {{ description = "Result signing key", required = true }}

[scopes.{REMOTE_SECRET_SCOPE}]
secrets = ["{TICKET_VERIFIER_KEY_SECRET}", "{RESULT_SIGNING_KEY_SECRET}"]
"#,
        )
    }

    #[test]
    fn manifest_policy_accepts_metadata_only_declarations() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("secretspec.toml");
        fs::write(&path, valid_manifest()).unwrap();

        assert!(validate_remote_secret_manifest(&path, REMOTE_SECRET_PRODUCTION_PROFILE).is_ok());
        assert!(validate_remote_secret_manifest(&path, REMOTE_SECRET_BOOTSTRAP_PROFILE).is_ok());
    }

    #[test]
    fn manifest_policy_rejects_values_generation_paths_and_caches() {
        for forbidden in [
            "default = \"plaintext\"",
            "generate = true",
            "as_path = true",
            "cache = { provider = \"dotenv:.cache\", max_age = \"1h\" }",
        ] {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("secretspec.toml");
            let manifest = valid_manifest().replace(
                "description = \"Ticket verifier key\", required = true",
                &format!("description = \"Ticket verifier key\", required = true, {forbidden}"),
            );
            fs::write(&path, manifest).unwrap();
            assert!(validate_remote_secret_manifest(&path, REMOTE_SECRET_PRODUCTION_PROFILE).is_err());
        }
    }

    #[test]
    fn provider_policy_is_profile_specific() {
        let path = PathBuf::from("secretspec.toml");
        assert!(
            validate_remote_secret_request(&RemoteServiceSecretRequest {
                manifest_path: path.clone(),
                profile: REMOTE_SECRET_PRODUCTION_PROFILE.to_string(),
                provider: REMOTE_SYSTEMD_CREDENTIAL_PROVIDER.to_string(),
            })
            .is_ok()
        );
        assert!(
            validate_remote_secret_request(&RemoteServiceSecretRequest {
                manifest_path: path.clone(),
                profile: REMOTE_SECRET_BOOTSTRAP_PROFILE.to_string(),
                provider: "sops://secrets.enc.json".to_string(),
            })
            .is_ok()
        );
        assert!(
            validate_remote_secret_request(&RemoteServiceSecretRequest {
                manifest_path: path,
                profile: REMOTE_SECRET_PRODUCTION_PROFILE.to_string(),
                provider: "env://".to_string(),
            })
            .is_err()
        );
    }

    #[test]
    fn service_key_parser_accepts_valid_keys_and_redacts_debug() {
        use base64::Engine as _;
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;

        let wire = RemoteServiceKeyWire {
            ticket_verifier_key: format!("ticket-key-1:{}", URL_SAFE_NO_PAD.encode([0x41_u8; 32])),
            result_signing_key: TEST_SIGNING_KEY.to_string(),
        };
        let keys = parse_remote_service_keys(&wire).unwrap();
        let debug = format!("{keys:?}");

        assert_eq!(keys.verifier_key.id(), "ticket-key-1");
        assert_eq!(keys.result_signing_key.verifying_key.name(), "cache.example.com-1");
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains(&wire.ticket_verifier_key));
        assert!(!debug.contains(TEST_SIGNING_KEY));
    }

    #[test]
    fn worker_output_cleanup_zeroizes_retained_secret_json() {
        let secret_json = br#"{"ticket_verifier_key":"secret-a","result_signing_key":"secret-b"}"#.to_vec();
        assert!(!secret_json.is_empty());
        let mut output = SecretWorkerOutput {
            stdout: zeroize::Zeroizing::new(secret_json),
            stdout_exceeded: false,
            stderr_exceeded: false,
            status_success: true,
        };

        output.zeroize_retained_stdout();

        assert!(output.stdout.is_empty());
        assert!(!output.stdout_exceeded);
        assert!(!output.stderr_exceeded);
    }

    #[test]
    fn service_key_parser_rejects_invalid_key_material() {
        let invalid_verifier = RemoteServiceKeyWire {
            ticket_verifier_key: "ticket-key-1:not-base64".to_string(),
            result_signing_key: TEST_SIGNING_KEY.to_string(),
        };
        let invalid_signing = RemoteServiceKeyWire {
            ticket_verifier_key: format!(
                "ticket-key-1:{}",
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([0x41_u8; 32])
            ),
            result_signing_key: "not-a-signing-key".to_string(),
        };

        assert!(parse_remote_service_keys(&invalid_verifier).is_err());
        assert!(parse_remote_service_keys(&invalid_signing).is_err());
    }
}
