## Phase 1: Rust Input Variant

- [x] Add `OutputRef` struct and `Input::OutputSelection` variant to `types.rs` ✅ 5m (started: 2026-04-05T13:04Z → completed: 2026-04-05T13:05Z)
- [x] Add `resolve_inputs` handling for `OutputSelection`: recursive convert + single-output insert + coalescing via `entry().or_default().insert()` ✅ 3m (started: 2026-04-05T13:04Z → completed: 2026-04-05T13:05Z)
- [x] Add assertion: selected output name must exist in `drv.outputs`, fail with descriptive error ✅ 2m (started: 2026-04-05T13:04Z → completed: 2026-04-05T13:05Z)
- [x] Unit tests: deserialize `OutputSelection` from JSON, verify serde ordering (OutputSelection before Derivation) ✅ 6m (started: 2026-04-05T13:05Z → completed: 2026-04-05T13:06Z)
- [x] Unit tests: `resolve_inputs` with single selection, coalescing, invalid output name ✅ 6m (started: 2026-04-05T13:05Z → completed: 2026-04-05T13:06Z)

## Phase 2: Nickel Contract + Helper

- [x] Extend `Input` contract in `contracts.ncl` to accept `{ drv, output }` records ✅ 5m (started: 2026-04-05T13:07Z → completed: 2026-04-05T13:08Z)
- [x] Add `select` function to `lib/lib.ncl`: `select : Derivation -> String -> { drv, output }` ✅ 5m (started: 2026-04-05T13:07Z → completed: 2026-04-05T13:08Z)
- [x] Stdlib test: `select` produces correct structure, mixed inputs array validates ✅ 5m (started: 2026-04-05T13:07Z → completed: 2026-04-05T13:08Z)

## Phase 3: Integration Test

- [x] Integration test: build a package that uses only `dev` output of a multi-output dep, verify only that output is in the sandbox ✅ 3m (started: 2026-04-05T13:09Z → completed: 2026-04-05T13:11Z)
- [x] Integration test: coalescing — same dep selected for `dev` and `lib`, verify both mounted ✅ 3m (started: 2026-04-05T13:09Z → completed: 2026-04-05T13:11Z)
