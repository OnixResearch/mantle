//! Pure decoding and binding of static inputs to accepted native plan roots.
//! No unit output has to be built to resolve the selected root derivation.

use std::collections::BTreeMap;

use nix_compat::store_path::StorePath;
use nix_compat::store_path::hash_placeholder;
use serde::Deserialize;

use crate::dynamic_plan::OutputName;
use crate::dynamic_plan::UnitId;
use crate::worker::CanonicalNativePlan;

pub const MAX_PLAN_OUTPUT_REFERENCES: usize = 16;
pub const PLAN_OUTPUT_BINDINGS_ENV_KEY: &str = "__MANTLE_PLAN_OUTPUT_BINDINGS";
pub const PLAN_OUTPUT_PROVENANCE_CLAIM_KEY: &str = "mantle.plan_output_bindings";
const MAX_BINDING_RECORD_BYTES: usize = 1024;

/// One checked reference to a root of a producer's declared plan output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanOutputBindingRecord {
    pub name: String,
    pub producer_drv_path: StorePath<String>,
    pub plan_output: OutputName,
    pub root: UnitId,
    pub unit_output: OutputName,
    pub placeholder: String,
}

/// The selected root's derivation, not its as-yet-unavailable output store path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundPlanRoot {
    pub drv_path: StorePath<String>,
    pub output: OutputName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PlanOutputBindingFailureKind {
    #[error("plan-output-invalid")]
    Invalid,
    #[error("plan-output-bounded")]
    Bounded,
    #[error("plan-output-unbound")]
    Unbound,
    #[error("plan-output-producer-failed")]
    ProducerFailed,
    #[error("plan-output-plan-rejected")]
    PlanRejected,
    #[error("plan-output-undeclared")]
    Undeclared,
    #[error("plan-output-root-missing")]
    RootMissing,
    #[error("plan-output-output-missing")]
    OutputMissing,
    #[error("plan-output-root-failed")]
    RootFailed,
}

impl PlanOutputBindingFailureKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Invalid => "plan-output-invalid",
            Self::Bounded => "plan-output-bounded",
            Self::Unbound => "plan-output-unbound",
            Self::ProducerFailed => "plan-output-producer-failed",
            Self::PlanRejected => "plan-output-plan-rejected",
            Self::Undeclared => "plan-output-undeclared",
            Self::RootMissing => "plan-output-root-missing",
            Self::OutputMissing => "plan-output-output-missing",
            Self::RootFailed => "plan-output-root-failed",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireBindingRecord<'a> {
    name: &'a str,
    producer_drv_path: &'a str,
    plan_output: &'a str,
    root: &'a str,
    unit_output: &'a str,
    placeholder: &'a str,
}

fn valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 64
        && bytes[0].is_ascii_lowercase()
        && bytes[1..]
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-'))
}

/// Decode a bounded, closed array of name-ordered references under the active
/// logical store prefix. The path and marker are checked, not trusted from JSON.
pub fn decode_binding_records(
    bytes: &[u8],
    store_dir: &str,
) -> Result<Vec<PlanOutputBindingRecord>, PlanOutputBindingFailureKind> {
    if !store_dir.starts_with('/')
        || store_dir.ends_with('/')
        || store_dir[1..]
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(PlanOutputBindingFailureKind::Invalid);
    }
    // The only prefix-dependent field is its absolute store path. Reject
    // oversized JSON before deserializing; canonical records fit this bound.
    let max_bytes = MAX_PLAN_OUTPUT_REFERENCES
        .saturating_mul(store_dir.len().saturating_add(MAX_BINDING_RECORD_BYTES))
        .saturating_add(2);
    if bytes.len() > max_bytes {
        return Err(PlanOutputBindingFailureKind::Bounded);
    }
    let wire: Vec<WireBindingRecord<'_>> =
        serde_json::from_slice(bytes).map_err(|_| PlanOutputBindingFailureKind::Invalid)?;
    if wire.len() > MAX_PLAN_OUTPUT_REFERENCES {
        return Err(PlanOutputBindingFailureKind::Bounded);
    }
    let mut records: Vec<PlanOutputBindingRecord> = Vec::with_capacity(wire.len());
    for row in wire {
        if !valid_name(row.name) || records.last().is_some_and(|previous| previous.name.as_str() >= row.name) {
            return Err(PlanOutputBindingFailureKind::Invalid);
        }
        let expected_placeholder = hash_placeholder(&format!("mantle-plan-output:{}", row.name));
        if row.placeholder != expected_placeholder {
            return Err(PlanOutputBindingFailureKind::Invalid);
        }
        let producer_drv_path =
            StorePath::<String>::from_absolute_path_with_prefix(row.producer_drv_path.as_bytes(), store_dir)
                .map_err(|_| PlanOutputBindingFailureKind::Invalid)?;
        if !producer_drv_path.name().ends_with(".drv") {
            return Err(PlanOutputBindingFailureKind::Invalid);
        }
        let plan_output = OutputName::new(row.plan_output).map_err(|_| PlanOutputBindingFailureKind::Invalid)?;
        let root = UnitId::new(row.root).map_err(|_| PlanOutputBindingFailureKind::Invalid)?;
        let unit_output = OutputName::new(row.unit_output).map_err(|_| PlanOutputBindingFailureKind::Invalid)?;
        records.push(PlanOutputBindingRecord {
            name: row.name.to_owned(),
            producer_drv_path,
            plan_output,
            root,
            unit_output,
            placeholder: row.placeholder.to_owned(),
        });
    }
    debug_assert!(records.len() <= MAX_PLAN_OUTPUT_REFERENCES);
    debug_assert!(records.windows(2).all(|pair| pair[0].name < pair[1].name), "records are strictly name-ordered");
    Ok(records)
}

/// Resolve a declared root from the accepted plan itself before any unit output
/// is available. A missing root wins over a missing output or registration fact.
// r[impl mantle.dynamic_plan_output_inputs.binding_failures]
pub fn bind_accepted_plan_root(
    record: &PlanOutputBindingRecord,
    plan: &CanonicalNativePlan,
    registered_unit_paths: &BTreeMap<UnitId, StorePath<String>>,
) -> Result<BoundPlanRoot, PlanOutputBindingFailureKind> {
    let (roots, units) = match plan {
        CanonicalNativePlan::V1(plan) => (&plan.plan.roots, &plan.plan.units),
        CanonicalNativePlan::V2(plan) => (&plan.plan.roots, &plan.plan.units),
    };
    if !roots.contains(&record.root) {
        return Err(PlanOutputBindingFailureKind::RootMissing);
    }
    let root_unit =
        units.iter().find(|unit| unit.id == record.root).ok_or(PlanOutputBindingFailureKind::RootMissing)?;
    if !root_unit.derivation.outputs.contains(&record.unit_output) {
        return Err(PlanOutputBindingFailureKind::OutputMissing);
    }
    let drv_path = registered_unit_paths.get(&record.root).ok_or(PlanOutputBindingFailureKind::RootFailed)?;
    Ok(BoundPlanRoot {
        drv_path: drv_path.clone(),
        output: record.unit_output.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic_plan::decode_validated_plan_v1;
    use crate::dynamic_plan::decode_validated_plan_v2;

    const STORE_DIR: &str = "/nix/store";
    const PRODUCER: &str = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-producer.drv";
    const BUILDER: &str = "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-builder";

    fn row(name: &str) -> serde_json::Value {
        serde_json::json!({
            "name": name,
            "producer_drv_path": PRODUCER,
            "plan_output": "plan",
            "root": "unit.app",
            "unit_output": "out",
            "placeholder": hash_placeholder(&format!("mantle-plan-output:{name}")),
        })
    }

    fn decode(rows: &[serde_json::Value]) -> Result<Vec<PlanOutputBindingRecord>, PlanOutputBindingFailureKind> {
        decode_binding_records(&serde_json::to_vec(rows).unwrap(), STORE_DIR)
    }

    fn accepted(version: u8) -> CanonicalNativePlan {
        let mut json = serde_json::json!({
            "schema": "mantle-plan-v1",
            "producer": { "logical_name": "producer", "goal_hint": null },
            "sources": [],
            "units": [{
                "id": "unit.app",
                "derivation": {
                    "name": "unit-app", "builder": BUILDER, "system": "x86_64-linux",
                    "args": [], "outputs": ["out", "dev"], "env": {}, "inputs": [],
                    "fixed_output": null, "addressing_mode": "content-addressed",
                    "sandbox": "native", "dynamic_plan_outputs": []
                },
                "requested_outputs": ["out"],
                "policy": {
                    "sandbox": "inherit", "substitutions": "inherit",
                    "store_prefix": "inherit", "host_paths": "none"
                }
            }],
            "roots": ["unit.app"],
            "provenance": {}
        });
        if version == 2 {
            json["schema"] = serde_json::json!("mantle-plan-v2");
        }
        let bytes = serde_json::to_vec(&json).unwrap();
        match version {
            1 => CanonicalNativePlan::V1(decode_validated_plan_v1(&bytes, STORE_DIR).unwrap()),
            2 => CanonicalNativePlan::V2(decode_validated_plan_v2(&bytes, STORE_DIR).unwrap()),
            _ => unreachable!(),
        }
    }

    #[test]
    fn binds_selected_output_of_accepted_root_in_v1_and_v2_before_output_exists() {
        let record = decode(&[row("app")]).unwrap().remove(0);
        let path = StorePath::from_absolute_path_with_prefix(PRODUCER.as_bytes(), STORE_DIR).unwrap();
        let registered = BTreeMap::from([(record.root.clone(), path.clone())]);
        for version in [1, 2] {
            let plan = accepted(version);
            let bound = bind_accepted_plan_root(&record, &plan, &registered).unwrap();
            assert_eq!(bound.drv_path, path);
            assert_eq!(bound.output.as_str(), "out");
        }
        let mut selected = record.clone();
        selected.unit_output = OutputName::new("dev").unwrap();
        assert_eq!(bind_accepted_plan_root(&selected, &accepted(2), &registered).unwrap().output.as_str(), "dev");
    }

    #[test]
    // r[verify mantle.dynamic_plan_output_inputs.binding_failures]
    fn missing_root_precedes_missing_output_and_registered_path() {
        let mut record = decode(&[row("app")]).unwrap().remove(0);
        record.root = UnitId::new("unit.other").unwrap();
        record.unit_output = OutputName::new("absent").unwrap();
        for version in [1, 2] {
            assert_eq!(
                bind_accepted_plan_root(&record, &accepted(version), &BTreeMap::new()),
                Err(PlanOutputBindingFailureKind::RootMissing)
            );
        }
    }

    #[test]
    fn missing_output_precedes_missing_registered_path() {
        let mut record = decode(&[row("app")]).unwrap().remove(0);
        record.unit_output = OutputName::new("absent").unwrap();
        assert_eq!(
            bind_accepted_plan_root(&record, &accepted(1), &BTreeMap::new()),
            Err(PlanOutputBindingFailureKind::OutputMissing)
        );
        record.unit_output = OutputName::new("out").unwrap();
        assert_eq!(
            bind_accepted_plan_root(&record, &accepted(1), &BTreeMap::new()),
            Err(PlanOutputBindingFailureKind::RootFailed)
        );
    }

    #[test]
    fn records_require_sorted_unique_names_and_exact_closed_scalar_fields() {
        assert_eq!(decode(&[row("z"), row("a")]), Err(PlanOutputBindingFailureKind::Invalid));
        let sorted = decode(&[row("a"), row("z")]).unwrap();
        assert_eq!(sorted.iter().map(|record| record.name.as_str()).collect::<Vec<_>>(), ["a", "z"]);
        assert_eq!(decode(&[row(&"a".repeat(64))]).unwrap()[0].name.len(), 64);
        assert_eq!(decode(&[row(&"a".repeat(65))]), Err(PlanOutputBindingFailureKind::Invalid));
        assert_eq!(decode(&[row("a"), row("a")]), Err(PlanOutputBindingFailureKind::Invalid));
        let mut bad = row("app");
        bad["unknown"] = serde_json::json!(true);
        assert_eq!(decode(&[bad]), Err(PlanOutputBindingFailureKind::Invalid));
        let mut bad = row("app");
        bad["root"] = serde_json::json!("Unit.app");
        assert_eq!(decode(&[bad]), Err(PlanOutputBindingFailureKind::Invalid));
        let mut bad = row("app");
        bad["plan_output"] = serde_json::json!("Bad");
        assert_eq!(decode(&[bad]), Err(PlanOutputBindingFailureKind::Invalid));
        assert_eq!(decode(&[row("App")]), Err(PlanOutputBindingFailureKind::Invalid));
        let mut bad = row("app");
        bad["placeholder"] = serde_json::json!("/wrong");
        assert_eq!(decode(&[bad]), Err(PlanOutputBindingFailureKind::Invalid));
        let mut bad = row("app");
        bad["producer_drv_path"] = serde_json::json!("/other/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-producer.drv");
        assert_eq!(decode(&[bad]), Err(PlanOutputBindingFailureKind::Invalid));
        let mut not_a_drv = row("app");
        not_a_drv["producer_drv_path"] = serde_json::json!("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-producer");
        assert_eq!(decode(&[not_a_drv]), Err(PlanOutputBindingFailureKind::Invalid));
        assert_eq!(
            decode_binding_records(&serde_json::to_vec(&[row("app")]).unwrap(), "relative/store"),
            Err(PlanOutputBindingFailureKind::Invalid)
        );
        let duplicate = format!(
            "[{{\"name\":\"app\",\"name\":\"app\",\"producer_drv_path\":\"{PRODUCER}\",\"plan_output\":\"plan\",\"root\":\"unit.app\",\"unit_output\":\"out\",\"placeholder\":\"{}\"}}]",
            hash_placeholder("mantle-plan-output:app")
        );
        assert_eq!(decode_binding_records(duplicate.as_bytes(), STORE_DIR), Err(PlanOutputBindingFailureKind::Invalid));
    }

    #[test]
    fn records_reject_count_and_byte_bound_before_large_json_parse() {
        let valid_rows = (0..MAX_PLAN_OUTPUT_REFERENCES).map(|n| row(&format!("app{n:02}"))).collect::<Vec<_>>();
        assert_eq!(decode(&valid_rows).unwrap().len(), MAX_PLAN_OUTPUT_REFERENCES);
        let rows = (0..=MAX_PLAN_OUTPUT_REFERENCES).map(|n| row(&format!("app{n}"))).collect::<Vec<_>>();
        assert_eq!(decode(&rows), Err(PlanOutputBindingFailureKind::Bounded));
        let bytes = vec![b' '; MAX_PLAN_OUTPUT_REFERENCES * (STORE_DIR.len() + MAX_BINDING_RECORD_BYTES) + 3];
        assert_eq!(decode_binding_records(&bytes, STORE_DIR), Err(PlanOutputBindingFailureKind::Bounded));
    }
}
