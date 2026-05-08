# GCC 4.0 c-parse decl0 libtcc macro toggle trace

Diagnostic: `bootstrap/diag-gcc40-c-parse-boundary.ncl`.

Scope: predecessor TinyCC `libtcc.c` compile-boundary macro toggles after the range/prefix/varargs-stub reductions. This is pre-marker evidence only; direct `decl0` runtime execution is not claimed unless runtime markers are present.

## Focused results

| stage | source | flags | rc |
|---|---|---:|---:|
| `baseline` | `tccgen.c` | `common` | `139` |
| `baseline` | `libtcc.c` | `common` | `139` |
| `baseline` | `tccgen.c` | `one_source` | `139` |
| `baseline` | `libtcc.c` | `one_source` | `139` |
| `native387_disabled` | `tccgen.c` | `common` | `0` |
| `native387_disabled` | `libtcc.c` | `common` | `139` |
| `native387_disabled` | `tccgen.c` | `one_source` | `0` |
| `native387_disabled` | `libtcc.c` | `one_source` | `139` |
| `libtcc_string_patch` | `libtcc.c` | `common` | `139` |
| `libtcc_string_patch` | `libtcc.c` | `one_source` | `139` |
| `libtcc_macro_minimal` | `libtcc.c` | `macro` | `139` |
| `libtcc_macro_no_setjmp` | `libtcc.c` | `macro` | `139` |
| `libtcc_macro_no_float` | `libtcc.c` | `macro` | `139` |
| `libtcc_macro_no_long_long` | `libtcc.c` | `macro` | `139` |
| `libtcc_macro_no_bitfield` | `libtcc.c` | `macro` | `139` |
| `libtcc_macro_no_static` | `libtcc.c` | `macro` | `139` |
| `libtcc_macro_no_use_libgcc` | `libtcc.c` | `macro` | `139` |

## Findings

- Macro toggle matrix: `libtcc_macro_minimal` rc=139, `libtcc_macro_no_setjmp` rc=139, `libtcc_macro_no_float` rc=139, `libtcc_macro_no_long_long` rc=139, `libtcc_macro_no_bitfield` rc=139, `libtcc_macro_no_static` rc=139, `libtcc_macro_no_use_libgcc` rc=139.
- Failing macro subsets: `libtcc_macro_minimal rc=139`, `libtcc_macro_no_setjmp rc=139`, `libtcc_macro_no_float rc=139`, `libtcc_macro_no_long_long rc=139`, `libtcc_macro_no_bitfield rc=139`, `libtcc_macro_no_static rc=139`, `libtcc_macro_no_use_libgcc rc=139`.
- No `diag-tcc-decl0-runtime:` markers emitted; direct `decl0` runtime validation remains blocked before marker execution.

## Artifacts

- Raw build report: `evidence/gcc40-cparse-decl0-libtcc-macro-toggle-20260507/crunch-build-report.json`
- Focused build log: `evidence/gcc40-cparse-decl0-libtcc-macro-toggle-20260507/focused-build-log.txt`
- Original Crunch saved log: `/home/brittonr/.local/state/crunch/logs/gmnqalkxlkqjm8m1p162lfyblfgfrzdv-diag-gcc40-c-parse-boundary.drv.log`
