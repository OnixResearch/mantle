use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::model::*;
use super::registry::freshness_fields;

const PRODUCER_DIGEST_CONTEXT: &[u8] = b"mantle-machine-contract-producer-v1\0";
const FIXTURE_DIGEST_CONTEXT: &[u8] = b"mantle-machine-contract-fixtures-v1\0";
const POLICY_DIGEST_CONTEXT: &[u8] = b"mantle-machine-contract-consumer-policy-v1\0";
const DIGEST_FIELD_SEPARATOR: &[u8] = b"\0";

pub fn expected_freshness(surface: &Surface, files: &BTreeMap<String, Vec<u8>>) -> Result<Freshness, String> {
    if surface.class != CONTRACTED_CLASS {
        return Ok(Freshness::default());
    }
    let fixture_paths = surface
        .artifacts
        .positive_fixtures
        .iter()
        .chain(std::iter::once(&surface.artifacts.negative_fixture_set))
        .chain(surface.version_policy.compatibility_fixtures.iter())
        .cloned()
        .collect::<Vec<_>>();
    let policy_bytes = serde_json::to_vec(&serde_json::json!({
        "surface_id": surface.id,
        "class": surface.class,
        "consumers": surface.consumers,
        "version_policy": surface.version_policy,
        "validation_commands": surface.validation_commands,
        "fixture_coverage": surface.fixture_coverage,
        "freshness_strategy": surface.freshness_strategy,
        "non_claims": surface.non_claims,
        "rationale": surface.rationale,
    }))
    .map_err(|err| format!("serializing consumer policy for {}: {err}", surface.id))?;
    Ok(Freshness {
        schema_blake3: digest_file(files, &surface.artifacts.schema)?,
        contract_blake3: digest_file(files, &surface.artifacts.generated_contract)?,
        prelude_blake3: digest_file(files, PRELUDE_PATH)?,
        fixture_set_blake3: digest_path_set(FIXTURE_DIGEST_CONTEXT, files, &fixture_paths)?,
        producer_identity_blake3: digest_producer_identity(surface, files)?,
        consumer_policy_blake3: digest_bytes_with_context(POLICY_DIGEST_CONTEXT, &policy_bytes),
    })
}

pub fn validate_freshness(surface: &Surface, expected: &Freshness) -> Vec<Issue> {
    let mut issues = Vec::new();
    for ((name, actual), (_, expected)) in
        freshness_fields(&surface.freshness).into_iter().zip(freshness_fields(expected).into_iter())
    {
        if actual != expected {
            push_issue(
                &mut issues,
                Issue::surface(
                    &surface.id,
                    "digest",
                    format!("/freshness/{name}"),
                    format!("stale BLAKE3 binding: recorded={actual} expected={expected}"),
                ),
            );
        }
    }
    issues
}

fn digest_file(files: &BTreeMap<String, Vec<u8>>, path: &str) -> Result<String, String> {
    let bytes = files.get(path).ok_or_else(|| format!("freshness input {path} was not loaded"))?;
    Ok(blake3::hash(bytes).to_hex().to_string())
}

fn digest_producer_identity(surface: &Surface, files: &BTreeMap<String, Vec<u8>>) -> Result<String, String> {
    let metadata = serde_json::to_vec(&serde_json::json!({
        "surface_id": surface.id,
        "rust_owner": surface.rust_owner,
        "producer": surface.producer,
    }))
    .map_err(|err| format!("serializing producer identity for {}: {err}", surface.id))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(PRODUCER_DIGEST_CONTEXT);
    update_length_prefixed(&mut hasher, &metadata)?;
    update_path_set(&mut hasher, files, &surface.producer.source_paths)?;
    Ok(hasher.finalize().to_hex().to_string())
}

fn digest_path_set(context: &[u8], files: &BTreeMap<String, Vec<u8>>, paths: &[String]) -> Result<String, String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(context);
    update_path_set(&mut hasher, files, paths)?;
    Ok(hasher.finalize().to_hex().to_string())
}

fn update_path_set(
    hasher: &mut blake3::Hasher,
    files: &BTreeMap<String, Vec<u8>>,
    paths: &[String],
) -> Result<(), String> {
    let mut sorted = paths.to_vec();
    sorted.sort();
    sorted.dedup();
    for path in sorted {
        let bytes = files.get(&path).ok_or_else(|| format!("freshness input {path} was not loaded"))?;
        update_length_prefixed(hasher, path.as_bytes())?;
        update_length_prefixed(hasher, bytes)?;
    }
    Ok(())
}

fn update_length_prefixed(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), String> {
    let byte_count = u64::try_from(bytes.len()).map_err(|_| "freshness input length exceeds u64".to_string())?;
    hasher.update(&byte_count.to_le_bytes());
    hasher.update(DIGEST_FIELD_SEPARATOR);
    hasher.update(bytes);
    hasher.update(DIGEST_FIELD_SEPARATOR);
    Ok(())
}

fn digest_bytes_with_context(context: &[u8], bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(context);
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

pub fn all_referenced_paths(registry: &Registry) -> BTreeSet<String> {
    let mut paths = BTreeSet::from([registry.prelude_path.clone()]);
    for surface in &registry.surfaces {
        paths.extend(surface.producer.source_paths.iter().cloned());
        if surface.class != CONTRACTED_CLASS {
            continue;
        }
        paths.insert(surface.artifacts.schema.clone());
        paths.insert(surface.artifacts.generated_contract.clone());
        paths.insert(surface.artifacts.negative_fixture_set.clone());
        paths.extend(surface.artifacts.positive_fixtures.iter().cloned());
        paths.extend(surface.version_policy.compatibility_fixtures.iter().cloned());
    }
    paths
}
