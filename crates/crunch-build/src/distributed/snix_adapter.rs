//! Snix build-service compatibility adapter.
//!
//! Vendor build request/result and I/O types terminate in this module. The
//! remote core and application ports consume only Mantle-owned facts.

use super::RealizerProfileFacts;
use super::RemoteBuildFallbackPolicy;

pub(super) const REMOTE_BUILD_SERVICE_PHASE_REQUEST_VALIDATION: &str = "request-validation";
pub(super) const REMOTE_BUILD_SERVICE_PHASE_REMOTE_DISPATCH: &str = "remote-dispatch";

#[async_trait::async_trait]
pub trait DerivationRealizer: Send + Sync {
    fn profile(&self) -> RealizerProfileFacts;

    async fn realize(
        &self,
        request: snix_build::buildservice::BuildRequest,
    ) -> std::io::Result<snix_build::buildservice::BuildResult>;
}

#[derive(Debug, Clone)]
pub struct LocalBuildServiceRealizer<S> {
    service: S,
    profile: RealizerProfileFacts,
}

impl<S> LocalBuildServiceRealizer<S> {
    pub fn new(service: S, profile: RealizerProfileFacts) -> Self {
        Self { service, profile }
    }
}

#[async_trait::async_trait]
impl<S> DerivationRealizer for LocalBuildServiceRealizer<S>
where S: snix_build::buildservice::BuildService + Send + Sync
{
    fn profile(&self) -> RealizerProfileFacts {
        self.profile.clone()
    }

    async fn realize(
        &self,
        request: snix_build::buildservice::BuildRequest,
    ) -> std::io::Result<snix_build::buildservice::BuildResult> {
        self.service.do_build(request).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RemoteBuildServiceDispatchError {
    #[error("remote build service {phase} failed: {reason}")]
    Phase { phase: &'static str, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFailureDecision {
    ReturnFailure(RemoteBuildServiceDispatchError),
    FallbackToLocal { phase: &'static str, reason: String },
}

pub fn validate_remote_build_service_request(
    request: &snix_build::buildservice::BuildRequest,
) -> Result<(), RemoteBuildServiceDispatchError> {
    let command_arg_count = u32::try_from(request.command_args.len())
        .map_err(|_| request_validation_error("remote-build-service-command-arg-count-overflow"))?;
    let output_count = u32::try_from(request.outputs.len())
        .map_err(|_| request_validation_error("remote-build-service-output-count-overflow"))?;
    crunch_remote_core::validate_build_service_request(crunch_remote_core::RemoteBuildServiceRequestFacts {
        command_arg_count,
        output_count,
    })
    .map_err(|error| request_validation_error(error.code()))
}

fn request_validation_error(reason: &str) -> RemoteBuildServiceDispatchError {
    RemoteBuildServiceDispatchError::Phase {
        phase: REMOTE_BUILD_SERVICE_PHASE_REQUEST_VALIDATION,
        reason: reason.to_string(),
    }
}

pub fn classify_remote_build_service_failure(
    policy: RemoteBuildFallbackPolicy,
    phase: &'static str,
    reason: impl Into<String>,
) -> RemoteFailureDecision {
    let plan = crunch_remote_core::classify_remote_failure(policy, phase.to_string(), reason.into());
    match plan {
        crunch_remote_core::RemoteFailurePlan::ReturnFailure { reason, .. } => {
            RemoteFailureDecision::ReturnFailure(RemoteBuildServiceDispatchError::Phase { phase, reason })
        }
        crunch_remote_core::RemoteFailurePlan::FallbackToLocal { reason, .. } => {
            RemoteFailureDecision::FallbackToLocal { phase, reason }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoteBuildServiceAdapter<R> {
    remote: R,
}

impl<R> RemoteBuildServiceAdapter<R> {
    pub fn new(remote: R) -> Self {
        Self { remote }
    }
}

#[async_trait::async_trait]
impl<R> snix_build::buildservice::BuildService for RemoteBuildServiceAdapter<R>
where R: DerivationRealizer
{
    async fn do_build(
        &self,
        request: snix_build::buildservice::BuildRequest,
    ) -> std::io::Result<snix_build::buildservice::BuildResult> {
        validate_remote_build_service_request(&request).map_err(remote_dispatch_io_error)?;
        self.remote.realize(request).await.map_err(|source| {
            remote_dispatch_io_error(RemoteBuildServiceDispatchError::Phase {
                phase: REMOTE_BUILD_SERVICE_PHASE_REMOTE_DISPATCH,
                reason: source.to_string(),
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct RemoteFirstBuildService<R, L> {
    remote: R,
    local: L,
    fallback_policy: RemoteBuildFallbackPolicy,
}

impl<R, L> RemoteFirstBuildService<R, L> {
    pub fn new(remote: R, local: L, fallback_policy: RemoteBuildFallbackPolicy) -> Self {
        Self {
            remote,
            local,
            fallback_policy,
        }
    }
}

#[async_trait::async_trait]
impl<R, L> snix_build::buildservice::BuildService for RemoteFirstBuildService<R, L>
where
    R: DerivationRealizer,
    L: snix_build::buildservice::BuildService,
{
    async fn do_build(
        &self,
        request: snix_build::buildservice::BuildRequest,
    ) -> std::io::Result<snix_build::buildservice::BuildResult> {
        validate_remote_build_service_request(&request).map_err(remote_dispatch_io_error)?;
        match self.remote.realize(request.clone()).await {
            Ok(result) => Ok(result),
            Err(source) => match classify_remote_build_service_failure(
                self.fallback_policy,
                REMOTE_BUILD_SERVICE_PHASE_REMOTE_DISPATCH,
                source.to_string(),
            ) {
                RemoteFailureDecision::ReturnFailure(error) => Err(remote_dispatch_io_error(error)),
                RemoteFailureDecision::FallbackToLocal { .. } => self.local.do_build(request).await,
            },
        }
    }
}

fn remote_dispatch_io_error(error: RemoteBuildServiceDispatchError) -> std::io::Error {
    std::io::Error::other(error.to_string())
}
