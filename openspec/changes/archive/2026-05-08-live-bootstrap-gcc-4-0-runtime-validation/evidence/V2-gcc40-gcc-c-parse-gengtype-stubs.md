# V2 GCC 4.0 c-parse generated gengtype stubs

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured

## Change

Seeded the GCC build directory with the inert generated header forest emitted by the existing `build/gengtype` wrapper before `c-parse.c` can include `c-tree.h`/`c-common.h`/`ggc.h`.

This covers `gtype-desc.h` plus the related `gt-*.h`/`gtype-c.h` files. Missing generated headers previously routed through TinyCC/Mes diagnostic paths and produced misleading segfaults before the c-parse diagnostic could reach the next real boundary.

## Diagnostic command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-cparse-ctree-fine-20260506-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-cparse-ctree-fine-20260506-state" \
  bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl \
  --evidence-dir openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-gtype-desc-20260507 \
  --resume
```

Exit status: `1` (`build-failed`, expected for this diagnostic boundary).

## Evidence

Transcript bundle:

- `evidence/gcc40-cparse-gtype-desc-20260507/doctor.json`
- `evidence/gcc40-cparse-gtype-desc-20260507/build.stdout.log`
- `evidence/gcc40-cparse-gtype-desc-20260507/build.stderr.log`
- `evidence/gcc40-cparse-gtype-desc-20260507/validation-summary.json`
- `evidence/gcc40-cparse-gtype-desc-20260507/validation-summary.md`

Key diagnostic results from `build.stdout.log` after seeding generated headers:

```text
diag-cparse: cparse_undef6_c_tree_balanced_lines_25 rc=0
diag-cparse: cparse_undef6_c_tree_balanced_lines_26 rc=0
diag-cparse: cparse_undef6_c_tree_balanced_lines_80 rc=0
diag-cparse: cparse_undef6_c_common_balanced_lines_27 rc=0
diag-cparse: cparse_undef6_ggc_balanced_lines_40 rc=0
diag-cparse: cparse_undef6_ggc_balanced_lines_100 rc=0
diag-gtype-desc-head: 001 /* bootstrap gengtype stub: gtype-desc.h */
```

The `cparse_undef6_gtype_desc_balanced_lines_* rc=139` probes are diagnostic artifacts: the generated stub is a one-line comment with no include guard, while the helper appends a balancing `#endif`. They do not represent the GCC build path.

## Conclusion

The c-tree/c-common/ggc include chain no longer crashes once the generated gengtype header forest is present. The production bootstrap now seeds those stubs before invoking the fragile GCC make phase, matching the existing `build/gengtype` wrapper output but avoiding the missing-header segfault path.
