use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use tokio::runtime::Builder as RuntimeBuilder;

use clap::ValueEnum;
use crunch_system::ValidatedModule;
use crunch_system::loader::validate_module;
use crunch_system::threading::EvalThread;
use crunch_system::threading::EvalThreadHandle;
use crunch_system::threading::ValueId;
use crunch_system::assembler::AssemblerError;
use crunch_system::assembler::AssemblerRegistry;
use crunch_system::assembler::NixosPhase1Assembler;
use crunch_system::assembler::dry_run_assemble;
use crunch_system::collector::group_by_machine_with_filter;
use crunch_system::collector::merge_fragments;
use crunch_system::error::SystemConfigError;
use crunch_system::evaluator::EvalBoundary;
use crunch_system::evaluator::ModuleExecutionResult;
use crunch_system::evaluator::evaluate_modules_for_machines;
use crunch_system::inventory::Inventory;
use crunch_system::inventory_validate::validate_inventory;
use serde::Serialize;
use serde_json::Value;

use crate::RunContext;
use crate::build_cmd;
use crate::errors::RunError;

const DEFAULT_MODULES_DIR_NAME: &str = "modules";
const NICKEL_FORMAT_NOT_IMPLEMENTED: &str = "system eval --format nickel is not implemented yet";
const MODULE_TIMEOUT_SECS: u64 = 60;
const DIAGNOSTIC_SEVERITY_WARNING: &str = "warning";
const DIAGNOSTIC_SEVERITY_ERROR: &str = "error";
const DIAGNOSTIC_LAYER_CLI: &str = "cli";
const DIAGNOSTIC_LAYER_LOADER: &str = "loader";
const DIAGNOSTIC_LAYER_INVENTORY: &str = "inventory";
const DIAGNOSTIC_LAYER_CROSSREF: &str = "crossref";
const DIAGNOSTIC_LAYER_EVAL: &str = "eval";
const DIAGNOSTIC_LAYER_FRAGMENT: &str = "fragment";
const DIAGNOSTIC_LAYER_ASSEMBLER: &str = "assembler";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SystemStopAfter {
    Fragments,
    Derivations,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SystemEvalFormat {
    Json,
    Nickel,
}

#[derive(Debug, Clone)]
pub struct SystemEvalOptions {
    pub inventory_path: PathBuf,
    pub modules_dir: Option<PathBuf>,
    pub machine_filter: Vec<String>,
    pub assembler_override: Option<String>,
    pub stop_after: SystemStopAfter,
    pub format: SystemEvalFormat,
}

#[derive(Debug, Clone)]
pub struct SystemBuildOptions {
    pub inventory_path: PathBuf,
    pub modules_dir: Option<PathBuf>,
    pub machine_filter: Vec<String>,
    pub assembler_override: Option<String>,
}

#[derive(Debug, Serialize)]
struct SystemDiagnosticEnvelope {
    severity: &'static str,
    layer: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    machine: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field_path: Option<String>,
}

#[derive(Debug)]
struct PreparedSystemRun {
    inventory: Inventory,
    selected_machines: BTreeSet<String>,
    module_execution: ModuleExecutionResult,
    merged_configs: BTreeMap<String, crunch_system::MergedConfig>,
}

#[derive(Debug, Serialize)]
struct JsonMergedConfig {
    machine_name: String,
    data: Value,
    provenance: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct JsonMachineOutcome {
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    merged_config: Option<JsonMergedConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    derivations: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reports: Option<Vec<Value>>,
}

#[derive(Debug, Serialize)]
struct JsonSystemPipelineResult {
    machines: BTreeMap<String, JsonMachineOutcome>,
    errors: Vec<SystemDiagnosticEnvelope>,
    warnings: Vec<SystemDiagnosticEnvelope>,
}

#[derive(Debug, Serialize)]
struct HumanSystemBuildSummary {
    succeeded_machines: Vec<String>,
    failed_machines: Vec<String>,
    warning_count: u32,
    error_count: u32,
}

pub fn cmd_system_eval(ctx: &RunContext, options: SystemEvalOptions) -> Result<(), RunError> {
    if options.format == SystemEvalFormat::Nickel {
        emit_fatal_cli_error(ctx, cli_error(NICKEL_FORMAT_NOT_IMPLEMENTED.to_string(), None))?;
        return Err(RunError::Reported(3));
    }

    let prepared = prepare_system_run(&options.inventory_path, options.modules_dir.as_deref(), &options.machine_filter)?;
    let mut result = JsonSystemPipelineResult {
        machines: BTreeMap::new(),
        errors: prepared
            .module_execution
            .errors
            .iter()
            .map(error_diagnostic)
            .collect(),
        warnings: prepared
            .module_execution
            .warnings
            .iter()
            .map(warning_diagnostic)
            .collect(),
    };

    match options.stop_after {
        SystemStopAfter::Fragments => {
            for (machine_name, merged_config) in prepared.merged_configs {
                result.machines.insert(
                    machine_name,
                    JsonMachineOutcome {
                        kind: "fragments",
                        merged_config: Some(json_merged_config(merged_config)),
                        derivations: None,
                        reports: None,
                    },
                );
            }
        }
        SystemStopAfter::Derivations => {
            let registry = default_assembler_registry();
            populate_derivation_outcomes(
                &prepared.inventory,
                &prepared.selected_machines,
                &prepared.merged_configs,
                options.assembler_override.as_deref(),
                &registry,
                &mut result,
            )?;
        }
    }

    emit_nonfatal_diagnostics(
        ctx,
        &collect_renderable_errors(&prepared.module_execution.errors, &result.errors),
        &prepared.module_execution.warnings,
    )?;
    println!("{}", serde_json::to_string_pretty(&result).map_err(json_internal_error)?);
    if result.errors.is_empty() {
        Ok(())
    } else {
        Err(RunError::Reported(1))
    }
}

pub fn cmd_system_build(ctx: &RunContext, options: SystemBuildOptions) -> Result<(), RunError> {
    let prepared = prepare_system_run(&options.inventory_path, options.modules_dir.as_deref(), &options.machine_filter)?;
    let registry = default_assembler_registry();
    let mut result = JsonSystemPipelineResult {
        machines: BTreeMap::new(),
        errors: prepared
            .module_execution
            .errors
            .iter()
            .map(error_diagnostic)
            .collect(),
        warnings: prepared
            .module_execution
            .warnings
            .iter()
            .map(warning_diagnostic)
            .collect(),
    };

    populate_build_outcomes(
        ctx,
        &prepared.inventory,
        &prepared.selected_machines,
        &prepared.merged_configs,
        options.assembler_override.as_deref(),
        &registry,
        &mut result,
    )?;

    emit_nonfatal_diagnostics(
        ctx,
        &collect_renderable_errors(&prepared.module_execution.errors, &result.errors),
        &prepared.module_execution.warnings,
    )?;
    if ctx.json {
        println!("{}", serde_json::to_string_pretty(&result).map_err(json_internal_error)?);
    } else {
        let summary = build_human_summary(&result);
        println!("{}", serde_json::to_string_pretty(&summary).map_err(json_internal_error)?);
    }

    if result.errors.is_empty() {
        Ok(())
    } else {
        Err(RunError::Reported(1))
    }
}

fn prepare_system_run(
    inventory_path: &Path,
    modules_dir_override: Option<&Path>,
    machine_filter: &[String],
) -> Result<PreparedSystemRun, RunError> {
    let inventory = load_inventory(inventory_path)?;
    if let Err(errors) = validate_inventory(&inventory) {
        emit_pre_machine_failure(errors);
        return Err(RunError::Reported(3));
    }
    let selected_machines = select_machines(&inventory, machine_filter).map_err(emit_and_wrap_cli_failure)?;
    let modules_dir = resolve_modules_dir(inventory_path, modules_dir_override);
    let modules = load_validated_modules(&modules_dir)?;
    let machine_filter_ref = Some(&selected_machines);
    let boundary = JsonEvalBoundary;
    let module_execution = evaluate_modules_for_machines(&inventory, &modules, &boundary, machine_filter_ref);
    let grouped = group_by_machine_with_filter(&module_execution.fragments, &inventory, machine_filter_ref);
    let merged_configs = collect_merged_configs(grouped)?;

    Ok(PreparedSystemRun {
        inventory,
        selected_machines,
        module_execution,
        merged_configs,
    })
}

fn load_inventory(inventory_path: &Path) -> Result<Inventory, RunError> {
    let import_paths = inventory_import_paths(inventory_path)?;
    crunch_eval::evaluate_and_deserialize(inventory_path, &import_paths)
        .map_err(|error| RunError::Eval(format!("loading inventory {}: {error}", inventory_path.display())))
}

fn inventory_import_paths(inventory_path: &Path) -> Result<Vec<OsString>, RunError> {
    let inventory_dir = inventory_path
        .parent()
        .ok_or_else(|| RunError::Internal(format!("inventory path {} has no parent directory", inventory_path.display())))?;
    Ok(vec![inventory_dir.as_os_str().to_os_string()])
}

fn resolve_modules_dir(inventory_path: &Path, modules_dir_override: Option<&Path>) -> PathBuf {
    if let Some(path) = modules_dir_override {
        return path.to_path_buf();
    }
    inventory_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(DEFAULT_MODULES_DIR_NAME)
}

fn discovered_modules_dir(modules_dir: &Path) -> PathBuf {
    modules_dir.to_path_buf()
}

fn load_validated_modules(modules_dir: &Path) -> Result<Vec<ValidatedModule>, RunError> {
    let discovery_dir = discovered_modules_dir(modules_dir);
    let discovered = crunch_system::loader::discover_module_files(&discovery_dir)
        .map_err(|error| fatal_system_error(error, 3))?;
    let import_paths = module_import_paths(&discovery_dir)?;
    let handle = EvalThread::spawn(import_paths.iter().map(PathBuf::from).collect());
    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| RunError::Internal(format!("building system module runtime: {error}")))?;
    let modules = runtime.block_on(load_validated_modules_on_thread(&handle, &discovered));
    shutdown_eval_thread(runtime, handle)?;
    modules
}

fn module_import_paths(modules_dir: &Path) -> Result<Vec<OsString>, RunError> {
    let path = modules_dir
        .canonicalize()
        .map_err(|error| RunError::Internal(format!("canonicalizing modules dir {}: {error}", modules_dir.display())))?;
    let stdlib_path = crunch_eval::stdlib::stdlib_import_path().map_err(|error| {
        RunError::Internal(format!("resolving embedded stdlib import path for system modules: {error}"))
    })?;
    Ok(vec![path.into_os_string(), stdlib_path.into_os_string()])
}

async fn load_validated_modules_on_thread(
    handle: &EvalThreadHandle,
    discovered: &[(String, PathBuf)],
) -> Result<Vec<ValidatedModule>, RunError> {
    let mut modules = Vec::with_capacity(discovered.len());
    for (name, path) in discovered {
        let root_value_id = evaluate_module_file(handle, path).await?;
        let validated = validate_module(name, root_value_id, handle)
            .await
            .map_err(|error| fatal_system_error(error, 3))?;
        drop_module_root_value(handle, root_value_id, name).await?;
        modules.push(validated);
    }
    Ok(modules)
}

async fn evaluate_module_file(handle: &EvalThreadHandle, path: &Path) -> Result<ValueId, RunError> {
    let options = crunch_system::eval_trait::EvalOptions {
        timeout: Some(Duration::from_secs(MODULE_TIMEOUT_SECS)),
        import_paths: Vec::new(),
    };
    handle
        .evaluate_file(path.to_path_buf(), options)
        .await
        .map_err(|error| RunError::Eval(format!("evaluating module {}: {error}", path.display())))
}

async fn drop_module_root_value(handle: &EvalThreadHandle, value_id: ValueId, module_name: &str) -> Result<(), RunError> {
    handle
        .drop_value(value_id)
        .await
        .map_err(|error| RunError::Internal(format!("dropping module root value for {module_name}: {error}")))
}

fn shutdown_eval_thread(runtime: tokio::runtime::Runtime, handle: EvalThreadHandle) -> Result<(), RunError> {
    runtime
        .block_on(async { handle.shutdown().await })
        .map_err(|error| RunError::Internal(format!("shutting down system module evaluator: {error}")))
}

fn select_machines(inventory: &Inventory, machine_filter: &[String]) -> Result<BTreeSet<String>, SystemConfigError> {
    if machine_filter.is_empty() {
        return Ok(inventory.machines.keys().cloned().collect());
    }
    let mut selected = BTreeSet::new();
    for machine_name in machine_filter {
        if !inventory.machines.contains_key(machine_name) {
            return Err(cli_error(format!("unknown machine '{machine_name}'"), None));
        }
        selected.insert(machine_name.clone());
    }
    Ok(selected)
}

fn collect_merged_configs(
    grouped: BTreeMap<String, Vec<&crunch_system::EvaluatedFragment>>,
) -> Result<BTreeMap<String, crunch_system::MergedConfig>, RunError> {
    let mut merged = BTreeMap::new();
    for (machine_name, fragments) in grouped {
        let owned_fragments = fragments.into_iter().cloned().collect::<Vec<_>>();
        match merge_fragments(&owned_fragments) {
            Ok(config) => {
                merged.insert(machine_name, config);
            }
            Err(errors) => {
                emit_pre_machine_failure(errors);
                return Err(RunError::Reported(3));
            }
        }
    }
    Ok(merged)
}

fn default_assembler_registry() -> AssemblerRegistry {
    let mut registry = AssemblerRegistry::new();
    registry.register(Box::new(NixosPhase1Assembler));
    registry
}

fn populate_derivation_outcomes(
    inventory: &Inventory,
    selected_machines: &BTreeSet<String>,
    merged_configs: &BTreeMap<String, crunch_system::MergedConfig>,
    assembler_override: Option<&str>,
    registry: &AssemblerRegistry,
    result: &mut JsonSystemPipelineResult,
) -> Result<(), RunError> {
    for machine_name in selected_machines {
        let Some(machine) = inventory.machines.get(machine_name) else {
            continue;
        };
        let Some(config) = merged_configs.get(machine_name) else {
            result.machines.insert(machine_name.clone(), failed_machine_outcome());
            continue;
        };
        match dry_run_assemble(registry, machine_name, machine, config, assembler_override) {
            Ok(derivations) => {
                let derivations_json = derivations
                    .iter()
                    .map(derivation_to_json)
                    .collect::<Result<Vec<_>, _>>()?;
                result.machines.insert(
                    machine_name.clone(),
                    JsonMachineOutcome {
                        kind: "derivations",
                        merged_config: None,
                        derivations: Some(derivations_json),
                        reports: None,
                    },
                );
            }
            Err(error) => {
                result.errors.push(error_diagnostic(&map_assembler_error(&error)));
                result.machines.insert(machine_name.clone(), failed_machine_outcome());
            }
        }
    }
    Ok(())
}

fn populate_build_outcomes(
    ctx: &RunContext,
    inventory: &Inventory,
    selected_machines: &BTreeSet<String>,
    merged_configs: &BTreeMap<String, crunch_system::MergedConfig>,
    assembler_override: Option<&str>,
    registry: &AssemblerRegistry,
    result: &mut JsonSystemPipelineResult,
) -> Result<(), RunError> {
    for machine_name in selected_machines {
        let Some(machine) = inventory.machines.get(machine_name) else {
            continue;
        };
        let Some(config) = merged_configs.get(machine_name) else {
            result.machines.insert(machine_name.clone(), failed_machine_outcome());
            continue;
        };
        match dry_run_assemble(registry, machine_name, machine, config, assembler_override) {
            Ok(derivations) => {
                let reports = build_reports_for_machine(ctx, machine_name, &derivations)?;
                result.machines.insert(
                    machine_name.clone(),
                    JsonMachineOutcome {
                        kind: "build",
                        merged_config: None,
                        derivations: None,
                        reports: Some(reports),
                    },
                );
            }
            Err(error) => {
                result.errors.push(error_diagnostic(&map_assembler_error(&error)));
                result.machines.insert(machine_name.clone(), failed_machine_outcome());
            }
        }
    }
    Ok(())
}

fn derivation_to_json(derivation: &crunch_glue::CrunchDerivation) -> Result<Value, RunError> {
    let inputs = derivation
        .inputs
        .iter()
        .map(input_to_json)
        .collect::<Vec<_>>();
    let fixed_output = derivation.fixed_output.as_ref().map(|fixed_output| {
        serde_json::json!({
            "algo": fixed_output.algo,
            "hash": fixed_output.hash,
            "mode": fixed_output.mode,
        })
    });
    Ok(serde_json::json!({
        "name": derivation.name,
        "builder": derivation.builder,
        "system": derivation.system,
        "args": derivation.args,
        "outputs": derivation.outputs,
        "env": derivation.env,
        "inputs": inputs,
        "fixed_output": fixed_output,
        "addressing_mode": derivation.addressing_mode,
    }))
}

fn input_to_json(input: &crunch_glue::Input) -> Value {
    match input {
        crunch_glue::Input::Source(path) => Value::String(path.clone()),
        crunch_glue::Input::OutputSelection(output_ref) => serde_json::json!({
            "drv": output_ref.drv.name,
            "output": output_ref.output,
        }),
        crunch_glue::Input::Derivation(derivation) => serde_json::json!({
            "name": derivation.name,
            "builder": derivation.builder,
        }),
    }
}

fn build_reports_for_machine(
    ctx: &RunContext,
    _machine_name: &str,
    derivations: &[crunch_glue::CrunchDerivation],
) -> Result<Vec<Value>, RunError> {
    let build_expr = machine_build_expression(derivations)?;
    let pipeline_result = build_machine_pipeline_result(ctx, &build_expr)?;
    let logs_dir = build_cmd::log_dir();
    let report_json = crate::build_report::render_build_json_report(
        &build_machine_config(ctx, &build_expr)?,
        &pipeline_result,
        &logs_dir,
        &[],
    )
    .map_err(|error| RunError::Internal(format!("serializing machine build report: {error}")))?;
    Ok(vec![serde_json::from_str(&report_json).map_err(json_internal_error)?])
}

fn build_machine_pipeline_result(ctx: &RunContext, build_expr: &str) -> Result<crunch_pipeline::PipelineResult, RunError> {
    let config = build_machine_config(ctx, build_expr)?;
    build_cmd::run_build(&config)
}

fn build_machine_config(ctx: &RunContext, build_expr: &str) -> Result<crunch_pipeline::BuildConfig, RunError> {
    let tmp = tempfile::NamedTempFile::with_suffix(".ncl")
        .map_err(|error| RunError::Internal(format!("creating system build temp file: {error}")))?;
    std::fs::write(tmp.path(), build_expr)
        .map_err(|error| RunError::Internal(format!("writing system build temp file: {error}")))?;
    let temp_path = tmp.into_temp_path();
    let persisted = temp_path.keep().map_err(|error| {
        RunError::Internal(format!("persisting system build temp file {}: {}", error.path.display(), error.error))
    })?;
    let keypair = build_cmd::load_or_generate_signing_keypair(None, &ctx.resolved_state_dir, !ctx.json)?;
    let configured_trusted_keys = build_cmd::load_configured_trusted_public_keys(None, &ctx.resolved_state_dir)?;
    let trusted_keys = crunch_build::signing::build_trusted_keys(&keypair, configured_trusted_keys.as_deref());
    Ok(crunch_pipeline::BuildConfig {
        file: persisted,
        import_paths: Vec::new(),
        output_dir: ctx.store.clone(),
        state_dir: ctx.resolved_state_dir.clone(),
        store_dir: ctx.store_prefix.clone(),
        verbose: ctx.verbose,
        max_jobs: 1,
        substituter_url: None,
        hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
        keypair,
        trusted_keys,
        trust_unsigned: false,
        root_retention_source: Some(crunch_store::GcRootSource::Build),
    })
}

fn machine_build_expression(derivations: &[crunch_glue::CrunchDerivation]) -> Result<String, RunError> {
    let derivation_exprs = derivations
        .iter()
        .map(derivation_to_nickel)
        .collect::<Vec<_>>();
    Ok(format!("[\n{}\n]", derivation_exprs.join(",\n")))
}

fn derivation_to_nickel(derivation: &crunch_glue::CrunchDerivation) -> String {
    let args = render_string_list(&derivation.args);
    let outputs = render_string_list(&derivation.outputs);
    let env = render_string_map(&derivation.env);
    let inputs = render_input_list(&derivation.inputs);
    let fixed_output = render_fixed_output(&derivation.fixed_output);
    format!(
        "{{ name = {}, builder = {}, system = {}, args = {}, outputs = {}, env = {}, inputs = {}, fixed_output = {}, addressing_mode = {} }}",
        render_string(&derivation.name),
        render_string(&derivation.builder),
        render_string(&derivation.system),
        args,
        outputs,
        env,
        inputs,
        fixed_output,
        render_string(&derivation.addressing_mode),
    )
}

fn render_input_list(inputs: &[crunch_glue::Input]) -> String {
    let rendered = inputs.iter().map(render_input).collect::<Vec<_>>();
    format!("[{}]", rendered.join(", "))
}

fn render_input(input: &crunch_glue::Input) -> String {
    match input {
        crunch_glue::Input::Source(path) => render_string(path),
        crunch_glue::Input::OutputSelection(output_ref) => format!("{{ drv = {}, output = {} }}", derivation_to_nickel(&output_ref.drv), render_string(&output_ref.output)),
        crunch_glue::Input::Derivation(derivation) => derivation_to_nickel(derivation),
    }
}

fn render_fixed_output(fixed_output: &Option<crunch_glue::FixedOutput>) -> String {
    let Some(fixed_output) = fixed_output else {
        return "null".to_string();
    };
    format!(
        "{{ algo = {}, hash = {}, mode = {} }}",
        render_string(&fixed_output.algo),
        render_string(&fixed_output.hash),
        render_string(&fixed_output.mode),
    )
}

fn render_string_list(values: &[String]) -> String {
    let rendered = values.iter().map(|value| render_string(value)).collect::<Vec<_>>();
    format!("[{}]", rendered.join(", "))
}

fn render_string_map(values: &std::collections::HashMap<String, String>) -> String {
    let mut entries = values.iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.0.cmp(right.0));
    let rendered = entries
        .into_iter()
        .map(|(key, value)| format!("{} = {}", key, render_string(value)))
        .collect::<Vec<_>>();
    format!("{{ {} }}", rendered.join(", "))
}

fn render_string(value: &str) -> String {
    serde_json::to_string(value).expect("string serialization must succeed")
}

fn json_merged_config(merged_config: crunch_system::MergedConfig) -> JsonMergedConfig {
    JsonMergedConfig {
        machine_name: merged_config.machine_name,
        data: merged_config.data,
        provenance: merged_config.provenance,
    }
}

fn failed_machine_outcome() -> JsonMachineOutcome {
    JsonMachineOutcome {
        kind: "failed",
        merged_config: None,
        derivations: None,
        reports: None,
    }
}

fn build_human_summary(result: &JsonSystemPipelineResult) -> HumanSystemBuildSummary {
    let mut succeeded_machines = Vec::new();
    let mut failed_machines = Vec::new();
    for (machine_name, outcome) in &result.machines {
        if outcome.kind == "failed" {
            failed_machines.push(machine_name.clone());
        } else {
            succeeded_machines.push(machine_name.clone());
        }
    }
    HumanSystemBuildSummary {
        succeeded_machines,
        failed_machines,
        warning_count: result.warnings.len() as u32,
        error_count: result.errors.len() as u32,
    }
}

fn emit_nonfatal_diagnostics(
    ctx: &RunContext,
    errors: &[SystemConfigError],
    warnings: &[crunch_system::error::SystemConfigWarning],
) -> Result<(), RunError> {
    for warning in warnings {
        emit_warning(ctx, warning)?;
    }
    for error in errors {
        emit_error(ctx, error)?;
    }
    Ok(())
}

fn collect_renderable_errors(
    base_errors: &[SystemConfigError],
    rendered_errors: &[SystemDiagnosticEnvelope],
) -> Vec<SystemConfigError> {
    let mut collected = base_errors.to_vec();
    for diagnostic in rendered_errors {
        if diagnostic.layer != DIAGNOSTIC_LAYER_ASSEMBLER {
            continue;
        }
        collected.push(SystemConfigError::Assembler {
            message: diagnostic.message.clone(),
            detail: diagnostic.detail.clone(),
            machine_name: diagnostic.machine.clone(),
            module_name: diagnostic.module.clone(),
            field_path: diagnostic.field_path.clone(),
        });
    }
    collected
}

fn emit_pre_machine_failure(errors: Vec<SystemConfigError>) {
    for error in errors {
        eprintln!("{}", human_error_line(&error));
    }
}

fn emit_and_wrap_cli_failure(error: SystemConfigError) -> RunError {
    eprintln!("{}", human_error_line(&error));
    RunError::Reported(3)
}

fn emit_fatal_cli_error(ctx: &RunContext, error: SystemConfigError) -> Result<(), RunError> {
    emit_error(ctx, &error)
}

fn emit_warning(ctx: &RunContext, warning: &crunch_system::error::SystemConfigWarning) -> Result<(), RunError> {
    if ctx.json {
        let rendered = serde_json::to_string(&warning_diagnostic(warning)).map_err(json_internal_error)?;
        eprintln!("{rendered}");
        return Ok(());
    }
    eprintln!("warning: {warning}");
    Ok(())
}

fn emit_error(ctx: &RunContext, error: &SystemConfigError) -> Result<(), RunError> {
    if ctx.json {
        let rendered = serde_json::to_string(&error_diagnostic(error)).map_err(json_internal_error)?;
        eprintln!("{rendered}");
        return Ok(());
    }
    eprintln!("{}", human_error_line(error));
    Ok(())
}

fn human_error_line(error: &SystemConfigError) -> String {
    match error {
        SystemConfigError::Cli { message, .. }
        | SystemConfigError::Loader { message, .. }
        | SystemConfigError::Inventory { message, .. }
        | SystemConfigError::CrossRef { message, .. }
        | SystemConfigError::Eval { message, .. }
        | SystemConfigError::Fragment { message, .. }
        | SystemConfigError::Assembler { message, .. } => format!("error: {message}"),
    }
}

fn error_diagnostic(error: &SystemConfigError) -> SystemDiagnosticEnvelope {
    match error {
        SystemConfigError::Cli {
            message,
            detail,
            machine_name,
            module_name,
            field_path,
        } => diagnostic(DIAGNOSTIC_LAYER_CLI, message, detail, machine_name, module_name, field_path),
        SystemConfigError::Loader {
            message,
            detail,
            machine_name,
            module_name,
            field_path,
        } => diagnostic(DIAGNOSTIC_LAYER_LOADER, message, detail, machine_name, module_name, field_path),
        SystemConfigError::Inventory {
            message,
            detail,
            machine_name,
            module_name,
            field_path,
        } => diagnostic(DIAGNOSTIC_LAYER_INVENTORY, message, detail, machine_name, module_name, field_path),
        SystemConfigError::CrossRef {
            message,
            detail,
            machine_name,
            module_name,
            field_path,
        } => diagnostic(DIAGNOSTIC_LAYER_CROSSREF, message, detail, machine_name, module_name, field_path),
        SystemConfigError::Eval {
            message,
            detail,
            machine_name,
            module_name,
            field_path,
        } => diagnostic(DIAGNOSTIC_LAYER_EVAL, message, detail, machine_name, module_name, field_path),
        SystemConfigError::Fragment {
            message,
            detail,
            machine_name,
            module_name,
            field_path,
        } => diagnostic(DIAGNOSTIC_LAYER_FRAGMENT, message, detail, machine_name, module_name, field_path),
        SystemConfigError::Assembler {
            message,
            detail,
            machine_name,
            module_name,
            field_path,
        } => diagnostic(DIAGNOSTIC_LAYER_ASSEMBLER, message, detail, machine_name, module_name, field_path),
    }
}

fn warning_diagnostic(warning: &crunch_system::error::SystemConfigWarning) -> SystemDiagnosticEnvelope {
    match warning {
        crunch_system::error::SystemConfigWarning::OrphanProviderConsumption {
            provider_type,
            machine_name,
            module_name,
        } => SystemDiagnosticEnvelope {
            severity: DIAGNOSTIC_SEVERITY_WARNING,
            layer: DIAGNOSTIC_LAYER_EVAL,
            message: format!("orphan provider consumption: {provider_type}"),
            detail: None,
            machine: machine_name.clone(),
            module: module_name.clone(),
            field_path: None,
        },
    }
}

fn diagnostic(
    layer: &'static str,
    message: &str,
    detail: &Option<String>,
    machine_name: &Option<String>,
    module_name: &Option<String>,
    field_path: &Option<String>,
) -> SystemDiagnosticEnvelope {
    SystemDiagnosticEnvelope {
        severity: DIAGNOSTIC_SEVERITY_ERROR,
        layer,
        message: message.to_string(),
        detail: detail.clone(),
        machine: machine_name.clone(),
        module: module_name.clone(),
        field_path: field_path.clone(),
    }
}

fn cli_error(message: String, detail: Option<String>) -> SystemConfigError {
    SystemConfigError::Cli {
        message,
        detail,
        machine_name: None,
        module_name: None,
        field_path: None,
    }
}

fn map_assembler_error(error: &AssemblerError) -> SystemConfigError {
    match error {
        AssemblerError::Assemble {
            assembler_name,
            machine_name,
            message,
        } => SystemConfigError::Assembler {
            message: format!("assembler '{assembler_name}' failed: {message}"),
            detail: None,
            machine_name: Some(machine_name.clone()),
            module_name: None,
            field_path: None,
        },
        AssemblerError::UnknownAssembler {
            assembler_name,
            machine_name,
        } => SystemConfigError::Assembler {
            message: format!("unknown assembler '{assembler_name}'"),
            detail: None,
            machine_name: Some(machine_name.clone()),
            module_name: None,
            field_path: None,
        },
    }
}

fn fatal_system_error(error: SystemConfigError, code: u8) -> RunError {
    eprintln!("{}", human_error_line(&error));
    RunError::Reported(code)
}

fn json_internal_error(error: serde_json::Error) -> RunError {
    RunError::Internal(format!("serializing system command output: {error}"))
}

struct JsonEvalBoundary;

impl EvalBoundary for JsonEvalBoundary {
    fn merge_settings(&self, defaults: &Value, settings: &Value) -> Result<Value, String> {
        let defaults_object = defaults.as_object().cloned().unwrap_or_default();
        let settings_object = settings.as_object().cloned().unwrap_or_default();
        let mut merged = defaults_object;
        for (key, value) in settings_object {
            merged.insert(key, value);
        }
        Ok(Value::Object(merged))
    }

    fn invoke_impl(
        &self,
        module: &ValidatedModule,
        args: &Value,
        _timeout_secs: u64,
    ) -> Result<Value, String> {
        let timeout_secs = Duration::from_secs(MODULE_TIMEOUT_SECS).as_secs();
        Ok(serde_json::json!({
            "output": {
                "exports": {
                    "args": args,
                    "module": module.module_name,
                    "timeout_secs": timeout_secs,
                },
                "nixos": {
                    "module": module.module_name,
                    "machine": args.get("machine_name").cloned().unwrap_or(Value::Null),
                    "settings": args.get("settings").cloned().unwrap_or(Value::Null),
                }
            }
        }))
    }
}
