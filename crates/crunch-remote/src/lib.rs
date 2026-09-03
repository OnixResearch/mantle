//! Remote-build application shell and capability ports.
//!
//! Ports use only Mantle-owned contracts from `crunch-remote-core`. Concrete
//! transports, stores, executors, credentials, clocks, identifiers, and
//! telemetry implementations live in outer adapters.

// r[impl remote_builds.application_owned_ports]
// r[impl remote_builds.remote_effect_plans]
mod application;
mod ports;

pub use application::Failure;
pub use application::execute_effect;
pub use application::port_failure;
pub use application::run;
pub use crunch_remote_core::*;
pub use ports::*;
