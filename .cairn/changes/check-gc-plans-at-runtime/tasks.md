# Tasks: Refuse GC plans the Terrane kernel rejects

Checked tasks identify delivered source and local test runs. They are not
Cairn gate acceptance, fresh Verus verification inside Mantle, or proof
acceptance.

## Phase 1: Check

- [x] [serial] T1.1 Add `crunch-gc-check` with the kernel include, `GcCheck::prepare`, and `GcCheck::check`. r[mantle.gc_plan.runtime_check]
- [x] [serial] T1.2 Move the parity rail into `crunch-gc-check` on the production projection; remove the test include and `vstd` dev-dependency from `crunch-gc-core`; point the importer at the new manifest. r[mantle.gc_plan.planning_parity]
- [x] [parallel] T1.3 Add corrupted-plan and duplicate-entry tests. r[mantle.gc_plan.runtime_check]
- [x] [serial] T1.4 Replace the trusted path-to-index projection with the kernel's verified `project_keys` and `key_mask` (Terrane 5aec06f), and compare the production projection with an independent test projection on every enumerated case. r[mantle.gc_plan.runtime_check]

## Phase 2: Store

- [x] [serial] T2.1 Route `build_plan` and the Casita castore planner through `checked_plan_gc`. r[mantle.gc_plan.runtime_check]
- [x] [parallel] T2.2 Add a store-level test and run the store's GC tests with a mutated planner. r[mantle.gc_plan.runtime_check]

## Phase 3: Validation and lifecycle

- [ ] [serial] T3.1 Review `vstd` and the Verus macro crates for the production supply chain (licenses, `cargo deny`, Nix vendoring). r[mantle.gc_plan.runtime_check]
- [ ] [serial] T3.2 Wire `scripts/import-terrane-reclaim-plan.py --check` into a flake check, and decide how Tiger Style treats the vendored module. r[mantle.gc_plan.kernel_import]
- [ ] [serial] T3.3 Run the first-party quality gate, Tiger Style, `nix flake check`, and `cairn validate --root .`; capture transcripts. r[mantle.gc_plan.runtime_check]
- [ ] [serial] T3.4 Accept or revise ADR 0158. r[mantle.gc_plan.runtime_check]
