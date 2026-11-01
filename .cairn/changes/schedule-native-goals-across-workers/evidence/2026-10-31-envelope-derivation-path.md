# Envelope derivation path (2026-10-31)

Scope: carry the concrete derivation's absolute logical store path in the pool
goal envelope, so an executor can build the remote request (which needs the
declared drv path) without re-deriving it.

## Change

- `PreparedPoolGoal`/`PreparedPoolGoalFacts` gain `drv_path`; construction rejects
  an empty, non-absolute, or oversized path (`InvalidDerivationPath`).
- `Worker::spawn_prepared_build` fills it from `prepared.drv_path` with the
  builder's store prefix.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p crunch-build --lib distributed::tests::prepared_goal` | 1 passed (adds the empty-path denial) |
| `cargo test -p crunch-build --lib worker::tests::pool_dispatch` | 1 passed |
| strict Clippy `crunch-build`, rustfmt | exit 0 |

## Remaining gap for the executor

The bin-side executor still needs to map admitted remote outputs to
`BuildResult.outputs` in the concrete `BuildRequest.outputs` order (the Builder
persists them positionally), and to apply the terminal attempt transition in the
client coordinator state after success or failure.
