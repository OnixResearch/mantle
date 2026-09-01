# ADR 0101: Export full-bootstrap parity as an independent bundle

## Status

Accepted (2026-09-01)

## Context

The V98 fixed-point proof records complete local root action trust. The initial
parity promotion bound four summary files. That boundary could verify aggregate
counts, but it could not reconstruct each native row, StageX action, Rust
provider stage, or Mantle Rust-unit action outside the proof output tree.

The full decoded audits are large. The Rust-provider audit is about 554 MB, and
the StageX audit is about 88 MB. Committing the decoded files would make review
and transport difficult.

A checker that imports the parity collector shares its implementation defects.
It is not an independent verifier.

## Decision Drivers

- Preserve every required V98 action and audit event.
- Keep repository transport bounded.
- Bind both transport bytes and decoded evidence bytes.
- Verify the bundle without importing the collector core.
- Reject absolute bundle paths and target authority.
- Keep build-witness policy separate from bootstrap status.
- Preserve BLAKE3 as the Mantle-owned identity algorithm.

## Decision

Export the accepted evidence as
`bootstrap/evidence/full-bootstrap-parity-v98/`.

Compress each JSON member with pinned zstd. The manifest records the compressed
BLAKE3, decoded BLAKE3, compressed size, decoded size, schema, role, and safe
relative source path.

Use `scripts/export-source-built-parity-bundle.rs` as the bounded export shell.
It streams each input, publishes through an absent staging directory, and does
not alter the V98 source evidence.

Use `scripts/check-source-built-parity-promotion.rs` as a separate verifier. It
does not import `src/source_built_parity_promotion.rs`. It checks five native
rows, five action adapters, all exported audits, 1,914 actions, 478,870 events,
and witness-policy separation.

Bind the bundle manifest, verification receipt, verifier source, and exporter
source in the compatibility descriptor by BLAKE3. A release can carry the
verification receipt as external evidence. That handoff does not select or
satisfy witness quorum.

## Alternatives Considered

### Keep only aggregate root counts

Rejected. Aggregate counts do not let an independent verifier inspect every
underlying action domain.

### Commit decoded audits

Rejected. The decoded files are too large for practical repository transport.

### Reuse the collector core in the standalone checker

Rejected. Shared code would preserve correlated validation defects.

### Make witness quorum part of bootstrap promotion

Rejected. Bootstrap provenance and social rebuild policy prove different facts.

## Consequences

- The committed bundle is about 2.9 MiB.
- The checker decodes and validates the complete large audits at run time.
- The descriptor gains independent file, semantic, verifier, and exporter
  identities.
- Release evidence can carry the receipt without changing bootstrap-axis status.
- The claim remains bounded to the recorded StageX-to-Mantle V98 fixed point.
