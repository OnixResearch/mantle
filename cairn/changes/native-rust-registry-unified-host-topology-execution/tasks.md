# Tasks

- [ ] [serial] Define the bounded registry-backed unified host topology execution requirement and non-goals. r[rust_package_planning.native_registry_unified_host_topology_execution]
- [ ] [serial] Require ready native registry source facts and ready native host-unit graph facts before registry-backed host producers or affected targets execute on `--execute-topology`. r[rust_package_planning.native_registry_unified_host_topology_execution.source_and_host_facts]
- [ ] [serial] Execute supported vendored registry build-script/proc-macro host producers before target consumers in unified topology order. r[rust_package_planning.native_registry_unified_host_topology_execution.executes]
- [ ] [serial] Bind registry source identity, checksum, vendor root, source digest, host artifact digests, build-script metadata, consumed host material, and target consumer digests into unified topology receipts. r[rust_package_planning.native_registry_unified_host_topology_execution.receipts]
- [ ] [serial] Fail closed before host or target execution for missing, stale, unsupported, or ambient-cache-dependent registry host material. r[rust_package_planning.native_registry_unified_host_topology_execution.blockers]
- [ ] [serial] Add positive and negative CLI fixtures for vendored registry-backed build-script or proc-macro execution through `rust-plan --execute-topology`. r[rust_package_planning.native_registry_unified_host_topology_execution.tests]
- [ ] [serial] Run focused Rust verification, Cairn validation, and proposal/design/tasks gates before sync/archive/commit/push. r[rust_package_planning.native_registry_unified_host_topology_execution.verify]
