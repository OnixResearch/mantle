# Design: First-class Cargo-free fixed-point command

## Context

`self-build --cargo-free` now produces one Mantle binary through native topology execution. `scripts/prove-cargo-free-fixed-point.rs` proves the stronger stage1/stage2 fixed point, but it lives outside the CLI and relies on caller wiring for rustc/linker compatibility.

## Decisions

### 1. CLI owns fixed-point orchestration

**Choice:** Add a fixed-point mode to `mantle self-build --cargo-free` that runs two Cargo-free topology stages, clears a shared outside-root execution directory between stages, copies each produced Mantle binary into durable stage directories, and compares BLAKE3 digests.

**Rationale:** The operator should run one reviewed Mantle command. The proof remains bounded but no longer depends on an external script as the orchestration source of truth.

### 2. Keep pure planning separate from process execution

**Choice:** Extract a pure fixed-point planning core that computes stage directories, guard paths, execution roots, expected evidence paths, and command descriptors from owned inputs. Keep process spawning, filesystem writes, smoke execution, and JSON output in the imperative shell.

**Rationale:** The proof has non-trivial path and status rules. Pure planning makes positive and negative cases testable without launching the full world.

### 3. Command-owned toolchain compatibility

**Choice:** Add a reviewed compatibility layer for rustc/linker behavior. The command should probe whether the selected rustc accepts Mantle's topology runtime args, whether `-C link-self-contained=no` is supported, and whether the selected linker works in the cleared child environment. If normalization is needed, the command must create and record bundle-local compatibility data instead of requiring a caller-provided `/tmp` wrapper.

**Rationale:** Today raw rustup nightly hit a stale `gcc-ld/ld.lld` wrapper, while Nix rustc rejected `-C link-self-contained=no`. The CLI should either fix this through recorded normalization or fail closed with an actionable diagnostic.

### 4. Evidence remains audit-grade and bounded

**Choice:** Reuse the existing evidence vocabulary: preflight, per-stage receipts, stderr/status, smoke output, Cargo marker state, copied binaries, BLAKE3 digests, fixed-point status, and non-claims.

**Rationale:** Reviewers can compare the new command directly against the already successful script proof and existing `mantle self-build --cargo-free` summary.

## Risks / Trade-offs

- Toolchain normalization can accidentally hide host dependencies unless every synthesized path/wrapper is recorded with BLAKE3 and command inputs.
- Keeping the old script during migration risks drift; tests should force the CLI to be the canonical path.
- A real Mantle fixed-point run is several minutes; fast tests need tiny fixtures and blocker fixtures, with the full run as explicit evidence.
