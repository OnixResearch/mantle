// machine-artifact-public: bootstrap.validation-reports
use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::thread;

use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::CommandFamily;
use mantle_application_contract::EffectKind;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::EffectPlan;
use mantle_application_contract::EffectSpec;
use mantle_application_contract::ExpectedOutput;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::classify_observations;
use mantle_application_contract::plan_effects;
use serde::Serialize;

use crate::RunContext;
use crate::build_cmd::build_import_paths;
use crate::errors::RunError;
use crate::operator_diagnostics::DoctorEnvironmentReader;
use crate::operator_diagnostics::DoctorProfile;
use crate::operator_diagnostics::{self};

const SUMMARY_SCHEMA: &str = "crunch-bootstrap-validation-v1";
const BUILD_STDOUT_FILE: &str = "build.stdout.log";
const BUILD_STDERR_FILE: &str = "build.stderr.log";
const DOCTOR_JSON_FILE: &str = "doctor.json";
const SUMMARY_JSON_FILE: &str = "validation-summary.json";
const SUMMARY_MD_FILE: &str = "validation-summary.md";
const MAX_WARMUP_BUILDS: usize = 4_095;
const EFFECT_CWD: &str = "bootstrap-cwd";
const EFFECT_EVIDENCE_DIR: &str = "bootstrap-evidence-dir";
const EFFECT_DOCTOR: &str = "bootstrap-doctor";
const EFFECT_DOCTOR_ENV: &str = "bootstrap-doctor-environment";
const EFFECT_DOCTOR_JSON: &str = "bootstrap-doctor-json";
const EFFECT_IMPORT_PATHS: &str = "bootstrap-import-paths";
const EFFECT_CLOCK: &str = "bootstrap-clock";
const EFFECT_LOG_WRITE: &str = "bootstrap-log-write";
const EFFECT_CHILD: &str = "bootstrap-build-child";
const EFFECT_LOG_READ: &str = "bootstrap-log-read";
const EFFECT_SUMMARY_WRITE: &str = "bootstrap-summary-write";
const EFFECT_SUMMARY_READ: &str = "bootstrap-summary-readback";

/// The concrete effect port. A plan is admitted before even cwd resolution;
/// each observation is recorded only after the corresponding OS call.
struct BootstrapPort {
    plan: EffectPlan,
    observations: Vec<Observation>,
    doctor_env_reads: [u32; 3],
}

impl BootstrapPort {
    fn new(warmups: usize, needs_cwd: bool) -> Result<Self, RunError> {
        let child_calls = u32::try_from(warmups)
            .ok()
            .and_then(|count| count.checked_add(1))
            .filter(|_| warmups <= MAX_WARMUP_BUILDS)
            .ok_or_else(|| RunError::Internal("too many bootstrap validation warmups".to_string()))?;
        let summary_calls = child_calls + 1;
        let specs = [
            (EFFECT_CWD, EffectKind::ReadFiles, 1),
            (EFFECT_EVIDENCE_DIR, EffectKind::WriteFiles, 1),
            (EFFECT_DOCTOR_ENV, EffectKind::ReadEnvironment, 4),
            (EFFECT_DOCTOR, EffectKind::ReadFiles, 1),
            (EFFECT_DOCTOR_JSON, EffectKind::WriteFiles, 1),
            (EFFECT_IMPORT_PATHS, EffectKind::ReadFiles, 1),
            (EFFECT_CLOCK, EffectKind::ReadClock, summary_calls),
            (EFFECT_LOG_WRITE, EffectKind::WriteFiles, child_calls * 4),
            (EFFECT_CHILD, EffectKind::RunProcess, child_calls),
            (EFFECT_LOG_READ, EffectKind::ReadFiles, 2),
            (EFFECT_SUMMARY_WRITE, EffectKind::WriteFiles, summary_calls * 2),
            (EFFECT_SUMMARY_READ, EffectKind::ReadFiles, summary_calls * 2),
        ]
        .map(|(effect_id, kind, calls)| EffectSpec {
            effect_id,
            kind,
            limit: EffectMeasure::Calls(calls),
            expected_output: ExpectedOutput::None,
        });
        let specs = if needs_cwd { &specs[..] } else { &specs[1..] };
        let plan = plan_effects(CommandFamily::Bootstrap, specs)
            .map_err(|err| RunError::Internal(format!("planning bootstrap validation effects: {}", err.code())))?;
        let observations = plan
            .effects
            .iter()
            .map(|effect| Observation {
                effect_id: effect.effect_id.clone(),
                kind: effect.kind,
                status: ObservationStatus::Skipped,
                output: EffectOutput::None,
                usage: EffectMeasure::Calls(0),
                diagnostics_code: None,
            })
            .collect();
        Ok(Self {
            plan,
            observations,
            doctor_env_reads: [0; 3],
        })
    }

    fn observe(&mut self, effect_id: &str, succeeded: bool) {
        let observation = self
            .observations
            .iter_mut()
            .find(|item| item.effect_id.0 == effect_id)
            .expect("bootstrap effect is declared before execution");
        if let EffectMeasure::Calls(count) = &mut observation.usage {
            *count += 1;
        }
        if !succeeded {
            observation.status = ObservationStatus::Failed;
            observation.diagnostics_code = Some("observed-failure".to_string());
        } else if observation.status == ObservationStatus::Skipped {
            observation.status = ObservationStatus::Succeeded;
        }
    }

    fn classify(&self, expected_failure: bool) -> Result<(), RunError> {
        match (classify_observations(&self.plan, &self.observations), expected_failure) {
            (ApplicationOutcome::Completed, false) | (ApplicationOutcome::Failed { .. }, true) => Ok(()),
            (other, _) => {
                Err(RunError::Internal(format!("bootstrap validation effect observation mismatch: {other:?}")))
            }
        }
    }

    fn evidence_dir(&mut self, path: &Path) -> Result<(), RunError> {
        let result = fs::create_dir_all(path);
        self.observe(EFFECT_EVIDENCE_DIR, result.is_ok());
        result.map_err(|err| RunError::Internal(format!("creating evidence dir {}: {err}", path.display())))
    }

    fn doctor(&mut self, ctx: &RunContext) -> operator_diagnostics::PreflightReport {
        let doctor = operator_diagnostics::collect_doctor_report_with_env(
            operator_diagnostics::DoctorRequest {
                profile: DoctorProfile::Build,
                store_dir: &ctx.store,
                state_dir: &ctx.resolved_state_dir,
            },
            self,
        );
        if self.doctor_env_reads[0] != 1 || self.doctor_env_reads[1] != 1 || self.doctor_env_reads[2] > 2 {
            let observation = self
                .observations
                .iter_mut()
                .find(|item| item.effect_id.0 == EFFECT_DOCTOR_ENV)
                .expect("doctor environment effect planned");
            observation.status = ObservationStatus::Failed;
            observation.diagnostics_code = Some("unexpected-doctor-environment-reads".to_string());
        }
        self.observe(EFFECT_DOCTOR, doctor.ok);
        doctor
    }

    fn clock(&mut self) -> Result<u64, RunError> {
        let result = crate::unix_time_now_s();
        self.observe(EFFECT_CLOCK, result.is_ok());
        result
    }

    fn write(&mut self, effect_id: &str, path: &Path, text: &str) -> Result<(), RunError> {
        let result = write_text(path, text);
        self.observe(effect_id, result.is_ok());
        result
    }

    fn read(&mut self, effect_id: &str, path: &Path) -> Result<String, RunError> {
        let result = fs::read_to_string(path)
            .map_err(|error| RunError::Internal(format!("reading {}: {error}", path.display())));
        self.observe(effect_id, result.is_ok());
        result
    }

    fn summary_readback(
        &mut self,
        evidence_dir: &Path,
        expected_json: &str,
        expected_markdown: &str,
    ) -> Result<(), RunError> {
        let json_path = evidence_dir.join(SUMMARY_JSON_FILE);
        let markdown_path = evidence_dir.join(SUMMARY_MD_FILE);
        let json = match self.read(EFFECT_SUMMARY_READ, &json_path) {
            Ok(json) => json,
            Err(error) => {
                self.classify(true)?;
                return Err(error);
            }
        };
        let markdown = match self.read(EFFECT_SUMMARY_READ, &markdown_path) {
            Ok(markdown) => markdown,
            Err(error) => {
                self.classify(true)?;
                return Err(error);
            }
        };
        if json != expected_json || markdown != expected_markdown {
            let observation = self
                .observations
                .iter_mut()
                .find(|item| item.effect_id.0 == EFFECT_SUMMARY_READ)
                .expect("readback effect planned");
            observation.status = ObservationStatus::Failed;
            observation.diagnostics_code = Some("summary-readback-mismatch".to_string());
            self.classify(true)?;
            return Err(RunError::Internal(
                "bootstrap validation summary readback differs from persisted evidence".to_string(),
            ));
        }
        Ok(())
    }
}

impl DoctorEnvironmentReader for BootstrapPort {
    fn var_os(&mut self, key: &'static str) -> Option<OsString> {
        let (index, maximum) = match key {
            operator_diagnostics::BWRAP_PATH_ENV => (0, 1),
            "SNIX_BUILD_SANDBOX_SHELL" => (1, 1),
            "PATH" => (2, 2),
            unexpected => panic!("unplanned doctor environment read: {unexpected}"),
        };
        assert!(self.doctor_env_reads[index] < maximum, "doctor environment key read above admitted bound: {key}");
        let value = std::env::var_os(key);
        self.doctor_env_reads[index] += 1;
        self.observe(EFFECT_DOCTOR_ENV, true);
        value
    }
}

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
    debug_assert!(!SUMMARY_SCHEMA.is_empty());
    debug_assert!(!DOCTOR_JSON_FILE.is_empty());
    let needs_cwd = opts.evidence_dir.as_deref().is_none_or(|path| !path.is_absolute());
    let mut port = BootstrapPort::new(opts.warmups.len(), needs_cwd)?;
    let result = run_bootstrap_validate(ctx, &opts, &mut port);
    if result.is_err() {
        // A pure renderer can fail after completed effects; all other early
        // errors must classify the actual partial ledger as failed.
        match classify_observations(&port.plan, &port.observations) {
            ApplicationOutcome::Completed | ApplicationOutcome::Failed { .. } => {}
            other => {
                return Err(RunError::Internal(format!("bootstrap validation effect observation mismatch: {other:?}")));
            }
        }
    }
    result
}

fn run_bootstrap_validate(
    ctx: &RunContext,
    opts: &BootstrapValidateOptions,
    port: &mut BootstrapPort,
) -> Result<(), RunError> {
    let evidence_dir = resolve_evidence_dir(opts.evidence_dir.as_deref(), &opts.target, port)?;
    port.evidence_dir(&evidence_dir)?;
    let doctor = port.doctor(ctx);
    let doctor_json = doctor
        .render_json()
        .map_err(|err| RunError::Internal(format!("serializing doctor report: {err}")))?;
    port.write(EFFECT_DOCTOR_JSON, &evidence_dir.join(DOCTOR_JSON_FILE), &doctor_json)?;
    if !doctor.ok {
        let summary = make_summary(
            SummaryRequest {
                ctx,
                opts,
                evidence_dir: &evidence_dir,
                build_attempted: false,
                build_exit_code: None,
                warmups: Vec::new(),
                status: ValidationStatus::PreflightFailed,
                failure_class: Some("preflight"),
                leakage_findings: Vec::new(),
            },
            port,
        )?;
        let (json, markdown) = write_summaries(&evidence_dir, &summary, port)?;
        port.summary_readback(&evidence_dir, &json, &markdown)?;
        port.classify(true)?;
        render_summary(ctx, &summary)?;
        return Err(RunError::Reported(3));
    }
    let evaluation_search_paths = build_import_paths(&opts.import_paths);
    port.observe(EFFECT_IMPORT_PATHS, evaluation_search_paths.is_ok());
    let evaluation_search_paths = evaluation_search_paths?;
    let warmup_summaries = run_warmup_builds(ctx, opts, &evidence_dir, &evaluation_search_paths, port)?;
    run_primary_build(ctx, opts, &evidence_dir, &evaluation_search_paths, warmup_summaries, port)
}

fn run_warmup_builds(
    ctx: &RunContext,
    opts: &BootstrapValidateOptions,
    evidence_dir: &Path,
    evaluation_search_paths: &[std::ffi::OsString],
    port: &mut BootstrapPort,
) -> Result<Vec<WarmupSummary>, RunError> {
    debug_assert!(!SUMMARY_SCHEMA.is_empty());
    debug_assert!(!SUMMARY_MD_FILE.is_empty());
    let mut warmup_summaries = Vec::with_capacity(opts.warmups.len());
    for warmup in &opts.warmups {
        let stem = evidence_stem(warmup);
        let stdout_path = evidence_dir.join(format!("warmup-{stem}.stdout.log"));
        let stderr_path = evidence_dir.join(format!("warmup-{stem}.stderr.log"));
        port.write(EFFECT_LOG_WRITE, &stdout_path, "")?;
        port.write(EFFECT_LOG_WRITE, &stderr_path, "")?;
        let status = run_build_child(
            BuildChildRequest {
                ctx,
                target: warmup,
                opts,
                evaluation_search_paths,
                stdout_path: &stdout_path,
                stderr_path: &stderr_path,
            },
            port,
        )?;
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
            status: warmup_status,
        });
        if !status.success() {
            let summary = make_summary(
                SummaryRequest {
                    ctx,
                    opts,
                    evidence_dir,
                    build_attempted: false,
                    build_exit_code: None,
                    warmups: warmup_summaries,
                    status: ValidationStatus::WarmupFailed,
                    failure_class: Some("warmup"),
                    leakage_findings: Vec::new(),
                },
                port,
            )?;
            let (json, markdown) = write_summaries(evidence_dir, &summary, port)?;
            port.summary_readback(evidence_dir, &json, &markdown)?;
            port.classify(true)?;
            render_summary(ctx, &summary)?;
            return Err(RunError::Reported(1));
        }
    }
    Ok(warmup_summaries)
}

fn run_primary_build(
    ctx: &RunContext,
    opts: &BootstrapValidateOptions,
    evidence_dir: &Path,
    evaluation_search_paths: &[std::ffi::OsString],
    warmup_summaries: Vec<WarmupSummary>,
    port: &mut BootstrapPort,
) -> Result<(), RunError> {
    debug_assert!(!BUILD_STDOUT_FILE.is_empty());
    debug_assert!(!BUILD_STDERR_FILE.is_empty());
    let stdout_path = evidence_dir.join(BUILD_STDOUT_FILE);
    let stderr_path = evidence_dir.join(BUILD_STDERR_FILE);
    port.write(EFFECT_LOG_WRITE, &stdout_path, "")?;
    port.write(EFFECT_LOG_WRITE, &stderr_path, "")?;
    let checkpoint_summary = make_summary(
        SummaryRequest {
            ctx,
            opts,
            evidence_dir,
            build_attempted: true,
            build_exit_code: None,
            warmups: warmup_summaries.clone(),
            status: ValidationStatus::Running,
            failure_class: Some("running"),
            leakage_findings: Vec::new(),
        },
        port,
    )?;
    write_summaries(evidence_dir, &checkpoint_summary, port)?;
    let status = run_build_child(
        BuildChildRequest {
            ctx,
            target: &opts.target,
            opts,
            evaluation_search_paths,
            stdout_path: &stdout_path,
            stderr_path: &stderr_path,
        },
        port,
    )?;
    let stdout = port.read(EFFECT_LOG_READ, &stdout_path)?;
    let stderr = port.read(EFFECT_LOG_READ, &stderr_path)?;
    let mut leakage_findings = scan_leakage("stdout", &stdout);
    leakage_findings.extend(scan_leakage("stderr", &stderr));
    let (validation_status, failure_class) = if status.success() {
        (ValidationStatus::Passed, None)
    } else {
        (ValidationStatus::BuildFailed, Some("build"))
    };
    let summary = make_summary(
        SummaryRequest {
            ctx,
            opts,
            evidence_dir,
            build_attempted: true,
            build_exit_code: status.code(),
            warmups: warmup_summaries,
            status: validation_status,
            failure_class,
            leakage_findings,
        },
        port,
    )?;
    let (json, markdown) = write_summaries(evidence_dir, &summary, port)?;
    port.summary_readback(evidence_dir, &json, &markdown)?;
    port.classify(!status.success())?;
    render_summary(ctx, &summary)?;
    if status.success() {
        Ok(())
    } else {
        Err(RunError::Reported(1))
    }
}

struct BuildChildRequest<'a> {
    ctx: &'a RunContext,
    target: &'a Path,
    opts: &'a BootstrapValidateOptions,
    evaluation_search_paths: &'a [std::ffi::OsString],
    stdout_path: &'a Path,
    stderr_path: &'a Path,
}

fn run_build_child(request: BuildChildRequest<'_>, port: &mut BootstrapPort) -> Result<ExitStatus, RunError> {
    let result = run_build_child_with_os(request);
    port.observe(EFFECT_CHILD, result.as_ref().is_ok_and(ExitStatus::success));
    if result.is_ok() {
        // Both transcript-copy threads completed and flushed before the child status returned.
        port.observe(EFFECT_LOG_WRITE, true);
        port.observe(EFFECT_LOG_WRITE, true);
    }
    result
}

fn run_build_child_with_os(request: BuildChildRequest<'_>) -> Result<ExitStatus, RunError> {
    debug_assert!(!BUILD_STDOUT_FILE.is_empty());
    debug_assert!(!BUILD_STDERR_FILE.is_empty());
    let exe =
        std::env::current_exe().map_err(|err| RunError::Internal(format!("resolving current executable: {err}")))?;
    let mut command = Command::new(exe);
    command
        .arg("--json")
        .arg("--store")
        .arg(&request.ctx.store)
        .arg("--store-prefix")
        .arg(&request.ctx.store_prefix)
        .arg("--state-dir")
        .arg(&request.ctx.resolved_state_dir)
        .arg("--store-backend")
        .arg(request.ctx.store_backend.as_str())
        .arg("build")
        .arg(request.target)
        .arg("--no-substitute");
    if request.opts.strict_hermetic {
        command.arg("--strict-hermetic");
    }
    if request.opts.impure {
        command.arg("--impure");
    }
    if let Some(jobs) = request.opts.jobs {
        command.arg("--jobs").arg(jobs.to_string());
    }
    for path in request.evaluation_search_paths {
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

    let stdout_path_for_thread = request.stdout_path.to_path_buf();
    let stderr_path_for_thread = request.stderr_path.to_path_buf();
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

fn resolve_evidence_dir(
    evidence_dir: Option<&Path>,
    target: &Path,
    port: &mut BootstrapPort,
) -> Result<PathBuf, RunError> {
    if let Some(path) = evidence_dir {
        return absolutize(path, port);
    }
    let stem = target.file_stem().and_then(|s| s.to_str()).unwrap_or("bootstrap-target");
    absolutize(Path::new("target").join("bootstrap-validation").join(stem).as_path(), port)
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

fn absolutize(path: &Path, port: &mut BootstrapPort) -> Result<PathBuf, RunError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let current_dir = std::env::current_dir();
    port.observe(EFFECT_CWD, current_dir.is_ok());
    current_dir
        .map(|cwd| cwd.join(path))
        .map_err(|err| RunError::Internal(format!("resolving current directory: {err}")))
}

fn write_text(path: &Path, text: &str) -> Result<(), RunError> {
    let mut file =
        fs::File::create(path).map_err(|err| RunError::Internal(format!("creating {}: {err}", path.display())))?;
    file.write_all(text.as_bytes())
        .map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))
}

struct SummaryRequest<'a> {
    ctx: &'a RunContext,
    opts: &'a BootstrapValidateOptions,
    evidence_dir: &'a Path,
    build_attempted: bool,
    build_exit_code: Option<i32>,
    warmups: Vec<WarmupSummary>,
    status: ValidationStatus,
    failure_class: Option<&'static str>,
    leakage_findings: Vec<LeakageFinding>,
}

fn make_summary(request: SummaryRequest<'_>, port: &mut BootstrapPort) -> Result<BootstrapValidationSummary, RunError> {
    debug_assert!(!SUMMARY_SCHEMA.is_empty());
    debug_assert!(!SUMMARY_JSON_FILE.is_empty());
    let is_doctor_ok = !matches!(request.status, ValidationStatus::PreflightFailed);
    let generated_at_unix = port.clock()?;
    Ok(BootstrapValidationSummary {
        schema: SUMMARY_SCHEMA,
        target: request.opts.target.display().to_string(),
        evidence_dir: request.evidence_dir.display().to_string(),
        store: request.ctx.store.display().to_string(),
        state_dir: request.ctx.resolved_state_dir.display().to_string(),
        store_prefix: request.ctx.store_prefix.clone(),
        resume: request.opts.resume,
        doctor_ok: is_doctor_ok,
        build_attempted: request.build_attempted,
        build_exit_code: request.build_exit_code,
        warmups: request.warmups,
        status: request.status,
        failure_class: request.failure_class,
        evidence: ValidationEvidence {
            doctor_json: request.evidence_dir.join(DOCTOR_JSON_FILE).display().to_string(),
            build_stdout: request
                .build_attempted
                .then(|| request.evidence_dir.join(BUILD_STDOUT_FILE).display().to_string()),
            build_stderr: request
                .build_attempted
                .then(|| request.evidence_dir.join(BUILD_STDERR_FILE).display().to_string()),
            summary_json: request.evidence_dir.join(SUMMARY_JSON_FILE).display().to_string(),
            summary_md: request.evidence_dir.join(SUMMARY_MD_FILE).display().to_string(),
        },
        leakage_findings: request.leakage_findings,
        generated_at_unix,
    })
}

fn write_summaries(
    evidence_dir: &Path,
    summary: &BootstrapValidationSummary,
    port: &mut BootstrapPort,
) -> Result<(String, String), RunError> {
    let json = serde_json::to_string_pretty(summary)
        .map_err(|err| RunError::Internal(format!("serializing bootstrap validation summary: {err}")))?;
    port.write(EFFECT_SUMMARY_WRITE, &evidence_dir.join(SUMMARY_JSON_FILE), &json)?;
    let markdown = render_markdown(summary);
    port.write(EFFECT_SUMMARY_WRITE, &evidence_dir.join(SUMMARY_MD_FILE), &markdown)?;
    Ok((json, markdown))
}

fn render_markdown(summary: &BootstrapValidationSummary) -> String {
    debug_assert_eq!(summary.schema, SUMMARY_SCHEMA);
    debug_assert!(!summary.evidence.summary_json.is_empty());
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
    debug_assert!(out.starts_with("# Bootstrap validation summary"));
    debug_assert!(out.contains("## Evidence"));
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

fn scan_leakage(source: &'static str, text: impl AsRef<str>) -> Vec<LeakageFinding> {
    const NEEDLES: [&str; 5] = ["/usr/bin", "/usr/lib", "/bin/", "/lib64", "HOME="];
    let text = text.as_ref();
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
        let mut port = BootstrapPort::new(0, true).unwrap();
        let dir = resolve_evidence_dir(None, Path::new("bootstrap/make-tcc.ncl"), &mut port).unwrap();
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
