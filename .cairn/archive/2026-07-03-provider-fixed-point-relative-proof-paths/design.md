## Context

The provider fixed-point verifier accepts both relative paths and legacy absolute paths. For old release bundles, it rebases absolute paths under the recorded `bundle_dir` to the copied proof directory. That compatibility is useful, but it leaves new proof generation with unnecessary absolute stage paths in `meta.json` and `preflight.json`.

## Decisions

### 1. Relativize proof-owned paths at summary generation

**Choice:** `fixed_point_summary(...)` stores `bundle_dir` as `.` and rewrites stage-owned paths under the proof bundle to relative components. `write_fixed_point_preflight(...)` does the same for the proof-owned shared execution directory.

**Rationale:** The summary writer is the boundary where in-memory execution paths become durable proof metadata. Relativizing there keeps execution code simple and preserves absolute paths while commands are running.

### 2. Keep external provenance paths absolute

**Choice:** Paths outside the proof bundle are not rewritten. Source root, source-built Rust provider paths, requested/stage rustc paths, and toolchain closure metadata remain as recorded provenance.

**Rationale:** Those paths identify external evidence inputs or host/provider routes, not copied proof artifacts. Rewriting them to bundle-local paths would be dishonest and could hide the remaining trust boundary.

### 3. Keep legacy absolute-path verification support

**Choice:** The verifier keeps its absolute-path rebasing fallback for older bundles, but new metadata does not require that path.

**Rationale:** Existing durable release evidence should remain verifiable, while new proofs become portable by construction.

## Risks / Trade-offs

- JSON output from the fixed-point command now reflects portable proof metadata paths for proof-owned artifacts. Human operators needing the original output directory still have the command-line `--out` value and filesystem context.
- This change does not remove all absolute paths from proof metadata; external provenance paths intentionally remain explicit non-bundle facts.
