# Pool capacity queueing slice (2026-11-01)

Scope: a pool with fewer slots than ready goals must queue a goal until an
endpoint has capacity instead of failing it with `no-endpoint-with-capacity`.

## What changed

- `crates/crunch-build/src/distributed.rs`: `PoolDispatch::dispatch_with` now
  loops when every endpoint reports `worker-concurrency-limit`. It re-selects
  after 50 ms, honours cancellation while waiting, and fails with
  `no-endpoint-with-capacity` only after 12,000 attempts (ten minutes).
- `tests/remote_transfer_production.rs`:
  `production_pool_diamond_runs_on_one_endpoint_at_concurrency_one` builds the
  four-goal diamond through one endpoint with session `max_concurrency = 1` and
  asserts the joined graph. The shared single-endpoint fixture now provisions
  eight ticket uses so multi-goal fixtures fit.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 20 passed; 0 failed; 0 ignored` (49.22s) |
| `cargo test -p crunch-build --lib -- pool_dispatch -- --test-threads 1` | `test result: ok. 14 passed; 0 failed` |
| `cargo clippy -p crunch-build -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle -p crunch-build` | exit 0 |

## Non-claims

- Queueing is bounded by ten minutes; a permanently saturated endpoint fails
  the goal after that bound.
- There is no fairness or priority between queued goals; the Worker's own
  scheduling decides which goal is dispatched next.
- Overlap of two executors on two workers remains unmeasured; the diamond
  proves exact execution counts, not simultaneous lifetimes.
