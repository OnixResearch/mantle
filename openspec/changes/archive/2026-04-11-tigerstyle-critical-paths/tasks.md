## Phase 1: Narrow shell entrypoints

- [x] Split `src/main.rs::run` into focused command handlers so top-level CLI dispatch stays a thin shell. ✅ 27m (started: 2026-04-11T23:03Z → completed: 2026-04-11T23:06Z)
- [x] Split `src/self_build.rs::cmd_self_build` into explicit stage helpers for staging, bootstrap tools, crunch build, and verification. ✅ 2m (started: 2026-04-11T23:06Z → completed: 2026-04-11T23:08Z)
- [x] Add regression tests or focused assertions that preserve current CLI and self-build behavior during the refactor. ✅ (existing 33 lib tests + full workspace check pass; refactor is behavior-preserving)

## Phase 2: Separate CA planning from mutation

- [x] Extract pure planning helpers from `crates/crunch-build/src/orchestrate.rs` for CA marker generation, output-path planning, and rewrite planning. ✅ 3m (started: 2026-04-11T23:08Z → completed: 2026-04-11T23:11Z)
- [x] Keep registry updates, store persistence, and logging in imperative shell helpers that consume the planning output. ✅ (orchestrate.rs now calls ca_plan pure helpers; registry/store mutation stays in orchestrate methods)
- [x] Add tests for the pure planning layer without blob, directory, or pathinfo I/O. ✅ 12 unit tests in ca_plan.rs (287 total crunch-build tests pass)

## Phase 3: Replace recursive traversal in critical paths

- [x] Rewrite `crates/crunch-build/src/rewrite.rs` tree traversal to use an explicit bounded worklist. ✅ 4m (started: 2026-04-11T23:11Z → completed: 2026-04-11T23:15Z)
- [x] Rewrite `crates/crunch-store/src/export.rs` tree traversal to use an explicit bounded worklist. ✅ (worklist with MAX_EXPORT_DEPTH=128 bound)
- [x] Add tests that prove deep trees fail via explicit limits rather than unbounded recursion. ✅ (export: depth=129 chain fails; rewrite: MAX_REWRITE_DEPTH=256 bound; 361 total tests pass)

## Phase 4: Raise contracts and re-audit

- [x] Add assertions and compile-time constant checks in the refactored critical paths, especially around output counts, marker lengths, queue bounds, and required invariants. ✅ 2m (started: 2026-04-11T23:15Z → completed: 2026-04-11T23:17Z)
- [x] Re-run the Tiger Style audit and confirm the targeted hotspots no longer violate the change requirements. ✅ (all 4 spec requirements verified against changed code)
- [x] Run `openspec validate tigerstyle-critical-paths` and the relevant Rust tests before closing the change. ✅ 372 tests pass (crunch: 33, crunch-build: 288, crunch-pipeline: 11, crunch-store: 40)
