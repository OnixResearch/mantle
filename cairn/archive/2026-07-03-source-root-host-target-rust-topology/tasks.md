## Implementation

- [x] [serial] I1 Derive role-aware native unit identities and artifact keys in the pure Rust topology core. r[rust_package_planning.source_root_host_target_topology]
- [x] [serial] I2 Route host custom-build dependencies through host-dependency units while preserving target-unit routing for target libraries and binaries. r[rust_package_planning.source_root_host_target_topology]
- [x] [serial] I3 Add pre-rustc artifact validation for role, triple, source digest, metadata hash, and toolchain-policy digest mismatches. r[rust_package_planning.source_root_host_target_topology]
- [x] [serial] I4 Extend topology receipts and blocker summaries with unit role, selected triple, and toolchain-policy digest evidence. r[rust_package_planning.source_root_host_target_topology]

## Verification

- [x] [serial] V1 Positive: run a fixture where a build script depends on support crates that must compile as host-dependency units while the package library remains target-scoped. r[rust_package_planning.source_root_host_target_topology]
- [x] [serial] V2 Positive: run a target-library fixture proving target units keep the requested target triple and do not consume host-dependency artifacts. r[rust_package_planning.source_root_host_target_topology]
- [x] [serial] V3 Negative: inject or plan a wrong-role or wrong-triple dependency artifact and prove Mantle fails before invoking `rustc`. r[rust_package_planning.source_root_host_target_topology]
- [x] [serial] V4 Rerun the current source-root Cargo-free fixed-point proof or record the next deterministic blocker with role-aware receipt evidence. r[rust_package_planning.source_root_host_target_topology]
- [x] [serial] V5 Run focused Rust topology tests, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[rust_package_planning.source_root_host_target_topology]
