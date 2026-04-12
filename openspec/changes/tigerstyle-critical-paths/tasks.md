## Phase 1: Narrow shell entrypoints

- [ ] Split `src/main.rs::run` into focused command handlers so top-level CLI dispatch stays a thin shell.
- [ ] Split `src/self_build.rs::cmd_self_build` into explicit stage helpers for staging, bootstrap tools, crunch build, and verification.
- [ ] Add regression tests or focused assertions that preserve current CLI and self-build behavior during the refactor.

## Phase 2: Separate CA planning from mutation

- [ ] Extract pure planning helpers from `crates/crunch-build/src/orchestrate.rs` for CA marker generation, output-path planning, and rewrite planning.
- [ ] Keep registry updates, store persistence, and logging in imperative shell helpers that consume the planning output.
- [ ] Add tests for the pure planning layer without blob, directory, or pathinfo I/O.

## Phase 3: Replace recursive traversal in critical paths

- [ ] Rewrite `crates/crunch-build/src/rewrite.rs` tree traversal to use an explicit bounded worklist.
- [ ] Rewrite `crates/crunch-store/src/export.rs` tree traversal to use an explicit bounded worklist.
- [ ] Add tests that prove deep trees fail via explicit limits rather than unbounded recursion.

## Phase 4: Raise contracts and re-audit

- [ ] Add assertions and compile-time constant checks in the refactored critical paths, especially around output counts, marker lengths, queue bounds, and required invariants.
- [ ] Re-run the Tiger Style audit and confirm the targeted hotspots no longer violate the change requirements.
- [ ] Run `openspec validate tigerstyle-critical-paths` and the relevant Rust tests before closing the change.
