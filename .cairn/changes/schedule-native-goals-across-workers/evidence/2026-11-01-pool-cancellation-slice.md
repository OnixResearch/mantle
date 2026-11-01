# Pool cancellation slice (2026-11-01)

Scope: operator cancellation (`SIGINT`/`SIGTERM`) now stops an in-flight pool
goal, observes transport-child teardown, settles the attempt, and releases the
endpoint slot.

## What changed

- `crates/crunch-build/src/distributed.rs`: `PoolGoalExecutor` gains
  `execute_cancellable` (default ignores the signal). `PoolDispatch` gains
  `dispatch_cancellable`; cancellation is never retried on another endpoint.
- `crates/crunch-build/src/worker.rs`: the `remote-pool:watch-unsupported`
  refusal is replaced with a cancellable pool dispatch using the Worker's
  watch receiver.
- `src/remote_build.rs`: the stdio runner publishes its transport child's
  process-group leader in a shared kill slot, clears it after teardown, and
  exposes `run_stdio_remote_child_with_kill`.
- `src/remote_build/pool_goal_executor.rs`: `execute_cancellable` refuses a goal
  already cancelled, otherwise spawns a bounded watcher that kills the
  transport process group and runs the goal; the drop guard settles the
  attempt.
- `src/build_cmd.rs`: pool builds install an operator signal monitor and drive
  `build_with_owned_cancellation`.
- `docs/remote-pool.md`: cancellation section describes the implemented
  behavior.
- Tests: `production_pool_cancellation_settles_the_attempt` (spawn, wait for a
  running attempt, SIGINT, assert the attempt is `failed`, assert the transport
  child pid recorded by the wrapper is gone within 10 s, then a follow-up goal
  succeeds) and `pool_dispatch_does_not_retry_cancellation_on_another_endpoint`.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 19 passed; 0 failed; 0 ignored` (64.46s) |
| `cargo test -p crunch-build --lib -- pool_dispatch -- --test-threads 1` | `test result: ok. 14 passed; 0 failed` |
| `cargo test -p crunch-build --lib -- distributed worker -- --test-threads 1` | `test result: ok. 224 passed; 0 failed` |
| `cargo clippy -p crunch-build -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle -p crunch-build` | exit 0 |

## Non-claims

- The watcher polls the kill slot for up to 100 ms after the signal, so a
  cancellation racing the spawn may miss the child; the pre-dispatch flag check
  covers earlier arrivals. The test therefore waits for the transport pid file
  before signalling.
- Only `SIGINT` is exercised; `SIGTERM` shares the same monitor.
- The process-group kill relies on the runner's `setpgid` setup; non-Unix
  platforms keep the no-op group configuration and are untested.
- The teardown probe uses `kill(pid, 0)`; a pid reuse inside the 10 s window
  would mask a live child (not observed).
