#![no_std]
extern crate alloc;

mod error;
mod plan;
mod types;

pub use error::ShellError;
pub use plan::compute_activation;
pub use types::ActivationPlan;
pub use types::HostEnv;
pub use types::ShellSidecar;
pub use types::ShellWarning;
pub use types::parse_shell_sidecar_json;
