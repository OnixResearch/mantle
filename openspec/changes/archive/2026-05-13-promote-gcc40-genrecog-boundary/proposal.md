## Why

GCC 4.0 still has a grouped generated-source wrapper for several late generator executables. `genrecog` currently shares the generic `generated_bootstrap_stub` output, which hides the exact recognition-generator boundary from parity evidence.

## What Changes

- Split `genrecog` out of the grouped generated-source wrapper.
- Emit a named empty-recognition source boundary for `genrecog`.
- Add derivation-local checks that the wrapper exists, emits the expected marker/symbol, and rejects the prior generic label.
- Keep `gcc.4.0` evidence-backed partial; this does not prove native `genrecog` correctness.

## Capabilities

### Modified Capabilities
- `bootstrap.gcc.version-ladder`: records the checked `genrecog` boundary contract.

## Impact

- Files: `bootstrap/gcc-4.0.ncl`, `bootstrap/evidence/gcc-4.0-placeholder-inventory.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- APIs: none.
- Testing: eval/shell syntax, Crunch build, parity tests/report, OpenSpec validation, diff check.
