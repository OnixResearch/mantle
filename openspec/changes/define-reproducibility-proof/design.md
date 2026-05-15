## Context

Mantle already has release-evidence and release-verification specs that separate bundle-local integrity, self-proof validity, witness agreement, and social policy sufficiency. Bootstrap specs also bind self-build proof metadata to BLAKE3 proof bundle and canonical reproducibility report digests. This change fills the missing middle: a precise report contract and claim ladder for proving that a release artifact reproduced.

## Goals / Non-Goals

**Goals:**
- Define the evidence tuple required for a bounded reproducibility claim.
- Make claim classes monotonic and hard to overstate.
- Reuse existing release evidence and witness rebuild workflows where possible.
- Require BLAKE3 for Mantle-owned output digest comparisons.
- Fail closed on mismatches, dirty inputs, unknown recipes, and incomplete evidence.

**Non-Goals:**
- Prove full-source bootstrap by this change alone.
- Require cross-architecture or cross-platform deterministic outputs in the first implementation.
- Replace social verification policy; reproducibility evidence feeds it but does not define trust quorum.
- Change the existing release evidence bundle format unless implementation discovers a missing required field.

## Decisions

### 1. Claim ladder instead of one boolean

**Choice:** Use closed proof classes that extend the existing release-verification ladder: `bundle-consistent`, `self-proof-valid`, `self-rebuild-match`, `external-witness-match`, and `policy-satisfied`.

**Rationale:** Reproducibility is not a single yes/no in this repo. Bundle verification, local clean rebuild, independent witness agreement, and policy quorum are different strengths.

**Alternative:** Emit `reproducible=true`. Rejected because it invites over-claiming and hides which evidence actually exists.

### 2. Canonical report as the durable proof artifact

**Choice:** Define a canonical report that binds source digest, recipe identity, environment assumptions, clean rebuild stores, output digest sets, evidence digests, and verdict.

**Rationale:** A durable report lets release verification, bootstrap proof metadata, docs, and future witness tooling point at the same evidence object.

**Alternative:** Rely on terminal transcripts only. Rejected because transcripts are useful supporting evidence but are too hard to validate deterministically.

### 3. Replayable recipe identity

**Choice:** Require every proof to name a versioned recipe identity instead of letting operators choose arbitrary commands.

**Rationale:** A second environment must be able to replay the same workflow from the release request. This aligns with existing witness rebuild work and blocks accidental ad hoc proofs.

**Alternative:** Document a prose command sequence only. Rejected because it is brittle and cannot be reliably checked by the verifier.

### 4. BLAKE3 comparison boundary

**Choice:** Use BLAKE3 for Mantle-owned proof comparisons, while allowing explicitly justified interoperability hashes inside supporting metadata.

**Rationale:** This matches existing project proof conventions and avoids weakening final proofs through accidental SHA-centric wording.

**Alternative:** Accept any digest algorithm without policy. Rejected because it makes proofs ambiguous and conflicts with existing proof-hash expectations.

## Risks / Trade-offs

**Report schema churn** → Start with a narrow required field set and add versioned optional fields rather than broad untyped blobs.

**Over-claiming in docs** → Add explicit negative scenarios and docs checks that keep artifact reproducibility separate from full-source bootstrap and social policy sufficiency.

**Long-running rebuilds** → Keep implementation tasks friendly to managed background execution and allow report creation to consume existing witness evidence where available.

**Environment drift** → Record environment assumptions in the report and require clean rebuild store identity so comparisons are auditable.
