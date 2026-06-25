# bundle deterministic release proof

## Problem

`mantle release reproduce` can produce a reproducibility report and external deterministic proof sidecars, and `mantle release verify` can require those sidecars with explicit paths. That leaves a portability gap: release bundles can carry the reproducibility report while the deterministic proof receipt and sandbox isolation evidence remain outside the bundle and outside `manifest.json`.

This mirrors the gap that was closed for provider fixed-point proof evidence. Downstream verifiers should be able to require deterministic-release evidence from the release bundle itself, while still supporting explicit external proof paths for older bundles and operator overrides.

## Proposed change

Make deterministic-release proof artifacts first-class release evidence bundle members. `mantle release reproduce` should write or package the deterministic build proof receipt and sandbox isolation evidence into the release bundle, record their relative paths, BLAKE3 digests, sizes, and bounded role in manifest evidence, and keep the existing reproducibility report linkage.

Update `mantle release verify --require-deterministic-release` to use bundle-local deterministic proof artifacts when external `--deterministic-proof` and `--deterministic-sandbox-isolation-evidence` paths are not supplied. Preserve explicit external sidecar paths as overrides and report the proof source used.

## Success criteria

- Release evidence bundles can carry deterministic build proof and sandbox isolation evidence artifacts.
- Manifest evidence records relative paths, BLAKE3 digests, sizes, and bounded roles for deterministic proof artifacts.
- Required deterministic-release verification succeeds from a bundle-local proof without external sidecar paths.
- External deterministic proof paths remain supported and are reported as explicit overrides.
- Invalid, missing, or mismatched bundled deterministic proof artifacts fail closed.
- Human and JSON output preserve non-claims for full bootstrap reproducibility, compiler correctness, full Cargo compatibility, and deployability.
