# Adopt chaptered release transport

## Why

Mantle release evidence is a verified directory. Moving a large bundle currently requires directory-aware transfer or a separate archive step that loses release-role boundaries.

`chapter-tgz` creates an ordinary-compatible `.tar.gz` stream with embedded chapter boundaries. Mantle can use those boundaries for fast manifest inspection and deterministic role grouping without changing the canonical release directory or manifest.

The upstream crate is new. Version `0.1.0` was published on 2026-08-02 and has limited independent use evidence. Mantle therefore needs an opt-in format, an exact dependency pin, local negative tests, and explicit non-claims.

## What Changes

- Add an opt-in `mantle release transport` command group for pack, inspect, and unpack operations.
- Keep the verified release directory and `manifest.json` as the canonical release evidence.
- Add a pure `crunch-release-core` planner for deterministic chapter grouping, indexes, receipts, limits, and diagnostics.
- Add a filesystem shell that uses `chapter-tgz` only after normal release verification succeeds.
- Bind each compressed archive to a detached BLAKE3 transport receipt.
- Stage unpacked content, validate every member without following links, run normal release verification, and publish with no replacement.
- Preserve ordinary gzip and tar reader compatibility.
- Document the external dependency and add it to the Mantle README references.

## Out of Scope

- Replacing canonical OCI layers, Android image archives, NARs, upstream source archives, or the existing release source tar.
- Changing the `mantle-source-bundle-v1` JSON format.
- Replacing Aspen deterministic `tar.zst` exports.
- Parallel decompression or remote range transport before representative benchmarks exist.
- Claiming that transport validity proves build correctness, source correctness, reproducibility, or release eligibility.

## Impact

- **Planned files**: `Cargo.toml`, generated `Cargo.lock`, `crates/crunch-release-core`, release transport shell and CLI files, tests, operator documentation, ADR, README references, Cairn lifecycle files, and Tracey references.
- **Testing**: deterministic pack, standard gzip/tar compatibility, random chapter inspection, safe round-trip unpack, legacy tgz rejection, truncation, digest mismatch, malformed index, path escape, symlink escape, bounds, no-clobber publication, and existing release verification.
- **Compatibility**: the verified directory remains canonical. The new chaptered `.tgz` is opt-in and versioned.
- **Dependency**: exact crate `chapter-tgz = "=0.1.0"`, crates.io checksum `71d3b546f2d916ddf609e8f8e271c93c9dcd66d7500a87ee8d81d4c159839d7c`, release commit `0090e5d002188228a6c94742f99d0c1983e52f6a`.
