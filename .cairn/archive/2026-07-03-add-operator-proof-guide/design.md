## Context

Mantle has several proof-related workflows with different scopes: ordinary self-build, Cargo-free fixed-point proof, source-root guard evidence, and demo-bundle validation. The guide should help an operator choose and interpret these workflows without implying broader release or compiler correctness claims.

## Decisions

### 1. Guide is evidence-oriented

**Choice:** Organize the guide around commands, outputs, evidence bundle fields, and claim boundaries.

**Rationale:** The user needs to reproduce and audit proof evidence, not just read conceptual prose.

### 2. Blocked outcomes are documented

**Choice:** Show how to report a blocked proof, where to find receipts, and how to map blockers to follow-on Cairn changes.

**Rationale:** A blocked proof is the common frontier and should be handled without overclaiming.

### 3. Drift checks cover commands and fields

**Choice:** Add a lightweight guard that verifies referenced commands, paths, or proof fields remain current enough to prevent stale documentation.

**Rationale:** Proof documentation ages quickly as CLI surfaces and bundle schemas change.

## Risks / Trade-offs

- Full command execution can be too slow for docs tests, so fast validation may need to check snippets structurally.
- The guide must distinguish Nix-free demo evidence from broader release reproducibility.
- Host-specific paths should appear only as examples or evidence, not as universal requirements.
