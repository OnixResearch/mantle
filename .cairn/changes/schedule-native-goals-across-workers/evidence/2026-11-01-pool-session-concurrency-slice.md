# Pool session concurrency slice (2026-11-01)

Scope: the admitted pool session `max_concurrency` now bounds concurrent goal
dispatches, not just the inventory admission check.

## What changed

- `crates/crunch-build/src/distributed.rs`: `PoolDispatch` carries an optional
  session semaphore. `with_session_limit(max)` creates it;
  `dispatch_with` acquires a permit for the whole goal (cancellation-aware while
  waiting) and releases it when the goal returns or is cancelled.
- `src/remote_build/pool_factory.rs`: the factory wires the admitted pool's
  `session().max_concurrency` into the dispatch.
- `tests/remote_transfer_production.rs`:
  `production_pool_respects_the_session_concurrency_limit` runs the two-worker
  diamond with session `max_concurrency = 1` and requires the four transport
  lifetimes to be pairwise disjoint.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 24 passed; 0 failed; 0 ignored` (52.80s) |
| `cargo test -p crunch-build --lib -- pool_dispatch -- --test-threads 1` | `test result: ok. 14 passed; 0 failed` |
| `cargo clippy -p crunch-build -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle -p crunch-build` | exit 0 |

## Non-claims

- The semaphore bounds one `PoolDispatch` instance in one process; concurrent
  processes sharing a pool are not coordinated by it.
- Endpoint concurrency and the global `--jobs` bound still apply independently;
  the effective limit is the tightest of the three.
- Quantified resource leases are still not enforced in placement.
