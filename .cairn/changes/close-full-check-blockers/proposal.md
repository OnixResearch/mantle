## Why

The promoted full-bootstrap parity proof is complete, but the repository-wide
Nix check still stops on lexical blocker matches inside the accepted V98 source.
Earlier distributed builds also reported fixed-output mismatches for the
SpaceWasm reference bundle and the pinned Octet toolchain.

The remaining work must close or precisely classify these failures. It must not
make the blocker inventory report-only, hide unknown marker text, change pinned
source semantics, or accept a new fixed-output hash without a fresh local
rebuild.

## What Changes

- Classify explicit negative facts and bounded timeout controls as structural
  non-blockers.
- Classify selected V98 marker files only after exact BLAKE3 verification of the
  independent promotion receipts and every named file.
- Keep unknown paths, unknown classes, changed files, positive bridge use, and
  observed timeout diagnostics actionable.
- Rebuild the SpaceWasm and wasm-component toolchain outputs locally. Repair
  only stale immutable hashes that current source bytes independently confirm.
- Rerun the complete local and distributed Nix checks without weakening any
  check.

## Impact

- **Files:** blocker inventory checker and documentation, ADR 0103, narrow Nix
  fixed-output pins if current rebuild evidence requires them, and lifecycle
  evidence.
- **Testing:** positive clean inventory, proof-byte tamper, semantic near-miss
  negatives, direct Nix builds, local-builder flake checks, ordinary flake
  checks, Cairn validation, and Tracey coverage.
