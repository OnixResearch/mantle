use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use crunch_wasm_component::ComponentPipelineRequest;
use crunch_wasm_component::PipelinePaths;
use crunch_wasm_component::run_component_pipeline;

use crate::errors::RunError;

pub(crate) struct WasmComponentBuildOptions<'a> {
    pub request_path: &'a Path,
    pub import_paths: &'a [PathBuf],
    pub output_dir: &'a Path,
    pub scratch_parent: Option<&'a Path>,
    pub json: bool,
}

pub(crate) fn cmd_wasm_component_build(options: WasmComponentBuildOptions<'_>) -> Result<(), RunError> {
    let request_path = absolute_path(options.request_path)?;
    let output_dir = absolute_path(options.output_dir)?;
    let output_parent = output_dir
        .parent()
        .ok_or_else(|| RunError::Internal(format!("Wasm component output has no parent: {}", output_dir.display())))?;
    let scratch_parent = match options.scratch_parent {
        Some(path) => absolute_path(path)?,
        None => output_parent.to_path_buf(),
    };
    let import_paths = evaluation_import_paths(&request_path, options.import_paths)?;
    let request: ComponentPipelineRequest = crunch_eval::evaluate_and_deserialize(&request_path, &import_paths)
        .map_err(|error| {
            RunError::Eval(format!("loading Wasm component request {}: {error}", request_path.display()))
        })?;
    let report = run_component_pipeline(request, PipelinePaths {
        evidence_dir: output_dir.clone(),
        scratch_parent,
    })
    .map_err(|error| RunError::Internal(format!("Wasm component pipeline: {error}")))?;
    render_report(&report, &output_dir, options.json)?;
    if report.final_status != "succeeded" {
        return Err(RunError::Internal(format!(
            "Wasm component pipeline stopped at an authoritative blocker; report: {}",
            output_dir.join("execution-report.json").display()
        )));
    }
    Ok(())
}

fn evaluation_import_paths(request_path: &Path, explicit: &[PathBuf]) -> Result<Vec<OsString>, RunError> {
    let mut paths = Vec::with_capacity(explicit.len().saturating_add(2));
    if let Some(parent) = request_path.parent() {
        paths.push(parent.as_os_str().to_owned());
    }
    let stdlib = crunch_eval::stdlib::stdlib_import_path()
        .map_err(|error| RunError::Internal(format!("materializing embedded Nickel stdlib: {error}")))?;
    paths.push(stdlib.into_os_string());
    for path in explicit {
        paths.push(absolute_path(path)?.into_os_string());
    }
    debug_assert!(!paths.is_empty());
    debug_assert!(paths.len() <= explicit.len().saturating_add(2));
    Ok(paths)
}

fn render_report(
    report: &crunch_wasm_component::PipelineExecutionReport,
    output_dir: &Path,
    json: bool,
) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::to_string(report)
            .map_err(|error| RunError::Internal(format!("serializing Wasm component report: {error}")))?;
        println!("{rendered}");
    } else {
        println!("Wasm component pipeline status: {}", report.final_status);
        println!("report: {}", output_dir.join("execution-report.json").display());
        for blocker in &report.blockers {
            println!("blocked: {} [{}]: {}", blocker.stage_key, blocker.code, blocker.message);
        }
    }
    debug_assert_eq!(report.final_status == "succeeded", report.blockers.is_empty());
    debug_assert!(output_dir.is_absolute());
    Ok(())
}

fn absolute_path(path: &Path) -> Result<PathBuf, RunError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let current =
        std::env::current_dir().map_err(|error| RunError::Internal(format!("resolving current directory: {error}")))?;
    Ok(current.join(path))
}
