## ADDED Requirements

### Requirement: Dependency security audit evidence

r[verification_evidence.dependency_security_audit] Mantle MUST tie dependency and security audit status to current audit evidence produced with the repository's checked-in policy and explicit residual-risk classification.

#### Scenario: audit uses checked-in policy

GIVEN an operator runs the dependency/security audit
WHEN audit evidence is recorded
THEN the evidence MUST name the checked-in policy path or digest and the command used.
AND default-policy or missing-policy tool output MUST NOT be treated as the repository's audit result.

#### Scenario: findings are classified

GIVEN the audit reports advisories, license findings, policy errors, or unmaintained dependencies
WHEN the evidence summary is written
THEN every finding MUST be classified as fixed, accepted waiver, upstream-blocked, action-required, or tooling/config issue.
AND action-required findings MUST name the next owner action.

#### Scenario: residual risk prevents clean claims

GIVEN accepted-waiver or upstream-blocked findings remain
WHEN status, release-readiness, or task evidence reports dependency security state
THEN the report MUST describe the residual classified findings.
AND it MUST NOT claim the dependency audit is clean unless the audit command reports no relevant findings under the checked-in policy.

#### Scenario: dependency updates are verified

GIVEN dependency pins, lockfile entries, waivers, or audit policy change
WHEN the change is validated
THEN Mantle MUST run the relevant audit and lockfile/manifest consistency checks.
AND failures MUST be fixed or recorded as bounded blockers before completion is claimed.
