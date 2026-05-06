# GCC 4.0 c-parse post-alias boundary

Date: 2026-05-06

Diagnostic target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
Active target: `bootstrap/gcc-4.0.ncl`

## Change under test

`bootstrap/gcc-4.0.ncl` and the focused diagnostic now make the generated
`insn-modes.h` postcondition explicit: if generated `MIN_MODE_COMPLEX_FLOAT` /
`MAX_MODE_COMPLEX_FLOAT` aliases are absent, append:

```c
#define MIN_MODE_COMPLEX_FLOAT SFmode
#define MAX_MODE_COMPLEX_FLOAT TFmode
```

The focused diagnostic also prints the generated aliases before running the
`c-parse` reduction matrix.

## Validation command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-cparse-full-diag-20260506-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-cparse-full-diag-20260506-state" \
  bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl \
  --evidence-dir openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-full-diag-20260506-r4 \
  --resume
```

Runner result: `BuildFailed`, build exit code `Some(1)`. This is expected for
the focused diagnostic target while it still records failing probes.

## Result

The long warm-cache diagnostic now confirms the alias repair actually affects
the generated header path:

```text
diag-insn-modes-aliases:
#define MIN_MODE_COMPLEX_FLOAT SFmode
#define MAX_MODE_COMPLEX_FLOAT TFmode
diag-insn-modes-head: insn-modes.h:023 #define MIN_MODE_COMPLEX_FLOAT SFmode
diag-insn-modes-head: insn-modes.h:024 #define MAX_MODE_COMPLEX_FLOAT TFmode
```

The previous complex builtin enum seam is cleared under the `config-undef6`
context:

```text
cparse_undef6_tree_builtin_complex_arith_only rc=0
cparse_undef6_tree_builtin_complex_max_only rc=0
cparse_undef6_tree_builtin_complex_min_only rc=0
cparse_undef6_tree_builtin_complex_mode_delta_one rc=0
cparse_undef6_tree_builtin_complex_mode_delta_two rc=0
cparse_undef6_inc_tree rc=0
```

The remaining active boundary is no longer `tree.h` complex builtin enum
arithmetic. It has moved to the real `c-parse.c` include context using the
full generated `config.h`; the earliest direct include probe still failing is:

```text
cparse_inc_system rc=139
```

The surrounding focused probes show the likely next seam is `config.h`/system
macro interaction rather than `system.h` alone:

```text
cparse_ansidecl_system rc=0
cparse_inc_system_only rc=0
ansimacro_inline_empty_sys_types rc=139
ansimacro_inline_builtin_sys_types rc=0
cparse_inc_system rc=139
```

## Evidence files

- Runner bundle: `evidence/gcc40-cparse-full-diag-20260506-r4/`
- Diagnostic excerpt: `evidence/V2-gcc40-gcc-c-parse-post-alias-boundary-build.diag.log`

## Follow-up

The next high-signal slice is a focused `config.h + system.h` macro reduction
under real `c-parse.c` flags, especially around the inline spelling path where
`ansimacro_inline_empty_sys_types` crashes but `ansimacro_inline_builtin_sys_types` passes.
