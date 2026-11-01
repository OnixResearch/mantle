# Pool mid-exchange worker death slice (2026-11-01)

Scope: a worker that dies after the exchange starts must fail the goal, publish
nothing, and release the endpoint slot for a later goal.

## What changed

- `tests/remote_transfer_production.rs`:
  `production_pool_failures_release_attempt_capacity` gains a mid-exchange
  worker-death phase. Its pool endpoint uses a wrapper that starts the real
  `remote serve` worker in the background and SIGKILLs it after one second; the
  goal is a slow derivation. The build fails with a transport failure
  (`stdio remote child`/`TransportSetup`), the client store count is unchanged,
  and the final good goal still runs.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 19 passed; 0 failed; 0 ignored` (38.13s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

## Non-claims

- The kill happens one second after the wrapper starts the worker; a worker
  death at another point in the exchange is not separately exercised.
- The wrapper uses host `sleep` from the test environment's PATH.
- The test observes failure and settlement, not the exact exit status or stderr
  of the killed worker.
