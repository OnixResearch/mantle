# Design: VectorCDC chunk-profile evaluation inventory

## Current FastCDC/default profile seam

- `crates/crunch-delta-core/src/model.rs` defines protocol-v1 runtime chunk bounds as named constants:
  - min: `131_072`
  - avg: `262_144`
  - max: `524_288`
- `crates/crunch-delta-core/src/model.rs::chunk_profile_v1()` is the core source of truth for those bounds.
- `crates/crunch-delta/src/model.rs::chunk_profile_v1()` mirrors the core profile into the `snix_castore::B3Digest`-using facade model.
- `crates/crunch-delta/src/manifest.rs::chunk_profile_for_manifest()` is the manifest-building seam that currently selects the profile for receiver manifests.

## Negotiation and wire compatibility seam

- `crates/crunch-delta-core/src/negotiation.rs` defines the wire profile shape:
  - `ChunkingAlgorithmWire::FastCdc`
  - `ChunkDigestAlgorithmWire::Blake3`
  - `ChunkProfileWire { chunking, chunk_digest, min_chunk_bytes, avg_chunk_bytes, max_chunk_bytes }`
- `chunk_profile_wire_v1()` maps the runtime v1 profile to the advertised wire profile.
- `NegotiationOffer::protocol_v1()` advertises exactly protocol version 1 and exactly `chunk_profile_wire_v1()`.
- `select_shared_chunk_profile()` currently pins a negotiated version to `chunk_profile_for_version(version)`, so protocol v1 cannot negotiate any alternate chunker/profile without a deliberate future protocol-semantic change.
- `crates/crunch-delta/src/substitution.rs::DeltaCapabilityAdvertisement` carries supported profiles to the cache/substitution facade, and `to_negotiation_offer()` forwards them into core negotiation.

## Manifest/reuse planning seam

- `crates/crunch-delta/src/manifest.rs` threads a `&ChunkProfile` through manifest walking and chunk-reuse discovery.
- `record_local_blob()`, `record_local_blob_lossy()`, and `record_local_blob_probabilistic()` use the selected profile only as a minimum-size gate before querying seeded chunk metadata.
- These paths trust blob identity separately from chunk reuse: the blob digest is carried as `B3Digest`/`DeltaDigest`, while chunks are opportunistic `known_chunks` evidence for reuse.

## Physical blobstore chunking seam inherited from Snix/castore

- `vendor/snix-castore/src/blobservice/object_store.rs::chunk_and_upload()` is the concrete physical chunking/upload path for object-store-backed blob service writes.
- It wraps the input reader in `B3HashingReader`, runs `fastcdc::v2020::AsyncStreamCDC::new(&mut b3_r, min, avg, max)`, uploads per-chunk objects keyed by `BLAKE3(chunk bytes)`, then stores a blob metadata object under `BLAKE3(raw finalized blob bytes)`.
- The function already preserves the required trust boundary: chunking affects physical chunk metadata, but the returned/stored blob digest comes from `b3_r.digest()` over the raw byte stream.
- This seam is the right place to introduce a future boundary-only chunker abstraction, because it can keep raw BLAKE3 hashing outside candidate chunker semantics.

## Existing tests/guardrails

- Core profile bounds are covered by `crates/crunch-delta-core/src/model.rs::tests::chunk_profile_v1_matches_spec`.
- Facade profile bounds are covered by `crates/crunch-delta/src/model.rs::tests::chunk_profile_v1_matches_spec`.
- Wire/runtime profile agreement is covered by `crates/crunch-delta-core/src/negotiation.rs::tests::chunk_profile_wire_v1_matches_runtime_profile` and `crates/crunch-delta/src/planner.rs` facade tests.
- Snix/castore object-store chunk upload has `vendor/snix-castore/src/blobservice/object_store.rs::tests::test_chunk_and_upload`, which asserts the returned digest equals the raw blob digest after chunking/upload.

## Evaluation boundary implication

The first implementation seam should not add VectorCDC to protocol negotiation. It should extract a deterministic chunk-boundary interface below the object-store upload path, with FastCDC as the default implementation and tests that candidate boundaries are ordered, contiguous, non-overlapping, and cover the raw byte stream exactly. Protocol-v1 advertising should continue to expose only `FastCdc/Blake3/131072/262144/524288` until a later evidence-backed promotion OpenSpec changes negotiation semantics.
