// machine-artifact-public: nickel-export.report
// machine-artifact-public: nickel-export.receipt
// r[impl mantle.nickel_export_cutover.authority]
// r[impl mantle.nickel_export_cutover.rollback]
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;
use crate::nickel_export_core_adapter::AdapterFailure;
use crate::nickel_export_core_adapter::AdapterRequest;
use crate::nickel_export_core_adapter::CanonicalAdmission;
use crate::nickel_export_core_adapter::ExplicitArtifact;
use crate::nickel_export_core_adapter::ExplicitObservation;

pub const NICKEL_EXPORT_REPORT_SCHEMA: &str = "mantle-nickel-export-report-v1";
pub const NICKEL_EXPORT_RECEIPT_SCHEMA: &str = "mantle-nickel-export-receipt-v1";
pub const NICKEL_EXPORT_NON_CLAIM: &str = "Nickel export success proves only the declared evaluation output digest under the recorded evaluator descriptor; it does not prove deployability, frontend correctness, or build success";

const FORMAT_JSON: &str = "json";
const OUTPUT_TARGET_STDOUT: &str = "stdout";
const DEFAULT_EVALUATOR_ID: &str = "mantle-embedded-crunch-eval";
const EXPORT_FAILURE_EXIT_CODE: u8 = 3;
const KIB_BYTES: u64 = 1024;
const MIB_BYTES: u64 = KIB_BYTES * KIB_BYTES;
const MAX_EXPORT_SOURCE_BYTES: u64 = 16 * MIB_BYTES;
const MAX_EXPORT_OUTPUT_BYTES: usize = 64 * MIB_BYTES as usize;
#[cfg(unix)]
const NO_FOLLOW_OPEN_FLAGS: i32 = libc::O_NOFOLLOW;

#[derive(Debug, Clone)]
pub struct NickelExportOptions<'a> {
    pub root: &'a Path,
    pub file: &'a Path,
    pub deps: &'a [PathBuf],
    pub import_paths: &'a [PathBuf],
    pub format: &'a str,
    pub out: Option<&'a Path>,
    pub evaluator_id: &'a str,
    pub evaluator_version: &'a str,
    pub json: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NickelExportReport {
    pub schema: String,
    pub success: bool,
    pub format: String,
    pub output_target: String,
    pub receipt_digest_blake3: Option<String>,
    pub output_digest_blake3: Option<String>,
    pub failure_class: Option<String>,
    pub diagnostics: Vec<NickelExportDiagnostic>,
    pub receipt: Option<NickelExportReceipt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NickelExportDiagnostic {
    pub class: String,
    pub subject: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NickelExportReceipt {
    pub schema: String,
    pub root_source: ExportSourceRef,
    pub deps: Vec<ExportSourceRef>,
    pub import_paths: Vec<String>,
    pub format: String,
    pub output_target: String,
    pub output_digest_blake3: String,
    pub evaluator: NickelEvaluatorDescriptor,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportSourceRef {
    pub path: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NickelEvaluatorDescriptor {
    pub identity: String,
    pub version: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedExportRequest {
    file: String,
    deps: Vec<String>,
    import_paths: Vec<String>,
    format: String,
    output_target: String,
    evaluator: NickelEvaluatorDescriptor,
}

#[derive(Debug, Clone)]
struct CapturedSource {
    source_ref: ExportSourceRef,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
struct CapturedSources {
    root_source: CapturedSource,
    deps: Vec<CapturedSource>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CutoverAuthority {
    Legacy,
    Canonical,
}

const CANONICAL_CUTOVER_VALIDATED: bool = true;

fn active_cutover_authority() -> CutoverAuthority {
    if CANONICAL_CUTOVER_VALIDATED {
        CutoverAuthority::Canonical
    } else {
        CutoverAuthority::Legacy
    }
}

pub fn cmd_nickel_export(options: NickelExportOptions<'_>) -> Result<(), RunError> {
    let normalized = match normalize_export_options(&options) {
        Ok(normalized) => normalized,
        Err(diagnostics) => {
            let report = failed_report(options.format, output_target_label(options.out), "validation", diagnostics);
            render_report(&report, options.json)?;
            return Err(RunError::Reported(EXPORT_FAILURE_EXIT_CODE));
        }
    };

    let captured = match capture_sources(options.root, &normalized) {
        Ok(captured) => captured,
        Err(diagnostics) => {
            let report = failed_report(&normalized.format, normalized.output_target.clone(), "source", diagnostics);
            render_report(&report, options.json)?;
            return Err(RunError::Reported(EXPORT_FAILURE_EXIT_CODE));
        }
    };

    let output = match evaluate_export_output(options.root, &normalized) {
        Ok(output) => output,
        Err(message) => {
            return handle_evaluator_failure(&normalized, &captured, message, options.json);
        }
    };
    if output.len() > MAX_EXPORT_OUTPUT_BYTES {
        let report = failed_report(&normalized.format, normalized.output_target.clone(), "eval", vec![diagnostic(
            "output-bound",
            &normalized.file,
            format!("Nickel export output exceeds {MAX_EXPORT_OUTPUT_BYTES} bytes"),
        )]);
        render_report(&report, options.json)?;
        return Err(RunError::Reported(EXPORT_FAILURE_EXIT_CODE));
    }
    if let Err(diagnostics) = verify_captured_sources_unchanged(options.root, &captured) {
        let report = failed_report(&normalized.format, normalized.output_target.clone(), "source-changed", diagnostics);
        render_report(&report, options.json)?;
        return Err(RunError::Reported(EXPORT_FAILURE_EXIT_CODE));
    }

    let legacy_receipt = build_legacy_export_receipt(&normalized, &captured, output.as_bytes());
    let canonical_result = canonical_admission(&normalized, &captured, output.as_bytes(), &legacy_receipt);
    let receipt = match select_authoritative_receipt(active_cutover_authority(), &legacy_receipt, canonical_result) {
        Ok(receipt) => receipt,
        Err(failure) => {
            let report = failed_report(
                &normalized.format,
                normalized.output_target.clone(),
                &failure.class,
                failure.diagnostics,
            );
            render_report(&report, options.json)?;
            return Err(RunError::Reported(EXPORT_FAILURE_EXIT_CODE));
        }
    };
    let receipt_digest = crate::nickel_export_core_adapter::mantle_receipt_digest(&receipt)
        .map_err(|failure| RunError::Internal(adapter_failure_message(&failure)))?;
    if normalized.output_target != OUTPUT_TARGET_STDOUT {
        write_output_file(options.root, Path::new(&normalized.output_target), output.as_bytes())?;
    }
    let report = success_report(normalized, receipt, receipt_digest, output);
    render_success(&report, options.json)
}

fn normalize_export_options(
    options: &NickelExportOptions<'_>,
) -> Result<NormalizedExportRequest, Vec<NickelExportDiagnostic>> {
    let mut diagnostics = Vec::new();
    let file = path_text(options.file, "non-utf8-source-path", &mut diagnostics);
    let deps = path_text_list(options.deps, "non-utf8-dependency-path", &mut diagnostics);
    let import_paths = path_text_list(options.import_paths, "non-utf8-import-path", &mut diagnostics);
    let output_target = output_target_text(options.out, &mut diagnostics);
    if options.evaluator_id.trim().is_empty() {
        diagnostics.push(diagnostic(
            "invalid-evaluator",
            "evaluator.identity",
            "evaluator id must not be empty".to_string(),
        ));
    }
    if options.evaluator_version.trim().is_empty() {
        diagnostics.push(diagnostic(
            "invalid-evaluator",
            "evaluator.version",
            "evaluator version must not be empty".to_string(),
        ));
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let evaluator = NickelEvaluatorDescriptor {
        identity: options.evaluator_id.to_string(),
        version: options.evaluator_version.to_string(),
        options: vec![
            format!("format={}", options.format),
            format!("import-paths={}", import_paths.len()),
        ],
    };
    let adapter_request = AdapterRequest {
        source: &file,
        dependencies: &deps,
        import_paths: &import_paths,
        format: options.format,
        output_target: &output_target,
        evaluator: &evaluator,
    };
    let canonical = crate::nickel_export_core_adapter::normalize_adapter_request(&adapter_request)
        .map_err(|failure| failure.diagnostics)?;
    Ok(NormalizedExportRequest {
        file: canonical.source,
        deps: canonical.dependencies,
        import_paths: canonical.import_paths,
        format: canonical.format.as_str().to_string(),
        output_target: canonical.destination,
        evaluator,
    })
}

fn capture_sources(
    root: &Path,
    request: &NormalizedExportRequest,
) -> Result<CapturedSources, Vec<NickelExportDiagnostic>> {
    let mut diagnostics = Vec::new();
    let root_source = match capture_source(root, &request.file) {
        Ok(source) => Some(source),
        Err(failure) => {
            diagnostics.push(failure);
            None
        }
    };
    let mut deps = Vec::with_capacity(request.deps.len());
    for dep in &request.deps {
        match capture_source(root, dep) {
            Ok(source) => deps.push(source),
            Err(failure) => diagnostics.push(failure),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let Some(root_source) = root_source else {
        return Err(vec![diagnostic(
            "missing-source",
            &request.file,
            "root source capture failed without a diagnostic".to_string(),
        )]);
    };
    Ok(CapturedSources { root_source, deps })
}

fn verify_captured_sources_unchanged(
    root: &Path,
    captured: &CapturedSources,
) -> Result<(), Vec<NickelExportDiagnostic>> {
    let sources = core::iter::once(&captured.root_source).chain(captured.deps.iter());
    let mut diagnostics = Vec::new();
    for expected in sources {
        match capture_source(root, &expected.source_ref.path) {
            Ok(actual) if actual.bytes == expected.bytes && actual.source_ref == expected.source_ref => {}
            Ok(_) => diagnostics.push(diagnostic(
                "source-changed",
                &expected.source_ref.path,
                "declared source bytes changed during embedded evaluation".to_string(),
            )),
            Err(failure) => diagnostics.push(failure),
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn capture_source(root: &Path, relative_path: &str) -> Result<CapturedSource, NickelExportDiagnostic> {
    let relative = Path::new(relative_path);
    reject_symlink_components(root, relative, false)
        .map_err(|message| diagnostic("symlink-source", relative_path, message))?;
    let path = root.join(relative);
    let metadata = fs::symlink_metadata(&path).map_err(|error| {
        diagnostic(
            "missing-source",
            relative_path,
            format!("declared Nickel export source `{relative_path}` could not be inspected: {error}"),
        )
    })?;
    if !metadata.file_type().is_file() {
        return Err(diagnostic(
            "source-not-regular",
            relative_path,
            "declared Nickel export source must be a no-follow regular file".to_string(),
        ));
    }
    if metadata.len() > MAX_EXPORT_SOURCE_BYTES {
        return Err(diagnostic(
            "source-bound",
            relative_path,
            format!("declared Nickel export source exceeds {MAX_EXPORT_SOURCE_BYTES} bytes"),
        ));
    }
    let mut file = open_regular_file_no_follow(&path).map_err(|error| {
        diagnostic(
            "source-open",
            relative_path,
            format!("declared Nickel export source `{relative_path}` could not be opened: {error}"),
        )
    })?;
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or(0));
    file.read_to_end(&mut bytes).map_err(|error| {
        diagnostic(
            "source-read",
            relative_path,
            format!("declared Nickel export source `{relative_path}` could not be read: {error}"),
        )
    })?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) != metadata.len() {
        return Err(diagnostic(
            "source-changed",
            relative_path,
            "declared Nickel export source changed while it was captured".to_string(),
        ));
    }
    Ok(CapturedSource {
        source_ref: crate::nickel_export_core_adapter::export_source_ref(relative_path, &bytes),
        bytes,
    })
}

fn evaluate_export_output(root: &Path, request: &NormalizedExportRequest) -> Result<String, String> {
    assert_eq!(request.format, FORMAT_JSON, "caller validates export format");
    let file = root.join(&request.file);
    let import_paths =
        request.import_paths.iter().map(|path| root.join(path).into_os_string()).collect::<Vec<OsString>>();
    crunch_eval::evaluate_to_json(&file, &import_paths).map_err(|error| error.to_string())
}

fn handle_evaluator_failure(
    request: &NormalizedExportRequest,
    captured: &CapturedSources,
    message: String,
    json: bool,
) -> Result<(), RunError> {
    let legacy_diagnostic = diagnostic("eval", &request.file, message);
    let diagnostics = [legacy_diagnostic.clone()];
    let dependency_artifacts = explicit_dependencies(captured);
    let adapter_request = adapter_request(request);
    let observation = ExplicitObservation {
        request: adapter_request,
        source: ExplicitArtifact {
            path: &captured.root_source.source_ref.path,
            bytes: &captured.root_source.bytes,
        },
        dependencies: &dependency_artifacts,
        output: ExplicitArtifact {
            path: &request.output_target,
            bytes: &[],
        },
        diagnostics: &diagnostics,
    };
    let comparison = crate::nickel_export_core_adapter::compare_evaluator_failure(&observation);
    let (failure_class, report_diagnostics) = match (active_cutover_authority(), comparison) {
        (_, Ok(evidence)) if evidence.diagnostics_match && !evidence.receipt_emitted => {
            ("eval".to_string(), vec![legacy_diagnostic])
        }
        (CutoverAuthority::Legacy, _) => ("eval".to_string(), vec![legacy_diagnostic]),
        (CutoverAuthority::Canonical, Err(failure)) => (failure.class, failure.diagnostics),
        (CutoverAuthority::Canonical, Ok(_)) => ("dual-run-drift".to_string(), vec![diagnostic(
            "dual-run-drift",
            &request.file,
            "evaluator failure comparison did not preserve the expected failure boundary".to_string(),
        )]),
    };
    let report = failed_report(&request.format, request.output_target.clone(), &failure_class, report_diagnostics);
    render_report(&report, json)?;
    Err(RunError::Reported(EXPORT_FAILURE_EXIT_CODE))
}

fn build_legacy_export_receipt(
    request: &NormalizedExportRequest,
    captured: &CapturedSources,
    output: &[u8],
) -> NickelExportReceipt {
    NickelExportReceipt {
        schema: NICKEL_EXPORT_RECEIPT_SCHEMA.to_string(),
        root_source: captured.root_source.source_ref.clone(),
        deps: captured.deps.iter().map(|source| source.source_ref.clone()).collect(),
        import_paths: request.import_paths.clone(),
        format: request.format.clone(),
        output_target: request.output_target.clone(),
        output_digest_blake3: crate::nickel_export_core_adapter::export_source_ref("output", output).digest_blake3,
        evaluator: request.evaluator.clone(),
        non_claim: NICKEL_EXPORT_NON_CLAIM.to_string(),
    }
}

fn canonical_admission(
    request: &NormalizedExportRequest,
    captured: &CapturedSources,
    output: &[u8],
    legacy_receipt: &NickelExportReceipt,
) -> Result<CanonicalAdmission, AdapterFailure> {
    let dependency_artifacts = explicit_dependencies(captured);
    let adapter_request = adapter_request(request);
    let observation = ExplicitObservation {
        request: adapter_request,
        source: ExplicitArtifact {
            path: &captured.root_source.source_ref.path,
            bytes: &captured.root_source.bytes,
        },
        dependencies: &dependency_artifacts,
        output: ExplicitArtifact {
            path: &request.output_target,
            bytes: output,
        },
        diagnostics: &[],
    };
    crate::nickel_export_core_adapter::admit_and_compare(&observation, legacy_receipt)
}

fn explicit_dependencies(captured: &CapturedSources) -> Vec<ExplicitArtifact<'_>> {
    captured
        .deps
        .iter()
        .map(|source| ExplicitArtifact {
            path: &source.source_ref.path,
            bytes: &source.bytes,
        })
        .collect()
}

fn adapter_request(request: &NormalizedExportRequest) -> AdapterRequest<'_> {
    AdapterRequest {
        source: &request.file,
        dependencies: &request.deps,
        import_paths: &request.import_paths,
        format: &request.format,
        output_target: &request.output_target,
        evaluator: &request.evaluator,
    }
}

fn select_authoritative_receipt(
    authority: CutoverAuthority,
    legacy: &NickelExportReceipt,
    canonical: Result<CanonicalAdmission, AdapterFailure>,
) -> Result<NickelExportReceipt, AdapterFailure> {
    match authority {
        CutoverAuthority::Legacy => Ok(legacy.clone()),
        CutoverAuthority::Canonical => canonical.map(|admission| admission.mantle_receipt),
    }
}

fn success_report(
    request: NormalizedExportRequest,
    receipt: NickelExportReceipt,
    receipt_digest: String,
    output: String,
) -> NickelExportReport {
    let output_text = if request.output_target == OUTPUT_TARGET_STDOUT {
        Some(output)
    } else {
        None
    };
    NickelExportReport {
        schema: NICKEL_EXPORT_REPORT_SCHEMA.to_string(),
        success: true,
        format: receipt.format.clone(),
        output_target: receipt.output_target.clone(),
        receipt_digest_blake3: Some(receipt_digest),
        output_digest_blake3: Some(receipt.output_digest_blake3.clone()),
        failure_class: None,
        diagnostics: Vec::new(),
        receipt: Some(receipt),
        output_text,
    }
}

fn failed_report(
    format: &str,
    output_target: String,
    failure_class: &str,
    diagnostics: Vec<NickelExportDiagnostic>,
) -> NickelExportReport {
    assert!(!failure_class.is_empty(), "failure class must not be empty");
    NickelExportReport {
        schema: NICKEL_EXPORT_REPORT_SCHEMA.to_string(),
        success: false,
        format: format.to_string(),
        output_target,
        receipt_digest_blake3: None,
        output_digest_blake3: None,
        failure_class: Some(failure_class.to_string()),
        diagnostics,
        receipt: None,
        output_text: None,
    }
}

fn render_success(report: &NickelExportReport, json: bool) -> Result<(), RunError> {
    if json {
        render_report(report, true)?;
        return Ok(());
    }
    if report.output_target == OUTPUT_TARGET_STDOUT {
        let output = report.output_text.as_deref().unwrap_or_default();
        print!("{output}");
        std::io::stdout()
            .flush()
            .map_err(|error| RunError::Internal(format!("flushing Nickel export stdout: {error}")))?;
    }
    eprintln!(
        "nickel export ok: format={} output_digest={} receipt={} ({})",
        report.format,
        report.output_digest_blake3.as_deref().unwrap_or("<missing>"),
        report.receipt_digest_blake3.as_deref().unwrap_or("<missing>"),
        NICKEL_EXPORT_NON_CLAIM
    );
    Ok(())
}

fn render_report(report: &NickelExportReport, json: bool) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::to_string_pretty(report)
            .map_err(|error| RunError::Internal(format!("rendering Nickel export JSON: {error}")))?;
        println!("{rendered}");
        return Ok(());
    }
    for diagnostic in &report.diagnostics {
        eprintln!("nickel export {} {}: {}", diagnostic.class, diagnostic.subject, diagnostic.message);
    }
    Ok(())
}

fn write_output_file(root: &Path, out: &Path, content: &[u8]) -> Result<(), RunError> {
    assert!(!out.is_absolute(), "export output must be normalized before writing");
    reject_symlink_components(root, out, true)
        .map_err(|message| RunError::Internal(format!("unsafe output path: {message}")))?;
    create_output_parents(root, out)?;
    let output_path = root.join(out);
    reject_existing_output_kind(&output_path)?;
    let mut file = open_output_no_follow(&output_path).map_err(|error| {
        RunError::Internal(format!("opening Nickel export output {}: {error}", output_path.display()))
    })?;
    file.write_all(content).map_err(|error| {
        RunError::Internal(format!("writing Nickel export output {}: {error}", output_path.display()))
    })?;
    file.flush().map_err(|error| {
        RunError::Internal(format!("flushing Nickel export output {}: {error}", output_path.display()))
    })
}

fn create_output_parents(root: &Path, out: &Path) -> Result<(), RunError> {
    let Some(parent) = out.parent() else {
        return Ok(());
    };
    let mut cursor = root.to_path_buf();
    for component in parent.components() {
        let Component::Normal(name) = component else {
            return Err(RunError::Internal(format!("output parent contains unsafe component: {}", out.display())));
        };
        cursor.push(name);
        match fs::symlink_metadata(&cursor) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(RunError::Internal(format!("output parent is not a real directory: {}", cursor.display())));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&cursor).map_err(|create_error| {
                    RunError::Internal(format!("creating output dir {}: {create_error}", cursor.display()))
                })?;
            }
            Err(error) => {
                return Err(RunError::Internal(format!("inspecting output dir {}: {error}", cursor.display())));
            }
        }
    }
    Ok(())
}

fn reject_existing_output_kind(path: &Path) -> Result<(), RunError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => Err(RunError::Internal(format!(
            "Nickel export output must be absent or a regular file: {}",
            path.display()
        ))),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(RunError::Internal(format!("inspecting output {}: {error}", path.display()))),
    }
}

fn reject_symlink_components(root: &Path, relative: &Path, allow_missing: bool) -> Result<(), String> {
    let root_metadata =
        fs::symlink_metadata(root).map_err(|error| format!("inspect export root {}: {error}", root.display()))?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(format!("export root is not a real directory: {}", root.display()));
    }
    let mut cursor = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(format!("path contains an unsafe component: {}", relative.display()));
        };
        cursor.push(name);
        match fs::symlink_metadata(&cursor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!("symlink path component is not admitted: {}", cursor.display()));
            }
            Ok(_) => {}
            Err(error) if allow_missing && error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("inspect path component {}: {error}", cursor.display())),
        }
    }
    Ok(())
}

fn open_regular_file_no_follow(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(NO_FOLLOW_OPEN_FLAGS);
    options.open(path)
}

fn open_output_no_follow(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.custom_flags(NO_FOLLOW_OPEN_FLAGS);
    options.open(path)
}

fn path_text(path: &Path, code: &str, diagnostics: &mut Vec<NickelExportDiagnostic>) -> String {
    match path.to_str() {
        Some(value) => value.to_string(),
        None => {
            diagnostics.push(diagnostic(code, "path", "Nickel export paths must be valid UTF-8".to_string()));
            String::new()
        }
    }
}

fn path_text_list(paths: &[PathBuf], code: &str, diagnostics: &mut Vec<NickelExportDiagnostic>) -> Vec<String> {
    paths.iter().map(|path| path_text(path, code, diagnostics)).collect()
}

fn output_target_text(out: Option<&Path>, diagnostics: &mut Vec<NickelExportDiagnostic>) -> String {
    out.map_or_else(|| OUTPUT_TARGET_STDOUT.to_string(), |path| path_text(path, "non-utf8-output-path", diagnostics))
}

fn output_target_label(out: Option<&Path>) -> String {
    out.map(|path| path.display().to_string()).unwrap_or_else(|| OUTPUT_TARGET_STDOUT.to_string())
}

fn diagnostic(class: &str, subject: &str, message: String) -> NickelExportDiagnostic {
    assert!(!class.is_empty(), "diagnostic class must not be empty");
    assert!(!subject.is_empty(), "diagnostic subject must not be empty");
    assert!(!message.is_empty(), "diagnostic message must not be empty");
    NickelExportDiagnostic {
        class: class.to_string(),
        subject: subject.to_string(),
        message,
    }
}

fn adapter_failure_message(failure: &AdapterFailure) -> String {
    let details = failure
        .diagnostics
        .iter()
        .map(|diagnostic| format!("{}:{}:{}", diagnostic.class, diagnostic.subject, diagnostic.message))
        .collect::<Vec<_>>()
        .join("; ");
    format!("{}: {details}", failure.class)
}

pub fn default_evaluator_id() -> String {
    DEFAULT_EVALUATOR_ID.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_EVALUATOR_VERSION: &str = "mantle-evaluator-fixture-v1";
    const TEST_OUTPUT: &[u8] = b"{\"answer\":42}";

    fn options<'a>(
        root: &'a Path,
        file: &'a Path,
        deps: &'a [PathBuf],
        out: Option<&'a Path>,
    ) -> NickelExportOptions<'a> {
        NickelExportOptions {
            root,
            file,
            deps,
            import_paths: &[],
            format: FORMAT_JSON,
            out,
            evaluator_id: DEFAULT_EVALUATOR_ID,
            evaluator_version: TEST_EVALUATOR_VERSION,
            json: true,
        }
    }

    #[test]
    fn export_request_rejects_absolute_escape_and_unsupported_format() {
        let root = Path::new("/workspace");
        let dependencies = [PathBuf::from("../dep.ncl")];
        let import_paths = [PathBuf::from("../lib")];
        let request = NickelExportOptions {
            root,
            file: Path::new("/abs/main.ncl"),
            deps: &dependencies,
            import_paths: &import_paths,
            format: "toml",
            out: Some(Path::new("../out.json")),
            evaluator_id: DEFAULT_EVALUATOR_ID,
            evaluator_version: TEST_EVALUATOR_VERSION,
            json: true,
        };

        let errors = normalize_export_options(&request).unwrap_err();
        let classes = errors.iter().map(|diagnostic| diagnostic.class.as_str()).collect::<Vec<_>>();
        assert!(classes.contains(&"unsupported-format"));
        assert!(classes.contains(&"unsafe-source-path"));
        assert!(classes.contains(&"unsafe-dependency-path"));
        assert!(classes.contains(&"unsafe-import-path"));
        assert!(classes.contains(&"unsafe-output-path"));
    }

    #[test]
    fn canonical_and_legacy_authority_selectors_are_explicit() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("main.ncl"), "{ answer = 42 }").unwrap();
        let normalized = normalize_export_options(&options(temp.path(), Path::new("main.ncl"), &[], None)).unwrap();
        let captured = capture_sources(temp.path(), &normalized).unwrap();
        let legacy = build_legacy_export_receipt(&normalized, &captured, TEST_OUTPUT);
        let canonical = canonical_admission(&normalized, &captured, TEST_OUTPUT, &legacy);
        let legacy_selected =
            select_authoritative_receipt(CutoverAuthority::Legacy, &legacy, canonical.clone()).unwrap();
        let canonical_selected = select_authoritative_receipt(CutoverAuthority::Canonical, &legacy, canonical).unwrap();
        assert_eq!(legacy_selected, legacy);
        assert_eq!(canonical_selected, legacy);
    }

    #[cfg(unix)]
    #[test]
    fn source_and_output_symlinks_are_rejected_without_following() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("real.ncl"), "{ answer = 42 }").unwrap();
        symlink("real.ncl", temp.path().join("linked.ncl")).unwrap();
        let normalized = normalize_export_options(&options(temp.path(), Path::new("linked.ncl"), &[], None)).unwrap();
        let source_errors = capture_sources(temp.path(), &normalized).unwrap_err();
        assert_eq!(source_errors[0].class, "symlink-source");

        fs::create_dir(temp.path().join("outside")).unwrap();
        symlink("outside", temp.path().join("generated")).unwrap();
        let output_error = write_output_file(temp.path(), Path::new("generated/config.json"), TEST_OUTPUT).unwrap_err();
        assert!(output_error.to_string().contains("symlink"));
        assert!(!temp.path().join("outside/config.json").exists());
    }

    #[test]
    fn shell_keeps_evaluator_filesystem_destination_and_policy_authority() {
        let shell = include_str!("nickel_export.rs");
        let adapter = include_str!("nickel_export_core_adapter.rs");
        assert!(shell.contains("crunch_eval::evaluate_to_json"));
        assert!(shell.contains("capture_source"));
        assert!(shell.contains("write_output_file"));
        assert!(shell.contains("select_authoritative_receipt"));
        assert!(!adapter.contains(&["crunch", "_eval::evaluate_to_json"].concat()));
        assert!(!adapter.contains(&["std", "::fs::"].concat()));
    }

    #[test]
    fn output_write_replaces_regular_stale_bytes_but_not_receipt_identity() {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("generated/config.json");
        fs::create_dir(output.parent().unwrap()).unwrap();
        fs::write(&output, b"stale").unwrap();
        write_output_file(temp.path(), Path::new("generated/config.json"), TEST_OUTPUT).unwrap();
        assert_eq!(fs::read(&output).unwrap(), TEST_OUTPUT);
        assert!(fs::symlink_metadata(&output).unwrap().file_type().is_file());
    }

    #[test]
    fn source_tamper_after_capture_is_rejected_before_receipt() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("main.ncl");
        fs::write(&source, "{ answer = 42 }").unwrap();
        let normalized = normalize_export_options(&options(temp.path(), Path::new("main.ncl"), &[], None)).unwrap();
        let captured = capture_sources(temp.path(), &normalized).unwrap();
        fs::write(&source, "{ answer = 43 }").unwrap();
        let diagnostics = verify_captured_sources_unchanged(temp.path(), &captured).unwrap_err();
        assert_eq!(diagnostics[0].class, "source-changed");
        assert_eq!(diagnostics[0].subject, "main.ncl");
    }
}
