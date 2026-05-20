# Design: GCC 4.0 c-parse generated-header frontier v5

## Approach

Advance only the checked source-frontier metadata. The diagnostic derivation already contains bounded probes that apply the v4 autohost six-undef workaround and then test generated header surfaces. The receipt should capture the stable frontier matrix:

- six-undef autohost prefixes/full `auto-host.h` pass through `system.h`/`coretypes.h`/`tm.h`;
- `insn-modes.h` include-only and 40-line prefix probes pass while shorter 1/5/10/20-line prefixes fail;
- `machmode.h` include-only and 40+ line prefixes pass while the 20-line prefix fails;
- `tree.h` early prefixes through 28 lines pass, line 36 fails, later balanced prefixes and bounded builtins enum probes pass.

## Fail-closed contract

Validation must reject stale v4/autohost-only evidence, missing required generated-header frontier fragments, and diagnostic marker drift against `bootstrap/diag-gcc40-c-parse-boundary.ncl`.

## Claim boundary

The receipt remains `frontier-only`; `gcc.4.0` remains evidence-backed `partial` and live-bootstrap/Guix/StageX blocked until real native source-build/compiler correctness exists.
