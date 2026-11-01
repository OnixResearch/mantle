# Worker pool routing slice (2026-10-31)

Scope: the ordinary lazy Worker now routes a ready goal through an attached
`PoolDispatch` instead of the single build service. Identity is the resolved
action identity; realization-key/closure identity remains a later step.

## Wiring

- `Builder` gains an optional `Arc<PoolDispatch>`: `with_pool_dispatch`,
  `pool_dispatch()`, and a crate-internal `hermeticity_mode()` accessor.
  Default construction leaves it absent, so every existing path is unchanged.
- `Worker::spawn_prepared_build` builds `PreparedPoolGoal` from facts it already
  holds — the concrete request, `PreparedBuild.action_ref`, the derivation
  system, and the Builder's hermeticity mode — with no declared features,
  accelerator classes, or capacity (zeros mean "no quantified requirement", so
  capacity is never inferred), then dispatches through the pool when attached.
  Otherwise it uses the ordinary build service exactly as before.
- Watch is refused for a pool executor (`remote-pool:watch-unsupported`) rather
  than ignoring cancellation, because pool teardown is not proven.
- Goal construction failure fails the goal with `remote-pool:*`; the ordinary
  service is not called as a silent fallback.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p crunch-build --lib worker::tests::pool_dispatch` | 1 passed |
| `cargo test -p crunch-build --lib` | 784 passed; 2 failed — the same two tests fail identically on the parent commit in a parallel full run (`finish_gates::tests::real_glibc_audit_…` and `workspace_shell::tests::production_workspace_local_dispatch_stack_…`), and both pass when run alone; they are pre-existing order/environment failures, not pool regressions |
| strict Clippy `crunch-build`, `cargo fmt --check -p crunch-build` | exit 0 |
| `cairn validate --root . --strict` | 0 issues |

The routing fixture runs a real `Worker` with an ordinary service and two
recording endpoint services: the selected endpoint runs the goal once, the
unselected endpoint and the ordinary service run zero times.

## Not claimed

- The pipeline does not yet construct real per-endpoint services; the CLI cannot
  supply a pool to the Builder, so this is composition evidence, not a
  user-facing `build --builder-pool` dispatch.
- No reservation, transport, output verification, or realization-key identity is
  exercised here.
