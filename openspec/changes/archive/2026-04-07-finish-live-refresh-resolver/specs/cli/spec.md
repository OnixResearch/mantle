## ADDED Requirements

### Requirement: Refresh and stale commands report resolver failures distinctly

The CLI MUST distinguish successful refresh/stale results from resolver
failures.

`crunch refresh` MUST report per-input failures and exit non-zero when any
selected input cannot be resolved or hashed, even if other inputs were updated
successfully.

`crunch list-stale` MUST report stale inputs and failed checks separately. It
MUST NOT print `all inputs up to date` when any check failed.

#### Scenario: Partial refresh reports updates and failures together

- GIVEN a manifest with one reachable input and one unreachable input
- WHEN `crunch refresh` runs
- THEN it reports the successful update for the reachable input
- AND it reports the failed resolution for the unreachable input
- AND it exits non-zero
- AND successful lock updates are still written

#### Scenario: Stale check failure is not reported as clean

- GIVEN one input is stale and another input cannot be checked
- WHEN `crunch list-stale` runs
- THEN it reports the stale input
- AND it separately reports the failed check
- AND it exits non-zero
- AND it does not print `all inputs up to date`
