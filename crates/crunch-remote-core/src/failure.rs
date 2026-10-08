/// Admitted remote failure phase. A reason is owned by the boundary that
/// observed it, rather than interpreted as evidence of an effect succeeding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailurePhase {
    TransportSetup,
    Authentication,
    RequestValidation,
    InputSync,
    Queue,
    BuildExecution,
    OutputImport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryClass {
    Retryable,
    BuildOutcome,
    Terminal,
}

/// Accepted classification; the shell decides whether another attempt is
/// authorized after applying its separate fenced-attempt retry policy.
// r[impl remote_builds.hexagonal_core]
pub const fn classify_failure(phase: FailurePhase) -> RetryClass {
    match phase {
        FailurePhase::TransportSetup => RetryClass::Retryable,
        FailurePhase::BuildExecution => RetryClass::BuildOutcome,
        FailurePhase::Authentication
        | FailurePhase::RequestValidation
        | FailurePhase::InputSync
        | FailurePhase::Queue
        | FailurePhase::OutputImport => RetryClass::Terminal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transient_transport_and_terminal_trust_failures_are_distinct() {
        assert_eq!(classify_failure(FailurePhase::TransportSetup), RetryClass::Retryable);
        assert_eq!(classify_failure(FailurePhase::BuildExecution), RetryClass::BuildOutcome);
        for phase in [
            FailurePhase::Authentication,
            FailurePhase::RequestValidation,
            FailurePhase::InputSync,
            FailurePhase::Queue,
            FailurePhase::OutputImport,
        ] {
            assert_eq!(classify_failure(phase), RetryClass::Terminal);
        }
    }
}
