## Context

Recent deterministic proof work introduced sandbox execution, per-run sandbox profile identities, and negative CLI coverage for undeclared host paths, host networking, and main-output/proof-store reuse. The main spec already says maintained isolation-regression evidence is required, but the verifier core only checks receipt-local proof data.

## Goals / Non-Goals

**Goals:**
- Make isolation-regression evidence a typed, canonical release-core input.
- Fail closed when deterministic promotion lacks passing evidence for the supported proof sandbox profile family.
- Keep evidence scoped to the sandbox profile family, not to every digest-derived per-run profile instance.

**Non-Goals:**
- No new real `bwrap` integration test requirement.
- No global determinism or full-source bootstrap claim.
- No replacement of existing deterministic proof receipt schema.

## Decisions

### 1. Evidence receipt lives in release-core

**Choice:** Add a serializable `DeterministicSandboxIsolationEvidence` type next to deterministic proof receipt validation.

**Rationale:** Release eligibility is already computed in release-core and needs a pure deterministic input suitable for tests, future CLI loading, and canonical evidence bundles.

**Alternative:** Encode evidence as an opaque string in `normalized_execution_envelope`. Rejected because it cannot fail closed on missing checks or profile-family mismatch.

### 2. Family-level binding

**Choice:** Evidence binds `profile_family = "mantle-proof-sandbox-v1"`, while proof receipts continue to record digest-derived `mantle-proof-sandbox-v1:<blake3>` profile identities.

**Rationale:** Isolation regressions validate the executor/profile family behavior. Per-run profile identities include run-specific paths and cannot have one static regression receipt each.

### 3. Closed check set

**Choice:** The first required checks are `denies-undeclared-host-access`, `denies-host-network-by-default`, and `denies-main-output-and-proof-store-reuse`.

**Rationale:** These match the existing spec and CLI regression surface and cover the highest-risk escape paths.

## Risks / Trade-offs

- **API churn**: callers of deterministic eligibility must supply evidence. This is intentional fail-closed behavior and currently only release-core tests call the function.
- **Evidence freshness**: this change adds a version/status/check contract but not wall-clock expiry. Future work can bind evidence digests into release bundles or CI artifacts.
