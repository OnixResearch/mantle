# Pool executor overlap slice (2026-11-01)

Scope: prove that the two branch goals of the diamond have overlapping
transport-executor lifetimes in a real two-worker run, not merely that both
workers were used.

## What changed

- `tests/remote_transfer_production.rs`: `run_two_worker_pool_build` accepts an
  optional lifecycle log. The transport wrapper then records
  `start <ns> <pid>` / `end <ns> <pid>` around each worker invocation.
  `production_pool_diamond_worker_lifetimes_overlap` runs the input-addressed
  diamond with that wrapper, asserts one lifetime per goal (four), sorts them by
  start, and requires the second and third lifetimes to overlap
  (`starts[2] < ends[1]`).

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 21 passed; 0 failed; 0 ignored` (66.67s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

The run passes: the two branch workers are alive at the same time while the
shared dependency and the join are strictly ordered around them.

## Non-claims

- Timestamps come from the host's `date +%s%N` in the test wrapper, not from a
  guest or worker clock.
- Overlap is measured for transport-process lifetimes. It does not prove
  simultaneous compiler/sandbox work inside the workers, nor a builder-side
  synchronization barrier.
- The test uses the input-addressed diamond; the CA diamond has its own
  execution-count proof without a lifetime assertion.
