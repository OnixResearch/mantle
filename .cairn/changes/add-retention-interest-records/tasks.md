# Tasks: Retention interests replace the shared GC roots file

Completed tasks below record the implementation and exercised evidence; remaining verification and archival steps stay open until observed.

## Phase 1: Baseline and contract

- [x] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current roots-file behavior, root registration call sites, release path, and one concurrent-writer observation. r[mantle.store_lifecycle.retention_interest_records]
- [x] [serial] T1.2 Define the versioned retention-interest record: owner identity, logical store path, reason, declaration facts, canonical encoding, and the BLAKE3 record identity. r[mantle.store_lifecycle.retention_interest_records]
- [x] [serial] T1.3 Record the record-per-interest, owner-scoped-release, and merged-view decisions in an ADR. r[mantle.store_lifecycle.retention_owner_scope]

## Phase 2: Core and persistence

- [x] [serial] T2.1 Implement pure record validation, canonical encoding, identity computation, deterministic merge, and owner-scoped release planning. r[mantle.store_lifecycle.retention_interest_records]
- [x] [serial] T2.2 Publish and remove single records atomically under the state directory, with bounded size and count limits. r[mantle.store_lifecycle.retention_owner_scope]
- [x] [parallel] T2.3 Add positive fixtures: two owners retain one path, both survive; release removes exactly one record; merged set feeds the GC planner unchanged. r[mantle.store_lifecycle.retention_interest_records]
- [x] [parallel] T2.4 Add negative fixtures: malformed record, unknown version, duplicate record, foreign-owner release, oversized record, and record count over limit. r[mantle.store_lifecycle.retention_owner_scope]

## Phase 3: Command surface and migration

- [x] [serial] T3.1 Extend `store roots`, `store usage`, and `store info` with owner, reason, and record-count facts, keeping JSON output stable under a schema version. r[mantle.store_lifecycle.retention_owner_scope]
- [x] [serial] T3.2 Add explicit `store roots --migrate` conversion from the legacy JSON roots file, preserving `legacy-unmanaged` entries. r[mantle.store_lifecycle.retention_interest_records]
- [x] [serial] T3.3 Prove plan equivalence: a GC plan from migrated records matches the plan from the equivalent legacy root set. r[mantle.store_lifecycle.retention_interest_records]

## Phase 4: Verification

- [ ] [serial] T4.1 Run the concurrent-owner, retraction, and migration rails before and after the change. Preserve exact results. r[mantle.store_lifecycle.retention_owner_scope]
- [ ] [serial] T4.2 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, and the relevant Nix checks. r[mantle.store_lifecycle.retention_interest_records]
- [ ] [serial] T4.3 Sync accepted specs and retain final-source completion evidence in the isolated branch, leaving the change archive-ready for the separately verified terminal Cairn archive command. r[mantle.store_lifecycle.retention_owner_scope]
