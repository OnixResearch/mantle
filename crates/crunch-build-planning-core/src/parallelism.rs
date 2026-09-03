use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const DEFAULT_POLICY_JOBS_MAX: u32 = 16;
pub const DEFAULT_FALLBACK_JOBS: u32 = 1;
pub const ABSOLUTE_JOBS_MAX: u32 = 4_096;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ParallelismObservation {
    Available(u32),
    Unavailable,
    ConversionFailed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UnavailableParallelismPolicy {
    UseFallback(u32),
    Block,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ZeroRequestPolicy {
    ClampToOne,
    Block,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ParallelismFacts {
    pub requested_jobs: Option<u32>,
    pub observed_parallelism: ParallelismObservation,
    pub policy_jobs_max: u32,
    pub executor_jobs_max: Option<u32>,
    pub unavailable_policy: UnavailableParallelismPolicy,
    pub zero_request_policy: ZeroRequestPolicy,
}

impl ParallelismFacts {
    #[must_use]
    pub const fn legacy_compatible(requested_jobs: Option<u32>, observed_parallelism: ParallelismObservation) -> Self {
        Self {
            requested_jobs,
            observed_parallelism,
            policy_jobs_max: DEFAULT_POLICY_JOBS_MAX,
            executor_jobs_max: None,
            unavailable_policy: UnavailableParallelismPolicy::UseFallback(DEFAULT_FALLBACK_JOBS),
            zero_request_policy: ZeroRequestPolicy::ClampToOne,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct JobLimitDecision {
    pub requested_jobs: Option<u32>,
    pub observed_parallelism: ParallelismObservation,
    pub policy_jobs_max: u32,
    pub executor_jobs_max: Option<u32>,
    pub effective_jobs: u32,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum ParallelismBlocker {
    PolicyLimitZero,
    PolicyLimitExceeded,
    ExecutorLimitZero,
    ObservedParallelismZero,
    ParallelismUnavailable,
    ParallelismConversionFailed,
    FallbackZero,
    RequestedJobsZero,
}

pub fn plan_parallelism(facts: ParallelismFacts) -> Result<JobLimitDecision, ParallelismBlocker> {
    validate_parallelism_facts(facts)?;
    let (candidate_jobs, source_reason) = candidate_jobs(facts)?;
    let mut effective_jobs = candidate_jobs.min(facts.policy_jobs_max);
    let mut reason_codes = vec![source_reason.to_string()];
    if effective_jobs != candidate_jobs {
        reason_codes.push("policy-cap".to_string());
    }
    if let Some(executor_jobs_max) = facts.executor_jobs_max
        && effective_jobs > executor_jobs_max
    {
        effective_jobs = executor_jobs_max;
        reason_codes.push("executor-cap".to_string());
    }
    debug_assert!(effective_jobs > 0);
    debug_assert!(effective_jobs <= facts.policy_jobs_max);
    Ok(JobLimitDecision {
        requested_jobs: facts.requested_jobs,
        observed_parallelism: facts.observed_parallelism,
        policy_jobs_max: facts.policy_jobs_max,
        executor_jobs_max: facts.executor_jobs_max,
        effective_jobs,
        reason_codes,
    })
}

fn validate_parallelism_facts(facts: ParallelismFacts) -> Result<(), ParallelismBlocker> {
    if facts.policy_jobs_max == 0 {
        return Err(ParallelismBlocker::PolicyLimitZero);
    }
    if facts.policy_jobs_max > ABSOLUTE_JOBS_MAX {
        return Err(ParallelismBlocker::PolicyLimitExceeded);
    }
    if facts.executor_jobs_max == Some(0) {
        return Err(ParallelismBlocker::ExecutorLimitZero);
    }
    debug_assert!(facts.policy_jobs_max <= ABSOLUTE_JOBS_MAX);
    debug_assert!(facts.executor_jobs_max.is_none_or(|jobs| jobs > 0));
    Ok(())
}

fn candidate_jobs(facts: ParallelismFacts) -> Result<(u32, &'static str), ParallelismBlocker> {
    if let Some(requested_jobs) = facts.requested_jobs {
        if requested_jobs > 0 {
            return Ok((requested_jobs, "requested-jobs"));
        }
        return match facts.zero_request_policy {
            ZeroRequestPolicy::ClampToOne => Ok((DEFAULT_FALLBACK_JOBS, "zero-request-clamped")),
            ZeroRequestPolicy::Block => Err(ParallelismBlocker::RequestedJobsZero),
        };
    }
    observed_candidate(facts.observed_parallelism, facts.unavailable_policy)
}

fn observed_candidate(
    observation: ParallelismObservation,
    unavailable_policy: UnavailableParallelismPolicy,
) -> Result<(u32, &'static str), ParallelismBlocker> {
    match observation {
        ParallelismObservation::Available(0) => Err(ParallelismBlocker::ObservedParallelismZero),
        ParallelismObservation::Available(jobs) => Ok((jobs, "observed-parallelism")),
        ParallelismObservation::ConversionFailed => Err(ParallelismBlocker::ParallelismConversionFailed),
        ParallelismObservation::Unavailable => match unavailable_policy {
            UnavailableParallelismPolicy::UseFallback(0) => Err(ParallelismBlocker::FallbackZero),
            UnavailableParallelismPolicy::UseFallback(jobs) => Ok((jobs, "unavailable-fallback")),
            UnavailableParallelismPolicy::Block => Err(ParallelismBlocker::ParallelismUnavailable),
        },
    }
}
