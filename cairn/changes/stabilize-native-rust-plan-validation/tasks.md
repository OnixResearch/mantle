## Implementation

- [ ] [serial] I1 Inventory native rust-plan tests that currently require `--test-threads=1` or have known fixture races. r[rust_package_planning.native_topology_validation_stability]
- [ ] [serial] I2 Isolate shared compiler-policy, rustc-wrapper, cargo-shim, OUT_DIR, and topology execution fixtures with per-test temp roots or explicit locks. r[rust_package_planning.native_topology_race_free_fixtures]
- [ ] [serial] I3 Keep ambient-env negative tests subprocessed and add guards against in-process `set_var` regressions. r[rust_package_planning.native_topology_race_free_fixtures]
- [ ] [serial] I4 Document stable serial and parallel validation commands plus any remaining justified serial-only tests. r[rust_package_planning.native_topology_parallel_evidence]

## Verification

- [ ] [serial] V1 Run focused native rust-plan tests serially and record pass evidence. r[rust_package_planning.native_topology_validation_stability]
- [ ] [serial] V2 Run the same focused native rust-plan tests in parallel repeatedly or with a stress count and record pass evidence or exact remaining serial blocker. r[rust_package_planning.native_topology_parallel_evidence]
- [ ] [serial] V3 Run negative ambient-env tests and prove they use subprocess boundaries. r[rust_package_planning.native_topology_race_free_fixtures]
- [ ] [serial] V4 Run `cargo fmt -p mantle --check`, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[rust_package_planning.native_topology_validation_stability]
