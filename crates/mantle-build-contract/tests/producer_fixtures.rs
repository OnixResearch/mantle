use mantle_build_contract::BuildObservation;
use mantle_build_contract::BuildRequest;
use mantle_build_contract::Identity;
use mantle_build_contract::validate_observation;
use mantle_build_contract::validate_request;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProducerFixtures {
    schema: String,
    request: BuildRequest,
    success: BuildObservation,
    cache_success: BuildObservation,
    failure: BuildObservation,
}

#[test]
fn producer_fixtures_decode_and_admit() -> Result<(), serde_json::Error> {
    // r[verify mantle.build_interchange.conformance]
    // r[verify mantle.build_interchange.conformance.scenario.drift]
    let fixtures: ProducerFixtures =
        serde_json::from_str(include_str!("../../../fixtures/mantle-build-contract/producer/fixtures-v1.json"))?;
    assert_eq!(fixtures.schema, "mantle.build-producer-fixtures.v1");
    assert_eq!(validate_request(&fixtures.request), Ok(()));
    let builder = fixtures.success.builder_identity.clone();
    for observation in [&fixtures.success, &fixtures.cache_success, &fixtures.failure] {
        assert_eq!(validate_observation(&fixtures.request, &builder, observation), Ok(()));
    }
    assert!(Identity::new("invalid".to_string()).is_err());
    Ok(())
}
