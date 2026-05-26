# Tasks: Native Rust registry target-cfg topology planning

- [x] [serial] Add native target-cfg dependency fact planning for `r[rust_package_planning.native_registry_target_cfg_topology_planning]`.
  - Parse bounded supported `target.'cfg(...)'.dependencies` tables from native manifests.
  - Evaluate supported cfg predicates against an explicit active Rust target triple.
  - Record deterministic selected/not-selected/blocker evidence in receipts.

- [x] [serial] Connect supported target-cfg registry graphs to topology evidence for `r[rust_package_planning.native_registry_target_cfg_topology_planning]`.
  - Require ready native registry source facts for all selected target-cfg registry packages.
  - Ensure unselected target-cfg registry packages do not enter topology execution.
  - Bind selected target-cfg dependency artifacts through explicit topology receipts.

- [x] [serial] Add CLI coverage for `r[rust_package_planning.native_registry_target_cfg_topology_planning]`.
  - Add a positive vendored registry fixture with a supported target-cfg dependency edge.
  - Add a negative fixture for unsupported cfg/platform resolver behavior.
  - Assert no Cargo orchestration, network, `$CARGO_HOME`, registry cache, or version-solving fallback claim.

- [x] [serial] Verify and archive `r[rust_package_planning.native_registry_target_cfg_topology_planning]`.
  - Run focused Rust package-planning and CLI tests.
  - Run Cairn validate plus proposal/design/tasks gates.
  - Sync/archive the change.
  - Verify accepted spec contains the target-cfg requirement after archive and repair if needed.
