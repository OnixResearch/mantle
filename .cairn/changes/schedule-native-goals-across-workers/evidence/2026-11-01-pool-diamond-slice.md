# Two-worker pool diamond slice (2026-11-01)

Scope: execute a real four-goal diamond (shared, left, right, join) through
`build --builder-pool` with two admitted stdio endpoints, each backed by its own
`mantle remote serve` worker, and admit the final output into the client store.

## What changed

- `tests/remote_transfer_production.rs`:
  - `production_pool_dispatches_a_diamond_across_two_workers` builds an
    input-addressed diamond, binds two endpoints through `build_bindings`,
    supplies one ticket descriptor per endpoint, and asserts the joined graph
    bytes plus a non-empty physical store on both workers;
  - ticket descriptors are opened independently per target FD (a shared open
    file description is consumed to EOF by the first reader);
  - pool endpoint profiles declare a bounded resource inventory, which
    concurrent session admission requires.
- `tests/support/remote_endpoint_binding.rs`: `write_pool_endpoint_bindings`
  writes the singular `build` plus additional `build_bindings` records.
- `src/remote_build/pool_goal_executor.rs`:
  - dependency inputs travel as the dependency's built output paths resolved
    from the goal closure, not as `.drv` paths (neither store keeps `.drv` as a
    stored object);
  - coordinator admission and completion reload and reapply on
    `remote-coordinator-save-conflict` (bounded attempts), so concurrent goals
    cannot overwrite each other's attempt or lease;
  - `start` → `result-ready` → `completion` use strictly increasing event
    numbers.
- `crates/crunch-build/src/distributed.rs`: when the selected endpoint reports
  `worker-concurrency-limit`, dispatch tries the next endpoint once each. This
  is a placement correction for the selection race, not a build retry; a new
  unit test pins it.
- `src/remote_build/endpoint_config.rs`, `src/remote_build.rs`: a pool receiver
  resolves and admits the presented binding by endpoint name, so a worker that
  serves `build_bindings` no longer fails startup against the singular
  `build` entry.
- `src/remote_build.rs`: coordinator save assertions run while the save lock is
  still held; `commit` releases it.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 14 passed; 0 failed; 0 ignored` (44.81s) |
| `cargo test -p crunch-build --lib -- pool_dispatch` | `test result: ok. 5 passed; 0 failed; 0 ignored` |
| `cargo test -p mantle --bin mantle -- endpoint_config::tests endpoint_binding remote_serve` | `test result: ok. 11 passed; 0 failed; 0 ignored` |
| `cargo clippy -p crunch-build -p crunch-remote-core -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle -p crunch-build -p crunch-remote-core` | exit 0 |

The strict gate also required two repairs on this checkout unrelated to pool
behavior: `chunks_exact(2)` → `as_chunks::<2>()` in
`src/source_built_stagex_programs_middle.rs` and a struct-literal indentation
fix in `tests/android_sources_offline.rs`.

## Non-claims

- Both workers are on this host over stdio loopback; no remote host, real
  transport binary, or network condition is exercised.
- The diamond uses input-addressed derivations. Content-addressed derivations
  currently fail the pool goal JSON round-trip because
  `nix_compat::derivation::Output` serializes an unset path as `null` and its
  deserializer rejects `null`; that gap is recorded, not fixed.
- No cancellation, worker death, stale fence, capacity shrink, dynamic-plan
  return, or saturated/unknown resource case is exercised.
- Both worker stores being non-empty proves each endpoint executed at least one
  goal; it is not a deterministic placement or overlap measurement.
- The assertion is now exact: the two worker stores together hold exactly four
  exported outputs, one per diamond goal (shared, left, right, join), so no goal
  executed twice and none was skipped. Placement itself (which worker got which
  goal) is not pinned.
