# `nix-derivation` source admission audit

## Reviewed package

- Package: `nix-derivation` `0.1.0`
- crates.io archive SHA-256: `a5d03dfde06a8ce7e0e007f4795ab74c546e5c7d6c6375a08ceb20677ec8a074`
- Repository: `https://github.com/cachix/nix-derivation`
- Upstream commit: `2cfc0f90ed83ea3cc983e5c305f89494a6df073e`
- License: `Apache-2.0`
- Recorded compatibility: Nix 2.34
- Mantle base revision: `465e4b45abd7ff8887ada01df93c9a45d30faf06`

The crates.io package contains `.cargo_vcs_info.json` with the reviewed commit.
`Cargo.toml.orig` matches the upstream `Cargo.toml`.
The guard compares every other packaged source, test, benchmark, fixture, README, and license file with the reviewed upstream tree.

The guard permits only Cargo-generated `.cargo-ok`, `.cargo_vcs_info.json`, `Cargo.lock`, and normalized `Cargo.toml` package files outside that projection.

## Authority boundary

`src/nix_derivation_adapter.rs` is the only production Rust file that imports `nix_derivation`.
The adapter returns Mantle-owned values and performs no file, process, network, clock, registry, or publication effects.

Mantle owns byte limits, closure limits, graph projection, diagnostics, execution, store behavior, receipts, and release decisions.
The dependency owns only the reviewed Nix derivation metadata behavior.

## Validation

The source and boundary guard is `scripts/check-nix-derivation-boundary.rs`.
It has positive and negative self-tests for direct imports, package drift, VCS drift, and archive-checksum drift.

The actual parity run used the exact registry package, crates.io archive, and detached upstream commit.
Its final transcript is stored in `source-admission-validation.log`.

Cargo generated `vendor-deps/nix-derivation/.cargo-checksum.json` from the admitted package.
`cargo metadata --offline --locked --config .cargo/vendor-config.toml` passed with that package.
The transcript is stored in `vendor-offline-validation.log`.

## Rollback

Remove `nix-derivation = "=0.1.0"` and regenerate `Cargo.lock`.
Restore the prior Nix ATerm call sites from revision `465e4b45abd7ff8887ada01df93c9a45d30faf06` in the same change.
Do not keep only the dependency or only the adapter.

This rollback does not change the accepted foreign graph, package-index, receipt, or CLI schemas.
