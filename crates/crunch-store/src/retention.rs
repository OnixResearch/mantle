use std::collections::BTreeMap;
use std::sync::OnceLock;

use crunch_gc_core::retention::LeaseFacts;
use crunch_gc_core::retention::RetentionPolicy;
use crunch_gc_core::retention::RetentionRoot;
use crunch_gc_core::retention::RootClass;
use data_encoding::HEXLOWER;
use serde::Deserialize;
use serde::Serialize;

use crate::Error;
use crate::roots::GcRootRecord;

const STORE_RETENTION_POLICY_SCHEMA: &str = "mantle-store-retention-policy-v1";
const STORE_RETENTION_POLICY_JSON: &str =
    include_str!("../../../config/store-retention/generated/store-retention-policy.json");
static STORE_RETENTION_RUNTIME_POLICY: OnceLock<StoreRetentionRuntimePolicy> = OnceLock::new();

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct StoreRetentionRuntimePolicy {
    pub schema: String,
    pub policy_name: String,
    pub hash_algorithm: String,
    pub limits: StoreRetentionLimits,
    pub rules: StoreRetentionRules,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct StoreRetentionRules {
    pub explicit_pin: StoreRetentionRule,
    pub project_output_generation: StoreRetentionRule,
    pub project_source_generation: StoreRetentionRule,
    pub active_shell_lease: StoreRetentionRule,
    pub bootstrap: StoreRetentionRule,
    pub self_build: StoreRetentionRule,
    pub remote_result: StoreRetentionRule,
    pub legacy_unmanaged: StoreRetentionRule,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct StoreRetentionRule {
    pub eligible_owner_scopes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct StoreRetentionLimits {
    pub max_roots: usize,
    pub max_decisions: usize,
    pub max_usage_objects: usize,
    pub max_roots_per_object: usize,
    pub retained_project_output_generations: usize,
    pub retained_project_source_generations: usize,
    pub max_lease_renewals: u32,
    pub default_lease_seconds: u64,
    pub max_lease_seconds: u64,
    pub max_closure_links_per_explanation: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GcRootClass {
    ExplicitPin,
    ProjectOutputGeneration,
    ProjectSourceGeneration,
    ActiveShellLease,
    Bootstrap,
    SelfBuild,
    RemoteResult,
    LegacyUnmanaged,
}

impl GcRootClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.into_core().as_str()
    }

    #[must_use]
    pub const fn into_core(self) -> RootClass {
        match self {
            Self::ExplicitPin => RootClass::ExplicitPin,
            Self::ProjectOutputGeneration => RootClass::ProjectOutputGeneration,
            Self::ProjectSourceGeneration => RootClass::ProjectSourceGeneration,
            Self::ActiveShellLease => RootClass::ActiveShellLease,
            Self::Bootstrap => RootClass::Bootstrap,
            Self::SelfBuild => RootClass::SelfBuild,
            Self::RemoteResult => RootClass::RemoteResult,
            Self::LegacyUnmanaged => RootClass::LegacyUnmanaged,
        }
    }
}

impl std::fmt::Display for GcRootClass {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcRootLease {
    pub lease_id: String,
    pub expires_unix_s: i64,
    pub last_observed_unix_s: i64,
    pub renewal_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootRegistration {
    pub class: GcRootClass,
    pub owner_scope: String,
    pub project_identity: Option<String>,
    pub selector: Option<String>,
    pub generation: Option<u64>,
    pub generation_identity: Option<String>,
    pub lease: Option<GcRootLease>,
    pub removal_requested: bool,
}

#[must_use]
pub fn store_retention_runtime_policy() -> &'static StoreRetentionRuntimePolicy {
    STORE_RETENTION_RUNTIME_POLICY.get_or_init(|| {
        let parsed_policy: Result<StoreRetentionRuntimePolicy, serde_json::Error> =
            serde_json::from_str(STORE_RETENTION_POLICY_JSON);
        assert!(parsed_policy.is_ok(), "embedded store retention policy must parse");
        let Ok(policy) = parsed_policy else {
            std::process::abort();
        };
        assert_eq!(policy.schema, STORE_RETENTION_POLICY_SCHEMA);
        assert_eq!(policy.hash_algorithm, "BLAKE3");
        assert!(!policy.policy_name.is_empty());
        assert!(policy.limits.max_roots > 0);
        assert!(policy.limits.max_decisions > 0);
        assert!(policy.limits.max_usage_objects > 0);
        assert!(policy.limits.max_roots_per_object > 0);
        assert!(policy.limits.default_lease_seconds > 0);
        assert!(policy.limits.default_lease_seconds <= policy.limits.max_lease_seconds);
        let owner_scope_rules = core_owner_scope_rules(&policy.rules);
        assert!(
            RootClass::ALL
                .into_iter()
                .all(|class| owner_scope_rules.get(&class).is_some_and(|scopes| !scopes.is_empty()))
        );
        policy
    })
}

#[must_use]
pub fn store_retention_policy_blake3() -> String {
    let digest = blake3::hash(STORE_RETENTION_POLICY_JSON.as_bytes());
    format!("b3:{}", HEXLOWER.encode(digest.as_bytes()))
}

#[must_use]
pub fn core_retention_policy() -> RetentionPolicy {
    let policy = store_retention_runtime_policy();
    RetentionPolicy {
        policy_id: store_retention_policy_blake3(),
        eligible_owner_scopes: core_owner_scope_rules(&policy.rules),
        max_roots: policy.limits.max_roots,
        max_decisions: policy.limits.max_decisions,
        retained_project_output_generations: policy.limits.retained_project_output_generations,
        retained_project_source_generations: policy.limits.retained_project_source_generations,
        max_lease_renewals: policy.limits.max_lease_renewals,
        max_lease_seconds: policy.limits.max_lease_seconds,
    }
}

fn core_owner_scope_rules(rules: &StoreRetentionRules) -> BTreeMap<RootClass, Vec<String>> {
    BTreeMap::from([
        (RootClass::ExplicitPin, rules.explicit_pin.eligible_owner_scopes.clone()),
        (RootClass::ProjectOutputGeneration, rules.project_output_generation.eligible_owner_scopes.clone()),
        (RootClass::ProjectSourceGeneration, rules.project_source_generation.eligible_owner_scopes.clone()),
        (RootClass::ActiveShellLease, rules.active_shell_lease.eligible_owner_scopes.clone()),
        (RootClass::Bootstrap, rules.bootstrap.eligible_owner_scopes.clone()),
        (RootClass::SelfBuild, rules.self_build.eligible_owner_scopes.clone()),
        (RootClass::RemoteResult, rules.remote_result.eligible_owner_scopes.clone()),
        (RootClass::LegacyUnmanaged, rules.legacy_unmanaged.eligible_owner_scopes.clone()),
    ])
}

pub fn records_to_core(records: &[GcRootRecord]) -> Result<Vec<RetentionRoot>, Error> {
    let current_policy = store_retention_policy_blake3();
    if records.len() > store_retention_runtime_policy().limits.max_roots {
        return Err(Error::RootRegistry(format!(
            "root count exceeds retention policy limit {}",
            store_retention_runtime_policy().limits.max_roots
        )));
    }
    records.iter().map(|record| record_to_core(record, &current_policy)).collect()
}

fn record_to_core(record: &GcRootRecord, current_policy: &str) -> Result<RetentionRoot, Error> {
    if record.policy_blake3 != current_policy {
        return Err(Error::RootRegistry(format!(
            "root policy identity is stale for {}: expected {current_policy}, observed {}",
            record.logical_path, record.policy_blake3
        )));
    }
    let lease = record.lease.as_ref().map(|lease| LeaseFacts {
        lease_id: lease.lease_id.clone(),
        expires_unix_s: lease.expires_unix_s,
        last_observed_unix_s: lease.last_observed_unix_s,
        renewal_count: lease.renewal_count,
    });
    assert_eq!(record.policy_blake3, current_policy);
    assert_eq!(lease.is_some(), record.lease.is_some());
    Ok(RetentionRoot {
        path_id: record.logical_path.clone(),
        class: record.root_class.into_core(),
        owner_scope: record.owner_scope.clone(),
        project_identity: record.project_identity.clone(),
        selector: record.selector.clone(),
        generation: record.generation,
        generation_identity: record.generation_identity.clone(),
        lease,
        policy_id: record.policy_blake3.clone(),
        created_unix_s: record.created_unix_s,
        last_transition_id: record.last_transition_id.clone(),
        removal_requested: record.removal_requested,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_policy_is_typed_and_bounded() {
        let policy = store_retention_runtime_policy();
        assert_eq!(policy.schema, STORE_RETENTION_POLICY_SCHEMA);
        assert_eq!(policy.hash_algorithm, "BLAKE3");
        assert!(policy.limits.max_roots <= crunch_gc_core::retention::MAX_RETENTION_ROOTS);
        assert!(policy.limits.max_usage_objects <= crunch_gc_core::retention::MAX_USAGE_OBJECTS);
        assert!(policy.limits.max_roots_per_object <= crunch_gc_core::retention::MAX_USAGE_ROOTS_PER_OBJECT);
        assert!(store_retention_policy_blake3().starts_with("b3:"));
        let core = core_retention_policy();
        assert_eq!(core.eligible_owner_scopes.len(), RootClass::ALL.len());
        assert!(
            core.eligible_owner_scopes
                .get(&RootClass::LegacyUnmanaged)
                .is_some_and(|scopes| scopes == &["legacy-unmanaged"])
        );
    }
}
