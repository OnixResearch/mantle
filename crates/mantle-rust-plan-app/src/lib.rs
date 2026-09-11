#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Rust-plan application contract: ports, typed capability failures, and the
//! orchestration that runs a plan through them.
//!
//! Ports own external capability (workspace facts, Cargo oracle capture,
//! compiler inspection, unit execution, Rust cache access). They return
//! Mantle-owned typed values only: no `RunError`, process types, Cargo JSON,
//! host paths, or raw store values cross this boundary. The shell implements
//! the ports; this crate never performs I/O.

extern crate alloc;

#[cfg(test)]
extern crate std;

mod application;
mod ports;

pub use application::ApplicationOutcome;
pub use application::CacheDisposition;
pub use application::ExecutedPlan;
pub use application::RustPlanApplication;
pub use application::UnitDisposition;
pub use ports::AdapterError;
pub use ports::CacheLookup;
pub use ports::CacheLookupRequest;
pub use ports::CargoOracleCapture;
pub use ports::CompilerFacts;
pub use ports::CompilerInspection;
pub use ports::CompilerInspectionRequest;
pub use ports::OracleCaptureRequest;
pub use ports::OracleFacts;
pub use ports::RustCacheAccess;
pub use ports::UnitExecutor;
pub use ports::WorkspaceFactsRequest;
pub use ports::WorkspaceFactsSource;
pub use ports::WorkspaceFactsView;
