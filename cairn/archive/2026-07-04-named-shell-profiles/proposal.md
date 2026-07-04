# Proposal: Named shell profiles

## Summary

Adopt Organist's named shell profile shape in Mantle terms: projects may declare `build`, `dev`, and default shell profiles that lower to ordinary Mantle build inputs and sidecar activation plans. Dev shells stay decoupled from build action identity, file generation, release evidence, and reproducibility claims, while services and module-layer behavior remain outside Mantle core.

## Motivation

Mantle already has `mantle shell`, but the project surface should let teams distinguish build inputs from operator convenience. Organist's `shells.build`, `shells.dev`, and `shells.default` shape is a useful user-facing model. Mantle should adopt the profile idea while keeping the output build-shaped: a shell profile is data that lowers to derivations and activation sidecars, not a service manager, Nix-flake compatibility layer, or source of truth for build action specs.

Dev shells need an especially strong boundary. Entering a richer development environment must not change build hashes, regenerate files, refresh locks, or upgrade proof claims. If shell activation itself ever needs evidence, it should have a separate shell-activation receipt instead of contaminating build or release receipts.

## Scope

- Add project-manifest shell profile declarations with stable names, package/build inputs, environment entries, PATH entries, and optional command hooks supported by existing shell planning boundaries.
- Define `build`, `dev`, and `default` profile conventions without hard-coding frontend module semantics.
- Make `mantle shell [name]` resolve named profiles deterministically.
- Keep shell profile env/path planning in the existing functional-core / adapter split.
- Require dev shell activation to be non-mutating by default and decoupled from build action identity, file generation, lock refresh, and release evidence.
- Explicitly reject or defer service lifecycle declarations in Mantle core.

## Non-goals

- No `nix develop` or flake output compatibility requirement.
- No long-running service manager, `docker-compose` replacement, or process supervisor.
- No implicit package import from Nixpkgs, Onix modules, or frontend-specific package sets.
- No claim that entering a dev shell proves builds, tests, services, release workflows, reproducibility, or build action correctness.
- No use of `shells.dev` as the source of truth for package build inputs or action specs.

## Target Spec Domains

- `project-workflows` for named shell profile semantics and build-tool boundary constraints.
