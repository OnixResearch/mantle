# Rust Package Planning Delta

## ADDED Requirements

### Requirement: Cargo built-in profile table

r[rust_package_planning.profile_defaults_table] Mantle MUST implement the four Cargo built-in profiles as a deterministic setting table with `test` inheriting `dev` and `bench` inheriting `release`.

#### Scenario: Built-in profiles resolve to Cargo defaults

- GIVEN a request for `dev`, `release`, `test`, or `bench`
- WHEN Mantle resolves the profile
- THEN the resolved settings MUST equal Cargo's documented defaults for opt-level, debug, debug-assertions, overflow-checks, lto, panic, incremental, codegen-units, and rpath
- AND `test` MUST equal `dev` and `bench` MUST equal `release`

#### Scenario: Unknown profile fails closed

- GIVEN a profile name that is not a built-in or a declared custom profile
- WHEN Mantle resolves the profile
- THEN planning MUST fail with a deterministic unknown-profile blocker before any unit execution

### Requirement: Profile codegen flags reach rustc

r[rust_package_planning.profile_codegen_flags] Mantle MUST lower resolved profile settings into explicit rustc codegen arguments for every planned Rust unit.

#### Scenario: Release profile changes compiler arguments

- GIVEN a native Rust unit planned with the `release` profile
- WHEN Mantle builds the rustc argument list
- THEN the arguments MUST include `-C opt-level=3`, `-C debuginfo=0`, disabled debug assertions, and disabled overflow checks
- AND the same unit planned with the `dev` profile MUST produce different codegen arguments

#### Scenario: All unit kinds receive profile flags

- GIVEN native target units, host units, Cargo-derived units, integration-test units, and dev-dependency lib units
- WHEN Mantle builds each rustc argument list
- THEN each list MUST carry the resolved profile codegen flags
- AND no argument builder MAY rely on rustc defaults for these settings

### Requirement: Profile-aware unit identity

r[rust_package_planning.profile_unit_identity] Mantle MUST include the resolved profile in rustc metadata identity and artifact identity so units that differ only by profile cannot collide.

#### Scenario: Dev and release units of one crate stay separate

- GIVEN the same package, target, features, and source digest planned under `dev` and under `release`
- WHEN Mantle computes the rustc metadata disambiguator for both units
- THEN the two disambiguators MUST differ
- AND local result reuse MUST NOT treat one profile's artifact as valid for the other profile

### Requirement: Deterministic profile deviations are recorded

r[rust_package_planning.profile_determinism_policy] Mantle MUST state its deterministic deviations from Cargo profile defaults as explicit receipt-bound policy instead of silent behavior.

#### Scenario: Incremental compilation stays disabled

- GIVEN any profile, including `dev` where Cargo defaults incremental to true
- WHEN Mantle plans or executes a Rust unit
- THEN Mantle MUST not enable incremental compilation
- AND the receipt MUST record incremental compilation as explicitly disabled

#### Scenario: Unsupported profile settings fail closed

- GIVEN a request for `lto`, `panic`, `rpath`, `strip`, or `split-debuginfo` values that differ from rustc defaults
- WHEN Mantle evaluates the request
- THEN Mantle MUST emit a deterministic unsupported-profile-surface blocker
- AND Mantle MUST NOT silently apply a different value
