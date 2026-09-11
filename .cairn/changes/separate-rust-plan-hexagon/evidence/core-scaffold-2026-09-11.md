# Evidence: Rust-plan core scaffold (2026-09-11)

Task-ID: mantle.rust_package_planning.hexagonal_core
Covers: hexagonal_core, application_owned_ports

## What landed (I1)

`crates/mantle-rust-plan-core` — strict `no_std + alloc`, no filesystem,
process, environment, clock, Cargo, rustc, cache, or CLI dependency:

- Bounded structural facts: `PackageFacts`, `PackageSource`, `TargetFacts`,
  `DependencyFacts`, `FeatureFacts`, `FeatureReference` with named bounds
  (`MAX_PACKAGES`, `MAX_TARGETS_PER_PACKAGE`, `MAX_DEPENDENCIES_PER_PACKAGE`,
  `MAX_FEATURES_PER_PACKAGE`, `MAX_ARGS_PER_EFFECT`,
  `MAX_ENVIRONMENT_ENTRIES_PER_EFFECT`).
- Typed `PlanBlocker` rejection instead of panics for malformed, duplicate,
  ambiguous, or unsupported facts.
- Nominal planning: `PlanRequest`, `BuildProfile`, `UnitLimitFacts`,
  `PlannedUnit`, `UnitEffect`, `RustPlan`, `PlanOutcome` with ordered effects
  and deterministic domain-separated unit and plan identities.
- Observation classification: `UnitObservation`,
  `UnitObservationStatus`, `PlanExecutionOutcome` (completed, exact failure
  count, or rejected on unknown/duplicate/missing effect identities).
- Receipt preimages: `PlanReceiptPreimage`, `build_receipt_preimage`.

## Rails

- `cargo test -p mantle-rust-plan-core`: 10 fixtures pass (positive planning,
  dependency-target scoping, determinism and receipt binding, and negatives
  for missing roots, ambiguous package names, missing dependency packages,
  duplicate and mismatched targets, malformed feature/dependency facts,
  declared unit limits, and observation drift).
- `cargo clippy -p mantle-rust-plan-core --all-targets -- -D warnings`:
  exit 0.
- `cargo check -p mantle-rust-plan-core --target wasm32-unknown-unknown`:
  clean.
- Tiger Style consumer check: exit 0.

## Open

I2–I6 (moving logic out of `src/rust_plan.rs`, application ports, adapter
replacement, effect execution loop, legacy-path removal) and V1–V5.
