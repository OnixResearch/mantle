# Pool global job-limit slice (2026-11-01)

Scope: the ordinary `--jobs` bound must constrain how many pool goals execute
at once, independent of endpoint concurrency.

## What changed

- `tests/remote_transfer_production.rs`: `run_two_worker_pool_build` accepts an
  optional `--jobs` value.
  `production_pool_respects_the_global_job_limit` runs the two-worker diamond
  with `--jobs 1` and a lifecycle-logging wrapper, then requires the four
  transport lifetimes to be pairwise disjoint. The shared lifetime parser moved
  to `worker_lifetimes`.
- The multi-ticket FD setup now duplicates every ticket source above the target
  range before `dup2`, so a parent fd equal to another target cannot overwrite
  it (observed as `remote-ticket-input-format-invalid` under the full suite).

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 23 passed; 0 failed; 0 ignored` (108.10s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

## Non-claims

- The test proves the global `--jobs` bound serializes pool goals; it does not
  exercise the pool session `max_concurrency`, per-endpoint concurrency, or
  quantified resource leases together.
- Worker lifetimes are measured by the test wrapper, not by in-sandbox work.
- With `--jobs 1` placement may use only one endpoint; the test does not require
  both workers.
