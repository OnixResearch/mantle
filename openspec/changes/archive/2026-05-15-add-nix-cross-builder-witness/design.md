## Context

The existing deterministic release proof answers: “did Mantle rebuild the selected artifact twice from recorded inputs under recorded proof sandboxes and get matching BLAKE3 digest sets?” That is the right primary proof, but it does not show that the Nix package recipe and Mantle self-build recipe are byte-aligned.

## Goals / Non-Goals

**Goals:**
- Add a machine-readable cross-builder witness contract for Nix-vs-Mantle bit identity.
- Keep digest comparisons BLAKE3-first for Mantle-owned artifact identity.
- Make mismatch explainable and proof-blocking only for the cross-builder class.
- Keep the claim bounded to a selected artifact, source, toolchain, target, and build policy.

**Non-Goals:**
- Do not replace the two-clean-store Mantle determinism proof.
- Do not claim Nix is an independent social witness.
- Do not require every developer build to run the heavy Nix witness.
- Do not claim full-source bootstrap, cross-platform determinism, or all-package reproducibility.

## Decisions

### 1. Cross-builder witness is additive

**Choice:** Introduce a new evidence class, `nix-cross-builder-witness`, rather than folding Nix agreement into `self-rebuild-match`.

**Rationale:** A Nix-vs-Mantle match proves recipe convergence across two builders, while `self-rebuild-match` proves repeatability inside Mantle’s supported proof envelope. These are related but distinct claims.

**Alternative:** Require Nix bit identity before reporting `self-rebuild-match`. Rejected because it would make the primary Mantle proof depend on a separate external build system and could block useful deterministic evidence for reasons unrelated to Mantle repeatability.

### 2. Compare final selected artifacts, not whole build trees first

**Choice:** The first witness compares selected release artifact bytes and canonical BLAKE3 digest sets. It records but does not initially require whole-closure equivalence.

**Rationale:** Whole build trees can include legitimate path, debug, or metadata differences. The release artifact is the operator-facing object being claimed.

**Alternative:** Require Nix store closure equality. Rejected as too broad for the first contract and likely to create false blockers.

### 3. Bind normalization policy explicitly

**Choice:** The witness receipt must record source/vendor digests, Rust toolchain identity, target triple, `RUSTFLAGS`, linker/strip/debug policy, `SOURCE_DATE_EPOCH` policy, Nix derivation identity, and Mantle proof receipt identity.

**Rationale:** Bit-exact comparison is only meaningful if both sides document the knobs that affect embedded bytes.

## Risks / Trade-offs

**False mismatch from benign metadata** → The report must name the mismatched digest and recorded normalization policy so implementation can decide whether to normalize or intentionally accept a weaker class.

**Overclaiming Nix agreement** → The spec requires wording that this is a cross-builder witness, not full-source bootstrap reproducibility, social witness sufficiency, or global determinism.

**Heavy release gate** → The build-pipeline spec makes the witness opt-in/required for release artifacts, not ordinary dev builds.
