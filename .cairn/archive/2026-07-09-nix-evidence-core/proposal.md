## Why

Mantle release bundles, Cairn Nix verification gates, Molten release promotion evidence, and Valence provenance adapters all describe Nix-store-shaped evidence: store paths, derivation outputs, output identity, builder/runtime caveats, and bounded build/release non-claims. Mantle owns the Nix store protocol build system in this stack, so it should own the reusable Nix evidence core.

This extraction shares Nix evidence DTOs and validators while keeping evaluation, builds, substituter access, sandbox execution, and release decisions in their existing shells.

## What Changes

- Define a pure Nix evidence core for store path refs, derivation/output identities, output digest metadata, realization role labels, build environment caveats, and bounded non-claims.
- Add compatibility adapters for Mantle release provenance, Cairn Nix gates, Molten release promotion evidence, and Valence stack-provenance inputs.
- Add positive fixtures for valid Mantle build/release evidence and optional external Nix evidence rows.
- Add negative fixtures for malformed store paths, mismatched outputs, stale digests, unsupported derivation identity, missing caveats, missing non-claims, and build-correctness overclaims.

## Impact

- **Mantle** owns shared Nix evidence identity primitives.
- **Cairn/Valence/Molten** can consume those primitives without invoking Mantle builds or Nix evaluation.
- **Release evidence** remains bounded: identity and bundle-local facts only, not semantic correctness.
