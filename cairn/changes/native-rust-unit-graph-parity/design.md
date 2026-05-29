# Design: Native Rust unit graph construction

## Context

A Cargo-free build needs Mantle to decide which lib/bin/proc-macro/custom-build units exist and how they depend on each other.

## Decisions

### 1. Use selected package/feature facts as graph input

**Choice:** Build units from native package targets plus resolved feature/source facts rather than Cargo unit graph JSON.

**Rationale:** Unit graph production is the planner boundary that must become Cargo-free.

### 2. Preserve explicit host/target roles

**Choice:** Custom-build and proc-macro units are host units; normal lib/bin units are target units. Cross-role edges are typed and fail closed on unsupported shapes.

**Rationale:** Host/target confusion has repeatedly caused false frontiers.

### 3. Generate stable unit IDs from native selected facts

**Choice:** Define deterministic unit identity over package ID, target, role, profile, feature set, source digest, and dependency partition facts.

**Rationale:** Once Cargo is removed, IDs must remain stable and sufficiently discriminating without Cargo's unit index.

## Risks / Trade-offs

- Native IDs will differ from old Cargo-index IDs; migration must preserve receipt clarity.
- Full Cargo parity remains non-goal until unsupported shapes are explicitly implemented.
