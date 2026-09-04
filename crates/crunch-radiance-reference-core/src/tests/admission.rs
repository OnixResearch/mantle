use alloc::string::ToString;

#[test]
fn exact_source_cohort_is_sorted_deterministic_and_admitted() {
    let first = super::source_cohort();
    let second = super::source_cohort();
    assert_eq!(first, second);
    assert_eq!(first.members.len(), crate::RADIANCE_SOURCE_COUNT);
    assert_eq!(first.members[0].role, crate::RadianceSourceRole::Radiance);
    assert!(crate::validate_source_cohort(first).is_ok());
}

#[test]
fn source_cohort_rejects_revision_content_and_license_drift() {
    let mut revision = super::source_member(crate::RadianceSourceRole::Radiance);
    revision.observation.immutable_revision.as_mut().unwrap().value = crate::RADIANCE_S0_REVISION.to_string();
    let mut object_format = super::source_member(crate::RadianceSourceRole::Radiance);
    object_format.observation.immutable_revision = Some(crunch_source_core::ImmutableRevisionWire {
        object_format: crunch_source_core::GitObjectFormat::Sha1,
        value: super::SHA1_REVISION.to_string(),
    });
    let mut content = super::source_member(crate::RadianceSourceRole::Radiance);
    content.observation.content_blake3 = super::DIGEST_1.to_string();
    let mut license = super::source_member(crate::RadianceSourceRole::Radiance);
    license.license_spdx = "Apache-2.0".to_string();
    assert!(
        crate::admit_source_cohort(alloc::vec![
            revision,
            super::source_member(crate::RadianceSourceRole::BootstrapCompiler),
            super::source_member(crate::RadianceSourceRole::Emulator)
        ])
        .is_err()
    );
    assert!(
        crate::admit_source_cohort(alloc::vec![
            object_format,
            super::source_member(crate::RadianceSourceRole::BootstrapCompiler),
            super::source_member(crate::RadianceSourceRole::Emulator)
        ])
        .is_err()
    );
    assert!(
        crate::admit_source_cohort(alloc::vec![
            content,
            super::source_member(crate::RadianceSourceRole::BootstrapCompiler),
            super::source_member(crate::RadianceSourceRole::Emulator)
        ])
        .is_err()
    );
    assert!(
        crate::admit_source_cohort(alloc::vec![
            license,
            super::source_member(crate::RadianceSourceRole::BootstrapCompiler),
            super::source_member(crate::RadianceSourceRole::Emulator)
        ])
        .is_err()
    );
}

#[test]
fn exact_stage_plan_requires_both_routes_and_immediate_predecessors() {
    let cohort = super::source_cohort();
    let plan = crate::plan_radiance_reference(cohort.cohort_blake3, super::DIGEST_7.to_string()).unwrap();
    let mut tampered = plan.clone();
    tampered.stages[1].predecessor_role = "radiance-seed".to_string();
    let mut unsafe_projection = plan.clone();
    unsafe_projection.stages[0].source_projection = "../radiance".to_string();
    assert_eq!(plan.stages.len(), crate::RADIANCE_STAGE_COUNT);
    assert_eq!(plan.stages[3].launch_kind, crate::RadianceLaunchKind::NativeBootstrap);
    assert!(crate::validate_radiance_reference_plan(&plan).is_ok());
    assert_eq!(crate::validate_radiance_reference_plan(&tampered), Err(crate::RadianceReferenceError::Plan));
    assert_eq!(
        crate::validate_radiance_reference_plan(&unsafe_projection),
        Err(crate::RadianceReferenceError::Plan)
    );
}

#[test]
fn exact_revisions_are_sha256_width_and_not_interchangeable() {
    let revisions = [
        crate::RADIANCE_REVISION,
        crate::RADIANCE_S0_REVISION,
        crate::RADIANCE_EMULATOR_REVISION,
    ];
    assert!(revisions.iter().all(|revision| revision.len() == crunch_source_core::GIT_SHA256_HEX_CHARS));
    assert!(revisions.iter().all(|revision| revision.bytes().all(|byte| byte.is_ascii_hexdigit())));
    assert_ne!(crate::RADIANCE_REVISION, crate::RADIANCE_S0_REVISION);
    assert_ne!(crate::RADIANCE_S0_REVISION, crate::RADIANCE_EMULATOR_REVISION);
}
