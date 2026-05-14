## ADDED Requirements

### Requirement: Clankers root Cargo paths are reproducibility-stabilized

Crunch MUST normalize or eliminate nondeterministic Cargo build-script output paths from the Clankers root output proof before claiming the Clankers root binary is byte-reproducible.
ID: bootstrap.external-fixed-bundle.clankers-cargo-path-stability

The Clankers derivation MUST pass deterministic Rust path-remapping controls for sandbox-local build roots, run fresh-store rebuilds after the controls are applied, compare rebuilt output binary BLAKE3 digests, and record either a matching stable digest or a fail-closed mismatch receipt. A successful stability proof MUST update final proof metadata to the stable binary digest produced by the remapped derivation.

#### Scenario: Fresh remapped rebuilds match

- GIVEN the Clankers derivation applies deterministic Rust path remapping
- WHEN two fresh-store Crunch rebuilds complete
- THEN both rebuilt `$out/bin/clankers` files have the same BLAKE3 digest
- AND proof metadata records that digest as the stable binary proof input

#### Scenario: Remaining path mismatch is recorded fail-closed

- GIVEN the remapped derivation still produces differing binary BLAKE3 digests
- WHEN the rebuild receipt is generated
- THEN the receipt records verdict `mismatch`
- AND it records the first bounded observed difference
- AND final proof metadata is not updated to claim reproducibility
