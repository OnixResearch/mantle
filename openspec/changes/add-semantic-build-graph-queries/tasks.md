## Phase 1: Graph model

- [ ] [serial] Define semantic graph node and edge schema over existing provenance/proof/store identities.
- [ ] [parallel] Add alias/name records that point to immutable identities without changing identity digests.
- [ ] [parallel] Define incomplete-graph diagnostics for legacy or missing records.

## Phase 2: Query surfaces

- [ ] [depends:Phase 1] Implement graph persistence for new build/proof events.
- [ ] [depends:Phase 1] Add `mantle graph` JSON and human output for one output/proof root.
- [ ] [parallel] Add `mantle why` and `mantle dependents` query tests.
- [ ] [parallel] Add proof-explain output that links releases to recipes, sandboxes, outputs, and witness receipts.
- [ ] [depends:Phase 2] Document graph query examples and incomplete-graph limitations.
