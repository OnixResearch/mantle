## ADDED Requirements

### Requirement: Deterministic proof receipts MUST bind declared and observed build effects [r[release.verification.tech.effect-receipts]]

Mantle deterministic proof receipts MUST record a versioned build-effect policy, the closed set of effects declared by the proof recipe, and the closed set of effects observed by the sandbox or audit layer during the proof run.

The first-phase policy MUST use `mantle-build-effects-v1` and MUST reject unknown effect names unless an explicit forward-compatible policy version is selected.

#### Scenario: Deterministic proof records effect sets [r[release.verification.tech.effect-receipts.records]]

- GIVEN a deterministic proof run whose recipe declares local store reads and output writes
- WHEN the proof receipt is written
- THEN the receipt records `effect_policy_version = mantle-build-effects-v1`
- AND it records the declared effect set
- AND it records the observed effect set from sandbox/audit events

#### Scenario: Deterministic verification rejects undeclared effects [r[release.verification.tech.effect-receipts.rejects-undeclared]]

- GIVEN a proof receipt whose observed effects include `network`
- AND the declared effect set does not include `network`
- WHEN `mantle release verify --require-deterministic-release` evaluates the receipt
- THEN verification fails closed
- AND the diagnostic names the undeclared observed effect

#### Scenario: Missing effect observation blocks deterministic claim [r[release.verification.tech.effect-receipts.missing-observation]]

- GIVEN a proof receipt that omits observed effect evidence
- WHEN deterministic-release verification is required
- THEN verification fails closed
- AND the output explains that deterministic effect auditing was incomplete
