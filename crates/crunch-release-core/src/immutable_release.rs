use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ReleaseEvidenceError;
use crate::manifest::BLAKE3_HEX_LENGTH_CHARS;
use crate::manifest::validate_blake3_hex;
use crate::manifest::validation_error;

pub const IMMUTABLE_RELEASE_OBJECT_PLAN_SCHEMA: &str = "mantle-immutable-release-object-plan-v1";
pub const CURRENT_RELEASE_POINTER_PLAN_SCHEMA: &str = "mantle-current-release-pointer-plan-v1";
pub const IMMUTABLE_RELEASE_EVIDENCE_SCHEMA: &str = "mantle-immutable-release-evidence-v1";
pub const RELEASE_OBJECT_DIGEST_ROLE: &str = "immutable-release-object";
pub const CURRENT_POINTER_DIGEST_ROLE: &str = "current-pointer-target";
pub const RELEASE_OBJECT_BYTES_MAX: u64 = 1_073_741_824;
pub const RELEASE_METADATA_FIELDS_MAX: u32 = 64;
pub const RELEASE_METADATA_TEXT_BYTES_MAX: u32 = 4_096;
pub const RELEASE_DIGEST_ROLE_BINDINGS_COUNT: usize = 2;

const _: () = {
    assert!(RELEASE_OBJECT_BYTES_MAX > 0, "release object byte limit must be positive");
    assert!(RELEASE_METADATA_FIELDS_MAX > 0, "release metadata field limit must be positive");
    assert!(RELEASE_METADATA_TEXT_BYTES_MAX > 0, "release metadata text limit must be positive");
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImmutableReleaseObjectDisposition {
    PublishNew,
    AlreadyPublishedIdentical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImmutableReleaseObjectRequest {
    pub object_bytes: Vec<u8>,
    pub declared_identity_blake3: String,
    pub existing_object_bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImmutableReleaseObjectPlan {
    pub schema: String,
    pub object_identity_blake3: String,
    pub object_size_bytes: u64,
    pub disposition: ImmutableReleaseObjectDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CurrentReleasePointerDisposition {
    SetInitial,
    Switch,
    Unchanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentReleasePointerRequest {
    pub target_identity_blake3: String,
    pub target_object_exists: bool,
    pub current_identity_blake3: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurrentReleasePointerPlan {
    pub schema: String,
    pub target_identity_blake3: String,
    pub previous_identity_blake3: Option<String>,
    pub disposition: CurrentReleasePointerDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseRollbackRequest {
    pub rollback_identity_blake3: String,
    pub rollback_object_exists: bool,
    pub current_identity_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseMetadataField {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseDigestRoleBinding {
    pub role: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReleaseAuthorityOwner {
    Caller,
    Mantle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImmutableReleaseAuthorityBoundary {
    pub distribution_owner: ReleaseAuthorityOwner,
    pub deployment_owner: ReleaseAuthorityOwner,
    pub retention_owner: ReleaseAuthorityOwner,
    pub deletion_owner: ReleaseAuthorityOwner,
    pub release_readiness_claimed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImmutableReleaseEvidenceInput {
    pub object_plan: ImmutableReleaseObjectPlan,
    pub pointer_plan: CurrentReleasePointerPlan,
    pub release_metadata: Vec<ReleaseMetadataField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImmutableReleaseEvidence {
    pub schema: String,
    pub object_identity_blake3: String,
    pub current_pointer_identity_blake3: String,
    pub previous_release_identity_blake3: Option<String>,
    pub release_metadata: Vec<ReleaseMetadataField>,
    pub digest_roles: Vec<ReleaseDigestRoleBinding>,
    pub authority_boundary: ImmutableReleaseAuthorityBoundary,
}

// r[impl mantle.release.object]
pub fn plan_immutable_release_object(
    request: ImmutableReleaseObjectRequest,
) -> Result<ImmutableReleaseObjectPlan, ReleaseEvidenceError> {
    validate_blake3_hex(&request.declared_identity_blake3, "declared_identity_blake3")?;
    let object_size_bytes = bounded_object_size(&request.object_bytes)?;
    let observed_identity_blake3 = blake3::hash(&request.object_bytes).to_hex().to_string();
    if observed_identity_blake3 != request.declared_identity_blake3 {
        return Err(validation_error(format!(
            "immutable release object identity mismatch: declared {}, observed {observed_identity_blake3}",
            request.declared_identity_blake3
        )));
    }
    let disposition = validate_existing_object(
        request.existing_object_bytes,
        &request.object_bytes,
        &request.declared_identity_blake3,
    )?;
    assert_eq!(observed_identity_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
    assert!(object_size_bytes <= RELEASE_OBJECT_BYTES_MAX);
    Ok(ImmutableReleaseObjectPlan {
        schema: IMMUTABLE_RELEASE_OBJECT_PLAN_SCHEMA.to_string(),
        object_identity_blake3: observed_identity_blake3,
        object_size_bytes,
        disposition,
    })
}

fn bounded_object_size(bytes: &[u8]) -> Result<u64, ReleaseEvidenceError> {
    let size_bytes = u64::try_from(bytes.len())
        .map_err(|_| validation_error("immutable release object size overflowed u64".to_string()))?;
    if size_bytes == 0 {
        return Err(validation_error("immutable release object must not be empty".to_string()));
    }
    if size_bytes > RELEASE_OBJECT_BYTES_MAX {
        return Err(validation_error(format!(
            "immutable release object size {size_bytes} exceeds {RELEASE_OBJECT_BYTES_MAX} bytes"
        )));
    }
    Ok(size_bytes)
}

fn validate_existing_object(
    existing_bytes: Option<Vec<u8>>,
    object_bytes: &[u8],
    declared_identity_blake3: &str,
) -> Result<ImmutableReleaseObjectDisposition, ReleaseEvidenceError> {
    let Some(existing_bytes) = existing_bytes else {
        return Ok(ImmutableReleaseObjectDisposition::PublishNew);
    };
    let existing_identity_blake3 = blake3::hash(&existing_bytes).to_hex().to_string();
    if existing_identity_blake3 != declared_identity_blake3 || existing_bytes != object_bytes {
        return Err(validation_error(format!(
            "published immutable release object {declared_identity_blake3} differs from the supplied object"
        )));
    }
    assert_eq!(existing_bytes, object_bytes);
    assert_eq!(existing_identity_blake3, declared_identity_blake3);
    Ok(ImmutableReleaseObjectDisposition::AlreadyPublishedIdentical)
}

// r[impl mantle.release.pointer]
pub fn plan_current_release_pointer(
    request: CurrentReleasePointerRequest,
) -> Result<CurrentReleasePointerPlan, ReleaseEvidenceError> {
    validate_blake3_hex(&request.target_identity_blake3, "target_identity_blake3")?;
    if !request.target_object_exists {
        return Err(validation_error(format!(
            "current release pointer target object is missing: {}",
            request.target_identity_blake3
        )));
    }
    if let Some(current) = &request.current_identity_blake3 {
        validate_blake3_hex(current, "current_identity_blake3")?;
    }
    let disposition = pointer_disposition(request.current_identity_blake3.as_deref(), &request.target_identity_blake3);
    assert!(request.target_object_exists);
    assert_eq!(request.target_identity_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
    Ok(CurrentReleasePointerPlan {
        schema: CURRENT_RELEASE_POINTER_PLAN_SCHEMA.to_string(),
        target_identity_blake3: request.target_identity_blake3,
        previous_identity_blake3: request.current_identity_blake3,
        disposition,
    })
}

fn pointer_disposition(current: Option<&str>, target: &str) -> CurrentReleasePointerDisposition {
    match current {
        None => CurrentReleasePointerDisposition::SetInitial,
        Some(value) if value == target => CurrentReleasePointerDisposition::Unchanged,
        Some(_) => CurrentReleasePointerDisposition::Switch,
    }
}

// r[impl mantle.release.pointer]
pub fn plan_release_rollback(
    request: ReleaseRollbackRequest,
) -> Result<CurrentReleasePointerPlan, ReleaseEvidenceError> {
    if request.rollback_identity_blake3 == request.current_identity_blake3 {
        return Err(validation_error(
            "release rollback identity must differ from the current release identity".to_string(),
        ));
    }
    let plan = plan_current_release_pointer(CurrentReleasePointerRequest {
        target_identity_blake3: request.rollback_identity_blake3,
        target_object_exists: request.rollback_object_exists,
        current_identity_blake3: Some(request.current_identity_blake3),
    })?;
    assert_eq!(plan.disposition, CurrentReleasePointerDisposition::Switch);
    assert!(plan.previous_identity_blake3.is_some());
    Ok(plan)
}

// r[impl mantle.release.evidence]
// r[impl mantle.release.boundary]
pub fn build_immutable_release_evidence(
    input: ImmutableReleaseEvidenceInput,
) -> Result<ImmutableReleaseEvidence, ReleaseEvidenceError> {
    validate_plan_linkage(&input.object_plan, &input.pointer_plan)?;
    let object_identity_blake3 = input.object_plan.object_identity_blake3;
    let evidence = ImmutableReleaseEvidence {
        schema: IMMUTABLE_RELEASE_EVIDENCE_SCHEMA.to_string(),
        object_identity_blake3: object_identity_blake3.clone(),
        current_pointer_identity_blake3: input.pointer_plan.target_identity_blake3,
        previous_release_identity_blake3: input.pointer_plan.previous_identity_blake3,
        release_metadata: input.release_metadata,
        digest_roles: required_digest_roles(&object_identity_blake3),
        authority_boundary: caller_owned_boundary(),
    };
    validate_immutable_release_evidence(evidence)
}

fn validate_plan_linkage(
    object_plan: &ImmutableReleaseObjectPlan,
    pointer_plan: &CurrentReleasePointerPlan,
) -> Result<(), ReleaseEvidenceError> {
    if object_plan.schema != IMMUTABLE_RELEASE_OBJECT_PLAN_SCHEMA {
        return Err(validation_error("immutable release object plan schema is unsupported".to_string()));
    }
    if pointer_plan.schema != CURRENT_RELEASE_POINTER_PLAN_SCHEMA {
        return Err(validation_error("current release pointer plan schema is unsupported".to_string()));
    }
    if object_plan.object_identity_blake3 != pointer_plan.target_identity_blake3 {
        return Err(validation_error(
            "immutable release evidence object and current pointer identities differ".to_string(),
        ));
    }
    Ok(())
}

fn required_digest_roles(identity_blake3: &str) -> Vec<ReleaseDigestRoleBinding> {
    vec![
        ReleaseDigestRoleBinding {
            role: CURRENT_POINTER_DIGEST_ROLE.to_string(),
            digest_blake3: identity_blake3.to_string(),
        },
        ReleaseDigestRoleBinding {
            role: RELEASE_OBJECT_DIGEST_ROLE.to_string(),
            digest_blake3: identity_blake3.to_string(),
        },
    ]
}

fn caller_owned_boundary() -> ImmutableReleaseAuthorityBoundary {
    ImmutableReleaseAuthorityBoundary {
        distribution_owner: ReleaseAuthorityOwner::Caller,
        deployment_owner: ReleaseAuthorityOwner::Caller,
        retention_owner: ReleaseAuthorityOwner::Caller,
        deletion_owner: ReleaseAuthorityOwner::Caller,
        release_readiness_claimed: false,
    }
}

pub fn canonical_immutable_release_evidence(
    mut evidence: ImmutableReleaseEvidence,
) -> Result<ImmutableReleaseEvidence, ReleaseEvidenceError> {
    evidence.release_metadata.sort_by(|left, right| left.name.cmp(&right.name));
    evidence.digest_roles.sort_by(|left, right| left.role.cmp(&right.role));
    validate_evidence_fields(&evidence)?;
    assert!(evidence.release_metadata.windows(2).all(|pair| pair[0].name <= pair[1].name));
    assert!(evidence.digest_roles.windows(2).all(|pair| pair[0].role <= pair[1].role));
    Ok(evidence)
}

pub fn validate_immutable_release_evidence(
    evidence: ImmutableReleaseEvidence,
) -> Result<ImmutableReleaseEvidence, ReleaseEvidenceError> {
    canonical_immutable_release_evidence(evidence)
}

pub fn immutable_release_evidence_canonical_bytes(
    evidence: ImmutableReleaseEvidence,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let canonical = canonical_immutable_release_evidence(evidence)?;
    serde_json::to_vec(&canonical)
        .map_err(|error| ReleaseEvidenceError::Parse(format!("serializing immutable release evidence: {error}")))
}

pub fn immutable_release_evidence_digest_blake3(
    evidence: ImmutableReleaseEvidence,
) -> Result<String, ReleaseEvidenceError> {
    let bytes = immutable_release_evidence_canonical_bytes(evidence)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn validate_evidence_fields(evidence: &ImmutableReleaseEvidence) -> Result<(), ReleaseEvidenceError> {
    if evidence.schema != IMMUTABLE_RELEASE_EVIDENCE_SCHEMA {
        return Err(validation_error(format!(
            "immutable release evidence schema must be {IMMUTABLE_RELEASE_EVIDENCE_SCHEMA}"
        )));
    }
    validate_blake3_hex(&evidence.object_identity_blake3, "object_identity_blake3")?;
    validate_blake3_hex(&evidence.current_pointer_identity_blake3, "current_pointer_identity_blake3")?;
    if evidence.object_identity_blake3 != evidence.current_pointer_identity_blake3 {
        return Err(validation_error(
            "immutable release evidence current pointer does not name the release object".to_string(),
        ));
    }
    validate_optional_previous_identity(&evidence.previous_release_identity_blake3)?;
    validate_metadata(&evidence.release_metadata)?;
    validate_digest_roles(&evidence.digest_roles, &evidence.object_identity_blake3)?;
    validate_authority_boundary(&evidence.authority_boundary)
}

fn validate_optional_previous_identity(identity: &Option<String>) -> Result<(), ReleaseEvidenceError> {
    if let Some(identity) = identity {
        validate_blake3_hex(identity, "previous_release_identity_blake3")?;
    }
    Ok(())
}

fn validate_metadata(metadata: &[ReleaseMetadataField]) -> Result<(), ReleaseEvidenceError> {
    let count = u32::try_from(metadata.len())
        .map_err(|_| validation_error("immutable release metadata field count overflowed u32".to_string()))?;
    if count == 0 || count > RELEASE_METADATA_FIELDS_MAX {
        return Err(validation_error(format!(
            "immutable release metadata field count must be from 1 through {RELEASE_METADATA_FIELDS_MAX}"
        )));
    }
    let mut names = BTreeSet::new();
    for field in metadata {
        validate_metadata_field(field)?;
        if !names.insert(field.name.clone()) {
            return Err(validation_error(format!(
                "immutable release metadata contains duplicate field {}",
                field.name
            )));
        }
    }
    Ok(())
}

fn validate_metadata_field(field: &ReleaseMetadataField) -> Result<(), ReleaseEvidenceError> {
    if field.name.trim().is_empty() || field.value.trim().is_empty() {
        return Err(validation_error("immutable release metadata names and values must not be empty".to_string()));
    }
    for (label, value) in [("name", &field.name), ("value", &field.value)] {
        let length = u32::try_from(value.len())
            .map_err(|_| validation_error(format!("immutable release metadata {label} length overflowed u32")))?;
        if length > RELEASE_METADATA_TEXT_BYTES_MAX {
            return Err(validation_error(format!(
                "immutable release metadata {label} exceeds {RELEASE_METADATA_TEXT_BYTES_MAX} bytes"
            )));
        }
    }
    Ok(())
}

fn validate_digest_roles(
    roles: &[ReleaseDigestRoleBinding],
    object_identity_blake3: &str,
) -> Result<(), ReleaseEvidenceError> {
    let expected = required_digest_roles(object_identity_blake3);
    if roles != expected {
        return Err(validation_error(
            "immutable release evidence digest-role bindings do not match the required object and pointer roles"
                .to_string(),
        ));
    }
    assert_eq!(roles.len(), RELEASE_DIGEST_ROLE_BINDINGS_COUNT);
    assert!(roles.iter().all(|binding| binding.digest_blake3 == object_identity_blake3));
    Ok(())
}

fn validate_authority_boundary(boundary: &ImmutableReleaseAuthorityBoundary) -> Result<(), ReleaseEvidenceError> {
    let owners = [
        boundary.distribution_owner,
        boundary.deployment_owner,
        boundary.retention_owner,
        boundary.deletion_owner,
    ];
    if owners.iter().any(|owner| *owner != ReleaseAuthorityOwner::Caller) {
        return Err(validation_error(
            "immutable release evidence must keep distribution, deployment, retention, and deletion caller-owned"
                .to_string(),
        ));
    }
    if boundary.release_readiness_claimed {
        return Err(validation_error("immutable release evidence must not claim release readiness".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST_OBJECT_BYTES: &[u8] = b"mantle-release-object-one";
    const SECOND_OBJECT_BYTES: &[u8] = b"mantle-release-object-two";
    const RELEASE_ID_FIELD: &str = "release-id";
    const RELEASE_ID_VALUE: &str = "mantle-1.0.0";

    fn identity(bytes: &[u8]) -> String {
        blake3::hash(bytes).to_hex().to_string()
    }

    fn metadata() -> Vec<ReleaseMetadataField> {
        vec![ReleaseMetadataField {
            name: RELEASE_ID_FIELD.to_string(),
            value: RELEASE_ID_VALUE.to_string(),
        }]
    }

    fn object_plan(bytes: &[u8], existing: Option<Vec<u8>>) -> ImmutableReleaseObjectPlan {
        plan_immutable_release_object(ImmutableReleaseObjectRequest {
            object_bytes: bytes.to_vec(),
            declared_identity_blake3: identity(bytes),
            existing_object_bytes: existing,
        })
        .expect("valid immutable object plan")
    }

    fn pointer_plan(target: &str, current: Option<String>) -> CurrentReleasePointerPlan {
        plan_current_release_pointer(CurrentReleasePointerRequest {
            target_identity_blake3: target.to_string(),
            target_object_exists: true,
            current_identity_blake3: current,
        })
        .expect("valid pointer plan")
    }

    fn evidence() -> ImmutableReleaseEvidence {
        let object_plan = object_plan(FIRST_OBJECT_BYTES, None);
        let pointer_plan = pointer_plan(&object_plan.object_identity_blake3, None);
        build_immutable_release_evidence(ImmutableReleaseEvidenceInput {
            object_plan,
            pointer_plan,
            release_metadata: metadata(),
        })
        .expect("valid immutable release evidence")
    }

    // r[verify mantle.release.verification]
    #[test]
    fn immutable_object_accepts_new_and_identical_publication() {
        let new_plan = object_plan(FIRST_OBJECT_BYTES, None);
        let existing_plan = object_plan(FIRST_OBJECT_BYTES, Some(FIRST_OBJECT_BYTES.to_vec()));

        assert_eq!(new_plan.disposition, ImmutableReleaseObjectDisposition::PublishNew);
        assert_eq!(existing_plan.disposition, ImmutableReleaseObjectDisposition::AlreadyPublishedIdentical);
        assert_eq!(new_plan.object_identity_blake3, existing_plan.object_identity_blake3);
        assert_eq!(new_plan.object_size_bytes, existing_plan.object_size_bytes);
    }

    // r[verify mantle.release.verification]
    #[test]
    fn different_object_identity_creates_a_distinct_plan() {
        let first = object_plan(FIRST_OBJECT_BYTES, None);
        let second = object_plan(SECOND_OBJECT_BYTES, None);

        assert_ne!(first.object_identity_blake3, second.object_identity_blake3);
        assert_eq!(first.disposition, ImmutableReleaseObjectDisposition::PublishNew);
        assert_eq!(second.disposition, ImmutableReleaseObjectDisposition::PublishNew);
    }

    // r[verify mantle.release.verification]
    #[test]
    fn mutated_object_and_declared_identity_mismatch_fail_closed() {
        let mutated = plan_immutable_release_object(ImmutableReleaseObjectRequest {
            object_bytes: FIRST_OBJECT_BYTES.to_vec(),
            declared_identity_blake3: identity(FIRST_OBJECT_BYTES),
            existing_object_bytes: Some(SECOND_OBJECT_BYTES.to_vec()),
        })
        .expect_err("mutated published object must fail");
        let mismatched = plan_immutable_release_object(ImmutableReleaseObjectRequest {
            object_bytes: FIRST_OBJECT_BYTES.to_vec(),
            declared_identity_blake3: identity(SECOND_OBJECT_BYTES),
            existing_object_bytes: None,
        })
        .expect_err("declared identity mismatch must fail");

        assert!(mutated.to_string().contains("differs from the supplied object"));
        assert!(mismatched.to_string().contains("identity mismatch"));
    }

    // r[verify mantle.release.verification]
    #[test]
    fn pointer_switch_and_rollback_preserve_previous_identity() {
        let first_identity = identity(FIRST_OBJECT_BYTES);
        let second_identity = identity(SECOND_OBJECT_BYTES);
        let switched = pointer_plan(&second_identity, Some(first_identity.clone()));
        let rollback = plan_release_rollback(ReleaseRollbackRequest {
            rollback_identity_blake3: first_identity.clone(),
            rollback_object_exists: true,
            current_identity_blake3: second_identity.clone(),
        })
        .expect("rollback plan");

        assert_eq!(switched.disposition, CurrentReleasePointerDisposition::Switch);
        assert_eq!(switched.previous_identity_blake3, Some(first_identity.clone()));
        assert_eq!(rollback.target_identity_blake3, first_identity);
        assert_eq!(rollback.previous_identity_blake3, Some(second_identity));
    }

    // r[verify mantle.release.verification]
    #[test]
    fn pointer_to_missing_object_fails_closed() {
        let target = identity(FIRST_OBJECT_BYTES);
        let error = plan_current_release_pointer(CurrentReleasePointerRequest {
            target_identity_blake3: target.clone(),
            target_object_exists: false,
            current_identity_blake3: None,
        })
        .expect_err("missing target must fail");

        assert!(error.to_string().contains("target object is missing"));
        assert!(error.to_string().contains(&target));
    }

    // r[verify mantle.release.verification]
    #[test]
    fn evidence_binds_object_pointer_metadata_and_roles() {
        let evidence = evidence();
        let digest = immutable_release_evidence_digest_blake3(evidence.clone()).expect("evidence digest");

        assert_eq!(evidence.object_identity_blake3, evidence.current_pointer_identity_blake3);
        assert_eq!(evidence.digest_roles.len(), RELEASE_DIGEST_ROLE_BINDINGS_COUNT);
        assert_eq!(evidence.release_metadata, metadata());
        assert_eq!(digest.len(), BLAKE3_HEX_LENGTH_CHARS);
    }

    // r[verify mantle.release.verification]
    #[test]
    fn digest_role_substitution_and_missing_binding_fail_closed() {
        let mut substituted = evidence();
        substituted.digest_roles[0].role = "deployment-result".to_string();
        let mut missing = evidence();
        missing.digest_roles.pop();

        let substitution_error =
            validate_immutable_release_evidence(substituted).expect_err("digest-role substitution must fail");
        let missing_error =
            validate_immutable_release_evidence(missing).expect_err("missing digest-role binding must fail");

        assert!(substitution_error.to_string().contains("digest-role bindings"));
        assert!(missing_error.to_string().contains("digest-role bindings"));
    }

    // r[verify mantle.release.verification]
    #[test]
    fn deployment_retention_and_readiness_claims_fail_closed() {
        let mut deployment = evidence();
        deployment.authority_boundary.deployment_owner = ReleaseAuthorityOwner::Mantle;
        let mut retention = evidence();
        retention.authority_boundary.retention_owner = ReleaseAuthorityOwner::Mantle;
        let mut readiness = evidence();
        readiness.authority_boundary.release_readiness_claimed = true;

        let deployment_error = validate_immutable_release_evidence(deployment).expect_err("deployment claim must fail");
        let retention_error = validate_immutable_release_evidence(retention).expect_err("retention claim must fail");
        let readiness_error = validate_immutable_release_evidence(readiness).expect_err("readiness claim must fail");

        assert!(deployment_error.to_string().contains("caller-owned"));
        assert!(retention_error.to_string().contains("caller-owned"));
        assert!(readiness_error.to_string().contains("must not claim release readiness"));
    }

    // r[verify mantle.release.verification]
    #[test]
    fn canonical_evidence_is_metadata_order_independent() {
        let object_plan = object_plan(FIRST_OBJECT_BYTES, None);
        let pointer_plan = pointer_plan(&object_plan.object_identity_blake3, None);
        let first = build_immutable_release_evidence(ImmutableReleaseEvidenceInput {
            object_plan: object_plan.clone(),
            pointer_plan: pointer_plan.clone(),
            release_metadata: vec![
                ReleaseMetadataField {
                    name: "version".to_string(),
                    value: "1".to_string(),
                },
                ReleaseMetadataField {
                    name: RELEASE_ID_FIELD.to_string(),
                    value: RELEASE_ID_VALUE.to_string(),
                },
            ],
        })
        .expect("first evidence");
        let second = build_immutable_release_evidence(ImmutableReleaseEvidenceInput {
            object_plan,
            pointer_plan,
            release_metadata: first.release_metadata.iter().cloned().rev().collect(),
        })
        .expect("second evidence");

        let first_bytes = immutable_release_evidence_canonical_bytes(first).expect("first bytes");
        let second_bytes = immutable_release_evidence_canonical_bytes(second).expect("second bytes");
        assert_eq!(first_bytes, second_bytes);
        assert!(!first_bytes.is_empty());
    }
}
