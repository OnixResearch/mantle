# Verification Evidence Specification

## Purpose

Defines fail-closed handling for actionable dependency-audit findings across advisory, source, and license policy.

## ADDED Requirements

### Requirement: Actionable dependency-audit findings are resolved narrowly

r[verification_evidence.dependency_audit_actionable_findings] Mantle MUST resolve actionable dependency-audit findings through the narrowest reviewable dependency or policy change, MUST retain fail-closed defaults for unknown sources and licenses, and MUST NOT classify a checked-policy audit as clean while relevant findings remain.

#### Scenario: Compatible fixed dependency release exists

GIVEN an affected locked dependency has a compatible release that fixes a reported vulnerability
WHEN Mantle refreshes dependency audit state
THEN Mantle MUST select and verify the fixed release before considering an advisory waiver
AND focused locked checks MUST cover the consuming dependency path.

#### Scenario: Exact reviewed Git repository is required

GIVEN an accepted Mantle requirement mandates a Git dependency at an immutable revision
WHEN source policy evaluates that dependency
THEN policy MAY admit only the reviewed repository while independent checks MUST continue to enforce the exact revision
AND unknown repositories, floating refs, and alternate release overrides MUST remain rejected.

#### Scenario: Exact compound SPDX expression is reviewed

GIVEN a selected dependency declares a compound SPDX expression composed of reviewed license terms and exceptions
WHEN license policy is updated
THEN policy MAY admit that exact expression without disabling crate license checks or lowering confidence
AND unreviewed expressions and missing license metadata MUST remain findings.

#### Scenario: Audit evidence omits checked-in policy

GIVEN an audit transcript omits Mantle's checked-in `deny.toml` or uses default policy
WHEN validation evaluates the transcript
THEN the transcript MUST remain non-authoritative
AND it MUST NOT be cited as proof that advisory, license, or source findings are resolved.
