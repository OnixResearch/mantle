use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Serialize;

use crate::CacheDecision;
use crate::CacheObservation;
use crate::CacheObservationStatus;
use crate::ObservedArtifact;
use crate::RustPlanBlocker;
use crate::RustPlanCoreError;
use crate::RustUnitEffect;
use crate::RustUnitLimits;
use crate::RustUnitObservation;
use crate::RustUnitOutcome;

const CRATE_NAME_FLAG: &str = "--crate-name";
const CRATE_TYPE_FLAG: &str = "--crate-type";
const EDITION_FLAG: &str = "--edition";
const TARGET_FLAG: &str = "--target";
const PROFILE_ENV: &str = "PROFILE";
const HOST_ENV: &str = "HOST";
const TARGET_ENV: &str = "TARGET";
const EXECUTION_KIND_ENV: &str = "MANTLE_RUST_UNIT_EXECUTION_KIND";
const OUTCOME_SUCCESS: &str = "success";
const OUTCOME_FAILED: &str = "failed";
const OUTCOME_REUSED: &str = "reused";

#[derive(Serialize)]
struct EffectPreimage<'a> {
    unit_id: &'a str,
    compiler_identity: &'a str,
    arguments: &'a [String],
    environment: &'a BTreeMap<String, String>,
    input_identities: &'a [String],
    expected_outputs: &'a [String],
    limits: RustUnitLimits,
}

#[derive(Serialize)]
struct OutcomePreimage<'a> {
    unit_id: &'a str,
    effect_id: &'a str,
    status: &'a str,
    artifacts: &'a [ObservedArtifact],
    blocker: &'a Option<RustPlanBlocker>,
}

struct DigestValidation<'a> {
    code: &'static str,
    value: &'a str,
}

struct BlockerInput<'a> {
    class: &'a str,
    subject: &'a str,
    detail: &'a str,
}

pub(crate) fn effect_for_unit(
    workspace: &crate::AdmittedWorkspace,
    draft: &crate::planning::UnitDraft,
    dependency_unit_ids: &[String],
) -> Result<RustUnitEffect, RustPlanCoreError> {
    let arguments = unit_arguments(draft);
    let environment = unit_environment(workspace, draft);
    let expected_outputs = vec![expected_output_name(draft)];
    let unit_bounds = RustUnitLimits::default();
    validate_effect_bounds(&arguments, &environment, dependency_unit_ids, &expected_outputs, unit_bounds)?;
    let preimage = EffectPreimage {
        unit_id: &draft.unit_id,
        compiler_identity: &workspace.toolchain.compiler_identity,
        arguments: &arguments,
        environment: &environment,
        input_identities: dependency_unit_ids,
        expected_outputs: &expected_outputs,
        limits: unit_bounds,
    };
    let effect_id = crate::identity::effect_digest(&preimage)?;
    debug_assert_eq!(effect_id.len(), crate::BLAKE3_HEX_CHARS);
    debug_assert!(!arguments.is_empty());
    Ok(RustUnitEffect {
        effect_id,
        unit_id: draft.unit_id.clone(),
        compiler_identity: workspace.toolchain.compiler_identity.clone(),
        arguments,
        environment,
        input_identities: dependency_unit_ids.to_vec(),
        expected_outputs,
        limits: unit_bounds,
    })
}

pub fn classify_cache_observation(
    effect: &RustUnitEffect,
    mut observation: CacheObservation,
) -> Result<CacheDecision, RustPlanCoreError> {
    require_effect_identity(effect, &observation.effect_id)?;
    observation.artifacts.sort();
    observation.artifacts.dedup();
    debug_assert_eq!(effect.effect_id, observation.effect_id);
    debug_assert!(
        u32::try_from(observation.artifacts.len()).is_ok_and(|count| count <= effect.limits.output_identities_max)
    );
    match observation.status {
        CacheObservationStatus::Hit => {
            validate_observed_outputs(effect, &observation.artifacts)?;
            Ok(CacheDecision::Reuse {
                artifacts: observation.artifacts,
            })
        }
        CacheObservationStatus::Miss => Ok(CacheDecision::Execute),
        CacheObservationStatus::Failed => {
            let code = observation
                .failure_code
                .filter(|value| !value.is_empty())
                .ok_or_else(|| RustPlanCoreError::mismatch("cache-failure-code-empty", effect.unit_id.clone()))?;
            Ok(CacheDecision::Block {
                blocker: blocker(BlockerInput {
                    class: "cache-capability-failed",
                    subject: &effect.unit_id,
                    detail: &code,
                }),
            })
        }
    }
}

pub fn classify_unit_observation(
    effect: &RustUnitEffect,
    mut observation: RustUnitObservation,
) -> Result<RustUnitOutcome, RustPlanCoreError> {
    require_effect_identity(effect, &observation.effect_id)?;
    validate_observation_bounds(effect, &observation)?;
    observation.artifacts.sort();
    observation.artifacts.dedup();
    debug_assert_eq!(effect.effect_id, observation.effect_id);
    debug_assert!(
        u32::try_from(observation.artifacts.len()).is_ok_and(|count| count <= effect.limits.output_identities_max)
    );
    let (status, blocker) = if observation.status_code == 0 {
        validate_observed_outputs(effect, &observation.artifacts)?;
        (OUTCOME_SUCCESS, None)
    } else {
        (
            OUTCOME_FAILED,
            Some(blocker(BlockerInput {
                class: "compiler-execution-failed",
                subject: &effect.unit_id,
                detail: &observation.status_code.to_string(),
            })),
        )
    };
    finish_outcome(effect, status, observation.artifacts, blocker)
}

pub fn reused_unit_outcome(
    effect: &RustUnitEffect,
    artifacts: Vec<ObservedArtifact>,
) -> Result<RustUnitOutcome, RustPlanCoreError> {
    validate_observed_outputs(effect, &artifacts)?;
    debug_assert!(!artifacts.is_empty());
    debug_assert!(!effect.effect_id.is_empty());
    finish_outcome(effect, OUTCOME_REUSED, artifacts, None)
}

fn finish_outcome(
    effect: &RustUnitEffect,
    status: &str,
    artifacts: Vec<ObservedArtifact>,
    blocker: Option<RustPlanBlocker>,
) -> Result<RustUnitOutcome, RustPlanCoreError> {
    let preimage = OutcomePreimage {
        unit_id: &effect.unit_id,
        effect_id: &effect.effect_id,
        status,
        artifacts: &artifacts,
        blocker: &blocker,
    };
    let receipt_blake3 = crate::identity::outcome_digest(&preimage)?;
    debug_assert_eq!(receipt_blake3.len(), crate::BLAKE3_HEX_CHARS);
    debug_assert!(!status.is_empty());
    Ok(RustUnitOutcome {
        unit_id: effect.unit_id.clone(),
        effect_id: effect.effect_id.clone(),
        status: status.to_string(),
        artifacts,
        blocker,
        receipt_blake3,
    })
}

fn unit_arguments(draft: &crate::planning::UnitDraft) -> Vec<String> {
    let arguments = vec![
        CRATE_NAME_FLAG.to_string(),
        draft.target.crate_name.clone(),
        draft.target.source_identity.clone(),
        CRATE_TYPE_FLAG.to_string(),
        draft.target.kind.label().to_string(),
        EDITION_FLAG.to_string(),
        draft.target.edition.clone(),
        TARGET_FLAG.to_string(),
        draft.selected_triple.clone(),
    ];
    debug_assert!(!arguments.is_empty());
    debug_assert!(u32::try_from(arguments.len()).is_ok_and(|count| count <= crate::MAX_UNIT_ARGUMENTS));
    arguments
}

fn unit_environment(
    workspace: &crate::AdmittedWorkspace,
    draft: &crate::planning::UnitDraft,
) -> BTreeMap<String, String> {
    let environment = BTreeMap::from([
        (PROFILE_ENV.to_string(), workspace.profile.clone()),
        (HOST_ENV.to_string(), workspace.toolchain.host_triple.clone()),
        (TARGET_ENV.to_string(), draft.selected_triple.clone()),
        (EXECUTION_KIND_ENV.to_string(), draft.execution_kind.label().to_string()),
    ]);
    debug_assert_eq!(environment.len(), crate::UNIT_ENVIRONMENT_ENTRY_COUNT);
    debug_assert!(environment.values().all(|value| !value.is_empty()));
    environment
}

fn expected_output_name(draft: &crate::planning::UnitDraft) -> String {
    let name = alloc::format!("{}:{}", draft.target.crate_name, draft.target.kind.label());
    debug_assert!(!name.is_empty());
    debug_assert!(name.contains(':'));
    name
}

fn validate_effect_bounds(
    arguments: &[String],
    environment: &BTreeMap<String, String>,
    inputs: &[String],
    outputs: &[String],
    unit_bounds: RustUnitLimits,
) -> Result<(), RustPlanCoreError> {
    require_count("effect-arguments", arguments.len(), unit_bounds.arguments_max)?;
    require_count("effect-environment", environment.len(), unit_bounds.environment_entries_max)?;
    require_count("effect-inputs", inputs.len(), unit_bounds.input_identities_max)?;
    require_count("effect-outputs", outputs.len(), unit_bounds.output_identities_max)?;
    debug_assert!(!outputs.is_empty());
    debug_assert!(!arguments.is_empty());
    Ok(())
}

fn validate_observation_bounds(
    effect: &RustUnitEffect,
    observation: &RustUnitObservation,
) -> Result<(), RustPlanCoreError> {
    if observation.stdout_bytes > effect.limits.stdout_bytes_max {
        return Err(RustPlanCoreError::mismatch("stdout-limit", effect.unit_id.clone()));
    }
    if observation.stderr_bytes > effect.limits.stderr_bytes_max {
        return Err(RustPlanCoreError::mismatch("stderr-limit", effect.unit_id.clone()));
    }
    require_count("observed-artifacts", observation.artifacts.len(), effect.limits.output_identities_max)?;
    validate_digest(DigestValidation {
        code: "stdout-digest",
        value: &observation.stdout_blake3,
    })?;
    validate_digest(DigestValidation {
        code: "stderr-digest",
        value: &observation.stderr_blake3,
    })?;
    Ok(())
}

fn validate_observed_outputs(effect: &RustUnitEffect, artifacts: &[ObservedArtifact]) -> Result<(), RustPlanCoreError> {
    let names = artifacts.iter().map(|artifact| artifact.name.as_str()).collect::<alloc::collections::BTreeSet<_>>();
    for artifact in artifacts {
        validate_digest(DigestValidation {
            code: "artifact-digest",
            value: &artifact.digest_blake3,
        })?;
    }
    if !effect.expected_outputs.iter().all(|expected| names.contains(expected.as_str())) {
        return Err(RustPlanCoreError::mismatch("expected-output-missing", effect.unit_id.clone()));
    }
    debug_assert!(names.len() <= artifacts.len());
    debug_assert!(!effect.expected_outputs.is_empty());
    Ok(())
}

fn require_effect_identity(effect: &RustUnitEffect, observed: &str) -> Result<(), RustPlanCoreError> {
    if effect.effect_id != observed {
        return Err(RustPlanCoreError::mismatch("effect-identity", effect.unit_id.clone()));
    }
    debug_assert!(!effect.effect_id.is_empty());
    debug_assert_eq!(effect.effect_id.len(), crate::BLAKE3_HEX_CHARS);
    Ok(())
}

fn validate_digest(input: DigestValidation<'_>) -> Result<(), RustPlanCoreError> {
    if !crate::identity::lowercase_blake3(input.value) {
        return Err(RustPlanCoreError::mismatch(input.code, input.value.to_string()));
    }
    Ok(())
}

fn require_count(code: &'static str, observed: usize, maximum: u32) -> Result<(), RustPlanCoreError> {
    let observed = u32::try_from(observed).map_err(|_| RustPlanCoreError::invalid("count-width", code))?;
    if observed > maximum {
        return Err(RustPlanCoreError::LimitExceeded {
            code,
            observed,
            maximum,
        });
    }
    Ok(())
}

fn blocker(input: BlockerInput<'_>) -> RustPlanBlocker {
    debug_assert!(!input.class.is_empty());
    debug_assert!(!input.subject.is_empty());
    RustPlanBlocker {
        class: input.class.to_string(),
        subject: input.subject.to_string(),
        detail: input.detail.to_string(),
    }
}
