## Phase 1: Specification

- [x] [serial] Validate the Nix cross-builder witness proposal, design, and delta specs with OpenSpec strict validation.
- [x] [serial] Review claim boundaries so `nix-cross-builder-witness` remains separate from `self-rebuild-match`, `external-witness-match`, and `policy-satisfied`.

## Phase 2: Future Implementation

- [x] [serial] Add `mantle-nix-cross-builder-witness-v1` receipt data model and canonical BLAKE3 digest helper.
- [x] [depends:receipt-model] Add release build/CLI wiring to build or locate both Mantle self-built and Nix-built selected artifacts under the recorded policy.
- [x] [depends:release-wiring] Add positive and negative tests for bit-exact match, digest mismatch, missing Mantle proof, and bounded output wording.
- [x] [depends:tests] Expose the heavy check as a release-only flake package/check while keeping ordinary developer builds fast.
