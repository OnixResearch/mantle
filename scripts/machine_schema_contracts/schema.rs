use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde_json::Map;
use serde_json::Value;
use syn::Fields;
use syn::Item;
use syn::Lit;
use syn::Meta;
use syn::Token;
use syn::parse::Parser;
use syn::punctuated::Punctuated;

use super::model::*;

pub fn validate_schema(surface_id: &str, schema: &Value) -> Vec<Issue> {
    let mut issues = Vec::new();
    let Some(root) = schema.as_object() else {
        return vec![Issue::surface(
            surface_id,
            "schema",
            "#",
            "schema root must be an object",
        )];
    };
    if root.get("$schema").and_then(Value::as_str) != Some(JSON_SCHEMA_DRAFT) {
        push_issue(
            &mut issues,
            Issue::surface(surface_id, "schema", "#/$schema", format!("schema draft must be {JSON_SCHEMA_DRAFT}")),
        );
    }
    if root.get("$id").and_then(Value::as_str).is_none() {
        push_issue(&mut issues, Issue::surface(surface_id, "schema", "#/$id", "schema requires a stable $id"));
    }
    validate_schema_node(surface_id, schema, schema, "#", 0, &mut issues);
    issues.sort();
    issues
}

fn validate_schema_node(
    surface_id: &str,
    root: &Value,
    schema: &Value,
    path: &str,
    depth: u32,
    issues: &mut Vec<Issue>,
) {
    if depth > MAX_SCHEMA_DEPTH {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "bounds",
                path,
                format!("schema depth exceeds MAX_SCHEMA_DEPTH={MAX_SCHEMA_DEPTH}"),
            ),
        );
        return;
    }
    let Some(object) = schema.as_object() else {
        push_issue(issues, Issue::surface(surface_id, "schema", path, "schema node must be an object"));
        return;
    };
    validate_schema_keys(surface_id, root, object, path, issues);
    validate_type_declaration(surface_id, object, path, issues);
    validate_keyword_compatibility(surface_id, object, path, issues);
    validate_literal_constraints(surface_id, object, path, issues);
    validate_named_bounds(surface_id, object, path, issues);
    validate_semantic(surface_id, object, path, issues);
    validate_exact_object_policy(surface_id, object, path, issues);
    validate_schema_children(surface_id, root, object, path, depth, issues);
    validate_schema_definitions(surface_id, root, object, path, depth, issues);
    if let Some(invariants) = object.get("x-mantle-invariants") {
        validate_invariants(surface_id, invariants, path, issues);
    }
}

fn validate_schema_keys(
    surface_id: &str,
    root: &Value,
    object: &Map<String, Value>,
    path: &str,
    issues: &mut Vec<Issue>,
) {
    for key in object.keys().filter(|key| !supported_schema_key(key)) {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "schema",
                format!("{path}/{key}"),
                format!("unsupported schema construct {key}"),
            ),
        );
    }
    let Some(reference_value) = object.get("$ref") else {
        return;
    };
    let Some(reference) = reference_value.as_str() else {
        push_issue(issues, Issue::surface(surface_id, "reference", format!("{path}/$ref"), "$ref must be a string"));
        return;
    };
    if object.len() != 1 {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "reference",
                path,
                "$ref siblings are outside the supported exact-rendering subset",
            ),
        );
    }
    if resolve_reference(root, reference).is_none() {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "reference",
                format!("{path}/$ref"),
                format!("unsupported or missing root definition reference {reference}"),
            ),
        );
    }
}

fn validate_schema_children(
    surface_id: &str,
    root: &Value,
    object: &Map<String, Value>,
    path: &str,
    depth: u32,
    issues: &mut Vec<Issue>,
) {
    if let Some(properties) = object.get("properties") {
        let Some(properties) = properties.as_object() else {
            push_issue(
                issues,
                Issue::surface(surface_id, "schema", format!("{path}/properties"), "properties must be an object"),
            );
            return;
        };
        let property_count = u32::try_from(properties.len()).unwrap_or(u32::MAX);
        if property_count > MAX_SCHEMA_PROPERTIES {
            push_issue(
                issues,
                Issue::surface(
                    surface_id,
                    "bounds",
                    format!("{path}/properties"),
                    format!("property count exceeds MAX_SCHEMA_PROPERTIES={MAX_SCHEMA_PROPERTIES}"),
                ),
            );
        }
        validate_required_fields(surface_id, object, properties, path, issues);
        for (name, child) in properties {
            validate_schema_node(
                surface_id,
                root,
                child,
                &format!("{path}/properties/{name}"),
                depth.saturating_add(1),
                issues,
            );
        }
    }
    if let Some(items) = object.get("items") {
        validate_schema_node(surface_id, root, items, &format!("{path}/items"), depth.saturating_add(1), issues);
    }
    if let Some(additional) = object.get("additionalProperties")
        && additional.is_object()
    {
        validate_schema_node(
            surface_id,
            root,
            additional,
            &format!("{path}/additionalProperties"),
            depth.saturating_add(1),
            issues,
        );
    }
}

fn validate_schema_definitions(
    surface_id: &str,
    root: &Value,
    object: &Map<String, Value>,
    path: &str,
    depth: u32,
    issues: &mut Vec<Issue>,
) {
    let Some(value) = object.get("$defs") else {
        return;
    };
    if path != "#" {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "reference",
                format!("{path}/$defs"),
                "nested $defs are outside the supported root-definition subset",
            ),
        );
    }
    let Some(definitions) = value.as_object() else {
        push_issue(issues, Issue::surface(surface_id, "schema", format!("{path}/$defs"), "$defs must be an object"));
        return;
    };
    let definition_count = u32::try_from(definitions.len()).unwrap_or(u32::MAX);
    if definition_count > MAX_CONTRACT_DEFINITIONS {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "bounds",
                format!("{path}/$defs"),
                format!("definition count exceeds MAX_CONTRACT_DEFINITIONS={MAX_CONTRACT_DEFINITIONS}"),
            ),
        );
    }
    for (name, child) in definitions {
        if !valid_definition_name(name) {
            push_issue(
                issues,
                Issue::surface(
                    surface_id,
                    "reference",
                    format!("{path}/$defs/{name}"),
                    "definition names must be lowercase ASCII identifiers",
                ),
            );
        }
        validate_schema_node(surface_id, root, child, &format!("{path}/$defs/{name}"), depth.saturating_add(1), issues);
    }
}

fn supported_schema_key(key: &str) -> bool {
    matches!(
        key,
        "$schema"
            | "$id"
            | "$defs"
            | "$ref"
            | "title"
            | "description"
            | "type"
            | "const"
            | "enum"
            | "required"
            | "properties"
            | "additionalProperties"
            | "items"
            | "minItems"
            | "maxItems"
            | "minLength"
            | "maxLength"
            | "minProperties"
            | "maxProperties"
            | "minimum"
            | "maximum"
            | "x-mantle-bound-name"
            | "x-mantle-length-unit"
            | "x-mantle-semantic"
            | "x-mantle-invariants"
            | "x-mantle-rust-owner"
            | "x-mantle-issue-class"
    )
}

fn validate_type_declaration(surface_id: &str, object: &Map<String, Value>, path: &str, issues: &mut Vec<Issue>) {
    let Some(value) = object.get("type") else {
        if !object.contains_key("$ref") && !object.contains_key("const") && !object.contains_key("enum") {
            push_issue(
                issues,
                Issue::surface(surface_id, "schema", path, "schema node requires type, $ref, const, or enum"),
            );
        }
        return;
    };
    let valid = match value {
        Value::String(kind) => supported_json_type(kind),
        Value::Array(kinds) => valid_nullable_type_declaration(kinds),
        _ => false,
    };
    if !valid {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "schema",
                format!("{path}/type"),
                "type must be one supported scalar or exactly one non-null type plus null",
            ),
        );
    }
}

fn valid_nullable_type_declaration(kinds: &[Value]) -> bool {
    if kinds.len() != 2 {
        return false;
    }
    let parsed = kinds.iter().filter_map(Value::as_str).collect::<Vec<_>>();
    parsed.len() == kinds.len()
        && parsed.iter().all(|kind| supported_json_type(kind))
        && parsed.iter().filter(|kind| **kind == "null").count() == 1
        && parsed.iter().filter(|kind| **kind != "null").count() == 1
}

fn supported_json_type(kind: &str) -> bool {
    matches!(kind, "object" | "array" | "string" | "integer" | "number" | "boolean" | "null")
}

fn validate_keyword_compatibility(surface_id: &str, object: &Map<String, Value>, path: &str, issues: &mut Vec<Issue>) {
    let kind = declared_non_null_type(object);
    reject_misplaced_keywords(
        surface_id,
        object,
        path,
        kind == Some("object"),
        &[
            "required",
            "properties",
            "additionalProperties",
            "minProperties",
            "maxProperties",
            "x-mantle-invariants",
        ],
        issues,
    );
    reject_misplaced_keywords(
        surface_id,
        object,
        path,
        kind == Some("array"),
        &["items", "minItems", "maxItems"],
        issues,
    );
    reject_misplaced_keywords(
        surface_id,
        object,
        path,
        kind == Some("string"),
        &["minLength", "maxLength", "x-mantle-length-unit", "x-mantle-semantic"],
        issues,
    );
    reject_misplaced_keywords(
        surface_id,
        object,
        path,
        matches!(kind, Some("integer" | "number")),
        &["minimum", "maximum"],
        issues,
    );
    validate_root_only_keywords(surface_id, object, path, issues);
    if kind == Some("array") && !object.contains_key("items") {
        push_issue(
            issues,
            Issue::surface(surface_id, "schema", format!("{path}/items"), "array schemas require typed items"),
        );
    }
}

fn reject_misplaced_keywords(
    surface_id: &str,
    object: &Map<String, Value>,
    path: &str,
    accepted: bool,
    keywords: &[&str],
    issues: &mut Vec<Issue>,
) {
    if accepted {
        return;
    }
    for keyword in keywords.iter().filter(|keyword| object.contains_key(**keyword)) {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "schema",
                format!("{path}/{keyword}"),
                format!("keyword {keyword} is incompatible with the declared type"),
            ),
        );
    }
}

fn validate_root_only_keywords(surface_id: &str, object: &Map<String, Value>, path: &str, issues: &mut Vec<Issue>) {
    if path == "#" {
        return;
    }
    for keyword in ["$schema", "$id", "$defs", "x-mantle-rust-owner"]
        .iter()
        .filter(|keyword| object.contains_key(**keyword))
    {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "schema",
                format!("{path}/{keyword}"),
                format!("keyword {keyword} is supported only at the schema root"),
            ),
        );
    }
}

fn validate_literal_constraints(surface_id: &str, object: &Map<String, Value>, path: &str, issues: &mut Vec<Issue>) {
    if object.contains_key("const") && object.contains_key("enum") {
        push_issue(
            issues,
            Issue::surface(surface_id, "schema", path, "const and enum may not be combined in the supported subset"),
        );
    }
    if let Some(expected) = object.get("const")
        && !literal_matches_declared_type(object, expected)
    {
        push_issue(
            issues,
            Issue::surface(surface_id, "schema", format!("{path}/const"), "const violates the declared type"),
        );
    }
    if let Some(raw_enum) = object.get("enum") {
        validate_enum(surface_id, object, raw_enum, path, issues);
    }
    validate_metadata_values(surface_id, object, path, issues);
}

fn validate_enum(surface_id: &str, object: &Map<String, Value>, raw_enum: &Value, path: &str, issues: &mut Vec<Issue>) {
    let Some(values) = raw_enum.as_array() else {
        push_issue(issues, Issue::surface(surface_id, "schema", format!("{path}/enum"), "enum must be an array"));
        return;
    };
    let value_count = u32::try_from(values.len()).unwrap_or(u32::MAX);
    let distinct = values.iter().map(Value::to_string).collect::<BTreeSet<_>>();
    if values.is_empty() || value_count > MAX_ENUM_VALUES || distinct.len() != values.len() {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "bounds",
                format!("{path}/enum"),
                format!("enum must contain 1..={MAX_ENUM_VALUES} distinct values"),
            ),
        );
    }
    if values.iter().any(|value| !literal_matches_declared_type(object, value)) {
        push_issue(
            issues,
            Issue::surface(surface_id, "schema", format!("{path}/enum"), "enum value violates the declared type"),
        );
    }
}

fn validate_metadata_values(surface_id: &str, object: &Map<String, Value>, path: &str, issues: &mut Vec<Issue>) {
    for keyword in ["$schema", "$id", "title", "description", "x-mantle-rust-owner"] {
        if object.get(keyword).is_some_and(|value| value.as_str().is_none_or(str::is_empty)) {
            push_issue(
                issues,
                Issue::surface(surface_id, "schema", format!("{path}/{keyword}"), "metadata must be non-empty text"),
            );
        }
    }
    if object
        .get("x-mantle-issue-class")
        .is_some_and(|value| value.as_str().is_none_or(|class| !EXPECTED_ISSUE_CLASSES.contains(&class)))
    {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "schema",
                format!("{path}/x-mantle-issue-class"),
                "issue class must use the declared deterministic vocabulary",
            ),
        );
    }
}

fn literal_matches_declared_type(object: &Map<String, Value>, value: &Value) -> bool {
    let Some(declared) = object.get("type") else {
        return true;
    };
    match declared {
        Value::String(kind) => literal_matches_type(kind, value),
        Value::Array(kinds) => kinds.iter().filter_map(Value::as_str).any(|kind| literal_matches_type(kind, value)),
        _ => false,
    }
}

fn literal_matches_type(kind: &str, value: &Value) -> bool {
    match kind {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => false,
    }
}

fn validate_named_bounds(surface_id: &str, object: &Map<String, Value>, path: &str, issues: &mut Vec<Issue>) {
    validate_bound_values(surface_id, object, path, issues);
    let bound_names = object.get("x-mantle-bound-name").and_then(Value::as_object);
    for &keyword in BOUND_KEYWORDS {
        if !object.contains_key(keyword) {
            continue;
        }
        let named = bound_names
            .and_then(|names| names.get(keyword))
            .and_then(Value::as_str)
            .is_some_and(valid_bound_name);
        if !named {
            push_issue(
                issues,
                Issue::surface(
                    surface_id,
                    "bounds",
                    format!("{path}/{keyword}"),
                    format!("numeric bound {keyword} requires x-mantle-bound-name"),
                ),
            );
        }
    }
    if let Some(names) = bound_names {
        for (keyword, name) in names {
            let valid_mapping = BOUND_KEYWORDS.contains(&keyword.as_str())
                && object.contains_key(keyword)
                && name.as_str().is_some_and(valid_bound_name);
            if !valid_mapping {
                push_issue(
                    issues,
                    Issue::surface(
                        surface_id,
                        "bounds",
                        format!("{path}/x-mantle-bound-name/{keyword}"),
                        "bound name must map a present bound to a valid Nickel identifier",
                    ),
                );
            }
        }
    }
    let has_string_bound = object.contains_key("minLength") || object.contains_key("maxLength");
    if has_string_bound && object.get("x-mantle-length-unit").and_then(Value::as_str) != Some("utf8-bytes") {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "bounds",
                format!("{path}/x-mantle-length-unit"),
                "string bounds require exact utf8-bytes length semantics",
            ),
        );
    }
}

fn validate_bound_values(surface_id: &str, object: &Map<String, Value>, path: &str, issues: &mut Vec<Issue>) {
    for &keyword in BOUND_KEYWORDS {
        let Some(value) = object.get(keyword) else {
            continue;
        };
        let collection_bound =
            matches!(keyword, "minItems" | "maxItems" | "minLength" | "maxLength" | "minProperties" | "maxProperties");
        let integer_value = value.as_i64().is_some() || value.as_u64().is_some();
        let valid = if collection_bound {
            value.as_u64().is_some()
        } else if declared_non_null_type(object) == Some("integer") {
            integer_value
        } else {
            value.is_number()
        };
        if !valid {
            push_issue(
                issues,
                Issue::surface(
                    surface_id,
                    "bounds",
                    format!("{path}/{keyword}"),
                    "bound must use a compatible finite integer or number",
                ),
            );
        }
    }
    for (minimum, maximum) in [
        ("minItems", "maxItems"),
        ("minLength", "maxLength"),
        ("minProperties", "maxProperties"),
        ("minimum", "maximum"),
    ] {
        if object
            .get(minimum)
            .zip(object.get(maximum))
            .is_some_and(|(lower, upper)| numeric_less_or_equal(lower, upper) == Some(false))
        {
            push_issue(
                issues,
                Issue::surface(surface_id, "bounds", path, format!("{minimum} must not exceed {maximum}")),
            );
        }
    }
}

fn numeric_less_or_equal(left: &Value, right: &Value) -> Option<bool> {
    if let Some(pair) = left.as_i64().zip(right.as_i64()) {
        return Some(pair.0 <= pair.1);
    }
    if let Some(pair) = left.as_u64().zip(right.as_u64()) {
        return Some(pair.0 <= pair.1);
    }
    if left.as_i64().is_some_and(|value| value < 0) && right.as_u64().is_some() {
        return Some(true);
    }
    if left.as_u64().is_some() && right.as_i64().is_some_and(|value| value < 0) {
        return Some(false);
    }
    left.as_f64().zip(right.as_f64()).map(|pair| pair.0 <= pair.1)
}

fn valid_bound_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    value != "C"
        && (first.is_ascii_uppercase() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn validate_semantic(surface_id: &str, object: &Map<String, Value>, path: &str, issues: &mut Vec<Issue>) {
    let Some(value) = object.get("x-mantle-semantic") else {
        return;
    };
    let supported = value.as_str().is_some_and(|semantic| {
        matches!(
            semantic,
            "blake3"
                | "mantle-digest"
                | "sha256"
                | "sha256-digest"
                | "mantle-reference"
                | "safe-reference"
                | "redaction-safe"
                | "non-empty"
        )
    });
    if !supported {
        push_issue(
            issues,
            Issue::surface(surface_id, "schema", format!("{path}/x-mantle-semantic"), "unsupported semantic contract"),
        );
    }
}

fn validate_exact_object_policy(surface_id: &str, object: &Map<String, Value>, path: &str, issues: &mut Vec<Issue>) {
    if declared_non_null_type(object) != Some("object") {
        return;
    }
    let accepted = object
        .get("additionalProperties")
        .is_some_and(|additional| additional == &Value::Bool(false) || additional.is_object());
    if !accepted {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "unknown-field",
                format!("{path}/additionalProperties"),
                "object schemas must be closed records or typed dictionaries",
            ),
        );
    }
}

fn validate_required_fields(
    surface_id: &str,
    object: &Map<String, Value>,
    properties: &Map<String, Value>,
    path: &str,
    issues: &mut Vec<Issue>,
) {
    let required = object.get("required").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut seen = BTreeSet::<String>::new();
    for value in required {
        let Some(field) = value.as_str() else {
            push_issue(
                issues,
                Issue::surface(surface_id, "schema", format!("{path}/required"), "required values must be strings"),
            );
            continue;
        };
        if !properties.contains_key(field) || !seen.insert(field.to_string()) {
            push_issue(
                issues,
                Issue::surface(
                    surface_id,
                    "schema",
                    format!("{path}/required/{field}"),
                    "required field is missing from properties or duplicated",
                ),
            );
        }
    }
}

fn validate_invariants(surface_id: &str, value: &Value, path: &str, issues: &mut Vec<Issue>) {
    let Some(invariants) = value.as_array() else {
        push_issue(
            issues,
            Issue::surface(surface_id, "schema", format!("{path}/x-mantle-invariants"), "invariants must be an array"),
        );
        return;
    };
    let invariant_count = u32::try_from(invariants.len()).unwrap_or(u32::MAX);
    if invariant_count > MAX_INVARIANTS_PER_SCHEMA {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "bounds",
                format!("{path}/x-mantle-invariants"),
                format!("invariant count exceeds MAX_INVARIANTS_PER_SCHEMA={MAX_INVARIANTS_PER_SCHEMA}"),
            ),
        );
    }
    for (index, raw) in invariants.iter().enumerate() {
        let Ok(invariant) = serde_json::from_value::<Invariant>(raw.clone()) else {
            push_issue(
                issues,
                Issue::surface(
                    surface_id,
                    "schema",
                    format!("{path}/x-mantle-invariants/{index}"),
                    "invalid invariant declaration",
                ),
            );
            continue;
        };
        if !supported_invariant_kind(&invariant.kind) {
            push_issue(
                issues,
                Issue::surface(
                    surface_id,
                    "schema",
                    format!("{path}/x-mantle-invariants/{index}/kind"),
                    format!("unsupported invariant kind {}", invariant.kind),
                ),
            );
            continue;
        }
        if !valid_invariant_declaration(&invariant) {
            push_issue(
                issues,
                Issue::surface(
                    surface_id,
                    "schema",
                    format!("{path}/x-mantle-invariants/{index}"),
                    format!("invariant {} is missing required bounded JSON pointers", invariant.kind),
                ),
            );
        }
    }
}

fn valid_invariant_declaration(invariant: &Invariant) -> bool {
    let required = match invariant.kind.as_str() {
        "length-equals" => valid_pointer(&invariant.array, false) && valid_pointer(&invariant.integer, false),
        "sum-equals" => valid_pointer(&invariant.target, false) && valid_pointer_list(&invariant.terms),
        "integer-less-than" => valid_pointer(&invariant.integer, false) && valid_pointer(&invariant.target, false),
        "empty-iff" => valid_pointer(&invariant.array, false) && valid_pointer(&invariant.boolean, false),
        "present-iff" => valid_pointer(&invariant.boolean, false) && valid_pointer_list(&invariant.fields),
        "boolean-not" => valid_pointer(&invariant.boolean, false) && valid_pointer(&invariant.target, false),
        "boolean-or" => valid_pointer(&invariant.target, false) && valid_pointer_list(&invariant.terms),
        "selected-not-in-array" => {
            valid_pointer(&invariant.target, false)
                && valid_pointer(&invariant.array, false)
                && valid_pointer(&invariant.item_field, true)
        }
        "all-item-equals-iff" => {
            valid_pointer(&invariant.array, false)
                && valid_pointer(&invariant.boolean, false)
                && valid_pointer(&invariant.item_field, true)
        }
        "unique-by" => valid_pointer(&invariant.array, false) && valid_pointer(&invariant.item_field, true),
        "disjoint-by" => {
            valid_pointer(&invariant.array, false)
                && valid_pointer(&invariant.other_array, false)
                && valid_pointer(&invariant.item_field, true)
                && valid_pointer(&invariant.other_item_field, true)
        }
        "field-equals-const-when" => {
            valid_pointer(&invariant.boolean, false) && valid_pointer(&invariant.target, false)
        }
        "arrays-empty-iff-enum" => valid_pointer(&invariant.target, false) && valid_pointer_list(&invariant.fields),
        _ => false,
    };
    required && bounded_invariant_paths(invariant)
}

fn valid_pointer_list(paths: &[String]) -> bool {
    !paths.is_empty() && paths.iter().all(|path| valid_pointer(path, false))
}

fn bounded_invariant_paths(invariant: &Invariant) -> bool {
    u32::try_from(invariant.terms.len()).unwrap_or(u32::MAX) <= MAX_SCHEMA_PROPERTIES
        && u32::try_from(invariant.fields.len()).unwrap_or(u32::MAX) <= MAX_SCHEMA_PROPERTIES
}

fn valid_pointer(pointer: &str, allow_empty: bool) -> bool {
    if pointer.is_empty() {
        return allow_empty;
    }
    if !pointer.starts_with('/') {
        return false;
    }
    let bytes = pointer.as_bytes();
    let mut index = 0u32;
    while let Some(byte) = bytes.get(usize::try_from(index).unwrap_or(usize::MAX)) {
        if *byte == b'~' {
            let next = index.checked_add(1).and_then(|next| bytes.get(usize::try_from(next).ok()?));
            if !next.is_some_and(|next| matches!(next, b'0' | b'1')) {
                return false;
            }
            index = index.saturating_add(1);
        }
        index = index.saturating_add(1);
    }
    true
}

pub fn supported_invariant_kind(kind: &str) -> bool {
    matches!(
        kind,
        "length-equals"
            | "sum-equals"
            | "integer-less-than"
            | "empty-iff"
            | "present-iff"
            | "boolean-not"
            | "boolean-or"
            | "selected-not-in-array"
            | "all-item-equals-iff"
            | "unique-by"
            | "disjoint-by"
            | "field-equals-const-when"
            | "arrays-empty-iff-enum"
    )
}

pub fn declared_non_null_type(object: &Map<String, Value>) -> Option<&str> {
    match object.get("type") {
        Some(Value::String(kind)) => Some(kind.as_str()),
        Some(Value::Array(kinds)) => kinds.iter().filter_map(Value::as_str).find(|kind| *kind != "null"),
        _ => None,
    }
}

pub fn resolve_reference<'a>(root: &'a Value, reference: &str) -> Option<&'a Value> {
    let name = reference.strip_prefix("#/$defs/")?;
    if !valid_definition_name(name) {
        return None;
    }
    root.get("$defs")?.get(name)
}

fn valid_definition_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_lowercase() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RustFieldShape {
    wire_name: String,
    required: bool,
    flattened: bool,
    optional: bool,
    json_type: Option<String>,
}

pub fn validate_rust_owner_shape(surface: &Surface, schema: &Value, sources: &BTreeMap<String, String>) -> Vec<Issue> {
    let mut issues = Vec::new();
    let Some((owner_path, owner_type)) = surface.rust_owner.rsplit_once("::") else {
        return vec![Issue::surface(
            &surface.id,
            "schema",
            "/rust_owner",
            "Rust owner must be path::Type",
        )];
    };
    let Some(source) = sources.get(owner_path) else {
        return vec![Issue::surface(
            &surface.id,
            "schema",
            "/rust_owner",
            format!("Rust owner source {owner_path} was not loaded"),
        )];
    };
    let schema_owner = schema.get("x-mantle-rust-owner").and_then(Value::as_str);
    if schema_owner != Some(surface.rust_owner.as_str()) {
        push_issue(
            &mut issues,
            Issue::surface(
                &surface.id,
                "schema",
                "#/x-mantle-rust-owner",
                format!("schema owner {schema_owner:?} does not match {}", surface.rust_owner),
            ),
        );
    }
    let fields = match rust_struct_fields(source, owner_type) {
        Ok(fields) => fields,
        Err(error) => {
            push_issue(
                &mut issues,
                Issue::surface(
                    &surface.id,
                    "schema",
                    "/rust_owner",
                    format!("could not extract Rust struct {owner_type}: {error}"),
                ),
            );
            return issues;
        }
    };
    compare_rust_schema_fields(surface, schema, &fields, &mut issues);
    issues
}

fn compare_rust_schema_fields(surface: &Surface, schema: &Value, fields: &[RustFieldShape], issues: &mut Vec<Issue>) {
    let rust_fields = fields.iter().map(|field| field.wire_name.clone()).collect::<BTreeSet<_>>();
    if rust_fields.len() != fields.len() {
        push_issue(
            issues,
            Issue::surface(&surface.id, "schema", "/rust_owner", "duplicate Rust serialized field name"),
        );
    }
    let rust_required = fields
        .iter()
        .filter(|field| field.required)
        .map(|field| field.wire_name.clone())
        .collect::<BTreeSet<_>>();
    let schema_fields = schema
        .get("properties")
        .and_then(Value::as_object)
        .map(|properties| properties.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();
    let schema_required = schema
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if rust_fields != schema_fields {
        let missing = rust_fields.difference(&schema_fields).cloned().collect::<Vec<_>>();
        let extra = schema_fields.difference(&rust_fields).cloned().collect::<Vec<_>>();
        push_issue(
            issues,
            Issue::surface(
                &surface.id,
                "schema",
                "#/properties",
                format!("Rust/schema field drift: missing_from_schema={missing:?} extra_in_schema={extra:?}"),
            ),
        );
    }
    if rust_required != schema_required {
        push_issue(
            issues,
            Issue::surface(
                &surface.id,
                "schema",
                "#/required",
                format!("Rust/schema required-field drift: Rust={rust_required:?} schema={schema_required:?}"),
            ),
        );
    }
    if fields.iter().any(|field| field.flattened) {
        push_issue(
            issues,
            Issue::surface(&surface.id, "schema", "/rust_owner", "serde(flatten) is unsupported for contracted roots"),
        );
    }
    compare_rust_schema_types(surface, schema, fields, issues);
}

fn compare_rust_schema_types(surface: &Surface, schema: &Value, fields: &[RustFieldShape], issues: &mut Vec<Issue>) {
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return;
    };
    for field in fields {
        let Some(expected) = field.json_type.as_deref() else {
            continue;
        };
        let Some(property) = properties.get(&field.wire_name) else {
            continue;
        };
        let Some((actual, nullable)) = schema_type_shape(schema, property, 0) else {
            push_issue(
                issues,
                Issue::surface(
                    &surface.id,
                    "schema",
                    format!("#/properties/{}", field.wire_name),
                    "schema field has no resolvable declared type",
                ),
            );
            continue;
        };
        if actual != expected || (!field.optional && nullable) || (field.optional && field.required && !nullable) {
            push_issue(
                issues,
                Issue::surface(
                    &surface.id,
                    "schema",
                    format!("#/properties/{}", field.wire_name),
                    format!(
                        "Rust/schema type drift: Rust={expected} option={} required={} schema={actual} nullable={nullable}",
                        field.optional, field.required
                    ),
                ),
            );
        }
    }
}

fn schema_type_shape<'a>(root: &'a Value, schema: &'a Value, depth: u32) -> Option<(&'a str, bool)> {
    if depth > MAX_SCHEMA_DEPTH {
        return None;
    }
    let object = schema.as_object()?;
    if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
        return schema_type_shape(root, resolve_reference(root, reference)?, depth.saturating_add(1));
    }
    let kind = declared_non_null_type(object)?;
    let nullable = object
        .get("type")
        .and_then(Value::as_array)
        .is_some_and(|types| types.iter().any(|value| value.as_str() == Some("null")));
    Some((kind, nullable))
}

fn rust_struct_fields(source: &str, owner_type: &str) -> Result<Vec<RustFieldShape>, String> {
    let syntax = syn::parse_file(source).map_err(|error| format!("parsing owner source: {error}"))?;
    let item = syntax
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(item) if item.ident == owner_type => Some(item),
            _ => None,
        })
        .ok_or_else(|| "exact owner struct declaration is absent".to_string())?;
    reject_unsupported_struct_attributes(&item.attrs)?;
    let Fields::Named(named) = &item.fields else {
        return Err("contracted owner must be a named-field struct".to_string());
    };
    let mut fields = Vec::new();
    for field in &named.named {
        reject_conditional_field_attributes(&field.attrs)?;
        let directives = serde_metas(&field.attrs)?;
        reject_custom_serializers(&directives)?;
        if has_serde_directive(&directives, "skip") || has_serde_directive(&directives, "skip_serializing") {
            continue;
        }
        let name = field.ident.as_ref().ok_or_else(|| "named field lost its identifier".to_string())?.to_string();
        let default_name = name.strip_prefix("r#").unwrap_or(&name).to_string();
        fields.push(RustFieldShape {
            wire_name: serde_serialize_rename(&directives)?.unwrap_or(default_name),
            required: !has_serde_directive(&directives, "skip_serializing_if"),
            flattened: has_serde_directive(&directives, "flatten"),
            optional: rust_type_is_option(&field.ty),
            json_type: rust_json_type(&field.ty, &syntax.items, 0),
        });
    }
    Ok(fields)
}

fn rust_type_is_option(value: &syn::Type) -> bool {
    let syn::Type::Path(path) = value else {
        return false;
    };
    path.path.segments.last().is_some_and(|segment| segment.ident == "Option")
}

fn rust_json_type(value: &syn::Type, items: &[Item], depth: u32) -> Option<String> {
    if depth > MAX_RUST_TYPE_DEPTH {
        return None;
    }
    match value {
        syn::Type::Reference(reference) => rust_json_type(&reference.elem, items, depth.saturating_add(1)),
        syn::Type::Slice(_) | syn::Type::Array(_) => Some("array".to_string()),
        syn::Type::Path(path) => {
            let segment = path.path.segments.last()?;
            let identifier = segment.ident.to_string();
            match identifier.as_str() {
                "Option" | "Box" | "Arc" | "Rc" | "Cow" => {
                    let inner = first_type_argument(&segment.arguments)?;
                    rust_json_type(inner, items, depth.saturating_add(1))
                }
                "Vec" | "VecDeque" | "HashSet" | "BTreeSet" => Some("array".to_string()),
                "HashMap" | "BTreeMap" => Some("object".to_string()),
                "String" | "str" => Some("string".to_string()),
                "bool" => Some("boolean".to_string()),
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => {
                    Some("integer".to_string())
                }
                "f32" | "f64" => Some("number".to_string()),
                _ => declared_rust_json_type(&identifier, items, depth.saturating_add(1)),
            }
        }
        _ => None,
    }
}

fn first_type_argument(arguments: &syn::PathArguments) -> Option<&syn::Type> {
    let syn::PathArguments::AngleBracketed(arguments) = arguments else {
        return None;
    };
    arguments.args.iter().find_map(|argument| match argument {
        syn::GenericArgument::Type(value) => Some(value),
        _ => None,
    })
}

fn declared_rust_json_type(identifier: &str, items: &[Item], depth: u32) -> Option<String> {
    items.iter().find_map(|item| match item {
        Item::Struct(item) if item.ident == identifier => match &item.fields {
            Fields::Named(_) => Some("object".to_string()),
            _ => None,
        },
        Item::Enum(item)
            if item.ident == identifier
                && item.variants.iter().all(|variant| matches!(&variant.fields, Fields::Unit)) =>
        {
            Some("string".to_string())
        }
        Item::Type(item) if item.ident == identifier => rust_json_type(&item.ty, items, depth.saturating_add(1)),
        _ => None,
    })
}

fn serde_metas(attributes: &[syn::Attribute]) -> Result<Vec<Meta>, String> {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let mut metas = Vec::new();
    for attribute in attributes.iter().filter(|attribute| attribute.path().is_ident("serde")) {
        let Meta::List(list) = &attribute.meta else {
            return Err("serde attribute must be a directive list".to_string());
        };
        let parsed =
            parser.parse2(list.tokens.clone()).map_err(|error| format!("parsing serde directives: {error}"))?;
        metas.extend(parsed);
    }
    Ok(metas)
}

fn reject_unsupported_struct_attributes(attributes: &[syn::Attribute]) -> Result<(), String> {
    if !has_serialize_derive(attributes)? {
        return Err("contracted owner must derive Serialize".to_string());
    }
    if attributes.iter().any(|attribute| {
        attribute.path().segments.last().is_some_and(|segment| segment.ident == "skip_serializing_none")
    }) {
        return Err("skip_serializing_none is unsupported for contracted roots".to_string());
    }
    let directives = serde_metas(attributes)?;
    const UNSUPPORTED: &[&str] = &[
        "rename_all",
        "rename_all_fields",
        "tag",
        "content",
        "untagged",
        "transparent",
        "remote",
        "from",
        "try_from",
        "into",
    ];
    if let Some(name) = UNSUPPORTED.iter().find(|name| has_serde_directive(&directives, name)) {
        return Err(format!("serde({name}) is unsupported for contracted roots"));
    }
    Ok(())
}

fn has_serialize_derive(attributes: &[syn::Attribute]) -> Result<bool, String> {
    let parser = Punctuated::<syn::Path, Token![,]>::parse_terminated;
    for attribute in attributes.iter().filter(|attribute| attribute.path().is_ident("derive")) {
        let Meta::List(list) = &attribute.meta else {
            return Err("derive attribute must be a path list".to_string());
        };
        let paths =
            parser.parse2(list.tokens.clone()).map_err(|error| format!("parsing derive attributes: {error}"))?;
        if paths.iter().any(|path| path.segments.last().is_some_and(|segment| segment.ident == "Serialize")) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn reject_conditional_field_attributes(attributes: &[syn::Attribute]) -> Result<(), String> {
    if attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr"))
    {
        return Err("conditional fields are unsupported for contracted roots".to_string());
    }
    Ok(())
}

fn reject_custom_serializers(directives: &[Meta]) -> Result<(), String> {
    const UNSUPPORTED: &[&str] = &["with", "serialize_with", "getter"];
    if let Some(name) = UNSUPPORTED.iter().find(|name| has_serde_directive(directives, name)) {
        return Err(format!("serde({name}) is unsupported for contracted root fields"));
    }
    Ok(())
}

fn has_serde_directive(directives: &[Meta], expected: &str) -> bool {
    directives.iter().any(|directive| meta_path(directive).is_ident(expected))
}

fn meta_path(meta: &Meta) -> &syn::Path {
    match meta {
        Meta::Path(path) => path,
        Meta::List(list) => &list.path,
        Meta::NameValue(value) => &value.path,
    }
}

fn serde_serialize_rename(directives: &[Meta]) -> Result<Option<String>, String> {
    let Some(rename) = directives.iter().find(|directive| meta_path(directive).is_ident("rename")) else {
        return Ok(None);
    };
    match rename {
        Meta::NameValue(value) => literal_string(&value.value).map(Some),
        Meta::List(list) => {
            let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
            let nested = parser
                .parse2(list.tokens.clone())
                .map_err(|error| format!("parsing serde rename directive: {error}"))?;
            nested
                .iter()
                .find_map(|meta| match meta {
                    Meta::NameValue(value) if value.path.is_ident("serialize") => Some(literal_string(&value.value)),
                    _ => None,
                })
                .transpose()
        }
        Meta::Path(_) => Err("serde(rename) requires a serialization name".to_string()),
    }
}

fn literal_string(expression: &syn::Expr) -> Result<String, String> {
    match expression {
        syn::Expr::Lit(literal) => match &literal.lit {
            Lit::Str(value) if !value.value().is_empty() => Ok(value.value()),
            _ => Err("serde wire name must be a non-empty string".to_string()),
        },
        _ => Err("serde wire name must be a string literal".to_string()),
    }
}
