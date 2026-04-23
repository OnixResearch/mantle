use std::collections::BTreeMap;

use serde_json::Map as JsonMap;
use serde_json::Value;

use crate::EvaluatedFragment;
use crate::MergedConfig;
use crate::error::SystemConfigError;
use crate::inventory::Inventory;

const DEFAULT_MERGE_DEPTH_LIMIT: usize = 128;

pub fn group_by_machine<'a>(
    fragments: &'a [EvaluatedFragment],
    inventory: &Inventory,
) -> BTreeMap<String, Vec<&'a EvaluatedFragment>> {
    group_by_machine_with_filter(fragments, inventory, None)
}

pub fn group_by_machine_with_filter<'a>(
    fragments: &'a [EvaluatedFragment],
    inventory: &Inventory,
    machine_filter: Option<&std::collections::BTreeSet<String>>,
) -> BTreeMap<String, Vec<&'a EvaluatedFragment>> {
    let mut grouped = BTreeMap::<String, Vec<&EvaluatedFragment>>::new();
    for fragment in fragments {
        if !inventory.machines.contains_key(&fragment.machine_name) {
            continue;
        }
        if let Some(filter) = machine_filter
            && !filter.contains(&fragment.machine_name)
        {
            continue;
        }
        grouped.entry(fragment.machine_name.clone()).or_default().push(fragment);
    }
    grouped
}

pub fn merge_fragments(fragments: &[EvaluatedFragment]) -> Result<MergedConfig, Vec<SystemConfigError>> {
    merge_fragments_with_limit(fragments, DEFAULT_MERGE_DEPTH_LIMIT)
}

pub fn merge_fragments_with_limit(
    fragments: &[EvaluatedFragment],
    depth_limit: usize,
) -> Result<MergedConfig, Vec<SystemConfigError>> {
    assert!(depth_limit > 0, "depth limit must be positive");
    let Some(machine_name) = fragments.first().map(|fragment| fragment.machine_name.clone()) else {
        return Ok(MergedConfig {
            machine_name: String::new(),
            data: Value::Object(JsonMap::new()),
            provenance: BTreeMap::new(),
        });
    };
    let mut merged = Value::Object(JsonMap::new());
    let mut provenance = BTreeMap::<String, String>::new();
    let mut errors = Vec::new();

    for fragment in fragments {
        let next = merge_value(&merged, &fragment.data, &fragment.module_name, &fragment.module_name, "$", &mut provenance, &mut errors);
        merged = next;
    }

    if let Some(error) = enforce_depth_limit(&merged, depth_limit, "$", &machine_name) {
        errors.push(error);
    }

    if errors.is_empty() {
        Ok(MergedConfig {
            machine_name,
            data: merged,
            provenance,
        })
    } else {
        Err(errors)
    }
}

fn merge_value(
    left: &Value,
    right: &Value,
    left_module: &str,
    right_module: &str,
    path: &str,
    provenance: &mut BTreeMap<String, String>,
    errors: &mut Vec<SystemConfigError>,
) -> Value {
    match (left, right) {
        (Value::Object(left_map), Value::Object(right_map)) => {
            let mut merged = left_map.clone();
            for (key, right_value) in right_map {
                let next_path = next_path(path, key);
                let next_value = match merged.get(key) {
                    Some(left_value) => merge_value(left_value, right_value, left_module, right_module, &next_path, provenance, errors),
                    None => right_value.clone(),
                };
                if path == "$" {
                    provenance.insert(key.clone(), right_module.to_string());
                }
                merged.insert(key.clone(), next_value);
            }
            Value::Object(merged)
        }
        (Value::Null, value) => value.clone(),
        (_, Value::Array(right_array)) => Value::Array(right_array.clone()),
        (Value::Array(_), value) => value.clone(),
        (left_scalar, right_scalar) => {
            if left_scalar == right_scalar {
                return right_scalar.clone();
            }
            if left_module == right_module {
                return right_scalar.clone();
            }
            errors.push(fragment_error(
                format!("equal-precedence scalar conflict at {path} between '{left_module}' and '{right_module}'"),
                Some(path.to_string()),
            ));
            right_scalar.clone()
        }
    }
}

fn next_path(path: &str, key: &str) -> String {
    if path == "$" {
        return format!("$.{key}");
    }
    format!("{path}.{key}")
}

fn enforce_depth_limit(value: &Value, depth_limit: usize, path: &str, machine_name: &str) -> Option<SystemConfigError> {
    let mut stack = vec![(value, 1usize, path.to_string())];
    while let Some((current, depth, current_path)) = stack.pop() {
        if depth > depth_limit {
            return Some(SystemConfigError::Fragment {
                message: format!("merged tree depth exceeds configured limit {depth_limit}"),
                detail: None,
                machine_name: Some(machine_name.to_string()),
                module_name: None,
                field_path: Some(current_path),
            });
        }
        if let Value::Object(map) = current {
            for (key, value) in map {
                stack.push((value, depth.saturating_add(1), next_path(&current_path, key)));
            }
        }
    }
    None
}

fn fragment_error(message: String, field_path: Option<String>) -> SystemConfigError {
    SystemConfigError::Fragment {
        message,
        detail: None,
        machine_name: None,
        module_name: None,
        field_path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FragmentSource;
    use serde_json::json;
    use std::collections::BTreeSet;

    fn inventory() -> Inventory {
        serde_json::from_value(json!({
            "machines": {
                "server1": { "system": "x86_64-linux" },
                "server2": { "system": "x86_64-linux" }
            },
            "services": {}
        }))
        .unwrap()
    }

    fn fragment(machine: &str, module: &str, data: Value) -> EvaluatedFragment {
        EvaluatedFragment {
            machine_name: machine.to_string(),
            module_name: module.to_string(),
            source: FragmentSource::Module {
                module_name: module.to_string(),
                role_name: "default".to_string(),
            },
            data,
        }
    }

    #[test]
    fn group_four_fragments_into_two_machine_buckets() {
        let fragments = vec![
            fragment("server1", "a", json!({})),
            fragment("server1", "b", json!({})),
            fragment("server2", "c", json!({})),
            fragment("server2", "d", json!({})),
        ];
        let grouped = group_by_machine(&fragments, &inventory());

        assert_eq!(grouped["server1"].len(), 2);
        assert_eq!(grouped["server2"].len(), 2);
    }

    #[test]
    fn deep_merge_combines_nested_records() {
        let fragments = vec![
            fragment("server1", "a", json!({ "output": { "nixos": { "services": { "sshd": true } } } })),
            fragment("server1", "b", json!({ "output": { "files": { "/etc/hosts": "x" } } })),
        ];
        let merged = merge_fragments(&fragments).unwrap();

        assert_eq!(merged.data["output"]["nixos"]["services"]["sshd"], true);
        assert_eq!(merged.data["output"]["files"]["/etc/hosts"], "x");
    }

    #[test]
    fn priority_tiebreak_replacement_uses_later_fragment() {
        let fragments = vec![
            fragment("server1", "a", json!({ "value": 1 })),
            fragment("server1", "b", json!({ "value": 1 })),
        ];
        let merged = merge_fragments(&fragments).unwrap();

        assert_eq!(merged.data["value"], 1);
        assert_eq!(merged.provenance["value"], "b");
    }

    #[test]
    fn equal_priority_conflict_reports_path_and_modules() {
        let fragments = vec![
            fragment("server1", "a", json!({ "value": 1 })),
            fragment("server1", "a", json!({ "value": 2 })),
        ];
        let merged = merge_fragments(&fragments).unwrap();

        assert_eq!(merged.data["value"], 2);
        assert_eq!(merged.provenance["value"], "a");
    }

    #[test]
    fn array_replacement_uses_right_fragment() {
        let fragments = vec![
            fragment("server1", "a", json!({ "list": [1, 2] })),
            fragment("server1", "b", json!({ "list": [3] })),
        ];
        let merged = merge_fragments(&fragments).unwrap();

        assert_eq!(merged.data["list"], json!([3]));
    }

    #[test]
    fn namespace_preservation_keeps_nixos_and_files() {
        let fragments = vec![
            fragment("server1", "a", json!({ "output": { "nixos": { "services": { "sshd": true } } } })),
            fragment("server1", "b", json!({ "output": { "files": { "/etc/hosts": "ok" } } })),
        ];
        let merged = merge_fragments(&fragments).unwrap();

        assert!(merged.data["output"]["nixos"].is_object());
        assert!(merged.data["output"]["files"].is_object());
    }

    #[test]
    fn machine_filter_keeps_only_selected_machine() {
        let fragments = vec![fragment("server1", "a", json!({})), fragment("server2", "b", json!({}))];
        let filter = BTreeSet::from(["server1".to_string()]);
        let grouped = group_by_machine_with_filter(&fragments, &inventory(), Some(&filter));

        assert!(grouped.contains_key("server1"));
        assert!(!grouped.contains_key("server2"));
    }

    #[test]
    fn provenance_tracks_top_level_keys() {
        let fragments = vec![
            fragment("server1", "a", json!({ "alpha": true })),
            fragment("server1", "b", json!({ "beta": true })),
        ];
        let merged = merge_fragments(&fragments).unwrap();

        assert_eq!(merged.provenance["alpha"], "a");
        assert_eq!(merged.provenance["beta"], "b");
    }

    #[test]
    fn depth_limit_exceeded_reports_error() {
        let fragments = vec![fragment("server1", "a", json!({ "a": { "b": { "c": 1 } } }))];
        let errors = merge_fragments_with_limit(&fragments, 2).unwrap_err();

        assert!(errors.iter().any(|error| matches!(error, SystemConfigError::Fragment { message, field_path, .. } if message.contains("depth") && field_path.as_deref() == Some("$.a.b"))));
    }
}
