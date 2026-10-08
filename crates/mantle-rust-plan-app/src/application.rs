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
    let mut observations = Vec::with_capacity(effects.len());
    let mut cache_observations = Vec::with_capacity(effects.len());
    let mut process_attempts = Vec::new();
    let mut produced_by_unit: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    let mut hits = 0usize;
    let mut unknown_cache = false;
    for effect in effects {
        let (result, expected_primary, restored_primary, from_cache) = match executor
            .prepare_unit(effect)
            .map_err(ExecutionError::Adapter)?
        {
            PrepareOutcome::Stopped(result) => (result, None, None, false),
            PrepareOutcome::Ready { mut prepared, facts } => {
                verify_producer_observations(&produced_by_unit, &facts)?;
                let primary = admit_resolved_unit_effect(effect, facts, None).map_err(ExecutionError::Blocked)?;
                let restored = executor.restore_or_miss(&mut prepared, &primary).map_err(ExecutionError::Adapter)?;
                let (receipt, cache, from_cache, expected_primary, restored_primary) = match restored {
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
                (result, expected_primary, restored_primary, from_cache)
            }
        };
        let UnitExecutionResult {
            observation,
            cache,
            process_attempts: attempts,
            restored_compiler,
            build_script_postprocess_failure,
            produced_artifacts,
        } = result;
        let identity_matches = observation.effect_id == effect.effect_id && observation.unit_id == effect.unit_id;
        let success = observation.status == UnitObservationStatus::Succeeded
            && observation.exit_code.is_none_or(|code| code == 0)
            && identity_matches;
        if let Some(restored) = restored_compiler.as_ref() {
            let Some(primary) = restored_primary.as_ref() else {
                return Err(ExecutionError::Adapter(AdapterError::new(
                    "unexpected-restored-compiler",
                    "a compiler predecessor was supplied without a cache terminal result",
                )));
            };
            if !cache_matches_restore_kind(cache, restored.cache_kind) {
                return Err(ExecutionError::Adapter(AdapterError::new(
                    "cache-observation-mismatch",
                    "restored compiler kind differs from the actual cache result",
                )));
            }
            classify_restored_compiler_artifact(effect, primary, restored)
                .map_err(|blocker| ExecutionError::Blocked(vec![blocker]))?;
        }
        if success && !from_cache && expected_primary.is_some() && attempts.is_empty() {
            return Err(ExecutionError::Adapter(AdapterError::new(
                "missing-process-observation",
                "successful compiler miss omitted its process attempt",
            )));
        }
        if from_cache && success && effect.target_kind == "custom-build" && attempts.is_empty() {
            return Err(ExecutionError::Adapter(AdapterError::new(
                "missing-build-script-observation",
                "cached build-script compiler output does not prove its script ran",
            )));
        }
        let prior_attempt_count = process_attempts.len();
        if from_cache && !attempts.is_empty() {
            let (Some(primary), Some(restored)) = (restored_primary.as_ref(), restored_compiler.as_ref()) else {
                return Err(ExecutionError::Adapter(AdapterError::new(
                    "fabricated-cache-process",
                    "cache terminal process lacked a real restored compiler predecessor",
                )));
            };
            classify_cached_script_attempt(
                effect,
                attempts,
                primary,
                restored,
                &produced_by_unit,
                &mut process_attempts,
            )?;
        } else {
            classify_attempts(effect, attempts, expected_primary.as_ref(), &produced_by_unit, &mut process_attempts)?;
        }
        let actual_attempts = &process_attempts[prior_attempt_count..];
        let postprocess_failed = if let Some(failure) = build_script_postprocess_failure.as_ref() {
            let actual_script = actual_attempts.first();
            if !from_cache
                || effect.target_kind != "custom-build"
                || !identity_matches
                || observation.status != UnitObservationStatus::Failed
                || observation.exit_code.is_some()
                || failure.blocker_class.is_empty()
                || observation.diagnostics_code.as_deref() != Some(failure.blocker_class.as_str())
                || failure.effect_id != effect.effect_id
                || failure.unit_id != effect.unit_id
                || actual_attempts.len() != 1
                || !actual_script.is_some_and(|script| {
                    script.effect.resolved_blake3 == failure.script_resolved_blake3
                        && script.observation.resolved_blake3 == failure.script_resolved_blake3
                        && script.observation.role == ProcessEffectRole::BuildScriptRun
                        && script.observation.status == UnitObservationStatus::Succeeded
                        && script.observation.exit_code == Some(0)
                })
            {
                return Err(ExecutionError::Adapter(AdapterError::new(
                    "wrong-build-script-postprocess",
                    "metadata blocker must follow the matching real successful build-script process",
                )));
            }
            true
        } else {
            false
        };
        if success && expected_primary.is_some() {
            let compiler_succeeded = actual_attempts
                .iter()
                .rev()
                .find(|attempt| attempt.observation.role == ProcessEffectRole::RustCompiler)
                .is_some_and(|attempt| {
                    attempt.observation.status == UnitObservationStatus::Succeeded
                        && attempt.observation.exit_code == Some(0)
                });
            let final_role = if effect.target_kind == "custom-build" {
                ProcessEffectRole::BuildScriptRun
            } else {
                ProcessEffectRole::RustCompiler
            };
            let final_succeeded = actual_attempts.last().is_some_and(|attempt| {
                attempt.observation.role == final_role
                    && attempt.observation.status == UnitObservationStatus::Succeeded
                    && attempt.observation.exit_code == Some(0)
            });
            if !compiler_succeeded || !final_succeeded {
                return Err(ExecutionError::Adapter(AdapterError::new(
                    "unproven-unit-success",
                    "a successful compiler miss requires a successful final admitted process",
                )));
            }
        }
        if success
            && from_cache
            && effect.target_kind == "custom-build"
            && !actual_attempts.last().is_some_and(|attempt| {
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
        let cache_script_failed = from_cache
            && actual_attempts.len() == 1
            && actual_attempts[0].observation.role == ProcessEffectRole::BuildScriptRun
            && actual_attempts[0].observation.status == UnitObservationStatus::Failed;
        match cache {
            CacheObservation::NotObserved => unknown_cache = true,
            CacheObservation::Miss => {}
            CacheObservation::ReusedOutput | CacheObservation::RestoredLocal | CacheObservation::RestoredShared => {
                if !identity_matches
                    || (!success
                        && !(effect.target_kind == "custom-build" && (cache_script_failed || postprocess_failed)))
                {
                    return Err(ExecutionError::Adapter(AdapterError::new(
                        "cache-observation-mismatch",
                        "cache hit cannot attest a failed compiler or substituted unit",
                    )));
                }
                hits += 1;
            }
        }
        if produced_artifacts.iter().any(String::is_empty) {
            return Err(ExecutionError::Adapter(AdapterError::new(
                "invalid-produced-artifact",
                "shell supplied an empty produced artifact identity",
            )));
        }
        produced_by_unit.insert(effect.unit_id.0.as_str(), produced_artifacts);
        cache_observations.push(cache);
        observations.push(observation);
        if !success {
            break;
        }
    }
    let execution = classify_existing_unit_observations(effects, &observations);
    let cache = if unknown_cache || cache_observations.is_empty() {
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

fn classify_cached_script_attempt(
    planned: &ExistingUnitEffect,
    attempts: Vec<ProcessAttempt>,
    primary: &ResolvedUnitEffect,
    restored: &RestoredCompilerArtifactObservation,
    produced_by_unit: &BTreeMap<&str, Vec<String>>,
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
    verify_producer_observations(produced_by_unit, &effect.facts)?;
    let supplied_identity = effect.resolved_blake3;
    let admitted = admit_resolved_build_script_after_cache(planned, effect.facts, primary, restored)
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
    }
    Ok(())
}
