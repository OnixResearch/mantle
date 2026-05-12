## Context

The parity report intentionally leaves broad Guix/StageX claims incomplete until evidence exists. The immediate gap is not a long bootstrap rebuild; it is the evidence contract that lets future runs prove which provider kind was selected.

## Goals / Non-Goals

**Goals:**
- Make seed-full provider kind explicit in generated provider metadata.
- Bind self-hosting proof and packaged release evidence to the selected provider kind.
- Reject missing/unknown provider kinds in proof/release validation.

**Non-Goals:**
- Claim StageX parity from the existing source-root provider.
- Run a full self-hosting proof in this change.
- Replace the legacy provider default.

## Decisions

### 1. Provider kind is a closed string in proof prerequisites

**Choice:** Store `provider_kind` under the self-hosting proof `prerequisites` object.

**Rationale:** Provider selection is a prerequisite fact for interpreting the proof; keeping it near mode/inventory/tool prerequisites makes release evidence extraction cheap and deterministic.

**Alternative:** Infer provider kind from derivation names or prose notes. Rejected because it is ambiguous and hard to validate.

### 2. Release proof linkage copies provider kind

**Choice:** Add `selected_provider_kind` to release `proof_linkage` and verify it against the copied proof bundle.

**Rationale:** Release bundles should remain self-contained and fail closed if the proof bundle changes under them.

### 3. StageX remains distinct

**Choice:** `seed-full` is labeled `source-root`; StageX remains reserved for `stagex-lineage` evidence.

**Rationale:** This drains the Guix/source-root evidence seam without overstating audited-seed/StageX parity.

## Risks / Trade-offs

**Schema evolution** → Existing sample fixtures need updates. This is acceptable because full proof schema is already versioned and tests own the fixtures.
