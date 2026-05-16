## Phase 1: Graph model

- [x] [serial] Define semantic graph node and edge schema over existing provenance/proof/store identities.
  - Completed: added `src/semantic_graph.rs` with immutable node kinds for source trees, recipes, store outputs, sandbox profiles, providers, proof receipts, witness requests, release evidence, and aliases, plus typed edge kinds and validation.
- [x] [parallel] Add alias/name records that point to immutable identities without changing identity digests.
  - Completed: aliases resolve to node identities separately from node digest fields; tests prove alias rename does not alter content identity.
- [x] [parallel] Define incomplete-graph diagnostics for legacy or missing records.
  - Completed: graph load/query paths return typed `mantle-incomplete-semantic-graph-v1` diagnostics instead of inventing missing nodes or edges.

## Phase 2: Query surfaces

- [x] [depends:Phase 1] Implement graph persistence for new build/proof events.
  - Completed: added validated JSON load/save support for `mantle-semantic-build-graph-v1` records under the state graph file boundary. The graph store tolerates missing legacy graph files with typed incomplete diagnostics.
- [x] [depends:Phase 1] Add `mantle graph` JSON and human output for one output/proof root.
  - Completed: added `mantle graph <root> [--graph-file ...]` with JSON parity through global `--json`.
- [x] [parallel] Add `mantle why` and `mantle dependents` query tests.
  - Completed: added semantic graph tests for why/explain, dependents, alias identity stability, and incomplete graph fail-closed behavior.
- [x] [parallel] Add proof-explain output that links releases to recipes, sandboxes, outputs, and witness receipts.
  - Completed: `mantle why` returns producing recipe, source, provider, sandbox, proof receipt, witness request, and release evidence nodes when those graph edges exist.
- [x] [depends:Phase 2] Document graph query examples and incomplete-graph limitations.
  - Completed: documented query commands and incomplete-graph behavior in `docs/operator-workflows.md`.
