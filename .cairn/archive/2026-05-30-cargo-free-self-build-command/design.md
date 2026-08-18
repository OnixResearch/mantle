## Context

The fixed-point proof runner already knows how to guard Cargo, invoke `mantle rust-plan --no-cargo-oracle --execute-topology`, find the produced Mantle binary, hash it, and write audit evidence. That runner is proof-oriented and compares two stages; it is not the normal operator entry point for producing a binary.

## Decisions

### 1. Reuse the native topology rail through the CLI boundary

**Choice:** Implement `self-build --cargo-free` as a thin shell that invokes the current Mantle executable with `rust-plan --no-cargo-oracle --execute-topology` and a guarded environment.

**Rationale:** The existing `rust-plan` command is the stable public execution rail and already emits the full receipt shape. Spawning it keeps the Cargo guard explicit without mutating process-global environment and keeps the new command small.

### 2. Keep output evidence outside the source digest by default

**Choice:** Require `--out` to be outside the selected source root and place the execution root, guard marker, receipt, smoke output, summary, and copied binary under that output directory.

**Rationale:** Cargo-free source digests must not be perturbed by proof/build artifacts. The fixed-point proof already found that source-root output directories create false drift.

### 3. Emit a bounded build summary, not a release claim

**Choice:** The command writes a `meta.json` summary with schema, root, binary path, BLAKE3 digest, source digest, receipt path, Cargo guard status, unit counts, and non-claims.

**Rationale:** Operators need a concise build result, while the full receipt remains available for audit. The summary must not imply Crunch bootstrap, release reproducibility, full Cargo compatibility, or source-built compiler provenance.

## Risks / Trade-offs

- Spawning the CLI has process overhead, but the build itself dominates cost and this keeps environment guarding deterministic.
- Requiring output outside the source root is stricter than `target/`, but avoids source-digest self-contamination.
- This mode remains bounded to Mantle's native Rust topology subset; unsupported Cargo behavior must continue to block.
