# Design: Rust-plan verification lane boundary

## Product boundary

`mantle rust-plan` remains the explicit command for native Rust planning and Cargo-free execution experiments. `mantle build .#name` may use the offline Cargo project lane for Rust packages, but it must not silently switch to native topology execution unless a future promotion change updates the contract and evidence.

## Receipt classification

Rust-plan receipts should continue to bind unit graph facts, source closure facts, host/target topology facts, rustc args, output digests, blockers, and reuse status. The top-level evidence should include a compatibility class such as:

- `cargo-oracle-evidence` when Cargo material was captured for comparison;
- `cargo-free-bounded-topology` when no Cargo oracle/build orchestration was used for the explicit supported graph;
- `blocked-unsupported-surface` when the graph needs unsupported Cargo behavior;
- `not-default-project-build` when shown in product docs or project workflow reports.

The exact strings can be adjusted, but the claim boundary must be machine-readable.

## Default build behavior

Project build commands must choose explicit behavior:

- ordinary Rust project builds use the offline Cargo lane and label the result accordingly;
- native Rust planner execution requires explicit `mantle rust-plan ... --execute-*` or a future opt-in flag;
- unsupported native planner surfaces produce blockers and must not fall back to Cargo while claiming Cargo-free execution.

## Promotion criteria

A future change may promote native Rust planning into a broader product path only after it provides:

- compatibility rail evidence for a representative workspace;
- positive and negative coverage for source closure, host artifacts, build scripts, proc macros, feature/cfg selection, and registry/vendor dependencies;
- report wording that distinguishes native planner claims from compiler/release/bootstrap claims;
- migration docs explaining when users should choose offline Cargo versus native rust-plan;
- current validation receipts.

## Verification strategy

- CLI tests prove `mantle build` does not invoke native topology implicitly for Rust project fixtures.
- Rust-plan tests prove explicit `--execute-topology` still emits bounded receipts or deterministic blockers.
- Documentation tests reject wording that presents rust-plan as full Cargo compatibility.
- Negative tests prove unsupported native topology does not repair itself by invoking Cargo as hidden orchestration.

## Requirement trace

- r[rust_package_planning.verification_lane_boundary]
