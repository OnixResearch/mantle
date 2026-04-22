mod adapter;
mod types;

pub use adapter::compute_activation;
pub use types::ActivationPlan;
pub use types::ExecMode;
pub use types::ExecTarget;
pub use types::HostEnv;
pub use types::ShellError;
pub use types::ShellSidecar;
pub use types::ShellWarning;
