## Why

`mantle bootstrap --fetch` is a useful Nix-free seed-assisted path, but it still requires live network access to acquire the pinned provider tarball. `mantle self-build` also relies on explicit source tree and `vendor-deps/` inputs, but those inputs are not packaged through one operator-facing offline source bundle workflow. This makes air-gapped or intermittently connected bootstrap runs harder than they need to be.

Mantle already has source-bundle record kinds for bootstrap archives, provider manifests, toolchain source roots, and proof inputs. The bootstrap workflow should consume those records directly so an operator can prepare or copy all bootstrap source material on one machine, import and pin it on another, and run bootstrap/self-build without live source downloads.

## What Changes

- Define an offline bootstrap source-bundle profile for seed/provider/toolchain/source/vendor/proof inputs required by bootstrap and self-build workflows.
- Allow bootstrap commands to preflight and consume imported/pinned bootstrap source records instead of fetching live network sources when the bundle is complete.
- Bind provider metadata, source identities, fixed-output hashes, and reduced-provider provenance into the offline bootstrap report.
- Fail closed when required bootstrap source records are missing, stale, unpinned, wrong-prefix, wrong-provider-kind, or not supported by the selected proof mode.
- Document the claim boundary: offline bootstrap source bundles prove source/input availability and identity, not provider trust, compiler correctness, or bootstrap correctness by themselves.

## Impact

- **Files**: `src/bootstrap.rs`, `src/bootstrap_validate.rs`, `src/source_bundle.rs`, `src/self_build.rs`, `scripts/prove-self-hosting.sh`, bootstrap docs, operator proof guide, and this Cairn spec delta.
- **Testing**: source-bundle export/import/preflight for bootstrap roots, offline bootstrap fetch replacement, self-build source/vendor bundle checks, wrong provider metadata negatives, and no-network guards.

## Out of Scope

- Eliminating trust in the reduced seed provider.
- Proving source-built compiler correctness.
- Downloading missing bootstrap sources during offline bootstrap execution.
- Changing release reproducibility or witness proof admission rules beyond recording the new input source boundary.
