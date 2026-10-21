//! Build capability observations, independent of terminal presentation.
use std::path::Path;

use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::EffectId;
use mantle_application_contract::EffectKind;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::EffectPlan;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::realization::RealizationEffect;

use crate::RunError;

pub(crate) fn plan(effect: RealizationEffect) -> Result<EffectPlan, RunError> {
    mantle_application_contract::realization::realization_effect_plan(effect)
        .map_err(|error| RunError::Internal(format!("planning realization: {}", error.code())))
}

/// Classify an actual adapter fact, not the handler's return value.
pub(crate) fn classify(plan: &EffectPlan, id: &str, kind: EffectKind, succeeded: bool) -> Result<(), RunError> {
    let observed = Observation {
        effect_id: EffectId(id.to_string()),
        kind,
        status: if succeeded {
            ObservationStatus::Succeeded
        } else {
            ObservationStatus::Failed
        },
        output: EffectOutput::None,
        usage: EffectMeasure::Calls(1),
        diagnostics_code: (!succeeded).then(|| "realization-capability-failed".to_string()),
    };
    match (mantle_application_contract::classify_observations(plan, &[observed]), succeeded) {
        (ApplicationOutcome::Completed, true) | (ApplicationOutcome::Failed { .. }, false) => Ok(()),
        (outcome, _) => Err(RunError::Internal(format!("realization observations contradict plan: {outcome:?}"))),
    }
}

/// Check every reported successful root, even when another root failed.
/// `symlink_metadata` accepts store symlinks without requiring their host target.
pub(crate) fn verify_outputs(outcomes: &[crunch_build::BuildOutcome], output_dir: &Path) -> Result<(), RunError> {
    for outcome in outcomes {
        if outcome.outputs.is_empty() {
            return Err(RunError::Build(format!("build returned no outputs for {}", outcome.drv_path)));
        }
        for info in outcome.outputs.values() {
            let path = output_dir.join(info.store_path.to_string());
            std::fs::symlink_metadata(&path).map_err(|error| {
                RunError::Build(format!("reported build output is missing: {}: {error}", path.display()))
            })?;
        }
    }
    Ok(())
}

pub(crate) fn pipeline(
    plan: &EffectPlan,
    config: &crunch_pipeline::BuildConfig,
    result: Result<crunch_pipeline::PipelineResult, RunError>,
) -> Result<crunch_pipeline::PipelineResult, RunError> {
    let verified = result.as_ref().ok().map(|result| {
        verify_outputs(&result.outcomes, &config.output_dir)?;
        if result.failed.is_empty() && !result.root_labels.is_empty() && result.outcomes.is_empty() {
            return Err(RunError::Build("build completed without realizing its selected roots".to_string()));
        }
        Ok(())
    });
    let succeeded = matches!(&verified, Some(Ok(()))) && matches!(&result, Ok(result) if result.failed.is_empty());
    classify(plan, "realization-build", EffectKind::StoreAccess, succeeded)?;
    if let Some(verification) = verified {
        verification?;
    }
    let result = result?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use nix_compat::store_path::StorePath;

    use super::*;

    #[test]
    fn a_successful_root_cannot_have_an_empty_output_set() {
        let dir = tempfile::tempdir().expect("temporary output directory");
        let outcome = crunch_build::BuildOutcome {
            drv_path: StorePath::from_name_and_digest_fixed("test.drv", [1; 20]).expect("drv"),
            outputs: Default::default(),
            substitutions: Default::default(),
            cached: false,
            log: None,
        };
        assert!(verify_outputs(&[outcome], dir.path()).is_err());
    }

    #[test]
    fn reported_path_must_exist_but_store_symlink_target_need_not() {
        let directory = tempfile::tempdir().expect("output directory");
        let store_path = StorePath::from_name_and_digest_fixed("output", [2; 20]).expect("store path");
        let path = directory.path().join(store_path.to_string());
        let info = snix_store::path_info::PathInfo {
            store_path,
            node: snix_castore::Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("/unmaterialized/logical-store-target").expect("target"),
            },
            references: Vec::new(),
            nar_size: 1,
            nar_sha256: [0; 32],
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        };
        let outcome = crunch_build::BuildOutcome {
            drv_path: StorePath::from_name_and_digest_fixed("test.drv", [1; 20]).expect("drv"),
            outputs: std::collections::BTreeMap::from([("out".to_string(), info)]),
            substitutions: Default::default(),
            cached: true,
            log: None,
        };
        assert!(verify_outputs(std::slice::from_ref(&outcome), directory.path()).is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("/unmaterialized/logical-store-target", &path).expect("store symlink");
            assert!(verify_outputs(&[outcome], directory.path()).is_ok());
        }
        #[cfg(not(unix))]
        {
            std::fs::write(&path, b"materialized output").expect("output");
            assert!(verify_outputs(&[outcome], directory.path()).is_ok());
        }
    }
}
