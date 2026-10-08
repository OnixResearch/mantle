//! Shared application envelope: blockers, effect plans, observations, and
//! typed capability outcomes used by every command family.

use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::family::CommandFamily;

/// Maximum admitted effects in one application plan.
pub const MAX_EFFECTS_PER_PLAN: u32 = 4_096;

/// One bounded typed blocker produced by domain policy.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ApplicationBlocker {
    pub code: String,
    pub subject: String,
    pub message: String,
}

impl ApplicationBlocker {
    /// Construct one bounded blocker.
    pub fn new(code: &str, subject: &str, message: &str) -> Self {
        debug_assert!(!code.is_empty() && !subject.is_empty() && !message.is_empty());
        Self {
            code: String::from(code),
            subject: String::from(subject),
            message: String::from(message),
        }
    }
}

/// Typed capability failure raised by a port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityError {
    pub code: String,
    pub detail: String,
}

impl CapabilityError {
    /// Construct one typed capability failure.
    pub fn new(code: &str, detail: &str) -> Self {
        debug_assert!(!code.is_empty() && !detail.is_empty());
        Self {
            code: String::from(code),
            detail: String::from(detail),
        }
    }
}

/// Effect class an application plan may request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EffectKind {
    ReadFiles,
    WriteFiles,
    RunProcess,
    UseNetwork,
    ReadClock,
    ReadRandom,
    /// Composite access to a store-backed capability port.
    StoreAccess,
    /// An explicitly admitted read of the process environment by a port.
    ReadEnvironment,
}

/// Stable effect identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EffectId(pub String);
/// The unit of both an admitted effect limit and its observed usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectMeasure {
    Calls(u32),
    Items(u32),
    Bytes(u64),
}

/// The identity of an effect's produced output, if it produces one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectOutput {
    None,
    Identity(String),
}

/// A borrowed declaration; identifiers do not imply authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedOutput<'a> {
    None,
    Identity(&'a str),
}

/// One explicit bounded declaration supplied by application policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectSpec<'a> {
    pub effect_id: &'a str,
    pub kind: EffectKind,
    pub limit: EffectMeasure,
    pub expected_output: ExpectedOutput<'a>,
}

/// One planned effect with its declared capability and observation contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effect {
    pub effect_id: EffectId,
    pub kind: EffectKind,
    /// Family that owns the effect.
    pub family: CommandFamily,
    pub limit: EffectMeasure,
    pub expected_output: EffectOutput,
}

/// Ordered bounded effect plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectPlan {
    pub effects: Vec<Effect>,
}
/// The reason an application effect declaration was rejected before dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanError {
    EmptyPlan,
    TooManyEffects,
    EmptyEffectId { index: u32 },
    DuplicateEffectId { index: u32 },
    ZeroLimit { index: u32 },
    EmptyExpectedOutput { index: u32 },
}

impl PlanError {
    /// Stable diagnostic class retained through presentation.
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptyPlan => "effect-plan-empty",
            Self::TooManyEffects => "effect-plan-too-many-effects",
            Self::EmptyEffectId { .. } => "effect-plan-empty-id",
            Self::DuplicateEffectId { .. } => "effect-plan-duplicate-id",
            Self::ZeroLimit { .. } => "effect-plan-zero-limit",
            Self::EmptyExpectedOutput { .. } => "effect-plan-empty-expected-output",
        }
    }
}

/// Build a bounded plan from explicit effect declarations.
///
/// An empty or repeated identity, empty output identity, zero limit, or
/// oversized plan is rejected before any effect can run.
pub fn plan_effects(family: CommandFamily, specs: &[EffectSpec<'_>]) -> Result<EffectPlan, PlanError> {
    if specs.is_empty() {
        return Err(PlanError::EmptyPlan);
    }
    let requested_count = u32::try_from(specs.len()).map_err(|_| PlanError::TooManyEffects)?;
    if requested_count > MAX_EFFECTS_PER_PLAN {
        return Err(PlanError::TooManyEffects);
    }
    let mut effects = Vec::with_capacity(specs.len());
    for (index, spec) in specs.iter().enumerate() {
        let index = u32::try_from(index).map_err(|_| PlanError::TooManyEffects)?;
        if spec.effect_id.is_empty() {
            return Err(PlanError::EmptyEffectId { index });
        }
        if effects.iter().any(|effect: &Effect| effect.effect_id.0 == spec.effect_id) {
            return Err(PlanError::DuplicateEffectId { index });
        }
        let valid_limit = match spec.limit {
            EffectMeasure::Calls(count) | EffectMeasure::Items(count) => count > 0,
            EffectMeasure::Bytes(bytes) => bytes > 0,
        };
        if !valid_limit {
            return Err(PlanError::ZeroLimit { index });
        }
        let expected_output = match spec.expected_output {
            ExpectedOutput::None => EffectOutput::None,
            ExpectedOutput::Identity(identity) if !identity.is_empty() => {
                EffectOutput::Identity(String::from(identity))
            }
            ExpectedOutput::Identity(_) => return Err(PlanError::EmptyExpectedOutput { index }),
        };
        effects.push(Effect {
            effect_id: EffectId(String::from(spec.effect_id)),
            kind: spec.kind,
            family,
            limit: spec.limit,
            expected_output,
        });
    }
    debug_assert_eq!(effects.len(), specs.len());
    Ok(EffectPlan { effects })
}

/// Observation status reported by an adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservationStatus {
    Succeeded,
    Failed,
    Skipped,
}

/// One typed observation supplied by the executed adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub effect_id: EffectId,
    pub kind: EffectKind,
    pub status: ObservationStatus,
    pub output: EffectOutput,
    pub usage: EffectMeasure,
    pub diagnostics_code: Option<String>,
}

/// Terminal application outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationOutcome {
    /// Domain policy admitted the request and every effect observed success.
    Completed,
    /// Domain policy rejected the request; no effect ran.
    Blocked { blockers: Vec<ApplicationBlocker> },
    /// Effects observed failure; the count is exact.
    Failed { failed_effect_count: u32 },
    /// Observations do not match the plan.
    Rejected {
        unknown_effect_count: u32,
        missing_effect_count: u32,
    },
    /// A known effect contradicted its declared authority, output, or limit.
    Contradicted { effect_count: u32 },
}

/// Classify real adapter observations against explicit policy.
///
/// Unknown, repeated, or missing identities reject. Authority, output, unit
/// dimension, and usage mismatches reject even if the adapter claims success.
pub fn classify_observations(plan: &EffectPlan, observations: &[Observation]) -> ApplicationOutcome {
    if plan.effects.is_empty()
        || plan.effects.len() > MAX_EFFECTS_PER_PLAN as usize
        || plan.effects.iter().enumerate().any(|(index, effect)| {
            effect.effect_id.0.is_empty()
                || effect.family != plan.effects[0].family
                || matches!(effect.limit, EffectMeasure::Calls(0) | EffectMeasure::Items(0) | EffectMeasure::Bytes(0))
                || matches!(&effect.expected_output, EffectOutput::Identity(identity) if identity.is_empty())
                || plan.effects[..index].iter().any(|prior| prior.effect_id == effect.effect_id)
        })
    {
        return ApplicationOutcome::Contradicted { effect_count: 1 };
    }
    let mut seen = Vec::with_capacity(plan.effects.len());
    let mut unknown: u32 = 0;
    let mut contradicted: u32 = 0;
    let mut failed: u32 = 0;
    for observation in observations {
        let Some(effect) = plan.effects.iter().find(|effect| effect.effect_id == observation.effect_id) else {
            unknown = unknown.saturating_add(1);
            continue;
        };
        if seen.contains(&observation.effect_id.0.as_str()) {
            unknown = unknown.saturating_add(1);
            continue;
        }
        seen.push(observation.effect_id.0.as_str());
        let within_limit = match (effect.limit, observation.usage) {
            (EffectMeasure::Calls(max), EffectMeasure::Calls(used))
            | (EffectMeasure::Items(max), EffectMeasure::Items(used)) => used <= max,
            (EffectMeasure::Bytes(max), EffectMeasure::Bytes(used)) => used <= max,
            _ => false,
        };
        let output_matches = (observation.status != ObservationStatus::Succeeded
            && observation.output == EffectOutput::None)
            || effect.expected_output == observation.output;
        let skipped_is_unexecuted = observation.status != ObservationStatus::Skipped
            || (observation.output == EffectOutput::None
                && matches!(
                    observation.usage,
                    EffectMeasure::Calls(0) | EffectMeasure::Items(0) | EffectMeasure::Bytes(0)
                ));
        if observation.kind != effect.kind || !within_limit || !output_matches || !skipped_is_unexecuted {
            contradicted = contradicted.saturating_add(1);
        } else if observation.status != ObservationStatus::Succeeded {
            failed = failed.saturating_add(1);
        }
    }
    let missing = u32::try_from(plan.effects.len().saturating_sub(seen.len())).unwrap_or(u32::MAX);
    if unknown > 0 || missing > 0 {
        return ApplicationOutcome::Rejected {
            unknown_effect_count: unknown,
            missing_effect_count: missing,
        };
    }
    if contradicted > 0 {
        return ApplicationOutcome::Contradicted {
            effect_count: contradicted,
        };
    }
    if failed > 0 {
        return ApplicationOutcome::Failed {
            failed_effect_count: failed,
        };
    }
    debug_assert_eq!(seen.len(), plan.effects.len());
    ApplicationOutcome::Completed
}

#[cfg(test)]
mod environment_tests {
    use super::*;

    #[test]
    fn observed_environment_reads_respect_kind_and_bounded_actual_usage() {
        let plan = plan_effects(CommandFamily::Bootstrap, &[EffectSpec {
            effect_id: "bootstrap-doctor-environment",
            kind: EffectKind::ReadEnvironment,
            limit: EffectMeasure::Calls(4),
            expected_output: ExpectedOutput::None,
        }])
        .unwrap();
        let observed = Observation {
            effect_id: EffectId(String::from("bootstrap-doctor-environment")),
            kind: EffectKind::ReadEnvironment,
            status: ObservationStatus::Succeeded,
            output: EffectOutput::None,
            usage: EffectMeasure::Calls(2),
            diagnostics_code: None,
        };
        assert_eq!(classify_observations(&plan, core::slice::from_ref(&observed)), ApplicationOutcome::Completed);
        let mut misclassified_file = observed.clone();
        misclassified_file.kind = EffectKind::ReadFiles;
        assert_eq!(classify_observations(&plan, &[misclassified_file]), ApplicationOutcome::Contradicted {
            effect_count: 1
        });
        let mut over_budget = observed;
        over_budget.usage = EffectMeasure::Calls(5);
        assert_eq!(classify_observations(&plan, &[over_budget]), ApplicationOutcome::Contradicted { effect_count: 1 });
    }

    #[test]
    fn environment_kind_accepts_its_public_serde_name() {
        let encoded = serde::de::value::StrDeserializer::<serde::de::value::Error>::new("read-environment");
        assert_eq!(EffectKind::deserialize(encoded), Ok(EffectKind::ReadEnvironment));
    }
}
