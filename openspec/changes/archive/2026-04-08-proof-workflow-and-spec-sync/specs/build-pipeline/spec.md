## MODIFIED Requirements

### Requirement: Parallel builds

The build pipeline MUST dispatch independent ready derivations concurrently.
The maximum number of in-flight builds MUST be bounded by the configured
`max_jobs` value.

The scheduler MUST deduplicate goals by derivation path so the same derivation
is not built twice while concurrency is enabled.

#### Scenario: Independent derivations build concurrently

- GIVEN two derivations whose dependencies are already satisfied
- AND `max_jobs` is at least 2
- WHEN the pipeline runs
- THEN the worker may dispatch both builds without waiting for the first one to finish

#### Scenario: Jobs cap serializes dispatch

- GIVEN two derivations whose dependencies are already satisfied
- AND `max_jobs` is 1
- WHEN the pipeline runs
- THEN the worker dispatches at most one build at a time
- AND the second build waits until the first one reaches a terminal state
