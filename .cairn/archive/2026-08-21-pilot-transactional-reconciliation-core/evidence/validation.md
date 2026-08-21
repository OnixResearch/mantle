# Reconciliation pilot validation

## Focused results

- `cargo test -p mantle --test transactional_reconciliation_pilot`: 1 passed, 0 failed (`exact_gc_plan_is_current_and_changed_plan_is_stale`).
- The pilot binds an exact GC plan identity through the shared planner at revision `eb2bd3441753af97bfcb247cef7cc22d72675b62` and classifies a changed plan as stale.
- Cargo and the Nix flake select the same Radicle source and revision; no ambient sibling path is used.

## Gates

- `cargo clippy -p mantle --lib --no-deps -- -D warnings`: passed.
- Scoped tigerstyle on `-p mantle`: 33 findings, all in `src/operator_contract.rs` and `src/protected_exec_seccomp.rs`, which this pilot does not touch. A parent-revision control at base `43f07de5` reports 34 findings in the same files: pre-existing, zero introduced.
- Whole-workspace quality and tigerstyle wrappers: pre-existing failures in `mantlepkgs-core` and vendored crates unrelated to this pilot.
- `nix flake check -L`: fails on exactly one check, `bootstrap-blocker-inventory`, identically at base `43f07de5` and on this branch (115 findings, enforce=true). Parent-revision control confirms the failure is pre-existing; the pilot introduces no new Nix-check failures.

## Pre-existing environment blocker

Mantle pins cairn at revision `fb1a7403a7897f7fa161e0b3c5d86b4cf19e52f8`, whose parser rejects the committed `cairn-policy/generated/cairn-policy.json` shape (`task_marker_policy.markers`). Lifecycle commands for this change ran with the newer cairn revision `695124d459574ba7aeba6097310d237f393c243c` against `--root`, which validates this repository cleanly. Bumping Mantle's pin is left as an explicit follow-up because it touches shared dependency inputs.

## Boundaries

The pilot proves composition and identity binding only. It does not prove Redb durability behavior, GC correctness, storage-uncertainty adoption, or release eligibility.
