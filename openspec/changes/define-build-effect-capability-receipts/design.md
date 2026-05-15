## Context

Mantle already treats deterministic release claims as bounded proof classes and has sandbox-envelope evidence. The missing abstraction is a stable, operator-visible capability/effect vocabulary that mirrors Unison's abilities without importing language-level algebraic effects.

## Goals / Non-Goals

**Goals:**
- Define a closed first-phase effect enum.
- Bind declared and observed effects into proof, release, and provenance records.
- Fail closed when deterministic claims rely on undeclared or unaudited effects.

**Non-Goals:**
- Replace the existing sandbox/proof class ladder.
- Prove kernel-level noninterference.
- Add remote build orchestration.

## Decisions

### 1. Effect vocabulary is closed and versioned

**Choice:** Introduce `mantle-build-effects-v1` with values such as `read-store`, `write-output`, `network`, `clock`, `random`, `environment`, `secret`, `host-tool`, and `remote-build`.

**Rationale:** Closed enums make receipts reviewable and fail-closed. Future values require schema/version updates.

**Alternative:** Free-form strings. Rejected because typos and unreviewed effect names would weaken policy.

### 2. Receipts carry declared and observed sets

**Choice:** Proof and build receipts record `declared_effects`, `observed_effects`, and `effect_policy_version`.

**Rationale:** This preserves the difference between what a recipe intended and what the sandbox/audit layer observed.

### 3. Deterministic release verification rejects excess effects

**Choice:** `release verify --require-deterministic-release` fails if observed effects are missing, unsupported, or not allowed by the selected policy.

**Rationale:** Determinism claims need receipts, not narrative.

## Risks / Trade-offs

**Incomplete observation on some hosts** → deterministic proof mode must mark the effect audit incomplete and fail closed rather than silently accepting it.

**Overly broad first enum** → start with a small closed set and require explicit version bumps for new effect families.
