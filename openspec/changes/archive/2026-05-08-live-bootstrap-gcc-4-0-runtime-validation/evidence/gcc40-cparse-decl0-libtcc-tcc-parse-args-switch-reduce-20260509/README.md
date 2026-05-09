# GCC 4.0 c-parse / libtcc `tcc_parse_args` switch reduction

Date: 2026-05-09

This evidence slice continues after the known-good post-`options_m` boundary:

- remove `copy_linker_arg(&s->fini_symbol)` style calls
- flatten the `options_m` `#ifdef TCC_TARGET_ARM` table island
- simplify the `CONFIG_TCC_ASM` island

## Scripts

- `run-local-tcc-parse-args-switch-ifdef.sh`
  - verifies that the early `tcc_parse_args` switch cases through `TCC_OPTION_c` compile after keeping balanced inactive `CONFIG_TCC_BACKTRACE` / `CONFIG_TCC_BCHECK` islands.
- `run-local-tcc-parse-args-d-case-reduce.sh`
  - verifies the `TCC_OPTION_d` body, including the `if`/`else if` chain and `goto unsupported_option`, compiles when the target label is present.
- `run-local-tcc-parse-args-post-d-cumulative.sh`
  - verifies cumulative switch cases from `static` through `m`, including `shared`/`r`/`run` gotos back to `set_output_type`, compile.
- `run-local-tcc-parse-args-tail-cumulative.sh`
  - verifies cumulative cases through `MF` compile, then the first real tail failure appears when adding `TCC_OPTION_dumpversion`.

## Receipts

- `focused-local-tcc-parse-args-switch-ifdef.txt`
  - all generated variants compile (`rc=0`) for common and `-D ONE_SOURCE=1`.
- `focused-local-tcc-parse-args-d-case-reduce.txt`
  - all generated `d`-case reductions compile (`rc=0`) for common and `-D ONE_SOURCE=1`.
- `focused-local-tcc-parse-args-post-d-cumulative.txt`
  - all generated variants through `m` compile (`rc=0`) for common and `-D ONE_SOURCE=1`.
- `focused-local-tcc-parse-args-tail-cumulative.txt`
  - variants through `MF` compile (`rc=0`) for common and `-D ONE_SOURCE=1`.
  - first tail failure is `10_through_dumpversion` (`rc=-11`) for both common and `-D ONE_SOURCE=1`.

## Current narrowed frontier

`TCC_OPTION_dumpversion` is the next frontier after the post-`options_m` normalization path. A valid cumulative switch body through `MF` compiles; adding:

```c
case TCC_OPTION_dumpversion:
    printf ("%s\n", TCC_VERSION);
    exit(0);
    break;
```

causes the local Mes/TCC compiler to segfault (`rc=-11`).

## Dumpversion split follow-up

- `run-local-tcc-parse-args-dumpversion-split.sh`
  - preserves the known-good manual cumulative body through `TCC_OPTION_MF` and varies only the next `TCC_OPTION_dumpversion` case.
- `focused-local-tcc-parse-args-dumpversion-split.txt`
  - `00_through_MF`: `rc=0` for common and `-D ONE_SOURCE=1`.
  - `01_dump_case_only_break`: `rc=0` for common and `-D ONE_SOURCE=1`.
  - `02_dump_printf_empty_break`: `rc=0` for common and `-D ONE_SOURCE=1`.
  - `04_dump_exit_only`: `rc=0` for common and `-D ONE_SOURCE=1`.
  - `03_dump_printf_version_break`: `rc=-11` for common and `-D ONE_SOURCE=1`.
  - `05_dump_printf_version_exit`: `rc=-11` for common and `-D ONE_SOURCE=1`.

The narrowed trigger is now specifically the `printf("%s\n", TCC_VERSION)` varargs call shape inside `TCC_OPTION_dumpversion`, not the case label or `exit(0)`.

Next useful reduction: split that printf shape (`printf("literal")`, `printf("%s", "literal")`, `printf("%s", TCC_VERSION)`, `fputs(TCC_VERSION, stdout)`) while preserving the through-`MF` body and required labels. Avoid generating incomplete switch slices with dangling `#ifdef` or missing labels, because those can produce unrelated Mes/TCC crashes.
