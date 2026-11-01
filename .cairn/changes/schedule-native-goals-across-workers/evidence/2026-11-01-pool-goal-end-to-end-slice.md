# Pool goal end-to-end execution slice (2026-11-01)

Scope: execute one ready goal through `build --builder-pool` with a real
`mantle remote serve` worker, a Nickel-admitted pool endpoint, a bound ticket
and the existing remote application ports, then admit the returned output into
the client store.

## What changed

- `crates/crunch-build/src/orchestrate.rs`: the pool goal closure computes each
  entry's derivation-modulo hash from the collected closure with the client
  store prefix instead of trusting registry HDMs computed under a different
  store directory. The receiver recomputes the same identities.
- `src/remote_build/pool_goal_executor.rs`:
  - applies the production attempt transitions in order: `start`,
    `result-ready`, then `completion`, with strictly increasing event numbers
    even when the exchange finishes inside one second;
  - imports admitted outputs into the client store through the owned
    transfer-object capability: streamed NAR files are read under the
    PathInfo-declared size, ingested with the declared content address, size and
    SHA-256 rechecked, then exported to the physical output path;
  - returns the imported castore nodes as the goal's `BuildResult` outputs so
    the ordinary goal/store path persists PathInfo.
- `src/remote_build.rs`: exposes `mark_remote_production_result_ready` and the
  streamed import-action planner to the pool executor; no CLI behavior changes.
- `tests/remote_transfer_production.rs`: adds
  `production_pool_dispatches_one_goal_through_the_worker`, which admits a
  stdio pool endpoint with a measured transport program identity, a trusted
  output key and a bound ticket, then asserts one succeeded outcome with a
  materialized output.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 13 passed; 0 failed; 0 ignored` (32.93s) |
| `cargo test -p crunch-build --lib -- distributed pool` | `test result: ok. 123 passed; 0 failed; 0 ignored` |
| `cargo test -p crunch-remote-core --lib` | `test result: ok. 119 passed; 0 failed; 0 ignored` (66.26s) |
| `cargo clippy -p crunch-build -p crunch-remote-core -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle -p crunch-build -p crunch-remote-core` | exit 0 |

## Non-claims

- One goal, one endpoint, one worker generation, same-host stdio loopback.
- Does not exercise multi-worker diamonds, returned dynamic plans, concurrent
  coordinators/completions, cancellation, stale fences, worker death, or
  resource saturation; those remain the phase 3 acceptance tasks.
- The test transport program is a local wrapper script whose BLAKE3 identity is
  measured; this is not evidence for a remote host or a real transport binary.
- A successful pool dispatch is not a release, self-hosting, or distributed
  determinism claim.
