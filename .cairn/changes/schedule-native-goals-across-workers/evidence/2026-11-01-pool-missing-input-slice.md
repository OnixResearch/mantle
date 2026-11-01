# Pool missing-input slice (2026-11-01)

Scope: a pool goal whose derivation names a source input that is absent from
the client store must fail closed before any transfer, worker dispatch, or
publication.

## What changed

- `tests/remote_transfer_production.rs`:
  `production_pool_failures_release_attempt_capacity` gains a missing-input
  phase. The derivation declares `inputs = [<known source path>]` but the path
  is never materialized. The build fails with
  `missing-source-input: source input not found in store`, the client store
  entry count is unchanged, and the final good goal still runs (attempt
  settlement).

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 18 passed; 0 failed; 0 ignored` (33.67s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

## Non-claims

- The failure happens during pipeline source admission, before the pool
  executor prepares input transfers; the executor's own
  `remote-input-source-path-missing` path is not exercised by this phase.
- No private/secret credential or environment leak case is covered.
