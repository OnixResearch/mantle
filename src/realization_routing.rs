// machine-artifact-public: realization.route-plan-report
use serde::Serialize;

pub const ROUTE_REPORT_SCHEMA: &str = "mantle-realization-route-plan-v1";
pub const ROUTE_TIE_BREAKER: &str =
    "local-cache,trusted-substitute,archive-import,source-bundle,p2p-remote-builder,local-build,preflight-error";
pub const ROUTE_NON_CLAIM: &str =
    "route planning is advisory and does not prove transport execution or artifact correctness";

const MAX_ROUTE_CANDIDATES: usize = 7;
const MAX_REJECTED_ROUTES: usize = 7;
const MAX_REASON_CODE_BYTES: usize = 96;
const MAX_DETAIL_BYTES: usize = 256;
const MAX_UPLOAD_CLASSES: usize = 4;
const MAX_UPLOAD_OBJECTS: u32 = 65_536;
const MAX_UPLOAD_BYTES: u64 = 1_099_511_627_776;
const MIN_ROUTE_STORE_LAYER_INDEX: usize = 1;
const MAX_ROUTE_STORE_BASES: usize = 8;
const MAX_ROUTE_STORE_LAYER_LABEL_BYTES: usize = 32;
const MAX_ROUTE_STORE_LAYER_SELECTIONS: usize = 65_536;
const REMOTE_CAPABILITY_DEFAULT: &str = "stdio-default";
const REMOTE_PLAN_NON_CLAIM: &str = "route-eligibility-only";

const _: () = {
    assert!(MAX_ROUTE_CANDIDATES > 0);
    assert!(MAX_REJECTED_ROUTES >= MAX_ROUTE_CANDIDATES.saturating_sub(1));
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RouteClass {
    CachedLocal,
    TrustedSubstitute,
    ArchiveImport,
    SourceBundle,
    P2pRemoteBuilder,
    LocalBuild,
    PreflightError,
}

impl RouteClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CachedLocal => "cached-local",
            Self::TrustedSubstitute => "trusted-substitute",
            Self::ArchiveImport => "archive-import",
            Self::SourceBundle => "source-bundle",
            Self::P2pRemoteBuilder => "p2p-remote-builder",
            Self::LocalBuild => "local-build",
            Self::PreflightError => "preflight-error",
        }
    }

    fn rank(self) -> u8 {
        match self {
            Self::CachedLocal => 0,
            Self::TrustedSubstitute => 1,
            Self::ArchiveImport => 2,
            Self::SourceBundle => 3,
            Self::P2pRemoteBuilder => 4,
            Self::LocalBuild => 5,
            Self::PreflightError => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkPolicy {
    Online,
    Offline,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClaimStrength {
    Practical,
    StrongActionCorrectness,
    ReleaseFacing,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UploadClass {
    Source,
    StoreObject,
    Proof,
    SecretDescriptor,
}

impl UploadClass {
    fn is_allowed_by(self, policy: UploadPolicy) -> bool {
        match self {
            Self::Source => policy.allow_sources,
            Self::StoreObject => policy.allow_store_objects,
            Self::Proof => policy.allow_proofs,
            Self::SecretDescriptor => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UploadPolicy {
    pub allow_sources: bool,
    pub allow_store_objects: bool,
    pub allow_proofs: bool,
}

impl UploadPolicy {
    pub fn local_only() -> Self {
        Self {
            allow_sources: false,
            allow_store_objects: false,
            allow_proofs: false,
        }
    }

    #[allow(dead_code)]
    pub fn allow_sources_and_store_objects() -> Self {
        Self {
            allow_sources: true,
            allow_store_objects: true,
            allow_proofs: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RoutePlanError {
    TooManyUploadClasses { actual: usize, limit: usize },
    UploadObjectLimitExceeded { actual: u32, limit: u32 },
    UploadByteLimitExceeded { actual: u64, limit: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UploadSummary {
    pub classes: Vec<UploadClass>,
    pub object_count: u32,
    pub byte_count: u64,
}

impl UploadSummary {
    #[cfg(test)]
    pub fn new(classes: Vec<UploadClass>, object_count: u32, byte_count: u64) -> Self {
        Self::try_new(classes, object_count, byte_count).expect("upload summary fixture within bounds")
    }

    pub fn try_new(mut classes: Vec<UploadClass>, object_count: u32, byte_count: u64) -> Result<Self, RoutePlanError> {
        if classes.len() > MAX_UPLOAD_CLASSES {
            return Err(RoutePlanError::TooManyUploadClasses {
                actual: classes.len(),
                limit: MAX_UPLOAD_CLASSES,
            });
        }
        if object_count > MAX_UPLOAD_OBJECTS {
            return Err(RoutePlanError::UploadObjectLimitExceeded {
                actual: object_count,
                limit: MAX_UPLOAD_OBJECTS,
            });
        }
        if byte_count > MAX_UPLOAD_BYTES {
            return Err(RoutePlanError::UploadByteLimitExceeded {
                actual: byte_count,
                limit: MAX_UPLOAD_BYTES,
            });
        }
        classes.sort();
        classes.dedup();
        assert!(classes.len() <= MAX_UPLOAD_CLASSES);
        Ok(Self {
            classes,
            object_count,
            byte_count,
        })
    }

    fn forbidden_by(&self, policy: UploadPolicy) -> Option<UploadClass> {
        assert!(self.classes.len() <= MAX_UPLOAD_CLASSES);
        self.classes.iter().copied().find(|class| !class.is_allowed_by(policy))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoutePolicy {
    pub network: NetworkPolicy,
    pub requested_claim_strength: ClaimStrength,
    pub upload_policy: UploadPolicy,
}

impl RoutePolicy {
    pub fn practical_online() -> Self {
        Self {
            network: NetworkPolicy::Online,
            requested_claim_strength: ClaimStrength::Practical,
            upload_policy: UploadPolicy::local_only(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteCandidateFacts {
    pub route: RouteClass,
    pub eligible: bool,
    pub reason_code: &'static str,
    pub detail: Option<String>,
    pub requires_network: bool,
    pub proves_strong_action_correctness: bool,
    pub upload_summary: Option<UploadSummary>,
}

impl RouteCandidateFacts {
    pub fn eligible(route: RouteClass, reason_code: &'static str) -> Self {
        Self {
            route,
            eligible: true,
            reason_code,
            detail: None,
            requires_network: false,
            proves_strong_action_correctness: true,
            upload_summary: None,
        }
    }

    pub fn rejected(route: RouteClass, reason_code: &'static str) -> Self {
        Self {
            route,
            eligible: false,
            reason_code,
            detail: None,
            requires_network: false,
            proves_strong_action_correctness: false,
            upload_summary: None,
        }
    }

    pub fn with_detail(mut self, detail: Option<&str>) -> Self {
        self.detail = detail.map(truncate_detail);
        self
    }

    pub fn with_owned_detail(mut self, detail: Option<String>) -> Self {
        self.detail = detail.map(|value| truncate_detail(&value));
        self
    }

    pub fn requiring_network(mut self) -> Self {
        self.requires_network = true;
        self
    }

    #[allow(dead_code)]
    pub fn practical_only(mut self) -> Self {
        self.proves_strong_action_correctness = false;
        self
    }

    #[allow(dead_code)]
    pub fn with_upload_summary(mut self, summary: UploadSummary) -> Self {
        self.upload_summary = Some(summary);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteBuilderPlanFacts {
    pub endpoint_id: String,
    pub builder_configured: bool,
    pub ticket_configured: bool,
    pub concrete_inputs: bool,
    pub capabilities_match: bool,
    pub source_inputs_ready: bool,
    pub upload_summary: UploadSummary,
    pub trusted_output_key_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBundleRouteFacts {
    pub required: bool,
    pub ready: bool,
    pub reason_code: &'static str,
    pub detail: Option<String>,
}

impl RemoteBuilderPlanFacts {
    pub fn cli_configured(
        endpoint_id: Option<&str>,
        ticket_configured: bool,
        trusted_output_key_count: u32,
    ) -> Option<Self> {
        if endpoint_id.is_none() && !ticket_configured && trusted_output_key_count == 0 {
            return None;
        }
        let is_builder_configured = endpoint_id.is_some();
        let endpoint_id = endpoint_id.unwrap_or("unconfigured").to_string();
        Some(Self {
            endpoint_id,
            builder_configured: is_builder_configured,
            ticket_configured,
            concrete_inputs: true,
            capabilities_match: true,
            source_inputs_ready: true,
            upload_summary: UploadSummary {
                classes: Vec::new(),
                object_count: 0,
                byte_count: 0,
            },
            trusted_output_key_count,
        })
    }

    fn redacted_detail(&self) -> String {
        format!(
            "endpoint={}; capabilities={}; upload_objects={}; upload_bytes={}; output_trust_keys={}; non_claim={}",
            self.endpoint_id,
            REMOTE_CAPABILITY_DEFAULT,
            self.upload_summary.object_count,
            self.upload_summary.byte_count,
            self.trusted_output_key_count,
            REMOTE_PLAN_NON_CLAIM,
        )
    }
}

// r[impl realization_routing.source_bundle_route_execution]
pub fn source_bundle_candidate_from_facts(facts: Option<&SourceBundleRouteFacts>) -> RouteCandidateFacts {
    let Some(facts) = facts else {
        return RouteCandidateFacts::rejected(RouteClass::SourceBundle, "source-inputs-present-or-fetchable");
    };
    if !facts.required {
        return RouteCandidateFacts::rejected(RouteClass::SourceBundle, "no-declared-source-inputs")
            .with_owned_detail(facts.detail.clone());
    }
    if facts.ready {
        return RouteCandidateFacts::eligible(RouteClass::SourceBundle, "declared-source-bundle-ready")
            .with_owned_detail(facts.detail.clone());
    }
    RouteCandidateFacts::rejected(RouteClass::SourceBundle, facts.reason_code).with_owned_detail(facts.detail.clone())
}

fn source_bundle_blocks_local_build(facts: Option<&SourceBundleRouteFacts>) -> bool {
    facts.is_some_and(|facts| facts.required && !facts.ready)
}

pub fn remote_builder_candidate_from_facts(facts: &RemoteBuilderPlanFacts) -> RouteCandidateFacts {
    debug_assert!(facts.upload_summary.classes.len() <= MAX_UPLOAD_CLASSES);
    debug_assert!(facts.upload_summary.object_count <= MAX_UPLOAD_OBJECTS);
    let detail = Some(facts.redacted_detail());
    if !facts.builder_configured {
        return RouteCandidateFacts::rejected(RouteClass::P2pRemoteBuilder, "remote-builder-not-configured")
            .with_owned_detail(detail);
    }
    if !facts.ticket_configured {
        return RouteCandidateFacts::rejected(RouteClass::P2pRemoteBuilder, "remote-ticket-missing")
            .with_owned_detail(detail);
    }
    if !facts.concrete_inputs {
        return RouteCandidateFacts::rejected(RouteClass::P2pRemoteBuilder, "concrete-inputs-missing")
            .with_owned_detail(detail);
    }
    if !facts.capabilities_match {
        return RouteCandidateFacts::rejected(RouteClass::P2pRemoteBuilder, "builder-capability-mismatch")
            .with_owned_detail(detail);
    }
    if !facts.source_inputs_ready {
        return RouteCandidateFacts::rejected(RouteClass::P2pRemoteBuilder, "source-readiness-missing")
            .with_owned_detail(detail);
    }
    if facts.trusted_output_key_count == 0 {
        return RouteCandidateFacts::rejected(RouteClass::P2pRemoteBuilder, "output-trust-missing")
            .with_owned_detail(detail);
    }
    RouteCandidateFacts::eligible(RouteClass::P2pRemoteBuilder, "builder-capability-and-output-trust-match")
        .requiring_network()
        .practical_only()
        .with_upload_summary(facts.upload_summary.clone())
        .with_owned_detail(detail)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoutePlannerInput {
    pub policy: RoutePolicy,
    pub candidates: Vec<RouteCandidateFacts>,
}

impl RoutePlannerInput {
    pub fn new(policy: RoutePolicy, candidates: Vec<RouteCandidateFacts>) -> Self {
        assert!(!candidates.is_empty());
        assert!(candidates.len() <= MAX_ROUTE_CANDIDATES);
        Self { policy, candidates }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoutePolicyReport {
    pub network: NetworkPolicy,
    pub requested_claim_strength: ClaimStrength,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RouteRejection {
    pub route: RouteClass,
    pub reason_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RouteStoreOverlayBaseEvidence {
    pub layer_index: usize,
    pub descriptor_blake3: String,
    pub generation_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RouteStoreOverlayEvidence {
    pub plan_blake3: String,
    pub bases: Vec<RouteStoreOverlayBaseEvidence>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RouteStoreLayerEvidence {
    pub store_path: String,
    pub selected_layer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_descriptor_blake3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_generation_blake3: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoutePlanReport {
    pub schema: &'static str,
    pub selected_route: RouteClass,
    pub selected_reason_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_detail: Option<String>,
    pub policy: RoutePolicyReport,
    pub tie_breaker: &'static str,
    pub rejected_routes: Vec<RouteRejection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_summary: Option<UploadSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_overlay: Option<RouteStoreOverlayEvidence>,
    pub selected_store_layers: Vec<RouteStoreLayerEvidence>,
    pub non_claim: &'static str,
}

impl RoutePlanReport {
    pub fn bind_store_evidence(
        &mut self,
        overlay: Option<RouteStoreOverlayEvidence>,
        selected_layers: Vec<RouteStoreLayerEvidence>,
    ) -> Result<(), String> {
        if selected_layers.len() > MAX_ROUTE_STORE_LAYER_SELECTIONS {
            return Err("route store layer selection limit exceeded".to_string());
        }
        if selected_layers.iter().any(|selection| {
            selection.selected_layer.is_empty() || selection.selected_layer.len() > MAX_ROUTE_STORE_LAYER_LABEL_BYTES
        }) {
            return Err("route store layer label is invalid".to_string());
        }
        if let Some(overlay) = overlay.as_ref() {
            if overlay.bases.len() > MAX_ROUTE_STORE_BASES
                || overlay.bases.iter().any(|base| {
                    base.layer_index < MIN_ROUTE_STORE_LAYER_INDEX || base.layer_index > MAX_ROUTE_STORE_BASES
                })
            {
                return Err("route overlay base evidence is invalid".to_string());
            }
        }
        self.store_overlay = overlay;
        self.selected_store_layers = selected_layers;
        Ok(())
    }
}

pub fn plan_realization_route(input: RoutePlannerInput) -> RoutePlanReport {
    assert!(!input.candidates.is_empty());
    assert!(input.candidates.len() <= MAX_ROUTE_CANDIDATES);

    let mut candidates = input.candidates;
    candidates.sort_by_key(|candidate| candidate.route.rank());
    assert!(candidates.windows(2).all(|pair| pair[0].route != pair[1].route));

    let mut rejected_routes = Vec::with_capacity(MAX_REJECTED_ROUTES);
    let mut selected = None;

    for candidate in candidates {
        assert!(candidate.reason_code.len() <= MAX_REASON_CODE_BYTES);
        if let Some(detail) = &candidate.detail {
            assert!(detail.len() <= MAX_DETAIL_BYTES);
        }

        match classify_candidate(&candidate, input.policy) {
            CandidateDecision::Select => {
                selected = Some(candidate);
                break;
            }
            CandidateDecision::Reject { reason_code, detail } => {
                rejected_routes.push(RouteRejection {
                    route: candidate.route,
                    reason_code,
                    detail,
                });
            }
        }
    }

    assert!(rejected_routes.len() <= MAX_REJECTED_ROUTES);
    let selected =
        selected.unwrap_or_else(|| RouteCandidateFacts::eligible(RouteClass::PreflightError, "no-route-eligible"));

    RoutePlanReport {
        schema: ROUTE_REPORT_SCHEMA,
        selected_route: selected.route,
        selected_reason_code: selected.reason_code.to_string(),
        selected_detail: selected.detail,
        policy: RoutePolicyReport {
            network: input.policy.network,
            requested_claim_strength: input.policy.requested_claim_strength,
        },
        tie_breaker: ROUTE_TIE_BREAKER,
        rejected_routes,
        upload_summary: selected.upload_summary,
        store_overlay: None,
        selected_store_layers: Vec::new(),
        non_claim: ROUTE_NON_CLAIM,
    }
}

enum CandidateDecision {
    Select,
    Reject {
        reason_code: String,
        detail: Option<String>,
    },
}

fn classify_candidate(candidate: &RouteCandidateFacts, policy: RoutePolicy) -> CandidateDecision {
    debug_assert!(candidate.reason_code.len() <= MAX_REASON_CODE_BYTES);
    debug_assert!(candidate.detail.as_ref().is_none_or(|detail| detail.len() <= MAX_DETAIL_BYTES));
    if !candidate.eligible {
        return CandidateDecision::Reject {
            reason_code: candidate.reason_code.to_string(),
            detail: candidate.detail.clone(),
        };
    }
    if policy.network == NetworkPolicy::Offline && candidate.requires_network {
        return CandidateDecision::Reject {
            reason_code: "offline-network-required".to_string(),
            detail: Some(candidate.route.as_str().to_string()),
        };
    }
    if policy.requested_claim_strength != ClaimStrength::Practical && !candidate.proves_strong_action_correctness {
        return CandidateDecision::Reject {
            reason_code: "claim-strength-mismatch".to_string(),
            detail: Some(candidate.route.as_str().to_string()),
        };
    }
    if let Some(summary) = &candidate.upload_summary
        && let Some(forbidden) = summary.forbidden_by(policy.upload_policy)
    {
        return CandidateDecision::Reject {
            reason_code: "upload-privacy-denied".to_string(),
            detail: Some(format!("forbidden upload class: {forbidden:?}")),
        };
    }
    CandidateDecision::Select
}

pub fn route_plan_for_existing_build_action(action: &str, detail: Option<&str>) -> RoutePlanReport {
    route_plan_for_build_action_with_remote(action, detail, None)
}

pub fn route_plan_for_build_action_with_remote(
    action: &str,
    detail: Option<&str>,
    remote_builder: Option<&RemoteBuilderPlanFacts>,
) -> RoutePlanReport {
    route_plan_for_build_action_with_remote_and_source(action, detail, remote_builder, None)
}

pub fn route_plan_for_build_action_with_remote_and_source(
    action: &str,
    detail: Option<&str>,
    remote_builder: Option<&RemoteBuilderPlanFacts>,
    source_bundle: Option<&SourceBundleRouteFacts>,
) -> RoutePlanReport {
    let detail = detail.map(truncate_detail);
    let mut candidates = Vec::with_capacity(MAX_ROUTE_CANDIDATES);
    push_build_action_candidates(action, detail.as_deref(), remote_builder, source_bundle, &mut candidates);
    debug_assert!(!candidates.is_empty());
    debug_assert!(candidates.len() <= MAX_ROUTE_CANDIDATES);
    let policy = route_policy_for_source_bundle(source_bundle);
    plan_realization_route(RoutePlannerInput::new(policy, candidates))
}

fn push_build_action_candidates(
    action: &str,
    detail: Option<&str>,
    remote_builder: Option<&RemoteBuilderPlanFacts>,
    source_bundle: Option<&SourceBundleRouteFacts>,
    candidates: &mut Vec<RouteCandidateFacts>,
) {
    match action {
        "cached" => push_cached_action_candidates(candidates),
        "substitute" => push_substitute_action_candidates(detail, source_bundle, candidates),
        "build" => push_local_build_action_candidates(detail, remote_builder, source_bundle, candidates),
        "preflight-error" => push_preflight_error_candidates(detail, remote_builder, source_bundle, candidates),
        _ => candidates.push(
            RouteCandidateFacts::eligible(RouteClass::PreflightError, "unknown-build-plan-action").with_detail(detail),
        ),
    }
}

fn push_cached_action_candidates(candidates: &mut Vec<RouteCandidateFacts>) {
    candidates.extend([
        RouteCandidateFacts::eligible(RouteClass::CachedLocal, "local-pathinfo-and-castore-present"),
        RouteCandidateFacts::rejected(RouteClass::TrustedSubstitute, "not-needed-local-hit"),
        RouteCandidateFacts::rejected(RouteClass::ArchiveImport, "not-configured"),
        RouteCandidateFacts::rejected(RouteClass::SourceBundle, "not-needed-local-hit"),
        RouteCandidateFacts::rejected(RouteClass::P2pRemoteBuilder, "not-configured"),
        RouteCandidateFacts::rejected(RouteClass::LocalBuild, "not-needed-local-hit"),
    ]);
}

fn push_substitute_action_candidates(
    detail: Option<&str>,
    source_bundle: Option<&SourceBundleRouteFacts>,
    candidates: &mut Vec<RouteCandidateFacts>,
) {
    candidates.extend([
        RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing").with_detail(detail),
        RouteCandidateFacts::eligible(RouteClass::TrustedSubstitute, "trusted-remote-pathinfo-available")
            .requiring_network(),
        RouteCandidateFacts::rejected(RouteClass::ArchiveImport, "not-configured"),
        source_bundle_candidate_from_facts(source_bundle),
        RouteCandidateFacts::rejected(RouteClass::P2pRemoteBuilder, "not-configured"),
        local_build_candidate(source_bundle, false),
    ]);
}

fn push_local_build_action_candidates(
    detail: Option<&str>,
    remote_builder: Option<&RemoteBuilderPlanFacts>,
    source_bundle: Option<&SourceBundleRouteFacts>,
    candidates: &mut Vec<RouteCandidateFacts>,
) {
    candidates.extend([
        RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing").with_detail(detail),
        RouteCandidateFacts::rejected(RouteClass::TrustedSubstitute, "trusted-substitute-missing"),
        RouteCandidateFacts::rejected(RouteClass::ArchiveImport, "not-configured"),
        source_bundle_candidate_from_facts(source_bundle),
    ]);
    push_remote_candidate(candidates, remote_builder);
    candidates.push(local_build_candidate(source_bundle, true));
}

fn push_preflight_error_candidates(
    detail: Option<&str>,
    remote_builder: Option<&RemoteBuilderPlanFacts>,
    source_bundle: Option<&SourceBundleRouteFacts>,
    candidates: &mut Vec<RouteCandidateFacts>,
) {
    candidates.extend([
        RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
        RouteCandidateFacts::rejected(RouteClass::TrustedSubstitute, "trusted-substitute-missing"),
        RouteCandidateFacts::rejected(RouteClass::ArchiveImport, "not-configured"),
        source_bundle_candidate_from_facts(source_bundle),
    ]);
    push_remote_candidate(candidates, remote_builder);
    candidates.extend([
        RouteCandidateFacts::rejected(RouteClass::LocalBuild, "local-preflight-failed").with_detail(detail),
        RouteCandidateFacts::eligible(RouteClass::PreflightError, "local-preflight-failed").with_detail(detail),
    ]);
}

fn local_build_candidate(
    source_bundle: Option<&SourceBundleRouteFacts>,
    is_build_eligible: bool,
) -> RouteCandidateFacts {
    if source_bundle_blocks_local_build(source_bundle) {
        return RouteCandidateFacts::rejected(RouteClass::LocalBuild, "offline-source-readiness-incomplete");
    }
    if is_build_eligible {
        return RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok");
    }
    RouteCandidateFacts::rejected(RouteClass::LocalBuild, "not-selected-higher-priority-route")
}

fn route_policy_for_source_bundle(source_bundle: Option<&SourceBundleRouteFacts>) -> RoutePolicy {
    let mut policy = RoutePolicy::practical_online();
    if source_bundle.is_some() {
        policy.network = NetworkPolicy::Offline;
    }
    policy
}

fn push_remote_candidate(candidates: &mut Vec<RouteCandidateFacts>, remote_builder: Option<&RemoteBuilderPlanFacts>) {
    let candidate = match remote_builder {
        Some(facts) => remote_builder_candidate_from_facts(facts),
        None => RouteCandidateFacts::rejected(RouteClass::P2pRemoteBuilder, "not-configured"),
    };
    candidates.push(candidate);
}

fn truncate_detail(input: &str) -> String {
    if input.len() <= MAX_DETAIL_BYTES {
        return input.to_string();
    }
    let mut end = MAX_DETAIL_BYTES;
    while !input.is_char_boundary(end) {
        end -= 1;
    }
    input[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selected_route(input: RoutePlannerInput) -> RouteClass {
        plan_realization_route(input).selected_route
    }

    #[test]
    fn unknown_build_action_fails_closed_as_preflight_error() {
        let report = route_plan_for_existing_build_action("unknown-action", Some("operator input"));

        assert_eq!(report.selected_route, RouteClass::PreflightError);
        assert_eq!(report.selected_reason_code, "unknown-build-plan-action");
        assert_eq!(report.selected_detail.as_deref(), Some("operator input"));
    }

    #[test]
    fn selects_cached_local_before_remote_candidates() {
        let report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
            RouteCandidateFacts::eligible(RouteClass::CachedLocal, "local-hit"),
            RouteCandidateFacts::eligible(RouteClass::TrustedSubstitute, "remote-hit").requiring_network(),
        ]));

        assert_eq!(report.selected_route, RouteClass::CachedLocal);
        assert_eq!(report.selected_reason_code, "local-hit");
        assert!(report.rejected_routes.is_empty());
    }

    #[test]
    fn candidate_order_does_not_change_tie_breaker_result() {
        let policy = RoutePolicy::practical_online();
        let left = selected_route(RoutePlannerInput::new(policy, vec![
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
            RouteCandidateFacts::eligible(RouteClass::ArchiveImport, "archive-hit"),
            RouteCandidateFacts::eligible(RouteClass::P2pRemoteBuilder, "remote-builder-hit").requiring_network(),
        ]));
        let right = selected_route(RoutePlannerInput::new(policy, vec![
            RouteCandidateFacts::eligible(RouteClass::P2pRemoteBuilder, "remote-builder-hit").requiring_network(),
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
            RouteCandidateFacts::eligible(RouteClass::ArchiveImport, "archive-hit"),
        ]));

        assert_eq!(left, RouteClass::ArchiveImport);
        assert_eq!(right, RouteClass::ArchiveImport);
    }

    #[test]
    fn offline_policy_rejects_network_routes_before_local_build() {
        let report = plan_realization_route(RoutePlannerInput::new(
            RoutePolicy {
                network: NetworkPolicy::Offline,
                requested_claim_strength: ClaimStrength::Practical,
                upload_policy: UploadPolicy::local_only(),
            },
            vec![
                RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-miss"),
                RouteCandidateFacts::eligible(RouteClass::TrustedSubstitute, "remote-hit").requiring_network(),
                RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
            ],
        ));

        assert_eq!(report.selected_route, RouteClass::LocalBuild);
        assert_eq!(report.rejected_routes[1].route, RouteClass::TrustedSubstitute);
        assert_eq!(report.rejected_routes[1].reason_code, "offline-network-required");
    }

    #[test]
    fn strong_claim_policy_rejects_practical_only_remote_build() {
        let report = plan_realization_route(RoutePlannerInput::new(
            RoutePolicy {
                network: NetworkPolicy::Online,
                requested_claim_strength: ClaimStrength::StrongActionCorrectness,
                upload_policy: UploadPolicy::allow_sources_and_store_objects(),
            },
            vec![
                RouteCandidateFacts::eligible(RouteClass::P2pRemoteBuilder, "remote-builder-hit")
                    .requiring_network()
                    .practical_only()
                    .with_upload_summary(UploadSummary::new(
                        vec![UploadClass::Source, UploadClass::StoreObject],
                        2,
                        4_096,
                    )),
                RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
            ],
        ));

        assert_eq!(report.selected_route, RouteClass::LocalBuild);
        assert_eq!(report.rejected_routes[0].route, RouteClass::P2pRemoteBuilder);
        assert_eq!(report.rejected_routes[0].reason_code, "claim-strength-mismatch");
    }

    #[test]
    fn upload_policy_rejects_forbidden_secret_descriptors() {
        let report = plan_realization_route(RoutePlannerInput::new(
            RoutePolicy {
                network: NetworkPolicy::Online,
                requested_claim_strength: ClaimStrength::Practical,
                upload_policy: UploadPolicy::allow_sources_and_store_objects(),
            },
            vec![
                RouteCandidateFacts::eligible(RouteClass::P2pRemoteBuilder, "remote-builder-hit")
                    .requiring_network()
                    .practical_only()
                    .with_upload_summary(UploadSummary::new(vec![UploadClass::SecretDescriptor], 1, 128)),
                RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
            ],
        ));

        assert_eq!(report.selected_route, RouteClass::LocalBuild);
        assert_eq!(report.rejected_routes[0].reason_code, "upload-privacy-denied");
        assert!(report.rejected_routes[0].detail.as_deref().unwrap_or_default().contains("SecretDescriptor"));
    }

    #[test]
    fn reports_preflight_when_no_realization_route_is_eligible() {
        let report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-miss"),
            RouteCandidateFacts::rejected(RouteClass::TrustedSubstitute, "remote-miss"),
            RouteCandidateFacts::rejected(RouteClass::LocalBuild, "local-preflight-failed"),
        ]));

        assert_eq!(report.selected_route, RouteClass::PreflightError);
        assert_eq!(report.selected_reason_code, "no-route-eligible");
        assert_eq!(report.rejected_routes.len(), 3);
    }

    #[test]
    fn existing_action_adapter_records_selected_and_rejected_routes() {
        let report = route_plan_for_existing_build_action("build", Some("out=build"));

        assert_eq!(report.selected_route, RouteClass::LocalBuild);
        assert!(report.rejected_routes.iter().any(|route| route.route == RouteClass::TrustedSubstitute));
        assert!(report.rejected_routes.iter().all(|route| route.reason_code.len() <= MAX_REASON_CODE_BYTES));
    }

    #[test]
    fn selects_trusted_substitute_when_local_cache_misses() {
        let report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
            RouteCandidateFacts::eligible(RouteClass::TrustedSubstitute, "trusted-remote-pathinfo-available")
                .requiring_network(),
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
        ]));

        assert_eq!(report.selected_route, RouteClass::TrustedSubstitute);
        assert_eq!(report.rejected_routes[0].reason_code, "local-output-missing");
    }

    #[test]
    fn selects_archive_import_when_archive_candidate_is_eligible() {
        let report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
            RouteCandidateFacts::rejected(RouteClass::TrustedSubstitute, "trusted-substitute-missing"),
            RouteCandidateFacts::eligible(RouteClass::ArchiveImport, "archive-prefix-and-signature-match"),
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
        ]));

        assert_eq!(report.selected_route, RouteClass::ArchiveImport);
        assert_eq!(report.selected_reason_code, "archive-prefix-and-signature-match");
    }

    #[test]
    fn selects_source_bundle_route_when_source_material_is_required() {
        let report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
            RouteCandidateFacts::rejected(RouteClass::TrustedSubstitute, "trusted-substitute-missing"),
            RouteCandidateFacts::rejected(RouteClass::ArchiveImport, "archive-candidate-missing"),
            RouteCandidateFacts::eligible(RouteClass::SourceBundle, "declared-source-bundle-ready"),
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
        ]));

        assert_eq!(report.selected_route, RouteClass::SourceBundle);
        assert_eq!(report.rejected_routes.len(), 3);
    }

    // r[verify realization_routing.source_bundle_route_execution]
    #[test]
    fn offline_ready_source_bundle_facts_select_input_realization_route() {
        let source_facts = SourceBundleRouteFacts {
            required: true,
            ready: true,
            reason_code: "declared-source-bundle-ready",
            detail: Some("source_state_blake3=abc123; non_claim=input-realization-only".to_string()),
        };
        let remote =
            RemoteBuilderPlanFacts::cli_configured(Some("builder-1"), true, 1).expect("remote facts configured");

        let report = route_plan_for_build_action_with_remote_and_source(
            "build",
            Some("out=build"),
            Some(&remote),
            Some(&source_facts),
        );

        assert_eq!(report.policy.network, NetworkPolicy::Offline);
        assert_eq!(report.selected_route, RouteClass::SourceBundle);
        assert_eq!(report.selected_reason_code, "declared-source-bundle-ready");
        assert!(report.selected_detail.as_deref().unwrap_or_default().contains("source_state_blake3"));
    }

    // r[verify realization_routing.source_bundle_route_execution]
    #[test]
    fn offline_incomplete_source_bundle_facts_reject_network_and_local_fetch_routes() {
        let source_facts = SourceBundleRouteFacts {
            required: true,
            ready: false,
            reason_code: "missing-source-state",
            detail: Some("record=fixed-url-demo".to_string()),
        };
        let remote =
            RemoteBuilderPlanFacts::cli_configured(Some("builder-1"), true, 1).expect("remote facts configured");

        let report = route_plan_for_build_action_with_remote_and_source(
            "build",
            Some("out=build"),
            Some(&remote),
            Some(&source_facts),
        );

        assert_eq!(report.policy.network, NetworkPolicy::Offline);
        assert_eq!(report.selected_route, RouteClass::PreflightError);
        assert!(
            report.rejected_routes.iter().any(|route| {
                route.route == RouteClass::SourceBundle && route.reason_code == "missing-source-state"
            })
        );
        assert!(report.rejected_routes.iter().any(|route| {
            route.route == RouteClass::P2pRemoteBuilder && route.reason_code == "offline-network-required"
        }));
        assert!(report.rejected_routes.iter().any(|route| {
            route.route == RouteClass::LocalBuild && route.reason_code == "offline-source-readiness-incomplete"
        }));
    }

    #[test]
    fn selects_remote_builder_with_privacy_safe_upload_summary() {
        let report = plan_realization_route(RoutePlannerInput::new(
            RoutePolicy {
                network: NetworkPolicy::Online,
                requested_claim_strength: ClaimStrength::Practical,
                upload_policy: UploadPolicy::allow_sources_and_store_objects(),
            },
            vec![
                RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
                RouteCandidateFacts::rejected(RouteClass::TrustedSubstitute, "trusted-substitute-missing"),
                RouteCandidateFacts::rejected(RouteClass::ArchiveImport, "archive-candidate-missing"),
                RouteCandidateFacts::rejected(RouteClass::SourceBundle, "source-bundle-unavailable"),
                RouteCandidateFacts::eligible(
                    RouteClass::P2pRemoteBuilder,
                    "builder-capability-and-output-trust-match",
                )
                .requiring_network()
                .practical_only()
                .with_upload_summary(UploadSummary::new(
                    vec![UploadClass::StoreObject, UploadClass::Source],
                    12,
                    8_192,
                )),
                RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
            ],
        ));

        let upload = report.upload_summary.as_ref().expect("upload summary");
        assert_eq!(report.selected_route, RouteClass::P2pRemoteBuilder);
        assert_eq!(upload.object_count, 12);
        assert_eq!(upload.byte_count, 8_192);
        assert!(upload.classes.contains(&UploadClass::Source));
    }

    #[test]
    fn cli_remote_builder_plan_facts_select_remote_route_without_secrets() {
        let facts =
            RemoteBuilderPlanFacts::cli_configured(Some("builder-1"), true, 1).expect("remote facts configured");
        let report = route_plan_for_build_action_with_remote("build", Some("out=build"), Some(&facts));
        let json = serde_json::to_string(&report).expect("route report serializes");

        assert_eq!(report.selected_route, RouteClass::P2pRemoteBuilder);
        assert_eq!(report.selected_reason_code, "builder-capability-and-output-trust-match");
        assert!(report.selected_detail.as_deref().unwrap_or_default().contains("endpoint=builder-1"));
        assert!(report.upload_summary.is_some());
        assert!(!json.contains("secret"));
        assert!(!json.contains("ticket"));
    }

    #[test]
    fn cli_remote_builder_plan_rejects_missing_output_trust_before_local_build() {
        let facts =
            RemoteBuilderPlanFacts::cli_configured(Some("builder-1"), true, 0).expect("remote facts configured");
        let report = route_plan_for_build_action_with_remote("build", Some("out=build"), Some(&facts));

        assert_eq!(report.selected_route, RouteClass::LocalBuild);
        assert!(report.rejected_routes.iter().any(|route| {
            route.route == RouteClass::P2pRemoteBuilder && route.reason_code == "output-trust-missing"
        }));
    }

    #[test]
    fn remote_builder_plan_facts_reject_missing_input_capability_and_source_readiness() {
        let mut missing_inputs =
            RemoteBuilderPlanFacts::cli_configured(Some("builder-1"), true, 1).expect("remote facts configured");
        missing_inputs.concrete_inputs = false;
        let mut capability_mismatch = missing_inputs.clone();
        capability_mismatch.concrete_inputs = true;
        capability_mismatch.capabilities_match = false;
        let mut source_missing = capability_mismatch.clone();
        source_missing.capabilities_match = true;
        source_missing.source_inputs_ready = false;

        assert_eq!(remote_builder_candidate_from_facts(&missing_inputs).reason_code, "concrete-inputs-missing");
        assert_eq!(
            remote_builder_candidate_from_facts(&capability_mismatch).reason_code,
            "builder-capability-mismatch"
        );
        assert_eq!(remote_builder_candidate_from_facts(&source_missing).reason_code, "source-readiness-missing");
    }

    #[test]
    fn strong_claim_compatible_route_can_be_selected() {
        let report = plan_realization_route(RoutePlannerInput::new(
            RoutePolicy {
                network: NetworkPolicy::Online,
                requested_claim_strength: ClaimStrength::StrongActionCorrectness,
                upload_policy: UploadPolicy::local_only(),
            },
            vec![
                RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
                RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-strong-receipts-ready"),
            ],
        ));

        assert_eq!(report.selected_route, RouteClass::LocalBuild);
        assert_eq!(report.policy.requested_claim_strength, ClaimStrength::StrongActionCorrectness);
    }

    #[test]
    fn negative_route_facts_keep_specific_blocker_codes() {
        let report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
            RouteCandidateFacts::rejected(RouteClass::ArchiveImport, "archive-prefix-mismatch"),
            RouteCandidateFacts::rejected(RouteClass::SourceBundle, "missing-source-state"),
            RouteCandidateFacts::rejected(RouteClass::P2pRemoteBuilder, "builder-capability-mismatch"),
            RouteCandidateFacts::rejected(RouteClass::LocalBuild, "local-preflight-failed"),
        ]));

        let reasons = report.rejected_routes.iter().map(|route| route.reason_code.as_str()).collect::<Vec<_>>();
        assert!(reasons.contains(&"archive-prefix-mismatch"));
        assert!(reasons.contains(&"missing-source-state"));
        assert!(reasons.contains(&"builder-capability-mismatch"));
        assert!(reasons.contains(&"local-preflight-failed"));
    }

    #[test]
    fn remote_builder_ticket_without_output_trust_is_rejected() {
        let report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
            RouteCandidateFacts::rejected(RouteClass::P2pRemoteBuilder, "untrusted-builder-output-key")
                .requiring_network(),
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
        ]));

        assert_eq!(report.selected_route, RouteClass::LocalBuild);
        assert_eq!(report.rejected_routes[1].reason_code, "untrusted-builder-output-key");
    }

    #[test]
    fn upload_summary_rejects_overflow_without_panicking() {
        let err =
            UploadSummary::try_new(vec![UploadClass::Source], MAX_UPLOAD_OBJECTS.saturating_add(1), MAX_UPLOAD_BYTES)
                .expect_err("object limit must fail");

        assert_eq!(err, RoutePlanError::UploadObjectLimitExceeded {
            actual: MAX_UPLOAD_OBJECTS.saturating_add(1),
            limit: MAX_UPLOAD_OBJECTS,
        });
    }

    #[test]
    fn route_report_binds_base_descriptor_generation_and_selected_layer() {
        let digest = blake3::hash(b"route-store-evidence").to_hex().to_string();
        let mut report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::eligible(RouteClass::CachedLocal, "base-cache-hit"),
        ]));

        report
            .bind_store_evidence(
                Some(RouteStoreOverlayEvidence {
                    plan_blake3: digest.clone(),
                    bases: vec![RouteStoreOverlayBaseEvidence {
                        layer_index: MIN_ROUTE_STORE_LAYER_INDEX,
                        descriptor_blake3: digest.clone(),
                        generation_blake3: digest.clone(),
                    }],
                }),
                vec![RouteStoreLayerEvidence {
                    store_path: "/mantle/store/example".to_string(),
                    selected_layer: "base[1]".to_string(),
                    base_descriptor_blake3: Some(digest.clone()),
                    base_generation_blake3: Some(digest),
                }],
            )
            .expect("bounded route store evidence must bind");

        assert_eq!(report.selected_store_layers[0].selected_layer, "base[1]");
        assert!(report.selected_store_layers[0].base_descriptor_blake3.is_some());
    }

    #[test]
    fn route_report_rejects_oversized_layer_label() {
        let mut report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-build"),
        ]));
        let oversized_label = "x".repeat(MAX_ROUTE_STORE_LAYER_LABEL_BYTES.saturating_add(1));

        let error = report
            .bind_store_evidence(None, vec![RouteStoreLayerEvidence {
                store_path: "/mantle/store/example".to_string(),
                selected_layer: oversized_label,
                base_descriptor_blake3: None,
                base_generation_blake3: None,
            }])
            .expect_err("oversized route layer label must fail");

        assert_eq!(error, "route store layer label is invalid");
        assert!(report.selected_store_layers.is_empty());
    }

    #[test]
    fn route_reports_do_not_include_raw_secret_values() {
        let report = plan_realization_route(RoutePlannerInput::new(
            RoutePolicy {
                network: NetworkPolicy::Online,
                requested_claim_strength: ClaimStrength::Practical,
                upload_policy: UploadPolicy::allow_sources_and_store_objects(),
            },
            vec![
                RouteCandidateFacts::eligible(RouteClass::P2pRemoteBuilder, "remote-builder-hit")
                    .requiring_network()
                    .with_upload_summary(UploadSummary::new(vec![UploadClass::Source], 1, 128)),
            ],
        ));
        let json = serde_json::to_string(&report).expect("route report serializes");

        assert_eq!(report.selected_route, RouteClass::P2pRemoteBuilder);
        assert!(report.non_claim.contains("advisory"));
        assert!(!report.non_claim.contains("build success"));
        assert!(!json.contains("token"));
        assert!(!json.contains("private"));
    }
}
