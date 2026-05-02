## Why

The i386 tcc27 handoff now has a Mes runtime and libtcc1 archive, but the predecessor tcc26-i386 still segfaults while preprocessing line markers and compiling `tccgen.c` / the full `ONE_SOURCE=1` TinyCC 0.9.27 object. This blocks the next Make 3.82 i386 validation slice.

## What Changes

- Narrow or repair the TinyCC 0.9.27 source-normalization seams in `bootstrap/spike-i386-mes-runtime-layout.ncl`.
- Preserve focused diagnostics for the predecessor compiler until the full tcc27 compile/link path passes or a new smaller blocker is identified.

## Impact

- Files: `bootstrap/spike-i386-mes-runtime-layout.ncl`, OpenSpec evidence.
- Testing: Crunch-build the sibling diagnostic derivation and save the logs/summary.
