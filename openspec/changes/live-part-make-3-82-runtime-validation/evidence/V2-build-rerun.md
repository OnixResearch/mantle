# V2 build rerun evidence: make 3.82 runtime validation

Task-ID: V2
Covers: bootstrap.part.make.3.82.runtime-validation

## Command

```text
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/make-3-82-rerun-store" \
  --state-dir "$PWD/.crunch-drain/make-3-82-rerun-state" \
  bootstrap validate bootstrap/make-tcc.ncl \
  --evidence-dir openspec/changes/live-part-make-3-82-runtime-validation/evidence \
  --resume
```

## Result

- Validation status: `build-failed`
- Failure class: `build`
- Build exit code: `1`
- Doctor/preflight OK: `True`
- Failed root: `make-3.82-tcc`
- Failed derivation key: `/crunch/store/bwkphpkw52sz9zf3xqlyj8bqnqavn1ym-make-3.82-tcc.drv`
- Saved derivation transcript: `evidence/build.derivation.log`
- Runner summary: `evidence/validation-summary.json` and `evidence/validation-summary.md`

The improved validation runner completed instead of hanging. It captured a structured
build failure for GNU Make 3.82 under the bootstrapped TinyCC/Mes path. The builder
exited with status 139 after many TinyCC/Mes diagnostics with literal `%s:%d` and
`%s` placeholders, ending in:

```text
%s:%d: warning: assignment makes pointer from integer without a cast
%s:%d: warning: assignment makes pointer from integer without a cast
%s:%d: warning: assignment makes pointer from integer without a cast
%s:%d: warning: implicit declaration of function '%s'
%s:%d: warning: implicit declaration of function '%s'
%s:%d: warning: assignment makes pointer from integer without a cast
%s:%d: warning: assignment makes pointer from integer without a cast
%s:%d: warning: implicit declaration of function '%s'
%s:%d: warning: implicit declaration of function '%s'
%s:%d: warning: implicit declaration of function '%s'
%s:%d: warning: implicit declaration of function '%s'
Segmentation fault (core dumped)
```

## Interpretation

This replaces the earlier `incomplete-hung-validation-run` evidence with an
actionable build failure: `make-3.82-tcc` does not produce a GNU Make output yet,
so runtime smoke testing remains blocked on repairing the TinyCC/Mes Make build
segmentation fault.
