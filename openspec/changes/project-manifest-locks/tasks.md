## Phase 1: Project crate scaffold

- [x] Add `crates/crunch-project` to the workspace with manifest, lock, and upgrade modules ✅ 1h 5m (started: 2026-04-07T09:46Z -> completed: 2026-04-07T09:51Z)
- [x] Add pure types for project inputs, lock entries, patches, mirrors, and schema versions ✅ (done with crate scaffold)
- [x] Add serialization tests for `crunch.lock` ✅ 11 tests in lock.rs
- [x] Add merge/validation tests for manifest + lock ✅ 11 tests in merge.rs

## Phase 2: Nickel manifest and generated inputs

- [x] Define the `crunch-project.ncl` schema and validation rules ✅ lib/project.ncl with contracts
- [x] Implement manifest loading through Nickel evaluation ✅ evaluate_and_deserialize into ProjectManifest
- [x] Implement `.crunch/inputs.ncl` generation from the resolved lock ✅ generate.rs
- [x] Add tests that generated inputs import cleanly from package Nickel code ✅ 5 integration tests
- [x] Add drift detection: `crunch check` fails when the generated inputs file does not match the current lock ✅ drift.rs + 4 tests

## Phase 3: Refresh and stale detection

- [x] Implement `refresh` for selected or all inputs ✅ refresh_inputs() with RefreshResolver trait
- [x] Implement `list-stale` without mutating files ✅ list_stale() in refresh.rs
- [x] Implement `frozen` input handling ✅ frozen inputs skipped in refresh_one()
- [x] Implement kind-specific refresh helpers for URL, tarball, and git inputs ✅ resolve_input() handles File/Tarball/Git
- [x] Add integration tests covering unchanged, updated, and frozen inputs ✅ 8 tests in refresh.rs

## Phase 4: CLI integration

- [x] Add `crunch init` to scaffold a project manifest, lockfile, and `.crunch/` ignore rules ✅ cmd_init()
- [x] Add `crunch check` to validate manifest, lock, and generated inputs ✅ cmd_check()
- [x] Add `crunch show` to render the resolved input state ✅ cmd_show()
- [x] Add `crunch refresh`, `crunch list-stale`, and `crunch upgrade` ✅ cmd_refresh(), cmd_list_stale(), cmd_upgrade()
- [x] Keep the binary crate limited to argument parsing, output formatting, and delegation into `crunch-project` ✅ project_cmd.rs delegates to crunch-project

## Phase 5: Mirrors, patches, and upgrades

- [ ] Add mirrors to the manifest and lock models
- [ ] Add patch definitions and per-input patch lists to the manifest and lock models
- [ ] Connect locked mirrors and patches to the existing fetch/build pipeline
- [ ] Implement manifest and lock schema migrations in `crunch-project`
- [ ] Add end-to-end tests covering upgrades from at least one older schema version
