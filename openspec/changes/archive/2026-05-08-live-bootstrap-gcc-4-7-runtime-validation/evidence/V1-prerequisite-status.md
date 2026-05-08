# V1 prerequisite runtime-validation status

Task-ID: V1
Covers: bootstrap.gcc47.runtime-validation
Captured: 2026-05-08T21:09:49Z

## Result

Prerequisite runtime-validation evidence is available, but the chain is not promotable:

- `binutils-tcc` is archived at `openspec/changes/archive/2026-05-05-live-bootstrap-binutils-tcc-chain-runtime-validation`. Its final evidence validates explicit bootstrap bridge fallbacks for binutils tools; native gas/binutils/ld remain recorded as non-fatal bridge-boundary failures.
- `gcc-4.0` is archived at `openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation`. It is a negative runtime-validation archive: no `gcc-4.0.4` compiler output exists because the deterministic predecessor TinyCC/Mes path remains blocked before direct `decl0` runtime markers, currently at the `libtcc.c`/`tcc_compile`/varargs source-shape boundary.

`bootstrap/gcc-4.7.ncl` depends on a real `gcc-4.0.4` provider (`let gcc4 = import "gcc-4.0.ncl"`; builder `find_input gcc-4.0.4`; `CC="$GCC4/bin/gcc"`; `CXX="$GCC4/bin/g++"`). Therefore gcc-4.7 validation cannot be promoted until the gcc-4.0 negative boundary is repaired and emits a compiler output.

No gcc-4.7 runtime success is claimed in this evidence.
