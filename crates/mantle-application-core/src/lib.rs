#![no_std]
//! Typed command dispatch effects and observations for Mantle applications.

// r[impl application_architecture.thin_composition_root]
// r[impl application_architecture.typed_error_ownership]
// r[impl application_architecture.effect_observation_boundary]
extern crate alloc;

#[cfg(test)]
extern crate std;

use alloc::string::String;
use alloc::string::ToString;

use serde::Deserialize;
use serde::Serialize;

pub const APPLICATION_COMMAND_SCHEMA: &str = "mantle-application-command-v1";
pub const APPLICATION_EFFECT_SCHEMA: &str = "mantle-application-effect-v1";
pub const APPLICATION_NON_CLAIM: &str =
    "dispatch-does-not-prove-operation-effect-success-provider-correctness-deployment-or-release-eligibility";
pub const BLAKE3_HEX_CHARS: usize = 64;
pub const OPERATION_BYTES_MAX: u32 = 256;
pub const FAILURE_CODE_BYTES_MAX: u32 = 128;
pub const FAILURE_MESSAGE_BYTES_MAX: u32 = 16_384;
pub const OBSERVATION_COUNT_MAX: u32 = 1;
const COMMAND_DOMAIN: &[u8] = b"mantle.application.command.v1";
const EFFECT_DOMAIN: &[u8] = b"mantle.application.effect.v1";
const DOMAIN_SEPARATOR: u8 = 0;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum CommandFamily {
    Build,
    RustPlan,
    Remote,
    Store,
    Source,
    Release,
    Project,
    Bootstrap,
    Artifact,
    Evaluation,
    Package,
    Utility,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum MutationClass {
    ReadOnly,
    LocalMutation,
    ExternalEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ApplicationCommand {
    pub schema: String,
    pub family: CommandFamily,
    pub operation: String,
    pub request_blake3: String,
    pub mutation: MutationClass,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ApplicationEffectLimits {
    pub observations_max: u32,
    pub failure_code_bytes_max: u32,
    pub failure_message_bytes_max: u32,
}

impl Default for ApplicationEffectLimits {
    fn default() -> Self {
        Self {
            observations_max: OBSERVATION_COUNT_MAX,
            failure_code_bytes_max: FAILURE_CODE_BYTES_MAX,
            failure_message_bytes_max: FAILURE_MESSAGE_BYTES_MAX,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ApplicationEffect {
    pub schema: String,
    pub effect_id_blake3: String,
    pub command_blake3: String,
    pub command: ApplicationCommand,
    pub limits: ApplicationEffectLimits,
    pub non_claim: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ApplicationErrorClass {
    Evaluation,
    Build,
    Internal,
    Reported,
    Capability,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CapabilityFailure {
    pub family: CommandFamily,
    pub effect_id_blake3: String,
    pub class: ApplicationErrorClass,
    pub code: String,
    pub message: String,
    pub reported_exit_code: Option<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ApplicationObservationStatus {
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ApplicationObservation {
    pub effect_id_blake3: String,
    pub family: CommandFamily,
    pub status: ApplicationObservationStatus,
    pub failure: Option<CapabilityFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ApplicationOutcome {
    pub family: CommandFamily,
    pub operation: String,
    pub status: ApplicationObservationStatus,
    pub effect_id_blake3: String,
    pub failure: Option<CapabilityFailure>,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationCoreError {
    InvalidCommand(&'static str),
    InvalidFailure(&'static str),
    ObservationMismatch(&'static str),
    Serialization,
}

pub fn plan_dispatch(mut command: ApplicationCommand) -> Result<ApplicationEffect, ApplicationCoreError> {
    normalize_command(&mut command);
    validate_command(&command)?;
    let command_blake3 = canonical_digest(COMMAND_DOMAIN, &command)?;
    let effect_constraints = ApplicationEffectLimits::default();
    let effect_preimage = (&command_blake3, &command, effect_constraints);
    let effect_id_blake3 = canonical_digest(EFFECT_DOMAIN, &effect_preimage)?;
    debug_assert_eq!(command_blake3.len(), BLAKE3_HEX_CHARS);
    debug_assert_eq!(effect_id_blake3.len(), BLAKE3_HEX_CHARS);
    Ok(ApplicationEffect {
        schema: APPLICATION_EFFECT_SCHEMA.to_string(),
        effect_id_blake3,
        command_blake3,
        command,
        limits: effect_constraints,
        non_claim: APPLICATION_NON_CLAIM.to_string(),
    })
}

#[must_use]
pub fn successful_observation(effect: &ApplicationEffect) -> ApplicationObservation {
    debug_assert_eq!(effect.schema, APPLICATION_EFFECT_SCHEMA);
    debug_assert_eq!(effect.effect_id_blake3.len(), BLAKE3_HEX_CHARS);
    ApplicationObservation {
        effect_id_blake3: effect.effect_id_blake3.clone(),
        family: effect.command.family,
        status: ApplicationObservationStatus::Succeeded,
        failure: None,
    }
}

pub fn failed_observation(
    effect: &ApplicationEffect,
    failure: CapabilityFailure,
) -> Result<ApplicationObservation, ApplicationCoreError> {
    validate_failure(effect, &failure)?;
    debug_assert_eq!(failure.family, effect.command.family);
    debug_assert_eq!(failure.effect_id_blake3, effect.effect_id_blake3);
    Ok(ApplicationObservation {
        effect_id_blake3: effect.effect_id_blake3.clone(),
        family: effect.command.family,
        status: ApplicationObservationStatus::Failed,
        failure: Some(failure),
    })
}

pub fn classify_observation(
    effect: &ApplicationEffect,
    observation: ApplicationObservation,
) -> Result<ApplicationOutcome, ApplicationCoreError> {
    if observation.effect_id_blake3 != effect.effect_id_blake3 {
        return Err(ApplicationCoreError::ObservationMismatch("effect-identity"));
    }
    if observation.family != effect.command.family {
        return Err(ApplicationCoreError::ObservationMismatch("command-family"));
    }
    validate_observation_shape(effect, &observation)?;
    debug_assert_eq!(observation.effect_id_blake3, effect.effect_id_blake3);
    debug_assert_eq!(observation.family, effect.command.family);
    Ok(ApplicationOutcome {
        family: effect.command.family,
        operation: effect.command.operation.clone(),
        status: observation.status,
        effect_id_blake3: effect.effect_id_blake3.clone(),
        failure: observation.failure,
        non_claim: APPLICATION_NON_CLAIM.to_string(),
    })
}

fn normalize_command(command: &mut ApplicationCommand) {
    let operation_bytes_before = command.operation.len();
    command.operation = command.operation.trim().to_string();
    debug_assert!(command.operation.len() <= operation_bytes_before);
    debug_assert_eq!(command.operation, command.operation.trim());
}

fn validate_command(command: &ApplicationCommand) -> Result<(), ApplicationCoreError> {
    if command.schema != APPLICATION_COMMAND_SCHEMA {
        return Err(ApplicationCoreError::InvalidCommand("schema"));
    }
    let operation_bytes =
        u32::try_from(command.operation.len()).map_err(|_| ApplicationCoreError::InvalidCommand("operation-width"))?;
    if operation_bytes == 0 || operation_bytes > OPERATION_BYTES_MAX {
        return Err(ApplicationCoreError::InvalidCommand("operation"));
    }
    if !is_blake3(&command.request_blake3) {
        return Err(ApplicationCoreError::InvalidCommand("request-identity"));
    }
    Ok(())
}

fn validate_observation_shape(
    effect: &ApplicationEffect,
    observation: &ApplicationObservation,
) -> Result<(), ApplicationCoreError> {
    match observation.status {
        ApplicationObservationStatus::Succeeded if observation.failure.is_none() => Ok(()),
        ApplicationObservationStatus::Failed => {
            let failure = observation.failure.as_ref().ok_or(ApplicationCoreError::InvalidFailure("missing"))?;
            validate_failure(effect, failure)
        }
        ApplicationObservationStatus::Succeeded => Err(ApplicationCoreError::InvalidFailure("success-with-failure")),
    }
}

fn validate_failure(effect: &ApplicationEffect, failure: &CapabilityFailure) -> Result<(), ApplicationCoreError> {
    if failure.family != effect.command.family || failure.effect_id_blake3 != effect.effect_id_blake3 {
        return Err(ApplicationCoreError::ObservationMismatch("failure-binding"));
    }
    let code_bytes =
        u32::try_from(failure.code.len()).map_err(|_| ApplicationCoreError::InvalidFailure("code-width"))?;
    let message_bytes =
        u32::try_from(failure.message.len()).map_err(|_| ApplicationCoreError::InvalidFailure("message-width"))?;
    if code_bytes == 0 || code_bytes > effect.limits.failure_code_bytes_max {
        return Err(ApplicationCoreError::InvalidFailure("code"));
    }
    if message_bytes > effect.limits.failure_message_bytes_max {
        return Err(ApplicationCoreError::InvalidFailure("message"));
    }
    validate_reported_failure(failure)
}

fn validate_reported_failure(failure: &CapabilityFailure) -> Result<(), ApplicationCoreError> {
    if failure.class == ApplicationErrorClass::Reported && failure.reported_exit_code.is_none() {
        return Err(ApplicationCoreError::InvalidFailure("reported-exit-code-missing"));
    }
    if failure.class != ApplicationErrorClass::Reported && failure.reported_exit_code.is_some() {
        return Err(ApplicationCoreError::InvalidFailure("reported-exit-code-unexpected"));
    }
    Ok(())
}

fn canonical_digest<T: Serialize>(domain: &[u8], value: &T) -> Result<String, ApplicationCoreError> {
    let bytes = serde_json::to_vec(value).map_err(|_| ApplicationCoreError::Serialization)?;
    let length_bytes = u64::try_from(bytes.len()).map_err(|_| ApplicationCoreError::Serialization)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&u64::try_from(domain.len()).map_err(|_| ApplicationCoreError::Serialization)?.to_le_bytes());
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(domain);
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(&bytes);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    debug_assert!(!domain.is_empty());
    Ok(digest)
}

fn is_blake3(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests;
