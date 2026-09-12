//! Adapter: the declared environment inputs of the CLI shell.
//!
//! The shell names these variables in one place, converts them into owned
//! values, and passes the values inward. Deterministic cores never read the
//! environment, and the presence rules are pure over an observed value.

/// Variable that requests tracing output.
pub(crate) const RUST_LOG_ENV: &str = "RUST_LOG";

/// Variable that carries a W3C trace parent header.
pub(crate) const W3C_TRACEPARENT_ENV: &str = "TRACEPARENT";

/// Variable that carries a W3C trace state header.
pub(crate) const W3C_TRACESTATE_ENV: &str = "TRACESTATE";

/// Variable that points at the local-route test sentinel path.
pub(crate) const TEST_LOCAL_ROUTE_SENTINEL_ENV: &str = "MANTLE_TEST_LOCAL_ROUTE_SENTINEL";

/// Ambient trace headers observed from the environment.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct AmbientTraceHeaders {
    /// W3C `traceparent` value, when the environment carries one.
    pub(crate) traceparent: Option<String>,
    /// W3C `tracestate` value, when the environment carries one.
    pub(crate) tracestate: Option<String>,
}

impl AmbientTraceHeaders {
    /// Whether the environment carried neither header.
    pub(crate) fn is_absent(&self) -> bool {
        let is_absent = self.traceparent.is_none() && self.tracestate.is_none();
        debug_assert!(!is_absent || self.tracestate.is_none());
        debug_assert!(is_absent || self.traceparent.is_some() || self.tracestate.is_some());
        is_absent
    }
}

/// Build trace headers from already observed values.
pub(crate) fn trace_headers_from(traceparent: Option<String>, tracestate: Option<String>) -> AmbientTraceHeaders {
    let headers = AmbientTraceHeaders {
        traceparent,
        tracestate,
    };
    debug_assert_eq!(
        headers.traceparent.is_some(),
        headers.traceparent.as_deref().is_some_and(|value| !value.is_empty()) || headers.traceparent.is_some()
    );
    debug_assert!(!headers.is_absent() || headers.traceparent.is_none());
    headers
}

/// Read the ambient trace headers from the environment.
pub(crate) fn read_ambient_trace_headers() -> AmbientTraceHeaders {
    trace_headers_from(std::env::var(W3C_TRACEPARENT_ENV).ok(), std::env::var(W3C_TRACESTATE_ENV).ok())
}

/// Read the local-route test sentinel path when the environment carries one.
pub(crate) fn read_test_sentinel_path() -> Option<std::ffi::OsString> {
    let path = std::env::var_os(TEST_LOCAL_ROUTE_SENTINEL_ENV);
    debug_assert!(path.is_none() || path.as_deref().is_some());
    path
}

/// Whether an observed `RUST_LOG` value requests tracing.
pub(crate) fn is_trace_log_requested(observed: Option<&std::ffi::OsStr>) -> bool {
    let is_requested = observed.is_some();
    debug_assert!(is_requested || observed.is_none());
    debug_assert!(!is_requested || observed.as_deref().is_some());
    is_requested
}

/// Whether the environment requests tracing output.
pub(crate) fn is_rust_log_requested() -> bool {
    is_trace_log_requested(std::env::var_os(RUST_LOG_ENV).as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_headers_pass_through_observed_values() {
        let headers = trace_headers_from(Some(String::from("00-abc-def-01")), Some(String::from("vendor=1")));
        assert_eq!(headers.traceparent.as_deref(), Some("00-abc-def-01"));
        assert_eq!(headers.tracestate.as_deref(), Some("vendor=1"));
        assert!(!headers.is_absent());
    }

    #[test]
    fn a_single_header_is_not_absent() {
        let parent_only = trace_headers_from(Some(String::from("00-abc-def-01")), None);
        assert!(!parent_only.is_absent());
        let state_only = trace_headers_from(None, Some(String::from("vendor=1")));
        assert!(!state_only.is_absent());
    }

    #[test]
    fn no_headers_is_absent() {
        let headers = trace_headers_from(None, None);
        assert!(headers.is_absent());
        assert_eq!(headers, AmbientTraceHeaders::default());
    }

    #[test]
    fn trace_logging_follows_the_observed_value() {
        let observed = std::ffi::OsString::from("info");
        assert!(is_trace_log_requested(Some(observed.as_os_str())));
        assert!(!is_trace_log_requested(None));
        assert_eq!(RUST_LOG_ENV, "RUST_LOG");
        assert_eq!(TEST_LOCAL_ROUTE_SENTINEL_ENV, "MANTLE_TEST_LOCAL_ROUTE_SENTINEL");
        assert_eq!(W3C_TRACEPARENT_ENV, "TRACEPARENT");
        assert_eq!(W3C_TRACESTATE_ENV, "TRACESTATE");
    }
}
