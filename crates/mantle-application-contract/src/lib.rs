#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Application-architecture contract for the Mantle CLI composition root.
//!
//! This crate owns the typed vocabulary the root dispatches with: command
//! families, application commands, blockers, effect plans, observations, and
//! typed capability outcomes. Domain policy lives in the owning cores; the
//! root keeps parsing, adapter selection, dispatch, and presentation.

extern crate alloc;

#[cfg(test)]
extern crate std;

mod envelope;
mod family;
mod realization;

pub use envelope::ApplicationBlocker;
pub use envelope::ApplicationOutcome;
pub use envelope::CapabilityError;
pub use envelope::Effect;
pub use envelope::EffectId;
pub use envelope::EffectKind;
pub use envelope::EffectPlan;
pub use envelope::MAX_EFFECTS_PER_PLAN;
pub use envelope::Observation;
pub use envelope::ObservationStatus;
pub use envelope::classify_observations;
pub use envelope::plan_effects;
pub use family::CommandFamily;
pub use family::MAX_COMMAND_ROOTS;
pub use realization::MAX_REALIZATION_ROOTS;
pub use realization::RealizationBlocker;
pub use realization::RealizeCommand;
pub use realization::RealizeOutcome;
pub use realization::RealizePort;
pub use realization::RealizeResult;
pub use realization::validate_realize_command;
