# Proposal: Backport selected Snix correctness fixes

## Why

Mantle vendors adapted Snix crates without recording one upstream revision. The local crates also contain intentional BLAKE3, postcard, store-prefix, substitution, scheduler, and transport changes.

A bulk upstream sync can remove those boundaries. A focused Gerrit review found specific correctness, trust, filesystem, database, ingestion, and tracing fixes that are absent from the vendored code.

The highest-risk gap lets a remote PathInfo service return valid signed metadata for a different store path than the requested digest. Mantle can persist that result and perform side effects before it detects the identity mismatch.

## What Changes

- Record a selective-backport policy and an upstream disposition ledger.
- Reject a returned remote PathInfo when its store-path digest differs from the requested digest.
- Add a second request-identity guard before Mantle persists remote metadata or emits substitution side effects.
- Decode all zstd frames and preserve binary-cache base paths during URL joins.
- Correct castore directory size accounting and FUSE entry metadata.
- Move redb write-transaction creation into the blocking worker and make cache listing use the writable near service.
- Adopt the upstream filesystem-ingestion and tracing-layer fixes without importing unrelated API refactors.
- Keep conditional and incompatible upstream changes deferred with explicit reopen conditions.

## Dependencies

- ADR 0054 defines selective Snix backports as Mantle’s vendor-maintenance policy.
- Upstream Gerrit changes are review inputs. They do not transfer compatibility or release authority to Snix.

## Non-Goals

- A bulk update to the latest Snix tree.
- A claim that Mantle matches one complete upstream Snix revision.
- Protobuf directory serialization, gRPC restoration, or removal of Mantle’s postcard identities.
- Adoption of unmerged derivation-builder, output-model, combined-store-trait, or concurrent-uploader refactors.
- OCI backend work while Mantle-owned code does not use that backend.

## Impact

- **Vendored files**: selected files under `vendor/snix-store`, `vendor/snix-castore`, and `vendor/snix-tracing`.
- **First-party files**: `crates/crunch-store` remote substitution and compressed pull paths, focused tests, Tracey bridge references, and maintenance evidence.
- **Compatibility**: Mantle keeps its BLAKE3, postcard, configurable store-prefix, iRPC, substitution, and scheduler adaptations.
- **Testing**: every selected fix needs positive and negative coverage, including mutation-before-rejection checks.
