/// A reconnect decision is not a transport connection or authorization grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconnectDecision {
    SameSession,
    NewSession,
    BackoffRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinatorJobPhase {
    Queued,
    Running,
    Finished,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseReleaseDecision {
    Retain,
    Release,
}

/// A same-session reconnect takes precedence even when retry attempts are
/// exhausted. A different session can be selected only below the limit.
// r[impl remote_builds.hexagonal_core]
pub fn decide_reconnect(
    active_session_id: &str,
    candidate_session_id: &str,
    attempts: u32,
    max_attempts: u32,
) -> ReconnectDecision {
    if active_session_id == candidate_session_id {
        return ReconnectDecision::SameSession;
    }
    if attempts >= max_attempts {
        return ReconnectDecision::BackoffRequired;
    }
    ReconnectDecision::NewSession
}

/// A lease is retained while its job may still produce output; a terminal
/// phase plans release, never an observed release or a successful write.
// r[impl remote_builds.hexagonal_core]
pub const fn decide_lease_release(phase: CoordinatorJobPhase) -> LeaseReleaseDecision {
    match phase {
        CoordinatorJobPhase::Queued | CoordinatorJobPhase::Running => LeaseReleaseDecision::Retain,
        CoordinatorJobPhase::Finished | CoordinatorJobPhase::Lost => LeaseReleaseDecision::Release,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_session_precedes_exhausted_reconnect_attempts() {
        assert_eq!(decide_reconnect("session-a", "session-a", 3, 3), ReconnectDecision::SameSession);
        assert_eq!(decide_reconnect("session-a", "session-b", 3, 3), ReconnectDecision::BackoffRequired);
        assert_eq!(decide_reconnect("session-a", "session-b", 2, 3), ReconnectDecision::NewSession);
        assert_eq!(decide_reconnect("session-a", "session-b", 0, 0), ReconnectDecision::BackoffRequired);
    }

    #[test]
    fn only_terminal_jobs_plan_lease_release() {
        for phase in [CoordinatorJobPhase::Queued, CoordinatorJobPhase::Running] {
            assert_eq!(decide_lease_release(phase), LeaseReleaseDecision::Retain);
        }
        for phase in [CoordinatorJobPhase::Finished, CoordinatorJobPhase::Lost] {
            assert_eq!(decide_lease_release(phase), LeaseReleaseDecision::Release);
        }
    }
}
