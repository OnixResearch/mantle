## Phase 1: Session schema

- [x] [serial] Define structured refactor session schema for aliases, paths, CLI names, store prefixes, schema versions, and compatibility policy.
  - Completed: added `src/structured_refactor.rs` with `mantle-structured-refactor-session-v1` records, affected aliases/CLI/files/store-prefixes, compatibility policy, checks, operations, plans, and typed diagnostics.
- [x] [parallel] Encode Crunch-to-Mantle as an example/fixture session.
  - Completed: added built-in `crunch-to-mantle-project-identity` session recording `mantle` canonical identity, `crunch` compatibility aliases, canonical/legacy files, and `/mantle/store` vs `/crunch/store` policy.
- [x] [parallel] Add typed diagnostics for conflicting canonical and legacy surfaces.
  - Completed: planning emits `mixed-file-conflict` and `ambiguous-store-prefix-conflict` diagnostics with remediation.

## Phase 2: Plan/apply/check workflow

- [x] [depends:Phase 1] Implement no-mutate plan/check behavior for a session.
  - Completed: added `mantle refactor plan` and `mantle refactor check` for explicit sessions; `check` returns non-zero on conflicts.
- [x] [parallel] Add dry-run tests proving project files and store state are not mutated.
  - Completed: tests snapshot project files before/after planning.
- [x] [parallel] Add negative tests for conflicting `mantle`/`crunch` files and ambiguous store prefixes.
  - Completed: added mixed-file and ambiguous-prefix tests.
- [x] [depends:Phase 2] Implement explicit apply for the first bounded file/path migration, or record apply as a follow-up if only planning lands.
  - Completed: `mantle refactor apply crunch-to-mantle-project-identity` explicitly applies supported legacy-to-canonical file/directory renames and refuses plan-only prefix changes.
- [x] [depends:Phase 2] Document how future migrations should add session records instead of ad hoc text rewrites.
  - Completed: documented structured refactor sessions in `docs/operator-workflows.md`.
