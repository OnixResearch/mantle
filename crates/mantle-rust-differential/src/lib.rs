//! Ordinary, offline Cargo differential evidence. This is not a build authority.
#[cfg(not(target_os = "linux"))]
compile_error!("the differential runner requires Linux process containment and /proc");

pub mod cohorts;
pub mod fixtures;
pub mod negative;
pub mod process;
pub mod runner;
pub mod serial;

pub use runner::Options;
pub use runner::Report;
pub use runner::execute;
pub type Result<T, E = String> = std::result::Result<T, E>;
pub(crate) fn err(value: impl std::fmt::Display) -> String {
    value.to_string()
}
