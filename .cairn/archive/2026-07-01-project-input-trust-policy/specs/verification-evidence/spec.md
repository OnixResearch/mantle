## ADDED Requirements

### Requirement: Project input trust claims are evidence bounded [r[verification_evidence.project_input_trust_claims]]

Mantle MUST keep project input trust claims bounded to the configured policy and inspected trust evidence. A successful trust check MUST NOT be described as build reproducibility, compiler correctness, release validity, forge trust, or global upstream authenticity without separate evidence.

#### Scenario: Trust claim cites verifier evidence [r[verification_evidence.project_input_trust_claims.scenario.claim]]

- GIVEN a task, report, attestation, status reply, or evidence file claims a project input satisfied trust policy
- WHEN the claim is made
- THEN it MUST cite current refresh, verification, or attestation evidence identifying the input, policy, verifier kind, signer identity, and digest binding
- AND the claim MUST be no broader than that evidence.

#### Scenario: Missing trust evidence blocks trust claim [r[verification_evidence.project_input_trust_claims.scenario.missing]]

- GIVEN an input has only a lockfile hash or stale/uninspected signature material
- WHEN status or evidence is summarized
- THEN Mantle MUST NOT claim the input is signed, trusted, or authenticated
- AND the report MUST state that trust evidence is missing or not proven.
