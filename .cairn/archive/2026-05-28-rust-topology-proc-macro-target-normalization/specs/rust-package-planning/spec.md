# rust-package-planning delta

## ADDED Requirements

### Requirement: Native proc-macro host selection normalizes Cargo target-name spelling

r[rust_package_planning.native_proc_macro_target_name_normalization] Native Rust host-unit planning MUST match Cargo-selected proc-macro host units by package id, proc-macro kind, and Rust crate-name spelling so manifest package-target names with hyphens match Cargo unit-graph target names with underscores.

#### Scenario: selected hyphenated proc macro is planned

- **GIVEN** a native package fact for `curve25519-dalek-derive` whose proc-macro target name uses hyphen spelling
- **AND** the Cargo unit graph contains the selected proc-macro host unit using `curve25519_dalek_derive` target-name spelling
- **WHEN** native host-unit planning matches selected host keys
- **THEN** the proc-macro host unit is planned
- **AND** the target consumer records that proc-macro as a consumed host artifact

#### Scenario: absent proc macro remains omitted

- **GIVEN** another proc-macro package exists in native package facts
- **AND** the Cargo unit graph does not contain that package's proc-macro host unit
- **WHEN** native host-unit planning matches selected host keys
- **THEN** the absent proc-macro package is not planned
