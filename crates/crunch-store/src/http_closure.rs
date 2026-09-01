use std::collections::BTreeMap;
use std::collections::BTreeSet;

use nix_compat::store_path::StorePath;
use serde::Serialize;
use thiserror::Error;

pub const HTTP_CLOSURE_PLAN_SCHEMA: &str = "mantle-http-cache-closure-plan-v1";
pub const MAX_HTTP_CLOSURE_MEMBERS: u32 = 100_000;
pub const MAX_HTTP_CLOSURE_REFERENCES: u32 = 1_000_000;
pub const MAX_HTTP_CLOSURE_DEPTH: u32 = 1_024;
pub const MAX_HTTP_CLOSURE_NARINFO_BYTES: u64 = 1_048_576;
pub const MAX_HTTP_CLOSURE_TOTAL_NAR_BYTES: u64 = 1_099_511_627_776;

const HTTP_CLOSURE_PLAN_DOMAIN: &[u8] = b"mantle-http-cache-closure-plan-v1\0";
const BLAKE3_BYTES: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct HttpClosureLimits {
    pub max_members: u32,
    pub max_references: u32,
    pub max_depth: u32,
    pub max_narinfo_bytes: u64,
    pub max_total_nar_bytes: u64,
}

impl Default for HttpClosureLimits {
    fn default() -> Self {
        Self {
            max_members: MAX_HTTP_CLOSURE_MEMBERS,
            max_references: MAX_HTTP_CLOSURE_REFERENCES,
            max_depth: MAX_HTTP_CLOSURE_DEPTH,
            max_narinfo_bytes: MAX_HTTP_CLOSURE_NARINFO_BYTES,
            max_total_nar_bytes: MAX_HTTP_CLOSURE_TOTAL_NAR_BYTES,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpClosureRequest {
    pub path: StorePath<String>,
    pub depth: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpClosureObservation {
    pub requested_path: StorePath<String>,
    pub returned_path: StorePath<String>,
    pub references: Vec<StorePath<String>>,
    pub nar_sha256: [u8; BLAKE3_BYTES],
    pub nar_size: u64,
    pub narinfo_blake3: [u8; BLAKE3_BYTES],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HttpClosurePlanMember {
    pub store_path: String,
    pub depth: u32,
    pub references: Vec<String>,
    pub nar_sha256_hex: String,
    pub nar_size: u64,
    pub narinfo_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HttpClosurePlan {
    pub schema: String,
    pub cache_identity: String,
    pub trust_policy_blake3: String,
    pub store_dir: String,
    pub root: String,
    pub limits: HttpClosureLimits,
    pub total_nar_bytes: u64,
    pub members: Vec<HttpClosurePlanMember>,
    pub plan_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingMember {
    path: StorePath<String>,
    depth: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ObservedMember {
    path: StorePath<String>,
    depth: u32,
    references: Vec<StorePath<String>>,
    nar_sha256: [u8; BLAKE3_BYTES],
    nar_size: u64,
    narinfo_blake3: [u8; BLAKE3_BYTES],
}

// r[impl cache_substitution.complete_http_closure_pull]
#[derive(Debug)]
pub struct HttpClosurePlanBuilder {
    cache_identity: String,
    trust_policy_blake3: String,
    store_dir: String,
    root: StorePath<String>,
    limits: HttpClosureLimits,
    pending_order: BTreeSet<(u32, [u8; 20])>,
    pending_by_digest: BTreeMap<[u8; 20], PendingMember>,
    observed_by_digest: BTreeMap<[u8; 20], ObservedMember>,
    active_request: Option<HttpClosureRequest>,
    total_nar_bytes: u64,
    total_reference_count: u32,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum HttpClosurePlanError {
    #[error("http-closure-invalid-identity: {field}")]
    InvalidIdentity { field: &'static str },
    #[error("http-closure-invalid-limits: {field}")]
    InvalidLimits { field: &'static str },
    #[error("http-closure-request-already-active: {path}")]
    RequestAlreadyActive { path: String },
    #[error("http-closure-unexpected-observation: expected {expected}, got {actual}")]
    UnexpectedObservation { expected: String, actual: String },
    #[error("http-closure-returned-path-mismatch: requested {requested}, returned {returned}")]
    ReturnedPathMismatch { requested: String, returned: String },
    #[error("http-closure-member-limit: maximum {maximum}")]
    MemberLimit { maximum: u32 },
    #[error("http-closure-reference-limit: total {total} exceeds maximum {maximum}")]
    ReferenceLimit { total: u32, maximum: u32 },
    #[error("http-closure-reference-count-overflow")]
    ReferenceCountOverflow,
    #[error("http-closure-depth-limit: {path} depth {depth} exceeds maximum {maximum}")]
    DepthLimit { path: String, depth: u32, maximum: u32 },
    #[error("http-closure-total-nar-size-limit: total {total} exceeds maximum {maximum}")]
    TotalNarSizeLimit { total: u64, maximum: u64 },
    #[error("http-closure-total-nar-size-overflow")]
    TotalNarSizeOverflow,
    #[error("http-closure-duplicate-reference: {path}")]
    DuplicateReference { path: String },
    #[error("http-closure-conflicting-path-identity: {existing} conflicts with {observed}")]
    ConflictingPathIdentity { existing: String, observed: String },
    #[error("http-closure-incomplete-plan: {pending_count} member(s) pending")]
    IncompletePlan { pending_count: u32 },
    #[error("http-closure-state-invariant: {detail}")]
    StateInvariant { detail: &'static str },
    #[error("http-closure-plan-serialization: {detail}")]
    PlanSerialization { detail: String },
}

impl HttpClosurePlanError {
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::InvalidIdentity { .. } => "http-closure-invalid-identity",
            Self::InvalidLimits { .. } => "http-closure-invalid-limits",
            Self::RequestAlreadyActive { .. } => "http-closure-request-already-active",
            Self::UnexpectedObservation { .. } => "http-closure-unexpected-observation",
            Self::ReturnedPathMismatch { .. } => "http-closure-returned-path-mismatch",
            Self::MemberLimit { .. } => "http-closure-member-limit",
            Self::ReferenceLimit { .. } => "http-closure-reference-limit",
            Self::ReferenceCountOverflow => "http-closure-reference-count-overflow",
            Self::DepthLimit { .. } => "http-closure-depth-limit",
            Self::TotalNarSizeLimit { .. } => "http-closure-total-nar-size-limit",
            Self::TotalNarSizeOverflow => "http-closure-total-nar-size-overflow",
            Self::DuplicateReference { .. } => "http-closure-duplicate-reference",
            Self::ConflictingPathIdentity { .. } => "http-closure-conflicting-path-identity",
            Self::IncompletePlan { .. } => "http-closure-incomplete-plan",
            Self::StateInvariant { .. } => "http-closure-state-invariant",
            Self::PlanSerialization { .. } => "http-closure-plan-serialization",
        }
    }
}

struct IdentityField<'a> {
    value: &'a str,
    field: &'static str,
}

impl HttpClosurePlanBuilder {
    pub fn new(
        root: StorePath<String>,
        cache_identity: String,
        trust_policy_blake3: String,
        store_dir: String,
        limits: HttpClosureLimits,
    ) -> Result<Self, HttpClosurePlanError> {
        validate_identity_field(IdentityField {
            value: &cache_identity,
            field: "cache_identity",
        })?;
        validate_identity_field(IdentityField {
            value: &trust_policy_blake3,
            field: "trust_policy_blake3",
        })?;
        validate_identity_field(IdentityField {
            value: &store_dir,
            field: "store_dir",
        })?;
        validate_limits(limits)?;

        let root_digest = *root.digest();
        let root_pending = PendingMember {
            path: root.clone(),
            depth: 0,
        };
        let mut pending_order = BTreeSet::new();
        let mut pending_by_digest = BTreeMap::new();
        pending_order.insert((0, root_digest));
        pending_by_digest.insert(root_digest, root_pending);

        debug_assert_eq!(pending_order.len(), pending_by_digest.len());
        debug_assert!(pending_by_digest.contains_key(&root_digest));
        Ok(Self {
            cache_identity,
            trust_policy_blake3,
            store_dir,
            root,
            limits,
            pending_order,
            pending_by_digest,
            observed_by_digest: BTreeMap::new(),
            active_request: None,
            total_nar_bytes: 0,
            total_reference_count: 0,
        })
    }

    pub fn take_next_request(&mut self) -> Result<Option<HttpClosureRequest>, HttpClosurePlanError> {
        if let Some(active) = &self.active_request {
            return Err(HttpClosurePlanError::RequestAlreadyActive {
                path: active.path.to_string(),
            });
        }
        let Some((depth, digest)) = self.pending_order.pop_first() else {
            debug_assert!(self.pending_by_digest.is_empty());
            return Ok(None);
        };
        let pending = self.pending_by_digest.get(&digest).ok_or(HttpClosurePlanError::StateInvariant {
            detail: "pending-order-map-mismatch",
        })?;
        let request = HttpClosureRequest {
            path: pending.path.clone(),
            depth,
        };
        self.active_request = Some(request.clone());
        debug_assert_eq!(pending.depth, depth);
        debug_assert_eq!(pending.path.digest(), &digest);
        Ok(Some(request))
    }

    pub fn observe(&mut self, observation: HttpClosureObservation) -> Result<(), HttpClosurePlanError> {
        let active = self.active_request.as_ref().ok_or_else(|| HttpClosurePlanError::UnexpectedObservation {
            expected: "no-active-request".to_string(),
            actual: observation.requested_path.to_string(),
        })?;
        if active.path != observation.requested_path {
            return Err(HttpClosurePlanError::UnexpectedObservation {
                expected: active.path.to_string(),
                actual: observation.requested_path.to_string(),
            });
        }
        if observation.requested_path != observation.returned_path {
            return Err(HttpClosurePlanError::ReturnedPathMismatch {
                requested: observation.requested_path.to_string(),
                returned: observation.returned_path.to_string(),
            });
        }

        let next_total = self
            .total_nar_bytes
            .checked_add(observation.nar_size)
            .ok_or(HttpClosurePlanError::TotalNarSizeOverflow)?;
        if next_total > self.limits.max_total_nar_bytes {
            return Err(HttpClosurePlanError::TotalNarSizeLimit {
                total: next_total,
                maximum: self.limits.max_total_nar_bytes,
            });
        }

        let references = canonical_references(observation.references)?;
        let observed_reference_count =
            u32::try_from(references.len()).map_err(|_| HttpClosurePlanError::ReferenceCountOverflow)?;
        let next_reference_count = self
            .total_reference_count
            .checked_add(observed_reference_count)
            .ok_or(HttpClosurePlanError::ReferenceCountOverflow)?;
        if next_reference_count > self.limits.max_references {
            return Err(HttpClosurePlanError::ReferenceLimit {
                total: next_reference_count,
                maximum: self.limits.max_references,
            });
        }
        self.add_references(&references, active.depth)?;
        let requested_digest = *observation.requested_path.digest();
        let pending = self.pending_by_digest.remove(&requested_digest).ok_or(HttpClosurePlanError::StateInvariant {
            detail: "active-request-pending-member-missing",
        })?;
        self.observed_by_digest.insert(requested_digest, ObservedMember {
            path: observation.requested_path,
            depth: pending.depth,
            references,
            nar_sha256: observation.nar_sha256,
            nar_size: observation.nar_size,
            narinfo_blake3: observation.narinfo_blake3,
        });
        self.total_nar_bytes = next_total;
        self.total_reference_count = next_reference_count;
        self.active_request = None;

        debug_assert_eq!(
            self.member_count(),
            self.pending_by_digest.len().saturating_add(self.observed_by_digest.len())
        );
        debug_assert!(self.observed_by_digest.contains_key(&requested_digest));
        Ok(())
    }

    pub fn finalize(self) -> Result<HttpClosurePlan, HttpClosurePlanError> {
        if self.active_request.is_some() || !self.pending_by_digest.is_empty() {
            let pending_count =
                u32::try_from(self.pending_by_digest.len()).map_err(|_| HttpClosurePlanError::MemberLimit {
                    maximum: self.limits.max_members,
                })?;
            return Err(HttpClosurePlanError::IncompletePlan { pending_count });
        }
        let root_digest = *self.root.digest();
        let root_member = self.observed_by_digest.get(&root_digest).ok_or(HttpClosurePlanError::StateInvariant {
            detail: "complete-plan-root-missing",
        })?;
        let mut members = self
            .observed_by_digest
            .values()
            .filter(|member| member.path != self.root)
            .map(plan_member)
            .collect::<Vec<_>>();
        members.sort_by(|left, right| left.store_path.cmp(&right.store_path));
        members.push(plan_member(root_member));

        let mut plan = HttpClosurePlan {
            schema: HTTP_CLOSURE_PLAN_SCHEMA.to_string(),
            cache_identity: self.cache_identity,
            trust_policy_blake3: self.trust_policy_blake3,
            store_dir: self.store_dir,
            root: self.root.to_string(),
            limits: self.limits,
            total_nar_bytes: self.total_nar_bytes,
            members,
            plan_blake3: String::new(),
        };
        plan.plan_blake3 = compute_plan_blake3(&plan)?;
        debug_assert_eq!(plan.members.last().map(|member| member.store_path.as_str()), Some(plan.root.as_str()));
        debug_assert_eq!(plan.plan_blake3.len(), BLAKE3_BYTES.saturating_mul(2));
        Ok(plan)
    }

    fn add_references(
        &mut self,
        references: &[StorePath<String>],
        parent_depth: u32,
    ) -> Result<(), HttpClosurePlanError> {
        let child_depth = parent_depth.checked_add(1).ok_or_else(|| HttpClosurePlanError::DepthLimit {
            path: self.root.to_string(),
            depth: u32::MAX,
            maximum: self.limits.max_depth,
        })?;
        for reference in references {
            if let Some(existing_path) = self.known_path_for_digest(reference.digest()) {
                if existing_path != reference {
                    return Err(HttpClosurePlanError::ConflictingPathIdentity {
                        existing: existing_path.to_string(),
                        observed: reference.to_string(),
                    });
                }
                continue;
            }
            if child_depth > self.limits.max_depth {
                return Err(HttpClosurePlanError::DepthLimit {
                    path: reference.to_string(),
                    depth: child_depth,
                    maximum: self.limits.max_depth,
                });
            }
            self.insert_pending(reference.clone(), child_depth)?;
        }
        if let Ok(maximum_members) = usize::try_from(self.limits.max_members) {
            debug_assert!(self.member_count() <= maximum_members);
        }
        debug_assert!(self.pending_order.len() <= self.pending_by_digest.len());
        Ok(())
    }

    fn insert_pending(&mut self, path: StorePath<String>, depth: u32) -> Result<(), HttpClosurePlanError> {
        let next_count = self.member_count().checked_add(1).ok_or(HttpClosurePlanError::MemberLimit {
            maximum: self.limits.max_members,
        })?;
        let maximum_members =
            usize::try_from(self.limits.max_members).map_err(|_| HttpClosurePlanError::MemberLimit {
                maximum: self.limits.max_members,
            })?;
        if next_count > maximum_members {
            return Err(HttpClosurePlanError::MemberLimit {
                maximum: self.limits.max_members,
            });
        }
        let digest = *path.digest();
        self.pending_order.insert((depth, digest));
        self.pending_by_digest.insert(digest, PendingMember { path, depth });
        debug_assert!(self.pending_order.contains(&(depth, digest)));
        debug_assert!(self.pending_by_digest.contains_key(&digest));
        Ok(())
    }

    fn known_path_for_digest(&self, digest: &[u8; 20]) -> Option<&StorePath<String>> {
        if let Some(observed) = self.observed_by_digest.get(digest) {
            return Some(&observed.path);
        }
        self.pending_by_digest.get(digest).map(|pending| &pending.path)
    }

    fn member_count(&self) -> usize {
        self.pending_by_digest.len().saturating_add(self.observed_by_digest.len())
    }
}

fn validate_identity_field(input: IdentityField<'_>) -> Result<(), HttpClosurePlanError> {
    if input.value.is_empty() {
        return Err(HttpClosurePlanError::InvalidIdentity { field: input.field });
    }
    debug_assert!(!input.value.is_empty());
    debug_assert!(!input.field.is_empty());
    Ok(())
}

fn validate_limits(limits: HttpClosureLimits) -> Result<(), HttpClosurePlanError> {
    if limits.max_members == 0 {
        return Err(HttpClosurePlanError::InvalidLimits { field: "max_members" });
    }
    if limits.max_references == 0 {
        return Err(HttpClosurePlanError::InvalidLimits {
            field: "max_references",
        });
    }
    if limits.max_narinfo_bytes == 0 {
        return Err(HttpClosurePlanError::InvalidLimits {
            field: "max_narinfo_bytes",
        });
    }
    if limits.max_total_nar_bytes == 0 {
        return Err(HttpClosurePlanError::InvalidLimits {
            field: "max_total_nar_bytes",
        });
    }
    debug_assert!(limits.max_members > 0);
    debug_assert!(limits.max_references > 0);
    debug_assert!(limits.max_narinfo_bytes > 0);
    Ok(())
}

fn canonical_references(references: Vec<StorePath<String>>) -> Result<Vec<StorePath<String>>, HttpClosurePlanError> {
    let mut canonical = references;
    canonical.sort_by_key(ToString::to_string);
    for pair in canonical.windows(2) {
        if pair[0] == pair[1] {
            return Err(HttpClosurePlanError::DuplicateReference {
                path: pair[0].to_string(),
            });
        }
    }
    debug_assert!(canonical.windows(2).all(|pair| pair[0].to_string() < pair[1].to_string()));
    debug_assert_eq!(canonical.len(), canonical.iter().collect::<BTreeSet<_>>().len());
    Ok(canonical)
}

fn plan_member(member: &ObservedMember) -> HttpClosurePlanMember {
    let references = member.references.iter().map(ToString::to_string).collect::<Vec<_>>();
    let plan_member = HttpClosurePlanMember {
        store_path: member.path.to_string(),
        depth: member.depth,
        references,
        nar_sha256_hex: data_encoding::HEXLOWER.encode(&member.nar_sha256),
        nar_size: member.nar_size,
        narinfo_blake3: data_encoding::HEXLOWER.encode(&member.narinfo_blake3),
    };
    debug_assert!(!plan_member.store_path.is_empty());
    debug_assert_eq!(plan_member.nar_sha256_hex.len(), BLAKE3_BYTES.saturating_mul(2));
    plan_member
}

#[derive(Serialize)]
struct HttpClosurePlanIdentity<'a> {
    schema: &'a str,
    cache_identity: &'a str,
    trust_policy_blake3: &'a str,
    store_dir: &'a str,
    root: &'a str,
    limits: HttpClosureLimits,
    total_nar_bytes: u64,
    members: &'a [HttpClosurePlanMember],
}

fn compute_plan_blake3(plan: &HttpClosurePlan) -> Result<String, HttpClosurePlanError> {
    let identity = HttpClosurePlanIdentity {
        schema: &plan.schema,
        cache_identity: &plan.cache_identity,
        trust_policy_blake3: &plan.trust_policy_blake3,
        store_dir: &plan.store_dir,
        root: &plan.root,
        limits: plan.limits,
        total_nar_bytes: plan.total_nar_bytes,
        members: &plan.members,
    };
    let canonical_bytes = serde_json::to_vec(&identity).map_err(|error| HttpClosurePlanError::PlanSerialization {
        detail: error.to_string(),
    })?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(HTTP_CLOSURE_PLAN_DOMAIN);
    hasher.update(&canonical_bytes);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert!(!canonical_bytes.is_empty());
    debug_assert_eq!(digest.len(), BLAKE3_BYTES.saturating_mul(2));
    Ok(digest)
}

pub fn verify_http_closure_plan_identity(plan: &HttpClosurePlan) -> Result<bool, HttpClosurePlanError> {
    let expected = compute_plan_blake3(plan)?;
    let is_valid = expected == plan.plan_blake3;
    debug_assert_eq!(expected.len(), BLAKE3_BYTES.saturating_mul(2));
    debug_assert!(!plan.schema.is_empty());
    Ok(is_valid)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT_SEED: u8 = 11;
    const FIRST_SEED: u8 = 12;
    const SECOND_SEED: u8 = 13;
    const THIRD_SEED: u8 = 14;
    const NAR_SIZE: u64 = 64;
    const SMALL_MEMBER_LIMIT: u32 = 2;
    const SMALL_REFERENCE_LIMIT: u32 = 1;
    const EXCESSIVE_REFERENCE_COUNT: u32 = 2;
    const ZERO_DEPTH: u32 = 0;
    const ONE_DEPTH: u32 = 1;
    const TWO_DEPTH: u32 = 2;
    const SMALL_TOTAL_NAR_BYTES: u64 = 100;

    fn make_path(name: &str, seed: u8) -> StorePath<String> {
        let mut digest = [0u8; 20];
        digest[0] = seed;
        StorePath::from_name_and_digest_fixed(name, digest).unwrap()
    }

    fn limits() -> HttpClosureLimits {
        HttpClosureLimits {
            max_members: MAX_HTTP_CLOSURE_MEMBERS,
            max_references: MAX_HTTP_CLOSURE_REFERENCES,
            max_depth: MAX_HTTP_CLOSURE_DEPTH,
            max_narinfo_bytes: MAX_HTTP_CLOSURE_NARINFO_BYTES,
            max_total_nar_bytes: MAX_HTTP_CLOSURE_TOTAL_NAR_BYTES,
        }
    }

    fn builder(root: &StorePath<String>) -> HttpClosurePlanBuilder {
        HttpClosurePlanBuilder::new(
            root.clone(),
            "https://cache.example.test/".to_string(),
            "trust-blake3".to_string(),
            "/nix/store".to_string(),
            limits(),
        )
        .unwrap()
    }

    fn observe(
        builder: &mut HttpClosurePlanBuilder,
        request: HttpClosureRequest,
        references: Vec<StorePath<String>>,
        nar_size: u64,
    ) {
        builder
            .observe(HttpClosureObservation {
                requested_path: request.path.clone(),
                returned_path: request.path,
                references,
                nar_sha256: [FIRST_SEED; BLAKE3_BYTES],
                nar_size,
                narinfo_blake3: [SECOND_SEED; BLAKE3_BYTES],
            })
            .unwrap();
    }

    // r[verify cache_substitution.complete_http_closure_pull]
    #[test]
    fn one_member_plan_is_stable_and_root_last() {
        let root = make_path("root", ROOT_SEED);
        let mut builder = builder(&root);
        let request = builder.take_next_request().unwrap().unwrap();
        observe(&mut builder, request, vec![], NAR_SIZE);
        assert!(builder.take_next_request().unwrap().is_none());

        let plan = builder.finalize().unwrap();
        assert_eq!(plan.members.len(), 1);
        assert_eq!(plan.members[0].store_path, root.to_string());
        assert_eq!(plan.total_nar_bytes, NAR_SIZE);
        assert_eq!(plan.plan_blake3.len(), BLAKE3_BYTES * 2);
        assert!(verify_http_closure_plan_identity(&plan).unwrap());
        let mut tampered = plan.clone();
        tampered.total_nar_bytes = tampered.total_nar_bytes.saturating_add(1);
        assert!(!verify_http_closure_plan_identity(&tampered).unwrap());
    }

    #[test]
    fn linear_plan_imports_root_last() {
        let root = make_path("root", ROOT_SEED);
        let child = make_path("child", FIRST_SEED);
        let leaf = make_path("leaf", SECOND_SEED);
        let mut builder = builder(&root);

        let root_request = builder.take_next_request().unwrap().unwrap();
        assert_eq!(root_request.depth, ZERO_DEPTH);
        observe(&mut builder, root_request, vec![child.clone()], NAR_SIZE);
        let child_request = builder.take_next_request().unwrap().unwrap();
        assert_eq!(child_request.depth, ONE_DEPTH);
        observe(&mut builder, child_request, vec![leaf.clone()], NAR_SIZE);
        let leaf_request = builder.take_next_request().unwrap().unwrap();
        assert_eq!(leaf_request.depth, TWO_DEPTH);
        observe(&mut builder, leaf_request, vec![], NAR_SIZE);

        let plan = builder.finalize().unwrap();
        assert_eq!(plan.members.len(), 3);
        assert_eq!(plan.members.last().unwrap().store_path, root.to_string());
        assert!(plan.members.iter().any(|member| member.store_path == child.to_string()));
        assert!(plan.members.iter().any(|member| member.store_path == leaf.to_string()));
    }

    #[test]
    fn diamond_and_cycle_are_deduplicated() {
        let root = make_path("root", ROOT_SEED);
        let left = make_path("left", FIRST_SEED);
        let right = make_path("right", SECOND_SEED);
        let leaf = make_path("leaf", THIRD_SEED);
        let mut builder = builder(&root);

        let request = builder.take_next_request().unwrap().unwrap();
        observe(&mut builder, request, vec![right.clone(), left.clone()], NAR_SIZE);
        let request = builder.take_next_request().unwrap().unwrap();
        let first_path = request.path.clone();
        observe(&mut builder, request, vec![leaf.clone()], NAR_SIZE);
        let request = builder.take_next_request().unwrap().unwrap();
        let second_path = request.path.clone();
        observe(&mut builder, request, vec![leaf.clone()], NAR_SIZE);
        assert_ne!(first_path, second_path);
        let request = builder.take_next_request().unwrap().unwrap();
        observe(&mut builder, request, vec![root.clone()], NAR_SIZE);

        let plan = builder.finalize().unwrap();
        assert_eq!(plan.members.len(), 4);
        assert_eq!(plan.members.iter().filter(|member| member.store_path == leaf.to_string()).count(), 1);
        assert_eq!(plan.members.last().unwrap().store_path, root.to_string());
    }

    #[test]
    fn reference_order_does_not_change_plan_identity() {
        let root = make_path("root", ROOT_SEED);
        let left = make_path("left", FIRST_SEED);
        let right = make_path("right", SECOND_SEED);
        let build_plan = |references: Vec<StorePath<String>>| {
            let mut builder = builder(&root);
            let request = builder.take_next_request().unwrap().unwrap();
            observe(&mut builder, request, references, NAR_SIZE);
            while let Some(request) = builder.take_next_request().unwrap() {
                observe(&mut builder, request, vec![], NAR_SIZE);
            }
            builder.finalize().unwrap()
        };

        let first = build_plan(vec![left.clone(), right.clone()]);
        let second = build_plan(vec![right, left]);
        assert_eq!(first.plan_blake3, second.plan_blake3);
        assert_eq!(first.members, second.members);
    }

    // r[verify cache_substitution.complete_http_closure_pull]
    #[test]
    fn member_limit_fails_closed() {
        let root = make_path("root", ROOT_SEED);
        let left = make_path("left", FIRST_SEED);
        let right = make_path("right", SECOND_SEED);
        let limited = HttpClosureLimits {
            max_members: SMALL_MEMBER_LIMIT,
            ..limits()
        };
        let mut builder = HttpClosurePlanBuilder::new(
            root.clone(),
            "cache".to_string(),
            "trust".to_string(),
            "/nix/store".to_string(),
            limited,
        )
        .unwrap();
        let request = builder.take_next_request().unwrap().unwrap();
        let error = builder
            .observe(HttpClosureObservation {
                requested_path: request.path.clone(),
                returned_path: request.path,
                references: vec![left, right],
                nar_sha256: [FIRST_SEED; BLAKE3_BYTES],
                nar_size: NAR_SIZE,
                narinfo_blake3: [SECOND_SEED; BLAKE3_BYTES],
            })
            .unwrap_err();
        assert_eq!(error.reason_code(), "http-closure-member-limit");
        assert_eq!(error, HttpClosurePlanError::MemberLimit {
            maximum: SMALL_MEMBER_LIMIT
        });
    }

    #[test]
    fn reference_limit_fails_before_pending_members_change() {
        let root = make_path("root", ROOT_SEED);
        let left = make_path("left", FIRST_SEED);
        let right = make_path("right", SECOND_SEED);
        let limited = HttpClosureLimits {
            max_references: SMALL_REFERENCE_LIMIT,
            ..limits()
        };
        let mut builder = HttpClosurePlanBuilder::new(
            root,
            "cache".to_string(),
            "trust".to_string(),
            "/nix/store".to_string(),
            limited,
        )
        .unwrap();
        let request = builder.take_next_request().unwrap().unwrap();
        let error = builder
            .observe(HttpClosureObservation {
                requested_path: request.path.clone(),
                returned_path: request.path,
                references: vec![left, right],
                nar_sha256: [FIRST_SEED; BLAKE3_BYTES],
                nar_size: NAR_SIZE,
                narinfo_blake3: [SECOND_SEED; BLAKE3_BYTES],
            })
            .unwrap_err();
        assert_eq!(error.reason_code(), "http-closure-reference-limit");
        assert_eq!(error, HttpClosurePlanError::ReferenceLimit {
            total: EXCESSIVE_REFERENCE_COUNT,
            maximum: SMALL_REFERENCE_LIMIT,
        });
        assert_eq!(builder.member_count(), 1);
    }

    #[test]
    fn depth_limit_fails_closed() {
        let root = make_path("root", ROOT_SEED);
        let child = make_path("child", FIRST_SEED);
        let limited = HttpClosureLimits {
            max_depth: ZERO_DEPTH,
            ..limits()
        };
        let mut builder = HttpClosurePlanBuilder::new(
            root,
            "cache".to_string(),
            "trust".to_string(),
            "/nix/store".to_string(),
            limited,
        )
        .unwrap();
        let request = builder.take_next_request().unwrap().unwrap();
        let error = builder
            .observe(HttpClosureObservation {
                requested_path: request.path.clone(),
                returned_path: request.path,
                references: vec![child.clone()],
                nar_sha256: [FIRST_SEED; BLAKE3_BYTES],
                nar_size: NAR_SIZE,
                narinfo_blake3: [SECOND_SEED; BLAKE3_BYTES],
            })
            .unwrap_err();
        assert_eq!(error.reason_code(), "http-closure-depth-limit");
        assert_eq!(error, HttpClosurePlanError::DepthLimit {
            path: child.to_string(),
            depth: ONE_DEPTH,
            maximum: ZERO_DEPTH,
        });
    }

    #[test]
    fn total_nar_size_limit_fails_closed() {
        let root = make_path("root", ROOT_SEED);
        let limited = HttpClosureLimits {
            max_total_nar_bytes: SMALL_TOTAL_NAR_BYTES,
            ..limits()
        };
        let mut builder = HttpClosurePlanBuilder::new(
            root,
            "cache".to_string(),
            "trust".to_string(),
            "/nix/store".to_string(),
            limited,
        )
        .unwrap();
        let request = builder.take_next_request().unwrap().unwrap();
        let excessive_nar_size = SMALL_TOTAL_NAR_BYTES.saturating_add(1);
        let error = builder
            .observe(HttpClosureObservation {
                requested_path: request.path.clone(),
                returned_path: request.path,
                references: vec![],
                nar_sha256: [FIRST_SEED; BLAKE3_BYTES],
                nar_size: excessive_nar_size,
                narinfo_blake3: [SECOND_SEED; BLAKE3_BYTES],
            })
            .unwrap_err();
        assert_eq!(error.reason_code(), "http-closure-total-nar-size-limit");
        assert_eq!(error, HttpClosurePlanError::TotalNarSizeLimit {
            total: excessive_nar_size,
            maximum: SMALL_TOTAL_NAR_BYTES,
        });
    }

    #[test]
    fn duplicate_reference_is_rejected() {
        let root = make_path("root", ROOT_SEED);
        let child = make_path("child", FIRST_SEED);
        let mut builder = builder(&root);
        let request = builder.take_next_request().unwrap().unwrap();
        let error = builder
            .observe(HttpClosureObservation {
                requested_path: request.path.clone(),
                returned_path: request.path,
                references: vec![child.clone(), child.clone()],
                nar_sha256: [FIRST_SEED; BLAKE3_BYTES],
                nar_size: NAR_SIZE,
                narinfo_blake3: [SECOND_SEED; BLAKE3_BYTES],
            })
            .unwrap_err();
        assert_eq!(error.reason_code(), "http-closure-duplicate-reference");
        assert_eq!(error, HttpClosurePlanError::DuplicateReference {
            path: child.to_string()
        });
    }

    #[test]
    fn conflicting_digest_path_identity_is_rejected() {
        let root = make_path("root", ROOT_SEED);
        let first = make_path("first", FIRST_SEED);
        let conflict = make_path("conflict", FIRST_SEED);
        let mut builder = builder(&root);
        let request = builder.take_next_request().unwrap().unwrap();
        let error = builder
            .observe(HttpClosureObservation {
                requested_path: request.path.clone(),
                returned_path: request.path,
                references: vec![first.clone(), conflict.clone()],
                nar_sha256: [FIRST_SEED; BLAKE3_BYTES],
                nar_size: NAR_SIZE,
                narinfo_blake3: [SECOND_SEED; BLAKE3_BYTES],
            })
            .unwrap_err();
        assert_eq!(error.reason_code(), "http-closure-conflicting-path-identity");
        assert_eq!(error, HttpClosurePlanError::ConflictingPathIdentity {
            existing: conflict.to_string(),
            observed: first.to_string(),
        });
    }

    #[test]
    fn returned_path_mismatch_is_rejected() {
        let root = make_path("root", ROOT_SEED);
        let other = make_path("other", SECOND_SEED);
        let mut builder = builder(&root);
        let request = builder.take_next_request().unwrap().unwrap();
        let error = builder
            .observe(HttpClosureObservation {
                requested_path: request.path,
                returned_path: other.clone(),
                references: vec![],
                nar_sha256: [FIRST_SEED; BLAKE3_BYTES],
                nar_size: NAR_SIZE,
                narinfo_blake3: [SECOND_SEED; BLAKE3_BYTES],
            })
            .unwrap_err();
        assert_eq!(error.reason_code(), "http-closure-returned-path-mismatch");
        assert!(error.to_string().contains(&other.to_string()));
    }

    #[test]
    fn incomplete_plan_and_active_request_fail_closed() {
        let root = make_path("root", ROOT_SEED);
        let mut builder = builder(&root);
        let request = builder.take_next_request().unwrap().unwrap();
        let active_error = builder.take_next_request().unwrap_err();
        assert_eq!(active_error.reason_code(), "http-closure-request-already-active");
        assert!(active_error.to_string().contains(&request.path.to_string()));

        let incomplete_error = builder.finalize().unwrap_err();
        assert_eq!(incomplete_error.reason_code(), "http-closure-incomplete-plan");
        assert_eq!(incomplete_error, HttpClosurePlanError::IncompletePlan { pending_count: 1 });
    }

    #[test]
    fn observation_without_active_request_is_rejected() {
        let root = make_path("root", ROOT_SEED);
        let mut builder = builder(&root);
        let error = builder
            .observe(HttpClosureObservation {
                requested_path: root.clone(),
                returned_path: root,
                references: vec![],
                nar_sha256: [FIRST_SEED; BLAKE3_BYTES],
                nar_size: NAR_SIZE,
                narinfo_blake3: [SECOND_SEED; BLAKE3_BYTES],
            })
            .unwrap_err();
        assert_eq!(error.reason_code(), "http-closure-unexpected-observation");
        assert!(error.to_string().contains("no-active-request"));
    }

    #[test]
    fn invalid_limits_and_identity_are_rejected() {
        let root = make_path("root", ROOT_SEED);
        let invalid_limits = HttpClosureLimits {
            max_members: 0,
            ..limits()
        };
        let limit_error = HttpClosurePlanBuilder::new(
            root.clone(),
            "cache".to_string(),
            "trust".to_string(),
            "/nix/store".to_string(),
            invalid_limits,
        )
        .unwrap_err();
        assert_eq!(limit_error.reason_code(), "http-closure-invalid-limits");
        assert_eq!(limit_error, HttpClosurePlanError::InvalidLimits { field: "max_members" });

        let identity_error =
            HttpClosurePlanBuilder::new(root, String::new(), "trust".to_string(), "/nix/store".to_string(), limits())
                .unwrap_err();
        assert_eq!(identity_error.reason_code(), "http-closure-invalid-identity");
        assert_eq!(identity_error, HttpClosurePlanError::InvalidIdentity {
            field: "cache_identity"
        });
    }
}
