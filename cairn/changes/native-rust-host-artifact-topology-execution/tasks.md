# Tasks

- [ ] [serial] r[rust_package_planning.native_host_artifact_topology_execution] Confirm the existing host-artifact topology rail and receipt shape, then identify the minimal integration points where ready native host-unit graph facts must gate supported execution.
- [ ] [serial] r[rust_package_planning.native_host_artifact_topology_execution.executes] Execute supported native-planned `custom-build` and `proc-macro` host units before target consumers without invoking Cargo as orchestrator.
- [ ] [serial] r[rust_package_planning.native_host_artifact_topology_execution.binds] Rebind produced host artifacts and supported build-script metadata into target `consumed_host_artifacts`, derivation inputs, env, and `rustc` args before target execution.
- [ ] [serial] r[rust_package_planning.native_host_artifact_topology_execution.blockers] Add deterministic fail-closed blockers for missing, stale, unsupported, or ambiguous native host-artifact topology material.
- [ ] [serial] r[rust_package_planning.native_host_artifact_topology_execution.metadata] Add focused positive/negative `rust_plan` CLI tests for native-planned proc-macro and custom-build host artifacts plus metadata/blocker behavior, then run focused Rust verification.
- [ ] [serial] r[rust_package_planning.native_host_artifact_topology_execution.bounded_claim] Run Cairn validate/gates, sync/archive the completed change, commit, push, and verify clean/synced state.
