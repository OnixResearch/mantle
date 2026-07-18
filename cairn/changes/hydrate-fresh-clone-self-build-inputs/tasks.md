# Tasks: Hydrate fresh-clone self-build inputs

## Phase 1: Lifecycle and baseline

- [x] [serial] P1 Record the prepared-checkout boundary, ignored vendor size, candidate designs, selected source-bundle handoff, and explicit non-claims. r[bootstrap_inventory.fresh_clone_source_hydration]
  - Evidence: proposal, design, and `evidence/baseline.md`; current `vendor-deps/` measured 810.82 MiB and `.gitignore` excludes it.
- [x] [serial] V1 Capture the pre-change focused source-bundle test baseline. r[bootstrap_inventory.fresh_clone_source_hydration]
  - Evidence: pueue task `53` passed `40` focused source-bundle tests with zero failures.

## Phase 2: Implementation

- [x] [serial] I1 Add the pure hydration-plan validator for expected manifest identity and required bootstrap profile record classes. r[bootstrap_inventory.fresh_clone_source_hydration]
  - Evidence: `plan_self_build_hydration` validates the external digest and unique linked vendor/provider classes before returning indexes; focused positive and negative tests passed in task `265`.
- [x] [serial] I2 Add the Linux no-clobber hydration shell, pre-publication Cargo lock/checksum validation, provider-state import/pinning, rollback, and machine report. r[bootstrap_inventory.fresh_clone_source_hydration]
  - Evidence: real hydration report and identities are preserved in `evidence/fresh-clone-summary.json`; Linux no-replace, checksum drift, rollback, read-only provider, and override-lifetime tests passed.
- [x] [serial] I3 Expose the public CLI and document the transfer, expected-digest, empty-cache, offline-provider, and fixed-point non-claim boundaries in ADR 0031 and operator docs. r[bootstrap_inventory.fresh_clone_source_hydration]
  - Evidence: public CLI tests passed; ADR 0031, README, operator workflow, machine-contract docs, schema, fixtures, and AGENTS notes describe the bounded workflow and non-claims.

## Phase 3: Adversarial validation

- [x] [serial] V2 Add positive fresh-clone hydration coverage and negative wrong-digest, tampered-bundle, missing-record, existing-destination, invalid-vendor, and state-persistence coverage. r[bootstrap_inventory.fresh_clone_source_hydration]
  - Evidence: task `265` passed 55 source-bundle tests plus 2 public hydration CLI tests; `evidence/validation.md` enumerates the negative cases and real-payload corrections.
- [x] [serial] V3 Run focused unit/CLI tests, locked offline Cargo metadata with an empty `CARGO_HOME`, and the offline legacy-provider preflight rail. r[bootstrap_inventory.fresh_clone_source_hydration]
  - Evidence: empty-`CARGO_HOME` locked metadata exited zero with `CARGO_NET_OFFLINE=true`; final provider task `307` reported source state `b76723aa...e98ad0` and built the reduced provider without fallback.
- [x] [serial] V4 Run Rustfmt, focused Clippy, Tiger Style, machine-contract/docs checks, dependency policy, first-party quality, and Nix evaluation. r[bootstrap_inventory.fresh_clone_source_hydration]
  - Evidence: tasks `283`, `287`, `291`, `292`, and `300` passed; machine contracts reported `19 contracted, 48 classified`, both root suites reported `1570 passed; 0 failed`, and Nix reported `all checks passed!`.
- [ ] [serial] V5 Run Cairn validation/gates, sync and inspect the accepted requirement, add evidence-backed Tracey links, archive exact post-state evidence, commit, and push. r[bootstrap_inventory.fresh_clone_source_hydration]
