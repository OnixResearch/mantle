#![no_std]

extern crate alloc;

mod identity;
mod ledger;
mod projection;
mod types;

pub use ledger::OutcomeLedger;
pub use projection::ProcessMode;
pub use projection::ProcessOutcome;
pub use projection::RunStartValue;
pub use projection::StreamRecordValue;
pub use projection::process_status;
pub use types::BoundedDiagnostic;
pub use types::DispatchDecision;
pub use types::FailureFact;
pub use types::FailureScope;
pub use types::IdentityContext;
pub use types::OutcomeError;
pub use types::RootCounts;
pub use types::RootOutcome;
pub use types::RootReferences;
pub use types::RootSet;
pub use types::RunDisposition;
pub use types::RunSummary;
pub use types::SelectedRoot;
pub use types::SourceSequence;
pub use types::TerminalState;
pub use types::TransitionResult;

pub const STREAM_SCHEMA: &str = "mantle-evaluation-stream-v1";
pub const ROOTS_MAX: u32 = 65_536;
pub const ROOT_LABEL_BYTES_MAX: u32 = 1_024;
pub const CONTEXT_FIELD_BYTES_MAX: u32 = 4_096;
pub const DIAGNOSTIC_BYTES_MAX: u32 = 16_384;
pub const REFERENCE_BYTES_MAX: u32 = 4_096;

pub(crate) const ROOTS_MAX_USIZE: usize = 65_536;
pub(crate) const ROOT_LABEL_BYTES_MAX_USIZE: usize = 1_024;
pub(crate) const CONTEXT_FIELD_BYTES_MAX_USIZE: usize = 4_096;
pub(crate) const DIAGNOSTIC_BYTES_MAX_USIZE: usize = 16_384;
pub(crate) const REFERENCE_BYTES_MAX_USIZE: usize = 4_096;
pub const SUCCESS_EXIT_CODE: u8 = 0;
pub const PIPELINE_NON_SUCCESS_EXIT_CODE: u8 = 1;
pub const EVALUATION_NON_SUCCESS_EXIT_CODE: u8 = 2;
pub const INTERNAL_EXIT_CODE: u8 = 3;
pub const CANCELLED_EXIT_CODE: u8 = 130;

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests;
