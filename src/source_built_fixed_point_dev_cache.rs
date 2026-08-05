//! Pure dev-cache and snapshot core for the source-built Mantle fixed-point proof.
//!
//! The promoted cold proof never reads this module's state. Every dev-cache
//! decision, key, and validation in this file is a pure function over in-memory
//! values. The imperative proof shell supplies observations (parsed cache
//! entries, stage markers, digests) and performs all file I/O.
//!
//! Trust rule repeated here because it is the whole point: a dev-cache hit is
//! visibly a hit, is keyed and receipt-validated against the exact current
//! source-authority and policy digests, and never authorizes a promoted or
//! release claim.

use serde::Deserialize;
use serde::Serialize;

use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;

pub(crate) const DEV_CACHE_SCHEMA: &str = "mantle-source-built-fixed-point-dev-cache-v1";
pub(crate) const STAGE_MARKER_SCHEMA: &str = "mantle-source-built-fixed-point-stage-marker-v1";
pub(crate) const FAST_FAIL_SCHEMA: &str = "mantle-source-built-fixed-point-fast-fail-v1";
pub(crate) const DEV_CACHE_KEY_CONTEXT: &str = "mantle-source-built-fixed-point-dev-cache-key-v1";
pub(crate) const DEV_CACHE_ENTRY_FILE: &str = "entry.json";
const BLAKE3_HEX_LENGTH: usize = 64;
const POLICY_DIGEST_COUNT: usize = 5;

/// The dev cache policy digests that, together with the source-authority digest,
/// key every cached provider and store snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DevCachePolicies {
    pub(crate) closure_policy_digest_blake3: String,
    pub(crate) hermeticity_policy_digest_blake3: String,
    pub(crate) protected_execution_policy_digest_blake3: String,
    pub(crate) effect_policy_digest_blake3: String,
    pub(crate) normalization_policy_digest_blake3: String,
}

impl DevCachePolicies {
    /// Extract the five policy digests from the current immutable plan.
    pub(crate) fn from_plan(plan: &SourceBuiltFixedPointPlan) -> Self {
        let policies = &plan.policies;
        Self {
            closure_policy_digest_blake3: policies.closure_policy_digest_blake3.clone(),
            hermeticity_policy_digest_blake3: policies.hermeticity_policy_digest_blake3.clone(),
            protected_execution_policy_digest_blake3: policies.protected_execution_policy_digest_blake3.clone(),
            effect_policy_digest_blake3: policies.effect_policy_digest_blake3.clone(),
            normalization_policy_digest_blake3: policies.normalization_policy_digest_blake3.clone(),
        }
    }

    /// Ordered stable list used for serialization and keying.
    pub(crate) fn ordered_digests(&self) -> [&str; POLICY_DIGEST_COUNT] {
        [
            &self.closure_policy_digest_blake3,
            &self.hermeticity_policy_digest_blake3,
            &self.protected_execution_policy_digest_blake3,
            &self.effect_policy_digest_blake3,
            &self.normalization_policy_digest_blake3,
        ]
    }
}

/// A content-reference to one adopted provider store path inside the dev cache.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DevProviderReference {
    pub(crate) logical_path: String,
    pub(crate) store_basename: String,
    pub(crate) digest_blake3: String,
}

/// A receipt-bound provider-output cache entry.
///
/// Membership re-validates against the exact current source-authority and
/// policy digests. Any missing, stale, or mismatched field is a hard miss.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DevProviderCacheEntry {
    pub(crate) schema: String,
    pub(crate) plan_digest_blake3: String,
    pub(crate) cache_key: String,
    pub(crate) source_authority_digest_blake3: String,
    pub(crate) closure_policy_digest_blake3: String,
    pub(crate) hermeticity_policy_digest_blake3: String,
    pub(crate) protected_execution_policy_digest_blake3: String,
    pub(crate) effect_policy_digest_blake3: String,
    pub(crate) normalization_policy_digest_blake3: String,
    pub(crate) stagex_provider: DevProviderReference,
    pub(crate) native_provider: DevProviderReference,
}

/// A per-stage, per-run completion marker recorded into the staging directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StageCompletionMarker {
    pub(crate) schema: String,
    pub(crate) plan_digest_blake3: String,
    pub(crate) stage_id: String,
    pub(crate) digest_blake3: String,
}

/// Result of a dev provider-cache lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DevProviderCacheLookup {
    /// A receipt-validated entry matches the current digests exactly.
    Hit,
    /// No cache flag, no entry, or any stale/mismatched digest field.
    Miss,
}

/// Result of validating one saved stage marker against a fresh source replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StageMarkerValidation {
    /// The marker's recorded digest matches the fresh replay for this plan.
    Trusted,
    /// The marker is missing, belongs to another plan, or is mutated/stale.
    Reject,
}

/// Outcome of the dev fast-fail baseline check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FastFailDecision {
    /// The current source profile equals a prior published successful fixed-point
    /// binding; report that exact prior digest instead of a fresh rebuild.
    ReportPriorSuccess {
        matched_source_authority_digest_blake3: String,
    },
    /// The profile changed or no published binding exists; proceed with the run.
    Proceed,
}

/// Compute the dev provider-cache key from the current source-authority digest
/// and the five policy digests. The key is stable across input order because it
/// feeds only these digests in a fixed order.
pub(crate) fn dev_provider_cache_key(source_authority_digest_blake3: &str, policies: &DevCachePolicies) -> String {
    debug_assert!(is_lower_hex_digest(source_authority_digest_blake3));
    let mut payload = Vec::with_capacity((BLAKE3_HEX_LENGTH + 1) * (POLICY_DIGEST_COUNT + 1));
    payload.extend_from_slice(source_authority_digest_blake3.as_bytes());
    payload.push(b'\n');
    for digest in policies.ordered_digests() {
        debug_assert!(is_lower_hex_digest(digest));
        payload.extend_from_slice(digest.as_bytes());
        payload.push(b'\n');
    }
    let mut hasher = blake3::Hasher::new_derive_key(DEV_CACHE_KEY_CONTEXT);
    hasher.update(&payload);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert!(!payload.is_empty());
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    digest
}

/// Decide whether a parsed (possibly absent) entry is a validated hit. A hit
/// requires the entry to exist with the correct schema, plan digest, computed
/// cache key, and every source-authority/policy digest matching the current plan.
pub(crate) fn evaluate_provider_cache_lookup(
    entry: Option<&DevProviderCacheEntry>,
    plan: &SourceBuiltFixedPointPlan,
    expected_policies: &DevCachePolicies,
) -> DevProviderCacheLookup {
    let Some(entry) = entry else {
        return DevProviderCacheLookup::Miss;
    };
    let expected_key = dev_provider_cache_key(&plan.source_authority_digest_blake3, expected_policies);
    let fields_match = entry.schema == DEV_CACHE_SCHEMA
        && entry.plan_digest_blake3 == plan.plan_digest_blake3
        && entry.cache_key == expected_key
        && entry.source_authority_digest_blake3 == plan.source_authority_digest_blake3
        && entry.closure_policy_digest_blake3 == expected_policies.closure_policy_digest_blake3
        && entry.hermeticity_policy_digest_blake3 == expected_policies.hermeticity_policy_digest_blake3
        && entry.protected_execution_policy_digest_blake3 == expected_policies.protected_execution_policy_digest_blake3
        && entry.effect_policy_digest_blake3 == expected_policies.effect_policy_digest_blake3
        && entry.normalization_policy_digest_blake3 == expected_policies.normalization_policy_digest_blake3
        && is_lower_hex_digest(&entry.stagex_provider.digest_blake3)
        && entry.native_provider.digest_blake3 == plan.policies.expected_native_provider_digest_blake3;
    if fields_match {
        DevProviderCacheLookup::Hit
    } else {
        DevProviderCacheLookup::Miss
    }
}

/// Validate one saved stage marker against the current plan and a fresh replay
/// digest for that stage. Only an exact match of schema, plan, stage id, and
/// digest is trusted; anything else forces that stage to restart.
pub(crate) fn validate_stage_marker(
    marker: Option<&StageCompletionMarker>,
    plan_digest_blake3: &str,
    stage_id: &str,
    fresh_stage_digest_blake3: &str,
) -> StageMarkerValidation {
    let Some(marker) = marker else {
        return StageMarkerValidation::Reject;
    };
    let trusted = marker.schema == STAGE_MARKER_SCHEMA
        && marker.plan_digest_blake3 == plan_digest_blake3
        && marker.stage_id == stage_id
        && marker.digest_blake3 == fresh_stage_digest_blake3;
    if trusted {
        StageMarkerValidation::Trusted
    } else {
        StageMarkerValidation::Reject
    }
}

/// Decide the fast-fail baseline outcome from the current source profile digest
/// and the last published successful fixed-point source binding.
pub(crate) fn evaluate_fast_fail(
    current_source_authority_digest_blake3: &str,
    last_published_source_authority_digest_blake3: Option<&str>,
) -> FastFailDecision {
    debug_assert!(is_lower_hex_digest(current_source_authority_digest_blake3));
    match last_published_source_authority_digest_blake3 {
        Some(prior) if prior == current_source_authority_digest_blake3 => FastFailDecision::ReportPriorSuccess {
            matched_source_authority_digest_blake3: prior.to_string(),
        },
        _ => FastFailDecision::Proceed,
    }
}

/// Policy digest helper used only by tests. Kept out of the non-test build to
/// avoid dead-code warnings under `-D warnings`.
#[cfg(test)]
pub(crate) fn ordered_policy_digest(role: crate::source_built_fixed_point::ProofPolicyRole) -> String {
    let (context, index): (&[u8], u8) = match role {
        crate::source_built_fixed_point::ProofPolicyRole::Closure => (b"closure\0", 1u8),
        crate::source_built_fixed_point::ProofPolicyRole::Hermeticity => (b"hermeticity\0", 2u8),
        crate::source_built_fixed_point::ProofPolicyRole::ProtectedExecution => (b"protected-execution\0", 3u8),
        crate::source_built_fixed_point::ProofPolicyRole::Effect => (b"effect\0", 4u8),
        crate::source_built_fixed_point::ProofPolicyRole::Normalization => (b"normalization\0", 5u8),
    };
    let mut hasher = blake3::Hasher::new_derive_key(DEV_CACHE_KEY_CONTEXT);
    hasher.update(context);
    hasher.update(&[index]);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    digest
}

fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const OTHER_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn policies() -> DevCachePolicies {
        DevCachePolicies {
            closure_policy_digest_blake3: DIGEST.to_string(),
            hermeticity_policy_digest_blake3: DIGEST.to_string(),
            protected_execution_policy_digest_blake3: DIGEST.to_string(),
            effect_policy_digest_blake3: DIGEST.to_string(),
            normalization_policy_digest_blake3: DIGEST.to_string(),
        }
    }

    fn valid_entry(cache_key: &str) -> DevProviderCacheEntry {
        DevProviderCacheEntry {
            schema: DEV_CACHE_SCHEMA.to_string(),
            plan_digest_blake3: DIGEST.to_string(),
            cache_key: cache_key.to_string(),
            source_authority_digest_blake3: DIGEST.to_string(),
            closure_policy_digest_blake3: DIGEST.to_string(),
            hermeticity_policy_digest_blake3: DIGEST.to_string(),
            protected_execution_policy_digest_blake3: DIGEST.to_string(),
            effect_policy_digest_blake3: DIGEST.to_string(),
            normalization_policy_digest_blake3: DIGEST.to_string(),
            stagex_provider: DevProviderReference {
                logical_path: "/mantle/store/aaa-stagex".to_string(),
                store_basename: "aaa-stagex".to_string(),
                digest_blake3: DIGEST.to_string(),
            },
            native_provider: DevProviderReference {
                logical_path: "/mantle/store/aaa-native".to_string(),
                store_basename: "aaa-native".to_string(),
                digest_blake3: DIGEST.to_string(),
            },
        }
    }

    #[test]
    fn provider_cache_key_is_stable_and_digest_sensitive() {
        let policies = policies();
        let key = dev_provider_cache_key(DIGEST, &policies);
        let again = dev_provider_cache_key(DIGEST, &policies);
        let changed_source = dev_provider_cache_key(OTHER_DIGEST, &policies);
        let mut shifted = policies.clone();
        shifted.normalization_policy_digest_blake3 = OTHER_DIGEST.to_string();
        let changed_policy = dev_provider_cache_key(DIGEST, &shifted);

        assert_eq!(key, again);
        assert_eq!(key.len(), BLAKE3_HEX_LENGTH);
        assert_ne!(key, changed_source);
        assert_ne!(key, changed_policy);
    }

    #[test]
    fn provider_cache_lookup_accepts_only_exact_receipt_validated_entry() {
        let policies = policies();
        let source = DIGEST.to_string();
        let key = dev_provider_cache_key(&source, &policies);
        let hit = evaluate_provider_cache_lookup(Some(&valid_entry(&key)), &plan_for(DIGEST), &policies);
        let missing = evaluate_provider_cache_lookup(None, &plan_for(DIGEST), &policies);

        assert_eq!(hit, DevProviderCacheLookup::Hit);
        assert_eq!(missing, DevProviderCacheLookup::Miss);
    }

    #[test]
    fn provider_cache_rejects_stale_or_mismatched_digests_as_hard_miss() {
        let policies = policies();
        let key = dev_provider_cache_key(DIGEST, &policies);
        let mut stale = valid_entry(&key);
        stale.source_authority_digest_blake3 = OTHER_DIGEST.to_string();
        let stale_lookup = evaluate_provider_cache_lookup(Some(&stale), &plan_for(DIGEST), &policies);
        let mut wrong_schema = valid_entry(&key);
        wrong_schema.schema = "other-schema".to_string();
        let schema_lookup = evaluate_provider_cache_lookup(Some(&wrong_schema), &plan_for(DIGEST), &policies);
        let mut wrong_key = valid_entry(&key);
        wrong_key.cache_key = OTHER_DIGEST.to_string();
        let key_lookup = evaluate_provider_cache_lookup(Some(&wrong_key), &plan_for(DIGEST), &policies);
        let mut mutated_provider = valid_entry(&key);
        mutated_provider.native_provider.digest_blake3 = OTHER_DIGEST.to_string();
        let provider_lookup = evaluate_provider_cache_lookup(Some(&mutated_provider), &plan_for(DIGEST), &policies);

        assert_eq!(stale_lookup, DevProviderCacheLookup::Miss);
        assert_eq!(schema_lookup, DevProviderCacheLookup::Miss);
        assert_eq!(key_lookup, DevProviderCacheLookup::Miss);
        assert_eq!(provider_lookup, DevProviderCacheLookup::Miss);
    }

    #[test]
    fn stage_marker_trusts_only_exact_plan_stage_and_digest_match() {
        let trusted = validate_stage_marker(
            Some(&StageCompletionMarker {
                schema: STAGE_MARKER_SCHEMA.to_string(),
                plan_digest_blake3: DIGEST.to_string(),
                stage_id: "stagex-transition".to_string(),
                digest_blake3: DIGEST.to_string(),
            }),
            DIGEST,
            "stagex-transition",
            DIGEST,
        );
        let mutated = validate_stage_marker(
            Some(&StageCompletionMarker {
                schema: STAGE_MARKER_SCHEMA.to_string(),
                plan_digest_blake3: DIGEST.to_string(),
                stage_id: "stagex-transition".to_string(),
                digest_blake3: OTHER_DIGEST.to_string(),
            }),
            DIGEST,
            "stagex-transition",
            DIGEST,
        );
        let missing = validate_stage_marker(None, DIGEST, "stagex-transition", DIGEST);

        assert_eq!(trusted, StageMarkerValidation::Trusted);
        assert_eq!(mutated, StageMarkerValidation::Reject);
        assert_eq!(missing, StageMarkerValidation::Reject);
    }

    #[test]
    fn fast_fail_reports_exact_matched_prior_digest_or_proceeds() {
        let report = evaluate_fast_fail(DIGEST, Some(DIGEST));
        let proceed = evaluate_fast_fail(DIGEST, Some(OTHER_DIGEST));
        let no_binding = evaluate_fast_fail(DIGEST, None);

        assert!(matches!(
            report,
            FastFailDecision::ReportPriorSuccess { ref matched_source_authority_digest_blake3 }
                if matched_source_authority_digest_blake3 == DIGEST
        ));
        assert_eq!(proceed, FastFailDecision::Proceed);
        assert_eq!(no_binding, FastFailDecision::Proceed);
    }

    #[test]
    fn ordered_policy_digests_are_distinct_and_stable() {
        let closure = ordered_policy_digest(crate::source_built_fixed_point::ProofPolicyRole::Closure);
        let hermeticity = ordered_policy_digest(crate::source_built_fixed_point::ProofPolicyRole::Hermeticity);

        assert_eq!(closure, ordered_policy_digest(crate::source_built_fixed_point::ProofPolicyRole::Closure));
        assert_eq!(closure.len(), BLAKE3_HEX_LENGTH);
        assert_ne!(closure, hermeticity);
    }

    fn plan_for(digest: &str) -> SourceBuiltFixedPointPlan {
        let policies = policies();
        let mut plan = SourceBuiltFixedPointPlan {
            schema: crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_PLAN_SCHEMA.to_string(),
            proof_id: "dev-cache-test".to_string(),
            logical_store_prefix: "/mantle/store".to_string(),
            source_authority_digest_blake3: digest.to_string(),
            source_inputs: vec![],
            policies: crate::source_built_fixed_point::SourceBuiltFixedPointPolicies {
                expected_native_provider_digest_blake3: digest.to_string(),
                closure_policy_digest_blake3: policies.closure_policy_digest_blake3.clone(),
                hermeticity_policy_digest_blake3: policies.hermeticity_policy_digest_blake3.clone(),
                protected_execution_policy_digest_blake3: policies.protected_execution_policy_digest_blake3.clone(),
                effect_policy_digest_blake3: policies.effect_policy_digest_blake3.clone(),
                normalization_policy_digest_blake3: policies.normalization_policy_digest_blake3.clone(),
                hermeticity_mode: crate::source_built_fixed_point::ProofHermeticityMode::Strict,
                live_fetch_allowed: false,
                cargo_invocation_allowed: false,
                ambient_discovery_allowed: false,
                fallback_allowed: false,
                provider_cache_completion_allowed: false,
            },
            resource_bounds: crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds {
                elapsed_seconds_max: 1,
                disk_bytes_max: 1,
                protected_exec_events_max: 1,
                source_records_max: 1,
                open_file_descriptors_max: 1,
            },
            stages: vec![],
            receipt_contract: crate::source_built_fixed_point::SourceBuiltFixedPointReceiptContract {
                schema: String::new(),
                workflow_version: String::new(),
                selected_provider_kind: String::new(),
                source_blake3: digest.to_string(),
                vendor_blake3: digest.to_string(),
                logical_store_prefix: "/mantle/store".to_string(),
                effect_policy_version: String::new(),
                run_ids: vec![],
                derivation_identity_source: String::new(),
                toolchain_provider_identity_source: String::new(),
                toolchain_stage_roots_source: String::new(),
                physical_store_isolation: String::new(),
                declared_effects: vec![],
                normalized_execution_envelope: vec![],
                sandbox_profile_identities: vec![],
                require_content_bound_rebuild_descriptor: false,
                require_rebuild_authority_plan: false,
                require_matching_output_digests: false,
                require_zero_authority_violations: false,
                require_zero_hermeticity_events: false,
                require_zero_substitutions: false,
                require_zero_live_fetches: false,
                require_zero_cargo_invocations: false,
                require_zero_fallback_events: false,
            },
            non_claims: vec![],
            plan_digest_blake3: digest.to_string(),
        };
        plan.plan_digest_blake3 = digest.to_string();
        plan
    }
}
