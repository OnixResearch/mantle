#![allow(
    unknown_lints,
    non_trait_imports,
    path_segment_repetition,
    reason = "stable identity frame labels intentionally repeat contract role names"
)]

use crate::BuildObservation;
use crate::BuildRequest;
use crate::Identity;

const REQUEST_DOMAIN: &str = "mantle.build-request.identity.v1";
const OBSERVATION_DOMAIN: &str = "mantle.build-observation.identity.v1";

struct Frame(blake3::Hasher);

impl Frame {
    fn new(domain: &str) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(domain.as_bytes());
        Self(hasher)
    }

    fn bytes(&mut self, label: &str, value: &[u8]) {
        self.part(label.as_bytes());
        self.part(value);
    }

    fn text(&mut self, label: &str, value: &str) {
        self.bytes(label, value.as_bytes());
    }

    fn count(&mut self, label: &str, count: usize) {
        match u64::try_from(count) {
            Ok(count) => self.bytes(label, &count.to_le_bytes()),
            Err(_) => self.text(label, "unrepresentable-count"),
        }
    }

    fn part(&mut self, value: &[u8]) {
        match u64::try_from(value.len()) {
            Ok(length) => {
                self.0.update(&length.to_le_bytes());
                self.0.update(value);
            }
            Err(_) => {
                self.0.update(b"unrepresentable-length");
            }
        }
    }

    fn finish(self) -> Identity {
        Identity::from_digest(self.0.finalize())
    }
}

#[must_use]
pub fn request_identity(request: &BuildRequest) -> Identity {
    let mut frame = Frame::new(REQUEST_DOMAIN);
    frame.text("schema", &request.schema);
    frame.text("idempotency", request.idempotency_key.as_str());
    frame.text("effect", request.effect_identity.as_str());
    frame.text("attempt", request.attempt_identity.as_str());
    frame.text("candidate", request.candidate_identity.as_str());
    frame.text("pipeline", request.pipeline_revision_identity.as_str());
    frame.text("plan", request.plan_identity.as_str());
    frame.text("policy", request.policy_identity.as_str());
    frame.text("platform", request.platform.as_str());
    frame.count("required_product_count", request.required_products.len());
    for product in &request.required_products {
        frame.text("required_product", product.as_str());
    }
    frame.finish()
}

#[must_use]
pub fn observation_identity(observation: &BuildObservation) -> Identity {
    let mut frame = Frame::new(OBSERVATION_DOMAIN);
    frame.text("schema", &observation.schema);
    frame.text("request", observation.request_identity.as_str());
    frame.text("outcome", observation.outcome.as_str());
    hash_observation_products(&mut frame, observation);
    frame.text("builder", observation.builder_identity.as_str());
    frame.text("worker", observation.worker_identity.as_str());
    frame.text("store", observation.store_identity.as_str());
    frame.text("cache_kind", observation.cache.kind.as_str());
    if let Some(source) = &observation.cache.source_identity {
        frame.text("cache_source", source.as_str());
    }
    hash_observation_details(&mut frame, observation);
    frame.finish()
}

fn hash_observation_products(frame: &mut Frame, observation: &BuildObservation) {
    frame.count("product_count", observation.products.len());
    for product in &observation.products {
        frame.text("product_name", product.name.as_str());
        frame.text("product_artifact", product.artifact_identity.as_str());
        frame.text("product_store", product.store_identity.as_str());
    }
}

fn hash_observation_details(frame: &mut Frame, observation: &BuildObservation) {
    frame.count("log_count", observation.logs.len());
    for log in &observation.logs {
        frame.text("log", log.as_str());
    }
    frame.count("metric_count", observation.metrics.len());
    for metric in &observation.metrics {
        frame.text("metric_name", metric.name.as_str());
        frame.bytes("metric_value", &metric.value.to_le_bytes());
    }
    if let Some(receipt) = &observation.receipt_identity {
        frame.text("receipt", receipt.as_str());
    }
    frame.count("non_claim_count", observation.non_claims.len());
    for non_claim in &observation.non_claims {
        frame.text("non_claim", non_claim);
    }
}
