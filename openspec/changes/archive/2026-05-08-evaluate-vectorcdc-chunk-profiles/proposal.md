# Evaluate VectorCDC chunk profiles

## Why

Crunch delta transfer currently fixes protocol version 1 to FastCDC with a 128 KiB / 256 KiB / 512 KiB chunk profile. The VectorCDC paper shows that hashless CDC algorithms such as RAM, AE, and MAXP can be accelerated with SIMD while preserving their chunk boundaries relative to scalar implementations. That makes them plausible candidates for faster castore physical chunking and delta planning, but only if measured on Crunch/Nix-store-like corpora and kept below the raw-content BLAKE3 identity boundary.

## What Changes

- Add an evaluation-track OpenSpec for optional VectorCDC/hashless CDC chunk-profile candidates.
- Preserve FastCDC as the default and as delta protocol v1 behavior.
- Require a deterministic pluggable chunker boundary before any candidate can be benchmarked or promoted.
- Require reproducible evidence for ingest throughput, chunk distribution, dedup/reuse, storage/object-count effects, and platform behavior before a future protocol profile or default can change.

## Scope

In scope: benchmark harness design, chunker invariants, corpus selection, optional candidate prototype boundaries, and promotion evidence requirements.

Out of scope: changing delta protocol v1, changing blob/file identity, importing SIMD dependencies into default builds, or promoting VectorCDC/RAM/AE/MAXP without measured Crunch evidence.

## Verification

- `openspec validate evaluate-vectorcdc-chunk-profiles --strict`
- Future implementation evidence must compare current FastCDC against any candidate on representative local corpora and prove identity/trust semantics remain unchanged.
