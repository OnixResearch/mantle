# Tasks

## Spec

- [x] [serial] Add build-script runtime parity requirement and design. r[rust_package_planning.native_build_script_runtime]

## Implementation

- [x] [serial] Centralize build-script environment derivation from native facts. r[rust_package_planning.native_build_script_runtime]
  - Evidence: `build_script_child_env_*`, target-cfg, profile-env, and manifest-package tests cover native package/profile/target/manifest facts plus ambient-env rejection. Verified in pueue task 474.
- [x] [serial] Complete typed parser for supported Cargo metadata lines. r[rust_package_planning.native_build_script_runtime]
  - Evidence: `parse_build_script_metadata_*` tests cover typed cfg/env/link/rerun/links parsing plus malformed custom keys and unsafe link-lib forms. Verified in pueue task 474.
- [x] [serial] Thread typed metadata into dependent rustc args/env/link search. r[rust_package_planning.native_build_script_runtime]
  - Evidence: `bind_build_script_metadata_*` tests and CLI topology tests prove `DEP_*` env, `rustc-link-lib`, and `rustc-link-search` reach dependent rustc invocations. Fixed target dependency planning so custom-build/proc-macro host edges do not become normal target dependency artifacts; verified by `unit_dependency_artifacts_skip_host_producer_edges` and link-metadata CLI coverage in pueue task 474.
- [x] [serial] Add deterministic blockers for unsupported host probing and malformed metadata. r[rust_package_planning.native_build_script_runtime]
  - Evidence: ambiguous build-script metadata producer tests and `rust_plan_cli_blocks_malformed_build_script_metadata` produce deterministic pre-dependent blockers. Verified in pueue task 474.
- [x] [serial] Add fixtures for cc-rs-like, links metadata, and malformed metadata cases. r[rust_package_planning.native_build_script_runtime]
  - Evidence: CLI fixtures in `tests/rust_plan_cli.rs` exercise build-script metadata topology, native link metadata, and malformed metadata paths; unit fixtures exercise linked dependency metadata binding. Verified in pueue task 474.

## Verification

- [x] [serial] Run build-script env/metadata tests, topology self-probe, and Cairn validation. r[rust_package_planning.native_build_script_runtime]
  - Evidence: focused build-script runtime tests passed in pueue task 474. Full topology self-probe pueue task 475 wrote `target/mantle-self-rust-plan-probe-build-script-runtime/receipt.json` and blocked before executor work at `native-package-target-planning-blocked` / `native-host-unit-graph-blocked`, outside this runtime parity slice. Cairn validation and formatting run after this task update.
