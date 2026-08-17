# Tasks

## Contract

- [x] [serial] Define the rust-plan verification lane boundary, default project-build separation, machine-readable claim classes, and promotion criteria. r[rust_package_planning.verification_lane_boundary]
  - Evidence: change spec defines explicit rust-plan-only execution, promotion criteria, and the `cargo-oracle-evidence` / `cargo-free-bounded-topology` / `blocked-unsupported-surface` / `not-default-project-build` receipt vocabulary.
- [x] [serial] Define non-claim wording for full Cargo compatibility, hidden Cargo fallback, compiler correctness, release reproducibility, and bootstrap correctness. r[rust_package_planning.verification_lane_boundary]
  - Evidence: `src/rust_plan.rs` now emits shared non-claims for full Cargo compatibility, compiler correctness, release reproducibility, and bootstrap correctness; docs repeat the same boundaries.

## Implementation

- [x] [serial] Add or normalize rust-plan receipt fields that distinguish Cargo oracle evidence, Cargo-free bounded topology evidence, unsupported blockers, and not-default-project-build status. r[rust_package_planning.verification_lane_boundary]
  - Evidence: `RustPlanCargoModeSummary` now carries `compatibility_class` values for Cargo oracle, Cargo-free bounded topology, and blocked unsupported surfaces, plus `project_build_status = "not-default-project-build"`.
- [x] [serial] Ensure project build commands select the offline Cargo lane unless the operator explicitly invokes rust-plan or a future native-planner opt-in. r[rust_package_planning.verification_lane_boundary]
  - Evidence: `tests::build_cli_project_selector_is_not_implicit_rust_plan` asserts `mantle build .#app` parses as `Command::Build`, not `Command::RustPlan`; current project builds remain on project-output extraction and do not call native rust-plan implicitly.
- [x] [serial] Preserve fail-closed behavior for unsupported native planner surfaces without hidden Cargo orchestration fallback. r[rust_package_planning.verification_lane_boundary]
  - Evidence: `rust_plan_cli_no_cargo_oracle_blocks_unsupported_surface_without_cargo_fallback` passes a failing Cargo shim, asserts the marker is absent, and receives `blocked-unsupported-surface` plus a native unsupported-feature blocker.
- [x] [serial] Update README/operator docs to present rust-plan as the verification lane and offline Cargo as the near-term project build lane. r[rust_package_planning.verification_lane_boundary]
  - Evidence: README and `docs/operator-workflows.md` describe `mantle rust-plan` as an explicit bounded Rust planner verification lane and list the non-claims.

## Verification

- [x] [serial] Add positive tests proving explicit `mantle rust-plan --execute-topology` still emits bounded receipts for supported fixtures. r[rust_package_planning.verification_lane_boundary]
  - Evidence: pueue task 67 ran `rust_plan_cli_no_cargo_oracle_executes_path_workspace_without_invoking_cargo`, which asserts `cargo-free-bounded-topology`, empty blockers, no Cargo shim invocation, and topology success.
- [x] [serial] Add negative tests proving unsupported rust-plan surfaces produce blockers and do not invoke Cargo as hidden orchestration. r[rust_package_planning.verification_lane_boundary]
  - Evidence: pueue task 67 ran `rust_plan_cli_no_cargo_oracle_blocks_unsupported_surface_without_cargo_fallback`, which asserts `blocked-unsupported-surface`, `unsupported-feature-surface`, and no Cargo shim marker.
- [x] [serial] Add project-build tests proving ordinary Rust package builds do not silently use native rust-plan execution. r[rust_package_planning.verification_lane_boundary]
  - Evidence: pueue task 67 ran `cargo test -p mantle --bin mantle build_cli_project_selector_is_not_implicit_rust_plan -- --nocapture` successfully.
- [x] [serial] Add documentation/claim checks that reject broad Cargo-compatibility wording without current evidence. r[rust_package_planning.verification_lane_boundary]
  - Evidence: README and operator docs state that rust-plan evidence is not proof of full Cargo compatibility, compiler correctness, release reproducibility, or bootstrap correctness.
- [x] [serial] Run focused rust-plan tests, project workflow tests, docs checks, and Cairn validation. r[rust_package_planning.verification_lane_boundary]
  - Evidence: baseline pueue task 38 passed the pre-change no-Cargo rust-plan test after setting `SNIX_BUILD_SANDBOX_SHELL=/bin/sh`; pueue task 67 passed the post-change focused rust-plan/project selector bundle. Cairn validation is run after task completion before sync/archive.
