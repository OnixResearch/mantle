//! Pure bounded remote resource, lease, locality, and placement decisions.
//!
//! This functional core performs no I/O, clock reads, persistence, transport,
//! provider discovery, or output admission. Shell adapters own receiver probes
//! and durable coordinator mutation.
//!
//! r[impl build_scheduling.quantified_resource_admission]
//! r[impl build_scheduling.verified_locality_placement]
//! r[impl remote_builds.fenced_worker_resource_leases]

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use super::remote_attempt::RemoteAttemptId;
use super::remote_attempt::RemoteAttemptPhase;
use super::remote_attempt::RemoteFenceGeneration;
use super::remote_attempt::RemoteJobId;
use super::remote_transfer::CanonicalRemoteTransferManifest;
use super::remote_transfer::RemoteTransferReceiverFacts;
use super::remote_transfer::plan_remote_transfer_demand;
use crate::scheduling::ContentLocalityClass;
use crate::scheduling::PreferenceField;
use crate::scheduling::ResourceFitClass;
use crate::scheduling::SchedulingPolicy;
use crate::scheduling::TransferCostClass;

pub const REMOTE_RESOURCE_LEASE_SCHEMA: &str = "mantle-remote-resource-lease-v1";
pub const REMOTE_LOCALITY_SUMMARY_SCHEMA: &str = "mantle-verified-locality-summary-v1";
pub const REMOTE_RESOURCE_REQUIREMENT_DOMAIN: &str = "mantle-remote-resource-requirement-v1";
pub const REMOTE_RESOURCE_RESERVATION_DOMAIN: &str = "mantle-remote-resource-reservation-v1";
pub const REMOTE_RESOURCE_LEASE_DOMAIN: &str = "mantle-remote-resource-lease-v1";
pub const REMOTE_LOCALITY_CLAIM_SCOPE: &str = "receiver-verified-current-manifest-placement-only";
pub const REMOTE_RESOURCE_CLAIM_SCOPE: &str = "current-fenced-scheduling-capacity-only";
pub const MAX_REMOTE_RESOURCE_CLASSES: usize = 32;
pub const MAX_REMOTE_RESOURCE_CLASS_NAME_BYTES: usize = 64;
pub const MAX_REMOTE_RESOURCE_LEASES: usize = 4_096;
pub const MAX_REMOTE_LOCALITY_OBSERVATIONS: usize = 4_096;
pub const MAX_REMOTE_CPU_UNITS: u32 = 1_048_576;
pub const MAX_REMOTE_RESOURCE_COUNT: u32 = 1_048_576;
pub const MAX_REMOTE_RESOURCE_BYTES: u64 = 1_125_899_906_842_624;
pub const MAX_REMOTE_PLACEMENT_CANDIDATES: usize = 1_024;
const BLAKE3_HEX_LENGTH: usize = 64;
const REMOTE_LOCALITY_NON_CLAIMS: [&str; 5] = [
    "no global placement optimality",
    "no future content availability",
    "no execution success",
    "no output trust",
    "no release reproducibility",
];
const REMOTE_RESOURCE_NON_CLAIMS: [&str; 5] = [
    "resource authorization is not tool identity",
    "named tokens are not license credentials or compliance evidence",
    "no provider autoscaling claim",
    "no execution success",
    "no output trust",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteNamedResourceQuantity {
    pub name: String,
    pub quantity: u32,
}

fn empty_named_resource_quantities() -> Vec<RemoteNamedResourceQuantity> {
    Vec::new()
}

fn empty_resource_class_names() -> Vec<String> {
    Vec::new()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteResourceVector {
    pub cpu_units: u32,
    pub memory_bytes: u64,
    pub scratch_bytes: u64,
    #[serde(default = "empty_named_resource_quantities")]
    pub accelerators: Vec<RemoteNamedResourceQuantity>,
    #[serde(default = "empty_named_resource_quantities")]
    pub named_tokens: Vec<RemoteNamedResourceQuantity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteResourceRequirements {
    pub quantities: RemoteResourceVector,
    #[serde(default = "empty_resource_class_names")]
    pub semantic_accelerator_classes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteWorkerResourceInventory {
    pub total: RemoteResourceVector,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteResourceAvailability {
    pub cpu_units: u32,
    pub memory_bytes: u64,
    pub scratch_bytes: u64,
    pub accelerators: Vec<RemoteNamedResourceQuantity>,
    pub named_tokens: Vec<RemoteNamedResourceQuantity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteResourceLeaseScope {
    pub worker_endpoint_id: String,
    pub worker_generation: u64,
    pub job_id: RemoteJobId,
    pub attempt_id: RemoteAttemptId,
    pub fence_generation: RemoteFenceGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteResourceLease {
    pub schema: String,
    pub lease_id_blake3: String,
    pub scope: RemoteResourceLeaseScope,
    pub requirement_digest_blake3: String,
    pub reservation_digest_blake3: String,
    pub reserved: RemoteResourceVector,
    pub claim_scope: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteResourceAdmissionPlan {
    pub lease: RemoteResourceLease,
    pub remaining_before: RemoteResourceAvailability,
    pub remaining_after: RemoteResourceAvailability,
    pub resource_fit: ResourceFitClass,
    pub reason_code: RemoteResourceReasonCode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteResourceAvailabilityPlan {
    pub remaining_before: RemoteResourceAvailability,
    pub remaining_after: RemoteResourceAvailability,
    pub resource_fit: ResourceFitClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteResourceLeaseMutationKind {
    Renew,
    Release,
    Resize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteResourceLeaseReleasePlan {
    pub lease_id_blake3: String,
    pub released: RemoteResourceVector,
    pub reason_code: RemoteResourceReasonCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteResourceRecoveryDisposition {
    Preserve,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteResourceRecoveryPlan {
    pub disposition: RemoteResourceRecoveryDisposition,
    pub reason_code: RemoteResourceReasonCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteLocalityScope {
    pub manifest_digest_blake3: String,
    pub policy_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteLocalityProbeObservation {
    pub worker_endpoint_id: String,
    pub worker_generation: u64,
    pub scope: RemoteLocalityScope,
    pub receiver_probe_verified: bool,
    pub receiver_facts: RemoteTransferReceiverFacts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteVerifiedLocalitySummary {
    pub schema: String,
    pub worker_endpoint_id: String,
    pub worker_generation: u64,
    pub scope: RemoteLocalityScope,
    pub demanded_object_count: u32,
    pub verified_present_object_count: u32,
    pub missing_object_count: u32,
    pub demanded_bytes: u64,
    pub verified_present_bytes: u64,
    pub missing_bytes: u64,
    pub content_locality: ContentLocalityClass,
    pub transfer_cost: TransferCostClass,
    pub reason_code: RemoteLocalityReasonCode,
    pub claim_scope: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteWorkerPlacementFacts {
    pub worker_endpoint_id: String,
    pub resource_fit: ResourceFitClass,
    pub content_locality: ContentLocalityClass,
    pub transfer_cost: TransferCostClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteResourceReasonCode {
    ReservationPlanned,
    LeaseCurrent,
    LeaseReleased,
    LeaseResized,
    LeasePreservedOnRecovery,
    LeaseReleasedOnRecovery,
    QuantityZero,
    QuantityExceedsLimit,
    ResourceClassCountExceeded,
    ResourceClassNameInvalid,
    ResourceClassDuplicate,
    SemanticAcceleratorMismatch,
    CpuUnavailable,
    MemoryUnavailable,
    ScratchUnavailable,
    AcceleratorUnavailable,
    NamedTokenUnavailable,
    ArithmeticOverflow,
    LeaseCountExceeded,
    LeaseSchemaUnsupported,
    LeaseDigestInvalid,
    LeaseWorkerMismatch,
    LeaseJobMismatch,
    LeaseAttemptMismatch,
    LeaseStaleFence,
    LeaseUnknownFence,
    LeaseSnapshotOvercommitted,
    PlacementCandidatesInvalid,
}

impl RemoteResourceReasonCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReservationPlanned => "resource-reservation-planned",
            Self::LeaseCurrent => "resource-lease-current",
            Self::LeaseReleased => "resource-lease-released",
            Self::LeaseResized => "resource-lease-resized",
            Self::LeasePreservedOnRecovery => "resource-lease-preserved-on-recovery",
            Self::LeaseReleasedOnRecovery => "resource-lease-released-on-recovery",
            Self::QuantityZero => "resource-quantity-zero",
            Self::QuantityExceedsLimit => "resource-quantity-exceeds-limit",
            Self::ResourceClassCountExceeded => "resource-class-count-exceeded",
            Self::ResourceClassNameInvalid => "resource-class-name-invalid",
            Self::ResourceClassDuplicate => "resource-class-duplicate",
            Self::SemanticAcceleratorMismatch => "semantic-accelerator-class-mismatch",
            Self::CpuUnavailable => "resource-cpu-unavailable",
            Self::MemoryUnavailable => "resource-memory-unavailable",
            Self::ScratchUnavailable => "resource-scratch-unavailable",
            Self::AcceleratorUnavailable => "resource-accelerator-unavailable",
            Self::NamedTokenUnavailable => "resource-named-token-unavailable",
            Self::ArithmeticOverflow => "resource-arithmetic-overflow",
            Self::LeaseCountExceeded => "resource-lease-count-exceeded",
            Self::LeaseSchemaUnsupported => "resource-lease-schema-unsupported",
            Self::LeaseDigestInvalid => "resource-lease-digest-invalid",
            Self::LeaseWorkerMismatch => "resource-lease-worker-mismatch",
            Self::LeaseJobMismatch => "resource-lease-job-mismatch",
            Self::LeaseAttemptMismatch => "resource-lease-attempt-mismatch",
            Self::LeaseStaleFence => "resource-lease-stale-fence",
            Self::LeaseUnknownFence => "resource-lease-unknown-fence",
            Self::LeaseSnapshotOvercommitted => "resource-lease-snapshot-overcommitted",
            Self::PlacementCandidatesInvalid => "resource-placement-candidates-invalid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteLocalityReasonCode {
    VerifiedFullyPresent,
    VerifiedPartiallyPresent,
    VerifiedNoContent,
    UnverifiedHintDowngraded,
    WorkerIdentityMismatch,
    WorkerGenerationStale,
    ManifestMismatch,
    PolicyMismatch,
    TransferFactsInvalid,
    ArithmeticOverflow,
}

impl RemoteLocalityReasonCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VerifiedFullyPresent => "locality-verified-fully-present",
            Self::VerifiedPartiallyPresent => "locality-verified-partially-present",
            Self::VerifiedNoContent => "locality-verified-no-content",
            Self::UnverifiedHintDowngraded => "locality-unverified-hint-downgraded",
            Self::WorkerIdentityMismatch => "locality-worker-identity-mismatch",
            Self::WorkerGenerationStale => "locality-worker-generation-stale",
            Self::ManifestMismatch => "locality-manifest-mismatch",
            Self::PolicyMismatch => "locality-policy-mismatch",
            Self::TransferFactsInvalid => "locality-transfer-facts-invalid",
            Self::ArithmeticOverflow => "locality-arithmetic-overflow",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ResourceTotals {
    cpu_units: u32,
    memory_bytes: u64,
    scratch_bytes: u64,
    accelerators: BTreeMap<String, u32>,
    named_tokens: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LocalityQuantities {
    present_object_count: u32,
    missing_object_count: u32,
    present_bytes: u64,
    missing_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LocalityClassification {
    content_locality: ContentLocalityClass,
    transfer_cost: TransferCostClass,
    reason_code: RemoteLocalityReasonCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PositiveU32Bound {
    value: u32,
    maximum: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PositiveU64Bound {
    value: u64,
    maximum: u64,
}

pub fn canonical_remote_resource_requirements(
    requirements: &RemoteResourceRequirements,
) -> Result<RemoteResourceRequirements, RemoteResourceReasonCode> {
    let quantities = canonical_positive_resource_vector(&requirements.quantities)?;
    let semantic = canonical_resource_class_names(&requirements.semantic_accelerator_classes)?;
    let accelerator_names = quantities.accelerators.iter().map(|quantity| quantity.name.clone()).collect::<Vec<_>>();
    if semantic != accelerator_names {
        return Err(RemoteResourceReasonCode::SemanticAcceleratorMismatch);
    }
    debug_assert_eq!(semantic.len(), quantities.accelerators.len());
    debug_assert!(quantities.cpu_units > 0);
    Ok(RemoteResourceRequirements {
        quantities,
        semantic_accelerator_classes: semantic,
    })
}

pub fn canonical_remote_worker_resource_inventory(
    inventory: &RemoteWorkerResourceInventory,
) -> Result<RemoteWorkerResourceInventory, RemoteResourceReasonCode> {
    let total = canonical_positive_resource_vector(&inventory.total)?;
    debug_assert!(total.memory_bytes > 0);
    debug_assert!(total.scratch_bytes > 0);
    Ok(RemoteWorkerResourceInventory { total })
}

pub fn remote_resource_requirement_digest(
    requirements: &RemoteResourceRequirements,
) -> Result<String, RemoteResourceReasonCode> {
    let canonical = canonical_remote_resource_requirements(requirements)?;
    domain_hash_json(REMOTE_RESOURCE_REQUIREMENT_DOMAIN, &canonical)
}

pub fn remote_resource_remaining_capacity(
    worker_endpoint_id: &str,
    inventory: &RemoteWorkerResourceInventory,
    active_leases: &[RemoteResourceLease],
) -> Result<RemoteResourceAvailability, RemoteResourceReasonCode> {
    validate_resource_identity(worker_endpoint_id)?;
    let inventory = canonical_remote_worker_resource_inventory(inventory)?;
    let total = totals_from_vector(&inventory.total);
    let used = aggregate_worker_leases(worker_endpoint_id, active_leases)?;
    let remaining = subtract_totals(&total, &used, RemoteResourceReasonCode::LeaseSnapshotOvercommitted)?;
    debug_assert!(remaining_within_total(&remaining, &total));
    debug_assert!(active_leases.len() <= MAX_REMOTE_RESOURCE_LEASES);
    Ok(availability_from_totals(&remaining))
}

pub fn plan_remote_resource_availability(
    worker_endpoint_id: &str,
    inventory: &RemoteWorkerResourceInventory,
    requirements: &RemoteResourceRequirements,
    active_leases: &[RemoteResourceLease],
) -> Result<RemoteResourceAvailabilityPlan, RemoteResourceReasonCode> {
    validate_resource_identity(worker_endpoint_id)?;
    let inventory = canonical_remote_worker_resource_inventory(inventory)?;
    let requirements = canonical_remote_resource_requirements(requirements)?;
    let total = totals_from_vector(&inventory.total);
    let used = aggregate_worker_leases(worker_endpoint_id, active_leases)?;
    let remaining_before = subtract_totals(&total, &used, RemoteResourceReasonCode::LeaseSnapshotOvercommitted)?;
    let requested = totals_from_vector(&requirements.quantities);
    let remaining_after = subtract_requested(&remaining_before, &requested)?;
    let resource_fit = resource_fit_class(&remaining_after, &requested);
    debug_assert!(remaining_within_total(&remaining_after, &total));
    debug_assert!(resource_fit != ResourceFitClass::Unknown);
    Ok(RemoteResourceAvailabilityPlan {
        remaining_before: availability_from_totals(&remaining_before),
        remaining_after: availability_from_totals(&remaining_after),
        resource_fit,
    })
}

pub fn plan_remote_resource_reservation(
    scope: RemoteResourceLeaseScope,
    inventory: &RemoteWorkerResourceInventory,
    requirements: &RemoteResourceRequirements,
    active_leases: &[RemoteResourceLease],
) -> Result<RemoteResourceAdmissionPlan, RemoteResourceReasonCode> {
    validate_lease_scope(&scope)?;
    let requirements = canonical_remote_resource_requirements(requirements)?;
    let availability =
        plan_remote_resource_availability(&scope.worker_endpoint_id, inventory, &requirements, active_leases)?;
    let requirement_digest_blake3 = remote_resource_requirement_digest(&requirements)?;
    let lease = build_resource_lease(scope, requirement_digest_blake3, requirements.quantities)?;
    debug_assert_eq!(lease.schema, REMOTE_RESOURCE_LEASE_SCHEMA);
    debug_assert!(is_blake3_digest(&lease.lease_id_blake3));
    Ok(RemoteResourceAdmissionPlan {
        lease,
        remaining_before: availability.remaining_before,
        remaining_after: availability.remaining_after,
        resource_fit: availability.resource_fit,
        reason_code: RemoteResourceReasonCode::ReservationPlanned,
    })
}

pub fn authorize_remote_resource_lease_mutation(
    lease: &RemoteResourceLease,
    scope: &RemoteResourceLeaseScope,
    _kind: RemoteResourceLeaseMutationKind,
) -> Result<(), RemoteResourceReasonCode> {
    validate_resource_lease(lease)?;
    validate_lease_scope(scope)?;
    if lease.scope.worker_endpoint_id != scope.worker_endpoint_id {
        return Err(RemoteResourceReasonCode::LeaseWorkerMismatch);
    }
    if lease.scope.job_id != scope.job_id {
        return Err(RemoteResourceReasonCode::LeaseJobMismatch);
    }
    if scope.fence_generation < lease.scope.fence_generation {
        return Err(RemoteResourceReasonCode::LeaseStaleFence);
    }
    if scope.fence_generation > lease.scope.fence_generation {
        return Err(RemoteResourceReasonCode::LeaseUnknownFence);
    }
    if lease.scope.attempt_id != scope.attempt_id {
        return Err(RemoteResourceReasonCode::LeaseAttemptMismatch);
    }
    debug_assert_eq!(lease.scope.fence_generation, scope.fence_generation);
    debug_assert_eq!(lease.scope.attempt_id, scope.attempt_id);
    Ok(())
}

pub fn plan_remote_resource_lease_release(
    lease: &RemoteResourceLease,
    scope: &RemoteResourceLeaseScope,
) -> Result<RemoteResourceLeaseReleasePlan, RemoteResourceReasonCode> {
    authorize_remote_resource_lease_mutation(lease, scope, RemoteResourceLeaseMutationKind::Release)?;
    debug_assert!(!lease.reserved.accelerators.iter().any(|entry| entry.quantity == 0));
    debug_assert!(lease.reserved.cpu_units > 0);
    Ok(RemoteResourceLeaseReleasePlan {
        lease_id_blake3: lease.lease_id_blake3.clone(),
        released: lease.reserved.clone(),
        reason_code: RemoteResourceReasonCode::LeaseReleased,
    })
}

pub fn plan_remote_resource_lease_resize(
    lease: &RemoteResourceLease,
    scope: &RemoteResourceLeaseScope,
    inventory: &RemoteWorkerResourceInventory,
    requirements: &RemoteResourceRequirements,
    active_leases: &[RemoteResourceLease],
) -> Result<RemoteResourceAdmissionPlan, RemoteResourceReasonCode> {
    authorize_remote_resource_lease_mutation(lease, scope, RemoteResourceLeaseMutationKind::Resize)?;
    let others = active_leases
        .iter()
        .filter(|candidate| candidate.lease_id_blake3 != lease.lease_id_blake3)
        .cloned()
        .collect::<Vec<_>>();
    let mut plan = plan_remote_resource_reservation(scope.clone(), inventory, requirements, &others)?;
    plan.reason_code = RemoteResourceReasonCode::LeaseResized;
    debug_assert!(!plan.lease.requirement_digest_blake3.is_empty());
    debug_assert_eq!(others.len().saturating_add(1), active_leases.len());
    Ok(plan)
}

pub fn plan_remote_resource_lease_recovery(
    lease: &RemoteResourceLease,
    current_scope: Option<&RemoteResourceLeaseScope>,
    current_phase: Option<RemoteAttemptPhase>,
) -> Result<RemoteResourceRecoveryPlan, RemoteResourceReasonCode> {
    validate_resource_lease(lease)?;
    let Some(current_scope) = current_scope else {
        return Ok(recovery_release());
    };
    if authorize_remote_resource_lease_mutation(lease, current_scope, RemoteResourceLeaseMutationKind::Renew).is_err() {
        return Ok(recovery_release());
    }
    if current_phase.is_some_and(RemoteAttemptPhase::is_live) {
        return Ok(RemoteResourceRecoveryPlan {
            disposition: RemoteResourceRecoveryDisposition::Preserve,
            reason_code: RemoteResourceReasonCode::LeasePreservedOnRecovery,
        });
    }
    debug_assert!(current_phase.is_none_or(RemoteAttemptPhase::is_terminal));
    debug_assert_eq!(lease.scope, *current_scope);
    Ok(recovery_release())
}

pub fn validate_remote_resource_lease_snapshot(
    inventories: &BTreeMap<String, RemoteWorkerResourceInventory>,
    leases: &[RemoteResourceLease],
) -> Result<(), RemoteResourceReasonCode> {
    if leases.len() > MAX_REMOTE_RESOURCE_LEASES {
        return Err(RemoteResourceReasonCode::LeaseCountExceeded);
    }
    for (worker_endpoint_id, inventory) in inventories {
        let inventory = canonical_remote_worker_resource_inventory(inventory)?;
        let used = aggregate_worker_leases(worker_endpoint_id, leases)?;
        let total = totals_from_vector(&inventory.total);
        subtract_totals(&total, &used, RemoteResourceReasonCode::LeaseSnapshotOvercommitted)?;
    }
    if leases.iter().any(|lease| !inventories.contains_key(&lease.scope.worker_endpoint_id)) {
        return Err(RemoteResourceReasonCode::LeaseWorkerMismatch);
    }
    debug_assert!(leases.len() <= MAX_REMOTE_RESOURCE_LEASES);
    debug_assert!(leases.iter().all(|lease| inventories.contains_key(&lease.scope.worker_endpoint_id)));
    Ok(())
}

pub fn normalize_remote_verified_locality(
    expected_worker_endpoint_id: &str,
    expected_worker_generation: u64,
    manifest: &CanonicalRemoteTransferManifest,
    observation: &RemoteLocalityProbeObservation,
) -> Result<RemoteVerifiedLocalitySummary, RemoteLocalityReasonCode> {
    validate_locality_binding(expected_worker_endpoint_id, expected_worker_generation, manifest, observation)?;
    if !observation.receiver_probe_verified {
        return conservative_unverified_locality(manifest, observation);
    }
    let demand = plan_remote_transfer_demand(manifest, &observation.receiver_facts)
        .map_err(|_| RemoteLocalityReasonCode::TransferFactsInvalid)?;
    let missing_object_count =
        u32::try_from(demand.missing_chunks.len()).map_err(|_| RemoteLocalityReasonCode::ArithmeticOverflow)?;
    let verified_present_object_count = manifest
        .chunk_count
        .checked_sub(missing_object_count)
        .ok_or(RemoteLocalityReasonCode::ArithmeticOverflow)?;
    let quantities = LocalityQuantities {
        present_object_count: verified_present_object_count,
        missing_object_count,
        present_bytes: demand.reused_bytes,
        missing_bytes: demand.missing_bytes,
    };
    let classification = locality_classes(quantities);
    debug_assert_eq!(verified_present_object_count.checked_add(missing_object_count), Some(manifest.chunk_count));
    debug_assert!(demand.reused_bytes <= manifest.total_bytes);
    Ok(locality_summary(manifest, observation, quantities, classification))
}

pub fn rank_remote_worker_placement_candidates(
    policy: &SchedulingPolicy,
    candidates: &[RemoteWorkerPlacementFacts],
) -> Result<Vec<RemoteWorkerPlacementFacts>, RemoteResourceReasonCode> {
    policy.validate().map_err(|_| RemoteResourceReasonCode::PlacementCandidatesInvalid)?;
    if candidates.is_empty() || candidates.len() > MAX_REMOTE_PLACEMENT_CANDIDATES {
        return Err(RemoteResourceReasonCode::PlacementCandidatesInvalid);
    }
    let mut identities = BTreeSet::new();
    if candidates.iter().any(|candidate| {
        validate_resource_identity(&candidate.worker_endpoint_id).is_err()
            || !identities.insert(candidate.worker_endpoint_id.as_str())
    }) {
        return Err(RemoteResourceReasonCode::PlacementCandidatesInvalid);
    }
    let mut ranked = candidates.to_vec();
    ranked.sort_by(|left, right| compare_worker_placement(policy, left, right));
    debug_assert_eq!(ranked.len(), candidates.len());
    debug_assert!(
        ranked
            .windows(2)
            .all(|pair| compare_worker_placement(policy, &pair[0], &pair[1]) != Ordering::Greater)
    );
    Ok(ranked)
}

fn canonical_positive_resource_vector(
    vector: &RemoteResourceVector,
) -> Result<RemoteResourceVector, RemoteResourceReasonCode> {
    validate_positive_u32(PositiveU32Bound {
        value: vector.cpu_units,
        maximum: MAX_REMOTE_CPU_UNITS,
    })?;
    validate_positive_u64(PositiveU64Bound {
        value: vector.memory_bytes,
        maximum: MAX_REMOTE_RESOURCE_BYTES,
    })?;
    validate_positive_u64(PositiveU64Bound {
        value: vector.scratch_bytes,
        maximum: MAX_REMOTE_RESOURCE_BYTES,
    })?;
    let accelerators = canonical_named_resources(&vector.accelerators)?;
    let named_tokens = canonical_named_resources(&vector.named_tokens)?;
    debug_assert!(accelerators.len() <= MAX_REMOTE_RESOURCE_CLASSES);
    debug_assert!(named_tokens.len() <= MAX_REMOTE_RESOURCE_CLASSES);
    Ok(RemoteResourceVector {
        cpu_units: vector.cpu_units,
        memory_bytes: vector.memory_bytes,
        scratch_bytes: vector.scratch_bytes,
        accelerators,
        named_tokens,
    })
}

fn canonical_named_resources(
    entries: &[RemoteNamedResourceQuantity],
) -> Result<Vec<RemoteNamedResourceQuantity>, RemoteResourceReasonCode> {
    if entries.len() > MAX_REMOTE_RESOURCE_CLASSES {
        return Err(RemoteResourceReasonCode::ResourceClassCountExceeded);
    }
    let mut canonical = entries.to_vec();
    canonical.sort_by(|left, right| left.name.cmp(&right.name));
    let mut previous: Option<&str> = None;
    for entry in &canonical {
        validate_resource_class_name(&entry.name)?;
        validate_positive_u32(PositiveU32Bound {
            value: entry.quantity,
            maximum: MAX_REMOTE_RESOURCE_COUNT,
        })?;
        if previous == Some(entry.name.as_str()) {
            return Err(RemoteResourceReasonCode::ResourceClassDuplicate);
        }
        previous = Some(entry.name.as_str());
    }
    debug_assert_eq!(canonical.len(), entries.len());
    debug_assert!(canonical.windows(2).all(|pair| pair[0].name < pair[1].name));
    Ok(canonical)
}

fn canonical_resource_class_names(names: &[String]) -> Result<Vec<String>, RemoteResourceReasonCode> {
    if names.len() > MAX_REMOTE_RESOURCE_CLASSES {
        return Err(RemoteResourceReasonCode::ResourceClassCountExceeded);
    }
    let mut canonical = names.to_vec();
    canonical.sort();
    for name in &canonical {
        validate_resource_class_name(name)?;
    }
    if canonical.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(RemoteResourceReasonCode::ResourceClassDuplicate);
    }
    debug_assert_eq!(canonical.len(), names.len());
    debug_assert!(canonical.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(canonical)
}

fn validate_resource_class_name(name: &str) -> Result<(), RemoteResourceReasonCode> {
    let is_name_length_within_limit_bytes = !name.is_empty() && name.len() <= MAX_REMOTE_RESOURCE_CLASS_NAME_BYTES;
    let is_name_character_set_valid =
        name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
    if !is_name_length_within_limit_bytes || !is_name_character_set_valid {
        return Err(RemoteResourceReasonCode::ResourceClassNameInvalid);
    }
    debug_assert!(!name.is_empty());
    debug_assert!(!name.chars().any(char::is_control));
    Ok(())
}

fn validate_resource_identity(value: &str) -> Result<(), RemoteResourceReasonCode> {
    if value.is_empty() || value.len() > MAX_REMOTE_RESOURCE_CLASS_NAME_BYTES || value.chars().any(char::is_control) {
        return Err(RemoteResourceReasonCode::ResourceClassNameInvalid);
    }
    debug_assert!(!value.is_empty());
    debug_assert!(value.len() <= MAX_REMOTE_RESOURCE_CLASS_NAME_BYTES);
    Ok(())
}

fn validate_positive_u32(bound: PositiveU32Bound) -> Result<(), RemoteResourceReasonCode> {
    if bound.value == 0 {
        return Err(RemoteResourceReasonCode::QuantityZero);
    }
    if bound.value > bound.maximum {
        return Err(RemoteResourceReasonCode::QuantityExceedsLimit);
    }
    debug_assert!(bound.value > 0);
    debug_assert!(bound.value <= bound.maximum);
    Ok(())
}

fn validate_positive_u64(bound: PositiveU64Bound) -> Result<(), RemoteResourceReasonCode> {
    if bound.value == 0 {
        return Err(RemoteResourceReasonCode::QuantityZero);
    }
    if bound.value > bound.maximum {
        return Err(RemoteResourceReasonCode::QuantityExceedsLimit);
    }
    debug_assert!(bound.value > 0);
    debug_assert!(bound.value <= bound.maximum);
    Ok(())
}

fn totals_from_vector(vector: &RemoteResourceVector) -> ResourceTotals {
    ResourceTotals {
        cpu_units: vector.cpu_units,
        memory_bytes: vector.memory_bytes,
        scratch_bytes: vector.scratch_bytes,
        accelerators: vector.accelerators.iter().map(|entry| (entry.name.clone(), entry.quantity)).collect(),
        named_tokens: vector.named_tokens.iter().map(|entry| (entry.name.clone(), entry.quantity)).collect(),
    }
}

fn aggregate_worker_leases(
    worker_endpoint_id: &str,
    active_leases: &[RemoteResourceLease],
) -> Result<ResourceTotals, RemoteResourceReasonCode> {
    if active_leases.len() > MAX_REMOTE_RESOURCE_LEASES {
        return Err(RemoteResourceReasonCode::LeaseCountExceeded);
    }
    let mut used = ResourceTotals::default();
    let mut lease_ids = BTreeSet::new();
    for lease in active_leases.iter().filter(|lease| lease.scope.worker_endpoint_id == worker_endpoint_id) {
        validate_resource_lease(lease)?;
        if !lease_ids.insert(lease.lease_id_blake3.as_str()) {
            return Err(RemoteResourceReasonCode::LeaseDigestInvalid);
        }
        add_vector_to_totals(&mut used, &lease.reserved)?;
    }
    debug_assert!(lease_ids.len() <= active_leases.len());
    debug_assert!(active_leases.len() <= MAX_REMOTE_RESOURCE_LEASES);
    Ok(used)
}

fn add_vector_to_totals(
    totals: &mut ResourceTotals,
    vector: &RemoteResourceVector,
) -> Result<(), RemoteResourceReasonCode> {
    let vector = canonical_positive_resource_vector(vector)?;
    totals.cpu_units =
        totals.cpu_units.checked_add(vector.cpu_units).ok_or(RemoteResourceReasonCode::ArithmeticOverflow)?;
    totals.memory_bytes = totals
        .memory_bytes
        .checked_add(vector.memory_bytes)
        .ok_or(RemoteResourceReasonCode::ArithmeticOverflow)?;
    totals.scratch_bytes = totals
        .scratch_bytes
        .checked_add(vector.scratch_bytes)
        .ok_or(RemoteResourceReasonCode::ArithmeticOverflow)?;
    add_named_totals(&mut totals.accelerators, &vector.accelerators)?;
    add_named_totals(&mut totals.named_tokens, &vector.named_tokens)?;
    debug_assert!(totals.cpu_units >= vector.cpu_units);
    debug_assert!(totals.memory_bytes >= vector.memory_bytes);
    Ok(())
}

fn add_named_totals(
    totals: &mut BTreeMap<String, u32>,
    entries: &[RemoteNamedResourceQuantity],
) -> Result<(), RemoteResourceReasonCode> {
    for entry in entries {
        let current = totals.get(&entry.name).copied().unwrap_or(0);
        let next = current.checked_add(entry.quantity).ok_or(RemoteResourceReasonCode::ArithmeticOverflow)?;
        totals.insert(entry.name.clone(), next);
    }
    debug_assert!(totals.len() <= MAX_REMOTE_RESOURCE_CLASSES);
    debug_assert!(entries.len() <= MAX_REMOTE_RESOURCE_CLASSES);
    Ok(())
}

fn subtract_requested(
    available: &ResourceTotals,
    requested: &ResourceTotals,
) -> Result<ResourceTotals, RemoteResourceReasonCode> {
    let cpu_units = available
        .cpu_units
        .checked_sub(requested.cpu_units)
        .ok_or(RemoteResourceReasonCode::CpuUnavailable)?;
    let memory_bytes = available
        .memory_bytes
        .checked_sub(requested.memory_bytes)
        .ok_or(RemoteResourceReasonCode::MemoryUnavailable)?;
    let scratch_bytes = available
        .scratch_bytes
        .checked_sub(requested.scratch_bytes)
        .ok_or(RemoteResourceReasonCode::ScratchUnavailable)?;
    let accelerators = subtract_named(
        &available.accelerators,
        &requested.accelerators,
        RemoteResourceReasonCode::AcceleratorUnavailable,
    )?;
    let named_tokens = subtract_named(
        &available.named_tokens,
        &requested.named_tokens,
        RemoteResourceReasonCode::NamedTokenUnavailable,
    )?;
    debug_assert!(cpu_units <= available.cpu_units);
    debug_assert!(memory_bytes <= available.memory_bytes);
    Ok(ResourceTotals {
        cpu_units,
        memory_bytes,
        scratch_bytes,
        accelerators,
        named_tokens,
    })
}

fn subtract_totals(
    available: &ResourceTotals,
    used: &ResourceTotals,
    reason: RemoteResourceReasonCode,
) -> Result<ResourceTotals, RemoteResourceReasonCode> {
    let cpu_units = available.cpu_units.checked_sub(used.cpu_units).ok_or(reason)?;
    let memory_bytes = available.memory_bytes.checked_sub(used.memory_bytes).ok_or(reason)?;
    let scratch_bytes = available.scratch_bytes.checked_sub(used.scratch_bytes).ok_or(reason)?;
    let accelerators = subtract_named(&available.accelerators, &used.accelerators, reason)?;
    let named_tokens = subtract_named(&available.named_tokens, &used.named_tokens, reason)?;
    debug_assert!(cpu_units <= available.cpu_units);
    debug_assert!(scratch_bytes <= available.scratch_bytes);
    Ok(ResourceTotals {
        cpu_units,
        memory_bytes,
        scratch_bytes,
        accelerators,
        named_tokens,
    })
}

fn subtract_named(
    available: &BTreeMap<String, u32>,
    used: &BTreeMap<String, u32>,
    reason: RemoteResourceReasonCode,
) -> Result<BTreeMap<String, u32>, RemoteResourceReasonCode> {
    let mut remaining = available.clone();
    for (name, quantity) in used {
        let current = remaining.get(name).copied().unwrap_or(0);
        let next = current.checked_sub(*quantity).ok_or(reason)?;
        if next == 0 {
            remaining.remove(name);
        } else {
            remaining.insert(name.clone(), next);
        }
    }
    debug_assert!(remaining.len() <= available.len());
    debug_assert!(remaining.values().all(|quantity| *quantity > 0));
    Ok(remaining)
}

fn resource_fit_class(remaining: &ResourceTotals, requested: &ResourceTotals) -> ResourceFitClass {
    if totals_are_zero(remaining) {
        return ResourceFitClass::Exact;
    }
    let is_resource_constrained = remaining.cpu_units < requested.cpu_units
        || remaining.memory_bytes < requested.memory_bytes
        || remaining.scratch_bytes < requested.scratch_bytes
        || named_remaining_is_constrained(&remaining.accelerators, &requested.accelerators)
        || named_remaining_is_constrained(&remaining.named_tokens, &requested.named_tokens);
    let class = if is_resource_constrained {
        ResourceFitClass::Constrained
    } else {
        ResourceFitClass::Compatible
    };
    debug_assert_ne!(class, ResourceFitClass::Unknown);
    debug_assert!(!totals_are_zero(remaining));
    class
}

fn named_remaining_is_constrained(remaining: &BTreeMap<String, u32>, requested: &BTreeMap<String, u32>) -> bool {
    requested.iter().any(|(name, quantity)| remaining.get(name).copied().unwrap_or(0) < *quantity)
}

fn totals_are_zero(totals: &ResourceTotals) -> bool {
    totals.cpu_units == 0
        && totals.memory_bytes == 0
        && totals.scratch_bytes == 0
        && totals.accelerators.is_empty()
        && totals.named_tokens.is_empty()
}

fn remaining_within_total(remaining: &ResourceTotals, total: &ResourceTotals) -> bool {
    remaining.cpu_units <= total.cpu_units
        && remaining.memory_bytes <= total.memory_bytes
        && remaining.scratch_bytes <= total.scratch_bytes
        && remaining
            .accelerators
            .iter()
            .all(|(name, quantity)| total.accelerators.get(name).is_some_and(|total| quantity <= total))
        && remaining
            .named_tokens
            .iter()
            .all(|(name, quantity)| total.named_tokens.get(name).is_some_and(|total| quantity <= total))
}

fn availability_from_totals(totals: &ResourceTotals) -> RemoteResourceAvailability {
    RemoteResourceAvailability {
        cpu_units: totals.cpu_units,
        memory_bytes: totals.memory_bytes,
        scratch_bytes: totals.scratch_bytes,
        accelerators: named_quantities_from_map(&totals.accelerators),
        named_tokens: named_quantities_from_map(&totals.named_tokens),
    }
}

fn named_quantities_from_map(values: &BTreeMap<String, u32>) -> Vec<RemoteNamedResourceQuantity> {
    values
        .iter()
        .filter(|(_, quantity)| **quantity > 0)
        .map(|(name, quantity)| RemoteNamedResourceQuantity {
            name: name.clone(),
            quantity: *quantity,
        })
        .collect()
}

fn validate_lease_scope(scope: &RemoteResourceLeaseScope) -> Result<(), RemoteResourceReasonCode> {
    validate_resource_identity(&scope.worker_endpoint_id)?;
    if scope.worker_generation == 0 {
        return Err(RemoteResourceReasonCode::LeaseDigestInvalid);
    }
    if scope.job_id.as_str().is_empty() {
        return Err(RemoteResourceReasonCode::LeaseDigestInvalid);
    }
    if scope.attempt_id.as_str().is_empty() {
        return Err(RemoteResourceReasonCode::LeaseDigestInvalid);
    }
    if scope.fence_generation.get() == 0 {
        return Err(RemoteResourceReasonCode::LeaseDigestInvalid);
    }
    debug_assert!(!scope.worker_endpoint_id.is_empty());
    debug_assert!(scope.worker_generation > 0);
    debug_assert!(scope.fence_generation.get() > 0);
    Ok(())
}

fn build_resource_lease(
    scope: RemoteResourceLeaseScope,
    requirement_digest_blake3: String,
    reserved: RemoteResourceVector,
) -> Result<RemoteResourceLease, RemoteResourceReasonCode> {
    let reservation_digest_blake3 = reservation_digest(&scope, &requirement_digest_blake3, &reserved)?;
    let lease_id_blake3 = lease_identity_digest(&scope, &reservation_digest_blake3)?;
    let lease = RemoteResourceLease {
        schema: REMOTE_RESOURCE_LEASE_SCHEMA.to_string(),
        lease_id_blake3,
        scope,
        requirement_digest_blake3,
        reservation_digest_blake3,
        reserved,
        claim_scope: REMOTE_RESOURCE_CLAIM_SCOPE.to_string(),
        non_claims: REMOTE_RESOURCE_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    };
    validate_resource_lease(&lease)?;
    debug_assert_eq!(lease.non_claims.len(), REMOTE_RESOURCE_NON_CLAIMS.len());
    debug_assert!(is_blake3_digest(&lease.lease_id_blake3));
    Ok(lease)
}

fn validate_resource_lease(lease: &RemoteResourceLease) -> Result<(), RemoteResourceReasonCode> {
    if lease.schema != REMOTE_RESOURCE_LEASE_SCHEMA {
        return Err(RemoteResourceReasonCode::LeaseSchemaUnsupported);
    }
    validate_lease_scope(&lease.scope)?;
    let reserved = canonical_positive_resource_vector(&lease.reserved)?;
    if reserved != lease.reserved || !is_blake3_digest(&lease.requirement_digest_blake3) {
        return Err(RemoteResourceReasonCode::LeaseDigestInvalid);
    }
    let expected_reservation = reservation_digest(&lease.scope, &lease.requirement_digest_blake3, &reserved)?;
    let expected_lease_id = lease_identity_digest(&lease.scope, &expected_reservation)?;
    if lease.reservation_digest_blake3 != expected_reservation || lease.lease_id_blake3 != expected_lease_id {
        return Err(RemoteResourceReasonCode::LeaseDigestInvalid);
    }
    if lease.claim_scope != REMOTE_RESOURCE_CLAIM_SCOPE
        || lease.non_claims != REMOTE_RESOURCE_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>()
    {
        return Err(RemoteResourceReasonCode::LeaseDigestInvalid);
    }
    debug_assert!(is_blake3_digest(&lease.lease_id_blake3));
    debug_assert_eq!(lease.schema, REMOTE_RESOURCE_LEASE_SCHEMA);
    Ok(())
}

fn reservation_digest(
    scope: &RemoteResourceLeaseScope,
    requirement_digest_blake3: &str,
    reserved: &RemoteResourceVector,
) -> Result<String, RemoteResourceReasonCode> {
    #[derive(Serialize)]
    struct ReservationDigestInput<'a> {
        scope: &'a RemoteResourceLeaseScope,
        requirement_digest_blake3: &'a str,
        reserved: &'a RemoteResourceVector,
    }
    if !is_blake3_digest(requirement_digest_blake3) {
        return Err(RemoteResourceReasonCode::LeaseDigestInvalid);
    }
    domain_hash_json(REMOTE_RESOURCE_RESERVATION_DOMAIN, &ReservationDigestInput {
        scope,
        requirement_digest_blake3,
        reserved,
    })
}

fn lease_identity_digest(
    scope: &RemoteResourceLeaseScope,
    reservation_digest_blake3: &str,
) -> Result<String, RemoteResourceReasonCode> {
    #[derive(Serialize)]
    struct LeaseIdentityInput<'a> {
        scope: &'a RemoteResourceLeaseScope,
        reservation_digest_blake3: &'a str,
    }
    if !is_blake3_digest(reservation_digest_blake3) {
        return Err(RemoteResourceReasonCode::LeaseDigestInvalid);
    }
    domain_hash_json(REMOTE_RESOURCE_LEASE_DOMAIN, &LeaseIdentityInput {
        scope,
        reservation_digest_blake3,
    })
}

fn domain_hash_json<T: Serialize>(domain: &str, value: &T) -> Result<String, RemoteResourceReasonCode> {
    let bytes = serde_json::to_vec(value).map_err(|_| RemoteResourceReasonCode::LeaseDigestInvalid)?;
    let mut hasher = blake3::Hasher::new();
    hash_part(&mut hasher, domain)?;
    hasher.update(&bytes);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert!(!bytes.is_empty());
    debug_assert!(is_blake3_digest(&digest));
    Ok(digest)
}

fn hash_part(hasher: &mut blake3::Hasher, value: &str) -> Result<(), RemoteResourceReasonCode> {
    let length_bytes = u64::try_from(value.len()).map_err(|_| RemoteResourceReasonCode::ArithmeticOverflow)?;
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(value.as_bytes());
    Ok(())
}

fn is_blake3_digest(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn recovery_release() -> RemoteResourceRecoveryPlan {
    RemoteResourceRecoveryPlan {
        disposition: RemoteResourceRecoveryDisposition::Release,
        reason_code: RemoteResourceReasonCode::LeaseReleasedOnRecovery,
    }
}

fn validate_locality_binding(
    expected_worker_endpoint_id: &str,
    expected_worker_generation: u64,
    manifest: &CanonicalRemoteTransferManifest,
    observation: &RemoteLocalityProbeObservation,
) -> Result<(), RemoteLocalityReasonCode> {
    if observation.worker_endpoint_id != expected_worker_endpoint_id {
        return Err(RemoteLocalityReasonCode::WorkerIdentityMismatch);
    }
    if expected_worker_generation == 0 || observation.worker_generation != expected_worker_generation {
        return Err(RemoteLocalityReasonCode::WorkerGenerationStale);
    }
    if observation.scope.manifest_digest_blake3 != manifest.digest_blake3.as_str() {
        return Err(RemoteLocalityReasonCode::ManifestMismatch);
    }
    if observation.scope.policy_digest_blake3 != manifest.manifest.policy_digest_blake3.as_str() {
        return Err(RemoteLocalityReasonCode::PolicyMismatch);
    }
    debug_assert_eq!(observation.worker_endpoint_id, expected_worker_endpoint_id);
    debug_assert_eq!(observation.worker_generation, expected_worker_generation);
    Ok(())
}

fn conservative_unverified_locality(
    manifest: &CanonicalRemoteTransferManifest,
    observation: &RemoteLocalityProbeObservation,
) -> Result<RemoteVerifiedLocalitySummary, RemoteLocalityReasonCode> {
    let missing_object_count = manifest.chunk_count;
    let missing_bytes = manifest.total_bytes;
    debug_assert!(!observation.receiver_probe_verified);
    debug_assert!(missing_bytes > 0);
    let quantities = LocalityQuantities {
        present_object_count: 0,
        missing_object_count,
        present_bytes: 0,
        missing_bytes,
    };
    let classification = LocalityClassification {
        content_locality: ContentLocalityClass::NoVerifiedContent,
        transfer_cost: TransferCostClass::Large,
        reason_code: RemoteLocalityReasonCode::UnverifiedHintDowngraded,
    };
    Ok(locality_summary(manifest, observation, quantities, classification))
}

fn locality_classes(quantities: LocalityQuantities) -> LocalityClassification {
    if quantities.missing_object_count == 0 && quantities.missing_bytes == 0 {
        return LocalityClassification {
            content_locality: ContentLocalityClass::FullyPresent,
            transfer_cost: TransferCostClass::None,
            reason_code: RemoteLocalityReasonCode::VerifiedFullyPresent,
        };
    }
    if quantities.present_object_count == 0 && quantities.present_bytes == 0 {
        return LocalityClassification {
            content_locality: ContentLocalityClass::NoVerifiedContent,
            transfer_cost: TransferCostClass::Large,
            reason_code: RemoteLocalityReasonCode::VerifiedNoContent,
        };
    }
    let transfer_cost = match quantities.missing_bytes.cmp(&quantities.present_bytes) {
        Ordering::Less => TransferCostClass::Small,
        Ordering::Equal => TransferCostClass::Medium,
        Ordering::Greater => TransferCostClass::Large,
    };
    debug_assert!(quantities.missing_object_count > 0);
    debug_assert!(quantities.present_object_count > 0 || quantities.present_bytes > 0);
    LocalityClassification {
        content_locality: ContentLocalityClass::PartiallyPresent,
        transfer_cost,
        reason_code: RemoteLocalityReasonCode::VerifiedPartiallyPresent,
    }
}

fn locality_summary(
    manifest: &CanonicalRemoteTransferManifest,
    observation: &RemoteLocalityProbeObservation,
    quantities: LocalityQuantities,
    classification: LocalityClassification,
) -> RemoteVerifiedLocalitySummary {
    let summary = RemoteVerifiedLocalitySummary {
        schema: REMOTE_LOCALITY_SUMMARY_SCHEMA.to_string(),
        worker_endpoint_id: observation.worker_endpoint_id.clone(),
        worker_generation: observation.worker_generation,
        scope: observation.scope.clone(),
        demanded_object_count: manifest.chunk_count,
        verified_present_object_count: quantities.present_object_count,
        missing_object_count: quantities.missing_object_count,
        demanded_bytes: manifest.total_bytes,
        verified_present_bytes: quantities.present_bytes,
        missing_bytes: quantities.missing_bytes,
        content_locality: classification.content_locality,
        transfer_cost: classification.transfer_cost,
        reason_code: classification.reason_code,
        claim_scope: REMOTE_LOCALITY_CLAIM_SCOPE.to_string(),
        non_claims: REMOTE_LOCALITY_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    };
    debug_assert_eq!(summary.non_claims.len(), REMOTE_LOCALITY_NON_CLAIMS.len());
    debug_assert_eq!(summary.schema, REMOTE_LOCALITY_SUMMARY_SCHEMA);
    summary
}

fn compare_worker_placement(
    policy: &SchedulingPolicy,
    left: &RemoteWorkerPlacementFacts,
    right: &RemoteWorkerPlacementFacts,
) -> Ordering {
    for field in &policy.preference_order {
        let ordering = match field {
            PreferenceField::KnownGraph => Ordering::Equal,
            PreferenceField::ResourceFit => right.resource_fit.cmp(&left.resource_fit),
            PreferenceField::LocalityTransfer => right
                .content_locality
                .cmp(&left.content_locality)
                .then_with(|| right.transfer_cost.cmp(&left.transfer_cost)),
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    left.worker_endpoint_id.cmp(&right.worker_endpoint_id)
}

#[cfg(test)]
mod tests {
    // r[verify build_scheduling.quantified_resource_admission]
    // r[verify build_scheduling.verified_locality_placement]
    // r[verify remote_builds.fenced_worker_resource_leases]
    use proptest::prelude::*;

    use super::*;
    use crate::distributed::RemoteTransferArtifact;
    use crate::distributed::RemoteTransferArtifactId;
    use crate::distributed::RemoteTransferArtifactKind;
    use crate::distributed::RemoteTransferChunkDescriptor;
    use crate::distributed::RemoteTransferDigest;
    use crate::distributed::RemoteTransferManifest;
    use crate::distributed::RemoteTransferPolicy;
    use crate::distributed::RemoteTransferSessionId;
    use crate::distributed::canonical_remote_transfer_policy_digest;
    use crate::distributed::canonicalize_remote_transfer_manifest;
    use crate::scheduling::HardEligibilityFacts;
    use crate::scheduling::IneligibleReason;
    use crate::scheduling::KnownGraphPressure;
    use crate::scheduling::ReadyGoalFacts;
    use crate::scheduling::normalize_eligible_preference;
    use crate::scheduling::rank_ready_goals;

    const TEST_CPU_TOTAL: u32 = 8;
    const TEST_CPU_REQUEST: u32 = 2;
    const TEST_MEMORY_TOTAL: u64 = 16_384;
    const TEST_MEMORY_REQUEST: u64 = 4_096;
    const TEST_SCRATCH_TOTAL: u64 = 32_768;
    const TEST_SCRATCH_REQUEST: u64 = 8_192;
    const TEST_ACCELERATOR_TOTAL: u32 = 2;
    const TEST_ACCELERATOR_REQUEST: u32 = 1;
    const TEST_TOKEN_TOTAL: u32 = 4;
    const TEST_TOKEN_REQUEST: u32 = 1;
    const TEST_WORKER_GENERATION: u64 = 7;
    const TEST_ARTIFACT_BYTES: u64 = 8;
    const TEST_CHUNK_BYTES: u32 = 4;
    const TEST_SECOND_CHUNK_OFFSET: u64 = 4;
    const TEST_SECOND_CHUNK_INDEX: u32 = 1;
    const TEST_SCHEDULING_EPOCH: u32 = 8;

    fn named(name: &str, quantity: u32) -> RemoteNamedResourceQuantity {
        RemoteNamedResourceQuantity {
            name: name.to_string(),
            quantity,
        }
    }

    fn inventory() -> RemoteWorkerResourceInventory {
        RemoteWorkerResourceInventory {
            total: RemoteResourceVector {
                cpu_units: TEST_CPU_TOTAL,
                memory_bytes: TEST_MEMORY_TOTAL,
                scratch_bytes: TEST_SCRATCH_TOTAL,
                accelerators: vec![named("gpu", TEST_ACCELERATOR_TOTAL)],
                named_tokens: vec![named("licensed-tool", TEST_TOKEN_TOTAL)],
            },
        }
    }

    fn requirements() -> RemoteResourceRequirements {
        RemoteResourceRequirements {
            quantities: RemoteResourceVector {
                cpu_units: TEST_CPU_REQUEST,
                memory_bytes: TEST_MEMORY_REQUEST,
                scratch_bytes: TEST_SCRATCH_REQUEST,
                accelerators: vec![named("gpu", TEST_ACCELERATOR_REQUEST)],
                named_tokens: vec![named("licensed-tool", TEST_TOKEN_REQUEST)],
            },
            semantic_accelerator_classes: vec!["gpu".to_string()],
        }
    }

    fn scope(label: &str, fence: u64) -> RemoteResourceLeaseScope {
        RemoteResourceLeaseScope {
            worker_endpoint_id: "worker-a".to_string(),
            worker_generation: TEST_WORKER_GENERATION,
            job_id: RemoteJobId::new(format!("job-{label}")).unwrap(),
            attempt_id: RemoteAttemptId::new(format!("attempt-{label}")).unwrap(),
            fence_generation: RemoteFenceGeneration::new(fence).unwrap(),
        }
    }

    fn reservation(label: &str, fence: u64, active: &[RemoteResourceLease]) -> RemoteResourceAdmissionPlan {
        plan_remote_resource_reservation(scope(label, fence), &inventory(), &requirements(), active).unwrap()
    }

    fn transfer_manifest() -> CanonicalRemoteTransferManifest {
        let policy = RemoteTransferPolicy::default();
        let policy_digest_blake3 = canonical_remote_transfer_policy_digest(policy).unwrap();
        let chunks = vec![
            RemoteTransferChunkDescriptor {
                index: 0,
                offset_bytes: 0,
                size_bytes: TEST_CHUNK_BYTES,
                digest_blake3: RemoteTransferDigest::new(blake3::hash(b"chunk-a").to_hex().to_string()).unwrap(),
            },
            RemoteTransferChunkDescriptor {
                index: TEST_SECOND_CHUNK_INDEX,
                offset_bytes: TEST_SECOND_CHUNK_OFFSET,
                size_bytes: TEST_CHUNK_BYTES,
                digest_blake3: RemoteTransferDigest::new(blake3::hash(b"chunk-b").to_hex().to_string()).unwrap(),
            },
        ];
        canonicalize_remote_transfer_manifest(
            RemoteTransferManifest {
                schema: super::super::remote_transfer::REMOTE_TRANSFER_MANIFEST_SCHEMA.to_string(),
                session_id: RemoteTransferSessionId::new(blake3::hash(b"session").to_hex().to_string()).unwrap(),
                job_id: RemoteJobId::new("locality-job").unwrap(),
                attempt_id: RemoteAttemptId::new("locality-attempt").unwrap(),
                fence_generation: RemoteFenceGeneration::INITIAL,
                policy_digest_blake3,
                store_prefix: "/mantle/store".to_string(),
                requested_content_blake3: RemoteTransferDigest::new(
                    blake3::hash(b"requested-content").to_hex().to_string(),
                )
                .unwrap(),
                artifacts: vec![RemoteTransferArtifact {
                    artifact_id: RemoteTransferArtifactId::new("artifact-a").unwrap(),
                    artifact_kind: RemoteTransferArtifactKind::CastoreBlob,
                    digest_blake3: RemoteTransferDigest::new(blake3::hash(b"artifact").to_hex().to_string()).unwrap(),
                    size_bytes: TEST_ARTIFACT_BYTES,
                    required_for_completion: true,
                    nar_sha256_hex: None,
                    chunks,
                }],
            },
            policy,
        )
        .unwrap()
    }

    fn locality_observation(manifest: &CanonicalRemoteTransferManifest) -> RemoteLocalityProbeObservation {
        RemoteLocalityProbeObservation {
            worker_endpoint_id: "worker-a".to_string(),
            worker_generation: TEST_WORKER_GENERATION,
            scope: RemoteLocalityScope {
                manifest_digest_blake3: manifest.digest_blake3.as_str().to_string(),
                policy_digest_blake3: manifest.manifest.policy_digest_blake3.as_str().to_string(),
            },
            receiver_probe_verified: true,
            receiver_facts: RemoteTransferReceiverFacts {
                requested_content_identity_verified: true,
                required_closure_metadata_verified: true,
                path_info_admitted: false,
                ..RemoteTransferReceiverFacts::default()
            },
        }
    }

    #[test]
    fn compatible_vectors_reserve_release_and_replay_deterministically() {
        let first = reservation("one", RemoteFenceGeneration::INITIAL.get(), &[]);
        let replay = reservation("one", RemoteFenceGeneration::INITIAL.get(), &[]);
        let released = plan_remote_resource_lease_release(&first.lease, &first.lease.scope).unwrap();

        assert_eq!(first, replay);
        assert_eq!(released.released, requirements().quantities);
        assert_eq!(first.remaining_after.cpu_units, TEST_CPU_TOTAL - TEST_CPU_REQUEST);
        assert!(first.remaining_after.memory_bytes < TEST_MEMORY_TOTAL);
        assert_eq!(released.reason_code, RemoteResourceReasonCode::LeaseReleased);
    }

    #[test]
    fn aggregate_reservations_prevent_every_overcommit_class() {
        let first = reservation("first", RemoteFenceGeneration::INITIAL.get(), &[]);
        let leases = vec![first.lease.clone()];
        let mut cases = Vec::new();
        let mut cpu = requirements();
        cpu.quantities.cpu_units = TEST_CPU_TOTAL;
        cases.push((cpu, RemoteResourceReasonCode::CpuUnavailable));
        let mut memory = requirements();
        memory.quantities.memory_bytes = TEST_MEMORY_TOTAL;
        cases.push((memory, RemoteResourceReasonCode::MemoryUnavailable));
        let mut scratch = requirements();
        scratch.quantities.scratch_bytes = TEST_SCRATCH_TOTAL;
        cases.push((scratch, RemoteResourceReasonCode::ScratchUnavailable));
        let mut accelerator = requirements();
        accelerator.quantities.accelerators[0].quantity = TEST_ACCELERATOR_TOTAL;
        cases.push((accelerator, RemoteResourceReasonCode::AcceleratorUnavailable));
        let mut token = requirements();
        token.quantities.named_tokens[0].quantity = TEST_TOKEN_TOTAL;
        cases.push((token, RemoteResourceReasonCode::NamedTokenUnavailable));

        for (request, expected) in cases {
            let actual =
                plan_remote_resource_reservation(scope("second", 1), &inventory(), &request, &leases).unwrap_err();
            assert_eq!(actual, expected);
        }
        assert_eq!(leases.len(), 1);
        assert_eq!(leases[0], first.lease);
    }

    #[test]
    fn invalid_resource_shapes_fail_closed() {
        let mut zero = requirements();
        zero.quantities.cpu_units = 0;
        let mut oversized = requirements();
        oversized.quantities.memory_bytes = MAX_REMOTE_RESOURCE_BYTES + 1;
        let mut duplicate = requirements();
        duplicate.quantities.named_tokens.push(named("licensed-tool", 1));
        let mut conflicting = requirements();
        conflicting.semantic_accelerator_classes = vec!["different-gpu".to_string()];
        let mut invalid_name = requirements();
        invalid_name.quantities.named_tokens[0].name = "token/secret".to_string();

        assert_eq!(canonical_remote_resource_requirements(&zero), Err(RemoteResourceReasonCode::QuantityZero));
        assert_eq!(
            canonical_remote_resource_requirements(&oversized),
            Err(RemoteResourceReasonCode::QuantityExceedsLimit)
        );
        assert_eq!(
            canonical_remote_resource_requirements(&duplicate),
            Err(RemoteResourceReasonCode::ResourceClassDuplicate)
        );
        assert_eq!(
            canonical_remote_resource_requirements(&conflicting),
            Err(RemoteResourceReasonCode::SemanticAcceleratorMismatch)
        );
        assert_eq!(
            canonical_remote_resource_requirements(&invalid_name),
            Err(RemoteResourceReasonCode::ResourceClassNameInvalid)
        );
    }

    #[test]
    fn stale_attempt_cannot_renew_release_or_resize_current_lease() {
        let plan = reservation("current", 2, &[]);
        let stale = scope("current", 1);
        let future = scope("current", 3);
        let wrong_attempt = RemoteResourceLeaseScope {
            attempt_id: RemoteAttemptId::new("different-attempt").unwrap(),
            ..plan.lease.scope.clone()
        };

        for kind in [
            RemoteResourceLeaseMutationKind::Renew,
            RemoteResourceLeaseMutationKind::Release,
            RemoteResourceLeaseMutationKind::Resize,
        ] {
            assert_eq!(
                authorize_remote_resource_lease_mutation(&plan.lease, &stale, kind),
                Err(RemoteResourceReasonCode::LeaseStaleFence)
            );
            assert_eq!(
                authorize_remote_resource_lease_mutation(&plan.lease, &future, kind),
                Err(RemoteResourceReasonCode::LeaseUnknownFence)
            );
            assert_eq!(
                authorize_remote_resource_lease_mutation(&plan.lease, &wrong_attempt, kind),
                Err(RemoteResourceReasonCode::LeaseAttemptMismatch)
            );
        }
        assert_eq!(plan.lease.scope.fence_generation.get(), 2);
        assert_eq!(plan.lease.reserved, requirements().quantities);
    }

    #[test]
    fn restart_recovery_preserves_only_current_live_lease() {
        let plan = reservation("restart", 2, &[]);
        let current = plan_remote_resource_lease_recovery(
            &plan.lease,
            Some(&plan.lease.scope),
            Some(RemoteAttemptPhase::Running),
        )
        .unwrap();
        let terminal = plan_remote_resource_lease_recovery(
            &plan.lease,
            Some(&plan.lease.scope),
            Some(RemoteAttemptPhase::Completed),
        )
        .unwrap();
        let superseded = plan_remote_resource_lease_recovery(&plan.lease, Some(&scope("restart", 3)), None).unwrap();

        assert_eq!(current.disposition, RemoteResourceRecoveryDisposition::Preserve);
        assert_eq!(terminal.disposition, RemoteResourceRecoveryDisposition::Release);
        assert_eq!(superseded.disposition, RemoteResourceRecoveryDisposition::Release);
        assert_eq!(current.reason_code, RemoteResourceReasonCode::LeasePreservedOnRecovery);
    }

    #[test]
    fn lease_snapshot_rejects_corruption_and_overcommit() {
        let first = reservation("first", 1, &[]);
        let mut corrupt = first.lease.clone();
        corrupt.reserved.cpu_units = TEST_CPU_TOTAL;
        let inventories = BTreeMap::from([("worker-a".to_string(), inventory())]);
        let overcommitted = vec![
            reservation("one", 1, &[]).lease,
            reservation("two", 1, &[]).lease,
            reservation("three", 1, &[]).lease,
            reservation("four", 1, &[]).lease,
            reservation("five", 1, &[]).lease,
        ];

        assert_eq!(
            validate_remote_resource_lease_snapshot(&inventories, &[corrupt]),
            Err(RemoteResourceReasonCode::LeaseDigestInvalid)
        );
        assert_eq!(
            validate_remote_resource_lease_snapshot(&inventories, &overcommitted),
            Err(RemoteResourceReasonCode::LeaseSnapshotOvercommitted)
        );
    }

    #[test]
    fn verified_full_partial_and_missing_locality_are_derived_from_receiver_facts() {
        let manifest = transfer_manifest();
        let mut full = locality_observation(&manifest);
        full.receiver_facts.complete_artifact_ids.insert(manifest.manifest.artifacts[0].artifact_id.clone());
        let full = normalize_remote_verified_locality("worker-a", TEST_WORKER_GENERATION, &manifest, &full).unwrap();
        let mut partial = locality_observation(&manifest);
        partial
            .receiver_facts
            .complete_chunk_digests
            .insert(manifest.manifest.artifacts[0].chunks[0].digest_blake3.clone());
        let partial =
            normalize_remote_verified_locality("worker-a", TEST_WORKER_GENERATION, &manifest, &partial).unwrap();
        let missing = normalize_remote_verified_locality(
            "worker-a",
            TEST_WORKER_GENERATION,
            &manifest,
            &locality_observation(&manifest),
        )
        .unwrap();

        assert_eq!(full.content_locality, ContentLocalityClass::FullyPresent);
        assert_eq!(full.transfer_cost, TransferCostClass::None);
        assert_eq!(partial.content_locality, ContentLocalityClass::PartiallyPresent);
        assert_eq!(partial.transfer_cost, TransferCostClass::Medium);
        assert_eq!(missing.content_locality, ContentLocalityClass::NoVerifiedContent);
        assert_eq!(missing.transfer_cost, TransferCostClass::Large);
        assert_eq!(partial.verified_present_bytes + partial.missing_bytes, manifest.total_bytes);
    }

    #[test]
    fn stale_wrong_scope_and_unverified_locality_never_claim_zero_transfer() {
        let manifest = transfer_manifest();
        let mut stale = locality_observation(&manifest);
        stale.worker_generation -= 1;
        let mut wrong_manifest = locality_observation(&manifest);
        wrong_manifest.scope.manifest_digest_blake3 = blake3::hash(b"wrong-manifest").to_hex().to_string();
        let mut wrong_policy = locality_observation(&manifest);
        wrong_policy.scope.policy_digest_blake3 = blake3::hash(b"wrong-policy").to_hex().to_string();
        let mut unverified = locality_observation(&manifest);
        unverified.receiver_probe_verified = false;
        unverified
            .receiver_facts
            .complete_artifact_ids
            .insert(manifest.manifest.artifacts[0].artifact_id.clone());
        let downgraded =
            normalize_remote_verified_locality("worker-a", TEST_WORKER_GENERATION, &manifest, &unverified).unwrap();

        assert_eq!(
            normalize_remote_verified_locality("worker-a", TEST_WORKER_GENERATION, &manifest, &stale),
            Err(RemoteLocalityReasonCode::WorkerGenerationStale)
        );
        assert_eq!(
            normalize_remote_verified_locality("worker-a", TEST_WORKER_GENERATION, &manifest, &wrong_manifest),
            Err(RemoteLocalityReasonCode::ManifestMismatch)
        );
        assert_eq!(
            normalize_remote_verified_locality("worker-a", TEST_WORKER_GENERATION, &manifest, &wrong_policy),
            Err(RemoteLocalityReasonCode::PolicyMismatch)
        );
        assert_eq!(downgraded.verified_present_bytes, 0);
        assert_eq!(downgraded.missing_bytes, manifest.total_bytes);
        assert_ne!(downgraded.transfer_cost, TransferCostClass::None);
        assert_ne!(downgraded.content_locality, ContentLocalityClass::FullyPresent);
    }

    #[test]
    fn placement_uses_classes_and_stable_identity_not_input_order() {
        let policy = SchedulingPolicy::default();
        let local = RemoteWorkerPlacementFacts {
            worker_endpoint_id: "worker-local".to_string(),
            resource_fit: ResourceFitClass::Compatible,
            content_locality: ContentLocalityClass::FullyPresent,
            transfer_cost: TransferCostClass::None,
        };
        let remote = RemoteWorkerPlacementFacts {
            worker_endpoint_id: "worker-remote".to_string(),
            resource_fit: ResourceFitClass::Compatible,
            content_locality: ContentLocalityClass::NoVerifiedContent,
            transfer_cost: TransferCostClass::Large,
        };
        let first = rank_remote_worker_placement_candidates(&policy, &[remote.clone(), local.clone()]).unwrap();
        let second = rank_remote_worker_placement_candidates(&policy, &[local, remote]).unwrap();

        assert_eq!(first, second);
        assert_eq!(first[0].worker_endpoint_id, "worker-local");
        assert_eq!(first[0].transfer_cost, TransferCostClass::None);
        assert_eq!(first.len(), 2);
    }

    #[test]
    fn locality_cannot_create_eligibility_and_starvation_remains_authoritative() {
        let preferred = normalize_eligible_preference(
            HardEligibilityFacts::ELIGIBLE,
            ResourceFitClass::Exact,
            ContentLocalityClass::FullyPresent,
            TransferCostClass::None,
        )
        .unwrap();
        let blocked = normalize_eligible_preference(
            HardEligibilityFacts {
                output_trust_allowed: false,
                ..HardEligibilityFacts::ELIGIBLE
            },
            ResourceFitClass::Exact,
            ContentLocalityClass::FullyPresent,
            TransferCostClass::None,
        );
        let mut protected = ReadyGoalFacts::ordinary("protected".to_string(), 0);
        let mut fresh = ReadyGoalFacts::ordinary("fresh".to_string(), TEST_SCHEDULING_EPOCH);
        protected.preference = Default::default();
        fresh.preference = preferred;
        let pressures = BTreeMap::from([
            ("protected".to_string(), KnownGraphPressure::default()),
            ("fresh".to_string(), KnownGraphPressure::default()),
        ]);
        let ranked =
            rank_ready_goals(&SchedulingPolicy::default(), TEST_SCHEDULING_EPOCH, &[fresh, protected], &pressures)
                .unwrap();

        assert_eq!(blocked, Err(IneligibleReason::OutputTrustRejected));
        assert_eq!(ranked[0].goal_key, "protected");
        assert_eq!(ranked.len(), 2);
        assert_ne!(ranked[0].priority.content_locality, ContentLocalityClass::FullyPresent);
    }

    proptest! {
        #[test]
        fn checked_cpu_reservations_preserve_capacity_invariant(
            total in 2_u32..1_000_u32,
            requested in 1_u32..500_u32,
        ) {
            prop_assume!(requested <= total);
            let inventory = RemoteWorkerResourceInventory {
                total: RemoteResourceVector {
                    cpu_units: total,
                    memory_bytes: TEST_MEMORY_TOTAL,
                    scratch_bytes: TEST_SCRATCH_TOTAL,
                    accelerators: Vec::new(),
                    named_tokens: Vec::new(),
                },
            };
            let requirements = RemoteResourceRequirements {
                quantities: RemoteResourceVector {
                    cpu_units: requested,
                    memory_bytes: TEST_MEMORY_REQUEST,
                    scratch_bytes: TEST_SCRATCH_REQUEST,
                    accelerators: Vec::new(),
                    named_tokens: Vec::new(),
                },
                semantic_accelerator_classes: Vec::new(),
            };
            let plan = plan_remote_resource_reservation(scope("property", 1), &inventory, &requirements, &[]).unwrap();

            prop_assert_eq!(plan.remaining_after.cpu_units.checked_add(requested), Some(total));
            prop_assert!(plan.remaining_after.cpu_units <= total);
            prop_assert_ne!(plan.resource_fit, ResourceFitClass::Unknown);
        }
    }
}
