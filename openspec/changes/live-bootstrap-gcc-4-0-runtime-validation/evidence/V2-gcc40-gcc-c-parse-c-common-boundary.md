# V2 GCC 4.0 c-parse c-common boundary

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured

## Change

Extended `bootstrap/diag-gcc40-c-parse-boundary.ncl` to split the `c-common.h` include that is reached by the balanced `c-tree.h` line-25 probe.

The new probes keep the known-good prelude through `c-pragma.h`, open a balanced diagnostic `GCC_C_TREE_H` guard, then test balanced prefixes of `gcc/c-common.h`.

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-cparse-ctree-fine-20260506-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-cparse-ctree-fine-20260506-state" \
  bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl \
  --evidence-dir openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-c-common-20260507 \
  --resume
```

Exit status: `1` (`build-failed`, expected for this diagnostic boundary).

## Evidence

Transcript bundle:

- `evidence/gcc40-cparse-c-common-20260507/doctor.json`
- `evidence/gcc40-cparse-c-common-20260507/build.stdout.log`
- `evidence/gcc40-cparse-c-common-20260507/build.stderr.log`
- `evidence/gcc40-cparse-c-common-20260507/validation-summary.json`
- `evidence/gcc40-cparse-c-common-20260507/validation-summary.md`

Key diagnostic results from `build.stdout.log`:

```text
diag-cparse: cparse_undef6_c_tree_balanced_lines_25 rc=139
diag-c-common-head: 022 #ifndef GCC_C_COMMON_H
diag-c-common-head: 023 #define GCC_C_COMMON_H
diag-c-common-head: 024
diag-c-common-head: 025 #include "splay-tree.h"
diag-c-common-head: 026 #include "cpplib.h"
diag-c-common-head: 027 #include "ggc.h"
diag-cparse: cparse_undef6_c_common_balanced_lines_24 rc=0
diag-cparse: cparse_undef6_c_common_balanced_lines_25 rc=0
diag-cparse: cparse_undef6_c_common_balanced_lines_26 rc=0
diag-cparse: cparse_undef6_c_common_balanced_lines_27 rc=139
```

`cparse_undef6_c_common_balanced_lines_20 rc=139` is a malformed diagnostic artifact because the generated probe appends a balancing `#endif` before the `GCC_C_COMMON_H` guard has opened. The meaningful balanced prefix begins at line 24.

## Conclusion

The `c-common.h` guard, `splay-tree.h`, and repeated `cpplib.h` include pass under the known-good c-parse prelude. The first meaningful failure is `c-common.h` line 27: `#include "ggc.h"`.

Next diagnostic slice: split `ggc.h` under the same prelude plus balanced `GCC_C_TREE_H`/`GCC_C_COMMON_H` context. Do not revisit the earlier malformed `c-tree.h` line-24 or `c-common.h` line-20 prefix artifacts.
