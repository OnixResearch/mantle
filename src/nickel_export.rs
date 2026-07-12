// machine-artifact-public: nickel-export.report
// machine-artifact-public: nickel-export.receipt
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;

pub const NICKEL_EXPORT_REPORT_SCHEMA: &str = "mantle-nickel-export-report-v1";
pub const NICKEL_EXPORT_RECEIPT_SCHEMA: &str = "mantle-nickel-export-receipt-v1";
pub const NICKEL_EXPORT_NON_CLAIM: &str = "Nickel export success proves only the declared evaluation output digest under the recorded evaluator descriptor; it does not prove deployability, frontend correctness, or build success";

const FORMAT_JSON: &str = "json";
const OUTPUT_TARGET_STDOUT: &str = "stdout";
const DEFAULT_EVALUATOR_ID: &str = "mantle-embedded-crunch-eval";
const MAX_EXPORT_PATHS: usize = 4096;
const EXPORT_FAILURE_EXIT_CODE: u8 = 3;
const BLAKE3_HEX_BYTES: usize = 64;

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

pub fn cmd_nickel_export(options: NickelExportOptions<'_>) -> Result<(), RunError> {
    let normalized = match normalize_export_options(&options) {
        Ok(normalized) => normalized,
        Err(diagnostics) => {
            let report = failed_report(options.format, output_target_label(options.out), "validation", diagnostics);
            render_report(&report, options.json)?;
            return Err(RunError::Reported(EXPORT_FAILURE_EXIT_CODE));
        }
    };

    let refs = match read_source_refs(options.root, &normalized) {
        Ok(refs) => refs,
        Err(diagnostics) => {
            let report = failed_report(&normalized.format, normalized.output_target.clone(), "source", diagnostics);
            render_report(&report, options.json)?;
            return Err(RunError::Reported(EXPORT_FAILURE_EXIT_CODE));
        }
    };

    let output = match evaluate_export_output(options.root, &normalized) {
        Ok(output) => output,
        Err(message) => {
            let report = failed_report(&normalized.format, normalized.output_target.clone(), "eval", vec![diagnostic(
                "eval",
                &normalized.file,
                message,
            )]);
            render_report(&report, options.json)?;
            return Err(RunError::Reported(EXPORT_FAILURE_EXIT_CODE));
        }
    };

    let receipt = build_export_receipt(normalized.clone(), refs.root_source, refs.deps, &output);
    let receipt_digest = receipt_digest_blake3(&receipt)?;
    if normalized.output_target != OUTPUT_TARGET_STDOUT {
        write_output_file(options.root, Path::new(&normalized.output_target), output.as_bytes())?;
    }
    let report = success_report(normalized, receipt, receipt_digest, output);
    render_success(&report, options.json)
}

fn normalize_export_options(
    options: &NickelExportOptions<'_>,
) -> Result<NormalizedExportRequest, Vec<NickelExportDiagnostic>> {
    assert!(!options.evaluator_id.is_empty(), "evaluator id must not be empty");
    assert!(!options.evaluator_version.is_empty(), "evaluator version must not be empty");
    assert!(options.deps.len() <= MAX_EXPORT_PATHS, "export dep count exceeds limit");
    assert!(options.import_paths.len() <= MAX_EXPORT_PATHS, "export import path count exceeds limit");

    let mut diagnostics = Vec::new();
    if options.format != FORMAT_JSON {
        diagnostics.push(diagnostic(
            "unsupported-format",
            options.format,
            format!("Nickel export format `{}` is not supported yet; supported: {FORMAT_JSON}", options.format),
        ));
    }
    let file = normalize_relative_path(options.file).unwrap_or_else(|message| {
        diagnostics.push(diagnostic("unsafe-source-path", &options.file.display().to_string(), message));
        String::new()
    });
    let deps = normalize_path_list(options.deps, "unsafe-dependency-path", &mut diagnostics);
    let import_paths = normalize_path_list(options.import_paths, "unsafe-import-path", &mut diagnostics);
    let output_target = normalize_output_target(options.out, &mut diagnostics);
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
    Ok(NormalizedExportRequest {
        file,
        deps,
        import_paths,
        format: options.format.to_string(),
        output_target,
        evaluator,
    })
}

struct SourceRefs {
    root_source: ExportSourceRef,
    deps: Vec<ExportSourceRef>,
}

fn read_source_refs(root: &Path, request: &NormalizedExportRequest) -> Result<SourceRefs, Vec<NickelExportDiagnostic>> {
    let mut diagnostics = Vec::new();
    let root_source = match source_ref(root, &request.file) {
        Ok(source_ref) => source_ref,
        Err(diagnostic) => {
            diagnostics.push(diagnostic);
            ExportSourceRef {
                path: request.file.clone(),
                digest_blake3: String::new(),
            }
        }
    };
    let mut deps = Vec::with_capacity(request.deps.len());
    for dep in &request.deps {
        match source_ref(root, dep) {
            Ok(source_ref) => deps.push(source_ref),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    if diagnostics.is_empty() {
        Ok(SourceRefs { root_source, deps })
    } else {
        Err(diagnostics)
    }
}

fn source_ref(root: &Path, relative_path: &str) -> Result<ExportSourceRef, NickelExportDiagnostic> {
    let path = root.join(relative_path);
    let bytes = fs::read(&path).map_err(|err| {
        diagnostic(
            "missing-source",
            relative_path,
            format!("declared Nickel export source `{relative_path}` could not be read: {err}"),
        )
    })?;
    Ok(ExportSourceRef {
        path: relative_path.to_string(),
        digest_blake3: blake3_hex(&bytes),
    })
}

fn evaluate_export_output(root: &Path, request: &NormalizedExportRequest) -> Result<String, String> {
    assert_eq!(request.format, FORMAT_JSON, "caller validates export format");
    let file = root.join(&request.file);
    let import_paths =
        request.import_paths.iter().map(|path| root.join(path).into_os_string()).collect::<Vec<OsString>>();
    crunch_eval::evaluate_to_json(&file, &import_paths).map_err(|err| err.to_string())
}

fn build_export_receipt(
    request: NormalizedExportRequest,
    root_source: ExportSourceRef,
    deps: Vec<ExportSourceRef>,
    output: &str,
) -> NickelExportReceipt {
    assert!(!output.is_empty(), "export output must not be empty");
    NickelExportReceipt {
        schema: NICKEL_EXPORT_RECEIPT_SCHEMA.to_string(),
        root_source,
        deps,
        import_paths: request.import_paths,
        format: request.format,
        output_target: request.output_target,
        output_digest_blake3: blake3_hex(output.as_bytes()),
        evaluator: request.evaluator,
        non_claim: NICKEL_EXPORT_NON_CLAIM.to_string(),
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
            .map_err(|err| RunError::Internal(format!("flushing Nickel export stdout: {err}")))?;
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
            .map_err(|err| RunError::Internal(format!("rendering Nickel export JSON: {err}")))?;
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
    let output_path = root.join(out);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            RunError::Internal(format!("creating Nickel export output dir {}: {err}", parent.display()))
        })?;
    }
    fs::write(&output_path, content)
        .map_err(|err| RunError::Internal(format!("writing Nickel export output {}: {err}", output_path.display())))
}

fn receipt_digest_blake3(receipt: &NickelExportReceipt) -> Result<String, RunError> {
    let bytes = serde_json::to_vec(receipt)
        .map_err(|err| RunError::Internal(format!("serializing Nickel export receipt: {err}")))?;
    Ok(blake3_hex(&bytes))
}

fn normalize_output_target(out: Option<&Path>, diagnostics: &mut Vec<NickelExportDiagnostic>) -> String {
    let Some(out) = out else {
        return OUTPUT_TARGET_STDOUT.to_string();
    };
    match normalize_relative_path(out) {
        Ok(path) => path,
        Err(message) => {
            diagnostics.push(diagnostic("unsafe-output-path", &out.display().to_string(), message));
            String::new()
        }
    }
}

fn normalize_path_list(
    paths: &[PathBuf],
    code: &'static str,
    diagnostics: &mut Vec<NickelExportDiagnostic>,
) -> Vec<String> {
    paths
        .iter()
        .filter_map(|path| match normalize_relative_path(path) {
            Ok(path) => Some(path),
            Err(message) => {
                diagnostics.push(diagnostic(code, &path.display().to_string(), message));
                None
            }
        })
        .collect()
}

fn normalize_relative_path(path: &Path) -> Result<String, String> {
    if path.as_os_str().is_empty() {
        return Err("path must not be empty".to_string());
    }
    if path.is_absolute() {
        return Err(format!("path `{}` must be relative to the export root", path.display()));
    }
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(format!("path `{}` escapes the export root", path.display()));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("path `{}` must be relative to the export root", path.display()));
            }
        }
    }
    if parts.is_empty() {
        return Err(format!("path `{}` does not name a source", path.display()));
    }
    Ok(parts.join("/"))
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

fn blake3_hex(bytes: &[u8]) -> String {
    let hex = blake3::hash(bytes).to_hex().to_string();
    assert_eq!(hex.len(), BLAKE3_HEX_BYTES, "BLAKE3 hex digest length must stay fixed");
    hex
}

pub fn default_evaluator_id() -> String {
    DEFAULT_EVALUATOR_ID.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_request_rejects_absolute_escape_and_unsupported_format() {
        let root = Path::new("/workspace");
        let options = NickelExportOptions {
            root,
            file: Path::new("/abs/main.ncl"),
            deps: &[PathBuf::from("../dep.ncl")],
            import_paths: &[PathBuf::from("../lib")],
            format: "toml",
            out: Some(Path::new("../out.json")),
            evaluator_id: DEFAULT_EVALUATOR_ID,
            evaluator_version: "test",
            json: true,
        };

        let err = normalize_export_options(&options).unwrap_err();
        let classes = err.iter().map(|diag| diag.class.as_str()).collect::<Vec<_>>();

        assert!(classes.contains(&"unsupported-format"));
        assert!(classes.contains(&"unsafe-source-path"));
        assert!(classes.contains(&"unsafe-dependency-path"));
        assert!(classes.contains(&"unsafe-import-path"));
        assert!(classes.contains(&"unsafe-output-path"));
    }

    #[test]
    fn export_receipt_identity_changes_with_evaluator_descriptor() {
        let request_a = NormalizedExportRequest {
            file: "main.ncl".to_string(),
            deps: Vec::new(),
            import_paths: Vec::new(),
            format: FORMAT_JSON.to_string(),
            output_target: OUTPUT_TARGET_STDOUT.to_string(),
            evaluator: NickelEvaluatorDescriptor {
                identity: "eval-a".to_string(),
                version: "1".to_string(),
                options: Vec::new(),
            },
        };
        let mut request_b = request_a.clone();
        request_b.evaluator.identity = "eval-b".to_string();
        let source = ExportSourceRef {
            path: "main.ncl".to_string(),
            digest_blake3: blake3_hex(b"source"),
        };

        let receipt_a = build_export_receipt(request_a, source.clone(), Vec::new(), "{\"x\":1}");
        let receipt_b = build_export_receipt(request_b, source, Vec::new(), "{\"x\":1}");

        assert_ne!(receipt_digest_blake3(&receipt_a).unwrap(), receipt_digest_blake3(&receipt_b).unwrap());
        assert_eq!(receipt_a.non_claim, NICKEL_EXPORT_NON_CLAIM);
    }

    #[test]
    fn normalize_relative_path_removes_current_dir_without_escaping() {
        assert_eq!(normalize_relative_path(Path::new("./src/main.ncl")).unwrap(), "src/main.ncl");
        assert!(normalize_relative_path(Path::new("src/../secret.ncl")).is_err());
    }
}
