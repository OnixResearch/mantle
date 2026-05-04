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

The first foreground run exceeded the 600s tool budget while rebuilding the prerequisite Mes/TinyCC chain and was killed. The captured stderr contains only startup/repair messages before the kill; no diagnostic builder output from `diag-tcc27-static-runtime-inspect.ncl` was reached.

A follow-up Hermes background run used the same derivation with reusable local state/store:

```sh
nix shell nixpkgs#bubblewrap -c env \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  ./target/debug/crunch build \
  --store "$PWD/.crunch-drain/bg-tcc-runtime-store" \
  --state-dir "$PWD/.crunch-drain/bg-tcc-runtime-state" \
  bootstrap/diag-tcc27-static-runtime-inspect.ncl
```

Background run timing:

- started: 2026-05-03T11:54:30-04:00
- killed: 2026-05-03T12:25:10-04:00 after no final output
- state snapshot before cleanup: `44M` under `.crunch-drain/bg-tcc-runtime-state`, empty logical output store, no saved derivation logs
- tmp scratch snapshot: only the prerequisite `fig7zpsmkr6yvpsa6sf7gpnk8dgw8cji-mes` path existed under the active scratch store

Captured transcripts:

- `evidence/diag-tcc27-static-runtime-inspect.current.stdout.log`
- `evidence/diag-tcc27-static-runtime-inspect.current.stderr.log`
- `evidence/diag-tcc27-static-runtime-inspect.bg.stdout.log`
- `evidence/diag-tcc27-static-runtime-inspect.bg.stderr.log`

## Warmup runner rerun

After adding `crunch bootstrap validate --warmup`, the same diagnostic reached the intended builder with a reusable TinyCC prerequisite:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-store" \
  --state-dir "$PWD/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-state" \
  bootstrap validate bootstrap/diag-tcc27-static-runtime-inspect.ncl \
  --warmup bootstrap/tinycc.ncl \
  --evidence-dir target/bootstrap-validation/diag-tcc27-static-runtime-inspect-warmup-r4
```

Captured evidence copied into this change:

- `evidence/diag-tcc27-static-runtime-inspect-warmup-r4-validation-summary.md`
- `evidence/diag-tcc27-static-runtime-inspect-warmup-r4-validation-summary.json`
- `evidence/diag-tcc27-static-runtime-inspect-warmup-r4-build.stdout.log`
- `evidence/diag-tcc27-static-runtime-inspect-warmup-r4-build.stderr.log`
- `evidence/diag-tcc27-static-runtime-inspect-warmup-r4-warmup-tinycc.stdout.log`
- `evidence/diag-tcc27-static-runtime-inspect-warmup-r4-warmup-tinycc.stderr.log`
- `evidence/diag-tcc27-static-runtime-inspect-warmup-r4-doctor.json`
- `evidence/diag-tcc27-static-runtime-inspect-warmup-r4.derivation.log`
- `evidence/diag-tcc27-static-runtime-inspect-warmup-r4-runtime-probe.log`

Result:

- Warmup `bootstrap/tinycc.ncl`: passed, built `tinycc-0.9.27`.
- Diagnostic `bootstrap/diag-tcc27-static-runtime-inspect.ncl`: passed, built output path `23ghal4ifiqhvs6x4dkg9ql32z489lsn-diag-tcc27-static-runtime-inspect`.
- Derivation log reached all intended seam markers: `compile helper`, `compile main`, and `link static executable`.
- Output contains `bin/diag-tcc27-static-runtime` as a static x86-64 ELF plus preserved `helper.o`, `main.o`, and source files.
- A host runtime probe of the generated executable still exits `139` (`Segmentation fault`).
- The validation runner's coarse host-leakage scan found no host-path needles in captured build output.

## Interpretation

The warmup runner removed the earlier Mes/TinyCC prerequisite-cacheability blocker and the link-only diagnostic now classifies the seam: TinyCC 0.9.27/Mes can compile the two objects and produce a static executable without link-time failure, but the produced executable still segfaults at runtime. V3 remains blocked rather than passed because GNU Make 3.82 has no working output/smoke yet. The next repair target is the TinyCC/Mes static executable runtime startup/relocation path (the same class already implicated by the smaller runtime diagnostic), not bootstrap-validation runner cacheability.
