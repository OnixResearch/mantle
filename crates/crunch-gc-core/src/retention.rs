//! Pure, bounded store-retention and usage decisions.
//!
//! The standard-library shell supplies root, clock, closure, and byte facts.
//! This module does not read a clock, filesystem, service, or environment.

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_RETENTION_ROOTS: usize = 65_536;
pub const MAX_RETENTION_DECISIONS: usize = MAX_RETENTION_ROOTS;
pub const MAX_USAGE_OBJECTS: usize = 1_000_000;
pub const MAX_USAGE_ROOTS_PER_OBJECT: usize = 65_536;
pub const RETENTION_PLAN_ID_BYTES: usize = blake3::OUT_LEN;
pub const ROOT_CLASS_COUNT: usize = 8;

const RETENTION_PLAN_DOMAIN: &[u8] = b"mantle.retention.plan.v2";
const FIELD_SEPARATOR: u8 = 0;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RootClass {
    ExplicitPin,
    ProjectOutputGeneration,
    ProjectSourceGeneration,
    ActiveShellLease,
    Bootstrap,
    SelfBuild,
    RemoteResult,
    LegacyUnmanaged,
}

impl RootClass {
    pub const ALL: [Self; ROOT_CLASS_COUNT] = [
        Self::ExplicitPin,
        Self::ProjectOutputGeneration,
        Self::ProjectSourceGeneration,
        Self::ActiveShellLease,
        Self::Bootstrap,
        Self::SelfBuild,
        Self::RemoteResult,
        Self::LegacyUnmanaged,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitPin => "explicit-pin",
            Self::ProjectOutputGeneration => "project-output-generation",
            Self::ProjectSourceGeneration => "project-source-generation",
            Self::ActiveShellLease => "active-shell-lease",
            Self::Bootstrap => "bootstrap",
            Self::SelfBuild => "self-build",
            Self::RemoteResult => "remote-result",
            Self::LegacyUnmanaged => "legacy-unmanaged",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseFacts {
    pub lease_id: String,
    pub expires_unix_s: i64,
    pub last_observed_unix_s: i64,
    pub renewal_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionRoot {
    pub path_id: String,
    pub class: RootClass,
    pub owner_scope: String,
    pub project_identity: Option<String>,
    pub selector: Option<String>,
    pub generation: Option<u64>,
    pub generation_identity: Option<String>,
    pub lease: Option<LeaseFacts>,
    pub policy_id: String,
    pub created_unix_s: i64,
    pub last_transition_id: String,
    pub removal_requested: bool,
}

// r[impl store_lifecycle.retention_policy]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionPolicy {
    pub policy_id: String,
    pub eligible_owner_scopes: BTreeMap<RootClass, Vec<String>>,
    pub max_roots: usize,
    pub max_decisions: usize,
    pub retained_project_output_generations: usize,
    pub retained_project_source_generations: usize,
    pub max_lease_renewals: u32,
    pub max_lease_seconds: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionDisposition {
    Keep,
    Expire,
    Migrate,
    Quarantine,
    Remove,
}

impl RetentionDisposition {
    #[must_use]
    pub const fn retains_path(self) -> bool {
        matches!(self, Self::Keep | Self::Migrate | Self::Quarantine)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionReason {
    ExplicitPin,
    ProjectOutputGenerationCurrent,
    ProjectOutputGenerationSuperseded,
    ProjectSourceGenerationCurrent,
    ProjectSourceGenerationSuperseded,
    ShellLeaseActive,
    ShellLeaseExpired,
    ShellLeaseUnsafe,
    Bootstrap,
    SelfBuild,
    RemoteResult,
    LegacyUnmanagedProtected,
    ExplicitRemoval,
}

impl RetentionReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitPin => "retained-explicit-pin",
            Self::ProjectOutputGenerationCurrent => "retained-project-output-generation",
            Self::ProjectOutputGenerationSuperseded => "expired-project-output-generation",
            Self::ProjectSourceGenerationCurrent => "retained-project-source-generation",
            Self::ProjectSourceGenerationSuperseded => "expired-project-source-generation",
            Self::ShellLeaseActive => "retained-shell-lease-active",
            Self::ShellLeaseExpired => "expired-shell-lease",
            Self::ShellLeaseUnsafe => "quarantined-shell-lease-unsafe",
            Self::Bootstrap => "retained-bootstrap",
            Self::SelfBuild => "retained-self-build",
            Self::RemoteResult => "retained-remote-result",
            Self::LegacyUnmanagedProtected => "migrate-legacy-unmanaged-protected",
            Self::ExplicitRemoval => "remove-explicitly-authorized",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionDecision {
    pub path_id: String,
    pub class: RootClass,
    pub owner_scope: String,
    pub disposition: RetentionDisposition,
    pub reason: RetentionReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RetentionPlanId([u8; RETENTION_PLAN_ID_BYTES]);

impl RetentionPlanId {
    #[must_use]
    pub const fn into_bytes(self) -> [u8; RETENTION_PLAN_ID_BYTES] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionPlan {
    pub plan_id: RetentionPlanId,
    pub decisions: Vec<RetentionDecision>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetentionError {
    EmptyPolicyId,
    InvalidPolicyBound,
    InvalidOwnerRule {
        class: RootClass,
    },
    IneligibleOwner {
        path_id: String,
        class: RootClass,
        owner_scope: String,
    },
    TooManyRoots {
        actual: usize,
        maximum: usize,
    },
    TooManyDecisions {
        actual: usize,
        maximum: usize,
    },
    InvalidPathId {
        path_id: String,
    },
    DuplicateRoot {
        path_id: String,
    },
    MissingOwner {
        path_id: String,
    },
    UnknownPolicy {
        path_id: String,
        policy_id: String,
    },
    MissingProjectFacts {
        path_id: String,
    },
    MissingLease {
        path_id: String,
    },
    ClockRollback {
        path_id: String,
        current_unix_s: i64,
        last_observed_unix_s: i64,
    },
    IdentityEncodingOverflow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageObjectObservation {
    pub object_id: String,
    pub bytes: Option<u64>,
    pub unknown_reason: Option<String>,
    pub retaining_root_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootUsage {
    pub root_id: String,
    pub inclusive_bytes: u64,
    pub unique_bytes: u64,
    pub unknown_object_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageReport {
    pub observed_bytes: u64,
    pub retained_bytes: u64,
    pub reclaimable_bytes: u64,
    pub quarantined_bytes: u64,
    pub unclassified_bytes: u64,
    pub shared_bytes: u64,
    pub unknown_object_count: usize,
    pub roots: Vec<RootUsage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UsageError {
    TooManyObjects {
        actual: usize,
        maximum: usize,
    },
    TooManyRootsForObject {
        object_id: String,
        actual: usize,
        maximum: usize,
    },
    InvalidObjectId,
    DuplicateObject {
        object_id: String,
    },
    UnknownRoot {
        object_id: String,
        root_id: String,
    },
    MissingUnknownReason {
        object_id: String,
    },
    ByteOverflow,
}

pub fn plan_retention(
    policy: &RetentionPolicy,
    current_unix_s: i64,
    roots: Vec<RetentionRoot>,
) -> Result<RetentionPlan, RetentionError> {
    validate_policy(policy)?;
    if roots.len() > policy.max_roots || roots.len() > MAX_RETENTION_ROOTS {
        return Err(RetentionError::TooManyRoots {
            actual: roots.len(),
            maximum: policy.max_roots.min(MAX_RETENTION_ROOTS),
        });
    }
    let normalized = normalize_roots(policy, roots)?;
    let generation_ranks = generation_ranks(&normalized)?;
    let mut decisions = Vec::with_capacity(normalized.len());
    for root in &normalized {
        decisions.push(decide_root(policy, current_unix_s, root, &generation_ranks)?);
        if decisions.len() > policy.max_decisions || decisions.len() > MAX_RETENTION_DECISIONS {
            return Err(RetentionError::TooManyDecisions {
                actual: decisions.len(),
                maximum: policy.max_decisions.min(MAX_RETENTION_DECISIONS),
            });
        }
    }
    let plan_id = retention_plan_id(policy, &normalized, &decisions)?;
    debug_assert_eq!(normalized.len(), decisions.len());
    debug_assert!(decisions.windows(2).all(|pair| pair[0].path_id < pair[1].path_id));
    Ok(RetentionPlan { plan_id, decisions })
}

// r[impl store_lifecycle.usage_report]
pub fn aggregate_usage(
    decisions: &[RetentionDecision],
    mut objects: Vec<UsageObjectObservation>,
) -> Result<UsageReport, UsageError> {
    if objects.len() > MAX_USAGE_OBJECTS {
        return Err(UsageError::TooManyObjects {
            actual: objects.len(),
            maximum: MAX_USAGE_OBJECTS,
        });
    }
    let decisions_by_path =
        decisions.iter().map(|decision| (decision.path_id.as_str(), decision)).collect::<BTreeMap<_, _>>();
    let mut root_usage = decisions
        .iter()
        .map(|decision| {
            (decision.path_id.as_str(), RootUsage {
                root_id: decision.path_id.clone(),
                inclusive_bytes: 0,
                unique_bytes: 0,
                unknown_object_count: 0,
            })
        })
        .collect::<BTreeMap<_, _>>();
    objects.sort_by(|left, right| left.object_id.cmp(&right.object_id));
    validate_objects(&mut objects, &decisions_by_path)?;

    let mut report = UsageReport {
        observed_bytes: 0,
        retained_bytes: 0,
        reclaimable_bytes: 0,
        quarantined_bytes: 0,
        unclassified_bytes: 0,
        shared_bytes: 0,
        unknown_object_count: 0,
        roots: Vec::new(),
    };
    for object in &objects {
        accumulate_usage_object(&mut report, &mut root_usage, &decisions_by_path, object)?;
    }
    report.roots = root_usage.into_values().collect();
    debug_assert_eq!(report.roots.len(), decisions.len());
    Ok(report)
}

fn validate_policy(policy: &RetentionPolicy) -> Result<(), RetentionError> {
    if policy.policy_id.is_empty() {
        return Err(RetentionError::EmptyPolicyId);
    }
    let has_valid_bounds = policy.max_roots > 0
        && policy.max_roots <= MAX_RETENTION_ROOTS
        && policy.max_decisions > 0
        && policy.max_decisions <= MAX_RETENTION_DECISIONS
        && policy.retained_project_output_generations > 0
        && policy.retained_project_source_generations > 0
        && policy.max_lease_seconds > 0;
    if !has_valid_bounds {
        return Err(RetentionError::InvalidPolicyBound);
    }
    for class in RootClass::ALL {
        let Some(scopes) = policy.eligible_owner_scopes.get(&class) else {
            return Err(RetentionError::InvalidOwnerRule { class });
        };
        let unique_scopes = scopes.iter().map(String::as_str).collect::<BTreeSet<_>>();
        if scopes.is_empty() || unique_scopes.len() != scopes.len() || scopes.iter().any(String::is_empty) {
            return Err(RetentionError::InvalidOwnerRule { class });
        }
    }
    Ok(())
}

fn normalize_roots(
    policy: &RetentionPolicy,
    mut roots: Vec<RetentionRoot>,
) -> Result<Vec<RetentionRoot>, RetentionError> {
    for root in &roots {
        validate_path_id(&root.path_id)?;
        if root.owner_scope.is_empty() {
            return Err(RetentionError::MissingOwner {
                path_id: root.path_id.clone(),
            });
        }
        let owner_kind = root.owner_scope.split_once(':').map_or(root.owner_scope.as_str(), |(kind, _)| kind);
        let owner_is_eligible = policy
            .eligible_owner_scopes
            .get(&root.class)
            .is_some_and(|scopes| scopes.iter().any(|scope| scope == owner_kind));
        if !owner_is_eligible {
            return Err(RetentionError::IneligibleOwner {
                path_id: root.path_id.clone(),
                class: root.class,
                owner_scope: root.owner_scope.clone(),
            });
        }
        if root.policy_id != policy.policy_id {
            return Err(RetentionError::UnknownPolicy {
                path_id: root.path_id.clone(),
                policy_id: root.policy_id.clone(),
            });
        }
    }
    roots.sort_by(|left, right| left.path_id.cmp(&right.path_id));
    for pair in roots.windows(2) {
        if pair[0].path_id == pair[1].path_id {
            return Err(RetentionError::DuplicateRoot {
                path_id: pair[0].path_id.clone(),
            });
        }
    }
    Ok(roots)
}

fn generation_ranks(roots: &[RetentionRoot]) -> Result<BTreeMap<String, usize>, RetentionError> {
    let mut groups = BTreeMap::<(RootClass, String, String, String), Vec<(u64, String)>>::new();
    for root in roots {
        if !matches!(root.class, RootClass::ProjectOutputGeneration | RootClass::ProjectSourceGeneration) {
            continue;
        }
        let (Some(project_identity), Some(selector), Some(generation)) =
            (&root.project_identity, &root.selector, root.generation)
        else {
            return Err(RetentionError::MissingProjectFacts {
                path_id: root.path_id.clone(),
            });
        };
        if project_identity.is_empty() || selector.is_empty() {
            return Err(RetentionError::MissingProjectFacts {
                path_id: root.path_id.clone(),
            });
        }
        groups
            .entry((root.class, root.owner_scope.clone(), project_identity.clone(), selector.clone()))
            .or_default()
            .push((generation, root.path_id.clone()));
    }

    let mut ranks = BTreeMap::new();
    for entries in groups.values_mut() {
        entries.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
        let mut current_rank = 0_usize;
        let mut previous_generation = None;
        for (generation, path_id) in entries {
            if previous_generation.is_some_and(|previous| previous != *generation) {
                current_rank = current_rank.checked_add(1).ok_or(RetentionError::IdentityEncodingOverflow)?;
            }
            ranks.insert(path_id.clone(), current_rank);
            previous_generation = Some(*generation);
        }
    }
    Ok(ranks)
}

fn decide_root(
    policy: &RetentionPolicy,
    current_unix_s: i64,
    root: &RetentionRoot,
    generation_ranks: &BTreeMap<String, usize>,
) -> Result<RetentionDecision, RetentionError> {
    let (mut disposition, mut reason) = match root.class {
        RootClass::ExplicitPin => (RetentionDisposition::Keep, RetentionReason::ExplicitPin),
        RootClass::ProjectOutputGeneration => generation_decision(
            root,
            generation_ranks,
            policy.retained_project_output_generations,
            RetentionReason::ProjectOutputGenerationCurrent,
            RetentionReason::ProjectOutputGenerationSuperseded,
        )?,
        RootClass::ProjectSourceGeneration => generation_decision(
            root,
            generation_ranks,
            policy.retained_project_source_generations,
            RetentionReason::ProjectSourceGenerationCurrent,
            RetentionReason::ProjectSourceGenerationSuperseded,
        )?,
        RootClass::ActiveShellLease => lease_decision(policy, current_unix_s, root)?,
        RootClass::Bootstrap => (RetentionDisposition::Keep, RetentionReason::Bootstrap),
        RootClass::SelfBuild => (RetentionDisposition::Keep, RetentionReason::SelfBuild),
        RootClass::RemoteResult => (RetentionDisposition::Keep, RetentionReason::RemoteResult),
        RootClass::LegacyUnmanaged => (RetentionDisposition::Migrate, RetentionReason::LegacyUnmanagedProtected),
    };
    if root.removal_requested && matches!(disposition, RetentionDisposition::Expire | RetentionDisposition::Keep) {
        disposition = RetentionDisposition::Remove;
        reason = RetentionReason::ExplicitRemoval;
    }
    Ok(RetentionDecision {
        path_id: root.path_id.clone(),
        class: root.class,
        owner_scope: root.owner_scope.clone(),
        disposition,
        reason,
    })
}

fn generation_decision(
    root: &RetentionRoot,
    ranks: &BTreeMap<String, usize>,
    retained_generations: usize,
    retained_reason: RetentionReason,
    expired_reason: RetentionReason,
) -> Result<(RetentionDisposition, RetentionReason), RetentionError> {
    let Some(rank) = ranks.get(&root.path_id) else {
        return Err(RetentionError::MissingProjectFacts {
            path_id: root.path_id.clone(),
        });
    };
    if *rank < retained_generations {
        Ok((RetentionDisposition::Keep, retained_reason))
    } else {
        Ok((RetentionDisposition::Expire, expired_reason))
    }
}

fn lease_decision(
    policy: &RetentionPolicy,
    current_unix_s: i64,
    root: &RetentionRoot,
) -> Result<(RetentionDisposition, RetentionReason), RetentionError> {
    let Some(lease) = &root.lease else {
        return Err(RetentionError::MissingLease {
            path_id: root.path_id.clone(),
        });
    };
    if current_unix_s < lease.last_observed_unix_s {
        return Err(RetentionError::ClockRollback {
            path_id: root.path_id.clone(),
            current_unix_s,
            last_observed_unix_s: lease.last_observed_unix_s,
        });
    }
    let lease_duration_is_unsafe = lease
        .expires_unix_s
        .checked_sub(lease.last_observed_unix_s)
        .and_then(|duration| u64::try_from(duration).ok())
        .is_none_or(|duration| duration > policy.max_lease_seconds);
    let is_unsafe = lease.lease_id.is_empty()
        || lease.expires_unix_s < root.created_unix_s
        || lease.renewal_count > policy.max_lease_renewals
        || lease_duration_is_unsafe;
    if is_unsafe {
        return Ok((RetentionDisposition::Quarantine, RetentionReason::ShellLeaseUnsafe));
    }
    if current_unix_s <= lease.expires_unix_s {
        Ok((RetentionDisposition::Keep, RetentionReason::ShellLeaseActive))
    } else {
        Ok((RetentionDisposition::Expire, RetentionReason::ShellLeaseExpired))
    }
}

fn retention_plan_id(
    policy: &RetentionPolicy,
    roots: &[RetentionRoot],
    decisions: &[RetentionDecision],
) -> Result<RetentionPlanId, RetentionError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(RETENTION_PLAN_DOMAIN);
    hash_string(&mut hasher, &policy.policy_id)?;
    for class in RootClass::ALL {
        hash_string(&mut hasher, class.as_str())?;
        let mut scopes = policy
            .eligible_owner_scopes
            .get(&class)
            .cloned()
            .ok_or(RetentionError::InvalidOwnerRule { class })?;
        scopes.sort();
        hash_usize(&mut hasher, scopes.len())?;
        for scope in &scopes {
            hash_string(&mut hasher, scope)?;
        }
    }
    hash_usize(&mut hasher, policy.max_roots)?;
    hash_usize(&mut hasher, policy.max_decisions)?;
    hash_usize(&mut hasher, policy.retained_project_output_generations)?;
    hash_usize(&mut hasher, policy.retained_project_source_generations)?;
    hasher.update(&policy.max_lease_renewals.to_be_bytes());
    hasher.update(&policy.max_lease_seconds.to_be_bytes());
    hash_usize(&mut hasher, roots.len())?;
    for root in roots {
        hash_root(&mut hasher, root)?;
    }
    hash_usize(&mut hasher, decisions.len())?;
    for decision in decisions {
        hash_string(&mut hasher, &decision.path_id)?;
        hash_string(&mut hasher, decision.class.as_str())?;
        hash_string(&mut hasher, &decision.owner_scope)?;
        hasher.update(&[disposition_byte(decision.disposition)]);
        hash_string(&mut hasher, decision.reason.as_str())?;
    }
    Ok(RetentionPlanId(*hasher.finalize().as_bytes()))
}

fn hash_root(hasher: &mut blake3::Hasher, root: &RetentionRoot) -> Result<(), RetentionError> {
    hash_string(hasher, &root.path_id)?;
    hash_string(hasher, root.class.as_str())?;
    hash_string(hasher, &root.owner_scope)?;
    hash_optional_string(hasher, root.project_identity.as_deref())?;
    hash_optional_string(hasher, root.selector.as_deref())?;
    hash_optional_u64(hasher, root.generation);
    hash_optional_string(hasher, root.generation_identity.as_deref())?;
    match &root.lease {
        Some(lease) => {
            hasher.update(&[1]);
            hash_string(hasher, &lease.lease_id)?;
            hasher.update(&lease.expires_unix_s.to_be_bytes());
            hasher.update(&lease.last_observed_unix_s.to_be_bytes());
            hasher.update(&lease.renewal_count.to_be_bytes());
        }
        None => {
            hasher.update(&[0]);
        }
    }
    hash_string(hasher, &root.policy_id)?;
    hasher.update(&root.created_unix_s.to_be_bytes());
    hash_string(hasher, &root.last_transition_id)?;
    hasher.update(&[u8::from(root.removal_requested)]);
    Ok(())
}

fn disposition_byte(disposition: RetentionDisposition) -> u8 {
    match disposition {
        RetentionDisposition::Keep => 0,
        RetentionDisposition::Expire => 1,
        RetentionDisposition::Migrate => 2,
        RetentionDisposition::Quarantine => 3,
        RetentionDisposition::Remove => 4,
    }
}

fn hash_optional_string(hasher: &mut blake3::Hasher, value: Option<&str>) -> Result<(), RetentionError> {
    match value {
        Some(value) => {
            hasher.update(&[1]);
            hash_string(hasher, value)
        }
        None => {
            hasher.update(&[0]);
            Ok(())
        }
    }
}

fn hash_optional_u64(hasher: &mut blake3::Hasher, value: Option<u64>) {
    match value {
        Some(value) => {
            hasher.update(&[1]);
            hasher.update(&value.to_be_bytes());
        }
        None => {
            hasher.update(&[0]);
        }
    }
}

fn hash_usize(hasher: &mut blake3::Hasher, value: usize) -> Result<(), RetentionError> {
    let encoded = u64::try_from(value).map_err(|_| RetentionError::IdentityEncodingOverflow)?;
    hasher.update(&encoded.to_be_bytes());
    Ok(())
}

fn hash_string(hasher: &mut blake3::Hasher, value: &str) -> Result<(), RetentionError> {
    let byte_count = u64::try_from(value.len()).map_err(|_| RetentionError::IdentityEncodingOverflow)?;
    hasher.update(&byte_count.to_be_bytes());
    hasher.update(value.as_bytes());
    hasher.update(&[FIELD_SEPARATOR]);
    Ok(())
}

fn validate_path_id(path_id: &str) -> Result<(), RetentionError> {
    let has_valid_segments =
        path_id.split('/').skip(1).all(|segment| !segment.is_empty() && segment != "." && segment != "..");
    let is_valid = !path_id.is_empty()
        && path_id.len() <= super::MAX_STORE_PATH_ID_BYTES
        && path_id.starts_with('/')
        && !path_id.ends_with('/')
        && !path_id.chars().any(char::is_control)
        && has_valid_segments;
    if is_valid {
        Ok(())
    } else {
        Err(RetentionError::InvalidPathId {
            path_id: String::from(path_id),
        })
    }
}

fn validate_objects(
    objects: &mut [UsageObjectObservation],
    decisions: &BTreeMap<&str, &RetentionDecision>,
) -> Result<(), UsageError> {
    for pair in objects.windows(2) {
        if pair[0].object_id == pair[1].object_id {
            return Err(UsageError::DuplicateObject {
                object_id: pair[0].object_id.clone(),
            });
        }
    }
    for object in objects {
        if object.object_id.is_empty() {
            return Err(UsageError::InvalidObjectId);
        }
        object.retaining_root_ids.sort();
        object.retaining_root_ids.dedup();
        if object.retaining_root_ids.len() > MAX_USAGE_ROOTS_PER_OBJECT {
            return Err(UsageError::TooManyRootsForObject {
                object_id: object.object_id.clone(),
                actual: object.retaining_root_ids.len(),
                maximum: MAX_USAGE_ROOTS_PER_OBJECT,
            });
        }
        if object.bytes.is_none() && object.unknown_reason.as_ref().is_none_or(String::is_empty) {
            return Err(UsageError::MissingUnknownReason {
                object_id: object.object_id.clone(),
            });
        }
        for root_id in &object.retaining_root_ids {
            if !decisions.contains_key(root_id.as_str()) {
                return Err(UsageError::UnknownRoot {
                    object_id: object.object_id.clone(),
                    root_id: root_id.clone(),
                });
            }
        }
    }
    Ok(())
}

fn accumulate_usage_object(
    report: &mut UsageReport,
    root_usage: &mut BTreeMap<&str, RootUsage>,
    decisions: &BTreeMap<&str, &RetentionDecision>,
    object: &UsageObjectObservation,
) -> Result<(), UsageError> {
    let Some(bytes) = object.bytes else {
        report.unknown_object_count = report.unknown_object_count.checked_add(1).ok_or(UsageError::ByteOverflow)?;
        for root_id in &object.retaining_root_ids {
            let usage = root_usage.get_mut(root_id.as_str()).ok_or_else(|| UsageError::UnknownRoot {
                object_id: object.object_id.clone(),
                root_id: root_id.clone(),
            })?;
            usage.unknown_object_count = usage.unknown_object_count.checked_add(1).ok_or(UsageError::ByteOverflow)?;
        }
        return Ok(());
    };
    report.observed_bytes = checked_add(report.observed_bytes, bytes)?;
    let class = usage_class(decisions, &object.retaining_root_ids)?;
    match class {
        UsageClass::Retained => report.retained_bytes = checked_add(report.retained_bytes, bytes)?,
        UsageClass::Reclaimable => report.reclaimable_bytes = checked_add(report.reclaimable_bytes, bytes)?,
        UsageClass::Quarantined => report.quarantined_bytes = checked_add(report.quarantined_bytes, bytes)?,
        UsageClass::Unclassified => report.unclassified_bytes = checked_add(report.unclassified_bytes, bytes)?,
    }
    if object.retaining_root_ids.len() > 1 {
        report.shared_bytes = checked_add(report.shared_bytes, bytes)?;
    }
    for root_id in &object.retaining_root_ids {
        let usage = root_usage.get_mut(root_id.as_str()).ok_or_else(|| UsageError::UnknownRoot {
            object_id: object.object_id.clone(),
            root_id: root_id.clone(),
        })?;
        usage.inclusive_bytes = checked_add(usage.inclusive_bytes, bytes)?;
        if object.retaining_root_ids.len() == 1 {
            usage.unique_bytes = checked_add(usage.unique_bytes, bytes)?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UsageClass {
    Retained,
    Reclaimable,
    Quarantined,
    Unclassified,
}

fn usage_class(decisions: &BTreeMap<&str, &RetentionDecision>, root_ids: &[String]) -> Result<UsageClass, UsageError> {
    if root_ids.is_empty() {
        return Ok(UsageClass::Reclaimable);
    }
    let mut dispositions = BTreeSet::new();
    for root_id in root_ids {
        let decision = decisions.get(root_id.as_str()).ok_or_else(|| UsageError::UnknownRoot {
            object_id: String::new(),
            root_id: root_id.clone(),
        })?;
        dispositions.insert(disposition_byte(decision.disposition));
    }
    if dispositions.contains(&disposition_byte(RetentionDisposition::Keep)) {
        return Ok(UsageClass::Retained);
    }
    if dispositions.contains(&disposition_byte(RetentionDisposition::Quarantine)) {
        return Ok(UsageClass::Quarantined);
    }
    if dispositions.contains(&disposition_byte(RetentionDisposition::Migrate)) {
        return Ok(UsageClass::Unclassified);
    }
    Ok(UsageClass::Reclaimable)
}

fn checked_add(left: u64, right: u64) -> Result<u64, UsageError> {
    left.checked_add(right).ok_or(UsageError::ByteOverflow)
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    const POLICY_ID: &str = "b3:retention-policy";
    const OWNER: &str = "project:b3:alice";
    const PROJECT: &str = "b3:project";
    const SELECTOR: &str = "default";
    const CREATED_UNIX_S: i64 = 100;
    const CURRENT_UNIX_S: i64 = 200;
    const ACTIVE_EXPIRY_UNIX_S: i64 = 300;
    const EXPIRED_EXPIRY_UNIX_S: i64 = 150;
    const DEFAULT_MAX_ROOTS: usize = 64;
    const DEFAULT_MAX_RENEWALS: u32 = 8;
    const DEFAULT_MAX_LEASE_SECONDS: u64 = 3_600;
    const RETAINED_GENERATIONS: usize = 2;
    const ROOT_A: &str = "/mantle/store/aaaaaaaa-root-a";
    const ROOT_B: &str = "/mantle/store/bbbbbbbb-root-b";
    const ROOT_C: &str = "/mantle/store/cccccccc-root-c";
    const SHARED_BYTES: u64 = 10;
    const UNIQUE_BYTES: u64 = 5;

    fn policy() -> RetentionPolicy {
        RetentionPolicy {
            policy_id: POLICY_ID.to_string(),
            eligible_owner_scopes: RootClass::ALL
                .into_iter()
                .map(|class| {
                    let owner = if class == RootClass::LegacyUnmanaged {
                        "legacy-unmanaged"
                    } else {
                        "project"
                    };
                    (class, vec![owner.to_string()])
                })
                .collect(),
            max_roots: DEFAULT_MAX_ROOTS,
            max_decisions: DEFAULT_MAX_ROOTS,
            retained_project_output_generations: RETAINED_GENERATIONS,
            retained_project_source_generations: RETAINED_GENERATIONS,
            max_lease_renewals: DEFAULT_MAX_RENEWALS,
            max_lease_seconds: DEFAULT_MAX_LEASE_SECONDS,
        }
    }

    fn root(path_id: &str, class: RootClass) -> RetentionRoot {
        RetentionRoot {
            path_id: path_id.to_string(),
            class,
            owner_scope: OWNER.to_string(),
            project_identity: None,
            selector: None,
            generation: None,
            generation_identity: None,
            lease: None,
            policy_id: POLICY_ID.to_string(),
            created_unix_s: CREATED_UNIX_S,
            last_transition_id: "created".to_string(),
            removal_requested: false,
        }
    }

    fn project_root(path_id: &str, generation: u64) -> RetentionRoot {
        let mut root = root(path_id, RootClass::ProjectOutputGeneration);
        root.project_identity = Some(PROJECT.to_string());
        root.selector = Some(SELECTOR.to_string());
        root.generation = Some(generation);
        root.generation_identity = Some(format!("generation-{generation}"));
        root
    }

    // r[verify store_lifecycle.retention_policy]
    #[test]
    fn project_generations_and_active_lease_are_deterministic() {
        let mut lease = root(ROOT_C, RootClass::ActiveShellLease);
        lease.lease = Some(LeaseFacts {
            lease_id: "lease-1".to_string(),
            expires_unix_s: ACTIVE_EXPIRY_UNIX_S,
            last_observed_unix_s: CREATED_UNIX_S,
            renewal_count: 1,
        });
        let roots = vec![project_root(ROOT_A, 1), lease, project_root(ROOT_B, 2)];
        let expected = plan_retention(&policy(), CURRENT_UNIX_S, roots.clone()).expect("valid retention facts");
        let mut shuffled = roots;
        shuffled.reverse();
        let actual = plan_retention(&policy(), CURRENT_UNIX_S, shuffled).expect("equivalent facts");

        assert_eq!(actual, expected);
        assert_eq!(actual.decisions.len(), 3);
        assert!(actual.decisions.iter().all(|decision| decision.disposition == RetentionDisposition::Keep));
    }

    #[test]
    fn superseded_generation_expires_and_requires_explicit_removal() {
        let newest_generation = 3;
        let middle_generation = 2;
        let oldest_generation = 1;
        let mut oldest = project_root(ROOT_A, oldest_generation);
        oldest.removal_requested = true;
        let plan = plan_retention(&policy(), CURRENT_UNIX_S, vec![
            oldest,
            project_root(ROOT_B, middle_generation),
            project_root(ROOT_C, newest_generation),
        ])
        .expect("bounded generations");

        assert_eq!(plan.decisions[0].disposition, RetentionDisposition::Remove);
        assert_eq!(plan.decisions[0].reason, RetentionReason::ExplicitRemoval);
    }

    #[test]
    fn unsafe_lease_is_quarantined_and_clock_rollback_fails_closed() {
        let mut malformed = root(ROOT_A, RootClass::ActiveShellLease);
        malformed.lease = Some(LeaseFacts {
            lease_id: "lease-unsafe".to_string(),
            expires_unix_s: EXPIRED_EXPIRY_UNIX_S,
            last_observed_unix_s: CREATED_UNIX_S,
            renewal_count: DEFAULT_MAX_RENEWALS.saturating_add(1),
        });
        let plan = plan_retention(&policy(), CURRENT_UNIX_S, vec![malformed]).expect("unsafe lease quarantines");
        assert_eq!(plan.decisions[0].disposition, RetentionDisposition::Quarantine);

        let mut rollback = root(ROOT_B, RootClass::ActiveShellLease);
        rollback.lease = Some(LeaseFacts {
            lease_id: "lease-rollback".to_string(),
            expires_unix_s: ACTIVE_EXPIRY_UNIX_S,
            last_observed_unix_s: ACTIVE_EXPIRY_UNIX_S,
            renewal_count: 1,
        });
        assert!(matches!(
            plan_retention(&policy(), CURRENT_UNIX_S, vec![rollback]),
            Err(RetentionError::ClockRollback { .. })
        ));
    }

    #[test]
    fn legacy_root_remains_protected_and_unclassified() {
        let mut legacy = root(ROOT_A, RootClass::LegacyUnmanaged);
        legacy.owner_scope = "legacy-unmanaged".to_string();
        let plan = plan_retention(&policy(), CURRENT_UNIX_S, vec![legacy]).expect("legacy root stays protected");
        assert_eq!(plan.decisions[0].disposition, RetentionDisposition::Migrate);
        assert!(plan.decisions[0].disposition.retains_path());
    }

    #[test]
    fn duplicate_roots_and_decision_bounds_are_rejected() {
        let duplicate = root(ROOT_A, RootClass::ExplicitPin);
        let duplicate_error = plan_retention(&policy(), CURRENT_UNIX_S, vec![duplicate.clone(), duplicate])
            .expect_err("duplicate root identity must fail");
        assert!(matches!(duplicate_error, RetentionError::DuplicateRoot { .. }));

        let mut bounded_policy = policy();
        bounded_policy.max_decisions = 1;
        let bound_error = plan_retention(&bounded_policy, CURRENT_UNIX_S, vec![
            root(ROOT_A, RootClass::ExplicitPin),
            root(ROOT_B, RootClass::ExplicitPin),
        ])
        .expect_err("decision count above the policy bound must fail");
        assert!(matches!(bound_error, RetentionError::TooManyDecisions { .. }));
    }

    #[test]
    fn ineligible_owner_scope_is_rejected_for_the_root_class() {
        let mut ineligible = root(ROOT_A, RootClass::LegacyUnmanaged);
        ineligible.owner_scope = OWNER.to_string();

        let error = plan_retention(&policy(), CURRENT_UNIX_S, vec![ineligible])
            .expect_err("project owner cannot claim a legacy root");

        assert!(matches!(error, RetentionError::IneligibleOwner {
            class: RootClass::LegacyUnmanaged,
            ..
        }));
    }

    #[test]
    fn generation_sets_share_a_rank_and_unknown_policy_is_rejected() {
        const OLDER_GENERATION: u64 = 1;
        const NEWER_GENERATION: u64 = 2;
        let mut one_generation_policy = policy();
        one_generation_policy.retained_project_output_generations = 1;
        let plan = plan_retention(&one_generation_policy, CURRENT_UNIX_S, vec![
            project_root(ROOT_A, OLDER_GENERATION),
            project_root(ROOT_B, OLDER_GENERATION),
            project_root(ROOT_C, NEWER_GENERATION),
        ])
        .expect("multiple roots can belong to one project generation");
        assert_eq!(plan.decisions[0].disposition, RetentionDisposition::Expire);
        assert_eq!(plan.decisions[1].disposition, RetentionDisposition::Expire);
        assert_eq!(plan.decisions[2].disposition, RetentionDisposition::Keep);

        let mut unknown_policy = root(ROOT_C, RootClass::ExplicitPin);
        unknown_policy.policy_id = "b3:unknown".to_string();
        assert!(matches!(
            plan_retention(&policy(), CURRENT_UNIX_S, vec![unknown_policy]),
            Err(RetentionError::UnknownPolicy { .. })
        ));
    }

    // r[verify store_lifecycle.usage_report]
    #[test]
    fn usage_counts_shared_content_once_and_keeps_root_views_separate() {
        let decisions = vec![
            RetentionDecision {
                path_id: ROOT_A.to_string(),
                class: RootClass::ExplicitPin,
                owner_scope: OWNER.to_string(),
                disposition: RetentionDisposition::Keep,
                reason: RetentionReason::ExplicitPin,
            },
            RetentionDecision {
                path_id: ROOT_B.to_string(),
                class: RootClass::ExplicitPin,
                owner_scope: OWNER.to_string(),
                disposition: RetentionDisposition::Keep,
                reason: RetentionReason::ExplicitPin,
            },
        ];
        let report = aggregate_usage(&decisions, vec![
            UsageObjectObservation {
                object_id: "shared".to_string(),
                bytes: Some(SHARED_BYTES),
                unknown_reason: None,
                retaining_root_ids: vec![ROOT_A.to_string(), ROOT_B.to_string()],
            },
            UsageObjectObservation {
                object_id: "unique".to_string(),
                bytes: Some(UNIQUE_BYTES),
                unknown_reason: None,
                retaining_root_ids: vec![ROOT_A.to_string()],
            },
        ])
        .expect("bounded usage facts");

        assert_eq!(report.observed_bytes, SHARED_BYTES + UNIQUE_BYTES);
        assert_eq!(report.retained_bytes, SHARED_BYTES + UNIQUE_BYTES);
        assert_eq!(report.shared_bytes, SHARED_BYTES);
        assert_eq!(report.roots[0].inclusive_bytes, SHARED_BYTES + UNIQUE_BYTES);
        assert_eq!(report.roots[0].unique_bytes, UNIQUE_BYTES);
        assert_eq!(report.roots[1].inclusive_bytes, SHARED_BYTES);
        assert_eq!(report.roots[1].unique_bytes, 0);
    }

    #[test]
    fn usage_preserves_unknown_bytes_and_rejects_overflow() {
        let decisions = vec![RetentionDecision {
            path_id: ROOT_A.to_string(),
            class: RootClass::ExplicitPin,
            owner_scope: OWNER.to_string(),
            disposition: RetentionDisposition::Keep,
            reason: RetentionReason::ExplicitPin,
        }];
        let unknown = aggregate_usage(&decisions, vec![UsageObjectObservation {
            object_id: "unknown".to_string(),
            bytes: None,
            unknown_reason: Some("metadata-unreadable".to_string()),
            retaining_root_ids: vec![ROOT_A.to_string()],
        }])
        .expect("explicit unknown observation");
        assert_eq!(unknown.unknown_object_count, 1);
        assert_eq!(unknown.observed_bytes, 0);

        let overflow = aggregate_usage(&decisions, vec![
            UsageObjectObservation {
                object_id: "first".to_string(),
                bytes: Some(u64::MAX),
                unknown_reason: None,
                retaining_root_ids: vec![ROOT_A.to_string()],
            },
            UsageObjectObservation {
                object_id: "second".to_string(),
                bytes: Some(1),
                unknown_reason: None,
                retaining_root_ids: vec![ROOT_A.to_string()],
            },
        ]);
        assert_eq!(overflow, Err(UsageError::ByteOverflow));
    }
}
