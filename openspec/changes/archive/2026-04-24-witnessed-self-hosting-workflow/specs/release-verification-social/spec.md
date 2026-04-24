## ADDED Requirements

### Requirement: CLI MUST scaffold verifier-local release policy artifacts

Crunch MUST provide `crunch attest policy-init <verification-dir>` to write the
first-phase `policy.json` and `revocations.json` files for a verification
directory without requiring the operator to hand-author JSON.
ID: release.verification.social.policy.scaffold.cli

The command MUST:
- refuse to overwrite existing policy or revocation files unless an explicit
  overwrite flag is set,
- support at least `self-proof-only` and `single-witness` profiles,
- require explicit trusted release signer names for all profiles,
- require explicit trusted witness identities for witness-count profiles, and
- write deterministic JSON that `crunch attest release-verify` can consume
  directly.

#### Scenario: Self-proof-only profile initializes zero-witness policy

- GIVEN a verification directory with a release attestation and no policy files
- WHEN the operator initializes the `self-proof-only` profile with one trusted
  release signer
- THEN `policy.json` sets `min_matching_witnesses` to `0`
- AND `policy.json` leaves `trusted_witness_signers` empty
- AND `revocations.json` exists with empty revocation arrays

#### Scenario: Single-witness profile initializes quorum policy

- GIVEN a verification directory with a release attestation and no policy files
- WHEN the operator initializes the `single-witness` profile with one or more
  trusted witness identities
- THEN `policy.json` sets `min_matching_witnesses` to `1`
- AND `policy.json` sets `independence_field` to `witness_identity`
- AND `policy.json` records the configured trusted witness identities

#### Scenario: Existing policy artifacts are not clobbered by default

- GIVEN a verification directory with existing `policy.json` or
  `revocations.json`
- WHEN the operator initializes policy without an explicit overwrite flag
- THEN the command exits non-zero
- AND the existing policy artifacts remain unchanged
