# Change: Kani reproducible toolchain evidence

## Why

Kani receipts are only reviewable release evidence when the verifier stack is identified: Kani, Rust, CBMC, solvers, and their Onix/Nix closure. Mantle should package or record that toolchain identity so release bundles can carry Kani evidence without relying on ambient developer machines.

## What Changes

- Add Mantle release-evidence support for Kani verifier toolchain identity.
- Record Kani, Rust, CBMC, solver, wrapper, and closure identities alongside Kani receipts.
- Let release bundles include Kani evidence as external scoped verification evidence with explicit non-claims.
- Add positive and negative checks for matching toolchain identity, stale closures, unsupported solver metadata, and missing non-claims.

## Impact

- **Files**: package/toolchain declarations, release-evidence schema handling, fixtures, docs, and bundle verification tests.
- **Consumers**: Octet/Valence Kani receipts can be bundled reproducibly in Mantle release evidence.
- **Non-claims**: Mantle records and verifies toolchain identity and bundle linkage; it does not prove Kani, CBMC, or solver soundness.
