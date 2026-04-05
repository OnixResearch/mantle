## Phase 1: Rust Input Variant

- [ ] Add `OutputRef` struct and `Input::OutputSelection` variant to `types.rs`
- [ ] Add `resolve_inputs` handling for `OutputSelection`: recursive convert + single-output insert + coalescing via `entry().or_default().insert()`
- [ ] Add assertion: selected output name must exist in `drv.outputs`, fail with descriptive error
- [ ] Unit tests: deserialize `OutputSelection` from JSON, verify serde ordering (OutputSelection before Derivation)
- [ ] Unit tests: `resolve_inputs` with single selection, coalescing, invalid output name

## Phase 2: Nickel Contract + Helper

- [ ] Extend `Input` contract in `contracts.ncl` to accept `{ drv, output }` records
- [ ] Add `select` function to `lib/lib.ncl`: `select : Derivation -> String -> { drv, output }`
- [ ] Stdlib test: `select` produces correct structure, mixed inputs array validates

## Phase 3: Integration Test

- [ ] Integration test: build a package that uses only `dev` output of a multi-output dep, verify only that output is in the sandbox
- [ ] Integration test: coalescing — same dep selected for `dev` and `lib`, verify both mounted
