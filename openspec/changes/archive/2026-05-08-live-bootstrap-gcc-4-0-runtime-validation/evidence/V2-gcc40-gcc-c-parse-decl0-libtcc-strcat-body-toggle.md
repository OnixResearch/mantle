# GCC 4.0 c-parse decl0 libtcc strcat_printf body toggle trace

Diagnostic: `bootstrap/diag-gcc40-c-parse-boundary.ncl`.

Scope: predecessor TinyCC `libtcc.c` compile-boundary body toggles for `strcat_printf`, after prior prefix evidence showed the boundary flips between `strcat_vprintf` and `strcat_printf`. This is pre-marker evidence only.

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
| `libtcc_strcat_body_empty` | `libtcc-strcat-body-empty.c` | `common` | `139` |
| `libtcc_strcat_body_declare_ap` | `libtcc-strcat-body-declare_ap.c` | `common` | `139` |
| `libtcc_strcat_body_va_start_only` | `libtcc-strcat-body-va_start_only.c` | `common` | `139` |
| `libtcc_strcat_body_va_start_end` | `libtcc-strcat-body-va_start_end.c` | `common` | `139` |
| `libtcc_strcat_body_direct_pstrcat` | `libtcc-strcat-body-direct_pstrcat.c` | `common` | `139` |

## Findings

- `strcat_printf` body variants: `libtcc_strcat_body_empty` rc=139, `libtcc_strcat_body_declare_ap` rc=139, `libtcc_strcat_body_va_start_only` rc=139, `libtcc_strcat_body_va_start_end` rc=139, `libtcc_strcat_body_direct_pstrcat` rc=139.
- Failing body variants: `libtcc_strcat_body_empty rc=139`, `libtcc_strcat_body_declare_ap rc=139`, `libtcc_strcat_body_va_start_only rc=139`, `libtcc_strcat_body_va_start_end rc=139`, `libtcc_strcat_body_direct_pstrcat rc=139`.
- No `diag-tcc-decl0-runtime:` markers emitted; direct `decl0` runtime validation remains blocked before marker execution.

## Artifacts

- Raw build report: `evidence/gcc40-cparse-decl0-libtcc-strcat-body-toggle-20260508/crunch-build-report.json`
- Focused build log: `evidence/gcc40-cparse-decl0-libtcc-strcat-body-toggle-20260508/focused-build-log.txt`
- Original Crunch saved log: `/home/brittonr/.local/state/crunch/logs/wsz8zqa1kvkyqkyvww17fpr7nb9a2mah-diag-gcc40-c-parse-boundary.drv.log`
