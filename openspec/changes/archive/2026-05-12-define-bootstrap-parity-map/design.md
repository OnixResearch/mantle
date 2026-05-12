## Context

Crunch has accumulated many per-stage live-bootstrap OpenSpec archives and a StageX-class no-quorum profile, but the current drain loop can finish a bounded stage slice without a single canonical statement of whole-chain parity. The new spec is intentionally claim-gating and evidence-focused; it does not implement the map yet.

## Goals / Non-Goals

**Goals:** Define the parity axes, required map fields, gap-report classifications, and claim gates for live-bootstrap, Guix, and StageX parity.

**Non-Goals:** Implement a validator, add derivations, or claim that existing bootstrap work has reached parity.

## Decisions

### 1. Use one canonical parity map with axis-specific evidence

**Choice:** Add requirements for a canonical map plus deterministic gap report.
**Rationale:** A map prevents archived per-stage changes from becoming a misleading implicit checklist and gives future drains a bounded target.
**Alternative:** Continue creating one OpenSpec per missing stage. Rejected because it does not answer whether the whole bootstrap matches the reference ecosystems.

### 2. Keep Guix and StageX as claim semantics, not copied implementations

**Choice:** Treat Guix as full-source trust-root disclosure semantics and StageX as no-quorum audited-seed lineage semantics.
**Rationale:** Crunch should reach parity in claims/evidence, while still allowing Crunch-specific bridge stages or replacements when explicitly justified.
**Alternative:** Require byte-for-byte reproduction of each external project's implementation. Rejected as too narrow and not necessary for Crunch's replacement-for-Nix goal.

## Risks / Trade-offs

**Spec breadth** → The map is intentionally broad. Mitigation: future implementation should generate deterministic rows and fail closed rather than relying on prose.

**False parity claims** → The requirements explicitly separate graph completion, semantic correctness, provider kind, and axis-specific claims.
