use std::fs;
use std::path::Path;

use crunch_resource_policy::*;
use serde::Deserialize;
use serde_json::Value;

const EXPECTED_VALENCE_REVISION: &str = "e40c76b4d2070a29636e00c85c0dff93f03dba2f";
const EXPECTED_NIXBENCH_REVISION: &str = "b256cd275d8c79ba485be8d317005f973879825a";
const EXPECTED_ONIXOS_REVISION: &str = "8c7f0155492118c98ada55162f984dc24c00e150";
const EXPECTED_CHAOSCONTROL_REVISION: &str = "31300fa1a2d29c7496e8316f065c156f80343143";
const NOW_UNIX_S: u64 = 5_000_000;
const QUOTA_UNITS: u64 = 1_000_000;
const EXPECTED_EXCLUDED_AUTHORITY_COUNT_MIN: usize = 5;
const EXPECTED_FAULT_CASE_COUNT: usize = 6;

#[derive(Deserialize)]
struct NegativeFixtureSet {
    surface_id: String,
    cases: Vec<NegativeCase>,
}

#[derive(Deserialize)]
struct NegativeCase {
    id: String,
    artifact: Value,
}

#[derive(Deserialize)]
struct BenchmarkCorpus {
    schema: String,
    baseline_policy_id: String,
    candidate_policy_id: String,
    workloads: Vec<ResourceBenchmarkWorkload>,
    observations: Vec<ResourceBenchmarkObservation>,
    non_claim: String,
}

#[derive(Deserialize)]
struct OnixMachineClassFixture {
    schema: String,
    revision: String,
    contract_blake3: String,
    classes: Vec<MachineClass>,
    non_claim: String,
}

fn fixture(relative: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

#[test]
fn contracted_resource_fixtures_accept_positive_and_reject_every_negative() {
    let action: ActionFamilyIdentity =
        serde_json::from_slice(&fixture("schemas/machine-contracts/fixtures/resource-action-family.valid.json"))
            .unwrap();
    validate_action_family_identity(action).unwrap();
    let observation: ResourceObservation =
        serde_json::from_slice(&fixture("schemas/machine-contracts/fixtures/resource-observation.valid.json")).unwrap();
    validate_resource_observation(observation).unwrap();

    let action_negatives: NegativeFixtureSet =
        serde_json::from_slice(&fixture("schemas/machine-contracts/fixtures/resource-action-family.negatives.json"))
            .unwrap();
    assert_eq!(action_negatives.surface_id, "resource.action-family");
    for case in action_negatives.cases {
        let accepted = serde_json::from_value::<ActionFamilyIdentity>(case.artifact)
            .ok()
            .and_then(|value| validate_action_family_identity(value).ok());
        assert!(accepted.is_none(), "negative action-family case passed: {}", case.id);
    }

    let observation_negatives: NegativeFixtureSet =
        serde_json::from_slice(&fixture("schemas/machine-contracts/fixtures/resource-observation.negatives.json"))
            .unwrap();
    assert_eq!(observation_negatives.surface_id, "resource.resource-observation");
    for case in observation_negatives.cases {
        let accepted = serde_json::from_value::<ResourceObservation>(case.artifact)
            .ok()
            .and_then(|value| validate_resource_observation(value).ok());
        assert!(accepted.is_none(), "negative observation case passed: {}", case.id);
    }
}

#[test]
fn benchmark_corpus_replays_to_the_checked_report_and_keeps_measurements_separate() {
    let corpus: BenchmarkCorpus =
        serde_json::from_slice(&fixture("fixtures/resource-policy/benchmark-corpus.json")).unwrap();
    assert_eq!(corpus.schema, "mantle-resource-benchmark-corpus-v1");
    assert_eq!(corpus.non_claim, RESOURCE_POLICY_NON_CLAIM);
    let replay = build_resource_benchmark_report(
        corpus.workloads,
        corpus.observations,
        corpus.baseline_policy_id,
        corpus.candidate_policy_id,
    )
    .unwrap();
    let expected: ResourceBenchmarkReport =
        serde_json::from_slice(&fixture("fixtures/resource-policy/benchmark-report.valid.json")).unwrap();
    assert_eq!(replay, expected);
    assert!(replay.correctness_passed);
    assert!(replay.compatibility_passed);
}

#[test]
fn source_review_adapts_only_the_licensed_nixbench_workload_shape() {
    let review: Value =
        serde_json::from_slice(&fixture("fixtures/resource-policy/nixbench-source-review.json")).unwrap();
    assert_eq!(review["revision"], EXPECTED_NIXBENCH_REVISION);
    assert_eq!(review["license"], "Apache-2.0");
    assert_eq!(review["copied_source"], false);
    assert_eq!(review["tree_path"], "nix/packages/write-one-file.nix");
    assert!(
        review["excluded_authority"]
            .as_array()
            .is_some_and(|rows| rows.len() >= EXPECTED_EXCLUDED_AUTHORITY_COUNT_MIN)
    );
}

#[test]
fn onixos_machine_classes_are_source_bound_and_feed_observe_only_selection() {
    let source: OnixMachineClassFixture =
        serde_json::from_slice(&fixture("fixtures/resource-policy/onixos-machine-classes.valid.json")).unwrap();
    assert_eq!(source.schema, "mantle-onixos-resource-machine-classes-v1");
    assert_eq!(source.revision, EXPECTED_ONIXOS_REVISION);
    assert_eq!(
        source.non_claim,
        "OnixOS inventory identity supplies declared machine-class provenance but does not prove worker availability, capacity, trust, or execution success"
    );
    assert!(source.classes.iter().all(|class| class.onixos_source_blake3 == source.contract_blake3));
    let action: ActionFamilyIdentity =
        serde_json::from_slice(&fixture("schemas/machine-contracts/fixtures/resource-action-family.valid.json"))
            .unwrap();
    let observation: ResourceObservation =
        serde_json::from_slice(&fixture("schemas/machine-contracts/fixtures/resource-observation.valid.json")).unwrap();
    let declared = DeclaredResourceRequirements {
        minima: observation.declared,
        platform: PlatformRequirements {
            architecture: "x86_64".into(),
            platform: "x86_64-linux".into(),
            kvm_required: true,
            trust_tier: "trusted-builder".into(),
            isolation: "microvm".into(),
            required_features: vec!["avx2".into(), "kvm".into()],
        },
    };
    let decision = select_resource_class(ResourceSelectionRequest {
        now_unix_s: NOW_UNIX_S,
        action_family: action,
        declared,
        machine_classes: source.classes,
        quota: QuotaFacts {
            project_remaining_units: QUOTA_UNITS,
            account_remaining_units: QUOTA_UNITS,
        },
        observations: vec![observation],
        policy: ResourceSelectionPolicy::default(),
        controls: ResourceFeatureControls::default(),
    })
    .unwrap();
    assert!(decision.observe_only);
    assert_eq!(decision.scheduled_class_id, "onixos-small");
}

#[test]
fn chaoscontrol_campaign_binds_source_and_all_required_failure_classes() {
    let campaign: ResourceFaultCampaign =
        serde_json::from_slice(&fixture("fixtures/resource-policy/chaoscontrol-campaign.valid.json")).unwrap();
    assert_eq!(campaign.chaoscontrol_revision, EXPECTED_CHAOSCONTROL_REVISION);
    let admitted = validate_resource_fault_campaign(campaign).unwrap();
    assert_eq!(admitted.cases.len(), EXPECTED_FAULT_CASE_COUNT);
    assert!(admitted.cases.iter().any(|case| case.kind == ResourceFaultKind::PositiveOom));
    assert!(admitted.cases.iter().any(|case| case.kind == ResourceFaultKind::AccountingInterruption));
    assert!(admitted.cases.iter().any(|case| case.kind == ResourceFaultKind::CasUnavailable));
}

#[test]
fn generated_profile_defaults_every_authority_feature_to_off() {
    let profile: Value = serde_json::from_slice(&fixture("config/generated/resource-policy.json")).unwrap();
    assert_eq!(profile["valence_revision"], EXPECTED_VALENCE_REVISION);
    assert_eq!(profile["rollout"]["mode"], "observe-only");
    assert_eq!(profile["rollout"]["project_opted_in"], false);
    assert_eq!(profile["rollout"]["historical_selection_enabled"], false);
    assert_eq!(profile["rollout"]["oom_retry_enabled"], false);
    assert_eq!(profile["rollout"]["quota_enforcement_enabled"], false);
    assert_eq!(profile["rollout"]["result_sharing_enabled"], false);
}
