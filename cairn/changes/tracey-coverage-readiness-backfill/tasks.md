# Tasks

## Baseline and inventory

- [ ] [serial] Record a fresh Tracey coverage baseline with total requirements, referenced count, missing count, dangling count, and missing IDs grouped by accepted spec. r[verification_evidence.tracey_coverage_readiness]
- [ ] [serial] Classify each missing requirement group as direct-marker, bridge-marker, scanner-gap, or real implementation debt, with owner and next action. r[verification_evidence.tracey_coverage_readiness]

## Implementation

- [ ] [serial] Add scanner support or evidence-backed bridge refs so root-package Mantle implementation and tests can satisfy coverage without misleading comments. r[verification_evidence.tracey_coverage_readiness]
- [ ] [serial] Backfill the first coverage batch, including build-tool-boundary and source-built toolchain closure refs, with durable evidence links. r[verification_evidence.tracey_coverage_readiness]
- [ ] [serial] Preserve proof-before-claim wording for any bridge refs: traceability evidence MUST NOT become a broader feature-support claim. r[verification_evidence.tracey_coverage_readiness]

## Verification

- [ ] [serial] Run `cairn tracey coverage --root . --json` and record whether global coverage is green or the exact remaining missing list. r[verification_evidence.tracey_coverage_readiness]
- [ ] [serial] Run `cairn validate --root .` and record output. r[verification_evidence.tracey_coverage_readiness]
- [ ] [serial] Archive only after completed tasks cite durable evidence and any remaining coverage debt is explicitly scoped. r[verification_evidence.tracey_coverage_readiness]
