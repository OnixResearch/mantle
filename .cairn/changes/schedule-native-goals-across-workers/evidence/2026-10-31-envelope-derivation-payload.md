# Envelope derivation payload (2026-10-31)

Scope: carry the concrete derivation payload in the pool goal envelope so a
remote executor can send it. The Worker already holds the derivation; the
Nickel closure is not retained.

## Change

- `PreparedPoolGoal`/`PreparedPoolGoalFacts` gain `derivation_json`: the
  canonically serialized `nix_compat::derivation::Derivation`.
- `PreparedPoolGoal::new` rejects an empty payload and one above the remote
  payload bound (1 MiB, `MAX_POOL_GOAL_DERIVATION_BYTES`).
- `Worker::spawn_prepared_build` serializes `prepared.derivation` into the
  envelope; a serialization failure fails the goal with
  `remote-pool:derivation-serialize` instead of dispatching an incomplete goal.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p crunch-build --lib distributed::tests::prepared_goal` | 1 passed (adds empty-payload denial) |
| `cargo test -p crunch-build --lib worker::tests::pool_dispatch` | 1 passed (selector parses the envelope payload and requires its system to match) |
| strict Clippy `crunch-build`, `cargo fmt --check -p crunch-build` | exit 0 |
| `cairn validate --root . --strict` | 0 issues |

## Not claimed

- No executor consumes the payload yet; the bin-side remote executor and
  pipeline plumbing remain.
- Carrying a serialized derivation is not transfer, execution, or output
  admission.
