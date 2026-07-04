#![no_std]
extern crate alloc;

mod error;
mod plan;
mod profile;
mod types;

pub use error::ShellError;
pub use plan::compute_activation;
pub use profile::DEV_SHELL_DECOUPLING_NON_CLAIM;
pub use profile::NAMED_SHELL_PROFILE_NON_CLAIM;
pub use profile::NamedShellProfiles;
pub use profile::ShellProfileDeclaration;
pub use profile::ShellProfileDiagnostic;
pub use profile::ShellProfileEnvEntry;
pub use profile::ShellProfilePlan;
pub use profile::ShellProfileSelectionRequest;
pub use profile::plan_named_shell_profile;
pub use types::ActivationPlan;
pub use types::HostEnv;
pub use types::ShellSidecar;
pub use types::ShellWarning;
pub use types::parse_shell_sidecar_json;
