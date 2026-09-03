use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const BUILD_PLANNING_SCHEMA: &str = "mantle-build-planning-decision-v1";
pub const BUILD_PLANNING_NON_CLAIM: &str =
    "planning-does-not-prove-effect-execution-output-admission-route-optimality-or-release-eligibility";
pub const BUILD_PLANNING_EFFECTS_MAX: usize = 2;
const OBSERVATION_KIND_COUNT: usize = 10;
const BLAKE3_HEX_CHARS: usize = 64;
const DOMAIN_SEPARATOR: u8 = 0;
const FACT_DOMAIN: &[u8] = b"mantle.build-planning.facts.v1";
const EFFECT_DOMAIN: &[u8] = b"mantle.build-planning.effect.v1";
const PLAN_DOMAIN: &[u8] = b"mantle.build-planning.plan.v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuildPlanningPolicy {
    pub requested_claim_strength: crate::ClaimStrength,
    pub allow_source_uploads: bool,
    pub allow_store_object_uploads: bool,
    pub allow_proof_uploads: bool,
}

impl BuildPlanningPolicy {
    #[must_use]
    pub const fn practical_local() -> Self {
        Self {
            requested_claim_strength: crate::ClaimStrength::Practical,
            allow_source_uploads: false,
            allow_store_object_uploads: false,
            allow_proof_uploads: false,
        }
    }

    const fn upload_policy(self) -> crate::UploadPolicy {
        crate::UploadPolicy {
            allow_sources: self.allow_source_uploads,
            allow_store_objects: self.allow_store_object_uploads,
            allow_proofs: self.allow_proof_uploads,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuildPlanningRequest {
    pub facts: crate::BuildPlanningFacts,
    pub policy: BuildPlanningPolicy,
    pub parallelism: crate::ParallelismFacts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "class", rename_all = "kebab-case")]
pub enum BuildPlanningBlocker {
    MissingObservation { observation: crate::ObservationKind },
    StoreAbsent,
    SourceGap,
    OutputTrustGap,
    UnsupportedPlatform,
    OfflineNetworkNeed,
    ExecutorAbsent,
    DoctorPreflightFailed,
    CandidateUnavailable { route: crate::RouteClass },
    Parallelism { blocker: crate::ParallelismBlocker },
    StalePlan,
    WrongEffectIdentity,
    SilentRouteSubstitution,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct TypedRouteRejection {
    pub route: crate::RouteClass,
    pub reason_code: String,
    pub blocker: BuildPlanningBlocker,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum BuildPlanningEffectKind {
    RecheckFacts,
    ReuseLocalOutput,
    ImportSubstitute,
    ImportArchive,
    MaterializeSourceBundle,
    LaunchRemoteBuild,
    LaunchLocalBuild,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct BuildPlanningEffect {
    pub sequence: u32,
    pub effect_id_blake3: String,
    pub selected_route: crate::RouteClass,
    pub kind: BuildPlanningEffectKind,
    pub facts_blake3: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuildPlanningDecision {
    pub schema: String,
    pub facts_blake3: String,
    pub route_plan: crate::RoutePlanReport,
    pub rejected_routes: Vec<TypedRouteRejection>,
    pub blockers: Vec<BuildPlanningBlocker>,
    pub jobs: Option<crate::JobLimitDecision>,
    pub effects: Vec<BuildPlanningEffect>,
    pub plan_blake3: String,
    pub non_claim: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuildPlanningEffectObservation {
    pub effect_id_blake3: String,
    pub selected_route: crate::RouteClass,
    pub kind: BuildPlanningEffectKind,
    pub succeeded: bool,
    pub reason_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AcceptedEffectObservation {
    pub effect_id_blake3: String,
    pub succeeded: bool,
    pub reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildPlanningError {
    InvalidFact(&'static str),
    Serialization,
}

#[derive(Serialize)]
struct EffectPreimage<'a> {
    sequence: u32,
    selected_route: crate::RouteClass,
    kind: BuildPlanningEffectKind,
    facts_blake3: &'a str,
}

#[derive(Serialize)]
struct PlanPreimage<'a> {
    schema: &'a str,
    facts_blake3: &'a str,
    route_plan: &'a crate::RoutePlanReport,
    rejected_routes: &'a [TypedRouteRejection],
    blockers: &'a [BuildPlanningBlocker],
    jobs: &'a Option<crate::JobLimitDecision>,
    effects: &'a [BuildPlanningEffect],
    non_claim: &'a str,
}

#[derive(Clone, Copy)]
struct CompleteFacts<'a> {
    local_output: &'a crate::LocalOutputObservation,
    source_readiness: &'a crate::SourceReadinessObservation,
    substituter: &'a crate::SubstituterObservation,
    archive: &'a crate::ArchiveObservation,
    remote_candidate: &'a crate::RemoteCandidateObservation,
    doctor: &'a crate::DoctorObservation,
    platform: &'a crate::PlatformObservation,
    trust: &'a crate::TrustObservation,
    network: &'a crate::NetworkObservation,
    executor: &'a crate::ExecutorObservation,
}

struct Blake3Validation<'a> {
    value: &'a str,
    field: &'static str,
}

struct DecisionParts {
    facts_blake3: String,
    route_plan: crate::RoutePlanReport,
    rejected_routes: Vec<TypedRouteRejection>,
    blockers: Vec<BuildPlanningBlocker>,
    jobs: Option<crate::JobLimitDecision>,
    effects: Vec<BuildPlanningEffect>,
}

pub fn plan_build(mut request: BuildPlanningRequest) -> Result<BuildPlanningDecision, BuildPlanningError> {
    normalize_request(&mut request);
    validate_present_facts(&request.facts)?;
    let facts_blake3 = canonical_digest(FACT_DOMAIN, &request)?;
    let mut blockers = missing_observation_blockers(&request.facts);
    blockers.reserve(crate::MAX_ROUTE_CANDIDATES.saturating_add(1));
    debug_assert_eq!(facts_blake3.len(), BLAKE3_HEX_CHARS);
    debug_assert!(blockers.len() <= OBSERVATION_KIND_COUNT);
    let (jobs, parallelism_blocker) = planned_jobs(request.parallelism);
    if let Some(blocker) = parallelism_blocker {
        blockers.push(blocker);
    }
    let route_plan = if blockers.iter().any(is_missing_observation) {
        crate::route_plan_for_existing_build_action("preflight-error", Some("required-planning-observation-missing"))
    } else {
        plan_observed_route(&request.facts, request.policy)
    };
    let rejected_routes = typed_rejections(&route_plan);
    if route_plan.selected_route == crate::RouteClass::PreflightError && blockers.is_empty() {
        blockers.push(BuildPlanningBlocker::CandidateUnavailable {
            route: crate::RouteClass::PreflightError,
        });
    }
    blockers.sort();
    blockers.dedup();
    let effects = planned_effects(&facts_blake3, route_plan.selected_route, &blockers)?;
    finish_decision(DecisionParts {
        facts_blake3,
        route_plan,
        rejected_routes,
        blockers,
        jobs,
        effects,
    })
}

pub fn validate_plan_freshness(
    decision: &BuildPlanningDecision,
    current_facts_blake3: &str,
) -> Result<(), BuildPlanningBlocker> {
    if decision.facts_blake3 != current_facts_blake3 {
        return Err(BuildPlanningBlocker::StalePlan);
    }
    debug_assert_eq!(decision.facts_blake3.len(), BLAKE3_HEX_CHARS);
    debug_assert_eq!(decision.schema, BUILD_PLANNING_SCHEMA);
    Ok(())
}

pub fn accept_effect_observation(
    effect: &BuildPlanningEffect,
    observation: BuildPlanningEffectObservation,
) -> Result<AcceptedEffectObservation, BuildPlanningBlocker> {
    if effect.effect_id_blake3 != observation.effect_id_blake3 {
        return Err(BuildPlanningBlocker::WrongEffectIdentity);
    }
    if effect.selected_route != observation.selected_route || effect.kind != observation.kind {
        return Err(BuildPlanningBlocker::SilentRouteSubstitution);
    }
    debug_assert!(!observation.reason_code.is_empty());
    debug_assert_eq!(effect.effect_id_blake3.len(), BLAKE3_HEX_CHARS);
    Ok(AcceptedEffectObservation {
        effect_id_blake3: observation.effect_id_blake3,
        succeeded: observation.succeeded,
        reason_code: observation.reason_code,
    })
}

fn plan_observed_route(facts: &crate::BuildPlanningFacts, policy: BuildPlanningPolicy) -> crate::RoutePlanReport {
    let Some(facts) = complete_facts(facts) else {
        return crate::route_plan_for_existing_build_action(
            "preflight-error",
            Some("required-planning-observation-missing"),
        );
    };
    let candidates = route_candidates(facts);
    debug_assert!(!candidates.is_empty());
    debug_assert!(candidates.len() <= crate::MAX_ROUTE_CANDIDATES);
    crate::plan_realization_route(crate::RoutePlannerInput::new(
        crate::RoutePolicy {
            network: if facts.network.online {
                crate::NetworkPolicy::Online
            } else {
                crate::NetworkPolicy::Offline
            },
            requested_claim_strength: policy.requested_claim_strength,
            upload_policy: policy.upload_policy(),
        },
        candidates,
    ))
}

fn route_candidates(facts: CompleteFacts<'_>) -> Vec<crate::RouteCandidateFacts> {
    let mut candidates = Vec::with_capacity(crate::MAX_ROUTE_CANDIDATES);
    candidates.push(local_output_candidate(facts.local_output));
    candidates.push(substituter_candidate(facts.substituter, facts.trust));
    candidates.push(archive_candidate(facts.archive));
    candidates.push(source_candidate(facts.source_readiness));
    candidates.push(remote_candidate(facts.remote_candidate, facts.trust, facts.executor));
    candidates.push(local_build_candidate(facts.source_readiness, facts.doctor, facts.platform, facts.executor));
    candidates.push(crate::RouteCandidateFacts::eligible(crate::RouteClass::PreflightError, "no-route-eligible"));
    debug_assert_eq!(candidates.len(), crate::MAX_ROUTE_CANDIDATES);
    debug_assert!(candidates.iter().all(|candidate| !candidate.reason_code.is_empty()));
    candidates
}

fn complete_facts(facts: &crate::BuildPlanningFacts) -> Option<CompleteFacts<'_>> {
    Some(CompleteFacts {
        local_output: facts.local_output.as_ref()?,
        source_readiness: facts.source_readiness.as_ref()?,
        substituter: facts.substituter.as_ref()?,
        archive: facts.archive.as_ref()?,
        remote_candidate: facts.remote_candidate.as_ref()?,
        doctor: facts.doctor.as_ref()?,
        platform: facts.platform.as_ref()?,
        trust: facts.trust.as_ref()?,
        network: facts.network.as_ref()?,
        executor: facts.executor.as_ref()?,
    })
}

fn local_output_candidate(facts: &crate::LocalOutputObservation) -> crate::RouteCandidateFacts {
    if !facts.present {
        return crate::RouteCandidateFacts::rejected(crate::RouteClass::CachedLocal, "local-output-missing");
    }
    if !facts.content_complete {
        return crate::RouteCandidateFacts::rejected(crate::RouteClass::CachedLocal, "local-output-incomplete");
    }
    if !facts.trusted {
        return crate::RouteCandidateFacts::rejected(crate::RouteClass::CachedLocal, "local-output-untrusted");
    }
    crate::RouteCandidateFacts::eligible(crate::RouteClass::CachedLocal, "local-pathinfo-and-castore-present")
}

fn substituter_candidate(
    facts: &crate::SubstituterObservation,
    trust: &crate::TrustObservation,
) -> crate::RouteCandidateFacts {
    let candidate = if !facts.configured {
        crate::RouteCandidateFacts::rejected(crate::RouteClass::TrustedSubstitute, "substituter-not-configured")
    } else if !facts.candidate_available {
        crate::RouteCandidateFacts::rejected(crate::RouteClass::TrustedSubstitute, "trusted-substitute-missing")
    } else if !facts.trusted || trust.trusted_output_key_count == 0 {
        crate::RouteCandidateFacts::rejected(crate::RouteClass::TrustedSubstitute, "output-trust-missing")
    } else {
        let eligible = crate::RouteCandidateFacts::eligible(
            crate::RouteClass::TrustedSubstitute,
            "trusted-remote-pathinfo-available",
        );
        with_network_requirement(eligible, facts.requires_network)
    };
    debug_assert_eq!(candidate.route, crate::RouteClass::TrustedSubstitute);
    debug_assert!(!candidate.reason_code.is_empty());
    candidate
}

fn with_network_requirement(
    candidate: crate::RouteCandidateFacts,
    is_network_required: bool,
) -> crate::RouteCandidateFacts {
    debug_assert!(candidate.eligible);
    debug_assert!(!candidate.requires_network);
    if is_network_required {
        return candidate.requiring_network();
    }
    candidate
}

fn archive_candidate(facts: &crate::ArchiveObservation) -> crate::RouteCandidateFacts {
    if !facts.configured || !facts.candidate_available {
        return crate::RouteCandidateFacts::rejected(crate::RouteClass::ArchiveImport, "archive-candidate-missing");
    }
    if !facts.prefix_matches {
        return crate::RouteCandidateFacts::rejected(crate::RouteClass::ArchiveImport, "archive-prefix-mismatch");
    }
    if !facts.signature_trusted {
        return crate::RouteCandidateFacts::rejected(crate::RouteClass::ArchiveImport, "archive-signature-untrusted");
    }
    crate::RouteCandidateFacts::eligible(crate::RouteClass::ArchiveImport, "archive-prefix-and-signature-match")
}

fn source_candidate(facts: &crate::SourceReadinessObservation) -> crate::RouteCandidateFacts {
    if !facts.required {
        return crate::RouteCandidateFacts::rejected(crate::RouteClass::SourceBundle, "no-declared-source-inputs");
    }
    if !facts.ready {
        return crate::RouteCandidateFacts::rejected(crate::RouteClass::SourceBundle, "missing-source-state");
    }
    crate::RouteCandidateFacts::eligible(crate::RouteClass::SourceBundle, "declared-source-bundle-ready")
}

fn remote_candidate(
    facts: &crate::RemoteCandidateObservation,
    trust: &crate::TrustObservation,
    executor: &crate::ExecutorObservation,
) -> crate::RouteCandidateFacts {
    let rejection = remote_rejection_reason(facts, trust, executor);
    if let Some(reason_code) = rejection {
        return crate::RouteCandidateFacts::rejected(crate::RouteClass::P2pRemoteBuilder, reason_code);
    }
    let candidate = crate::RouteCandidateFacts::eligible(
        crate::RouteClass::P2pRemoteBuilder,
        "builder-capability-and-output-trust-match",
    )
    .practical_only()
    .with_upload_summary(facts.upload_summary.clone());
    if facts.requires_network {
        return candidate.requiring_network();
    }
    candidate
}

fn remote_rejection_reason(
    facts: &crate::RemoteCandidateObservation,
    trust: &crate::TrustObservation,
    executor: &crate::ExecutorObservation,
) -> Option<&'static str> {
    [
        (!facts.configured, "remote-builder-not-configured"),
        (!facts.credential_present, "remote-ticket-missing"),
        (!facts.capabilities_match, "builder-capability-mismatch"),
        (!facts.source_inputs_ready, "source-readiness-missing"),
        (!facts.output_trusted || trust.trusted_output_key_count == 0, "output-trust-missing"),
        (!executor.remote_available, "remote-executor-unavailable"),
    ]
    .into_iter()
    .find_map(|(rejected, reason)| rejected.then_some(reason))
}

fn local_build_candidate(
    source: &crate::SourceReadinessObservation,
    doctor: &crate::DoctorObservation,
    platform: &crate::PlatformObservation,
    executor: &crate::ExecutorObservation,
) -> crate::RouteCandidateFacts {
    if source.required && !source.ready {
        return crate::RouteCandidateFacts::rejected(
            crate::RouteClass::LocalBuild,
            "offline-source-readiness-incomplete",
        );
    }
    if !platform.supported {
        return crate::RouteCandidateFacts::rejected(crate::RouteClass::LocalBuild, "unsupported-platform");
    }
    if !executor.local_available {
        return crate::RouteCandidateFacts::rejected(crate::RouteClass::LocalBuild, "local-executor-unavailable");
    }
    if !doctor.preflight_ok {
        return crate::RouteCandidateFacts::rejected(crate::RouteClass::LocalBuild, "local-preflight-failed");
    }
    crate::RouteCandidateFacts::eligible(crate::RouteClass::LocalBuild, "local-preflight-ok")
}

fn planned_jobs(facts: crate::ParallelismFacts) -> (Option<crate::JobLimitDecision>, Option<BuildPlanningBlocker>) {
    match crate::plan_parallelism(facts) {
        Ok(decision) => (Some(decision), None),
        Err(blocker) => (None, Some(BuildPlanningBlocker::Parallelism { blocker })),
    }
}

fn missing_observation_blockers(facts: &crate::BuildPlanningFacts) -> Vec<BuildPlanningBlocker> {
    let checks = [
        (facts.local_output.as_ref().is_none(), crate::ObservationKind::LocalOutput),
        (facts.source_readiness.as_ref().is_none(), crate::ObservationKind::SourceReadiness),
        (facts.substituter.as_ref().is_none(), crate::ObservationKind::Substituter),
        (facts.archive.as_ref().is_none(), crate::ObservationKind::Archive),
        (facts.remote_candidate.as_ref().is_none(), crate::ObservationKind::RemoteCandidate),
        (facts.doctor.as_ref().is_none(), crate::ObservationKind::Doctor),
        (facts.platform.as_ref().is_none(), crate::ObservationKind::Platform),
        (facts.trust.as_ref().is_none(), crate::ObservationKind::Trust),
        (facts.network.as_ref().is_none(), crate::ObservationKind::Network),
        (facts.executor.as_ref().is_none(), crate::ObservationKind::Executor),
    ];
    let mut blockers = Vec::with_capacity(OBSERVATION_KIND_COUNT);
    for (is_missing, observation) in checks {
        if is_missing {
            blockers.push(BuildPlanningBlocker::MissingObservation { observation });
        }
    }
    debug_assert!(blockers.len() <= OBSERVATION_KIND_COUNT);
    debug_assert_eq!(checks.len(), OBSERVATION_KIND_COUNT);
    blockers
}

fn typed_rejections(route_plan: &crate::RoutePlanReport) -> Vec<TypedRouteRejection> {
    let mut typed = Vec::with_capacity(route_plan.rejected_routes.len());
    for rejection in &route_plan.rejected_routes {
        typed.push(TypedRouteRejection {
            route: rejection.route,
            reason_code: rejection.reason_code.clone(),
            blocker: blocker_for_reason(rejection.route, &rejection.reason_code),
        });
    }
    debug_assert_eq!(typed.len(), route_plan.rejected_routes.len());
    debug_assert!(typed.len() <= crate::MAX_ROUTE_CANDIDATES);
    typed
}

fn blocker_for_reason(route: crate::RouteClass, reason: &str) -> BuildPlanningBlocker {
    match reason {
        "local-output-missing" | "local-output-incomplete" => BuildPlanningBlocker::StoreAbsent,
        "missing-source-state" | "offline-source-readiness-incomplete" | "source-readiness-missing" => {
            BuildPlanningBlocker::SourceGap
        }
        "output-trust-missing" | "local-output-untrusted" | "archive-signature-untrusted" => {
            BuildPlanningBlocker::OutputTrustGap
        }
        "unsupported-platform" => BuildPlanningBlocker::UnsupportedPlatform,
        "offline-network-required" => BuildPlanningBlocker::OfflineNetworkNeed,
        "local-executor-unavailable" | "remote-executor-unavailable" => BuildPlanningBlocker::ExecutorAbsent,
        "local-preflight-failed" => BuildPlanningBlocker::DoctorPreflightFailed,
        _ => BuildPlanningBlocker::CandidateUnavailable { route },
    }
}

fn is_missing_observation(blocker: &BuildPlanningBlocker) -> bool {
    matches!(blocker, BuildPlanningBlocker::MissingObservation { .. })
}

fn planned_effects(
    facts_blake3: &str,
    selected_route: crate::RouteClass,
    blockers: &[BuildPlanningBlocker],
) -> Result<Vec<BuildPlanningEffect>, BuildPlanningError> {
    if selected_route == crate::RouteClass::PreflightError || !blockers.is_empty() {
        return Ok(Vec::new());
    }
    let route_effect = effect_kind(selected_route);
    let mut effects = Vec::with_capacity(BUILD_PLANNING_EFFECTS_MAX);
    effects.push(build_effect(0, selected_route, BuildPlanningEffectKind::RecheckFacts, facts_blake3)?);
    effects.push(build_effect(1, selected_route, route_effect, facts_blake3)?);
    debug_assert_eq!(effects.len(), BUILD_PLANNING_EFFECTS_MAX);
    debug_assert!(effects.windows(2).all(|pair| pair[0].sequence < pair[1].sequence));
    Ok(effects)
}

fn effect_kind(route: crate::RouteClass) -> BuildPlanningEffectKind {
    match route {
        crate::RouteClass::CachedLocal => BuildPlanningEffectKind::ReuseLocalOutput,
        crate::RouteClass::TrustedSubstitute => BuildPlanningEffectKind::ImportSubstitute,
        crate::RouteClass::ArchiveImport => BuildPlanningEffectKind::ImportArchive,
        crate::RouteClass::SourceBundle => BuildPlanningEffectKind::MaterializeSourceBundle,
        crate::RouteClass::P2pRemoteBuilder => BuildPlanningEffectKind::LaunchRemoteBuild,
        crate::RouteClass::LocalBuild => BuildPlanningEffectKind::LaunchLocalBuild,
        crate::RouteClass::PreflightError => BuildPlanningEffectKind::RecheckFacts,
    }
}

fn build_effect(
    sequence: u32,
    selected_route: crate::RouteClass,
    kind: BuildPlanningEffectKind,
    facts_blake3: &str,
) -> Result<BuildPlanningEffect, BuildPlanningError> {
    let preimage = EffectPreimage {
        sequence,
        selected_route,
        kind,
        facts_blake3,
    };
    let effect_id_blake3 = canonical_digest(EFFECT_DOMAIN, &preimage)?;
    debug_assert_eq!(effect_id_blake3.len(), BLAKE3_HEX_CHARS);
    debug_assert_eq!(facts_blake3.len(), BLAKE3_HEX_CHARS);
    Ok(BuildPlanningEffect {
        sequence,
        effect_id_blake3,
        selected_route,
        kind,
        facts_blake3: facts_blake3.to_string(),
    })
}

fn finish_decision(parts: DecisionParts) -> Result<BuildPlanningDecision, BuildPlanningError> {
    let preimage = PlanPreimage {
        schema: BUILD_PLANNING_SCHEMA,
        facts_blake3: &parts.facts_blake3,
        route_plan: &parts.route_plan,
        rejected_routes: &parts.rejected_routes,
        blockers: &parts.blockers,
        jobs: &parts.jobs,
        effects: &parts.effects,
        non_claim: BUILD_PLANNING_NON_CLAIM,
    };
    let plan_blake3 = canonical_digest(PLAN_DOMAIN, &preimage)?;
    debug_assert_eq!(plan_blake3.len(), BLAKE3_HEX_CHARS);
    debug_assert!(parts.effects.len() <= BUILD_PLANNING_EFFECTS_MAX);
    Ok(BuildPlanningDecision {
        schema: BUILD_PLANNING_SCHEMA.to_string(),
        facts_blake3: parts.facts_blake3,
        route_plan: parts.route_plan,
        rejected_routes: parts.rejected_routes,
        blockers: parts.blockers,
        jobs: parts.jobs,
        effects: parts.effects,
        plan_blake3,
        non_claim: BUILD_PLANNING_NON_CLAIM.to_string(),
    })
}

fn normalize_request(request: &mut BuildPlanningRequest) {
    let class_count_before =
        request.facts.remote_candidate.as_ref().map_or(0, |remote| remote.upload_summary.classes.len());
    if let crate::ObservedFact::Present(remote) = &mut request.facts.remote_candidate {
        remote.upload_summary.classes.sort();
        remote.upload_summary.classes.dedup();
    }
    debug_assert!(
        request.facts.remote_candidate.as_ref().is_none_or(|remote| remote
            .upload_summary
            .classes
            .windows(2)
            .all(|pair| pair[0] < pair[1]))
    );
    debug_assert!(
        request
            .facts
            .remote_candidate
            .as_ref()
            .is_none_or(|remote| remote.upload_summary.classes.len() <= class_count_before)
    );
}

fn validate_present_facts(facts: &crate::BuildPlanningFacts) -> Result<(), BuildPlanningError> {
    if let Some(source) = facts.source_readiness.as_ref()
        && let Some(identity) = &source.source_identity_blake3
    {
        validate_blake3(Blake3Validation {
            value: identity,
            field: "source-identity",
        })?;
    }
    if let Some(remote) = facts.remote_candidate.as_ref() {
        validate_blake3(Blake3Validation {
            value: &remote.candidate_identity_blake3,
            field: "remote-candidate-identity",
        })?;
        crate::UploadSummary::try_new(
            remote.upload_summary.classes.clone(),
            remote.upload_summary.object_count,
            remote.upload_summary.byte_count,
        )
        .map_err(|_| BuildPlanningError::InvalidFact("remote-upload-summary"))?;
    }
    if let Some(doctor) = facts.doctor.as_ref() {
        validate_blake3(Blake3Validation {
            value: &doctor.report_identity_blake3,
            field: "doctor-report-identity",
        })?;
    }
    if let Some(trust) = facts.trust.as_ref() {
        validate_blake3(Blake3Validation {
            value: &trust.trust_policy_identity_blake3,
            field: "trust-policy-identity",
        })?;
    }
    if facts.platform.as_ref().is_some_and(|platform| platform.platform_identity.is_empty()) {
        return Err(BuildPlanningError::InvalidFact("platform-identity"));
    }
    if facts.executor.as_ref().is_some_and(|executor| executor.executor_limit == Some(0)) {
        return Err(BuildPlanningError::InvalidFact("executor-limit"));
    }
    debug_assert!(facts.platform.as_ref().is_none_or(|platform| !platform.platform_identity.is_empty()));
    debug_assert!(facts.executor.as_ref().is_none_or(|executor| executor.executor_limit != Some(0)));
    Ok(())
}

fn validate_blake3(input: Blake3Validation<'_>) -> Result<(), BuildPlanningError> {
    let is_valid = input.value.len() == BLAKE3_HEX_CHARS
        && input.value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !is_valid {
        return Err(BuildPlanningError::InvalidFact(input.field));
    }
    Ok(())
}

fn canonical_digest<T: Serialize>(domain: &[u8], value: &T) -> Result<String, BuildPlanningError> {
    let bytes = serde_json::to_vec(value).map_err(|_| BuildPlanningError::Serialization)?;
    let mut hasher = blake3::Hasher::new();
    update_frame(&mut hasher, domain)?;
    update_frame(&mut hasher, &bytes)?;
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    debug_assert!(!domain.is_empty());
    Ok(digest)
}

fn update_frame(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), BuildPlanningError> {
    let length_bytes = u64::try_from(bytes.len()).map_err(|_| BuildPlanningError::Serialization)?;
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(bytes);
    debug_assert_eq!(usize::try_from(length_bytes).ok(), Some(bytes.len()));
    debug_assert_eq!(core::mem::size_of_val(&length_bytes), core::mem::size_of::<u64>());
    Ok(())
}
