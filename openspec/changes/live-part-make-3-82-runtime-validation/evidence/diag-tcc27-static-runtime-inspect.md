# TinyCC 0.9.27 static-runtime link-only diagnostic

Task-ID: diagnostic follow-up for V3
Covers: bootstrap.part.make.3.82.runtime-validation
Date: 2026-05-03

## Purpose

`bootstrap/diag-tcc27-static-runtime.ncl` already proved that a smaller TinyCC 0.9.27/Mes static executable segfaults when executed. This follow-up narrows the next seam by linking the same kind of nontrivial static executable but deliberately not executing it, so the build can distinguish link-output creation from runtime startup behavior.

Diagnostic derivation: `bootstrap/diag-tcc27-static-runtime-inspect.ncl`

## Command

```sh
nix shell nixpkgs#bubblewrap -c env \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  ./target/debug/crunch build \
  --store "$PWD/.crunch-drain/current-tcc-runtime-store" \
  --state-dir "$PWD/.crunch-drain/current-tcc-runtime-state" \
  bootstrap/diag-tcc27-static-runtime-inspect.ncl \
  > openspec/changes/live-part-make-3-82-runtime-validation/evidence/diag-tcc27-static-runtime-inspect.current.stdout.log \
  2> openspec/changes/live-part-make-3-82-runtime-validation/evidence/diag-tcc27-static-runtime-inspect.current.stderr.log
```

The first attempts fixed two local prerequisites before the long validation attempt:

- add a writable local store directory, because `--store` does not create it;
- run through `nix shell nixpkgs#bubblewrap` and set a static `SNIX_BUILD_SANDBOX_SHELL`, because the ambient session had no `bwrap` on `PATH` and exported `SNIX_BUILD_SANDBOX_SHELL=/bin/sh`.

## Result

The current run exceeded the 600s tool budget while rebuilding the prerequisite Mes/TinyCC chain and was killed. The captured stderr contains only startup/repair messages before the kill; no diagnostic builder output from `diag-tcc27-static-runtime-inspect.ncl` was reached.

Captured transcripts:

- `evidence/diag-tcc27-static-runtime-inspect.current.stdout.log`
- `evidence/diag-tcc27-static-runtime-inspect.current.stderr.log`

## Interpretation

The diagnostic is still the right smallest seam: if it completes, a successful output means the failure is after static link creation (startup/CRT/runtime execution), while a link failure means the seam is earlier in TCC's static link/library resolution. The local run did not reach that boundary within the foreground tool budget, so V3 remains blocked rather than passed.
