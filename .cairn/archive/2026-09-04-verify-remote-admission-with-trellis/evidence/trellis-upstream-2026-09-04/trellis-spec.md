# Fenced Attempt Admission Delta

## ADDED Requirements

### Requirement: Fenced-attempt proof boundary is finite and explicit

r[fenced-attempt-admission.boundary] Trellis MUST model fenced-attempt admission only over finite admitted phase, report, fence, event, authorization, linkage, count, and identity-relation facts.

#### Scenario: Consumer supplies an unsupported variant

- **GIVEN** a consumer phase, report kind, relation, or authority class has no exact model variant
- **WHEN** consumer mapping is evaluated
- **THEN** the mapping MUST reject that case as unsupported
- **AND** Trellis evidence MUST NOT claim coverage for it

#### Scenario: Stronger system claim is requested

- **GIVEN** all fenced-attempt proofs pass
- **WHEN** evidence requests transport, storage, cryptography, worker, sandbox, liveness, availability, implementation-equivalence, or release claims
- **THEN** the stronger claim MUST be rejected as outside the proof boundary

### Requirement: Executable admission refines the specification

r[fenced-attempt-admission.refinement] Trellis MUST provide spec and executable functions for fence classification, event classification, phase transition, result linkage, report application, and next-fence calculation, with each executable result equal to its specification.

#### Scenario: Valid current report is applied

- **GIVEN** admitted current identity, fence, history, phase, authorization, and linkage facts permit one report
- **WHEN** the executable admission function runs
- **THEN** its disposition and next state MUST equal the specification
- **AND** only the fields named by that transition MAY change

#### Scenario: Invalid report is rejected

- **GIVEN** one required admitted fact is invalid
- **WHEN** executable and specification decisions run
- **THEN** both MUST return the same rejected reason class
- **AND** both MUST preserve the original state

### Requirement: Only the current fenced attempt can change state

r[fenced-attempt-admission.fence-safety] Trellis MUST prove that stale, future, wrong-job, and wrong-attempt relations reject before a report changes modeled state.

#### Scenario: Stale attempt reports completion

- **GIVEN** a completion report belongs to a fence older than the current attempt
- **WHEN** admission evaluates the report
- **THEN** it MUST reject the report
- **AND** phase, event history, progress facts, result linkage, and completion state MUST remain unchanged

#### Scenario: Reassignment advances the fence

- **GIVEN** the current nonzero fence is below the fixed-width maximum
- **WHEN** next-fence calculation runs for reassignment
- **THEN** the next fence MUST be nonzero and strictly greater
- **AND** a fence at the maximum MUST reject rather than wrap

### Requirement: Attempt events are idempotent and conflict detecting

r[fenced-attempt-admission.event-idempotence] Trellis MUST classify an unseen event as new, an existing event with equal payload identity as already applied, and an existing event with different payload identity as conflict.

#### Scenario: Identical event repeats

- **GIVEN** an event identity and payload identity are already in modeled history
- **WHEN** the same pair is admitted again
- **THEN** admission MUST return an already-applied disposition
- **AND** it MUST preserve every modeled state field

#### Scenario: Event identity is reused with different content

- **GIVEN** modeled history contains an event identity with one payload identity
- **WHEN** a report reuses that event identity with a different payload identity
- **THEN** admission MUST reject the conflict
- **AND** it MUST NOT merge, replace, or apply either payload again

### Requirement: Terminal states reject later mutations

r[fenced-attempt-admission.terminal-closure] Trellis MUST prove that completed, failed, and superseded attempts reject every later state-changing report.

#### Scenario: Completed attempt receives progress

- **GIVEN** an attempt is completed
- **WHEN** start, progress, transfer, result-ready, failure, or completion is submitted again as a new event
- **THEN** admission MUST reject the transition
- **AND** the completed state MUST remain unchanged

### Requirement: Completion requires matching result authority

r[fenced-attempt-admission.completion-linkage] Trellis MUST admit completion only from result-ready state with current identity and fence, worker authority, output-admission authority, and matching result linkage.

#### Scenario: Matching completion succeeds

- **GIVEN** a current result-ready attempt has worker and output authority and the completion names the retained result identity
- **WHEN** completion admission runs
- **THEN** it MUST move the model to completed
- **AND** it MUST preserve the same result identity

#### Scenario: Completion lacks authority or linkage

- **GIVEN** completion is early, lacks output authority, or names a different result identity
- **WHEN** admission runs
- **THEN** it MUST reject completion
- **AND** it MUST not expose a completed result

### Requirement: Rejection preserves all modeled state

r[fenced-attempt-admission.rejection-preservation] Trellis MUST provide direct proof functions that every rejected and already-applied outcome preserves phase, fence, event history, progress facts, result linkage, and completion facts.

#### Scenario: Any rejection path is selected

- **GIVEN** fence, event, transition, authority, linkage, bound, or terminal checks reject a report
- **WHEN** rejection-preservation proofs are applied
- **THEN** every modeled state field before and after MUST be equal
- **AND** the proof MUST NOT assume that consumer shell effects rolled back

### Requirement: Fenced-attempt evidence includes positive and negative proof artifacts

r[fenced-attempt-admission.evidence] Trellis MUST export proof evidence that binds exact source, requirements, verifier, assumptions, trusted boundaries, executable tests, direct proof functions, consumer profile, and non-claims.

#### Scenario: Proof evidence is complete

- **GIVEN** focused tests, property tests, full relevant Verus verification, Tracey coverage, and proof-artifact export pass
- **WHEN** the evidence bundle is reviewed
- **THEN** each named theorem MUST link to its requirement and proof function
- **AND** positive and negative fixtures plus the consumer non-claim boundary MUST be present

#### Scenario: Proof or mapping evidence is stale

- **GIVEN** source, requirement, verifier, assumption, proof digest, consumer profile, or trusted-boundary facts differ from the evidence bundle
- **WHEN** evidence validation runs
- **THEN** it MUST reject the stale bundle
- **AND** it MUST NOT claim current proof coverage
