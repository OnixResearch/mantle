use std::collections::BTreeSet;

use crunch_action_result_core::ACTION_RECEIPT_REF_PREFIX;
use crunch_action_result_core::ACTION_RESULT_POLICY_SCHEMA;
use crunch_action_result_core::ActionResultOutput;
use crunch_action_result_core::ActionResultRecord;
use crunch_action_result_core::ActionResultRecordInput;
use crunch_action_result_core::ActionResultTrustPolicy;
use crunch_action_result_core::CandidateAdmissionFacts;
use crunch_action_result_core::DetachedRecordSignature;
use crunch_action_result_core::DiscoveredActionResultCandidate;
use crunch_action_result_core::NETWORK_POLICY_REF_PREFIX;
use crunch_action_result_core::OBJECT_REF_PREFIX;
use crunch_action_result_core::PATH_INFO_REF_PREFIX;
use crunch_action_result_core::PRODUCER_POLICY_REF_PREFIX;
use crunch_action_result_core::PUBLICATION_POLICY_REF_PREFIX;
use crunch_action_result_core::REFERENCE_SCAN_REF_PREFIX;
use crunch_action_result_core::SANDBOX_POLICY_REF_PREFIX;
use crunch_action_result_core::SIGNATURE_REF_PREFIX;
use crunch_action_result_core::STRONG_CLAIM;
use crunch_action_result_core::SignedActionResultRecord;
use crunch_action_result_core::StrongReuseRequest;
use crunch_action_result_core::canonical_action_result;
use crunch_action_result_core::plan_strong_reuse;
use crunch_build::ContentLocalityClass;
use crunch_build::ResourceFitClass;
use crunch_build::distributed::RemoteLocalityReasonCode;
use crunch_build::distributed::RemoteLocalityScope;
use crunch_build::distributed::RemoteNamedResourceQuantity;
use crunch_build::distributed::RemoteResourceRequirements;
use crunch_build::distributed::RemoteResourceVector;
use crunch_build::distributed::RemoteTransferArtifact;
use crunch_build::distributed::RemoteTransferArtifactId;
use crunch_build::distributed::RemoteTransferArtifactKind;
use crunch_build::distributed::RemoteTransferChunkDescriptor;
use crunch_build::distributed::RemoteTransferDigest;
use crunch_build::distributed::RemoteTransferManifest;
use crunch_build::distributed::RemoteTransferReceiverFacts;
use crunch_build::distributed::RemoteTransferSessionId;
use crunch_build::distributed::canonical_remote_transfer_policy_digest;
use crunch_build::distributed::canonicalize_remote_transfer_manifest;
use crunch_build::distributed::plan_remote_transfer_demand;
use crunch_hardware_simulation_core::ActionStage;
use crunch_hardware_simulation_core::EVIDENCE_BUNDLE_SCHEMA;
use crunch_hardware_simulation_core::EVIDENCE_NON_CLAIMS;
use crunch_hardware_simulation_core::HardwareEvidenceBundle;
use crunch_hardware_simulation_core::REQUIRED_NON_CLAIMS;
use serde::Deserialize;

use super::*;

const HARDWARE_ACTION_COUNT: usize = 13;
const HARDWARE_GENERATION_COUNT: u32 = 1;
const HARDWARE_COMPILE_COUNT: u32 = 9;
const HARDWARE_LINK_COUNT: u32 = 1;
const HARDWARE_SMOKE_COUNT: u32 = 2;
const HARDWARE_CPU_UNITS: u32 = 4;
const HARDWARE_WORKER_CPU_UNITS: u32 = 8;
const HARDWARE_MEMORY_BYTES: u64 = 2_147_483_648;
const HARDWARE_WORKER_MEMORY_BYTES: u64 = 4_294_967_296;
const HARDWARE_SCRATCH_BYTES: u64 = 1_073_741_824;
const HARDWARE_WORKER_SCRATCH_BYTES: u64 = 2_147_483_648;
const HARDWARE_TOKEN_QUANTITY: u32 = 1;
const HARDWARE_WORKER_TOKEN_QUANTITY: u32 = 2;
const HARDWARE_STORE_DIGEST_CHARS: usize = 32;
const HARDWARE_WORKER_ENDPOINT: &str = "z-hardware-worker";
const COLD_WORKER_ENDPOINT: &str = "a-cold-worker";
const HARDWARE_FEATURE: &str = "hardware-simulation";
const HARDWARE_TOKEN: &str = "verilator-capacity";
const HARDWARE_SOURCE_CLASS: &str = "provider-free-external-batch";
const HARDWARE_PRODUCER: &str = "hardware-fixture-worker";
const HARDWARE_RUNTIME_SCHEMA: &str = "mantle-hardware-runtime-summary-v1";
const HARDWARE_EVIDENCE_REF: &str =
    "mantle-hardware-evidence://blake3/9c581d319c676243a347de04a9da02d3b4f09540fadfa74f0cd4304268adba94";
const HARDWARE_PROFILE_REF: &str =
    "mantle-hardware-profile://blake3/29ae88cd1abea071db01d724167876f4caef643344cce36e1d0b6e634ffb5c0f";
const HARDWARE_COHORT_REF: &str =
    "mantle-hardware-cohort://blake3/672eb14a9e5494aba76794c35013b656b9d8525d15107ed8ac5f63b0ac78609c";
const ACTION_RESULT_NON_CLAIMS: [&str; 3] = [
    "ca-mapping-presence-is-not-output-trust",
    "executor-correctness",
    "index-presence-is-not-output-trust",
];
const HARDWARE_STORE_DIGEST_ALPHABET: &[u8] = b"abcdfghijklmnpq";

#[derive(Clone, Debug, Deserialize)]
struct HardwareRuntimeSummary {
    schema: String,
    full_shared_clean_client: HardwareSharedRun,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct HardwareSharedRun {
    action_refs: Vec<String>,
    admitted_shared_results: u32,
    complete_receipt_policy_reference_scan_records: u32,
    executor_calls: u32,
    output_count: u32,
    selected_result_refs: Vec<String>,
    selected_source_class: String,
    transferred_logical_nar_bytes: u64,
    trust_basis: Vec<String>,
}

#[derive(Clone, Debug)]
struct HardwareStageAction {
    stage: ActionStage,
    action_ref: String,
}

struct TrackedHardwareFixture {
    bundle: HardwareEvidenceBundle,
    runtime: HardwareRuntimeSummary,
    actions: Vec<HardwareStageAction>,
    input_refs: Vec<String>,
}

struct HardwareActionCase {
    action: HardwareStageAction,
    client: RemoteLoopbackClient,
    request: RemoteCoordinatorBuildRequest,
    logical_path: String,
}

struct AssignedHardwareAction {
    allocation: ExternalBatchOperationResponse,
    binding: RemoteProductionAttemptBinding,
}

struct AdmittedHardwareAction {
    composition: ExternalBatchCompositionEvidence,
    admission: RemoteOutputAdmissionReport,
    imported_output_count: u32,
    shared_result_ref: String,
}

#[derive(Default)]
struct HardwareCompositionCounts {
    generation: u32,
    compile: u32,
    link: u32,
    smoke: u32,
    imported_outputs: u32,
    admitted_shared_results: u32,
}

impl HardwareCompositionCounts {
    fn record(&mut self, stage: ActionStage, admitted: &AdmittedHardwareAction) {
        match stage {
            ActionStage::Generation => self.generation = self.generation.saturating_add(1),
            ActionStage::Compile => self.compile = self.compile.saturating_add(1),
            ActionStage::Link => self.link = self.link.saturating_add(1),
            ActionStage::Smoke => self.smoke = self.smoke.saturating_add(1),
        }
        self.imported_outputs = self.imported_outputs.saturating_add(admitted.imported_output_count);
        self.admitted_shared_results = self.admitted_shared_results.saturating_add(1);
        assert!(!admitted.shared_result_ref.is_empty());
        assert_eq!(admitted.composition.output_admission_digest_blake3, admitted.admission.output_digest_blake3);
    }
}

fn tracked_hardware_fixture() -> TrackedHardwareFixture {
    let bundle: HardwareEvidenceBundle = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/.cairn/archive/2026-07-14-prove-hardware-simulation-build-flow/evidence/hardware-evidence.json"
    )))
    .expect("tracked hardware evidence parses");
    let runtime: HardwareRuntimeSummary = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/.cairn/archive/2026-07-14-prove-hardware-simulation-build-flow/evidence/hardware-runtime-summary.json"
    )))
    .expect("tracked hardware runtime summary parses");
    validate_tracked_hardware_fixture(&bundle, &runtime).expect("tracked hardware fixture remains admissible");
    let actions = hardware_stage_actions(&bundle).expect("tracked hardware stages bind all actions");
    let input_refs = hardware_input_refs(&bundle);
    assert_eq!(actions.len(), HARDWARE_ACTION_COUNT);
    assert!(input_refs.len() > 1);
    TrackedHardwareFixture {
        bundle,
        runtime,
        actions,
        input_refs,
    }
}

fn validate_tracked_hardware_fixture(
    bundle: &HardwareEvidenceBundle,
    runtime: &HardwareRuntimeSummary,
) -> Result<(), String> {
    if bundle.schema != EVIDENCE_BUNDLE_SCHEMA || runtime.schema != HARDWARE_RUNTIME_SCHEMA {
        return Err("external-batch-hardware-schema-mismatch".to_string());
    }
    if bundle.evidence_ref != HARDWARE_EVIDENCE_REF
        || bundle.fresh.profile_ref != HARDWARE_PROFILE_REF
        || bundle.fresh.cohort_ref != HARDWARE_COHORT_REF
    {
        return Err("external-batch-hardware-identity-mismatch".to_string());
    }
    validate_hardware_stage_counts(bundle)?;
    validate_hardware_shared_run(bundle, &runtime.full_shared_clean_client)?;
    validate_hardware_non_claims(bundle, runtime)?;
    Ok(())
}

fn validate_hardware_stage_counts(bundle: &HardwareEvidenceBundle) -> Result<(), String> {
    let expected = [
        (ActionStage::Generation, HARDWARE_GENERATION_COUNT),
        (ActionStage::Compile, HARDWARE_COMPILE_COUNT),
        (ActionStage::Link, HARDWARE_LINK_COUNT),
        (ActionStage::Smoke, HARDWARE_SMOKE_COUNT),
    ];
    if bundle.fresh.action_refs.len() != HARDWARE_ACTION_COUNT {
        return Err("external-batch-hardware-action-count-mismatch".to_string());
    }
    for (stage, expected_count) in expected {
        let counts =
            bundle.fresh.counts.get(&stage).ok_or_else(|| "external-batch-hardware-stage-missing".to_string())?;
        if counts.requested != expected_count || counts.executed != expected_count || counts.reused != 0 {
            return Err("external-batch-hardware-stage-count-mismatch".to_string());
        }
    }
    Ok(())
}

fn validate_hardware_shared_run(bundle: &HardwareEvidenceBundle, shared: &HardwareSharedRun) -> Result<(), String> {
    let expected_count = u32::try_from(HARDWARE_ACTION_COUNT)
        .map_err(|_| "external-batch-hardware-action-count-overflow".to_string())?;
    if shared.action_refs.len() != HARDWARE_ACTION_COUNT
        || shared.selected_result_refs.len() != HARDWARE_ACTION_COUNT
        || shared.admitted_shared_results != expected_count
        || shared.complete_receipt_policy_reference_scan_records != expected_count
        || shared.output_count != expected_count
        || shared.executor_calls != 0
    {
        return Err("external-batch-hardware-shared-result-mismatch".to_string());
    }
    if shared.selected_source_class != "http"
        || shared.trust_basis.is_empty()
        || shared.transferred_logical_nar_bytes != bundle.full_shared_hit.transferred_bytes
    {
        return Err("external-batch-hardware-shared-result-trust-mismatch".to_string());
    }
    Ok(())
}

fn validate_hardware_non_claims(
    bundle: &HardwareEvidenceBundle,
    runtime: &HardwareRuntimeSummary,
) -> Result<(), String> {
    for non_claim in REQUIRED_NON_CLAIMS.iter().chain(EVIDENCE_NON_CLAIMS.iter()) {
        if !bundle.non_claims.iter().any(|value| value == non_claim) {
            return Err(format!("external-batch-hardware-non-claim-missing:{non_claim}"));
        }
    }
    if runtime.non_claims.is_empty() {
        return Err("external-batch-hardware-runtime-non-claims-empty".to_string());
    }
    Ok(())
}

fn hardware_stage_actions(bundle: &HardwareEvidenceBundle) -> Result<Vec<HardwareStageAction>, String> {
    let layout = [
        (ActionStage::Generation, HARDWARE_GENERATION_COUNT),
        (ActionStage::Compile, HARDWARE_COMPILE_COUNT),
        (ActionStage::Link, HARDWARE_LINK_COUNT),
        (ActionStage::Smoke, HARDWARE_SMOKE_COUNT),
    ];
    let mut action_refs = bundle.fresh.action_refs.iter();
    let mut actions = Vec::with_capacity(HARDWARE_ACTION_COUNT);
    for (stage, count) in layout {
        for _ in 0..count {
            let action_ref =
                action_refs.next().ok_or_else(|| "external-batch-hardware-stage-action-missing".to_string())?;
            actions.push(HardwareStageAction {
                stage,
                action_ref: action_ref.clone(),
            });
        }
    }
    if action_refs.next().is_some() || actions.len() != HARDWARE_ACTION_COUNT {
        return Err("external-batch-hardware-stage-action-overflow".to_string());
    }
    Ok(actions)
}

fn hardware_input_refs(bundle: &HardwareEvidenceBundle) -> Vec<String> {
    let mut refs = vec![bundle.fresh.profile_ref.clone(), bundle.fresh.cohort_ref.clone()];
    refs.extend(bundle.fresh.source_refs.iter().cloned());
    refs.sort();
    refs.dedup();
    refs
}

fn hardware_transfer_manifest(input_refs: &[String]) -> CanonicalRemoteTransferManifest {
    let policy = RemoteTransferPolicy::default();
    let policy_digest = canonical_remote_transfer_policy_digest(policy).expect("hardware transfer policy digest");
    let artifacts = input_refs
        .iter()
        .enumerate()
        .map(|(index, input_ref)| hardware_transfer_artifact(index, input_ref))
        .collect::<Vec<_>>();
    let requested_digest = blake3::hash(&serde_json::to_vec(input_refs).expect("hardware input refs serialize"));
    canonicalize_remote_transfer_manifest(
        RemoteTransferManifest {
            schema: crunch_build::distributed::REMOTE_TRANSFER_MANIFEST_SCHEMA.to_string(),
            session_id: RemoteTransferSessionId::new(
                blake3::hash(b"hardware-external-batch-session").to_hex().to_string(),
            )
            .expect("hardware transfer session id"),
            job_id: RemoteJobId::new("hardware-external-batch-job").expect("hardware transfer job id"),
            attempt_id: RemoteAttemptId::new("hardware-external-batch-attempt").expect("hardware transfer attempt id"),
            fence_generation: RemoteFenceGeneration::INITIAL,
            policy_digest_blake3: policy_digest,
            store_prefix: "/mantle/store".to_string(),
            requested_content_blake3: RemoteTransferDigest::new(requested_digest.to_hex().to_string())
                .expect("hardware requested content digest"),
            artifacts,
        },
        policy,
    )
    .expect("hardware transfer manifest canonicalizes")
}

fn hardware_transfer_artifact(index: usize, input_ref: &str) -> RemoteTransferArtifact {
    let digest = RemoteTransferDigest::new(blake3::hash(input_ref.as_bytes()).to_hex().to_string())
        .expect("hardware input digest");
    let size_bytes = u64::try_from(input_ref.len()).expect("hardware input ref length fits u64");
    let chunk_size_bytes = u32::try_from(size_bytes).expect("hardware input ref length fits u32");
    RemoteTransferArtifact {
        artifact_id: RemoteTransferArtifactId::new(format!("hardware-input-{index}"))
            .expect("hardware input artifact id"),
        artifact_kind: RemoteTransferArtifactKind::CastoreBlob,
        digest_blake3: digest.clone(),
        size_bytes,
        required_for_completion: true,
        nar_sha256_hex: None,
        chunks: vec![RemoteTransferChunkDescriptor {
            index: 0,
            offset_bytes: 0,
            size_bytes: chunk_size_bytes,
            digest_blake3: digest,
        }],
    }
}

fn hardware_partial_receiver_facts(manifest: &CanonicalRemoteTransferManifest) -> RemoteTransferReceiverFacts {
    let mut complete = BTreeSet::new();
    complete.insert(manifest.manifest.artifacts[0].artifact_id.clone());
    RemoteTransferReceiverFacts {
        complete_artifact_ids: complete,
        complete_chunk_digests: BTreeSet::new(),
        requested_content_identity_verified: true,
        required_closure_metadata_verified: true,
        path_info_admitted: true,
    }
}

fn hardware_complete_receiver_facts(manifest: &CanonicalRemoteTransferManifest) -> RemoteTransferReceiverFacts {
    RemoteTransferReceiverFacts {
        complete_artifact_ids: manifest
            .manifest
            .artifacts
            .iter()
            .map(|artifact| artifact.artifact_id.clone())
            .collect(),
        complete_chunk_digests: BTreeSet::new(),
        requested_content_identity_verified: true,
        required_closure_metadata_verified: true,
        path_info_admitted: true,
    }
}

fn hardware_resource_requirements() -> RemoteResourceRequirements {
    RemoteResourceRequirements {
        quantities: RemoteResourceVector {
            cpu_units: HARDWARE_CPU_UNITS,
            memory_bytes: HARDWARE_MEMORY_BYTES,
            scratch_bytes: HARDWARE_SCRATCH_BYTES,
            accelerators: Vec::new(),
            named_tokens: vec![RemoteNamedResourceQuantity {
                name: HARDWARE_TOKEN.to_string(),
                quantity: HARDWARE_TOKEN_QUANTITY,
            }],
        },
        semantic_accelerator_classes: Vec::new(),
    }
}

fn hardware_resource_inventory() -> RemoteWorkerResourceInventory {
    RemoteWorkerResourceInventory {
        total: RemoteResourceVector {
            cpu_units: HARDWARE_WORKER_CPU_UNITS,
            memory_bytes: HARDWARE_WORKER_MEMORY_BYTES,
            scratch_bytes: HARDWARE_WORKER_SCRATCH_BYTES,
            accelerators: Vec::new(),
            named_tokens: vec![RemoteNamedResourceQuantity {
                name: HARDWARE_TOKEN.to_string(),
                quantity: HARDWARE_WORKER_TOKEN_QUANTITY,
            }],
        },
    }
}

fn hardware_worker(endpoint_id: &str) -> RemoteWorkerRegistration {
    let mut worker = fixture_worker_registration();
    worker.endpoint_id = endpoint_id.to_string();
    worker.worker_generation = 1;
    worker.feature_labels = vec![HARDWARE_FEATURE.to_string()];
    worker.resource_inventory = Some(hardware_resource_inventory());
    worker.concurrency = TEST_WORKER_CONCURRENCY;
    worker
}

fn hardware_dispatcher_profile(temp: &tempfile::TempDir) -> crate::remote_farm_config::RemoteBatchDispatcherProfile {
    let script = temp.path().join("hardware-slurm-fixture");
    std::fs::write(
        &script,
        "#!/bin/sh\ncase \"$1\" in\n submit) for arg in \"$@\"; do case \"$arg\" in --comment=*) id=${arg#--comment=} ;; esac; done; printf 'h%.31s\\n' \"$id\" ;;\n observe|reconcile) printf 'COMPLETED\\n' ;;\n cancel) exit 0 ;;\n *) exit 64 ;;\nesac\n",
    )
    .expect("hardware fake Slurm CLI writes");
    let mut permissions = std::fs::metadata(&script).expect("hardware fake Slurm metadata").permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&script, permissions).expect("hardware fake Slurm mode applies");
    let digest = blake3::hash(&std::fs::read(&script).expect("hardware fake Slurm bytes read")).to_hex().to_string();
    let mut profile = fixture_batch_dispatcher_profile();
    profile.instance_id = "hardware-provider-free".to_string();
    profile.adapter = crate::remote_farm_config::RemoteBatchDispatcherAdapter::SlurmCliV1;
    for (command, operation) in [
        (&mut profile.submit, "submit"),
        (&mut profile.observe, "observe"),
        (&mut profile.cancel, "cancel"),
        (&mut profile.reconcile, "reconcile"),
    ] {
        command.program = script.clone();
        command.expected_digest_blake3 = digest.clone();
        command.args = vec![operation.to_string()];
    }
    profile
}

fn hardware_action_case(
    action: &HardwareStageAction,
    index: usize,
    input_refs: &[String],
    locality_scope: RemoteLocalityScope,
) -> HardwareActionCase {
    let logical_path = hardware_output_path(index, action.stage);
    let mut client = fixture_loopback_client(input_refs.to_vec(), vec!["builder-key".to_string()]);
    client.request.request_id = format!("hardware-external-{index}");
    client.request.input_refs = input_refs.to_vec();
    client.request.source_input_refs = Vec::new();
    client.request.payload = RemoteConcreteBuildPayload::Action {
        action_id: action.action_ref.clone(),
        spec_json: hardware_action_spec(action),
    };
    client.request.expected_outputs = vec![RemoteExpectedOutput {
        name: "out".to_string(),
        logical_path: Some(logical_path.clone()),
    }];
    client.request.resource_requirements = Some(hardware_resource_requirements());
    client.request.locality_scope = Some(locality_scope.clone());
    client.input_manifest.request_id = client.request.request_id.clone();
    client.input_manifest.input_refs = input_refs.to_vec();
    client.input_manifest.closure_refs = input_refs.to_vec();
    let mut request = fixture_remote_operator_coordinator_request(&client);
    request.required_features = vec![HARDWARE_FEATURE.to_string()];
    request.resource_requirements = client.request.resource_requirements.clone();
    request.locality_scope = Some(locality_scope);
    request.live_output_claims = vec![logical_path.clone()];
    HardwareActionCase {
        action: action.clone(),
        client,
        request,
        logical_path,
    }
}

fn hardware_action_spec(action: &HardwareStageAction) -> String {
    serde_json::json!({
        "schema": REMOTE_ACTION_SPEC_SCHEMA,
        "action_id": action.action_ref,
        "builder": "builtin:hardware-fixture",
        "args": [hardware_stage_label(action.stage)],
        "outputs": ["out"],
        "profile_ref": HARDWARE_PROFILE_REF,
        "cohort_ref": HARDWARE_COHORT_REF,
    })
    .to_string()
}

fn hardware_stage_label(stage: ActionStage) -> &'static str {
    match stage {
        ActionStage::Generation => "generation",
        ActionStage::Compile => "compile",
        ActionStage::Link => "link",
        ActionStage::Smoke => "smoke",
    }
}

fn hardware_output_path(index: usize, stage: ActionStage) -> String {
    let digest_character = char::from(HARDWARE_STORE_DIGEST_ALPHABET[index]);
    let digest = digest_character.to_string().repeat(HARDWARE_STORE_DIGEST_CHARS);
    format!("/mantle/store/{digest}-hardware-{}-{index}", hardware_stage_label(stage))
}

fn assign_hardware_action(
    state: &mut RemoteCoordinatorState,
    case: &HardwareActionCase,
    profile: &crate::remote_farm_config::RemoteBatchDispatcherProfile,
    dispatcher: &ConfiguredExternalBatchDispatcher<'_>,
    manifest: &CanonicalRemoteTransferManifest,
) -> AssignedHardwareAction {
    let allocation = submit_external_batch_allocation(
        state,
        &case.request,
        profile,
        HARDWARE_WORKER_ENDPOINT,
        1,
        dispatcher,
        fixture_retry_time().now_unix_s,
    )
    .expect("tracked hardware action submits through provider-free adapter");
    assert_hardware_resource_projection(&allocation, state);
    let blocked = authorize_external_batch_transfer(state, &allocation.dispatch_id_blake3, HARDWARE_WORKER_ENDPOINT)
        .expect_err("hardware input transfer remains blocked before registration");
    assert_eq!(blocked, "external-batch-worker-not-registered");
    register_external_batch_worker(state, &allocation.dispatch_id_blake3, hardware_worker(HARDWARE_WORKER_ENDPOINT))
        .expect("ordinary hardware worker registration succeeds");
    let locality = record_receiver_verified_worker_locality(
        state,
        HARDWARE_WORKER_ENDPOINT,
        1,
        manifest,
        hardware_partial_receiver_facts(manifest),
    )
    .expect("hardware locality is receiver verified");
    assert_eq!(locality.content_locality, ContentLocalityClass::PartiallyPresent);
    assert_eq!(locality.reason_code, RemoteLocalityReasonCode::VerifiedPartiallyPresent);
    let decision = admit_external_batch_coordinator_dispatch(
        state,
        &allocation.dispatch_id_blake3,
        &case.request,
        RemoteAttemptRetryPolicy::default(),
        fixture_retry_time(),
    )
    .expect("tracked hardware action enters ordinary coordinator assignment");
    let RemoteCoordinatorDispatchDecision::Dispatch {
        worker_endpoint_id,
        job_id,
        attempt_id,
        fence_generation,
        resource_fit,
        locality,
        ..
    } = decision
    else {
        panic!("fresh hardware action must dispatch");
    };
    assert_eq!(worker_endpoint_id, HARDWARE_WORKER_ENDPOINT);
    assert_eq!(resource_fit, ResourceFitClass::Compatible);
    assert_eq!(
        locality.expect("hardware locality is present").reason_code,
        RemoteLocalityReasonCode::VerifiedPartiallyPresent
    );
    let binding = RemoteProductionAttemptBinding {
        job_id,
        attempt_id,
        fence_generation,
    };
    start_remote_production_attempt_if_queued(state, &binding, fixture_retry_time().now_unix_s)
        .expect("ordinary hardware worker starts fenced attempt");
    AssignedHardwareAction { allocation, binding }
}

fn assert_hardware_resource_projection(allocation: &ExternalBatchOperationResponse, state: &RemoteCoordinatorState) {
    let operation = &state.external_batch_attempts[&allocation.dispatch_id_blake3].submit_operation;
    assert_eq!(operation.resources.cpu_units, HARDWARE_CPU_UNITS);
    assert_eq!(operation.resources.memory_bytes, HARDWARE_MEMORY_BYTES);
    assert_eq!(operation.resources.scratch_bytes, HARDWARE_SCRATCH_BYTES);
    assert_eq!(operation.resources.named_tokens, vec![RemoteNamedResourceQuantity {
        name: HARDWARE_TOKEN.to_string(),
        quantity: HARDWARE_TOKEN_QUANTITY,
    }]);
    assert!(operation.resources.accelerators.is_empty());
    assert!(operation.resources.semantic_accelerator_classes.is_empty());
}

async fn admit_hardware_action(
    state: &mut RemoteCoordinatorState,
    store: &mut crunch_store::StoreHandle,
    case: &HardwareActionCase,
    assigned: &AssignedHardwareAction,
    dispatcher: &ConfiguredExternalBatchDispatcher<'_>,
) -> AdmittedHardwareAction {
    let admission = admit_hardware_fenced_output(state, case, &assigned.binding);
    let imported = import_admitted_remote_outputs(
        store,
        &case.client.request,
        &admission,
        true,
        Some(crunch_store::GcRootSource::Build),
    )
    .await
    .expect("hardware output imports through ordinary CAS path");
    let observed = observe_external_batch_allocation(
        state,
        &assigned.allocation.dispatch_id_blake3,
        dispatcher,
        fixture_retry_time().now_unix_s,
    )
    .expect("provider-free allocation observes completion only after output admission");
    assert_eq!(observed.state, ExternalBatchJobState::Succeeded);
    let composition = external_batch_composition_evidence(
        state,
        &assigned.allocation.dispatch_id_blake3,
        &assigned.binding,
        &admission,
    )
    .expect("hardware output composes only after ordinary admission");
    let shared_result_ref = admit_hardware_shared_result(&case.action, &admission);
    complete_remote_production_attempt(
        state,
        &assigned.binding,
        &admission.output_digest_blake3,
        fixture_retry_time().now_unix_s,
    )
    .expect("hardware coordinator attempt completes after admitted output");
    AdmittedHardwareAction {
        composition,
        admission,
        imported_output_count: u32::try_from(imported.outputs.len()).expect("hardware output count fits u32"),
        shared_result_ref,
    }
}

fn admit_hardware_fenced_output(
    state: &mut RemoteCoordinatorState,
    case: &HardwareActionCase,
    binding: &RemoteProductionAttemptBinding,
) -> RemoteOutputAdmissionReport {
    let response = hardware_worker_response(case);
    let report = production_attempt_report(
        binding,
        "hardware-external-batch-result",
        fixture_retry_time().now_unix_s,
        RemoteAttemptReportPayload::ResultReady {
            output_digest_blake3: response.output_digest_blake3.clone(),
        },
    )
    .expect("hardware fenced result report derives");
    let admitted = admit_fenced_remote_builder_response(
        state,
        &report,
        true,
        RemoteLogRetentionPolicy::default(),
        &case.client.request,
        &case.client.trusted_output_keys,
        &response,
    )
    .expect("hardware output uses ordinary fenced admission");
    let RemoteFencedOutputAdmissionDecision::Admit { admission, .. } = admitted else {
        panic!("fresh hardware output must admit");
    };
    *admission
}

fn hardware_worker_response(case: &HardwareActionCase) -> RemoteBuilderFrameResponse {
    let builder = fixture_loopback_builder();
    let executor = PathInfoRemoteExecutor {
        path_info: hardware_pathinfo(&case.logical_path, &case.action.action_ref),
        artifact_attestation_digest_blake3: None,
        nar_payload: None,
    };
    let mut ticket_state = RemoteTicketState::default();
    ticket_state.tickets.insert("ticket-1".to_string(), fixture_ticket());
    let response = plan_stdio_remote_once_from_state_with_executor(
        std::io::Cursor::new(encode_frame_stream(&remote_client_request_frames(&case.client))),
        &builder,
        &mut ticket_state,
        case.client.transfer_capabilities,
        &executor,
    )
    .expect("hardware worker response crosses ordinary framed protocol");
    assert_eq!(response.missing_input_refs, case.client.request.input_refs);
    assert_eq!(case.client.uploaded_input_refs, case.client.request.input_refs);
    assert_eq!(response.transfer.mode, RemoteTransferMode::Delta);
    assert_eq!(response.outputs.len(), 1);
    response
}

fn hardware_pathinfo(logical_path: &str, action_ref: &str) -> PathInfo {
    let mut path_info = pathinfo_for_logical_path(logical_path);
    path_info.node = snix_castore::Node::Symlink {
        target: snix_castore::SymlinkTarget::try_from(action_ref).expect("hardware action ref is a symlink target"),
    };
    path_info
}

fn hardware_action_result_record(
    action: &HardwareStageAction,
    admission: &RemoteOutputAdmissionReport,
) -> (ActionResultRecord, String, String, Vec<String>) {
    let output = &admission.outputs[0];
    let path_info_bytes = serde_json::to_vec(output.path_info.as_ref().expect("admitted hardware PathInfo exists"))
        .expect("hardware PathInfo serializes");
    let sandbox_policy_ref = typed_fixture_ref(SANDBOX_POLICY_REF_PREFIX, b"hardware-strict-sandbox");
    let network_policy_ref = typed_fixture_ref(NETWORK_POLICY_REF_PREFIX, b"hardware-no-network");
    let non_claims = ACTION_RESULT_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let record = canonical_action_result(ActionResultRecordInput {
        action_ref: action.action_ref.clone(),
        outputs: vec![ActionResultOutput {
            name: output.name.clone(),
            object_ref: format!("{OBJECT_REF_PREFIX}{}", output.content_digest_blake3),
            store_path: output.logical_path.clone(),
            path_info_ref: typed_fixture_ref(PATH_INFO_REF_PREFIX, &path_info_bytes),
        }],
        action_receipt_ref: typed_fixture_ref(ACTION_RECEIPT_REF_PREFIX, action.action_ref.as_bytes()),
        reference_scan_refs: vec![typed_fixture_ref(
            REFERENCE_SCAN_REF_PREFIX,
            action.action_ref.as_bytes(),
        )],
        sandbox_policy_ref: sandbox_policy_ref.clone(),
        network_policy_ref: network_policy_ref.clone(),
        producer_identity: HARDWARE_PRODUCER.to_string(),
        producer_policy_ref: typed_fixture_ref(PRODUCER_POLICY_REF_PREFIX, b"hardware-producer-policy"),
        signature_refs: vec![typed_fixture_ref(SIGNATURE_REF_PREFIX, b"builder-key")],
        publication_policy_ref: typed_fixture_ref(PUBLICATION_POLICY_REF_PREFIX, b"hardware-publication-policy"),
        non_claims: non_claims.clone(),
    })
    .expect("ordinary hardware action result canonicalizes");
    (record, sandbox_policy_ref, network_policy_ref, non_claims)
}

fn admit_hardware_shared_result(action: &HardwareStageAction, admission: &RemoteOutputAdmissionReport) -> String {
    let (record, sandbox_policy_ref, network_policy_ref, non_claims) = hardware_action_result_record(action, admission);
    let signed_record = SignedActionResultRecord {
        record: record.clone(),
        record_signatures: vec![DetachedRecordSignature {
            key_name: "builder-key".to_string(),
            signature: "provider-free-fixture-signature".to_string(),
        }],
    };
    let policy = ActionResultTrustPolicy {
        schema: ACTION_RESULT_POLICY_SCHEMA.to_string(),
        policy_id: "hardware-provider-free-fixture-v1".to_string(),
        trusted_producers: vec![HARDWARE_PRODUCER.to_string()],
        trusted_record_signers: vec!["builder-key".to_string()],
        allowed_source_classes: vec![HARDWARE_SOURCE_CLASS.to_string()],
        allowed_sandbox_policy_refs: vec![sandbox_policy_ref],
        allowed_network_policy_refs: vec![network_policy_ref],
        required_non_claims: non_claims,
        require_record_signature: true,
        require_path_info_signature: true,
    };
    let candidates = vec![DiscoveredActionResultCandidate {
        signed_record,
        facts: CandidateAdmissionFacts {
            source_id: "hardware-provider-free-fixture".to_string(),
            source_class: HARDWARE_SOURCE_CLASS.to_string(),
            verified_record_signers: vec!["builder-key".to_string()],
            action_receipt_linked: true,
            object_refs_complete: true,
            path_info_refs_linked: true,
            path_info_signatures_verified: true,
            producer_policy_admitted: true,
            publication_policy_admitted: true,
            sandbox_policy_admitted: true,
            network_policy_admitted: true,
            reference_scans_admitted: true,
            claim_strength: STRONG_CLAIM.to_string(),
        },
    }];
    let plan = plan_strong_reuse(
        StrongReuseRequest {
            action_ref: action.action_ref.clone(),
            output_names: vec!["out".to_string()],
            policy,
        },
        candidates,
    )
    .expect("hardware result enters generic strong-result admission");
    assert!(plan.candidate_decisions[0].admitted);
    assert!(!plan.candidate_decisions[0].trust_basis.is_empty());
    assert_eq!(plan.selected_result_ref.as_deref(), Some(record.result_ref.as_str()));
    plan.selected_result_ref.expect("strong hardware result is selected")
}

fn typed_fixture_ref(prefix: &str, bytes: &[u8]) -> String {
    format!("{prefix}{}", blake3::hash(bytes).to_hex())
}

fn assert_hardware_non_claims_preserved(fixture: &TrackedHardwareFixture, admitted: &[AdmittedHardwareAction]) {
    let mut preserved = fixture.bundle.non_claims.iter().cloned().collect::<BTreeSet<_>>();
    preserved.extend(fixture.runtime.non_claims.iter().cloned());
    for result in admitted {
        preserved.extend(result.composition.non_claims.iter().cloned());
    }
    for non_claim in REQUIRED_NON_CLAIMS.iter().chain(EVIDENCE_NON_CLAIMS.iter()) {
        assert!(preserved.contains(*non_claim), "missing hardware non-claim {non_claim}");
    }
    assert!(preserved.contains(EXTERNAL_BATCH_NON_CLAIM));
    assert!(preserved.contains("output admission remains governed by the ordinary signed-output trust path"));
}

// r[impl external_batch_dispatchers.hardware_workload_composition]
// r[verify external_batch_dispatchers.hardware_workload_composition]
#[tokio::test]
async fn external_batch_dispatches_tracked_hardware_graph_through_ordinary_cas_and_admission() {
    let fixture = tracked_hardware_fixture();
    let temp = tempfile::tempdir().expect("hardware external batch tempdir");
    let profile = hardware_dispatcher_profile(&temp);
    let dispatcher = ConfiguredExternalBatchDispatcher::new(&profile);
    let manifest = hardware_transfer_manifest(&fixture.input_refs);
    let locality_scope = RemoteLocalityScope {
        manifest_digest_blake3: manifest.digest_blake3.as_str().to_string(),
        policy_digest_blake3: manifest.manifest.policy_digest_blake3.as_str().to_string(),
    };
    let partial_facts = hardware_partial_receiver_facts(&manifest);
    let partial_demand =
        plan_remote_transfer_demand(&manifest, &partial_facts).expect("partial hardware transfer plans");
    let complete_demand = plan_remote_transfer_demand(&manifest, &hardware_complete_receiver_facts(&manifest))
        .expect("complete hardware transfer plans");
    assert!(!partial_demand.missing_chunks.is_empty());
    assert!(partial_demand.missing_bytes > 0);
    assert!(fixture.input_refs.iter().all(|input_ref| input_ref.contains("://blake3/")));
    assert_eq!(complete_demand.missing_bytes, 0);
    assert!(complete_demand.missing_chunks.is_empty());

    let coordinator_dir = temp.path().join("coordinator");
    let mut state = RemoteCoordinatorState {
        state_dir: Some(coordinator_dir),
        ..RemoteCoordinatorState::default()
    };
    apply_worker_registration(&mut state, hardware_worker(COLD_WORKER_ENDPOINT))
        .expect("cold hardware worker registers without verified locality");
    let store_root = temp.path().join("store-root");
    let mut store = remote_import_store(&store_root).await;
    let mut counts = HardwareCompositionCounts::default();
    let mut admitted = Vec::with_capacity(HARDWARE_ACTION_COUNT);

    for (index, action) in fixture.actions.iter().enumerate() {
        let case = hardware_action_case(action, index, &fixture.input_refs, locality_scope.clone());
        let assigned = assign_hardware_action(&mut state, &case, &profile, &dispatcher, &manifest);
        let result = admit_hardware_action(&mut state, &mut store, &case, &assigned, &dispatcher).await;
        assert_eq!(result.imported_output_count, 1);
        assert_eq!(result.composition.worker_endpoint_id, HARDWARE_WORKER_ENDPOINT);
        assert_eq!(result.composition.queue_state, ExternalBatchJobState::Succeeded);
        assert_eq!(result.composition.transfer_mode, RemoteTransferMode::Delta);
        assert_eq!(result.composition.output_trust_key_id, "builder-key");
        counts.record(action.stage, &result);
        admitted.push(result);
    }

    assert_eq!(counts.generation, HARDWARE_GENERATION_COUNT);
    assert_eq!(counts.compile, HARDWARE_COMPILE_COUNT);
    assert_eq!(counts.link, HARDWARE_LINK_COUNT);
    assert_eq!(counts.smoke, HARDWARE_SMOKE_COUNT);
    assert_eq!(counts.imported_outputs, u32::try_from(HARDWARE_ACTION_COUNT).expect("hardware count fits u32"));
    assert_eq!(
        counts.admitted_shared_results,
        u32::try_from(HARDWARE_ACTION_COUNT).expect("hardware count fits u32")
    );
    assert_eq!(state.external_batch_attempts.len(), HARDWARE_ACTION_COUNT);
    assert!(state.jobs.values().all(|job| job.phase == RemoteCoordinatorJobPhase::Finished));
    assert!(state.resource_leases.is_empty());
    assert_hardware_non_claims_preserved(&fixture, &admitted);
}

#[test]
fn hardware_named_token_absence_is_ineligible_before_provider_submission() {
    let fixture = tracked_hardware_fixture();
    let manifest = hardware_transfer_manifest(&fixture.input_refs);
    let locality_scope = RemoteLocalityScope {
        manifest_digest_blake3: manifest.digest_blake3.as_str().to_string(),
        policy_digest_blake3: manifest.manifest.policy_digest_blake3.as_str().to_string(),
    };
    let case = hardware_action_case(&fixture.actions[0], 0, &fixture.input_refs, locality_scope);
    let mut worker = hardware_worker(HARDWARE_WORKER_ENDPOINT);
    worker.resource_inventory.as_mut().expect("hardware inventory exists").total.named_tokens.clear();
    let mut state = RemoteCoordinatorState::default();
    apply_worker_registration(&mut state, worker).expect("tokenless hardware worker registration remains valid");

    let placements = coordinator_worker_placement_candidates(&state, &case.request)
        .expect("tokenless hardware placement remains a bounded decision");
    assert!(placements.is_empty());
    assert_eq!(no_matching_worker_reason(&state, &case.request), "resource-named-token-unavailable");
    assert!(state.external_batch_attempts.is_empty());
}

#[test]
fn tracked_hardware_fixture_rejects_non_claim_and_executor_drift() {
    let fixture = tracked_hardware_fixture();
    validate_tracked_hardware_fixture(&fixture.bundle, &fixture.runtime).expect("accepted fixture remains valid");

    for required in REQUIRED_NON_CLAIMS.iter().chain(EVIDENCE_NON_CLAIMS.iter()) {
        let mut missing_non_claim = fixture.bundle.clone();
        missing_non_claim.non_claims.retain(|value| value != required);
        let error = validate_tracked_hardware_fixture(&missing_non_claim, &fixture.runtime)
            .expect_err("missing hardware non-claim fails closed");
        assert_eq!(error, format!("external-batch-hardware-non-claim-missing:{required}"));
    }

    let mut executor_drift = fixture.runtime.clone();
    executor_drift.full_shared_clean_client.executor_calls = 1;
    let error = validate_tracked_hardware_fixture(&fixture.bundle, &executor_drift)
        .expect_err("hardware shared-result executor drift fails closed");
    assert_eq!(error, "external-batch-hardware-shared-result-mismatch");
}
