## Context

The deterministic proof core already validates canonical build proof receipts and typed sandbox isolation evidence. `release reproduce` writes those artifacts, but `release verify` has no way to load them or require a deterministic-release claim.

## Goals / Non-Goals

**Goals:**
- Load deterministic proof and isolation evidence artifacts from explicit paths.
- Validate canonical compact JSON bytes and BLAKE3 digests before promotion.
- Report deterministic eligibility without overclaiming, and fail when explicitly required.

**Non-Goals:**
- Change artifact schemas.
- Bundle deterministic proof artifacts into `manifest.json` in this slice.
- Claim global Mantle determinism beyond release artifact determinism.

## Decisions

### 1. Explicit verifier artifact paths

**Choice:** Add `--deterministic-proof` and `--deterministic-sandbox-isolation-evidence` to `release verify`.
**Rationale:** The current reproduce output may live outside the release bundle; explicit paths avoid guessing and let tests bind exactly the generated artifacts.
**Alternative:** Auto-discover beside the reproducibility report. Rejected for now because default reproduce paths are not bundle-local.

### 2. Separate reporting from requirement

**Choice:** If deterministic artifact paths are present, verify reports deterministic status. `--require-deterministic-release` turns ineligibility/missing artifacts into a non-zero exit.
**Rationale:** Existing `release verify` remains backward-compatible, while release promotion can fail closed when requested.

### 3. Use core canonical validators

**Choice:** Deserialize artifacts, require canonical bytes match, then call `deterministic_release_claim_eligible` with the bundle binary digest set.
**Rationale:** Keeps deterministic semantics centralized in release-core and prevents pretty/stale/malformed JSON from being promoted.

## Risks / Trade-offs

**Digest self-reference remains schema-level** → This slice validates canonical bytes and BLAKE3 shape through existing helpers; a future manifest-binding slice can make artifact digest linkage stronger.

**Explicit paths are more operator work** → Acceptable because CLI output from reproduce already reports exact paths and digests.
