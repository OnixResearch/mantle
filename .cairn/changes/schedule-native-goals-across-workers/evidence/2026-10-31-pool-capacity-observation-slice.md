# Pool capacity observation slice (2026-10-31)

Scope: the capacity facts a pool dispatcher needs before choosing a worker. They
are computed from configuration inventory minus the coordinator's recorded live
leases — never from concurrency, worker presence, or a probe.

## Decision

`remote_build::pool_config::observed_capacity(pool, active_leases)` reports per
endpoint:

- `declares_resource_inventory` — whether the endpoint configured a quantified
  inventory at all;
- `remaining_cpu_units`, `remaining_memory_bytes`, `remaining_scratch_bytes` —
  present only when an inventory exists, computed with the existing
  `crunch_build::distributed::remote_resource_remaining_capacity`, which
  subtracts that endpoint's recorded leases and refuses an overcommitted
  snapshot;
- `active_lease_count` — recorded leases for that endpoint.

`remote pool check` now loads the coordinator state, passes its
`resource_leases`, and includes the observations in the
`mantle-remote-pool-preflight-v1` report as `capacity`. An endpoint with no
inventory reports absent values rather than zero or a guess.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --bin mantle remote_build::pool_config::tests::` | 6 passed (adds capacity observation with a real planned lease) |
| real CLI `remote pool check` | exit 0, report carries `capacity` with absent values for inventory-free endpoints |
| strict Clippy (mantle targets), rustfmt | exit 0 |
| `cairn validate --root . --strict` | 0 issues |

The fixture plans a real lease with `plan_remote_resource_reservation` and
asserts the observation drops from 4/8192/8192 to 3/7168/7168 with one active
lease, while the inventory-free endpoint reports `null` values.

## Not claimed

- These are configuration-minus-recorded-lease facts, not a live worker probe,
  health check, or reservation.
- S5 must still reserve through the fenced planner before dispatch; an
  observation alone never authorizes work.
