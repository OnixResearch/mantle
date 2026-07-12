use std::collections::BTreeSet;

use serde_json::Map;
use serde_json::Value;

use super::model::*;
use super::registry::is_blake3_hex;
use super::schema::declared_non_null_type;
use super::schema::resolve_reference;

pub fn validate_instance(surface_id: &str, schema: &Value, value: &Value) -> Vec<Issue> {
    let mut issues = Vec::new();
    validate_instance_node(surface_id, schema, schema, value, "", 0, &mut issues);
    issues.sort();
    issues
}

fn validate_instance_node(
    surface_id: &str,
    root: &Value,
    schema: &Value,
    value: &Value,
    path: &str,
    depth: u32,
    issues: &mut Vec<Issue>,
) {
    let issue_count = u32::try_from(issues.len()).unwrap_or(u32::MAX);
    if issue_count >= MAX_VALIDATION_ISSUES || depth > MAX_SCHEMA_DEPTH {
        return;
    }
    let Some(object) = schema.as_object() else {
        push_issue(issues, Issue::surface(surface_id, "schema", display_path(path), "schema node is not an object"));
        return;
    };
    if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
        let Some(resolved) = resolve_reference(root, reference) else {
            push_issue(
                issues,
                Issue::surface(surface_id, "reference", display_path(path), "schema reference is unresolved"),
            );
            return;
        };
        validate_instance_node(surface_id, root, resolved, value, path, depth.saturating_add(1), issues);
        return;
    }
    if !validate_instance_constraints(surface_id, object, value, path, issues) {
        return;
    }
    if value.is_null() {
        return;
    }
    match declared_non_null_type(object) {
        Some("object") => validate_object_instance(surface_id, root, object, value, path, depth, issues),
        Some("array") => validate_array_instance(surface_id, root, object, value, path, depth, issues),
        Some("string") => validate_string_instance(surface_id, object, value, path, issues),
        Some("integer") => validate_integer_instance(surface_id, object, value, path, issues),
        Some("number") => validate_number_instance(surface_id, object, value, path, issues),
        Some("boolean") | Some("null") | None => {}
        Some(other) => push_issue(
            issues,
            Issue::surface(surface_id, "schema", display_path(path), format!("unsupported declared type {other}")),
        ),
    }
    validate_instance_invariants(surface_id, object, value, path, issues);
}

fn validate_instance_constraints(
    surface_id: &str,
    object: &Map<String, Value>,
    value: &Value,
    path: &str,
    issues: &mut Vec<Issue>,
) -> bool {
    if let Some(expected) = object.get("const")
        && value != expected
    {
        push_issue(
            issues,
            Issue::surface(
                surface_id,
                object_issue_class(object, "version"),
                display_path(path),
                format!("expected exact value {expected}"),
            ),
        );
        return false;
    }
    if let Some(allowed) = object.get("enum").and_then(Value::as_array)
        && !allowed.iter().any(|candidate| candidate == value)
    {
        push_issue(issues, Issue::surface(surface_id, "enum", display_path(path), "value is not in the closed enum"));
        return false;
    }
    if value_matches_declared_type(object, value) {
        return true;
    }
    push_issue(issues, Issue::surface(surface_id, "schema", display_path(path), "value has the wrong JSON type"));
    false
}

fn validate_instance_invariants(
    surface_id: &str,
    object: &Map<String, Value>,
    value: &Value,
    path: &str,
    issues: &mut Vec<Issue>,
) {
    let Some(invariants) = object.get("x-mantle-invariants").and_then(Value::as_array) else {
        return;
    };
    for raw in invariants {
        if let Ok(invariant) = serde_json::from_value::<Invariant>(raw.clone())
            && !evaluate_invariant(&invariant, value)
        {
            push_issue(
                issues,
                Issue::surface(
                    surface_id,
                    "cross-field",
                    display_path(path),
                    format!("cross-field invariant {} failed", invariant.kind),
                ),
            );
        }
    }
}

fn value_matches_declared_type(object: &Map<String, Value>, value: &Value) -> bool {
    let Some(declared) = object.get("type") else {
        return true;
    };
    match declared {
        Value::String(kind) => value_matches_type(kind, value),
        Value::Array(kinds) => kinds.iter().filter_map(Value::as_str).any(|kind| value_matches_type(kind, value)),
        _ => false,
    }
}

fn value_matches_type(kind: &str, value: &Value) -> bool {
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

fn validate_object_instance(
    surface_id: &str,
    root: &Value,
    schema: &Map<String, Value>,
    value: &Value,
    path: &str,
    depth: u32,
    issues: &mut Vec<Issue>,
) {
    let object = value.as_object().expect("type checked object");
    let property_count = u64::try_from(object.len()).unwrap_or(u64::MAX);
    validate_collection_bounds(surface_id, schema, property_count, path, "Properties", issues);
    let properties = schema.get("properties").and_then(Value::as_object);
    let required = schema.get("required").and_then(Value::as_array).cloned().unwrap_or_default();
    for required_name in required.iter().filter_map(Value::as_str) {
        if !object.contains_key(required_name) {
            push_issue(
                issues,
                Issue::surface(
                    surface_id,
                    "schema",
                    child_path(path, required_name),
                    format!("missing required field {required_name}"),
                ),
            );
        }
    }
    for (name, child_value) in object {
        if let Some(child_schema) = properties.and_then(|properties| properties.get(name)) {
            validate_instance_node(
                surface_id,
                root,
                child_schema,
                child_value,
                &child_path(path, name),
                depth.saturating_add(1),
                issues,
            );
            continue;
        }
        match schema.get("additionalProperties") {
            Some(Value::Object(_)) => validate_instance_node(
                surface_id,
                root,
                schema.get("additionalProperties").expect("typed dictionary schema"),
                child_value,
                &child_path(path, name),
                depth.saturating_add(1),
                issues,
            ),
            _ => push_issue(
                issues,
                Issue::surface(surface_id, "unknown-field", child_path(path, name), format!("unexpected field {name}")),
            ),
        }
    }
}

fn validate_array_instance(
    surface_id: &str,
    root: &Value,
    schema: &Map<String, Value>,
    value: &Value,
    path: &str,
    depth: u32,
    issues: &mut Vec<Issue>,
) {
    let array = value.as_array().expect("type checked array");
    let item_count = u64::try_from(array.len()).unwrap_or(u64::MAX);
    validate_collection_bounds(surface_id, schema, item_count, path, "Items", issues);
    if let Some(item_schema) = schema.get("items") {
        for (index, item) in array.iter().enumerate() {
            validate_instance_node(
                surface_id,
                root,
                item_schema,
                item,
                &child_path(path, &index.to_string()),
                depth.saturating_add(1),
                issues,
            );
        }
    }
}

fn validate_string_instance(
    surface_id: &str,
    schema: &Map<String, Value>,
    value: &Value,
    path: &str,
    issues: &mut Vec<Issue>,
) {
    let text = value.as_str().expect("type checked string");
    let byte_count = u64::try_from(text.len()).unwrap_or(u64::MAX);
    validate_numeric_range(surface_id, schema, byte_count, path, "Length", issues);
    let Some(semantic) = schema.get("x-mantle-semantic").and_then(Value::as_str) else {
        return;
    };
    let valid = match semantic {
        "blake3" => is_blake3_hex(text),
        "mantle-digest" => is_mantle_digest(text),
        "sha256" => is_lower_hex(text, BLAKE3_HEX_LENGTH_CHARS),
        "sha256-digest" => text.strip_prefix("sha256:").is_some_and(is_blake3_hex),
        "mantle-reference" => text.strip_prefix("mantle://blake3/").is_some_and(is_blake3_hex),
        "safe-reference" => is_safe_reference(text),
        "redaction-safe" => is_redaction_safe(text),
        "non-empty" => !text.is_empty(),
        _ => false,
    };
    if valid {
        return;
    }
    let class = match semantic {
        "blake3" | "mantle-digest" | "sha256" | "sha256-digest" => "digest",
        "mantle-reference" | "safe-reference" => "reference",
        "redaction-safe" => "redaction",
        _ => "schema",
    };
    push_issue(
        issues,
        Issue::surface(surface_id, class, display_path(path), format!("string violates semantic contract {semantic}")),
    );
}

fn validate_integer_instance(
    surface_id: &str,
    schema: &Map<String, Value>,
    value: &Value,
    path: &str,
    issues: &mut Vec<Issue>,
) {
    if let Some(number) = value.as_i64() {
        if let Some(minimum) = schema.get("minimum").and_then(Value::as_i64)
            && number < minimum
        {
            push_issue(
                issues,
                Issue::surface(surface_id, "bounds", display_path(path), format!("value {number} is below {minimum}")),
            );
        }
        if let Some(maximum) = schema.get("maximum").and_then(Value::as_i64)
            && number > maximum
        {
            push_issue(
                issues,
                Issue::surface(surface_id, "bounds", display_path(path), format!("value {number} exceeds {maximum}")),
            );
        }
        return;
    }
    if let Some(number) = value.as_u64() {
        validate_numeric_range(surface_id, schema, number, path, "", issues);
    }
}

fn validate_number_instance(
    surface_id: &str,
    schema: &Map<String, Value>,
    value: &Value,
    path: &str,
    issues: &mut Vec<Issue>,
) {
    let Some(number) = value.as_f64() else {
        return;
    };
    if let Some(minimum) = schema.get("minimum").and_then(Value::as_f64)
        && number < minimum
    {
        push_issue(
            issues,
            Issue::surface(surface_id, "bounds", display_path(path), format!("number is below {minimum}")),
        );
    }
    if let Some(maximum) = schema.get("maximum").and_then(Value::as_f64)
        && number > maximum
    {
        push_issue(
            issues,
            Issue::surface(surface_id, "bounds", display_path(path), format!("number exceeds {maximum}")),
        );
    }
}

fn validate_collection_bounds(
    surface_id: &str,
    schema: &Map<String, Value>,
    count: u64,
    path: &str,
    suffix: &str,
    issues: &mut Vec<Issue>,
) {
    validate_numeric_range(surface_id, schema, count, path, suffix, issues);
}

fn validate_numeric_range(
    surface_id: &str,
    schema: &Map<String, Value>,
    number: u64,
    path: &str,
    suffix: &str,
    issues: &mut Vec<Issue>,
) {
    let minimum_key = if suffix.is_empty() {
        "minimum".to_string()
    } else {
        format!("min{suffix}")
    };
    let maximum_key = if suffix.is_empty() {
        "maximum".to_string()
    } else {
        format!("max{suffix}")
    };
    if let Some(minimum) = schema.get(&minimum_key).and_then(Value::as_u64)
        && number < minimum
    {
        push_issue(
            issues,
            Issue::surface(surface_id, "bounds", display_path(path), format!("value {number} is below {minimum}")),
        );
    }
    if let Some(maximum) = schema.get(&maximum_key).and_then(Value::as_u64)
        && number > maximum
    {
        push_issue(
            issues,
            Issue::surface(surface_id, "bounds", display_path(path), format!("value {number} exceeds {maximum}")),
        );
    }
}

fn object_issue_class<'a>(object: &'a Map<String, Value>, fallback: &'a str) -> &'a str {
    object.get("x-mantle-issue-class").and_then(Value::as_str).unwrap_or(fallback)
}

fn display_path(path: &str) -> String {
    if path.is_empty() {
        "/".to_string()
    } else {
        path.to_string()
    }
}

fn child_path(path: &str, child: &str) -> String {
    let escaped = child.replace('~', "~0").replace('/', "~1");
    if path.is_empty() {
        format!("/{escaped}")
    } else {
        format!("{path}/{escaped}")
    }
}

fn evaluate_invariant(invariant: &Invariant, value: &Value) -> bool {
    match invariant.kind.as_str() {
        "length-equals" => invariant_length_equals(invariant, value),
        "sum-equals" => invariant_sum_equals(invariant, value),
        "integer-less-than" => invariant_integer_less_than(invariant, value),
        "field-equals" => invariant_field_equals(invariant, value),
        "empty-iff" => invariant_empty_iff(invariant, value),
        "present-iff" => invariant_present_iff(invariant, value),
        "boolean-not" => invariant_boolean_not(invariant, value),
        "boolean-or" => invariant_boolean_or(invariant, value),
        "selected-not-in-array" => invariant_selected_not_in_array(invariant, value),
        "all-item-equals-iff" => invariant_all_item_equals_iff(invariant, value),
        "unique-by" => invariant_unique_by(invariant, value),
        "disjoint-by" => invariant_disjoint_by(invariant, value),
        "field-equals-const-when" => invariant_field_equals_const_when(invariant, value),
        "arrays-empty-iff-enum" => invariant_arrays_empty_iff_enum(invariant, value),
        _ => false,
    }
}

fn invariant_length_equals(invariant: &Invariant, value: &Value) -> bool {
    pointer(value, &invariant.array)
        .and_then(Value::as_array)
        .zip(pointer(value, &invariant.integer).and_then(Value::as_u64))
        .is_some_and(|(array, expected)| u64::try_from(array.len()).is_ok_and(|length| length == expected))
}

fn invariant_sum_equals(invariant: &Invariant, value: &Value) -> bool {
    let Some(target) = pointer(value, &invariant.target).and_then(Value::as_u64) else {
        return false;
    };
    invariant
        .terms
        .iter()
        .try_fold(0u64, |sum, path| {
            let term = pointer(value, path).and_then(Value::as_u64)?;
            sum.checked_add(term)
        })
        .is_some_and(|sum| sum == target)
}

fn invariant_integer_less_than(invariant: &Invariant, value: &Value) -> bool {
    pointer(value, &invariant.integer)
        .and_then(Value::as_u64)
        .zip(pointer(value, &invariant.target).and_then(Value::as_u64))
        .is_some_and(|(left, right)| left < right)
}

fn invariant_field_equals(invariant: &Invariant, value: &Value) -> bool {
    pointer(value, &invariant.left)
        .zip(pointer(value, &invariant.right))
        .is_some_and(|(left, right)| left == right)
}

fn invariant_empty_iff(invariant: &Invariant, value: &Value) -> bool {
    pointer(value, &invariant.array)
        .and_then(Value::as_array)
        .zip(pointer(value, &invariant.boolean).and_then(Value::as_bool))
        .is_some_and(|(array, expected_empty)| array.is_empty() == expected_empty)
}

fn invariant_present_iff(invariant: &Invariant, value: &Value) -> bool {
    let Some(expected_present) = pointer(value, &invariant.boolean).and_then(Value::as_bool) else {
        return false;
    };
    invariant
        .fields
        .iter()
        .all(|path| pointer(value, path).is_some_and(|field| !field.is_null()) == expected_present)
}

fn invariant_boolean_not(invariant: &Invariant, value: &Value) -> bool {
    pointer(value, &invariant.boolean)
        .and_then(Value::as_bool)
        .zip(pointer(value, &invariant.target).and_then(Value::as_bool))
        .is_some_and(|(left, right)| left != right)
}

fn invariant_boolean_or(invariant: &Invariant, value: &Value) -> bool {
    let Some(target) = pointer(value, &invariant.target).and_then(Value::as_bool) else {
        return false;
    };
    let Some(terms) = invariant
        .terms
        .iter()
        .map(|path| pointer(value, path).and_then(Value::as_bool))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    terms.into_iter().any(|term| term) == target
}

fn invariant_selected_not_in_array(invariant: &Invariant, value: &Value) -> bool {
    let Some(selected) = pointer(value, &invariant.target) else {
        return false;
    };
    pointer(value, &invariant.array).and_then(Value::as_array).is_some_and(|array| {
        array
            .iter()
            .all(|item| pointer(item, &invariant.item_field).is_some_and(|candidate| candidate != selected))
    })
}

fn invariant_all_item_equals_iff(invariant: &Invariant, value: &Value) -> bool {
    let Some(expected) = pointer(value, &invariant.boolean).and_then(Value::as_bool) else {
        return false;
    };
    let Some(array) = pointer(value, &invariant.array).and_then(Value::as_array) else {
        return false;
    };
    let Some(fields) = array.iter().map(|item| pointer(item, &invariant.item_field)).collect::<Option<Vec<_>>>() else {
        return false;
    };
    fields.into_iter().all(|field| field == &invariant.value) == expected
}

fn invariant_unique_by(invariant: &Invariant, value: &Value) -> bool {
    pointer(value, &invariant.array).and_then(Value::as_array).is_some_and(|array| {
        let mut seen = BTreeSet::new();
        array.iter().all(|item| {
            let candidate = if invariant.item_field.is_empty() {
                item
            } else {
                let Some(candidate) = pointer(item, &invariant.item_field) else {
                    return false;
                };
                candidate
            };
            seen.insert(candidate.to_string())
        })
    })
}

fn invariant_disjoint_by(invariant: &Invariant, value: &Value) -> bool {
    let Some(left) = pointer(value, &invariant.array).and_then(Value::as_array) else {
        return false;
    };
    let Some(right) = pointer(value, &invariant.other_array).and_then(Value::as_array) else {
        return false;
    };
    let Some(left_values) = left
        .iter()
        .map(|item| pointer(item, &invariant.item_field).map(Value::to_string))
        .collect::<Option<BTreeSet<_>>>()
    else {
        return false;
    };
    right.iter().all(|item| {
        pointer(item, &invariant.other_item_field)
            .is_some_and(|candidate| !left_values.contains(&candidate.to_string()))
    })
}

fn invariant_field_equals_const_when(invariant: &Invariant, value: &Value) -> bool {
    let Some(discriminator) = pointer(value, &invariant.boolean).and_then(Value::as_bool) else {
        return false;
    };
    !discriminator || pointer(value, &invariant.target).is_some_and(|actual| actual == &invariant.value)
}

fn invariant_arrays_empty_iff_enum(invariant: &Invariant, value: &Value) -> bool {
    let Some(target) = pointer(value, &invariant.target) else {
        return false;
    };
    let Some(arrays) = invariant
        .fields
        .iter()
        .map(|path| pointer(value, path).and_then(Value::as_array))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    arrays.into_iter().all(Vec::is_empty) == (target == &invariant.value)
}

fn pointer<'a>(value: &'a Value, pointer: &str) -> Option<&'a Value> {
    if pointer.is_empty() {
        Some(value)
    } else {
        value.pointer(pointer)
    }
}

pub fn validate_fixture_set(
    surface: &Surface,
    schema: &Value,
    positives: &[(String, Value)],
    negatives: &NegativeFixtureSet,
) -> Vec<Issue> {
    let mut issues = validate_fixture_declarations(surface, positives, negatives);
    validate_positive_fixtures(surface, schema, positives, &mut issues);
    let observed_classes = validate_negative_fixtures(surface, schema, negatives, &mut issues);
    validate_declared_fixture_coverage(surface, &observed_classes, &mut issues);
    issues.sort();
    issues
}

fn validate_fixture_declarations(
    surface: &Surface,
    positives: &[(String, Value)],
    negatives: &NegativeFixtureSet,
) -> Vec<Issue> {
    let mut issues = Vec::new();
    if negatives.surface_id != surface.id {
        push_issue(
            &mut issues,
            Issue::surface(
                &surface.id,
                "schema",
                "/negative-fixture-set/surface_id",
                format!("fixture set belongs to {}", negatives.surface_id),
            ),
        );
    }
    let negative_count = u32::try_from(negatives.cases.len()).unwrap_or(u32::MAX);
    if positives.is_empty() || negatives.cases.is_empty() {
        push_issue(
            &mut issues,
            Issue::surface(&surface.id, "schema", "/fixtures", "positive and negative fixtures are required"),
        );
    }
    if negative_count > MAX_NEGATIVE_FIXTURE_CASES {
        push_issue(
            &mut issues,
            Issue::surface(
                &surface.id,
                "bounds",
                "/negative-fixture-set/cases",
                format!("case count exceeds MAX_NEGATIVE_FIXTURE_CASES={MAX_NEGATIVE_FIXTURE_CASES}"),
            ),
        );
    }
    issues
}

fn validate_positive_fixtures(
    surface: &Surface,
    schema: &Value,
    positives: &[(String, Value)],
    issues: &mut Vec<Issue>,
) {
    for (path, fixture) in positives {
        for mut issue in validate_instance(&surface.id, schema, fixture) {
            issue.message = format!("positive fixture {path} rejected: {}", issue.message);
            push_issue(issues, issue);
        }
    }
}

fn validate_negative_fixtures(
    surface: &Surface,
    schema: &Value,
    negatives: &NegativeFixtureSet,
    issues: &mut Vec<Issue>,
) -> BTreeSet<String> {
    let mut case_ids = BTreeSet::new();
    let mut observed_classes = BTreeSet::new();
    for (index, case) in negatives.cases.iter().enumerate() {
        if !case_ids.insert(case.id.clone()) {
            push_issue(
                issues,
                Issue::surface(
                    &surface.id,
                    "schema",
                    format!("/negative-fixtures/{index}/id"),
                    format!("duplicate negative case id {}", case.id),
                ),
            );
        }
        let rejections = validate_instance(&surface.id, schema, &case.artifact);
        let matched = rejections.iter().any(|issue| {
            issue.class == case.issue_class
                && (case.expected_path.is_empty() || issue.path.starts_with(&case.expected_path))
        });
        if !matched {
            push_issue(
                issues,
                Issue::surface(
                    &surface.id,
                    "schema",
                    format!("/negative-fixtures/{index}"),
                    format!(
                        "negative case {} missed class={} path={}: actual={rejections:?}",
                        case.id, case.issue_class, case.expected_path
                    ),
                ),
            );
        }
        observed_classes.insert(case.issue_class.clone());
    }
    observed_classes
}

fn validate_declared_fixture_coverage(surface: &Surface, observed_classes: &BTreeSet<String>, issues: &mut Vec<Issue>) {
    for declared in &surface.fixture_coverage {
        if !observed_classes.contains(declared) {
            push_issue(
                issues,
                Issue::surface(
                    &surface.id,
                    "schema",
                    "/fixture_coverage",
                    format!("declared fixture class {declared} has no negative case"),
                ),
            );
        }
    }
}

pub fn validate_global_fixture_coverage(observed: &BTreeSet<String>) -> Vec<Issue> {
    EXPECTED_ISSUE_CLASSES
        .iter()
        .copied()
        .filter(|class| !observed.contains(*class))
        .map(|class| {
            Issue::registry(
                "schema",
                "/fixture-coverage",
                format!("contracted fixture cohort is missing adversarial class {class}"),
            )
        })
        .collect()
}

fn is_mantle_digest(value: &str) -> bool {
    is_blake3_hex(value)
        || value.strip_prefix("blake3:").is_some_and(is_blake3_hex)
        || value.strip_prefix("sha256:").is_some_and(|hex| is_lower_hex(hex, BLAKE3_HEX_LENGTH_CHARS))
}

fn is_lower_hex(value: &str, length: u32) -> bool {
    let byte_count = u32::try_from(value.len()).unwrap_or(u32::MAX);
    byte_count == length && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_safe_reference(value: &str) -> bool {
    let byte_count = u32::try_from(value.len()).unwrap_or(u32::MAX);
    !value.is_empty()
        && byte_count <= MAX_SAFE_REFERENCE_BYTES
        && !value.bytes().any(|byte| byte.is_ascii_control())
        && !value.split('/').any(|segment| segment == "..")
        && !value.contains('\\')
}

fn is_redaction_safe(value: &str) -> bool {
    let byte_count = u32::try_from(value.len()).unwrap_or(u32::MAX);
    if byte_count > MAX_REDACTION_TEXT_BYTES || value.bytes().any(|byte| byte == 0) {
        return false;
    }
    let lower = value.to_ascii_lowercase();
    ![
        "bearer ",
        "access_token",
        "access-token",
        "secret=",
        "password=",
        "private_key",
        "private-key",
    ]
    .iter()
    .any(|fragment| lower.contains(fragment))
}
