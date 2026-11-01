# Pool failure settlement slice (2026-11-01)

Scope: a pool goal whose exchange fails must not leave its attempt Running and
hold the endpoint's concurrency slot against later goals.

## What changed

- `src/remote_build/pool_goal_executor.rs`: the admitted work moved to
  `run_admitted_goal`. When it returns an error, `settle_failed_attempt` applies
  `fail_remote_production_attempt` (retryable class) at a strictly later event
  number under the same coordinator transaction lock and revision retry, so the
  attempt leaves `Running` and its lease is released. The original failure stays
  the reported outcome; settlement is best effort.
- `tests/remote_transfer_production.rs`:
  `production_pool_failed_goal_releases_attempt_capacity` runs a failing
  derivation and then a good derivation through the same single-endpoint pool
  and client store. The second build succeeding proves the slot was released.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 17 passed; 0 failed; 0 ignored` (38.18s) |
| `cargo test -p crunch-build --lib -- distributed worker -- --test-threads 1` | `test result: ok. 224 passed; 0 failed; 0 ignored` |
| `cargo clippy -p crunch-build -p crunch-remote-core -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle -p crunch-build -p crunch-remote-core` | exit 0 |

## Non-claims

- Settlement covers an exchange that returns an error to the executor. A
  process death between admission and settlement still needs external
  reconciliation; no crash-recovery claim is made.
- Retryable vs terminal classification is the existing attempt policy; this
  slice does not add retries.
- No last-owner cancellation, remote teardown, stale-result refusal, or
  worker-death case is exercised here.
