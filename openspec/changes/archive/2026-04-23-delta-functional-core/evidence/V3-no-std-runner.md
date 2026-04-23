Evidence-ID: delta-functional-core-v3-no-std-runner
Task-ID: V3
Artifact-Type: verification-note
Covers: functional.core.nostd.boundary.continuously.verified.regression.introduces.std.leak, functional.core.shell.adapters.effect.translation.project.refresh.io.in.shell, functional.core.shell.adapters.effect.translation.attestation.file.discovery.in.shell, functional.core.shell.adapters.effect.translation.shell.activation.path.translation.in.shell, functional.core.shell.adapters.effect.translation.release.evidence.bundle.io.in.shell, functional.core.shell.adapters.effect.translation.delta.facade.compatibility, portability.nostd.core.compiles.without.std.target, portability.nostd.core.dependency.allowlist, portability.nostd.core.dependency.allowlist.catches.std.leak
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-22

## Commands

Raw transcript:
- `openspec/changes/delta-functional-core/evidence/v3-rail-2026-04-22.txt`

Validation rail executed in order:
- `cargo check -p crunch-attestation-core`
- `cargo check -p crunch-project-core`
- `cargo check -p crunch-shell-core`
- `cargo check -p crunch-release-core`
- `cargo check -p crunch-delta-core`
- `cargo check -p crunch-attestation-core --target wasm32-unknown-unknown`
- `cargo check -p crunch-project-core --target wasm32-unknown-unknown`
- `cargo check -p crunch-shell-core --target wasm32-unknown-unknown`
- `cargo check -p crunch-release-core --target wasm32-unknown-unknown`
- `cargo check -p crunch-delta-core --target wasm32-unknown-unknown`
- `cargo test -p crunch-attestation-core`
- `cargo test -p crunch-project-core`
- `cargo test -p crunch-shell-core`
- `cargo test -p crunch-release-core`
- `cargo test -p crunch-delta-core`
- `cargo test -p crunch-delta-core duplicate_version_offer_is_rejected`
- `cargo test -p crunch-delta-core prefix_mismatch_is_rejected`
- `cargo test -p crunch-attestation shell_adapter_keeps_discovery_outside_core`
- `cargo test -p crunch-project shell_adapter_keeps_refresh_io_outside_core`
- `cargo test -p crunch-shell adapter_preserves_path_order_and_appends_bin`
- `cargo test -p crunch-shell non_utf8_with_path_is_rejected`
- `cargo test -p crunch --bin crunch create_and_verify_release_bundle_round_trip`
- `cargo test -p crunch --bin crunch load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact`
- `cargo test -p crunch --test release_cli release_verify_rejects_manifest_schema_mismatch`
- `cargo test -p crunch --test release_cli release_verify_rejects_missing_workflow_provenance`
- `cargo test -p crunch --test release_cli release_verify_rejects_claim_boundary_violation`
- `cargo test -p crunch --test release_cli release_verify_rejects_proof_linkage_source_digest_mismatch`
- `cargo test -p crunch-delta substitution_adapter_keeps_async_store_and_network_in_shell`
- `cargo test -p crunch-delta delta_facade_reexports_core_planner_types`
- `./scripts/check-no-std-core-deps.sh`
- `./scripts/check-no-std-core-purity.sh`
- `./scripts/check-no-std-core-scope.sh`
- `./scripts/check-no-std-core-api-shape.sh`
- `./scripts/check-no-std-core-ownership.sh`
- `./scripts/check-no-std-core.sh`

## Results

### Wasm prerequisite path

The rail captured the explicit pre-target fallback path before wasm checks:

- `rustup not found on PATH; verifying preinstalled target instead`
- `rustc --print target-libdir --target wasm32-unknown-unknown`
- observed target libdir:
  `/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/wasm32-unknown-unknown/lib`

### Host + wasm checks

All five adopted cores passed both host and `wasm32-unknown-unknown` checks:

- `crunch-attestation-core`
- `crunch-project-core`
- `crunch-shell-core`
- `crunch-release-core`
- `crunch-delta-core`

### Core suites and representative negative cases

Core suites stayed green:

- `crunch-attestation-core` → `test result: ok. 65 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
- `crunch-project-core` → `test result: ok. 77 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
- `crunch-shell-core` → `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
- `crunch-release-core` → `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
- `crunch-delta-core` → `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

Named negative cases required by the task appear in the transcript:

- `canonical::tests::canonicalization_rejects_invalid_closure_root_kind ... ok`
- `policy::tests::policy_rejects_unsupported_independence_field ... ok`
- `lock::tests::lockfile_detects_empty_hash ... ok`
- `refresh::tests::apply_outcomes_reverts_input_when_patch_resolution_fails ... ok`
- `attestation::tests::project_attestation_rejects_missing_locked_patch ... ok`
- `negotiation::tests::duplicate_version_offer_is_rejected ... ok`
- `planner::tests::prefix_mismatch_is_rejected ... ok`

### Shell-boundary and façade checks

Required std-shell / façade proofs stayed green:

- `discovery::tests::shell_adapter_keeps_discovery_outside_core ... ok`
- `refresh_adapter::tests::shell_adapter_keeps_refresh_io_outside_core ... ok`
- `adapter::tests::adapter_preserves_path_order_and_appends_bin ... ok`
- `adapter::tests::non_utf8_with_path_is_rejected ... ok`
- `release_evidence::tests::create_and_verify_release_bundle_round_trip ... ok`
- `release_evidence::tests::load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact ... ok`
- `release_verify_rejects_manifest_schema_mismatch ... ok`
- `release_verify_rejects_missing_workflow_provenance ... ok`
- `release_verify_rejects_claim_boundary_violation ... ok`
- `release_verify_rejects_proof_linkage_source_digest_mismatch ... ok`
- `substitution::tests::substitution_adapter_keeps_async_store_and_network_in_shell ... ok`
- `planner::tests::delta_facade_reexports_core_planner_types ... ok`

### Checker rail and ownership-review confirmation

All checker commands passed:

- `dependency allowlist OK: ... crunch-delta-core ...`
- `purity check OK`
- `scope check OK`
- `API shape check OK`
- `ownership check OK: crates/crunch-delta/src/fixtures.rs, crates/crunch-delta/src/lib.rs, crates/crunch-delta/src/manifest.rs, crates/crunch-delta/src/model.rs, crates/crunch-delta/src/negotiation.rs, crates/crunch-delta/src/planner.rs, crates/crunch-delta/src/substitution.rs, crates/crunch-shell/src/adapter.rs, crates/crunch-shell/src/lib.rs, crates/crunch-shell/src/types.rs, src/release_cmd.rs, src/release_evidence.rs`

That checker output, together with `openspec/specs/functional-core/evidence/ownership-review.md`, confirms:

- the approved no-std dependency allowlist still contains `crunch-delta-core` and excludes std-only leakage from the adopted core boundary
- the ownership review now names the touched delta std files `crates/crunch-delta/src/{fixtures.rs,model.rs,negotiation.rs,planner.rs}`
- the ownership review still names the required shell/release/delta std adapter files
- the touched delta std files are classified as `adapter-only` or `unrelated`
- the verdict states that shell/release business logic remains in `crunch-shell-core` / `crunch-release-core` while delta planning/protocol business logic remains in `crunch-delta-core`

### Umbrella runner

The final umbrella runner also passed:

- `./scripts/check-no-std-core.sh`
- observed output includes:
  - `toolchain: preinstalled`
  - `Specification 'functional-core' is valid`
  - host checks, wasm checks, core tests, adapter tests, and checker steps completing without failure

## Conclusion

V3 satisfied on 2026-04-22. The full exact rail completed, including the umbrella runner, and the resulting packet now proves host/wasm compilation, dependency-allowlist enforcement, ownership-review coverage, required negative cases, and the std-shell / façade boundaries for the delta wave.