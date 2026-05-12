## Why

`binutils.tcc` now has a checked tool-smoke transcript, but the parity row still reports `placeholder` because `bootstrap/binutils-tcc.ncl` contains an explicit placeholder marker for omitted libiberty sources. That marker correctly blocks accidental full parity, but it also obscures the current state: evidence-backed partial binutils tools whose remaining work is native/full-source correctness, not missing tool smoke evidence.

## What Changes

- Replace the explicit placeholder marker in `bootstrap/binutils-tcc.ncl` with an honest non-marker omitted-source shim name.
- Keep `binutils.tcc` `expected_complete=false`, so removing the marker can only promote the row to `partial`, never `complete`.
- Add a regression that documents the evidence-backed partial status and preserves fail-closed behavior for missing transcripts.

## Capabilities

### Modified Capabilities
- `bootstrap.parity.binutils-tcc-evidence`: distinguishes checked partial evidence from explicit placeholder markers.

## Impact

- **Files**: `bootstrap/binutils-tcc.ncl`, `src/bootstrap_parity.rs`, OpenSpec bootstrap delta.
- **APIs**: No public CLI shape change.
- **Dependencies**: None.
- **Testing**: targeted parity unit tests, `crunch bootstrap parity-report --json`, and `git diff --check`.
