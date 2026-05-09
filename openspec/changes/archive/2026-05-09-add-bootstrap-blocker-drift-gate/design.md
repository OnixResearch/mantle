## Context

The recent live-part drain archived source-hardening and bridge/normalization gates through `tar 1.12`; `drain-plan` is empty and strict OpenSpec validation passes. The canonical bootstrap spec still explicitly withholds full-source bootstrap status until named placeholders, bridge/fallback events, and prerequisite-only evidence are replaced with source-built transcripts.

## Decisions

### 1. Source-derived blocker inventory

**Choice:** Implement a repo-owned checker that scans the trusted source surfaces (`bootstrap/`, `openspec/specs/bootstrap/spec.md`, and selected checked-in evidence/metadata) for known full-source-blocking markers.

**Rationale:** A deterministic inventory prevents future agents and humans from treating an empty implementation queue as full-source readiness.

**Alternatives rejected:** A prose-only README update is too easy to drift. A full rebuild/proof is too expensive for an edit-time guard and does not replace an inventory gate.

### 2. Fail-closed promotion drift detection

**Choice:** The checker fails when a source or report claims full-source promotion/readiness while bridge, placeholder, fallback, or prerequisite-gated markers still exist.

**Rationale:** The highest-risk failure mode is overclaiming readiness, not merely having blockers.

**Alternatives rejected:** Allowing warnings only would not prevent status drift. Failing every time blockers exist would make the gate unusable before the chain is repaired.

### 3. Dual report output

**Choice:** Produce both a stable JSON report for automation and a concise Markdown/operator summary for review.

**Rationale:** JSON supports CI/readiness rails; Markdown preserves an inspectable receipt for OpenSpec evidence.

## Risks / Trade-offs

- **False positives:** Use a small explicit marker taxonomy and checked fixtures rather than broad grep-only rules.
- **Stale taxonomy:** Keep marker classes versioned and document how to add or retire them when a blocker is actually repaired.
- **Overbroad scope:** This change does not repair the bootstrap chain; it only prevents readiness drift and creates a durable inventory.
