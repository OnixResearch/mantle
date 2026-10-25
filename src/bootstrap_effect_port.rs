// HARDENING-BACKLOG 2026-10-22: pre-existing tigerstyle findings in this file are
// recorded in .cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/
// tigerstyle-backlog-2026-10-22.log and scheduled for the standalone hardening
// pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::no_unwrap)]

//! Bootstrap capability execution and independent output verification.
use std::path::Path;

use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::EffectPlan;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::bootstrap_effects::BootstrapEffect;
use mantle_application_contract::bootstrap_effects::plan_bootstrap_effects;
use mantle_application_contract::classify_observations;

use crate::errors::RunError;

/// Every boundary is admitted before the first capability is invoked.
pub(crate) struct BootstrapPort {
    plan: EffectPlan,
    observations: Vec<Observation>,
}

impl BootstrapPort {
    pub(crate) fn new(effects: &[(BootstrapEffect, u32)]) -> Result<Self, RunError> {
        let plan = plan_bootstrap_effects(effects)
            .map_err(|error| RunError::Internal(format!("planning bootstrap effects: {}", error.code())))?;
        let observations = plan
            .effects
            .iter()
            .map(|effect| Observation {
                effect_id: effect.effect_id.clone(),
                kind: effect.kind,
                status: ObservationStatus::Skipped,
                output: EffectOutput::None,
                usage: EffectMeasure::Calls(0),
                diagnostics_code: None,
            })
            .collect();
        Ok(Self { plan, observations })
    }

    /// Invoke an admitted capability and retain its actual return, including failure.
    pub(crate) fn call<T, E>(
        &mut self,
        effect: BootstrapEffect,
        capability: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        let observation = self
            .observations
            .iter_mut()
            .find(|item| item.effect_id.0 == effect.id())
            .expect("bootstrap capability must be planned before execution");
        debug_assert_eq!(observation.effect_id.0, effect.id());
        let result = capability();
        record_outcome(observation, effect, result.is_err());
        debug_assert!(!matches!(&observation.usage, EffectMeasure::Calls(0)));
        result
    }

    pub(crate) async fn call_async<T, E>(
        &mut self,
        effect: BootstrapEffect,
        capability: impl core::future::Future<Output = Result<T, E>>,
    ) -> Result<T, E> {
        let observation = self
            .observations
            .iter_mut()
            .find(|item| item.effect_id.0 == effect.id())
            .expect("bootstrap capability must be planned before execution");
        debug_assert_eq!(observation.effect_id.0, effect.id());
        let result = capability.await;
        record_outcome(observation, effect, result.is_err());
        debug_assert!(!matches!(&observation.usage, EffectMeasure::Calls(0)));
        result
    }

    /// No terminal success is admitted without a complete, consistent ledger.
    pub(crate) fn finish<T>(&self, result: Result<T, RunError>) -> Result<T, RunError> {
        self.finish_with(result, |outcome| {
            RunError::Internal(format!("bootstrap effect observations inconsistent: {outcome:?}"))
        })
    }

    pub(crate) fn finish_with<T, E>(
        &self,
        result: Result<T, E>,
        inconsistent: impl FnOnce(ApplicationOutcome) -> E,
    ) -> Result<T, E> {
        match (classify_observations(&self.plan, &self.observations), result) {
            (ApplicationOutcome::Completed, result) => result,
            (ApplicationOutcome::Failed { .. }, Err(error)) => Err(error),
            (outcome, _) => Err(inconsistent(outcome)),
        }
    }

    pub(crate) fn verify_bytes(&mut self, path: &Path, expected: &[u8]) -> Result<(), RunError> {
        self.call(BootstrapEffect::VerifyOutput, || verify_content(path, expected))
    }
}

/// The expected digest is admitted before the independent file read. Actual
/// byte usage and output identity are derived from the persisted bytes.
pub(crate) fn verify_content(path: &Path, expected: &[u8]) -> Result<(), RunError> {
    use mantle_application_contract::EffectId;
    use mantle_application_contract::EffectKind;
    let plan = readback_plan(expected)?;
    let (bytes, read_bytes) = hash_persisted_bytes(path, expected.len());
    let observation = Observation {
        effect_id: EffectId("bootstrap-content-readback".into()),
        kind: EffectKind::ReadFiles,
        status: if bytes.is_ok() {
            ObservationStatus::Succeeded
        } else {
            ObservationStatus::Failed
        },
        output: bytes
            .as_ref()
            .map_or(EffectOutput::None, |digest| EffectOutput::Identity(digest.to_hex().to_string())),
        usage: EffectMeasure::Bytes(read_bytes),
        diagnostics_code: bytes.as_ref().err().map(|_| "bootstrap-readback-failed".into()),
    };
    debug_assert_eq!(observation.effect_id.0, plan.effects[0].effect_id.0);
    debug_assert!(matches!(&observation.usage, EffectMeasure::Bytes(_)));
    match (classify_observations(&plan, &[observation]), bytes) {
        (ApplicationOutcome::Completed, Ok(_)) => Ok(()),
        (_, Err(error)) => Err(RunError::Internal(format!("reading back {}: {error}", path.display()))),
        (outcome, _) => Err(RunError::Build(format!(
            "published output {} differs from planned content: {outcome:?}",
            path.display()
        ))),
    }
}

/// Retain the actual capability outcome in the admitted ledger entry.
fn record_outcome(observation: &mut Observation, effect: BootstrapEffect, is_failed: bool) {
    debug_assert_eq!(observation.effect_id.0, effect.id());
    if let EffectMeasure::Calls(count) = &mut observation.usage {
        *count += 1;
    }
    if is_failed {
        observation.status = ObservationStatus::Failed;
        observation.diagnostics_code = Some(format!("{}-failed", effect.id()));
    } else if observation.status == ObservationStatus::Skipped {
        observation.status = ObservationStatus::Succeeded;
    }
    debug_assert!(!is_failed || observation.diagnostics_code.is_some());
}

/// Admit exactly one read-back effect for the expected content identity.
fn readback_plan(expected: &[u8]) -> Result<EffectPlan, RunError> {
    use mantle_application_contract::CommandFamily;
    use mantle_application_contract::EffectKind;
    use mantle_application_contract::EffectSpec;
    use mantle_application_contract::ExpectedOutput;
    use mantle_application_contract::plan_effects;
    let identity = blake3::hash(expected).to_hex().to_string();
    let plan = plan_effects(CommandFamily::Bootstrap, &[EffectSpec {
        effect_id: "bootstrap-content-readback",
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Bytes((expected.len() as u64).max(1)),
        expected_output: ExpectedOutput::Identity(&identity),
    }])
    .map_err(|error| RunError::Internal(format!("planning bootstrap readback: {}", error.code())))?;
    debug_assert_eq!(plan.effects.len(), 1);
    debug_assert_eq!(plan.effects[0].effect_id.0, "bootstrap-content-readback");
    Ok(plan)
}

/// Hash the persisted bytes. Every read before end of file consumes at least
/// one of the `limit_bytes` admitted bytes, so `limit_bytes + 1` reads bound
/// the loop even when the kernel returns short reads.
fn hash_persisted_bytes(path: &Path, expected_bytes: usize) -> (Result<blake3::Hash, std::io::Error>, u64) {
    use std::io::Read;
    const CHUNK_BYTES: usize = 8192;
    let limit_bytes = (expected_bytes as u64).saturating_add(1);
    let read_bound = limit_bytes.saturating_add(1);
    let mut read_bytes = 0u64;
    let digest = std::fs::File::open(path).and_then(|file| {
        let mut reader = file.take(limit_bytes);
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; CHUNK_BYTES];
        for _ in 0..read_bound {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
            read_bytes = read_bytes.saturating_add(read as u64);
        }
        Ok(hasher.finalize())
    });
    debug_assert!(read_bytes <= limit_bytes);
    debug_assert!(read_bound > limit_bytes);
    (digest, read_bytes)
}
