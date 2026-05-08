# GCC 4.0 c-parse tree.h boundary

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store $PWD/.crunch-drain/gcc40-cparse-tree-boundary-r2-store \
  --state-dir $PWD/.crunch-drain/gcc40-cparse-tree-boundary-r2-state \
  bootstrap/diag-gcc40-c-parse-boundary.ncl
```

## Result

- Diagnostic command completed with exit code `1` because the diagnostic target intentionally returns failure after probing the remaining TinyCC crashes.
- Transcript: `V2-gcc40-gcc-c-parse-tree-boundary-build.diag.log`.
- The prior config/`tm.h` setup still passes under the `config-undef6.h` boundary:

```text
diag-cparse: cparse_undef6_inc_system rc=0
diag-cparse: cparse_undef6_inc_coretypes rc=0
diag-cparse: cparse_undef6_inc_tm rc=0
```

## Boundary evidence

The added `tree.h` probes narrow the next crash to line 25 of the generated GCC 4.0 `gcc/tree.h` prefix:

```text
diag-tree-head: 022 #ifndef GCC_TREE_H
diag-tree-head: 023 #define GCC_TREE_H
diag-tree-head: 024
diag-tree-head: 025 #include "machmode.h"
diag-tree-head: 026 #include "input.h"
diag-tree-head: 027 #include "statistics.h"
diag-tree-head: 028 #include "vec.h"
```

Observed return codes:

```text
diag-cparse: cparse_undef6_tree_guard_only rc=0
diag-cparse: cparse_undef6_tree_machmode_only rc=139
diag-cparse: cparse_undef6_tree_input_only rc=0
diag-cparse: cparse_undef6_tree_statistics_only rc=0
diag-cparse: cparse_undef6_tree_vec_only rc=0
diag-cparse: cparse_undef6_tree_def_enum rc=0
diag-cparse: cparse_undef6_tree_lines_23 rc=0
diag-cparse: cparse_undef6_tree_lines_25 rc=139
diag-cparse: cparse_undef6_tree_lines_26 rc=139
diag-cparse: cparse_undef6_tree_lines_27 rc=139
diag-cparse: cparse_undef6_tree_lines_28 rc=139
diag-cparse: cparse_undef6_tree_lines_36 rc=139
diag-cparse: cparse_undef6_tree_lines_80 rc=139
diag-cparse: cparse_undef6_inc_tree rc=139
```

The full diagnostic repeats the same boundary later in the run and confirms the original full `c-parse.c` compile still segfaults after these narrowed probes:

```text
diag-cparse: cparse_full_config_undef6 rc=139
diag-cparse: cparse_full rc=139
```

## Interpretation

This slice moves the active blocker from broad `tree.h`/frontend-header setup to TinyCC 0.9.27-musl-v2 handling of GCC 4.0's `machmode.h` when parsed in the real GCC frontend context (`config-undef6.h`, `system.h`, `coretypes.h`, `tm.h`). `tree.h`'s guard-only prefix succeeds, and sibling early includes (`input.h`, `statistics.h`, `vec.h`) succeed when isolated. The next high-ROI probe should inspect `gcc/machmode.h` under the same include context before attempting another full `c-parse.c` fix.
