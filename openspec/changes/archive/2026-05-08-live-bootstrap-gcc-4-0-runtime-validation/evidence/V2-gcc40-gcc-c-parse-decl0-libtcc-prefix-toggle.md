# GCC 4.0 c-parse decl0 libtcc minimal-prefix trace

Diagnostic: `bootstrap/diag-gcc40-c-parse-boundary.ncl`.

Scope: predecessor TinyCC compile-boundary only; no direct `decl0` runtime success is claimed.

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
| `libtcc_prefix_globals_77` | `libtcc-prefix-globals.c` | `common` | `139` |
| `libtcc_prefix_globals_77` | `libtcc-prefix-globals.c` | `one_source` | `139` |
| `libtcc_prefix_asmstubs_89` | `libtcc-prefix-asmstubs.c` | `common` | `139` |
| `libtcc_prefix_asmstubs_89` | `libtcc-prefix-asmstubs.c` | `one_source` | `139` |
| `libtcc_prefix_pathutils_198` | `libtcc-prefix-pathutils.c` | `common` | `139` |
| `libtcc_prefix_pathutils_198` | `libtcc-prefix-pathutils.c` | `one_source` | `139` |
| `libtcc_prefix_alloc_237` | `libtcc-prefix-alloc.c` | `common` | `139` |
| `libtcc_prefix_alloc_237` | `libtcc-prefix-alloc.c` | `one_source` | `139` |
| `libtcc_prefix_dynarray_432` | `libtcc-prefix-dynarray.c` | `common` | `0` |
| `libtcc_prefix_dynarray_432` | `libtcc-prefix-dynarray.c` | `one_source` | `0` |
| `libtcc_prefix_split_path_457` | `libtcc-prefix-split_path.c` | `common` | `0` |
| `libtcc_prefix_split_path_457` | `libtcc-prefix-split_path.c` | `one_source` | `0` |
| `libtcc_prefix_strcat_vprintf_466` | `libtcc-prefix-strcat_vprintf.c` | `common` | `0` |
| `libtcc_prefix_strcat_vprintf_466` | `libtcc-prefix-strcat_vprintf.c` | `one_source` | `0` |
| `libtcc_prefix_strcat_printf_474` | `libtcc-prefix-strcat_printf.c` | `common` | `139` |
| `libtcc_prefix_strcat_printf_474` | `libtcc-prefix-strcat_printf.c` | `one_source` | `139` |
| `libtcc_prefix_error1_518` | `libtcc-prefix-error1.c` | `common` | `139` |
| `libtcc_prefix_error1_518` | `libtcc-prefix-error1.c` | `one_source` | `139` |
| `libtcc_prefix_set_error_func_525` | `libtcc-prefix-set_error_func.c` | `common` | `139` |
| `libtcc_prefix_set_error_func_525` | `libtcc-prefix-set_error_func.c` | `one_source` | `139` |
| `libtcc_prefix_error_noabort_536` | `libtcc-prefix-error_noabort.c` | `common` | `139` |
| `libtcc_prefix_error_noabort_536` | `libtcc-prefix-error_noabort.c` | `one_source` | `139` |
| `libtcc_prefix_tcc_error_553` | `libtcc-prefix-tcc_error.c` | `common` | `139` |
| `libtcc_prefix_tcc_error_553` | `libtcc-prefix-tcc_error.c` | `one_source` | `139` |
| `libtcc_prefix_warning_566` | `libtcc-prefix-warning.c` | `common` | `139` |
| `libtcc_prefix_warning_566` | `libtcc-prefix-warning.c` | `one_source` | `139` |
| `libtcc_prefix_error_api_570` | `libtcc-prefix-error_api.c` | `common` | `139` |
| `libtcc_prefix_error_api_570` | `libtcc-prefix-error_api.c` | `one_source` | `139` |
| `libtcc_prefix_compile_string_675` | `libtcc-prefix-compile_string.c` | `common` | `139` |
| `libtcc_prefix_compile_string_675` | `libtcc-prefix-compile_string.c` | `one_source` | `139` |
| `libtcc_prefix_state_new_894` | `libtcc-prefix-state_new.c` | `common` | `139` |
| `libtcc_prefix_state_new_894` | `libtcc-prefix-state_new.c` | `one_source` | `139` |
| `libtcc_prefix_output_type_984` | `libtcc-prefix-output_type.c` | `common` | `139` |
| `libtcc_prefix_output_type_984` | `libtcc-prefix-output_type.c` | `one_source` | `139` |
| `libtcc_prefix_add_files_1182` | `libtcc-prefix-add_files.c` | `common` | `139` |
| `libtcc_prefix_add_files_1182` | `libtcc-prefix-add_files.c` | `one_source` | `139` |
| `libtcc_prefix_option_linker_1583` | `libtcc-prefix-option_linker.c` | `common` | `139` |
| `libtcc_prefix_option_linker_1583` | `libtcc-prefix-option_linker.c` | `one_source` | `139` |
| `libtcc_prefix_full_1981` | `libtcc-prefix-full.c` | `common` | `139` |
| `libtcc_prefix_full_1981` | `libtcc-prefix-full.c` | `one_source` | `139` |

## Findings

- Baseline `libtcc.c` remains `rc=139` under both common and `ONE_SOURCE=1` flags.
- Some intentionally-short prefixes that end inside preprocessor blocks or declarations also return `rc=139`; treat those as syntax-shape controls, not source-region proof.
- The last syntactically complete passing prefix is through `dynarray_reset` at line 432 (`rc=0` under both flag sets).
- Adding `tcc_split_path` through line 457 and `strcat_vprintf` through line 466 still passes (`rc=0`), but adding the varargs wrapper `strcat_printf` through line 474 flips to `rc=139` under both common and `ONE_SOURCE=1` flags.
- Every longer error-path prefix remains `rc=139`, so the remaining `libtcc.c` predecessor compile crash narrows to the first varargs wrapper body, `strcat_printf(char *buf, int buf_size, const char *fmt, ...)`, specifically lines 468..474.
- Next actionable slice: rewrite or stub `strcat_printf`/varargs formatting in the instrumented TinyCC predecessor path and test whether full `libtcc.c` compiles far enough to unblock direct `decl0` runtime markers.

## Artifacts

- Raw build report: `evidence/gcc40-cparse-decl0-libtcc-prefix-toggle-20260507/crunch-build-report.json`
- Focused build log: `evidence/gcc40-cparse-decl0-libtcc-prefix-toggle-20260507/focused-build-log.txt`
- Original Crunch saved log: `/home/brittonr/.local/state/crunch/logs/ccrvpnciraqa2xmxmblr9x8q4nnxcqxi-diag-gcc40-c-parse-boundary.drv.log`
