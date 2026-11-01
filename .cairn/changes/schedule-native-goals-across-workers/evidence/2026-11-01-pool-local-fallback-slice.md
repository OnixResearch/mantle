# Pool local fallback slice (2026-11-01)

Scope: the pool's `fallback_policy = 'always` admits local execution when
placement fails before dispatch; other policies fail closed.

## What changed

- `crates/crunch-build/src/distributed.rs`: `PoolDispatch` carries the admitted
  fallback flag. Placement failures (selector `NoFit`, exhausted capacity wait)
  become `WouldBlock` errors only when fallback is allowed.
- `crates/crunch-build/src/worker.rs`: a `WouldBlock` pool placement error with
  fallback allowed runs the goal through the ordinary local build service; other
  errors remain failures.
- `src/remote_build/pool_factory.rs`: the factory carries
  `fallback_policy == 'always` from the evaluated pool.
- `tests/remote_transfer_production.rs`:
  `production_pool_local_fallback_runs_when_no_endpoint_fits` builds a goal with
  only an `aarch64-linux` endpoint: the default policy fails, `'always` builds
  locally and reports one successful outcome.
- `docs/remote-pool.md`: fallback section and updated verified list.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 28 passed; 0 failed; 0 ignored` (75.08s) |
| `cargo test -p crunch-build --lib -- pool_dispatch -- --test-threads 1` | `test result: ok. 14 passed; 0 failed` |
| `cargo clippy -p crunch-build -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle -p crunch-build` | exit 0 |

## Non-claims

- Fallback applies only to pre-dispatch placement failure; execution, trust,
  signature, transfer, and admission failures never fall back.
- A cancellation during a fallback goal is observed after the local build
  completes; the local service does not advertise watch sandbox teardown.
- `trusted-only` is treated as no local fallback in this slice.
