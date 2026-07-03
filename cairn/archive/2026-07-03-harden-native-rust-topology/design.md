## Context

The native Rust planner has accumulated compatibility behavior from real workspace frontiers. Hardening should make those frontiers easier to understand and replay, not merely add more special cases.

## Decisions

### 1. Diagnostics name native identities

**Choice:** Diagnostics should name stable native unit identity, package identity, role, selected triple, target kind, and artifact role whenever that information is available.

**Rationale:** Cargo unit indices are not the long-term identity model, and ambiguous diagnostics slow proof work.

### 2. Replay receipts stay bounded

**Choice:** Store the minimum deterministic facts needed to reproduce or explain a failing unit, with BLAKE3 digests for larger inputs.

**Rationale:** Receipt bloat makes proof bundles hard to review and can perturb fixed-point inputs.

### 3. Fixtures cover negative space

**Choice:** Add sad-path fixtures alongside happy-path host/target examples.

**Rationale:** Role-sensitive artifact identity is only trustworthy if wrong-role, wrong-triple, and wrong-metadata cases fail before rustc.

## Risks / Trade-offs

- More detailed diagnostics can expose unstable internal names unless normalized.
- Receipt minimization must not remove facts needed by proof-bundle validators.
- Edge fixtures may need careful isolation to avoid ambient host tool leakage.
