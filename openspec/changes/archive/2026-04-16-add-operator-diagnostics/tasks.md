# Tasks: Add operator diagnostics

## Phase 1: Preflight and plan surface

- [x] Add `crunch doctor` and its report type for no-mutate host and runtime
      preflight checks, explicitly covering nightly toolchain visibility,
      `bwrap`, sandbox shell availability, writable state or store paths, and
      FUSE or `fusermount3` availability for workflows that depend on it as
      part of the initial workflow profiles.
- [x] Define how `crunch doctor` selects or defaults the workflow profile and
      report that selected profile in success and failure output.
- [x] Verify CLI parsing and dispatch for explicit `crunch doctor` profile
      selection, and confirm the documented default profile is used when no
      explicit selection is supplied.
- [x] Add `crunch build --plan` plan-mode plumbing that reports per-root
      planned action labels (`cached`, `substitute`, `build`, `preflight-error`)
      and stops before substitution downloads, build dispatch, or store
      mutation.
- [x] Verify CLI parsing and dispatch for `crunch build --plan`, and confirm
      normal build semantics remain unchanged when `--plan` is absent.
- [x] Add a dedicated `tests/operator_diagnostics.rs` target for doctor, plan,
      JSON failure schema, and human-readable failure reporting coverage.
- [x] Document available doctor profiles, the default profile when none is
      supplied, and which preflight checks each profile covers.
- [x] Document which planned action labels crunch will report for each root.
- [x] Document the intended operator troubleshooting workflow.

## Phase 2: Structured failure reporting

- [x] Add a typed failure envelope carrying failing root, phase, error class,
      and saved log path when available.
- [x] Define, document, and snapshot-check the stable JSON failure schema for
      that envelope, including exact field names and omission of
      `saved_log_path` when no saved log exists.
- [x] Render that envelope in both human-readable build failures and JSON build
      reports.
- [x] Keep existing saved log behavior, but point operator-facing output at the
      structured failure summary first.

## Phase 3: Validation coverage

- [x] Add tests that `crunch doctor` reports missing prerequisites without
      starting builds or mutating store or state, exits non-zero on failure,
      identifies the checked workflow profile on success, and identifies the
      selected workflow profile in failure output.
- [x] Add tests that a successful `crunch doctor` run also starts no builds and
      mutates neither store nor state.
- [x] Add tests that workflow-profile selection or defaulting drives the
      expected prerequisite checks for doctor.
- [x] Add tests that plan mode reports cached, substitute, build, or preflight
      outcomes without starting a build, triggering substitution downloads, or
      mutating local store state.
- [x] Add tests that `crunch build --plan` parses and dispatches correctly,
      while ordinary `crunch build` behavior is unchanged when `--plan` is not
      present.
- [x] Add tests and human-readable snapshots that JSON and human failure
      reports expose the same failing root, phase, error class, and log-path
      facts, including omission of the log path when no saved log exists.

## Validation

- [x] Run `openspec validate add-operator-diagnostics`.
- [x] Run `cargo test -p crunch --test operator_diagnostics -- --nocapture`
      with the repo's documented build environment and keep the `test result:`
      lines.
