//! Admission-fact derivation and the local-route test sentinel decision.
//!
//! The composition root observes CLI variants and, in debug builds, one
//! environment path. Both decisions are pure over those observations, so they
//! live here where they can be tested rather than inline in the root.

use alloc::string::String;

/// Marker text written when the local-route test sentinel is active.
pub const TEST_SENTINEL_MARKER_TEXT: &str = "local-route-entered";

/// The CLI observations that decide portable-admission facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteAdmissionRequest {
    /// Whether the observed command is the build root.
    pub is_build_command: bool,
    /// Whether a builder program or endpoint was selected.
    pub has_builder_selection: bool,
    /// Whether the operator asked for a plan instead of a build.
    pub is_plan_requested: bool,
    /// Whether the observed command is the remote root.
    pub is_remote_command: bool,
    /// Whether the remote action is the serve action.
    pub is_serve_action: bool,
}

/// Portable-admission facts derived from the observed command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteAdmissionFacts {
    /// Whether the command selects a remote route.
    pub is_remote_route_selected: bool,
    /// Whether the remote command acts as a client.
    pub is_remote_operation_client: bool,
}

/// Derive the portable-admission facts from the observed CLI facts.
///
/// A build selects a remote route only when a builder was chosen or a plan was
/// requested, and a remote command acts as a client unless it serves.
pub fn remote_admission_facts(request: &RemoteAdmissionRequest) -> RemoteAdmissionFacts {
    let is_remote_route_selected =
        request.is_build_command && (request.has_builder_selection || request.is_plan_requested);
    let is_remote_operation_client = request.is_remote_command && !request.is_serve_action;
    let facts = RemoteAdmissionFacts {
        is_remote_route_selected,
        is_remote_operation_client,
    };
    debug_assert!(facts.is_remote_route_selected == is_remote_route_selected);
    debug_assert!(!facts.is_remote_route_selected || request.is_build_command);
    facts
}

/// The test-sentinel observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestSentinelFacts {
    /// Whether this build has debug assertions enabled.
    pub is_debug_build: bool,
    /// Whether a sentinel path was supplied through the environment.
    pub has_sentinel_path: bool,
}

/// The local-route test sentinel decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestSentinelDecision {
    /// Whether the sentinel marker should be written.
    pub is_active: bool,
    /// Marker text to write when active.
    pub marker_text: String,
}

/// Decide whether the local-route test sentinel is active.
///
/// The sentinel is never active in a release build, so a shipped binary cannot
/// write a marker even when the environment variable is present.
pub fn test_sentinel_decision(facts: &TestSentinelFacts) -> TestSentinelDecision {
    let is_active = facts.is_debug_build && facts.has_sentinel_path;
    let decision = TestSentinelDecision {
        is_active,
        marker_text: String::from(TEST_SENTINEL_MARKER_TEXT),
    };
    debug_assert!(!decision.is_active || facts.is_debug_build);
    debug_assert!(!decision.is_active || facts.has_sentinel_path);
    decision
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(
        is_build: bool,
        has_builder: bool,
        is_plan: bool,
        is_remote: bool,
        is_serve: bool,
    ) -> RemoteAdmissionRequest {
        RemoteAdmissionRequest {
            is_build_command: is_build,
            has_builder_selection: has_builder,
            is_plan_requested: is_plan,
            is_remote_command: is_remote,
            is_serve_action: is_serve,
        }
    }

    #[test]
    fn a_build_with_a_builder_selects_a_remote_route() {
        let facts = remote_admission_facts(&request(true, true, false, false, false));
        assert!(facts.is_remote_route_selected);
        assert!(!facts.is_remote_operation_client);
    }

    #[test]
    fn a_plain_build_or_another_command_selects_no_remote_route() {
        assert!(!remote_admission_facts(&request(true, false, false, false, false)).is_remote_route_selected);
        assert!(!remote_admission_facts(&request(false, true, true, false, false)).is_remote_route_selected);
        assert!(remote_admission_facts(&request(true, false, true, false, false)).is_remote_route_selected);
    }

    #[test]
    fn a_remote_client_operation_is_not_the_serve_action() {
        assert!(remote_admission_facts(&request(false, false, false, true, false)).is_remote_operation_client);
        assert!(!remote_admission_facts(&request(false, false, false, true, true)).is_remote_operation_client);
        assert!(!remote_admission_facts(&request(true, true, false, false, false)).is_remote_operation_client);
    }

    #[test]
    fn the_test_sentinel_needs_a_debug_build_and_a_path() {
        let active = test_sentinel_decision(&TestSentinelFacts {
            is_debug_build: true,
            has_sentinel_path: true,
        });
        assert!(active.is_active);
        assert_eq!(active.marker_text, TEST_SENTINEL_MARKER_TEXT);
        let release = test_sentinel_decision(&TestSentinelFacts {
            is_debug_build: false,
            has_sentinel_path: true,
        });
        assert!(!release.is_active);
        let no_path = test_sentinel_decision(&TestSentinelFacts {
            is_debug_build: true,
            has_sentinel_path: false,
        });
        assert!(!no_path.is_active);
        assert_eq!(no_path.marker_text.len(), TEST_SENTINEL_MARKER_TEXT.len());
    }
}
