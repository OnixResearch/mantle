use std::fs::File;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

const START_ATTEMPTS_MAX: u32 = 8;
const READY_TIMEOUT: Duration = Duration::from_secs(5);
const READY_POLL_INTERVAL: Duration = Duration::from_millis(25);
const LOOPBACK_EPHEMERAL: &str = "127.0.0.1:0";
const CONFIG_TEST_PORT: u16 = 5_000;
const KIBIBYTE_BYTES: usize = 1_024;
const PANIC_LOG_KIBIBYTES_MAX: usize = 16;
const PANIC_LOG_BYTES_MAX: usize = PANIC_LOG_KIBIBYTES_MAX * KIBIBYTE_BYTES;

pub struct DistributionRegistry {
    child: Child,
    url: String,
    version: String,
    stderr_path: PathBuf,
}

impl DistributionRegistry {
    pub fn start(binary: &Path, root: &Path) -> Self {
        assert!(binary.is_absolute(), "distribution registry binary path must be absolute");
        assert!(binary.is_file(), "distribution registry binary must exist");
        std::fs::create_dir_all(root).expect("distribution fixture root should exist");
        let version = query_version(binary);
        for attempt in 0..START_ATTEMPTS_MAX {
            if let Some(registry) = start_attempt(binary, root, attempt, &version) {
                return registry;
            }
        }
        panic!("distribution registry failed to start after {START_ATTEMPTS_MAX} attempts");
    }

    pub fn url(&self) -> &str {
        assert!(self.url.starts_with("http://127.0.0.1:"));
        assert!(!self.url.ends_with(':'));
        &self.url
    }

    pub fn version(&self) -> &str {
        assert!(!self.version.is_empty());
        assert!(self.version.contains("distribution"));
        &self.version
    }
}

impl Drop for DistributionRegistry {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        if let Err(error) = self.child.wait() {
            eprintln!("failed to reap distribution registry: {error}; stderr={}", self.stderr_path.display());
        }
        if std::thread::panicking() {
            emit_bounded_panic_log(&self.stderr_path);
        }
    }
}

fn emit_bounded_panic_log(stderr_path: &Path) {
    assert!(stderr_path.is_absolute());
    let Ok(bytes) = std::fs::read(stderr_path) else {
        return;
    };
    let retained_bytes = bytes.len().min(PANIC_LOG_BYTES_MAX);
    eprintln!("distribution registry stderr (bounded): {}", String::from_utf8_lossy(&bytes[..retained_bytes]));
}

fn start_attempt(binary: &Path, root: &Path, attempt: u32, version: &str) -> Option<DistributionRegistry> {
    assert!(attempt < START_ATTEMPTS_MAX);
    assert!(!version.is_empty());
    let reservation = TcpListener::bind(LOOPBACK_EPHEMERAL).expect("loopback port reservation should bind");
    let address = reservation.local_addr().expect("reserved loopback address should exist");
    let config_path = root.join(format!("distribution-{attempt}.yml"));
    let storage_path = root.join(format!("storage-{attempt}"));
    let stdout_path = root.join(format!("distribution-{attempt}.stdout"));
    let stderr_path = root.join(format!("distribution-{attempt}.stderr"));
    std::fs::create_dir_all(&storage_path).expect("distribution storage should exist");
    std::fs::write(&config_path, render_config(&storage_path, address.port()))
        .expect("distribution config should be written");
    let stdout = File::create(&stdout_path).expect("distribution stdout capture should open");
    let stderr = File::create(&stderr_path).expect("distribution stderr capture should open");
    drop(reservation);
    let mut child = Command::new(binary)
        .args(["serve", config_path.to_str().expect("config path should be UTF-8")])
        .env("OTEL_TRACES_EXPORTER", "none")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .expect("distribution registry should launch");
    if wait_until_ready(&mut child, address) {
        return Some(DistributionRegistry {
            child,
            url: format!("http://{address}"),
            version: version.to_owned(),
            stderr_path,
        });
    }
    let _ = child.kill();
    let _ = child.wait();
    emit_bounded_panic_log(&stdout_path);
    emit_bounded_panic_log(&stderr_path);
    None
}

fn wait_until_ready(child: &mut Child, address: std::net::SocketAddr) -> bool {
    let deadline = Instant::now() + READY_TIMEOUT;
    while Instant::now() < deadline {
        if TcpStream::connect(address).is_ok() {
            return child.try_wait().ok().flatten().is_none();
        }
        if child.try_wait().ok().flatten().is_some() {
            return false;
        }
        thread::sleep(READY_POLL_INTERVAL);
    }
    false
}

fn render_config(storage_path: &Path, port: u16) -> String {
    assert!(storage_path.is_absolute(), "distribution storage path must be absolute");
    assert!(port > 0, "distribution port must be nonzero");
    format!(
        "version: 0.1\nlog:\n  level: error\nstorage:\n  filesystem:\n    rootdirectory: {}\nhttp:\n  addr: 127.0.0.1:{}\n",
        storage_path.display(),
        port
    )
}

fn query_version(binary: &Path) -> String {
    let output = Command::new(binary).arg("--version").output().expect("distribution registry version should run");
    assert!(output.status.success(), "distribution registry version should succeed");
    let version = String::from_utf8(output.stdout).expect("distribution version should be UTF-8");
    let version = normalize_version(binary, &version);
    assert!(!version.is_empty(), "distribution version must not be empty");
    version
}

fn normalize_version(binary: &Path, raw: &str) -> String {
    assert!(binary.is_absolute());
    assert!(!raw.trim().is_empty());
    let executable_prefix = format!("{} ", binary.display());
    raw.trim().strip_prefix(&executable_prefix).unwrap_or(raw.trim()).to_owned()
}

#[test]
fn distribution_config_is_bounded_to_loopback_filesystem_storage() {
    let root = Path::new("/tmp/mantle-distribution-fixture");
    let config = render_config(root, CONFIG_TEST_PORT);
    assert!(config.contains("rootdirectory: /tmp/mantle-distribution-fixture"));
    assert!(config.contains(&format!("addr: 127.0.0.1:{CONFIG_TEST_PORT}")));
    assert!(!config.contains("auth:"));
    assert!(!config.contains("proxy:"));
}

#[test]
fn distribution_version_normalization_removes_only_the_local_executable_path() {
    let binary = Path::new("/nix/store/example-distribution/bin/registry");
    let version = "github.com/distribution/distribution/v3 v3.1.0+unknown";
    let raw = format!("{} {version}\n", binary.display());
    assert_eq!(normalize_version(binary, &raw), version);
    assert_eq!(normalize_version(binary, version), version);
}
