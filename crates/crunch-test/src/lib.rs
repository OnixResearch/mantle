//! Std adapter for a libtest derivation builder. Not a sandbox or store authority.
use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crunch_test_app::Capture;
use crunch_test_app::TestPort;
use crunch_test_core::CoverageReport;
pub use crunch_test_core::Invocation;
pub use crunch_test_core::Outcome;
pub use crunch_test_core::Report;
pub use crunch_test_core::SelectedRequest;
pub use crunch_test_core::Target;
pub use crunch_test_core::Tool;
use crunch_test_core::select;
type Result<T, E = String> = std::result::Result<T, E>;
use serde::Serialize;

const MAX_TARGET_BYTES: u64 = 262_144;
const MAX_TOOL_BYTES: u64 = 536_870_912;
static CANCELLED: AtomicBool = AtomicBool::new(false);
extern "C" fn cancel(_: libc::c_int) {
    CANCELLED.store(true, Ordering::Relaxed);
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn load_target(path: &Path, invocation: Invocation) -> Result<SelectedRequest> {
    let bytes = bounded(path, MAX_TARGET_BYTES)?;
    let target = serde_json::from_slice(&bytes).map_err(err)?;
    select(target, invocation)
}
fn bounded(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path).map_err(err)?.take(limit + 1).read_to_end(&mut bytes).map_err(err)?;
    if bytes.len() as u64 > limit {
        return Err("test-file-limit".into());
    }
    Ok(bytes)
}
fn executable(target: &Target, tool: &Tool) -> Result<PathBuf> {
    let root = fs::canonicalize(&tool.root).map_err(err)?;
    let path = fs::canonicalize(Path::new(&tool.root).join(&tool.member)).map_err(err)?;
    // Symlinked binaries may resolve into another declared runtime root.
    if !path.starts_with(&root) && !target.inputs.keys().any(|r| path.starts_with(r)) {
        return Err("test-tool-symlink-undeclared".into());
    }
    let mut file = File::open(&path).map_err(err)?;
    if file.metadata().map_err(err)?.len() > MAX_TOOL_BYTES {
        return Err("test-tool-byte-limit".into());
    }
    let mut hash = blake3::Hasher::new();
    let mut buffer = [0_u8; 65_536];
    loop {
        let count = file.read(&mut buffer).map_err(err)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    if hash.finalize().to_hex().as_str() != tool.blake3 {
        return Err("test-tool-digest-mismatch".into());
    }
    Ok(path)
}
#[derive(Serialize)]
pub struct Derivation {
    pub name: String,
    pub builder: String,
    pub args: Vec<String>,
    pub system: String,
    pub outputs: Vec<String>,
    pub addressing_mode: String,
    pub inputs: Vec<String>,
    pub env: BTreeMap<String, String>,
}
pub struct LoweredRequest {
    pub request: SelectedRequest,
    pub derivation: Derivation,
}
/// Caller must admit every input's declared NAR identity before scheduling this.
pub fn lower(request: SelectedRequest) -> Result<LoweredRequest> {
    let target = &request.target;
    let invocation = &request.invocation;
    let derivation = Derivation {
        name: format!("test-{}-{}", target.name, &request.identity[..16]),
        builder: format!("{}/{}", target.runner.root, target.runner.member),
        args: vec!["run".into()],
        system: target.platform.clone(),
        outputs: vec!["out".into()],
        addressing_mode: "input-addressed".into(),
        inputs: target.inputs.keys().cloned().collect(),
        env: BTreeMap::from([
            ("MANTLE_TEST_TARGET".into(), serde_json::to_string(target).map_err(err)?),
            ("MANTLE_TEST_INVOCATION".into(), serde_json::to_string(invocation).map_err(err)?),
        ]),
    };
    Ok(LoweredRequest { request, derivation })
}
/// Write a single ordinary recipe, with no ambient import and no shell escaping.
pub fn write_recipe(request: SelectedRequest, path: &Path) -> Result<SelectedRequest> {
    let LoweredRequest { request, derivation } = lower(request)?;
    let value = serde_json::to_value(&derivation).map_err(err)?;
    fn nickel(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Object(fields) => format!(
                "{{ {} }}",
                fields
                    .iter()
                    .map(|(k, v)| format!("{} = {}", serde_json::to_string(k).unwrap(), nickel(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            serde_json::Value::Array(items) => format!("[{}]", items.iter().map(nickel).collect::<Vec<_>>().join(", ")),
            serde_json::Value::String(_) => value.to_string().replace("%{", "\\%{"),
            _ => value.to_string(),
        }
    }
    let mut file = OpenOptions::new().create_new(true).write(true).open(path).map_err(err)?;
    writeln!(file, "{}", nickel(&value)).map_err(err)?;
    Ok(request)
}
/// Parse only an output already admitted by Mantle's existing signed-result path.
/// This function provides structural/identity checks, NOT signature admission.
pub fn read_report(root: &Path, request: SelectedRequest) -> Result<Report> {
    let SelectedRequest {
        target,
        invocation,
        selected,
        identity,
    } = request;
    let mut report: Report = serde_json::from_slice(&bounded(&root.join("report.json"), 1_048_576)?).map_err(err)?;
    if report.schema != crunch_test_core::REPORT_SCHEMA
        || report.identity != identity
        || report.target != target.name
        || report.selected != selected
    {
        return Err("test-result-stale-or-mismatched".into());
    }
    let expected = selected.len() * invocation.repeat as usize;
    if report.attempts.len() > expected
        || (report.attempts.len() != expected
            && report.outcome != Outcome::Cancelled
            && report.collection_failure.is_none())
    {
        return Err("test-result-incomplete".into());
    }
    for (index, attempt) in report.attempts.iter().enumerate() {
        if attempt.repeat != (index / selected.len()) as u32 || attempt.test != selected[index % selected.len()] {
            return Err("test-result-attempt-order".into());
        }
    }
    let expected_outcome = if let Some(failure) = &report.collection_failure {
        if !matches!(failure.outcome, Outcome::Cancelled | Outcome::Timeout | Outcome::InfrastructureFailure)
            || (!report.attempts.is_empty() && report.attempts.len() != expected)
        {
            return Err("test-result-collection-contradiction".into());
        }
        failure.outcome
    } else {
        let aggregated = crunch_test_core::aggregate(std::mem::take(&mut report.attempts));
        report.attempts = aggregated.attempts;
        aggregated.outcome
    };
    if report.outcome != expected_outcome {
        return Err("test-result-outcome-contradiction".into());
    }
    if invocation.coverage && report.outcome == Outcome::Pass && report.coverage.is_none() {
        return Err("test-result-coverage-missing".into());
    }
    if let Some(coverage) = &report.coverage {
        let contract = target.coverage.as_ref().ok_or("test-result-coverage-undeclared")?;
        let export = bounded(&root.join("coverage.json"), target.resources.output_bytes)?;
        if !invocation.coverage
            || coverage.cohort != target.cohort
            || coverage.source_root != contract.source_root
            || coverage.source_blake3 != contract.source_blake3
            || coverage.profdata_blake3 != contract.profdata.blake3
            || coverage.cov_blake3 != contract.cov.blake3
            || coverage.export_blake3 != blake3::hash(&export).to_hex().as_str()
            || coverage.total_lines == 0
            || coverage.covered_lines > coverage.total_lines
        {
            return Err("test-result-coverage-mismatch".into());
        }
    }
    Ok(report)
}
struct Runner {
    work: PathBuf,
    next: u32,
    harness: Option<PathBuf>,
    interruption: Option<Outcome>,
}
impl Runner {
    fn command(&self, path: &Path, target: &Target, invocation: &Invocation) -> Command {
        let mut command = Command::new(path);
        command
            .env_clear()
            .current_dir(self.work.join("runfiles"))
            .env("HOME", self.work.join("home"))
            .env("TMPDIR", self.work.join("tmp"))
            .env("RUSTC_BOOTSTRAP", "1")
            .env("TEST_SEED", invocation.seed.to_string())
            .envs(&target.environment);
        let memory = target.resources.memory_bytes;
        let file_bytes = target
            .coverage
            .as_ref()
            .map_or(target.resources.output_bytes, |c| c.max_profile_bytes.max(target.resources.output_bytes));
        // Safe subset: no allocation, locks, or Rust runtime in the fork child.
        unsafe {
            command.pre_exec(move || {
                if libc::setpgid(0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                let limit = libc::rlimit {
                    rlim_cur: memory,
                    rlim_max: memory,
                };
                if libc::setrlimit(libc::RLIMIT_AS, &limit) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                let files = libc::rlimit {
                    rlim_cur: file_bytes,
                    rlim_max: file_bytes,
                };
                if libc::setrlimit(libc::RLIMIT_FSIZE, &files) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        command
    }
    fn process(&mut self, mut command: Command, timeout_ms: u64, limit: u64) -> Result<Capture> {
        self.interruption = None;
        let index = self.next;
        self.next = self.next.checked_add(1).ok_or("test-process-bound")?;
        let stdout_path = self.work.join(format!("stdout-{index}"));
        let stderr_path = self.work.join(format!("stderr-{index}"));
        let stdout = OpenOptions::new().create_new(true).write(true).open(&stdout_path).map_err(err)?;
        let stderr = OpenOptions::new().create_new(true).write(true).open(&stderr_path).map_err(err)?;
        command.stdin(Stdio::null()).stdout(stdout).stderr(stderr);
        let deadline = Instant::now().checked_add(Duration::from_millis(timeout_ms)).ok_or("test-deadline-overflow")?;
        let child = command.spawn().map_err(err)?;
        let mut owned = OwnedChild(child, false);
        let (status, interrupted) = loop {
            if CANCELLED.load(Ordering::Relaxed) {
                break (None, Some(Outcome::Cancelled));
            }
            if Instant::now() >= deadline {
                break (None, Some(Outcome::Timeout));
            }
            if fs::metadata(&stdout_path).map_err(err)?.len() > limit
                || fs::metadata(&stderr_path).map_err(err)?.len() > limit
            {
                break (None, Some(Outcome::InfrastructureFailure));
            }
            if let Some(status) = owned.0.try_wait().map_err(err)? {
                break (Some(status), None);
            }
            thread::sleep(Duration::from_millis(5));
        };
        owned.terminate()?; // Also kill descendants that survived a successful parent.
        self.interruption = interrupted;
        let bytes = bounded(&stdout_path, limit)?;
        bounded(&stderr_path, limit)?;
        Ok(Capture {
            stdout: String::from_utf8(bytes).map_err(err)?,
            exit_code: status.and_then(|s| s.code()),
            interrupted,
        })
    }
}
struct OwnedChild(Child, bool);
impl OwnedChild {
    fn terminate(&mut self) -> Result<()> {
        if self.1 {
            return Ok(());
        }
        let pid = i32::try_from(self.0.id()).map_err(err)?;
        let result = unsafe { libc::kill(-pid, libc::SIGKILL) };
        if result != 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
            return Err("test-process-group-kill".into());
        }
        self.0.wait().map_err(err)?;
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let reaped = unsafe { libc::waitpid(-pid, std::ptr::null_mut(), libc::WNOHANG) };
            if reaped < 0 {
                if std::io::Error::last_os_error().raw_os_error() == Some(libc::ECHILD) {
                    break;
                }
                return Err("test-descendant-reap".into());
            }
            if Instant::now() >= deadline {
                return Err("test-descendant-teardown-deadline".into());
            }
            if reaped == 0 {
                thread::sleep(Duration::from_millis(5));
            }
        }
        self.1 = true;
        Ok(())
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}
impl TestPort for Runner {
    fn interruption(&self) -> Option<Outcome> {
        if CANCELLED.load(Ordering::Relaxed) {
            Some(Outcome::Cancelled)
        } else {
            self.interruption
        }
    }
    fn prepare(&mut self, target: &Target, invocation: &Invocation) -> Result<()> {
        self.harness = Some(executable(target, &target.executable)?);
        executable(target, &target.runner)?;
        for directory in ["home", "tmp", "runfiles", "profiles"] {
            fs::create_dir(self.work.join(directory)).map_err(err)?;
        }
        for (name, root) in &target.runfiles {
            let link = self.work.join("runfiles").join(name);
            fs::create_dir_all(link.parent().ok_or("test-runfile-parent")?).map_err(err)?;
            symlink(root, link).map_err(err)?;
        }
        if invocation.coverage {
            let coverage = target.coverage.as_ref().ok_or("test-coverage-contract")?;
            for tool in [&coverage.profdata, &coverage.cov] {
                let path = executable(target, tool)?;
                let mut command = self.command(&path, target, invocation);
                command.arg("--version");
                let capture = self.process(command, 10_000, target.resources.output_bytes)?;
                if capture.exit_code != Some(0)
                    || capture.interrupted.is_some()
                    || !capture.stdout.contains("LLVM version 20.1.8")
                {
                    return Err("test-coverage-tool-cohort".into());
                }
            }
        }
        Ok(())
    }
    fn capture(&mut self, target: &Target, invocation: &Invocation, name: &str, repeat: u32) -> Result<Capture> {
        let mut command = self.command(self.harness.as_deref().ok_or("test-harness-not-prepared")?, target, invocation);
        command
            .args([
                "--exact",
                name,
                "--test-threads=1",
                "--format=json",
                "-Z",
                "unstable-options",
            ])
            .env("TEST_REPEAT", repeat.to_string());
        if invocation.coverage {
            command.env(
                "LLVM_PROFILE_FILE",
                self.work.join("profiles").join(format!("{}-{repeat}-%p.profraw", self.next)),
            );
        }
        self.process(
            command,
            invocation.timeout_ms.unwrap_or(target.resources.timeout_ms),
            target.resources.output_bytes,
        )
    }
    fn coverage(&mut self, target: &Target, invocation: &Invocation) -> Result<CoverageReport> {
        let coverage = target.coverage.as_ref().ok_or("test-coverage-contract")?;
        let mut profiles = fs::read_dir(self.work.join("profiles"))
            .map_err(err)?
            .collect::<std::io::Result<Vec<_>>>()
            .map_err(err)?;
        profiles.sort_by_key(|entry| entry.file_name());
        if profiles.is_empty() || profiles.len() > 4096 {
            return Err("test-coverage-profile-count".into());
        }
        let mut bytes = 0_u64;
        let mut merge = self.command(&executable(target, &coverage.profdata)?, target, invocation);
        merge.args(["merge", "-sparse", "-o"]).arg(self.work.join("coverage.profdata"));
        for entry in profiles {
            let metadata = entry.file_type().map_err(err)?;
            if !metadata.is_file() || entry.path().extension().is_none_or(|e| e != "profraw") {
                return Err("test-coverage-profile-type".into());
            }
            bytes = bytes.checked_add(entry.metadata().map_err(err)?.len()).ok_or("test-coverage-byte-limit")?;
            if bytes > coverage.max_profile_bytes {
                return Err("test-coverage-byte-limit".into());
            }
            merge.arg(entry.path());
        }
        let capture = self.process(merge, target.resources.timeout_ms, target.resources.output_bytes)?;
        if capture.exit_code != Some(0) || capture.interrupted.is_some() {
            return Err("test-coverage-merge-failed".into());
        }
        let mut export = self.command(&executable(target, &coverage.cov)?, target, invocation);
        export
            .args(["export", "--summary-only"])
            .arg(self.harness.as_deref().ok_or("test-harness-not-prepared")?)
            .arg(format!("-instr-profile={}", self.work.join("coverage.profdata").display()))
            .arg(&coverage.source_root);
        let capture = self.process(export, target.resources.timeout_ms, target.resources.output_bytes)?;
        if capture.exit_code != Some(0) || capture.interrupted.is_some() {
            return Err("test-coverage-export-failed".into());
        }
        let value: serde_json::Value = serde_json::from_str(&capture.stdout).map_err(err)?;
        if value["type"] != "llvm.coverage.json.export" || value["version"] != "2.0.1" {
            return Err("test-coverage-export-schema".into());
        }
        let data = value["data"].as_array().ok_or("test-coverage-export-data")?;
        if data.len() != 1 {
            return Err("test-coverage-export-data".into());
        }
        let lines = &data[0]["totals"]["lines"];
        let total_lines = lines["count"].as_u64().ok_or("test-coverage-line-count")?;
        let covered_lines = lines["covered"].as_u64().ok_or("test-coverage-line-count")?;
        if total_lines == 0 || covered_lines > total_lines {
            return Err("test-coverage-line-bounds".into());
        }
        fs::write(self.work.join("coverage.json"), &capture.stdout).map_err(err)?;
        Ok(CoverageReport {
            cohort: target.cohort.clone(),
            source_root: coverage.source_root.clone(),
            source_blake3: coverage.source_blake3.clone(),
            profdata_blake3: coverage.profdata.blake3.clone(),
            cov_blake3: coverage.cov.blake3.clone(),
            export_blake3: blake3::hash(capture.stdout.as_bytes()).to_hex().to_string(),
            covered_lines,
            total_lines,
        })
    }
}
/// Called only as a declared ordinary sandbox builder; a direct call is not isolation evidence.
pub fn run_worker(request: SelectedRequest, out: &Path) -> Result<Report> {
    let request = crunch_test_core::admit_output(request, out.to_string_lossy().into_owned())?;
    let work = std::env::current_dir().map_err(err)?.join("mantle-test-work");
    fs::create_dir(&work).map_err(err)?;
    CANCELLED.store(false, Ordering::Relaxed);
    #[cfg(target_os = "linux")]
    if unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) } != 0 {
        return Err("test-subreaper-setup".into());
    }
    unsafe {
        libc::signal(libc::SIGTERM, cancel as *const () as libc::sighandler_t);
        libc::signal(libc::SIGINT, cancel as *const () as libc::sighandler_t);
    }
    let report = crunch_test_app::execute(
        &mut Runner {
            work: work.clone(),
            next: 0,
            harness: None,
            interruption: None,
        },
        request,
    )?;
    fs::create_dir(out).map_err(err)?;
    if report.coverage.is_some() {
        fs::copy(work.join("coverage.json"), out.join("coverage.json")).map_err(err)?;
        fs::copy(work.join("coverage.profdata"), out.join("coverage.profdata")).map_err(err)?;
    }
    fs::write(out.join("report.json"), serde_json::to_vec(&report).map_err(err)?).map_err(err)?;
    let summarized = crunch_test_core::summary(report);
    fs::write(out.join("summary.txt"), summarized.text).map_err(err)?;
    let report = summarized.report;
    // A failed assertion is a successfully collected test result, not a successful test.
    // Infrastructure failures remain typed reports; only complete reports are published.
    Ok(report)
}

#[cfg(test)]
mod tests;
