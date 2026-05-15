## Context

The verifier core has a `DeterministicSandboxIsolationEvidence` contract and rejects deterministic release promotion without it. The release reproduce command already constructs the bwrap proof sandbox profile and emits deterministic build proof receipts, so it is the smallest reliable place to create the matching isolation evidence artifact.

## Goals / Non-Goals

**Goals:**
- Generate deterministic sandbox isolation evidence from the same sandbox profile data used for proof runs.
- Write canonical compact JSON with a BLAKE3 digest available to humans/automation.
- Surface the evidence path/digest in JSON and text CLI output.

**Non-Goals:**
- Replace bwrap or broaden the sandbox profile family.
- Implement remote attestation of the sandbox executor.
- Change release bundle manifest format in this slice.

## Decisions

### 1. Evidence is emitted beside the proof receipt

**Choice:** write `deterministic-sandbox-isolation-evidence.json` under the deterministic proof directory.

**Rationale:** the evidence belongs to the repeated proof run workspace, not the main rebuild report, and can be consumed with the proof receipt as a pair.

### 2. Evidence digest binds generated profile facts

**Choice:** compute `evidence_digest_blake3` from canonical material containing the schema/profile family, evidence version, required checks, concrete profile identities, and canonical profile facts used to construct bwrap invocations.

**Rationale:** this avoids a self-referential digest field while making profile drift visible.

### 3. Canonical JSON remains in release-core

**Choice:** add release-core helpers for canonical evidence bytes and digest.

**Rationale:** the CLI should not duplicate canonicalization rules for a verifier-owned artifact.

## Risks / Trade-offs

- **Evidence overclaims isolation:** mitigate by deriving evidence only from the constrained bwrap profile construction and required checks already exercised by negative CLI tests.
- **Downstream output schema churn:** additive JSON fields only; existing consumers keep working.
