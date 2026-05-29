# Tasks

## Spec

- [x] [serial] Add Cargo-free self-build proof requirement and design. r[rust_package_planning.cargo_free_self_build_proof]

## Implementation

- [ ] [serial] Add proof runner with fast preflight and full proof modes. r[rust_package_planning.cargo_free_self_build_proof]
- [ ] [serial] Add Cargo-forbidden guard/shim and record it in proof evidence. r[rust_package_planning.cargo_free_self_build_proof]
- [ ] [serial] Run native Cargo-free planner/executor over the selected workspace. r[rust_package_planning.cargo_free_self_build_proof]
- [ ] [serial] Write durable audit bundle with receipts, streams, source/tool identities, outputs, and blocker summaries. r[rust_package_planning.cargo_free_self_build_proof]
- [ ] [serial] Add final binary smoke check and output digest verification. r[rust_package_planning.cargo_free_self_build_proof]

## Verification

- [ ] [serial] Run proof preflight, negative Cargo-shim test, full ignored proof or documented blocker run, and Cairn validation. r[rust_package_planning.cargo_free_self_build_proof]
