# Pool output-matching guards slice (2026-11-01)

Scope: the executor's output-to-goal matching is a named, unit-tested guard:
missing, extra, and unmatched outputs fail closed.

## What changed

- `src/remote_build/pool_goal_executor.rs`: extracted `match_pool_outputs`.
  It now rejects any count difference before matching (`output-count-mismatch`),
  rejects an unmatched requested path (`output-missing:<suffix>`), matches
  input-addressed outputs by declared logical path, and matches CA outputs by
  result order.
- Unit tests: `pool_output_matching_rejects_missing_extra_and_wrong_outputs`
  and `pool_output_matching_uses_result_order_for_content_addressed_outputs`.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --bin mantle -- pool_output_matching` | `test result: ok. 2 passed` |
| `cargo test -p mantle --test remote_transfer_production` | `test result: ok. 27 passed; 0 failed; 0 ignored` (69.26s) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

## Non-claims

- End-to-end extra-output rejection still depends on the admission validation
  before this guard; the unit test covers the executor guard itself.
- Private-input handling remains uncovered.
- A disk-full condition interrupted one suite run; the target's incremental
  directory was removed and the suite reran green.
