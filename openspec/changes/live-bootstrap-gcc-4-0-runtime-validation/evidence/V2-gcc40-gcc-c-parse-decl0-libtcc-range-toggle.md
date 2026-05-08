# GCC 4.0 c-parse decl0 libtcc source-range toggle trace

Diagnostic: `bootstrap/diag-gcc40-c-parse-boundary.ncl`.

Scope: predecessor TinyCC compile-boundary only. This does **not** claim direct `decl0` runtime marker execution; the instrumented compiler is still blocked before those markers can run.

Command result: wrapper/build exited non-zero as expected for this diagnostic target; raw report and filtered build log are stored in this evidence directory.

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
| `libtcc_range_util_78_470` | `libtcc-range-util.c` | `common` | `139` |
| `libtcc_range_util_78_470` | `libtcc-range-util.c` | `one_source` | `139` |
| `libtcc_range_error_471_570` | `libtcc-range-error.c` | `common` | `139` |
| `libtcc_range_error_471_570` | `libtcc-range-error.c` | `one_source` | `139` |
| `libtcc_range_state_571_996` | `libtcc-range-state.c` | `common` | `139` |
| `libtcc_range_state_571_996` | `libtcc-range-state.c` | `one_source` | `139` |
| `libtcc_range_addfile_997_1182` | `libtcc-range-addfile.c` | `common` | `139` |
| `libtcc_range_addfile_997_1182` | `libtcc-range-addfile.c` | `one_source` | `139` |
| `libtcc_range_options_1183_1583` | `libtcc-range-options.c` | `common` | `139` |
| `libtcc_range_options_1183_1583` | `libtcc-range-options.c` | `one_source` | `139` |
| `libtcc_range_args_1584_1981` | `libtcc-range-args.c` | `common` | `139` |
| `libtcc_range_args_1584_1981` | `libtcc-range-args.c` | `one_source` | `139` |

## Findings

- Baseline `tccgen.c` remains `rc=139`, and `native387_disabled` keeps the prior rescue for `tccgen.c` (`rc=0`).
- Baseline `libtcc.c` remains `rc=139` under both common and `ONE_SOURCE=1` flags.
- Disabling coarse `libtcc.c` source ranges (`78..470`, `471..570`, `571..996`, `997..1182`, `1183..1583`, `1584..1981`) does not rescue `libtcc.c`; every range-toggle probe remains `rc=139`.
- Result: the `libtcc.c` predecessor crash is not isolated to a single removable top-level source band by coarse `#if 0` range exclusion. Next useful slice is a macro/preinclude or minimal-prefix probe that avoids syntactically deleting declarations while narrowing parser state.

## Artifacts

- Raw build report: `evidence/gcc40-cparse-decl0-libtcc-range-toggle-20260507/crunch-build-report.json`
- Focused build log: `evidence/gcc40-cparse-decl0-libtcc-range-toggle-20260507/focused-build-log.txt`
- Original Crunch saved log: `/home/brittonr/.local/state/crunch/logs/885wr5s1hhan4jqgfdcsjbpbil6ibfkl-diag-gcc40-c-parse-boundary.drv.log`
