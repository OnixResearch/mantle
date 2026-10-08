//! Application fixtures over real, bounded unit facts and in-memory ports.

use mantle_rust_plan_app::AdapterError;
use mantle_rust_plan_app::BuildScriptPostprocessFailure;
use mantle_rust_plan_app::CacheDisposition;
use mantle_rust_plan_app::CacheObservation;
use mantle_rust_plan_app::CacheRestore;
use mantle_rust_plan_app::ExecutionError;
use mantle_rust_plan_app::PrepareOutcome;
use mantle_rust_plan_app::ProcessAttempt;
use mantle_rust_plan_app::RustCacheAccess;
use mantle_rust_plan_app::UnitExecutionResult;
use mantle_rust_plan_app::UnitExecutor;
use mantle_rust_plan_app::execute_existing_units;
use mantle_rust_plan_app::plan_existing_unit_effects;
use mantle_rust_plan_core::Blake3Digest;
use mantle_rust_plan_core::CompilerRestoreKind;
use mantle_rust_plan_core::EffectId;
use mantle_rust_plan_core::ExistingUnitEffect;
use mantle_rust_plan_core::ExistingUnitFacts;
use mantle_rust_plan_core::PlanExecutionOutcome;
use mantle_rust_plan_core::ProcessEffectRole;
use mantle_rust_plan_core::ProcessWord;
use mantle_rust_plan_core::ProducerArtifactObservation;
use mantle_rust_plan_core::ResolvedProcessObservation;
use mantle_rust_plan_core::ResolvedUnitEffect;
use mantle_rust_plan_core::ResolvedUnitFacts;
use mantle_rust_plan_core::RestoredCompilerArtifactObservation;
use mantle_rust_plan_core::UnitObservation;
use mantle_rust_plan_core::UnitObservationStatus;
use mantle_rust_plan_core::admit_resolved_build_script_after_cache;
use mantle_rust_plan_core::classify_restored_compiler_artifact;

fn unit(id: &str, order: u32, dependencies: &[&str]) -> ExistingUnitFacts {
    ExistingUnitFacts {
        unit_id: id.into(),
        package_id: "real-package@0.1.0".into(),
        target_name: "real_target".into(),
        target_kind: "lib".into(),
        execution_kind: "target".into(),
        dependency_unit_ids: dependencies.iter().map(|value| (*value).into()).collect(),
        arguments: vec!["--crate-name".into(), "real_target".into(), "--emit=link".into()],
        environment: vec![("CARGO_PKG_VERSION".into(), "0.1.0".into())],
        input_identities: vec!["source:real-target".into()],
        expected_outputs: vec!["artifact:real-target".into()],
        execution_order: order,
    }
}

fn effects() -> Vec<ExistingUnitEffect> {
    plan_existing_unit_effects(vec![unit("consumer", 1, &["producer"]), unit("producer", 0, &[])])
        .expect("valid selected topology")
}

fn observation(effect: &ExistingUnitEffect) -> UnitObservation {
    UnitObservation {
        effect_id: effect.effect_id.clone(),
        unit_id: effect.unit_id.clone(),
        status: UnitObservationStatus::Succeeded,
        exit_code: Some(0),
        diagnostics_code: None,
    }
}

#[derive(Clone, Copy)]
enum ExecutorMode {
    Failed,
    WrongIdentity,
    ContradictorySuccess,
    HitThenMiss,
    WrongHitIdentity,
    CacheUnavailable,
    MissingProcess,
    FalseSuccessAfterFailedCompiler,
    WrongProcess,
    WrongProducer,
}

struct Executor {
    mode: ExecutorMode,
    calls: usize,
    compiler_calls: usize,
}

struct Receipt {
    observation: UnitObservation,
    process_attempts: Vec<ProcessAttempt>,
    produced_artifacts: Vec<String>,
}

fn resolved_facts(effect: &ExistingUnitEffect) -> ResolvedUnitFacts {
    let dependency_observations: Vec<_> = effect
        .dependency_unit_ids
        .iter()
        .map(|producer_unit_id| ProducerArtifactObservation {
            producer_unit_id: producer_unit_id.clone(),
            artifact_identity: "verified:artifact:real-target".into(),
        })
        .collect();
    let mut input_identities = effect.input_identities.clone();
    if !dependency_observations.is_empty() {
        input_identities.push("verified:artifact:real-target".into());
    }
    ResolvedUnitFacts {
        effect_id: effect.effect_id.clone(),
        unit_id: effect.unit_id.clone(),
        role: ProcessEffectRole::RustCompiler,
        attempt: 0,
        prior_attempt_identity: None,
        executable: ProcessWord::Utf8("rustc".into()),
        toolchain_identity: "recorded-toolchain".into(),
        arguments: effect.arguments.iter().cloned().map(ProcessWord::Utf8).collect(),
        environment: effect
            .environment
            .iter()
            .map(|(name, value)| (name.clone(), ProcessWord::Utf8(value.clone())))
            .collect(),
        working_directory: None,
        input_identities,
        expected_outputs: effect.expected_outputs.clone(),
        dependency_observations,
    }
}

impl UnitExecutor for Executor {
    type Prepared = ExistingUnitEffect;
    type Receipt = Receipt;

    fn prepare_unit(&mut self, effect: &ExistingUnitEffect) -> Result<PrepareOutcome<Self::Prepared>, AdapterError> {
        self.calls += 1;
        let mut facts = resolved_facts(effect);
        if matches!(self.mode, ExecutorMode::WrongProducer)
            && let Some(producer) = facts.dependency_observations.first_mut()
        {
            producer.artifact_identity = "unobserved:wrong".into();
            facts.input_identities.push("unobserved:wrong".into());
        }
        Ok(PrepareOutcome::Ready {
            prepared: effect.clone(),
            facts,
        })
    }

    fn execute_miss(
        &mut self,
        prepared: &mut Self::Prepared,
        _planned: &ExistingUnitEffect,
        effect: ResolvedUnitEffect,
    ) -> Result<Self::Receipt, AdapterError> {
        self.compiler_calls += 1;
        let mut observed = observation(prepared);
        if matches!(self.mode, ExecutorMode::Failed) {
            observed.status = UnitObservationStatus::Failed;
            observed.exit_code = Some(101);
        }
        let mut process_observation = ResolvedProcessObservation {
            effect_id: effect.facts.effect_id.clone(),
            unit_id: effect.facts.unit_id.clone(),
            resolved_blake3: effect.resolved_blake3.clone(),
            role: effect.facts.role,
            attempt: effect.facts.attempt,
            status: observed.status,
            exit_code: observed.exit_code,
            output_identities: if observed.status == UnitObservationStatus::Succeeded {
                effect
                    .facts
                    .expected_outputs
                    .iter()
                    .map(|output| (output.clone(), format!("verified:{output}")))
                    .collect()
            } else {
                Vec::new()
            },
        };
        if matches!(self.mode, ExecutorMode::FalseSuccessAfterFailedCompiler) {
            process_observation.status = UnitObservationStatus::Failed;
            process_observation.exit_code = Some(101);
            process_observation.output_identities.clear();
        }
        if matches!(self.mode, ExecutorMode::WrongProcess) {
            process_observation.effect_id = EffectId("effect:other".into());
        }
        let produced_artifacts =
            process_observation.output_identities.iter().map(|(_, digest)| digest.clone()).collect();
        let process_attempts = if matches!(self.mode, ExecutorMode::MissingProcess) {
            Vec::new()
        } else {
            vec![ProcessAttempt {
                effect,
                observation: process_observation,
            }]
        };
        Ok(Receipt {
            observation: observed,
            process_attempts,
            produced_artifacts,
        })
    }

    fn finish_unit(
        &mut self,
        _planned: &ExistingUnitEffect,
        _prepared: Self::Prepared,
        mut receipt: Self::Receipt,
        cache: CacheObservation,
        _primary: Option<&ResolvedUnitEffect>,
    ) -> Result<UnitExecutionResult, AdapterError> {
        if matches!(self.mode, ExecutorMode::WrongHitIdentity) {
            receipt.observation.unit_id.0 = "other-unit".into();
        }
        if matches!(self.mode, ExecutorMode::WrongIdentity) {
            receipt.observation.effect_id = EffectId("effect:other".into());
        }
        if matches!(self.mode, ExecutorMode::ContradictorySuccess) {
            receipt.observation.exit_code = Some(101);
        }
        let cache = if receipt.observation.status == UnitObservationStatus::Failed {
            CacheObservation::NotObserved
        } else {
            cache
        };
        Ok(UnitExecutionResult {
            observation: receipt.observation,
            cache,
            process_attempts: receipt.process_attempts,
            restored_compiler: None,
            build_script_postprocess_failure: None,
            produced_artifacts: receipt.produced_artifacts,
        })
    }
}

impl RustCacheAccess<ExistingUnitEffect, Receipt> for Executor {
    fn restore_or_miss(
        &mut self,
        prepared: &mut ExistingUnitEffect,
        _effect: &ResolvedUnitEffect,
    ) -> Result<CacheRestore<Receipt>, AdapterError> {
        if matches!(self.mode, ExecutorMode::CacheUnavailable) {
            return Err(AdapterError::new("cache-unavailable", "fixture cache failure"));
        }
        if matches!(self.mode, ExecutorMode::WrongHitIdentity)
            || (matches!(self.mode, ExecutorMode::HitThenMiss) && self.calls == 1)
        {
            let cache = if matches!(self.mode, ExecutorMode::WrongHitIdentity) {
                CacheObservation::ReusedOutput
            } else {
                CacheObservation::RestoredShared
            };
            return Ok(CacheRestore::Terminal {
                receipt: Receipt {
                    observation: observation(prepared),
                    process_attempts: Vec::new(),
                    produced_artifacts: vec!["verified:artifact:real-target".into()],
                },
                cache,
            });
        }
        Ok(CacheRestore::Miss)
    }
}

#[test]
fn cache_hit_and_miss_are_observed_without_a_second_lookup() {
    let effects = effects();
    let mut executor = Executor {
        mode: ExecutorMode::HitThenMiss,
        calls: 0,
        compiler_calls: 0,
    };
    let executed = execute_existing_units(&effects, &mut executor).expect("executor observed both effects");
    assert_eq!(executed.execution, PlanExecutionOutcome::Completed);
    assert_eq!(executed.cache, CacheDisposition::PartiallyHit);
    assert_eq!(executed.cache_observations, [CacheObservation::RestoredShared, CacheObservation::Miss]);
    assert_eq!(executor.calls, 2);
    assert_eq!(executor.compiler_calls, 1, "verified cache hit must not invoke compiler");
    assert_eq!(executed.process_attempts.len(), 1);
    assert_eq!(executed.observations[0].effect_id, effects[0].effect_id);
}

#[test]
fn wrong_cache_observation_and_cache_fault_never_complete_a_plan() {
    let effects = effects();
    let mut executor = Executor {
        mode: ExecutorMode::WrongHitIdentity,
        calls: 0,
        compiler_calls: 0,
    };
    let err = execute_existing_units(&effects, &mut executor)
        .expect_err("a reused artifact for another unit cannot attest this effect");
    assert!(matches!(err, ExecutionError::Adapter(error) if error.code == "cache-observation-mismatch"));
    assert_eq!(executor.calls, 1);

    let mut executor = Executor {
        mode: ExecutorMode::CacheUnavailable,
        calls: 0,
        compiler_calls: 0,
    };
    let err = execute_existing_units(&effects, &mut executor)
        .expect_err("a cache capability fault is never treated as a miss");
    assert!(matches!(err, ExecutionError::Adapter(error) if error.code == "cache-unavailable"));
    assert_eq!(executor.calls, 1);
}

#[test]
fn wrong_execution_observations_cannot_complete_the_plan() {
    let effects = effects();
    for mode in [ExecutorMode::WrongIdentity, ExecutorMode::ContradictorySuccess] {
        let mut executor = Executor {
            mode,
            calls: 0,
            compiler_calls: 0,
        };
        let observed = execute_existing_units(&effects, &mut executor).expect("executor responded");
        assert!(matches!(observed.execution, PlanExecutionOutcome::Rejected { .. }));
        assert_eq!(executor.calls, 1, "subsequent units must not run on a wrong observation");
        assert_eq!(observed.observations.len(), 1);
    }
}

#[test]
fn stopped_execution_does_not_claim_success_for_unobserved_consumers() {
    let effects = effects();
    let mut executor = Executor {
        mode: ExecutorMode::Failed,
        calls: 0,
        compiler_calls: 0,
    };
    let observed = execute_existing_units(&effects, &mut executor).expect("failure was observed");
    assert_eq!(executor.calls, 1);
    assert_eq!(observed.observations.len(), 1);
    assert_eq!(observed.cache, CacheDisposition::NotObserved);
    assert_eq!(observed.cache_observations, [CacheObservation::NotObserved]);
    assert!(matches!(observed.execution, PlanExecutionOutcome::Rejected {
        missing_effect_count: 1,
        ..
    }));
}

#[test]
fn missing_or_substituted_process_attempt_is_rejected_after_compiler_miss() {
    let effects = effects();
    for mode in [ExecutorMode::MissingProcess, ExecutorMode::WrongProcess] {
        let mut executor = Executor {
            mode,
            calls: 0,
            compiler_calls: 0,
        };
        let error = execute_existing_units(&effects, &mut executor)
            .expect_err("actual process attempt must be bound to admitted effect");
        assert!(match (mode, error) {
            (ExecutorMode::MissingProcess, ExecutionError::Adapter(error)) =>
                error.code == "missing-process-observation",
            (ExecutorMode::WrongProcess, ExecutionError::Blocked(blockers)) =>
                blockers.iter().any(|blocker| blocker.code == "wrong-resolved-observation"),
            _ => false,
        });
        assert_eq!(executor.calls, 1);
        assert_eq!(executor.compiler_calls, 1);
    }
}

#[test]
fn failed_policy_compiler_cannot_attest_a_successful_unit() {
    let effects = effects();
    let mut executor = Executor {
        mode: ExecutorMode::FalseSuccessAfterFailedCompiler,
        calls: 0,
        compiler_calls: 0,
    };
    let error = execute_existing_units(&effects, &mut executor)
        .expect_err("a failed compiler without its successful audit fallback cannot complete a unit");
    assert!(matches!(error, ExecutionError::Adapter(error) if error.code == "unproven-unit-success"));
    assert_eq!(executor.calls, 1);
    assert_eq!(executor.compiler_calls, 1);
}

#[test]
fn consumer_cannot_claim_an_unproduced_dependency_digest() {
    let effects = effects();
    let mut executor = Executor {
        mode: ExecutorMode::WrongProducer,
        calls: 0,
        compiler_calls: 0,
    };
    let error = execute_existing_units(&effects, &mut executor)
        .expect_err("consumer may only bind artifacts observed from its producer");
    assert!(matches!(error, ExecutionError::Adapter(error) if error.code == "wrong-producer-observation"));
    assert_eq!(executor.calls, 2);
    assert_eq!(executor.compiler_calls, 1, "consumer compiler must not run after wrong producer evidence");
}

/// A cached build-script compiler still requires a real script process.
#[derive(Clone, Copy)]
enum CachedScriptResult {
    ProcessFailed,
    PostprocessFailed,
    WrongScriptEvidence,
    WrongBlockerEvidence,
}

struct CachedScriptExecutor {
    script_calls: usize,
    result: CachedScriptResult,
}

impl UnitExecutor for CachedScriptExecutor {
    type Prepared = ExistingUnitEffect;
    type Receipt = Receipt;

    fn prepare_unit(&mut self, planned: &ExistingUnitEffect) -> Result<PrepareOutcome<Self::Prepared>, AdapterError> {
        Ok(PrepareOutcome::Ready {
            prepared: planned.clone(),
            facts: resolved_facts(planned),
        })
    }

    fn execute_miss(
        &mut self,
        _: &mut Self::Prepared,
        _: &ExistingUnitEffect,
        _: ResolvedUnitEffect,
    ) -> Result<Self::Receipt, AdapterError> {
        Err(AdapterError::new("unexpected-compiler", "restored compiler must not run again"))
    }

    fn finish_unit(
        &mut self,
        planned: &ExistingUnitEffect,
        _: Self::Prepared,
        mut receipt: Self::Receipt,
        cache: CacheObservation,
        primary: Option<&ResolvedUnitEffect>,
    ) -> Result<UnitExecutionResult, AdapterError> {
        let primary = primary.expect("cache terminal retains the admitted compiler");
        let restored = RestoredCompilerArtifactObservation {
            effect_id: planned.effect_id.clone(),
            unit_id: planned.unit_id.clone(),
            compiler_resolved_blake3: primary.resolved_blake3.clone(),
            cache_receipt_identity: Blake3Digest::from_slice(b"fixture-restored-compiler-receipt"),
            declared_outputs: planned.expected_outputs.clone(),
            output_artifact_set_identity: Blake3Digest::from_slice(b"fixture-restored-artifacts"),
            output_artifact_count: 1,
            cache_kind: CompilerRestoreKind::RestoredLocal,
        };
        let predecessor =
            classify_restored_compiler_artifact(planned, primary, &restored).expect("valid cache evidence");
        let mut script_facts = resolved_facts(planned);
        script_facts.role = ProcessEffectRole::BuildScriptRun;
        script_facts.prior_attempt_identity = Some(predecessor);
        script_facts.executable = ProcessWord::Utf8("fixture-build-script".into());
        script_facts.arguments = Vec::new();
        script_facts.expected_outputs = vec!["OUT_DIR".into(), "metadata-stdout".into()];
        script_facts.input_identities.push(restored.cache_receipt_identity.as_str().into());
        script_facts.input_identities.push(restored.output_artifact_set_identity.as_str().into());
        let script = admit_resolved_build_script_after_cache(planned, script_facts, primary, &restored)
            .expect("restored compiler may precede its real script");
        receipt.observation.status = UnitObservationStatus::Failed;
        let script_succeeded = !matches!(self.result, CachedScriptResult::ProcessFailed);
        let postprocess_failure = if script_succeeded {
            let blocker_class = "malformed-build-script-metadata";
            receipt.observation.exit_code = None;
            receipt.observation.diagnostics_code = Some(blocker_class.into());
            Some(BuildScriptPostprocessFailure {
                effect_id: planned.effect_id.clone(),
                unit_id: planned.unit_id.clone(),
                script_resolved_blake3: if matches!(self.result, CachedScriptResult::WrongScriptEvidence) {
                    Blake3Digest::from_slice(b"wrong-build-script-process")
                } else {
                    script.resolved_blake3.clone()
                },
                blocker_class: if matches!(self.result, CachedScriptResult::WrongBlockerEvidence) {
                    "substituted-metadata-blocker".into()
                } else {
                    blocker_class.into()
                },
            })
        } else {
            receipt.observation.exit_code = Some(101);
            None
        };
        receipt.process_attempts.push(ProcessAttempt {
            observation: ResolvedProcessObservation {
                effect_id: script.facts.effect_id.clone(),
                unit_id: script.facts.unit_id.clone(),
                resolved_blake3: script.resolved_blake3.clone(),
                role: ProcessEffectRole::BuildScriptRun,
                attempt: 0,
                status: if script_succeeded {
                    UnitObservationStatus::Succeeded
                } else {
                    UnitObservationStatus::Failed
                },
                exit_code: Some(if script_succeeded { 0 } else { 101 }),
                output_identities: if script_succeeded {
                    vec![
                        ("OUT_DIR".into(), "verified:out-dir".into()),
                        ("metadata-stdout".into(), "verified:metadata-stdout".into()),
                    ]
                } else {
                    Vec::new()
                },
            },
            effect: script,
        });
        self.script_calls += 1;
        Ok(UnitExecutionResult {
            observation: receipt.observation,
            cache,
            process_attempts: receipt.process_attempts,
            restored_compiler: Some(restored),
            build_script_postprocess_failure: postprocess_failure,
            produced_artifacts: receipt.produced_artifacts,
        })
    }
}

impl RustCacheAccess<ExistingUnitEffect, Receipt> for CachedScriptExecutor {
    fn restore_or_miss(
        &mut self,
        prepared: &mut ExistingUnitEffect,
        _: &ResolvedUnitEffect,
    ) -> Result<CacheRestore<Receipt>, AdapterError> {
        Ok(CacheRestore::Terminal {
            receipt: Receipt {
                observation: observation(prepared),
                process_attempts: Vec::new(),
                produced_artifacts: vec!["verified:artifact:real-target".into()],
            },
            cache: CacheObservation::RestoredLocal,
        })
    }
}

#[test]
fn restored_compiler_does_not_mask_a_failed_real_build_script() {
    let mut script_unit = unit("build-script", 0, &[]);
    script_unit.target_kind = "custom-build".into();
    script_unit.execution_kind = "host".into();
    let effects = plan_existing_unit_effects(vec![script_unit]).expect("build-script compiler effect admitted");
    let mut executor = CachedScriptExecutor {
        script_calls: 0,
        result: CachedScriptResult::ProcessFailed,
    };
    let observed = execute_existing_units(&effects, &mut executor).expect("real failed script observation classified");
    assert_eq!(executor.script_calls, 1);
    assert_eq!(observed.cache, CacheDisposition::AllHit);
    assert_eq!(observed.cache_observations, [CacheObservation::RestoredLocal]);
    assert_eq!(observed.process_attempts.len(), 1);
    assert_eq!(observed.process_attempts[0].observation.role, ProcessEffectRole::BuildScriptRun);
    assert_eq!(observed.execution, PlanExecutionOutcome::Failed { failed_effect_count: 1 });
}

#[test]
fn cached_compiler_and_successful_script_retain_a_real_metadata_blocker() {
    let mut script_unit = unit("build-script", 0, &[]);
    script_unit.target_kind = "custom-build".into();
    script_unit.execution_kind = "host".into();
    let effects = plan_existing_unit_effects(vec![script_unit]).expect("build-script compiler effect admitted");
    let mut executor = CachedScriptExecutor {
        script_calls: 0,
        result: CachedScriptResult::PostprocessFailed,
    };
    let observed = execute_existing_units(&effects, &mut executor).expect("real metadata blocker classified");
    assert_eq!(executor.script_calls, 1);
    assert_eq!(observed.cache, CacheDisposition::AllHit);
    assert_eq!(observed.execution, PlanExecutionOutcome::Failed { failed_effect_count: 1 });
    assert_eq!(observed.observations[0].diagnostics_code.as_deref(), Some("malformed-build-script-metadata"));
    assert_eq!(observed.process_attempts[0].observation.status, UnitObservationStatus::Succeeded);
    assert_eq!(observed.process_attempts[0].observation.exit_code, Some(0));
}

#[test]
fn cached_script_metadata_blocker_requires_matching_real_process_and_diagnostic() {
    let mut script_unit = unit("build-script", 0, &[]);
    script_unit.target_kind = "custom-build".into();
    script_unit.execution_kind = "host".into();
    let effects = plan_existing_unit_effects(vec![script_unit]).expect("build-script compiler effect admitted");
    for result in [
        CachedScriptResult::WrongScriptEvidence,
        CachedScriptResult::WrongBlockerEvidence,
    ] {
        let mut executor = CachedScriptExecutor {
            script_calls: 0,
            result,
        };
        let error = execute_existing_units(&effects, &mut executor)
            .expect_err("fabricated metadata blocker cannot turn a cache hit into proven failure");
        assert!(matches!(error, ExecutionError::Adapter(error) if error.code == "wrong-build-script-postprocess"));
        assert_eq!(executor.script_calls, 1);
    }
}
