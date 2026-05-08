# V2 GCC 4.0 c-parse c-tree balanced-prefix boundary

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured

## Change

Extended `bootstrap/diag-gcc40-c-parse-boundary.ncl` with `c-tree.h` front-matter probes that separate malformed prefix artifacts from the real include boundary:

- print the first 40 lines of `gcc/c-tree.h`;
- test manual guard fragments after the already-passing prelude through `c-pragma.h`;
- test balanced `c-tree.h` prefixes by appending a diagnostic `#endif` after the sampled prefix.

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-cparse-ctree-fine-20260506-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-cparse-ctree-fine-20260506-state" \
  bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl \
  --evidence-dir openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-ctree-balanced-20260507 \
  --resume
```

Exit status: `1` (`build-failed`, expected for this diagnostic boundary).

## Evidence

Transcript bundle:

- `evidence/gcc40-cparse-ctree-balanced-20260507/doctor.json`
- `evidence/gcc40-cparse-ctree-balanced-20260507/build.stdout.log`
- `evidence/gcc40-cparse-ctree-balanced-20260507/build.stderr.log`
- `evidence/gcc40-cparse-ctree-balanced-20260507/validation-summary.json`
- `evidence/gcc40-cparse-ctree-balanced-20260507/validation-summary.md`

Key diagnostic results from `build.stdout.log`:

```text
diag-cparse: cparse_undef6_inc_c_pragma rc=0
diag-c-tree-head: 022 #ifndef GCC_C_TREE_H
diag-c-tree-head: 023 #define GCC_C_TREE_H
diag-c-tree-head: 024
diag-c-tree-head: 025 #include "c-common.h"
diag-c-tree-head: 026 #include "diagnostic.h"
diag-cparse: cparse_undef6_c_tree_lines_20 rc=0
diag-cparse: cparse_undef6_c_tree_guard_ifndef_only rc=139
diag-cparse: cparse_undef6_c_tree_guard_define_only rc=0
diag-cparse: cparse_undef6_c_tree_guard_empty_balanced rc=0
diag-cparse: cparse_undef6_c_tree_balanced_lines_24 rc=0
diag-cparse: cparse_undef6_c_tree_balanced_lines_25 rc=139
diag-cparse: cparse_undef6_c_tree_balanced_lines_26 rc=139
diag-cparse: cparse_undef6_inc_c_tree rc=139
```

## Conclusion

The previous line-24 failure is a malformed-prefix artifact: an unterminated `#ifndef GCC_C_TREE_H` crashes TinyCC, while the balanced empty guard passes. The first meaningful balanced `c-tree.h` boundary is line 25, `#include "c-common.h"`, which still segfaults under the known-good `config-undef6.h + system.h + coretypes.h + tm.h + tree.h + langhooks.h + input.h + cpplib.h + intl.h + timevar.h + c-pragma.h` prelude.

Next diagnostic slice: split `c-common.h` include/front matter under the same prelude, starting with `c-common.h` header guard and first include/define block, rather than revisiting `c-tree.h` line-24 or autohost aliases.
