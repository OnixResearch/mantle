# V2 gcc40 c-parse decl0 normalized libtcc derivation/runtime-marker probe

Task-ID: V2 focused TinyCC `libtcc.c` accumulated-normalization probe for GCC 4.0 c-parse decl0 predecessor build boundary.
Covers: bootstrap.gcc40.runtime-validation

## Commands

```sh
./target/debug/crunch eval bootstrap/diag-gcc40-c-parse-boundary.ncl

timeout 590 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --json --store "$PWD/.crunch-drain/store" --no-substitute bootstrap/diag-gcc40-c-parse-boundary.ncl

openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-derivation-normalized-20260510/run-local-decl0-instrumented-normalized.sh
```

## Evidence

- Run directory: `evidence/gcc40-cparse-decl0-libtcc-derivation-normalized-20260510/`
- Local focused replay: `local-decl0-instrumented-normalized.txt`
- Split phase replay: `local-decl0-split-phases.txt`
- Crunch build JSON: `build.json`
- Crunch build stderr: `build.stderr`
- Derivation log: `/home/brittonr/.local/state/crunch/logs/bwi8ijf4i878cd3m7i6vbximf4iqswhk-diag-gcc40-c-parse-boundary.drv.log`

## Focused results

- `bootstrap/diag-gcc40-c-parse-boundary.ncl` evaluates successfully after adding the accumulated local `libtcc.c` normalizations.
- The full diagnostic build did not reach the target derivation in this run: Crunch reported `dependency bash-2.05b-tcc.drv failed` before executing `diag-gcc40-c-parse-boundary`.
- The restored-output local replay applies the same accumulated normalizations to the predecessor TCC source and compiles full `libtcc.c` with `rc=0`.
- Building the instrumented one-source TinyCC executable still fails with `rc=139` before any `diag-tcc-decl0-runtime:` marker can be emitted.
- The split phase replay proves this is already a compile/add-file phase blocker, not a final link-only blocker:
  - `compile-only-quiet rc=139`, with no object produced.
  - `compile-only-verbose rc=139`, printing `-> -> %s` before the segfault.
  - Both object-link probes fail only because `/tmp/tcc-decl0-normalized.o` is absent after compile-only failure.
  - All expected explicit archives and CRT inputs exist (`libtcc1.a`, `libc.a`, `crt1.o`, `crti.o`, `crtn.o`).
- The latest failing surface is no longer local `libtcc.c` compilation or final object linking. It is predecessor TinyCC source-file add/compile for instrumented one-source `tcc.c`, currently reporting a malformed missing-file diagnostic:
  - `tcc: error: file 'file '%s' not found' not found`

## Interpretation

The local `libtcc.c` compile bisection is exhausted in the cumulatively-normalized predecessor context, and the split replay rules out a final link-only missing input: the predecessor dies while adding/compiling one-source `tcc.c` itself. The next ROI seam is to instrument predecessor `tcc.c`'s `files` list/add-file loop and `tcc_add_file` boundary, because verbose mode shows the source input name has already degraded to `-> %s` before the malformed `file '%s' not found` diagnostic.
