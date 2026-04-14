## ADDED Requirements

### Requirement: Strict hermetic build selection

The CLI MUST let operators request strict hermetic execution for build-entry
commands.

At minimum `crunch build` and `crunch self-build` MUST accept a
`--strict-hermetic` flag and pass that selection unchanged into the pipeline or
self-build orchestration.

#### Scenario: Strict build selects strict profile

- GIVEN a Nickel file that can be built normally
- WHEN `crunch build --strict-hermetic hello.ncl` runs
- THEN the pipeline executes with hermeticity mode `strict`
- AND later strict-mode blockers are treated as build errors instead of warnings

#### Scenario: Self-build selects strict profile

- GIVEN a self-build invocation
- WHEN `crunch self-build --strict-hermetic --store /tmp/store` runs
- THEN the self-build flow records hermeticity mode `strict`
- AND later proof-oriented checks can act on that mode selection

### Requirement: Build reporting exposes hermeticity audit facts

`crunch build` and `crunch --json build` MUST report the selected hermeticity
mode and any hermeticity audit events recorded during the run.

Human-readable output MUST summarize degraded execution facts before reporting a
successful result. JSON output MUST expose the same facts in a stable
machine-readable shape.

#### Scenario: Clean strict run reports no degraded facts

- GIVEN a successful strict build with no degraded conditions
- WHEN `crunch --json build --strict-hermetic hello.ncl` runs
- THEN the JSON report records hermeticity mode `strict`
- AND the hermeticity audit-event list is empty

#### Scenario: Practical run reports degraded facts

- GIVEN a successful practical build that recorded a degraded execution fact
- WHEN `crunch --json build hello.ncl` runs
- THEN the JSON report records hermeticity mode `practical`
- AND the report includes the recorded hermeticity audit event
- AND the human-readable output warns that the run used degraded hermeticity
