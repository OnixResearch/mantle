# Tasks

- [x] [serial] Record pushed-head self-probe evidence for the `itertools@0.10.5` missing producer blocker. Evidence: `evidence/current-blocker.md`. r[rust_package_planning.native_registry_transitive_producer_coverage]
- [ ] [serial] Add focused positive coverage where a transitive vendored-registry dependency with ready package/source facts gets an executable producer `lib` unit. r[rust_package_planning.native_registry_transitive_producer_coverage]
- [ ] [serial] Add negative coverage where a consumer dependency artifact lacks native package/source/lib producer facts and graph readiness fails before topology execution. r[rust_package_planning.native_registry_transitive_producer_coverage]
- [ ] [serial] Implement native unit/derivation graph producer closure so supported transitive registry dependency artifacts have producer units before consumers execute. r[rust_package_planning.native_registry_transitive_producer_coverage]
- [ ] [serial] Verify focused Rust plan tests, Mantle self probe blocker movement, Cairn validation, Cairn gates, sync, archive, and commit. r[rust_package_planning.native_registry_transitive_producer_coverage]
