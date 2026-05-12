## Why

`gcc.4.0` remains an evidence-backed partial stage because the installed driver is still a pass1 bridge. The next useful native-correctness slice is not another generic marker inventory, but a bounded GCC driver contract that downstream bootstrap stages can query deterministically.

## What Changes

- Promote the GCC 4.0 installed driver to answer core query flags with GCC-shaped values: `-dumpversion`, `-dumpmachine`, `-print-libgcc-file-name`, and `-print-search-dirs`.
- Add derivation-local smoke checks proving those query flags return the expected version, target, and installed libgcc path.
- Keep `gcc.4.0` classified as partial until real `cc1`/native compiler correctness is proven.

## Capabilities

### Modified Capabilities
- `bootstrap.gcc40.native-correctness`: Adds a bounded driver-query semantic slice while preserving fail-closed parity status.

## Impact

- **Files**: `bootstrap/gcc-4.0.ncl`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`, and the archived change.
- **Testing**: eval/shell syntax, Rust parity tests, Crunch build, driver query smoke, parity report, OpenSpec validation.
