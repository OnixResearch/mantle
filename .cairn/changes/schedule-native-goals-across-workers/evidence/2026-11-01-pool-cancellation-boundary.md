# Pool cancellation boundary (2026-11-01)

Scope: record why operator cancellation does not yet drive a pool attempt, and
what would have to change before it can.

## Finding

`crates/crunch-build/src/worker.rs` rejects pool dispatch whenever the Worker
has a cancellation receiver:

```rust
Ok(_p) if pool.is_some() => match (pool, pool_goal) {
    (Some(dispatch), Some(goal)) => {
        if cancellation.is_some() {
            // Watch teardown is not proven for a pool executor.
            Err(Error::Store("remote-pool:watch-unsupported".to_string()))
        } else { ... }
```

An attempt to drive an ordinary pool build through
`crunch_pipeline::build_with_owned_cancellation` therefore fails every goal
with `store error: remote-pool:watch-unsupported` before admission. The attempt
was reverted rather than landing a cancellation path the Worker refuses.

Consequence: the `AttemptSettlement` drop guard in the pool executor covers
ordinary error returns and a future dropped for reasons other than the Worker's
cancellation branch (for example, an aborted pipeline future). It does **not**
cover SIGINT on an ordinary pool build, because the Worker refuses to dispatch
a cancellable pool goal at all.

## What a real slice needs

1. Replace the `watch-unsupported` refusal with a cancellable pool dispatch that
   races the cancellation receiver against `dispatch(goal)`.
2. Prove transport-child teardown on that drop: the executor spawns the
   transport with `std::process::Child`, and dropping the future must close
   stdin, observe the child exit, and bound the wait.
3. Prove last-owner semantics: cancellation removes only the cancelling
   waiter's ownership, and a late result cannot complete a replacement attempt.
4. Add a SIGINT integration test with a slow worker and assert the coordinator
   attempt is terminal afterwards and a later goal is admitted.

## Verification

This slice adds no code. The finding is read from `worker.rs:3945-3955`; the
reverted attempt reproduced the failure
(`remote-pool:watch-unsupported`) against a slow pool goal.

## Non-claims

- No cancellation, owned teardown, or last-owner behaviour is claimed for pool
  mode.
- The drop guard remains covered only by the error-return paths exercised in
  `2026-11-01-pool-failure-settlement-slice.md` and
  `2026-11-01-pool-transport-death-slice.md`.
