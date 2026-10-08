#![no_std]
//! Bounded readiness decisions over caller-observed facts; no process, clock,
//! filesystem, transport, or evidence authority lives in this crate.
//!
//! A shell sets `request_acknowledged` only after an actual successful request.
//! For proof stages it verifies execution, output and evidence identities, and
//! stage-specific action/receipt authority before supplying completion facts.
//! Neither this pure core nor its JSON-shaped report authenticates those facts.

extern crate alloc;

mod admission;
mod evaluation;
mod model;

pub use evaluation::advance;
pub use evaluation::evaluate;
pub use evaluation::may_start;
pub use model::Component;
pub use model::ComponentKind;
pub use model::ErrorCode;
pub use model::ExitKind;
pub use model::Observation;
pub use model::ProofCompletion;
pub use model::ReadinessError;
pub use model::ReadinessReport;
pub use model::ReadinessRow;
pub use model::RestartAction;
pub use model::RestartPolicy;
pub use model::Snapshot;
pub use model::StageRequirements;

pub const READINESS_SCHEMA: &str = "mantle-service-readiness-v1";
pub const COORDINATION_CLASSIFICATION: &str = "coordination-state";
pub const MAX_COMPONENTS: u32 = 128;
pub const MAX_DEPENDENCIES_PER_COMPONENT: u32 = 128;
pub const MAX_USER_STATES: u32 = 32;
pub const MAX_OBSERVATION_STATES: u32 = 36;
pub const MAX_LABEL_BYTES: u32 = 64;

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests;
