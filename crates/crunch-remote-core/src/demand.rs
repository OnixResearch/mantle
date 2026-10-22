//! Lifetime-bound diagnostic demand. Nothing here can admit a worker, reserve
//! resources, grant a lease, or change placement preference.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

pub const MAX_DEMANDS: usize = 1_024;
pub const MAX_ID_BYTES: usize = 256;
pub const MAX_CLASSES: usize = 32;
pub const MAX_CLASS_BYTES: usize = 64;
pub const MAX_RESOURCE_COUNT: u32 = 1_048_576;
pub const MAX_RESOURCE_BYTES: u64 = 1_125_899_906_842_624;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct NamedQuantity {
    pub name: String,
    pub quantity: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct Requirement {
    pub cpu_units: u32,
    pub memory_bytes: u64,
    pub scratch_bytes: u64,
    pub accelerators: Vec<NamedQuantity>,
    pub named_tokens: Vec<NamedQuantity>,
    pub semantic_accelerator_classes: Vec<String>,
    /// Binds the admitted system/features/sandbox/network and semantic build
    /// identity. This is not an observation of available worker capacity.
    pub placement_key_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct WorkerDemand {
    pub job_id: String,
    pub policy_id: String,
    pub requirement_id_blake3: String,
    pub requirement: Requirement,
    pub blocker_reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemandError {
    InvalidIdentity,
    InvalidRequirement,
    IdentityMismatch,
    CountExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Retraction {
    Placed,
    Cancelled,
    TimedOut,
    RequirementChanged,
    OwnerStopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemandChange {
    Published,
    Replaced,
    Unchanged,
    Retracted,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DemandBook {
    by_job: BTreeMap<String, WorkerDemand>,
}

impl DemandBook {
    pub fn facts(&self) -> impl Iterator<Item = &WorkerDemand> {
        self.by_job.values()
    }

    /// Only a building-plane caller with an admitted unmet requirement calls
    /// this method. Repeated rounds do not add another fact or accumulate time.
    // r[impl build_scheduling.declared_worker_demand]
    pub fn observe_unmet(&mut self, fact: WorkerDemand) -> Result<DemandChange, DemandError> {
        fact.validate()?;
        if self.by_job.get(&fact.job_id) == Some(&fact) {
            return Ok(DemandChange::Unchanged);
        }
        let is_replacing = self.by_job.contains_key(&fact.job_id);
        if !is_replacing && self.by_job.len() >= MAX_DEMANDS {
            return Err(DemandError::CountExceeded);
        }
        self.by_job.insert(fact.job_id.clone(), fact);
        Ok(if is_replacing {
            DemandChange::Replaced
        } else {
            DemandChange::Published
        })
    }

    /// No TTL: a real owning request transition explicitly retracts its fact.
    pub fn retract(&mut self, job_id: &str, _cause: Retraction) -> DemandChange {
        if self.by_job.remove(job_id).is_some() {
            DemandChange::Retracted
        } else {
            DemandChange::Unchanged
        }
    }
}

impl WorkerDemand {
    pub fn new(
        job_id: &str,
        policy_id: &str,
        requirement: Requirement,
        blocker_reason: &str,
    ) -> Result<Self, DemandError> {
        admit_identity(job_id)?;
        admit_identity(policy_id)?;
        admit_identity(blocker_reason)?;
        requirement.validate()?;
        let requirement_id_blake3 = requirement_identity(RequirementIdentity {
            job: job_id,
            policy: policy_id,
            requirement: &requirement,
        });
        Ok(Self {
            job_id: job_id.to_string(),
            policy_id: policy_id.to_string(),
            requirement_id_blake3,
            requirement,
            blocker_reason: blocker_reason.to_string(),
        })
    }

    /// Daemon validation checks bounded restatement and identity only. It must
    /// never derive demand, or treat this diagnostic as a verified inventory.
    pub fn validate(&self) -> Result<(), DemandError> {
        admit_identity(&self.job_id)?;
        admit_identity(&self.policy_id)?;
        admit_identity(&self.blocker_reason)?;
        self.requirement.validate()?;
        let identity = requirement_identity(RequirementIdentity {
            job: self.job_id.as_str(),
            policy: self.policy_id.as_str(),
            requirement: &self.requirement,
        });
        if self.requirement_id_blake3 != identity {
            return Err(DemandError::IdentityMismatch);
        }
        Ok(())
    }
}

impl Requirement {
    pub fn validate(&self) -> Result<(), DemandError> {
        let has_valid_cpu = self.cpu_units != 0 && self.cpu_units <= MAX_RESOURCE_COUNT;
        let has_valid_memory = self.memory_bytes <= MAX_RESOURCE_BYTES;
        let has_valid_scratch = self.scratch_bytes <= MAX_RESOURCE_BYTES;
        let has_valid_placement = is_digest(&self.placement_key_blake3);
        let is_valid = has_valid_cpu && has_valid_memory && has_valid_scratch && has_valid_placement;
        if !is_valid {
            return Err(DemandError::InvalidRequirement);
        }
        validate_named(&self.accelerators)?;
        validate_named(&self.named_tokens)?;
        if self.semantic_accelerator_classes.len() > MAX_CLASSES {
            return Err(DemandError::InvalidRequirement);
        }
        let mut previous: Option<&str> = None;
        for name in &self.semantic_accelerator_classes {
            validate_class(name)?;
            if previous.is_some_and(|value| value >= name.as_str()) {
                return Err(DemandError::InvalidRequirement);
            }
            previous = Some(name);
        }
        Ok(())
    }
}

fn admit_identity(value: &str) -> Result<(), DemandError> {
    if value.is_empty() || value.len() > MAX_ID_BYTES || value.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(DemandError::InvalidIdentity);
    }
    Ok(())
}

fn validate_class(value: &str) -> Result<(), DemandError> {
    if value.is_empty()
        || value.len() > MAX_CLASS_BYTES
        || !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"-_.:".contains(&byte))
    {
        return Err(DemandError::InvalidRequirement);
    }
    Ok(())
}

fn validate_named(values: &[NamedQuantity]) -> Result<(), DemandError> {
    if values.len() > MAX_CLASSES {
        return Err(DemandError::InvalidRequirement);
    }
    let mut previous: Option<&str> = None;
    for value in values {
        validate_class(&value.name)?;
        if value.quantity == 0
            || value.quantity > MAX_RESOURCE_COUNT
            || previous.is_some_and(|name| name >= value.name.as_str())
        {
            return Err(DemandError::InvalidRequirement);
        }
        previous = Some(&value.name);
    }
    Ok(())
}

fn is_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Identity inputs for one worker demand requirement.
struct RequirementIdentity<'a> {
    job: &'a str,
    policy: &'a str,
    requirement: &'a Requirement,
}

fn requirement_identity(input: RequirementIdentity<'_>) -> String {
    let requirement = input.requirement;
    let mut hash = blake3::Hasher::new();
    hash.update(b"mantle-worker-demand-requirement-v1\0");
    for value in [input.job, input.policy, &requirement.placement_key_blake3] {
        hash_part(&mut hash, value);
    }
    hash.update(&requirement.cpu_units.to_le_bytes());
    hash.update(&requirement.memory_bytes.to_le_bytes());
    hash.update(&requirement.scratch_bytes.to_le_bytes());
    for values in [&requirement.accelerators, &requirement.named_tokens] {
        hash.update(&(values.len() as u32).to_le_bytes());
        for value in values {
            hash_part(&mut hash, &value.name);
            hash.update(&value.quantity.to_le_bytes());
        }
    }
    hash.update(&(requirement.semantic_accelerator_classes.len() as u32).to_le_bytes());
    for value in &requirement.semantic_accelerator_classes {
        hash_part(&mut hash, value);
    }
    let identity = hash.finalize().to_hex().to_string();
    debug_assert_eq!(identity.len(), 64);
    debug_assert!(is_digest(&identity));
    identity
}

fn hash_part(hash: &mut blake3::Hasher, value: &str) {
    hash.update(&(value.len() as u32).to_le_bytes());
    hash.update(value.as_bytes());
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn requirement() -> Requirement {
        Requirement {
            cpu_units: 2,
            memory_bytes: 8_192,
            scratch_bytes: 16_384,
            accelerators: vec![],
            named_tokens: vec![],
            semantic_accelerator_classes: vec![],
            placement_key_blake3: "a".repeat(64),
        }
    }

    #[test]
    fn repeated_rounds_dedupe_changed_requirements_replace_and_every_lifetime_end_retracts() {
        let mut book = DemandBook::default();
        let first = WorkerDemand::new("job", "policy", requirement(), "no-workers-registered").unwrap();
        assert_eq!(book.observe_unmet(first.clone()), Ok(DemandChange::Published));
        assert_eq!(book.observe_unmet(first.clone()), Ok(DemandChange::Unchanged));
        let mut changed = requirement();
        changed.memory_bytes += 1;
        let second = WorkerDemand::new("job", "policy", changed, "resource-memory-insufficient").unwrap();
        assert_ne!(first.requirement_id_blake3, second.requirement_id_blake3);
        assert_eq!(book.observe_unmet(second.clone()), Ok(DemandChange::Replaced));
        assert_eq!(book.facts().collect::<Vec<_>>(), vec![&second]);
        for cause in [
            Retraction::Placed,
            Retraction::Cancelled,
            Retraction::TimedOut,
            Retraction::RequirementChanged,
            Retraction::OwnerStopped,
        ] {
            book.observe_unmet(second.clone()).unwrap();
            assert_eq!(book.retract("job", cause), DemandChange::Retracted);
            assert_eq!(book.facts().count(), 0);
        }
    }

    #[test]
    fn forged_identity_noncanonical_resources_and_count_bound_do_not_mutate_book() {
        let mut book = DemandBook::default();
        let mut bad = WorkerDemand::new("job", "policy", requirement(), "blocked").unwrap();
        bad.requirement.memory_bytes += 1;
        assert_eq!(book.observe_unmet(bad), Err(DemandError::IdentityMismatch));
        let mut bad_requirement = requirement();
        bad_requirement.named_tokens = vec![NamedQuantity {
            name: "license".into(),
            quantity: 0,
        }];
        assert_eq!(bad_requirement.validate(), Err(DemandError::InvalidRequirement));
        for index in 0..MAX_DEMANDS {
            let job = alloc::format!("job-{index}");
            book.observe_unmet(WorkerDemand::new(&job, "policy", requirement(), "blocked").unwrap()).unwrap();
        }
        let before = book.clone();
        assert_eq!(
            book.observe_unmet(WorkerDemand::new("overflow", "policy", requirement(), "blocked").unwrap()),
            Err(DemandError::CountExceeded)
        );
        assert_eq!(book, before);
    }
}
