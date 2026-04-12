//! DispatchBuildService: composite that routes fetch vs sandbox builds.
//!
//! Inspects `request.command_args[0]` and delegates to either
//! `FetchBuildService` (for `builtin:fetchurl`) or the underlying
//! sandbox `BuildService` (for everything else).
//!
//! The orchestrator constructs one `DispatchBuildService` and calls
//! `do_build()` on it. The fetch/sandbox split lives here, not in
//! `prepare_build()`.

use std::io;

use async_trait::async_trait;
use snix_build::buildservice::BuildRequest;
use snix_build::buildservice::BuildResult;
use snix_build::buildservice::BuildService;
use tracing::debug;

use crate::fetch_build_service::is_fetch_request;

/// Composite `BuildService` that routes between a fetch service and a
/// sandbox service based on the builder selector in the request.
///
/// ```text
/// DispatchBuildService
/// ├── F (FetchBuildService) — handles builtin:fetchurl
/// └── S (BubblewrapBuildService) — handles everything else
/// ```
pub struct DispatchBuildService<F, S> {
    fetch: F,
    sandbox: S,
}

impl<F, S> DispatchBuildService<F, S> {
    pub fn new(fetch: F, sandbox: S) -> Self {
        Self { fetch, sandbox }
    }
}

#[async_trait]
impl<F, S> BuildService for DispatchBuildService<F, S>
where
    F: BuildService + 'static,
    S: BuildService + 'static,
{
    async fn do_build(&self, request: BuildRequest) -> io::Result<BuildResult> {
        assert!(!request.command_args.is_empty(), "BuildRequest must have at least one command arg (the builder)");

        if is_fetch_request(&request) {
            let builder = request.command_args.first().cloned().unwrap_or_default();
            debug!(builder = %builder, "dispatching to fetch service");
            self.fetch.do_build(request).await
        } else {
            let builder = request.command_args.first().cloned().unwrap_or_default();
            debug!(builder = %builder, "dispatching to sandbox service");
            self.sandbox.do_build(request).await
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering;

    use bytes::Bytes;
    use snix_build::buildservice::BuildOutput;
    use snix_build::buildservice::EnvVar;
    use snix_castore::Node;

    use super::*;
    use crate::fetch_build_service::FETCH_BUILDER;

    /// Counting mock that records how many times `do_build` is called.
    struct CountingService {
        name: &'static str,
        call_count: Arc<AtomicU32>,
    }

    impl CountingService {
        fn new(name: &'static str) -> (Self, Arc<AtomicU32>) {
            let count = Arc::new(AtomicU32::new(0));
            (
                Self {
                    name,
                    call_count: count.clone(),
                },
                count,
            )
        }
    }

    #[async_trait]
    impl BuildService for CountingService {
        async fn do_build(&self, _request: BuildRequest) -> io::Result<BuildResult> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            Ok(BuildResult {
                outputs: vec![BuildOutput {
                    node: Node::Symlink {
                        target: snix_castore::SymlinkTarget::try_from(self.name).unwrap(),
                    },
                    output_needles: BTreeSet::new(),
                }],
                log: Some(format!("handled by {}", self.name)),
            })
        }
    }

    fn env(key: &str, value: &str) -> EnvVar {
        EnvVar {
            key: key.to_string(),
            value: Bytes::from(value.as_bytes().to_vec()),
        }
    }

    fn fetch_request() -> BuildRequest {
        BuildRequest {
            command_args: vec![FETCH_BUILDER.to_string()],
            outputs: vec![PathBuf::from("nix/store/abc-test")],
            environment_vars: vec![env("url", "https://example.com/f.txt")],
            ..BuildRequest::default()
        }
    }

    fn sandbox_request() -> BuildRequest {
        BuildRequest {
            command_args: vec!["/bin/sh".to_string(), "-c".to_string()],
            outputs: vec![PathBuf::from("nix/store/abc-test")],
            ..BuildRequest::default()
        }
    }

    #[tokio::test]
    async fn dispatch_routes_fetch_to_fetch_service() {
        let (fetch_svc, fetch_count) = CountingService::new("fetch");
        let (sandbox_svc, sandbox_count) = CountingService::new("sandbox");
        let dispatch = DispatchBuildService::new(fetch_svc, sandbox_svc);

        let result = dispatch.do_build(fetch_request()).await.unwrap();

        assert_eq!(fetch_count.load(Ordering::Relaxed), 1);
        assert_eq!(sandbox_count.load(Ordering::Relaxed), 0);
        assert_eq!(result.log.as_deref(), Some("handled by fetch"));
    }

    #[tokio::test]
    async fn dispatch_routes_sandbox_to_sandbox_service() {
        let (fetch_svc, fetch_count) = CountingService::new("fetch");
        let (sandbox_svc, sandbox_count) = CountingService::new("sandbox");
        let dispatch = DispatchBuildService::new(fetch_svc, sandbox_svc);

        let result = dispatch.do_build(sandbox_request()).await.unwrap();

        assert_eq!(fetch_count.load(Ordering::Relaxed), 0);
        assert_eq!(sandbox_count.load(Ordering::Relaxed), 1);
        assert_eq!(result.log.as_deref(), Some("handled by sandbox"));
    }

    #[tokio::test]
    async fn dispatch_never_sends_fetch_to_sandbox() {
        let (fetch_svc, _) = CountingService::new("fetch");
        let (sandbox_svc, sandbox_count) = CountingService::new("sandbox");
        let dispatch = DispatchBuildService::new(fetch_svc, sandbox_svc);

        // Send 3 fetch requests.
        for _ in 0u32..3 {
            let _ = dispatch.do_build(fetch_request()).await;
        }

        assert_eq!(sandbox_count.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn dispatch_never_sends_sandbox_to_fetch() {
        let (fetch_svc, fetch_count) = CountingService::new("fetch");
        let (sandbox_svc, _) = CountingService::new("sandbox");
        let dispatch = DispatchBuildService::new(fetch_svc, sandbox_svc);

        // Send 3 sandbox requests.
        for _ in 0u32..3 {
            let _ = dispatch.do_build(sandbox_request()).await;
        }

        assert_eq!(fetch_count.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn dispatch_propagates_fetch_error() {
        /// Always-failing fetch service.
        struct FailingFetch;
        #[async_trait]
        impl BuildService for FailingFetch {
            async fn do_build(&self, _request: BuildRequest) -> io::Result<BuildResult> {
                Err(io::Error::other("fetch failed"))
            }
        }
        let (sandbox_svc, _) = CountingService::new("sandbox");
        let dispatch = DispatchBuildService::new(FailingFetch, sandbox_svc);

        let err = dispatch.do_build(fetch_request()).await.unwrap_err();
        assert!(err.to_string().contains("fetch failed"));
    }

    #[tokio::test]
    async fn dispatch_propagates_sandbox_error() {
        /// Always-failing sandbox service.
        struct FailingSandbox;
        #[async_trait]
        impl BuildService for FailingSandbox {
            async fn do_build(&self, _request: BuildRequest) -> io::Result<BuildResult> {
                Err(io::Error::other("sandbox failed"))
            }
        }
        let (fetch_svc, _) = CountingService::new("fetch");
        let dispatch = DispatchBuildService::new(fetch_svc, FailingSandbox);

        let err = dispatch.do_build(sandbox_request()).await.unwrap_err();
        assert!(err.to_string().contains("sandbox failed"));
    }
}
