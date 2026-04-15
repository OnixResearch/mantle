# Add determinism regression harness

## Sequence

Step 6 of 6. This change lands after the stricter behavior is specified so the
harness can assert the final operator-facing contract.

## Why

Hermeticity regressions usually arrive through ambient host-state leaks:
`HOME`, `PATH`, `USER`, `TZ`, `LANG`, `TMPDIR`, cwd, or umask. Broad
integration tests do not reliably catch those leaks.

Crunch needs a dedicated regression harness that perturbs ambient state and
checks that output identity plus hermeticity audit facts stay stable.

## What Changes

- add a repo-local determinism regression harness for representative builds
- vary ambient host state across repeated runs
- compare output digests and hermeticity audit facts across runs
- cover at least a normal derivation, a fetcher-rooted build, and a self-build-friendly path

## Capabilities

### New Capabilities

- `determinism-regression-harness`: ambient-state perturbation tests guard crunch against future impurity leaks

## Impact

- **Files**: new regression-test helpers and selected integration tests or scripts
- **Behavior**: no runtime behavior change; this is acceptance coverage
- **Testing**: adds repeated-build matrix coverage for ambient host-state variation
