# GCC 4.0 c-parse decl0 libtcc strcat_printf stub trace

Diagnostic: `bootstrap/diag-gcc40-c-parse-boundary.ncl`.

Scope: predecessor TinyCC compile-boundary only; no direct `decl0` runtime success is claimed unless `diag-tcc-decl0-runtime:` markers appear below.

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
| `libtcc_prefix_strcat_vprintf_466` | `libtcc-prefix-strcat_vprintf.c` | `common` | `0` |
| `libtcc_prefix_strcat_vprintf_466` | `libtcc-prefix-strcat_vprintf.c` | `one_source` | `0` |
| `libtcc_prefix_strcat_printf_474` | `libtcc-prefix-strcat_printf.c` | `common` | `139` |
| `libtcc_prefix_strcat_printf_474` | `libtcc-prefix-strcat_printf.c` | `one_source` | `139` |
| `libtcc_prefix_error1_518` | `libtcc-prefix-error1.c` | `common` | `139` |
| `libtcc_prefix_error1_518` | `libtcc-prefix-error1.c` | `one_source` | `139` |
| `libtcc_prefix_error_noabort_536` | `libtcc-prefix-error_noabort.c` | `common` | `139` |
| `libtcc_prefix_error_noabort_536` | `libtcc-prefix-error_noabort.c` | `one_source` | `139` |
| `libtcc_prefix_error_api_570` | `libtcc-prefix-error_api.c` | `common` | `139` |
| `libtcc_prefix_error_api_570` | `libtcc-prefix-error_api.c` | `one_source` | `139` |
| `libtcc_strcat_printf_stub` | `libtcc-strcat-printf-stub.c` | `common` | `139` |
| `libtcc_strcat_printf_stub` | `libtcc-strcat-printf-stub.c` | `one_source` | `139` |

## Findings

- `strcat_printf` stub toggle result: common: rc=139, one_source: rc=139.
- Prefix evidence remains: through `strcat_vprintf` line 466 compiles, adding `strcat_printf` line 474 segfaults (`rc=139`).
- No `diag-tcc-decl0-runtime:` markers emitted in this run; the diagnostic remains predecessor-compile scoped.
- Stubbing only `strcat_printf` does not fully clear the predecessor compile; next slice should continue through subsequent varargs/error wrappers.

## Artifacts

- Raw build report: `evidence/gcc40-cparse-decl0-libtcc-strcat-stub-20260507/crunch-build-report.json`
- Focused build log: `evidence/gcc40-cparse-decl0-libtcc-strcat-stub-20260507/focused-build-log.txt`
- Original Crunch saved log: `/home/brittonr/.local/state/crunch/logs/hpfl9lsxhcy1r91zqw25s8lnd2654z1q-diag-gcc40-c-parse-boundary.drv.log`
