## Context

The current strongest operator path can prove a Cargo-free fixed point, but successful runs may require a local wrapper that strips or rewrites rustc/linker flags outside Mantle's receipt model. To beat Nix on auditability, Mantle must either generate and record that compatibility layer itself from declared inputs or report an actionable toolchain blocker.

## Decisions

### 1. Command-owned wrappers are receipt material

**Choice:** Any generated rustc, linker, ar, ranlib, pkg-config, or helper wrapper used by the fixed-point command is written under the stage bundle, content hashed with BLAKE3, and recorded with its normalization rules and source closure inputs.

**Rationale:** A wrapper can be legitimate only if it is reviewable, deterministic, and replayable from the proof bundle.

### 2. External wrappers are not trusted by default

**Choice:** If `RUSTC`, `PATH`, or explicit CLI arguments point at an untracked wrapper outside the declared closure, preflight records the path class and fails before a proof claim.

**Rationale:** Hidden host compatibility shims are exactly the dependency class the source-root proof is meant to retire.

### 3. Both stages share the same proof policy

**Choice:** Stage1 and stage2 use the same closure policy, guard policy, and command-owned normalization plan unless the summary records a deterministic policy mismatch blocker.

**Rationale:** A stage1==stage2 digest comparison is only meaningful if both stages were built under the same declared proof boundary.

## Risks / Trade-offs

- Some host rustc/linker combinations may stay unsupported until source-root toolchain closure gaps are closed.
- Generated wrappers must remain simple and auditable; complex shell glue should be rejected rather than hidden in the proof.
- Positive proof runs are long, so focused unit/CLI negatives should cover most regressions before rerunning the full fixed point.
