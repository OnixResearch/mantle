use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use crunch_build::signing;
use crunch_pipeline::BuildConfig;
use crunch_pipeline::PipelineResult;
use crunch_pipeline::drv_key_for;
use crunch_pipeline::label_for_key;
use crunch_pipeline::parse_drv_key;
use nix_compat::store_path::StorePath;

use crate::errors::RunError;

#[allow(clippy::too_many_arguments)]
pub fn cmd_build(
    file: &Path,
    import_paths: &[OsString],
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    verbose: bool,
    fix: bool,
    max_jobs: u32,
    substituter_url: Option<&str>,
    signing_key_path: Option<&Path>,
    trusted_public_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
    trust_unsigned: bool,
) -> Result<(), RunError> {
    let keypair = load_or_generate_signing_keypair(signing_key_path, state_dir)?;
    let configured_trusted_keys = load_configured_trusted_public_keys(trusted_public_keys, state_dir)?;
    let trusted_keys = signing::build_trusted_keys(&keypair, configured_trusted_keys.as_deref());

    let config = BuildConfig {
        file: file.to_path_buf(),
        import_paths: import_paths.to_vec(),
        output_dir: output_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        store_dir: store_dir.to_string(),
        verbose,
        max_jobs,
        substituter_url: substituter_url.map(str::to_owned),
        keypair,
        trusted_keys,
        trust_unsigned,
    };

    let result = run_build(&config)?;
    report_build_result(&config, &result, fix)
}

pub fn run_build(config: &BuildConfig) -> Result<PipelineResult, RunError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;
    rt.block_on(crunch_pipeline::build(config)).map_err(Into::into)
}

pub fn report_build_result(config: &BuildConfig, result: &PipelineResult, fix: bool) -> Result<(), RunError> {
    let logs_dir = log_dir();
    let _ = std::fs::create_dir_all(&logs_dir);

    write_success_logs_and_outputs(config, result, &logs_dir);

    if result.failed.is_empty() {
        return Ok(());
    }

    if let Some(single_mismatch) = maybe_single_fod_mismatch(config, result, &logs_dir, fix) {
        return single_mismatch;
    }

    write_failure_logs(config, result, &logs_dir);
    print_failed_builds(result);

    Err(RunError::Build(format!("{} root build(s) failed", result.failed.len(),)))
}

fn maybe_single_fod_mismatch(
    config: &BuildConfig,
    result: &PipelineResult,
    logs_dir: &Path,
    fix: bool,
) -> Option<Result<(), RunError>> {
    if result.failed.len() != 1 {
        return None;
    }
    if result.fod_mismatches.len() != 1 {
        return None;
    }

    let failed = &result.failed[0];
    let mismatch = &result.fod_mismatches[0];
    let drv_path = parse_drv_key(&config.store_dir, &failed.drv_key)?;
    let label = label_for_key(result, &failed.drv_key).unwrap_or(drv_path.name());

    Some(crate::fix::handle_fod_mismatch(mismatch, &drv_path, label, logs_dir, &config.file, fix))
}

fn write_success_logs_and_outputs(config: &BuildConfig, result: &PipelineResult, logs_dir: &Path) {
    let output_dir_str = config.output_dir.to_str().unwrap_or(&config.store_dir);

    for outcome in &result.outcomes {
        let drv_key = drv_key_for(&config.store_dir, &outcome.drv_path);
        let label = label_for_key(result, &drv_key).unwrap_or(outcome.drv_path.name());

        if let Some(log) = &outcome.log {
            write_log(logs_dir, &outcome.drv_path, label, true, log);
            if config.verbose {
                eprintln!("--- build log: {label} ---");
                eprintln!("{log}");
                eprintln!("--- end log ---");
            }
        } else if !outcome.cached {
            write_log(logs_dir, &outcome.drv_path, label, true, "(no output captured)");
        }

        let multi = outcome.outputs.len() > 1;
        for (output_name, path_info) in &outcome.outputs {
            let path = path_info.store_path.to_absolute_path_with_prefix(output_dir_str);
            let suffix = match (outcome.cached, multi && output_name != "out") {
                (true, true) => format!(" ({output_name}, cached)"),
                (true, false) => " (cached)".to_string(),
                (false, true) => format!(" ({output_name})"),
                (false, false) => String::new(),
            };
            println!("{path}{suffix}");
        }
    }
}

fn write_failure_logs(config: &BuildConfig, result: &PipelineResult, logs_dir: &Path) {
    for failed in &result.failed {
        let Some(drv_path) = parse_drv_key(&config.store_dir, &failed.drv_key) else {
            continue;
        };
        let label = label_for_key(result, &failed.drv_key).unwrap_or(drv_path.name());
        write_log(logs_dir, &drv_path, label, false, &failed.error);
    }
}

fn print_failed_builds(result: &PipelineResult) {
    for failed in &result.failed {
        eprintln!("FAILED: {}", failed.drv_key);
        eprintln!("  {}", failed.error);
    }
}

pub fn write_log(log_dir: &Path, drv_path: &StorePath<String>, label: &str, success: bool, body: &str) {
    let log_file = log_dir.join(format!("{}.log", drv_path));
    let status = if success { "success" } else { "failure" };
    let timestamp =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let content = format!(
        "# crunch build log\n# derivation: {label}\n# drv_path: {drv_path}\n# status: {status}\n# timestamp: {timestamp}\n\n{body}\n"
    );
    let _ = std::fs::write(&log_file, &content);
}

pub fn state_dir() -> PathBuf {
    std::env::var("CRUNCH_STATE_DIR").map(PathBuf::from).unwrap_or_else(|_| {
        let state = std::env::var("XDG_STATE_HOME").map(PathBuf::from).unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".local/state")
        });
        state.join("crunch")
    })
}

pub fn log_dir() -> PathBuf {
    std::env::var("CRUNCH_LOG_DIR").map(PathBuf::from).unwrap_or_else(|_| state_dir().join("logs"))
}

/// Load a signing keypair from the given path, the default config location,
/// or generate one automatically.
pub fn load_or_generate_signing_keypair(
    explicit_path: Option<&Path>,
    state_dir: &Path,
) -> Result<signing::KeyPair, RunError> {
    // 1. Explicit path from --signing-key.
    if let Some(path) = explicit_path {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| RunError::Internal(format!("reading signing key {}: {e}", path.display())))?;
        return signing::load_keypair(&contents)
            .map_err(|e| RunError::Internal(format!("parsing signing key {}: {e}", path.display())));
    }

    // 2. Default location: $CRUNCH_CONFIG_DIR/signing-key or $state_dir/signing-key.
    let config_dir = config_dir_or(state_dir);
    let default_path = config_dir.join("signing-key");
    if default_path.exists() {
        let contents = std::fs::read_to_string(&default_path)
            .map_err(|e| RunError::Internal(format!("reading signing key {}: {e}", default_path.display())))?;
        return signing::load_keypair(&contents)
            .map_err(|e| RunError::Internal(format!("parsing signing key {}: {e}", default_path.display())));
    }

    // 3. Auto-generate.
    let (keypair, line) = signing::generate_keypair();
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| RunError::Internal(format!("creating config dir {}: {e}", config_dir.display())))?;

    // Write with 0600 permissions.
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&default_path)
            .and_then(|mut f| {
                use std::io::Write;
                f.write_all(line.as_bytes())?;
                f.write_all(b"\n")?;
                Ok(())
            })
            .map_err(|e| RunError::Internal(format!("writing signing key {}: {e}", default_path.display())))?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(&default_path, format!("{line}\n"))
            .map_err(|e| RunError::Internal(format!("writing signing key {}: {e}", default_path.display())))?;
    }

    eprintln!("Generated signing key: {} ({})", keypair.verifying_key.name(), default_path.display());
    Ok(keypair)
}

pub fn load_configured_trusted_public_keys(
    explicit_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
    state_dir: &Path,
) -> Result<Option<Vec<nix_compat::narinfo::VerifyingKey>>, RunError> {
    if let Some(keys) = explicit_keys {
        return Ok(Some(keys.to_vec()));
    }

    let config_dir = config_dir_or(state_dir);
    let default_path = config_dir.join("trusted-public-keys");
    if !default_path.exists() {
        return Ok(None);
    }

    let contents = std::fs::read_to_string(&default_path)
        .map_err(|e| RunError::Internal(format!("reading trusted public keys {}: {e}", default_path.display())))?;

    let mut parsed = Vec::new();
    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        for key_str in line.split(',') {
            let trimmed = key_str.trim();
            if trimmed.is_empty() {
                continue;
            }
            parsed.push(nix_compat::narinfo::VerifyingKey::parse(trimmed).map_err(|e| {
                RunError::Internal(format!(
                    "invalid trusted public key '{}' in {}: {e}",
                    trimmed,
                    default_path.display()
                ))
            })?);
        }
    }

    if parsed.is_empty() {
        return Ok(None);
    }

    Ok(Some(parsed))
}

fn config_dir_or(state_dir: &Path) -> PathBuf {
    std::env::var("CRUNCH_CONFIG_DIR").map(PathBuf::from).unwrap_or_else(|_| state_dir.to_path_buf())
}

pub fn build_import_paths(extra: &[PathBuf]) -> Result<Vec<OsString>, RunError> {
    let stdlib_dir =
        crunch_eval::stdlib::stdlib_import_path().map_err(|e| RunError::Internal(format!("stdlib: {e}")))?;
    let mut paths: Vec<OsString> = vec![stdlib_dir.into()];
    for path in extra {
        paths.push(path.into());
    }
    Ok(paths)
}
