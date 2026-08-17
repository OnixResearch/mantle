# Tasks

## Contract

- [x] [serial] Define the representative workspace compatibility class, fixture shape, positive rail expectations, negative blocker classes, and allowed evidence claims. r[rust_package_planning.compatibility_workspace_rail]
  - Evidence: `tests/rust_compatibility_rail.rs` defines the generated fixture shape, offline Cargo smoke expectation, rust-plan bounded receipt assertion, and docs non-claim assertions.
- [x] [serial] Add example/gallery requirements so user-facing docs classify the rail as representative evidence rather than full Cargo compatibility. r[examples.rust_project_compatibility_gallery]
  - Evidence: `cairn/changes/rust-compatibility-workspace-rail/specs/examples/spec.md` plus `examples/catalog.ncl` and `examples/README.md` classify the rail as representative lane-scoped evidence.

## Implementation

- [x] [serial] Add or generate the representative Rust workspace fixture with binary, local library, vendored registry dependency, proc macro, build script, and bounded metadata. r[rust_package_planning.compatibility_workspace_rail]
  - Evidence: pueue task 37 passed `representative_fixture_contains_required_practical_rust_surfaces`.
- [x] [serial] Wire the fixture into the offline Cargo project build lane with no external network, ambient Cargo cache, or target-directory fallback. r[rust_package_planning.compatibility_workspace_rail]
  - Evidence: pueue task 37 passed `representative_offline_cargo_rail_runs_sandbox_smoke_without_network_inputs` with bubblewrap on PATH.
- [x] [serial] Wire the fixture into explicit rust-plan topology execution and preserve bounded success or deterministic blocker receipts. r[rust_package_planning.compatibility_workspace_rail]
  - Evidence: pueue task 37 passed `representative_rust_plan_receipt_is_bounded_success_or_blocker_without_cargo_fallback` using a failing Cargo shim and `--no-cargo-oracle`.
- [x] [serial] Add example catalog/docs entries that state prerequisites, command, expected output, support tier, and non-claims. r[examples.rust_project_compatibility_gallery]
  - Evidence: `examples/rust_compatibility_rail.rs`, `examples/catalog.ncl`, `examples/README.md`, `README.md`, and `docs/operator-workflows.md` now describe support tier, validation command, expected lane evidence, and non-claims.

## Verification

- [x] [serial] Add a positive offline Cargo integration test that builds the representative workspace and runs the binary smoke. r[rust_package_planning.compatibility_workspace_rail]
  - Evidence: pueue task 37 passed the representative offline Cargo sandbox smoke and asserted stdout `representative-rail-ok`.
- [x] [serial] Add a rust-plan receipt test that asserts bounded success or deterministic unsupported-surface blocker for the same workspace. r[rust_package_planning.compatibility_workspace_rail]
  - Evidence: pueue task 37 passed the rust-plan no-Cargo-oracle receipt assertion.
- [x] [serial] Add negative tests for missing vendor source, stale source digest, malformed build-script metadata, missing proc-macro host artifact, and unsupported native link metadata. r[rust_package_planning.compatibility_workspace_rail]
  - Evidence: pueue task 64 passed `representative_negative_fixtures_report_stable_blockers`, which asserts stable blockers for all five negative surfaces.
- [x] [serial] Add docs/status claim tests so passing one rail cannot be reported as full Cargo compatibility, release reproducibility, or bootstrap correctness. r[examples.rust_project_compatibility_gallery]
  - Evidence: pueue task 37 passed `representative_docs_and_gallery_do_not_overclaim_compatibility`.
- [x] [serial] Run focused Rust project rail tests, rust-plan tests, example catalog checks, and Cairn validation. r[rust_package_planning.compatibility_workspace_rail]
  - Evidence: pueue task 37 passed the rail tests and pueue task 35 passed `examples_catalog_covers_checked_in_user_facing_examples`; Cairn validation follows this task update.
