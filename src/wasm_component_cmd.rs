use std::ffi::OsString;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use crunch_wasm_component::ComponentPipelineRequest;
use crunch_wasm_component::PipelinePaths;
use crunch_wasm_component::run_component_pipeline;
use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::CommandFamily;
use mantle_application_contract::EffectId;
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

use crate::errors::RunError;

const MAX_COMPONENT_REPORT_BYTES: u64 = 64 * 1024 * 1024;

pub(crate) struct WasmComponentBuildOptions<'a> {
    pub request_path: &'a Path,
    pub import_paths: &'a [PathBuf],
    pub output_dir: &'a Path,
    pub scratch_parent: Option<&'a Path>,
    pub json: bool,
}

pub(crate) fn cmd_wasm_component_build(options: WasmComponentBuildOptions<'_>) -> Result<(), RunError> {
    let json = options.json;
    let plan = planned_component_build()?;
    let mut observations = Vec::with_capacity(plan.effects.len());
    let port = LocalComponentPort;
    let prepared = component_step(
        &plan,
        &mut observations,
        "prepare-component",
        EffectKind::ReadFiles,
        EffectMeasure::Calls(1),
        port.prepare(options),
    )?;
    let output_dir = prepared.output_dir.clone();
    let execution_result = component_step(
        &plan,
        &mut observations,
        "run-component-pipeline",
        EffectKind::RunProcess,
        EffectMeasure::Calls(1),
        port.execute(prepared),
    )?;
    let (readback, bytes_read) = port.observe_report(&execution_result, &output_dir);
    let publication_status = if readback.is_ok() && execution_result.final_status == "succeeded" {
        ObservationStatus::Succeeded
    } else {
        ObservationStatus::Failed
    };
    observations.push(component_observation(
        "publish-component-evidence",
        EffectKind::WriteFiles,
        publication_status,
        EffectMeasure::Calls(1),
    ));
    let published = component_step(
        &plan,
        &mut observations,
        "read-component-report",
        EffectKind::ReadFiles,
        EffectMeasure::Bytes(bytes_read),
        readback,
    )?;
    if execution_result.final_status != "succeeded" {
        observations[1].status = ObservationStatus::Failed;
        observations[1].output = EffectOutput::None;
        observations[1].diagnostics_code = Some("component-pipeline-blocked".to_string());
    }
    finish_component_report(&plan, &observations, &published, &output_dir, json)
}

fn planned_component_build() -> Result<EffectPlan, RunError> {
    // Preparation reads cwd, Nickel inputs and stdlib; the pipeline runs tools
    // and publishes evidence; bounded report readback verifies that publication.
    plan_effects(CommandFamily::Component, &[
        component_effect("prepare-component", EffectKind::ReadFiles),
        component_effect("run-component-pipeline", EffectKind::RunProcess),
        component_effect("publish-component-evidence", EffectKind::WriteFiles),
        EffectSpec {
            effect_id: "read-component-report",
            kind: EffectKind::ReadFiles,
            limit: EffectMeasure::Bytes(MAX_COMPONENT_REPORT_BYTES),
            expected_output: ExpectedOutput::None,
        },
    ])
    .map_err(|error| RunError::Internal(format!("component effect plan: {}", error.code())))
}

fn finish_component_report(
    plan: &EffectPlan,
    observations: &[Observation],
    published: &crunch_wasm_component::PipelineExecutionReport,
    output_dir: &Path,
    json: bool,
) -> Result<(), RunError> {
    match classify_observations(plan, observations) {
        ApplicationOutcome::Completed => {
            render_report(published, output_dir, json)?;
            Ok(())
        }
        ApplicationOutcome::Failed { .. } if published.final_status != "succeeded" => {
            render_report(published, output_dir, json)?;
            Err(RunError::Internal(format!(
                "Wasm component pipeline stopped at an authoritative blocker; report: {}",
                output_dir.join("execution-report.json").display()
            )))
        }
        outcome => Err(RunError::Internal(format!("component observations were inconsistent: {outcome:?}"))),
    }
}

fn component_effect(effect_id: &'static str, kind: EffectKind) -> EffectSpec<'static> {
    EffectSpec {
        effect_id,
        kind,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    }
}

fn component_observation(
    effect_id: &str,
    kind: EffectKind,
    status: ObservationStatus,
    usage: EffectMeasure,
) -> Observation {
    Observation {
        effect_id: EffectId(effect_id.to_string()),
        kind,
        status,
        output: EffectOutput::None,
        usage,
        diagnostics_code: (status == ObservationStatus::Failed).then(|| "component-port-failed".to_string()),
    }
}

fn component_step<T>(
    plan: &EffectPlan,
    observations: &mut Vec<Observation>,
    effect_id: &str,
    kind: EffectKind,
    usage: EffectMeasure,
    result: Result<T, RunError>,
) -> Result<T, RunError> {
    match result {
        Ok(value) => {
            observations.push(component_observation(effect_id, kind, ObservationStatus::Succeeded, usage));
            Ok(value)
        }
        Err(error) => {
            observations.push(component_observation(effect_id, kind, ObservationStatus::Failed, usage));
            for remaining in plan.effects.iter().skip(observations.len()) {
                let usage = match remaining.limit {
                    EffectMeasure::Calls(_) => EffectMeasure::Calls(0),
                    EffectMeasure::Items(_) => EffectMeasure::Items(0),
                    EffectMeasure::Bytes(_) => EffectMeasure::Bytes(0),
                };
                observations.push(Observation {
                    effect_id: remaining.effect_id.clone(),
                    kind: remaining.kind,
                    status: ObservationStatus::Skipped,
                    output: EffectOutput::None,
                    usage,
                    diagnostics_code: None,
                });
            }
            match classify_observations(plan, observations) {
                ApplicationOutcome::Failed { .. } => Err(error),
                outcome => Err(RunError::Internal(format!("component observations were inconsistent: {outcome:?}"))),
            }
        }
    }
}

struct PreparedComponent {
    request: ComponentPipelineRequest,
    output_dir: PathBuf,
    scratch_parent: PathBuf,
}

trait ComponentExecutionPort {
    fn prepare(&self, options: WasmComponentBuildOptions<'_>) -> Result<PreparedComponent, RunError>;
    fn execute(&self, prepared: PreparedComponent) -> Result<crunch_wasm_component::PipelineExecutionReport, RunError>;
    fn observe_report(
        &self,
        execution: &crunch_wasm_component::PipelineExecutionReport,
        output_dir: &Path,
    ) -> (Result<crunch_wasm_component::PipelineExecutionReport, RunError>, u64);
}

struct LocalComponentPort;

impl ComponentExecutionPort for LocalComponentPort {
    fn prepare(&self, options: WasmComponentBuildOptions<'_>) -> Result<PreparedComponent, RunError> {
        let request_path = absolute_path(options.request_path)?;
        let output_dir = absolute_path(options.output_dir)?;
        let output_parent = output_dir.parent().ok_or_else(|| {
            RunError::Internal(format!("Wasm component output has no parent: {}", output_dir.display()))
        })?;
        let scratch_parent = match options.scratch_parent {
            Some(path) => absolute_path(path)?,
            None => output_parent.to_path_buf(),
        };
        let evaluation_search_roots = evaluation_import_paths(&request_path, options.import_paths)?;
        let request =
            crunch_eval::evaluate_and_deserialize(&request_path, &evaluation_search_roots).map_err(|error| {
                RunError::Eval(format!("loading Wasm component request {}: {error}", request_path.display()))
            })?;
        Ok(PreparedComponent {
            request,
            output_dir,
            scratch_parent,
        })
    }

    fn execute(&self, prepared: PreparedComponent) -> Result<crunch_wasm_component::PipelineExecutionReport, RunError> {
        run_component_pipeline(prepared.request, PipelinePaths {
            evidence_dir: prepared.output_dir,
            scratch_parent: prepared.scratch_parent,
        })
        .map_err(|error| RunError::Internal(format!("Wasm component pipeline: {error}")))
    }

    fn observe_report(
        &self,
        execution: &crunch_wasm_component::PipelineExecutionReport,
        output_dir: &Path,
    ) -> (Result<crunch_wasm_component::PipelineExecutionReport, RunError>, u64) {
        let path = output_dir.join("execution-report.json");
        let file = match std::fs::File::open(&path) {
            Ok(file) => file,
            Err(error) => {
                return (
                    Err(RunError::Internal(format!("reading Wasm component report {}: {error}", path.display()))),
                    0,
                );
            }
        };
        let mut bytes = Vec::new();
        let read_result = file.take(MAX_COMPONENT_REPORT_BYTES + 1).read_to_end(&mut bytes);
        let bytes_read = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if let Err(error) = read_result {
            return (
                Err(RunError::Internal(format!("reading Wasm component report {}: {error}", path.display()))),
                bytes_read,
            );
        }
        if bytes_read > MAX_COMPONENT_REPORT_BYTES {
            return (
                Err(RunError::Internal(format!("Wasm component report exceeds {} bytes", MAX_COMPONENT_REPORT_BYTES))),
                bytes_read,
            );
        }
        let result = serde_json::from_slice::<crunch_wasm_component::PipelineExecutionReport>(&bytes)
            .map_err(|error| RunError::Internal(format!("parsing Wasm component report {}: {error}", path.display())))
            .and_then(|published| {
                if &published != execution {
                    return Err(RunError::Internal(format!(
                        "Wasm component published report differs from execution at {}",
                        path.display()
                    )));
                }
                if published.final_status == "succeeded" {
                    crunch_wasm_component::verify_pipeline_execution_report_files(&published).map_err(|error| {
                        RunError::Internal(format!("verifying Wasm component publication: {error}"))
                    })?;
                }
                Ok(published)
            });
        (result, bytes_read)
    }
}

fn evaluation_import_paths(request_path: &Path, explicit: &[PathBuf]) -> Result<Vec<OsString>, RunError> {
    debug_assert!(request_path.is_absolute());
    debug_assert!(explicit.len().checked_add(2).is_some());
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
    execution_result: &crunch_wasm_component::PipelineExecutionReport,
    output_dir: &Path,
    json: bool,
) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::to_string(execution_result)
            .map_err(|error| RunError::Internal(format!("serializing Wasm component report: {error}")))?;
        println!("{rendered}");
    } else {
        println!("Wasm component pipeline status: {}", execution_result.final_status);
        println!("report: {}", output_dir.join("execution-report.json").display());
        for blocker in &execution_result.blockers {
            println!("blocked: {} [{}]: {}", blocker.stage_key, blocker.code, blocker.message);
        }
    }
    debug_assert_eq!(execution_result.final_status == "succeeded", execution_result.blockers.is_empty());
    debug_assert!(output_dir.is_absolute());
    Ok(())
}

fn absolute_path(path: &Path) -> Result<PathBuf, RunError> {
    if path.is_absolute() {
        debug_assert!(path.has_root());
        debug_assert!(!path.as_os_str().is_empty());
        return Ok(path.to_path_buf());
    }
    let current =
        std::env::current_dir().map_err(|error| RunError::Internal(format!("resolving current directory: {error}")))?;
    let absolute = current.join(path);
    debug_assert!(absolute.is_absolute());
    debug_assert!(absolute.starts_with(&current));
    Ok(absolute)
}
