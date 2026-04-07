## Why

`crunch self-build` is documented as working, but the tree does not contain a
repeatable proof that self-hosting holds end to end.

Today we have three gaps:

- `README.md` says crunch can build itself with zero Nix runtime dependency.
- `examples/crunch.ncl` still says self-hosting is aspirational.
- There is no in-tree proof run that shows a crunch-built `crunch` binary can
  drive another self-build and produce a working successor.

That leaves an uncomfortable middle state: the feature exists, but the evidence
for it is still a mix of one-off runs, code inspection, and old notes. We need
one proof path that a contributor can run and that the repo can keep as a
regression check.

## What Changes

- Add a repeatable self-hosting proof flow that runs two self-build stages:
  checkout binary -> stage1 binary -> stage2 binary.
- Make the proof assert that stage2 is driven by the stage1 binary, not by the
  original checkout binary.
- Make the proof assert that the second stage selects crunch-built sandbox tools
  when they are present, instead of silently falling back to host tools.
- Document the proof path and retire aspirational wording once the proof exists.

## Capabilities

### New Capabilities
- `self-hosting-proof`: repeatable stage0 -> stage1 -> stage2 self-build proof
- `self-build-proof-report`: stable evidence for which binary and sandbox tools
  each stage used

### Modified Capabilities
- `self-build`: emits enough evidence to prove which bwrap/busybox paths were
  selected during a run
- `documentation`: points at a real proof path instead of placeholder
  self-hosting text

## Impact

- **Files**: `src/self_build.rs`, a slow proof test/helper, `README.md`,
  `examples/crunch.ncl`, OpenSpec bootstrap specs
- **APIs**: internal proof/reporting helpers unless implementation chooses a
  small user-facing wrapper
- **Dependencies**: none new; reuse current self-build prerequisites
- **Testing**: one long-running proof test plus smaller unit coverage for proof
  reporting and stage cleanup behavior
