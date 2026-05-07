# V2 GCC 4.0 c-parse decl0 runtime trace attempt

Task-ID: V2
Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
Status: captured-blocked
Evidence: `evidence/gcc40-cparse-decl0-runtime-trace-20260507/`

This slice adds a guarded direct TinyCC `tccgen.c` `decl0` runtime-instrumentation path. The diagnostic copies fixed TinyCC 0.9.27 source, injects `decl0` branch markers, and attempts to compile an instrumented `/tmp/tcc-decl0-instrumented` compiler before running the known declaration-error probes through it.

Result: the instrumented TinyCC compiler build segfaults (`diag-tcc-decl0: build rc=139`) before any runtime `diag-tcc-decl0-runtime:*` markers can execute. The guarded path records the build failure and falls back to the existing authoritative matrix, which continues to show valid semicolon declarations pass while EOF/malformed declaration paths return `rc=139`.

Next seam: lower-impact instrumentation that avoids rebuilding the whole TinyCC one-source compiler, or a source-level repair for the TinyCC self-compile segfault before runtime `decl0` branch markers can be used.
