use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde_json::Map as JsonMap;
use serde_json::Value;

use crate::EvaluatedFragment;
use crate::FragmentSource;
use crate::ValidatedModule;
use crate::error::SystemConfigError;
use crate::error::SystemConfigWarning;
use crate::graph::DependencyGraph;
use crate::graph::build_dependency_graph;
use crate::graph::topological_sort;
use crate::inventory::InstanceRecord;
use crate::inventory::Inventory;

const DEFAULT_MODULE_TIMEOUT_SECS: u64 = 60;

#[derive(Debug, Clone)]
pub struct ModuleInstance {
    pub machine_name: String,
    pub module_name: String,
    pub role_name: String,
    pub settings: Value,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleExecutionResult {
    pub fragments: Vec<EvaluatedFragment>,
    pub errors: Vec<SystemConfigError>,
    pub warnings: Vec<SystemConfigWarning>,
}

pub trait EvalBoundary {
    fn merge_settings(&self, defaults: &Value, settings: &Value) -> Result<Value, String>;
    fn invoke_impl(&self, module: &ValidatedModule, args: &Value, timeout_secs: u64) -> Result<Value, String>;
}

pub fn validate_inventory_cross_references(
    inventory: &Inventory,
    modules: &[ValidatedModule],
) -> Result<Vec<ModuleInstance>, Vec<SystemConfigError>> {
    validate_inventory_cross_references_for_machines(inventory, modules, None)
}

pub fn validate_inventory_cross_references_for_machines(
    inventory: &Inventory,
    modules: &[ValidatedModule],
    machine_filter: Option<&BTreeSet<String>>,
) -> Result<Vec<ModuleInstance>, Vec<SystemConfigError>> {
    let modules_by_name = modules_by_name(modules);
    let mut instances = Vec::new();
    let mut errors = Vec::new();

    for (service_name, service_record) in &inventory.services {
        let Some(module) = modules_by_name.get(service_name) else {
            errors.push(cross_ref_error(
                format!("service '{service_name}' references unknown module"),
                None,
                Some(service_name.clone()),
            ));
            continue;
        };
        for instance in &service_record.instances {
            if let Some(filter) = machine_filter
                && !filter.contains(&instance.machine)
            {
                continue;
            }
            let error_count_before = errors.len();
            validate_instance(inventory, module, instance, &mut errors);
            if errors.len() == error_count_before {
                instances.push(ModuleInstance {
                    machine_name: instance.machine.clone(),
                    module_name: service_name.clone(),
                    role_name: instance.role.clone(),
                    settings: instance.settings.clone().unwrap_or(Value::Object(JsonMap::new())),
                    tags: instance.tags.clone(),
                });
            }
        }
    }

    if errors.is_empty() { Ok(instances) } else { Err(errors) }
}

pub fn evaluate_modules(
    inventory: &Inventory,
    modules: &[ValidatedModule],
    boundary: &dyn EvalBoundary,
) -> ModuleExecutionResult {
    evaluate_modules_for_machines(inventory, modules, boundary, None)
}

pub fn evaluate_modules_for_machines(
    inventory: &Inventory,
    modules: &[ValidatedModule],
    boundary: &dyn EvalBoundary,
    machine_filter: Option<&BTreeSet<String>>,
) -> ModuleExecutionResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let instances = match validate_inventory_cross_references_for_machines(inventory, modules, machine_filter) {
        Ok(instances) => instances,
        Err(cross_ref_errors) => {
            return ModuleExecutionResult {
                fragments: Vec::new(),
                errors: cross_ref_errors,
                warnings,
            };
        }
    };
    let graph = match build_dependency_graph(modules) {
        Ok(graph) => graph,
        Err(error) => {
            return ModuleExecutionResult {
                fragments: Vec::new(),
                errors: vec![error],
                warnings,
            };
        }
    };
    let order = match topological_sort(&graph, modules) {
        Ok(order) => order,
        Err(error) => {
            return ModuleExecutionResult {
                fragments: Vec::new(),
                errors: vec![error],
                warnings,
            };
        }
    };
    let modules_by_name = modules_by_name(modules);
    let role_defaults = role_defaults_by_module(modules);
    let instances_by_machine = group_instances_by_machine(&instances);
    let mut fragments = Vec::new();

    for (machine_name, machine_instances) in instances_by_machine {
        let mut upstream_exports = BTreeMap::<String, Value>::new();
        let mut provider_values = BTreeMap::<String, Vec<Value>>::new();
        let mut machine_failed_modules = BTreeMap::<String, String>::new();

        for module_name in &order {
            let relevant_instances: Vec<&ModuleInstance> =
                machine_instances.iter().copied().filter(|instance| &instance.module_name == module_name).collect();
            if relevant_instances.is_empty() {
                continue;
            }
            let module = modules_by_name.get(module_name).expect("module order must reference known modules");
            if let Some(blocking_message) = dependency_failure_message(module, &graph, &machine_failed_modules) {
                errors.push(eval_error(
                    blocking_message.clone(),
                    Some(machine_name.clone()),
                    Some(module_name.clone()),
                ));
                machine_failed_modules.insert(module_name.clone(), blocking_message);
                continue;
            }
            if let Some(producer_failure) = provider_failure_message(module, &graph, &machine_failed_modules) {
                errors.push(eval_error(
                    producer_failure.clone(),
                    Some(machine_name.clone()),
                    Some(module_name.clone()),
                ));
                machine_failed_modules.insert(module_name.clone(), producer_failure);
                continue;
            }

            let providers_json =
                build_provider_input(module, &provider_values, &mut warnings, &machine_name, module_name);
            let upstream_json = build_upstream_input(module, &upstream_exports);
            for instance in relevant_instances {
                let defaults = role_defaults
                    .get(&module.module_name)
                    .and_then(|defaults_by_role| defaults_by_role.get(&instance.role_name))
                    .cloned()
                    .unwrap_or(Value::Object(JsonMap::new()));
                let merged_settings = match boundary.merge_settings(&defaults, &instance.settings) {
                    Ok(settings) => settings,
                    Err(message) => {
                        let error = eval_error(
                            format!(
                                "settings merge failed for module '{}' role '{}': {message}",
                                module.module_name, instance.role_name
                            ),
                            Some(machine_name.clone()),
                            Some(module_name.clone()),
                        );
                        errors.push(error.clone());
                        machine_failed_modules.insert(module_name.clone(), error.to_string());
                        continue;
                    }
                };
                let args = build_impl_args(instance, &merged_settings, &upstream_json, &providers_json);
                let output = match boundary.invoke_impl(module, &args, DEFAULT_MODULE_TIMEOUT_SECS) {
                    Ok(output) => output,
                    Err(message) => {
                        let error = eval_error(
                            format!("module '{}' failed: {message}", module.module_name),
                            Some(machine_name.clone()),
                            Some(module_name.clone()),
                        );
                        errors.push(error.clone());
                        machine_failed_modules.insert(module_name.clone(), error.to_string());
                        continue;
                    }
                };
                if let Some(exports) = output.get("output").and_then(|value| value.get("exports")).cloned() {
                    upstream_exports.insert(module_name.clone(), exports);
                }
                if let Some(providers) =
                    output.get("output").and_then(|value| value.get("providers")).and_then(Value::as_object)
                {
                    for (provider_type, provider_value) in providers {
                        provider_values.entry(provider_type.clone()).or_default().push(provider_value.clone());
                    }
                }
                fragments.push(EvaluatedFragment {
                    machine_name: machine_name.clone(),
                    module_name: module_name.clone(),
                    source: FragmentSource::Module {
                        module_name: module_name.clone(),
                        role_name: instance.role_name.clone(),
                    },
                    data: output,
                });
            }
        }
    }

    ModuleExecutionResult {
        fragments,
        errors,
        warnings,
    }
}

fn validate_instance(
    inventory: &Inventory,
    module: &ValidatedModule,
    instance: &InstanceRecord,
    errors: &mut Vec<SystemConfigError>,
) {
    if !inventory.machines.contains_key(&instance.machine) {
        errors.push(cross_ref_error(
            format!("instance references unknown machine '{}'", instance.machine),
            Some(instance.machine.clone()),
            Some(module.module_name.clone()),
        ));
    }
    if !module.role_names.iter().any(|role| role == &instance.role) {
        errors.push(cross_ref_error(
            format!("instance references unknown role '{}' for module '{}'", instance.role, module.module_name),
            Some(instance.machine.clone()),
            Some(module.module_name.clone()),
        ));
    }
}

fn dependency_failure_message(
    module: &ValidatedModule,
    graph: &DependencyGraph,
    failed_modules: &BTreeMap<String, String>,
) -> Option<String> {
    for dependency_name in &module.inputs {
        if let Some(message) = failed_modules.get(dependency_name) {
            return Some(format!(
                "module '{}' depends on failed module '{}': {message}",
                module.module_name, dependency_name
            ));
        }
    }
    for provider_edge in &graph.provider_edges {
        if provider_edge.consumer_module == module.module_name
            && let Some(message) = failed_modules.get(&provider_edge.producer_module)
        {
            return Some(format!(
                "module '{}' depends on failed provider producer '{}' for provider '{}': {message}",
                module.module_name, provider_edge.producer_module, provider_edge.provider_type
            ));
        }
    }
    None
}

fn provider_failure_message(
    module: &ValidatedModule,
    graph: &DependencyGraph,
    failed_modules: &BTreeMap<String, String>,
) -> Option<String> {
    for provider_edge in &graph.provider_edges {
        if provider_edge.consumer_module == module.module_name
            && let Some(message) = failed_modules.get(&provider_edge.producer_module)
        {
            return Some(format!(
                "module '{}' cannot consume provider '{}' because producer '{}' failed: {message}",
                module.module_name, provider_edge.provider_type, provider_edge.producer_module
            ));
        }
    }
    None
}

fn build_provider_input(
    module: &ValidatedModule,
    provider_values: &BTreeMap<String, Vec<Value>>,
    warnings: &mut Vec<SystemConfigWarning>,
    machine_name: &str,
    module_name: &str,
) -> Value {
    let mut providers = JsonMap::new();
    for provider_type in &module.consumes_providers {
        let values = provider_values.get(provider_type).cloned().unwrap_or_default();
        if values.is_empty() {
            warnings.push(SystemConfigWarning::OrphanProviderConsumption {
                provider_type: provider_type.clone(),
                machine_name: Some(machine_name.to_string()),
                module_name: Some(module_name.to_string()),
            });
        }
        providers.insert(provider_type.clone(), Value::Array(values));
    }
    Value::Object(providers)
}

fn build_upstream_input(module: &ValidatedModule, upstream_exports: &BTreeMap<String, Value>) -> Value {
    let mut upstream = JsonMap::new();
    for dependency_name in &module.inputs {
        if let Some(value) = upstream_exports.get(dependency_name) {
            upstream.insert(dependency_name.clone(), value.clone());
        }
    }
    Value::Object(upstream)
}

fn build_impl_args(instance: &ModuleInstance, merged_settings: &Value, upstream: &Value, providers: &Value) -> Value {
    let mut object = JsonMap::new();
    object.insert("settings".to_string(), merged_settings.clone());
    object.insert("machine_name".to_string(), Value::String(instance.machine_name.clone()));
    object.insert("role_name".to_string(), Value::String(instance.role_name.clone()));
    object.insert("upstream".to_string(), upstream.clone());
    object.insert("providers".to_string(), providers.clone());
    Value::Object(object)
}

fn group_instances_by_machine(instances: &[ModuleInstance]) -> BTreeMap<String, Vec<&ModuleInstance>> {
    let mut grouped = BTreeMap::<String, Vec<&ModuleInstance>>::new();
    for instance in instances {
        grouped.entry(instance.machine_name.clone()).or_default().push(instance);
    }
    grouped
}

fn modules_by_name(modules: &[ValidatedModule]) -> BTreeMap<String, &ValidatedModule> {
    modules.iter().map(|module| (module.module_name.clone(), module)).collect()
}

fn role_defaults_by_module(modules: &[ValidatedModule]) -> BTreeMap<String, BTreeMap<String, Value>> {
    modules
        .iter()
        .map(|module| {
            let defaults = module
                .role_names
                .iter()
                .cloned()
                .map(|role_name| (role_name, Value::Object(JsonMap::new())))
                .collect();
            (module.module_name.clone(), defaults)
        })
        .collect()
}

fn cross_ref_error(message: String, machine_name: Option<String>, module_name: Option<String>) -> SystemConfigError {
    SystemConfigError::CrossRef {
        message,
        detail: None,
        machine_name,
        module_name,
        field_path: None,
    }
}

fn eval_error(message: String, machine_name: Option<String>, module_name: Option<String>) -> SystemConfigError {
    SystemConfigError::Eval {
        message,
        detail: None,
        machine_name,
        module_name,
        field_path: None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[derive(Default)]
    struct FakeBoundary {
        outputs: BTreeMap<String, Result<Value, String>>,
    }

    impl EvalBoundary for FakeBoundary {
        fn merge_settings(&self, defaults: &Value, settings: &Value) -> Result<Value, String> {
            merge_json(defaults, settings)
        }

        fn invoke_impl(&self, module: &ValidatedModule, args: &Value, _timeout_secs: u64) -> Result<Value, String> {
            if let Some(result) = self.outputs.get(&module.module_name) {
                return result.clone();
            }
            Ok(json!({
                "output": {
                    "exports": { "args": args.clone() },
                    "providers": {},
                    "nixos": { module.module_name.clone(): true }
                }
            }))
        }
    }

    fn merge_json(left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Object(left_map), Value::Object(right_map)) => {
                let mut merged = left_map.clone();
                for (key, right_value) in right_map {
                    let next = match merged.get(key) {
                        Some(left_value) => merge_json(left_value, right_value)?,
                        None => right_value.clone(),
                    };
                    merged.insert(key.clone(), next);
                }
                Ok(Value::Object(merged))
            }
            (Value::Null, value) => Ok(value.clone()),
            (_, value) => Ok(value.clone()),
        }
    }

    fn module(name: &str, roles: &[&str], inputs: &[&str], consumes: &[&str], produces: &[&str]) -> ValidatedModule {
        ValidatedModule {
            module_name: name.to_string(),
            role_names: roles.iter().map(|role| (*role).to_string()).collect(),
            inputs: inputs.iter().map(|value| (*value).to_string()).collect(),
            consumes_providers: consumes.iter().map(|value| (*value).to_string()).collect(),
            produces_providers: produces.iter().map(|value| (*value).to_string()).collect(),
            priority: 1000,
        }
    }

    fn inventory() -> Inventory {
        serde_json::from_value(json!({
            "machines": {
                "server1": { "system": "x86_64-linux", "class": "nixos" },
                "server2": { "system": "x86_64-linux", "class": "nixos" }
            },
            "services": {
                "a": { "instances": [{ "machine": "server1", "role": "default", "settings": { "port": 22 }, "tags": [] }] }
            }
        }))
        .unwrap()
    }

    #[test]
    fn cross_reference_validation_rejects_unknown_module() {
        let inventory: Inventory = serde_json::from_value(json!({
            "machines": { "server1": { "system": "x86_64-linux" } },
            "services": { "missing": { "instances": [{ "machine": "server1", "role": "default", "tags": [] }] } }
        }))
        .unwrap();
        let result = validate_inventory_cross_references(&inventory, &[module("a", &["default"], &[], &[], &[])]);

        assert!(
            matches!(result, Err(errors) if errors.iter().any(|error| matches!(error, SystemConfigError::CrossRef { message, .. } if message.contains("unknown module"))))
        );
    }

    #[test]
    fn cross_reference_validation_rejects_unknown_role() {
        let inventory: Inventory = serde_json::from_value(json!({
            "machines": { "server1": { "system": "x86_64-linux" } },
            "services": { "a": { "instances": [{ "machine": "server1", "role": "missing", "tags": [] }] } }
        }))
        .unwrap();
        let result = validate_inventory_cross_references(&inventory, &[module("a", &["default"], &[], &[], &[])]);

        assert!(
            matches!(result, Err(errors) if errors.iter().any(|error| matches!(error, SystemConfigError::CrossRef { message, .. } if message.contains("unknown role"))))
        );
    }

    #[test]
    fn cross_reference_validation_rejects_unknown_machine() {
        let inventory: Inventory = serde_json::from_value(json!({
            "machines": { "server1": { "system": "x86_64-linux" } },
            "services": { "a": { "instances": [{ "machine": "server2", "role": "default", "tags": [] }] } }
        }))
        .unwrap();
        let result = validate_inventory_cross_references(&inventory, &[module("a", &["default"], &[], &[], &[])]);

        assert!(
            matches!(result, Err(errors) if errors.iter().any(|error| matches!(error, SystemConfigError::CrossRef { message, .. } if message.contains("unknown machine"))))
        );
    }

    #[test]
    fn settings_merge_with_fake_boundary_returns_merged_value() {
        let boundary = FakeBoundary::default();
        let merged = boundary
            .merge_settings(
                &json!({ "a": 1, "nested": { "left": true } }),
                &json!({ "b": 2, "nested": { "right": true } }),
            )
            .unwrap();

        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"], 2);
        assert_eq!(merged["nested"]["left"], true);
        assert_eq!(merged["nested"]["right"], true);
    }

    #[test]
    fn impl_invocation_args_record_shape_is_preserved() {
        let mut boundary = FakeBoundary::default();
        boundary
            .outputs
            .insert("a".to_string(), Ok(json!({ "output": { "exports": { "ok": true }, "providers": {} } })));
        let inventory = inventory();
        let modules = vec![module("a", &["default"], &[], &[], &[])];

        let result = evaluate_modules(&inventory, &modules, &boundary);

        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert_eq!(result.fragments.len(), 1);
        assert!(result.fragments[0].data["output"]["exports"]["ok"].as_bool().unwrap());
    }

    #[test]
    fn export_threading_passes_upstream_exports_to_dependents() {
        let boundary = FakeBoundary::default();
        let inventory: Inventory = serde_json::from_value(json!({
            "machines": { "server1": { "system": "x86_64-linux" } },
            "services": {
                "a": { "instances": [{ "machine": "server1", "role": "default", "tags": [] }] },
                "b": { "instances": [{ "machine": "server1", "role": "default", "tags": [] }] }
            }
        }))
        .unwrap();
        let modules = vec![
            module("a", &["default"], &[], &[], &[]),
            module("b", &["default"], &["a"], &[], &[]),
        ];

        let result = evaluate_modules(&inventory, &modules, &boundary);

        assert!(result.errors.is_empty());
        let b_fragment = result.fragments.iter().find(|fragment| fragment.module_name == "b").unwrap();
        assert!(b_fragment.data["output"]["exports"]["args"]["upstream"]["a"].is_object());
    }

    #[test]
    fn provider_merging_collects_ordered_arrays() {
        let mut boundary = FakeBoundary::default();
        boundary.outputs.insert(
            "a".to_string(),
            Ok(json!({ "output": { "exports": {}, "providers": { "firewall": { "name": "a" } } } })),
        );
        boundary.outputs.insert(
            "b".to_string(),
            Ok(json!({ "output": { "exports": {}, "providers": { "firewall": { "name": "b" } } } })),
        );
        let inventory: Inventory = serde_json::from_value(json!({
            "machines": { "server1": { "system": "x86_64-linux" } },
            "services": {
                "a": { "instances": [{ "machine": "server1", "role": "default", "tags": [] }] },
                "b": { "instances": [{ "machine": "server1", "role": "default", "tags": [] }] },
                "c": { "instances": [{ "machine": "server1", "role": "default", "tags": [] }] }
            }
        }))
        .unwrap();
        let modules = vec![
            module("a", &["default"], &[], &[], &["firewall"]),
            module("b", &["default"], &[], &[], &["firewall"]),
            module("c", &["default"], &[], &["firewall"], &[]),
        ];

        let result = evaluate_modules(&inventory, &modules, &boundary);

        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let c_fragment = result.fragments.iter().find(|fragment| fragment.module_name == "c").unwrap();
        let provider_array = c_fragment.data["output"]["exports"]["args"]["providers"]["firewall"].as_array().unwrap();
        assert_eq!(provider_array.len(), 2);
    }

    #[test]
    fn fail_open_keeps_independent_module_results() {
        let mut boundary = FakeBoundary::default();
        boundary.outputs.insert("a".to_string(), Err("boom".to_string()));
        let inventory: Inventory = serde_json::from_value(json!({
            "machines": {
                "server1": { "system": "x86_64-linux" },
                "server2": { "system": "x86_64-linux" }
            },
            "services": {
                "a": { "instances": [{ "machine": "server1", "role": "default", "tags": [] }] },
                "b": { "instances": [{ "machine": "server2", "role": "default", "tags": [] }] }
            }
        }))
        .unwrap();
        let modules = vec![
            module("a", &["default"], &[], &[], &[]),
            module("b", &["default"], &[], &[], &[]),
        ];

        let result = evaluate_modules(&inventory, &modules, &boundary);

        assert!(result.fragments.iter().any(|fragment| fragment.module_name == "b"));
        assert!(
            result
                .errors
                .iter()
                .any(|error| matches!(error, SystemConfigError::Eval { message, .. } if message.contains("boom")))
        );
    }

    #[test]
    fn orphan_provider_warning_records_empty_provider_array() {
        let boundary = FakeBoundary::default();
        let inventory: Inventory = serde_json::from_value(json!({
            "machines": { "server1": { "system": "x86_64-linux" } },
            "services": { "c": { "instances": [{ "machine": "server1", "role": "default", "tags": [] }] } }
        }))
        .unwrap();
        let modules = vec![module("c", &["default"], &[], &["firewall"], &[])];

        let result = evaluate_modules(&inventory, &modules, &boundary);

        assert!(result.warnings.iter().any(|warning| matches!(warning, SystemConfigWarning::OrphanProviderConsumption { provider_type, .. } if provider_type == "firewall")));
        let fragment = result.fragments.first().unwrap();
        assert_eq!(fragment.data["output"]["exports"]["args"]["providers"]["firewall"], json!([]));
    }

    #[test]
    fn dependent_module_failure_is_chained() {
        let mut boundary = FakeBoundary::default();
        boundary.outputs.insert("a".to_string(), Err("boom".to_string()));
        let inventory: Inventory = serde_json::from_value(json!({
            "machines": { "server1": { "system": "x86_64-linux" } },
            "services": {
                "a": { "instances": [{ "machine": "server1", "role": "default", "tags": [] }] },
                "c": { "instances": [{ "machine": "server1", "role": "default", "tags": [] }] }
            }
        }))
        .unwrap();
        let modules = vec![
            module("a", &["default"], &[], &[], &[]),
            module("c", &["default"], &["a"], &[], &[]),
        ];

        let result = evaluate_modules(&inventory, &modules, &boundary);

        assert!(result.errors.iter().any(|error| matches!(error, SystemConfigError::Eval { message, .. } if message.contains("depends on failed module 'a'"))));
    }

    #[test]
    fn evaluator_is_deterministic_for_same_inputs() {
        let boundary = FakeBoundary::default();
        let inventory = inventory();
        let modules = vec![
            module("a", &["default"], &[], &[], &[]),
            module("b", &["default"], &[], &[], &[]),
        ];

        let left = evaluate_modules(&inventory, &modules, &boundary);
        let right = evaluate_modules(&inventory, &modules, &boundary);

        let left_json: Vec<Value> = left.fragments.iter().map(|fragment| fragment.data.clone()).collect();
        let right_json: Vec<Value> = right.fragments.iter().map(|fragment| fragment.data.clone()).collect();
        assert_eq!(left_json, right_json);
    }
}
