## MODIFIED Requirements

### Requirement: Build pipeline carries explicit hermeticity mode

The build pipeline MUST accept and preserve an explicit hermeticity mode for all
build-entry runs.

At minimum the pipeline MUST distinguish between `practical`, `strict`, and
`impure` execution modes and make that selection available to build-finalization,
reporting, attestation, cache-publication, and proof-classification code.

#### Scenario: Pipeline receives strict mode unchanged

- GIVEN a build-entry command selected hermeticity mode `strict`
- WHEN the pipeline starts the build run
- THEN the pipeline retains that exact mode selection
- AND later build stages can branch on `strict` without guessing from CLI flags

#### Scenario: Pipeline receives impure mode unchanged

- GIVEN a build-entry command selected hermeticity mode `impure`
- WHEN the pipeline starts the build run
- THEN the pipeline retains that exact mode selection
- AND later build stages can block deterministic proof or cache-publication paths
  without guessing from CLI flags

## ADDED Requirements

### Requirement: Impure builds are labeled and proof-blocking

An impure build MUST be labeled as impure in human output, JSON build reports,
hermeticity audit facts, and any generated attestations. The label MUST survive
successful builds so downstream commands can distinguish an impure success from
a proof-eligible success.

Impure builds MUST NOT satisfy deterministic-build proof, release
reproducibility proof, witness rebuild, or policy-satisfied release classes by
default. Future policy MAY define an explicit impure evidence class, but no
existing reproducibility or determinism class may silently accept impure
material.

#### Scenario: Successful impure build remains marked impure

- GIVEN an operator runs `mantle build --impure hello.ncl`
- AND the builder exits successfully
- WHEN Mantle returns the build result
- THEN the result includes hermeticity mode `impure`
- AND any attestation or JSON report contains an impure audit fact

#### Scenario: Impure output cannot satisfy release reproducibility

- GIVEN a release artifact was produced by an impure build
- WHEN release reproducibility or deterministic proof classification runs
- THEN the artifact is rejected for existing proof classes
- AND the report names impure execution as the blocker

### Requirement: Impure mode has an explicit host-input policy

Impure mode MUST define which host inputs are allowed to vary. The first policy
MAY allow selected host environment variables, current working directory access,
and network behavior required by development workflows, but each allowed class
MUST be represented as a typed impure audit event.

The build pipeline MUST NOT treat impure mode as an unbounded privilege escape:
security isolation such as no-new-privileges and namespace containment SHOULD
remain enabled where compatible. If a platform cannot keep a security boundary in
impure mode, the result MUST record that degraded security fact.

#### Scenario: Allowed host environment is audited

- GIVEN an impure build reads an allowed host environment variable
- WHEN the build result is finalized
- THEN the result records a typed impure host-environment audit event

#### Scenario: Security boundary degradation is explicit

- GIVEN impure mode requires disabling a sandbox control on a platform
- WHEN the build starts or completes
- THEN the build report records the disabled control as a typed degraded-security
  event
- AND the event is visible in JSON output
