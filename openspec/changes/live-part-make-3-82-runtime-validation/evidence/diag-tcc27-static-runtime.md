# TinyCC 0.9.27 static-runtime diagnostic

Task-ID: diagnostic follow-up for V3
Covers: bootstrap.part.make.3.82.runtime-validation
Date: 2026-05-03

## Purpose

`bootstrap/make-tcc.ncl` currently builds far enough to produce or start the GNU Make 3.82 runtime-smoke boundary, but the resulting binary exits via a segmentation fault. This focused diagnostic narrows that class away from GNU Make source behavior by compiling and executing a smaller nontrivial static executable with the same bootstrapped `tinycc-0.9.27` runtime path.

Diagnostic derivation: `bootstrap/diag-tcc27-static-runtime.ncl`

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store "$PWD/.crunch-drain/diag-tcc27-runtime-store" \
  --store-prefix /crunch/store \
  --state-dir "$PWD/.crunch-drain/diag-tcc27-runtime-state" \
  bootstrap/diag-tcc27-static-runtime.ncl \
  > openspec/changes/live-part-make-3-82-runtime-validation/evidence/diag-tcc27-static-runtime.stdout.log \
  2> openspec/changes/live-part-make-3-82-runtime-validation/evidence/diag-tcc27-static-runtime.stderr.log
```

Exit status: `1` from `crunch build`.

Crunch saved derivation log:

- `evidence/diag-tcc27-static-runtime.derivation.log`

Top-level transcripts:

- `evidence/diag-tcc27-static-runtime.stdout.log`
- `evidence/diag-tcc27-static-runtime.stderr.log`

## Result

The diagnostic compiled both objects and linked the static executable successfully, then failed only when executing the resulting binary:

```text
diag: compile helper
diag: compile main
diag: link static executable
diag: execute static executable
Segmentation fault (core dumped)
```

The sandbox builder exited with status `139`.

## Interpretation

This reproduces the Make 3.82 runtime failure class with a smaller TinyCC 0.9.27/Mes static executable. The next repair target should remain in the TinyCC/Mes static runtime/link-output path rather than piling more GNU Make source edits onto `bootstrap/make-tcc.ncl`.

V3 stays blocked: there is still no usable GNU Make output to smoke-test.
