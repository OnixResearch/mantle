# V2 GCC 4.0 c-parse TinyCC source trace

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation
Status: captured
Date: 2026-05-07

## Target

- Diagnostic: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
- Evidence dir: `evidence/gcc40-cparse-tcc-source-trace-20260507/`

## Captured command

```sh
timeout 600 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-cparse-tcc-source-trace-20260507-store" \
  --state-dir "$PWD/.crunch-drain/gcc40-runtime-v4-20260507-state" \
  bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl \
  --evidence-dir openspec/changes/live-bootstrap-gcc-4-0-runtime-validation/evidence \
  --resume
```

## Result

- Result: captured expected build-failed diagnostic boundary; runtime reached `diag-gcc40-c-parse-boundary`.
- The diagnostic now imports the fixed TinyCC 0.9.27 source and prints `tccgen.c` source anchors/windows for the declaration parser path.
- The relevant source path is `parse_btype` -> `type_decl`/`post_type` -> `decl0`, with `decl0` deciding declaration continuation, semicolon termination, initializer handling, and function-body handoff.
- Existing runtime boundary remains unchanged: valid semicolon declarations pass, while malformed or EOF/incomplete declarator paths return `rc=139`.
- Top-level phase tracing remains a dead end: `-S` returns `rc=60` with literal `%s` diagnostics and `-E` segfaults even on a valid semicolon declaration.

## Artifacts

- `gcc40-cparse-tcc-source-trace-20260507/source-trace.md`
- `gcc40-cparse-tcc-source-trace-20260507/source-trace.json`
- `gcc40-cparse-tcc-source-trace-20260507/source-trace-excerpt.log`
- `gcc40-cparse-tcc-source-trace-20260507/build.stdout.log`
- `gcc40-cparse-tcc-source-trace-20260507/build.stderr.log`
- `gcc40-cparse-tcc-source-trace-20260507/doctor.json`
- `gcc40-cparse-tcc-source-trace-20260507/validation-summary.json`
- `gcc40-cparse-tcc-source-trace-20260507/validation-summary.md`

## Next reduction

Use this source map for the next narrow slice: instrument or repair TinyCC `tccgen.c::decl0` / declaration-error recovery directly. Do not add more top-level source-shape probes unless they exercise a new branch in this mapped path.
