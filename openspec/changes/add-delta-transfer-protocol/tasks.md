# Tasks: Add delta transfer protocol

## Phase 1: Spec the concept for crunch

- [x] Write proposal describing how DeltaNAR-style reuse maps onto crunch
- [x] Add a `delta-transfer` delta spec instead of copying a NAR-specific
      format directly
- [x] Record binary-cache integration points and fallback behavior
- [x] Write design notes covering castore-native reuse, store-prefix
      compatibility, and trust preservation

## Phase 2: Protocol model and ownership

- [ ] Decide where the delta protocol layer lives: `crunch-store`, vendored
      `snix-castore`, or a dedicated protocol crate
- [ ] Define receiver compatibility manifest types, requested-scope bounds, and
      metadata-only construction rules; manifest probes must use
      `PathInfoService::get_references()` or an equivalent metadata-only path,
      not `get()`
- [ ] Add unit tests for manifest construction: no content download, stale
      entries treated as absent, and prefix mismatch rejection
- [ ] Define protocol version negotiation and the initial FastCDC/BLAKE3 chunk
      profile wire types
- [ ] Add unit tests for negotiation: version agreement, chunk-profile
      agreement, and version-mismatch rejection
- [ ] Define the 4-step HTTP interaction under the existing cache authority:
      request -> candidate identities -> receiver has-set -> streamed content

## Phase 3: Substitution integration

- [ ] Add capability negotiation so a trusted substituter can advertise delta
      transfer support
- [ ] Implement coarsest-first reuse planning for whole outputs, unchanged
      subtrees, whole blobs, and chunks
- [ ] Add bounded cross-output blob-digest tracking so shared blobs are sent at
      most once per transfer session
- [ ] Verify the reconstructed castore root digest matches the final signed
      `PathInfo` before accepting a delta-transferred output
- [ ] Keep successfully verified chunks and blobs from interrupted transfers in
      castore while discarding incomplete session state
- [ ] Fall back cleanly to full-artifact substitution when capability,
      negotiation, or reuse planning does not line up

## Phase 4: Verification and measurement

- [ ] Add planner unit tests for whole-output, subtree, blob, chunk, and
      cross-output reuse cases
- [ ] Add unit tests proving finalized CA outputs are planned from post-rewrite
      bytes only
- [ ] Add an integration test that builds version 1, pre-seeds reusable
      receiver content, serves version 2 from a delta-capable cache, and
      verifies only missing content is transferred
- [ ] Add an integration test that rejects a delta hit with untrusted
      signatures and falls back according to substitution policy
- [ ] Add an integration test that uses a legacy cache with no delta endpoints
      and falls back transparently to ordinary substitution
- [ ] Benchmark transferred bytes and wall-clock behavior against full-artifact
      substitution on representative closures

## Validation

- [x] Run `openspec validate add-delta-transfer-protocol`
