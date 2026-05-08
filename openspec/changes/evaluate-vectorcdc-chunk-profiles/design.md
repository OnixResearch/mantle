# Design: Evaluate VectorCDC chunk profiles

## Context

Crunch delta transfer is castore-native: blobs and chunks are identified by BLAKE3 digests of finalized content bytes, and `delta-transfer` protocol version 1 currently mandates FastCDC with a 128 KiB minimum, 256 KiB average, and 512 KiB maximum. Upstream Snix castore documentation makes the same key distinction: the stable blob identity is BLAKE3 over raw bytes, while physical chunking is a storage/transport concern. VectorCDC is attractive because CDC scanning can dominate ingest, but the paper accelerates hashless CDC families rather than FastCDC itself.

## Goals / Non-Goals

**Goals:**

- Establish a safe research track for evaluating hashless/SIMD CDC candidates.
- Preserve existing FastCDC behavior until evidence justifies a new opt-in profile.
- Define invariants for deterministic, ordered, full-coverage chunk streams.
- Capture enough benchmark evidence to distinguish chunker speed from BLAKE3, compression, object-store, and protocol effects.

**Non-Goals:**

- No default chunker change in this OpenSpec.
- No delta protocol v1 compatibility change.
- No trust in candidate chunk metadata without recomputing/verifying BLAKE3 content.
- No mandatory SIMD dependency in default builds.

## Decisions

### 1. Keep FastCDC as the compatibility baseline

**Choice:** FastCDC remains the default chunk profile and the only protocol v1 interpretation.

**Rationale:** Existing peers and manifests already rely on the v1 profile. A silent change would reduce chunk reuse compatibility and make performance regressions hard to attribute.

**Alternative:** Replace FastCDC directly with RAM/VectorCDC. Rejected because the paper's datasets are not Crunch/Nix store corpora and because chunk-profile changes affect storage/object reuse even though blob identity remains stable.

### 2. Evaluate candidates behind a pure chunker boundary

**Choice:** A candidate chunker returns deterministic chunk offset/length boundaries for finalized blob bytes; upload, compression, manifest persistence, and BLAKE3 verification stay outside the chunker.

**Rationale:** This isolates algorithm evidence and keeps the trust boundary in the existing BLAKE3/castore layer. It also enables positive and negative invariant tests without invoking object storage.

**Alternative:** Prototype inside object-store upload code first. Rejected because it would entangle CDC behavior with I/O costs and increase the chance of persisting invalid metadata.

### 3. Promote only by measured profile evidence

**Choice:** Any candidate adoption must be a later OpenSpec with corpus metrics and explicit platform behavior.

**Rationale:** VectorCDC's claimed speedups are promising, but Crunch needs end-to-end evidence: total ingest time, chunk distribution, dedup/reuse, object count, compression impact, and substitution/read behavior.

## Risks / Trade-offs

**Reduced cross-peer reuse** → Mitigate by preserving v1 FastCDC and requiring profile negotiation for any future candidate.

**Object-count explosion or poor chunk distribution** → Mitigate with corpus metrics and non-final chunk-size bounds before promotion.

**Unsupported SIMD platforms** → Mitigate with scalar fallback or fail-closed opt-in behavior.

**Trust-boundary confusion** → Mitigate by specifying that chunk metadata is advisory for planning; raw blob BLAKE3 and final PathInfo verification remain authoritative.

## Inventory: current FastCDC/default profile seam

- `crates/crunch-delta-core/src/model.rs` defines protocol-v1 runtime chunk bounds as named constants:
  - min: `131_072`
  - avg: `262_144`
  - max: `524_288`
- `crates/crunch-delta-core/src/model.rs::chunk_profile_v1()` is the core source of truth for those bounds.
- `crates/crunch-delta/src/model.rs::chunk_profile_v1()` mirrors the core profile into the `snix_castore::B3Digest`-using facade model.
- `crates/crunch-delta/src/manifest.rs::chunk_profile_for_manifest()` is the manifest-building seam that currently selects the profile for receiver manifests.

## Inventory: negotiation and wire compatibility seam

- `crates/crunch-delta-core/src/negotiation.rs` defines the wire profile shape:
  - `ChunkingAlgorithmWire::FastCdc`
  - `ChunkDigestAlgorithmWire::Blake3`
  - `ChunkProfileWire { chunking, chunk_digest, min_chunk_bytes, avg_chunk_bytes, max_chunk_bytes }`
- `chunk_profile_wire_v1()` maps the runtime v1 profile to the advertised wire profile.
- `NegotiationOffer::protocol_v1()` advertises exactly protocol version 1 and exactly `chunk_profile_wire_v1()`.
- `select_shared_chunk_profile()` currently pins a negotiated version to `chunk_profile_for_version(version)`, so protocol v1 cannot negotiate any alternate chunker/profile without a deliberate future protocol-semantic change.
- `crates/crunch-delta/src/substitution.rs::DeltaCapabilityAdvertisement` carries supported profiles to the cache/substitution facade, and `to_negotiation_offer()` forwards them into core negotiation.

## Inventory: manifest/reuse planning seam

- `crates/crunch-delta/src/manifest.rs` threads a `&ChunkProfile` through manifest walking and chunk-reuse discovery.
- `record_local_blob()`, `record_local_blob_lossy()`, and `record_local_blob_probabilistic()` use the selected profile only as a minimum-size gate before querying seeded chunk metadata.
- These paths trust blob identity separately from chunk reuse: the blob digest is carried as `B3Digest`/`DeltaDigest`, while chunks are opportunistic `known_chunks` evidence for reuse.

## Inventory: physical blobstore chunking seam inherited from Snix/castore

- `vendor/snix-castore/src/blobservice/object_store.rs::chunk_and_upload()` is the concrete physical chunking/upload path for object-store-backed blob service writes.
- It wraps the input reader in `B3HashingReader`, runs `fastcdc::v2020::AsyncStreamCDC::new(&mut b3_r, min, avg, max)`, uploads per-chunk objects keyed by `BLAKE3(chunk bytes)`, then stores a blob metadata object under `BLAKE3(raw finalized blob bytes)`.
- The function already preserves the required trust boundary: chunking affects physical chunk metadata, but the returned/stored blob digest comes from `b3_r.digest()` over the raw byte stream.
- This seam is the right place to introduce a future boundary-only chunker abstraction, because it can keep raw BLAKE3 hashing outside candidate chunker semantics.

## Inventory: existing tests/guardrails

- Core profile bounds are covered by `crates/crunch-delta-core/src/model.rs::tests::chunk_profile_v1_matches_spec`.
- Facade profile bounds are covered by `crates/crunch-delta/src/model.rs::tests::chunk_profile_v1_matches_spec`.
- Wire/runtime profile agreement is covered by `crates/crunch-delta-core/src/negotiation.rs::tests::chunk_profile_wire_v1_matches_runtime_profile` and `crates/crunch-delta/src/planner.rs` facade tests.
- Snix/castore object-store chunk upload has `vendor/snix-castore/src/blobservice/object_store.rs::tests::test_chunk_and_upload`, which asserts the returned digest equals the raw blob digest after chunking/upload.

## Evaluation boundary implication

The first implementation seam should not add VectorCDC to protocol negotiation. It should extract a deterministic chunk-boundary interface below the object-store upload path, with FastCDC as the default implementation and tests that candidate boundaries are ordered, contiguous, non-overlapping, and cover the raw byte stream exactly. Protocol-v1 advertising should continue to expose only `FastCdc/Blake3/131072/262144/524288` until a later evidence-backed promotion OpenSpec changes negotiation semantics.

## I5 prototype boundary

`vendor/snix-castore/src/blobservice/chunker.rs::ExperimentalVectorCdcChunker` is an intentionally non-default candidate behind the `snix-castore/experimental-vectorcdc` Cargo feature. It uses a portable scalar vector-window boundary score so every supported platform has a deterministic fallback; there is no mandatory SIMD dependency, and default builds do not export or compile the candidate. The candidate still returns only physical offset/length boundaries and reuses the same `validate_chunk_boundaries()` invariant gate as FastCDC, so `BLAKE3(raw blob)` remains the only stable blob identity.
