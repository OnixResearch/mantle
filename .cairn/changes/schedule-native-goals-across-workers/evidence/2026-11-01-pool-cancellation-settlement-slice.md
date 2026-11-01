# Pool attempt-settlement guard slice (2026-11-01)

Scope: an admitted pool attempt must be settled when its goal returns an error
or when the executor future is dropped for cancellation.

## What changed

- `src/remote_build/pool_goal_executor.rs`: replaced the explicit
  error-only settlement with `AttemptSettlement`, a drop guard created after
  admission and disarmed only on success. Ordinary error returns and a dropped
  executor future both run the same `fail_remote_production_attempt` (retryable)
  transition under the coordinator save lock and revision retry.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 17 passed; 0 failed; 0 ignored` (34.71s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

The failure phases (dead transport, failing derivation, untrusted key) exercise
the drop guard on error returns and prove a later goal is admitted.

## Non-claims

- A future dropped for cancellation is not covered by an integration test; the
  guard is shared with the tested error path.
- A process kill between admission and guard drop bypasses settlement; external
  reconciliation is still required for that case.
- Settlement performs bounded synchronous file IO on the dropping thread while
  holding the coordinator save lock; it never awaits.
