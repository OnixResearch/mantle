#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]
//! Pure, bounded overlay-store admission and identity decisions.
//!
//! The standard-library shell supplies file observations. This crate does not
//! read files, inspect a clock or environment, open services, or mutate state.

extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_BASE_LAYERS: usize = 32;
pub const MAX_GENERATION_MEMBERS: usize = 1_000_000;
pub const MAX_GENERATION_BYTES: u64 = 1_099_511_627_776;
pub const MAX_IDENTITY_BYTES: usize = 4_096;
pub const OVERLAY_ID_BYTES: usize = blake3::OUT_LEN;

const GENERATION_DOMAIN: &[u8] = b"mantle.overlay.base-generation.v1";
const DESCRIPTOR_DOMAIN: &[u8] = b"mantle.overlay.base-descriptor.v1";
const PLAN_DOMAIN: &[u8] = b"mantle.overlay.plan.v1";
const FIELD_SEPARATOR: u8 = 0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaseCapabilityClass {
    ReadOnly,
    Writable,
}

impl BaseCapabilityClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::Writable => "writable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationMemberKind {
    File,
    Directory,
}

impl GenerationMemberKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "directory",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverlayPolicy {
    pub policy_id: String,
    pub trust_policy_id: String,
    pub allowed_state_schemas: Vec<String>,
    pub max_base_layers: usize,
    pub max_generation_members: usize,
    pub max_generation_bytes: u64,
    pub require_same_prefix: bool,
    pub require_read_only_bases: bool,
    pub reject_duplicate_bases: bool,
    pub no_backfill: bool,
    pub revalidate_before_execution: bool,
    pub revalidate_before_output_admission: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationMember {
    pub relative_path: String,
    pub kind: GenerationMemberKind,
    pub bytes: u64,
    pub content_blake3: [u8; OVERLAY_ID_BYTES],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseObservation {
    pub declaration_index: usize,
    pub declaration_identity: String,
    pub logical_prefix: String,
    pub state_schema: String,
    pub trust_policy_id: String,
    pub capability_class: BaseCapabilityClass,
    pub members: Vec<GenerationMember>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OverlayIdentity([u8; OVERLAY_ID_BYTES]);

impl OverlayIdentity {
    #[must_use]
    pub const fn into_bytes(self) -> [u8; OVERLAY_ID_BYTES] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseDescriptor {
    pub declaration_index: usize,
    pub declaration_identity: String,
    pub logical_prefix: String,
    pub state_schema: String,
    pub trust_policy_id: String,
    pub capability_class: BaseCapabilityClass,
    pub generation_identity: OverlayIdentity,
    pub descriptor_identity: OverlayIdentity,
    pub observed_member_count: usize,
    pub observed_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverlayPlan {
    pub policy_id: String,
    pub logical_prefix: String,
    pub descriptors: Vec<BaseDescriptor>,
    pub plan_identity: OverlayIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OverlayError {
    EmptyPolicyId,
    EmptyTrustPolicyId,
    InvalidPolicyBound,
    InvalidLogicalPrefix {
        logical_prefix: String,
    },
    InvalidAllowedStateSchemas,
    NoBaseLayers,
    TooManyBaseLayers {
        actual: usize,
        maximum: usize,
    },
    InvalidDeclarationIndex {
        expected: usize,
        actual: usize,
    },
    InvalidDeclarationIdentity,
    DuplicateBase {
        declaration_identity: String,
    },
    PrefixMismatch {
        declaration_index: usize,
        expected: String,
        actual: String,
    },
    UnsupportedStateSchema {
        declaration_index: usize,
        state_schema: String,
    },
    UnknownTrustPolicy {
        declaration_index: usize,
        trust_policy_id: String,
    },
    WritableBase {
        declaration_index: usize,
    },
    EmptyGeneration {
        declaration_index: usize,
    },
    TooManyGenerationMembers {
        declaration_index: usize,
        actual: usize,
        maximum: usize,
    },
    InvalidGenerationPath {
        declaration_index: usize,
        relative_path: String,
    },
    DuplicateGenerationPath {
        declaration_index: usize,
        relative_path: String,
    },
    GenerationBytesOverflow {
        declaration_index: usize,
    },
    TooManyGenerationBytes {
        declaration_index: usize,
        actual: u64,
        maximum: u64,
    },
    IdentityEncodingOverflow,
    BaseGenerationDrift {
        declaration_index: usize,
    },
    CompositionDrift,
}

// r[impl store_lifecycle.overlay_composition]
// r[impl store_lifecycle.overlay_base_generation]
pub fn plan_overlay(
    policy: &OverlayPolicy,
    logical_prefix: &str,
    observations: Vec<BaseObservation>,
) -> Result<OverlayPlan, OverlayError> {
    validate_policy(policy)?;
    validate_logical_prefix(logical_prefix)?;
    validate_layer_count(policy, observations.len())?;

    let mut declaration_identities = BTreeSet::new();
    let mut descriptors = Vec::with_capacity(observations.len());
    for (expected_index, observation) in observations.into_iter().enumerate() {
        validate_observation(policy, logical_prefix, expected_index, &observation, &mut declaration_identities)?;
        descriptors.push(descriptor_for_observation(observation)?);
    }
    let plan_identity = plan_identity(policy, logical_prefix, &descriptors)?;
    debug_assert_eq!(descriptors.len(), declaration_identities.len());
    Ok(OverlayPlan {
        policy_id: policy.policy_id.clone(),
        logical_prefix: String::from(logical_prefix),
        descriptors,
        plan_identity,
    })
}

// r[impl store_lifecycle.overlay_base_generation]
pub fn revalidate_overlay(expected: &OverlayPlan, observed: &OverlayPlan) -> Result<(), OverlayError> {
    if expected.policy_id != observed.policy_id || expected.logical_prefix != observed.logical_prefix {
        return Err(OverlayError::CompositionDrift);
    }
    if expected.descriptors.len() != observed.descriptors.len() {
        return Err(OverlayError::CompositionDrift);
    }
    for (expected_descriptor, observed_descriptor) in expected.descriptors.iter().zip(&observed.descriptors) {
        if expected_descriptor.declaration_index != observed_descriptor.declaration_index
            || expected_descriptor.declaration_identity != observed_descriptor.declaration_identity
            || expected_descriptor.descriptor_identity != observed_descriptor.descriptor_identity
        {
            return Err(OverlayError::BaseGenerationDrift {
                declaration_index: expected_descriptor.declaration_index,
            });
        }
    }
    if expected.plan_identity != observed.plan_identity {
        return Err(OverlayError::CompositionDrift);
    }
    Ok(())
}

fn validate_policy(policy: &OverlayPolicy) -> Result<(), OverlayError> {
    if policy.policy_id.is_empty() {
        return Err(OverlayError::EmptyPolicyId);
    }
    if policy.trust_policy_id.is_empty() {
        return Err(OverlayError::EmptyTrustPolicyId);
    }
    let bounds_are_valid = policy.max_base_layers > 0
        && policy.max_base_layers <= MAX_BASE_LAYERS
        && policy.max_generation_members > 0
        && policy.max_generation_members <= MAX_GENERATION_MEMBERS
        && policy.max_generation_bytes > 0
        && policy.max_generation_bytes <= MAX_GENERATION_BYTES;
    if !bounds_are_valid {
        return Err(OverlayError::InvalidPolicyBound);
    }
    let schemas = policy.allowed_state_schemas.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if policy.allowed_state_schemas.is_empty()
        || schemas.len() != policy.allowed_state_schemas.len()
        || policy.allowed_state_schemas.iter().any(String::is_empty)
    {
        return Err(OverlayError::InvalidAllowedStateSchemas);
    }
    Ok(())
}

fn validate_logical_prefix(logical_prefix: &str) -> Result<(), OverlayError> {
    let valid = logical_prefix.starts_with('/')
        && logical_prefix.len() > 1
        && logical_prefix.len() <= MAX_IDENTITY_BYTES
        && !logical_prefix.ends_with('/')
        && !logical_prefix.chars().any(char::is_control)
        && logical_prefix
            .split('/')
            .skip(1)
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..");
    if !valid {
        return Err(OverlayError::InvalidLogicalPrefix {
            logical_prefix: String::from(logical_prefix),
        });
    }
    Ok(())
}

fn validate_layer_count(policy: &OverlayPolicy, actual: usize) -> Result<(), OverlayError> {
    if actual == 0 {
        return Err(OverlayError::NoBaseLayers);
    }
    if actual > policy.max_base_layers || actual > MAX_BASE_LAYERS {
        return Err(OverlayError::TooManyBaseLayers {
            actual,
            maximum: policy.max_base_layers.min(MAX_BASE_LAYERS),
        });
    }
    Ok(())
}

fn validate_observation(
    policy: &OverlayPolicy,
    logical_prefix: &str,
    expected_index: usize,
    observation: &BaseObservation,
    declaration_identities: &mut BTreeSet<String>,
) -> Result<(), OverlayError> {
    if observation.declaration_index != expected_index {
        return Err(OverlayError::InvalidDeclarationIndex {
            expected: expected_index,
            actual: observation.declaration_index,
        });
    }
    if !valid_identity(&observation.declaration_identity) {
        return Err(OverlayError::InvalidDeclarationIdentity);
    }
    if policy.reject_duplicate_bases && !declaration_identities.insert(observation.declaration_identity.clone()) {
        return Err(OverlayError::DuplicateBase {
            declaration_identity: observation.declaration_identity.clone(),
        });
    }
    if policy.require_same_prefix && observation.logical_prefix != logical_prefix {
        return Err(OverlayError::PrefixMismatch {
            declaration_index: expected_index,
            expected: String::from(logical_prefix),
            actual: observation.logical_prefix.clone(),
        });
    }
    if !policy.allowed_state_schemas.iter().any(|schema| schema == &observation.state_schema) {
        return Err(OverlayError::UnsupportedStateSchema {
            declaration_index: expected_index,
            state_schema: observation.state_schema.clone(),
        });
    }
    if observation.trust_policy_id != policy.trust_policy_id {
        return Err(OverlayError::UnknownTrustPolicy {
            declaration_index: expected_index,
            trust_policy_id: observation.trust_policy_id.clone(),
        });
    }
    if policy.require_read_only_bases && observation.capability_class != BaseCapabilityClass::ReadOnly {
        return Err(OverlayError::WritableBase {
            declaration_index: expected_index,
        });
    }
    validate_generation_members(policy, observation)
}

fn validate_generation_members(policy: &OverlayPolicy, observation: &BaseObservation) -> Result<(), OverlayError> {
    if observation.members.is_empty() {
        return Err(OverlayError::EmptyGeneration {
            declaration_index: observation.declaration_index,
        });
    }
    if observation.members.len() > policy.max_generation_members || observation.members.len() > MAX_GENERATION_MEMBERS {
        return Err(OverlayError::TooManyGenerationMembers {
            declaration_index: observation.declaration_index,
            actual: observation.members.len(),
            maximum: policy.max_generation_members.min(MAX_GENERATION_MEMBERS),
        });
    }
    let mut paths = BTreeSet::new();
    let mut observed_bytes = 0_u64;
    for member in &observation.members {
        if !valid_relative_path(&member.relative_path) {
            return Err(OverlayError::InvalidGenerationPath {
                declaration_index: observation.declaration_index,
                relative_path: member.relative_path.clone(),
            });
        }
        if !paths.insert(member.relative_path.as_str()) {
            return Err(OverlayError::DuplicateGenerationPath {
                declaration_index: observation.declaration_index,
                relative_path: member.relative_path.clone(),
            });
        }
        observed_bytes = observed_bytes.checked_add(member.bytes).ok_or(OverlayError::GenerationBytesOverflow {
            declaration_index: observation.declaration_index,
        })?;
        if observed_bytes > policy.max_generation_bytes || observed_bytes > MAX_GENERATION_BYTES {
            return Err(OverlayError::TooManyGenerationBytes {
                declaration_index: observation.declaration_index,
                actual: observed_bytes,
                maximum: policy.max_generation_bytes.min(MAX_GENERATION_BYTES),
            });
        }
    }
    Ok(())
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_IDENTITY_BYTES && !value.chars().any(char::is_control)
}

fn valid_relative_path(value: &str) -> bool {
    valid_identity(value)
        && !value.starts_with('/')
        && !value.ends_with('/')
        && value.split('/').all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

fn descriptor_for_observation(mut observation: BaseObservation) -> Result<BaseDescriptor, OverlayError> {
    observation.members.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    let observed_bytes = observation.members.iter().try_fold(0_u64, |total, member| {
        total.checked_add(member.bytes).ok_or(OverlayError::GenerationBytesOverflow {
            declaration_index: observation.declaration_index,
        })
    })?;
    let generation_identity = generation_identity(&observation)?;
    let descriptor_identity = descriptor_identity(&observation, generation_identity)?;
    Ok(BaseDescriptor {
        declaration_index: observation.declaration_index,
        declaration_identity: observation.declaration_identity,
        logical_prefix: observation.logical_prefix,
        state_schema: observation.state_schema,
        trust_policy_id: observation.trust_policy_id,
        capability_class: observation.capability_class,
        generation_identity,
        descriptor_identity,
        observed_member_count: observation.members.len(),
        observed_bytes,
    })
}

fn generation_identity(observation: &BaseObservation) -> Result<OverlayIdentity, OverlayError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(GENERATION_DOMAIN);
    hash_usize(&mut hasher, observation.declaration_index)?;
    hash_string(&mut hasher, &observation.declaration_identity)?;
    hash_usize(&mut hasher, observation.members.len())?;
    for member in &observation.members {
        hash_string(&mut hasher, &member.relative_path)?;
        hash_string(&mut hasher, member.kind.as_str())?;
        hasher.update(&member.bytes.to_be_bytes());
        hasher.update(&member.content_blake3);
    }
    Ok(OverlayIdentity(*hasher.finalize().as_bytes()))
}

fn descriptor_identity(
    observation: &BaseObservation,
    generation_identity: OverlayIdentity,
) -> Result<OverlayIdentity, OverlayError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(DESCRIPTOR_DOMAIN);
    hash_usize(&mut hasher, observation.declaration_index)?;
    hash_string(&mut hasher, &observation.declaration_identity)?;
    hash_string(&mut hasher, &observation.logical_prefix)?;
    hash_string(&mut hasher, &observation.state_schema)?;
    hash_string(&mut hasher, &observation.trust_policy_id)?;
    hash_string(&mut hasher, observation.capability_class.as_str())?;
    hasher.update(&generation_identity.into_bytes());
    Ok(OverlayIdentity(*hasher.finalize().as_bytes()))
}

fn plan_identity(
    policy: &OverlayPolicy,
    logical_prefix: &str,
    descriptors: &[BaseDescriptor],
) -> Result<OverlayIdentity, OverlayError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(PLAN_DOMAIN);
    hash_string(&mut hasher, &policy.policy_id)?;
    hash_string(&mut hasher, logical_prefix)?;
    hash_usize(&mut hasher, descriptors.len())?;
    for descriptor in descriptors {
        hash_usize(&mut hasher, descriptor.declaration_index)?;
        hasher.update(&descriptor.descriptor_identity.into_bytes());
    }
    Ok(OverlayIdentity(*hasher.finalize().as_bytes()))
}

fn hash_usize(hasher: &mut blake3::Hasher, value: usize) -> Result<(), OverlayError> {
    let encoded = u64::try_from(value).map_err(|_| OverlayError::IdentityEncodingOverflow)?;
    hasher.update(&encoded.to_be_bytes());
    Ok(())
}

fn hash_string(hasher: &mut blake3::Hasher, value: &str) -> Result<(), OverlayError> {
    let byte_count = u64::try_from(value.len()).map_err(|_| OverlayError::IdentityEncodingOverflow)?;
    hasher.update(&byte_count.to_be_bytes());
    hasher.update(value.as_bytes());
    hasher.update(&[FIELD_SEPARATOR]);
    Ok(())
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    const POLICY_ID: &str = "mantle-overlay-policy-v1";
    const TRUST_POLICY_ID: &str = "mantle-overlay-trust-v1";
    const STATE_SCHEMA: &str = "mantle-store-state-v1";
    const LOGICAL_PREFIX: &str = "/nix/store";
    const FIRST_BYTES: u64 = 11;
    const SECOND_BYTES: u64 = 17;

    fn policy() -> OverlayPolicy {
        OverlayPolicy {
            policy_id: POLICY_ID.to_string(),
            trust_policy_id: TRUST_POLICY_ID.to_string(),
            allowed_state_schemas: vec![STATE_SCHEMA.to_string()],
            max_base_layers: MAX_BASE_LAYERS,
            max_generation_members: MAX_GENERATION_MEMBERS,
            max_generation_bytes: MAX_GENERATION_BYTES,
            require_same_prefix: true,
            require_read_only_bases: true,
            reject_duplicate_bases: true,
            no_backfill: true,
            revalidate_before_execution: true,
            revalidate_before_output_admission: true,
        }
    }

    fn member(path: &str, byte: u8, bytes: u64) -> GenerationMember {
        GenerationMember {
            relative_path: path.to_string(),
            kind: GenerationMemberKind::File,
            bytes,
            content_blake3: [byte; OVERLAY_ID_BYTES],
        }
    }

    fn observation(index: usize, identity: &str) -> BaseObservation {
        BaseObservation {
            declaration_index: index,
            declaration_identity: identity.to_string(),
            logical_prefix: LOGICAL_PREFIX.to_string(),
            state_schema: STATE_SCHEMA.to_string(),
            trust_policy_id: TRUST_POLICY_ID.to_string(),
            capability_class: BaseCapabilityClass::ReadOnly,
            members: vec![
                member("pathinfo.redb", 1, FIRST_BYTES),
                member("directories.redb", 2, SECOND_BYTES),
            ],
        }
    }

    #[test]
    fn ordered_descriptors_and_shuffled_members_are_deterministic() {
        let first = plan_overlay(&policy(), LOGICAL_PREFIX, vec![observation(0, "base-a"), observation(1, "base-b")])
            .expect("valid overlay facts must plan");
        let mut shuffled_a = observation(0, "base-a");
        shuffled_a.members.reverse();
        let second = plan_overlay(&policy(), LOGICAL_PREFIX, vec![shuffled_a, observation(1, "base-b")])
            .expect("equivalent overlay facts must plan");
        assert_eq!(first, second);
        assert_eq!(first.descriptors[0].declaration_identity, "base-a");
        assert_eq!(first.descriptors[1].declaration_identity, "base-b");
    }

    #[test]
    fn generation_drift_is_rejected() {
        let expected = plan_overlay(&policy(), LOGICAL_PREFIX, vec![observation(0, "base-a")])
            .expect("valid overlay facts must plan");
        let mut changed = observation(0, "base-a");
        changed.members[0].content_blake3 = [9; OVERLAY_ID_BYTES];
        let observed =
            plan_overlay(&policy(), LOGICAL_PREFIX, vec![changed]).expect("changed but valid facts must plan");
        assert_eq!(
            revalidate_overlay(&expected, &observed),
            Err(OverlayError::BaseGenerationDrift { declaration_index: 0 })
        );
    }

    #[test]
    fn prefix_mismatch_is_rejected() {
        let mut base = observation(0, "base-a");
        base.logical_prefix = "/mantle/store".to_string();
        assert!(matches!(
            plan_overlay(&policy(), LOGICAL_PREFIX, vec![base]),
            Err(OverlayError::PrefixMismatch {
                declaration_index: 0,
                ..
            })
        ));
    }

    #[test]
    fn duplicate_base_is_rejected() {
        assert_eq!(
            plan_overlay(&policy(), LOGICAL_PREFIX, vec![observation(0, "base-a"), observation(1, "base-a")]),
            Err(OverlayError::DuplicateBase {
                declaration_identity: "base-a".to_string(),
            })
        );
    }

    #[test]
    fn writable_base_is_rejected() {
        let mut base = observation(0, "base-a");
        base.capability_class = BaseCapabilityClass::Writable;
        assert_eq!(
            plan_overlay(&policy(), LOGICAL_PREFIX, vec![base]),
            Err(OverlayError::WritableBase { declaration_index: 0 })
        );
    }

    #[test]
    fn unknown_trust_policy_is_rejected() {
        let mut base = observation(0, "base-a");
        base.trust_policy_id = "unknown".to_string();
        assert_eq!(
            plan_overlay(&policy(), LOGICAL_PREFIX, vec![base]),
            Err(OverlayError::UnknownTrustPolicy {
                declaration_index: 0,
                trust_policy_id: "unknown".to_string(),
            })
        );
    }

    #[test]
    fn excess_layers_are_rejected() {
        let mut bounded_policy = policy();
        bounded_policy.max_base_layers = 1;
        assert_eq!(
            plan_overlay(&bounded_policy, LOGICAL_PREFIX, vec![observation(0, "base-a"), observation(1, "base-b")],),
            Err(OverlayError::TooManyBaseLayers { actual: 2, maximum: 1 })
        );
    }

    #[test]
    fn malformed_generation_path_is_rejected() {
        let mut base = observation(0, "base-a");
        base.members[0].relative_path = "../pathinfo.redb".to_string();
        assert!(matches!(
            plan_overlay(&policy(), LOGICAL_PREFIX, vec![base]),
            Err(OverlayError::InvalidGenerationPath {
                declaration_index: 0,
                ..
            })
        ));
    }
}
