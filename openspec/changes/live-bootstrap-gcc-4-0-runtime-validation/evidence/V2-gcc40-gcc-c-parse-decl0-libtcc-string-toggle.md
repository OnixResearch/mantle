# V2 GCC 4.0 c-parse decl0 libtcc string-toggle trace

Task-ID: V2
Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
Status: captured
Evidence: `evidence/gcc40-cparse-decl0-libtcc-string-toggle-20260507/`

This slice tests whether the Mes-runtime-sensitive `libtcc.c` direct-string/error-path patches used by the i386 TinyCC spike move the remaining predecessor `libtcc.c` compile segfault after the native x87 `tccgen.c` toggle.

Result: `tccgen.c` remains fixed by the native x87 toggle (`rc=0`), but `libtcc.c` stays `rc=139` under baseline, native387-disabled, and `libtcc_string_patch` stages for both common and `ONE_SOURCE=1` flags. The full instrumented compiler build remains `rc=139` before `diag-tcc-decl0-runtime:*` markers execute.

Conclusion: direct-string/error-path patches are not sufficient for the amd64 predecessor `libtcc.c` compile crash; the next seam is a finer `libtcc.c` source-range or macro-toggle reduction.
