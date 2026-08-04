# Design: opt-in chaptered release transport

## Context

Mantle release evidence is a verified directory with a canonical `manifest.json`. The directory can contain a source archive, several binaries, proof trees, reproducibility data, and external evidence.

A normal `.tar.gz` preserves portability but does not expose role boundaries for direct seeking. `chapter-tgz` writes a normal gzip and tar stream with embedded chapter markers. Standard readers ignore the markers, while a chapter-aware reader can seek to one group.

The first release, `chapter-tgz` 0.1.0, requires Rust 1.91 and uses `flate2` plus `tar`. The upstream repository has no dedicated test files. A local Mantle pilot proved deterministic bytes, standard reader compatibility, random access, and ordinary tgz fallback. It also found that chapter access accepted a one-byte-truncated gzip trailer while a full gzip decoder rejected it.

## Goals

- Provide deterministic pack, inspect, and unpack operations for verified release directories.
- Preserve ordinary gzip and tar compatibility.
- Keep integrity and safe extraction under Mantle control.
- Keep format planning pure and filesystem effects capability-confined.
- Make the maturity and claim boundaries visible.

## Non-Goals

- Change the canonical release evidence format.
- Change OCI, Android image, NAR, upstream source, release source tar, or Aspen export formats.
- Add parallel decompression before representative benchmarks.
- Treat gzip CRC or chapter markers as trusted integrity evidence.

## Decisions

### Decision: Pin the exact published crate

**Choice:** Add `chapter-tgz = "=0.1.0"` to the Mantle binary shell. Record crates.io checksum `71d3b546f2d916ddf609e8f8e271c93c9dcd66d7500a87ee8d81d4c159839d7c` and release commit `0090e5d002188228a6c94742f99d0c1983e52f6a` in lifecycle and reference documentation.

**Rationale:** The crate is new and defines byte-level transport behavior. An exact release and Cargo checksum make review and reproduction finite.

### Decision: Keep `chapter-tgz` out of the no-std core

**Choice:** `crunch-release-core` owns transport entry types, chapter plans, canonical index and receipt models, bounds, and deterministic validation. The Mantle binary owns files, capabilities, tar headers, compression, BLAKE3 measurement, and CLI output.

**Rationale:** Planning and validation remain testable without a filesystem or decompressor. The no-std core does not gain transport or I/O dependencies.

### Decision: Use deterministic path groups

**Choice:** Chapter zero contains the reserved transport index followed by `manifest.json`. Other members use sorted deterministic groups:

- `source/<...>` uses one source chapter.
- Each immediate `binaries/<name>` child uses one binary chapter.
- Other paths use one chapter for each top-level component.

The reserved index path is `__mantle_release_transport_index__.json`. Input trees containing that path are rejected.

**Rationale:** The rule works for current and future manifest fields, keeps each binary selectable, and does not duplicate release semantics in transport code.

### Decision: Bind all compressed bytes with a detached BLAKE3 receipt

**Choice:** Pack writes a canonical detached receipt. Inspect and unpack require that receipt and compare the complete compressed archive BLAKE3 before opening chapters.

The receipt binds archive size, archive BLAKE3, source manifest BLAKE3, index BLAKE3, chapter count, member count, format version, dependency version, claim scope, and non-claims.

**Rationale:** Chapter reads do not validate the final gzip trailer. A BLAKE3 over compressed bytes catches truncation, trailer changes, and payload replacement without full decompression.

The receipt still needs authentication from the publication or handoff layer. A matching archive and attacker-written receipt do not establish authority.

### Decision: Validate metadata before extraction

**Choice:** Unpack uses two passes over the seekable archive. The first pass reads bounded metadata from every chapter and compares it with the index. It then runs the pure tree planner for paths, kinds, modes, sizes, duplicates, limits, and links.

The second pass writes files, directories, and links through capability-relative no-follow operations into a private sibling stage. It removes the reserved index, runs normal release verification, and uses Linux atomic no-replace rename for publication.

**Rationale:** Complete validation before mutation prevents archive order from granting authority. Capability-relative writes and no-replace publication preserve the existing release confinement model.

### Decision: Preserve standard-reader compatibility

**Choice:** The output remains one valid gzip stream and one valid tar stream. A standard extractor sees the reserved index plus all original release members.

Mantle unpack omits the reserved index so the published directory recreates the original verified release tree.

**Rationale:** Existing gzip and tar tools remain useful for inspection and recovery. Transport metadata stays visibly separate from canonical release evidence.

### Decision: Keep the first reader sequential

**Choice:** The first shell uses a seekable file and chapter jumps for inspection. Complete metadata validation and extraction are sequential. A `FileExt::read_at` `IndependentRead` adapter and parallel decompression remain deferred.

**Rationale:** The first adoption must establish format safety, determinism, and workload evidence before concurrency adds complexity.

### Decision: Publish outputs without replacement

**Choice:** Pack stages archive and receipt files, synchronizes them, then publishes both without replacement. If receipt publication fails after archive publication, the shell removes only the archive it created.

Unpack stages a complete verified directory beside the destination and uses atomic no-replace rename.

**Rationale:** Failed or concurrent operations must not expose partial output or replace another publisher's path.

## Named Limits

The core defines named limits for archive members, chapters, path bytes, index bytes, one member size, and total uncompressed bytes. The limits use checked arithmetic and appear in diagnostics.

The initial values align with the existing release tree entry bound and permit large research bundles. They are compatibility constants for transport version 1.

## Risks / Trade-offs

- New upstream code has limited independent use evidence.
- Each chapter adds compressed marker overhead.
- Standard extraction leaves the reserved transport index in the output.
- Two-pass unpack reads compressed data more than once.
- A detached receipt needs an authenticated parent handoff for adversarial transport authority.
- Sequential extraction does not yet prove a parallel performance gain.

## Validation Strategy

- Core tests cover deterministic grouping, canonical models, bounds, duplicates, missing control members, reserved-path collisions, and invalid links.
- Shell tests cover deterministic bytes, standard gzip/tar readers, chapter zero inspection, full round trip, internal links, and normal release verification.
- Negative shell tests cover truncation, digest mismatch, malformed index, legacy tgz, path escape, link escape, unsupported entry types, source drift, bounds, and destination races.
- CLI tests cover argument parsing plus human and JSON output contracts.
- Lifecycle evidence records focused Cargo results, formatting, Clippy, Tiger Style, Cairn gates, Tracey coverage, and any external Nix blockers.
