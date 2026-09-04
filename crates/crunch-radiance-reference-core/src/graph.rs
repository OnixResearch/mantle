use alloc::string::ToString;

const REFERENCE_PLAN_DOMAIN: &[u8] = b"mantle.radiance-reference.plan.v1";

pub fn plan_radiance_reference(
    source_cohort_blake3: alloc::string::String,
    source_bundle_blake3: alloc::string::String,
) -> Result<crate::RadianceReferencePlan, crate::RadianceReferenceError> {
    if !crate::valid_blake3(&source_cohort_blake3) || !crate::valid_blake3(&source_bundle_blake3) {
        return Err(crate::RadianceReferenceError::Digest);
    }
    let stages = expected_stages();
    let mut plan = crate::RadianceReferencePlan {
        schema: crate::RADIANCE_REFERENCE_PLAN_SCHEMA.to_string(),
        source_cohort_blake3,
        source_bundle_blake3,
        stages,
        plan_blake3: alloc::string::String::new(),
        non_claim: crate::RADIANCE_REFERENCE_NON_CLAIM.to_string(),
    };
    plan.plan_blake3 = plan_identity(&plan)?;
    debug_assert_eq!(plan.stages.len(), crate::RADIANCE_STAGE_COUNT);
    debug_assert!(crate::valid_blake3(&plan.plan_blake3));
    Ok(plan)
}

pub fn validate_radiance_reference_plan(
    plan: &crate::RadianceReferencePlan,
) -> Result<(), crate::RadianceReferenceError> {
    if plan.schema != crate::RADIANCE_REFERENCE_PLAN_SCHEMA || plan.non_claim != crate::RADIANCE_REFERENCE_NON_CLAIM {
        return Err(crate::RadianceReferenceError::Plan);
    }
    if !crate::valid_blake3(&plan.source_cohort_blake3) || !crate::valid_blake3(&plan.source_bundle_blake3) {
        return Err(crate::RadianceReferenceError::Digest);
    }
    if plan.stages != expected_stages() {
        return Err(crate::RadianceReferenceError::Plan);
    }
    let expected_blake3 = plan_identity(plan)?;
    if plan.plan_blake3 != expected_blake3 {
        return Err(crate::RadianceReferenceError::Plan);
    }
    debug_assert_eq!(plan.stages.len(), crate::RADIANCE_STAGE_COUNT);
    debug_assert!(crate::valid_blake3(&plan.plan_blake3));
    Ok(())
}

fn expected_stages() -> alloc::vec::Vec<crate::RadianceStagePlan> {
    let mut stages = alloc::vec::Vec::with_capacity(crate::RADIANCE_STAGE_COUNT);
    stages.extend(route_stages(crate::RadianceRoute::Seed));
    stages.extend(route_stages(crate::RadianceRoute::C99));
    debug_assert_eq!(stages.len(), crate::RADIANCE_STAGE_COUNT);
    debug_assert_eq!(stages.first().map(|stage| stage.stage), Some(crate::RADIANCE_FIRST_STAGE));
    stages
}

fn route_stages(route: crate::RadianceRoute) -> alloc::vec::Vec<crate::RadianceStagePlan> {
    let route_stage_count = usize::from(crate::RADIANCE_STAGES_PER_ROUTE);
    let mut stages = alloc::vec::Vec::with_capacity(route_stage_count);
    for stage in crate::RADIANCE_FIRST_STAGE..=crate::RADIANCE_STAGES_PER_ROUTE {
        let launch_kind = if route == crate::RadianceRoute::C99 && stage == crate::RADIANCE_FIRST_STAGE {
            crate::RadianceLaunchKind::NativeBootstrap
        } else {
            crate::RadianceLaunchKind::EmulatedCompiler
        };
        let predecessor_role = if stage == crate::RADIANCE_FIRST_STAGE {
            match route {
                crate::RadianceRoute::Seed => "radiance-seed".to_string(),
                crate::RadianceRoute::C99 => "radiance-s0".to_string(),
            }
        } else {
            alloc::format!("{}-stage-{}", route_label(route), stage.saturating_sub(1))
        };
        stages.push(crate::RadianceStagePlan {
            route,
            stage,
            launch_kind,
            predecessor_role,
            source_projection: ".".to_string(),
            output_role: alloc::format!("{}-stage-{stage}", route_label(route)),
        });
    }
    debug_assert_eq!(stages.len(), route_stage_count);
    debug_assert!(stages.capacity() >= stages.len());
    stages
}

#[must_use]
pub const fn route_label(route: crate::RadianceRoute) -> &'static str {
    match route {
        crate::RadianceRoute::Seed => "seed",
        crate::RadianceRoute::C99 => "c99",
    }
}

fn plan_identity(plan: &crate::RadianceReferencePlan) -> Result<alloc::string::String, crate::RadianceReferenceError> {
    let mut candidate = plan.clone();
    candidate.plan_blake3.clear();
    let digest = crate::canonical_blake3(REFERENCE_PLAN_DOMAIN, &candidate)?;
    debug_assert!(crate::valid_blake3(&digest));
    debug_assert!(candidate.plan_blake3.is_empty());
    Ok(digest)
}
