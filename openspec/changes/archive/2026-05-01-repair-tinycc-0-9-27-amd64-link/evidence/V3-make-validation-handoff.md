# V3 Make 3.82 validation handoff

Task-ID: V3
Covers: bootstrap.part.make.3.82.amd64.runtime-validation

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store "$PWD/.crunch-drain/tcc-got-static4-store" \
  --state-dir "$PWD/.crunch-drain/tcc-got-static4-state" \
  bootstrap/make-tcc.ncl
```

## Result

Status: blocker recorded 2026-05-01T12:41:35-04:00

The repaired TinyCC static hello smoke passes, but the downstream GNU Make 3.82
validation handoff still fails while building `bootstrap/make-tcc.ncl`.

Failure class:

- root: `make-3.82-tcc`
- phase: build
- error_class: builder
- exit status: 139 / segmentation fault
- preserved transcript:
  `evidence/V3-make-validation-handoff.log`
- scratch derivation log during the run:
  `.crunch-drain/tcc-got-static4-state/logs/bwkphpkw52sz9zf3xqlyj8bqnqavn1ym-make-3.82-tcc.drv.log`

## Interpretation

This records the next concrete downstream blocker for Make 3.82 runtime
validation. It is follow-on work for `live-part-make-3-82-runtime-validation`,
not a blocker for this TinyCC static-link repair: V1 and V2 prove the repaired
TinyCC builds, links, and executes a minimal static executable without host-tool
substitution.
