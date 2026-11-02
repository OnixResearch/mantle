# Waiter-notification count slice (2026-11-01)

Scope: the Worker reports how many waiters it woke, so a diamond or chain can
assert one notification per dependent goal.

## What changed

- `crates/crunch-build/src/worker.rs`: `WorkerResult` gains
  `waiter_notifications`; `complete_goal` adds the drained waiter count with
  saturating arithmetic.
- `crates/crunch-build/src/orchestrate.rs`: the empty-roots result reports 0.
- Tests: `run_diamond_builds_shared_once` asserts 2 (shared wakes both
  branches, roots have no waiters); `run_chain_builds_in_order` asserts 2
  (dep wakes mid, mid wakes top).

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p crunch-build --lib -- --test-threads 1` | `test result: ok. 789 passed; 0 failed` |
| `cargo test -p crunch-build --lib -- run_diamond_builds_shared_once run_chain_builds_in_order -- --test-threads 1` | both pass |
| `cargo check -p mantle --all-targets` | exit 0 |
| `cargo clippy -p crunch-build --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p crunch-build` | exit 0 |

## Non-claims

- The count is per Worker run, not a per-goal report row and not yet serialized
  in the JSON build report.
- One earlier full-suite run failed `production_workspace_local_dispatch_stack_reaches_owned_watch_cancel`
  with "watch sandbox did not start"; the test and the full suite pass on
  rerun, so it is treated as an environment flake.
