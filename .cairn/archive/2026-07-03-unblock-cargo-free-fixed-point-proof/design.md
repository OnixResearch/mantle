## Context

The current fixed-point path has enough guardrails to reject ambient Cargo/Nix/rustup/wrapper leakage, but the proof still cannot complete because the topology executor reports a blocked stage. A broad blocked status is too coarse for implementation work and too vague for operator evidence.

## Decisions

### 1. Receipt-first triage

**Choice:** Start from the emitted fixed-point receipt and derive the blocker from receipt fields before changing planner behavior.

**Rationale:** The receipt is the operator-facing artifact. If a human has to inspect logs manually, the proof path is not yet auditable.

### 2. Implementation-owned blockers are fixed, not waived

**Choice:** If the blocker is caused by Mantle planner/executor behavior, the change must fix that behavior or narrow the failing fixture until the next root cause is exposed.

**Rationale:** The fixed-point proof should make forward progress. Waivers are only appropriate for external source-root/toolchain gaps that Mantle cannot honestly satisfy in this slice.

### 3. Success remains evidence-gated

**Choice:** The change may report either a successful fixed-point proof or a narrower next-blocker artifact, but it must not synthesize success from partial execution.

**Rationale:** This preserves the proof-before-claim policy while still making blocked runs useful.

## Risks / Trade-offs

- Resolving one blocker may expose another expensive blocker in the next stage.
- The proof run can be slow; focused tests should exercise the deterministic classifier before the expensive rerun.
- The blocker may depend on source-root toolchain capabilities that need a separate change.
