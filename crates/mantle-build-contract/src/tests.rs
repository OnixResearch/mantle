use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::*;

fn identity(character: char) -> Identity {
    Identity::new(format!("b3:{}", character.to_string().repeat(BLAKE3_HEX_LENGTH))).unwrap_or_else(|_| unreachable!())
}

fn label(value: &str) -> Label {
    Label::new(value.to_string()).unwrap_or_else(|_| unreachable!())
}

fn request() -> BuildRequest {
    let mut request = BuildRequest {
        schema: REQUEST_SCHEMA.to_string(),
        identity: identity('0'),
        idempotency_key: identity('1'),
        effect_identity: identity('2'),
        attempt_identity: identity('3'),
        candidate_identity: identity('4'),
        pipeline_revision_identity: identity('5'),
        plan_identity: identity('6'),
        policy_identity: identity('7'),
        platform: label("x86_64-linux"),
        required_products: vec![label("binary"), label("metadata")],
    };
    request.identity = request_identity(&request);
    request
}

fn observation() -> BuildObservation {
    let request = request();
    let mut observation = BuildObservation {
        schema: OBSERVATION_SCHEMA.to_string(),
        identity: identity('0'),
        request_identity: request.identity,
        outcome: BuildOutcome::Success,
        products: vec![
            ProductObservation {
                name: label("binary"),
                artifact_identity: identity('8'),
                store_identity: identity('9'),
            },
            ProductObservation {
                name: label("metadata"),
                artifact_identity: identity('a'),
                store_identity: identity('b'),
            },
        ],
        builder_identity: identity('c'),
        worker_identity: identity('d'),
        store_identity: identity('e'),
        cache: CacheObservation {
            kind: CacheKind::None,
            source_identity: None,
        },
        logs: vec![identity('f')],
        metrics: vec![MetricObservation {
            name: label("duration-ms"),
            value: 1,
        }],
        receipt_identity: Some(identity('1')),
        non_claims: REQUIRED_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    };
    observation.identity = observation_identity(&observation);
    observation
}

// r[verify mantle.build_interchange.contract.scenario.pinned]
// r[verify mantle.build_interchange.request.scenario.valid]
// r[verify mantle.build_interchange.observation.scenario.success]
// r[verify mantle.build_interchange.products.scenario.match]
#[test]
fn exact_request_and_success_are_admitted() {
    let request = request();
    let observation = observation();
    assert_eq!(validate_request(&request), Ok(()));
    assert_eq!(validate_observation(&request, &identity('c'), &observation), Ok(()));
}

// r[verify mantle.build_interchange.contract.scenario.schema]
// r[verify mantle.build_interchange.request.scenario.substitution]
#[test]
fn schema_and_candidate_drift_are_rejected() {
    let mut schema = request();
    schema.schema = "mantle.build-request.v2".to_string();
    schema.identity = request_identity(&schema);
    assert_eq!(validate_request(&schema), Err(ContractError::UnsupportedSchema));

    let mut candidate = request();
    candidate.candidate_identity = identity('f');
    assert_eq!(validate_request(&candidate), Err(ContractError::RequestIdentityMismatch));
}

// r[verify mantle.build_interchange.products.scenario.cache]
#[test]
fn cache_use_cannot_hide_a_missing_product() {
    let request = request();
    let mut observation = observation();
    observation.products.pop();
    observation.cache = CacheObservation {
        kind: CacheKind::Substitution,
        source_identity: Some(identity('f')),
    };
    observation.identity = observation_identity(&observation);
    assert_eq!(validate_observation(&request, &identity('c'), &observation), Err(ContractError::ProductMismatch));
}

// r[verify mantle.build_interchange.observation.scenario.builder]
#[test]
fn builder_and_request_substitution_are_rejected() {
    let request = request();
    let mut builder_drift = observation();
    builder_drift.builder_identity = identity('f');
    builder_drift.identity = observation_identity(&builder_drift);
    assert_eq!(validate_observation(&request, &identity('c'), &builder_drift), Err(ContractError::BuilderMismatch));

    let mut request_drift = observation();
    request_drift.request_identity = identity('f');
    request_drift.identity = observation_identity(&request_drift);
    assert_eq!(validate_observation(&request, &identity('c'), &request_drift), Err(ContractError::RequestMismatch));
}

#[test]
fn unknown_and_cache_shapes_fail_closed() {
    let request = request();
    let mut unknown = observation();
    unknown.outcome = BuildOutcome::Unknown;
    unknown.receipt_identity = None;
    unknown.products.clear();
    unknown.identity = observation_identity(&unknown);
    assert_eq!(validate_observation(&request, &identity('c'), &unknown), Ok(()));

    let mut contradictory = observation();
    contradictory.cache.kind = CacheKind::Local;
    contradictory.identity = observation_identity(&contradictory);
    assert_eq!(
        validate_observation(&request, &identity('c'), &contradictory),
        Err(ContractError::CacheContradiction)
    );
}

// r[verify mantle.build_interchange.boundary.scenario.scope]
// r[verify mantle.build_interchange.conformance.scenario.drift]
#[test]
fn serialized_values_keep_exact_required_non_claims() {
    let encoded = serde_json::to_value(observation()).unwrap_or_else(|_| unreachable!());
    assert_eq!(
        encoded.get("non_claims").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(REQUIRED_NON_CLAIMS.len())
    );
}
