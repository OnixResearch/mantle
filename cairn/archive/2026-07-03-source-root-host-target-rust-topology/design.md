## Context

Cargo-free topology already derives selected units and can execute large native graphs. The remaining source-root frontier is platform-sensitive: host build scripts and their support libraries execute on the compiler host, while target crates may use the musl target closure. The native planner must make that distinction explicit without falling back to Cargo's unit IDs as the source of truth.

## Decisions

### 1. Role is part of native unit identity

**Choice:** Add a stable role dimension for `host`, `target`, and `host-dependency` units, and include the role in artifact keys, dependency facts, output directories, receipts, and blocker diagnostics.

**Rationale:** Package name and target name are not enough when the same crate must be built once for the host and once for the target. Role-aware identity prevents accidental artifact reuse.

### 2. Toolchain policy is selected by role before execution

**Choice:** Derive a pure unit-role plan from package facts, dependency edges, selected target triple, and toolchain-closure policy. The shell only materializes the already-selected host or target execution environment.

**Rationale:** The rule is deterministic and testable without executing rustc. The shell stays limited to creating PATH/env/output directories and invoking the planned unit.

### 3. Mismatches fail before rustc

**Choice:** Validate every consumed artifact's role, triple, source digest, metadata hash, and toolchain-policy digest before launching a unit.

**Rationale:** A mismatched `.rlib` often fails later with opaque rustc metadata errors. Mantle should report the declared planning violation directly.

## Risks / Trade-offs

- More role variants can increase graph size; receipts must keep bounded counts and clear blockers.
- Some Cargo-selected duplicate units may still need exact selected-unit preservation. The role split must not collapse legitimate same-role variants with different features or metadata.
- The first source-root fixed-point rerun may still block later in native linking; the evidence must record the new frontier without overclaiming.
