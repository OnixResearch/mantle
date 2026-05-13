## Why

GCC 4.0 still records libiberty demangling as a disabled boundary. A small bounded Itanium C++ symbol slice can make the seam semantic without claiming full native cp-demangle correctness.

## What Changes

- Replace the disabled-demangle marker with a named bounded-demangle semantic boundary.
- Implement a tiny deterministic `_Z<length><name>v` demangle slice for zero-argument Itanium functions.
- Add derivation-local smoke checks and parity regressions while keeping `gcc.4.0` partial.

## Impact

- Files: `bootstrap/gcc-4.0.ncl`, `bootstrap/evidence/gcc-4.0-native-boundary.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- Testing: eval/shell syntax, parity tests, GCC 4.0 build, parity report, OpenSpec validation, diff check.
