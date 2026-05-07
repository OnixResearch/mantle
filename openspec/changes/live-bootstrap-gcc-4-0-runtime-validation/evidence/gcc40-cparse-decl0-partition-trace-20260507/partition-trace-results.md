# GCC 4.0 c-parse decl0 partition trace

Task-ID: V2
Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
Status: captured

## Result
- Added compile-only partition probes before the full instrumented TinyCC rebuild.
- One-source preprocessing succeeds: `rc=0`, `19823` lines.
- Component compile results:
  - `tccpp.c` rc=`0`
  - `tccgen.c` rc=`139`
  - `tccelf.c` rc=`0`
  - `x86_64-gen.c` rc=`0`
  - `libtcc.c` rc=`139`
- Full instrumented compiler build remains `rc=139` before runtime markers execute.
- This narrows the instrumentation blocker to TinyCC compiling `tccgen.c`/`libtcc.c`, rather than preprocessing, `tccpp.c`, `tccelf.c`, or `x86_64-gen.c`.

## Excerpt
```text
diag-tcc-decl0-part: preprocess one-source
diag-tcc-decl0-part: preprocess rc=0 lines=19823
diag-tcc-decl0-part: compile tccpp.c
diag-tcc-decl0-part: compile tccpp.c rc=0
diag-tcc-decl0-part: compile tccgen.c
diag-tcc-decl0-part: compile tccgen.c rc=139
diag-tcc-decl0-part: tccgen.c stderr: Segmentation fault (core dumped)
diag-tcc-decl0-part: compile tccelf.c
diag-tcc-decl0-part: compile tccelf.c rc=0
diag-tcc-decl0-part: compile x86_64-gen.c
diag-tcc-decl0-part: compile x86_64-gen.c rc=0
diag-tcc-decl0-part: compile libtcc.c
diag-tcc-decl0-part: compile libtcc.c rc=139
diag-tcc-decl0-part: libtcc.c stderr: Segmentation fault (core dumped)
diag-tcc-decl0: build instrumented compiler
diag-tcc-decl0: build rc=139
diag-tcc-decl0: build-stderr: Segmentation fault (core dumped)
diag-tcc-decl0: instrumented compiler unavailable
```
