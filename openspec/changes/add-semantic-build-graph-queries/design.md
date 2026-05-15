## Context

Mantle already has native provenance node/edge concepts, but the operator workflow still tends to inspect separate artifacts manually. This change turns provenance into the front door for graph queries without requiring Mantle to store all Nickel source as an AST database.

## Goals / Non-Goals

**Goals:**
- Define a semantic graph schema over existing Mantle identities.
- Persist enough graph records to answer why/dependents/proof questions.
- Keep names and aliases distinct from content identities.

**Non-Goals:**
- Replace Git/Nickel source editing.
- Implement semantic merge/version control.
- Require complete graph history migration for old stores.

## Decisions

### 1. Graph identity uses immutable content/proof identities

**Choice:** Nodes are keyed by canonical BLAKE3-backed identities or existing stable store/proof identifiers; names are aliases.

**Rationale:** This mirrors Unison's names-as-metadata property while fitting Mantle's store/proof model.

### 2. Queries are CLI-first and JSON-backed

**Choice:** Add human and JSON output for core graph queries.

**Rationale:** Operators need readable diagnostics; automation needs stable machine-readable output.

### 3. Persist missing-edge diagnostics

**Choice:** If a query cannot explain an output because graph records are missing, the command emits a typed incomplete-graph diagnostic rather than guessing.

**Rationale:** Proofs and determinism claims must stay evidence-backed.

## Risks / Trade-offs

**State migration complexity** → start with new builds/proofs and allow legacy outputs to report incomplete graph coverage.

**Overlapping with provenance spec** → treat this as query/persistence refinement over native provenance, not a competing model.
