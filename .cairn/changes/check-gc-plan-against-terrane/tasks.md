# Tasks: Check GC planning against a Terrane kernel

Checked tasks identify delivered source and local test runs. They are not
Cairn gate acceptance, fresh Verus verification inside Mantle, or proof
acceptance. The pinned Terrane revision is not yet published, so identity
tasks stay open.

## Phase 1: Import

- [x] [serial] T1.1 Add `scripts/import-terrane-reclaim-plan.py` with exact member inventory, BLAKE3 checks, and separate producer and consumer `vstd` pins. r[mantle.gc_plan.kernel_import]
- [x] [serial] T1.2 Import the kernel, its tests, and the license into `vendor/terrane-reclaim-plan/`; add the `vstd` dev-dependency and the flake source-filter entry. r[mantle.gc_plan.kernel_import]
- [ ] [serial] T1.3 Re-run the importer against the published Terrane revision and confirm the recorded identities. r[mantle.gc_plan.kernel_import]

## Phase 2: Parity

- [x] [serial] T2.1 Add the projection and the enumerated parity rail. r[mantle.gc_plan.planning_parity]
- [x] [serial] T2.2 Apply the kernel's `check_plan` to every Mantle plan in the rail. r[mantle.gc_plan.checked_output]
- [x] [parallel] T2.3 Run seven `plan_gc` mutations against the rail and record which fail. r[mantle.gc_plan.planning_parity]

## Phase 3: Validation and lifecycle

- [ ] [serial] T3.1 Run the ordinary first-party quality gate and Tiger Style in the dev shell; capture transcripts. r[mantle.gc_plan.planning_parity]
- [ ] [serial] T3.2 Run `cairn validate --root .` and the change gates; capture transcripts. r[mantle.gc_plan.kernel_import]
- [ ] [serial] T3.3 Accept or revise ADR 0157. r[mantle.gc_plan.kernel_import]
