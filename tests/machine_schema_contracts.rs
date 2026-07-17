use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;

const INVENTORY_PATH: &str = "schemas/machine-contracts/inventory.ncl";
const CONTRACTED_SURFACE_COUNT: u32 = 18;
const UTF8_EXACT_BYTES: u32 = 4;
const CONTRACT_EVALUATION_STACK_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct RegistryProjection {
    initial_cohort: Vec<String>,
    surfaces: Vec<SurfaceProjection>,
}

#[derive(Debug, Deserialize)]
struct SurfaceProjection {
    id: String,
    class: String,
    artifacts: ArtifactProjection,
}

#[derive(Debug, Deserialize)]
struct ArtifactProjection {
    generated_contract: String,
    positive_fixtures: Vec<String>,
    negative_fixture_set: String,
}

#[derive(Debug, Deserialize)]
struct NegativeFixtureSet {
    surface_id: String,
    cases: Vec<NegativeFixtureCase>,
}

#[derive(Debug, Deserialize)]
struct NegativeFixtureCase {
    id: String,
    artifact: Value,
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn contract_directory() -> PathBuf {
    repository_root().join("schemas/machine-contracts")
}

fn evaluate_nickel(path: &Path) -> Result<Value, crunch_eval::Error> {
    let import_paths = Vec::<OsString>::new();
    crunch_eval::evaluate_and_deserialize(path, &import_paths)
}

fn load_registry() -> RegistryProjection {
    let inventory = repository_root().join(INVENTORY_PATH);
    let value = evaluate_nickel(&inventory).expect("typed Nickel inventory must evaluate");
    serde_json::from_value(value).expect("typed inventory must match the test projection")
}

fn contract_probe_source(contract: &Path, fixture: &Path) -> String {
    let contract_literal = serde_json::to_string(&contract.display().to_string()).expect("serialize contract path");
    let fixture_literal = serde_json::to_string(&fixture.display().to_string()).expect("serialize fixture path");
    format!("let Contract = import {contract_literal} in\n(import {fixture_literal}) | Contract\n")
}

fn evaluate_source(source: &str) -> Result<Value, crunch_eval::Error> {
    let temp = tempfile::tempdir().expect("create contract probe directory");
    let probe = temp.path().join("probe.ncl");
    fs::write(&probe, source).expect("write contract probe");
    evaluate_nickel(&probe)
}

fn evaluate_contract(contract: &Path, fixture: &Path) -> Result<Value, crunch_eval::Error> {
    evaluate_source(&contract_probe_source(contract, fixture))
}

fn string_bounds_probe(value: &str) -> String {
    let prelude = contract_directory().join("prelude.ncl");
    let prelude_literal = serde_json::to_string(&prelude.display().to_string()).expect("serialize prelude path");
    let value_literal = serde_json::to_string(value).expect("serialize UTF-8 fixture");
    format!(
        "let C = import {prelude_literal} in\n{value_literal} | C.StringBounds {UTF8_EXACT_BYTES} {UTF8_EXACT_BYTES}\n"
    )
}

fn prelude_predicate_probe(predicate: &str, value: &str) -> String {
    let prelude = contract_directory().join("prelude.ncl");
    let prelude_literal = serde_json::to_string(&prelude.display().to_string()).expect("serialize prelude path");
    let value_literal = serde_json::to_string(value).expect("serialize predicate fixture");
    format!("let C = import {prelude_literal} in\nC.{predicate} {value_literal}\n")
}

#[test]
fn inventory_is_typed_nickel_data() {
    let registry = load_registry();
    let contracted_count =
        u32::try_from(registry.surfaces.iter().filter(|surface| surface.class == "contracted").count())
            .expect("contracted cohort count fits u32");
    let cohort_count = u32::try_from(registry.initial_cohort.len()).expect("initial cohort count fits u32");
    assert_eq!(contracted_count, CONTRACTED_SURFACE_COUNT);
    assert_eq!(cohort_count, contracted_count);
    assert!(registry.surfaces.iter().all(|surface| !surface.id.is_empty()));
}

#[test]
fn shared_string_bounds_count_utf8_bytes_exactly() {
    let exact_bytes = evaluate_source(&string_bounds_probe("éé"));
    assert!(exact_bytes.is_ok(), "four UTF-8 bytes should satisfy exact byte bounds: {exact_bytes:?}");
    let too_many_bytes = evaluate_source(&string_bounds_probe("ééé"));
    assert!(too_many_bytes.is_err(), "six UTF-8 bytes must fail a four-byte bound");
}

#[test]
fn shared_oci_identity_predicates_are_callable_and_fail_closed() {
    let cases = [
        (
            "IsSha256Digest",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ),
        (
            "IsMantleReference",
            "mantle://blake3/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "../aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ),
    ];

    for (predicate, valid, invalid) in cases {
        let accepted = evaluate_source(&prelude_predicate_probe(predicate, valid))
            .unwrap_or_else(|error| panic!("{predicate} rejected a valid identity: {error}"));
        let rejected = evaluate_source(&prelude_predicate_probe(predicate, invalid))
            .unwrap_or_else(|error| panic!("{predicate} failed to evaluate an invalid identity: {error}"));
        assert_eq!(accepted, Value::Bool(true), "{predicate} must accept its canonical identity");
        assert_eq!(rejected, Value::Bool(false), "{predicate} must reject its malformed identity");
    }
}

#[test]
fn generated_contracts_accept_positive_and_reject_adversarial_fixtures() {
    let evaluator = std::thread::Builder::new()
        .name("machine-contract-evaluator".to_string())
        .stack_size(CONTRACT_EVALUATION_STACK_BYTES)
        .spawn(run_generated_contract_fixture_checks)
        .expect("spawn bounded-stack contract evaluator");
    evaluator.join().expect("contract evaluator must not panic");
}

fn run_generated_contract_fixture_checks() {
    let registry = load_registry();
    for surface in registry.surfaces.into_iter().filter(|surface| surface.class == "contracted") {
        let contract = repository_root().join(&surface.artifacts.generated_contract);
        assert!(!surface.artifacts.positive_fixtures.is_empty(), "{} has no positive fixtures", surface.id);
        for positive_path in &surface.artifacts.positive_fixtures {
            let positive = repository_root().join(positive_path);
            evaluate_contract(&contract, &positive)
                .unwrap_or_else(|error| panic!("{} rejected positive fixture {positive_path}: {error}", surface.id));
        }

        let negative_path = repository_root().join(&surface.artifacts.negative_fixture_set);
        let negative_set: NegativeFixtureSet =
            serde_json::from_str(&fs::read_to_string(&negative_path).expect("read negative fixture set"))
                .expect("parse negative fixture set");
        assert_eq!(negative_set.surface_id, surface.id);
        assert!(!negative_set.cases.is_empty());
        for case in negative_set.cases {
            let temp = tempfile::tempdir().expect("create negative fixture directory");
            let fixture = temp.path().join("artifact.json");
            fs::write(&fixture, serde_json::to_vec_pretty(&case.artifact).expect("serialize negative artifact"))
                .expect("write negative artifact");
            let result = evaluate_contract(&contract, &fixture);
            assert!(result.is_err(), "{} negative case {} was accepted", surface.id, case.id);
        }
    }
}
