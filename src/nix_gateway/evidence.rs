//! Production projection into Valence's accepted build-service evidence profile.
//!
//! This adapter records facts; neither a linked record nor a valid bundle grants
//! execution, store, billing or reuse authority. It never accepts bearer material.
use crunch_remote_core::resource_policy as resource;
use crunch_remote_core::resource_quota as quota;
use valence_core::build_service as v;

use crate::remote_build::gateway::GatewayAttemptRecord;

pub const MAX_EVIDENCE_BYTES: usize = 1_048_576;
pub const MAX_EVIDENCE_RECORDS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error(pub &'static str);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for Error {}

fn boundary() -> Vec<String> {
    vec![v::BUILD_SERVICE_REQUIRED_NON_CLAIM.to_owned()]
}
fn hex(bytes: &[u8; 32]) -> String {
    blake3::Hash::from(*bytes).to_hex().to_string()
}
fn json(value: &impl serde::Serialize) -> Result<String, Error> {
    serde_json::to_string(value).map_err(|_| Error("gateway-evidence-encoding"))
}

/// Public operation decision supplied by the current authority verifier. Do not
/// use a credential identity, private claim digest or token proof as its ID.
pub struct PublicAuthority<'a> {
    pub subject: &'a str,
    pub account: &'a str,
    pub project: &'a str,
    pub policy_identity: &'a str,
    pub operation: &'a str,
    pub decision_id: &'a str,
}

/// A committed observed transition, not an inferred execution timeline.
pub struct Transition<'a> {
    pub prior: crunch_nix_gateway::api::Phase,
    pub next: crunch_nix_gateway::api::Phase,
    pub sequence: u64,
    pub observed_unix_s: u64,
    pub reason: &'a str,
}
fn phase(value: crunch_nix_gateway::api::Phase) -> &'static str {
    use crunch_nix_gateway::api::Phase;
    match value {
        Phase::Queued => "queued",
        Phase::Running => "running",
        Phase::Succeeded => "succeeded",
        Phase::Failed => "failed",
        Phase::Cancelled => "cancelled",
    }
}

/// Mutable producer, never deserialized from a caller. Each mutator checks
/// against the original durable scope; finish runs the accepted validator.
pub struct GatewayEvidence<'a> {
    bundle: v::BuildServiceBundle,
    gateway: &'a GatewayAttemptRecord,
    request_identity: String,
    attempt_identity: String,
}

impl<'a> GatewayEvidence<'a> {
    // r[impl remote_builds.gateway_non_claims]
    pub fn new(
        record: &'a GatewayAttemptRecord,
        producer_policy: [u8; 32],
        entry: &quota::Entry,
    ) -> Result<Self, Error> {
        if record.reuse.is_some() {
            return Err(Error("gateway-evidence-reuse-no-reservation"));
        }
        let binding = record.binding.as_ref().ok_or(Error("gateway-evidence-binding-missing"))?;
        let r = &entry.reservation;
        if !crunch_remote_core::output::is_blake3_hex_digest(&record.public_id)
            || !crunch_nix_gateway_core::valid_store_path(&record.drv_path, true)
            || [&record.subject, &record.account, &record.project, &record.request_id]
                .iter()
                .any(|id| !resource::valid_id(id))
        {
            return Err(Error("gateway-evidence-public-identity"));
        }
        if record.request_digest != *blake3::hash(record.drv_path.as_bytes()).as_bytes()
            || r.job != binding.job_id
            || r.attempt != binding.attempt_id
            || r.fence != binding.fence_generation
            || r.project != record.project
            || r.account != record.account
            || r.request_digest != hex(&record.request_digest)
        {
            return Err(Error("gateway-evidence-scope-conflict"));
        }
        let mut ledger = quota::Ledger::default();
        ledger.entries.insert(r.attempt.as_str().to_owned(), entry.clone());
        ledger.validate().map_err(|_| Error("gateway-evidence-quota-invalid"))?;
        let request = v::ServiceRequestRecord {
            request_id: record.public_id.clone(),
            concrete_request_hash: hex(&record.request_digest),
            // The gateway's public operation ID is the durable, scope-bound
            // idempotency identity. Never export the caller's opaque key.
            idempotency_key: record.public_id.clone(),
            role: v::BuildServiceRole::Linked,
            non_claims: boundary(),
        };
        let request_identity = v::service_request_identity(&request);
        let attempt = v::AttemptRecord {
            attempt_id: binding.attempt_id.as_str().to_owned(),
            request_identity: request_identity.clone(),
            generation: u32::try_from(binding.fence_generation.get())
                .map_err(|_| Error("gateway-evidence-fence-overflow"))?,
            parent_attempt_identity: None,
            service_policy: hex(&producer_policy),
            role: v::BuildServiceRole::Linked,
            non_claims: boundary(),
        };
        let attempt_identity = v::attempt_identity(&attempt);
        let bundle = v::BuildServiceBundle {
            profile: v::BuildServiceEvidenceProfile {
                profile_id: v::BUILD_SERVICE_PROFILE_ID.into(),
                profile_version: 1,
                canonicalization: v::BUILD_SERVICE_CANONICALIZATION.into(),
                producer_policy: hex(&producer_policy),
                claimed_observation_classes: vec!["usage".into()],
                non_claims: boundary(),
            },
            service_requests: vec![request],
            attempts: vec![attempt],
            transitions: vec![],
            authority_decisions: vec![],
            machine_classes: vec![],
            resource_decisions: vec![],
            resource_observations: vec![],
            retry_edges: vec![],
            usage_records: vec![],
            build_results: vec![],
            reuse_edges: vec![],
            orchestration_jobs: vec![],
            external_statuses: vec![],
        };
        let mut out = Self {
            bundle,
            gateway: record,
            request_identity,
            attempt_identity,
        };
        out.record_usage(entry)?;
        Ok(out)
    }

    /// A consumer alias links the original execution; it is not a new attempt
    /// and must never create a second usage reservation or result producer.
    pub fn new_reuse(
        record: &'a GatewayAttemptRecord,
        producer_policy: [u8; 32],
        producer_bundle: &v::BuildServiceBundle,
    ) -> Result<Self, Error> {
        validate(producer_bundle)?;
        if !crunch_remote_core::output::is_blake3_hex_digest(&record.public_id)
            || !crunch_nix_gateway_core::valid_store_path(&record.drv_path, true)
            || [&record.subject, &record.account, &record.project, &record.request_id]
                .iter()
                .any(|id| !resource::valid_id(id))
        {
            return Err(Error("gateway-evidence-public-identity"));
        }
        let reuse = record.reuse.as_ref().ok_or(Error("gateway-evidence-reuse-missing"))?;
        let digest = reuse
            .producer_evidence
            .strip_suffix(".json")
            .ok_or(Error("gateway-evidence-reuse-producer-reference"))?;
        let bytes = serde_json::to_vec(producer_bundle).map_err(|_| Error("gateway-evidence-encoding"))?;
        if bytes.len() > MAX_EVIDENCE_BYTES || blake3::hash(&bytes).to_hex().as_str() != digest {
            return Err(Error("gateway-evidence-reuse-producer-content"));
        }
        let binding = record.binding.as_ref().ok_or(Error("gateway-evidence-binding-missing"))?;
        let producer_request = producer_bundle
            .service_requests
            .iter()
            .find(|r| r.request_id == reuse.producer_public_id)
            .ok_or(Error("gateway-evidence-reuse-request-missing"))?;
        let producer_identity = v::service_request_identity(producer_request);
        let result = producer_bundle
            .build_results
            .iter()
            .find(|r| r.result_id == hex(&reuse.result_evidence) && r.request_identity == producer_identity)
            .ok_or(Error("gateway-evidence-reuse-result-missing"))?;
        let attempt = producer_bundle
            .attempts
            .iter()
            .find(|a| v::attempt_identity(a) == result.attempt_identity)
            .ok_or(Error("gateway-evidence-reuse-parent-missing"))?;
        if reuse.consumer_project != record.project
            || record.public_id == reuse.producer_public_id
            || record.request_digest != *blake3::hash(record.drv_path.as_bytes()).as_bytes()
            || record.request_id != reuse.producer_request_id
            || producer_request.concrete_request_hash != hex(&record.request_digest)
            || record.result_evidence != Some(reuse.result_evidence)
            || attempt.attempt_id != binding.attempt_id.as_str()
            || u64::from(attempt.generation) != binding.fence_generation.get()
            || result.platform != record.platform
            || result.outputs.len() != record.produced_outputs.len()
            || result.outputs.iter().any(|o| {
                !record
                    .produced_outputs
                    .iter()
                    .any(|p| p.logical_path == o.path && p.content_digest_blake3 == o.content_hash)
            })
            || (reuse.producer_project != reuse.consumer_project && reuse.sharing_scope.is_none())
        {
            return Err(Error("gateway-evidence-reuse-scope"));
        }
        let request = v::ServiceRequestRecord {
            request_id: record.public_id.clone(),
            concrete_request_hash: hex(&record.request_digest),
            idempotency_key: record.public_id.clone(),
            role: v::BuildServiceRole::Linked,
            non_claims: boundary(),
        };
        let request_identity = v::service_request_identity(&request);
        let attempt_identity = result.attempt_identity.clone();
        let mut bundle = producer_bundle.clone();
        bundle.profile.producer_policy = hex(&producer_policy);
        bundle.profile.claimed_observation_classes = vec!["build".into(), "reuse".into()];
        bundle.service_requests.push(request);
        Ok(Self {
            bundle,
            gateway: record,
            request_identity,
            attempt_identity,
        })
    }

    /// Call only after current independent authority, sharing, signature,
    /// request/platform, output and CAS checks at the ordinary host boundary.
    pub fn record_alias_reuse(
        &mut self,
        authorization_identity: &str,
        output_check_ref: &str,
        cas_observation: &str,
    ) -> Result<(), Error> {
        let reuse = self.gateway.reuse.as_ref().ok_or(Error("gateway-evidence-reuse-missing"))?;
        let result = self
            .bundle
            .build_results
            .iter()
            .find(|r| r.result_id == hex(&reuse.result_evidence) && r.attempt_identity == self.attempt_identity)
            .ok_or(Error("gateway-evidence-reuse-result-missing"))?;
        let producer_authority = self
            .bundle
            .authority_decisions
            .iter()
            .find(|a| {
                a.attempt_identity == result.attempt_identity && a.operation == "build-submit" && a.decision == "allow"
            })
            .ok_or(Error("gateway-evidence-reuse-producer-authority"))?;
        let projection: serde_json::Value = serde_json::from_str(&producer_authority.subject_projection)
            .map_err(|_| Error("gateway-evidence-reuse-producer-scope"))?;
        if projection.get("project").and_then(serde_json::Value::as_str) != Some(reuse.producer_project.as_str())
            || projection.get("account").and_then(serde_json::Value::as_str) != Some(reuse.producer_account.as_str())
        {
            return Err(Error("gateway-evidence-reuse-producer-scope"));
        }
        self.bundle.reuse_edges.push(v::ReuseEdgeRecord {
            producer_result_identity: v::build_result_identity(result),
            consumer_request_identity: self.request_identity.clone(),
            sharing_policy: reuse.sharing_scope.clone().unwrap_or_else(|| "private-same-project".into()),
            authorization_decision_identity: Some(authorization_identity.into()),
            request_compatible: true,
            platform_compatible: true,
            output_check_ref: Some(output_check_ref.into()),
            signature_ref: Some(result.producer_signature_ref.clone()),
            cas_observation: Some(cas_observation.into()),
            usability_outcome: "usable".into(),
            role: v::BuildServiceRole::Verified,
            non_claims: boundary(),
        });
        Ok(())
    }

    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
    // I3).
    #[allow(dead_code)]
    pub fn attempt_identity(&self) -> &str {
        &self.attempt_identity
    }
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
    // I3).
    #[allow(dead_code)]
    pub fn request_identity(&self) -> &str {
        &self.request_identity
    }

    pub fn record_durable_facts(&mut self) -> Result<(), Error> {
        if self.gateway.reuse.is_some() {
            return Err(Error("gateway-evidence-reuse-not-execution"));
        }
        let record = self.gateway;
        self.record_authority(PublicAuthority {
            subject: &record.subject,
            account: &record.account,
            project: &record.project,
            policy_identity: &record.authority_policy,
            operation: "build-submit",
            decision_id: &record.authority_decision,
        })?;
        // The initial queued entry establishes the observed origin. It is not
        // fabricated into a queued-to-queued transition with an invented prior.
        for pair in record.transitions.windows(2) {
            if pair[1].sequence != pair[0].sequence.checked_add(1).ok_or(Error("gateway-evidence-sequence-overflow"))?
                || pair[1].observed_unix_s < pair[0].observed_unix_s
            {
                return Err(Error("gateway-evidence-transition-conflict"));
            }
            self.record_transition(Transition {
                prior: gateway_phase(pair[0].phase),
                next: gateway_phase(pair[1].phase),
                sequence: pair[1].sequence,
                observed_unix_s: pair[1].observed_unix_s,
                reason: &pair[1].reason,
            })?;
        }
        Ok(())
    }

    pub fn record_authority(&mut self, fact: PublicAuthority<'_>) -> Result<String, Error> {
        if fact.subject != self.gateway.subject
            || fact.account != self.gateway.account
            || fact.project != self.gateway.project
        {
            return Err(Error("gateway-evidence-authority-scope"));
        }
        let record = v::AuthorityDecisionRecord {
            decision_id: fact.decision_id.into(),
            attempt_identity: self.attempt_identity.clone(),
            subject_projection: json(
                &serde_json::json!({"subject":fact.subject,"account":fact.account,"project":fact.project,"request_identity":self.request_identity,"platform":self.gateway.platform}),
            )?,
            policy_identity: fact.policy_identity.strip_prefix("blake3:").unwrap_or(fact.policy_identity).into(),
            operation: fact.operation.into(),
            decision: "allow".into(),
            verifier_evidence_ref: None,
            role: v::BuildServiceRole::Verified,
            non_claims: boundary(),
        };
        let identity = v::authority_decision_identity(&record);
        self.bundle.authority_decisions.push(record);
        Ok(identity)
    }

    pub fn record_transition(&mut self, fact: Transition<'_>) -> Result<(), Error> {
        if self.gateway.reuse.is_some() {
            return Err(Error("gateway-evidence-reuse-not-execution"));
        }
        if self.bundle.transitions.len() >= MAX_EVIDENCE_RECORDS || fact.prior == fact.next {
            return Err(Error("gateway-evidence-transition-invalid"));
        }
        self.bundle.transitions.push(v::AttemptTransitionRecord {
            attempt_identity: self.attempt_identity.clone(),
            prior_state: phase(fact.prior).into(),
            next_state: phase(fact.next).into(),
            sequence: u32::try_from(fact.sequence).map_err(|_| Error("gateway-evidence-sequence-overflow"))?,
            observed_at: utc(fact.observed_unix_s)?,
            producer: "mantle-gateway".into(),
            reason: fact.reason.into(),
            role: v::BuildServiceRole::RecordedOnly,
            non_claims: boundary(),
        });
        Ok(())
    }

    fn record_usage(&mut self, entry: &quota::Entry) -> Result<(), Error> {
        let r = &entry.reservation;
        let schedule = json(&r.schedule)?;
        let scope = json(&serde_json::json!({"account":r.account,"project":r.project}))?;
        let window = json(
            &serde_json::json!({"id":r.schedule.window_id,"start_unix_s":r.schedule.window_start_s,"end_unix_s":r.schedule.window_end_s}),
        )?;
        let id = format!("{}:{}", r.attempt.as_str(), r.fence.get());
        self.bundle.usage_records.push(v::UsageRecord {
            usage_id: format!("{id}:reservation"),
            attempt_identity: self.attempt_identity.clone(),
            usage_kind: "reservation".into(),
            unit_schedule: schedule.clone(),
            window: window.clone(),
            scope: scope.clone(),
            outcome: json(&serde_json::json!({"reserved_units":r.units,"state":"reserved"}))?,
            role: v::BuildServiceRole::RecordedOnly,
            non_claims: boundary(),
        });
        if entry.reconciliation != quota::Reconciliation::Reserved {
            self.bundle.usage_records.push(v::UsageRecord {
                usage_id: format!("{id}:reconciliation"),
                attempt_identity: self.attempt_identity.clone(),
                usage_kind: "reconciliation".into(),
                unit_schedule: schedule,
                window,
                scope,
                outcome: json(&entry.reconciliation)?,
                role: v::BuildServiceRole::RecordedOnly,
                non_claims: boundary(),
            });
        }
        Ok(())
    }

    // r[impl remote_builds.resource_observations]
    pub fn record_observation(&mut self, observation: &resource::Observation) -> Result<String, Error> {
        if self.gateway.reuse.is_some() {
            return Err(Error("gateway-evidence-reuse-not-execution"));
        }
        let binding = self.gateway.binding.as_ref().ok_or(Error("gateway-evidence-binding-missing"))?;
        if observation.attempt_id != binding.attempt_id || observation.schema != resource::OBSERVATION_SCHEMA {
            return Err(Error("gateway-evidence-observation-scope"));
        }
        let record = observation_record(observation, self.attempt_identity.clone())?;
        let identity = v::resource_observation_identity(&record);
        self.bundle.resource_observations.push(record);
        Ok(identity)
    }

    pub fn record_authenticated_usage(
        &mut self,
        usage: &crate::remote_build::usage::RemoteAuthenticatedUsage,
    ) -> Result<String, Error> {
        if self.gateway.reuse.is_some() {
            return Err(Error("gateway-evidence-reuse-not-execution"));
        }
        let binding = self.gateway.binding.as_ref().ok_or(Error("gateway-evidence-binding-missing"))?;
        let expected_result =
            self.gateway.result_evidence.as_ref().map(hex).unwrap_or_else(|| {
                crate::remote_build::usage::failure_result_digest(binding, &self.gateway.request_id)
            });
        usage
            .validate_current(binding, &self.gateway.request_id, &expected_result)
            .map_err(|_| Error("gateway-evidence-measurement-scope"))?;
        let record = signed_usage_record(usage, self.attempt_identity.clone())?;
        let identity = v::resource_observation_identity(&record);
        self.bundle.resource_observations.push(record);
        Ok(identity)
    }

    pub fn record_selection_snapshot(&mut self, value: &serde_json::Value) -> Result<(), Error> {
        use serde::Deserialize;
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Snapshot {
            classes: Vec<resource::MachineClass>,
            decision: resource::RolloutDecision,
            replay_input_blake3: String,
            replay_input_json: Vec<u8>,
            non_claims: Vec<String>,
        }
        let snapshot = Snapshot::deserialize(value).map_err(|_| Error("gateway-evidence-selection-snapshot"))?;
        if snapshot.non_claims.iter().map(String::as_str).ne(resource::NON_CLAIMS) {
            return Err(Error("gateway-evidence-selection-boundary"));
        }
        let evaluation = crunch_build::distributed::PolicyEvaluation {
            classes: snapshot.classes,
            decision: snapshot.decision,
            replay_input_blake3: snapshot.replay_input_blake3,
            replay_input_json: snapshot.replay_input_json,
            non_claims: &resource::NON_CLAIMS,
        };
        if serde_json::to_value(&evaluation).map_err(|_| Error("gateway-evidence-encoding"))? != *value {
            return Err(Error("gateway-evidence-selection-snapshot-fields"));
        }
        self.record_selection(&evaluation)
    }

    // r[impl remote_builds.replayable_resource_selection]
    pub fn record_selection(&mut self, evaluation: &crunch_build::distributed::PolicyEvaluation) -> Result<(), Error> {
        if self.gateway.reuse.is_some() {
            return Err(Error("gateway-evidence-reuse-not-execution"));
        }
        if !self.gateway.retries.is_empty() {
            return Err(Error("gateway-evidence-retry-use-escalation-selection"));
        }
        let replay = Replay::read(evaluation)?;
        let selected = evaluation.decision.active.as_ref().map_err(|_| Error("gateway-evidence-selection-rejected"))?;
        let mut classes = Vec::new();
        for class in &replay.classes {
            classes.push(class_record(class)?);
        }
        let identities: Vec<_> = classes.iter().map(v::machine_class_identity).collect();
        let selected_identity =
            identities.get(selected.class_index).ok_or(Error("gateway-evidence-selection-index"))?.clone();
        let mut history_links = Vec::new();
        for (index, observation) in replay.history.iter().enumerate() {
            if selected.samples == 0 || selected.exclusions.iter().any(|(excluded, _)| *excluded == index) {
                continue;
            }
            let record = self
                .bundle
                .resource_observations
                .iter()
                .find(|r| r.observation_id == observation.attempt_id.as_str())
                .ok_or(Error("gateway-evidence-history-link-missing"))?;
            if observation_record(observation, record.attempt_identity.clone())? != *record {
                return Err(Error("gateway-evidence-history-content-conflict"));
            }
            history_links.push(v::resource_observation_identity(record));
        }
        let eligible = replay
            .classes
            .iter()
            .enumerate()
            .filter_map(|(index, class)| {
                // The selector itself, not this adapter, owns hard-feature eligibility.
                let fits = resource::select(resource::SelectionRequest {
                    declared: replay.declared(),
                    features: &replay.features,
                    family: &replay.family,
                    classes: std::slice::from_ref(class),
                    history: &[],
                    now: replay.now_unix_s,
                    quota: replay.quota(),
                    policy: replay.policy,
                })
                .is_ok();
                (class.available && class.existing_admission_compatible && fits).then(|| identities[index].clone())
            })
            .collect();
        self.bundle.resource_decisions.push(v::ResourceDecisionRecord {
            decision_id: evaluation.replay_input_blake3.clone(),
            attempt_identity: self.attempt_identity.clone(),
            declared_minima: vec![json(&replay.requirements)?, json(&replay.features)?],
            eligible_class_identities: eligible,
            selected_class_identity: selected_identity,
            policy: String::from_utf8(evaluation.replay_input_json.clone())
                .map_err(|_| Error("gateway-evidence-replay-json"))?,
            compatible_observation_identities: history_links,
            reasons: vec![json(&evaluation.decision)?],
            role: v::BuildServiceRole::Verified,
            non_claims: boundary(),
        });
        for class in classes {
            if !self.bundle.machine_classes.contains(&class) {
                self.bundle.machine_classes.push(class);
            }
        }
        Ok(())
    }

    /// Only after ordinary signed-result admission and current CAS checks. The
    /// references name public evidence, not the signature bytes or private key.
    pub fn record_result(
        &mut self,
        outputs: Vec<v::BuildOutputRef>,
        platform: &str,
        policy: &str,
        signature_ref: &str,
        cas_observation: &str,
    ) -> Result<String, Error> {
        if self.gateway.reuse.is_some() {
            return Err(Error("gateway-evidence-reuse-not-execution"));
        }
        if self.gateway.result_evidence.is_none()
            || outputs.len() != self.gateway.outputs.len()
            || outputs.len() != self.gateway.produced_outputs.len()
            || outputs.iter().any(|output| {
                !self.gateway.outputs.contains(&output.path)
                    || !self.gateway.produced_outputs.iter().any(|produced| {
                        produced.logical_path == output.path
                            && produced.content_digest_blake3 == output.content_hash
                            && !produced.path_info_signing_key_id.is_empty()
                    })
            })
        {
            return Err(Error("gateway-evidence-result-conflict"));
        }
        let result = v::BuildResultRecord {
            result_id: hex(&self.gateway.result_evidence.ok_or(Error("gateway-evidence-result-missing"))?),
            request_identity: self.request_identity.clone(),
            attempt_identity: self.attempt_identity.clone(),
            platform: platform.into(),
            policy: policy.into(),
            outputs,
            producer_signature_ref: signature_ref.into(),
            cas_observation: cas_observation.into(),
            role: v::BuildServiceRole::Signed,
            non_claims: boundary(),
        };
        let identity = v::build_result_identity(&result);
        self.bundle.build_results.push(result);
        self.bundle.profile.claimed_observation_classes.push("build".into());
        Ok(identity)
    }

    pub fn record_retained_retry(
        &mut self,
        retained: &crate::remote_build::gateway::GatewayRetryRecord,
        predecessor: &v::BuildServiceBundle,
        usage: &crate::remote_build::usage::RemoteAuthenticatedUsage,
    ) -> Result<(), Error> {
        use serde::Deserialize;

        use crate::remote_build::usage::RemoteUsageCompleteness;
        use crate::remote_build::usage::RemoteUsageOomEvidence;
        validate(predecessor)?;
        let digest = retained
            .predecessor_evidence
            .strip_suffix(".json")
            .ok_or(Error("gateway-evidence-retry-reference"))?;
        let bytes = serde_json::to_vec(predecessor).map_err(|_| Error("gateway-evidence-encoding"))?;
        if bytes.len() > MAX_EVIDENCE_BYTES || blake3::hash(&bytes).to_hex().as_str() != digest {
            return Err(Error("gateway-evidence-retry-content"));
        }
        let replay =
            EscalationReplay::deserialize(&retained.replay_json).map_err(|_| Error("gateway-evidence-retry-replay"))?;
        if serde_json::to_value(&replay).map_err(|_| Error("gateway-evidence-encoding"))? != retained.replay_json {
            return Err(Error("gateway-evidence-retry-replay-fields"));
        }
        let failure =
            crate::remote_build::usage::failure_result_digest(&retained.predecessor, &self.gateway.request_id);
        usage
            .validate_current(&retained.predecessor, &self.gateway.request_id, &failure)
            .map_err(|_| Error("gateway-evidence-retry-usage"))?;
        if !matches!(usage.oom_evidence(),RemoteUsageOomEvidence::DedicatedCgroupOom {oom_kill_delta} if oom_kill_delta>0)
            || usage.measurement_completeness() != RemoteUsageCompleteness::DedicatedCgroup
            || replay.evidence.category != resource::OomCategory::LinuxCgroupV2Kill
            || !replay.evidence.trusted_positive_platform_evidence
            || replay.schema != "mantle-resource-escalation-replay-v1"
            || replay.classes.len() > MAX_EVIDENCE_RECORDS
            || replay.prior.id != retained.machine_class
            || replay.prior.reservation_units != retained.reservation_units
            || replay.policy.version != retained.policy_version
            || replay.context.job != retained.predecessor.job_id
            || replay.context.attempt != retained.predecessor.attempt_id
            || replay.context.fence != retained.predecessor.fence_generation
            || replay.context.next_attempt != retained.successor.attempt_id
            || replay.context.next_fence != retained.successor.fence_generation
            || retained.predecessor.job_id != retained.successor.job_id
        {
            return Err(Error("gateway-evidence-retry-source-conflict"));
        }
        let prior = predecessor
            .attempts
            .iter()
            .find(|a| {
                a.attempt_id == retained.predecessor.attempt_id.as_str()
                    && u64::from(a.generation) == retained.predecessor.fence_generation.get()
            })
            .ok_or(Error("gateway-evidence-retry-parent-missing"))?;
        let parent = v::attempt_identity(prior);
        let observation = signed_usage_record(usage, parent.clone())?;
        let class_identity = v::machine_class_identity(&class_record(&replay.prior)?);
        let selected_prior = predecessor
            .resource_decisions
            .iter()
            .any(|d| d.attempt_identity == parent && d.selected_class_identity == class_identity)
            || predecessor
                .retry_edges
                .iter()
                .any(|e| e.successor_attempt_identity == parent && e.next_class_identity == class_identity);
        if prior.request_identity != self.request_identity
            || predecessor.service_requests.iter().any(|r| v::service_request_identity(r) != self.request_identity)
            || !predecessor.resource_observations.contains(&observation)
            || !selected_prior
        {
            return Err(Error("gateway-evidence-retry-observation-link"));
        }
        let context = replay.context.borrowed();
        let evidence = replay.evidence.borrowed();
        let quota = replay.quota.borrowed();
        let policy = replay.policy.into();
        let plan = resource::plan_escalation(resource::EscalationRequest {
            controls: replay.controls,
            context: context.clone(),
            evidence: evidence.clone(),
            prior: &replay.prior,
            classes: &replay.classes,
            quota,
            policy,
        })
        .map_err(|_| Error("gateway-evidence-retry-denied"))?;
        if plan != replay.plan {
            return Err(Error("gateway-evidence-retry-replay-conflict"));
        }
        if self.gateway.binding.as_ref() != Some(&retained.successor) {
            return Ok(());
        }
        let event = usage.event_digest_blake3();
        self.record_retry_inner(
            RetryFacts {
                predecessor_bundle: predecessor,
                controls: replay.controls,
                context,
                evidence,
                prior: &replay.prior,
                classes: &replay.classes,
                quota,
                policy,
                plan: &plan,
            },
            Some(&event),
        )?;
        let edge = self.bundle.retry_edges.last_mut().ok_or(Error("gateway-evidence-retry-link-missing"))?;
        edge.trigger_evidence = json(
            &serde_json::json!({"observation_identity":v::resource_observation_identity(&observation),"signed_event_identity":event,"oom":usage.oom_evidence()}),
        )?;
        edge.policy = json(&retained.replay_json)?;
        Ok(())
    }

    // r[impl remote_builds.positive_oom_retry]
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
    // I3).
    #[allow(dead_code)]
    pub fn record_retry(&mut self, facts: RetryFacts<'_>) -> Result<(), Error> {
        self.record_retry_inner(facts, None)
    }

    fn record_retry_inner(&mut self, facts: RetryFacts<'_>, authenticated_event: Option<&str>) -> Result<(), Error> {
        if self.gateway.reuse.is_some() {
            return Err(Error("gateway-evidence-reuse-not-execution"));
        }
        let planned = resource::plan_escalation(resource::EscalationRequest {
            controls: facts.controls,
            context: facts.context.clone(),
            evidence: facts.evidence.clone(),
            prior: facts.prior,
            classes: facts.classes,
            quota: facts.quota,
            policy: facts.policy,
        })
        .map_err(|_| Error("gateway-evidence-retry-denied"))?;
        let binding = self.gateway.binding.as_ref().ok_or(Error("gateway-evidence-binding-missing"))?;
        if planned != *facts.plan
            || planned.next_attempt != binding.attempt_id
            || planned.next_fence != binding.fence_generation
            || !self.bundle.authority_decisions.is_empty()
            || !self.bundle.transitions.is_empty()
            || !self.bundle.resource_decisions.is_empty()
            || !self.bundle.resource_observations.is_empty()
            || !self.bundle.build_results.is_empty()
        {
            return Err(Error("gateway-evidence-retry-conflict"));
        }
        validate(facts.predecessor_bundle)?;
        let predecessor = facts
            .predecessor_bundle
            .attempts
            .iter()
            .find(|a| a.attempt_id == planned.predecessor.as_str())
            .ok_or(Error("gateway-evidence-retry-parent-missing"))?;
        let parent = v::attempt_identity(predecessor);
        let expected_oom =
            serde_json::to_value(facts.evidence.category).map_err(|_| Error("gateway-evidence-encoding"))?;
        let positive_observation = facts.predecessor_bundle.resource_observations.iter().any(|o| {
            if let Some(event) = authenticated_event {
                return o.attempt_identity == parent && o.observation_id == event;
            }
            if o.attempt_identity != parent || o.terminal_outcome != "\"Failed\"" {
                return false;
            }
            serde_json::from_str::<serde_json::Value>(&o.compatibility_scope).ok().is_some_and(|scope| {
                scope.get("oom") == Some(&expected_oom)
                    && scope.get("machine_class").and_then(serde_json::Value::as_str) == Some(facts.prior.id.as_str())
                    && scope.get("features").and_then(|f| f.get("platform")).and_then(serde_json::Value::as_str)
                        == Some(facts.evidence.platform)
            })
        });
        if predecessor.generation.checked_add(1) != Some(self.bundle.attempts[0].generation) || !positive_observation {
            return Err(Error("gateway-evidence-retry-parent-conflict"));
        }
        self.bundle.attempts[0].parent_attempt_identity = Some(parent.clone());
        self.attempt_identity = v::attempt_identity(&self.bundle.attempts[0]);
        for usage in &mut self.bundle.usage_records {
            usage.attempt_identity = self.attempt_identity.clone();
        }
        self.include(facts.predecessor_bundle)?;
        let prior = class_record(facts.prior)?;
        let next = class_record(&facts.classes[planned.class_index])?;
        let edge = v::RetryEdgeRecord {
            predecessor_attempt_identity: parent,
            successor_attempt_identity: self.attempt_identity.clone(),
            trigger: "oom_escalation".into(),
            trigger_evidence: json(
                &serde_json::json!({"category":facts.evidence.category,"job":facts.evidence.job,"attempt":facts.evidence.attempt,"fence":facts.evidence.fence,"platform":facts.evidence.platform}),
            )?,
            prior_class_identity: v::machine_class_identity(&prior),
            next_class_identity: v::machine_class_identity(&next),
            limit_state: json(
                &serde_json::json!({"retries":facts.context.retries,"cumulative_units":facts.context.cumulative_units,"elapsed_ms":facts.context.elapsed_ms,
                "max_retries":facts.policy.max_retries,"largest_class_order":facts.policy.largest_class_order,"max_cumulative_units":facts.policy.max_cumulative_units,"max_wall_ms":facts.policy.max_wall_ms}),
            )?,
            policy: format!("mantle-resource-escalation-v{}", facts.policy.version),
            role: v::BuildServiceRole::Verified,
            non_claims: boundary(),
        };
        for class in [prior, next] {
            if !self.bundle.machine_classes.contains(&class) {
                self.bundle.machine_classes.push(class);
            }
        }
        self.bundle.retry_edges.push(edge);
        self.bundle.profile.claimed_observation_classes.push("retry".into());
        Ok(())
    }

    // r[impl remote_builds.authorized_result_sharing]
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
    // I3).
    #[allow(dead_code)]
    pub fn record_reuse(&mut self, facts: ReuseFacts<'_>) -> Result<(), Error> {
        validate(facts.producer_bundle)?;
        let result = facts
            .producer_bundle
            .build_results
            .iter()
            .find(|r| v::build_result_identity(r) == facts.producer_result_identity)
            .ok_or(Error("gateway-evidence-reuse-result-missing"))?;
        let producer_authority = facts
            .producer_bundle
            .authority_decisions
            .iter()
            .find(|a| {
                a.attempt_identity == result.attempt_identity && a.decision == "allow" && a.operation == "build-submit"
            })
            .ok_or(Error("gateway-evidence-reuse-producer-authority"))?;
        let projection: serde_json::Value = serde_json::from_str(&producer_authority.subject_projection)
            .map_err(|_| Error("gateway-evidence-reuse-producer-scope"))?;
        if projection.get("project").and_then(serde_json::Value::as_str) != Some(facts.producer_project) {
            return Err(Error("gateway-evidence-reuse-producer-scope"));
        }
        let link = resource::authorize_sharing(resource::SharingRequest {
            controls: facts.controls,
            producer: facts.producer_project,
            consumer: &self.gateway.project,
            result: &result.result_id,
            scope: facts.scope,
            checks: facts.checks,
        })
        .map_err(|_| Error("gateway-evidence-reuse-denied"))?;
        let request = facts
            .producer_bundle
            .service_requests
            .iter()
            .find(|r| v::service_request_identity(r) == result.request_identity)
            .ok_or(Error("gateway-evidence-reuse-request-missing"))?;
        if request.concrete_request_hash != hex(&self.gateway.request_digest)
            || facts.signature_ref != result.producer_signature_ref
        {
            return Err(Error("gateway-evidence-reuse-result-conflict"));
        }
        self.include(facts.producer_bundle)?;
        self.bundle.reuse_edges.push(v::ReuseEdgeRecord {
            producer_result_identity: facts.producer_result_identity.into(),
            consumer_request_identity: self.request_identity.clone(),
            sharing_policy: link.scope.unwrap_or_else(|| "private-same-project".into()),
            authorization_decision_identity: Some(facts.authorization_identity.into()),
            request_compatible: true,
            platform_compatible: true,
            output_check_ref: Some(facts.output_check_ref.into()),
            signature_ref: Some(facts.signature_ref.into()),
            cas_observation: Some(facts.cas_observation.into()),
            usability_outcome: "usable".into(),
            role: v::BuildServiceRole::Verified,
            non_claims: boundary(),
        });
        self.bundle.profile.claimed_observation_classes.push("reuse".into());
        Ok(())
    }

    /// Merge validated producer/ancestor bundles. Exact repeated records are
    /// deduplicated; conflicting records remain visible to fail-closed checks.
    pub fn include(&mut self, bundle: &v::BuildServiceBundle) -> Result<(), Error> {
        validate(bundle)?;
        macro_rules! append { ($($field:ident),+) => { $(for record in &bundle.$field { if !self.bundle.$field.contains(record) { self.bundle.$field.push(record.clone()); } })+ }; }
        append!(
            service_requests,
            attempts,
            transitions,
            authority_decisions,
            machine_classes,
            resource_decisions,
            resource_observations,
            retry_edges,
            usage_records,
            build_results,
            reuse_edges
        );
        Ok(())
    }

    pub fn finish(self) -> Result<Export, Error> {
        let current_authority = self.bundle.authority_decisions.iter().any(|a| {
            a.attempt_identity == self.attempt_identity
                && serde_json::from_str::<serde_json::Value>(&a.subject_projection).ok().is_some_and(|p| {
                    p.get("request_identity").and_then(serde_json::Value::as_str)
                        == Some(self.request_identity.as_str())
                        && p.get("subject").and_then(serde_json::Value::as_str) == Some(self.gateway.subject.as_str())
                        && p.get("project").and_then(serde_json::Value::as_str) == Some(self.gateway.project.as_str())
                })
        });
        if !current_authority {
            return Err(Error("gateway-evidence-authority-missing"));
        }
        if self.gateway.reuse.is_some()
            && !self.bundle.reuse_edges.iter().any(|e| e.consumer_request_identity == self.request_identity)
        {
            return Err(Error("gateway-evidence-current-reuse-missing"));
        }
        for retry in &self.gateway.retries {
            let predecessor = self
                .bundle
                .attempts
                .iter()
                .find(|a| {
                    a.attempt_id == retry.predecessor.attempt_id.as_str()
                        && u64::from(a.generation) == retry.predecessor.fence_generation.get()
                })
                .map(v::attempt_identity)
                .ok_or(Error("gateway-evidence-retry-chain-missing"))?;
            let successor = self
                .bundle
                .attempts
                .iter()
                .find(|a| {
                    a.attempt_id == retry.successor.attempt_id.as_str()
                        && u64::from(a.generation) == retry.successor.fence_generation.get()
                })
                .map(v::attempt_identity)
                .ok_or(Error("gateway-evidence-retry-chain-missing"))?;
            if !self
                .bundle
                .retry_edges
                .iter()
                .any(|e| e.predecessor_attempt_identity == predecessor && e.successor_attempt_identity == successor)
            {
                return Err(Error("gateway-evidence-retry-chain-missing"));
            }
        }
        let report = validate(&self.bundle)?;
        let bytes = serde_json::to_vec(&self.bundle).map_err(|_| Error("gateway-evidence-encoding"))?;
        if bytes.len() > MAX_EVIDENCE_BYTES {
            return Err(Error("gateway-evidence-byte-limit"));
        }
        let digest = blake3::hash(&bytes).to_hex().to_string();
        Ok(Export {
            bundle: self.bundle,
            report,
            bytes,
            digest_blake3: digest,
        })
    }
}

// Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
// I3).
#[allow(dead_code)]
pub struct Export {
    pub bundle: v::BuildServiceBundle,
    pub report: v::BuildServiceReport,
    pub bytes: Vec<u8>,
    pub digest_blake3: String,
}
fn measurement(name: &str, value: u64, unit: &str) -> v::ResourceMeasurement {
    v::ResourceMeasurement {
        name: name.into(),
        value,
        unit: unit.into(),
    }
}

/// Verify retained bytes against a separately retained public artifact digest.
/// Recomputing a digest from untrusted bytes and passing it here proves nothing.
pub fn verify_bytes(bytes: &[u8], expected_blake3: &str) -> Result<v::BuildServiceBundle, Error> {
    if bytes.len() > MAX_EVIDENCE_BYTES || blake3::hash(bytes).to_hex().as_str() != expected_blake3 {
        return Err(Error("gateway-evidence-content-mismatch"));
    }
    let bundle: v::BuildServiceBundle = serde_json::from_slice(bytes).map_err(|_| Error("gateway-evidence-json"))?;
    // This reader accepts retained bytes from this producer, not arbitrary
    // reformatting. Exact encoding also rejects duplicate JSON control keys
    // which a generic Value parser would otherwise silently overwrite.
    if serde_json::to_vec(&bundle).map_err(|_| Error("gateway-evidence-json"))? != bytes {
        return Err(Error("gateway-evidence-producer-encoding"));
    }
    validate(&bundle)?;
    Ok(bundle)
}

fn validate(bundle: &v::BuildServiceBundle) -> Result<v::BuildServiceReport, Error> {
    let counts = [
        bundle.service_requests.len(),
        bundle.attempts.len(),
        bundle.transitions.len(),
        bundle.authority_decisions.len(),
        bundle.machine_classes.len(),
        bundle.resource_decisions.len(),
        bundle.resource_observations.len(),
        bundle.retry_edges.len(),
        bundle.usage_records.len(),
        bundle.build_results.len(),
        bundle.reuse_edges.len(),
        bundle.orchestration_jobs.len(),
        bundle.external_statuses.len(),
    ];
    if counts.iter().any(|count| *count > MAX_EVIDENCE_RECORDS) {
        return Err(Error("gateway-evidence-record-limit"));
    }
    let exact_boundary = |claims: &[String]| claims.iter().any(|claim| claim == v::BUILD_SERVICE_REQUIRED_NON_CLAIM);
    if !exact_boundary(&bundle.profile.non_claims) {
        return Err(Error("gateway-evidence-boundary"));
    }
    macro_rules! boundaries { ($($field:ident),+) => { $(if bundle.$field.iter().any(|record| !exact_boundary(&record.non_claims)) { return Err(Error("gateway-evidence-boundary")); })+ }; }
    boundaries!(
        service_requests,
        attempts,
        transitions,
        authority_decisions,
        machine_classes,
        resource_decisions,
        resource_observations,
        retry_edges,
        usage_records,
        build_results,
        reuse_edges,
        orchestration_jobs,
        external_statuses
    );
    let report = v::validate_build_service_bundle(bundle);
    if !report.valid {
        return Err(Error("gateway-evidence-profile-invalid"));
    }
    if bundle.resource_observations.iter().any(|r| r.role != v::BuildServiceRole::RecordedOnly)
        || bundle.usage_records.iter().any(|r| r.role != v::BuildServiceRole::RecordedOnly)
        || bundle.transitions.iter().any(|r| r.role != v::BuildServiceRole::RecordedOnly)
        || bundle.build_results.iter().any(|r| r.role != v::BuildServiceRole::Signed)
    {
        return Err(Error("gateway-evidence-role-conflict"));
    }
    if bundle.service_requests.iter().any(|r| r.role != v::BuildServiceRole::Linked)
        || bundle.attempts.iter().any(|r| r.role != v::BuildServiceRole::Linked)
        || bundle.machine_classes.iter().any(|r| r.role != v::BuildServiceRole::Linked)
        || bundle
            .authority_decisions
            .iter()
            .any(|r| r.role != v::BuildServiceRole::Verified || r.decision_id.is_empty())
        || bundle.resource_decisions.iter().any(|r| r.role != v::BuildServiceRole::Verified)
        || bundle.retry_edges.iter().any(|r| r.role != v::BuildServiceRole::Verified)
        || bundle.reuse_edges.iter().any(|r| r.role != v::BuildServiceRole::Verified)
    {
        return Err(Error("gateway-evidence-role-conflict"));
    }
    for (index, record) in bundle.usage_records.iter().enumerate() {
        if bundle.usage_records[..index].iter().any(|prior| prior.usage_id == record.usage_id) {
            return Err(Error("gateway-evidence-charge-conflict"));
        }
    }
    for result in &bundle.build_results {
        if !bundle
            .attempts
            .iter()
            .any(|a| v::attempt_identity(a) == result.attempt_identity && a.request_identity == result.request_identity)
        {
            return Err(Error("gateway-evidence-result-scope"));
        }
    }
    for edge in &bundle.reuse_edges {
        if edge.usability_outcome == "usable" {
            let authority = bundle
                .authority_decisions
                .iter()
                .find(|a| Some(v::authority_decision_identity(a)) == edge.authorization_decision_identity);
            let consumer_projection =
                authority.and_then(|a| serde_json::from_str::<serde_json::Value>(&a.subject_projection).ok());
            let producer = bundle
                .build_results
                .iter()
                .find(|r| v::build_result_identity(r) == edge.producer_result_identity)
                .ok_or(Error("gateway-evidence-reuse-result-missing"))?;
            let producer_request = bundle
                .service_requests
                .iter()
                .find(|r| v::service_request_identity(r) == producer.request_identity)
                .ok_or(Error("gateway-evidence-reuse-request-missing"))?;
            let consumer_request = bundle
                .service_requests
                .iter()
                .find(|r| v::service_request_identity(r) == edge.consumer_request_identity)
                .ok_or(Error("gateway-evidence-reuse-request-missing"))?;
            if producer_request.concrete_request_hash != consumer_request.concrete_request_hash
                || edge.signature_ref.as_deref() != Some(producer.producer_signature_ref.as_str())
                || consumer_projection.as_ref().and_then(|p| p.get("platform")).and_then(serde_json::Value::as_str)
                    != Some(producer.platform.as_str())
            {
                return Err(Error("gateway-evidence-reuse-fact-conflict"));
            }
            if edge.sharing_policy.is_empty()
                || authority.is_none_or(|a| {
                    a.decision != "allow" || a.operation != "result-read" || a.role != v::BuildServiceRole::Verified
                })
                || consumer_projection
                    .as_ref()
                    .and_then(|p| p.get("request_identity"))
                    .and_then(serde_json::Value::as_str)
                    != Some(edge.consumer_request_identity.as_str())
                || edge.output_check_ref.as_deref().is_none_or(str::is_empty)
                || edge.signature_ref.as_deref().is_none_or(str::is_empty)
                || edge.cas_observation.as_deref().is_none_or(str::is_empty)
            {
                return Err(Error("gateway-evidence-reuse-unauthorized"));
            }
        }
    }
    Ok(report)
}

fn utc(seconds: u64) -> Result<String, Error> {
    // Howard Hinnant's civil-from-days algorithm, bounded to four-digit UTC.
    if seconds > 253_402_300_799 {
        return Err(Error("gateway-evidence-timestamp-range"));
    }
    let z = i64::try_from(seconds / 86_400).map_err(|_| Error("gateway-evidence-timestamp-range"))? + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        seconds / 3600 % 24,
        seconds / 60 % 60,
        seconds % 60
    ))
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Replay {
    schema: String,
    requirements: crunch_build::distributed::RemoteResourceRequirements,
    features: resource::Features,
    family: String,
    classes: Vec<resource::MachineClass>,
    history: Vec<resource::Observation>,
    now_unix_s: u64,
    project_remaining_units: u64,
    account_remaining_units: u64,
    policy: resource::SelectionPolicy,
    switches: [bool; 6],
}
impl Replay {
    fn declared(&self) -> resource::Quantities {
        resource::Quantities {
            cpu_units: self.requirements.quantities.cpu_units,
            memory_bytes: self.requirements.quantities.memory_bytes,
            scratch_bytes: self.requirements.quantities.scratch_bytes,
        }
    }
    fn controls(&self) -> resource::Controls {
        let s = self.switches;
        resource::Controls {
            mode: if s[0] {
                resource::SelectionMode::ProjectOptIn
            } else {
                resource::SelectionMode::ObserveOnly
            },
            project_opted_in: s[1],
            selection_enabled: s[2],
            oom_retry_enabled: s[3],
            quota_enforcement_enabled: s[4],
            sharing_enabled: s[5],
        }
    }
    fn quota(&self) -> resource::QuotaFacts {
        if self.controls().quota_active() {
            resource::QuotaFacts {
                project_remaining_units: self.project_remaining_units,
                account_remaining_units: self.account_remaining_units,
            }
        } else {
            resource::QuotaFacts {
                project_remaining_units: u64::MAX,
                account_remaining_units: u64::MAX,
            }
        }
    }
    fn read(evaluation: &crunch_build::distributed::PolicyEvaluation) -> Result<Self, Error> {
        if evaluation.replay_input_json.len() > MAX_EVIDENCE_BYTES
            || blake3::hash(&evaluation.replay_input_json).to_hex().as_str() != evaluation.replay_input_blake3
        {
            return Err(Error("gateway-evidence-replay-content"));
        }
        let r: Self =
            serde_json::from_slice(&evaluation.replay_input_json).map_err(|_| Error("gateway-evidence-replay-json"))?;
        if r.schema != "mantle-resource-policy-replay-v1"
            || r.classes != evaluation.classes
            || r.classes.len() > MAX_EVIDENCE_RECORDS
            || r.history.len() > MAX_EVIDENCE_RECORDS
        {
            return Err(Error("gateway-evidence-replay-scope"));
        }
        let static_decision = resource::select(resource::SelectionRequest {
            declared: r.declared(),
            features: &r.features,
            family: &r.family,
            classes: &r.classes,
            history: &[],
            now: r.now_unix_s,
            quota: r.quota(),
            policy: r.policy,
        });
        let historical = resource::select(resource::SelectionRequest {
            declared: r.declared(),
            features: &r.features,
            family: &r.family,
            classes: &r.classes,
            history: &r.history,
            now: r.now_unix_s,
            quota: r.quota(),
            policy: r.policy,
        });
        if resource::rollout(r.controls(), static_decision, historical) != evaluation.decision {
            return Err(Error("gateway-evidence-replay-conflict"));
        }
        Ok(r)
    }
}

#[cfg(target_os = "linux")]
impl Export {
    /// The caller opens its configured private evidence directory once. All
    /// writes and duplicate reads are relative to that capability, not paths.
    pub fn publish(&self, directory: &std::fs::File) -> Result<String, Error> {
        use std::os::unix::fs::MetadataExt;

        use durable_file_publication::core::CleanupDisposition;
        use durable_file_publication::core::DurabilityMode;
        use durable_file_publication::core::PublicationDisposition;
        use durable_file_publication::core::PublicationRequest;
        use durable_file_publication::core::ReplacementMode;
        use durable_file_publication::shell::LinuxDirectory;
        use durable_file_publication::shell::StageNameSource;
        use durable_file_publication::shell::publish_one_file;
        let metadata = directory.metadata().map_err(|_| Error("gateway-evidence-directory"))?;
        // SAFETY: geteuid has no preconditions.
        if !metadata.is_dir() || metadata.mode() & 0o077 != 0 || metadata.uid() != unsafe { libc::geteuid() } {
            return Err(Error("gateway-evidence-directory-not-private"));
        }
        verify_bytes(&self.bytes, &self.digest_blake3)?;
        struct Names;
        impl StageNameSource for Names {
            fn next_stage_leaf(&mut self) -> String {
                use rand::RngCore;
                let mut nonce = [0_u8; 32];
                rand::rngs::OsRng.fill_bytes(&mut nonce);
                format!(".evidence-{}.tmp", hex(&nonce))
            }
        }
        let leaf = format!("{}.json", self.digest_blake3);
        let request = PublicationRequest {
            destination_leaf: leaf.clone(),
            payload_bytes: self.bytes.len(),
            maximum_payload_bytes: MAX_EVIDENCE_BYTES,
            collision_attempt_limit: 8,
            final_mode: 0o600,
            replacement: ReplacementMode::NoReplace,
            durability: DurabilityMode::DurabilityRequired,
        };
        let mut io = LinuxDirectory::from_open_directory(
            directory.try_clone().map_err(|_| Error("gateway-evidence-directory"))?,
        )
        .map_err(|_| Error("gateway-evidence-directory"))?;
        let outcome = publish_one_file(&mut io, &mut Names, &request, &self.bytes)
            .map_err(|_| Error("gateway-evidence-publication"))?;
        match outcome.disposition {
            PublicationDisposition::CommittedAndParentSynchronized => Ok(leaf),
            PublicationDisposition::DestinationExists {
                cleanup: CleanupDisposition::Removed,
            } => {
                verify_existing(directory, &leaf, &self.bytes)?;
                directory.sync_all().map_err(|_| Error("gateway-evidence-durability"))?;
                Ok(leaf)
            }
            _ => Err(Error("gateway-evidence-publication-not-durable")),
        }
    }
}

#[cfg(target_os = "linux")]
fn verify_existing(directory: &std::fs::File, leaf: &str, expected: &[u8]) -> Result<(), Error> {
    if read_bytes_at(directory, leaf)? != expected {
        return Err(Error("gateway-evidence-existing-conflict"));
    }
    Ok(())
}

/// Resolve only a generated immutable leaf returned by `Export::publish`.
/// The retained leaf identity, not a caller path or signature, selects bytes.
#[cfg(target_os = "linux")]
pub fn read_publication(directory: &std::fs::File, leaf: &str) -> Result<v::BuildServiceBundle, Error> {
    let digest = leaf.strip_suffix(".json").ok_or(Error("gateway-evidence-name"))?;
    if !crunch_remote_core::output::is_blake3_hex_digest(digest) {
        return Err(Error("gateway-evidence-name"));
    }
    verify_bytes(&read_bytes_at(directory, leaf)?, digest)
}

#[cfg(target_os = "linux")]
fn read_bytes_at(directory: &std::fs::File, leaf: &str) -> Result<Vec<u8>, Error> {
    use std::io::Read;
    use std::os::fd::AsRawFd;
    use std::os::fd::FromRawFd;
    let name = std::ffi::CString::new(leaf).map_err(|_| Error("gateway-evidence-name"))?;
    // SAFETY: both descriptors and the NUL-terminated generated leaf are live.
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
        )
    };
    if fd < 0 {
        return Err(Error("gateway-evidence-existing-unreadable"));
    }
    // SAFETY: openat returned a fresh owned descriptor.
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    if !file.metadata().map_err(|_| Error("gateway-evidence-existing-unreadable"))?.is_file() {
        return Err(Error("gateway-evidence-existing-not-file"));
    }
    let mut bytes = Vec::new();
    file.take(MAX_EVIDENCE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error("gateway-evidence-existing-unreadable"))?;
    if bytes.len() > MAX_EVIDENCE_BYTES {
        return Err(Error("gateway-evidence-byte-limit"));
    }
    Ok(bytes)
}

pub struct RetryFacts<'a> {
    pub predecessor_bundle: &'a v::BuildServiceBundle,
    pub controls: resource::Controls,
    pub context: resource::RetryContext<'a>,
    pub evidence: resource::OomEvidence<'a>,
    pub prior: &'a resource::MachineClass,
    pub classes: &'a [resource::MachineClass],
    pub quota: resource::QuotaFacts,
    pub policy: resource::EscalationPolicy,
    pub plan: &'a resource::EscalationPlan,
}
// Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
// I3).
#[allow(dead_code)]
pub struct ReuseFacts<'a> {
    pub producer_bundle: &'a v::BuildServiceBundle,
    pub producer_result_identity: &'a str,
    pub producer_project: &'a str,
    pub controls: resource::Controls,
    pub scope: Option<resource::SharingScope<'a>>,
    pub checks: resource::SharingChecks,
    pub authorization_identity: &'a str,
    pub output_check_ref: &'a str,
    pub signature_ref: &'a str,
    pub cas_observation: &'a str,
}
fn class_record(class: &resource::MachineClass) -> Result<v::MachineClassRecord, Error> {
    Ok(v::MachineClassRecord {
        class_id: class.id.clone(),
        platform: class.features.platform.clone(),
        resource_class: json(
            &serde_json::json!({"features":class.features,"quantities":class.quantities,"reservation_units":class.reservation_units}),
        )?,
        ordinal: class.order,
        role: v::BuildServiceRole::Linked,
        non_claims: boundary(),
    })
}

fn gateway_phase(value: crate::remote_build::gateway::GatewayPhase) -> crunch_nix_gateway::api::Phase {
    use crunch_nix_gateway::api::Phase as P;

    use crate::remote_build::gateway::GatewayPhase as G;
    match value {
        G::Queued => P::Queued,
        G::Running => P::Running,
        G::Succeeded => P::Succeeded,
        G::Failed => P::Failed,
        G::Cancelled => P::Cancelled,
    }
}

#[cfg(test)]
mod tests;

fn observation_record(
    observation: &resource::Observation,
    attempt_identity: String,
) -> Result<v::ResourceObservationRecord, Error> {
    let mut measurements = vec![measurement(
        "terminal-observed-unix-time",
        observation.terminal_unix_s,
        "s",
    )];
    for (name, value, unit) in [
        ("cpu", observation.cpu_ms, "ms"),
        ("execution", observation.execution_ms, "ms"),
        ("peak-memory", observation.measured.memory_bytes, "bytes"),
        ("scratch-high-water", observation.measured.scratch_bytes, "bytes"),
        ("sampled-scratch-lower-bound", observation.scratch_sampled_peak_bytes, "bytes"),
        ("queue", observation.queue_ms, "ms"),
        ("io", observation.io_bytes, "bytes"),
        ("transfer", observation.transfer_bytes, "bytes"),
        ("executor-output-not-network", observation.executor_output_bytes, "bytes"),
        ("exact-child-peak-rss-not-tree-peak", observation.process_peak_rss_bytes, "bytes"),
    ] {
        if let Some(value) = value {
            measurements.push(measurement(name, value, unit));
        }
    }
    Ok(v::ResourceObservationRecord {
        observation_id: observation.attempt_id.as_str().into(),
        attempt_identity,
        measurements,
        collector: observation.collector_version.clone(),
        compatibility_scope: json(
            &serde_json::json!({"schema":observation.schema,"action_family":observation.action_family,
            "features":observation.features,"machine_class":observation.machine_class,"declared":observation.declared,"selected":observation.selected,
            "policy_version":observation.policy_version,"known_resource_dimensions":observation.measured,
            "trusted_collector":observation.trusted_collector,
            "unknown_measurements":{"cpu":observation.cpu_ms.is_none(),"execution":observation.execution_ms.is_none(),"queue":observation.queue_ms.is_none(),"io":observation.io_bytes.is_none(),"transfer":observation.transfer_bytes.is_none(),
            "cpu_units":observation.measured.cpu_units.is_none(),"peak_memory":observation.measured.memory_bytes.is_none(),"scratch_high_water":observation.measured.scratch_bytes.is_none(),
            "sampled_scratch_lower_bound":observation.scratch_sampled_peak_bytes.is_none(),"executor_output":observation.executor_output_bytes.is_none(),"process_peak_rss":observation.process_peak_rss_bytes.is_none()},
            "terminal_unix_s":observation.terminal_unix_s,"oom":observation.oom,"predecessor":observation.predecessor}),
        )?,
        terminal_outcome: json(&observation.outcome)?,
        role: v::BuildServiceRole::RecordedOnly,
        non_claims: boundary(),
    })
}

fn signed_usage_record(
    usage: &crate::remote_build::usage::RemoteAuthenticatedUsage,
    attempt_identity: String,
) -> Result<v::ResourceObservationRecord, Error> {
    let binding = usage.binding();
    let mut measurements = vec![
        measurement("cpu", usage.cpu_time_millis(), "ms"),
        measurement("wall", usage.wall_time_millis(), "ms"),
        measurement("peak-rss", usage.peak_rss_bytes(), "bytes"),
        measurement("stdout", usage.stdout_bytes(), "bytes"),
        measurement("stderr", usage.stderr_bytes(), "bytes"),
    ];
    if let Some(peak) = usage.cgroup_peak_memory_bytes() {
        measurements.push(measurement("dedicated-cgroup-peak-memory", peak, "bytes"));
    }
    if let Some(peak) = usage.scratch_observation().sampled_peak_bytes {
        measurements.push(measurement("sampled-workspace-allocated-blocks-peak-lower-bound", peak, "bytes"));
    }
    Ok(v::ResourceObservationRecord {
        observation_id: usage.event_digest_blake3(),
        attempt_identity,
        measurements,
        collector: "mantle-exact-child-usage-v1".into(),
        compatibility_scope: json(
            &serde_json::json!({"job":binding.job_id,"attempt":binding.attempt_id,"fence":binding.fence_generation,"request_id":usage.request_id(),
            "result":usage.result_digest_blake3(),"oom":usage.oom_evidence(),"completeness":usage.measurement_completeness(),"scratch":usage.scratch_observation(),
            "memory_accounting":super::resource_host::MEMORY_ACCOUNTING,"output_accounting":super::resource_host::OUTPUT_ACCOUNTING}),
        )?,
        terminal_outcome: "signed-child-completion-observation".into(),
        role: v::BuildServiceRole::RecordedOnly,
        non_claims: boundary(),
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct EscalationReplay {
    schema: String,
    controls: resource::Controls,
    context: EscalationContext,
    evidence: EscalationEvidence,
    prior: resource::MachineClass,
    classes: Vec<resource::MachineClass>,
    quota: EscalationQuota,
    policy: super::resource_host::RetryPolicy,
    plan: resource::EscalationPlan,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct EscalationContext {
    job: crunch_remote_core::attempt::RemoteJobId,
    attempt: crunch_remote_core::attempt::RemoteAttemptId,
    fence: crunch_remote_core::attempt::RemoteFenceGeneration,
    next_attempt: crunch_remote_core::attempt::RemoteAttemptId,
    next_fence: crunch_remote_core::attempt::RemoteFenceGeneration,
    failed: bool,
    retries: u32,
    cumulative_units: u64,
    elapsed_ms: u64,
    next_wall_budget_ms: u64,
}
impl EscalationContext {
    fn borrowed(&self) -> resource::RetryContext<'_> {
        resource::RetryContext {
            job: &self.job,
            attempt: &self.attempt,
            fence: self.fence,
            next_attempt: &self.next_attempt,
            next_fence: self.next_fence,
            failed: self.failed,
            retries: self.retries,
            cumulative_units: self.cumulative_units,
            elapsed_ms: self.elapsed_ms,
            next_wall_budget_ms: self.next_wall_budget_ms,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct EscalationEvidence {
    job: crunch_remote_core::attempt::RemoteJobId,
    attempt: crunch_remote_core::attempt::RemoteAttemptId,
    fence: crunch_remote_core::attempt::RemoteFenceGeneration,
    platform: String,
    category: resource::OomCategory,
    trusted_positive_platform_evidence: bool,
}
impl EscalationEvidence {
    fn borrowed(&self) -> resource::OomEvidence<'_> {
        resource::OomEvidence {
            job: &self.job,
            attempt: &self.attempt,
            fence: self.fence,
            platform: &self.platform,
            category: self.category,
            trusted_positive_platform_evidence: self.trusted_positive_platform_evidence,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct EscalationQuota {
    project_remaining_units: u64,
    account_remaining_units: u64,
}
impl EscalationQuota {
    fn borrowed(&self) -> resource::QuotaFacts {
        resource::QuotaFacts {
            project_remaining_units: self.project_remaining_units,
            account_remaining_units: self.account_remaining_units,
        }
    }
}
