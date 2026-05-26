# Tasks

- [x] [serial] Define the bounded registry-backed host-artifact topology execution requirement and non-goals. r[rust_package_planning.native_registry_host_artifact_topology_execution]
- [x] [serial] Require ready native registry source facts and ready native host-unit graph facts before registry-backed host units execute. r[rust_package_planning.native_registry_host_artifact_topology_execution.source_and_host_facts]
- [x] [serial] Bind registry source identity, checksum, vendor root, source digest, produced host artifact digests, build-script metadata, and target consumer digests into receipts. r[rust_package_planning.native_registry_host_artifact_topology_execution.receipts]
- [x] [serial] Fail closed before host or target execution for missing, stale, unsupported, or ambient-cache-dependent registry host material. r[rust_package_planning.native_registry_host_artifact_topology_execution.blockers]
- [x] [serial] Add positive and negative CLI fixtures for vendored registry-backed build-script or proc-macro topology execution. r[rust_package_planning.native_registry_host_artifact_topology_execution.tests]
- [x] [serial] Run focused Rust verification, Cairn validation, and proposal/design/tasks gates before sync/archive/commit/push. r[rust_package_planning.native_registry_host_artifact_topology_execution.verify]
