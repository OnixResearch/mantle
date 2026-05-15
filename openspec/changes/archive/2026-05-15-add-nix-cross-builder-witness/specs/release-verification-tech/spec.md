## ADDED Requirements

### Requirement: Nix cross-builder witness receipt

Mantle MUST define a canonical Nix cross-builder witness receipt that compares the selected Mantle self-built release artifact against a Nix-built artifact from the same recorded source and build policy.
ID: release.verification.tech.cross-builder.nix-witness

The receipt schema MUST be `mantle-nix-cross-builder-witness-v1`. The receipt MUST record the release identifier, selected artifact identity, Mantle deterministic proof receipt digest, Mantle artifact BLAKE3 digest set, Nix artifact BLAKE3 digest set, source tree BLAKE3, vendor/input BLAKE3, Rust toolchain identity, target triple, build flags, linker identity when known, strip/debug policy, `SOURCE_DATE_EPOCH` policy, Nix derivation identity, Nix output path or store identity, comparison verdict, and receipt BLAKE3 over canonical compact JSON bytes.

A witness MAY report proof class `nix-cross-builder-witness` only when the Mantle artifact digest set and the Nix artifact digest set are byte-identical for the selected artifact outputs and the referenced Mantle deterministic proof validates as a promoting `self-rebuild-match`. A Nix witness MUST NOT replace the primary two-clean-store Mantle determinism proof.

#### Scenario: Nix-built binary matches Mantle self-built binary

- GIVEN a release artifact with a valid Mantle deterministic proof receipt whose verdict promotes `self-rebuild-match`
- AND Nix builds the same selected artifact from the recorded source tree, vendor inputs, Rust toolchain, target triple, and build policy
- WHEN Mantle compares the Nix artifact bytes with the Mantle self-built artifact bytes
- AND their canonical BLAKE3 digest sets match exactly
- THEN the Nix cross-builder witness receipt records verdict `nix-witness-match`
- AND release verification may report proof class `nix-cross-builder-witness` alongside `self-rebuild-match`

#### Scenario: Nix mismatch is explainable and fail-closed

- GIVEN the Mantle deterministic proof receipt validates as `self-rebuild-match`
- BUT the Nix-built artifact has a different BLAKE3 digest for a selected output
- WHEN the Nix witness receipt is finalized
- THEN the receipt records verdict `cross-builder-mismatch`
- AND names the mismatched output and both digests
- AND release verification MUST NOT report proof class `nix-cross-builder-witness`
- AND it MUST NOT downgrade or erase the separate Mantle `self-rebuild-match` result solely because the cross-builder witness mismatched

#### Scenario: Nix witness cannot stand in for Mantle proof

- GIVEN a Nix-built artifact matches a Mantle-built artifact byte-for-byte
- BUT the referenced Mantle deterministic proof receipt is missing, malformed, non-promoting, or unsupported
- WHEN release verification evaluates the Nix witness receipt
- THEN it rejects proof class `nix-cross-builder-witness`
- AND it reports the missing or non-promoting Mantle deterministic proof as the blocker

### Requirement: Cross-builder claim wording remains bounded

Mantle MUST describe Nix cross-builder agreement as bit-exact agreement between two recorded builder recipes for selected release artifacts, not as full-source bootstrap reproducibility or global determinism.
ID: release.verification.tech.cross-builder.claims

Human-facing output and JSON reports MUST keep `nix-cross-builder-witness` distinct from `self-rebuild-match`, `external-witness-match`, and `policy-satisfied`. The claim wording MUST be equivalent to: “Nix and Mantle produced bit-identical bytes for this selected artifact from the recorded source, toolchain, target, and build policy.”

#### Scenario: Docs do not overclaim Nix agreement

- GIVEN release verification reports `nix-cross-builder-witness`
- WHEN an operator reads the human-facing summary or JSON class list
- THEN the output distinguishes Nix cross-builder agreement from Mantle self-rebuild determinism and social witness policy
- AND it does not claim full bootstrap reproducibility, cross-platform determinism, or all-package reproducibility
