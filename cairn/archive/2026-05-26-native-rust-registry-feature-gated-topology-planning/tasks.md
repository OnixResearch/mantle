# Tasks: Native Rust registry feature-gated topology planning

- [x] [serial] Add native feature fact planning for `r[rust_package_planning.native_registry_feature_gated_topology_planning]`.
  - Parse a bounded supported feature shape from native manifests rather than Cargo resolver output.
  - Record selected feature facts, optional dependency activation, and deterministic blockers in receipt evidence.
  - Preserve existing fail-closed behavior for unsupported feature surfaces.

- [x] [serial] Connect supported feature-selected registry graphs to topology evidence for `r[rust_package_planning.native_registry_feature_gated_topology_planning]`.
  - Require ready native registry source facts for all feature-selected registry packages.
  - Ensure unselected optional dependencies do not enter the executed topology.
  - Bind selected optional dependency artifacts through explicit topology receipts.

- [x] [serial] Add CLI coverage for `r[rust_package_planning.native_registry_feature_gated_topology_planning]`.
  - Add a positive vendored registry fixture with an explicitly selected feature that activates an optional registry dependency.
  - Add a negative fixture for unsupported or resolver-dependent feature behavior.
  - Assert no Cargo orchestration, network, `$CARGO_HOME`, registry cache, or version-solving fallback claim.

- [x] [serial] Verify and archive `r[rust_package_planning.native_registry_feature_gated_topology_planning]`.
  - Run focused Rust package-planning and CLI tests.
  - Run Cairn validate plus proposal/design/tasks gates.
  - Sync/archive the change.
  - Verify the accepted spec contains the new requirement after archive; repair from archived delta if Cairn sync omits it.
