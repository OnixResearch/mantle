#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"

[dependencies]
serde_json = "1.0"
blake3 = "=1.8.2"
---

//! Deterministic schema-to-Nickel contract rail for selected Mantle machine JSON
//! surfaces. This checker intentionally validates fixtures against a bounded
//! local subset instead of depending on remote schema conversion at runtime.

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use serde_json::Value;

const INVENTORY_PATH: &str = "schemas/machine-contracts/inventory.ncl";
const DOCTOR_SCHEMA_PATH: &str = "schemas/machine-contracts/doctor-report.schema.json";
const DOCTOR_CONTRACT_PATH: &str = "schemas/machine-contracts/doctor-report.contract.ncl";
const VALID_DOCTOR_FIXTURE: &str = "schemas/machine-contracts/fixtures/doctor-report-valid.json";
const NEGATIVE_DOCTOR_FIXTURES: &[&str] = &[
    "schemas/machine-contracts/fixtures/doctor-report-missing-schema.json",
    "schemas/machine-contracts/fixtures/doctor-report-invalid-status.json",
    "schemas/machine-contracts/fixtures/doctor-report-invalid-version.json",
];

const DOCTOR_SCHEMA_VALUE: &str = "crunch-doctor-report-v1";
const DOCTOR_COMMAND_VALUE: &str = "doctor";
const DOCTOR_SURFACE_ID: &str = "operator-diagnostics.doctor-report";
const JSON_SCHEMA_DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";
const EXPECTED_CHECK_STATUS_OK: &str = "ok";
const EXPECTED_CHECK_STATUS_FAILED: &str = "failed";
const EXPECTED_PROFILE_BUILD: &str = "build";
const EXPECTED_PROFILE_SELF_BUILD: &str = "self-build";
const EXPECTED_BLAKE3_HEX_BYTES: usize = 64;

fn main() -> ExitCode {
    let args = env::args().collect::<Vec<_>>();
    let result = if args.iter().any(|arg| arg == "--self-test") {
        run_self_test()
    } else {
        run_check()
    };
    match result {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("machine-schema-contract check failed: {err}");
            ExitCode::from(1)
        }
    }
}

fn run_check() -> Result<String, String> {
    validate_inventory()?;
    validate_schema_owner()?;
    validate_contract_snapshot()?;
    validate_positive_fixture(VALID_DOCTOR_FIXTURE)?;
    for fixture in NEGATIVE_DOCTOR_FIXTURES {
        validate_negative_fixture(fixture)?;
    }
    let contract_digest = blake3_file_hex(Path::new(DOCTOR_CONTRACT_PATH))?;
    Ok(format!(
        "machine schema contracts ok: surface={DOCTOR_SURFACE_ID} contract={} digest_blake3={contract_digest} positive=1 negative={}",
        DOCTOR_CONTRACT_PATH,
        NEGATIVE_DOCTOR_FIXTURES.len()
    ))
}

fn run_self_test() -> Result<String, String> {
    let valid = serde_json::json!({
        "schema": DOCTOR_SCHEMA_VALUE,
        "command": DOCTOR_COMMAND_VALUE,
        "profile": EXPECTED_PROFILE_BUILD,
        "ok": true,
        "checks": [{"id": "bwrap", "status": EXPECTED_CHECK_STATUS_OK, "summary": "present"}],
    });
    validate_doctor_payload(&valid)?;
    let mut invalid = valid.clone();
    invalid["checks"][0]["status"] = Value::String("unknown".to_string());
    if validate_doctor_payload(&invalid).is_ok() {
        return Err("self-test invalid status unexpectedly passed".to_string());
    }
    let mut stale_contract = render_doctor_contract_snapshot();
    stale_contract.push_str("# stale\n");
    if stale_contract == render_doctor_contract_snapshot() {
        return Err("self-test stale contract mutation was not observable".to_string());
    }
    Ok("machine schema contract self-test ok".to_string())
}

fn validate_inventory() -> Result<(), String> {
    let inventory = read_text(INVENTORY_PATH)?;
    for required in [
        DOCTOR_SURFACE_ID,
        DOCTOR_SCHEMA_PATH,
        DOCTOR_CONTRACT_PATH,
        VALID_DOCTOR_FIXTURE,
        "fixture contract validation does not prove command correctness",
    ] {
        if !inventory.contains(required) {
            return Err(format!("inventory missing required fragment `{required}`"));
        }
    }
    Ok(())
}

fn validate_schema_owner() -> Result<(), String> {
    let schema = read_json(DOCTOR_SCHEMA_PATH)?;
    require_string_field(&schema, "$schema", JSON_SCHEMA_DRAFT)?;
    require_string_field(&schema, "$id", "mantle://schemas/operator-diagnostics/doctor-report-v1")?;
    require_string_field(&schema, "type", "object")?;
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .ok_or_else(|| "doctor schema missing required[]".to_string())?;
    for field in ["schema", "command", "profile", "ok", "checks"] {
        if !required.iter().any(|value| value.as_str() == Some(field)) {
            return Err(format!("doctor schema required[] missing `{field}`"));
        }
    }
    Ok(())
}

fn validate_contract_snapshot() -> Result<(), String> {
    let actual = read_text(DOCTOR_CONTRACT_PATH)?;
    let expected = render_doctor_contract_snapshot();
    if actual != expected {
        return Err(format!(
            "stale generated contract snapshot: regenerate {DOCTOR_CONTRACT_PATH} from {DOCTOR_SCHEMA_PATH}"
        ));
    }
    Ok(())
}

fn validate_positive_fixture(path: &str) -> Result<(), String> {
    let value = read_json(path)?;
    validate_doctor_payload(&value).map_err(|err| format!("positive fixture {path} rejected: {err}"))
}

fn validate_negative_fixture(path: &str) -> Result<(), String> {
    let value = read_json(path)?;
    match validate_doctor_payload(&value) {
        Ok(()) => Err(format!("negative fixture {path} unexpectedly satisfied generated contract")),
        Err(_) => Ok(()),
    }
}

fn validate_doctor_payload(value: &Value) -> Result<(), String> {
    let object = value.as_object().ok_or_else(|| "doctor report must be a JSON object".to_string())?;
    for key in object.keys() {
        if !matches!(key.as_str(), "schema" | "command" | "profile" | "ok" | "checks") {
            return Err(format!("doctor report has unexpected field `{key}`"));
        }
    }
    require_string_field(value, "schema", DOCTOR_SCHEMA_VALUE)?;
    require_string_field(value, "command", DOCTOR_COMMAND_VALUE)?;
    require_one_of(value, "profile", &[EXPECTED_PROFILE_BUILD, EXPECTED_PROFILE_SELF_BUILD])?;
    require_bool_field(value, "ok")?;
    let checks = value
        .get("checks")
        .and_then(Value::as_array)
        .ok_or_else(|| "doctor report checks must be an array".to_string())?;
    if checks.is_empty() {
        return Err("doctor report checks must not be empty".to_string());
    }
    for (index, check) in checks.iter().enumerate() {
        validate_doctor_check(index, check)?;
    }
    Ok(())
}

fn validate_doctor_check(index: usize, value: &Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("doctor check {index} must be an object"))?;
    for key in object.keys() {
        if !matches!(key.as_str(), "id" | "status" | "summary" | "detail") {
            return Err(format!("doctor check {index} has unexpected field `{key}`"));
        }
    }
    require_non_empty_string(value, "id")?;
    require_one_of(value, "status", &[EXPECTED_CHECK_STATUS_OK, EXPECTED_CHECK_STATUS_FAILED])?;
    require_non_empty_string(value, "summary")?;
    if let Some(detail) = value.get("detail") {
        if !detail.is_string() {
            return Err(format!("doctor check {index} detail must be a string"));
        }
    }
    Ok(())
}

fn render_doctor_contract_snapshot() -> String {
    concat!(
        "# Generated by scripts/check-machine-schema-contracts.rs from doctor-report.schema.json.\n",
        "# Do not edit by hand; update the schema and rerun the checker.\n",
        "let DoctorStatus = [| 'ok, 'failed |] in\n",
        "let DoctorProfile = [| 'build, 'self-build |] in\n",
        "let DoctorCheck = {\n",
        "  id | String,\n",
        "  status | DoctorStatus,\n",
        "  summary | String,\n",
        "  detail | String | optional,\n",
        "} in\n",
        "{\n",
        "  schema | force = \"crunch-doctor-report-v1\",\n",
        "  command | force = \"doctor\",\n",
        "  profile | DoctorProfile,\n",
        "  ok | Bool,\n",
        "  checks | Array DoctorCheck,\n",
        "}\n",
    )
    .to_string()
}

fn require_string_field(value: &Value, field: &str, expected: &str) -> Result<(), String> {
    let actual = value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field `{field}`"))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("field `{field}` expected `{expected}`, got `{actual}`"))
    }
}

fn require_non_empty_string(value: &Value, field: &str) -> Result<(), String> {
    let actual = value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field `{field}`"))?;
    if actual.is_empty() {
        Err(format!("field `{field}` must not be empty"))
    } else {
        Ok(())
    }
}

fn require_one_of(value: &Value, field: &str, allowed: &[&str]) -> Result<(), String> {
    let actual = value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field `{field}`"))?;
    if allowed.contains(&actual) {
        Ok(())
    } else {
        Err(format!("field `{field}` has unsupported value `{actual}`"))
    }
}

fn require_bool_field(value: &Value, field: &str) -> Result<(), String> {
    value
        .get(field)
        .and_then(Value::as_bool)
        .map(|_| ())
        .ok_or_else(|| format!("missing bool field `{field}`"))
}

fn read_text(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|err| format!("reading {path}: {err}"))
}

fn read_json(path: &str) -> Result<Value, String> {
    let text = read_text(path)?;
    serde_json::from_str(&text).map_err(|err| format!("parsing {path}: {err}"))
}

fn blake3_file_hex(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|err| format!("reading {}: {err}", path.display()))?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    if digest.len() != EXPECTED_BLAKE3_HEX_BYTES {
        return Err(format!("unexpected BLAKE3 digest length for {}", path.display()));
    }
    Ok(digest)
}
