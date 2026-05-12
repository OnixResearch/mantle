## Context

Provider-kind linkage is already part of the release evidence and full self-hosting proof identity code paths, but `crunch bootstrap parity-report` only has prose for `crunch.self-build`. A missing or mismatched proof-linkage artifact should be a first-class evidence check so future proof production can remove ambiguity without changing parity semantics.

## Goals / Non-Goals

**Goals:**
- Add a small JSON receipt contract for self-build provider-kind linkage.
- Validate closed provider-kind values and equality across proof identity, release proof linkage, and proof prerequisites.
- Preserve `partial` status for `crunch.self-build` until full self-build proof exists.

**Non-Goals:**
- Run full `crunch self-build`.
- Claim Guix or StageX parity.
- Change release evidence bundle layout.

## Decisions

### 1. Parity consumes a linkage receipt

**Choice:** Add `bootstrap/evidence/crunch-self-build-provider-kind-linkage.json` as a parity-only receipt with the fields needed to prove provider-kind equality.

**Rationale:** The parity report can validate a compact, auditable artifact without copying or parsing a full release bundle during every report.

**Alternative:** Inspect release evidence bundles directly. Rejected for this increment because there is no stable repo-local bundle path, and large bundles would make a cheap parity report fragile.

### 2. Closed provider-kind allowlist is reused locally

**Choice:** Accept only `legacy-fetch`, `source-root`, or `stagex-lineage`, matching release-core validation.

**Rationale:** Parity should fail closed on unknown provider strings instead of echoing unchecked metadata.

## Risks / Trade-offs

**Receipt drift** → Tests cover missing, mismatched, and valid receipts. Future full-proof automation should generate the receipt from verified release evidence rather than hand-writing it.
