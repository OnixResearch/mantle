# Rust-plan architecture search

Source commit: `971b5c576e61005f2ee8f5857358471a2f468837`

## Question

Which existing component can own Mantle's Rust package and unit-planning meaning without importing host authority?

## Inspected evidence

- `src/rust_plan.rs` is 28,234 lines and combines normalized planning records with filesystem traversal, TOML and Cargo JSON parsing, Cargo and rustc processes, environment reads, cache services, receipt rendering, and `RunError`.
- The in-module `CargoOracle` port accepts `Path`, returns `RunError`, and is implemented beside `ProcessCargoOracle`.
- `crunch-rust-cache-core` owns Rust action and cache-result identity. It does not own package selection, feature resolution, target classification, topology, or compiler effect planning.
- No existing OnixResearch crate or Mantle workspace crate provides a compatible Rust package-planning core.
- The completed remote hexagon supplies a local pattern for a strict core, application-owned ports, outer adapters, architecture fixtures, and focused Nix gates.

## Decision

Create `mantle-rust-plan-core` and `mantle-rust-plan`. Reuse existing cache identities only through adapter projections. Do not move cache authority into the planning core.

The core will own bounded normalized package facts, feature closure, package closure, host and target unit classification, topology, unit effects, observation classification, compatibility status, and receipt preimages. The application will own workspace, Cargo oracle, compiler inspection, unit execution, and cache ports.

The root shell will preserve current CLI and JSON behavior. It will move the process-backed Cargo oracle out of `src/rust_plan.rs`, delegate compatible deterministic decisions to the extracted core, and keep unsupported Cargo surfaces fail-closed.

## Non-claims

The search does not prove Cargo equivalence, rustc correctness, linker correctness, cache correctness, or support for Cargo behavior outside the accepted matrix.
