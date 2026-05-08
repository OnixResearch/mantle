# GCC 4.0 c-parse decl0 libtcc broad varargs/error stub trace

Diagnostic: `bootstrap/diag-gcc40-c-parse-boundary.ncl`.

Scope: predecessor TinyCC compile-boundary only; no direct `decl0` runtime success is claimed unless runtime markers appear.

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
| `libtcc_strcat_printf_stub` | `libtcc-strcat-printf-stub.c` | `common` | `139` |
| `libtcc_strcat_printf_stub` | `libtcc-strcat-printf-stub.c` | `one_source` | `139` |
| `libtcc_error_varargs_stub` | `libtcc-error-varargs-stub.c` | `common` | `139` |
| `libtcc_error_varargs_stub` | `libtcc-error-varargs-stub.c` | `one_source` | `139` |

## Findings

- `strcat_printf`-only stub remains: common rc=139, one_source rc=139.
- Broad error/varargs stub (`strcat_printf`, `tcc_error_noabort`, `tcc_error`, `tcc_warning`) result: common rc=139, one_source rc=139.
- No `diag-tcc-decl0-runtime:` markers emitted; direct `decl0` runtime validation remains blocked before marker execution.
- Broad varargs/error stubbing still leaves `libtcc.c` at `rc=139`; the blocker is not only these varargs/error wrappers.

## Artifacts

- Raw build report: `evidence/gcc40-cparse-decl0-libtcc-error-varargs-stub-20260507/crunch-build-report.json`
- Focused build log: `evidence/gcc40-cparse-decl0-libtcc-error-varargs-stub-20260507/focused-build-log.txt`
- Original Crunch saved log: `/home/brittonr/.local/state/crunch/logs/kb0m0gawf1axn038y62ymx03qs4m9v8f-diag-gcc40-c-parse-boundary.drv.log`
