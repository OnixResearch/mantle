# V2 GCC 4.0 c-parse decl0 native387 toggle trace

Task-ID: V2
Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
Status: captured
Evidence: `evidence/gcc40-cparse-decl0-native387-toggle-20260507/`

This slice tests whether the predecessor TinyCC `tccgen.c` compile segfault is caused by the native x87 / `TCC_IS_NATIVE_387` block before direct `decl0` runtime marker insertion.

Result: baseline `tccgen.c` remains `rc=139` under both common and `ONE_SOURCE=1` flags. After disabling the first `#if defined TCC_IS_NATIVE_387` block, `tccgen.c` becomes `rc=0` under both flag sets, and post-marker `tccgen.c` also compiles. `libtcc.c` remains `rc=139`, and the full instrumented compiler build still returns `rc=139` before any `diag-tcc-decl0-runtime:*` markers execute.

Conclusion: the `tccgen.c` compile blocker is now isolated to native x87 handling; the remaining blocker for running direct `decl0` markers is `libtcc.c` under the predecessor TinyCC compiler.
