//! Explicit bootstrap capability boundaries and their shared effect envelope.
use alloc::vec::Vec;

use crate::CommandFamily;
use crate::EffectKind;
use crate::EffectMeasure;
use crate::EffectPlan;
use crate::EffectSpec;
use crate::ExpectedOutput;
use crate::PlanError;
use crate::plan_effects;

/// A concrete capability boundary, not a CLI handler outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapEffect {
    /// Load a policy, route, tree, or inventory through the filesystem port.
    ReadInput,
    /// Read admitted process environment values.
    ReadEnvironment,
    /// Read monotonic time for bounded execution and evidence.
    ReadClock,
    /// Publish files or prepare private output directories.
    WriteOutput,
    /// Ask the package resolver process for an output path.
    ResolvePackages,
    /// Realize a fixed-output seed through the store port.
    FetchSeed,
    /// Fetch source bytes through a network reader.
    FetchSources,
    /// Run one provider build process.
    MaterializeProvider,
    /// Independently re-read and validate published artifacts.
    VerifyOutput,
    /// Read release or fixed-output bytes from the network.
    ResolvePins,
    /// Atomically publish the reviewed pin replacement set.
    PublishPins,
    /// Run a provider smoke or expected-rejection process.
    SmokeProvider,
    /// Exchange replay control messages with the admitting parent.
    ReplayControl,
}

impl BootstrapEffect {
    /// Stable identity for observations of the boundary.
    pub const fn id(self) -> &'static str {
        match self {
            Self::ReadInput => "bootstrap-input",
            Self::ReadEnvironment => "bootstrap-environment",
            Self::ReadClock => "bootstrap-monotonic-clock",
            Self::WriteOutput => "bootstrap-output",
            Self::ResolvePackages => "bootstrap-package-resolution",
            Self::FetchSeed => "bootstrap-seed-fetch",
            Self::FetchSources => "bootstrap-source-fetch",
            Self::MaterializeProvider => "bootstrap-provider-materialization",
            Self::VerifyOutput => "bootstrap-output-verification",
            Self::ResolvePins => "bootstrap-pin-resolution",
            Self::PublishPins => "bootstrap-pin-publication",
            Self::SmokeProvider => "bootstrap-provider-smoke",
            Self::ReplayControl => "bootstrap-source-replay-control",
        }
    }

    /// Capability authority requested by this boundary.
    pub const fn kind(self) -> EffectKind {
        match self {
            Self::ReadInput | Self::VerifyOutput => EffectKind::ReadFiles,
            Self::ReadEnvironment => EffectKind::ReadEnvironment,
            Self::ReadClock => EffectKind::ReadClock,
            Self::WriteOutput | Self::PublishPins => EffectKind::WriteFiles,
            Self::ResolvePackages | Self::SmokeProvider | Self::MaterializeProvider => EffectKind::RunProcess,
            Self::FetchSeed => EffectKind::StoreAccess,
            Self::ResolvePins | Self::FetchSources => EffectKind::UseNetwork,
            Self::ReplayControl => EffectKind::InterprocessCommunication,
        }
    }
}

/// Admit all boundaries before invoking any bootstrap capability.
pub fn plan_bootstrap_effects(effects: &[(BootstrapEffect, u32)]) -> Result<EffectPlan, PlanError> {
    let specs: Vec<_> = effects
        .iter()
        .map(|(effect, calls)| EffectSpec {
            effect_id: effect.id(),
            kind: effect.kind(),
            limit: EffectMeasure::Calls(*calls),
            expected_output: ExpectedOutput::None,
        })
        .collect();
    plan_effects(CommandFamily::Bootstrap, &specs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApplicationOutcome;
    use crate::EffectOutput;
    use crate::Observation;
    use crate::ObservationStatus;
    use crate::classify_observations;

    #[test]
    fn publication_without_independent_readback_cannot_complete() {
        let plan =
            plan_bootstrap_effects(&[(BootstrapEffect::WriteOutput, 1), (BootstrapEffect::VerifyOutput, 1)]).unwrap();
        let observations = [Observation {
            effect_id: plan.effects[0].effect_id.clone(),
            kind: EffectKind::WriteFiles,
            status: ObservationStatus::Succeeded,
            output: EffectOutput::None,
            usage: EffectMeasure::Calls(1),
            diagnostics_code: None,
        }];
        assert!(matches!(classify_observations(&plan, &observations), ApplicationOutcome::Rejected {
            missing_effect_count: 1,
            ..
        }));
    }

    #[test]
    fn rejects_zero_or_duplicate_boundaries() {
        assert!(matches!(
            plan_bootstrap_effects(&[(BootstrapEffect::ReadInput, 0)]),
            Err(PlanError::ZeroLimit { .. })
        ));
        assert!(matches!(
            plan_bootstrap_effects(&[(BootstrapEffect::ReadInput, 1), (BootstrapEffect::ReadInput, 1)]),
            Err(PlanError::DuplicateEffectId { .. })
        ));
    }
}
