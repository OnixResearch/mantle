# Build Scheduling Explicit Parallelism Delta

## ADDED Requirements

### Requirement: Concurrency policy uses explicit parallelism facts

r[build_scheduling.explicit_parallelism_facts] Mantle MUST compute the build job limit from explicit requested jobs, observed available parallelism, configured policy cap, and applicable executor limit. The deterministic policy MUST NOT call host, environment, clock, provider, or async-runtime observation APIs.

#### Scenario: User supplies a valid job count

GIVEN requested jobs, observed host parallelism, policy cap, and executor limit are valid bounded values
WHEN concurrency policy selects the effective job count
THEN it MUST return the same bounded count for equivalent facts
AND insertion order, host timing, or ambient runtime state MUST NOT affect the result.

#### Scenario: Host observation is unavailable

GIVEN the user did not request a job count and the shell cannot obtain valid available-parallelism facts
WHEN concurrency planning runs
THEN it MUST return the declared fallback or a typed blocker according to policy
AND the core MUST NOT query the host directly.

#### Scenario: Parallelism facts exceed bounds

GIVEN requested, observed, policy, or executor values are zero where forbidden, exceed named limits, or cannot convert safely
WHEN concurrency policy validates them
THEN it MUST fail or clamp only as the explicit policy declares
AND it MUST use checked arithmetic and a stable reason code.
