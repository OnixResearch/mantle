# Tasks

## Spec

- [x] [serial] Add Cargo-free self-build proof requirement, proposal, and design. r[rust_package_planning.cargo_free_self_build_proof]

## Implementation

- [x] [serial] Teach `--no-cargo-oracle` to bind declared vendored registry sources from `Cargo.lock` without Cargo. r[rust_package_planning.cargo_free_self_build_proof]
- [x] [serial] Teach `--no-cargo-oracle` to bind captured git sources from local vendor material without Cargo. r[rust_package_planning.cargo_free_self_build_proof]
- [x] [serial] Add self-build proof mode or runner path that targets the current Mantle workspace with a failing Cargo shim. r[rust_package_planning.cargo_free_self_build_proof]
- [x] [serial] Smoke-check the produced Mantle CLI and record output digests/non-claims in the proof bundle. r[rust_package_planning.cargo_free_self_build_proof]

## Verification

- [x] [serial] Run focused positive and negative no-Cargo registry/git source tests. r[rust_package_planning.cargo_free_self_build_proof]
- [x] [serial] Run the full Cargo-free self-build proof or record a deterministic blocker bundle. r[rust_package_planning.cargo_free_self_build_proof]
- [x] [serial] Run `cairn validate --root .`. r[rust_package_planning.cargo_free_self_build_proof]
