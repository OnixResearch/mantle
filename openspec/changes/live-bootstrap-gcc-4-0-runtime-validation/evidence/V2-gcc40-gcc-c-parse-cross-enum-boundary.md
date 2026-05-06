# V2 gcc-4.0 `c-parse.c` cross-enum / undefined-identifier boundary

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store "$PWD/.crunch-drain/gcc40-cparse-cross-enum-boundary-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-cparse-cross-enum-boundary-state" \
  bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Transcript: `evidence/V2-gcc40-gcc-c-parse-cross-enum-boundary-build.diag.log`.

The diagnostic intentionally exits nonzero because the full `c-parse.c` compile
still segfaults; this slice records the next narrower boundary after the mode
identifier probes.

## Result

This run adds two probe classes under the same sanitized
`config-undef6.h + system.h + coretypes.h + tm.h` context and balanced `tree.h`
prefix through line 180:

1. a no-`machmode.h` prefix with a minimal local `enum machine_mode` and local
   `MIN_MODE_COMPLEX_FLOAT` / `MAX_MODE_COMPLEX_FLOAT` aliases; and
2. generated-context direct `SFmode` / `TFmode` probes, explicit aliases for
   `MIN_MODE_COMPLEX_FLOAT` / `MAX_MODE_COMPLEX_FLOAT`, and deliberately
   undefined enum-expression operands.

Key transcript lines:

```text
diag-mode-bound: 015   SFmode,
diag-mode-bound: 018   TFmode,
diag-cparse: cparse_undef6_tree_builtin_complex_max_only rc=139
diag-cparse: cparse_undef6_tree_builtin_complex_min_only rc=139
diag-cparse: cparse_undef6_tree_builtin_complex_mode_delta_one rc=139
diag-cparse: cparse_undef6_tree_builtin_include_plus_mode_delta_one rc=139
diag-cparse: cparse_undef6_tree_builtin_local_machine_mode_literal_sub rc=0
diag-cparse: cparse_undef6_tree_builtin_local_machine_mode_direct_sftf rc=0
diag-cparse: cparse_undef6_tree_builtin_local_machine_mode_macro_delta rc=0
diag-cparse: cparse_undef6_tree_builtin_local_machine_mode_enum_alias_delta rc=0
diag-cparse: cparse_undef6_tree_builtin_generated_direct_sftf rc=0
diag-cparse: cparse_undef6_tree_builtin_generated_direct_tf_only rc=0
diag-cparse: cparse_undef6_tree_builtin_generated_direct_sf_only rc=0
diag-cparse: cparse_undef6_tree_builtin_generated_macro_alias_delta rc=0
diag-cparse: cparse_undef6_tree_builtin_generated_enum_alias_delta rc=0
diag-cparse: cparse_undef6_tree_builtin_undefined_symbol_only rc=139
diag-cparse: cparse_undef6_tree_builtin_undefined_symbol_delta rc=139
exit_code: 1
completed_utc: 2026-05-06T21:37:05Z
```

The generated `insn-modes.h` head window confirms the direct enum constants are
present:

```text
diag-insn-modes-head: insn-modes.h:005 enum machine_mode {
diag-insn-modes-head: insn-modes.h:015   SFmode,
diag-insn-modes-head: insn-modes.h:018   TFmode,
diag-insn-modes-head: insn-modes.h:019   MAX_MACHINE_MODE,
diag-insn-modes-head: insn-modes.h:020   NUM_MACHINE_MODES = MAX_MACHINE_MODE
```

## Interpretation

The active seam is now narrower than cross-enum arithmetic or generated
`insn-modes.h` shape:

- local machine-mode enum arithmetic passes;
- generated direct `TFmode` / `SFmode` arithmetic passes;
- generated-context aliases that define `MIN_MODE_COMPLEX_FLOAT` and
  `MAX_MODE_COMPLEX_FLOAT` to `SFmode` / `TFmode` pass; but
- the original `tree.h` names `MIN_MODE_COMPLEX_FLOAT` and
  `MAX_MODE_COMPLEX_FLOAT` still crash, and arbitrary undefined identifiers in
  enum constant expressions reproduce the same `rc=139` failure class.

So the most likely immediate blocker is that this diagnostic path has not
materialized GCC's expected `MIN_MODE_COMPLEX_FLOAT` / `MAX_MODE_COMPLEX_FLOAT`
definitions before `tree.h` consumes them, and TinyCC 0.9.26/Mes crashes rather
than issuing a normal undefined-identifier diagnostic for enum constant
expressions.

## Next probe / repair

Patch the focused gcc-4.0 derivation path to materialize the missing mode-class
bound aliases before compiling `tree.h` / `c-parse.c`:

```c
#define MIN_MODE_COMPLEX_FLOAT SFmode
#define MAX_MODE_COMPLEX_FLOAT TFmode
```

Then rerun the focused `c-parse.c` diagnostic. If it moves past the complex
builtin enum area, continue reducing the next `tree.h`/frontend boundary. If it
still crashes at full `tree.h`, the next seam is after line 202 rather than the
complex builtin range arithmetic.
