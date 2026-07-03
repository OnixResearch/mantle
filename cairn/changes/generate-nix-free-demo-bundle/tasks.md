## Implementation

- [ ] [serial] I1 Add a pure Nix-free demo bundle manifest builder that accepts explicit proof status, receipt digests, transcript references, artifact digests, and non-claims. r[verification_evidence.nix_free_demo_bundle_generator]
- [ ] [serial] I2 Add a shell command or script that writes the bundle directory, `summary.json`, README, transcript copies or digests, and generated metadata. r[verification_evidence.nix_free_demo_bundle_manifest]
- [ ] [serial] I3 Reuse existing validation and README rendering after generation. r[verification_evidence.nix_free_demo_bundle_generator]
- [ ] [serial] I4 Update docs with generator usage and blocked-proof examples. r[verification_evidence.nix_free_demo_generator_non_claims]

## Verification

- [ ] [serial] V1 Positive: generate a demo bundle from fixture evidence, validate it, render its README, and prove deterministic output across repeated generation. r[verification_evidence.nix_free_demo_bundle_generator]
- [ ] [serial] V2 Negative: reject missing transcripts, contradictory proof status, absent non-claims for blocked/synthetic evidence, malformed digest fields, and output paths that would overwrite unrelated files. r[verification_evidence.nix_free_demo_bundle_manifest]
- [ ] [serial] V3 Run focused demo CLI tests, guide guard tests, `cargo fmt -p mantle --check`, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[verification_evidence.nix_free_demo_generator_non_claims]
