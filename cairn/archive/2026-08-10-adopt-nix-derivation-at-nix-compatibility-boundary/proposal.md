# Change: Adopt `nix-derivation` at the Nix compatibility boundary

## Why

Mantle parses concrete Nix `.drv` files through an adapted vendored `nix-compat`. That fork intentionally uses BLAKE3 and configurable store prefixes for native Mantle derivations.

The accepted foreign-import and producer specifications require Nix-compatible derivation identities. They also require proof that compatibility paths do not use Mantle-native BLAKE3 derivation hashing.

`cachix/nix-derivation` version `0.1.0` provides a narrow Apache-2.0 Rust implementation of Nix 2.34 derivation parsing, validation, canonical serialization, hashing, and standard `/nix/store` path construction. Its scope matches the compatibility seam, but its recent publication requires a staged adoption.

## What Changes

- Admit one exact `nix-derivation` package after package-source parity validates the reviewed upstream commit and crates.io checksum.
- Add one Mantle-owned pure adapter as the only production import site for the dependency.
- Use the adapter for concrete Nix `.drv` admission in foreign-import and Nix-producer paths.
- Derive the out-of-band derivation name from the logical `.drv` identity and validate it before projection.
- Preserve Nix-required SHA-256 derivation and store-path semantics inside the compatibility domain.
- Preserve BLAKE3 for Mantle receipts, policies, translated artifacts, and native derivations.
- Dual-run existing and candidate implementations against positive and negative fixtures before cutover.
- Preserve Mantle byte, collection, depth, field, and closure limits outside the dependency.
- Keep adapted `nix-compat` for native Mantle derivations, daemon protocol, NARInfo, wire, store, and castore integration.
- Record a coupled dependency and adapter rollback.

## Non-Goals

- Replacing Mantle's native derivation model or configurable store-prefix behavior.
- Replacing the Nix daemon protocol, store database, NAR, builder, scheduler, or evaluator.
- Claiming compatibility beyond the recorded Nix versions and fixture corpus.
- Cutting over Guix-prefix ATerm admission before separate prefix-rewrite parity passes.
- Exposing upstream crate types in public Mantle plans, receipts, gateway contracts, or store APIs.

## Impact

- **Affected specs:** `foreign-derivation-import` and `nix-producer-adapter`.
- **Planned code:** workspace dependency declarations, one pure adapter crate or module, `src/foreign_import_cmd.rs`, `src/foreign_derivation_import.rs`, `src/nix_producer_shell.rs`, dependency guards, fixtures, and evidence.
- **Compatibility:** accepted foreign graph, package-index, receipt, and CLI schemas remain unchanged.
- **Testing:** package-source parity, Nix 2.34 corpus parity, structured attributes, output variants, recursive dynamic inputs, malformed data, bounds, and wrong-domain hashes.
- **Boundary:** Mantle retains import policy, graph meaning, limits, diagnostics, execution, store, evidence, and release authority.
