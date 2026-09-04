//! Pure checkpoint policy for source-built fixed-point provider stages.
//!
//! The shell owns checkpoint storage, tree observation, publication, and restore.
//! This module owns stage-specific identity, checkpoint construction, and
//! promoted-proof admission. A checkpoint can omit unrelated later-stage source
//! inputs, but it cannot omit any source, policy, predecessor output, resource,
//! or stage-shape fact used by the completed provider stages.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;

use serde::Deserialize;
use serde::Serialize;

use crate::source_built_fixed_point::ProofOutputRole;
use crate::source_built_fixed_point::ProofPolicyRole;
use crate::source_built_fixed_point::SourceAuthorityInput;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds;
use crate::source_built_fixed_point::StageAuthorityInput;

pub(crate) const PROVIDER_CHECKPOINT_SCHEMA: &str = "mantle-source-built-provider-checkpoint-v2";
pub(crate) const PROVIDER_CHECKPOINT_STAGE_COUNT: usize = 4;
pub(crate) const PROVIDER_CHECKPOINT_PAYLOAD_COUNT: usize = 17;
#[cfg(test)]
const FIRST_PROVIDER_STAGE_INDEX: usize = 0;
const STAGEX_PROVIDER_STAGE_INDEX: usize = 1;
const NATIVE_PROVIDER_STAGE_INDEX: usize = 2;
const PROVIDER_CHECKPOINT_LOOKUP_CONTEXT: &str = "mantle-source-built-provider-checkpoint-lookup-v3";
const PROVIDER_CHECKPOINT_STAGE_CONTEXT: &str = "mantle-source-built-provider-checkpoint-stage-v2";
const PROVIDER_CHECKPOINT_MANIFEST_CONTEXT: &str = "mantle-source-built-provider-checkpoint-manifest-v2";
const PROVIDER_ACTION_TRUST_POLICY_ID: &str = "mantle-provider-action-trust-v2";
const RESOURCE_BOUNDS_CONTEXT: &str = "mantle-source-built-provider-checkpoint-resource-bounds-v1";
const BLAKE3_HEX_LENGTH: usize = 64;
const TEXT_BYTES_MAX: usize = 4_096;

pub(crate) const PAYLOAD_STAGEX_TRANSITION: &str = "stagex-transition-execution";
pub(crate) const PAYLOAD_STAGEX_PROVIDER: &str = "stagex-provider";
pub(crate) const PAYLOAD_NATIVE_PROVIDER: &str = "native-provider";
pub(crate) const PAYLOAD_RUST_PROVIDER: &str = "rust-provider";
pub(crate) const PAYLOAD_NATIVE_ADMISSION: &str = "native-admission";
pub(crate) const PAYLOAD_NATIVE_TRANSCRIPT: &str = "native-transcript";
pub(crate) const PAYLOAD_TOOLCHAIN_CLOSURE: &str = "toolchain-closure";
pub(crate) const PAYLOAD_NATIVE_ACTION_PLAN: &str = "native-action-plan";
pub(crate) const PAYLOAD_NATIVE_ACTION_RECONCILIATION: &str = "native-action-reconciliation";
pub(crate) const PAYLOAD_RUST_HOST_MAKE: &str = "rust-host-make";
pub(crate) const PAYLOAD_RUST_HOST_CMAKE: &str = "rust-host-cmake";
pub(crate) const PAYLOAD_RUST_HOST_PYTHON: &str = "rust-host-python";
pub(crate) const PAYLOAD_RUST_HOST_PERL: &str = "rust-host-perl";
pub(crate) const PAYLOAD_RUST_HOST_BUSYBOX: &str = "rust-host-busybox";
pub(crate) const PAYLOAD_RUST_HOST_LINUX_HEADERS: &str = "rust-host-linux-headers";
pub(crate) const PAYLOAD_RUST_HOST_EVIDENCE: &str = "rust-host-evidence";
pub(crate) const PAYLOAD_RUST_ACTION_TRUST: &str = "rust-action-trust";
const REQUIRED_PAYLOAD_IDS: [&str; PROVIDER_CHECKPOINT_PAYLOAD_COUNT] = [
    PAYLOAD_STAGEX_TRANSITION,
    PAYLOAD_STAGEX_PROVIDER,
    PAYLOAD_NATIVE_PROVIDER,
    PAYLOAD_RUST_PROVIDER,
    PAYLOAD_NATIVE_ADMISSION,
    PAYLOAD_NATIVE_TRANSCRIPT,
    PAYLOAD_TOOLCHAIN_CLOSURE,
    PAYLOAD_NATIVE_ACTION_PLAN,
    PAYLOAD_NATIVE_ACTION_RECONCILIATION,
    PAYLOAD_RUST_HOST_MAKE,
    PAYLOAD_RUST_HOST_CMAKE,
    PAYLOAD_RUST_HOST_PYTHON,
    PAYLOAD_RUST_HOST_PERL,
    PAYLOAD_RUST_HOST_BUSYBOX,
    PAYLOAD_RUST_HOST_LINUX_HEADERS,
    PAYLOAD_RUST_HOST_EVIDENCE,
    PAYLOAD_RUST_ACTION_TRUST,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProofCheckpointOrigin {
    PromotedExecution,
    DevExecution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CheckpointPayloadKind {
    PreservedTree,
    Directory,
    RegularFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CheckpointPayloadIdentity {
    pub(crate) payload_id: String,
    pub(crate) relative_path: String,
    pub(crate) kind: CheckpointPayloadKind,
    pub(crate) digest_blake3: String,
    pub(crate) total_file_bytes: u64,
    pub(crate) entry_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ProviderCheckpointStageObservation {
    pub(crate) output_role: ProofOutputRole,
    pub(crate) output_digest_blake3: String,
    pub(crate) semantic_output_digest_blake3: String,
    pub(crate) payload_digest_blake3: String,
    pub(crate) execution_evidence_digest_blake3: String,
    pub(crate) producer_executable_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ProviderCheckpointStageRecord {
    pub(crate) stage_id: String,
    pub(crate) output_role: ProofOutputRole,
    pub(crate) stage_authority_digest_blake3: String,
    pub(crate) output_digest_blake3: String,
    pub(crate) semantic_output_digest_blake3: String,
    pub(crate) payload_digest_blake3: String,
    pub(crate) execution_evidence_digest_blake3: String,
    pub(crate) producer_executable_digest_blake3: String,
    pub(crate) authority_violations: Vec<String>,
    pub(crate) fallback_events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ProviderCheckpointManifest {
    pub(crate) schema: String,
    pub(crate) origin: ProofCheckpointOrigin,
    pub(crate) lookup_key_blake3: String,
    pub(crate) resource_bounds_digest_blake3: String,
    pub(crate) stages: Vec<ProviderCheckpointStageRecord>,
    pub(crate) payloads: Vec<CheckpointPayloadIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderCheckpointAdmission {
    pub(crate) checkpoint_digest_blake3: String,
    pub(crate) lookup_key_blake3: String,
    pub(crate) completed_stage_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckpointError {
    message: String,
}

impl fmt::Display for CheckpointError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CheckpointError {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct ResolvedAuthorityInput {
    authority_kind: String,
    authority_role: String,
    digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProviderCheckpointLookupMaterial {
    logical_store_prefix: String,
    action_trust_policy_id: String,
    resource_bounds_digest_blake3: String,
    stages: Vec<ProviderCheckpointLookupStage>,
    sources: Vec<SourceAuthorityInput>,
    policies: BTreeMap<ProofPolicyRole, String>,
    expected_native_provider_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProviderCheckpointLookupStage {
    stage_id: String,
    output_role: ProofOutputRole,
    inputs: Vec<StageAuthorityInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct StageAuthorityMaterial {
    stage_id: String,
    output_role: ProofOutputRole,
    resource_bounds_digest_blake3: String,
    inputs: Vec<ResolvedAuthorityInput>,
}

pub(crate) fn build_provider_checkpoint_manifest(
    plan: &SourceBuiltFixedPointPlan,
    origin: ProofCheckpointOrigin,
    observations: Vec<ProviderCheckpointStageObservation>,
    payloads: Vec<CheckpointPayloadIdentity>,
) -> Result<ProviderCheckpointManifest, CheckpointError> {
    validate_observation_count(&observations)?;
    validate_payloads(&payloads)?;
    let resource_bounds_digest_blake3 = resource_bounds_digest(&plan.resource_bounds)?;
    let lookup_key_blake3 = provider_checkpoint_lookup_key(plan)?;
    let stages = build_stage_records(plan, observations, &resource_bounds_digest_blake3)?;
    let manifest = ProviderCheckpointManifest {
        schema: PROVIDER_CHECKPOINT_SCHEMA.to_string(),
        origin,
        lookup_key_blake3,
        resource_bounds_digest_blake3,
        stages,
        payloads,
    };
    validate_manifest_shape(&manifest)?;
    validate_stage_payload_links(&manifest.stages, &manifest.payloads)?;
    assert_eq!(manifest.stages.len(), PROVIDER_CHECKPOINT_STAGE_COUNT);
    assert_eq!(manifest.payloads.len(), PROVIDER_CHECKPOINT_PAYLOAD_COUNT);
    Ok(manifest)
}

pub(crate) fn admit_promoted_provider_checkpoint(
    plan: &SourceBuiltFixedPointPlan,
    manifest: &ProviderCheckpointManifest,
    observed_payloads: &[CheckpointPayloadIdentity],
    expected_stagex_provider_digest_blake3: &str,
) -> Result<ProviderCheckpointAdmission, CheckpointError> {
    admit_provider_checkpoint(
        plan,
        manifest,
        observed_payloads,
        expected_stagex_provider_digest_blake3,
        ProofCheckpointOrigin::PromotedExecution,
    )
}

pub(crate) fn admit_dev_provider_checkpoint(
    plan: &SourceBuiltFixedPointPlan,
    manifest: &ProviderCheckpointManifest,
    observed_payloads: &[CheckpointPayloadIdentity],
    expected_stagex_provider_digest_blake3: &str,
) -> Result<ProviderCheckpointAdmission, CheckpointError> {
    admit_provider_checkpoint(
        plan,
        manifest,
        observed_payloads,
        expected_stagex_provider_digest_blake3,
        ProofCheckpointOrigin::DevExecution,
    )
}

fn admit_provider_checkpoint(
    plan: &SourceBuiltFixedPointPlan,
    manifest: &ProviderCheckpointManifest,
    observed_payloads: &[CheckpointPayloadIdentity],
    expected_stagex_provider_digest_blake3: &str,
    expected_origin: ProofCheckpointOrigin,
) -> Result<ProviderCheckpointAdmission, CheckpointError> {
    validate_digest("expected StageX provider", expected_stagex_provider_digest_blake3)?;
    validate_manifest_shape(manifest)?;
    if manifest.origin != expected_origin {
        if expected_origin == ProofCheckpointOrigin::PromotedExecution {
            return Err(checkpoint_error("promoted proof rejects a dev checkpoint"));
        }
        return Err(checkpoint_error("dev resume rejects a promoted checkpoint"));
    }
    let expected_lookup_key = provider_checkpoint_lookup_key(plan)?;
    if manifest.lookup_key_blake3 != expected_lookup_key {
        return Err(checkpoint_error("provider checkpoint lookup authority does not match the current plan"));
    }
    let expected_resource_bounds = resource_bounds_digest(&plan.resource_bounds)?;
    if manifest.resource_bounds_digest_blake3 != expected_resource_bounds {
        return Err(checkpoint_error("provider checkpoint resource bounds do not match the current plan"));
    }
    validate_stage_records(plan, &manifest.stages, &expected_resource_bounds)?;
    validate_semantic_provider_outputs(plan, &manifest.stages, expected_stagex_provider_digest_blake3)?;
    validate_stage_payload_links(&manifest.stages, &manifest.payloads)?;
    validate_payloads(observed_payloads)?;
    if manifest.payloads != observed_payloads {
        return Err(checkpoint_error("provider checkpoint payload observations do not match the manifest"));
    }
    let checkpoint_digest_blake3 = provider_checkpoint_manifest_digest(manifest)?;
    let completed_stage_count = u32::try_from(manifest.stages.len())
        .map_err(|_| checkpoint_error("provider checkpoint stage count does not fit u32"))?;
    assert_eq!(usize::try_from(completed_stage_count).ok(), Some(PROVIDER_CHECKPOINT_STAGE_COUNT));
    assert_eq!(checkpoint_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    Ok(ProviderCheckpointAdmission {
        checkpoint_digest_blake3,
        lookup_key_blake3: expected_lookup_key,
        completed_stage_count,
    })
}

pub(crate) fn provider_checkpoint_lookup_key(plan: &SourceBuiltFixedPointPlan) -> Result<String, CheckpointError> {
    require_provider_stage_prefix(plan)?;
    let mut source_roles = BTreeSet::new();
    let mut policy_roles = BTreeSet::new();
    let mut stages = Vec::with_capacity(PROVIDER_CHECKPOINT_STAGE_COUNT);
    for stage in plan.stages.iter().take(PROVIDER_CHECKPOINT_STAGE_COUNT) {
        collect_non_output_roles(&stage.inputs, &mut source_roles, &mut policy_roles);
        stages.push(ProviderCheckpointLookupStage {
            stage_id: stage.stage_id.clone(),
            output_role: stage.output,
            inputs: stage.inputs.clone(),
        });
    }
    let sources = source_roles
        .into_iter()
        .map(|role| source_input(plan, role).cloned())
        .collect::<Result<Vec<_>, _>>()?;
    let policies = policy_roles
        .into_iter()
        .map(|role| Ok((role, policy_digest(plan, role)?.to_string())))
        .collect::<Result<BTreeMap<_, _>, CheckpointError>>()?;
    let material = ProviderCheckpointLookupMaterial {
        logical_store_prefix: plan.logical_store_prefix.clone(),
        action_trust_policy_id: PROVIDER_ACTION_TRUST_POLICY_ID.to_string(),
        resource_bounds_digest_blake3: resource_bounds_digest(&plan.resource_bounds)?,
        stages,
        sources,
        policies,
        expected_native_provider_digest_blake3: plan.policies.expected_native_provider_digest_blake3.clone(),
    };
    digest_serialized(PROVIDER_CHECKPOINT_LOOKUP_CONTEXT, &material)
}

pub(crate) fn provider_checkpoint_manifest_digest(
    manifest: &ProviderCheckpointManifest,
) -> Result<String, CheckpointError> {
    validate_manifest_shape(manifest)?;
    digest_serialized(PROVIDER_CHECKPOINT_MANIFEST_CONTEXT, manifest)
}

fn build_stage_records(
    plan: &SourceBuiltFixedPointPlan,
    observations: Vec<ProviderCheckpointStageObservation>,
    resource_bounds_digest_blake3: &str,
) -> Result<Vec<ProviderCheckpointStageRecord>, CheckpointError> {
    let mut outputs = BTreeMap::new();
    let mut records = Vec::with_capacity(PROVIDER_CHECKPOINT_STAGE_COUNT);
    for (stage, observation) in plan.stages.iter().take(PROVIDER_CHECKPOINT_STAGE_COUNT).zip(observations) {
        validate_stage_observation(stage.output, &observation)?;
        let stage_authority_digest_blake3 =
            stage_authority_digest(plan, stage, &outputs, resource_bounds_digest_blake3)?;
        outputs.insert(stage.output, observation.output_digest_blake3.clone());
        records.push(ProviderCheckpointStageRecord {
            stage_id: stage.stage_id.clone(),
            output_role: stage.output,
            stage_authority_digest_blake3,
            output_digest_blake3: observation.output_digest_blake3,
            semantic_output_digest_blake3: observation.semantic_output_digest_blake3,
            payload_digest_blake3: observation.payload_digest_blake3,
            execution_evidence_digest_blake3: observation.execution_evidence_digest_blake3,
            producer_executable_digest_blake3: observation.producer_executable_digest_blake3,
            authority_violations: Vec::new(),
            fallback_events: Vec::new(),
        });
    }
    assert_eq!(records.len(), PROVIDER_CHECKPOINT_STAGE_COUNT);
    Ok(records)
}

fn validate_stage_records(
    plan: &SourceBuiltFixedPointPlan,
    records: &[ProviderCheckpointStageRecord],
    resource_bounds_digest_blake3: &str,
) -> Result<(), CheckpointError> {
    if records.len() != PROVIDER_CHECKPOINT_STAGE_COUNT {
        return Err(checkpoint_error("provider checkpoint does not contain the exact provider-stage prefix"));
    }
    let mut outputs = BTreeMap::new();
    for (stage, record) in plan.stages.iter().take(PROVIDER_CHECKPOINT_STAGE_COUNT).zip(records) {
        validate_stage_record(stage.stage_id.as_str(), stage.output, record)?;
        let expected = stage_authority_digest(plan, stage, &outputs, resource_bounds_digest_blake3)?;
        if record.stage_authority_digest_blake3 != expected {
            return Err(checkpoint_error(format!("checkpoint stage authority mismatch for {}", stage.stage_id)));
        }
        outputs.insert(record.output_role, record.output_digest_blake3.clone());
    }
    assert_eq!(outputs.len(), PROVIDER_CHECKPOINT_STAGE_COUNT);
    Ok(())
}

fn stage_authority_digest(
    plan: &SourceBuiltFixedPointPlan,
    stage: &crate::source_built_fixed_point::SourceBuiltFixedPointStagePlan,
    outputs: &BTreeMap<ProofOutputRole, String>,
    resource_bounds_digest_blake3: &str,
) -> Result<String, CheckpointError> {
    let mut inputs = stage
        .inputs
        .iter()
        .map(|input| resolve_authority_input(plan, *input, outputs))
        .collect::<Result<Vec<_>, _>>()?;
    inputs.sort();
    let material = StageAuthorityMaterial {
        stage_id: stage.stage_id.clone(),
        output_role: stage.output,
        resource_bounds_digest_blake3: resource_bounds_digest_blake3.to_string(),
        inputs,
    };
    digest_serialized(PROVIDER_CHECKPOINT_STAGE_CONTEXT, &material)
}

fn resolve_authority_input(
    plan: &SourceBuiltFixedPointPlan,
    input: StageAuthorityInput,
    outputs: &BTreeMap<ProofOutputRole, String>,
) -> Result<ResolvedAuthorityInput, CheckpointError> {
    match input {
        StageAuthorityInput::Source { role } => Ok(ResolvedAuthorityInput {
            authority_kind: "source".to_string(),
            authority_role: enum_name(role)?,
            digest_blake3: source_input(plan, role)?.digest_blake3.clone(),
        }),
        StageAuthorityInput::Policy { role } => Ok(ResolvedAuthorityInput {
            authority_kind: "policy".to_string(),
            authority_role: enum_name(role)?,
            digest_blake3: policy_digest(plan, role)?.to_string(),
        }),
        StageAuthorityInput::Output { role } => Ok(ResolvedAuthorityInput {
            authority_kind: "output".to_string(),
            authority_role: enum_name(role)?,
            digest_blake3: outputs
                .get(&role)
                .cloned()
                .ok_or_else(|| checkpoint_error(format!("checkpoint stage is missing predecessor output {role:?}")))?,
        }),
    }
}

fn validate_semantic_provider_outputs(
    plan: &SourceBuiltFixedPointPlan,
    records: &[ProviderCheckpointStageRecord],
    expected_stagex_provider_digest_blake3: &str,
) -> Result<(), CheckpointError> {
    let stagex_record = records
        .get(STAGEX_PROVIDER_STAGE_INDEX)
        .ok_or_else(|| checkpoint_error("checkpoint is missing the StageX provider stage"))?;
    if stagex_record.semantic_output_digest_blake3 != expected_stagex_provider_digest_blake3 {
        return Err(checkpoint_error("checkpoint StageX provider semantic identity mismatch"));
    }
    let native_record = records
        .get(NATIVE_PROVIDER_STAGE_INDEX)
        .ok_or_else(|| checkpoint_error("checkpoint is missing the native provider stage"))?;
    if native_record.semantic_output_digest_blake3 != plan.policies.expected_native_provider_digest_blake3 {
        return Err(checkpoint_error("checkpoint native provider semantic identity mismatch"));
    }
    assert_eq!(stagex_record.output_role, ProofOutputRole::StagexProvider);
    assert_eq!(native_record.output_role, ProofOutputRole::FullSourceNativeProvider);
    Ok(())
}

fn validate_stage_payload_links(
    records: &[ProviderCheckpointStageRecord],
    payloads: &[CheckpointPayloadIdentity],
) -> Result<(), CheckpointError> {
    for record in records {
        let payload_id = payload_id_for_output(record.output_role)?;
        let mut matches = payloads.iter().filter(|payload| payload.payload_id == payload_id);
        let payload = matches
            .next()
            .ok_or_else(|| checkpoint_error(format!("checkpoint stage payload is missing for {payload_id}")))?;
        if matches.next().is_some() || record.payload_digest_blake3 != payload.digest_blake3 {
            return Err(checkpoint_error(format!("checkpoint stage payload identity mismatch for {payload_id}")));
        }
        if matches!(record.output_role, ProofOutputRole::StagexTransition | ProofOutputRole::FullSourceRustProvider)
            && record.output_digest_blake3 != payload.digest_blake3
        {
            return Err(checkpoint_error(format!("checkpoint stage output is not bound to payload {payload_id}")));
        }
    }
    assert_eq!(records.len(), PROVIDER_CHECKPOINT_STAGE_COUNT);
    debug_assert!(records.iter().all(|record| !record.payload_digest_blake3.is_empty()));
    Ok(())
}

fn payload_id_for_output(role: ProofOutputRole) -> Result<&'static str, CheckpointError> {
    match role {
        ProofOutputRole::StagexTransition => Ok(PAYLOAD_STAGEX_TRANSITION),
        ProofOutputRole::StagexProvider => Ok(PAYLOAD_STAGEX_PROVIDER),
        ProofOutputRole::FullSourceNativeProvider => Ok(PAYLOAD_NATIVE_PROVIDER),
        ProofOutputRole::FullSourceRustProvider => Ok(PAYLOAD_RUST_PROVIDER),
        ProofOutputRole::MantleStage1 | ProofOutputRole::MantleStage2 => {
            Err(checkpoint_error("provider checkpoint contains a Mantle output stage"))
        }
    }
}

fn validate_manifest_shape(manifest: &ProviderCheckpointManifest) -> Result<(), CheckpointError> {
    if manifest.schema != PROVIDER_CHECKPOINT_SCHEMA {
        return Err(checkpoint_error("provider checkpoint schema mismatch"));
    }
    validate_digest("checkpoint lookup key", &manifest.lookup_key_blake3)?;
    validate_digest("checkpoint resource bounds", &manifest.resource_bounds_digest_blake3)?;
    if manifest.stages.len() != PROVIDER_CHECKPOINT_STAGE_COUNT {
        return Err(checkpoint_error("provider checkpoint stage count mismatch"));
    }
    validate_payloads(&manifest.payloads)?;
    Ok(())
}

fn validate_stage_observation(
    expected_output_role: ProofOutputRole,
    observation: &ProviderCheckpointStageObservation,
) -> Result<(), CheckpointError> {
    if observation.output_role != expected_output_role {
        return Err(checkpoint_error("checkpoint observation output role does not match the stage"));
    }
    for (label, digest) in [
        ("stage output", observation.output_digest_blake3.as_str()),
        ("stage semantic output", observation.semantic_output_digest_blake3.as_str()),
        ("stage payload", observation.payload_digest_blake3.as_str()),
        ("stage execution evidence", observation.execution_evidence_digest_blake3.as_str()),
        ("stage producer executable", observation.producer_executable_digest_blake3.as_str()),
    ] {
        validate_digest(label, digest)?;
    }
    Ok(())
}

fn validate_stage_record(
    expected_stage_id: &str,
    expected_output_role: ProofOutputRole,
    record: &ProviderCheckpointStageRecord,
) -> Result<(), CheckpointError> {
    if record.stage_id != expected_stage_id || record.output_role != expected_output_role {
        return Err(checkpoint_error(format!("checkpoint stage shape mismatch for {expected_stage_id}")));
    }
    if !record.authority_violations.is_empty() || !record.fallback_events.is_empty() {
        return Err(checkpoint_error(format!("checkpoint stage {expected_stage_id} contains forbidden events")));
    }
    for (label, digest) in [
        ("stage authority", record.stage_authority_digest_blake3.as_str()),
        ("stage output", record.output_digest_blake3.as_str()),
        ("stage semantic output", record.semantic_output_digest_blake3.as_str()),
        ("stage payload", record.payload_digest_blake3.as_str()),
        ("stage execution evidence", record.execution_evidence_digest_blake3.as_str()),
        ("stage producer executable", record.producer_executable_digest_blake3.as_str()),
    ] {
        validate_digest(label, digest)?;
    }
    Ok(())
}

fn validate_observation_count(observations: &[ProviderCheckpointStageObservation]) -> Result<(), CheckpointError> {
    if observations.len() != PROVIDER_CHECKPOINT_STAGE_COUNT {
        return Err(checkpoint_error("provider checkpoint observations must cover the exact provider-stage prefix"));
    }
    Ok(())
}

fn validate_payloads(payloads: &[CheckpointPayloadIdentity]) -> Result<(), CheckpointError> {
    if payloads.len() != PROVIDER_CHECKPOINT_PAYLOAD_COUNT {
        return Err(checkpoint_error("provider checkpoint payload count mismatch"));
    }
    let required = REQUIRED_PAYLOAD_IDS.into_iter().collect::<BTreeSet<_>>();
    let mut observed = BTreeSet::new();
    for payload in payloads {
        validate_text("checkpoint payload id", &payload.payload_id)?;
        validate_text("checkpoint payload path", &payload.relative_path)?;
        validate_digest("checkpoint payload", &payload.digest_blake3)?;
        if payload.kind != expected_payload_kind(&payload.payload_id)? {
            return Err(checkpoint_error(format!(
                "checkpoint payload {} has the wrong content kind",
                payload.payload_id
            )));
        }
        if payload.total_file_bytes == 0 || payload.entry_count == 0 {
            return Err(checkpoint_error(format!("checkpoint payload {} has empty bounds", payload.payload_id)));
        }
        if !observed.insert(payload.payload_id.as_str()) {
            return Err(checkpoint_error(format!("duplicate checkpoint payload {}", payload.payload_id)));
        }
    }
    if observed != required {
        return Err(checkpoint_error("provider checkpoint payload roles are incomplete or unknown"));
    }
    Ok(())
}

fn expected_payload_kind(payload_id: &str) -> Result<CheckpointPayloadKind, CheckpointError> {
    match payload_id {
        PAYLOAD_STAGEX_TRANSITION => Ok(CheckpointPayloadKind::PreservedTree),
        PAYLOAD_STAGEX_PROVIDER
        | PAYLOAD_NATIVE_PROVIDER
        | PAYLOAD_RUST_PROVIDER
        | PAYLOAD_RUST_HOST_MAKE
        | PAYLOAD_RUST_HOST_CMAKE
        | PAYLOAD_RUST_HOST_PYTHON
        | PAYLOAD_RUST_HOST_PERL
        | PAYLOAD_RUST_HOST_BUSYBOX
        | PAYLOAD_RUST_HOST_LINUX_HEADERS => Ok(CheckpointPayloadKind::Directory),
        PAYLOAD_NATIVE_ADMISSION
        | PAYLOAD_NATIVE_TRANSCRIPT
        | PAYLOAD_TOOLCHAIN_CLOSURE
        | PAYLOAD_NATIVE_ACTION_PLAN
        | PAYLOAD_NATIVE_ACTION_RECONCILIATION => Ok(CheckpointPayloadKind::RegularFile),
        PAYLOAD_RUST_HOST_EVIDENCE | PAYLOAD_RUST_ACTION_TRUST => Ok(CheckpointPayloadKind::PreservedTree),
        _ => Err(checkpoint_error(format!("unknown checkpoint payload role {payload_id}"))),
    }
}

fn require_provider_stage_prefix(plan: &SourceBuiltFixedPointPlan) -> Result<(), CheckpointError> {
    if plan.stages.len() < PROVIDER_CHECKPOINT_STAGE_COUNT {
        return Err(checkpoint_error("source-built plan has fewer stages than the provider checkpoint boundary"));
    }
    let expected_outputs = [
        ProofOutputRole::StagexTransition,
        ProofOutputRole::StagexProvider,
        ProofOutputRole::FullSourceNativeProvider,
        ProofOutputRole::FullSourceRustProvider,
    ];
    for (stage, expected_output) in plan.stages.iter().take(PROVIDER_CHECKPOINT_STAGE_COUNT).zip(expected_outputs) {
        if stage.output != expected_output {
            return Err(checkpoint_error("source-built plan provider-stage order is incompatible with checkpoints"));
        }
    }
    Ok(())
}

fn collect_non_output_roles(
    inputs: &[StageAuthorityInput],
    source_roles: &mut BTreeSet<SourceAuthorityRole>,
    policy_roles: &mut BTreeSet<ProofPolicyRole>,
) {
    for input in inputs {
        match input {
            StageAuthorityInput::Source { role } => {
                source_roles.insert(*role);
            }
            StageAuthorityInput::Policy { role } => {
                policy_roles.insert(*role);
            }
            StageAuthorityInput::Output { .. } => {}
        }
    }
}

fn source_input(
    plan: &SourceBuiltFixedPointPlan,
    role: SourceAuthorityRole,
) -> Result<&SourceAuthorityInput, CheckpointError> {
    let mut matches = plan.source_inputs.iter().filter(|input| input.role == role);
    let input = matches
        .next()
        .ok_or_else(|| checkpoint_error(format!("checkpoint lookup is missing source role {role:?}")))?;
    if matches.next().is_some() {
        return Err(checkpoint_error(format!("checkpoint lookup has duplicate source role {role:?}")));
    }
    validate_digest("source authority", &input.digest_blake3)?;
    Ok(input)
}

fn policy_digest(plan: &SourceBuiltFixedPointPlan, role: ProofPolicyRole) -> Result<&str, CheckpointError> {
    let value = match role {
        ProofPolicyRole::Closure => &plan.policies.closure_policy_digest_blake3,
        ProofPolicyRole::Hermeticity => &plan.policies.hermeticity_policy_digest_blake3,
        ProofPolicyRole::ProtectedExecution => &plan.policies.protected_execution_policy_digest_blake3,
        ProofPolicyRole::Effect => &plan.policies.effect_policy_digest_blake3,
        ProofPolicyRole::Normalization => &plan.policies.normalization_policy_digest_blake3,
    };
    validate_digest("checkpoint policy", value)?;
    Ok(value)
}

fn resource_bounds_digest(bounds: &SourceBuiltFixedPointResourceBounds) -> Result<String, CheckpointError> {
    digest_serialized(RESOURCE_BOUNDS_CONTEXT, bounds)
}

fn enum_name<T: Serialize>(value: T) -> Result<String, CheckpointError> {
    let encoded = serde_json::to_string(&value)
        .map_err(|error| checkpoint_error(format!("serializing checkpoint authority role: {error}")))?;
    Ok(encoded.trim_matches('"').to_string())
}

fn digest_serialized<T: Serialize>(context: &str, value: &T) -> Result<String, CheckpointError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| checkpoint_error(format!("serializing checkpoint identity: {error}")))?;
    let mut hasher = blake3::Hasher::new_derive_key(context);
    hasher.update(&bytes);
    let digest = hasher.finalize().to_hex().to_string();
    assert!(!bytes.is_empty());
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    Ok(digest)
}

fn validate_digest(label: &str, value: &str) -> Result<(), CheckpointError> {
    let valid = value.len() == BLAKE3_HEX_LENGTH
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid {
        return Err(checkpoint_error(format!("{label} must be a lowercase BLAKE3 digest")));
    }
    Ok(())
}

fn validate_text(label: &str, value: &str) -> Result<(), CheckpointError> {
    let valid = !value.is_empty()
        && value.len() <= TEXT_BYTES_MAX
        && !value.chars().any(char::is_control)
        && !value.starts_with('/');
    if !valid {
        return Err(checkpoint_error(format!("{label} must be bounded relative UTF-8 text")));
    }
    Ok(())
}

fn checkpoint_error(message: impl Into<String>) -> CheckpointError {
    CheckpointError {
        message: message.into(),
    }
}

#[cfg(test)]
const CHECKPOINT_TEST_BYTES: u64 = 1;
#[cfg(test)]
const CHECKPOINT_TEST_EVENTS_MAX: u32 = 1;
#[cfg(test)]
const CHECKPOINT_TEST_RECORDS_MAX: u32 = 8;
#[cfg(test)]
const CHECKPOINT_TEST_FILE_DESCRIPTORS_MAX: u64 = 1;
#[cfg(test)]
const CHECKPOINT_TEST_POLICY_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[cfg(test)]
pub(crate) fn checkpoint_test_plan(
    mantle_source_digest: &str,
    native_provider_digest: &str,
) -> SourceBuiltFixedPointPlan {
    use crate::source_built_fixed_point::InitialOutputAuthorityState;
    use crate::source_built_fixed_point::SourceBuiltFixedPointPlanInput;
    use crate::source_built_fixed_point::plan_source_built_fixed_point;

    plan_source_built_fixed_point(SourceBuiltFixedPointPlanInput {
        proof_id: "checkpoint-test".to_string(),
        logical_store_prefix: "/mantle/store".to_string(),
        source_inputs: checkpoint_test_source_inputs(mantle_source_digest),
        initial_output_authority: InitialOutputAuthorityState {
            stagex_transition_entries: 0,
            native_provider_entries: 0,
            rust_provider_entries: 0,
            mantle_output_entries: 0,
        },
        policies: checkpoint_test_policies(native_provider_digest),
        resource_bounds: SourceBuiltFixedPointResourceBounds {
            elapsed_seconds_max: CHECKPOINT_TEST_BYTES,
            disk_bytes_max: CHECKPOINT_TEST_BYTES,
            open_file_descriptors_max: CHECKPOINT_TEST_FILE_DESCRIPTORS_MAX,
            protected_exec_events_max: CHECKPOINT_TEST_EVENTS_MAX,
            source_records_max: CHECKPOINT_TEST_RECORDS_MAX,
        },
    })
    .unwrap()
}

#[cfg(test)]
fn checkpoint_test_source_inputs(mantle_source_digest: &str) -> Vec<SourceAuthorityInput> {
    let roles = [
        SourceAuthorityRole::StagexSeed,
        SourceAuthorityRole::StagexLineage,
        SourceAuthorityRole::StagexSourceBundle,
        SourceAuthorityRole::NativeSourceBundle,
        SourceAuthorityRole::RustSourceArchiveSet,
        SourceAuthorityRole::ProviderRecipeProjection,
        SourceAuthorityRole::MantleSource,
        SourceAuthorityRole::VendorInputs,
    ];
    roles
        .into_iter()
        .enumerate()
        .map(|(index, role)| SourceAuthorityInput {
            id: format!("source-{index}"),
            role,
            kind: checkpoint_test_source_kind(role),
            digest_blake3: if role == SourceAuthorityRole::MantleSource {
                mantle_source_digest.to_string()
            } else {
                CHECKPOINT_TEST_POLICY_DIGEST.to_string()
            },
            size_bytes: CHECKPOINT_TEST_BYTES,
        })
        .collect()
}

#[cfg(test)]
fn checkpoint_test_policies(
    native_provider_digest: &str,
) -> crate::source_built_fixed_point::SourceBuiltFixedPointPolicies {
    use crate::source_built_fixed_point::ProofHermeticityMode;
    use crate::source_built_fixed_point::SourceBuiltFixedPointPolicies;

    SourceBuiltFixedPointPolicies {
        expected_native_provider_digest_blake3: native_provider_digest.to_string(),
        closure_policy_digest_blake3: CHECKPOINT_TEST_POLICY_DIGEST.to_string(),
        hermeticity_policy_digest_blake3: CHECKPOINT_TEST_POLICY_DIGEST.to_string(),
        protected_execution_policy_digest_blake3: CHECKPOINT_TEST_POLICY_DIGEST.to_string(),
        effect_policy_digest_blake3: CHECKPOINT_TEST_POLICY_DIGEST.to_string(),
        normalization_policy_digest_blake3: CHECKPOINT_TEST_POLICY_DIGEST.to_string(),
        hermeticity_mode: ProofHermeticityMode::Strict,
        live_fetch_allowed: false,
        cargo_invocation_allowed: false,
        ambient_discovery_allowed: false,
        fallback_allowed: false,
        provider_cache_completion_allowed: false,
    }
}

#[cfg(test)]
fn checkpoint_test_source_kind(role: SourceAuthorityRole) -> crate::source_built_fixed_point::SourceContentKind {
    use crate::source_built_fixed_point::SourceContentKind;

    match role {
        SourceAuthorityRole::StagexSeed
        | SourceAuthorityRole::StagexLineage
        | SourceAuthorityRole::StagexSourceBundle
        | SourceAuthorityRole::NativeSourceBundle => SourceContentKind::RegularFile,
        SourceAuthorityRole::RustSourceArchiveSet
        | SourceAuthorityRole::ProviderRecipeProjection
        | SourceAuthorityRole::MantleSource
        | SourceAuthorityRole::VendorInputs
        | SourceAuthorityRole::ImportedNativeProvider
        | SourceAuthorityRole::ImportedRustProvider
        | SourceAuthorityRole::PriorMantleOutput => SourceContentKind::Directory,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const DIGEST_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

    #[test]
    fn promoted_checkpoint_accepts_exact_stage_authority_and_payloads() {
        let plan = test_plan(DIGEST_A, DIGEST_B);
        let payloads = test_payloads();
        let manifest = build_provider_checkpoint_manifest(
            &plan,
            ProofCheckpointOrigin::PromotedExecution,
            test_observations(),
            payloads.clone(),
        )
        .unwrap();

        let admission = admit_promoted_provider_checkpoint(&plan, &manifest, &payloads, DIGEST_B).unwrap();

        assert_eq!(admission.lookup_key_blake3, manifest.lookup_key_blake3);
        assert_eq!(admission.completed_stage_count as usize, PROVIDER_CHECKPOINT_STAGE_COUNT);
        assert_eq!(admission.checkpoint_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    }

    #[test]
    fn provider_lookup_ignores_later_mantle_source_but_tracks_provider_sources() {
        let original = test_plan(DIGEST_A, DIGEST_B);
        let changed_mantle = test_plan(DIGEST_C, DIGEST_B);
        let mut changed_provider = test_plan(DIGEST_A, DIGEST_B);
        source_input_mut(&mut changed_provider, SourceAuthorityRole::RustSourceArchiveSet).digest_blake3 =
            DIGEST_D.to_string();
        let mut changed_recipes = test_plan(DIGEST_A, DIGEST_B);
        source_input_mut(&mut changed_recipes, SourceAuthorityRole::ProviderRecipeProjection).digest_blake3 =
            DIGEST_D.to_string();

        let original_key = provider_checkpoint_lookup_key(&original).unwrap();
        let changed_mantle_key = provider_checkpoint_lookup_key(&changed_mantle).unwrap();
        let changed_provider_key = provider_checkpoint_lookup_key(&changed_provider).unwrap();
        let changed_recipe_key = provider_checkpoint_lookup_key(&changed_recipes).unwrap();

        assert_eq!(original_key, changed_mantle_key);
        assert_ne!(original_key, changed_provider_key);
        assert_ne!(original_key, changed_recipe_key);
        assert_eq!(original_key.len(), BLAKE3_HEX_LENGTH);
    }

    #[test]
    fn promoted_checkpoint_rejects_legacy_schema_without_action_payloads() {
        let plan = test_plan(DIGEST_A, DIGEST_B);
        let payloads = test_payloads();
        let mut manifest = build_provider_checkpoint_manifest(
            &plan,
            ProofCheckpointOrigin::PromotedExecution,
            test_observations(),
            payloads.clone(),
        )
        .unwrap();
        manifest.schema = "mantle-source-built-provider-checkpoint-v1".to_string();
        manifest.payloads.retain(|payload| {
            payload.payload_id != PAYLOAD_NATIVE_ACTION_PLAN
                && payload.payload_id != PAYLOAD_NATIVE_ACTION_RECONCILIATION
        });

        let error = admit_promoted_provider_checkpoint(&plan, &manifest, &payloads, DIGEST_B).unwrap_err();

        assert!(error.to_string().contains("schema mismatch"));
        assert_ne!(manifest.schema, PROVIDER_CHECKPOINT_SCHEMA);
    }

    #[test]
    fn promoted_checkpoint_rejects_dev_origin_and_changed_payload() {
        let plan = test_plan(DIGEST_A, DIGEST_B);
        let payloads = test_payloads();
        let dev_manifest = build_provider_checkpoint_manifest(
            &plan,
            ProofCheckpointOrigin::DevExecution,
            test_observations(),
            payloads.clone(),
        )
        .unwrap();
        let mut changed_payloads = payloads.clone();
        changed_payloads[FIRST_PROVIDER_STAGE_INDEX].digest_blake3 = DIGEST_D.to_string();

        let dev_error = admit_promoted_provider_checkpoint(&plan, &dev_manifest, &payloads, DIGEST_B).unwrap_err();
        let promoted_manifest = build_provider_checkpoint_manifest(
            &plan,
            ProofCheckpointOrigin::PromotedExecution,
            test_observations(),
            payloads.clone(),
        )
        .unwrap();
        let payload_error =
            admit_promoted_provider_checkpoint(&plan, &promoted_manifest, &changed_payloads, DIGEST_B).unwrap_err();

        let dev_admission = admit_dev_provider_checkpoint(&plan, &dev_manifest, &payloads, DIGEST_B).unwrap();

        assert!(dev_error.to_string().contains("dev checkpoint"));
        assert_eq!(usize::try_from(dev_admission.completed_stage_count).unwrap(), PROVIDER_CHECKPOINT_STAGE_COUNT);
        assert!(payload_error.to_string().contains("payload observations"));
    }

    #[test]
    fn promoted_checkpoint_rejects_changed_relevant_policy_and_forbidden_events() {
        let plan = test_plan(DIGEST_A, DIGEST_B);
        let payloads = test_payloads();
        let mut manifest = build_provider_checkpoint_manifest(
            &plan,
            ProofCheckpointOrigin::PromotedExecution,
            test_observations(),
            payloads.clone(),
        )
        .unwrap();
        manifest.stages[FIRST_PROVIDER_STAGE_INDEX].fallback_events.push("ambient-tool".to_string());
        let event_error = admit_promoted_provider_checkpoint(&plan, &manifest, &payloads, DIGEST_B).unwrap_err();
        let mut changed_policy = plan.clone();
        changed_policy.policies.protected_execution_policy_digest_blake3 = DIGEST_D.to_string();
        manifest.stages[FIRST_PROVIDER_STAGE_INDEX].fallback_events.clear();
        let policy_error =
            admit_promoted_provider_checkpoint(&changed_policy, &manifest, &payloads, DIGEST_B).unwrap_err();

        assert!(event_error.to_string().contains("forbidden events"));
        assert!(policy_error.to_string().contains("lookup authority"));
    }

    #[test]
    fn promoted_checkpoint_rejects_wrong_semantic_provider_identity() {
        let plan = test_plan(DIGEST_A, DIGEST_B);
        let payloads = test_payloads();
        let mut observations = test_observations();
        observations[STAGEX_PROVIDER_STAGE_INDEX].semantic_output_digest_blake3 = DIGEST_D.to_string();
        let manifest = build_provider_checkpoint_manifest(
            &plan,
            ProofCheckpointOrigin::PromotedExecution,
            observations,
            payloads.clone(),
        )
        .unwrap();

        let error = admit_promoted_provider_checkpoint(&plan, &manifest, &payloads, DIGEST_B).unwrap_err();

        assert!(error.to_string().contains("StageX provider semantic identity"));
        assert!(!error.to_string().contains("native provider"));
    }

    fn test_plan(mantle_source_digest: &str, native_provider_digest: &str) -> SourceBuiltFixedPointPlan {
        checkpoint_test_plan(mantle_source_digest, native_provider_digest)
    }

    fn source_input_mut(plan: &mut SourceBuiltFixedPointPlan, role: SourceAuthorityRole) -> &mut SourceAuthorityInput {
        plan.source_inputs.iter_mut().find(|input| input.role == role).unwrap()
    }

    fn test_observations() -> Vec<ProviderCheckpointStageObservation> {
        vec![
            stage_observation(ProofOutputRole::StagexTransition, DIGEST_A),
            stage_observation(ProofOutputRole::StagexProvider, DIGEST_B),
            stage_observation(ProofOutputRole::FullSourceNativeProvider, DIGEST_B),
            stage_observation(ProofOutputRole::FullSourceRustProvider, DIGEST_C),
        ]
    }

    fn stage_observation(
        output_role: ProofOutputRole,
        semantic_output_digest_blake3: &str,
    ) -> ProviderCheckpointStageObservation {
        ProviderCheckpointStageObservation {
            output_role,
            output_digest_blake3: semantic_output_digest_blake3.to_string(),
            semantic_output_digest_blake3: semantic_output_digest_blake3.to_string(),
            payload_digest_blake3: semantic_output_digest_blake3.to_string(),
            execution_evidence_digest_blake3: DIGEST_A.to_string(),
            producer_executable_digest_blake3: DIGEST_A.to_string(),
        }
    }

    fn test_payloads() -> Vec<CheckpointPayloadIdentity> {
        REQUIRED_PAYLOAD_IDS
            .into_iter()
            .enumerate()
            .map(|(index, payload_id)| CheckpointPayloadIdentity {
                payload_id: payload_id.to_string(),
                relative_path: format!("payload/{index}"),
                kind: expected_payload_kind(payload_id).unwrap(),
                digest_blake3: match payload_id {
                    PAYLOAD_STAGEX_PROVIDER | PAYLOAD_NATIVE_PROVIDER => DIGEST_B.to_string(),
                    PAYLOAD_RUST_PROVIDER => DIGEST_C.to_string(),
                    _ => DIGEST_A.to_string(),
                },
                total_file_bytes: CHECKPOINT_TEST_BYTES,
                entry_count: 1,
            })
            .collect()
    }
}
