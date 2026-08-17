## ADDED Requirements

### Requirement: External witness expansion

r[verification_evidence.external_witness_expansion] Mantle MUST record multi-witness release evidence in a way that preserves cryptographic validity, digest agreement, and independence policy before making stronger external-rebuild claims.

#### Scenario: valid independent witnesses count toward quorum

GIVEN a release has multiple returned witness sidecars signed by trusted keys
WHEN Mantle evaluates witness expansion evidence
THEN each counted witness MUST bind witness identity, signer key name, independence domain, host class, source acquisition mode, matched output digest set, and release attestation digest
AND quorum counts MUST include only witnesses accepted by the configured independence policy.

#### Scenario: invalid witnesses remain visible but uncounted

GIVEN returned witness material has an unknown key, bad signature, stale request, wrong release digest, digest mismatch, revoked identity, or same independence domain as an already counted witness
WHEN Mantle emits witness expansion evidence
THEN the witness MUST be classified as skipped or failed with a deterministic reason
AND it MUST NOT inflate independent agreement or global reproducibility quorum.

#### Scenario: witness summaries stay bounded

GIVEN witness expansion evidence is available for a release
WHEN Mantle renders operator-facing summaries
THEN the summary MUST name the quorum policy, counted witness count, skipped witness count, failed witness count, and accepted identities
AND it MUST NOT claim broader reproducibility than the surfaces and policy covered by the witnesses.
