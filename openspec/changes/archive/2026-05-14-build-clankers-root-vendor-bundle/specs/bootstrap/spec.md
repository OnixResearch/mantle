## ADDED Requirements

### Requirement: Scalable Clankers Root Vendor Closure

Crunch MUST represent the root `clankers` Cargo source and vendor closure as a fixed, reproducible input without requiring a monolithic large artifact to be committed to git.

#### Scenario: root closure is larger than small-rung package artifacts
- **GIVEN** offline `cargo vendor --locked --offline --versioned-dirs` for `/home/brittonr/git/clankers`
- **WHEN** the root `clankers` binary derivation is prepared
- **THEN** the implementation records the vendor crate count and byte size
- **AND** the implementation chooses a scalable fixed-input representation before adding `packages/clankers/clankers.ncl`

### Requirement: Root Clankers Binary Build Evidence

Crunch MUST build the root `clankers` binary from the fixed source/vendor closure with offline Cargo and record the output binary and smoke result.

#### Scenario: root binary builds without network
- **GIVEN** the scalable source/vendor closure representation exists
- **WHEN** Crunch builds `packages/clankers/clankers.ncl`
- **THEN** Cargo runs with `--locked --offline`
- **AND** `CARGO_HOME` is sandbox-local
- **AND** `CARGO_TARGET_DIR` is deterministic
- **AND** `$out/bin/clankers` exists
- **AND** a network-free `$out/bin/clankers --help` or `$out/bin/clankers --version` smoke output is recorded
