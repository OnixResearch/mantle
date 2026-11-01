# Pool untrusted-output-key slice (2026-11-01)

Scope: a pool worker whose result is signed by a key the client did not admit
must fail the goal without publishing anything into the client store.

## What changed

- `tests/remote_transfer_production.rs`:
  `production_pool_failures_release_attempt_capacity` now has a third failure
  phase: the same real transport and worker run under a pool configuration
  whose `trusted_output_keys` name is `wrong-builder-key` while the worker still
  signs with `cache.example.com-1`. The goal fails with
  `untrusted-output-key`, the client store entry count is unchanged, and a
  later goal under the correct pool still succeeds (attempt settlement).

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 17 passed; 0 failed; 0 ignored` (35.79s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

## Non-claims

- Only the key-name mismatch case is exercised. Same-name-different-material,
  tampered NAR bytes, missing/extra outputs, revoked bindings, and private-input
  cases are not covered by this phase.
- The unchanged client-store count proves no output export for this run; it does
  not prove absence of all backend writes (PathInfo, castore scratch) on every
  rejection path.
