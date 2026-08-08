## ADDED Requirements

### Requirement: Fresh-directory dev resume revalidates every restored stage [r[source_built_fixed_point_improved_iteration.dev_cross_run_resume]]

A dev resume MUST restore a completed stage into a fresh staging directory only after current source, plan, policy, stage, producer, and output identities match the content-addressed resume bundle. The shell MUST continue at the first incomplete stage. The report MUST distinguish restored stages from executed stages. A promoted run MUST ignore every resume bundle.

#### Scenario: A valid transition bundle resumes in a fresh directory

- GIVEN a prior dev attempt published a complete transition-stage bundle
- AND current source, plan, policy, producer, and output identities match that bundle
- WHEN a new dev attempt resumes in a fresh staging directory
- THEN the shell MUST restore the required transition state
- AND it MUST continue at the first incomplete stage
- AND the report MUST identify the transition as restored rather than executed

#### Scenario: A changed bundle forces stage execution

- GIVEN a resume bundle is partial, stale, modified, or bound to different source, plan, policy, stage, producer, or output identities
- WHEN a dev attempt evaluates the bundle
- THEN the planner MUST reject the restored state
- AND the shell MUST execute that stage from its admitted inputs

#### Scenario: A promoted run ignores valid dev state

- GIVEN a valid dev resume bundle exists
- WHEN a promoted cold proof starts
- THEN the shell MUST ignore the bundle
- AND the proof MUST start from empty provider and output authority

### Requirement: Runtime evidence separates dev adoption from promoted proof [r[source_built_fixed_point_improved_iteration.dev_resume_runtime_confirmation]]

Runtime validation MUST record a complete cold-to-cached-to-adopt dev cycle and a separate fresh promoted cold run. Dev adoption MUST NOT write a promoted receipt or update a release alias. The promoted run MUST NOT read dev cache or resume state.

#### Scenario: A dev cycle adopts validated cached state

- GIVEN a cold dev run publishes valid cache and resume state
- WHEN a later dev run uses the same admitted source and plan
- THEN the later run MUST adopt only validated state
- AND it MUST emit a dev-only report
- AND it MUST NOT publish a promoted receipt or release alias

#### Scenario: A fresh promoted run remains independent

- GIVEN dev cache and resume state exist
- WHEN a fresh promoted proof runs
- THEN it MUST ignore all dev state
- AND it MUST start from empty authority
- AND its result MUST depend only on executed promoted-proof facts
