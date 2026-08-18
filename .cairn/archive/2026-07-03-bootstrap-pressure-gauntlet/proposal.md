## Why

Mantle has self-hosting fixed-point and source-built handoff evidence, but not a full-source bootstrap root. To compare honestly with Nix and push toward a stronger claim, Mantle needs a bootstrap pressure gauntlet that repeatedly reduces seed trust, runs no-host-tools/protected-exec profiles, records the remaining trusted root, and fails closed on undeclared host execution.

## What Changes

- Add a bootstrap gauntlet plan/report that runs default, non-Nix-host, no-host-tools, source-built-provider, and reduced-seed profiles with explicit trust boundaries.
- Record seed inventory digests, protected-exec audit digests, stage output digests, fixed-point status, host-tool absence, and remaining bootstrap blockers.
- Require negative checks for undeclared compiler/build/archive/Nix command execution and missing seed inventory entries.
- Keep full-source-bootstrap and compiler-correctness claims blocked until the seed root is actually derived from accepted source evidence.

## Impact

- **Files**: bootstrap proof drivers/reports, protected-exec tests, docs, Cairn verification-evidence/bootstrap-inventory spec deltas.
- **Testing**: positive declared-inventory fixed point, negative undeclared host exec, negative missing inventory, source-root blocker report, Cairn validation/gates.
