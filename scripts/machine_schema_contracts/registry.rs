use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Component;
use std::path::Path;

use super::model::*;

pub fn parse_registry_source(source: &str) -> Result<Registry, String> {
    let normalized = replace_unquoted_delimiter(registry_data_slice(source)?, '=', ':');
    serde_json::from_str(&normalized).map_err(|err| format!("parsing JSON-compatible Nickel registry data: {err}"))
}

pub fn render_registry_source(original: &str, registry: &Registry) -> Result<String, String> {
    let start_marker = original
        .find(REGISTRY_DATA_BEGIN)
        .ok_or_else(|| format!("inventory missing {REGISTRY_DATA_BEGIN}"))?;
    let data_start = original[start_marker..]
        .find('\n')
        .map(|offset| start_marker.saturating_add(offset).saturating_add(1))
        .ok_or_else(|| "inventory registry begin marker has no following line".to_string())?;
    let end = original[data_start..]
        .find(REGISTRY_DATA_END)
        .map(|offset| data_start.saturating_add(offset))
        .ok_or_else(|| format!("inventory missing {REGISTRY_DATA_END}"))?;
    let rendered_json =
        serde_json::to_string_pretty(registry).map_err(|err| format!("serializing registry JSON: {err}"))?;
    let rendered_nickel = replace_unquoted_delimiter(&rendered_json, ':', '=');
    let mut output = String::with_capacity(original.len().saturating_add(rendered_nickel.len()));
    output.push_str(&original[..data_start]);
    output.push_str(&rendered_nickel);
    output.push('\n');
    output.push_str(&original[end..]);
    Ok(output)
}

fn replace_unquoted_delimiter(source: &str, from: char, to: char) -> String {
    let mut output = String::with_capacity(source.len());
    let mut in_string = false;
    let mut escaped = false;
    for character in source.chars() {
        if in_string {
            output.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        if character == '"' {
            in_string = true;
            output.push(character);
        } else if character == from {
            output.push(to);
        } else {
            output.push(character);
        }
    }
    output
}

fn registry_data_slice(source: &str) -> Result<&str, String> {
    let start_marker =
        source.find(REGISTRY_DATA_BEGIN).ok_or_else(|| format!("inventory missing {REGISTRY_DATA_BEGIN}"))?;
    let data_start = source[start_marker..]
        .find('\n')
        .map(|offset| start_marker.saturating_add(offset).saturating_add(1))
        .ok_or_else(|| "inventory registry begin marker has no following line".to_string())?;
    let data_end = source[data_start..]
        .find(REGISTRY_DATA_END)
        .map(|offset| data_start.saturating_add(offset))
        .ok_or_else(|| format!("inventory missing {REGISTRY_DATA_END}"))?;
    let data = source[data_start..data_end].trim();
    if data.is_empty() {
        return Err("inventory embedded registry JSON is empty".to_string());
    }
    Ok(data)
}

pub fn validate_registry(registry: &Registry) -> Vec<Issue> {
    let mut issues = Vec::new();
    validate_registry_header(registry, &mut issues);
    let mut ids = BTreeSet::new();
    let mut markers = BTreeSet::new();
    let mut artifact_paths = BTreeMap::<String, String>::new();
    for (index, surface) in registry.surfaces.iter().enumerate() {
        let path = format!("/surfaces/{index}");
        validate_surface(surface, &path, &mut issues);
        if !ids.insert(surface.id.clone()) {
            push_issue(
                &mut issues,
                Issue::surface(&surface.id, "schema", format!("{path}/id"), "duplicate surface id"),
            );
        }
        if !markers.insert(surface.producer.marker.clone()) {
            push_issue(
                &mut issues,
                Issue::surface(&surface.id, "schema", format!("{path}/producer/marker"), "duplicate producer marker"),
            );
        }
        if surface.class == CONTRACTED_CLASS {
            for artifact_path in contracted_artifact_paths(surface) {
                if let Some(owner) = artifact_paths.insert(artifact_path.to_string(), surface.id.clone()) {
                    push_issue(
                        &mut issues,
                        Issue::surface(
                            &surface.id,
                            "schema",
                            format!("{path}/artifacts"),
                            format!("artifact path {artifact_path} already owned by {owner}"),
                        ),
                    );
                }
            }
        }
    }
    validate_initial_cohort(registry, &ids, &mut issues);
    issues.sort();
    issues
}

fn validate_registry_header(registry: &Registry, issues: &mut Vec<Issue>) {
    if registry.registry_schema != REGISTRY_SCHEMA {
        push_issue(
            issues,
            Issue::registry(
                "schema",
                "/registry_schema",
                format!("expected {REGISTRY_SCHEMA}, got {}", registry.registry_schema),
            ),
        );
    }
    if registry.prelude_path != PRELUDE_PATH {
        push_issue(
            issues,
            Issue::registry("schema", "/prelude_path", format!("expected shared prelude path {PRELUDE_PATH}")),
        );
    }
    let actual_classes = registry.supported_classes.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let expected_classes = SUPPORTED_SURFACE_CLASSES.iter().copied().collect::<BTreeSet<_>>();
    if actual_classes != expected_classes {
        push_issue(
            issues,
            Issue::registry(
                "schema",
                "/supported_classes",
                format!("supported classes must be exactly {expected_classes:?}"),
            ),
        );
    }
    let surface_count = u32::try_from(registry.surfaces.len()).unwrap_or(u32::MAX);
    if registry.surfaces.is_empty() || surface_count > MAX_REGISTRY_SURFACES {
        push_issue(
            issues,
            Issue::registry("bounds", "/surfaces", format!("surface count must be in 1..={MAX_REGISTRY_SURFACES}")),
        );
    }
}

fn validate_initial_cohort(registry: &Registry, ids: &BTreeSet<String>, issues: &mut Vec<Issue>) {
    let mut seen = BTreeSet::new();
    for (index, cohort_id) in registry.initial_cohort.iter().enumerate() {
        if !seen.insert(cohort_id) {
            push_issue(
                issues,
                Issue::registry(
                    "schema",
                    format!("/initial_cohort/{index}"),
                    format!("duplicate initial cohort id {cohort_id}"),
                ),
            );
        }
        if !ids.contains(cohort_id) {
            push_issue(
                issues,
                Issue::registry(
                    "schema",
                    format!("/initial_cohort/{index}"),
                    format!("initial cohort surface {cohort_id} is not registered"),
                ),
            );
        }
        if registry
            .surfaces
            .iter()
            .find(|surface| surface.id == *cohort_id)
            .is_some_and(|surface| surface.class != CONTRACTED_CLASS)
        {
            push_issue(
                issues,
                Issue::surface(
                    cohort_id,
                    "schema",
                    format!("/initial_cohort/{index}"),
                    "initial cohort surface must be contracted",
                ),
            );
        }
    }
    let contracted_count = registry.surfaces.iter().filter(|surface| surface.class == CONTRACTED_CLASS).count();
    if contracted_count != registry.initial_cohort.len() {
        push_issue(
            issues,
            Issue::registry(
                "schema",
                "/initial_cohort",
                format!(
                    "all contracted surfaces must be in the initial cohort: contracted={contracted_count} cohort={}",
                    registry.initial_cohort.len()
                ),
            ),
        );
    }
}

fn validate_surface(surface: &Surface, path: &str, issues: &mut Vec<Issue>) {
    if !valid_surface_id(&surface.id) {
        push_issue(
            issues,
            Issue::surface(&surface.id, "schema", format!("{path}/id"), "surface id must be dotted lowercase ASCII"),
        );
    }
    if !SUPPORTED_SURFACE_CLASSES.contains(&surface.class.as_str()) {
        push_issue(
            issues,
            Issue::surface(
                &surface.id,
                "schema",
                format!("{path}/class"),
                format!("unsupported class {}", surface.class),
            ),
        );
    }
    require_text(surface, &surface.rust_owner, "rust_owner", path, issues);
    require_text(surface, &surface.producer.command_or_api, "producer/command_or_api", path, issues);
    require_text(surface, &surface.producer.marker, "producer/marker", path, issues);
    require_text(surface, &surface.rationale, "rationale", path, issues);
    require_text(surface, &surface.freshness_strategy, "freshness_strategy", path, issues);
    require_list(surface, &surface.producer.source_paths, "producer/source_paths", path, issues);
    require_list(surface, &surface.consumers, "consumers", path, issues);
    require_list(surface, &surface.validation_commands, "validation_commands", path, issues);
    require_list(surface, &surface.non_claims, "non_claims", path, issues);
    validate_version_policy(surface, path, issues);
    validate_surface_paths(surface, path, issues);
    if surface.class == CONTRACTED_CLASS {
        require_text(surface, &surface.artifacts.schema, "artifacts/schema", path, issues);
        require_text(surface, &surface.artifacts.generated_contract, "artifacts/generated_contract", path, issues);
        require_text(surface, &surface.artifacts.negative_fixture_set, "artifacts/negative_fixture_set", path, issues);
        require_list(surface, &surface.artifacts.positive_fixtures, "artifacts/positive_fixtures", path, issues);
        require_list(surface, &surface.fixture_coverage, "fixture_coverage", path, issues);
        for (name, digest) in freshness_fields(&surface.freshness) {
            if !is_blake3_hex(digest) {
                push_issue(
                    issues,
                    Issue::surface(
                        &surface.id,
                        "digest",
                        format!("{path}/freshness/{name}"),
                        "freshness binding must be lowercase BLAKE3 hex",
                    ),
                );
            }
        }
    } else if !surface.artifacts.schema.is_empty()
        || !surface.artifacts.generated_contract.is_empty()
        || !surface.artifacts.positive_fixtures.is_empty()
        || !surface.artifacts.negative_fixture_set.is_empty()
        || freshness_fields(&surface.freshness).iter().any(|(_, digest)| !digest.is_empty())
    {
        push_issue(
            issues,
            Issue::surface(
                &surface.id,
                "schema",
                format!("{path}/artifacts"),
                "uncontracted surfaces must not claim generated contract artifacts or freshness",
            ),
        );
    }
}

fn validate_surface_paths(surface: &Surface, path: &str, issues: &mut Vec<Issue>) {
    for source_path in &surface.producer.source_paths {
        let allowed_root =
            source_path.starts_with("src/") || source_path.starts_with("crates/") || source_path.starts_with("tools/");
        if !safe_repo_relative_path(source_path) || !allowed_root || !source_path.ends_with(".rs") {
            push_issue(
                issues,
                Issue::surface(
                    &surface.id,
                    "reference",
                    format!("{path}/producer/source_paths"),
                    format!("producer source must be a safe Rust path under src, crates, or tools: {source_path}"),
                ),
            );
        }
    }
    if surface.class != CONTRACTED_CLASS {
        return;
    }
    for artifact_path in [
        surface.artifacts.schema.as_str(),
        surface.artifacts.generated_contract.as_str(),
        surface.artifacts.negative_fixture_set.as_str(),
    ] {
        validate_artifact_path(surface, path, artifact_path, false, issues);
    }
    for fixture_path in surface
        .artifacts
        .positive_fixtures
        .iter()
        .chain(surface.version_policy.compatibility_fixtures.iter())
    {
        validate_artifact_path(surface, path, fixture_path, true, issues);
    }
}

fn validate_artifact_path(surface: &Surface, path: &str, artifact_path: &str, fixture: bool, issues: &mut Vec<Issue>) {
    let allowed_root = artifact_path.starts_with("schemas/machine-contracts/")
        || (fixture && artifact_path.starts_with("tests/fixtures/"));
    if safe_repo_relative_path(artifact_path) && allowed_root {
        return;
    }
    push_issue(
        issues,
        Issue::surface(
            &surface.id,
            "reference",
            format!("{path}/artifacts"),
            format!("contract artifact path is outside an allowed review fixture root: {artifact_path}"),
        ),
    );
}

fn safe_repo_relative_path(value: &str) -> bool {
    if value.is_empty() || value.contains('\\') {
        return false;
    }
    let mut normalized = Vec::new();
    for component in Path::new(value).components() {
        let Component::Normal(segment) = component else {
            return false;
        };
        let Some(segment) = segment.to_str() else {
            return false;
        };
        normalized.push(segment);
    }
    !normalized.is_empty() && normalized.join("/") == value
}

fn validate_version_policy(surface: &Surface, path: &str, issues: &mut Vec<Issue>) {
    require_text(
        surface,
        &surface.version_policy.unknown_version_behavior,
        "version_policy/unknown_version_behavior",
        path,
        issues,
    );
    if surface.class != CONTRACTED_CLASS {
        return;
    }
    if surface.version_policy.current.is_empty() {
        push_issue(
            issues,
            Issue::surface(
                &surface.id,
                "version",
                format!("{path}/version_policy/current"),
                "contracted surface requires a current version",
            ),
        );
    }
    if !surface.version_policy.supported.iter().any(|version| version == &surface.version_policy.current) {
        push_issue(
            issues,
            Issue::surface(
                &surface.id,
                "version",
                format!("{path}/version_policy/supported"),
                "supported versions must include current",
            ),
        );
    }
    let supported_count = u32::try_from(surface.version_policy.supported.len()).unwrap_or(u32::MAX);
    let admits_prior = supported_count > MIN_REQUIRED_FIXTURE_CASES;
    if admits_prior
        && (surface.version_policy.compatibility_converter.is_empty()
            || surface.version_policy.compatibility_fixtures.is_empty())
    {
        push_issue(
            issues,
            Issue::surface(
                &surface.id,
                "version",
                format!("{path}/version_policy"),
                "prior versions require an explicit converter and migration fixtures",
            ),
        );
    }
    if !admits_prior && !surface.version_policy.compatibility_converter.is_empty() {
        push_issue(
            issues,
            Issue::surface(
                &surface.id,
                "version",
                format!("{path}/version_policy/compatibility_converter"),
                "converter is declared without a prior supported version",
            ),
        );
    }
}

fn require_text(surface: &Surface, value: &str, field: &str, path: &str, issues: &mut Vec<Issue>) {
    if value.trim().is_empty() {
        push_issue(
            issues,
            Issue::surface(&surface.id, "schema", format!("{path}/{field}"), format!("{field} must not be empty")),
        );
    }
}

fn require_list(surface: &Surface, values: &[String], field: &str, path: &str, issues: &mut Vec<Issue>) {
    let item_count = u32::try_from(values.len()).unwrap_or(u32::MAX);
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        push_issue(
            issues,
            Issue::surface(
                &surface.id,
                "schema",
                format!("{path}/{field}"),
                format!("{field} must contain non-empty values"),
            ),
        );
    }
    if item_count > MAX_REGISTRY_LIST_ITEMS {
        push_issue(
            issues,
            Issue::surface(
                &surface.id,
                "bounds",
                format!("{path}/{field}"),
                format!("{field} exceeds MAX_REGISTRY_LIST_ITEMS={MAX_REGISTRY_LIST_ITEMS}"),
            ),
        );
    }
    let distinct = values.iter().collect::<BTreeSet<_>>();
    if distinct.len() != values.len() {
        push_issue(
            issues,
            Issue::surface(
                &surface.id,
                "schema",
                format!("{path}/{field}"),
                format!("{field} must not contain duplicate values"),
            ),
        );
    }
}

fn contracted_artifact_paths(surface: &Surface) -> Vec<&str> {
    let mut paths = vec![
        surface.artifacts.schema.as_str(),
        surface.artifacts.generated_contract.as_str(),
        surface.artifacts.negative_fixture_set.as_str(),
    ];
    paths.extend(surface.artifacts.positive_fixtures.iter().map(String::as_str));
    paths.extend(surface.version_policy.compatibility_fixtures.iter().map(String::as_str));
    paths
}

fn valid_surface_id(value: &str) -> bool {
    !value.is_empty()
        && value.contains('.')
        && value.split('.').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_')
        })
}

pub fn extract_public_markers(sources: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut markers = BTreeMap::new();
    for (path, source) in sources {
        for line in source.lines() {
            let Some((_, raw_marker)) = line.split_once(PUBLIC_MARKER_PREFIX) else {
                continue;
            };
            let marker = raw_marker.trim();
            if !marker.is_empty() {
                markers.insert(marker.to_string(), path.clone());
            }
        }
    }
    markers
}

pub fn validate_registered_source_paths(registry: &Registry, sources: &BTreeMap<String, String>) -> Vec<Issue> {
    let mut issues = Vec::new();
    for surface in &registry.surfaces {
        for path in &surface.producer.source_paths {
            if !sources.contains_key(path) {
                push_issue(
                    &mut issues,
                    Issue::surface(
                        &surface.id,
                        "schema",
                        "/producer/source_paths",
                        format!("source path missing: {path}"),
                    ),
                );
            }
        }
        if let Some((owner_path, _)) = surface.rust_owner.split_once("::")
            && owner_path.ends_with(".rs")
            && !surface.producer.source_paths.iter().any(|path| path == owner_path)
        {
            push_issue(
                &mut issues,
                Issue::surface(
                    &surface.id,
                    "schema",
                    "/rust_owner",
                    format!("Rust owner path {owner_path} is absent from producer source_paths"),
                ),
            );
        }
    }
    issues.sort();
    issues
}

pub fn validate_root_json_source_coverage(registry: &Registry, sources: &BTreeMap<String, String>) -> Vec<Issue> {
    const PUBLIC_JSON_SERIALIZATION_TOKENS: &[&str] = &[
        "serde_json::to_string(",
        "serde_json::to_string_pretty",
        "serde_json::to_vec(",
        "serde_json::to_vec_pretty",
    ];
    let declared = registry
        .surfaces
        .iter()
        .flat_map(|surface| surface.producer.source_paths.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    let mut issues = Vec::new();
    for (path, source) in sources {
        if !path.starts_with("src/") {
            continue;
        }
        let serializes_public_shape = PUBLIC_JSON_SERIALIZATION_TOKENS.iter().any(|token| source.contains(token));
        if serializes_public_shape && !declared.contains(path.as_str()) {
            push_issue(
                &mut issues,
                Issue::registry(
                    "schema",
                    format!("source:{path}"),
                    "root JSON serialization source has no inventory family decision",
                ),
            );
        }
    }
    issues.sort();
    issues
}

pub fn validate_runtime_contract_boundary(sources: &BTreeMap<String, String>) -> Vec<Issue> {
    const RUNTIME_CONTRACT_TOKENS: &[&str] = &[
        "schemas/machine-contracts/inventory.ncl",
        "schemas/machine-contracts/prelude.ncl",
        ".contract.ncl",
    ];
    let mut issues = Vec::new();
    for (path, source) in sources {
        if !path.starts_with("src/") {
            continue;
        }
        for token in RUNTIME_CONTRACT_TOKENS.iter().filter(|token| source.contains(**token)) {
            push_issue(
                &mut issues,
                Issue::registry(
                    "schema",
                    format!("source:{path}"),
                    format!("runtime source references review-only Nickel contract token {token}"),
                ),
            );
        }
    }
    issues.sort();
    issues
}

pub fn validate_marker_completeness(registry: &Registry, discovered: &BTreeMap<String, String>) -> Vec<Issue> {
    let mut issues = Vec::new();
    let registered = registry
        .surfaces
        .iter()
        .map(|surface| (surface.producer.marker.as_str(), surface))
        .collect::<BTreeMap<_, _>>();
    for (marker, path) in discovered {
        if !registered.contains_key(marker.as_str()) {
            push_issue(
                &mut issues,
                Issue::registry(
                    "schema",
                    format!("marker:{marker}"),
                    format!("public machine producer in {path} is not classified"),
                ),
            );
        }
    }
    for surface in &registry.surfaces {
        match discovered.get(&surface.producer.marker) {
            None => push_issue(
                &mut issues,
                Issue::surface(
                    &surface.id,
                    "schema",
                    "/producer/marker",
                    format!("producer marker {} is absent from Rust sources", surface.producer.marker),
                ),
            ),
            Some(path) if !surface.producer.source_paths.iter().any(|source_path| source_path == path) => {
                push_issue(
                    &mut issues,
                    Issue::surface(
                        &surface.id,
                        "schema",
                        "/producer/source_paths",
                        format!("marker {} found in undeclared source {path}", surface.producer.marker),
                    ),
                );
            }
            Some(_) => {}
        }
    }
    issues.sort();
    issues
}

pub fn freshness_fields(freshness: &Freshness) -> Vec<(&'static str, &str)> {
    vec![
        ("schema_blake3", &freshness.schema_blake3),
        ("contract_blake3", &freshness.contract_blake3),
        ("prelude_blake3", &freshness.prelude_blake3),
        ("fixture_set_blake3", &freshness.fixture_set_blake3),
        ("producer_identity_blake3", &freshness.producer_identity_blake3),
        ("consumer_policy_blake3", &freshness.consumer_policy_blake3),
    ]
}

pub fn is_blake3_hex(value: &str) -> bool {
    let byte_count = u32::try_from(value.len()).unwrap_or(u32::MAX);
    byte_count == BLAKE3_HEX_LENGTH_CHARS
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
