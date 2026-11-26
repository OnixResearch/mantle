use async_trait::async_trait;
use tokio::sync::watch;

pub mod build_request;
pub use crate::buildservice::build_request::*;
pub mod policy;
pub use crate::buildservice::policy::*;
mod dummy;
mod ephemeral_dir;
mod from_addr;

#[cfg(target_os = "linux")]
mod oci;

#[cfg(target_os = "linux")]
mod bwrap;

#[cfg(target_os = "linux")]
pub use bwrap::BubblewrapBuildService;
#[cfg(target_os = "linux")]
pub use bwrap::DEFAULT_CONCURRENT_SANDBOX_BUILDS;
#[cfg(target_os = "linux")]
pub use bwrap::DEFAULT_SANDBOX_FD_BUDGET;
#[cfg(target_os = "linux")]
pub use bwrap::ESTIMATED_FDS_PER_SANDBOX;
#[cfg(target_os = "linux")]
pub use bwrap::MAX_CONCURRENT_SANDBOX_BUILDS;
#[cfg(target_os = "linux")]
pub use bwrap::SandboxConcurrencyPolicy;
pub use dummy::DummyBuildService;
pub use from_addr::from_addr;

#[cfg(target_os = "linux")]
pub use crate::bwrap::watch_cancel::UnobservedWatchSandbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxFailureClass {
    SandboxStartup,
    MissingOutputs,
    Builder,
    WallDeadlineExceeded,
    SilenceBudgetExceeded,
    ResourceBudgetExceeded,
    ResourceEnforcementUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedBuildServiceError {
    pub class: SandboxFailureClass,
    pub message: String,
}

impl std::fmt::Display for ClassifiedBuildServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ClassifiedBuildServiceError {}

pub fn classified_io_error(
    kind: std::io::ErrorKind,
    class: SandboxFailureClass,
    message: impl Into<String>,
) -> std::io::Error {
    std::io::Error::new(kind, ClassifiedBuildServiceError {
        class,
        message: message.into(),
    })
}

pub fn wrap_classified_io_error(error: std::io::Error, class: SandboxFailureClass) -> std::io::Error {
    if error.get_ref().is_some_and(|inner| inner.is::<ClassifiedBuildServiceError>()) {
        return error;
    }
    let kind = error.kind();
    classified_io_error(kind, class, error.to_string())
}

#[async_trait]
pub trait BuildService: Send + Sync {
    /// TODO: document
    async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult>;

    /// Watch-only local cancellation, never an implicit abort of `do_build`.
    /// Unsupported services fail closed instead of pretending process teardown.
    async fn do_build_cancellable(
        &self,
        _request: BuildRequest,
        _cancellation: watch::Receiver<bool>,
    ) -> std::io::Result<BuildResult> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "build service cannot prove watch sandbox teardown",
        ))
    }
}
