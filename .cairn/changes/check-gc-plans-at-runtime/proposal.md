# Change: Refuse GC plans the Terrane kernel rejects

## Why

The parity rail of ADR 0157 covers stores of up to three entries. Production
stores are larger, so a `plan_gc` defect outside that space could still delete
a live path. Running the verified checker on every real plan before deletion
closes that gap for the plan itself.

## What Changes

- Add `crunch-gc-check`, which compiles the imported kernel with `vstd` as a
  normal dependency and exposes a fail-closed projection and check (ADR 0158).
- Route both core-plan sites in `crunch-store` through `checked_plan_gc`, which
  refuses a plan the kernel rejects before any removal.
- Move the parity rail into `crunch-gc-check` so it exercises the production
  projection; remove the test-only include and `vstd` dev-dependency from
  `crunch-gc-core`.

## Non-Goals

- Verifying the projection, snapshot completeness, or removal execution.
- An override for a refused plan.

## Impact

- **Affected spec:** `gc-plan`
- **Affected code:** new `crates/crunch-gc-check`, `crates/crunch-store`
  (`gc.rs`, manifest), `crates/crunch-gc-core` (test include removed), the
  importer's consumer manifest, workspace members, ADRs 0157 and 0158
- **Testing:** the moved parity rail, corrupted-plan tests, a store-level
  test, and a mutated-planner run through the store's GC tests
