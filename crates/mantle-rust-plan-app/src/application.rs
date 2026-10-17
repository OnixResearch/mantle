//! Orchestration of admitted *real* Rust unit effects over explicit ports.
//!
//! The shell captures facts and translates process/cache results. This module
//! neither constructs rustc invocations nor invents execution observations.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use mantle_rust_plan_core::Blake3Digest;
use mantle_rust_plan_core::CompilerRestoreKind;
use mantle_rust_plan_core::ExistingUnitEffect;
use mantle_rust_plan_core::ExistingUnitFacts;
use mantle_rust_plan_core::PlanBlocker;
use mantle_rust_plan_core::PlanExecutionOutcome;
use mantle_rust_plan_core::ProcessEffectRole;
use mantle_rust_plan_core::ResolvedUnitEffect;
use mantle_rust_plan_core::ResolvedUnitFacts;
use mantle_rust_plan_core::RestoredCompilerArtifactObservation;
use mantle_rust_plan_core::RustPlanCompatibilityDecision;
use mantle_rust_plan_core::UnitObservation;
use mantle_rust_plan_core::UnitObservationStatus;
use mantle_rust_plan_core::admit_resolved_build_script_after_cache;
use mantle_rust_plan_core::admit_resolved_unit_effect;
use mantle_rust_plan_core::classify_resolved_process_observation;
use mantle_rust_plan_core::classify_restored_compiler_artifact;

use crate::ports::AdapterError;
use crate::ports::BuildScriptPostprocessFailure;
use crate::ports::CacheObservation;
use crate::ports::CacheRestore;
use crate::ports::CargoOracleCapture;
use crate::ports::CompilerInspection;
use crate::ports::CompilerInspectionRequest;
use crate::ports::OracleCaptureRequest;
use crate::ports::PrepareOutcome;
use crate::ports::ProcessAttempt;
use crate::ports::RustCacheAccess;
use crate::ports::UnitExecutionResult;
use crate::ports::UnitExecutor;
use crate::ports::WorkspaceFactsRequest;
use crate::ports::WorkspaceFactsSource;

/// Capability failures and semantic rejection remain distinct until the CLI
/// translates them into its existing diagnostic and exit status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    Adapter(AdapterError),
    Blocked(Vec<PlanBlocker>),
}

/// Cache accounting for effects actually observed by the executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheDisposition {
    NotObserved,
    AllMissed,
    PartiallyHit,
    AllHit,
}

/// The ordered observations may be partial if a unit failed or was skipped.
/// Classification rejects missing effects; it cannot turn partial execution
/// into evidence that the complete plan succeeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedExecution {
    pub observations: Vec<UnitObservation>,
    pub cache_observations: Vec<CacheObservation>,
    pub process_attempts: Vec<ProcessAttempt>,
    pub execution: PlanExecutionOutcome,
    pub cache: CacheDisposition,
}

/// Admit actual owned units captured by an adapter. Unlike the old scaffold
/// this does not derive IDs, arguments, inputs, outputs, or order from toy packages.
pub fn plan_existing_unit_effects(units: Vec<ExistingUnitFacts>) -> Result<Vec<ExistingUnitEffect>, Vec<PlanBlocker>> {
    mantle_rust_plan_core::plan_existing_unit_effects(units)
}

/// Classify typed observations against the *actual* ordered unit effects.
pub fn classify_existing_unit_observations(
    effects: &[ExistingUnitEffect],
    observations: &[UnitObservation],
) -> PlanExecutionOutcome {
    mantle_rust_plan_core::classify_existing_unit_observations(effects, observations)
}

/// Preserve legacy plan-surface classification without putting rendering or
/// Cargo JSON in this application contract.
pub fn classify_rust_plan_compatibility(no_cargo_oracle: bool, blockers: &[String]) -> RustPlanCompatibilityDecision {
    mantle_rust_plan_core::classify_rust_plan_compatibility(no_cargo_oracle, blockers)
}

/// Capture using one stateful shell adapter. The caller validates CLI options
/// before entering this operation. Cargo mode preserves the accepted order:
/// Cargo version, rustc version, metadata and unit graph, accepted receipt.
/// The no-Cargo path skips both Cargo calls. The selected versions are passed
/// directly into receipt materialization, not duplicated in adapter state.
/// Execution later plans from the actual serialized receipt and may occur in
/// a different process, so capture cannot return disposable process effects.
pub fn capture_existing_plan<A>(
    adapter: &mut A,
    request: &WorkspaceFactsRequest,
    include_oracle: bool,
) -> Result<(), AdapterError>
where
    A: WorkspaceFactsSource + CargoOracleCapture + CompilerInspection,
{
    let cargo_version = if include_oracle {
        Some(adapter.inspect_cargo_version()?)
    } else {
        None
    };
    let compiler = adapter.inspect_compiler(&CompilerInspectionRequest {
        profile: &request.profile,
    })?;
    if include_oracle {
        adapter.capture_metadata_and_graph(&OracleCaptureRequest {
            roots: &request.roots,
            profile: &request.profile,
        })?;
    }
    adapter.load_workspace_facts(request, cargo_version.as_deref(), &compiler)
}

/// Admit each final bound process effect before any cache or compiler action.
/// The same prepared shell inputs pass through a real cache restore or one
/// compiler miss. Both branches execute the shell's common topology finish
/// hook before a dependent unit is visited.
pub fn execute_existing_units<E>(
    effects: &[ExistingUnitEffect],
    executor: &mut E,
) -> Result<ObservedExecution, ExecutionError>
where
    E: UnitExecutor + RustCacheAccess<E::Prepared, E::Receipt>,
{
    let max_unit_count = usize::try_from(mantle_rust_plan_core::MAX_UNITS).map_err(|_| execution_unit_limit())?;
    if effects.len() > max_unit_count {
        return Err(execution_unit_limit());
    }
    let mut observations = Vec::with_capacity(effects.len());
    let mut cache_observations = Vec::with_capacity(effects.len());
    let mut process_attempts = Vec::new();
    let mut produced_by_unit: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    let mut hits = 0usize;
    let mut is_cache_observation_unknown = false;
    for effect in effects {
        let prepared = prepare_unit_execution(effect, executor, &produced_by_unit)?;
        let ClassifiedUnitExecution {
            observation,
            cache,
            produced_artifacts,
            is_success,
            is_cache_hit,
        } = classify_unit_execution(effect, prepared, &produced_by_unit, &mut process_attempts)?;
        if cache == CacheObservation::NotObserved {
            is_cache_observation_unknown = true;
        }
        if is_cache_hit {
            hits += 1;
        }
        if produced_artifacts.iter().any(String::is_empty) {
            return Err(ExecutionError::Adapter(AdapterError::new(
                "invalid-produced-artifact",
                "shell supplied an empty produced artifact identity",
            )));
        }
        if produced_by_unit.len() >= max_unit_count {
            return Err(execution_unit_limit());
        }
        produced_by_unit.insert(effect.unit_id.0.as_str(), produced_artifacts);
        cache_observations.push(cache);
        observations.push(observation);
        if !is_success {
            break;
        }
    }
    debug_assert_eq!(observations.len(), cache_observations.len());
    debug_assert!(hits <= cache_observations.len());
    let execution = classify_existing_unit_observations(effects, &observations);
    let cache = if is_cache_observation_unknown || cache_observations.is_empty() {
        CacheDisposition::NotObserved
    } else if hits == 0 {
        CacheDisposition::AllMissed
    } else if hits == cache_observations.len() {
        CacheDisposition::AllHit
    } else {
        CacheDisposition::PartiallyHit
    };
    Ok(ObservedExecution {
        observations,
        cache_observations,
        process_attempts,
        execution,
        cache,
    })
}

fn execution_unit_limit() -> ExecutionError {
    ExecutionError::Blocked(vec![PlanBlocker {
        code: String::from("unit-limit"),
        subject: String::from("units"),
        message: String::from("execution exceeds the admitted unit count"),
    }])
}

struct PreparedUnitExecution {
    result: UnitExecutionResult,
    expected_primary: Option<Blake3Digest>,
    restored_primary: Option<ResolvedUnitEffect>,
    is_from_cache: bool,
}

fn prepare_unit_execution<E>(
    effect: &ExistingUnitEffect,
    executor: &mut E,
    produced_by_unit: &BTreeMap<&str, Vec<String>>,
) -> Result<PreparedUnitExecution, ExecutionError>
where
    E: UnitExecutor + RustCacheAccess<E::Prepared, E::Receipt>,
{
    let (result, expected_primary, restored_primary, is_from_cache) =
        match executor.prepare_unit(effect).map_err(ExecutionError::Adapter)? {
            PrepareOutcome::Stopped(result) => (result, None, None, false),
            PrepareOutcome::Ready { mut prepared, facts } => {
                verify_producer_observations(produced_by_unit, &facts)?;
                let primary = admit_resolved_unit_effect(effect, facts, None).map_err(ExecutionError::Blocked)?;
                let restored = executor.restore_or_miss(&mut prepared, &primary).map_err(ExecutionError::Adapter)?;
                let (receipt, cache, is_from_cache, expected_primary, restored_primary) = match restored {
                    CacheRestore::Terminal { receipt, cache } => {
                        if cache == CacheObservation::Miss {
                            return Err(ExecutionError::Adapter(AdapterError::new(
                                "cache-observation-mismatch",
                                "terminal cache result cannot be a miss",
                            )));
                        }
                        (receipt, cache, true, None, Some(primary))
                    }
                    CacheRestore::Miss => {
                        let expected_primary = primary.resolved_blake3.clone();
                        let receipt =
                            executor.execute_miss(&mut prepared, effect, primary).map_err(ExecutionError::Adapter)?;
                        (receipt, CacheObservation::Miss, false, Some(expected_primary), None)
                    }
                };
                let result = executor
                    .finish_unit(effect, prepared, receipt, cache, restored_primary.as_ref())
                    .map_err(ExecutionError::Adapter)?;
                if result.cache != cache
                    && !(cache == CacheObservation::Miss && result.cache == CacheObservation::NotObserved)
                {
                    return Err(ExecutionError::Adapter(AdapterError::new(
                        "cache-observation-mismatch",
                        "finished unit disagrees with its cache decision",
                    )));
                }
                (result, expected_primary, restored_primary, is_from_cache)
            }
        };
    debug_assert!(!is_from_cache || expected_primary.is_none());
    debug_assert!(is_from_cache || restored_primary.is_none());
    Ok(PreparedUnitExecution {
        result,
        expected_primary,
        restored_primary,
        is_from_cache,
    })
}

struct ClassifiedUnitExecution {
    observation: UnitObservation,
    cache: CacheObservation,
    produced_artifacts: Vec<String>,
    is_success: bool,
    is_cache_hit: bool,
}

fn classify_unit_execution(
    effect: &ExistingUnitEffect,
    prepared: PreparedUnitExecution,
    produced_by_unit: &BTreeMap<&str, Vec<String>>,
    accepted: &mut Vec<ProcessAttempt>,
) -> Result<ClassifiedUnitExecution, ExecutionError> {
    let PreparedUnitExecution {
        result,
        expected_primary,
        restored_primary,
        is_from_cache,
    } = prepared;
    let UnitExecutionResult {
        observation,
        cache,
        process_attempts: attempts,
        restored_compiler,
        build_script_postprocess_failure,
        produced_artifacts,
    } = result;
    let is_identity_matched = observation.effect_id == effect.effect_id && observation.unit_id == effect.unit_id;
    let is_success = observation.status == UnitObservationStatus::Succeeded
        && observation.exit_code.is_none_or(|code| code == 0)
        && is_identity_matched;
    let prior_attempt_count = admit_observed_processes(
        ProcessAdmissionFacts {
            effect,
            cache,
            restored_compiler: restored_compiler.as_ref(),
            restored_primary: restored_primary.as_ref(),
            expected_primary: expected_primary.as_ref(),
            produced_by_unit,
            is_success,
            is_from_cache,
        },
        attempts,
        accepted,
    )?;
    let actual_attempts = &accepted[prior_attempt_count..];
    let is_postprocess_failed = validate_build_script_postprocess(
        effect,
        &observation,
        build_script_postprocess_failure.as_ref(),
        actual_attempts,
        is_from_cache,
    )?;
    validate_successful_unit_processes(SuccessfulProcessFacts {
        effect,
        actual_attempts,
        is_success,
        is_from_cache,
        has_expected_primary: expected_primary.is_some(),
    })?;
    let is_cache_hit = admit_cache_observation(CacheAdmissionFacts {
        effect,
        attempts: actual_attempts,
        cache,
        is_from_cache,
        is_success,
        is_identity_matched,
        is_postprocess_failed,
    })?;
    debug_assert!(accepted.len() >= prior_attempt_count);
    debug_assert!(is_from_cache || restored_primary.is_none());
    Ok(ClassifiedUnitExecution {
        observation,
        cache,
        produced_artifacts,
        is_success,
        is_cache_hit,
    })
}

struct ProcessAdmissionFacts<'a> {
    effect: &'a ExistingUnitEffect,
    cache: CacheObservation,
    restored_compiler: Option<&'a RestoredCompilerArtifactObservation>,
    restored_primary: Option<&'a ResolvedUnitEffect>,
    expected_primary: Option<&'a Blake3Digest>,
    produced_by_unit: &'a BTreeMap<&'a str, Vec<String>>,
    is_success: bool,
    is_from_cache: bool,
}

fn admit_observed_processes(
    facts: ProcessAdmissionFacts<'_>,
    attempts: Vec<ProcessAttempt>,
    accepted: &mut Vec<ProcessAttempt>,
) -> Result<usize, ExecutionError> {
    if let Some(restored) = facts.restored_compiler {
        let Some(primary) = facts.restored_primary else {
            return Err(ExecutionError::Adapter(AdapterError::new(
                "unexpected-restored-compiler",
                "a compiler predecessor was supplied without a cache terminal result",
            )));
        };
        if !cache_matches_restore_kind(facts.cache, restored.cache_kind) {
            return Err(ExecutionError::Adapter(AdapterError::new(
                "cache-observation-mismatch",
                "restored compiler kind differs from the actual cache result",
            )));
        }
        classify_restored_compiler_artifact(facts.effect, primary, restored)
            .map_err(|blocker| ExecutionError::Blocked(vec![blocker]))?;
    }
    let is_missing_compiler_attempt =
        facts.is_success && !facts.is_from_cache && facts.expected_primary.is_some() && attempts.is_empty();
    if is_missing_compiler_attempt {
        return Err(ExecutionError::Adapter(AdapterError::new(
            "missing-process-observation",
            "successful compiler miss omitted its process attempt",
        )));
    }
    let is_missing_cached_script =
        facts.is_from_cache && facts.is_success && facts.effect.target_kind == "custom-build" && attempts.is_empty();
    if is_missing_cached_script {
        return Err(ExecutionError::Adapter(AdapterError::new(
            "missing-build-script-observation",
            "cached build-script compiler output does not prove its script ran",
        )));
    }
    let prior_attempt_count = accepted.len();
    if facts.is_from_cache && !attempts.is_empty() {
        let (Some(primary), Some(restored)) = (facts.restored_primary, facts.restored_compiler) else {
            return Err(ExecutionError::Adapter(AdapterError::new(
                "fabricated-cache-process",
                "cache terminal process lacked a real restored compiler predecessor",
            )));
        };
        classify_cached_script_attempt(
            CachedScriptAdmission {
                planned: facts.effect,
                primary,
                restored,
                produced_by_unit: facts.produced_by_unit,
            },
            attempts,
            accepted,
        )?;
    } else {
        classify_attempts(facts.effect, attempts, facts.expected_primary, facts.produced_by_unit, accepted)?;
    }
    debug_assert!(accepted.len() >= prior_attempt_count);
    debug_assert!(!facts.is_from_cache || facts.expected_primary.is_none());
    Ok(prior_attempt_count)
}

fn validate_build_script_postprocess(
    effect: &ExistingUnitEffect,
    observation: &UnitObservation,
    failure: Option<&BuildScriptPostprocessFailure>,
    actual_attempts: &[ProcessAttempt],
    is_from_cache: bool,
) -> Result<bool, ExecutionError> {
    let Some(failure) = failure else {
        return Ok(false);
    };
    let is_failed_cached_script = is_from_cache
        && effect.target_kind == "custom-build"
        && observation.effect_id == effect.effect_id
        && observation.unit_id == effect.unit_id
        && observation.status == UnitObservationStatus::Failed
        && observation.exit_code.is_none();
    let is_matching_failure = !failure.blocker_class.is_empty()
        && observation.diagnostics_code.as_deref() == Some(failure.blocker_class.as_str())
        && failure.effect_id == effect.effect_id
        && failure.unit_id == effect.unit_id;
    let is_matching_script = actual_attempts.len() == 1
        && actual_attempts.first().is_some_and(|script| {
            script.effect.resolved_blake3 == failure.script_resolved_blake3
                && script.observation.resolved_blake3 == failure.script_resolved_blake3
                && script.observation.role == ProcessEffectRole::BuildScriptRun
                && script.observation.status == UnitObservationStatus::Succeeded
                && script.observation.exit_code == Some(0)
        });
    if !is_failed_cached_script || !is_matching_failure || !is_matching_script {
        return Err(ExecutionError::Adapter(AdapterError::new(
            "wrong-build-script-postprocess",
            "metadata blocker must follow the matching real successful build-script process",
        )));
    }
    debug_assert_eq!(actual_attempts.len(), 1);
    debug_assert_eq!(actual_attempts[0].observation.role, ProcessEffectRole::BuildScriptRun);
    Ok(true)
}

struct SuccessfulProcessFacts<'a> {
    effect: &'a ExistingUnitEffect,
    actual_attempts: &'a [ProcessAttempt],
    is_success: bool,
    is_from_cache: bool,
    has_expected_primary: bool,
}

fn validate_successful_unit_processes(facts: SuccessfulProcessFacts<'_>) -> Result<(), ExecutionError> {
    if facts.is_success && facts.has_expected_primary {
        let is_compiler_succeeded = facts
            .actual_attempts
            .iter()
            .rev()
            .find(|attempt| attempt.observation.role == ProcessEffectRole::RustCompiler)
            .is_some_and(|attempt| {
                attempt.observation.status == UnitObservationStatus::Succeeded
                    && attempt.observation.exit_code == Some(0)
            });
        let final_role = if facts.effect.target_kind == "custom-build" {
            ProcessEffectRole::BuildScriptRun
        } else {
            ProcessEffectRole::RustCompiler
        };
        let is_final_succeeded = facts.actual_attempts.last().is_some_and(|attempt| {
            attempt.observation.role == final_role
                && attempt.observation.status == UnitObservationStatus::Succeeded
                && attempt.observation.exit_code == Some(0)
        });
        if !is_compiler_succeeded || !is_final_succeeded {
            return Err(ExecutionError::Adapter(AdapterError::new(
                "unproven-unit-success",
                "a successful compiler miss requires a successful final admitted process",
            )));
        }
        debug_assert!(is_compiler_succeeded);
        debug_assert!(is_final_succeeded);
    }
    let is_cached_script_success =
        facts.is_success && facts.is_from_cache && facts.effect.target_kind == "custom-build";
    if is_cached_script_success
        && !facts.actual_attempts.last().is_some_and(|attempt| {
            attempt.observation.role == ProcessEffectRole::BuildScriptRun
                && attempt.observation.status == UnitObservationStatus::Succeeded
                && attempt.observation.exit_code == Some(0)
        })
    {
        return Err(ExecutionError::Adapter(AdapterError::new(
            "unproven-unit-success",
            "a restored build-script compiler does not prove its script succeeded",
        )));
    }
    Ok(())
}

struct CacheAdmissionFacts<'a> {
    effect: &'a ExistingUnitEffect,
    attempts: &'a [ProcessAttempt],
    cache: CacheObservation,
    is_from_cache: bool,
    is_success: bool,
    is_identity_matched: bool,
    is_postprocess_failed: bool,
}

fn admit_cache_observation(facts: CacheAdmissionFacts<'_>) -> Result<bool, ExecutionError> {
    if matches!(facts.cache, CacheObservation::NotObserved | CacheObservation::Miss) {
        return Ok(false);
    }
    let is_cache_script_failed = facts.is_from_cache
        && facts.attempts.len() == 1
        && facts.attempts[0].observation.role == ProcessEffectRole::BuildScriptRun
        && facts.attempts[0].observation.status == UnitObservationStatus::Failed;
    let is_failed_script_allowed =
        facts.effect.target_kind == "custom-build" && (is_cache_script_failed || facts.is_postprocess_failed);
    if !facts.is_identity_matched || (!facts.is_success && !is_failed_script_allowed) {
        return Err(ExecutionError::Adapter(AdapterError::new(
            "cache-observation-mismatch",
            "cache hit cannot attest a failed compiler or substituted unit",
        )));
    }
    debug_assert!(facts.is_identity_matched);
    debug_assert!(facts.is_success || is_failed_script_allowed);
    Ok(true)
}

fn verify_producer_observations(
    produced_by_unit: &BTreeMap<&str, Vec<String>>,
    facts: &ResolvedUnitFacts,
) -> Result<(), ExecutionError> {
    for producer in &facts.dependency_observations {
        if !produced_by_unit
            .get(producer.producer_unit_id.0.as_str())
            .is_some_and(|digests| digests.contains(&producer.artifact_identity))
        {
            return Err(ExecutionError::Adapter(AdapterError::new(
                "wrong-producer-observation",
                "resolved dependency identity was not produced by its declared earlier unit",
            )));
        }
    }
    Ok(())
}

fn cache_matches_restore_kind(cache: CacheObservation, kind: CompilerRestoreKind) -> bool {
    matches!(
        (cache, kind),
        (CacheObservation::ReusedOutput, CompilerRestoreKind::ReusedOutput)
            | (CacheObservation::RestoredLocal, CompilerRestoreKind::RestoredLocal)
            | (CacheObservation::RestoredShared, CompilerRestoreKind::RestoredShared)
    )
}

struct CachedScriptAdmission<'a> {
    planned: &'a ExistingUnitEffect,
    primary: &'a ResolvedUnitEffect,
    restored: &'a RestoredCompilerArtifactObservation,
    produced_by_unit: &'a BTreeMap<&'a str, Vec<String>>,
}

fn classify_cached_script_attempt(
    facts: CachedScriptAdmission<'_>,
    attempts: Vec<ProcessAttempt>,
    accepted: &mut Vec<ProcessAttempt>,
) -> Result<(), ExecutionError> {
    if attempts.len() != 1 {
        return Err(ExecutionError::Adapter(AdapterError::new(
            "fabricated-cache-process",
            "cached compiler can be followed by only one real build-script process",
        )));
    }
    let Some(ProcessAttempt { effect, observation }) = attempts.into_iter().next() else {
        return Err(ExecutionError::Adapter(AdapterError::new(
            "missing-build-script-observation",
            "cached build-script compiler output does not prove its script ran",
        )));
    };
    if effect.facts.role != ProcessEffectRole::BuildScriptRun {
        return Err(ExecutionError::Adapter(AdapterError::new(
            "fabricated-cache-process",
            "compiler cache reuse cannot stand in for an executed compiler process",
        )));
    }
    verify_producer_observations(facts.produced_by_unit, &effect.facts)?;
    let supplied_identity = effect.resolved_blake3;
    let admitted = admit_resolved_build_script_after_cache(facts.planned, effect.facts, facts.primary, facts.restored)
        .map_err(ExecutionError::Blocked)?;
    if admitted.resolved_blake3 != supplied_identity {
        return Err(ExecutionError::Adapter(AdapterError::new(
            "wrong-resolved-effect",
            "build-script process differs from its admitted cache predecessor",
        )));
    }
    classify_resolved_process_observation(&admitted, &observation)
        .map_err(|blocker| ExecutionError::Blocked(vec![blocker]))?;
    accepted.push(ProcessAttempt {
        effect: admitted,
        observation,
    });
    debug_assert_eq!(accepted.last().map(|entry| entry.observation.role), Some(ProcessEffectRole::BuildScriptRun));
    debug_assert_eq!(accepted.last().map(|entry| &entry.effect.resolved_blake3), Some(&supplied_identity));
    Ok(())
}

fn classify_attempts(
    planned: &ExistingUnitEffect,
    attempts: Vec<ProcessAttempt>,
    expected_primary: Option<&Blake3Digest>,
    produced_by_unit: &BTreeMap<&str, Vec<String>>,
    accepted: &mut Vec<ProcessAttempt>,
) -> Result<(), ExecutionError> {
    for (index, attempt) in attempts.into_iter().enumerate() {
        let prior = if index == 0 {
            None
        } else {
            accepted.last().map(|entry| &entry.observation)
        };
        let ProcessAttempt { effect, observation } = attempt;
        verify_producer_observations(produced_by_unit, &effect.facts)?;
        let supplied_identity = effect.resolved_blake3;
        let admitted = admit_resolved_unit_effect(planned, effect.facts, prior).map_err(ExecutionError::Blocked)?;
        if admitted.resolved_blake3 != supplied_identity
            || (index == 0 && expected_primary.is_some_and(|identity| identity != &admitted.resolved_blake3))
        {
            return Err(ExecutionError::Adapter(AdapterError::new(
                "wrong-resolved-effect",
                "executed process differs from its admitted attempt",
            )));
        }
        classify_resolved_process_observation(&admitted, &observation)
            .map_err(|blocker| ExecutionError::Blocked(vec![blocker]))?;
        accepted.push(ProcessAttempt {
            effect: admitted,
            observation,
        });
        debug_assert_eq!(accepted.last().map(|entry| &entry.effect.resolved_blake3), Some(&supplied_identity));
        debug_assert!(accepted.last().is_some_and(|entry| entry.observation.role == entry.effect.facts.role));
    }
    Ok(())
}
