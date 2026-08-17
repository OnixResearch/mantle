## Implementation

- [x] [serial] I1 Define the comparison corpus manifest and pure equivalence classifier for sources, toolchains, output surfaces, normalization policy, and allowed differences. r[verification_evidence.nix_mantle_comparison_corpus]
- [x] [serial] I2 Implement runner/report plumbing that builds selected corpus cases through Nix and Mantle and compares BLAKE3 object/NAR digest evidence. r[verification_evidence.nix_mantle_comparison_corpus]
- [x] [serial] I3 Document corpus scope, blockers, unsupported features, and why comparison evidence does not automatically prove global superiority. r[verification_evidence.nix_mantle_comparison_corpus]

## Verification

- [x] [serial] V1 Positive: run at least one equivalent fixture where Nix and Mantle outputs match by content digest. r[verification_evidence.nix_mantle_comparison_corpus]
- [x] [serial] V2 Negative: run a non-equivalent toolchain or unsupported-feature fixture and prove the report records a blocker instead of a mismatch-as-failure or false success. r[verification_evidence.nix_mantle_comparison_corpus]
- [x] [serial] V3 Run focused corpus tests, digest report checks, docs checks, and Cairn validate/gates for this change. r[verification_evidence.nix_mantle_comparison_corpus]
