#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema: std::string::String,
    request: mantle_build_contract::BuildRequest,
    success: mantle_build_contract::BuildObservation,
    cache_success: mantle_build_contract::BuildObservation,
    failure: mantle_build_contract::BuildObservation,
}

fn fixture() -> Result<Fixture, serde_json::Error> {
    let value: Fixture = serde_json::from_str(include_str!("../producer/fixtures-v1.json"))?;
    assert_eq!(value.schema, "mantle.build-producer-fixtures.v1");
    assert_eq!(value.request.schema, mantle_build_contract::REQUEST_SCHEMA);
    Ok(value)
}

// r[verify mantle.build_interchange.hash_matrix]
#[test]
fn original_request_and_all_observation_identities_remain_valid() -> Result<(), serde_json::Error> {
    let value = fixture()?;
    assert_eq!(mantle_build_contract::request_identity(&value.request), value.request.identity);
    assert_eq!(mantle_build_contract::validate_request(&value.request), Ok(()));
    for observed in [&value.success, &value.cache_success, &value.failure] {
        assert_eq!(mantle_build_contract::observation_identity(observed), observed.identity);
        assert_eq!(
            mantle_build_contract::validate_observation(&value.request, &value.success.builder_identity, observed),
            Ok(())
        );
    }
    Ok(())
}

#[test]
fn blake3_known_answers_remain_exact() {
    assert_eq!(
        blake3::hash(b"").to_hex().as_str(),
        "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
    );
    assert_eq!(
        blake3::hash(b"abc").to_hex().as_str(),
        "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85"
    );
}

// r[verify mantle.build_interchange.hash_matrix]
#[test]
fn changed_request_and_observation_bytes_still_reject_stale_identities() -> Result<(), serde_json::Error> {
    let mut value = fixture()?;
    value.request.candidate_identity = value.request.effect_identity.clone();
    assert_eq!(
        mantle_build_contract::validate_request(&value.request),
        Err(mantle_build_contract::ContractError::RequestIdentityMismatch)
    );
    let mut value = fixture()?;
    value.success.worker_identity = value.success.builder_identity.clone();
    assert_eq!(
        mantle_build_contract::validate_observation(&value.request, &value.success.builder_identity, &value.success),
        Err(mantle_build_contract::ContractError::ObservationIdentityMismatch)
    );
    Ok(())
}

#[test]
fn recomputed_builder_substitution_remains_denied() -> Result<(), serde_json::Error> {
    let mut value = fixture()?;
    let expected = value.success.builder_identity.clone();
    value.success.builder_identity = value.success.worker_identity.clone();
    assert_ne!(expected, value.success.builder_identity);
    value.success.identity = mantle_build_contract::observation_identity(&value.success);
    assert_eq!(
        mantle_build_contract::validate_observation(&value.request, &expected, &value.success),
        Err(mantle_build_contract::ContractError::BuilderMismatch)
    );
    Ok(())
}

#[test]
fn recomputed_cache_success_cannot_hide_a_missing_product() -> Result<(), serde_json::Error> {
    let mut value = fixture()?;
    assert!(value.cache_success.products.pop().is_some());
    value.cache_success.identity = mantle_build_contract::observation_identity(&value.cache_success);
    assert_eq!(
        mantle_build_contract::validate_observation(
            &value.request,
            &value.success.builder_identity,
            &value.cache_success
        ),
        Err(mantle_build_contract::ContractError::ProductMismatch)
    );
    Ok(())
}

#[test]
fn recomputed_cache_and_unknown_receipt_contradictions_remain_denied() -> Result<(), serde_json::Error> {
    let mut value = fixture()?;
    value.cache_success.cache.source_identity = None;
    value.cache_success.identity = mantle_build_contract::observation_identity(&value.cache_success);
    assert_eq!(
        mantle_build_contract::validate_observation(
            &value.request,
            &value.success.builder_identity,
            &value.cache_success
        ),
        Err(mantle_build_contract::ContractError::CacheContradiction)
    );
    value.success.outcome = mantle_build_contract::BuildOutcome::Unknown;
    value.success.identity = mantle_build_contract::observation_identity(&value.success);
    assert_eq!(
        mantle_build_contract::validate_observation(&value.request, &value.success.builder_identity, &value.success),
        Err(mantle_build_contract::ContractError::OutcomeContradiction)
    );
    Ok(())
}

#[test]
fn generic_exit_and_missing_nonclaims_remain_rejected() -> Result<(), serde_json::Error> {
    assert!(serde_json::from_str::<mantle_build_contract::BuildObservation>(r#"{"exit_code":0}"#).is_err());
    let mut value = fixture()?;
    value.success.non_claims.clear();
    value.success.identity = mantle_build_contract::observation_identity(&value.success);
    assert_eq!(
        mantle_build_contract::validate_observation(&value.request, &value.success.builder_identity, &value.success),
        Err(mantle_build_contract::ContractError::NonClaimsMismatch)
    );
    Ok(())
}
