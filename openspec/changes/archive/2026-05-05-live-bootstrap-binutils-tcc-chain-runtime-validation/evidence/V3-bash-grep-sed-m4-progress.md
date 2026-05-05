# V3 binutils-tcc parent-chain progress: bash/grep/sed to m4 boundary

Focused validation advanced the `bootstrap/binutils-tcc.ncl` parent chain past the previous Bash link closure and through the next text-tool prerequisites.

## Passing focused validations

- `bootstrap/bash-2.05b-tcc.ncl`: passed focused validation after completing the TinyCC static-link closure and chmod-normalizing the output shell. Evidence: `V3-bash-tcc-success-validation-summary.json`, `V3-bash-tcc-success-validation-summary.md`, `V3-bash-tcc-success-build.stdout.log`.
- `bootstrap/grep-2.4-musl.ncl`: passed focused validation with a small bootstrap grep bridge after the GNU grep 2.4 driver repeatedly segfaulted this TinyCC handoff while compiling `src/grep.c`. Evidence: `V3-grep-musl-success-validation-summary.json`, `V3-grep-musl-success-validation-summary.md`, `V3-grep-musl-success-build.stdout.log`.
- `bootstrap/sed-4.0.9-musl.ncl`: passed focused validation using the previously validated `sed-tcc` runtime as a temporary bridge after the GNU sed 4.0.9 musl rebuild segfaulted compiling `lib/getline.c`. Evidence: `V3-sed-musl-success-validation-summary.json`, `V3-sed-musl-success-validation-summary.md`, `V3-sed-musl-success-build.stdout.log`.

## Current boundary

A focused retry of `bootstrap/binutils-tcc.ncl` now reaches:

```text
binutils-2.30-tcc.drv -> m4-1.4.7-musl.drv
```

Direct focused validation of `bootstrap/m4-1.4.7-musl.ncl` currently fails with a TinyCC/runtime `Segmentation fault (core dumped)` before a more detailed source boundary is exposed. Evidence: `V3-binutils-tcc-m4-boundary-validation-summary.json`, `V3-binutils-tcc-m4-boundary-build.stdout.log`, `V3-m4-musl-boundary-validation-summary.json`, `V3-m4-musl-boundary-build.stdout.log`.

V3 remains incomplete until the parent epoch chain validates through `binutils-2.30-tcc` and the remaining V4/V5 audits are recorded.
