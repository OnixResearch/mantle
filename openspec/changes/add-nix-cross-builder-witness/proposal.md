## Why

Mantle now has a bounded two-clean-store deterministic proof for release artifacts, but the proof can still drift away from the Nix packaging path. A Nix build of the same Mantle source/toolchain recipe provides a useful independent cross-builder witness: if Nix and Mantle produce the same bit-exact binary, we gain evidence that both build descriptions agree on the artifact bytes.

## What Changes

- Define a `nix-cross-builder-witness` proof class/report that compares the Mantle self-built release artifact against a Nix-built artifact.
- Require the witness to bind source/vendor digests, Rust toolchain identity, target triple, build flags, strip/debug policy, Nix derivation identity, Mantle proof identity, and BLAKE3 artifact digests.
- Keep the claim separate from and subordinate to the existing Mantle two-clean-store proof: Nix agreement strengthens release evidence but does not replace `self-rebuild-match`.
- Specify fail-closed mismatch behavior and bounded claim wording.

## Capabilities

### New Capabilities
- `release.verification.tech.cross-builder.nix-witness`: verifies bit-exact agreement between Nix-built and Mantle self-built release artifacts.
- `build-pipeline.release.nix-cross-builder-witness`: lets the release build produce or require the Nix witness receipt without making ordinary dev builds pay the cost.

### Modified Capabilities
- `release.verification.tech.reproducibility.claims`: adds cross-builder agreement as a named evidence class, not a global reproducibility claim.

## Impact

- **Files**: OpenSpec deltas under `release-verification-tech` and `build-pipeline`.
- **APIs**: Future implementation likely adds a `mantle release build --require-nix-witness`/equivalent release option and a `mantle-nix-cross-builder-witness-v1` receipt.
- **Dependencies**: No immediate code dependency in this spec-only change; implementation will use the existing Nix flake package/check surface.
- **Testing**: Validate with `openspec validate add-nix-cross-builder-witness --strict`.
