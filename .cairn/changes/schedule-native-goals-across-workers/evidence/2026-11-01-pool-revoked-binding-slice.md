# Pool revoked-binding slice (2026-11-01)

Scope: a pool endpoint whose durable binding record is revoked must fail
selection before dispatch, with nothing published.

## What changed

- `tests/support/remote_endpoint_binding.rs`: `write_revoked_endpoint_binding`
  writes one builder record with `revoked: true` plus the config that names it.
- `tests/remote_transfer_production.rs`:
  `production_pool_rejects_revoked_endpoint_binding_before_dispatch` runs
  `build --builder-pool` against that table. Admission fails with a
  `remote-pool` error, the transport program is never executed, and the client
  store stays empty.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 18 passed; 0 failed; 0 ignored` (39.44s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

The core revocation rule was already covered by
`crates/crunch-remote-core/src/pool_tests.rs`; this slice proves the CLI pool
path reaches it before any transport or store effect.

## Non-claims

- Expired (non-revoked) bindings and generation changes are not exercised here.
- The transport program in this test is `/bin/true` with a placeholder digest;
  it never runs because admission fails first, so this is not transport
  evidence.
