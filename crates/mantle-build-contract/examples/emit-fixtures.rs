use mantle_build_contract::*;
use serde_json::json;

const IDENTITY_HEX_LENGTH: usize = 64;

fn identity(character: char) -> Identity {
    Identity::new(format!("b3:{}", character.to_string().repeat(IDENTITY_HEX_LENGTH)))
        .unwrap_or_else(|_| unreachable!())
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

fn success(request: &BuildRequest, cache: CacheObservation) -> BuildObservation {
    let mut observation = BuildObservation {
        schema: OBSERVATION_SCHEMA.to_string(),
        identity: identity('0'),
        request_identity: request.identity.clone(),
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
        cache,
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

fn failure(request: &BuildRequest) -> BuildObservation {
    let mut observation = success(request, CacheObservation {
        kind: CacheKind::None,
        source_identity: None,
    });
    observation.outcome = BuildOutcome::Failed;
    observation.products.clear();
    observation.identity = observation_identity(&observation);
    observation
}

fn main() {
    let request = request();
    let direct_success = success(&request, CacheObservation {
        kind: CacheKind::None,
        source_identity: None,
    });
    let cache_success = success(&request, CacheObservation {
        kind: CacheKind::Substitution,
        source_identity: Some(identity('2')),
    });
    let failure = failure(&request);
    let bundle = json!({
        "schema": "mantle.build-producer-fixtures.v1",
        "request": request,
        "success": direct_success,
        "cache_success": cache_success,
        "failure": failure,
    });
    println!("{}", serde_json::to_string_pretty(&bundle).unwrap_or_else(|_| unreachable!()));
}
