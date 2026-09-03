struct ParallelismFacts {
    requested_jobs: Option<u32>,
    observed_jobs: Option<u32>,
    policy_jobs_max: u32,
}

fn effective(facts: ParallelismFacts) -> u32 {
    facts.requested_jobs.or(facts.observed_jobs).unwrap_or(1).min(facts.policy_jobs_max)
}
