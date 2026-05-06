# V2 gcc-4.0 `c-parse.c` mode-enum arithmetic boundary

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store "$PWD/.crunch-drain/gcc40-cparse-mode-enum-boundary-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-cparse-mode-enum-boundary-state" \
  bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Transcript: `evidence/V2-gcc40-gcc-c-parse-mode-enum-boundary-build.diag.log`.

The diagnostic intentionally exits nonzero because the full `c-parse.c` compile
still segfaults; this slice records the next narrower complete-source boundary
after the prior `builtins.def` / complex enum arithmetic reduction.

## Result

This run keeps the sanitized `config-undef6.h + system.h + coretypes.h + tm.h`
context and the balanced `tree.h` prefix through line 180. It adds focused
manual `enum built_in_function` probes around the complex builtin range
expressions that use generated machine-mode enum identifiers from
`insn-modes.h`.

Key transcript lines:

```text
diag-cparse: cparse_undef6_tree_builtin_include_full rc=0
diag-cparse: cparse_undef6_tree_builtin_include_plus_complex rc=139
diag-mode-bound: 015   SFmode,
diag-mode-bound: 018   TFmode,
diag-cparse: cparse_undef6_tree_builtin_complex_literal_zero rc=0
diag-cparse: cparse_undef6_tree_builtin_complex_literal_sub rc=0
diag-cparse: cparse_undef6_tree_builtin_complex_max_only rc=139
diag-cparse: cparse_undef6_tree_builtin_complex_min_only rc=139
diag-cparse: cparse_undef6_tree_builtin_complex_mode_delta_one rc=139
diag-cparse: cparse_undef6_tree_builtin_complex_mode_delta_two rc=139
diag-cparse: cparse_undef6_tree_builtin_include_plus_literal_sub rc=0
diag-cparse: cparse_undef6_tree_builtin_include_plus_mode_delta_one rc=139
exit_code: 1
completed_utc: 2026-05-06T20:28:36Z
```

The log also prints the `tree.h` arithmetic source window:

```text
diag-tree-head: 195       + MAX_MODE_COMPLEX_FLOAT
diag-tree-head: 196       - MIN_MODE_COMPLEX_FLOAT,
diag-tree-head: 201       + MAX_MODE_COMPLEX_FLOAT
diag-tree-head: 202       - MIN_MODE_COMPLEX_FLOAT,
```

## Interpretation

The subtraction/addition operator shape is not sufficient by itself: explicit
literal arithmetic such as `BUILT_IN_COMPLEX_MUL_MIN + 2 - 1` passes both with
and without the full `builtins.def` include.

The generated mode enum identifiers are sufficient to reproduce the crash under
the same complete-source prefix. Even a single reference to either endpoint
crashes:

- `BUILT_IN_COMPLEX_MUL_MIN + MAX_MODE_COMPLEX_FLOAT` -> `rc=139`;
- `BUILT_IN_COMPLEX_MUL_MIN - MIN_MODE_COMPLEX_FLOAT` -> `rc=139`;
- `BUILT_IN_COMPLEX_MUL_MIN + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT` -> `rc=139`.

This narrows the active seam away from `builtins.def` expansion and away from
generic enum arithmetic. The next repair/probe should inspect TinyCC's handling
of generated `insn-modes.h` enum constants as operands in later enum constant
expressions, especially around the `SFmode`/`TFmode` values reported by the
mode-bound window.

## Next probe

Reduce the generated identifier dependency further:

1. create a minimal local `enum machine_mode { SFmode = 15, TFmode = 18 }` and
   use `MAX_MODE_COMPLEX_FLOAT` / `MIN_MODE_COMPLEX_FLOAT` aliases outside the
   full generated `insn-modes.h` include;
2. test direct `SFmode` / `TFmode` references versus the macro/enum aliases;
3. if the minimal enum passes, reduce the generated `insn-modes.h` declaration
   shape around complex-float mode aliases; if it fails, switch to a TinyCC enum
   constant-expression reducer for cross-enum references.
