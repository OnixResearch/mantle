# V2 gcc-4.0 `c-parse.c` post-`insn-modes.h` `tree.h` boundary

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store "$PWD/.crunch-drain/gcc40-cparse-tree-post-insn-boundary-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-cparse-tree-post-insn-boundary-state" \
  bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Transcript: `evidence/V2-gcc40-gcc-c-parse-tree-post-insn-boundary-build.diag.log`.

The diagnostic intentionally exits nonzero because the full `c-parse.c` compile
still segfaults; the useful result is the ordered post-`insn-modes.h` header
probe boundary.

## Result

This run follows the cleared generated `insn-modes.h` seam and re-tests `tree.h`
under the same sanitized `config-undef6.h + system.h + coretypes.h + tm.h`
context. The prior broad `tree.h`/`machmode.h` failure is now narrower:

```text
diag-cparse: cparse_undef6_tree_guard_only rc=0
diag-cparse: cparse_undef6_tree_machmode_only rc=0
diag-cparse: cparse_undef6_tree_input_only rc=0
diag-cparse: cparse_undef6_tree_statistics_only rc=0
diag-cparse: cparse_undef6_tree_vec_only rc=0
diag-cparse: cparse_undef6_tree_def_enum rc=0
diag-cparse: cparse_undef6_tree_lines_23 rc=0
diag-cparse: cparse_undef6_tree_lines_25 rc=0
diag-cparse: cparse_undef6_tree_lines_26 rc=0
diag-cparse: cparse_undef6_tree_lines_27 rc=0
diag-cparse: cparse_undef6_tree_lines_28 rc=0
diag-cparse: cparse_undef6_tree_lines_36 rc=139
diag-cparse: cparse_undef6_tree_lines_80 rc=0
diag-cparse: cparse_undef6_tree_lines_120 rc=0
diag-cparse: cparse_undef6_tree_lines_160 rc=0
diag-cparse: cparse_undef6_tree_lines_166 rc=0
diag-cparse: cparse_undef6_tree_lines_177 rc=0
diag-cparse: cparse_undef6_tree_lines_180 rc=0
diag-cparse: cparse_undef6_tree_lines_207 rc=139
diag-cparse: cparse_undef6_tree_lines_212 rc=139
diag-cparse: cparse_undef6_tree_lines_220 rc=139
diag-cparse: cparse_undef6_inc_tree rc=139
```

The generated `tree.h` window for the newly failing later prefix shows the first
large complete construct is the non-language-specific builtin enum:

```text
diag-tree-head: 185 #define DEF_BUILTIN(ENUM, N, C, T, LT, B, F, NA, AT, IM, COND) ENUM,
diag-tree-head: 186 enum built_in_function
diag-tree-head: 187 {
diag-tree-head: 188 #include "builtins.def"
diag-tree-head: 189
diag-tree-head: 190   /* Complex division routines in libgcc.  These are done via builtins
diag-tree-head: 191      because emit_library_call_value can't handle complex values.  */
diag-tree-head: 192   BUILT_IN_COMPLEX_MUL_MIN,
diag-tree-head: 193   BUILT_IN_COMPLEX_MUL_MAX
diag-tree-head: 194     = BUILT_IN_COMPLEX_MUL_MIN
diag-tree-head: 195       + MAX_MODE_COMPLEX_FLOAT
diag-tree-head: 196       - MIN_MODE_COMPLEX_FLOAT,
diag-tree-head: 197
diag-tree-head: 198   BUILT_IN_COMPLEX_DIV_MIN,
diag-tree-head: 199   BUILT_IN_COMPLEX_DIV_MAX
diag-tree-head: 200     = BUILT_IN_COMPLEX_DIV_MIN
diag-tree-head: 201       + MAX_MODE_COMPLEX_FLOAT
diag-tree-head: 202       - MIN_MODE_COMPLEX_FLOAT,
diag-tree-head: 203
diag-tree-head: 204   /* Upper bound on non-language-specific builtins.  */
diag-tree-head: 205   END_BUILTINS
diag-tree-head: 206 };
diag-tree-head: 207 #undef DEF_BUILTIN
```

## Interpretation

`machmode.h` plus the generated `insn-modes.h` is no longer the next real
frontend blocker. Complete early `tree.h` constructs through the tree-code enum,
class enum, vector declaration, and built-in class enum pass when parsed as
balanced probes. The next reproducible complete-prefix crash is at the
`enum built_in_function` / `builtins.def` expansion boundary: line 180 still
passes, while line 207 and later fail with `rc=139`.

The line-36 failure is retained in the transcript as a malformed partial enum
prefix signal, but it is not the active complete-source boundary because the
balanced line-80 through line-180 probes pass.

## Next probe

Reduce `tree.h`'s `built_in_function` construct under the same sanitized context:
start with direct `builtins.def` include/minimal `DEF_BUILTIN` expansion probes,
then isolate whether TinyCC is failing on the included builtin enumerators or on
the `MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT` enum arithmetic that
follows them.
