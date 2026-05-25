# Tasks

- [ ] [serial] Define the bounded registry-backed topology execution requirement and non-goals. r[rust_package_planning.native_registry_topology_execution]
- [ ] [serial] Require ready native registry source facts before registry-backed packages enter native unit/topology execution. r[rust_package_planning.native_registry_topology_execution.source_facts]
- [ ] [serial] Bind registry source identity, checksum, vendor root, source digest, and produced artifact digests into execution receipts. r[rust_package_planning.native_registry_topology_execution.receipts]
- [ ] [serial] Fail closed before rustc for missing, stale, unsupported, or ambient-cache-dependent registry source material. r[rust_package_planning.native_registry_topology_execution.blockers]
- [ ] [serial] Add positive and negative CLI fixtures for vendored registry-backed topology execution. r[rust_package_planning.native_registry_topology_execution.tests]
- [ ] [serial] Run focused Rust verification, Cairn validation, and proposal/design/tasks gates before sync/archive/commit/push. r[rust_package_planning.native_registry_topology_execution.verify]
