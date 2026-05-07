# V2 GCC 4.0 c-parse decl0 partition trace

Task-ID: V2
Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
Status: captured
Evidence: `evidence/gcc40-cparse-decl0-partition-trace-20260507/`

This slice adds lower-impact compile-only partition probes ahead of the full direct TinyCC `tccgen.c::decl0` runtime-instrumented rebuild. The diagnostic now preprocesses the patched one-source TinyCC file and compiles individual source components before attempting the full instrumented compiler build.

Result: one-source preprocessing succeeds (`rc=0`, 19823 lines). Component compilation narrows the blocked instrumentation build: `tccpp.c`, `tccelf.c`, and `x86_64-gen.c` compile with `rc=0`, while `tccgen.c` and `libtcc.c` both segfault with `rc=139`. The full instrumented compiler build still returns `rc=139`, so runtime `diag-tcc-decl0-runtime:*` markers still do not execute.

The existing c-parse matrix remains authoritative: valid semicolon declarations pass, while malformed/EOF declaration paths return `rc=139`.
