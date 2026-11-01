# Pool cache-hit slice (2026-11-01)

Scope: a repeated pool build of the same derivation must use the client store
cache and must not spawn another worker.

## What changed

- `tests/remote_transfer_production.rs`:
  `production_pool_second_build_hits_the_client_cache` runs the same pool build
  twice against one state/client store with a lifecycle-logging transport
  wrapper. The first report has `cached: false`, the second `cached: true`, and
  the lifecycle log holds exactly one worker invocation.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 26 passed; 0 failed; 0 ignored` (94.32s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

## Non-claims

- The hit is the client store's ordinary PathInfo cache check; this test does
  not separately prove signature verification beyond the store's existing
  checks.
- One input-addressed fixture is exercised; no CA cache-hit variant.
- No cross-process or remote-cache comparison is made.
