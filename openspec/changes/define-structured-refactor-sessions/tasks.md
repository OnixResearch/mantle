## Phase 1: Session schema

- [ ] [serial] Define structured refactor session schema for aliases, paths, CLI names, store prefixes, schema versions, and compatibility policy.
- [ ] [parallel] Encode Crunch-to-Mantle as an example/fixture session.
- [ ] [parallel] Add typed diagnostics for conflicting canonical and legacy surfaces.

## Phase 2: Plan/apply/check workflow

- [ ] [depends:Phase 1] Implement no-mutate plan/check behavior for a session.
- [ ] [parallel] Add dry-run tests proving project files and store state are not mutated.
- [ ] [parallel] Add negative tests for conflicting `mantle`/`crunch` files and ambiguous store prefixes.
- [ ] [depends:Phase 2] Implement explicit apply for the first bounded file/path migration, or record apply as a follow-up if only planning lands.
- [ ] [depends:Phase 2] Document how future migrations should add session records instead of ad hoc text rewrites.
