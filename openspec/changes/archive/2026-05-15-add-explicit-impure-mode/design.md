## Context

Mantle currently has `practical` and `strict` hermeticity concepts. The user
wants a Nix-like `--impure` escape hatch, but Mantle's release and determinism
proof work depends on not confusing convenience builds with proof-eligible
builds.

## Goals / Non-Goals

**Goals:**
- Add an explicit `impure` mode to the model and CLI.
- Keep the default path reproducibility-oriented.
- Make impure successes machine-readable and proof-blocking.
- Preserve security isolation where compatible; label any degraded controls.

**Non-Goals:**
- Make impure derivations eligible for existing release/determinism proofs.
- Depend on Nix experimental features or exact Nix implementation semantics.
- Turn `--impure` into a silent sandbox-off switch.

## Decisions

### 1. `--impure` is a mode, not an absence of checks

**Choice:** Add `HermeticityMode::Impure` and pass it through the pipeline.

**Rationale:** Downstream reporting, attestations, and verifiers need a typed
fact, not inferred behavior from missing sandbox settings.

### 2. Impure is mutually exclusive with strict

**Choice:** CLI rejects `--strict-hermetic --impure` before execution.

**Rationale:** The modes make opposite claims. Failing early avoids ambiguous
reports and prevents accidental proof promotion.

### 3. Security and reproducibility are separated

**Choice:** Impure mode may allow host-sensitive inputs while still keeping
security controls where compatible.

**Rationale:** Development compatibility should not imply privilege escape. If a
control must be disabled, that fact must be visible and machine-readable.

### 4. Existing proof classes reject impure material

**Choice:** Impure outputs cannot satisfy deterministic or release reproducibility
classes by default.

**Rationale:** This preserves the integrity of the proof ladder while leaving
room for a future explicitly named impure evidence class if needed.

## Risks / Trade-offs

**Operator confusion** → CLI and docs must say impure builds are useful for
development/compatibility, not proof.

**Implementation drift toward sandbox-off** → Require typed audit events for
allowed host inputs and degraded security controls.

**Release pipeline loopholes** → Release verification must consume hermeticity
mode and fail closed for impure material.
