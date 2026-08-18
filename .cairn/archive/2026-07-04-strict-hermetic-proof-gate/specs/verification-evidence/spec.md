## ADDED Requirements

### Requirement: Strict hermetic proof gate

r[verification_evidence.strict_hermetic_proof_gate] Mantle MUST admit release, deterministic-release, self-hosting, and witness proof evidence only when the producing run selected strict hermeticity and satisfied the strict hermetic proof eligibility classifier.

#### Scenario: strict proof evidence is admitted

GIVEN a release, deterministic-release, self-hosting, or witness proof run selected strict hermeticity
AND the run produced no degraded hermeticity audit events, missing closure facts, protected-environment leaks, or undeclared host-tool observations
WHEN Mantle evaluates proof eligibility
THEN the proof report MUST mark the hermetic proof gate as admitted
AND the report MUST bind the strict mode, classifier inputs, and classifier verdict.

#### Scenario: practical or impure evidence is blocked

GIVEN a proof-producing workflow ran in practical mode or explicit impure mode
WHEN Mantle evaluates proof eligibility
THEN the proof report MUST mark strict proof admission as blocked with a deterministic mode diagnostic
AND it MUST NOT satisfy release, deterministic-release, self-hosting, witness, or global reproducibility proof claims.

#### Scenario: degraded strict evidence fails closed

GIVEN a strict proof run observes missing closure facts, protected environment leakage, undeclared host execution, or any unapproved degraded hermeticity event
WHEN Mantle evaluates proof eligibility
THEN the proof report MUST fail closed or mark the proof class blocked before admission
AND the diagnostic MUST identify the violated hermeticity class.
