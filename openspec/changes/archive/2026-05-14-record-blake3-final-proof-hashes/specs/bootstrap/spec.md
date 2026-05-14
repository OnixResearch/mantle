## ADDED Requirements

### Requirement: External fixed bundle proofs expose BLAKE3 final proof hashes

Crunch MUST record a BLAKE3 final proof hash for external fixed bundle proofs even when the fetcher or upstream tooling requires SHA-256 compatibility hashes.
ID: bootstrap.external-fixed-bundle.final-proof-blake3

The final proof hash MUST be a Crunch-owned BLAKE3 digest over canonical proof metadata that binds the fixed source bundle identity, compatibility hashes, build recipe, output artifact identity, and smoke evidence. SHA-256 SRI values MAY remain as fetcher or Cargo compatibility hashes, but they MUST NOT be the only recorded proof digest.

#### Scenario: Clankers root proof carries BLAKE3 digest

- GIVEN `packages/clankers/clankers-root-bundle.json` records SHA-256 SRI compatibility hashes
- WHEN the Clankers root bundle proof is inspected
- THEN it records `final_proof_hash_blake3`
- AND the referenced proof file records the same final BLAKE3 proof hash
- AND the proof also records the BLAKE3 digest of the external bundle archive or output artifact.
