# Verification Evidence Specification

## Purpose

Defines requirements for retiring or sharpening remaining dependency-audit waivers.

## Requirements

### Requirement: Dependency audit waivers are actively resolved

r[verification_evidence.dependency_audit_waiver_resolution] Mantle dependency audit maintenance MUST attempt to retire remaining advisory waivers through minimal safe dependency movement before preserving a waiver as upstream-blocked.

#### Scenario: Waiver is retired by dependency movement

GIVEN a waived advisory can be removed by a safe dependency, feature, or lockfile update
WHEN the dependency audit is refreshed
THEN the waiver MUST be removed from policy and documentation
AND focused compile checks MUST cover the affected dependency path.

#### Scenario: Waiver cannot be retired safely

GIVEN a waived advisory remains blocked by an upstream dependency constraint
WHEN the audit policy is refreshed
THEN the waiver MUST remain explicit with a reason that identifies the path and blocker
AND the evidence MUST state why removal is not yet safe.

### Requirement: Upstream-blocked audit risks have unblock conditions

r[verification_evidence.dependency_audit_upstream_blockers] Every retained upstream-blocked dependency-audit waiver MUST name the advisory, dependency path, current constraint, and the condition that would allow the waiver to be removed.

#### Scenario: Waiver documentation is actionable

GIVEN an advisory waiver remains in `deny.toml`
WHEN an operator reviews dependency-audit documentation
THEN the documentation MUST identify the upstream owner or crate path, the blocking version relationship, and the next action needed to retire the waiver
AND it MUST NOT describe the risk as generally accepted without a follow-up condition.

### Requirement: Dependency audit evidence uses checked-in policy

r[verification_evidence.dependency_audit_policy_regression] Dependency-audit evidence MUST run against Mantle's checked-in `deny.toml` policy, and missing/default policy audit output MUST NOT be accepted as proof of audit status.

#### Scenario: Checked-in policy audit is accepted

GIVEN `deny.toml` is present in the repository
WHEN audit evidence is collected
THEN the command MUST load that policy explicitly or prove it used that policy
AND the transcript MUST include the command used.

#### Scenario: Missing policy audit is rejected

GIVEN an audit command runs without Mantle's checked-in policy
WHEN evidence validation reviews the transcript
THEN the evidence MUST be rejected or marked non-authoritative
AND it MUST NOT be cited as dependency audit success.
