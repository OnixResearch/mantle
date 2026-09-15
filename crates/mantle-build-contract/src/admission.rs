#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the closed interchange admission module keeps contract owner names together"
)]

use alloc::string::ToString;

use crate::BuildObservation;
use crate::BuildOutcome;
use crate::BuildRequest;
use crate::CacheKind;
use crate::ContractError;
use crate::Identity;
use crate::MAXIMUM_LOGS;
use crate::MAXIMUM_METRICS;
use crate::MAXIMUM_PRODUCTS;
use crate::MAXIMUM_REQUIRED_PRODUCTS;
use crate::OBSERVATION_SCHEMA;
use crate::REQUEST_SCHEMA;
use crate::REQUIRED_NON_CLAIMS;
use crate::observation_identity;
use crate::request_identity;

pub fn validate_request(request: &BuildRequest) -> Result<(), ContractError> {
    // r[impl mantle.build_interchange.request]
    if request.schema != REQUEST_SCHEMA {
        return Err(ContractError::UnsupportedSchema);
    }
    if request.required_products.len() > MAXIMUM_REQUIRED_PRODUCTS {
        return Err(ContractError::CollectionLimitExceeded);
    }
    if !strictly_sorted(request.required_products.iter().map(|value| value.as_str())) {
        return Err(ContractError::RequiredProductsNotCanonical);
    }
    if request.identity != request_identity(request) {
        return Err(ContractError::RequestIdentityMismatch);
    }
    Ok(())
}

#[cfg_attr(feature = "tigerstyle", allow(tigerstyle::assertion_density))] // pre-existing validation logic; dedicated assertions tracked for the hardening pass
pub fn validate_observation(
    request: &BuildRequest,
    expected_builder: &Identity,
    observation: &BuildObservation,
) -> Result<(), ContractError> {
    // r[impl mantle.build_interchange.observation]
    // r[impl mantle.build_interchange.products]
    validate_request(request)?;
    if observation.schema != OBSERVATION_SCHEMA {
        return Err(ContractError::UnsupportedSchema);
    }
    if observation.request_identity != request.identity {
        return Err(ContractError::RequestMismatch);
    }
    if &observation.builder_identity != expected_builder {
        return Err(ContractError::BuilderMismatch);
    }
    if observation.products.len() > MAXIMUM_PRODUCTS
        || observation.logs.len() > MAXIMUM_LOGS
        || observation.metrics.len() > MAXIMUM_METRICS
    {
        return Err(ContractError::CollectionLimitExceeded);
    }
    if !strictly_sorted(observation.products.iter().map(|product| product.name.as_str())) {
        return Err(ContractError::ProductMismatch);
    }
    if observation.outcome == BuildOutcome::Success && !same_products(request, observation) {
        return Err(ContractError::ProductMismatch);
    }
    match observation.outcome {
        BuildOutcome::Success | BuildOutcome::Failed | BuildOutcome::Cancelled | BuildOutcome::TimedOut
            if observation.receipt_identity.is_some() => {}
        BuildOutcome::Unknown if observation.receipt_identity.is_none() => {}
        BuildOutcome::Success
        | BuildOutcome::Failed
        | BuildOutcome::Cancelled
        | BuildOutcome::TimedOut
        | BuildOutcome::Unknown => return Err(ContractError::OutcomeContradiction),
    }
    match observation.cache.kind {
        CacheKind::None if observation.cache.source_identity.is_none() => {}
        CacheKind::Local | CacheKind::Substitution if observation.cache.source_identity.is_some() => {}
        CacheKind::None | CacheKind::Local | CacheKind::Substitution => {
            return Err(ContractError::CacheContradiction);
        }
    }
    let expected_non_claims =
        REQUIRED_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect::<alloc::vec::Vec<_>>();
    if observation.non_claims != expected_non_claims {
        return Err(ContractError::NonClaimsMismatch);
    }
    if observation.identity != observation_identity(observation) {
        return Err(ContractError::ObservationIdentityMismatch);
    }
    Ok(())
}

fn same_products(request: &BuildRequest, observation: &BuildObservation) -> bool {
    request.required_products.len() == observation.products.len()
        && request
            .required_products
            .iter()
            .zip(&observation.products)
            .all(|(required, observed)| required == &observed.name)
}

fn strictly_sorted<'a>(values: impl IntoIterator<Item = &'a str>) -> bool {
    let mut previous = None;
    for value in values {
        if previous.is_some_and(|previous| previous >= value) {
            return false;
        }
        previous = Some(value);
    }
    true
}
