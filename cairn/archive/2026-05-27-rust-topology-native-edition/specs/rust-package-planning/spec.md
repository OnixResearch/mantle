# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Native Rust manifest edition derivations

r[rust_package_planning.native_manifest_edition_derivations] Mantle MUST propagate each native package manifest edition into every generated native Rust unit derivation that compiles that package target.

#### Scenario: Declared edition is used for native derivation args

GIVEN a supported native package manifest declares `edition = "2024"`
WHEN Mantle plans native target or host unit derivations for that package
THEN Mantle MUST emit rustc args with `--edition 2024` for that package's generated unit derivations.
AND Mantle MUST NOT replace the declared edition with a hard-coded default.

#### Scenario: Workspace-inherited edition is used

GIVEN a supported native package manifest declares `edition.workspace = true`
AND the root workspace manifest declares `workspace.package.edition = "2024"`
WHEN Mantle plans native target or host unit derivations for that package
THEN Mantle MUST emit rustc args with `--edition 2024`.
AND Mantle MUST keep the inherited edition deterministic in native package/target facts.

#### Scenario: Missing edition uses Cargo default

GIVEN a supported native package manifest omits `package.edition`
WHEN Mantle plans native target or host unit derivations for that package
THEN Mantle MUST emit rustc args with Cargo's default `--edition 2015`.
AND Mantle MUST keep the default deterministic in native package/target facts.

#### Scenario: Rust 2024 topology blocker moves

GIVEN topology execution currently fails because a native proc-macro package that declares Rust 2024 is invoked with `--edition 2021`
WHEN native edition propagation is applied
THEN self-probe verification MUST show that the Rust 2024 let-chain edition error no longer blocks the first topology units.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
