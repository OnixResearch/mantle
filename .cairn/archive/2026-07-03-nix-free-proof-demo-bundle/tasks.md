## Implementation

- [x] [serial] I1 Define the demo bundle profile schema with source-root identity, toolchain-closure policy digest, guard evidence, stage digests, fixed-point verdict, replay hints, and non-claims. r[verification_evidence.nix_free_proof_demo_bundle]
- [x] [serial] I2 Generate the operator README from the machine summary without hand-edited pass/fail fields. r[verification_evidence.nix_free_proof_demo_bundle]
- [x] [serial] I3 Add a validator that accepts only bundles with matching positive fixed-point evidence plus required Cargo/Nix/rustup/ambient-wrapper denial evidence. r[verification_evidence.nix_free_proof_demo_bundle]
- [x] [serial] I4 Gate docs, release-readiness, or status wording that says "Nix-free" or equivalent demo claims on the validated demo profile. r[verification_evidence.nix_free_proof_demo_bundle]

## Verification

- [x] [serial] V1 Positive: validate a demo-profile bundle with matching stage1/stage2 digests, source-root/toolchain policy evidence, and all required guard-denial records. r[verification_evidence.nix_free_proof_demo_bundle]
- [x] [serial] V2 Negative: remove fixed-point digest evidence and prove the validator rejects the bundle without weakening the requested demo claim. r[verification_evidence.nix_free_proof_demo_bundle]
- [x] [serial] V3 Negative: remove one guard-denial record for Cargo, Nix, rustup, or ambient wrappers and prove the validator rejects Nix-free demo claimability. r[verification_evidence.nix_free_proof_demo_bundle]
- [x] [serial] V4 Run focused summary/README/validator tests, docs checks, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[verification_evidence.nix_free_proof_demo_bundle]
