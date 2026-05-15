## Context

`bootstrap parity-report` already tracks the `crunch.self-build` row as partial and consumes provider-kind linkage scaffolding. The release determinism workflow now separately generates a real proof bundle, deterministic proof receipt, sandbox evidence, release verify receipt, and portable summary artifacts. The next seam is to let parity-report consume that evidence as bounded self-build proof evidence, without converting legacy-provider or two-clean-store artifact equality into a full Guix or StageX bootstrap claim.

## Goals / Non-Goals

**Goals**
- Add a machine-checked evidence input for `crunch.self-build` real proof bundles.
- Require deterministic proof receipt, sandbox evidence, release verify receipt, and summary/linkage digest consistency.
- Surface selected provider kind and proof evidence digest in parity row details.
- Keep Guix and StageX axes fail-closed unless their stricter provider lineage requirements are satisfied.
- Cover missing/malformed/mismatched evidence with negative tests.

**Non-Goals**
- Do not make the current legacy fetched-provider proof satisfy Guix full-source or StageX lineage parity.
- Do not commit generated proof bundles from `target/` as source artifacts.
- Do not re-run the full self-hosting proof as part of ordinary parity unit tests.
- Do not redefine deterministic proof receipt semantics; consume the existing `mantle-deterministic-proof-receipt-v1` contract.

## Decisions

### 1. Consume an index or configured bundle path, not loose target artifacts

**Choice:** Add a repo/source evidence contract that points to or embeds the checked real self-build proof evidence linkage needed by parity-report.

**Rationale:** Generated `target/` proof bundles are large and environment-specific. Parity-report needs stable machine-checkable evidence without committing raw build outputs. A compact evidence descriptor can record paths/digests or fixture evidence for tests while keeping real generated artifacts outside git.

**Alternative:** Read `target/release-evidence/latest` implicitly. Rejected because parity would depend on mutable local build output and could silently change between checkouts.

### 2. Reuse release proof validation semantics

**Choice:** The parity evidence checker should mirror the real-proof validator invariants: workflow/version, `self-rebuild-match`, two distinct clean roots, matching artifact digest sets, provider linkage, sandbox profile prefix, release verify `eligible`, and digest linkage.

**Rationale:** This avoids creating a weaker second interpretation of release determinism proof evidence.

**Alternative:** Trust the summary Markdown/JSON alone. Rejected because summaries are review artifacts; parity must validate the underlying proof and verify receipts or a canonical descriptor derived from them.

### 3. Keep axis completion separate from self-build proof presence

**Choice:** Real proof evidence can improve the `crunch.self-build` row and remove “missing proof bundle” diagnostics, but it must not mark Guix or StageX complete unless provider kind and upstream lineage/source-root rows also satisfy their requirements.

**Rationale:** The current real proof was legacy-provider bounded artifact equality. It is valuable evidence, but it is not full-source bootstrap or audited StageX lineage proof.

## Risks / Trade-offs

**Evidence descriptor drift** → Mitigate with positive/negative parity tests that recompute or compare BLAKE3 linkage fields.

**Overclaiming parity** → Mitigate by making `complete` status conditional on axis-specific provider lineage requirements, not just self-build proof match.

**Fixture brittleness** → Use small synthetic JSON fixtures for unit/CLI tests and keep the full real rail as operator evidence, not a unit-test prerequisite.

## Validation Plan

- `openspec validate consume-real-self-build-proof-evidence --strict`
- Parity unit tests for valid bounded proof evidence and each negative case.
- CLI regression for `crunch --json bootstrap parity-report` showing `crunch.self-build` row evidence fields without satisfying Guix/StageX when provider kind is legacy.
- Existing release proof validator/summary self-tests remain the source of truth for full generated artifact validation.
