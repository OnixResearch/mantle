# V2 GCC 4.0 c-parse decl0 libtcc error1 body/callsite toggle evidence

Diagnostic: `bootstrap/diag-gcc40-c-parse-boundary.ncl`.

Scope: predecessor TinyCC `libtcc.c` compile-boundary only. Prior evidence showed the boundary begins when full `strcat_printf`/post-474 context is present, and declaration/prototype/body variants still returned `rc=139`. This slice replaces the immediately-following `error1` body to test whether its `strcat_printf` callsites or formatting/error-path statements are the active trigger. No direct `decl0` runtime success is claimed unless runtime markers appear.

## Focused results

| stage | source | flags | rc |
|---|---|---:|---:|
| `libtcc_error1_empty` | `libtcc-error1-empty.c` | `common` | `139` |
| `libtcc_error1_buf_only` | `libtcc-error1-buf_only.c` | `common` | `139` |
| `libtcc_error1_vprintf_only` | `libtcc-error1-vprintf_only.c` | `common` | `139` |
| `libtcc_error1_fixed_prefix` | `libtcc-error1-fixed_prefix.c` | `common` | `139` |

## Findings

- `error1` variants captured: `libtcc_error1_empty` rc=139, `libtcc_error1_buf_only` rc=139, `libtcc_error1_vprintf_only` rc=139, `libtcc_error1_fixed_prefix` rc=139.
- Replacing `error1` with an empty body, buffer-only body, `strcat_vprintf`-only body, or fixed `pstrcat` prefix body still leaves full `libtcc.c` at `rc=139`. The active crash is therefore not explained solely by the statements or `strcat_printf` callsites inside `error1`.
- Instrumented compiler build remains unavailable (`build rc=139`), so this remains pre-marker evidence.
- No `diag-tcc-decl0-runtime:` markers emitted; direct `decl0` runtime validation remains blocked before marker execution.

## Artifacts

- Build report: `evidence/gcc40-cparse-decl0-libtcc-error1-toggle-20260508/crunch-build-report.json`
- Focused diagnostic log: `evidence/gcc40-cparse-decl0-libtcc-error1-toggle-20260508/focused-build-log.txt`
- Saved derivation log: `/home/brittonr/.local/state/crunch/logs/dch3vm6ailm2vp5y90pk0rhmcw5yi7bd-diag-gcc40-c-parse-boundary.drv.log`
