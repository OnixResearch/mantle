# Tasks: harden eval and log boundaries

## Phase 1: Spec and boundary audit

- [ ] Add delta specs for `nickel-eval` and `operator-diagnostics` covering
      panic-free lazy-eval boundaries and explicit diagnostic persistence
      failures
- [ ] Audit the current user-reachable panic sites in
      `crates/crunch-eval/src/{lib,session}.rs` and the silent log-write drops
      in `src/build_cmd.rs`, then map each site to the new boundary rules

## Phase 2: Implementation

- [ ] Replace user-reachable `expect(...)` / `unwrap(...)` shape checks in the
      lazy session and root-extraction paths with typed `crunch-eval::Error`
      returns
- [ ] Add targeted malformed-shape regression tests for the hardened lazy-eval
      paths
- [ ] Surface build-log persistence failures explicitly in operator-facing
      output and ensure saved-log paths are emitted only when the file exists
- [ ] Add targeted regression coverage for unwritable or failing log
      persistence paths

## Phase 3: Verification

- [ ] Run the targeted lazy-eval regression tests and keep the transcript
- [ ] Run the targeted log-persistence regression tests, keep the transcript,
      and verify that no fake saved-log path is emitted when persistence fails
- [ ] Run `cargo test --workspace --lib --tests` under the documented build
      environment and keep the transcript

## Validation

- [ ] Run `openspec validate harden-eval-and-log-boundaries`
- [ ] Run proposal, design, and tasks gates for
      `harden-eval-and-log-boundaries`
