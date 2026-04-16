# Add delta transfer protocol

## Why

Crunch can already substitute complete outputs from remote caches, but that path
still moves whole artifacts even when the receiver already has most of the same
bytes. DeltaNAR shows useful ideas here: compare desired closure state against
receiver state, reuse whole directories and files when possible, and fall back
to sub-file chunk transfer when only part of a file changed.

Crunch should adopt those ideas without copying DeltaNAR literally. Crunch's
native data model is castore blobs plus directory nodes, not NAR archives. It
uses BLAKE3 content identities, native artifact attestations, signed `PathInfo`,
and configurable logical store prefixes. A crunch design that blindly copies a
Nix-specific DNAR payload would duplicate storage layers and fight the current
architecture.

## What Changes

- add a new `delta-transfer` spec for castore-native delta transport
- define a receiver compatibility manifest based on logical store identity and
  reusable castore content, scoped to the requested transfer instead of a
  whole-store inventory
- define negotiated protocol versioning plus a pinned initial chunk profile:
  FastCDC with 128 KiB minimum, 256 KiB average, and 512 KiB maximum chunk
  size, with BLAKE3 chunk identities
- define coarsest-first reuse: whole artifact, unchanged subdirectory, whole
  blob, then content-defined chunk, including reuse across multiple outputs in
  one closure transfer
- integrate delta transfer as an optional substitution path by extending the
  current remote-cache HTTP flow with delta manifest and stream endpoints, with
  fallback to the current full-artifact substitution flow
- require delta-transferred paths to keep the same trust checks and attestation
  outcomes as ordinary substitution

## Capabilities

### New Capabilities

- `castore-native-delta-transfer`: crunch peers can exchange only missing store
  content without introducing a second NAR-specific storage format
- `delta-aware-substitution`: remote substitution can prefer reuse-aware
  transfer when both sides support it
- `store-prefix-safe-delta-negotiation`: peers reject delta reuse across
  incompatible logical store prefixes

## Impact

- **Files**: new change-local spec under
  `openspec/changes/add-delta-transfer-protocol/specs/delta-transfer/spec.md`
  plus a binary-cache delta spec
- **Architecture**: future work will extend substitution around castore and
  `PathInfo` instead of adding a separate deployment-only data plane
- **Dependencies**: no new hash family or Nix-specific wire format is required
  by this change
- **Testing**: future implementation needs coverage for prefix mismatch,
  manifest accuracy without content downloads, multi-level and cross-output
  reuse, finalized CA outputs, fallback behavior, and trust parity with
  ordinary substitution

## Non-Goals

- clone DeltaNAR's DNAR protobuf or NAR-on-the-wire format byte-for-byte
- hardcode `/nix/store` assumptions into delta negotiation
- weaken signature verification or attestation generation for substituted paths
- ship a full deployment UX in the same change
