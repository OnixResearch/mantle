## Why

GCC 4.0 still contains broad pass1 generated-source boundaries. `genemit` is one remaining generator whose bridge currently emits a generic bootstrap stub source, which is too imprecise for parity evidence.

## What Changes

- Tighten the `genemit` generated-source seam to a named empty-emit source boundary.
- Add derivation-local checks that the wrapper exists, emits the expected boundary marker/symbol, and rejects the prior generic label.
- Keep `gcc.4.0` classified as evidence-backed partial; this is not native generator correctness.

## Capabilities

### Modified Capabilities
- `bootstrap.gcc.version-ladder`: records the checked `genemit` boundary contract.

## Impact

- Files: `bootstrap/gcc-4.0.ncl`, `bootstrap/evidence/gcc-4.0-placeholder-inventory.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- APIs: none.
- Testing: eval/shell syntax, Crunch build, parity tests/report, OpenSpec validation, diff check.
