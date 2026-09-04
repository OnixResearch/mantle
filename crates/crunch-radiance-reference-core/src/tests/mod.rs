use alloc::string::ToString;

use crate::*;

mod admission;
mod verification;

const DIGEST_1: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const DIGEST_2: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const DIGEST_3: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const DIGEST_4: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const DIGEST_5: &str = "5555555555555555555555555555555555555555555555555555555555555555";
const DIGEST_6: &str = "6666666666666666666666666666666666666666666666666666666666666666";
const DIGEST_7: &str = "7777777777777777777777777777777777777777777777777777777777777777";
const DIGEST_8: &str = "8888888888888888888888888888888888888888888888888888888888888888";
const DIGEST_9: &str = "9999999999999999999999999999999999999999999999999999999999999999";
const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const ARTIFACT_BYTES: u64 = 1_285_492;
const AUDIT_EVENT_COUNT: u32 = 1;
const SHA1_REVISION: &str = "1111111111111111111111111111111111111111";

fn source_member(role: RadianceSourceRole) -> RadianceSourceMemberWire {
    let observation = crunch_source_core::build_source_observation(crunch_source_core::SourceObservationDraft {
        source_kind: crunch_source_core::SourceKind::VcsSnapshot,
        locator_class: crunch_source_core::LocatorClass::GitRemote,
        locator_hint: Some(role.expected_repository_url().to_string()),
        immutable_revision: Some(crunch_source_core::ImmutableRevisionWire {
            object_format: crunch_source_core::GitObjectFormat::Sha256,
            value: role.expected_revision().to_string(),
        }),
        normalized_projection: ".".to_string(),
        snapshot_profile: crunch_source_core::SnapshotProfile::CanonicalTreeV1,
        content_blake3: role.expected_content_blake3().to_string(),
        mutable_reference_hint: Some("refs/heads/master".to_string()),
        provenance: crunch_source_core::ProvenanceDisposition::Complete,
    })
    .unwrap()
    .into_wire();
    let member = RadianceSourceMemberWire {
        role,
        repository_url: role.expected_repository_url().to_string(),
        observation,
        license_spdx: RADIANCE_LICENSE_SPDX.to_string(),
        license_blake3: RADIANCE_MIT_LICENSE_BLAKE3.to_string(),
    };
    assert_eq!(member.role, role);
    assert_eq!(member.repository_url, role.expected_repository_url());
    member
}

fn source_cohort() -> RadianceSourceCohortWire {
    admit_source_cohort(alloc::vec![
        source_member(RadianceSourceRole::Emulator),
        source_member(RadianceSourceRole::Radiance),
        source_member(RadianceSourceRole::BootstrapCompiler),
    ])
    .unwrap()
    .into_wire()
}

fn tools() -> alloc::vec::Vec<RadianceArtifactObservation> {
    alloc::vec![
        artifact(RadianceArtifactRole::HostCompilerLauncher, DIGEST_1),
        artifact(RadianceArtifactRole::HostCompilerDriver, DIGEST_A),
        artifact(RadianceArtifactRole::HostLinker, DIGEST_2),
        artifact(RadianceArtifactRole::HostCrtInputs, DIGEST_B),
        artifact(RadianceArtifactRole::HostLibgccInputs, DIGEST_C),
        artifact(RadianceArtifactRole::BootstrapCompiler, DIGEST_3),
        artifact(RadianceArtifactRole::Emulator, DIGEST_4),
        artifact(RadianceArtifactRole::Seed, DIGEST_5),
    ]
}

fn artifact(role: RadianceArtifactRole, digest: &str) -> RadianceArtifactObservation {
    RadianceArtifactObservation {
        role,
        digest_blake3: digest.to_string(),
        byte_count: ARTIFACT_BYTES,
    }
}

struct StageFixture<'a> {
    route: RadianceRoute,
    stage_index: u8,
    predecessor_blake3: &'a str,
    output_blake3: &'a str,
}

fn stages(c99_fixed_digest: &str) -> alloc::vec::Vec<RadianceStageObservation> {
    let observations = alloc::vec![
        stage(StageFixture {
            route: RadianceRoute::Seed,
            stage_index: RADIANCE_FIRST_STAGE,
            predecessor_blake3: DIGEST_5,
            output_blake3: DIGEST_7,
        }),
        stage(StageFixture {
            route: RadianceRoute::Seed,
            stage_index: RADIANCE_CONVERGENCE_LEFT_STAGE,
            predecessor_blake3: DIGEST_7,
            output_blake3: DIGEST_8,
        }),
        stage(StageFixture {
            route: RadianceRoute::Seed,
            stage_index: RADIANCE_CONVERGENCE_RIGHT_STAGE,
            predecessor_blake3: DIGEST_8,
            output_blake3: DIGEST_8,
        }),
        stage(StageFixture {
            route: RadianceRoute::C99,
            stage_index: RADIANCE_FIRST_STAGE,
            predecessor_blake3: DIGEST_3,
            output_blake3: DIGEST_9,
        }),
        stage(StageFixture {
            route: RadianceRoute::C99,
            stage_index: RADIANCE_CONVERGENCE_LEFT_STAGE,
            predecessor_blake3: DIGEST_9,
            output_blake3: c99_fixed_digest,
        }),
        stage(StageFixture {
            route: RadianceRoute::C99,
            stage_index: RADIANCE_CONVERGENCE_RIGHT_STAGE,
            predecessor_blake3: c99_fixed_digest,
            output_blake3: c99_fixed_digest,
        }),
    ];
    assert_eq!(observations.len(), RADIANCE_STAGE_COUNT);
    assert!(observations.iter().all(|observation| valid_blake3(&observation.output_blake3)));
    observations
}

fn stage(fixture: StageFixture<'_>) -> RadianceStageObservation {
    let launch_kind = if fixture.route == RadianceRoute::C99 && fixture.stage_index == RADIANCE_FIRST_STAGE {
        RadianceLaunchKind::NativeBootstrap
    } else {
        RadianceLaunchKind::EmulatedCompiler
    };
    let launcher_blake3 = match launch_kind {
        RadianceLaunchKind::NativeBootstrap => DIGEST_3,
        RadianceLaunchKind::EmulatedCompiler => DIGEST_4,
    };
    let observation = RadianceStageObservation {
        route: fixture.route,
        stage: fixture.stage_index,
        launch_kind,
        launcher_blake3: launcher_blake3.to_string(),
        predecessor_blake3: fixture.predecessor_blake3.to_string(),
        output_role: alloc::format!("{}-stage-{}", route_label(fixture.route), fixture.stage_index),
        output_blake3: fixture.output_blake3.to_string(),
        output_bytes: ARTIFACT_BYTES,
        exit_code: 0,
        stdout_blake3: DIGEST_1.to_string(),
        stderr_blake3: DIGEST_2.to_string(),
        protected_audit_blake3: DIGEST_6.to_string(),
        protected_audit_event_count: AUDIT_EVENT_COUNT,
        denied_event_count: 0,
    };
    assert_eq!(observation.route, fixture.route);
    assert_eq!(observation.stage, fixture.stage_index);
    observation
}

fn receipt(c99_fixed_digest: &str) -> RadianceReferenceReceipt {
    let source_cohort = source_cohort();
    let plan = plan_radiance_reference(source_cohort.cohort_blake3.clone(), DIGEST_7.to_string()).unwrap();
    let stages = stages(c99_fixed_digest);
    let seed_left = &stages[1];
    let seed_right = &stages[2];
    let c99_left = &stages[4];
    let c99_right = &stages[5];
    let route_convergence = alloc::vec![
        classify_route_convergence(RadianceRoute::Seed, seed_left, seed_right).unwrap(),
        classify_route_convergence(RadianceRoute::C99, c99_left, c99_right).unwrap(),
    ];
    let cross_route = classify_cross_route(seed_right, c99_right).unwrap();
    let publication = alloc::vec![
        publication(RadiancePublicationRole::SeedRouteFixedPoint, DIGEST_8),
        publication(RadiancePublicationRole::C99RouteFixedPoint, c99_fixed_digest),
        publication(RadiancePublicationRole::BootstrapCompiler, DIGEST_3),
        publication(RadiancePublicationRole::Emulator, DIGEST_4),
    ];
    let sealed = seal_radiance_reference_receipt(RadianceReferenceReceiptDraft {
        source_cohort,
        source_bundle_blake3: DIGEST_7.to_string(),
        source_state_blake3: DIGEST_6.to_string(),
        plan,
        tools: tools(),
        stages,
        route_convergence,
        cross_route,
        zero_events: RadianceZeroEventCounts {
            live_fetches: 0,
            source_fallbacks: 0,
            substitutions: 0,
            ambient_discoveries: 0,
        },
        publication,
        protected_execution_audit_blake3: DIGEST_5.to_string(),
    })
    .unwrap();
    assert_eq!(sealed.tools.len(), RADIANCE_TOOL_COUNT);
    assert_eq!(sealed.stages.len(), RADIANCE_STAGE_COUNT);
    sealed
}

fn publication(role: RadiancePublicationRole, digest: &str) -> RadiancePublicationArtifact {
    RadiancePublicationArtifact {
        role,
        relative_path: alloc::format!("objects/{digest}-artifact"),
        digest_blake3: digest.to_string(),
        byte_count: ARTIFACT_BYTES,
    }
}
