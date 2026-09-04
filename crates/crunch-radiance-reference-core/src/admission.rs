use alloc::string::ToString;

const SOURCE_COHORT_DOMAIN: &[u8] = b"mantle.radiance-reference.source-cohort.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadianceSourceCohort {
    wire: crate::RadianceSourceCohortWire,
}

impl RadianceSourceCohort {
    #[must_use]
    pub fn as_wire(&self) -> &crate::RadianceSourceCohortWire {
        debug_assert_eq!(self.wire.schema, crate::RADIANCE_REFERENCE_COHORT_SCHEMA);
        debug_assert!(crate::valid_blake3(&self.wire.cohort_blake3));
        &self.wire
    }

    #[must_use]
    pub fn into_wire(self) -> crate::RadianceSourceCohortWire {
        debug_assert_eq!(self.wire.members.len(), crate::RADIANCE_SOURCE_COUNT);
        debug_assert_eq!(self.wire.non_claim, crate::RADIANCE_REFERENCE_NON_CLAIM);
        self.wire
    }
}

#[derive(serde::Serialize)]
struct CanonicalCohort<'a> {
    schema: &'static str,
    members: alloc::vec::Vec<CanonicalMember<'a>>,
    non_claim: &'static str,
}

#[derive(serde::Serialize)]
struct CanonicalMember<'a> {
    role: crate::RadianceSourceRole,
    observation: crunch_source_core::SourceObservationSubject,
    license_spdx: &'a str,
    license_blake3: &'a str,
}

pub fn admit_source_cohort(
    mut members: alloc::vec::Vec<crate::RadianceSourceMemberWire>,
) -> Result<RadianceSourceCohort, crate::RadianceReferenceError> {
    if members.len() != crate::RADIANCE_SOURCE_COUNT {
        return Err(crate::RadianceReferenceError::SourceCount);
    }
    members.sort_by_key(|member| member.role);
    validate_role_order(&members)?;
    for member in &members {
        validate_member(member)?;
    }
    let cohort_blake3 = cohort_identity(&members)?;
    let wire = crate::RadianceSourceCohortWire {
        schema: crate::RADIANCE_REFERENCE_COHORT_SCHEMA.to_string(),
        members,
        cohort_blake3,
        non_claim: crate::RADIANCE_REFERENCE_NON_CLAIM.to_string(),
    };
    debug_assert_eq!(wire.members.len(), crate::RADIANCE_SOURCE_COUNT);
    debug_assert!(crate::valid_blake3(&wire.cohort_blake3));
    Ok(RadianceSourceCohort { wire })
}

pub fn validate_source_cohort(
    wire: crate::RadianceSourceCohortWire,
) -> Result<RadianceSourceCohort, crate::RadianceReferenceError> {
    if wire.schema != crate::RADIANCE_REFERENCE_COHORT_SCHEMA {
        return Err(crate::RadianceReferenceError::Schema);
    }
    if wire.non_claim != crate::RADIANCE_REFERENCE_NON_CLAIM {
        return Err(crate::RadianceReferenceError::NonClaim);
    }
    let expected = admit_source_cohort(wire.members.clone())?;
    if wire.cohort_blake3 != expected.as_wire().cohort_blake3 {
        return Err(crate::RadianceReferenceError::SourceIdentity);
    }
    debug_assert_eq!(wire.members, expected.as_wire().members);
    debug_assert!(crate::valid_blake3(&wire.cohort_blake3));
    Ok(RadianceSourceCohort { wire })
}

fn validate_role_order(members: &[crate::RadianceSourceMemberWire]) -> Result<(), crate::RadianceReferenceError> {
    let expected = [
        crate::RadianceSourceRole::Radiance,
        crate::RadianceSourceRole::BootstrapCompiler,
        crate::RadianceSourceRole::Emulator,
    ];
    for (index, role) in expected.iter().enumerate() {
        if members.get(index).map(|member| member.role) != Some(*role) {
            return Err(crate::RadianceReferenceError::SourceRole);
        }
    }
    debug_assert_eq!(members.len(), expected.len());
    debug_assert!(members.windows(2).all(|window| window[0].role < window[1].role));
    Ok(())
}

fn validate_member(member: &crate::RadianceSourceMemberWire) -> Result<(), crate::RadianceReferenceError> {
    if member.repository_url != member.role.expected_repository_url() {
        return Err(crate::RadianceReferenceError::SourceIdentity);
    }
    if member.license_spdx != crate::RADIANCE_LICENSE_SPDX
        || member.license_blake3 != crate::RADIANCE_MIT_LICENSE_BLAKE3
    {
        return Err(crate::RadianceReferenceError::SourceLicense);
    }
    let observation = crunch_source_core::admit_source_observation(member.observation.clone())
        .map_err(|_| crate::RadianceReferenceError::SourceObservation)?;
    validate_observation(member, observation.as_wire())?;
    debug_assert_eq!(observation.as_wire().content_blake3, member.role.expected_content_blake3());
    debug_assert_eq!(member.license_spdx, crate::RADIANCE_LICENSE_SPDX);
    Ok(())
}

fn validate_observation(
    member: &crate::RadianceSourceMemberWire,
    observation: &crunch_source_core::SourceObservationWire,
) -> Result<(), crate::RadianceReferenceError> {
    let revision = observation.immutable_revision.as_ref().ok_or(crate::RadianceReferenceError::SourceObservation)?;
    if observation.source_kind != crunch_source_core::SourceKind::VcsSnapshot
        || observation.locator_class != crunch_source_core::LocatorClass::GitRemote
    {
        return Err(crate::RadianceReferenceError::SourceObservation);
    }
    if revision.object_format != crunch_source_core::GitObjectFormat::Sha256
        || revision.value != member.role.expected_revision()
    {
        return Err(crate::RadianceReferenceError::SourceIdentity);
    }
    if observation.normalized_projection != "."
        || observation.snapshot_profile != crunch_source_core::SnapshotProfile::CanonicalTreeV1
        || observation.provenance != crunch_source_core::ProvenanceDisposition::Complete
    {
        return Err(crate::RadianceReferenceError::SourceObservation);
    }
    if observation.content_blake3 != member.role.expected_content_blake3() {
        return Err(crate::RadianceReferenceError::SourceIdentity);
    }
    if observation.locator_hint.as_deref() != Some(member.repository_url.as_str()) {
        return Err(crate::RadianceReferenceError::SourceObservation);
    }
    debug_assert_eq!(revision.value.len(), crunch_source_core::GIT_SHA256_HEX_CHARS);
    debug_assert!(crate::valid_blake3(&observation.content_blake3));
    Ok(())
}

fn cohort_identity(
    members: &[crate::RadianceSourceMemberWire],
) -> Result<alloc::string::String, crate::RadianceReferenceError> {
    let canonical_members = members
        .iter()
        .map(|member| CanonicalMember {
            role: member.role,
            observation: crunch_source_core::source_observation_subject(&member.observation),
            license_spdx: &member.license_spdx,
            license_blake3: &member.license_blake3,
        })
        .collect::<alloc::vec::Vec<_>>();
    let canonical = CanonicalCohort {
        schema: crate::RADIANCE_REFERENCE_COHORT_SCHEMA,
        members: canonical_members,
        non_claim: crate::RADIANCE_REFERENCE_NON_CLAIM,
    };
    let digest = crate::canonical_blake3(SOURCE_COHORT_DOMAIN, &canonical)?;
    debug_assert!(crate::valid_blake3(&digest));
    debug_assert_eq!(members.len(), crate::RADIANCE_SOURCE_COUNT);
    Ok(digest)
}
