## ADDED Requirements

### Requirement: Proof-mode hermeticity audit gate

r[verification_evidence.proof_mode_hermeticity_audit_gate] Mantle proof-mode evidence MUST evaluate typed hermeticity audit events against an explicit closed-by-default policy, fail admission on unapproved events, and report any approved downgrade without satisfying stricter proof claims.

#### Scenario: clean audit set is admitted

GIVEN a strict proof run emits no hermeticity audit events or emits only policy-classified informational events
WHEN Mantle evaluates proof-mode audit eligibility
THEN the audit gate MUST be admitted
AND the report MUST bind the event set digest and policy basis.

#### Scenario: unapproved event blocks proof

GIVEN a strict proof run emits an unknown, degraded, or policy-denied hermeticity audit event
WHEN Mantle evaluates proof-mode audit eligibility
THEN the proof admission MUST fail closed with a deterministic audit-event diagnostic
AND the produced output MUST NOT satisfy release, deterministic-release, self-hosting, witness, or global reproducibility proof claims.

#### Scenario: approved downgrade remains narrower evidence

GIVEN policy explicitly allows a hermeticity audit event only as a downgrade for a selected workflow
WHEN Mantle renders the proof report
THEN the report MUST name the event class, policy basis, affected proof class, and narrower claim
AND it MUST NOT present the downgraded evidence as satisfying a stricter proof class.
