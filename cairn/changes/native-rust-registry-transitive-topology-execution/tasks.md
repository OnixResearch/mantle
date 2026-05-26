# Tasks: Native Rust registry transitive topology execution

- [ ] [serial] Implement positive transitive registry topology fixture and assertions for `r[rust_package_planning.native_registry_transitive_topology_execution]`.
  - Build a local app depending on vendored registry crate A, where crate A depends on vendored registry crate B.
  - Assert ready source facts exist for both registry crates.
  - Assert topology execution order is registry B, registry A, local app.
  - Assert dependency artifact digests are bound from B into A and from A into the app.

- [ ] [serial] Implement fail-closed transitive registry blocker coverage for `r[rust_package_planning.native_registry_transitive_topology_execution]`.
  - Add a negative fixture with missing, stale, or unsupported transitive vendored registry material.
  - Assert blockers surface before downstream `rustc` execution.
  - Assert there is no network, `$CARGO_HOME`, registry cache, or Cargo orchestration fallback claim.

- [ ] [serial] Verify and archive `r[rust_package_planning.native_registry_transitive_topology_execution]`.
  - Run focused Rust package-planning tests and CLI tests.
  - Run Cairn validate plus proposal/design/tasks gates.
  - Sync/archive the change.
  - Verify the accepted spec contains the new requirement after archive; repair from archived delta if Cairn sync omits it.
