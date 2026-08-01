## Context

Mantle has two related tree paths. Release provenance uses a pure tree-copy planner and a capability-based shell. Frontend artifact storage performs another deterministic tree collection and BLAKE3 identity pass.

The future shared repository will own product-neutral tree mechanics. Mantle must retain release bundle policy, frontend identity compatibility, diagnostics, and evidence claims.

## Decisions

### Decision: Block work on an immutable prerequisite

**Choice:** Do not begin implementation until `bounded-tree` has archived its establishment change, passed its release gates, and published one reviewed Radicle revision.

**Rationale:** A sibling checkout or incomplete API would make Mantle the accidental integration test and would bypass the shared release boundary.

### Decision: Migrate release tree mechanics through a narrow adapter

**Choice:** Map Mantle's existing `TreeEntryObservation`, limits, blockers, and operations to the shared core. Use the shared capability shell only for no-follow observation, member reads, revalidation, and destination copy.

Mantle keeps release-root kinds, bundle layout, error presentation, evidence assembly, and release decisions.

**Rationale:** These meanings are Mantle policy, not generic tree mechanics.

### Decision: Retain frontend root identity

**Choice:** Use shared ordered member facts but keep Mantle's existing versioned frontend preimage, executable-bit policy, root record, and compatibility tests.

**Rationale:** A shared root digest would change artifact identity and transfer product compatibility into `bounded-tree`.

### Decision: Prove parity before deletion

**Choice:** Dual-run accepted positive and negative fixtures. Compare plans, member facts, root identities, blockers, and destination trees where each field is a compatibility surface.

Remove local mechanism code only after the parity corpus passes. Keep product adapters and explicit policy mappings.

**Rationale:** Happy-path copy equality cannot detect changed rejection behavior or identity drift.

### Decision: Preserve rollback

**Choice:** Keep the pre-adoption immutable Mantle revision and dependency state in the adoption receipt. Rollback restores both code and lock state together.

**Rationale:** A dependency rollback without adapter rollback can leave incompatible types or identity behavior.

## Risks / Trade-offs

- Mantle adapters remain necessary because product identity is intentionally not shared.
- Exact component-byte paths can expose old lossy path behavior in negative fixtures.
- Radicle-only acquisition makes dependency outages visible.
- Local code removal waits for full positive and negative parity.
