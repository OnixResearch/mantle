# Evidence: source-root-host-target-rust-topology

Date: 2026-07-03

## Implementation evidence

- `src/rust_plan.rs` now records each Rust unit derivation with `execution_kind`, `selected_triple`, and `rustc_metadata_hash`.
- Host-dependency clones retag their unit role, host triple, rustc metadata hash, rustc metadata arg, and args digest while preserving target consumers' target dependency routing.
- Topology execution now runs a pre-rustc role-sensitive artifact graph validator. It rejects wrong role/triple, source package, rustc metadata hash, and source-root toolchain-policy digest mismatches with `artifact-identity-mismatch` blockers.
- `RustUnitExecutionReceipt` now records `execution_kind`, `selected_triple`, `rustc_metadata_hash`, `toolchain_policy_digest_blake3`, `artifact_identity_digest_blake3`, and `consumed_artifact_roles`.
- `src/cargo_free_self_build.rs` threads the source-built toolchain closure policy digest into cargo-free rust-plan child invocations and fixed-point stages through `MANTLE_RUST_TOOLCHAIN_POLICY_DIGEST_BLAKE3`.

## Verification commands

All commands were run from `/home/brittonr/git/mantle`.

- Pueue task 42: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle host_dependency_topology -- --nocapture`
  - Result: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 0.00s`
- Pueue task 43: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle host_dependency_derivations_clone_libs_for_host_consumers_without_rewriting_targets -- --nocapture`
  - Result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1111 filtered out; finished in 0.00s`
- Pueue task 40: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle combined_unit_topology -- --nocapture`
  - Result: `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1104 filtered out; finished in 0.00s`
- Pueue task 37: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle role_validation_rejects -- --nocapture`
  - Result: `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.00s`
- Pueue task 40: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle finalized_receipt_records_role_triple_policy_and_identity -- --nocapture`
  - Result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1111 filtered out; finished in 0.00s`
- Pueue task 33: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle fixed_point_status_accepts_matching_policy_digest -- --nocapture`
  - Result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1108 filtered out; finished in 0.00s`
- Pueue task 47: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo build -p mantle --bin mantle`
  - Result: `Finished dev profile [unoptimized + debuginfo] target(s) in 49.36s`
- Pueue task 49: `/home/brittonr/.cargo-target/debug/mantle --json self-build --cargo-free --fixed-point --out /tmp/mantle-source-root-host-target-fixed-point --rustc "$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc" --target x86_64-unknown-linux-musl`
  - Result: deterministic blocker recorded: `stage1 blocked: topology execution status was blocked`.
  - Role-aware receipt evidence exists at `/tmp/mantle-source-root-host-target-fixed-point/stage1/receipt.json`.
  - `rg --count-matches 'selected_triple|toolchain_policy_digest_blake3|artifact_identity_digest_blake3|consumed_artifact_roles' /tmp/mantle-source-root-host-target-fixed-point/stage1/receipt.json` returned `741`.
- Pueue task 53: `cargo fmt -p mantle --check`
  - Result: completed successfully.
- Pueue task 54: `git diff --check`
  - Result: completed successfully.
- Pueue task 61: `nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle`
  - Result: `"valid": true`, `"changes": 3`, `"specs_validated": 18`.
- Pueue task 63: `nix run path:/home/brittonr/git/cairn#cairn -- gate proposal source-root-host-target-rust-topology --root /home/brittonr/git/mantle`
  - Result: `"stage": "proposal"`, `"valid": true`, `"verdict": "PASS"`.
- Pueue task 64: `nix run path:/home/brittonr/git/cairn#cairn -- gate design source-root-host-target-rust-topology --root /home/brittonr/git/mantle`
  - Result: `"stage": "design"`, `"valid": true`, `"verdict": "PASS"`.
- Pueue task 65: `nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-root-host-target-rust-topology --root /home/brittonr/git/mantle`
  - Result: `"stage": "tasks"`, `"valid": true`, `"verdict": "PASS"`.
