# V2 gcc40 c-parse decl0 libtcc tcc_compile body reduction

Task-ID: V2 focused TinyCC `libtcc.c` `tcc_compile` body-reduction diagnostic for the GCC 4.0 c-parse `decl0` predecessor compile boundary.
Covers: bootstrap.gcc40.runtime-validation

## Command

The replay runs under a `/crunch/store` bind because the restored TinyCC and musl outputs embed logical Crunch store paths:

```sh
BASH=$(command -v bash)
nix shell nixpkgs#bubblewrap -c bwrap \
  --dev /dev --proc /proc --tmpfs /tmp --dir /crunch \
  --bind "$PWD/.crunch-drain/post621-restored-store" /crunch/store \
  --ro-bind /nix /nix --ro-bind /usr /usr \
  --ro-bind /run/current-system /run/current-system --ro-bind /home /home \
  --chdir "$PWD" "$BASH" \
  openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-tcc-compile-body-20260509/run-local-compile-body.sh \
  > openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-tcc-compile-body-20260509/focused-local-compile-body.txt 2>&1
```

Exit status: 0.

## Evidence

- Run directory: `evidence/gcc40-cparse-decl0-libtcc-tcc-compile-body-20260509/`
- Script: `evidence/gcc40-cparse-decl0-libtcc-tcc-compile-body-20260509/run-local-compile-body.sh`
- Focused log: `evidence/gcc40-cparse-decl0-libtcc-tcc-compile-body-20260509/focused-local-compile-body.txt`
- Replay inputs:
  - TinyCC: `.crunch-drain/post621-restored-store/b557z7jmjqnlz118c8zvwdjvkv4v5qb3-tcc-0.9.27-musl-v2/bin/tcc`
  - musl headers: `.crunch-drain/post621-restored-store/g38fpdz6wrs7n2p5f3afqbcb3w4s2m64-musl-1.1.24-tcc-musl/include`
  - TinyCC source: `.crunch-drain/post621-restored-store/4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src`

## Focused results

All variants were run under both common and `ONE_SOURCE=1` flags.

| Variant | common | one_source |
| --- | ---: | ---: |
| `prefix621` | 0 | 0 |
| `proto` | 0 | 0 |
| `empty` | 0 | 0 |
| `locals` | 0 | 0 |
| `begin` | 0 | 0 |
| `setjmp_empty` | 0 | 0 |
| `setjmp_flags` | 0 | 0 |
| `full_no_codegen` | 0 | 0 |
| `direct_tccgen` | 0 | 0 |
| `preprocess_branch` | 0 | 0 |
| `asm_branch` | 0 | 0 |
| `else_tccgen_branch` | 0 | 0 |
| `full_branch_no_asm_ifdef` | 0 | 0 |
| `full_branch_no_return_ternary` | 0 | 0 |
| `full_original` | 139 | 139 |
| `full_libtcc_no_compile_asm_ifdef` | 139 | 139 |

## Interpretation

With the `native387_disabled` prerequisite applied, the line-621 prefix and reduced `tcc_compile` bodies all compile successfully. The complete original `tcc_compile` body still crashes the predecessor TinyCC (`rc=139`) under both flag sets.

The reduction narrows the first post-line-621 trigger to the `CONFIG_TCC_ASM` preprocessor island inside `tcc_compile`: replacing that island with the active `tcc_error_noabort("asm not supported")` branch makes the complete `tcc_compile` body pass (`full_branch_no_asm_ifdef`), while the unmodified body fails (`full_original`). However, the full `libtcc.c` file with only that island simplified still fails, so additional later full-file triggers remain after this local body-level trigger.

No direct `diag-tcc-decl0-runtime:*` markers executed; this remains predecessor-compile evidence only.
