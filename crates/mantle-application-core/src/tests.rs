use alloc::string::String;
use alloc::string::ToString;

const REQUEST_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const WRONG_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn command() -> crate::ApplicationCommand {
    crate::ApplicationCommand {
        schema: crate::APPLICATION_COMMAND_SCHEMA.to_string(),
        family: crate::CommandFamily::Build,
        operation: " build.plan ".to_string(),
        request_blake3: REQUEST_DIGEST.to_string(),
        mutation: crate::MutationClass::LocalMutation,
    }
}

#[test]
fn dispatch_plan_is_deterministic_and_normalized() {
    let first = crate::plan_dispatch(command()).unwrap();
    let second = crate::plan_dispatch(command()).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.command.operation, "build.plan");
    assert_eq!(first.effect_id_blake3.len(), crate::BLAKE3_HEX_CHARS);
    assert!(first.non_claim.contains("does-not-prove-operation-effect-success"));
}

#[test]
fn invalid_schema_operation_and_identity_fail_closed() {
    let mut invalid_schema = command();
    invalid_schema.schema = "wrong".to_string();
    let mut invalid_operation = command();
    invalid_operation.operation = " ".to_string();
    let mut invalid_identity = command();
    invalid_identity.request_blake3 = WRONG_DIGEST[..crate::BLAKE3_HEX_CHARS - 1].to_string();
    assert_eq!(crate::plan_dispatch(invalid_schema), Err(crate::ApplicationCoreError::InvalidCommand("schema")));
    assert_eq!(
        crate::plan_dispatch(invalid_operation),
        Err(crate::ApplicationCoreError::InvalidCommand("operation"))
    );
    assert_eq!(
        crate::plan_dispatch(invalid_identity),
        Err(crate::ApplicationCoreError::InvalidCommand("request-identity"))
    );
}

#[test]
fn success_requires_matching_effect_and_family() {
    let effect = crate::plan_dispatch(command()).unwrap();
    let outcome = crate::classify_observation(&effect, crate::successful_observation(&effect)).unwrap();
    let mut wrong_effect = crate::successful_observation(&effect);
    wrong_effect.effect_id_blake3 = WRONG_DIGEST.to_string();
    let mut wrong_family = crate::successful_observation(&effect);
    wrong_family.family = crate::CommandFamily::Store;
    assert_eq!(outcome.status, crate::ApplicationObservationStatus::Succeeded);
    assert!(outcome.failure.is_none());
    assert_eq!(
        crate::classify_observation(&effect, wrong_effect),
        Err(crate::ApplicationCoreError::ObservationMismatch("effect-identity"))
    );
    assert_eq!(
        crate::classify_observation(&effect, wrong_family),
        Err(crate::ApplicationCoreError::ObservationMismatch("command-family"))
    );
}

#[test]
fn typed_failure_survives_until_outcome() {
    let effect = crate::plan_dispatch(command()).unwrap();
    let failure = crate::CapabilityFailure {
        family: effect.command.family,
        effect_id_blake3: effect.effect_id_blake3.clone(),
        class: crate::ApplicationErrorClass::Build,
        code: "build-failed".to_string(),
        message: "exact adapter detail".to_string(),
        reported_exit_code: None,
    };
    let observation = crate::failed_observation(&effect, failure.clone()).unwrap();
    let outcome = crate::classify_observation(&effect, observation).unwrap();
    assert_eq!(outcome.failure, Some(failure));
    assert_eq!(outcome.status, crate::ApplicationObservationStatus::Failed);
}

#[test]
fn reported_and_oversized_failures_are_validated() {
    let effect = crate::plan_dispatch(command()).unwrap();
    let missing_code = crate::CapabilityFailure {
        family: effect.command.family,
        effect_id_blake3: effect.effect_id_blake3.clone(),
        class: crate::ApplicationErrorClass::Reported,
        code: "reported".to_string(),
        message: String::new(),
        reported_exit_code: None,
    };
    let oversized = crate::CapabilityFailure {
        family: effect.command.family,
        effect_id_blake3: effect.effect_id_blake3.clone(),
        class: crate::ApplicationErrorClass::Internal,
        code: "internal".to_string(),
        message: "x".repeat(crate::FAILURE_MESSAGE_BYTES_MAX as usize + 1),
        reported_exit_code: None,
    };
    assert_eq!(
        crate::failed_observation(&effect, missing_code),
        Err(crate::ApplicationCoreError::InvalidFailure("reported-exit-code-missing"))
    );
    assert_eq!(
        crate::failed_observation(&effect, oversized),
        Err(crate::ApplicationCoreError::InvalidFailure("message"))
    );
}
