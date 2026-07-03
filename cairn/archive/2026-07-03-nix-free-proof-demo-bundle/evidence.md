# Evidence: nix-free-proof-demo-bundle

Date: 2026-07-03

## Implementation evidence

- Added `src/nix_free_demo_bundle.rs`, a pure validation/rendering core for the source-root Cargo-free fixed-point demo profile.
- The validator requires matching stage1/stage2 BLAKE3 digests, source-root identity, BLAKE3 toolchain-policy digest, denial evidence for Cargo/Nix/rustup/ambient-wrapper guards, replay hints, and explicit non-claims before `demo_claimable` is true.
- The generated operator README is rendered from the machine summary and validation diagnostics; it only emits `Nix-free fixed-point demo: claimable` when validation succeeds.
- Added README wording that gates Nix-free fixed-point demo claims on the validated proof bundle and otherwise requires narrower blocker/fixed-point wording.

## Verification commands

All commands were run from `/home/brittonr/git/mantle`.

- Pueue task 17: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle nix_free_demo_bundle -- --nocapture`
  - Result: `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1113 filtered out; finished in 0.00s`.
  - Positive: `demo_bundle_validator_accepts_matching_fixed_point_and_guard_denials`.
  - Negative: `demo_bundle_validator_rejects_missing_fixed_point_evidence`.
  - Negative: `demo_bundle_validator_rejects_missing_guard_denial`.
  - README derivation: `generated_readme_is_derived_from_machine_summary`.
- Pueue task 19: `cargo fmt -p mantle --check && git diff --check`
  - Result: completed successfully.
- Pueue task 21: `nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle`
  - Result: `"valid": true`, `"changes": 1`, `"specs_validated": 16`.
- Pueue task 22: `nix run path:/home/brittonr/git/cairn#cairn -- gate proposal nix-free-proof-demo-bundle --root /home/brittonr/git/mantle`
  - Result: `"stage": "proposal"`, `"valid": true`, `"verdict": "PASS"`.
- Pueue task 24: `nix run path:/home/brittonr/git/cairn#cairn -- gate design nix-free-proof-demo-bundle --root /home/brittonr/git/mantle`
  - Result: `"stage": "design"`, `"valid": true`, `"verdict": "PASS"`.
- Pueue task 23: `nix run path:/home/brittonr/git/cairn#cairn -- gate tasks nix-free-proof-demo-bundle --root /home/brittonr/git/mantle`
  - Result: `"stage": "tasks"`, `"valid": true`, `"verdict": "PASS"`.
