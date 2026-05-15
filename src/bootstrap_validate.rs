use std::fs;
use std::io::Read;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::thread;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde::Serialize;

use crate::RunContext;
use crate::build_cmd::build_import_paths;
use crate::errors::RunError;
use crate::operator_diagnostics::DoctorProfile;
use crate::operator_diagnostics::{self};

const SUMMARY_SCHEMA: &str = "crunch-bootstrap-validation-v1";
const BUILD_STDOUT_FILE: &str = "build.stdout.log";
const BUILD_STDERR_FILE: &str = "build.stderr.log";
const DOCTOR_JSON_FILE: &str = "doctor.json";
const SUMMARY_JSON_FILE: &str = "validation-summary.json";
const SUMMARY_MD_FILE: &str = "validation-summary.md";

#[derive(Debug, Clone)]
pub(crate) struct BootstrapValidateOptions {
    pub(crate) target: PathBuf,
    pub(crate) import_paths: Vec<PathBuf>,
    pub(crate) evidence_dir: Option<PathBuf>,
    pub(crate) warmups: Vec<PathBuf>,
    pub(crate) resume: bool,
    pub(crate) jobs: Option<u32>,
    pub(crate) strict_hermetic: bool,
    pub(crate) impure: bool,
}

#[derive(Debug, Serialize)]
struct BootstrapValidationSummary {
    schema: &'static str,
    target: String,
    evidence_dir: String,
    store: String,
    state_dir: String,
    store_prefix: String,
    resume: bool,
    doctor_ok: bool,
    build_attempted: bool,
    build_exit_code: Option<i32>,
    warmups: Vec<WarmupSummary>,
    status: ValidationStatus,
    failure_class: Option<&'static str>,
    evidence: ValidationEvidence,
    leakage_findings: Vec<LeakageFinding>,
    generated_at_unix: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ValidationStatus {
    Passed,
    BuildFailed,
    PreflightFailed,
    Running,
    WarmupFailed,
}

#[derive(Debug, Clone, Serialize)]
struct WarmupSummary {
    target: String,
    stdout: String,
    stderr: String,
    exit_code: Option<i32>,
    status: ValidationStatus,
}

#[derive(Debug, Serialize)]
struct ValidationEvidence {
    doctor_json: String,
    build_stdout: Option<String>,
    build_stderr: Option<String>,
    summary_json: String,
    summary_md: String,
}

#[derive(Debug, Serialize)]
struct LeakageFinding {
    source: &'static str,
    needle: &'static str,
    count: usize,
}

pub(crate) fn cmd_bootstrap_validate(ctx: &RunContext, opts: BootstrapValidateOptions) -> Result<(), RunError> {
    let evidence_dir = resolve_evidence_dir(opts.evidence_dir.as_deref(), &opts.target)?;
    fs::create_dir_all(&evidence_dir)
        .map_err(|err| RunError::Internal(format!("creating evidence dir {}: {err}", evidence_dir.display())))?;

    let doctor = operator_diagnostics::collect_doctor_report(operator_diagnostics::DoctorRequest {
        profile: DoctorProfile::Build,
        store_dir: &ctx.store,
        state_dir: &ctx.resolved_state_dir,
    });
    let doctor_json = doctor
        .render_json()
        .map_err(|err| RunError::Internal(format!("serializing doctor report: {err}")))?;
    write_text(&evidence_dir.join(DOCTOR_JSON_FILE), &doctor_json)?;

    if !doctor.ok {
        let summary = make_summary(
            ctx,
            &opts,
            &evidence_dir,
            false,
            None,
            Vec::new(),
            ValidationStatus::PreflightFailed,
            Some("preflight"),
            Vec::new(),
        );
        write_summaries(&evidence_dir, &summary)?;
        render_summary(ctx, &summary)?;
        return Err(RunError::Reported(3));
    }

    let import_paths = build_import_paths(&opts.import_paths)?;
    let mut warmup_summaries = Vec::with_capacity(opts.warmups.len());
    for warmup in &opts.warmups {
        let stem = evidence_stem(warmup);
        let stdout_path = evidence_dir.join(format!("warmup-{stem}.stdout.log"));
        let stderr_path = evidence_dir.join(format!("warmup-{stem}.stderr.log"));
        write_text(&stdout_path, "")?;
        write_text(&stderr_path, "")?;
        let status = run_build_child(ctx, warmup, &opts, &import_paths, &stdout_path, &stderr_path)?;
        let warmup_status = if status.success() {
            ValidationStatus::Passed
        } else {
            ValidationStatus::WarmupFailed
        };
        warmup_summaries.push(WarmupSummary {
            target: warmup.display().to_string(),
            stdout: stdout_path.display().to_string(),
            stderr: stderr_path.display().to_string(),
            exit_code: status.code(),
            status: warmup_status.clone(),
        });
        if !status.success() {
            let summary = make_summary(
                ctx,
                &opts,
                &evidence_dir,
                false,
                None,
                warmup_summaries,
                ValidationStatus::WarmupFailed,
                Some("warmup"),
                Vec::new(),
            );
            write_summaries(&evidence_dir, &summary)?;
            render_summary(ctx, &summary)?;
            return Err(RunError::Reported(1));
        }
    }
    let stdout_path = evidence_dir.join(BUILD_STDOUT_FILE);
    let stderr_path = evidence_dir.join(BUILD_STDERR_FILE);
    write_text(&stdout_path, "")?;
    write_text(&stderr_path, "")?;

    let checkpoint_summary = make_summary(
        ctx,
        &opts,
        &evidence_dir,
        true,
        None,
        warmup_summaries.clone(),
        ValidationStatus::Running,
        Some("running"),
        Vec::new(),
    );
    write_summaries(&evidence_dir, &checkpoint_summary)?;

    let status = run_build_child(ctx, &opts.target, &opts, &import_paths, &stdout_path, &stderr_path)?;

    let stdout = fs::read_to_string(&stdout_path).unwrap_or_else(|_| String::new());
    let stderr = fs::read_to_string(&stderr_path).unwrap_or_else(|_| String::new());
    let mut leakage_findings = scan_leakage("stdout", &stdout);
    leakage_findings.extend(scan_leakage("stderr", &stderr));

    let exit_code = status.code();
    let (validation_status, failure_class) = if status.success() {
        (ValidationStatus::Passed, None)
    } else {
        (ValidationStatus::BuildFailed, Some("build"))
    };
    let summary = make_summary(
        ctx,
        &opts,
        &evidence_dir,
        true,
        exit_code,
        warmup_summaries,
        validation_status,
        failure_class,
        leakage_findings,
    );
    write_summaries(&evidence_dir, &summary)?;
    render_summary(ctx, &summary)?;

    if status.success() {
        Ok(())
    } else {
        Err(RunError::Reported(1))
    }
}

fn run_build_child(
    ctx: &RunContext,
    target: &Path,
    opts: &BootstrapValidateOptions,
    import_paths: &[std::ffi::OsString],
    stdout_path: &Path,
    stderr_path: &Path,
) -> Result<ExitStatus, RunError> {
    let exe =
        std::env::current_exe().map_err(|err| RunError::Internal(format!("resolving current executable: {err}")))?;
    let mut command = Command::new(exe);
    command
        .arg("--json")
        .arg("--store")
        .arg(&ctx.store)
        .arg("--store-prefix")
        .arg(&ctx.store_prefix)
        .arg("--state-dir")
        .arg(&ctx.resolved_state_dir)
        .arg("build")
        .arg(target)
        .arg("--no-substitute");
    if opts.strict_hermetic {
        command.arg("--strict-hermetic");
    }
    if opts.impure {
        command.arg("--impure");
    }
    if let Some(jobs) = opts.jobs {
        command.arg("--jobs").arg(jobs.to_string());
    }
    for path in import_paths {
        command.arg("--import-path").arg(path);
    }

    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|err| RunError::Internal(format!("running bootstrap validation build child: {err}")))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| RunError::Internal("capturing bootstrap validation child stdout".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| RunError::Internal("capturing bootstrap validation child stderr".to_string()))?;

    let stdout_path_for_thread = stdout_path.to_path_buf();
    let stderr_path_for_thread = stderr_path.to_path_buf();
    let stdout_thread = thread::spawn(move || copy_stream_to_file(stdout, &stdout_path_for_thread));
    let stderr_thread = thread::spawn(move || copy_stream_to_file(stderr, &stderr_path_for_thread));

    let status = child
        .wait()
        .map_err(|err| RunError::Internal(format!("waiting for bootstrap validation build child: {err}")))?;

    stdout_thread
        .join()
        .map_err(|_| RunError::Internal("joining bootstrap stdout capture thread".to_string()))??;
    stderr_thread
        .join()
        .map_err(|_| RunError::Internal("joining bootstrap stderr capture thread".to_string()))??;

    Ok(status)
}

fn copy_stream_to_file<R: Read>(mut reader: R, path: &Path) -> Result<(), RunError> {
    let mut file =
        fs::File::create(path).map_err(|err| RunError::Internal(format!("creating {}: {err}", path.display())))?;
    std::io::copy(&mut reader, &mut file)
        .map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))?;
    file.flush().map_err(|err| RunError::Internal(format!("flushing {}: {err}", path.display())))
}

fn resolve_evidence_dir(evidence_dir: Option<&Path>, target: &Path) -> Result<PathBuf, RunError> {
    if let Some(path) = evidence_dir {
        return absolutize(path);
    }
    let stem = target.file_stem().and_then(|s| s.to_str()).unwrap_or("bootstrap-target");
    absolutize(Path::new("target").join("bootstrap-validation").join(stem).as_path())
}

fn evidence_stem(target: &Path) -> String {
    target
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("bootstrap-warmup")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn absolutize(path: &Path) -> Result<PathBuf, RunError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .map_err(|err| RunError::Internal(format!("resolving current directory: {err}")))
}

fn write_text(path: &Path, text: &str) -> Result<(), RunError> {
    let mut file =
        fs::File::create(path).map_err(|err| RunError::Internal(format!("creating {}: {err}", path.display())))?;
    file.write_all(text.as_bytes())
        .map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))
}

fn make_summary(
    ctx: &RunContext,
    opts: &BootstrapValidateOptions,
    evidence_dir: &Path,
    build_attempted: bool,
    build_exit_code: Option<i32>,
    warmups: Vec<WarmupSummary>,
    status: ValidationStatus,
    failure_class: Option<&'static str>,
    leakage_findings: Vec<LeakageFinding>,
) -> BootstrapValidationSummary {
    BootstrapValidationSummary {
        schema: SUMMARY_SCHEMA,
        target: opts.target.display().to_string(),
        evidence_dir: evidence_dir.display().to_string(),
        store: ctx.store.display().to_string(),
        state_dir: ctx.resolved_state_dir.display().to_string(),
        store_prefix: ctx.store_prefix.clone(),
        resume: opts.resume,
        doctor_ok: !matches!(status, ValidationStatus::PreflightFailed),
        build_attempted,
        build_exit_code,
        warmups,
        status,
        failure_class,
        evidence: ValidationEvidence {
            doctor_json: evidence_dir.join(DOCTOR_JSON_FILE).display().to_string(),
            build_stdout: build_attempted.then(|| evidence_dir.join(BUILD_STDOUT_FILE).display().to_string()),
            build_stderr: build_attempted.then(|| evidence_dir.join(BUILD_STDERR_FILE).display().to_string()),
            summary_json: evidence_dir.join(SUMMARY_JSON_FILE).display().to_string(),
            summary_md: evidence_dir.join(SUMMARY_MD_FILE).display().to_string(),
        },
        leakage_findings,
        generated_at_unix: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
    }
}

fn write_summaries(evidence_dir: &Path, summary: &BootstrapValidationSummary) -> Result<(), RunError> {
    let json = serde_json::to_string_pretty(summary)
        .map_err(|err| RunError::Internal(format!("serializing bootstrap validation summary: {err}")))?;
    write_text(&evidence_dir.join(SUMMARY_JSON_FILE), &json)?;
    write_text(&evidence_dir.join(SUMMARY_MD_FILE), &render_markdown(summary))
}

fn render_markdown(summary: &BootstrapValidationSummary) -> String {
    let mut out = String::new();
    out.push_str("# Bootstrap validation summary\n\n");
    out.push_str(&format!("- Schema: `{}`\n", summary.schema));
    out.push_str(&format!("- Target: `{}`\n", summary.target));
    out.push_str(&format!("- Status: `{:?}`\n", summary.status));
    out.push_str(&format!("- Doctor OK: `{}`\n", summary.doctor_ok));
    out.push_str(&format!("- Build attempted: `{}`\n", summary.build_attempted));
    out.push_str(&format!("- Build exit code: `{:?}`\n", summary.build_exit_code));
    if !summary.warmups.is_empty() {
        out.push_str("- Warmups:\n");
        for warmup in &summary.warmups {
            out.push_str(&format!("  - `{}`: `{:?}` exit `{:?}`\n", warmup.target, warmup.status, warmup.exit_code));
        }
    }
    out.push_str(&format!("- Store: `{}`\n", summary.store));
    out.push_str(&format!("- State dir: `{}`\n", summary.state_dir));
    out.push_str("\n## Evidence\n\n");
    out.push_str(&format!("- Doctor JSON: `{}`\n", summary.evidence.doctor_json));
    if let Some(path) = &summary.evidence.build_stdout {
        out.push_str(&format!("- Build stdout: `{path}`\n"));
    }
    if let Some(path) = &summary.evidence.build_stderr {
        out.push_str(&format!("- Build stderr: `{path}`\n"));
    }
    out.push_str("\n## Host leakage scan\n\n");
    if summary.leakage_findings.is_empty() {
        out.push_str("No coarse host-path needles were found in captured build output.\n");
    } else {
        for finding in &summary.leakage_findings {
            out.push_str(&format!("- `{}` found `{}` {} time(s)\n", finding.source, finding.needle, finding.count));
        }
    }
    out
}

fn render_summary(ctx: &RunContext, summary: &BootstrapValidationSummary) -> Result<(), RunError> {
    if ctx.json {
        let rendered = serde_json::to_string_pretty(summary)
            .map_err(|err| RunError::Internal(format!("serializing bootstrap validation summary: {err}")))?;
        println!("{rendered}");
        return Ok(());
    }
    println!("bootstrap validation: {:?}", summary.status);
    println!("target: {}", summary.target);
    println!("evidence: {}", summary.evidence_dir);
    if let Some(code) = summary.build_exit_code {
        println!("build exit code: {code}");
    }
    if !summary.leakage_findings.is_empty() {
        println!("host leakage findings: {}", summary.leakage_findings.len());
    }
    Ok(())
}

fn scan_leakage(source: &'static str, text: &str) -> Vec<LeakageFinding> {
    const NEEDLES: [&str; 5] = ["/usr/bin", "/usr/lib", "/bin/", "/lib64", "HOME="];
    NEEDLES
        .iter()
        .filter_map(|needle| {
            let count = text.matches(needle).count();
            (count > 0).then_some(LeakageFinding { source, needle, count })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_evidence_dir_uses_target_stem() {
        let dir = resolve_evidence_dir(None, Path::new("bootstrap/make-tcc.ncl")).unwrap();
        assert!(dir.ends_with("target/bootstrap-validation/make-tcc"));
    }

    #[test]
    fn warmup_evidence_stem_is_log_filename_safe() {
        assert_eq!(evidence_stem(Path::new("bootstrap/diag tcc27.ncl")), "diag-tcc27");
    }

    #[test]
    fn leakage_scan_counts_host_needles() {
        let findings = scan_leakage("stdout", "PATH=/usr/bin:/bin/tool\nHOME=/home/me\n");
        assert!(findings.iter().any(|f| f.needle == "/usr/bin" && f.count == 1));
        assert!(findings.iter().any(|f| f.needle == "/bin/" && f.count == 1));
        assert!(findings.iter().any(|f| f.needle == "HOME=" && f.count == 1));
    }

    #[test]
    fn running_status_serializes_for_checkpoint_summary() {
        let rendered = serde_json::to_string(&ValidationStatus::Running).unwrap();
        assert_eq!(rendered, "\"running\"");
    }

    #[test]
    fn copy_stream_to_file_flushes_partial_transcript() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("build.stdout.log");
        copy_stream_to_file("first line\nsecond line\n".as_bytes(), &path).unwrap();
        assert_eq!(fs::read_to_string(path).unwrap(), "first line\nsecond line\n");
    }
}
