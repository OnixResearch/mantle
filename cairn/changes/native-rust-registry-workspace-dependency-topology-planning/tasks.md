# Tasks: Native Rust registry workspace dependency topology planning

- [ ] [serial] Add native workspace dependency inheritance facts for `r[rust_package_planning.native_registry_workspace_dependency_topology_planning]`.
  - Parse bounded root `[workspace.dependencies]` entries.
  - Resolve member `{ workspace = true }` dependency declarations only when the inherited entry is explicit and supported.
  - Record deterministic selected/blocker evidence in native receipts.

- [ ] [serial] Connect supported inherited registry edges to topology evidence for `r[rust_package_planning.native_registry_workspace_dependency_topology_planning]`.
  - Require ready native registry source facts for inherited vendored-registry packages.
  - Feed selected inherited dependencies into native package dependency facts.
  - Bind inherited dependency artifacts through existing topology receipts.

- [ ] [serial] Add CLI coverage for `r[rust_package_planning.native_registry_workspace_dependency_topology_planning]`.
  - Add a positive vendored registry workspace fixture with `[workspace.dependencies]` and member `{ workspace = true }`.
  - Add a negative fixture for missing or unsupported workspace inherited dependency behavior.
  - Assert no Cargo orchestration, network, `$CARGO_HOME`, registry cache, or version-solving fallback claim.

- [ ] [serial] Verify and archive `r[rust_package_planning.native_registry_workspace_dependency_topology_planning]`.
  - Run focused Rust package-planning and CLI tests.
  - Run Cairn validate plus proposal/design/tasks gates.
  - Sync/archive the change and verify the accepted spec contains the requirement.
