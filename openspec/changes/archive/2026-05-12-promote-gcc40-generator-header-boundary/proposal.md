## Why

`gcc.4.0` now has a checked native-boundary receipt. The next real boundary to tighten is the early generator-header seam: `insn-constants.h` and `insn-flags.h` are currently seeded with explicitly stub-labelled headers before the native generator path can be trusted. That label keeps the GCC row partial and makes the boundary less precise than it needs to be.

## What Changes

- Promote the GCC 4.0 `genconstants`/`genflags` header seam to a named empty-machine-header boundary instead of generic stub text.
- Add derivation-local checks that the generated/seeded headers have their guards and no legacy stub label.
- Refresh GCC 4.0 placeholder inventory and parity regression coverage.

## Capabilities

### Modified Capabilities
- `bootstrap.gcc40.transition`: tightens one generator/header boundary without claiming native GCC completion.

## Impact

- **Files**: `bootstrap/gcc-4.0.ncl`, GCC evidence receipt, parity tests, bootstrap spec.
- **APIs**: No public CLI change.
- **Testing**: GCC eval shell syntax, GCC build, Rust parity tests, parity-report JSON, OpenSpec validation.
