# Pool tampered-output slice (2026-11-01)

Scope: a pool output whose streamed NAR bytes are altered before ingest must
fail the goal and publish nothing.

## What changed

- `src/remote_build/pool_goal_executor.rs`: debug-only seam
  `MANTLE_TEST_TAMPER_POOL_OUTPUT_NAR` flips one payload byte of a streamed NAR
  after admission planning and before ingest.
- `tests/remote_transfer_production.rs`:
  `production_pool_failures_release_attempt_capacity` gains a tampered-output
  phase with a distinct derivation: the goal fails with
  `output-nar-ingest`/`output-nar-digest-mismatch`, the client store count is
  unchanged, and the later goal still runs. The cancellation test now waits for
  the transport pid file before signalling, so SIGINT lands on a live child
  instead of the admission window.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 19 passed; 0 failed; 0 ignored` (42.64s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

## Non-claims

- The seam exists only under `debug_assertions`; release builds have no tamper
  path.
- The rejection is observed at NAR ingest (parser or digest), after signature,
  key, and transfer-manifest validation; those earlier checks are covered by
  their own tests.
- Only the streamed (receiver-spool) NAR is tampered; inline transfer artifacts
  are covered by the existing import-layer tests.
