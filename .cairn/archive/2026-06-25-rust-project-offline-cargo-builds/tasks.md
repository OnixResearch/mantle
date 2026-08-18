# Tasks

## Contract

- [x] [serial] Define the offline Cargo project package contract, source-closure fields, supported outputs, failure classes, and bounded receipt label. r[project_workflows.offline_cargo_builds]
  - Evidence: `src/offline_cargo.rs` defines the pure `RustOfflineCargoPackageRequest`/source/blocker plan, `lib/offline_cargo.ncl` defines the project helper contract, and pueue task 183 reran the focused validation bundle.
- [x] [serial] Document non-claims for Cargo-free execution, full Cargo compatibility, rustc correctness, release reproducibility, and bootstrap correctness. r[project_workflows.offline_cargo_builds]
  - Evidence: `README.md` and `docs/operator-workflows.md` document the `cargo-inside-mantle-sandbox` evidence class and the five non-claims.

## Implementation

- [x] [serial] Add a pure planner for `RustOfflineCargoPackage` data that validates source roots, lockfile identity, dependency source entries, selected target/profile, toolchain refs, and output contracts. r[project_workflows.offline_cargo_builds]
  - Evidence: pueue task 183 ran positive, missing-source/network/missing-binary, and stale lock/source digest planner tests.
- [x] [serial] Add the shell lowering that turns a valid offline Cargo package plan into ordinary Mantle derivations with isolated Cargo environment and `--offline --locked` behavior. r[project_workflows.offline_cargo_builds]
  - Evidence: `lib/offline_cargo.ncl` lowers to a normal derivation with isolated `HOME`, `CARGO_HOME`, `CARGO_TARGET_DIR`, generated offline Cargo config, and `cargo build --locked --offline`.
- [x] [serial] Add project selection support so `mantle build .#name` and `mantle run .#name` can build and execute the declared Rust package output. r[project_workflows.offline_cargo_builds]
  - Evidence: existing project selection is reused through `packages.<name>`; pueue task 177 ran `offline_cargo_project_run_executes_declared_sandbox_cargo_output` and verified `mantle run .#demo` output.
- [x] [serial] Emit human and JSON report fields that identify the build as Cargo-orchestrated inside Mantle's sandbox and bind source-closure/output evidence. r[project_workflows.offline_cargo_builds]
  - Evidence: `src/build_report.rs` surfaces `cargo_build_evidence[]`; pueue task 181 ran positive and malformed-sidecar report tests.

## Verification

- [x] [serial] Add a positive fixture that builds a small offline Rust package and verifies the produced binary output through `mantle run`. r[project_workflows.offline_cargo_builds]
  - Evidence: pueue task 177 passed `offline_cargo_project_run_executes_declared_sandbox_cargo_output` with bubblewrap on PATH.
- [x] [serial] Add negative fixtures for missing vendor/source material, stale lockfile/source digest, undeclared network dependency, and missing selected binary. r[project_workflows.offline_cargo_builds]
  - Evidence: pueue task 183 ran the planner negative tests, including missing source material, unsupported network policy, missing selected binary, and stale lock/source digest blockers.
- [x] [serial] Add report assertions proving the JSON output contains bounded source-closure/build-output evidence and no Cargo-free claim. r[project_workflows.offline_cargo_builds]
  - Evidence: pueue task 181 passed `build_json_report_surfaces_offline_cargo_evidence_sidecar` and `build_json_report_ignores_malformed_offline_cargo_evidence_sidecar`.
- [x] [serial] Update README/operator docs and run focused validation for project workflows, Rust build fixtures, and Cairn validation. r[project_workflows.offline_cargo_builds]
  - Evidence: docs updated in `README.md` and `docs/operator-workflows.md`; pueue task 183 passed focused Cargo/project/stdlib validation. Cairn validation follows this task update.
