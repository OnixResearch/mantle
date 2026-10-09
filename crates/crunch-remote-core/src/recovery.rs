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

#[derive(Debug, Clone, Copy)]
pub struct ReconnectFacts<'a> {
    pub active_session_id: &'a str,
    pub candidate_session_id: &'a str,
    pub attempts: u32,
    pub max_attempts: u32,
}

/// A same-session reconnect takes precedence even when retry attempts are
/// exhausted. A different session can be selected only below the limit.
// r[impl remote_builds.hexagonal_core]
pub fn decide_reconnect(facts: ReconnectFacts<'_>) -> ReconnectDecision {
    if facts.active_session_id == facts.candidate_session_id {
        return ReconnectDecision::SameSession;
    }
    if facts.attempts >= facts.max_attempts {
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
        let same_session = ReconnectFacts {
            active_session_id: "session-a",
            candidate_session_id: "session-a",
            attempts: 3,
            max_attempts: 3,
        };
        assert_eq!(decide_reconnect(same_session), ReconnectDecision::SameSession);
        let different_session = ReconnectFacts {
            candidate_session_id: "session-b",
            ..same_session
        };
        assert_eq!(decide_reconnect(different_session), ReconnectDecision::BackoffRequired);
        assert_eq!(
            decide_reconnect(ReconnectFacts {
                attempts: 2,
                ..different_session
            }),
            ReconnectDecision::NewSession
        );
        assert_eq!(
            decide_reconnect(ReconnectFacts {
                attempts: 0,
                max_attempts: 0,
                ..different_session
            }),
            ReconnectDecision::BackoffRequired
        );
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
