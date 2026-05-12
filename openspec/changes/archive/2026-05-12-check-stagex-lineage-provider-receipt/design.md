## Context

The StageX lineage row intentionally must not be satisfied by Guix/source-root metadata. The next useful increment is a small, deterministic receipt contract that names the StageX lineage evidence tuple and proves the parity code will fail closed on missing, mismatched, or fallback-marked lineage data.

## Goals / Non-Goals

**Goals:**
- Add a JSON receipt schema for the StageX lineage provider row.
- Validate `provider_kind = stagex-lineage`, `lineage_receipt_status = scaffold-only`, digest-shaped fields, and `fallback_events = []`.
- Let the row move from uninspected `blocked` to evidence-backed `partial` while still blocking StageX parity.

**Non-Goals:**
- Claim StageX parity.
- Produce a real audited StageX lineage provider.
- Run full self-build proof.

## Decisions

### 1. Scaffold receipts are explicit

**Choice:** The receipt must say `lineage_receipt_status = scaffold-only`.

**Rationale:** This makes the evidence machine-readable without pretending the real audited lineage exists.

**Alternative:** Leave the row blocked until real StageX lineage exists. Rejected because it leaves no regression coverage for the future proof ingestion seam.

### 2. Digest-shaped placeholders are checked

**Choice:** Require 64-character lowercase hex digests for the named lineage fields even in the scaffold.

**Rationale:** Future automation can replace scaffold digests with real BLAKE3 evidence without changing the parser or tests.

## Risks / Trade-offs

**Overclaim risk** → The parity row remains `expected_complete=false`, status `partial`, provider kind `unknown`, and continues blocking StageX parity.
