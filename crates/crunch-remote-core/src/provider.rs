/// Request facts after an adapter translates the vendor's build request.
/// Counts alone do not prove the requested outputs exist in the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildServiceRequestFacts {
    pub command_arg_count: usize,
    pub output_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildServiceBlocker {
    RawEvalRequest,
    OutputsEmpty,
}

impl BuildServiceBlocker {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RawEvalRequest => "remote-build-service-raw-eval-request",
            Self::OutputsEmpty => "remote-build-service-outputs-empty",
        }
    }
}

/// Reject raw frontend evaluation before asking any remote provider to run.
// r[impl remote_builds.hexagonal_core]
pub const fn admit_build_service_request(facts: BuildServiceRequestFacts) -> Result<(), BuildServiceBlocker> {
    if facts.command_arg_count == 0 {
        return Err(BuildServiceBlocker::RawEvalRequest);
    }
    if facts.output_count == 0 {
        return Err(BuildServiceBlocker::OutputsEmpty);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchFallbackPolicy {
    Never,
    OnRemoteFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchFailureDecision {
    ReturnFailure,
    FallbackToLocal,
}

/// Fallback is authorized only for a remote dispatch failure; request
/// validation runs separately and always fails closed before dispatch.
// r[impl remote_builds.hexagonal_core]
pub const fn classify_dispatch_failure(policy: DispatchFallbackPolicy) -> DispatchFailureDecision {
    match policy {
        DispatchFallbackPolicy::Never => DispatchFailureDecision::ReturnFailure,
        DispatchFallbackPolicy::OnRemoteFailure => DispatchFailureDecision::FallbackToLocal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incomplete_build_request_never_reaches_remote_dispatch() {
        assert_eq!(
            admit_build_service_request(BuildServiceRequestFacts {
                command_arg_count: 0,
                output_count: 0
            }),
            Err(BuildServiceBlocker::RawEvalRequest)
        );
        assert_eq!(
            admit_build_service_request(BuildServiceRequestFacts {
                command_arg_count: 1,
                output_count: 0
            }),
            Err(BuildServiceBlocker::OutputsEmpty)
        );
        assert_eq!(
            admit_build_service_request(BuildServiceRequestFacts {
                command_arg_count: 1,
                output_count: 1
            }),
            Ok(())
        );
    }

    #[test]
    fn untrusted_remote_failure_cannot_enable_implicit_fallback() {
        assert_eq!(classify_dispatch_failure(DispatchFallbackPolicy::Never), DispatchFailureDecision::ReturnFailure);
        assert_eq!(
            classify_dispatch_failure(DispatchFallbackPolicy::OnRemoteFailure),
            DispatchFailureDecision::FallbackToLocal
        );
    }
}
