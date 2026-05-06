# V2 gcc-4.0 `c-parse.c` `builtins.def` / complex-enum boundary

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store "$PWD/.crunch-drain/gcc40-cparse-builtins-boundary-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-cparse-builtins-boundary-state" \
  bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Transcript: `evidence/V2-gcc40-gcc-c-parse-builtins-boundary-build.diag.log`.

The diagnostic intentionally exits nonzero because the full `c-parse.c` compile
still segfaults; this slice records a narrower, ordered probe boundary after the
prior `tree.h` post-`insn-modes.h` reduction.

## Result

This run keeps the sanitized `config-undef6.h + system.h + coretypes.h + tm.h`
context and reuses a balanced `tree.h` prefix through line 180. It then isolates
the `enum built_in_function` area by separating `builtins.def` inclusion from the
manual complex builtin enum arithmetic that follows the include in `tree.h`.

Key transcript lines:

```text
diag-cparse: cparse_undef6_tree_lines_180 rc=0
diag-cparse: cparse_undef6_tree_lines_207 rc=139
diag-cparse: cparse_undef6_tree_lines_212 rc=139
diag-cparse: cparse_undef6_tree_lines_220 rc=139
diag-cparse: cparse_undef6_tree_builtin_empty rc=0
diag-cparse: cparse_undef6_tree_builtin_complex_arith_only rc=139
diag-cparse: cparse_undef6_tree_builtin_include_full rc=0
diag-cparse: cparse_undef6_tree_builtin_include_plus_complex rc=139
diag-cparse: cparse_undef6_builtins_def_lines_20 rc=139
diag-cparse: cparse_undef6_builtins_def_lines_60 rc=139
diag-cparse: cparse_undef6_builtins_def_lines_120 rc=139
diag-cparse: cparse_undef6_builtins_def_lines_240 rc=0
diag-cparse: cparse_undef6_builtins_def_lines_480 rc=0
diag-cparse: cparse_undef6_builtins_def_lines_800 rc=0
diag-cparse: cparse_undef6_inc_tree rc=139
exit_code: 1
completed_utc: 2026-05-06T19:55:42Z
```

The `builtins.def` head window printed by the diagnostic confirms that early
prefixes are mostly comments/macro definitions; the first active macro wrapper
region starts around the `DEF_GCC_BUILTIN` / `DEF_LIB_BUILTIN` definitions, while
later complete prefixes include actual builtin entries.

## Interpretation

`builtins.def` inclusion by itself is not the current complete-source blocker:
including the full file inside a minimal `enum built_in_function` passes with
`rc=0`. The crash returns only when the following complex builtin range enum
arithmetic is present:

```c
BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN
  + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
BUILT_IN_COMPLEX_DIV_MAX = BUILT_IN_COMPLEX_DIV_MIN
  + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
```

The prefix-count probes also show why naive `builtins.def` prefix bisection is
misleading: 20/60/120-line prefixes crash, but larger 240/480/800-line prefixes
pass. Those early failures are malformed macro/backslash or comment-definition
windows, not a real included-builtin enumerator boundary.

The next real seam is therefore TinyCC's handling of enum constant expressions
that subtract/add the generated machine-mode enum values in this `tree.h`
context, not `builtins.def` enumerator expansion itself.

## Next probe

Reduce the complex enum arithmetic under the same sanitized context:

1. print and probe `MIN_MODE_COMPLEX_FLOAT` / `MAX_MODE_COMPLEX_FLOAT` numeric
   definitions from generated `insn-modes.h`;
2. try equivalent explicit-number enum arithmetic in the `tree.h` prefix;
3. try same-identifier arithmetic without subtraction, then with subtraction;
4. if only the generated identifiers fail, inspect their enum declaration shape
   from `insn-modes.h`; if explicit numbers also fail, switch to a TinyCC enum
   constant-expression buglet reduction.
