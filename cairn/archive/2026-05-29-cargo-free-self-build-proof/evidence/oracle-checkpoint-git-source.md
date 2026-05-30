# Oracle Checkpoint: Captured Git Source Completion Claim

- **Question:** Does the completed task “Teach `--no-cargo-oracle` to bind captured git sources from local vendor material without Cargo” have concrete verification evidence, independent of the self-build proof summary?
- **Inspected evidence:** `src/rust_plan.rs::tests::no_cargo_capture_binds_captured_git_source` builds a temp workspace with a git dependency, a `Cargo.lock` git source, and local `vendor-deps/wu-manber-0.1.0` source, then runs `capture_rust_plan` with `no_cargo_oracle=true` and asserts `native_git_source_planning.ready`, resolved revision, captured manifest path, package planning ready, unit graph ready, and derivation graph ready. `src/rust_plan.rs::tests::native_git_source_planning_binds_captured_source_and_resolves_dependency_edge` covers direct git source binding and dependency edge resolution. Negative git coverage includes missing captured manifest and mismatched revision tests.
- **Decision:** The git-source task is complete only with the focused git tests above; the self-build proof remains supporting end-to-end topology evidence and is not used as the sole git-source oracle.
- **Owner:** Mantle maintainers.
- **Next action:** Keep the focused `no_cargo_capture_binds_captured_git_source` test and the native git negative tests in the validation rail whenever the task remains checked.
