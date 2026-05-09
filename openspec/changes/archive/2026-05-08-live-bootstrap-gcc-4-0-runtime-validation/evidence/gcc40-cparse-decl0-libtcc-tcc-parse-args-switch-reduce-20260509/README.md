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

## Printf / macro shape follow-up

- `run-local-tcc-parse-args-dumpversion-printf-shape.sh`
  - literal calls compile: `printf("")`, `printf("0.9.27\n")`, and `printf("%s\n", "0.9.27")` all return `rc=0` under common and `-D ONE_SOURCE=1`.
  - every direct `TCC_VERSION` expression form tested segfaults (`rc=-11`): `printf(TCC_VERSION)`, `printf("%s\n", TCC_VERSION)`, `fputs(TCC_VERSION, stdout)`, `fputs(TCC_VERSION, stderr)`, `puts(TCC_VERSION)`, and `tcc_warning("%s", TCC_VERSION)`.
- `run-local-tcc-parse-args-dumpversion-macro-shape.sh`
  - local literal macro use compiles: `#define CRUNCH_LOCAL_VERSION "0.9.27"` + `puts(CRUNCH_LOCAL_VERSION)` returns `rc=0`.
  - aliasing through the original macro still fails: `#define CRUNCH_LOCAL_VERSION TCC_VERSION` + `puts(CRUNCH_LOCAL_VERSION)` returns `rc=-11`.
  - top-level literal storage compiles: `static const char *... = "0.9.27"` and `static char ...[] = "0.9.27"` return `rc=0`.
  - top-level storage initialized from the original `TCC_VERSION` macro fails (`rc=-11`), as does `sizeof(TCC_VERSION)`.
  - explicitly `#undef`/redefining `TCC_VERSION` to a literal immediately before `tcc_parse_args` makes `puts(TCC_VERSION)` compile (`rc=0`), including both `"0.9.27"` and `"x"`.

The narrowed trigger is now the original `TCC_VERSION` macro expansion path in the predecessor compiler context, not varargs, `printf`, string literal length, or use of stdout/stderr.

## TCC_VERSION source audit

- `run-local-tcc-version-source-audit.sh`
  - checks the focused replay scripts instead of rebuilding the unavailable `/crunch/store` inputs.
- `focused-local-tcc-version-source-audit.txt`
  - both printf/macro-shape focused replay scripts clear generated `config.h` with `: > config.h`.
  - both scripts' `flags_common` omit `-D TCC_VERSION=...`, and the extra-flag matrix only adds `-D ONE_SOURCE=1`.
  - Crunch's full diagnostic derivation still defines `TCC_VERSION` on the command line as `-D TCC_VERSION=\"0.9.27-decl0-diag\"`; the normal `tcc-musl-v2` build uses `-D TCC_VERSION=\"0.9.27\"`.
  - the focused replay's successful spelling is the local source-level `#undef TCC_VERSION` / `#define TCC_VERSION "0.9.27"` immediately before `tcc_parse_args`.

Conclusion: the latest focused replay did **not** have an active original `TCC_VERSION` macro definition. Its failing `TCC_VERSION` cases exercised unresolved identifier tokens in the predecessor compiler, while the local redefine converted those tokens to a normal string literal. The next correction is to rerun the printf/macro-shape reductions with a replay-faithful command-line `-D TCC_VERSION=\"0.9.27-decl0-diag\"` (or an equivalent generated `config.h`) before treating `TCC_VERSION` as a true macro-expansion blocker.
