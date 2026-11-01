# Pool goal executor seam (2026-10-31)

Scope: make a pool endpoint an executor of a *prepared goal* rather than a Snix
`BuildService`. The remote client needs the prepared payload, which a bare
`BuildRequest` does not carry, so a `BuildService` endpoint cannot represent it.

## Contract (`crunch_build::distributed`)

- `PoolGoalExecutor::execute(goal)` receives the resolved `PreparedPoolGoal` and
  returns a `BuildResult`.
- `PoolDispatch` endpoints are `PoolEndpointService<Arc<dyn PoolGoalExecutor>>`;
  selection and denials are unchanged.
- `BuildServiceExecutor<S: BuildService>` adapts an endpoint that really is a
  Snix build service; it executes `goal.request` and ignores the extra facts.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p crunch-build --lib distributed::tests::pool` | 3 passed |
| `cargo test -p crunch-build --lib worker::tests::pool_dispatch` | 1 passed |
| strict Clippy `crunch-build`, `cargo fmt --check -p crunch-build` | exit 0 |
| `cairn validate --root . --strict` | 0 issues |

## Remaining gap

The bin-side remote executor still needs the prepared wire payload. Today
`ConcreteBuildRequest.payload` carries a serialized `CrunchDerivation`, which the
Worker never sees: conversion in `crunch-pipeline` reduces it to a
`nix_compat::Derivation`. Until that payload is retained per goal (and carried in
the envelope), a real `build --builder-pool` executor cannot be constructed
without re-deriving or fabricating the payload. That retention is the next step.
