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

## Feature resolution slice (I2 progress, 2026-09-11)

`crates/mantle-rust-plan-core/src/features.rs` now owns deterministic
feature activation over supplied facts:

- `PackageFeatureRequest` (package, requested features, default-feature
  inclusion) and `FeatureResolution` (activated features, activated optional
  dependencies, both canonical order).
- Activation is a bounded forward fixed point (`MAX_FEATURE_PASSES`): seeds
  are the requested names plus the implicit `default` feature, and enabling a
  feature enables its declared feature references. Unknown requests, oversized
  requests, package mismatch, and non-stabilizing closures yield typed
  blockers.
- The planner consumes resolutions: optional dependencies participate only
  when a feature activates them, targets whose `required_features` are not
  activated are blocked with `target-required-feature-missing`, and activated
  features enter unit identity (so a feature change changes unit and plan
  identities without changing the unit set).

Fixtures: 14 pass, adding activated/inactive optional dependency behavior,
unknown feature requests, target required-feature gating, and identity
sensitivity to activation. Focused Clippy exit 0; wasm32 check clean.

## Application ports and orchestration (I3, 2026-09-11)

`crates/mantle-rust-plan-app` (strict `no_std + alloc`) owns the
application contract:

- Ports with Mantle-owned types only — `WorkspaceFactsSource`,
  `CargoOracleCapture`, `CompilerInspection`, `RustCacheAccess`,
  `UnitExecutor` — each returning `Result<_, AdapterError>` with a stable
  capability code and bounded detail. No `RunError`, process, Cargo JSON, host
  path, or store type appears in a signature.
- `RustPlanApplication::run` orchestrates: load facts, capture optional
  oracle material, inspect the compiler, plan in the core, consult the cache
  per effect, execute misses, classify observations against the plan, and
  build the receipt preimage. Planning blockers return
  `ApplicationOutcome::Blocked` and perform no work; capability failures
  propagate as typed adapter errors.
- Cache accounting is explicit: `CacheDisposition` (all missed, partially
  hit, all hit) plus per-unit `UnitDisposition`, with cache-served units
  recorded as succeeded observations carrying a `cache-hit` diagnostic.

Fixtures: 5 pass (successful run with receipt, partial cache hit without
execution, blocked plan performing no work, typed adapter failures from
workspace/cache/executor ports, and failed or substituted observations
classifying exactly). Focused Clippy exit 0; Tiger Style exit 0; wasm32
check clean.

Concrete std adapters and legacy-path removal remain under I4–I6.

## Boundary guard (V3 progress, 2026-09-11)

`scripts/check-rust-plan-boundary.rs` is the repository-owned guard for both
pure crates:

- Forbids filesystem, process, environment, clock, thread, network, async
  runtime, CLI error, rendering, and cargo/rustc tokens in
  `crates/mantle-rust-plan-core/src` and `crates/mantle-rust-plan-app/src`,
  skipping comment lines.
- Restricts manifest dependencies to the reviewed cohort (core: blake3,
  serde, serde_json; app: mantle-rust-plan-core, serde) and rejects any
  unreviewed dependency.
- `--self-test` proves a positive fixture passes and that filesystem,
  process, and rendering tokens plus unreviewed manifest dependencies fail.

Commands: `cargo -Zscript scripts/check-rust-plan-boundary.rs --self-test`
→ PASS; `cargo -Zscript scripts/check-rust-plan-boundary.rs` → PASS (9 files).

API-shape, compile-fail, and full Cargo-oracle compatibility checks remain
under V1–V3.
