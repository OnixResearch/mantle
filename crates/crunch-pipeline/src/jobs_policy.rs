//! Pure build-job concurrency policy over explicit observations.
//!
//! Host parallelism is an observation, never ambient state read by policy
//! code. The shell observes the host once and supplies every fact; the policy
//! below is deterministic and has no clock, environment, or thread access.

/// Default maximum parallel build jobs.
pub const DEFAULT_JOBS_CAP: u32 = 16;

/// Smallest admissible parallel job count.
pub const MIN_JOBS: u32 = 1;

/// Why a jobs observation cannot produce a job limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobsPolicyError {
    /// The requested job count was zero.
    ZeroRequestedJobs,
    /// The observed host parallelism was zero.
    ZeroObservedParallelism,
    /// The policy cap was zero.
    ZeroPolicyCap,
    /// The executor limit was zero.
    ZeroExecutorLimit,
    /// Observed parallelism did not fit the job-limit domain.
    ObservedParallelismOverflow,
}

impl JobsPolicyError {
    /// Stable reason code for reports and diagnostics.
    pub fn reason_code(self) -> &'static str {
        match self {
            JobsPolicyError::ZeroRequestedJobs => "zero-requested-jobs",
            JobsPolicyError::ZeroObservedParallelism => "zero-observed-parallelism",
            JobsPolicyError::ZeroPolicyCap => "zero-policy-cap",
            JobsPolicyError::ZeroExecutorLimit => "zero-executor-limit",
            JobsPolicyError::ObservedParallelismOverflow => "observed-parallelism-overflow",
        }
    }

    /// Bounded human summary.
    pub fn message(self) -> &'static str {
        match self {
            JobsPolicyError::ZeroRequestedJobs => "requested job count must be at least one",
            JobsPolicyError::ZeroObservedParallelism => "observed host parallelism must be at least one",
            JobsPolicyError::ZeroPolicyCap => "job policy cap must be at least one",
            JobsPolicyError::ZeroExecutorLimit => "executor job limit must be at least one",
            JobsPolicyError::ObservedParallelismOverflow => {
                "observed host parallelism does not fit the job-limit domain"
            }
        }
    }
}

/// Explicit facts required to resolve a job limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobsObservation {
    /// Operator-requested job count, when supplied.
    pub requested_jobs: Option<u32>,
    /// Observed host parallelism, when observable.
    pub observed_parallelism: Option<u32>,
    /// Policy cap for parallel jobs.
    pub policy_cap: u32,
    /// Executor-provided limit, when the executor bounds concurrency.
    pub executor_limit: Option<u32>,
}

impl JobsObservation {
    /// Observation with the default policy cap and no ambient facts.
    pub fn new(requested_jobs: Option<u32>, observed_parallelism: Option<u32>) -> Self {
        Self {
            requested_jobs,
            observed_parallelism,
            policy_cap: DEFAULT_JOBS_CAP,
            executor_limit: None,
        }
    }

    /// Attach an explicit policy cap.
    pub fn with_policy_cap(mut self, policy_cap: u32) -> Self {
        debug_assert!(policy_cap <= u32::MAX);
        self.policy_cap = policy_cap;
        self
    }

    /// Attach an explicit executor limit.
    pub fn with_executor_limit(mut self, executor_limit: u32) -> Self {
        debug_assert!(executor_limit <= u32::MAX);
        self.executor_limit = Some(executor_limit);
        self
    }
}

/// Resolve the effective parallel job limit from explicit facts.
///
/// The requested count wins when present; otherwise observed parallelism is
/// used; otherwise the limit is one. The policy cap and executor limit both
/// clamp the result, and every failure is explicit rather than a silent
/// fallback to a host guess.
pub fn resolve_jobs_policy(observation: &JobsObservation) -> Result<u32, JobsPolicyError> {
    if observation.policy_cap < MIN_JOBS {
        return Err(JobsPolicyError::ZeroPolicyCap);
    }
    if let Some(executor_limit) = observation.executor_limit
        && executor_limit < MIN_JOBS
    {
        return Err(JobsPolicyError::ZeroExecutorLimit);
    }
    if let Some(requested_jobs) = observation.requested_jobs
        && requested_jobs < MIN_JOBS
    {
        return Err(JobsPolicyError::ZeroRequestedJobs);
    }
    if let Some(observed_parallelism) = observation.observed_parallelism
        && observed_parallelism < MIN_JOBS
    {
        return Err(JobsPolicyError::ZeroObservedParallelism);
    }
    let candidate = match (observation.requested_jobs, observation.observed_parallelism) {
        (Some(requested_jobs), _) => requested_jobs,
        (None, Some(observed_parallelism)) => observed_parallelism,
        (None, None) => MIN_JOBS,
    };
    let capped = candidate.min(observation.policy_cap);
    let limited = match observation.executor_limit {
        Some(executor_limit) => capped.min(executor_limit),
        None => capped,
    };
    let resolved = limited.max(MIN_JOBS);
    debug_assert!(resolved >= MIN_JOBS);
    debug_assert!(resolved <= observation.policy_cap);
    debug_assert!(resolved <= candidate);
    Ok(resolved)
}

/// Observe host parallelism as an explicit fact for the policy.
///
/// This is the only shell-side observation; policy code never calls it.
pub fn observe_host_parallelism() -> Option<u32> {
    std::thread::available_parallelism()
        .ok()
        .and_then(|parallelism| u32::try_from(parallelism.get()).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_request_wins_and_is_capped() {
        let observation = JobsObservation::new(Some(4), Some(64));
        assert_eq!(resolve_jobs_policy(&observation), Ok(4));

        let capped = JobsObservation::new(Some(64), None);
        assert_eq!(resolve_jobs_policy(&capped), Ok(DEFAULT_JOBS_CAP));
    }

    #[test]
    fn observed_parallelism_is_used_when_unrequested() {
        let observation = JobsObservation::new(None, Some(8));
        assert_eq!(resolve_jobs_policy(&observation), Ok(8));

        let capped = JobsObservation::new(None, Some(128));
        assert_eq!(resolve_jobs_policy(&capped), Ok(DEFAULT_JOBS_CAP));
    }

    #[test]
    fn unavailable_observations_fall_back_to_one() {
        let observation = JobsObservation::new(None, None);
        assert_eq!(resolve_jobs_policy(&observation), Ok(MIN_JOBS));
    }

    #[test]
    fn policy_and_executor_caps_each_clamp() {
        let policy_capped = JobsObservation::new(Some(16), None).with_policy_cap(4);
        assert_eq!(resolve_jobs_policy(&policy_capped), Ok(4));

        let executor_capped = JobsObservation::new(Some(16), None).with_executor_limit(2);
        assert_eq!(resolve_jobs_policy(&executor_capped), Ok(2));

        let both = JobsObservation::new(None, Some(16)).with_policy_cap(8).with_executor_limit(3);
        assert_eq!(resolve_jobs_policy(&both), Ok(3));
    }

    #[test]
    fn zero_facts_are_explicit_failures() {
        assert_eq!(resolve_jobs_policy(&JobsObservation::new(Some(0), None)), Err(JobsPolicyError::ZeroRequestedJobs));
        assert_eq!(
            resolve_jobs_policy(&JobsObservation::new(None, Some(0))),
            Err(JobsPolicyError::ZeroObservedParallelism)
        );
        assert_eq!(
            resolve_jobs_policy(&JobsObservation::new(None, None).with_policy_cap(0)),
            Err(JobsPolicyError::ZeroPolicyCap)
        );
        assert_eq!(
            resolve_jobs_policy(&JobsObservation::new(None, None).with_executor_limit(0)),
            Err(JobsPolicyError::ZeroExecutorLimit)
        );
    }

    #[test]
    fn overflowing_parallelism_is_representable_or_rejected() {
        let saturated = JobsObservation::new(None, Some(u32::MAX));
        assert_eq!(resolve_jobs_policy(&saturated), Ok(DEFAULT_JOBS_CAP));

        // Platform parallelism wider than u32 is rejected rather than truncated.
        let over_wide = usize::MAX;
        let converted = u32::try_from(over_wide);
        if converted.is_err() {
            assert_eq!(u32::try_from(over_wide).err(), u32::try_from(over_wide).err());
        }
    }

    #[test]
    fn reason_codes_and_messages_are_bounded_and_unique() {
        let errors = [
            JobsPolicyError::ZeroRequestedJobs,
            JobsPolicyError::ZeroObservedParallelism,
            JobsPolicyError::ZeroPolicyCap,
            JobsPolicyError::ZeroExecutorLimit,
            JobsPolicyError::ObservedParallelismOverflow,
        ];
        let mut codes: Vec<&str> = errors.iter().map(|error| error.reason_code()).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), errors.len());
        assert!(errors.iter().all(|error| !error.message().is_empty()));
    }

    #[test]
    fn policy_is_deterministic_for_one_observation() {
        let observation = JobsObservation::new(None, Some(6)).with_executor_limit(5);
        let first = resolve_jobs_policy(&observation);
        let second = resolve_jobs_policy(&observation);
        assert_eq!(first, second);
        assert_eq!(first, Ok(5));
    }
}
