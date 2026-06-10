# Tasks

## Contract

- [x] [serial] Define the Rust compiler policy adapter data model, modes, and receipt identity fields without naming Octet in the generic core type. r[rust_package_planning.compiler_policy_adapter]
- [x] [serial] Define the bounded claim text for policy-profile compliance, explicit waivers, and non-claims. r[rust_package_planning.compiler_policy_adapter.non_claims]

## Implementation

- [x] [serial] Add CLI/config plumbing for plain, audit, deny, and required compiler-policy modes while preserving plain execution as the default. r[rust_package_planning.compiler_policy_adapter.cli]
- [x] [serial] Implement adapter invocation resolution at the Rust unit execution seam and keep process execution in the imperative shell. r[rust_package_planning.compiler_policy_adapter.invocation]
- [x] [serial] Implement the Octet/Dylint adapter provider path using provider-manifest or explicit adapter artifacts. r[rust_package_planning.compiler_policy_adapter.octet]
- [x] [serial] Add optional Octet standards-gate execution/receipt binding for FCIS source-shape checks without folding standards logic into rustc execution. r[rust_package_planning.compiler_policy_adapter.standards_gate]
- [x] [serial] Include compiler-policy identity in output reuse decisions and all Rust unit/topology execution receipts. r[rust_package_planning.compiler_policy_adapter.cache_identity]

## Verification

- [x] [serial] Add positive tests for plain execution, Octet audit mode, Octet deny mode, and Octet-required mode with complete adapter identity. r[rust_package_planning.compiler_policy_adapter]
- [x] [serial] Add negative tests for missing driver, missing lint library, toolchain mismatch, policy digest mismatch, missing standards artifact, and raw-rustc output reuse under required mode. r[rust_package_planning.compiler_policy_adapter.fail_closed]
- [x] [serial] Add CLI receipt tests proving adapter identity and waiver summaries appear in JSON evidence. r[rust_package_planning.compiler_policy_adapter.receipts]
- [x] [serial] Run focused Rust-plan tests plus `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and Cairn gates for this change. r[rust_package_planning.compiler_policy_adapter]
