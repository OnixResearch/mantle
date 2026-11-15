# Change: Check GC planning against a Terrane kernel

## Why

`plan_gc` decides which store paths may be deleted, and a wrong decision
deletes a live path. Terrane now has a verified `reclaim_plan` kernel whose
checker is proved to reject any plan that would delete a reachable or
protected path. Mantle needs an executable rail that shows `plan_gc` agrees
with that kernel and that the kernel's checker accepts Mantle's own output.

## What Changes

- Import the Terrane kernel and its tests unchanged into
  `vendor/terrane-reclaim-plan/` with a repository-owned importer that checks
  exact BLAKE3 identities and records the producer and consumer Verus standard
  library pins (ADR 0157).
- Add a test-only parity rail in `crunch-gc-core` over every store of up to
  three entries, with missing-root and dangling-reference variants.

## Non-Goals

- Moving runtime authority to the kernel.
- Checking runtime plans with the kernel before deletion.
- Fresh Verus verification inside Mantle, or proof acceptance through Kamacite
  and Valence.

## Impact

- **Affected spec:** `gc-plan`
- **Affected code:** `crates/crunch-gc-core` (parity tests, `vstd`
  dev-dependency), `vendor/terrane-reclaim-plan/`, the importer script, the
  flake source filter, ADR 0157
- **Testing:** the parity rail, a projection test for duplicate and unsorted
  input, and a mutated-reachability check
