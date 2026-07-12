use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde_json::Map;
use serde_json::Value;

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
    if let Some(reference) = object.get("$ref").and_then(Value::as_str)
        && resolve_reference(root, reference).is_none()
    {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                "reference",
                format!("{path}/$ref"),
                format!("unsupported or missing local reference {reference}"),
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
        Value::Array(kinds) => {
            !kinds.is_empty()
                && kinds.iter().all(|kind| kind.as_str().is_some_and(supported_json_type))
                && kinds.iter().filter(|kind| kind.as_str() == Some("null")).count() == 1
        }
        _ => false,
    };
    if !valid {
        push_issue(
            issues,
            Issue::surface(surface_id, "schema", format!("{path}/type"), "unsupported type declaration"),
        );
    }
}

fn supported_json_type(kind: &str) -> bool {
    matches!(kind, "object" | "array" | "string" | "integer" | "number" | "boolean" | "null")
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
        if object.get(keyword).is_some_and(|value| !value.is_number()) {
            push_issue(
                issues,
                Issue::surface(surface_id, "bounds", format!("{path}/{keyword}"), "bound value must be numeric"),
            );
        }
    }
    for (minimum, maximum) in [
        ("minItems", "maxItems"),
        ("minLength", "maxLength"),
        ("minProperties", "maxProperties"),
        ("minimum", "maximum"),
    ] {
        let ordered = object
            .get(minimum)
            .and_then(Value::as_f64)
            .zip(object.get(maximum).and_then(Value::as_f64))
            .is_none_or(|(lower, upper)| lower <= upper);
        if !ordered {
            push_issue(
                issues,
                Issue::surface(surface_id, "bounds", path, format!("{minimum} must not exceed {maximum}")),
            );
        }
    }
}

fn valid_bound_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == b'_') && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn validate_semantic(surface_id: &str, object: &Map<String, Value>, path: &str, issues: &mut Vec<Issue>) {
    let Some(value) = object.get("x-mantle-semantic") else {
        return;
    };
    let supported = value.as_str().is_some_and(|semantic| {
        matches!(semantic, "blake3" | "mantle-digest" | "sha256" | "safe-reference" | "redaction-safe" | "non-empty")
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
        }
    }
}

pub fn supported_invariant_kind(kind: &str) -> bool {
    matches!(
        kind,
        "length-equals"
            | "sum-equals"
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
    root.pointer(reference.strip_prefix('#')?)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RustFieldShape {
    wire_name: String,
    required: bool,
    flattened: bool,
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
    let Some(fields) = rust_struct_fields(source, owner_type) else {
        push_issue(
            &mut issues,
            Issue::surface(&surface.id, "schema", "/rust_owner", format!("could not extract Rust struct {owner_type}")),
        );
        return issues;
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
}

fn rust_struct_fields(source: &str, owner_type: &str) -> Option<Vec<RustFieldShape>> {
    let struct_start = u32::try_from(source.find(&format!("struct {owner_type}"))?).ok()?;
    let struct_start_index = struct_start.try_into().ok()?;
    let relative_open = u32::try_from(source[struct_start_index..].find('{')?).ok()?;
    let open = struct_start.checked_add(relative_open)?;
    let close = matching_brace(source, open)?;
    let body_start = open.checked_add(1)?.try_into().ok()?;
    let body_end = close.try_into().ok()?;
    let body = &source[body_start..body_end];
    let mut fields = Vec::new();
    for segment in split_top_level_fields(body) {
        let trimmed = strip_rust_attributes(segment.trim());
        let trimmed =
            trimmed.strip_prefix("pub(crate)").or_else(|| trimmed.strip_prefix("pub")).unwrap_or(trimmed).trim();
        let Some((name, _)) = trimmed.split_once(':') else {
            continue;
        };
        let name = name.trim();
        if valid_rust_field_name(name) && !serde_has_directive(segment, "skip_serializing") {
            fields.push(RustFieldShape {
                wire_name: serde_field_rename(segment).unwrap_or_else(|| name.to_string()),
                required: !serde_has_directive(segment, "skip_serializing_if"),
                flattened: serde_has_directive(segment, "flatten"),
            });
        }
    }
    Some(fields)
}

fn serde_has_directive(source: &str, expected: &str) -> bool {
    serde_directives(source).any(|directive| {
        directive == expected || directive.strip_prefix(expected).is_some_and(|rest| rest.trim_start().starts_with('='))
    })
}

fn serde_field_rename(source: &str) -> Option<String> {
    serde_directives(source).find_map(|directive| {
        let rest = directive.strip_prefix("rename")?.trim_start();
        let quoted = rest.strip_prefix('=')?.trim().strip_prefix('"')?;
        let end = quoted.find('"')?;
        Some(quoted[..end].to_string())
    })
}

fn serde_directives(source: &str) -> impl Iterator<Item = &str> {
    source
        .split("#[serde(")
        .skip(1)
        .filter_map(|rest| rest.split_once(")]"))
        .flat_map(|(directives, _)| directives.split(','))
        .map(str::trim)
}

fn matching_brace(source: &str, open: u32) -> Option<u32> {
    let mut depth = 0u32;
    let open_index = open.try_into().ok()?;
    for (offset, byte) in source.as_bytes()[open_index..].iter().copied().enumerate() {
        if byte == b'{' {
            depth = depth.saturating_add(1);
        } else if byte == b'}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return open.checked_add(u32::try_from(offset).ok()?);
            }
        }
    }
    None
}

fn split_top_level_fields(body: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut start = 0u32;
    let mut angle = 0u32;
    let mut round = 0u32;
    let mut square = 0u32;
    let mut curly = 0u32;
    for (index, byte) in body.bytes().enumerate() {
        let index = u32::try_from(index).expect("bounded Rust owner source index");
        match byte {
            b'<' => angle = angle.saturating_add(1),
            b'>' => angle = angle.saturating_sub(1),
            b'(' => round = round.saturating_add(1),
            b')' => round = round.saturating_sub(1),
            b'[' => square = square.saturating_add(1),
            b']' => square = square.saturating_sub(1),
            b'{' => curly = curly.saturating_add(1),
            b'}' => curly = curly.saturating_sub(1),
            b',' if angle == 0 && round == 0 && square == 0 && curly == 0 => {
                let start_index = start.try_into().expect("bounded Rust field start");
                let end_index = index.try_into().expect("bounded Rust field end");
                fields.push(&body[start_index..end_index]);
                start = index.saturating_add(1);
            }
            _ => {}
        }
    }
    let start_index = start.try_into().expect("bounded trailing Rust field start");
    fields.push(&body[start_index..]);
    fields
}

fn strip_rust_attributes(mut value: &str) -> &str {
    loop {
        let trimmed = value.trim_start();
        if !trimmed.starts_with('#') {
            return trimmed;
        }
        let Some(end) = trimmed.find(']') else {
            return trimmed;
        };
        value = &trimmed[end.saturating_add(1)..];
    }
}

fn valid_rust_field_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == b'_') && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
