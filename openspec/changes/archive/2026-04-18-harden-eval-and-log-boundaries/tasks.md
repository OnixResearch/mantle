# Tasks: harden eval and log boundaries

## Phase 1: Spec and boundary audit

- [x] Add delta specs for `nickel-eval` and `operator-diagnostics` covering
      panic-free lazy-eval boundaries and explicit diagnostic persistence
      failures
- [x] Audit the current user-reachable panic sites in
      `crates/crunch-eval/src/{lib,session}.rs` and the silent log-write drops
      in `src/build_cmd.rs` / `src/build_log.rs`, then map each site to the
      new boundary rules

## Phase 2: Implementation

- [x] Replace user-reachable `expect(...)` / `unwrap(...)` shape checks in the
      lazy session and root-extraction paths with typed `crunch-eval::Error`
      returns, adding or refining dedicated boundary/error variants if the
      current variants are too vague
- [x] Add targeted malformed-shape regression tests for the hardened lazy-eval
      paths
- [x] Surface build-log persistence failures explicitly in human output and the
      `--json` build report, and ensure saved-log paths are emitted only when
      the file exists
- [x] Add targeted regression coverage for unwritable or failing log
      persistence paths

## Phase 3: Verification

- [x] Run the targeted lazy-eval regression tests and keep the transcript
- [x] Run the targeted log-persistence regression tests, keep the transcript,
      and verify that no fake saved-log path is emitted when persistence fails
- [x] Run `cargo test --workspace --lib --tests` under the documented build
      environment (using a disk-backed `TMPDIR` / target dir if `/tmp` is low)
      and keep the transcript

## Validation

- [x] Run `openspec validate harden-eval-and-log-boundaries`
- [x] Run proposal, design, and tasks gates for
      `harden-eval-and-log-boundaries`
