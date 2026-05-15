## Context

The current reproducibility report proves bounded release-artifact matching. A
single rebuild match is valuable, but it does not prove the build is stable under
ambient host perturbation, clean-store rebuilds, or stricter hermeticity
conditions. Nix-like determinism requires a stronger, separately named proof.

## Goals / Non-Goals

**Goals:**
- Define when Mantle may claim a derivation or release artifact is deterministic.
- Bind the claim to canonical evidence: strict mode, repeated clean stores,
  BLAKE3 output digests, perturbation matrix, and audit events.
- Keep deterministic-release claims scoped and non-global.

**Non-Goals:**
- Prove all Mantle builds are deterministic.
- Prove full-source bootstrap determinism.
- Define social trust or witness quorum policy; those remain release verification
  policy concerns.
- Implement the feature in this change package.

## Decisions

### 1. Determinism is receipt-backed, not inferred

**Choice:** A deterministic claim requires a canonical receipt with closed
verdicts.

**Rationale:** Successful builds and even one self-rebuild match are empirical
evidence, but they do not identify whether the result survived host-state
variation and clean-store isolation.

**Alternative:** Treat any `self-rebuild-match` as deterministic. Rejected
because it would overclaim and repeat the ambiguity this spec is meant to avoid.

### 2. Strict hermetic mode is mandatory for the proof

**Choice:** Deterministic proof attempts run in strict mode, and practical or
impure builds cannot be promoted silently.

**Rationale:** A deterministic proof should fail closed when host/env leakage is
observed. Practical mode can still be useful for development, but it is not the
right basis for a high-confidence claim.

### 3. Release verification consumes but does not broaden the proof

**Choice:** Release verification may consume per-artifact deterministic proof
receipts, but output must state the exact claim scope.

**Rationale:** Even strong per-artifact evidence does not prove global system
properties or bootstrap provenance.

## Risks / Trade-offs

**More expensive verification** → Repeated clean builds cost time and disk. The
implementation can make proof commands explicit and cache receipts by canonical
input identity.

**False confidence from weak matrices** → The first matrix is mandatory but not
complete. The receipt records exactly which perturbations ran so stronger future
matrices can be introduced without rewriting old claims.

**Store substitution ambiguity** → Dependency substitution is allowed only when
recorded and held constant. The derivation under test must not be satisfied from
an existing output.
