# Pool transport-death settlement slice (2026-11-01)

Scope: a transport that dies before the protocol starts (worker death) must
fail the goal, settle its attempt, and not leak the endpoint concurrency slot.

## What changed

- `tests/remote_transfer_production.rs`:
  `production_pool_failures_release_attempt_capacity` now runs three phases
  against the same pool state and client store:
  1. a dead transport program (`exit 1`) with a good goal — the goal fails;
  2. the real transport with a failing derivation — the goal fails;
  3. the real transport with a distinct good goal — the goal succeeds.
  The third phase proves both failures settled their attempts and released the
  worker slot. The final goal has a distinct derivation name because the
  coordinator keeps the failed job identity; re-running the same terminal goal
  is a cross-run retry question governed by the existing attempt policy, not
  something this test claims.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 17 passed; 0 failed; 0 ignored` (39.57s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

## Non-claims

- The dead transport exits immediately. A worker that dies mid-exchange after
  partial frames is not exercised here; the existing stdio protocol error paths
  remain the coverage for that case.
- Cross-run retry of the same terminal goal is not implemented or claimed.
- No cancellation, stale fence, duplicate accepted result, or resource
  saturation case is exercised.
