# Design: Kani reproducible toolchain evidence

## Approach

Mantle treats Kani as an external verifier toolchain whose identity can be packaged, measured, and bundled. The build/release core records identity metadata; Valence remains responsible for Kani evidence semantics.

## Toolchain identity

Kani toolchain evidence records:

- Kani package or wrapper identity;
- Rust toolchain identity;
- CBMC identity;
- solver identity;
- Onix/Nix closure digest or package graph identity;
- invocation wrapper identity;
- Kani receipt artifact digest.

## Release bundle integration

A release bundle may include Kani receipts as external evidence. Mantle verifies that the referenced receipt and toolchain identities match the bundle metadata. It does not inspect Kani semantics beyond the bundle contract; semantic validation remains Valence-owned.

## Fixture strategy

Positive fixtures cover a bundle with matching Kani receipt and toolchain metadata. Negative fixtures cover missing Kani version, stale closure identity, unsupported solver metadata, mismatched receipt digest, and missing non-claims.

## What is explicitly not provided

- No Kani execution during ordinary Mantle builds.
- No verifier-soundness claim.
- No semantic interpretation beyond bundle identity and non-claim preservation.
