# Release Provenance Specification

## Purpose

Require release verification to decide complete selected policy before reporting command success.

## Requirements

### Requirement: Release verification produces one final policy decision

r[mantle.release_provenance.verification_decision.complete] Mantle MUST aggregate manifest integrity and every selected release policy contribution into one final deterministic decision before any success output is rendered.

#### Scenario: All selected checks pass

r[mantle.release_provenance.verification_decision.fixtures.positive]
- GIVEN manifest integrity and every optional or required contribution selected by the invocation satisfy policy
- WHEN release verification aggregates the normalized results
- THEN the final decision MUST be valid
- AND it MUST preserve each bounded contributor result without strengthening its claim.

#### Scenario: A late required check rejects the release

r[mantle.release_provenance.verification_decision.fixtures.negative]
- GIVEN manifest integrity passes but required reproducibility, deterministic proof, provider fixed-point proof, stack provenance, external role, StageX, function-address, or Cairn handoff policy is unsatisfied
- WHEN release verification aggregates the normalized results
- THEN the final decision MUST be invalid with ordered diagnostics naming every safely evaluable blocker
- AND manifest integrity success MUST NOT override the rejection.

### Requirement: Verification aggregation preserves core and shell boundaries

r[mantle.release_provenance.verification_decision.boundary] Mantle MUST compute final validity, disposition, contributor status, and ordered policy diagnostics in a pure core over supplied facts, while filesystem reads, specialized evaluator I/O, serialization, stdout/stderr, and process exit remain in the CLI shell.

#### Scenario: Decision is testable without CLI effects

r[mantle.release_provenance.verification_decision.boundary.test]
- GIVEN in-memory policy requirements and normalized contributor results
- WHEN the decision core evaluates them
- THEN it MUST return the same final decision independent of filesystem state, environment, clocks, output streams, or process state.

### Requirement: Required contributors cannot bypass the final decision

r[mantle.release_provenance.verification_decision.completeness] Every release policy contributor that can block an invocation MUST have an explicit mapping into the final decision, and adding a contributor without decision coverage MUST fail a compile-time exhaustive match or a focused completeness check.

#### Scenario: New evidence policy is wired into aggregation

r[mantle.release_provenance.verification_decision.completeness.test]
- GIVEN a release profile adds a required evidence contributor
- WHEN verification decision coverage is checked
- THEN the contributor MUST affect final validity and appear in ordered checks
- AND it MUST NOT be evaluated only after rendering or omitted from the decision.
