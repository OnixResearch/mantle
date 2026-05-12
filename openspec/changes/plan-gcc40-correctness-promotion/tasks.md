## Phase 1: Implementation

- [x] [serial] Inventory current GCC 4.0 bridge/stub surfaces and artifact-shape milestones. ✅ 1m (started: 2026-05-12T03:19:05Z → completed: 2026-05-12T03:19:43Z)
  Evidence: added `gcc40-correctness-roadmap.md` inventory covering bridge driver, cc1/xgcc/cpp stubs, generator scripts, frontend/backend object bridges, CRT/libgcc artifact shape, and caveats.
- [ ] [serial] Write a phased correctness roadmap covering libgcc, drivers, cc1/generators, and smoke evidence.
- [ ] [serial] Validate the change with `openspec validate plan-gcc40-correctness-promotion --strict`.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate plan-gcc40-correctness-promotion --strict` and record evidence before archive.
