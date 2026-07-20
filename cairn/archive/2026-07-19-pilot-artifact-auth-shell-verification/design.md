# Design: exact artifact-auth shell verification

## Completion contract

The pilot is complete when a real Mantle `KeyPair` signs canonical bytes produced by the same pure mapping consumed by dual-run evaluation, pinned `artifact-auth-ed25519` independently verifies those exact bytes, all carrier identities are recomputed, legacy record-signature and currentness admission is enforced before signing, adversarial fixtures fail with stable classes, and full Cargo/Nix/Cairn rails pass from the isolated worktree.

False completion includes reusing the legacy action-result signature, promoting a legacy verification boolean, signing `result_ref`, JSON, OCI, or another product preimage, trusting carrier hashes without recomputation, accepting caller-supplied authorization booleans, using a sibling or floating dependency, admitting standalone authority, suppressing unrelated failures, or including the source-seed worktree.

## Portfolio-search frame

Exact goal: produce bounded Mantle product-shell evidence for one standalone statement without changing product authority.

Observable evidence: exact statement/public-key/signature BLAKE3 references; raw signature bytes and stable failure class; legacy record-signature authorization reference; currentness binding; positive and negative tests; immutable dependency identity; clean full checks.

Audit risks: preimage confusion, Nix key-label substitution, full-key aliasing, detached-signature reuse, stale/revoked key signing, malformed carrier acceptance, decision drift, unrelated-failure false parity, authority widening, and private-source pin drift.

Budget: three correlated architecture lenses, two implementation/audit rounds, one advisory review attempt, exact local source plus published Molten/Valence pilots as authority references, and repository Cargo/Nix/Cairn tools as deterministic authority. Allowed outcomes are validated pilot, exact blocker, exhausted search, or user decision required.

## Approach registry

| Family | Mechanism | Claim | Artifact | State |
|---|---|---|---|---|
| action-result shell | Reuse the existing Nix-compatible action-result `KeyPair`, but sign separately mapped standalone bytes only after validating the admitted legacy record, record signature, key identity, and currentness | Matches the accepted Mantle adapter without transferring authority | `crates/crunch-build/src/artifact_auth.rs` | active |
| OCI registry shell | Attach standalone signing to registry trust-statement signing | Could reuse registry keys, but signs an OCI/repository subject rather than the action-result subject and would conflate registry authority | `src/oci_registry.rs` | falsified |
| release-attestation shell | Attach standalone signing to release evidence/attestation | Could create release-level evidence, but bypasses action-result policy and widens the pilot into release authority | `src/release_attestation.rs` | falsified |

The exploration passes are serial and correlated because isolated workers are unavailable. The action-result family survives because it alone binds the existing adapter's subject, output parents, publication-policy context, producer, and key.

## Functional core and shell

`crunch-action-result-core` owns pure deterministic scope and statement mapping. `crunch-build` owns the key adapter and composition shell. A pure admission helper validates the legacy decision, exact signed record, existing record signature, producer/key match, currentness, full public-key identity, and currentness reference before the thin shell signs. Evaluation reconstructs every carrier identity and calls pinned `artifact-auth-ed25519`; it never turns legacy verification into standalone proof.

## Evidence and authority

The authorization reference is a BLAKE3 observation over the existing detached action-result signature carrier. It is external evidence and is not part of the standalone signature preimage. The report fixes `legacy_authoritative = true`, `standalone_authority_admitted = false`, and `rollback_available = true`. Operational key discovery, key-file I/O, trust/currentness freshness, registry authorization, publication, persistence, and release effects remain outside this pure/testable pilot.
