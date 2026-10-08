# Tasks: Add bootstrap source pin tracking

Completed task boxes below cite the baseline and implementation receipts in
`evidence/`. Verification/lifecycle remain open until their exact gates and
isolated-branch requirements are fulfilled; advisory gates are not producer
acceptance.

## Phase 1: Baseline and contract

- [x] [serial] T1.1 Create an isolated worktree from current `origin/main`. Inventory the inline URL and hash pins under `bootstrap/`, record the `crunch-project` refresh surface and the accepted `mantlepkgs-update-plans` boundary, and capture focused baseline test output. r[mantle.bootstrap_source_pins.pin_data_contract]
- [x] [serial] T1.2 Define the versioned pin record format with package-URL identity, URL templates, pin fields, hash kinds, unknown-key rejection, and optional declared resolve hooks. r[mantle.bootstrap_source_pins.pin_data_contract]
- [x] [serial] T1.3 Record the boundary decision against `mantlepkgs-update-plans` and the plan-artifact decision in an ADR. r[mantle.bootstrap_source_pins.apply_writes_pins_only]

## Phase 2: Core pipeline

- [x] [serial] T2.1 Implement pure record validation, version comparison, and plan decision in the project core. r[mantle.bootstrap_source_pins.pin_data_contract]
- [x] [serial] T2.2 Implement the reader seam so migrated recipes resolve URL, version, and hash from pin data with no residual hardcodes. r[mantle.bootstrap_source_pins.nickel_reads_only]
- [x] [parallel] T2.3 Add positive fixtures: valid round-trip, unchanged-world cache answer, pending-update report with nonzero exit. r[mantle.bootstrap_source_pins.batched_resolution]
- [x] [parallel] T2.4 Add negative fixtures: unknown field, missing hash, flat-versus-tree hash confusion, residual hardcoded URL after migration. r[mantle.bootstrap_source_pins.pin_data_contract] r[mantle.bootstrap_source_pins.nickel_reads_only]

## Phase 3: Shell commands

- [x] [serial] T3.1 Implement the batched resolver with conditional-request caching, per-host bounds, and declared resolve hooks. r[mantle.bootstrap_source_pins.batched_resolution]
- [x] [serial] T3.2 Implement apply with plan consumption, template substitution, fixed-output prefetch, pin-only writes, idempotence, and fail-closed mismatches. r[mantle.bootstrap_source_pins.apply_writes_pins_only]
- [x] [parallel] T3.3 Add apply negative controls: mutated plan, unknown source, upstream hash mismatch, partial-write rollback. r[mantle.bootstrap_source_pins.apply_writes_pins_only]

## Phase 4: Migration and verification

- [x] [serial] T4.1 Migrate one bootstrap recipe family to pin data and prove a version bump with no Nickel edits and identical build phases. r[mantle.bootstrap_source_pins.nickel_reads_only]
- [ ] [serial] T4.2 Run focused core and shell tests before and after changes, strict Clippy, policy freshness, and relevant Nix checks. Preserve exact blockers. r[mantle.bootstrap_source_pins.pin_data_contract]
- [ ] [serial] T4.3 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.bootstrap_source_pins.batched_resolution]

Re-verified on published main `e24bbbc2` (2026-10-08 UTC) in the isolated
Leviathan worktree `finish/add-bootstrap-source-pins`:
`evidence/verification-2026-10-08.md` maps T1.1–T4.1 to the retained transcripts
`evidence/baseline-inventory-main-2026-10-08.md`,
`evidence/baseline-focused-main-2026-10-08.md`,
`evidence/focused-after-2026-10-08.md`, `evidence/recipe-rail-2026-10-08.md`,
`evidence/recipe-rail-negatives-2026-10-08.md`, and
`evidence/cli-smoke-2026-10-08.md`, plus ADR 0086 and the migrated
CMake/Picolibc pin records. The 2026-09-30/10-01 receipts are historical
(older integration base) and are not relied on. The pre-existing main
operator-contract drift and its separate repair are in
`evidence/operator-flag-drift-main-2026-10-08.md`. T4.2 gate transcripts are in
`evidence/gates-2026-10-08.md`; T4.3 lifecycle transcripts are in
`evidence/lifecycle-2026-10-08.md`.
An accepted isolated implementation commit is required before any Cairn
archive, per `AGENTS.md`.
