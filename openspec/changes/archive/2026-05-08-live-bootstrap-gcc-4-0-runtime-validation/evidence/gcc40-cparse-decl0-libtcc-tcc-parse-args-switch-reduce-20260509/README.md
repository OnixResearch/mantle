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

Next useful reduction: split `TCC_OPTION_dumpversion` into `case`/empty, `printf`, `exit`, and `printf+exit` variants while preserving the full known-good through-`MF` body and the `unsupported_option`/`set_output_type` labels. Avoid generating incomplete switch slices with dangling `#ifdef` or missing labels, because those can produce unrelated Mes/TCC crashes.
