## Phase 1: Implementation

- [x] [serial] Inventory current GCC 4.0 bridge/stub surfaces and artifact-shape milestones. ✅ 1m (started: 2026-05-12T03:19:05Z → completed: 2026-05-12T03:19:43Z)
  Evidence: added `gcc40-correctness-roadmap.md` inventory covering bridge driver, cc1/xgcc/cpp stubs, generator scripts, frontend/backend object bridges, CRT/libgcc artifact shape, and caveats.
- [x] [serial] Write a phased correctness roadmap covering libgcc, drivers, cc1/generators, and smoke evidence. ✅ 1m (started: 2026-05-12T03:20:04Z → completed: 2026-05-12T03:20:56Z)
  Evidence: roadmap now defines independent promotion phases for first semantic `libgcc.a` member, driver/preprocessor behavior, `cc1`/generator replacement, and CRT/link/runtime smokes.
- [ ] [serial] Validate the change with `openspec validate plan-gcc40-correctness-promotion --strict`.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate plan-gcc40-correctness-promotion --strict` and record evidence before archive.
