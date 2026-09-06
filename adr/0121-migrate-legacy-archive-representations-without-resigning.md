# ADR 0121: Migrate legacy archive representations without re-signing

## Status

Accepted for the explicit, offline migration tool. Ordinary cache and archive import behavior is unchanged.

## Context

An existing eleven-object tool closure failed transport admission. Final NAR bytes matched their signatures, but all stored castore nodes used doubled descendant counts. Four objects also used Mantle's marker-normalized CA instead of a final-NAR CA.

The operator forbids Stage0 builds and replacement of the existing Mantle executable. Re-signing guessed metadata, dropping references, deleting CA, and disabling integrity checks are not acceptable repairs.

## Decision

Use a standalone Cargo-script adapter in this repository. It consumes one explicitly pinned native archive and emits a separate native archive plus a migration receipt. It does not open store databases, discover keys, invoke builders, restore filesystem payloads, or import its output.

`crunch-repair-core::legacy_archive` owns directory-migration admission and closed-graph decisions. The adapter reuses vendored `nix-compat` for NAR decoding, signature fingerprints, signature verification, and CA path derivation. The bounded postcard encoder implements the documented castore directory wire shape without opening castore services.

Before any output publication, the tool requires:

1. The operator's input-archive BLAKE3, exact roots, prefix, and explicit public verification keys.
2. Every signature, final NAR size/hash, and payload BLAKE3 to pass.
3. Every declared CA to derive the original path and match the final NAR or the fixed `out` marker normalization.
4. Each recorded node to match either the current reconstruction or the exact doubled-count reconstruction from those same bytes.
5. Complete, duplicate-free reference closure with no unreachable records, malformed frames, lossy metadata conversion, or trailing bytes.

The sole permitted field change is `PathInfo.node`. The final NAR bytes, paths, signatures, references, deriver, and CA remain unchanged. The receipt retains both node identities and the input/output archive digests.

The shell writes an unpublished temporary directory, then uses Linux `renameat2(RENAME_NOREPLACE)` to publish the archive and receipt together. It never replaces a destination. The parent directory is an explicit same-user capability, not a hostile-writer security boundary.

## Alternatives rejected

- Rebuild Mantle: outside the operator's current permission.
- Rebuild the toolchain: Stage0 is permanently prohibited for these package builds.
- Rewrite CA to the final hash: changes content identity and can invalidate the original path.
- Drop rejected references or trust unsigned records: weakens closure or authenticity checks.
- Patch archive node expectations without proving the original node: hides unexplained differences.

## Consequences

The existing Mantle executable can import the migrated archive through its normal strict interface. The original archive and state remain available for audit.

This is a transport representation migration, not a package realization, provenance upgrade, compiler-correctness proof, or release authorization. NAR-cache import still needs its own explicit marker-CA protocol support. The helper does not make that route compatible.
