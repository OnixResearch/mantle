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
        let result = capability();
        if let EffectMeasure::Calls(count) = &mut observation.usage {
            *count += 1;
        }
        if result.is_err() {
            observation.status = ObservationStatus::Failed;
            observation.diagnostics_code = Some(format!("{}-failed", effect.id()));
        } else if observation.status == ObservationStatus::Skipped {
            observation.status = ObservationStatus::Succeeded;
        }
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
        let result = capability.await;
        if let EffectMeasure::Calls(count) = &mut observation.usage {
            *count += 1;
        }
        if result.is_err() {
            observation.status = ObservationStatus::Failed;
            observation.diagnostics_code = Some(format!("{}-failed", effect.id()));
        } else if observation.status == ObservationStatus::Skipped {
            observation.status = ObservationStatus::Succeeded;
        }
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
    use mantle_application_contract::CommandFamily;
    use mantle_application_contract::EffectId;
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
    use std::io::Read;
    let mut count = 0u64;
    let bytes = std::fs::File::open(path).and_then(|file| {
        let mut reader = file.take((expected.len() as u64).saturating_add(1));
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 8192];
        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
            count += read as u64;
        }
        Ok(hasher.finalize())
    });
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
        usage: EffectMeasure::Bytes(count),
        diagnostics_code: bytes.as_ref().err().map(|_| "bootstrap-readback-failed".into()),
    };
    match (classify_observations(&plan, &[observation]), bytes) {
        (ApplicationOutcome::Completed, Ok(_)) => Ok(()),
        (_, Err(error)) => Err(RunError::Internal(format!("reading back {}: {error}", path.display()))),
        (outcome, _) => Err(RunError::Build(format!(
            "published output {} differs from planned content: {outcome:?}",
            path.display()
        ))),
    }
}
