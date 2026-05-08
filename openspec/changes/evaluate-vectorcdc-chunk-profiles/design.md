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
