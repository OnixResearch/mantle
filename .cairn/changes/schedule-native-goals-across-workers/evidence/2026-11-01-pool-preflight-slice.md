# Pool mode operator preflight slice (2026-11-01)

Scope: reject pool mode combined with execution modes the pool path does not
implement, before any store, state, credential, or transport mutation.

## What changed

- `src/main.rs`: the clap surface already declared `--builder-pool` conflicts
  with `--watch`, `--plan`, `--impure`, `--strict-hermetic`,
  `--evaluation-stream`, and related local-only flags. `run_build_command` now
  repeats the watch, impure, evaluation-stream, and non-Linux checks for callers
  that bypass the clap surface.
- `tests/smoke.rs`: `pool_mode_rejects_unsupported_modes_before_build` asserts
  the CLI rejects `--watch`, `--impure`, and `--evaluation-stream` with
  `--builder-pool` and names the conflicting flag, using a pool config path that
  does not exist (so reaching config load would prove the rejection happened
  too late).

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --test smoke pool_mode_rejects_unsupported -- --nocapture` | `test result: ok. 1 passed` |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 17 passed; 0 failed; 0 ignored` (33.35s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

## Non-claims

- The operator-facing rejection is clap's conflict surface; the runtime checks
  are defense in depth for non-CLI callers.
- `--plan` with a pool is still accepted and validated through the pool factory;
  rendering intended pool routes without credentials is not proven here.
- No unsupported store backend or output-mode matrix beyond the three rejected
  flags is claimed.
