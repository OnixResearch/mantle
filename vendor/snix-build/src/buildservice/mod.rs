use async_trait::async_trait;
use tokio::sync::watch;

pub mod build_request;
pub use crate::buildservice::build_request::*;
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
pub use crate::bwrap::watch_cancel::UnobservedWatchSandbox;
pub use dummy::DummyBuildService;
pub use from_addr::from_addr;

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
