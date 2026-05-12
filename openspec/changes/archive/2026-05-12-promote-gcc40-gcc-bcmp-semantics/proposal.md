## Why

`gcc.4.0` is evidence-backed partial, but several libgcc archive members still use generic pass1 fallback bodies. Promoting one small member from fallback shape to real behavior reduces the native-correctness gap without pretending GCC 4.0 is complete.

## What Changes

- Promote `__gcc_bcmp` in `bootstrap/gcc-4.0.ncl` from generic fallback body to byte-wise comparison semantics.
- Add in-derivation semantic smoke coverage for equal and unequal byte buffers.
- Refresh the GCC 4.0 placeholder inventory after source movement.

## Impact

- Files: `bootstrap/gcc-4.0.ncl`, `bootstrap/evidence/gcc-4.0-placeholder-inventory.json`, bootstrap OpenSpec baseline.
- Testing: eval/shell syntax, targeted parity tests, parity report, OpenSpec validation, whitespace checks.
