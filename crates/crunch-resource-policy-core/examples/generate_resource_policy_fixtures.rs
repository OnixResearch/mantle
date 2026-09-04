#![feature(register_tool)]
#![register_tool(tigerstyle)]

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crunch_resource_policy_core::*;
use serde::Serialize;
use serde_json::Value;

const OBSERVED_AT_UNIX_S: u64 = 4_999_940;
const CPU_UNITS: u32 = 2;
const MEMORY_BYTES: u64 = 1_073_741_824;
const LARGE_MEMORY_BYTES: u64 = 2_147_483_648;
const SCRATCH_BYTES: u64 = 2_147_483_648;
const PEAK_MEMORY_BYTES: u64 = 805_306_368;
const IO_BYTES: u64 = 1_048_576;
const TRANSFER_BYTES: u64 = 524_288;
const QUEUE_MS: u64 = 50;
const EXECUTION_MS: u64 = 1_000;
const TERMINAL_MS: u64 = 10;
const CPU_TIME_MS: u64 = 800;
const FIXTURE_LATENCY_MS_MAX: u64 = 5_000;
const FIXTURE_MEMORY_BYTES_MAX: u64 = 2_147_483_648;
const FIXTURE_TRANSFER_BYTES_MAX: u64 = 1_073_741_824;
const FIXTURE_USAGE_UNITS_MAX: u64 = 10_000;
const FIXTURE_THROUGHPUT_UNITS: u64 = 1;
const FIXTURE_USAGE_UNITS: u64 = 2_000;
const SMALL_CLASS_ORDINAL: u32 = 1;
const LARGE_CLASS_ORDINAL: u32 = 2;
const SMALL_CLASS_CHARGE_UNITS: u64 = 100;
const LARGE_CLASS_CHARGE_UNITS: u64 = 200;
const INVALID_VERSION: &str = "v0";
const OVERSIZED_FEATURE_COUNT: usize = 65;

const NIXBENCH_REPOSITORY: &str = "github.com/nixbuild/nixbench";
const NIXBENCH_REVISION: &str = "b256cd275d8c79ba485be8d317005f973879825a";
const NIXBENCH_SOURCE_BLAKE3: &str = "737a5a42181fdd59ba451159db7558c516f31ea82ffcc771f66a0f2d6421a5a8";
const NIXBENCH_README_BLAKE3: &str = "95bbe021e75cbd2caf651604fa9b70adb23ad27845d49bc3c68a56b8ac829279";
const NIXBENCH_LICENSE_BLAKE3: &str = "14ec1590aae4c4e763d123c31085d5d37703f7a838e589b0d356058ad99177dd";
const CHAOSCONTROL_REVISION: &str = "31300fa1a2d29c7496e8316f065c156f80343143";
const CHAOSCONTROL_CONTRACT_BLAKE3: &str = "3e75d1dfb4dd8cb91e6b448aef3fbe4bcafe6c1bedf31b10d0e3bd7a18dd5e32";
const ONIXOS_REVISION: &str = "8c7f0155492118c98ada55162f984dc24c00e150";
const ONIXOS_INVENTORY_BLAKE3: &str = "ce9bdc96706f7b5c2e514ea33e65f897877e126c2ea75f2a70dc98184c7ccee8";

#[derive(Serialize)]
struct NegativeFixtureSet {
    surface_id: String,
    cases: Vec<NegativeCase>,
}

#[derive(Serialize)]
struct NegativeCase {
    id: String,
    issue_class: String,
    expected_path: String,
    artifact: Value,
}

#[derive(Serialize)]
struct BenchmarkCorpus {
    schema: String,
    baseline_policy_id: String,
    candidate_policy_id: String,
    workloads: Vec<ResourceBenchmarkWorkload>,
    observations: Vec<ResourceBenchmarkObservation>,
    non_claim: String,
}

#[derive(Serialize)]
struct SourceReview {
    schema: String,
    repository: String,
    revision: String,
    tree_path: String,
    source_blake3: String,
    readme_blake3: String,
    license: String,
    license_blake3: String,
    copied_source: bool,
    adapted_pattern: String,
    excluded_authority: Vec<String>,
}

#[derive(Serialize)]
struct OnixMachineClassFixture {
    schema: String,
    repository: String,
    revision: String,
    contract_path: String,
    contract_blake3: String,
    onixos_machine_class: String,
    classes: Vec<MachineClass>,
    non_claim: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::args().nth(1).map_or(std::env::current_dir()?, PathBuf::from);
    assert!(!root.as_os_str().is_empty(), "fixture root must not be empty");
    assert!(root.is_dir(), "fixture root must exist before generation");
    let action = action_family()?;
    let observation = observation(&action)?;
    write_json(&root.join("schemas/machine-contracts/fixtures/resource-action-family.valid.json"), &action)?;
    write_json(
        &root.join("schemas/machine-contracts/fixtures/resource-action-family.negatives.json"),
        &action_negatives(&action)?,
    )?;
    write_json(&root.join("schemas/machine-contracts/fixtures/resource-observation.valid.json"), &observation)?;
    write_json(
        &root.join("schemas/machine-contracts/fixtures/resource-observation.negatives.json"),
        &observation_negatives(&observation)?,
    )?;
    let (workloads, observations) = benchmark_fixture();
    let benchmark_evidence = build_resource_benchmark_report(
        workloads.clone(),
        observations.clone(),
        "static-v1".to_string(),
        "evidence-v1".to_string(),
    )?;
    write_json(&root.join("fixtures/resource-policy/benchmark-corpus.json"), &BenchmarkCorpus {
        schema: "mantle-resource-benchmark-corpus-v1".to_string(),
        baseline_policy_id: "static-v1".to_string(),
        candidate_policy_id: "evidence-v1".to_string(),
        workloads,
        observations,
        non_claim: RESOURCE_POLICY_NON_CLAIM.to_string(),
    })?;
    write_json(&root.join("fixtures/resource-policy/benchmark-report.valid.json"), &benchmark_evidence)?;
    write_json(&root.join("fixtures/resource-policy/chaoscontrol-campaign.valid.json"), &fault_campaign())?;
    write_json(&root.join("fixtures/resource-policy/nixbench-source-review.json"), &source_review())?;
    write_json(&root.join("fixtures/resource-policy/onixos-machine-classes.valid.json"), &onixos_fixture())?;
    Ok(())
}

fn action_family() -> Result<ActionFamilyIdentity, ResourcePolicyError> {
    derive_action_family_identity(ActionFamilyInput {
        request_kind: "derivation".to_string(),
        system: "x86_64-linux".to_string(),
        builder_class: "rust".to_string(),
        sandbox_mode: "microvm".to_string(),
        network_mode: "none".to_string(),
        required_features: vec!["avx2".to_string(), "kvm".to_string()],
        semantic_accelerator_classes: Vec::new(),
    })
}

fn observation(action: &ActionFamilyIdentity) -> Result<ResourceObservation, ResourcePolicyError> {
    build_resource_observation(ResourceObservationInput {
        attempt_id: "attempt-resource-fixture".to_string(),
        action_family_blake3: action.identity_blake3.clone(),
        platform_identity: "x86_64-linux".to_string(),
        machine_class_id: "onixos-small".to_string(),
        declared: quantities(),
        selected: quantities(),
        timing: ResourceTiming {
            queue_ms: QUEUE_MS,
            execution_ms: EXECUTION_MS,
            terminal_ms: TERMINAL_MS,
        },
        measurements: ResourceMeasurements {
            cpu_time_ms: CPU_TIME_MS,
            peak_memory_bytes: PEAK_MEMORY_BYTES,
            scratch_peak_bytes: SCRATCH_BYTES,
            io_bytes: IO_BYTES,
            transfer_bytes: TRANSFER_BYTES,
            wall_time_ms: EXECUTION_MS,
        },
        oom_evidence: OomEvidence {
            category: OomEvidenceCategory::None,
            platform: "x86_64-linux".to_string(),
            evidence_ref: None,
            trusted: false,
        },
        terminal_outcome: TerminalOutcome::Succeeded,
        retry_predecessor_attempt_id: None,
        collector_id: "mantle-worker".to_string(),
        collector_version: "v1".to_string(),
        compatibility_policy_id: "mantle-resource-selection-default".to_string(),
        observed_at_unix_s: OBSERVED_AT_UNIX_S,
        trusted: true,
    })
}

fn quantities() -> ResourceQuantities {
    ResourceQuantities {
        cpu_units: CPU_UNITS,
        memory_bytes: MEMORY_BYTES,
        scratch_bytes: SCRATCH_BYTES,
    }
}

fn action_negatives(action: &ActionFamilyIdentity) -> Result<NegativeFixtureSet, serde_json::Error> {
    assert_eq!(action.schema, ACTION_FAMILY_SCHEMA);
    assert_eq!(action.identity_blake3.len(), BLAKE3_HEX_CHARS);
    let mut wrong_schema_case = serde_json::to_value(action)?;
    wrong_schema_case["schema"] = Value::String(format!("mantle-resource-action-family-{INVALID_VERSION}"));
    let mut digest_case = serde_json::to_value(action)?;
    digest_case["identity_blake3"] = Value::String("not-a-digest".to_string());
    let mut feature_bound_case = serde_json::to_value(action)?;
    feature_bound_case["required_features"] =
        Value::Array((0..OVERSIZED_FEATURE_COUNT).map(|index| Value::String(format!("feature-{index}"))).collect());
    let mut unknown = serde_json::to_value(action)?;
    unknown["environment"] = serde_json::json!({"TOKEN": "forbidden"});
    Ok(NegativeFixtureSet {
        surface_id: "resource.action-family".to_string(),
        cases: vec![
            negative("missing-schema", "schema", "/schema", serde_json::json!({})),
            negative("unsupported-version", "version", "/schema", wrong_schema_case),
            negative("malformed-digest", "digest", "/identity_blake3", digest_case),
            negative("feature-bound", "bounds", "/required_features", feature_bound_case),
            negative("unknown-field", "unknown-field", "/environment", unknown),
        ],
    })
}

fn observation_negatives(observation: &ResourceObservation) -> Result<NegativeFixtureSet, serde_json::Error> {
    assert_eq!(observation.schema, RESOURCE_OBSERVATION_SCHEMA);
    assert_eq!(observation.observation_blake3.len(), BLAKE3_HEX_CHARS);
    let mut wrong_schema_case = serde_json::to_value(observation)?;
    wrong_schema_case["schema"] = Value::String(format!("mantle-resource-observation-{INVALID_VERSION}"));
    let mut digest_case = serde_json::to_value(observation)?;
    digest_case["observation_blake3"] = Value::String("not-a-digest".to_string());
    let mut measurement_bound_case = serde_json::to_value(observation)?;
    measurement_bound_case["measurements"]["peak_memory_bytes"] = Value::from(MAX_MEASUREMENT_BYTES.saturating_add(1));
    let mut unknown = serde_json::to_value(observation)?;
    unknown["raw_log"] = Value::String("forbidden".to_string());
    Ok(NegativeFixtureSet {
        surface_id: "resource.resource-observation".to_string(),
        cases: vec![
            negative("missing-schema", "schema", "/schema", serde_json::json!({})),
            negative("unsupported-version", "version", "/schema", wrong_schema_case),
            negative("malformed-digest", "digest", "/observation_blake3", digest_case),
            negative("measurement-bound", "bounds", "/measurements/peak_memory_bytes", measurement_bound_case),
            negative("unknown-field", "unknown-field", "/raw_log", unknown),
        ],
    })
}

#[allow(
    tigerstyle::ambiguous_params,
    reason = "fixture ID, issue class, and JSON pointer are distinct named fields"
)]
fn negative(id: &str, issue_class: &str, expected_path: &str, artifact: Value) -> NegativeCase {
    NegativeCase {
        id: id.to_string(),
        issue_class: issue_class.to_string(),
        expected_path: expected_path.to_string(),
        artifact,
    }
}

fn benchmark_fixture() -> (Vec<ResourceBenchmarkWorkload>, Vec<ResourceBenchmarkObservation>) {
    let source = BenchmarkSource {
        repository: NIXBENCH_REPOSITORY.to_string(),
        revision: NIXBENCH_REVISION.to_string(),
        source_blake3: NIXBENCH_SOURCE_BLAKE3.to_string(),
        license: "Apache-2.0".to_string(),
        license_blake3: NIXBENCH_LICENSE_BLAKE3.to_string(),
        adaptation: "fixed-seed-size-compressibility-and-cpu-shape-only".to_string(),
    };
    let workload = ResourceBenchmarkWorkload {
        workload_id: "fixed-output-write".to_string(),
        source,
        fixed_input_blake3: digest('a'),
        platform: platform(),
        expected_completion_class: "succeeded".to_string(),
        latency_ms_max: FIXTURE_LATENCY_MS_MAX,
        memory_bytes_max: FIXTURE_MEMORY_BYTES_MAX,
        transfer_bytes_max: FIXTURE_TRANSFER_BYTES_MAX,
        usage_units_max: FIXTURE_USAGE_UNITS_MAX,
    };
    let observation = ResourceBenchmarkObservation {
        workload_id: workload.workload_id.clone(),
        static_class_id: "onixos-small".to_string(),
        policy_class_id: "onixos-small".to_string(),
        completion_class: "succeeded".to_string(),
        compatible: true,
        correct_output: true,
        throughput_units: FIXTURE_THROUGHPUT_UNITS,
        latency_ms: EXECUTION_MS,
        memory_bytes: PEAK_MEMORY_BYTES,
        transfer_bytes: TRANSFER_BYTES,
        usage_units: FIXTURE_USAGE_UNITS,
    };
    assert_eq!(workload.workload_id, observation.workload_id);
    assert!(observation.latency_ms <= workload.latency_ms_max);
    (vec![workload], vec![observation])
}

fn platform() -> PlatformRequirements {
    PlatformRequirements {
        architecture: "x86_64".to_string(),
        platform: "x86_64-linux".to_string(),
        kvm_required: true,
        trust_tier: "trusted-builder".to_string(),
        isolation: "microvm".to_string(),
        required_features: vec!["avx2".to_string(), "kvm".to_string()],
    }
}

fn fault_campaign() -> ResourceFaultCampaign {
    ResourceFaultCampaign {
        schema: RESOURCE_FAULT_CAMPAIGN_SCHEMA.to_string(),
        chaoscontrol_revision: CHAOSCONTROL_REVISION.to_string(),
        chaoscontrol_contract_path: "crates/chaoscontrol-sim-core/src/runtime_capacity.rs".to_string(),
        chaoscontrol_contract_blake3: CHAOSCONTROL_CONTRACT_BLAKE3.to_string(),
        cases: vec![
            fault(ResourceFaultKind::PositiveOom, "positive-oom", "positive-oom-evidence", true),
            fault(ResourceFaultKind::AmbiguousFailure, "ambiguous", "oom-evidence-missing", false),
            fault(ResourceFaultKind::WorkerLoss, "worker-loss", "usage-worker-lost-reservation-retained", true),
            fault(ResourceFaultKind::DuplicateCompletion, "duplicate", "usage-reconciliation-reused", false),
            fault(ResourceFaultKind::AccountingInterruption, "accounting", "accounting-store-failed", false),
            fault(ResourceFaultKind::CasUnavailable, "cas", "result-cas-unavailable", false),
        ],
        non_claim: RESOURCE_POLICY_NON_CLAIM.to_string(),
    }
}

#[allow(
    tigerstyle::ambiguous_params,
    reason = "fixture case ID and reason code are distinct named fields"
)]
fn fault(kind: ResourceFaultKind, id: &str, reason: &str, mutation: bool) -> ResourceFaultCase {
    ResourceFaultCase {
        case_id: id.to_string(),
        kind,
        expected_reason_code: reason.to_string(),
        expected_mutation: mutation,
    }
}

fn source_review() -> SourceReview {
    SourceReview {
        schema: "mantle-resource-benchmark-source-review-v1".to_string(),
        repository: NIXBENCH_REPOSITORY.to_string(),
        revision: NIXBENCH_REVISION.to_string(),
        tree_path: "nix/packages/write-one-file.nix".to_string(),
        source_blake3: NIXBENCH_SOURCE_BLAKE3.to_string(),
        readme_blake3: NIXBENCH_README_BLAKE3.to_string(),
        license: "Apache-2.0".to_string(),
        license_blake3: NIXBENCH_LICENSE_BLAKE3.to_string(),
        copied_source: false,
        adapted_pattern: "separate fixed rebuild seed, content seed, size, compressibility, and CPU facts".to_string(),
        excluded_authority: vec![
            "ambient environment".to_string(),
            "hosted-service tags".to_string(),
            "hosted-service API".to_string(),
            "unbounded parallelism".to_string(),
            "random runtime bytes".to_string(),
        ],
    }
}

fn onixos_fixture() -> OnixMachineClassFixture {
    OnixMachineClassFixture {
        schema: "mantle-onixos-resource-machine-classes-v1".to_string(),
        repository: "github.com/onixcomputer/onix-modules".to_string(),
        revision: ONIXOS_REVISION.to_string(),
        contract_path: "lib/inventory.ncl".to_string(),
        contract_blake3: ONIXOS_INVENTORY_BLAKE3.to_string(),
        onixos_machine_class: "nixos".to_string(),
        classes: vec![
            machine_class(
                "onixos-small",
                "worker-small",
                SMALL_CLASS_ORDINAL,
                MEMORY_BYTES,
                SMALL_CLASS_CHARGE_UNITS,
            ),
            machine_class(
                "onixos-large",
                "worker-large",
                LARGE_CLASS_ORDINAL,
                LARGE_MEMORY_BYTES,
                LARGE_CLASS_CHARGE_UNITS,
            ),
        ],
        non_claim: "OnixOS inventory identity supplies declared machine-class provenance but does not prove worker availability, capacity, trust, or execution success".to_string(),
    }
}

#[allow(
    tigerstyle::ambiguous_params,
    reason = "fixture class and endpoint IDs are distinct named fields"
)]
fn machine_class(
    class_id: &str,
    endpoint_id: &str,
    ordinal: u32,
    memory_bytes: u64,
    charge_units: u64,
) -> MachineClass {
    MachineClass {
        class_id: class_id.to_string(),
        endpoint_ids: vec![endpoint_id.to_string()],
        ordinal,
        architecture: "x86_64".to_string(),
        platform: "x86_64-linux".to_string(),
        kvm_available: true,
        trust_tier: "trusted-builder".to_string(),
        isolation: "microvm".to_string(),
        features: vec!["avx2".to_string(), "kvm".to_string()],
        capacity: ResourceQuantities {
            cpu_units: CPU_UNITS,
            memory_bytes,
            scratch_bytes: SCRATCH_BYTES,
        },
        reservation_charge_units: charge_units,
        available: true,
        onixos_source_blake3: ONIXOS_INVENTORY_BLAKE3.to_string(),
    }
}

fn digest(byte: char) -> String {
    std::iter::repeat_n(byte, BLAKE3_HEX_CHARS).collect()
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), Box<dyn std::error::Error>> {
    let parent = path.parent().ok_or("fixture output has no parent")?;
    fs::create_dir_all(parent)?;
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}
