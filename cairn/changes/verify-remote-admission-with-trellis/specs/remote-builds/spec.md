# Remote Builds Trellis Verification Delta

## ADDED Requirements

### Requirement: Remote admission has a Trellis-verified abstract model

r[remote_builds.trellis_admission_model] Mantle MUST maintain a bounded abstract model for fenced remote-attempt admission in Trellis and MUST bind the exact modeled state, event, authorization, and result-linkage fields.

#### Scenario: Current report follows an allowed transition

- **GIVEN** a current job, attempt, nonzero fence, supported phase, valid event identity and digest, required authorization, and valid result linkage
- **WHEN** the Trellis executable model evaluates the report
- **THEN** its accepted next state MUST match the model specification
- **AND** the relevant Verus postcondition MUST prove that refinement for the recorded Trellis revision

#### Scenario: Model input omits a required fact

- **GIVEN** a Mantle phase, report kind, fence class, event disposition, authorization class, or result-linkage class has no admitted Trellis representation
- **WHEN** the Mantle projection runs
- **THEN** it MUST reject the projection as unsupported
- **AND** proof evidence MUST NOT cover that runtime case

### Requirement: Trellis proves the closed remote-attempt safety set

r[remote_builds.trellis_admission_safety] Trellis MUST prove stale and mismatched fence rejection, terminal-state closure, duplicate-event idempotence, conflicting-event rejection, completion linkage, representable fence advance, and rejected-state preservation for the abstract model.

#### Scenario: Stale or mismatched report is evaluated

- **GIVEN** a report has a stale or future fence, wrong job, wrong attempt, missing worker authority, or missing required output-admission authority
- **WHEN** the verified model evaluates it
- **THEN** the decision MUST reject the report
- **AND** the modeled current state MUST remain unchanged

#### Scenario: Duplicate and conflicting events are evaluated

- **GIVEN** an event identity is already present in modeled history
- **WHEN** the same digest or a different digest is supplied
- **THEN** the same digest MUST return an idempotent no-op and the different digest MUST be rejected
- **AND** neither case MUST apply a second state transition

#### Scenario: Completion lacks matching result linkage

- **GIVEN** a completion report arrives before result-ready state or names a different result identity
- **WHEN** the verified model evaluates it
- **THEN** completion MUST be rejected
- **AND** the model MUST NOT reach its completed state

### Requirement: Mantle decisions have a fail-closed model projection

r[remote_builds.trellis_admission_projection] Mantle MUST project admitted remote-attempt facts into the Trellis model through a pure function and MUST compare Mantle and Trellis executable decisions over a bounded complete fixture matrix.

#### Scenario: Supported finite matrix agrees

- **GIVEN** every supported phase and report-kind pair plus bounded fence, event, digest, authorization, and linkage classes
- **WHEN** the parity rail evaluates Mantle and Trellis decisions
- **THEN** accepted, idempotent, and rejected dispositions plus next modeled state MUST agree
- **AND** the evidence MUST bind Mantle source, Trellis source, fixture, policy, and toolchain BLAKE3 identities

#### Scenario: Mantle adds a new transition variant

- **GIVEN** Mantle adds or changes a remote phase, report kind, reason class, or admission field without updating the projection
- **WHEN** compile-time exhaustiveness or parity validation runs
- **THEN** validation MUST fail before proof coverage is reported
- **AND** the old proof artifact MUST remain visible only as stale evidence

### Requirement: Trellis proof evidence remains external to runtime authority

r[remote_builds.trellis_admission_evidence_boundary] Mantle MUST consume Trellis proof evidence through the accepted Kamacite and Valence profile and MUST NOT use proof-sidecar presence as runtime report or output authority.

#### Scenario: Accepted proof evidence is linked

- **GIVEN** Trellis proof artifacts, verifier status, assumptions, policy, source identity, Kamacite envelope, and Valence acceptance all match the modeled Mantle revision
- **WHEN** Mantle binds the sidecar
- **THEN** it MAY report the named abstract safety properties as accepted external evidence
- **AND** ordinary remote admission MUST still evaluate Mantle runtime facts and output trust

#### Scenario: Proof evidence is stale or absent

- **GIVEN** a Trellis revision, proof digest, verifier identity, assumption, model mapping, Kamacite envelope, or Valence receipt is missing or stale
- **WHEN** proof-required policy evaluates the evidence
- **THEN** it MUST fail or report unsupported according to policy
- **AND** it MUST NOT weaken runtime rejection or output-admission rules

### Requirement: Formal proof claims remain bounded

r[remote_builds.trellis_admission_claim_boundary] Mantle MUST limit the formal claim to the named abstract model properties and recorded projection evidence.

#### Scenario: Formal evidence passes

- **GIVEN** Trellis verification, Mantle parity, evidence binding, and negative fixtures pass
- **WHEN** Mantle reports the result
- **THEN** it MAY claim the named abstract safety properties for the recorded model
- **AND** it MUST NOT claim full implementation equivalence, persistence atomicity, transport reliability, cryptographic correctness, worker correctness, liveness, or release eligibility
