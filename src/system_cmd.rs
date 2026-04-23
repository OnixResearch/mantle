use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use clap::ValueEnum;
use crunch_system::ValidatedModule;
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
use crate::errors::RunError;

const DEFAULT_MODULES_DIR_NAME: &str = "modules";
const DEFAULT_MODULE_PRIORITY: i64 = 1000;
const NICKEL_FORMAT_NOT_IMPLEMENTED: &str = "system eval --format nickel is not implemented yet";
const BUILD_REPORT_SCHEMA: &str = "crunch-build-report-v1";

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
    kind: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    machine_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    module_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Debug, Serialize)]
struct BuildMachineReport {
    schema: &'static str,
    machine_name: String,
    derivation_count: u32,
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
    errors: Vec<SystemConfigError>,
    warnings: Vec<crunch_system::error::SystemConfigWarning>,
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
        errors: prepared.module_execution.errors.clone(),
        warnings: prepared.module_execution.warnings.clone(),
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

    emit_nonfatal_diagnostics(ctx, &result)?;
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
        errors: prepared.module_execution.errors.clone(),
        warnings: prepared.module_execution.warnings.clone(),
    };

    populate_build_outcomes(
        &prepared.inventory,
        &prepared.selected_machines,
        &prepared.merged_configs,
        options.assembler_override.as_deref(),
        &registry,
        &mut result,
    )?;

    emit_nonfatal_diagnostics(ctx, &result)?;
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

fn load_validated_modules(modules_dir: &Path) -> Result<Vec<ValidatedModule>, RunError> {
    let discovered = crunch_system::loader::discover_module_files(modules_dir)
        .map_err(|error| fatal_system_error(error, 3))?;
    let import_paths = module_import_paths(modules_dir)?;
    let mut modules = Vec::with_capacity(discovered.len());
    for (name, path) in discovered {
        let source = std::fs::read_to_string(&path)
            .map_err(|error| RunError::Internal(format!("reading module {}: {error}", path.display())))?;
        let metadata_json = module_metadata_json(&source, &path, &import_paths)?;
        let value: Value = serde_json::from_str(&metadata_json)
            .map_err(|error| RunError::Internal(format!("deserializing module {}: {error}", path.display())))?;
        modules.push(validate_module_json(&name, &value).map_err(|error| fatal_system_error(error, 3))?);
    }
    Ok(modules)
}

fn module_metadata_json(source: &str, path: &Path, import_paths: &[OsString]) -> Result<String, RunError> {
    let metadata_source = format!(
        "let module = ({source}) in let inputs = if std.record.has_field \"inputs\" module then module.inputs else [] in let consumes_providers = if std.record.has_field \"consumes_providers\" module then module.consumes_providers else [] in let produces_providers = if std.record.has_field \"produces_providers\" module then module.produces_providers else [] in let priority = if std.record.has_field \"priority\" module then module.priority else 1000 in {{ interface = module.interface, inputs = inputs, consumes_providers = consumes_providers, produces_providers = produces_providers, priority = priority, impl_present = true }}"
    );
    crunch_eval::evaluate_str_to_json(&metadata_source, import_paths)
        .map_err(|error| RunError::Eval(format!("evaluating module {}: {error}", path.display())))
}

fn module_import_paths(modules_dir: &Path) -> Result<Vec<OsString>, RunError> {
    let path = modules_dir
        .canonicalize()
        .map_err(|error| RunError::Internal(format!("canonicalizing modules dir {}: {error}", modules_dir.display())))?;
    Ok(vec![path.into_os_string()])
}

fn validate_module_json(name: &str, value: &Value) -> Result<ValidatedModule, SystemConfigError> {
    let object = value.as_object().ok_or_else(|| loader_error(format!("module '{name}' must evaluate to a record"), name))?;
    let interface = object
        .get("interface")
        .and_then(Value::as_object)
        .ok_or_else(|| loader_error(format!("module '{name}' is missing interface"), name))?;
    let roles = interface
        .get("roles")
        .and_then(Value::as_object)
        .ok_or_else(|| loader_error(format!("module '{name}' is missing interface.roles"), name))?;
    let mut role_names = roles.keys().cloned().collect::<Vec<_>>();
    role_names.sort();
    if !object.contains_key("impl") {
        return Err(loader_error(format!("module '{name}' is missing impl"), name));
    }
    Ok(ValidatedModule {
        module_name: name.to_string(),
        role_names,
        inputs: string_array_field(object.get("inputs"), "inputs", name)?,
        consumes_providers: string_array_field(object.get("consumes_providers"), "consumes_providers", name)?,
        produces_providers: string_array_field(object.get("produces_providers"), "produces_providers", name)?,
        priority: integer_field(object.get("priority"), "priority", name)?.unwrap_or(DEFAULT_MODULE_PRIORITY),
    })
}

fn string_array_field(value: Option<&Value>, field_name: &str, module_name: &str) -> Result<Vec<String>, SystemConfigError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let array = value
        .as_array()
        .ok_or_else(|| loader_error(format!("module '{module_name}' field '{field_name}' must be an array"), module_name))?;
    let mut strings = Vec::with_capacity(array.len());
    for item in array {
        let Some(text) = item.as_str() else {
            return Err(loader_error(
                format!("module '{module_name}' field '{field_name}' must contain only strings"),
                module_name,
            ));
        };
        strings.push(text.to_string());
    }
    Ok(strings)
}

fn integer_field(value: Option<&Value>, field_name: &str, module_name: &str) -> Result<Option<i64>, SystemConfigError> {
    let Some(value) = value else {
        return Ok(None);
    };
    value
        .as_i64()
        .ok_or_else(|| loader_error(format!("module '{module_name}' field '{field_name}' must be an integer"), module_name))
        .map(Some)
}

fn loader_error(message: String, module_name: &str) -> SystemConfigError {
    SystemConfigError::Loader {
        message,
        detail: None,
        machine_name: None,
        module_name: Some(module_name.to_string()),
        field_path: None,
    }
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
                result.errors.push(map_assembler_error(&error));
                result.machines.insert(machine_name.clone(), failed_machine_outcome());
            }
        }
    }
    Ok(())
}

fn populate_build_outcomes(
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
                let reports = build_reports_for_machine(machine_name, derivations.len())?;
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
                result.errors.push(map_assembler_error(&error));
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

fn build_reports_for_machine(machine_name: &str, derivation_count: usize) -> Result<Vec<Value>, RunError> {
    let derivation_count = u32::try_from(derivation_count)
        .map_err(|_| RunError::Internal(format!("derivation count for machine '{machine_name}' does not fit in u32")))?;
    let report = BuildMachineReport {
        schema: BUILD_REPORT_SCHEMA,
        machine_name: machine_name.to_string(),
        derivation_count,
    };
    Ok(vec![serde_json::to_value(report).map_err(json_internal_error)?])
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

fn emit_nonfatal_diagnostics(ctx: &RunContext, result: &JsonSystemPipelineResult) -> Result<(), RunError> {
    for warning in &result.warnings {
        emit_warning(ctx, warning)?;
    }
    for error in &result.errors {
        emit_error(ctx, error)?;
    }
    Ok(())
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
        } => diagnostic("cli", message, detail, machine_name, module_name, field_path),
        SystemConfigError::Loader {
            message,
            detail,
            machine_name,
            module_name,
            field_path,
        } => diagnostic("loader", message, detail, machine_name, module_name, field_path),
        SystemConfigError::Inventory {
            message,
            detail,
            machine_name,
            module_name,
            field_path,
        } => diagnostic("inventory", message, detail, machine_name, module_name, field_path),
        SystemConfigError::CrossRef {
            message,
            detail,
            machine_name,
            module_name,
            field_path,
        } => diagnostic("crossref", message, detail, machine_name, module_name, field_path),
        SystemConfigError::Eval {
            message,
            detail,
            machine_name,
            module_name,
            field_path,
        } => diagnostic("eval", message, detail, machine_name, module_name, field_path),
        SystemConfigError::Fragment {
            message,
            detail,
            machine_name,
            module_name,
            field_path,
        } => diagnostic("fragment", message, detail, machine_name, module_name, field_path),
        SystemConfigError::Assembler {
            message,
            detail,
            machine_name,
            module_name,
            field_path,
        } => diagnostic("assembler", message, detail, machine_name, module_name, field_path),
    }
}

fn warning_diagnostic(warning: &crunch_system::error::SystemConfigWarning) -> SystemDiagnosticEnvelope {
    match warning {
        crunch_system::error::SystemConfigWarning::OrphanProviderConsumption {
            provider_type,
            machine_name,
            module_name,
        } => SystemDiagnosticEnvelope {
            kind: "warning",
            message: format!("orphan provider consumption: {provider_type}"),
            machine_name: machine_name.clone(),
            module_name: module_name.clone(),
            field_path: None,
            detail: None,
        },
    }
}

fn diagnostic(
    kind: &'static str,
    message: &str,
    detail: &Option<String>,
    machine_name: &Option<String>,
    module_name: &Option<String>,
    field_path: &Option<String>,
) -> SystemDiagnosticEnvelope {
    SystemDiagnosticEnvelope {
        kind,
        message: message.to_string(),
        machine_name: machine_name.clone(),
        module_name: module_name.clone(),
        field_path: field_path.clone(),
        detail: detail.clone(),
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
        Ok(serde_json::json!({
            "output": {
                "exports": {
                    "args": args,
                    "module": module.module_name,
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
