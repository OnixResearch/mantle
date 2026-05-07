# V2 GCC 4.0 c-parse ggc boundary

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured

## Change

Extended `bootstrap/diag-gcc40-c-parse-boundary.ncl` to split the `ggc.h` include reached by `c-common.h` line 27.

The probes keep the known-good c-parse prelude through `c-pragma.h`, open balanced diagnostic `GCC_C_TREE_H` and `GCC_C_COMMON_H` guards, include the passing `c-common.h` prerequisites (`splay-tree.h`, `cpplib.h`), then test balanced `ggc.h` prefixes.

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-cparse-ctree-fine-20260506-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-cparse-ctree-fine-20260506-state" \
  bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl \
  --evidence-dir openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-ggc-20260507 \
  --resume
```

Exit status: `1` (`build-failed`, expected for this diagnostic boundary).

## Evidence

Transcript bundle:

- `evidence/gcc40-cparse-ggc-20260507/doctor.json`
- `evidence/gcc40-cparse-ggc-20260507/build.stdout.log`
- `evidence/gcc40-cparse-ggc-20260507/build.stderr.log`
- `evidence/gcc40-cparse-ggc-20260507/validation-summary.json`
- `evidence/gcc40-cparse-ggc-20260507/validation-summary.md`

Key diagnostic results from `build.stdout.log`:

```text
diag-cparse: cparse_undef6_c_common_balanced_lines_27 rc=139
diag-ggc-head: 022 #ifndef GCC_GGC_H
diag-ggc-head: 023 #define GCC_GGC_H
diag-ggc-head: 024 #include "statistics.h"
diag-ggc-head: 030 extern const char empty_string[];
diag-ggc-head: 038 typedef void (*gt_pointer_operator) (void *, void *);
diag-ggc-head: 040 #include "gtype-desc.h"
diag-cparse: cparse_undef6_ggc_balanced_lines_24 rc=0
diag-cparse: cparse_undef6_ggc_balanced_lines_25 rc=0
diag-cparse: cparse_undef6_ggc_balanced_lines_30 rc=0
diag-cparse: cparse_undef6_ggc_balanced_lines_40 rc=139
```

`cparse_undef6_ggc_balanced_lines_20 rc=139` is a malformed unmatched-endif diagnostic artifact before the `GCC_GGC_H` guard opens.

## Conclusion

`ggc.h` passes through its guard, `statistics.h`, early extern declarations, and `gt_pointer_operator`. The first meaningful balanced failure is line 40: `#include "gtype-desc.h"`.

Next diagnostic slice: split generated `gtype-desc.h` under the same prelude plus balanced `GCC_C_TREE_H`/`GCC_C_COMMON_H`/`GCC_GGC_H` context. Do not revisit malformed prefix artifacts.
