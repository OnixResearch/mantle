# Pool factory and placement selector (2026-10-31)

Scope: build the pool dispatch the pipeline attaches: one executor per admitted
endpoint over the pipeline's owned transfer capability, and a selector that
chooses an endpoint from resolved goal facts through the pool placement core.

## Shape

- `PoolEndpointPlan` carries an endpoint's client options, worker generation and
  concurrency, declared resource inventory, failure-debug and transfer policies,
  state directory, and declared capabilities.
- `RemotePoolExecutorFactory::new(admitted, endpoints, preferences)` requires the
  plan set to match the admitted endpoint names exactly.
- `PoolExecutorFactory::create(store)` clones the owned transfer capability into
  each `RemotePoolGoalExecutor` and builds a `PoolDispatch` with
  `PoolPolicySelector`.
- `PoolPolicySelector` maps the goal's system and execution modes into a
  `GoalRequirement` (no declared features, accelerator classes, or capacity — so
  nothing is inferred) and calls `plan_goal_placement` with the caller's
  preference order; an unplaceable goal is a `NoFit` denial.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --bin mantle pool_` | 14 passed (executor, factory/selector, preflight, plan) |
| `cargo check -p mantle --bin mantle` | exit 0 |
| nightly `rustfmt --check` on changed files | exit 0 |

Clippy could not be run against the pinned toolchain: the host's Nix daemon
entered auto-GC (min-free 100 GiB, ~77 GiB free) and deleted the devshell
closure, so the pinned clippy is unavailable; the fallback toolchain's newer
clippy flags pre-existing lints in unrelated files. Compilation and tests above
were run from the reconstructed devshell environment with `RUSTC_BOOTSTRAP=1`.

## Not claimed

- The factory is not yet constructed by the CLI; `build --builder-pool` flags and
  an end-to-end routed build remain.
- Placement here uses no observed capacity; a goal with no quantified requirement
  is matched by capability and stable identity only.
