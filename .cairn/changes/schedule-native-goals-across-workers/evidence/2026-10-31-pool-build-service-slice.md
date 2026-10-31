# Pool execution dispatcher slice (2026-10-31)

Scope: the Mantle-level executor that sends one prepared goal to exactly one
admitted pool endpoint. It owns no placement policy, reservation, or fallback: an
injected `PoolGoalSelector` decides from resolved facts, and the selected
endpoint's own `BuildService` performs the build.

## Correction recorded

The earlier `PoolBuildService` wrapped a `BuildService` and selected from a bare
`BuildRequest`. That cannot be correct: the request carries no realization key,
required system/features, sandbox/network mode, semantic accelerator classes, or
prepared capacity, so any request-only selector would have to invent them. It was
removed rather than wired, and replaced by `PoolDispatch`, which takes the
resolved `PreparedPoolGoal` envelope.

## Contract (`crunch_build::distributed`)

- `PoolDispatch::new(endpoints, selector)` pairs `PoolEndpointService` entries
  holding `Arc<dyn BuildService>` with a `PoolGoalSelector`; an empty pool and
  empty/duplicate endpoint ids are rejected.
- `dispatch(goal)` validates the concrete request through the existing
  `validate_remote_build_service_request`, asks the selector for one endpoint
  index, and calls that endpoint's service. Selector denials surface as
  `remote-pool:*` invalid-input errors; no local fallback is attempted.
- `PreparedPoolGoal`/`PoolGoalSelector` carry and consume the resolved facts.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p crunch-build --lib distributed::tests::` | 48 passed |
| strict Clippy `crunch-build`, `cargo fmt --check -p crunch-build` | exit 0 |
| `cairn validate --root . --strict` | 0 issues |

Fixtures use two recording services: the selected endpoint receives the goal and
the other receives none; a selector whose facts do not match denies without
calling either; an out-of-range index, empty pool, and duplicate ids are
rejected; an invalid request is rejected before selection.

## Not claimed

- The Worker does not construct `PreparedPoolGoal` yet; wiring the Builder seam
  to this dispatcher is the remaining step.
- Routing alone proves no reservation, transport, output verification, or
  execution correctness.
