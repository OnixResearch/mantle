## Context

The native Rust planner already reads each package manifest to derive targets, sources, and dependency facts. Target facts carry name, kind, crate name, and source path, but not the package edition. Native unit derivation and native host unit derivation shells therefore emit `--edition 2021` regardless of the manifest.

The current self-probe shows topology execution has moved beyond PATH/linker environment blockers and now fails because `vendor/nix-compat-derive` uses Rust 2024 let chains while Mantle invokes rustc with edition 2021.

## Decisions

### 1. Edition is target fact data

**Choice:** Add an `edition` field to `NativeTargetPlanningSummary`, `NativeRustUnitSummary`, and `NativeHostUnitSummary` and thread it from `[package].edition` through all native derivation planning.

**Rationale:** The edition is deterministic manifest data, not runtime environment state. Carrying it through native facts keeps unit derivation generation pure and testable.

### 2. Missing edition defaults to Cargo 2015

**Choice:** Treat absent `package.edition` as `2015`.

**Rationale:** Cargo's documented default package edition is 2015. Matching that behavior avoids silently granting newer syntax to crates that did not request it.

### 3. Preserve bounded scope

**Choice:** This change only handles package-level edition propagation for existing supported native target kinds and dev-dependency helper derivations.

**Rationale:** Target-specific edition overrides do not exist in Cargo manifests. Broader rustc argument parity remains outside this bounded topology frontier.

## Risks / Trade-offs

- Unit derivation graph digests will change because edition is part of rustc args.
- Existing fixture crates without `edition` will now plan as 2015; tests should make that default explicit where it matters.
