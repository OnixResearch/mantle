# V2 GCC 4.0 c-parse decl0 pre-marker trace

Task-ID: V2
Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
Status: captured
Evidence: `evidence/gcc40-cparse-decl0-premarker-trace-20260507/`

This slice adds pre-marker TinyCC component compile probes before inserting direct `tccgen.c::decl0` runtime markers. It separates failures caused by the predecessor TinyCC compiling patched `tccgen.c`/`libtcc.c` from failures caused by the diagnostic marker insertion itself.

Result: `tccgen.c` and `libtcc.c` both segfault (`rc=139`) before marker insertion, under both common component flags and `ONE_SOURCE=1` flags. Patched one-source preprocessing still succeeds, and the full instrumented compiler build still returns `rc=139` before runtime `diag-tcc-decl0-runtime:*` markers execute.

Conclusion: the direct `decl0` marker code is not the current blocker; the next repair seam is predecessor TinyCC 0.9.27 musl-v2 compiling `tccgen.c`/`libtcc.c` generally. The existing c-parse matrix remains authoritative: valid semicolon declarations pass, while malformed/EOF declaration paths return `rc=139`.
