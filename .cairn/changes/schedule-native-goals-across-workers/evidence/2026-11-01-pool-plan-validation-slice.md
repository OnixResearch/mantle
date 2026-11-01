# Pool plan-mode validation slice (2026-11-01)

Scope: `build --plan` accepts a pool and validates its configuration, endpoint
bindings, and transport-program identities without credentials, transport
sockets, uploads, reservations, or dispatch.

## What changed

- `src/main.rs`: `--plan` is no longer a clap conflict for `--builder-pool`.
  Plan mode calls a plan-only pool validator and does not construct the pool
  executor factory, so no ticket descriptors are read.
- `src/remote_build/pool_factory.rs`: `plan_pool_selection` loads the farm,
  projects the selected pool against current bindings, and re-measures each
  transport program against its declared BLAKE3 identity.
- `tests/smoke.rs`: `pool_plan_validates_selection_without_tickets` runs
  `--plan` with a pool config and binding, no ticket FDs, and asserts the
  `crunch-build-plan-v1` report; an unknown pool id fails with
  `remote-pool:pool-unknown`.
- `docs/remote-pool.md`: CLI section now documents plan-mode acceptance and
  what it does not do.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test smoke pool_ -- --test-threads 1` | `test result: ok. 2 passed` |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 19 passed; 0 failed` (52.38s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

## Non-claims

- Plan mode validates the selection; it does not render intended per-goal pool
  placements or predicted routes.
- Credential-reference uniqueness and the full `admits_pool` table are not
  re-evaluated in plan mode; ticket descriptors are deliberately skipped.
