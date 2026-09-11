# Proposal: Add lock-driven vendor fetches

## Why

Mantle's fresh-clone and self-build story carries vendored dependency trees
(`vendor-deps/` with cargo `.cargo-checksum.json` SHA-256s) inside the source
bundle. The recorded profiles run to gigabytes, hydration must bind and
materialize them, and every dependency bump re-stages the tree. The lock file
already contains the hashes Mantle ends up verifying; the vendored tree is
derived state shipped as source.

The reviewed external reference (`mic92/repkgs`, commit `1cd7b8b`, see
`evidence/repkgs-review.md`) shows the alternative: a small producer reads
`Cargo.lock` out of the already-fetched source and emits one fixed-output
fetch per artifact using the hash the lock already has. Nothing is vendored
into the repository, evaluation never sees the lock, and a 3,000-line lock
costs no evaluation. Mantle owns its whole eval→build pipeline, so it needs
no escape hatch to do this natively.

## What Changes

- Define a producer-derivation contract: a bounded producer reads a fetched
  source's lock file and emits one fixed-output fetch derivation per
  artifact plus one assembling derivation, admitted through Mantle's
  dynamic-derivation admission. r[mantle.lock_vendor_fetch.producer_derivation]
- Reuse upstream lock hashes: artifact hashes come from the lock file; the
  producer never invents hashes, and dependencies without a usable hash are
  rejected or routed to shared tables.
  r[mantle.lock_vendor_fetch.lock_hash_reuse]
- Add shared lock tables for hash-less ecosystems: one sorted table with
  union-merge semantics, where one package's entries do not rebuild another.
  r[mantle.lock_vendor_fetch.shared_lock_tables]
- Make vendored trees excludable from source bundles: fresh-clone and
  hydration profiles record lock-driven fetch as the acquisition path with
  verified identity. r[mantle.lock_vendor_fetch.bundle_payload_reduction]
- Fail closed: wrong hash, tampered lock entry, missing artifact, and
  over-bound inputs are typed denials with negative controls.
  r[mantle.lock_vendor_fetch.negative_controls]

## Impact

- **Immediate consumer**: `mantle self-build` staging, the fresh-clone-inputs
  and fixed-point source-bundle profiles, and the checked vendor-input
  validation.
- **Immediate outcome**: source bundles stop carrying vendored Cargo trees;
  acquisition is driven by the lock with per-artifact verification.
- **Durable capability**: a general lock-to-fetches producer pattern for any
  ecosystem whose lock carries hashes (Cargo first; npm-style integrity
  records as later families).
- **Maintenance owner**: Mantle source-tooling owner, covering the fetch
  producer, the bundle planner, and `source bundle` profile modes.
- **Repeatability evidence**: producer receipts binding lock identity to
  emitted fetch derivations, per-artifact hash verification, bundle-size
  comparisons, and negative fixtures.
- **Compatibility**: existing bundles with vendored trees keep building;
  lock-driven acquisition is a new profile mode before it becomes default.

## Scope

The change covers the producer derivation for Cargo locks, the shared-table
format, the assembling derivation, profile exclusion of vendored trees,
receipts, and negative controls.

## Out of Scope

- Updating lock files themselves (owned by project refresh and
  `add-bootstrap-source-pins` applies).
- Ecosystems beyond Cargo in the first slice; npm/pnpm-style integrity
  records are follow-up families under the same contract.
- Substitution or sharing of fetched artifacts between machines (existing
  substitution paths apply unchanged).
- Proof-of-compiler-correctness or dependency-trust claims; hashes are
  integrity, not trust.

## Success Criteria

- A fresh-clone profile without vendored trees materializes the same vendor
  layout with identical content hashes, verified per artifact.
- The producer's emitted fetch derivations use exactly the lock file's
  hashes; a tampered lock entry fails closed.
- Bundle payload shrinks by the vendored tree size for the adopted profile,
  recorded in the profile receipt.
- Evaluation never reads the lock file; nothing is fetched at evaluation
  time.
