# Design: bundle deterministic release proof

## Context

Release evidence currently has three related proof surfaces: the release manifest, the reproducibility report under the bundle, and deterministic proof sidecars supplied to verification with explicit paths. Provider fixed-point proof evidence is now bundle-local, but deterministic-release proof evidence still requires sidecar coordination.

## Decisions

### 1. Model deterministic proof artifacts separately from the report

The reproducibility report states artifact comparison results. The deterministic build proof receipt and sandbox isolation evidence prove the stronger repeated sandboxed rebuild condition. The manifest should record these as separate optional artifacts with relative path, digest, size, and bounded role.

### 2. Let `release reproduce` attach the proof to the bundle

When deterministic proof runs are requested, `release reproduce` should write the proof receipt and sandbox isolation evidence to a bundle-local location and update manifest evidence atomically enough that verification never sees a trusted manifest reference to a missing proof file. Existing `--deterministic-proof-dir` may remain as an output/staging location if needed, but the bundle-local copy is the durable release-evidence surface.

### 3. Verification prefers explicit override, then bundle-local proof

If an operator supplies external deterministic proof paths, verification should use those paths and report an external proof source. If no external paths are supplied, required deterministic-release verification should look for manifest-recorded bundle-local proof artifacts. Missing or invalid bundled proof evidence must produce deterministic blockers.

### 4. Preserve claim bounds

Bundling deterministic proof artifacts should change deterministic-release eligibility only when the deterministic proof validates. It must not imply compiler correctness, full bootstrap reproducibility, full Cargo compatibility, or deploy success.

## Validation

Focused validation should cover manifest serialization, `release reproduce` attaching proof artifacts, required verification from bundled proof, explicit external override behavior, missing proof failure, corrupted proof failure, and mismatched artifact digest failure. Run focused Rust tests, `cargo build -p mantle --bin mantle`, `git diff --check`, and Cairn validation/gates before implementation is marked complete.
