## Implementation

- [ ] [serial] I1 Extend native topology diagnostics to include stable unit identity, package identity, role, selected triple, target kind, artifact role, and blocker class where available. r[rust_package_planning.native_topology_hardening]
- [ ] [serial] I2 Define a bounded replay receipt shape for failing or blocked native Rust units, using BLAKE3 digests for large inputs. r[rust_package_planning.native_topology_hardening]
- [ ] [serial] I3 Add edge-case fixtures for host build-script dependencies, proc-macro dependencies, selected target features, and source-root host/target splits. r[rust_package_planning.native_topology_hardening]
- [ ] [serial] I4 Keep added planner logic in pure functions with thin execution-shell plumbing. r[rust_package_planning.native_topology_hardening]

## Verification

- [ ] [serial] V1 Positive: run focused native topology fixtures that exercise host/target split, proc-macro, and build-script metadata cases. r[rust_package_planning.native_topology_hardening]
- [ ] [serial] V2 Negative: assert wrong-role, wrong-triple, wrong-source-package, and wrong-metadata artifacts fail before rustc with deterministic diagnostics. r[rust_package_planning.native_topology_hardening]
- [ ] [serial] V3 Negative: assert malformed or oversized replay receipt inputs fail closed without hidden host-tool fallback. r[rust_package_planning.native_topology_hardening]
- [ ] [serial] V4 Run focused Rust topology tests, `cargo fmt -p mantle --check`, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[rust_package_planning.native_topology_hardening]
