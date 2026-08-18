# Design: Cargo-free fixed-point proof

## Context

The existing Cargo-free self-build proof shows host Mantle can produce a Mantle binary through native Rust topology execution. It does not prove that the produced binary can reproduce itself. A bounded fixed-point rail should exercise the produced binary as the stage2 builder and compare stage1/stage2 output digests.

## Decisions

### 1. Rust runner owns proof orchestration

**Choice:** Implement `scripts/prove-cargo-free-fixed-point.rs` as a single-file Rust script. It handles argument parsing, Cargo guard generation, stage command execution, JSON receipt inspection, binary discovery, smoke checks, BLAKE3 hashing, and evidence emission.

**Rationale:** The proof needs structured JSON and filesystem validation. Rust keeps the orchestration reviewable and avoids growing shell glue.

### 2. Stage output identity comes from receipts

**Choice:** Locate the produced Mantle binary by reading the successful `topology_execution.unit_executions` entry with `target_name=mantle` and `target_kind=bin`, then reconstructing the execution output directory from Mantle's safe unit-id component rule. Run both stages in the same outside-root execution directory, clearing it between stages, and copy each produced binary into its stage evidence directory before comparison.

**Rationale:** A raw filesystem search can accidentally pick a helper executable. Receipt-derived selection binds the smoke and digest checks to the claimed unit. The shared outside-root execution directory avoids false mismatches from stage-specific debug/build-script paths and avoids source digest drift from writing proof artifacts under the source root.

### 3. Fixed-point claim remains bounded

**Choice:** The proof succeeds only when stage1 and stage2 Mantle binary BLAKE3 digests match. The bundle still records non-claims for Crunch bootstrap, release reproducibility, source-built toolchain closure, and full Cargo compatibility.

**Rationale:** Matching stage binaries proves a stronger Mantle-native fixed point, not a whole-system bootstrap.

## Risks / Trade-offs

- Rust debug/build metadata can still prevent byte-for-byte convergence if any non-path nondeterminism remains.
- The rail still depends on the recorded host `rustc`, linker, and vendored/local source material.
- The script mirrors Mantle's safe unit-id path rule; if that rule changes, the proof runner must change with it.
