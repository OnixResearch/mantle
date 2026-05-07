# V2 GCC 4.0 c-parse c-tree include boundary

## Command

```sh
PATH=/nix/store/f020kzmpd027v8p4a4nyljiww06f3ml3-bubblewrap-0.11.0/bin:$PATH \
./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-cparse-ctree-fine-20260506-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-cparse-ctree-fine-20260506-state" \
  bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl \
  --evidence-dir openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-ctree-fine-20260506 \
  --resume
```

## Result

- Doctor: OK.
- Build attempted: yes.
- Build status: expected diagnostic `build-failed` at the c-parse TinyCC boundary.
- Summary: `evidence/gcc40-cparse-ctree-fine-20260506/validation-summary.md`.
- Full build report: `evidence/gcc40-cparse-ctree-fine-20260506/build.stdout.log`.
- Saved builder log: `.crunch-drain/gcc40-cparse-ctree-fine-20260506-state/logs/ijd01k9wwdmy48bm4sfa83s900hr6sm6-diag-gcc40-c-parse-boundary.drv.log`.

## New boundary

The previous run showed the fully accumulated `c-tree.h` include set crashed. This run split the handoff at `c-tree.h` itself:

```text
diag-cparse: cparse_undef6_inc_c_pragma rc=0
diag-cparse: cparse_undef6_c_tree_lines_20 rc=0
diag-cparse: cparse_undef6_c_tree_lines_24 rc=139
diag-cparse: cparse_undef6_c_tree_lines_25 rc=139
diag-cparse: cparse_undef6_c_tree_lines_26 rc=139
diag-cparse: cparse_undef6_c_tree_lines_27 rc=139
diag-cparse: cparse_undef6_c_tree_lines_31 rc=139
diag-cparse: cparse_undef6_c_tree_lines_38 rc=139
diag-cparse: cparse_undef6_c_tree_lines_40 rc=139
diag-cparse: cparse_undef6_c_tree_lines_80 rc=139
diag-cparse: cparse_undef6_c_tree_lines_120 rc=139
diag-cparse: cparse_undef6_c_tree_lines_200 rc=139
diag-cparse: cparse_undef6_inc_c_tree rc=139
```

`c-tree.h` line 20 is the license-comment terminator. Line 24 is the header guard opening after:

```c
#ifndef GCC_C_TREE_H
#define GCC_C_TREE_H
```

So the current minimized boundary is not the later `c-tree.h` payload or its `c-common.h`/`diagnostic.h` includes yet: TinyCC segfaults as soon as the generated probe appends the guarded `c-tree.h` front matter after the now-passing `config-undef6.h + system/coretypes/tm/tree/langhooks/input/cpplib/intl/timevar/c-pragma` include stack.

## Interpretation

The six autohost undefs remain valid: the same run keeps `cparse_undef6_inc_c_pragma rc=0`, while `c-tree.h` line 24 flips to `rc=139`. The next slice should make this probe print the exact preprocessed token window around the guard, or split the wrapper shape itself (`#ifndef` only, `#define` only, balanced empty guard) to tell whether this is a TinyCC preprocessor conditional-state bug triggered by include-stack depth rather than a specific GCC declaration.
