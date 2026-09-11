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
}

/// Stable effect identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EffectId(pub String);

/// One planned effect with its declared capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effect {
    pub effect_id: EffectId,
    pub kind: EffectKind,
    /// Family that owns the effect.
    pub family: CommandFamily,
}

/// Ordered bounded effect plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectPlan {
    pub effects: Vec<Effect>,
}

/// Build a bounded plan; more effects than the bound yields `None`.
pub fn plan_effects(family: CommandFamily, kinds: &[&str]) -> Option<EffectPlan> {
    let Ok(requested_effect_count) = u32::try_from(kinds.len()) else {
        return None;
    };
    if requested_effect_count > MAX_EFFECTS_PER_PLAN {
        return None;
    }
    let effects: Vec<Effect> = kinds
        .iter()
        .map(|kind| Effect {
            effect_id: EffectId(String::from(*kind)),
            kind: match *kind {
                "read-files" => EffectKind::ReadFiles,
                "write-files" => EffectKind::WriteFiles,
                "run-process" => EffectKind::RunProcess,
                "use-network" => EffectKind::UseNetwork,
                "read-clock" => EffectKind::ReadClock,
                _ => EffectKind::ReadRandom,
            },
            family,
        })
        .collect();
    debug_assert!(effects.len() == kinds.len());
    Some(EffectPlan { effects })
}

/// Observation status reported by an adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservationStatus {
    Succeeded,
    Failed,
    Skipped,
}

/// One typed observation for a planned effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub effect_id: EffectId,
    pub status: ObservationStatus,
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
}

/// Classify observations against a plan.
///
/// Unknown, duplicate, or missing effect identities reject; failed or skipped
/// effects count as failures; otherwise the run completes.
pub fn classify_observations(plan: &EffectPlan, observations: &[Observation]) -> ApplicationOutcome {
    let mut seen: Vec<&str> = Vec::with_capacity(plan.effects.len());
    let mut unknown: u32 = 0;
    let mut failed: u32 = 0;
    for observation in observations {
        let is_planned = plan.effects.iter().any(|effect| effect.effect_id == observation.effect_id);
        if !is_planned || seen.contains(&observation.effect_id.0.as_str()) {
            unknown = unknown.saturating_add(1);
            continue;
        }
        seen.push(observation.effect_id.0.as_str());
        if observation.status != ObservationStatus::Succeeded {
            failed = failed.saturating_add(1);
        }
    }
    let Ok(planned_count) = u32::try_from(plan.effects.len()) else {
        // A plan wider than the observation domain cannot be classified.
        return ApplicationOutcome::Rejected {
            unknown_effect_count: 0,
            missing_effect_count: 0,
        };
    };
    let Ok(seen_count) = u32::try_from(seen.len()) else {
        return ApplicationOutcome::Rejected {
            unknown_effect_count: 0,
            missing_effect_count: 0,
        };
    };
    let missing = planned_count.saturating_sub(seen_count);
    if unknown > 0 || missing > 0 {
        return ApplicationOutcome::Rejected {
            unknown_effect_count: unknown,
            missing_effect_count: missing,
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
