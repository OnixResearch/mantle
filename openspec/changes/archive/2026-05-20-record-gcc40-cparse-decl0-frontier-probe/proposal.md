## Why

We need to stop adding low-ROI installed-`cc1` semantic micro-slices and instead record a bounded source-frontier probe at the current TinyCC/Mes `c-parse` boundary. The latest diagnostic build shows a more precise seam: baseline TCC source compilation still segfaults, disabling native 387 compilation gets `tccgen.c`/`libtcc.c` through compile-only, and the instrumented `decl0` path reaches a valid global declaration probe before the remaining write-file fd branch crash.

## What Changes

- Refresh the GCC 4.0 native `cc1` build/source-frontier receipt to a v2 source-frontier reduction.
- Require the receipt to record the exact diagnostic markers and observed seam: native387-disabled source compile success, fallback musl-shim instrumented compiler selection, `cparse_decl0_trace_valid_var_semicolon rc=0`, and the remaining `fd_bad` branch frontier.
- Keep `gcc.4.0` evidence-backed `partial`; this is not native compiler correctness.

## Capabilities

### Modified Capabilities
- `bootstrap.gcc.version-ladder.gcc40-native-cc1-source-frontier-reduction`: source-frontier evidence becomes more precise and fail-closed.

## Impact

- **Files**: `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- **Testing**: focused bootstrap parity tests, CLI parity tests, OpenSpec strict validation, blocker/source-pin checks, parity-report confirmation that `gcc.4.0` remains partial.
