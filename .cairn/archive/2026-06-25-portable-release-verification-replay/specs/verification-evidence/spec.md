## ADDED Requirements

### Requirement: Portable release verification replay

r[verification_evidence.portable_release_verification_replay] Mantle MUST provide durable evidence when a release verification claim is intended to be replayable from copied release artifacts rather than the original source checkout.

#### Scenario: copied artifact set verifies successfully

GIVEN a release evidence bundle and required deterministic-release proof sidecars are copied to a fresh scratch or export directory
WHEN an operator runs release verification from that copied artifact set with required deterministic-release and provider fixed-point gates enabled
THEN verification MUST succeed using only the copied release bundle, copied deterministic proof receipt, copied sandbox isolation evidence, and bundle-local provider proof material.
AND the evidence transcript MUST record the scratch root, command, verifier status, proof digests, and bounded non-claims.

#### Scenario: missing copied proof material fails closed

GIVEN the copied artifact set omits a deterministic proof receipt or sandbox isolation evidence file required by the verification command
WHEN the operator runs verification with `--require-deterministic-release`
THEN Mantle MUST fail closed with a deterministic diagnostic rather than reporting deterministic-release eligibility.
AND the failure evidence MUST show the missing proof class without weakening the requested gate.

#### Scenario: portable replay avoids ambient source claims

GIVEN release verification succeeds from a copied artifact set
WHEN the result is reported as portable evidence
THEN the claim MUST be limited to replaying the recorded release proof material from copied artifacts.
AND it MUST NOT claim source checkout cleanliness, full bootstrap reproducibility, compiler correctness, full Cargo compatibility, or deploy success unless separate evidence proves those claims.
