use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crunch_pipeline::{BuildConfig, PipelineResult, drv_key_for, label_for_key, parse_drv_key};
use nix_compat::store_path::StorePath;

use crate::errors::RunError;

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
) -> Result<(), RunError> {
    let config = BuildConfig {
        file: file.to_path_buf(),
        import_paths: import_paths.to_vec(),
        output_dir: output_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        store_dir: store_dir.to_string(),
        verbose,
        max_jobs,
        substituter_url: substituter_url.map(str::to_owned),
    };

    let result = run_build(&config)?;
    report_build_result(&config, &result, fix)
}

pub fn run_build(config: &BuildConfig) -> Result<PipelineResult, RunError> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;
    rt.block_on(crunch_pipeline::build(config)).map_err(Into::into)
}

pub fn report_build_result(
    config: &BuildConfig,
    result: &PipelineResult,
    fix: bool,
) -> Result<(), RunError> {
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

    Err(RunError::Build(format!(
        "{} root build(s) failed",
        result.failed.len(),
    )))
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

    Some(crate::fix::handle_fod_mismatch(
        mismatch,
        &drv_path,
        label,
        logs_dir,
        &config.file,
        fix,
    ))
}

fn write_success_logs_and_outputs(
    config: &BuildConfig,
    result: &PipelineResult,
    logs_dir: &Path,
) {
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
            let path = path_info
                .store_path
                .to_absolute_path_with_prefix(output_dir_str);
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

fn write_failure_logs(
    config: &BuildConfig,
    result: &PipelineResult,
    logs_dir: &Path,
) {
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

pub fn write_log(
    log_dir: &Path,
    drv_path: &StorePath<String>,
    label: &str,
    success: bool,
    body: &str,
) {
    let log_file = log_dir.join(format!("{}.log", drv_path));
    let status = if success { "success" } else { "failure" };
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let content = format!(
        "# crunch build log\n# derivation: {label}\n# drv_path: {drv_path}\n# status: {status}\n# timestamp: {timestamp}\n\n{body}\n"
    );
    let _ = std::fs::write(&log_file, &content);
}

pub fn state_dir() -> PathBuf {
    std::env::var("CRUNCH_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let state = std::env::var("XDG_STATE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                    PathBuf::from(home).join(".local/state")
                });
            state.join("crunch")
        })
}

pub fn log_dir() -> PathBuf {
    std::env::var("CRUNCH_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| state_dir().join("logs"))
}

pub fn build_import_paths(extra: &[PathBuf]) -> Result<Vec<OsString>, RunError> {
    let stdlib_dir = crunch_eval::stdlib::stdlib_import_path()
        .map_err(|e| RunError::Internal(format!("stdlib: {e}")))?;
    let mut paths: Vec<OsString> = vec![stdlib_dir.into()];
    for path in extra {
        paths.push(path.into());
    }
    Ok(paths)
}
