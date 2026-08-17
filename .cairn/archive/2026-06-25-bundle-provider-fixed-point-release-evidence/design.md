# Design: bundle provider fixed-point release evidence

## Context

`mantle release verify` already has a provider fixed-point proof verifier that can validate an externally supplied proof bundle. Release evidence creation currently records the main self-hosting proof bundle, source archive, binaries, prerequisite inventory, and optional reproducibility evidence. The provider fixed-point proof needs the same bundle-local durability so downstream verifiers can evaluate it from the release evidence bundle alone.

## Decisions

### 1. Store provider fixed-point proof as a separate bounded artifact

The release manifest should add an optional provider fixed-point proof artifact instead of overloading the existing self-hosting proof bundle. The artifact must carry its relative path, digest, size, and bounded role so verifiers can distinguish it from reproducibility or self-hosting evidence.

### 2. Validate before packaging

Release creation should run the existing provider fixed-point proof verifier before copying the proof into the release bundle. Invalid proof bundles must fail closed and must not be packaged as trusted evidence.

### 3. Prefer bundle-local proof during verification

When `--require-provider-fixed-point-proof` is set and no external proof path is supplied, release verification should locate the manifest-recorded bundled proof and validate it. If both bundle-local and external paths are supplied, the external path is an explicit operator override and the JSON output should identify the source used.

### 4. Keep claims bounded

The provider fixed-point proof remains a release-adjacent evidence block. It must not change reproducibility status, deterministic release eligibility, or binary artifact matching. The manifest and verifier output must preserve non-claims for release reproducibility and full Cargo compatibility.

## Validation

Focused validation should cover manifest serialization, release creation with a valid proof directory, release verification using a bundle-local proof, required-mode failure when the manifest lacks the proof, and external override behavior. Run focused Rust tests, `cargo build -p mantle --bin mantle`, `git diff --check`, and Cairn validation/gates before implementation is marked complete.
