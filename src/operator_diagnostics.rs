// machine-artifact-public: operator-diagnostics.doctor-report
// machine-artifact-public: runtime.diagnostic-fingerprint
use std::ffi::CString;
use std::fmt::Write as _;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use clap::ValueEnum;
use serde::Serialize;

const SANDBOX_SHELL_PLACEHOLDER: &str = "/bin/sh";
const SANDBOX_SHELL_DEFAULT: &str = match option_env!("SNIX_BUILD_SANDBOX_SHELL") {
    Some(path) => path,
    None => SANDBOX_SHELL_PLACEHOLDER,
};
const BWRAP_PATH_ENV: &str = "SNIX_BUILD_BWRAP";
const MAX_NIX_STORE_SCAN_ENTRIES: u32 = 200_000;
const MAX_PARENT_ASCENT: u32 = 64;
const MAX_REDACTED_SUMMARY_COUNT: u32 = 4_096;
const MAX_SUBSTITUTER_SUMMARY_COUNT: u32 = 1_024;
const RUNTIME_FINGERPRINT_SCHEMA: &str = "mantle-runtime-fingerprint-v1";
pub const RUNTIME_FINGERPRINT_PREFIX: &str = "mantle-runtime-fingerprint";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum DoctorProfile {
    #[default]
    Build,
    SelfBuild,
}

impl DoctorProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::SelfBuild => "self-build",
        }
    }

    fn needs_nightly_toolchain(self) -> bool {
        matches!(self, Self::SelfBuild)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeFingerprintTriggerInput<'a> {
    pub verbose: bool,
    pub log_level: Option<&'a str>,
    pub diagnostic_mode: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct RuntimeFingerprintModeFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hermeticity_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub substitution_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub substituter_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nix_compat_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_key_selected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_public_key_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_unsigned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_ticket_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_value_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_argv_count: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFingerprintInput {
    pub mantle_version: String,
    pub command: String,
    pub logical_store_prefix: String,
    pub physical_store_dir: String,
    pub state_dir: String,
    pub json_mode: bool,
    pub verbosity_source: String,
    pub modes: RuntimeFingerprintModeFields,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RuntimeDiagnosticFingerprint {
    pub schema: &'static str,
    pub mantle_version: String,
    pub command: String,
    pub logical_store_prefix: String,
    pub physical_store_dir: String,
    pub state_dir: String,
    pub json_mode: bool,
    pub verbosity_source: String,
    pub diagnostic_scope: &'static str,
    #[serde(flatten)]
    pub modes: RuntimeFingerprintModeFields,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeSecretInputs<'a> {
    pub signing_key_path: Option<&'a str>,
    pub trusted_public_keys: &'a [&'a str],
    pub bearer_tickets: &'a [&'a str],
    pub environment_values: &'a [&'a str],
    pub raw_argv: &'a [&'a str],
}

pub fn runtime_fingerprint_verbosity_source(input: RuntimeFingerprintTriggerInput<'_>) -> Option<String> {
    if input.verbose {
        return Some("verbose-flag".to_string());
    }
    if let Some(level) = input.log_level {
        let normalized = normalize_log_level(level);
        if explicit_log_level_is_talkative(&normalized) {
            return Some(format!("log-level-{normalized}"));
        }
    }
    if input.diagnostic_mode {
        return Some("diagnostic-mode".to_string());
    }
    None
}

pub fn build_runtime_fingerprint(input: RuntimeFingerprintInput) -> RuntimeDiagnosticFingerprint {
    assert!(!input.mantle_version.is_empty(), "mantle_version must not be empty");
    assert!(!input.command.is_empty(), "command label must not be empty");
    assert!(input.logical_store_prefix.starts_with('/'), "logical store prefix must be absolute");
    assert!(!input.physical_store_dir.is_empty(), "physical store dir must not be empty");
    assert!(!input.state_dir.is_empty(), "state dir must not be empty");
    assert!(!input.verbosity_source.is_empty(), "verbosity source must not be empty");
    RuntimeDiagnosticFingerprint {
        schema: RUNTIME_FINGERPRINT_SCHEMA,
        mantle_version: input.mantle_version,
        command: input.command,
        logical_store_prefix: input.logical_store_prefix,
        physical_store_dir: input.physical_store_dir,
        state_dir: input.state_dir,
        json_mode: input.json_mode,
        verbosity_source: input.verbosity_source,
        diagnostic_scope: "selected-runtime-context-only",
        modes: input.modes,
    }
}

pub fn render_runtime_fingerprint(fingerprint: &RuntimeDiagnosticFingerprint) -> Result<String, serde_json::Error> {
    assert_eq!(fingerprint.schema, RUNTIME_FINGERPRINT_SCHEMA);
    assert!(!fingerprint.command.is_empty(), "command label must not be empty");
    let payload = serde_json::to_string(fingerprint)?;
    Ok(format!("{RUNTIME_FINGERPRINT_PREFIX} {payload}"))
}

#[cfg(test)]
pub fn summarize_runtime_secrets(input: RuntimeSecretInputs<'_>) -> RuntimeFingerprintModeFields {
    RuntimeFingerprintModeFields {
        signing_key_selected: Some(input.signing_key_path.is_some()),
        trusted_public_key_count: Some(bounded_runtime_count(input.trusted_public_keys.len())),
        bearer_ticket_count: Some(bounded_runtime_count(input.bearer_tickets.len())),
        environment_value_count: Some(bounded_runtime_count(input.environment_values.len())),
        raw_argv_count: Some(bounded_runtime_count(input.raw_argv.len())),
        ..RuntimeFingerprintModeFields::default()
    }
}

pub fn bounded_runtime_count(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(MAX_REDACTED_SUMMARY_COUNT).min(MAX_REDACTED_SUMMARY_COUNT)
}

pub fn count_substituters(value: &str) -> u32 {
    let mut count = 0_u32;
    for entry in value.split(',') {
        if entry.trim().is_empty() {
            continue;
        }
        if count == MAX_SUBSTITUTER_SUMMARY_COUNT {
            break;
        }
        count = count.saturating_add(1);
    }
    count
}

fn normalize_log_level(level: &str) -> String {
    level.trim().to_ascii_lowercase()
}

fn explicit_log_level_is_talkative(normalized_level: &str) -> bool {
    matches!(normalized_level, "trace" | "debug" | "info")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreflightStatus {
    Ok,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PreflightCheck {
    pub id: &'static str,
    pub status: PreflightStatus,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PreflightReport {
    pub schema: &'static str,
    pub command: &'static str,
    pub profile: DoctorProfile,
    pub ok: bool,
    pub checks: Vec<PreflightCheck>,
}

#[derive(Clone, Copy, Debug)]
pub struct DoctorRequest<'a> {
    pub profile: DoctorProfile,
    pub store_dir: &'a Path,
    pub state_dir: &'a Path,
}

pub fn collect_doctor_report(request: DoctorRequest<'_>) -> PreflightReport {
    assert!(!request.store_dir.as_os_str().is_empty(), "store_dir must not be empty");
    assert!(!request.state_dir.as_os_str().is_empty(), "state_dir must not be empty");

    let mut checks = Vec::with_capacity(6);
    if request.profile.needs_nightly_toolchain() {
        checks.push(check_nightly_toolchain_visibility());
    }
    checks.push(check_bwrap_visibility());
    checks.push(check_sandbox_shell_availability());
    checks.push(check_fusermount3_availability());
    checks.push(check_directory_writable("state-dir", request.state_dir, "state directory"));
    checks.push(check_directory_writable("store-dir", request.store_dir, "store directory"));
    let ok = checks.iter().all(|check| check.status == PreflightStatus::Ok);

    PreflightReport {
        schema: "crunch-doctor-report-v1",
        command: "doctor",
        profile: request.profile,
        ok,
        checks,
    }
}

impl PreflightReport {
    pub fn render_human(&self) -> String {
        assert!(!self.checks.is_empty(), "doctor report must include at least one check");
        let failed_count = self.failed_count();
        let mut out = String::new();
        let _ = writeln!(&mut out, "doctor profile: {}", self.profile.as_str());
        if self.ok {
            let _ = writeln!(&mut out, "status: ok");
        } else {
            let noun = if failed_count == 1 {
                "prerequisite"
            } else {
                "prerequisites"
            };
            let _ = writeln!(&mut out, "status: failed ({failed_count} {noun} failed)");
        }
        for check in &self.checks {
            let marker = if check.status == PreflightStatus::Ok {
                "ok"
            } else {
                "failed"
            };
            let _ = writeln!(&mut out, "- [{marker}] {}: {}", check.id, check.summary);
            if let Some(detail) = &check.detail {
                let _ = writeln!(&mut out, "  {detail}");
            }
        }
        out
    }

    pub fn render_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn failed_count(&self) -> u32 {
        let count = self.checks.iter().filter(|check| check.status == PreflightStatus::Failed).count();
        u32::try_from(count).expect("preflight failure count must fit in u32")
    }
}

fn ok_check(id: &'static str, summary: String, detail: Option<String>) -> PreflightCheck {
    assert!(!summary.is_empty(), "ok summary must not be empty");
    PreflightCheck {
        id,
        status: PreflightStatus::Ok,
        summary,
        detail,
    }
}

fn failed_check(id: &'static str, summary: String, detail: Option<String>) -> PreflightCheck {
    assert!(!summary.is_empty(), "failure summary must not be empty");
    PreflightCheck {
        id,
        status: PreflightStatus::Failed,
        summary,
        detail,
    }
}

fn check_nightly_toolchain_visibility() -> PreflightCheck {
    let cargo = command_supports_nightly("cargo");
    let rustc = command_supports_nightly("rustc");
    if cargo && rustc {
        return ok_check("nightly-toolchain", "cargo +nightly and rustc +nightly are visible".to_string(), None);
    }

    failed_check(
        "nightly-toolchain",
        "cargo +nightly or rustc +nightly is not visible".to_string(),
        Some(
            "put nightly cargo/rustc on PATH or install the nightly toolchain before self-build workflows".to_string(),
        ),
    )
}

fn command_supports_nightly(program: &str) -> bool {
    let output = Command::new(program).arg("+nightly").arg("-V").output();
    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn check_bwrap_visibility() -> PreflightCheck {
    if let Some(path) = explicit_bwrap_path_from_env() {
        return check_explicit_bwrap(&path);
    }
    if let Some(path) = find_bwrap() {
        return ok_check("bwrap", format!("found {}", path.display()), None);
    }

    failed_check(
        "bwrap",
        "bubblewrap executable not found".to_string(),
        Some(format!(
            "install bwrap, put it on PATH, or set {BWRAP_PATH_ENV} before running build-capable workflows"
        )),
    )
}

fn explicit_bwrap_path_from_env() -> Option<PathBuf> {
    explicit_bwrap_path(std::env::var_os(BWRAP_PATH_ENV))
}

fn explicit_bwrap_path(value: Option<std::ffi::OsString>) -> Option<PathBuf> {
    let path = value?;
    if path.is_empty() {
        return None;
    }
    Some(PathBuf::from(path))
}

fn check_explicit_bwrap(path: &Path) -> PreflightCheck {
    if is_executable_file(path) {
        return ok_check("bwrap", format!("using {BWRAP_PATH_ENV}={}", path.display()), None);
    }

    failed_check(
        "bwrap",
        format!("{BWRAP_PATH_ENV} points to a missing or non-executable bwrap"),
        Some(path.display().to_string()),
    )
}

fn find_bwrap() -> Option<PathBuf> {
    let wrapper = Path::new("/run/wrappers/bin/bwrap");
    if is_executable_file(wrapper) {
        return Some(wrapper.to_path_buf());
    }
    find_executable_on_path("bwrap")
}

fn check_sandbox_shell_availability() -> PreflightCheck {
    if let Some(env_shell) = std::env::var_os("SNIX_BUILD_SANDBOX_SHELL")
        && env_shell != SANDBOX_SHELL_PLACEHOLDER
    {
        let env_path = PathBuf::from(env_shell);
        return check_explicit_shell("sandbox-shell", &env_path, "SNIX_BUILD_SANDBOX_SHELL");
    }

    if SANDBOX_SHELL_DEFAULT != SANDBOX_SHELL_PLACEHOLDER {
        let compile_path = PathBuf::from(SANDBOX_SHELL_DEFAULT);
        if is_executable_file(&compile_path) {
            return ok_check(
                "sandbox-shell",
                format!("using compile-time sandbox shell {}", compile_path.display()),
                None,
            );
        }
    }

    if let Some(path) = find_static_busybox() {
        return ok_check("sandbox-shell", format!("found static sandbox shell {}", path.display()), None);
    }

    failed_check(
        "sandbox-shell",
        "no usable sandbox shell found".to_string(),
        Some("install busybox-static or set SNIX_BUILD_SANDBOX_SHELL to an executable sandbox shell".to_string()),
    )
}

fn check_explicit_shell(id: &'static str, path: &Path, source: &str) -> PreflightCheck {
    assert!(!source.is_empty(), "shell source must not be empty");
    if is_executable_file(path) {
        return ok_check(id, format!("using {source}={}", path.display()), None);
    }

    failed_check(
        id,
        format!("{source} points to a missing or non-executable shell"),
        Some(path.display().to_string()),
    )
}

fn find_static_busybox() -> Option<PathBuf> {
    for candidate in ["/run/current-system/sw/bin/busybox-static", "/bin/busybox.static"] {
        let path = Path::new(candidate);
        if is_executable_file(path) {
            return Some(path.to_path_buf());
        }
    }
    find_busybox_static_in_dir(Path::new("/nix/store"))
}

fn find_busybox_static_in_dir(store_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(store_dir).ok()?;
    let mut scanned_entries: u32 = 0;
    for entry in entries.flatten() {
        scanned_entries = scanned_entries.saturating_add(1);
        if scanned_entries > MAX_NIX_STORE_SCAN_ENTRIES {
            break;
        }
        let name_str = entry.file_name().to_string_lossy().into_owned();
        if !name_str.contains("busybox-static") {
            continue;
        }
        let candidate = entry.path().join("bin").join("busybox");
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn check_fusermount3_availability() -> PreflightCheck {
    if let Some(path) = find_fusermount3() {
        return ok_check("fusermount3", format!("found {}", path.display()), None);
    }

    failed_check(
        "fusermount3",
        "FUSE helper not found".to_string(),
        Some("install fusermount3 or make it visible on PATH for FUSE-backed build workflows".to_string()),
    )
}

fn find_fusermount3() -> Option<PathBuf> {
    let wrapper = Path::new("/run/wrappers/bin/fusermount3");
    if is_executable_file(wrapper) {
        return Some(wrapper.to_path_buf());
    }
    find_executable_on_path("fusermount3")
}

fn find_executable_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn check_directory_writable(id: &'static str, target_path: &Path, label: &str) -> PreflightCheck {
    assert!(!label.is_empty(), "writable-path label must not be empty");
    match writable_anchor(target_path) {
        Ok(anchor) => ok_check(id, format!("{label} is writable via {}", anchor.display()), None),
        Err(detail) => failed_check(id, format!("{label} is not writable"), Some(detail)),
    }
}

fn writable_anchor(target_path: &Path) -> Result<PathBuf, String> {
    assert!(!target_path.as_os_str().is_empty(), "target_path must not be empty");
    let anchor = nearest_existing_ancestor(target_path)
        .ok_or_else(|| format!("no existing parent found for {}", target_path.display()))?;
    let metadata = std::fs::metadata(&anchor).map_err(|e| format!("stat {}: {e}", anchor.display()))?;
    if !metadata.is_dir() {
        return Err(format!("{} is not a directory", anchor.display()));
    }
    if !directory_is_effectively_writable(&anchor)? {
        return Err(format!("{} is not writable by the current user", anchor.display()));
    }
    Ok(anchor)
}

fn nearest_existing_ancestor(target_path: &Path) -> Option<PathBuf> {
    let mut current = target_path;
    let mut ascents: u32 = 0;
    loop {
        if current.exists() {
            return Some(current.to_path_buf());
        }
        ascents = ascents.saturating_add(1);
        if ascents > MAX_PARENT_ASCENT {
            return None;
        }
        current = current.parent()?;
    }
}

fn directory_is_effectively_writable(path: &Path) -> Result<bool, String> {
    assert!(path.is_dir(), "effective-write probe expects a directory");
    let bytes = path.as_os_str().as_bytes();
    if bytes.contains(&0) {
        return Err(format!("{} contains an interior NUL byte", path.display()));
    }
    let c_path = CString::new(bytes).map_err(|e| format!("encoding {} for access(2): {e}", path.display()))?;
    let mode = libc::W_OK | libc::X_OK;
    let rc = unsafe { libc::access(c_path.as_ptr(), mode) };
    if rc == 0 {
        return Ok(true);
    }
    let error = std::io::Error::last_os_error();
    match error.raw_os_error() {
        Some(libc::EACCES) => Ok(false),
        Some(libc::EROFS) => Ok(false),
        Some(libc::EPERM) => Ok(false),
        _ => Err(format!("access {}: {error}", path.display())),
    }
}

fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let mode = match path.metadata() {
        Ok(metadata) => metadata.permissions().mode(),
        Err(_) => return false,
    };
    mode & 0o111 != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_TRUSTED_KEY_COUNT: u32 = 2;
    const TEST_SECRET_COUNT: u32 = 1;

    #[test]
    fn default_profile_is_build() {
        assert_eq!(DoctorProfile::default(), DoctorProfile::Build);
        assert_eq!(DoctorProfile::default().as_str(), "build");
    }

    #[test]
    fn runtime_fingerprint_triggers_for_verbose_and_talkative_log_levels() {
        let verbose = runtime_fingerprint_verbosity_source(RuntimeFingerprintTriggerInput {
            verbose: true,
            log_level: None,
            diagnostic_mode: false,
        });
        let debug = runtime_fingerprint_verbosity_source(RuntimeFingerprintTriggerInput {
            verbose: false,
            log_level: Some("DEBUG"),
            diagnostic_mode: false,
        });
        let info = runtime_fingerprint_verbosity_source(RuntimeFingerprintTriggerInput {
            verbose: false,
            log_level: Some("info"),
            diagnostic_mode: false,
        });

        assert_eq!(verbose.as_deref(), Some("verbose-flag"));
        assert_eq!(debug.as_deref(), Some("log-level-debug"));
        assert_eq!(info.as_deref(), Some("log-level-info"));
    }

    #[test]
    fn runtime_fingerprint_does_not_trigger_for_default_or_quiet_log_levels() {
        let default = runtime_fingerprint_verbosity_source(RuntimeFingerprintTriggerInput {
            verbose: false,
            log_level: None,
            diagnostic_mode: false,
        });
        let warn = runtime_fingerprint_verbosity_source(RuntimeFingerprintTriggerInput {
            verbose: false,
            log_level: Some("warn"),
            diagnostic_mode: false,
        });
        let error = runtime_fingerprint_verbosity_source(RuntimeFingerprintTriggerInput {
            verbose: false,
            log_level: Some("error"),
            diagnostic_mode: false,
        });

        assert!(default.is_none());
        assert!(warn.is_none());
        assert!(error.is_none());
    }

    #[test]
    fn runtime_fingerprint_renders_stable_context_fields() {
        let fingerprint = build_runtime_fingerprint(RuntimeFingerprintInput {
            mantle_version: "0.1.0".to_string(),
            command: "build".to_string(),
            logical_store_prefix: "/mantle/store".to_string(),
            physical_store_dir: "/tmp/mantle-store".to_string(),
            state_dir: "/tmp/mantle-state".to_string(),
            json_mode: true,
            verbosity_source: "verbose-flag".to_string(),
            modes: RuntimeFingerprintModeFields {
                hermeticity_mode: Some("strict".to_string()),
                substitution_mode: Some("enabled".to_string()),
                substituter_count: Some(TEST_SECRET_COUNT),
                nix_compat_mode: Some(true),
                ..RuntimeFingerprintModeFields::default()
            },
        });

        let rendered = render_runtime_fingerprint(&fingerprint).unwrap();
        let payload = rendered.strip_prefix(&format!("{RUNTIME_FINGERPRINT_PREFIX} ")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(payload).unwrap();

        assert_eq!(parsed["schema"], RUNTIME_FINGERPRINT_SCHEMA);
        assert_eq!(parsed["mantle_version"], "0.1.0");
        assert_eq!(parsed["command"], "build");
        assert_eq!(parsed["logical_store_prefix"], "/mantle/store");
        assert_eq!(parsed["physical_store_dir"], "/tmp/mantle-store");
        assert_eq!(parsed["state_dir"], "/tmp/mantle-state");
        assert_eq!(parsed["json_mode"], true);
        assert_eq!(parsed["verbosity_source"], "verbose-flag");
        assert_eq!(parsed["diagnostic_scope"], "selected-runtime-context-only");
        assert_eq!(parsed["hermeticity_mode"], "strict");
        assert_eq!(parsed["substitution_mode"], "enabled");
        assert_eq!(parsed["substituter_count"], TEST_SECRET_COUNT);
        assert_eq!(parsed["nix_compat_mode"], true);
    }

    #[test]
    fn runtime_fingerprint_redacts_secret_inputs_to_counts() {
        let signing_key = "/tmp/super-secret-signing-key";
        let trusted_key_one = "cache.example-1:secret-public-key-material";
        let trusted_key_two = "cache.example-2:other-secret-public-key-material";
        let bearer_ticket = "bearer-ticket-secret";
        let env_value = "TOKEN=secret-env-value";
        let raw_argv = "--password=secret-argv-value";
        let modes = summarize_runtime_secrets(RuntimeSecretInputs {
            signing_key_path: Some(signing_key),
            trusted_public_keys: &[trusted_key_one, trusted_key_two],
            bearer_tickets: &[bearer_ticket],
            environment_values: &[env_value],
            raw_argv: &[raw_argv],
        });
        let fingerprint = build_runtime_fingerprint(RuntimeFingerprintInput {
            mantle_version: "0.1.0".to_string(),
            command: "build".to_string(),
            logical_store_prefix: "/mantle/store".to_string(),
            physical_store_dir: "/tmp/mantle-store".to_string(),
            state_dir: "/tmp/mantle-state".to_string(),
            json_mode: false,
            verbosity_source: "verbose-flag".to_string(),
            modes,
        });

        let rendered = render_runtime_fingerprint(&fingerprint).unwrap();

        assert!(!rendered.contains(signing_key));
        assert!(!rendered.contains(trusted_key_one));
        assert!(!rendered.contains(trusted_key_two));
        assert!(!rendered.contains(bearer_ticket));
        assert!(!rendered.contains(env_value));
        assert!(!rendered.contains(raw_argv));
        assert!(rendered.contains("\"signing_key_selected\":true"));
        assert!(rendered.contains(&format!("\"trusted_public_key_count\":{TEST_TRUSTED_KEY_COUNT}")));
        assert!(rendered.contains(&format!("\"bearer_ticket_count\":{TEST_SECRET_COUNT}")));
        assert!(rendered.contains(&format!("\"environment_value_count\":{TEST_SECRET_COUNT}")));
        assert!(rendered.contains(&format!("\"raw_argv_count\":{TEST_SECRET_COUNT}")));
    }

    #[test]
    fn human_report_mentions_failed_count_and_profile() {
        let report = PreflightReport {
            schema: "crunch-doctor-report-v1",
            command: "doctor",
            profile: DoctorProfile::SelfBuild,
            ok: false,
            checks: vec![failed_check("bwrap", "missing".to_string(), None)],
        };

        let rendered = report.render_human();
        assert!(rendered.contains("doctor profile: self-build"));
        assert!(rendered.contains("status: failed (1 prerequisite failed)"));
    }

    #[test]
    fn writable_anchor_uses_existing_parent_for_missing_dir() {
        let dir = tempfile::tempdir().unwrap();
        let anchor = writable_anchor(&dir.path().join("missing")).unwrap();
        assert_eq!(anchor, dir.path());
    }

    #[test]
    fn effective_writable_probe_rejects_read_only_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        let original_mode = std::fs::metadata(path).unwrap().permissions().mode();
        let mut permissions = std::fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o555);
        std::fs::set_permissions(path, permissions).unwrap();

        let writable = directory_is_effectively_writable(path).unwrap();

        let mut restore = std::fs::metadata(path).unwrap().permissions();
        restore.set_mode(original_mode);
        std::fs::set_permissions(path, restore).unwrap();
        assert!(!writable, "read-only directory must fail effective write probe");
    }

    #[test]
    fn explicit_shell_failure_mentions_source() {
        let check = check_explicit_shell("sandbox-shell", Path::new("/missing/shell"), "SNIX_BUILD_SANDBOX_SHELL");
        assert_eq!(check.status, PreflightStatus::Failed);
        assert!(check.summary.contains("SNIX_BUILD_SANDBOX_SHELL"));
    }

    #[test]
    fn explicit_bwrap_path_uses_non_empty_env_value() {
        let path = explicit_bwrap_path(Some(std::ffi::OsString::from("/tools/bwrap"))).expect("explicit path");

        assert_eq!(path, PathBuf::from("/tools/bwrap"));
    }

    #[test]
    fn explicit_bwrap_path_ignores_empty_or_missing_env_value() {
        assert!(explicit_bwrap_path(None).is_none());
        assert!(explicit_bwrap_path(Some(std::ffi::OsString::new())).is_none());
    }

    #[test]
    fn explicit_bwrap_failure_mentions_source() {
        let check = check_explicit_bwrap(Path::new("/missing/bwrap"));

        assert_eq!(check.status, PreflightStatus::Failed);
        assert!(check.summary.contains(BWRAP_PATH_ENV));
    }
}
