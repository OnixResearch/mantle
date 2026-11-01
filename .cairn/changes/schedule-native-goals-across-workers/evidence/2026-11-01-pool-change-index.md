# Pool change evidence index (2026-11-01)

Scope: map the implemented pool slices to their evidence and record the two
remaining lifecycle blockers. This index adds no claim of its own.

## Implemented slices

| Area | Evidence |
| --- | --- |
| one goal end to end (closure HDM, attempt ordering, output import) | `2026-11-01-pool-goal-end-to-end-slice.md` |
| two-worker diamond, conflict retries, busy re-placement, per-endpoint bindings | `2026-11-01-pool-diamond-slice.md` |
| content-addressed leaf and diamond (wire parser, resolved action identity) | `2026-11-01-pool-content-addressed-leaf-slice.md` |
| failure settlement (error, transport death, dropped future) | `2026-11-01-pool-failure-settlement-slice.md`, `2026-11-01-pool-transport-death-slice.md`, `2026-11-01-pool-cancellation-settlement-slice.md` |
| untrusted key, revoked binding, missing source input | `2026-11-01-pool-untrusted-key-slice.md`, `2026-11-01-pool-revoked-binding-slice.md`, `2026-11-01-pool-missing-input-slice.md` |
| tampered streamed NAR | `2026-11-01-pool-tampered-output-slice.md` |
| mid-exchange worker death | `2026-11-01-pool-worker-death-slice.md` |
| capacity queueing, single-endpoint diamond | `2026-11-01-pool-capacity-queueing-slice.md` |
| overlapping worker lifetimes | `2026-11-01-pool-overlap-slice.md` |
| operator cancellation with owned teardown | `2026-11-01-pool-cancellation-slice.md` (supersedes `2026-11-01-pool-cancellation-boundary.md`) |
| plan-mode pool validation | `2026-11-01-pool-plan-validation-slice.md` |
| duplicate completion refusal | `2026-11-01-pool-duplicate-completion-slice.md` |
| operator documentation | `2026-11-01-pool-operator-docs-slice.md` (`docs/remote-pool.md`) |
| operator contract regeneration | `2026-11-01-pool-quality-gate-attempts.md` |
| ordinary quality gate | `2026-11-01-pool-quality-gate-attempts.md` (exit 0, 226 suites) |

## Open lifecycle blockers

1. **Cairn validation/archive** — `nix develop --offline -c cairn validate
   --root .` produces no output and is killed after 15 minutes; running the
   pinned `cairn` binary directly also hangs. This is an environment/tooling
   blocker, not a change-tree failure.
2. **Task flips** — `tasks.md` remains unchecked because the change's own rule
   requires Cairn validation and acceptance before archiving; the evidence
   above supports the implementation and gate obligations but is not a
   substitute for that lifecycle step.

## Non-claims

- This index does not certify any slice; each slice's own evidence carries its
  verification and non-claims.
