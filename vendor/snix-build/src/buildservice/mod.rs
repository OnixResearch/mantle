use async_trait::async_trait;

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
pub use dummy::DummyBuildService;
pub use from_addr::from_addr;

#[async_trait]
pub trait BuildService: Send + Sync {
    /// TODO: document
    async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult>;
}
