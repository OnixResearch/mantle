## Phase 1: Pure build-override resolver

- [x] [serial] Add a pure `build_override_env` resolver implementing Cargo's built-in build-override defaults (`OPT_LEVEL=0`, `DEBUG=false`, `NUM_JOBS=1`) for all profiles. r[rust_package_planning.profile_build_override_defaults]
- [x] [parallel] Add positive tests for dev, test, release, and bench build-override env and negative tests that reject the old `OPT_LEVEL=3` release and `DEBUG=true` dev behavior. r[rust_package_planning.profile_build_override_defaults]

## Phase 2: Scope and dual-use wiring

- [x] [serial] Route `append_build_script_profile_env`, proc-macro host units, and host-dependency units through the resolver. r[rust_package_planning.profile_build_override_scope]
- [x] [serial] Mark dual-use packages from the selected unit graph and apply the target profile debug value with a receipt record. r[rust_package_planning.profile_build_override_scope]
- [x] [parallel] Add a dual-use package test and a negative test that an unrecorded dual-use env fails validation. r[rust_package_planning.profile_build_override_scope]

## Phase 3: Verification

- [x] [serial] Run focused `cargo test` for build-script env parity, then rerun one affected topology execution and record the changed build-script receipt values as intended. r[rust_package_planning.profile_build_override_defaults]
