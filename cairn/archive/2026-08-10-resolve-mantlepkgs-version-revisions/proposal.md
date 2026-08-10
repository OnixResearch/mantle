# Resolve Mantlepkgs package versions to exact Nixpkgs revisions

## Why

Mantlepkgs accepts one exact Nixpkgs source lock per generation. It cannot resolve a requested package version across historical Nixpkgs revisions.

Direct use of `nixpkgs-multiverse` would add an external flake interface and network-backed evaluation to package selection. That behavior conflicts with the Nix-free Mantlepkgs consumer boundary.

Mantle needs a bounded resolver before catalog production. The resolver must turn package-version intent into exact source facts and explicit evidence.

## What Changes

- Add typed Nickel contracts for revision cohorts, per-system indexes, package-version requests, selection policy, and named limits.
- Add a pure core that resolves reported versions and groups accepted requests by exact Nixpkgs revision.
- Add a bounded producer shell that observes declared channel revisions and publishes compact per-system indexes.
- Emit a BLAKE3-bound resolution receipt with the selected revision and the Nix SHA-256 `narHash`.
- Recheck the reported package version at the selected revision before Mantlepkgs generation.
- Generate one existing Mantlepkgs source manifest per selected revision.
- Adapt each generation into a domain shard and compose versioned public selectors without ambiguity.
- Use a pinned `nixpkgs-multiverse` revision as a comparison fixture, not as product authority.

## Non-Goals

- Adding `nixpkgs-multiverse` as a Mantle flake input or runtime dependency.
- Running Nix during Mantlepkgs catalog consumption.
- Treating a reported version as package, source, or output identity.
- Claiming current binary-cache availability from historical channel publication.
- Supporting overlays, arbitrary Nix configuration, or all Nixpkgs attributes in the first cohort.
- Selecting kernels, C libraries, boot components, or other system-cohort packages for OnixOS.

## Dependencies

- `mantlepkgs` supplies exact locked producer generations and the no-Nix consumer boundary.
- `mantlepkgs-catalog-structure` supplies domain shards and deterministic composition.
- `mantlepkgs-update-plans` supplies bounded observations and explicit unavailable states.
- ADR 0010 keeps frontend package policy above Mantle build behavior.
- ADR 0056 keeps Mantlepkgs generation based on concrete package graphs.

## Impact

- **Affected specs:** new `mantlepkgs-version-resolution` specification.
- **Affected code:** Mantlepkgs core, producer shell, CLI, Nickel contracts, fixtures, and domain composition planning.
- **Affected evidence:** revision observations, compact indexes, resolution receipts, version rechecks, generated manifests, and pilot reports.
- **Compatibility:** existing Mantlepkgs v1 manifests remain valid. New resolution artifacts use separate versioned schemas.
