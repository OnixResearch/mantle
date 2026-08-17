## Context

The proof runner can take a long time and may fail because of external host capacity, source-root gaps, or implementation blockers. The goal of this change is current, honest evidence, not necessarily a green proof at any cost.

## Decisions

### 1. Current evidence beats historical claims

**Choice:** Re-run or inspect same-session evidence before updating any proof status.

**Rationale:** Fixed-point and self-build claims are only useful if they describe the current tree.

### 2. Blockers are first-class outcomes

**Choice:** If the proof does not complete, the evidence must record the blocker, command, output path, and next action.

**Rationale:** A blocked proof is actionable only when the blocker is durable and narrow.

### 3. Heavy artifacts stay out of source control

**Choice:** Commit transcripts, metadata, digest summaries, and docs changes, but not large proof stores or generated binaries.

**Rationale:** The repo should remain reviewable while preserving enough evidence to reproduce or audit the result.

## Risks / Trade-offs

- Host capacity or network instability may prevent a full proof rerun during the change.
- The proof may reveal implementation work that should become a separate blocker-resolution change.
- Evidence summaries must avoid broader claims than the command output supports.
