//! Pure, bounded store-retention and usage decisions.
//!
//! The standard-library shell supplies root, clock, closure, and byte facts.
//! This module does not read a clock, filesystem, service, or environment.

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
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
    const fn as_str(self) -> &'static str {
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

impl core::fmt::Display for RootClass {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.as_str())
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
    const fn as_str(self) -> &'static str {
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

impl core::fmt::Display for RetentionReason {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.as_str())
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
    policy: RetentionPolicy,
    current_unix_s: i64,
    roots: Vec<RetentionRoot>,
) -> Result<RetentionPlan, RetentionError> {
    validate_policy(&policy)?;
    if roots.len() > policy.max_roots || roots.len() > MAX_RETENTION_ROOTS {
        return Err(RetentionError::TooManyRoots {
            actual: roots.len(),
            maximum: policy.max_roots.min(MAX_RETENTION_ROOTS),
        });
    }
    let normalized = normalize_roots(&policy, roots)?;
    let generation_ranks = generation_ranks(&normalized)?;
    let mut decisions = Vec::with_capacity(normalized.len());
    for root in &normalized {
        decisions.push(decide_root(&policy, current_unix_s, root, &generation_ranks)?);
        if decisions.len() > policy.max_decisions || decisions.len() > MAX_RETENTION_DECISIONS {
            return Err(RetentionError::TooManyDecisions {
                actual: decisions.len(),
                maximum: policy.max_decisions.min(MAX_RETENTION_DECISIONS),
            });
        }
    }
    let plan_id = retention_plan_id(&policy, &normalized, &decisions)?;
    debug_assert_eq!(normalized.len(), decisions.len());
    debug_assert!(decisions.windows(2).all(|pair| pair[0].path_id < pair[1].path_id));
    Ok(RetentionPlan { plan_id, decisions })
}

// r[impl store_lifecycle.usage_report]
pub fn aggregate_usage(
    decisions: Vec<RetentionDecision>,
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

    let mut usage_totals = UsageReport {
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
        accumulate_usage_object(&mut usage_totals, &mut root_usage, &decisions_by_path, object)?;
    }
    usage_totals.roots = root_usage.into_values().collect();
    debug_assert_eq!(usage_totals.roots.len(), decisions.len());
    Ok(usage_totals)
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
    debug_assert!(policy.max_roots >= 1 && policy.max_decisions >= 1);
    debug_assert!(policy.max_lease_seconds > 0);
    Ok(())
}

fn normalize_roots(
    policy: &RetentionPolicy,
    mut roots: Vec<RetentionRoot>,
) -> Result<Vec<RetentionRoot>, RetentionError> {
    debug_assert!(!policy.policy_id.is_empty());
    for root in &roots {
        validate_path_id(&root.path_id)?;
        if root.owner_scope.is_empty() {
            return Err(RetentionError::MissingOwner {
                path_id: root.path_id.clone(),
            });
        }
        let owner_kind = root.owner_scope.split_once(':').map_or(root.owner_scope.as_str(), |(kind, _)| kind);
        let is_owner_eligible = policy
            .eligible_owner_scopes
            .get(&root.class)
            .is_some_and(|scopes| scopes.iter().any(|scope| scope == owner_kind));
        if !is_owner_eligible {
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
    debug_assert!(roots.windows(2).all(|pair| pair[0].path_id < pair[1].path_id));
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
            if ranks.len() >= roots.len() {
                return Err(RetentionError::IdentityEncodingOverflow);
            }
            if previous_generation.is_some_and(|previous| previous != *generation) {
                current_rank = current_rank.checked_add(1).ok_or(RetentionError::IdentityEncodingOverflow)?;
            }
            ranks.insert(path_id.clone(), current_rank);
            previous_generation = Some(*generation);
        }
    }
    debug_assert!(ranks.len() <= roots.len());
    Ok(ranks)
}

fn decide_root(
    policy: &RetentionPolicy,
    current_unix_s: i64,
    root: &RetentionRoot,
    generation_ranks: &BTreeMap<String, usize>,
) -> Result<RetentionDecision, RetentionError> {
    debug_assert!(!root.path_id.is_empty());
    debug_assert!(current_unix_s >= 0);
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
    debug_assert!(!reason.as_str().is_empty());
    debug_assert!(disposition != RetentionDisposition::Remove || root.removal_requested);
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
    debug_assert!(policy.max_lease_seconds > 0);
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
    debug_assert!(current_unix_s >= lease.last_observed_unix_s);
    let is_lease_too_long = lease
        .expires_unix_s
        .checked_sub(lease.last_observed_unix_s)
        .and_then(|duration| u64::try_from(duration).ok())
        .is_none_or(|duration| duration > policy.max_lease_seconds);
    let is_unsafe = lease.lease_id.is_empty()
        || lease.expires_unix_s < root.created_unix_s
        || lease.renewal_count > policy.max_lease_renewals
        || is_lease_too_long;
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
    debug_assert!(!policy.policy_id.is_empty());
    debug_assert!(decisions.len() <= roots.len());
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
    debug_assert!(!root.path_id.is_empty() && !root.policy_id.is_empty());
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
    debug_assert!(decisions.values().all(|decision| !decision.path_id.is_empty()));
    for pair in objects.windows(2) {
        if pair[0].object_id == pair[1].object_id {
            return Err(UsageError::DuplicateObject {
                object_id: pair[0].object_id.clone(),
            });
        }
    }
    for object in objects.iter_mut() {
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
    debug_assert!(objects.iter().all(|object| !object.object_id.is_empty()));
    Ok(())
}

fn accumulate_usage_object(
    report: &mut UsageReport,
    root_usage: &mut BTreeMap<&str, RootUsage>,
    decisions: &BTreeMap<&str, &RetentionDecision>,
    object: &UsageObjectObservation,
) -> Result<(), UsageError> {
    debug_assert!(!object.object_id.is_empty());
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
    report.observed_bytes = report.observed_bytes.checked_add(bytes).ok_or(UsageError::ByteOverflow)?;
    let class = usage_class(decisions, &object.retaining_root_ids)?;
    debug_assert!(!object.retaining_root_ids.is_empty() || class == UsageClass::Reclaimable);
    match class {
        UsageClass::Retained => {
            report.retained_bytes = report.retained_bytes.checked_add(bytes).ok_or(UsageError::ByteOverflow)?
        }
        UsageClass::Reclaimable => {
            report.reclaimable_bytes = report.reclaimable_bytes.checked_add(bytes).ok_or(UsageError::ByteOverflow)?
        }
        UsageClass::Quarantined => {
            report.quarantined_bytes = report.quarantined_bytes.checked_add(bytes).ok_or(UsageError::ByteOverflow)?
        }
        UsageClass::Unclassified => {
            report.unclassified_bytes = report.unclassified_bytes.checked_add(bytes).ok_or(UsageError::ByteOverflow)?
        }
    }
    if object.retaining_root_ids.len() > 1 {
        report.shared_bytes = report.shared_bytes.checked_add(bytes).ok_or(UsageError::ByteOverflow)?;
    }
    for root_id in &object.retaining_root_ids {
        let usage = root_usage.get_mut(root_id.as_str()).ok_or_else(|| UsageError::UnknownRoot {
            object_id: object.object_id.clone(),
            root_id: root_id.clone(),
        })?;
        usage.inclusive_bytes = usage.inclusive_bytes.checked_add(bytes).ok_or(UsageError::ByteOverflow)?;
        if object.retaining_root_ids.len() == 1 {
            usage.unique_bytes = usage.unique_bytes.checked_add(bytes).ok_or(UsageError::ByteOverflow)?;
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
    debug_assert!(!dispositions.is_empty());
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

pub const RUNTIME_CLOSURE_SCHEMA: &str = "mantle-runtime-closure-explanation-v1";
const RUNTIME_CLOSURE_SNAPSHOT_DOMAIN: &[u8] = b"mantle.runtime.closure.snapshot.v1\0";
pub const MAX_RUNTIME_CLOSURE_NODES_COUNT: u32 = 65_536;
pub const MAX_RUNTIME_CLOSURE_EDGES_COUNT: u32 = 262_144;
pub const MAX_RUNTIME_CLOSURE_DEPTH_COUNT: u32 = 1_024;
pub const MAX_RUNTIME_CLOSURE_WHY_PATHS_COUNT: u32 = 4;
pub const MAX_RUNTIME_CLOSURE_WHY_HOPS_COUNT: u32 = 64;
pub const MAX_RUNTIME_CLOSURE_LARGEST_MEMBERS_COUNT: u32 = 32;
pub const DEFAULT_RUNTIME_CLOSURE_TIME_BUDGET_MS: u64 = 5_000;
pub const DEFAULT_RUNTIME_CLOSURE_TOTAL_NAR_BYTES: u64 = 17_592_186_044_416;
const EXCLUSIVE_BYTES_MEANING: &str = "bytes in this bounded runtime closure snapshot unreachable from the selected root set if this member is removed from the closure graph (not GC-reclaimable disk space)";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeClosureLimits {
    pub max_nodes_count: u32,
    pub max_edges_count: u32,
    pub max_depth_count: u32,
    pub max_total_nar_bytes: u64,
    pub max_why_paths_count: u32,
    pub max_why_hops_count: u32,
    pub max_largest_members_count: u32,
    pub max_elapsed_ms: u64,
}

impl RuntimeClosureLimits {
    #[must_use]
    pub const fn standard() -> Self {
        Self {
            max_nodes_count: MAX_RUNTIME_CLOSURE_NODES_COUNT,
            max_edges_count: MAX_RUNTIME_CLOSURE_EDGES_COUNT,
            max_depth_count: MAX_RUNTIME_CLOSURE_DEPTH_COUNT,
            max_total_nar_bytes: DEFAULT_RUNTIME_CLOSURE_TOTAL_NAR_BYTES,
            max_why_paths_count: MAX_RUNTIME_CLOSURE_WHY_PATHS_COUNT,
            max_why_hops_count: MAX_RUNTIME_CLOSURE_WHY_HOPS_COUNT,
            max_largest_members_count: MAX_RUNTIME_CLOSURE_LARGEST_MEMBERS_COUNT,
            max_elapsed_ms: DEFAULT_RUNTIME_CLOSURE_TIME_BUDGET_MS,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeClosureSourceKind {
    LocalOverlay,
    LocalBase { layer_index: u32 },
    RemoteSubstituter { source_id: String },
    Unavailable,
}

impl RuntimeClosureSourceKind {
    #[must_use]
    pub const fn as_kind_str(&self) -> &'static str {
        match self {
            Self::LocalOverlay => "local-overlay",
            Self::LocalBase { .. } => "local-base",
            Self::RemoteSubstituter { .. } => "remote-substituter",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeClosureTrustStatus {
    Trusted {
        trusted_signature_count: u32,
        total_signature_count: u32,
    },
    Untrusted {
        total_signature_count: u32,
    },
    Unsigned,
    NotVerified {
        reason: String,
    },
}

impl RuntimeClosureTrustStatus {
    #[must_use]
    pub const fn as_kind_str(&self) -> &'static str {
        match self {
            Self::Trusted { .. } => "trusted",
            Self::Untrusted { .. } => "untrusted",
            Self::Unsigned => "unsigned",
            Self::NotVerified { .. } => "not-verified",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeClosureMemberState {
    Observed,
    MissingMetadata,
    LookupFailed,
    IdentityConflict,
    UntrustedSource,
    DepthLimitReached,
    BudgetExhausted,
}

impl RuntimeClosureMemberState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::MissingMetadata => "missing-metadata",
            Self::LookupFailed => "lookup-failed",
            Self::IdentityConflict => "identity-conflict",
            Self::UntrustedSource => "untrusted-source",
            Self::DepthLimitReached => "depth-limit-reached",
            Self::BudgetExhausted => "budget-exhausted",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeClosureMemberInput {
    pub path_id: String,
    pub nar_sha256_hex: Option<String>,
    pub nar_size_bytes: Option<u64>,
    pub size_unavailable_reason: Option<String>,
    pub references: Option<Vec<String>>,
    pub references_unavailable_reason: Option<String>,
    pub source: RuntimeClosureSourceKind,
    pub trust: RuntimeClosureTrustStatus,
    pub state: RuntimeClosureMemberState,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeClosureRequest {
    pub store_prefix: String,
    pub roots: Vec<String>,
    pub limits: RuntimeClosureLimits,
    pub elapsed_ms: u64,
    pub members: Vec<RuntimeClosureMemberInput>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeClosureCompleteness {
    Complete,
    Partial,
    Refused,
}

impl RuntimeClosureCompleteness {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Refused => "refused",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeClosureIssueCode {
    MissingMetadata,
    MissingSize,
    MissingReferences,
    LookupFailed,
    IdentityConflict,
    UntrustedSource,
    NodeLimitReached,
    EdgeLimitReached,
    DepthLimitReached,
    ByteLimitReached,
    TimeLimitReached,
    CycleDetected,
    InvalidStorePrefix,
    InvalidRoot,
}

impl RuntimeClosureIssueCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingMetadata => "missing-metadata",
            Self::MissingSize => "missing-size",
            Self::MissingReferences => "missing-references",
            Self::LookupFailed => "lookup-failed",
            Self::IdentityConflict => "identity-conflict",
            Self::UntrustedSource => "untrusted-source",
            Self::NodeLimitReached => "node-limit-reached",
            Self::EdgeLimitReached => "edge-limit-reached",
            Self::DepthLimitReached => "depth-limit-reached",
            Self::ByteLimitReached => "byte-limit-reached",
            Self::TimeLimitReached => "time-limit-reached",
            Self::CycleDetected => "cycle-detected",
            Self::InvalidStorePrefix => "invalid-store-prefix",
            Self::InvalidRoot => "invalid-root",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RuntimeClosureIssue {
    pub code: RuntimeClosureIssueCode,
    pub path_id: Option<String>,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeClosureWhyPath {
    pub root_id: String,
    pub hops: Vec<String>,
    pub is_truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeClosureMemberReport {
    pub path_id: String,
    pub is_root: bool,
    pub min_depth_count: u32,
    pub nar_sha256_hex: Option<String>,
    pub nar_size_bytes: Option<u64>,
    pub size_unavailable_reason: Option<String>,
    pub closure_inclusive_bytes: Option<u64>,
    pub closure_exclusive_bytes: Option<u64>,
    pub exclusive_bytes_meaning: String,
    pub references: Option<Vec<String>>,
    pub references_unavailable_reason: Option<String>,
    pub why_present: Vec<RuntimeClosureWhyPath>,
    pub is_why_paths_truncated: bool,
    pub source: RuntimeClosureSourceKind,
    pub trust: RuntimeClosureTrustStatus,
    pub state: RuntimeClosureMemberState,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeClosureContributor {
    pub path_id: String,
    pub nar_size_bytes: Option<u64>,
    pub closure_exclusive_bytes: Option<u64>,
    pub min_depth_count: u32,
    pub source: RuntimeClosureSourceKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeClosureExplanation {
    pub schema: String,
    pub snapshot_blake3: String,
    pub store_prefix: String,
    pub roots: Vec<String>,
    pub completeness: RuntimeClosureCompleteness,
    pub is_complete_size_total: bool,
    pub observed_known_nar_bytes: u64,
    pub observed_member_count: u32,
    pub known_size_member_count: u32,
    pub unknown_size_member_count: u32,
    pub unknown_reference_member_count: u32,
    pub examined_edge_count: u32,
    pub has_cycle: bool,
    pub issues: Vec<RuntimeClosureIssue>,
    pub largest_members: Vec<RuntimeClosureContributor>,
    pub members: Vec<RuntimeClosureMemberReport>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeClosureError {
    InvalidLimits,
    EmptyRoots,
    InvalidStorePrefix { prefix: String },
    InvalidPathId { path_id: String },
    RootPrefixMismatch { root_id: String, store_prefix: String },
    DuplicateMember { path_id: String },
    MissingUnavailableReason { path_id: String },
    NumericOverflow,
    IdentityEncodingOverflow,
}

pub fn explain_runtime_closure(
    request: RuntimeClosureRequest,
) -> Result<RuntimeClosureExplanation, RuntimeClosureError> {
    validate_runtime_closure_limits(&request.limits)?;
    validate_store_prefix(&request.store_prefix)?;
    if request.roots.is_empty() {
        return Err(RuntimeClosureError::EmptyRoots);
    }
    let normalized_roots = normalize_closure_roots(&request.store_prefix, &request.roots)?;
    let member_map = index_runtime_members(&request.store_prefix, &request.members)?;
    assert!(!normalized_roots.is_empty());
    assert!(request.limits.max_nodes_count > 0);
    build_runtime_closure_explanation(&request, &normalized_roots, &member_map)
}

fn validate_runtime_closure_limits(limits: &RuntimeClosureLimits) -> Result<(), RuntimeClosureError> {
    let is_nodes_valid = limits.max_nodes_count > 0 && limits.max_nodes_count <= MAX_RUNTIME_CLOSURE_NODES_COUNT;
    let is_edges_valid = limits.max_edges_count > 0 && limits.max_edges_count <= MAX_RUNTIME_CLOSURE_EDGES_COUNT;
    let is_depth_valid = limits.max_depth_count <= MAX_RUNTIME_CLOSURE_DEPTH_COUNT;
    let is_why_valid = limits.max_why_paths_count > 0
        && limits.max_why_paths_count <= MAX_RUNTIME_CLOSURE_WHY_PATHS_COUNT
        && limits.max_why_hops_count > 0
        && limits.max_why_hops_count <= MAX_RUNTIME_CLOSURE_WHY_HOPS_COUNT;
    let is_largest_valid = limits.max_largest_members_count > 0
        && limits.max_largest_members_count <= MAX_RUNTIME_CLOSURE_LARGEST_MEMBERS_COUNT;
    let is_budget_valid = limits.max_total_nar_bytes > 0 && limits.max_elapsed_ms > 0;
    let is_graph_valid = is_nodes_valid && is_edges_valid && is_depth_valid;
    if !is_graph_valid {
        return Err(RuntimeClosureError::InvalidLimits);
    }
    let is_output_valid = is_why_valid && is_largest_valid && is_budget_valid;
    if !is_output_valid {
        return Err(RuntimeClosureError::InvalidLimits);
    }
    debug_assert!(limits.max_nodes_count >= 1);
    debug_assert!(limits.max_edges_count >= 1);
    Ok(())
}

fn validate_store_prefix(store_prefix: &str) -> Result<(), RuntimeClosureError> {
    let is_ascii = store_prefix.is_ascii();
    if !is_ascii {
        return Err(RuntimeClosureError::InvalidStorePrefix {
            prefix: String::from(store_prefix),
        });
    }
    if validate_path_id(store_prefix).is_err() {
        return Err(RuntimeClosureError::InvalidStorePrefix {
            prefix: String::from(store_prefix),
        });
    }
    debug_assert!(store_prefix.is_ascii());
    debug_assert!(store_prefix.starts_with('/'));
    Ok(())
}

struct ClosurePathCheck<'a> {
    store_prefix: &'a str,
    path_id: &'a str,
}

fn validate_closure_path(check: ClosurePathCheck<'_>) -> Result<(), RuntimeClosureError> {
    assert!(check.store_prefix.starts_with('/'));
    if !check.path_id.is_ascii() {
        return Err(RuntimeClosureError::InvalidPathId {
            path_id: String::from(check.path_id),
        });
    }
    if validate_path_id(check.path_id).is_err() {
        return Err(RuntimeClosureError::InvalidPathId {
            path_id: String::from(check.path_id),
        });
    }
    let prefix_slash = format!("{}/", check.store_prefix);
    if !check.path_id.starts_with(&prefix_slash) {
        return Err(RuntimeClosureError::RootPrefixMismatch {
            root_id: String::from(check.path_id),
            store_prefix: String::from(check.store_prefix),
        });
    }
    debug_assert!(check.path_id.is_ascii());
    debug_assert!(check.path_id.len() > check.store_prefix.len());
    Ok(())
}

fn normalize_closure_roots(store_prefix: &str, roots: &[String]) -> Result<Vec<String>, RuntimeClosureError> {
    assert!(!roots.is_empty());
    let max_roots_count =
        usize::try_from(MAX_RUNTIME_CLOSURE_NODES_COUNT).map_err(|_| RuntimeClosureError::NumericOverflow)?;
    if roots.len() > max_roots_count {
        return Err(RuntimeClosureError::InvalidLimits);
    }
    let mut normalized = Vec::with_capacity(roots.len());
    for root in roots {
        validate_closure_path(ClosurePathCheck {
            store_prefix,
            path_id: root,
        })?;
        normalized.push(root.clone());
    }
    normalized.sort();
    normalized.dedup();
    debug_assert!(!normalized.is_empty());
    Ok(normalized)
}

fn index_runtime_members<'a>(
    store_prefix: &str,
    members: &'a [RuntimeClosureMemberInput],
) -> Result<BTreeMap<&'a str, &'a RuntimeClosureMemberInput>, RuntimeClosureError> {
    assert!(store_prefix.starts_with('/'));
    let mut member_map = BTreeMap::new();
    for member in members {
        validate_closure_path(ClosurePathCheck {
            store_prefix,
            path_id: &member.path_id,
        })?;
        if member.nar_size_bytes.is_none() && member.size_unavailable_reason.as_ref().is_none_or(String::is_empty) {
            return Err(RuntimeClosureError::MissingUnavailableReason {
                path_id: member.path_id.clone(),
            });
        }
        if member.references.is_none() && member.references_unavailable_reason.as_ref().is_none_or(String::is_empty) {
            return Err(RuntimeClosureError::MissingUnavailableReason {
                path_id: member.path_id.clone(),
            });
        }
        if let Some(refs) = member.references.as_ref() {
            for reference in refs {
                validate_closure_path(ClosurePathCheck {
                    store_prefix,
                    path_id: reference,
                })?;
            }
        }
        if member_map.insert(member.path_id.as_str(), member).is_some() {
            return Err(RuntimeClosureError::DuplicateMember {
                path_id: member.path_id.clone(),
            });
        }
    }
    debug_assert_eq!(member_map.len(), members.len());
    Ok(member_map)
}

struct ClosureWalkState<'a> {
    root_set: BTreeSet<&'a str>,
    visited_depth: BTreeMap<&'a str, u32>,
    parents: BTreeMap<&'a str, BTreeSet<&'a str>>,
    edges_from: BTreeMap<&'a str, Vec<&'a str>>,
    issues: BTreeSet<RuntimeClosureIssue>,
    examined_edge_count: u32,
    running_nar_bytes: u64,
    is_graph_truncated: bool,
    is_partial: bool,
    is_refused: bool,
}

fn build_runtime_closure_explanation(
    request: &RuntimeClosureRequest,
    roots: &[String],
    member_map: &BTreeMap<&str, &RuntimeClosureMemberInput>,
) -> Result<RuntimeClosureExplanation, RuntimeClosureError> {
    assert!(!roots.is_empty());
    assert!(request.store_prefix.starts_with('/'));
    let mut state = ClosureWalkState {
        root_set: roots.iter().map(String::as_str).collect(),
        visited_depth: BTreeMap::new(),
        parents: BTreeMap::new(),
        edges_from: BTreeMap::new(),
        issues: BTreeSet::new(),
        examined_edge_count: 0,
        running_nar_bytes: 0,
        is_graph_truncated: false,
        is_partial: false,
        is_refused: false,
    };
    let is_elapsed_exceeded = request.elapsed_ms > request.limits.max_elapsed_ms;
    let has_budget_exhausted_member =
        member_map.values().any(|member| member.state == RuntimeClosureMemberState::BudgetExhausted);
    if is_elapsed_exceeded && !has_budget_exhausted_member {
        mark_elapsed_time_limit_reached(request, &mut state);
    } else {
        walk_runtime_closure_bfs(request, roots, member_map, &mut state)?;
        record_post_walk_time_limit_if_missing(request, &mut state, is_elapsed_exceeded);
    }

    let has_cycle = detect_directed_cycle(&state.visited_depth, &state.edges_from)?;
    if has_cycle {
        state.issues.insert(RuntimeClosureIssue {
            code: RuntimeClosureIssueCode::CycleDetected,
            path_id: None,
            detail: "runtime references contain a directed cycle".to_string(),
        });
    }
    assemble_closure_explanation(request, roots, member_map, &state, has_cycle)
}

fn mark_elapsed_time_limit_reached(request: &RuntimeClosureRequest, state: &mut ClosureWalkState<'_>) {
    assert!(request.elapsed_ms > request.limits.max_elapsed_ms);
    state.is_graph_truncated = true;
    state.is_refused = true;
    state.issues.insert(RuntimeClosureIssue {
        code: RuntimeClosureIssueCode::TimeLimitReached,
        path_id: None,
        detail: format!("elapsed time {}ms exceeded budget {}ms", request.elapsed_ms, request.limits.max_elapsed_ms),
    });
    debug_assert!(state.is_refused);
}

fn record_post_walk_time_limit_if_missing(
    request: &RuntimeClosureRequest,
    state: &mut ClosureWalkState<'_>,
    is_elapsed_exceeded: bool,
) {
    assert!(request.limits.max_elapsed_ms > 0);
    let is_issue_recorded = state.issues.iter().any(|issue| issue.code == RuntimeClosureIssueCode::TimeLimitReached);
    if is_elapsed_exceeded && !is_issue_recorded {
        mark_elapsed_time_limit_reached(request, state);
    }
    debug_assert!(!is_elapsed_exceeded || state.is_graph_truncated);
}

fn effective_observed_nar_bytes(member: &RuntimeClosureMemberInput) -> Option<u64> {
    assert!(!member.path_id.is_empty());
    debug_assert!(member.nar_size_bytes.is_some() || member.size_unavailable_reason.is_some());
    if member.state == RuntimeClosureMemberState::Observed {
        member.nar_size_bytes
    } else {
        None
    }
}

fn effective_observed_references(member: &RuntimeClosureMemberInput) -> Option<&Vec<String>> {
    assert!(!member.path_id.is_empty());
    debug_assert!(member.references.is_some() || member.references_unavailable_reason.is_some());
    if member.state == RuntimeClosureMemberState::Observed {
        member.references.as_ref()
    } else {
        None
    }
}

fn member_unavailable_reason(member: &RuntimeClosureMemberInput, explicit: Option<&str>, fallback: &str) -> String {
    assert!(!member.path_id.is_empty());
    assert!(!fallback.is_empty());
    if let Some(reason) = explicit.filter(|text| !text.is_empty()) {
        return String::from(reason);
    }
    if let Some(detail) = member.detail.as_deref().filter(|text| !text.is_empty()) {
        return String::from(detail);
    }
    String::from(fallback)
}

fn walk_runtime_closure_bfs<'a>(
    request: &'a RuntimeClosureRequest,
    roots: &'a [String],
    member_map: &BTreeMap<&'a str, &'a RuntimeClosureMemberInput>,
    state: &mut ClosureWalkState<'a>,
) -> Result<(), RuntimeClosureError> {
    assert!(!roots.is_empty());
    let max_queue_steps =
        request.limits.max_nodes_count.saturating_add(request.limits.max_edges_count).saturating_add(1);
    let mut queue: VecDeque<(&'a str, u32)> = VecDeque::new();
    for root in roots {
        queue.push_back((root.as_str(), 0));
    }

    for _step_index in 0..max_queue_steps {
        let Some((path_id, depth_count)) = queue.pop_front() else {
            break;
        };
        if state.visited_depth.contains_key(path_id) {
            continue;
        }
        if !admit_bfs_node(request, path_id, depth_count, member_map, state)? {
            continue;
        }
        expand_bfs_node_edges(request, BfsExpandTarget { path_id, depth_count }, member_map, state, &mut queue)?;
    }
    debug_assert!(u32::try_from(state.visited_depth.len()).is_ok());
    Ok(())
}

fn admit_bfs_node<'a>(
    request: &RuntimeClosureRequest,
    path_id: &'a str,
    depth_count: u32,
    member_map: &BTreeMap<&'a str, &'a RuntimeClosureMemberInput>,
    state: &mut ClosureWalkState<'a>,
) -> Result<bool, RuntimeClosureError> {
    let current_nodes_count =
        u32::try_from(state.visited_depth.len()).map_err(|_| RuntimeClosureError::NumericOverflow)?;
    if current_nodes_count >= request.limits.max_nodes_count {
        state.is_graph_truncated = true;
        state.is_partial = true;
        state.issues.insert(RuntimeClosureIssue {
            code: RuntimeClosureIssueCode::NodeLimitReached,
            path_id: Some(String::from(path_id)),
            detail: format!("node limit {} reached", request.limits.max_nodes_count),
        });
        return Ok(false);
    }
    let member_bytes = member_map.get(path_id).and_then(|member| effective_observed_nar_bytes(member));
    if let Some(bytes) = member_bytes {
        let next_bytes = state.running_nar_bytes.checked_add(bytes);
        if next_bytes.is_none_or(|total_bytes| total_bytes > request.limits.max_total_nar_bytes) {
            state.is_graph_truncated = true;
            state.is_partial = true;
            state.issues.insert(RuntimeClosureIssue {
                code: RuntimeClosureIssueCode::ByteLimitReached,
                path_id: Some(String::from(path_id)),
                detail: format!("total NAR byte budget {} exceeded", request.limits.max_total_nar_bytes),
            });
            return Ok(false);
        }
        if let Some(total_bytes) = next_bytes {
            state.running_nar_bytes = total_bytes;
        }
    }
    state.visited_depth.insert(path_id, depth_count);
    record_member_observations(path_id, member_map, state);
    debug_assert!(state.visited_depth.contains_key(path_id));
    Ok(true)
}

fn record_member_observations<'a>(
    path_id: &'a str,
    member_map: &BTreeMap<&'a str, &'a RuntimeClosureMemberInput>,
    state: &mut ClosureWalkState<'a>,
) {
    let is_root = state.root_set.contains(path_id);
    let Some(member) = member_map.get(path_id) else {
        state.is_partial = true;
        if is_root {
            state.is_refused = true;
        }
        state.issues.insert(RuntimeClosureIssue {
            code: RuntimeClosureIssueCode::MissingMetadata,
            path_id: Some(String::from(path_id)),
            detail: "no PathInfo metadata observed".to_string(),
        });
        return;
    };
    record_member_state_issue(member, is_root, state);
    if effective_observed_nar_bytes(member).is_none() {
        state.is_partial = true;
        let fallback = if member.state == RuntimeClosureMemberState::Observed {
            "size unavailable"
        } else {
            member.state.as_str()
        };
        let detail = member_unavailable_reason(member, member.size_unavailable_reason.as_deref(), fallback);
        state.issues.insert(RuntimeClosureIssue {
            code: RuntimeClosureIssueCode::MissingSize,
            path_id: Some(member.path_id.clone()),
            detail,
        });
    }
    if effective_observed_references(member).is_none() {
        state.is_partial = true;
        let fallback = if member.state == RuntimeClosureMemberState::Observed {
            "references unavailable"
        } else {
            member.state.as_str()
        };
        let detail = member_unavailable_reason(member, member.references_unavailable_reason.as_deref(), fallback);
        state.issues.insert(RuntimeClosureIssue {
            code: RuntimeClosureIssueCode::MissingReferences,
            path_id: Some(member.path_id.clone()),
            detail,
        });
    }
    debug_assert!(!member.path_id.is_empty());
}

fn push_member_issue(
    state: &mut ClosureWalkState<'_>,
    code: RuntimeClosureIssueCode,
    path_id: &str,
    detail: String,
    is_refused: bool,
) {
    assert!(!path_id.is_empty());
    state.is_partial = true;
    if is_refused {
        state.is_refused = true;
    }
    state.issues.insert(RuntimeClosureIssue {
        code,
        path_id: Some(String::from(path_id)),
        detail,
    });
}

fn record_member_state_issue(member: &RuntimeClosureMemberInput, is_root: bool, state: &mut ClosureWalkState<'_>) {
    assert!(!member.path_id.is_empty());
    let detail = match &member.detail {
        Some(text) => text.clone(),
        None => String::from(member.state.as_str()),
    };
    match member.state {
        RuntimeClosureMemberState::Observed => {}
        RuntimeClosureMemberState::MissingMetadata => {
            push_member_issue(state, RuntimeClosureIssueCode::MissingMetadata, &member.path_id, detail, is_root);
        }
        RuntimeClosureMemberState::LookupFailed => {
            push_member_issue(state, RuntimeClosureIssueCode::LookupFailed, &member.path_id, detail, is_root);
        }
        RuntimeClosureMemberState::IdentityConflict => {
            push_member_issue(state, RuntimeClosureIssueCode::IdentityConflict, &member.path_id, detail, true);
        }
        RuntimeClosureMemberState::UntrustedSource => {
            push_member_issue(state, RuntimeClosureIssueCode::UntrustedSource, &member.path_id, detail, is_root);
        }
        RuntimeClosureMemberState::DepthLimitReached => {
            state.is_graph_truncated = true;
            push_member_issue(state, RuntimeClosureIssueCode::DepthLimitReached, &member.path_id, detail, false);
        }
        RuntimeClosureMemberState::BudgetExhausted => {
            state.is_graph_truncated = true;
            push_member_issue(state, RuntimeClosureIssueCode::TimeLimitReached, &member.path_id, detail, is_root);
        }
    }
    debug_assert!(state.is_partial || member.state == RuntimeClosureMemberState::Observed);
}

struct BfsExpandTarget<'a> {
    path_id: &'a str,
    depth_count: u32,
}

fn expand_bfs_node_edges<'a>(
    request: &RuntimeClosureRequest,
    target_node: BfsExpandTarget<'a>,
    member_map: &BTreeMap<&'a str, &'a RuntimeClosureMemberInput>,
    state: &mut ClosureWalkState<'a>,
    queue: &mut VecDeque<(&'a str, u32)>,
) -> Result<(), RuntimeClosureError> {
    let path_id = target_node.path_id;
    let depth_count = target_node.depth_count;
    let Some(member) = member_map.get(path_id) else {
        return Ok(());
    };
    let Some(refs) = effective_observed_references(member) else {
        return Ok(());
    };
    let mut sorted_refs: Vec<&'a str> = refs.iter().map(String::as_str).collect();
    sorted_refs.sort();
    sorted_refs.dedup();
    if sorted_refs.is_empty() {
        return Ok(());
    }
    if depth_count >= request.limits.max_depth_count {
        state.is_graph_truncated = true;
        state.is_partial = true;
        state.issues.insert(RuntimeClosureIssue {
            code: RuntimeClosureIssueCode::DepthLimitReached,
            path_id: Some(String::from(path_id)),
            detail: format!("depth limit {} reached", request.limits.max_depth_count),
        });
        return Ok(());
    }
    let next_depth = depth_count.checked_add(1).ok_or(RuntimeClosureError::NumericOverflow)?;
    let mut recorded_edges = Vec::with_capacity(sorted_refs.len());
    for target in sorted_refs {
        if state.examined_edge_count >= request.limits.max_edges_count {
            state.is_graph_truncated = true;
            state.is_partial = true;
            state.issues.insert(RuntimeClosureIssue {
                code: RuntimeClosureIssueCode::EdgeLimitReached,
                path_id: Some(String::from(path_id)),
                detail: format!("edge limit {} reached", request.limits.max_edges_count),
            });
            break;
        }
        state.examined_edge_count =
            state.examined_edge_count.checked_add(1).ok_or(RuntimeClosureError::NumericOverflow)?;
        recorded_edges.push(target);
        state.parents.entry(target).or_default().insert(path_id);
        if !state.visited_depth.contains_key(target) {
            queue.push_back((target, next_depth));
        }
    }
    state.edges_from.insert(path_id, recorded_edges);
    debug_assert!(state.examined_edge_count <= request.limits.max_edges_count);
    Ok(())
}

fn detect_directed_cycle(
    visited_depth: &BTreeMap<&str, u32>,
    edges_from: &BTreeMap<&str, Vec<&str>>,
) -> Result<bool, RuntimeClosureError> {
    assert!(u32::try_from(visited_depth.len()).is_ok_and(|nodes_count| nodes_count <= MAX_RUNTIME_CLOSURE_NODES_COUNT));
    // 0 = unvisited, 1 = active on stack, 2 = finished.
    let mut marks: BTreeMap<&str, u8> = visited_depth.keys().map(|&node| (node, 0_u8)).collect();
    let max_dfs_steps = visited_depth
        .len()
        .saturating_mul(2)
        .saturating_add(edges_from.values().map(Vec::len).sum::<usize>())
        .saturating_add(1);
    for &start_node in visited_depth.keys() {
        if marks.get(start_node).copied().unwrap_or(0) != 0 {
            continue;
        }
        let mut stack: Vec<(&str, usize)> = Vec::with_capacity(visited_depth.len());
        stack.push((start_node, 0));
        if let Some(mark) = marks.get_mut(start_node) {
            *mark = 1;
        }
        for _ in 0..max_dfs_steps {
            let Some((node, edge_index)) = stack.pop() else {
                break;
            };
            let neighbors = edges_from.get(node).map_or(&[][..], Vec::as_slice);
            if edge_index >= neighbors.len() {
                if let Some(mark) = marks.get_mut(node) {
                    *mark = 2;
                }
                continue;
            }
            stack.push((node, edge_index.saturating_add(1)));
            let target = neighbors[edge_index];
            match marks.get(target).copied() {
                None | Some(2) => {}
                Some(1) => return Ok(true),
                Some(_) => {
                    if let Some(mark) = marks.get_mut(target) {
                        *mark = 1;
                    }
                    stack.push((target, 0));
                }
            }
        }
    }
    debug_assert!(marks.len() <= visited_depth.len());
    Ok(false)
}

fn assemble_closure_explanation(
    request: &RuntimeClosureRequest,
    roots: &[String],
    member_map: &BTreeMap<&str, &RuntimeClosureMemberInput>,
    state: &ClosureWalkState<'_>,
    has_cycle: bool,
) -> Result<RuntimeClosureExplanation, RuntimeClosureError> {
    let completeness = resolve_completeness(roots, state);
    let mut members_with_bytes_count: u32 = 0;
    let mut members_without_bytes_count: u32 = 0;
    let mut unknown_reference_member_count: u32 = 0;
    let mut member_entries = Vec::with_capacity(state.visited_depth.len());

    for (&path_id, &min_depth_count) in &state.visited_depth {
        let member_entry = build_member_report(MemberReportContext {
            request,
            roots,
            member_map,
            state,
            path_id,
            min_depth_count,
        })?;
        if member_entry.nar_size_bytes.is_some() {
            members_with_bytes_count =
                members_with_bytes_count.checked_add(1).ok_or(RuntimeClosureError::NumericOverflow)?;
        } else {
            members_without_bytes_count =
                members_without_bytes_count.checked_add(1).ok_or(RuntimeClosureError::NumericOverflow)?;
        }
        if member_entry.references.is_none() {
            unknown_reference_member_count =
                unknown_reference_member_count.checked_add(1).ok_or(RuntimeClosureError::NumericOverflow)?;
        }
        member_entries.push(member_entry);
    }

    let observed_member_count =
        u32::try_from(member_entries.len()).map_err(|_| RuntimeClosureError::NumericOverflow)?;
    let has_complete_bytes = completeness == RuntimeClosureCompleteness::Complete
        && members_without_bytes_count == 0
        && unknown_reference_member_count == 0;
    let largest_members = select_largest_members(&member_entries, request.limits.max_largest_members_count)?;
    let issues: Vec<RuntimeClosureIssue> = state.issues.iter().cloned().collect();
    let non_claims = runtime_closure_non_claims();
    let snapshot_blake3 = compute_runtime_closure_snapshot_blake3(SnapshotHashInput {
        request,
        roots,
        completeness,
        is_complete_size_total: has_complete_bytes,
        observed_known_nar_bytes: state.running_nar_bytes,
        has_cycle,
        issues: &issues,
        largest_members: &largest_members,
        members: &member_entries,
    })?;

    debug_assert_eq!(observed_member_count, members_with_bytes_count.saturating_add(members_without_bytes_count));
    Ok(RuntimeClosureExplanation {
        schema: String::from(RUNTIME_CLOSURE_SCHEMA),
        snapshot_blake3,
        store_prefix: request.store_prefix.clone(),
        roots: roots.to_vec(),
        completeness,
        is_complete_size_total: has_complete_bytes,
        observed_known_nar_bytes: state.running_nar_bytes,
        observed_member_count,
        known_size_member_count: members_with_bytes_count,
        unknown_size_member_count: members_without_bytes_count,
        unknown_reference_member_count,
        examined_edge_count: state.examined_edge_count,
        has_cycle,
        issues,
        largest_members,
        members: member_entries,
        non_claims,
    })
}

fn resolve_completeness(roots: &[String], state: &ClosureWalkState<'_>) -> RuntimeClosureCompleteness {
    assert!(!roots.is_empty());
    debug_assert!(u32::try_from(state.visited_depth.len()).is_ok());
    if state.is_refused {
        return RuntimeClosureCompleteness::Refused;
    }
    if state.visited_depth.is_empty() {
        return RuntimeClosureCompleteness::Refused;
    }
    if state.is_partial || state.is_graph_truncated {
        return RuntimeClosureCompleteness::Partial;
    }
    RuntimeClosureCompleteness::Complete
}

struct MemberReportContext<'a> {
    request: &'a RuntimeClosureRequest,
    roots: &'a [String],
    member_map: &'a BTreeMap<&'a str, &'a RuntimeClosureMemberInput>,
    state: &'a ClosureWalkState<'a>,
    path_id: &'a str,
    min_depth_count: u32,
}

fn resolve_missing_field_reason(
    member: &RuntimeClosureMemberInput,
    is_missing: bool,
    explicit: Option<&str>,
    observed_fallback: &str,
) -> Option<String> {
    assert!(!member.path_id.is_empty());
    assert!(!observed_fallback.is_empty());
    if !is_missing {
        return None;
    }
    let fallback = if member.state == RuntimeClosureMemberState::Observed {
        observed_fallback
    } else {
        member.state.as_str()
    };
    Some(member_unavailable_reason(member, explicit, fallback))
}

fn build_member_report(context: MemberReportContext<'_>) -> Result<RuntimeClosureMemberReport, RuntimeClosureError> {
    assert!(!context.roots.is_empty());
    assert!(!context.path_id.is_empty());
    let is_root = context.state.root_set.contains(context.path_id);
    let (why_present, is_why_paths_truncated) = compute_why_present_paths(
        context.path_id,
        &context.state.root_set,
        &context.state.parents,
        &context.request.limits,
    )?;
    let closure_inclusive_bytes = compute_inclusive_bytes(context.path_id, context.member_map, context.state)?;
    let closure_exclusive_bytes =
        compute_exclusive_bytes(context.path_id, context.roots, context.member_map, context.state)?;

    let Some(member) = context.member_map.get(context.path_id) else {
        return Ok(missing_member_report(MissingMemberArgs {
            path_id: context.path_id,
            is_root,
            min_depth_count: context.min_depth_count,
            closure_inclusive_bytes,
            closure_exclusive_bytes,
            why_present,
            is_why_paths_truncated,
        }));
    };
    let is_observed = member.state == RuntimeClosureMemberState::Observed;
    let mut normalized_refs = effective_observed_references(member).cloned();
    if let Some(refs) = normalized_refs.as_mut() {
        refs.sort();
        refs.dedup();
    }
    let nar_sha256_hex = if is_observed {
        member.nar_sha256_hex.clone()
    } else {
        None
    };
    let nar_size_bytes = effective_observed_nar_bytes(member);
    let missing_nar_reason = resolve_missing_field_reason(
        member,
        nar_size_bytes.is_none(),
        member.size_unavailable_reason.as_deref(),
        "size unavailable",
    );
    let missing_refs_reason = resolve_missing_field_reason(
        member,
        normalized_refs.is_none(),
        member.references_unavailable_reason.as_deref(),
        "references unavailable",
    );
    Ok(RuntimeClosureMemberReport {
        path_id: member.path_id.clone(),
        is_root,
        min_depth_count: context.min_depth_count,
        nar_sha256_hex,
        nar_size_bytes,
        size_unavailable_reason: missing_nar_reason,
        closure_inclusive_bytes,
        closure_exclusive_bytes,
        exclusive_bytes_meaning: String::from(EXCLUSIVE_BYTES_MEANING),
        references: normalized_refs,
        references_unavailable_reason: missing_refs_reason,
        why_present,
        is_why_paths_truncated,
        source: member.source.clone(),
        trust: member.trust.clone(),
        state: member.state,
        detail: member.detail.clone(),
    })
}

struct MissingMemberArgs<'a> {
    path_id: &'a str,
    is_root: bool,
    min_depth_count: u32,
    closure_inclusive_bytes: Option<u64>,
    closure_exclusive_bytes: Option<u64>,
    why_present: Vec<RuntimeClosureWhyPath>,
    is_why_paths_truncated: bool,
}

fn missing_member_report(args: MissingMemberArgs<'_>) -> RuntimeClosureMemberReport {
    assert!(!args.path_id.is_empty());
    assert!(
        u32::try_from(args.why_present.len())
            .is_ok_and(|paths_count| paths_count <= MAX_RUNTIME_CLOSURE_WHY_PATHS_COUNT)
    );
    RuntimeClosureMemberReport {
        path_id: String::from(args.path_id),
        is_root: args.is_root,
        min_depth_count: args.min_depth_count,
        nar_sha256_hex: None,
        nar_size_bytes: None,
        size_unavailable_reason: Some("missing-metadata".to_string()),
        closure_inclusive_bytes: args.closure_inclusive_bytes,
        closure_exclusive_bytes: args.closure_exclusive_bytes,
        exclusive_bytes_meaning: String::from(EXCLUSIVE_BYTES_MEANING),
        references: None,
        references_unavailable_reason: Some("missing-metadata".to_string()),
        why_present: args.why_present,
        is_why_paths_truncated: args.is_why_paths_truncated,
        source: RuntimeClosureSourceKind::Unavailable,
        trust: RuntimeClosureTrustStatus::NotVerified {
            reason: "missing-metadata".to_string(),
        },
        state: RuntimeClosureMemberState::MissingMetadata,
        detail: Some("no PathInfo metadata observed".to_string()),
    }
}

fn compute_inclusive_bytes(
    start_id: &str,
    member_map: &BTreeMap<&str, &RuntimeClosureMemberInput>,
    state: &ClosureWalkState<'_>,
) -> Result<Option<u64>, RuntimeClosureError> {
    assert!(!start_id.is_empty());
    if state.is_graph_truncated || state.is_refused {
        return Ok(None);
    }
    let reachable = bfs_reachable_subgraph(&[start_id], None, state);
    let mut total_bytes: u64 = 0;
    for node_id in reachable {
        let Some(member) = member_map.get(node_id) else {
            return Ok(None);
        };
        if effective_observed_references(member).is_none() {
            return Ok(None);
        }
        let Some(bytes) = effective_observed_nar_bytes(member) else {
            return Ok(None);
        };
        total_bytes = total_bytes.checked_add(bytes).ok_or(RuntimeClosureError::NumericOverflow)?;
    }
    debug_assert!(total_bytes <= state.running_nar_bytes || state.running_nar_bytes == 0);
    Ok(Some(total_bytes))
}

fn compute_exclusive_bytes(
    removed_id: &str,
    roots: &[String],
    member_map: &BTreeMap<&str, &RuntimeClosureMemberInput>,
    state: &ClosureWalkState<'_>,
) -> Result<Option<u64>, RuntimeClosureError> {
    assert!(!removed_id.is_empty());
    if state.is_graph_truncated || state.is_refused {
        return Ok(None);
    }
    let has_any_unknown_refs = state
        .visited_depth
        .keys()
        .any(|&node_id| member_map.get(node_id).is_none_or(|member| effective_observed_references(member).is_none()));
    if has_any_unknown_refs {
        return Ok(None);
    }
    let remaining_roots: Vec<&str> =
        roots.iter().map(String::as_str).filter(|&root_id| root_id != removed_id).collect();
    let still_reachable = bfs_reachable_subgraph(&remaining_roots, Some(removed_id), state);
    let mut exclusive_bytes: u64 = 0;
    for &node_id in state.visited_depth.keys() {
        if still_reachable.contains(node_id) {
            continue;
        }
        let Some(member) = member_map.get(node_id) else {
            return Ok(None);
        };
        let Some(bytes) = effective_observed_nar_bytes(member) else {
            return Ok(None);
        };
        exclusive_bytes = exclusive_bytes.checked_add(bytes).ok_or(RuntimeClosureError::NumericOverflow)?;
    }
    debug_assert!(exclusive_bytes <= state.running_nar_bytes || state.running_nar_bytes == 0);
    Ok(Some(exclusive_bytes))
}

fn bfs_reachable_subgraph<'a>(
    starts: &[&'a str],
    excluded: Option<&str>,
    state: &ClosureWalkState<'a>,
) -> BTreeSet<&'a str> {
    assert!(
        u32::try_from(state.visited_depth.len())
            .is_ok_and(|nodes_count| nodes_count <= MAX_RUNTIME_CLOSURE_NODES_COUNT)
    );
    let mut reachable = BTreeSet::new();
    let mut queue = VecDeque::new();
    for &start in starts {
        if excluded != Some(start) && state.visited_depth.contains_key(start) {
            queue.push_back(start);
        }
    }
    let max_steps_count = state.visited_depth.len().saturating_add(1);
    for _ in 0..max_steps_count {
        let Some(node_id) = queue.pop_front() else {
            break;
        };
        if !reachable.insert(node_id) {
            continue;
        }
        let Some(targets) = state.edges_from.get(node_id) else {
            continue;
        };
        for &target in targets {
            if excluded != Some(target) && state.visited_depth.contains_key(target) && !reachable.contains(target) {
                queue.push_back(target);
            }
        }
    }
    debug_assert!(reachable.len() <= state.visited_depth.len());
    reachable
}

fn build_why_path(rev_path: &[&str], is_truncated: bool) -> Result<RuntimeClosureWhyPath, RuntimeClosureError> {
    assert!(!rev_path.is_empty());
    let hops: Vec<String> = rev_path.iter().rev().map(|hop| String::from(*hop)).collect();
    let root_id = hops.first().cloned().ok_or(RuntimeClosureError::NumericOverflow)?;
    Ok(RuntimeClosureWhyPath {
        root_id,
        hops,
        is_truncated,
    })
}

fn compute_why_present_paths(
    target_id: &str,
    root_set: &BTreeSet<&str>,
    parents: &BTreeMap<&str, BTreeSet<&str>>,
    limits: &RuntimeClosureLimits,
) -> Result<(Vec<RuntimeClosureWhyPath>, bool), RuntimeClosureError> {
    assert!(!target_id.is_empty());
    if root_set.contains(target_id) {
        return Ok((vec![build_why_path(&[target_id], false)?], false));
    }
    let max_paths_count =
        usize::try_from(limits.max_why_paths_count).map_err(|_| RuntimeClosureError::NumericOverflow)?;
    let max_hops_count =
        usize::try_from(limits.max_why_hops_count).map_err(|_| RuntimeClosureError::NumericOverflow)?;
    let mut collected: Vec<RuntimeClosureWhyPath> = Vec::with_capacity(max_paths_count);
    let mut is_paths_truncated = false;
    let mut queue: VecDeque<Vec<&str>> = VecDeque::new();
    queue.push_back(vec![target_id]);
    let max_search_steps_count = max_paths_count.saturating_mul(max_hops_count).saturating_mul(16).saturating_add(64);

    for _ in 0..max_search_steps_count {
        let Some(rev_path) = queue.pop_front() else {
            break;
        };
        let head = *rev_path.last().ok_or(RuntimeClosureError::NumericOverflow)?;
        if root_set.contains(head) {
            if collected.len() >= max_paths_count {
                is_paths_truncated = true;
                break;
            }
            collected.push(build_why_path(&rev_path, false)?);
            continue;
        }
        if rev_path.len() >= max_hops_count {
            is_paths_truncated = true;
            if collected.len() < max_paths_count {
                collected.push(build_why_path(&rev_path, true)?);
            }
            continue;
        }
        let Some(node_parents) = parents.get(head) else {
            continue;
        };
        for &parent in node_parents {
            if rev_path.contains(&parent) {
                continue;
            }
            if queue.len() >= max_search_steps_count {
                is_paths_truncated = true;
                break;
            }
            let mut next_path = Vec::with_capacity(rev_path.len().saturating_add(1));
            next_path.extend_from_slice(&rev_path);
            next_path.push(parent);
            queue.push_back(next_path);
        }
    }
    if !queue.is_empty() {
        is_paths_truncated = true;
    }
    collected.sort_by(|left, right| left.root_id.cmp(&right.root_id).then_with(|| left.hops.cmp(&right.hops)));
    debug_assert!(collected.len() <= max_paths_count);
    Ok((collected, is_paths_truncated))
}

fn select_largest_members(
    entries: &[RuntimeClosureMemberReport],
    max_largest_count: u32,
) -> Result<Vec<RuntimeClosureContributor>, RuntimeClosureError> {
    assert!(max_largest_count > 0);
    let max_items_count = usize::try_from(max_largest_count).map_err(|_| RuntimeClosureError::NumericOverflow)?;
    let mut ranked: Vec<&RuntimeClosureMemberReport> = entries.iter().collect();
    ranked.sort_by(|left, right| {
        right.nar_size_bytes.cmp(&left.nar_size_bytes).then_with(|| left.path_id.cmp(&right.path_id))
    });
    let mut result = Vec::with_capacity(max_items_count.min(ranked.len()));
    for member_entry in ranked.into_iter().take(max_items_count) {
        result.push(RuntimeClosureContributor {
            path_id: member_entry.path_id.clone(),
            nar_size_bytes: member_entry.nar_size_bytes,
            closure_exclusive_bytes: member_entry.closure_exclusive_bytes,
            min_depth_count: member_entry.min_depth_count,
            source: member_entry.source.clone(),
        });
    }
    debug_assert!(result.len() <= max_items_count);
    Ok(result)
}

fn runtime_closure_non_claims() -> Vec<String> {
    vec![
        "runtime closure reflects metadata references only, not NAR payload availability or execution trust".to_string(),
        "closure_exclusive_bytes reflects graph removal within this closure snapshot, not GC-reclaimable disk space (which depends on all retained store roots)".to_string(),
        "runtime closure reports PathInfo runtime references only, not build-time or derivation dependency graphs".to_string(),
    ]
}

struct SnapshotHashInput<'a> {
    request: &'a RuntimeClosureRequest,
    roots: &'a [String],
    completeness: RuntimeClosureCompleteness,
    is_complete_size_total: bool,
    observed_known_nar_bytes: u64,
    has_cycle: bool,
    issues: &'a [RuntimeClosureIssue],
    largest_members: &'a [RuntimeClosureContributor],
    members: &'a [RuntimeClosureMemberReport],
}

fn compute_runtime_closure_snapshot_blake3(input: SnapshotHashInput<'_>) -> Result<String, RuntimeClosureError> {
    assert!(!input.roots.is_empty());
    assert!(u32::try_from(input.members.len()).is_ok());
    let mut hasher = blake3::Hasher::new();
    hasher.update(RUNTIME_CLOSURE_SNAPSHOT_DOMAIN);
    hash_closure_str(&mut hasher, &input.request.store_prefix)?;
    hash_closure_u32(
        &mut hasher,
        u32::try_from(input.roots.len()).map_err(|_| RuntimeClosureError::IdentityEncodingOverflow)?,
    );
    for root in input.roots {
        hash_closure_str(&mut hasher, root)?;
    }
    hash_closure_u32(&mut hasher, input.request.limits.max_nodes_count);
    hash_closure_u32(&mut hasher, input.request.limits.max_edges_count);
    hash_closure_u32(&mut hasher, input.request.limits.max_depth_count);
    hasher.update(&input.request.limits.max_total_nar_bytes.to_be_bytes());
    hash_closure_u32(&mut hasher, input.request.limits.max_why_paths_count);
    hash_closure_u32(&mut hasher, input.request.limits.max_why_hops_count);
    hash_closure_u32(&mut hasher, input.request.limits.max_largest_members_count);
    hash_closure_str(&mut hasher, input.completeness.as_str())?;
    hasher.update(&[u8::from(input.is_complete_size_total)]);
    hasher.update(&input.observed_known_nar_bytes.to_be_bytes());
    hasher.update(&[u8::from(input.has_cycle)]);
    hash_closure_u32(
        &mut hasher,
        u32::try_from(input.issues.len()).map_err(|_| RuntimeClosureError::IdentityEncodingOverflow)?,
    );
    for issue in input.issues {
        hash_closure_str(&mut hasher, issue.code.as_str())?;
        hash_closure_opt_str(&mut hasher, issue.path_id.as_deref())?;
        hash_closure_str(&mut hasher, &issue.detail)?;
    }
    hash_largest_members(&mut hasher, input.largest_members)?;
    hash_closure_u32(
        &mut hasher,
        u32::try_from(input.members.len()).map_err(|_| RuntimeClosureError::IdentityEncodingOverflow)?,
    );
    for member in input.members {
        hash_member_report(&mut hasher, member)?;
    }
    let hex_digest = hasher.finalize().to_hex();
    debug_assert_eq!(hex_digest.as_str().len(), 64);
    Ok(format!("b3:{}", hex_digest.as_str()))
}

fn hash_largest_members(
    hasher: &mut blake3::Hasher,
    largest_members: &[RuntimeClosureContributor],
) -> Result<(), RuntimeClosureError> {
    let count = u32::try_from(largest_members.len()).map_err(|_| RuntimeClosureError::IdentityEncodingOverflow)?;
    hash_closure_u32(hasher, count);
    for contributor in largest_members {
        assert!(!contributor.path_id.is_empty());
        hash_closure_str(hasher, &contributor.path_id)?;
        hash_optional_u64(hasher, contributor.nar_size_bytes);
        hash_optional_u64(hasher, contributor.closure_exclusive_bytes);
        hash_closure_u32(hasher, contributor.min_depth_count);
        hash_source_kind(hasher, &contributor.source)?;
    }
    debug_assert!(u32::try_from(largest_members.len()).is_ok());
    Ok(())
}

fn hash_source_kind(hasher: &mut blake3::Hasher, source: &RuntimeClosureSourceKind) -> Result<(), RuntimeClosureError> {
    hash_closure_str(hasher, source.as_kind_str())?;
    match source {
        RuntimeClosureSourceKind::LocalBase { layer_index } => {
            hash_closure_u32(hasher, *layer_index);
        }
        RuntimeClosureSourceKind::RemoteSubstituter { source_id } => {
            hash_closure_str(hasher, source_id)?;
        }
        RuntimeClosureSourceKind::LocalOverlay | RuntimeClosureSourceKind::Unavailable => {}
    }
    debug_assert!(!source.as_kind_str().is_empty());
    Ok(())
}

fn hash_trust_status(
    hasher: &mut blake3::Hasher,
    trust: &RuntimeClosureTrustStatus,
) -> Result<(), RuntimeClosureError> {
    hash_closure_str(hasher, trust.as_kind_str())?;
    match trust {
        RuntimeClosureTrustStatus::Trusted {
            trusted_signature_count,
            total_signature_count,
        } => {
            hash_closure_u32(hasher, *trusted_signature_count);
            hash_closure_u32(hasher, *total_signature_count);
        }
        RuntimeClosureTrustStatus::Untrusted { total_signature_count } => {
            hash_closure_u32(hasher, *total_signature_count);
        }
        RuntimeClosureTrustStatus::Unsigned => {}
        RuntimeClosureTrustStatus::NotVerified { reason } => {
            hash_closure_str(hasher, reason)?;
        }
    }
    debug_assert!(!trust.as_kind_str().is_empty());
    Ok(())
}

fn hash_why_present(
    hasher: &mut blake3::Hasher,
    why_present: &[RuntimeClosureWhyPath],
    is_why_paths_truncated: bool,
) -> Result<(), RuntimeClosureError> {
    let count = u32::try_from(why_present.len()).map_err(|_| RuntimeClosureError::IdentityEncodingOverflow)?;
    hash_closure_u32(hasher, count);
    hasher.update(&[u8::from(is_why_paths_truncated)]);
    for why in why_present {
        assert!(!why.root_id.is_empty());
        hash_closure_str(hasher, &why.root_id)?;
        let hop_count = u32::try_from(why.hops.len()).map_err(|_| RuntimeClosureError::IdentityEncodingOverflow)?;
        hash_closure_u32(hasher, hop_count);
        for hop in &why.hops {
            hash_closure_str(hasher, hop)?;
        }
        hasher.update(&[u8::from(why.is_truncated)]);
    }
    debug_assert!(u32::try_from(why_present.len()).is_ok());
    Ok(())
}

fn hash_member_report(
    hasher: &mut blake3::Hasher,
    member: &RuntimeClosureMemberReport,
) -> Result<(), RuntimeClosureError> {
    assert!(!member.path_id.is_empty());
    hash_closure_str(hasher, &member.path_id)?;
    hasher.update(&[u8::from(member.is_root)]);
    hash_closure_u32(hasher, member.min_depth_count);
    hash_closure_opt_str(hasher, member.nar_sha256_hex.as_deref())?;
    hash_optional_u64(hasher, member.nar_size_bytes);
    hash_closure_str(hasher, member.state.as_str())?;
    hash_source_kind(hasher, &member.source)?;
    hash_trust_status(hasher, &member.trust)?;
    hash_why_present(hasher, &member.why_present, member.is_why_paths_truncated)?;
    match &member.references {
        Some(refs) => {
            hasher.update(&[1]);
            hash_closure_u32(
                hasher,
                u32::try_from(refs.len()).map_err(|_| RuntimeClosureError::IdentityEncodingOverflow)?,
            );
            for reference in refs {
                hash_closure_str(hasher, reference)?;
            }
        }
        None => {
            hasher.update(&[0]);
        }
    }
    debug_assert!(!member.exclusive_bytes_meaning.is_empty());
    Ok(())
}

fn hash_closure_u32(hasher: &mut blake3::Hasher, value: u32) {
    hasher.update(&value.to_be_bytes());
}

fn hash_closure_opt_str(hasher: &mut blake3::Hasher, value: Option<&str>) -> Result<(), RuntimeClosureError> {
    match value {
        Some(inner) => {
            hasher.update(&[1]);
            hash_closure_str(hasher, inner)
        }
        None => {
            hasher.update(&[0]);
            Ok(())
        }
    }
}

fn hash_closure_str(hasher: &mut blake3::Hasher, value: &str) -> Result<(), RuntimeClosureError> {
    hash_string(hasher, value).map_err(|_| RuntimeClosureError::IdentityEncodingOverflow)
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
        let expected = plan_retention(policy(), CURRENT_UNIX_S, roots.clone()).expect("valid retention facts");
        let mut shuffled = roots;
        shuffled.reverse();
        let actual = plan_retention(policy(), CURRENT_UNIX_S, shuffled).expect("equivalent facts");

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
        let plan = plan_retention(policy(), CURRENT_UNIX_S, vec![
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
        let plan = plan_retention(policy(), CURRENT_UNIX_S, vec![malformed]).expect("unsafe lease quarantines");
        assert_eq!(plan.decisions[0].disposition, RetentionDisposition::Quarantine);

        let mut rollback = root(ROOT_B, RootClass::ActiveShellLease);
        rollback.lease = Some(LeaseFacts {
            lease_id: "lease-rollback".to_string(),
            expires_unix_s: ACTIVE_EXPIRY_UNIX_S,
            last_observed_unix_s: ACTIVE_EXPIRY_UNIX_S,
            renewal_count: 1,
        });
        assert!(matches!(
            plan_retention(policy(), CURRENT_UNIX_S, vec![rollback]),
            Err(RetentionError::ClockRollback { .. })
        ));
    }

    #[test]
    fn legacy_root_remains_protected_and_unclassified() {
        let mut legacy = root(ROOT_A, RootClass::LegacyUnmanaged);
        legacy.owner_scope = "legacy-unmanaged".to_string();
        let plan = plan_retention(policy(), CURRENT_UNIX_S, vec![legacy]).expect("legacy root stays protected");
        assert_eq!(plan.decisions[0].disposition, RetentionDisposition::Migrate);
        assert!(plan.decisions[0].disposition.retains_path());
    }

    #[test]
    fn duplicate_roots_and_decision_bounds_are_rejected() {
        let duplicate = root(ROOT_A, RootClass::ExplicitPin);
        let duplicate_error = plan_retention(policy(), CURRENT_UNIX_S, vec![duplicate.clone(), duplicate])
            .expect_err("duplicate root identity must fail");
        assert!(matches!(duplicate_error, RetentionError::DuplicateRoot { .. }));

        let mut bounded_policy = policy();
        bounded_policy.max_decisions = 1;
        let bound_error = plan_retention(bounded_policy, CURRENT_UNIX_S, vec![
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

        let error = plan_retention(policy(), CURRENT_UNIX_S, vec![ineligible])
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
        let plan = plan_retention(one_generation_policy, CURRENT_UNIX_S, vec![
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
            plan_retention(policy(), CURRENT_UNIX_S, vec![unknown_policy]),
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
        let report = aggregate_usage(decisions, vec![
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
        let unknown = aggregate_usage(decisions.clone(), vec![UsageObjectObservation {
            object_id: "unknown".to_string(),
            bytes: None,
            unknown_reason: Some("metadata-unreadable".to_string()),
            retaining_root_ids: vec![ROOT_A.to_string()],
        }])
        .expect("explicit unknown observation");
        assert_eq!(unknown.unknown_object_count, 1);
        assert_eq!(unknown.observed_bytes, 0);

        let overflow = aggregate_usage(decisions, vec![
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

    fn closure_member(path_id: &str, bytes: Option<u64>, refs: Option<&[&str]>) -> RuntimeClosureMemberInput {
        RuntimeClosureMemberInput {
            path_id: path_id.to_string(),
            nar_sha256_hex: Some("aa".repeat(32)),
            nar_size_bytes: bytes,
            size_unavailable_reason: if bytes.is_none() {
                Some("missing-size".to_string())
            } else {
                None
            },
            references: refs.map(|items| items.iter().map(|s| (*s).to_string()).collect()),
            references_unavailable_reason: if refs.is_none() {
                Some("missing-refs".to_string())
            } else {
                None
            },
            source: RuntimeClosureSourceKind::LocalOverlay,
            trust: RuntimeClosureTrustStatus::Trusted {
                trusted_signature_count: 1,
                total_signature_count: 1,
            },
            state: RuntimeClosureMemberState::Observed,
            detail: None,
        }
    }

    #[test]
    fn runtime_closure_chain_diamond_multi_root_and_disconnected_exclusion() {
        let prefix = "/mantle/store";
        let root_a = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-app-a";
        let root_b = "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-app-b";
        let mid_l = "/mantle/store/cccccccccccccccccccccccccccccccc-mid-l";
        let mid_r = "/mantle/store/dddddddddddddddddddddddddddddddd-mid-r";
        let shared = "/mantle/store/eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee-shared";
        let leaf_a = "/mantle/store/ffffffffffffffffffffffffffffffff-leaf-a";
        let unrelated = "/mantle/store/gggggggggggggggggggggggggggggggg-unrelated";

        let members = vec![
            closure_member(root_a, Some(100), Some(&[mid_l, mid_r])),
            closure_member(root_b, Some(200), Some(&[shared])),
            closure_member(mid_l, Some(50), Some(&[shared, leaf_a])),
            closure_member(mid_r, Some(60), Some(&[shared])),
            closure_member(shared, Some(500), Some(&[])),
            closure_member(leaf_a, Some(25), Some(&[])),
            closure_member(unrelated, Some(9999), Some(&[])),
        ];

        let explanation = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![root_b.to_string(), root_a.to_string(), root_a.to_string()],
            limits: RuntimeClosureLimits::standard(),
            elapsed_ms: 10,
            members: members.clone(),
        })
        .expect("valid closure");

        assert_eq!(explanation.completeness, RuntimeClosureCompleteness::Complete);
        assert!(explanation.is_complete_size_total);
        assert!(!explanation.has_cycle);
        assert_eq!(explanation.observed_member_count, 6);
        assert_eq!(explanation.observed_known_nar_bytes, 100 + 200 + 50 + 60 + 500 + 25);

        let shared_report = explanation.members.iter().find(|m| m.path_id == shared).unwrap();
        assert_eq!(shared_report.min_depth_count, 1);
        assert_eq!(shared_report.why_present.len(), 3);

        let root_a_report = explanation.members.iter().find(|m| m.path_id == root_a).unwrap();
        assert_eq!(root_a_report.closure_inclusive_bytes, Some(100 + 50 + 60 + 500 + 25));
        assert_eq!(root_a_report.closure_exclusive_bytes, Some(100 + 50 + 60 + 25));
        assert!(root_a_report.exclusive_bytes_meaning.contains("not GC-reclaimable"));
        assert!(explanation.members.iter().all(|m| m.path_id != unrelated));

        let mut reversed_members = members;
        reversed_members.reverse();
        let repeat = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![root_a.to_string(), root_b.to_string()],
            limits: RuntimeClosureLimits::standard(),
            elapsed_ms: 10,
            members: reversed_members,
        })
        .expect("repeat closure");
        assert_eq!(explanation.snapshot_blake3, repeat.snapshot_blake3);
        assert_eq!(explanation.members, repeat.members);
    }

    #[test]
    fn runtime_closure_preserves_unknowns_detects_cycles_and_invalidates_snapshots() {
        let prefix = "/mantle/store";
        let a = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-a";
        let b = "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-b";
        let unknown = "/mantle/store/cccccccccccccccccccccccccccccccc-unknown";

        let partial = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![a.to_string()],
            limits: RuntimeClosureLimits::standard(),
            elapsed_ms: 5,
            members: vec![
                closure_member(a, Some(10), Some(&[b, unknown])),
                closure_member(b, Some(20), Some(&[a])),
                closure_member(unknown, None, None),
            ],
        })
        .expect("partial cyclic closure");

        assert_eq!(partial.completeness, RuntimeClosureCompleteness::Partial);
        assert!(!partial.is_complete_size_total);
        assert!(partial.has_cycle);
        assert_eq!(partial.observed_known_nar_bytes, 30);
        assert_eq!(partial.unknown_size_member_count, 1);
        assert_eq!(partial.unknown_reference_member_count, 1);
        assert_eq!(partial.members[0].closure_exclusive_bytes, None);

        let complete_a = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![a.to_string()],
            limits: RuntimeClosureLimits::standard(),
            elapsed_ms: 5,
            members: vec![
                closure_member(a, Some(10), Some(&[b])),
                closure_member(b, Some(20), Some(&[])),
            ],
        })
        .unwrap();
        let mutated_b = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![a.to_string()],
            limits: RuntimeClosureLimits::standard(),
            elapsed_ms: 5,
            members: vec![
                closure_member(a, Some(10), Some(&[b])),
                closure_member(b, Some(21), Some(&[])),
            ],
        })
        .unwrap();
        assert_ne!(complete_a.snapshot_blake3, mutated_b.snapshot_blake3);
    }

    #[test]
    fn runtime_closure_enforces_node_edge_depth_byte_and_time_limits() {
        let prefix = "/mantle/store";
        let a = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-a";
        let b = "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-b";
        let c = "/mantle/store/cccccccccccccccccccccccccccccccc-c";
        let members = vec![
            closure_member(a, Some(10), Some(&[b, c])),
            closure_member(b, Some(10), Some(&[c])),
            closure_member(c, Some(10), Some(&[])),
        ];

        let mut node_limits = RuntimeClosureLimits::standard();
        node_limits.max_nodes_count = 1;
        let node_res = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![a.to_string()],
            limits: node_limits,
            elapsed_ms: 1,
            members: members.clone(),
        })
        .unwrap();
        assert_eq!(node_res.completeness, RuntimeClosureCompleteness::Partial);
        assert!(node_res.issues.iter().any(|i| i.code == RuntimeClosureIssueCode::NodeLimitReached));

        let mut edge_limits = RuntimeClosureLimits::standard();
        edge_limits.max_edges_count = 1;
        let edge_res = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![a.to_string()],
            limits: edge_limits,
            elapsed_ms: 1,
            members: members.clone(),
        })
        .unwrap();
        assert!(edge_res.issues.iter().any(|i| i.code == RuntimeClosureIssueCode::EdgeLimitReached));

        let mut depth_limits = RuntimeClosureLimits::standard();
        depth_limits.max_depth_count = 0;
        let depth_res = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![a.to_string()],
            limits: depth_limits,
            elapsed_ms: 1,
            members: members.clone(),
        })
        .unwrap();
        assert!(depth_res.issues.iter().any(|i| i.code == RuntimeClosureIssueCode::DepthLimitReached));

        let mut byte_limits = RuntimeClosureLimits::standard();
        byte_limits.max_total_nar_bytes = 15;
        let byte_res = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![a.to_string()],
            limits: byte_limits,
            elapsed_ms: 1,
            members: members.clone(),
        })
        .unwrap();
        assert!(byte_res.issues.iter().any(|i| i.code == RuntimeClosureIssueCode::ByteLimitReached));

        let mut time_limits = RuntimeClosureLimits::standard();
        time_limits.max_elapsed_ms = 5;
        let time_res = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![a.to_string()],
            limits: time_limits,
            elapsed_ms: 6,
            members,
        })
        .unwrap();
        assert_eq!(time_res.completeness, RuntimeClosureCompleteness::Refused);
        assert!(time_res.issues.iter().any(|i| i.code == RuntimeClosureIssueCode::TimeLimitReached));
    }

    #[test]
    fn explain_runtime_closure_rejects_non_ascii_store_prefix_without_panicking() {
        let non_ascii_prefix = "/mantle/st\u{00f6}re";
        let root = "/mantle/st\u{00f6}re/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-app";
        let err = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: non_ascii_prefix.to_string(),
            roots: vec![root.to_string()],
            limits: RuntimeClosureLimits::standard(),
            elapsed_ms: 1,
            members: vec![closure_member(root, Some(10), Some(&[]))],
        })
        .unwrap_err();
        assert_eq!(err, RuntimeClosureError::InvalidStorePrefix {
            prefix: non_ascii_prefix.to_string(),
        });

        let valid_prefix = "/mantle/store";
        let non_ascii_root = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-app-\u{00fc}";
        let path_err = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: valid_prefix.to_string(),
            roots: vec![non_ascii_root.to_string()],
            limits: RuntimeClosureLimits::standard(),
            elapsed_ms: 1,
            members: vec![],
        })
        .unwrap_err();
        assert_eq!(path_err, RuntimeClosureError::InvalidPathId {
            path_id: non_ascii_root.to_string(),
        });
    }

    #[test]
    fn explain_runtime_closure_untrusted_u64_max_is_not_masked_by_byte_limit() {
        let prefix = "/mantle/store";
        let root = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-root";
        let untrusted = "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-untrusted";

        let mut untrusted_root_member = closure_member(root, Some(u64::MAX), Some(&[]));
        untrusted_root_member.state = RuntimeClosureMemberState::UntrustedSource;
        untrusted_root_member.source = RuntimeClosureSourceKind::RemoteSubstituter {
            source_id: "https://cache.example.com".to_string(),
        };
        untrusted_root_member.trust = RuntimeClosureTrustStatus::Untrusted {
            total_signature_count: 1,
        };
        untrusted_root_member.detail = Some("untrusted root signature".to_string());

        let refused_root = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![root.to_string()],
            limits: RuntimeClosureLimits::standard(),
            elapsed_ms: 1,
            members: vec![untrusted_root_member],
        })
        .unwrap();
        assert_eq!(refused_root.completeness, RuntimeClosureCompleteness::Refused);
        assert_eq!(refused_root.observed_known_nar_bytes, 0);
        assert_eq!(refused_root.observed_member_count, 1);
        assert!(refused_root.members[0].nar_size_bytes.is_none());
        assert!(refused_root.members[0].nar_sha256_hex.is_none());
        assert!(refused_root.issues.iter().any(|i| i.code == RuntimeClosureIssueCode::UntrustedSource));
        assert!(!refused_root.issues.iter().any(|i| i.code == RuntimeClosureIssueCode::ByteLimitReached));

        let mut untrusted_child = closure_member(untrusted, Some(u64::MAX), Some(&[]));
        untrusted_child.state = RuntimeClosureMemberState::UntrustedSource;
        untrusted_child.source = RuntimeClosureSourceKind::RemoteSubstituter {
            source_id: "https://cache.example.com".to_string(),
        };
        untrusted_child.trust = RuntimeClosureTrustStatus::Untrusted {
            total_signature_count: 1,
        };
        untrusted_child.detail = Some("untrusted child signature".to_string());

        let partial_child = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![root.to_string()],
            limits: RuntimeClosureLimits::standard(),
            elapsed_ms: 1,
            members: vec![closure_member(root, Some(100), Some(&[untrusted])), untrusted_child],
        })
        .unwrap();
        assert_eq!(partial_child.completeness, RuntimeClosureCompleteness::Partial);
        assert_eq!(partial_child.observed_known_nar_bytes, 100);
        assert_eq!(partial_child.observed_member_count, 2);
        assert_eq!(partial_child.known_size_member_count, 1);
        assert_eq!(partial_child.unknown_size_member_count, 1);
        assert!(partial_child.issues.iter().any(|i| i.code == RuntimeClosureIssueCode::UntrustedSource));
        assert!(!partial_child.issues.iter().any(|i| i.code == RuntimeClosureIssueCode::ByteLimitReached));
    }

    #[test]
    fn explain_runtime_closure_valid_root_exceeding_byte_budget_is_refused() {
        let prefix = "/mantle/store";
        let root = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-oversized-root";
        let mut limits = RuntimeClosureLimits::standard();
        limits.max_total_nar_bytes = 50;

        let res = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![root.to_string()],
            limits,
            elapsed_ms: 1,
            members: vec![closure_member(root, Some(100), Some(&[]))],
        })
        .unwrap();
        assert_eq!(res.completeness, RuntimeClosureCompleteness::Refused);
        assert_eq!(res.observed_member_count, 0);
        assert_eq!(res.observed_known_nar_bytes, 0);
        assert!(res.issues.iter().any(|i| i.code == RuntimeClosureIssueCode::ByteLimitReached));
    }

    #[test]
    fn explain_runtime_closure_snapshot_blake3_covers_trust_counts_why_limits_and_largest_members() {
        let prefix = "/mantle/store";
        let root_a = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-root-a";
        let root_b = "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-root-b";
        let shared = "/mantle/store/cccccccccccccccccccccccccccccccc-shared";

        let base_members = vec![
            closure_member(root_a, Some(10), Some(&[shared])),
            closure_member(root_b, Some(20), Some(&[shared])),
            closure_member(shared, Some(30), Some(&[])),
        ];
        let base = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![root_a.to_string(), root_b.to_string()],
            limits: RuntimeClosureLimits::standard(),
            elapsed_ms: 1,
            members: base_members.clone(),
        })
        .unwrap();

        let mut trust_count_changed = base_members.clone();
        trust_count_changed[0].trust = RuntimeClosureTrustStatus::Trusted {
            trusted_signature_count: 2,
            total_signature_count: 5,
        };
        let with_trust_count = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![root_a.to_string(), root_b.to_string()],
            limits: RuntimeClosureLimits::standard(),
            elapsed_ms: 1,
            members: trust_count_changed,
        })
        .unwrap();
        assert_ne!(base.snapshot_blake3, with_trust_count.snapshot_blake3);

        let mut why_limit = RuntimeClosureLimits::standard();
        why_limit.max_why_paths_count = 1;
        let with_why_limit = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![root_a.to_string(), root_b.to_string()],
            limits: why_limit,
            elapsed_ms: 1,
            members: base_members.clone(),
        })
        .unwrap();
        assert_ne!(base.snapshot_blake3, with_why_limit.snapshot_blake3);

        let mut largest_limit = RuntimeClosureLimits::standard();
        largest_limit.max_largest_members_count = 1;
        let with_largest_limit = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![root_a.to_string(), root_b.to_string()],
            limits: largest_limit,
            elapsed_ms: 1,
            members: base_members,
        })
        .unwrap();
        assert_ne!(base.snapshot_blake3, with_largest_limit.snapshot_blake3);
    }

    #[test]
    fn explain_runtime_closure_mid_walk_budget_exhausted_preserves_observed_members_and_refuses_root_timeout() {
        let prefix = "/mantle/store";
        let root = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-root";
        let child = "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-child";

        let mut budget_child = closure_member(child, None, None);
        budget_child.state = RuntimeClosureMemberState::BudgetExhausted;
        budget_child.source = RuntimeClosureSourceKind::Unavailable;
        budget_child.trust = RuntimeClosureTrustStatus::NotVerified {
            reason: "budget-exhausted".to_string(),
        };
        budget_child.detail = Some("elapsed time exceeded 5ms".to_string());

        let mut limits = RuntimeClosureLimits::standard();
        limits.max_elapsed_ms = 5;

        let mid_walk = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![root.to_string()],
            limits,
            elapsed_ms: 12,
            members: vec![closure_member(root, Some(40), Some(&[child])), budget_child],
        })
        .unwrap();
        assert_eq!(mid_walk.completeness, RuntimeClosureCompleteness::Partial);
        assert_eq!(mid_walk.observed_member_count, 2);
        assert_eq!(mid_walk.observed_known_nar_bytes, 40);
        assert!(mid_walk.issues.iter().any(|i| i.code == RuntimeClosureIssueCode::TimeLimitReached));

        let mut budget_root = closure_member(root, None, None);
        budget_root.state = RuntimeClosureMemberState::BudgetExhausted;
        budget_root.source = RuntimeClosureSourceKind::Unavailable;
        budget_root.trust = RuntimeClosureTrustStatus::NotVerified {
            reason: "budget-exhausted".to_string(),
        };
        budget_root.detail = Some("elapsed time exceeded 5ms".to_string());

        let root_timeout = explain_runtime_closure(RuntimeClosureRequest {
            store_prefix: prefix.to_string(),
            roots: vec![root.to_string()],
            limits,
            elapsed_ms: 12,
            members: vec![budget_root],
        })
        .unwrap();
        assert_eq!(root_timeout.completeness, RuntimeClosureCompleteness::Refused);
        assert!(root_timeout.issues.iter().any(|i| i.code == RuntimeClosureIssueCode::TimeLimitReached));
    }
}
