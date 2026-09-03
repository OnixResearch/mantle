use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;

const DIGEST_HEX_CHARS: usize = 64;
const POLICY_JOBS_MAX: u32 = 16;
const OBSERVED_JOBS: u32 = 12;
const EXECUTOR_JOBS_MAX: u32 = 8;
const REQUESTED_JOBS: u32 = 10;
const ROUTE_MATRIX_SCHEMA: &str = "mantle-build-planning-route-matrix-v1";

#[derive(Deserialize)]
struct RouteMatrix {
    schema: String,
    cases: Vec<RouteMatrixCase>,
}

#[derive(Deserialize)]
struct RouteMatrixCase {
    name: String,
    expected_route: crate::RouteClass,
}

fn digest(byte: char) -> String {
    core::iter::repeat_n(byte, DIGEST_HEX_CHARS).collect()
}

fn default_facts() -> crate::BuildPlanningFacts {
    crate::BuildPlanningFacts {
        local_output: crate::ObservedFact::Present(crate::LocalOutputObservation {
            present: false,
            content_complete: false,
            trusted: false,
        }),
        source_readiness: crate::ObservedFact::Present(crate::SourceReadinessObservation {
            required: false,
            ready: true,
            source_identity_blake3: Some(digest('a')),
        }),
        substituter: crate::ObservedFact::Present(crate::SubstituterObservation {
            configured: false,
            candidate_available: false,
            trusted: false,
            requires_network: true,
        }),
        archive: crate::ObservedFact::Present(crate::ArchiveObservation {
            configured: false,
            candidate_available: false,
            prefix_matches: false,
            signature_trusted: false,
        }),
        remote_candidate: crate::ObservedFact::Present(crate::RemoteCandidateObservation {
            candidate_identity_blake3: digest('b'),
            configured: false,
            credential_present: false,
            capabilities_match: false,
            source_inputs_ready: false,
            output_trusted: false,
            requires_network: true,
            upload_summary: crate::UploadSummary {
                classes: Vec::new(),
                object_count: 0,
                byte_count: 0,
            },
        }),
        doctor: crate::ObservedFact::Present(crate::DoctorObservation {
            preflight_ok: true,
            report_identity_blake3: digest('c'),
        }),
        platform: crate::ObservedFact::Present(crate::PlatformObservation {
            supported: true,
            platform_identity: "x86_64-linux".to_string(),
        }),
        trust: crate::ObservedFact::Present(crate::TrustObservation {
            trusted_output_key_count: 1,
            trust_policy_identity_blake3: digest('d'),
        }),
        network: crate::ObservedFact::Present(crate::NetworkObservation { online: true }),
        executor: crate::ObservedFact::Present(crate::ExecutorObservation {
            local_available: true,
            remote_available: true,
            executor_limit: Some(EXECUTOR_JOBS_MAX),
        }),
    }
}

fn request(facts: crate::BuildPlanningFacts) -> crate::BuildPlanningRequest {
    crate::BuildPlanningRequest {
        facts,
        policy: crate::BuildPlanningPolicy::practical_local(),
        parallelism: crate::ParallelismFacts {
            requested_jobs: None,
            observed_parallelism: crate::ParallelismObservation::Available(OBSERVED_JOBS),
            policy_jobs_max: POLICY_JOBS_MAX,
            executor_jobs_max: Some(EXECUTOR_JOBS_MAX),
            unavailable_policy: crate::UnavailableParallelismPolicy::UseFallback(crate::DEFAULT_FALLBACK_JOBS),
            zero_request_policy: crate::ZeroRequestPolicy::ClampToOne,
        },
    }
}

fn selected_route(facts: crate::BuildPlanningFacts) -> crate::RouteClass {
    crate::plan_build(request(facts)).unwrap().route_plan.selected_route
}

#[test]
fn checked_route_matrix_preserves_preference_order() {
    let mut local = default_facts();
    local.local_output = crate::ObservedFact::Present(crate::LocalOutputObservation {
        present: true,
        content_complete: true,
        trusted: true,
    });
    let mut substitute = default_facts();
    substitute.substituter = crate::ObservedFact::Present(crate::SubstituterObservation {
        configured: true,
        candidate_available: true,
        trusted: true,
        requires_network: true,
    });
    let mut archive = default_facts();
    archive.archive = crate::ObservedFact::Present(crate::ArchiveObservation {
        configured: true,
        candidate_available: true,
        prefix_matches: true,
        signature_trusted: true,
    });
    let mut source = default_facts();
    source.source_readiness = crate::ObservedFact::Present(crate::SourceReadinessObservation {
        required: true,
        ready: true,
        source_identity_blake3: Some(digest('e')),
    });
    let mut remote = default_facts();
    remote.remote_candidate = crate::ObservedFact::Present(eligible_remote());
    let local_build = default_facts();
    let mut preflight = default_facts();
    preflight.doctor = crate::ObservedFact::Present(crate::DoctorObservation {
        preflight_ok: false,
        report_identity_blake3: digest('f'),
    });
    assert_eq!(selected_route(local), crate::RouteClass::CachedLocal);
    assert_eq!(selected_route(substitute), crate::RouteClass::TrustedSubstitute);
    assert_eq!(selected_route(archive), crate::RouteClass::ArchiveImport);
    assert_eq!(selected_route(source), crate::RouteClass::SourceBundle);
    assert_eq!(selected_route(remote), crate::RouteClass::P2pRemoteBuilder);
    assert_eq!(selected_route(local_build), crate::RouteClass::LocalBuild);
    assert_eq!(selected_route(preflight), crate::RouteClass::PreflightError);
}

#[test]
fn route_plan_returns_bound_effects_without_execution_claim() {
    let decision = crate::plan_build(request(default_facts())).unwrap();
    assert_eq!(decision.route_plan.selected_route, crate::RouteClass::LocalBuild);
    assert_eq!(decision.effects.len(), crate::BUILD_PLANNING_EFFECTS_MAX);
    assert_eq!(decision.effects[0].kind, crate::BuildPlanningEffectKind::RecheckFacts);
    assert_eq!(decision.effects[1].kind, crate::BuildPlanningEffectKind::LaunchLocalBuild);
    assert!(decision.non_claim.contains("does-not-prove-effect-execution"));
}

#[test]
fn equivalent_upload_discovery_order_produces_one_plan() {
    let mut left = default_facts();
    let mut right = default_facts();
    let mut remote = eligible_remote();
    remote.upload_summary.classes = vec![crate::UploadClass::StoreObject, crate::UploadClass::Source];
    left.remote_candidate = crate::ObservedFact::Present(remote.clone());
    remote.upload_summary.classes.reverse();
    right.remote_candidate = crate::ObservedFact::Present(remote);
    let left = crate::plan_build(request(left)).unwrap();
    let right = crate::plan_build(request(right)).unwrap();
    assert_eq!(left.facts_blake3, right.facts_blake3);
    assert_eq!(left.plan_blake3, right.plan_blake3);
}

#[test]
fn simultaneous_route_failures_keep_typed_ordered_rejections() {
    let mut facts = default_facts();
    facts.source_readiness = crate::ObservedFact::Present(crate::SourceReadinessObservation {
        required: true,
        ready: false,
        source_identity_blake3: Some(digest('1')),
    });
    let decision = crate::plan_build(request(facts)).unwrap();
    let reasons = decision.rejected_routes.iter().map(|rejection| rejection.reason_code.as_str()).collect::<Vec<_>>();
    assert!(reasons.contains(&"local-output-missing"));
    assert!(reasons.contains(&"missing-source-state"));
    assert!(reasons.contains(&"offline-source-readiness-incomplete"));
    assert_eq!(decision.route_plan.selected_route, crate::RouteClass::PreflightError);
}

#[test]
fn missing_observation_blocks_without_effects() {
    let mut facts = default_facts();
    facts.trust = crate::ObservedFact::Missing;
    let decision = crate::plan_build(request(facts)).unwrap();
    assert!(decision.blockers.contains(&crate::BuildPlanningBlocker::MissingObservation {
        observation: crate::ObservationKind::Trust,
    }));
    assert!(decision.effects.is_empty());
    assert_eq!(decision.route_plan.selected_route, crate::RouteClass::PreflightError);
}

#[test]
fn offline_network_route_is_rejected_before_local_build() {
    let mut facts = default_facts();
    facts.substituter = crate::ObservedFact::Present(crate::SubstituterObservation {
        configured: true,
        candidate_available: true,
        trusted: true,
        requires_network: true,
    });
    facts.network = crate::ObservedFact::Present(crate::NetworkObservation { online: false });
    let decision = crate::plan_build(request(facts)).unwrap();
    assert_eq!(decision.route_plan.selected_route, crate::RouteClass::LocalBuild);
    assert!(decision.rejected_routes.iter().any(|rejection| {
        rejection.route == crate::RouteClass::TrustedSubstitute
            && rejection.blocker == crate::BuildPlanningBlocker::OfflineNetworkNeed
    }));
}

#[test]
fn remote_output_trust_gap_remains_typed() {
    let mut facts = default_facts();
    let mut remote = eligible_remote();
    remote.output_trusted = false;
    facts.remote_candidate = crate::ObservedFact::Present(remote);
    let decision = crate::plan_build(request(facts)).unwrap();
    assert_eq!(decision.route_plan.selected_route, crate::RouteClass::LocalBuild);
    assert!(decision.rejected_routes.iter().any(|rejection| {
        rejection.route == crate::RouteClass::P2pRemoteBuilder
            && rejection.blocker == crate::BuildPlanningBlocker::OutputTrustGap
    }));
}

#[test]
fn stale_facts_and_wrong_effects_fail_closed() {
    let decision = crate::plan_build(request(default_facts())).unwrap();
    let stale = crate::validate_plan_freshness(&decision, &digest('2'));
    let effect = decision.effects[1].clone();
    let wrong_identity = crate::accept_effect_observation(&effect, crate::BuildPlanningEffectObservation {
        effect_id_blake3: digest('3'),
        selected_route: effect.selected_route,
        kind: effect.kind,
        succeeded: true,
        reason_code: "observed".to_string(),
    });
    assert_eq!(stale, Err(crate::BuildPlanningBlocker::StalePlan));
    assert_eq!(wrong_identity, Err(crate::BuildPlanningBlocker::WrongEffectIdentity));
}

#[test]
fn silent_route_substitution_fails_closed() {
    let decision = crate::plan_build(request(default_facts())).unwrap();
    let effect = decision.effects[1].clone();
    let result = crate::accept_effect_observation(&effect, crate::BuildPlanningEffectObservation {
        effect_id_blake3: effect.effect_id_blake3.clone(),
        selected_route: crate::RouteClass::P2pRemoteBuilder,
        kind: effect.kind,
        succeeded: true,
        reason_code: "observed".to_string(),
    });
    assert_eq!(result, Err(crate::BuildPlanningBlocker::SilentRouteSubstitution));
}

#[test]
fn parallelism_uses_explicit_request_and_caps() {
    let decision = crate::plan_parallelism(crate::ParallelismFacts {
        requested_jobs: Some(REQUESTED_JOBS),
        observed_parallelism: crate::ParallelismObservation::Available(OBSERVED_JOBS),
        policy_jobs_max: POLICY_JOBS_MAX,
        executor_jobs_max: Some(EXECUTOR_JOBS_MAX),
        unavailable_policy: crate::UnavailableParallelismPolicy::Block,
        zero_request_policy: crate::ZeroRequestPolicy::ClampToOne,
    })
    .unwrap();
    assert_eq!(decision.effective_jobs, EXECUTOR_JOBS_MAX);
    assert_eq!(decision.reason_codes, vec!["requested-jobs", "executor-cap"]);
}

#[test]
fn parallelism_unavailable_uses_declared_fallback() {
    const FALLBACK_JOBS: u32 = 3;
    let decision = crate::plan_parallelism(crate::ParallelismFacts {
        requested_jobs: None,
        observed_parallelism: crate::ParallelismObservation::Unavailable,
        policy_jobs_max: POLICY_JOBS_MAX,
        executor_jobs_max: None,
        unavailable_policy: crate::UnavailableParallelismPolicy::UseFallback(FALLBACK_JOBS),
        zero_request_policy: crate::ZeroRequestPolicy::Block,
    })
    .unwrap();
    assert_eq!(decision.effective_jobs, FALLBACK_JOBS);
    assert_eq!(decision.reason_codes, vec!["unavailable-fallback"]);
}

#[test]
fn parallelism_conversion_zero_and_bounds_fail_typed() {
    let conversion = crate::plan_parallelism(crate::ParallelismFacts {
        requested_jobs: None,
        observed_parallelism: crate::ParallelismObservation::ConversionFailed,
        policy_jobs_max: POLICY_JOBS_MAX,
        executor_jobs_max: None,
        unavailable_policy: crate::UnavailableParallelismPolicy::Block,
        zero_request_policy: crate::ZeroRequestPolicy::Block,
    });
    let zero = crate::plan_parallelism(crate::ParallelismFacts {
        requested_jobs: Some(0),
        observed_parallelism: crate::ParallelismObservation::Unavailable,
        policy_jobs_max: POLICY_JOBS_MAX,
        executor_jobs_max: None,
        unavailable_policy: crate::UnavailableParallelismPolicy::Block,
        zero_request_policy: crate::ZeroRequestPolicy::Block,
    });
    let excessive = crate::plan_parallelism(crate::ParallelismFacts {
        requested_jobs: Some(1),
        observed_parallelism: crate::ParallelismObservation::Unavailable,
        policy_jobs_max: crate::ABSOLUTE_JOBS_MAX.saturating_add(1),
        executor_jobs_max: None,
        unavailable_policy: crate::UnavailableParallelismPolicy::Block,
        zero_request_policy: crate::ZeroRequestPolicy::ClampToOne,
    });
    assert_eq!(conversion, Err(crate::ParallelismBlocker::ParallelismConversionFailed));
    assert_eq!(zero, Err(crate::ParallelismBlocker::RequestedJobsZero));
    assert_eq!(excessive, Err(crate::ParallelismBlocker::PolicyLimitExceeded));
}

#[test]
fn route_matrix_fixture_matches_extracted_policy() {
    let matrix: RouteMatrix =
        serde_json::from_str(include_str!("../../../fixtures/build-planning/golden/route-matrix.json")).unwrap();
    assert_eq!(matrix.schema, ROUTE_MATRIX_SCHEMA);
    assert_eq!(matrix.cases.len(), crate::MAX_ROUTE_CANDIDATES);
    for case in matrix.cases {
        assert_eq!(selected_route(facts_for_case(&case.name)), case.expected_route, "{}", case.name);
    }
}

fn facts_for_case(name: &str) -> crate::BuildPlanningFacts {
    let mut facts = default_facts();
    match name {
        "local-cache" => {
            facts.local_output = crate::ObservedFact::Present(crate::LocalOutputObservation {
                present: true,
                content_complete: true,
                trusted: true,
            });
        }
        "substitution" => {
            facts.substituter = crate::ObservedFact::Present(crate::SubstituterObservation {
                configured: true,
                candidate_available: true,
                trusted: true,
                requires_network: true,
            });
        }
        "archive" => {
            facts.archive = crate::ObservedFact::Present(crate::ArchiveObservation {
                configured: true,
                candidate_available: true,
                prefix_matches: true,
                signature_trusted: true,
            });
        }
        "source-bundle" => {
            facts.source_readiness = crate::ObservedFact::Present(crate::SourceReadinessObservation {
                required: true,
                ready: true,
                source_identity_blake3: Some(digest('5')),
            });
        }
        "remote-builder" => facts.remote_candidate = crate::ObservedFact::Present(eligible_remote()),
        "local-build" => {}
        "preflight" => {
            facts.doctor = crate::ObservedFact::Present(crate::DoctorObservation {
                preflight_ok: false,
                report_identity_blake3: digest('6'),
            });
        }
        _ => panic!("unknown route fixture case: {name}"),
    }
    facts
}

fn eligible_remote() -> crate::RemoteCandidateObservation {
    crate::RemoteCandidateObservation {
        candidate_identity_blake3: digest('4'),
        configured: true,
        credential_present: true,
        capabilities_match: true,
        source_inputs_ready: true,
        output_trusted: true,
        requires_network: true,
        upload_summary: crate::UploadSummary {
            classes: Vec::new(),
            object_count: 0,
            byte_count: 0,
        },
    }
}
