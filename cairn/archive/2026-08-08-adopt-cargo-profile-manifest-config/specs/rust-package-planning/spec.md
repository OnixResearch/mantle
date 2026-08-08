# Rust Package Planning Delta

## ADDED Requirements

### Requirement: Root manifest profile authority

r[rust_package_planning.profile_root_manifest_authority] Mantle MUST treat only the workspace root manifest `[profile]` table as authoritative profile configuration.

#### Scenario: Root profile table is applied

- GIVEN a workspace root manifest with a `[profile]` table
- WHEN Mantle plans Rust units for that workspace
- THEN the resolved profile settings MUST come from that table layered over built-in defaults

#### Scenario: Dependency profile settings are ignored

- GIVEN a dependency manifest that contains a `[profile]` table
- WHEN Mantle plans Rust units
- THEN Mantle MUST ignore those settings
- AND Mantle MUST record a non-claim that dependency profile settings were present and ignored

### Requirement: Custom profiles with inheritance

r[rust_package_planning.profile_custom_inheritance] Mantle MUST support custom profiles that declare a required `inherits` key and resolve each unset setting from the inherited base.

#### Scenario: Custom profile inherits unset settings

- GIVEN `[profile.release-lto]` with `inherits = "release"` and no other supported setting changed
- WHEN Mantle resolves `release-lto`
- THEN every unset setting MUST equal the `release` built-in default
- AND the profile MUST be selectable with `--profile release-lto`

#### Scenario: Invalid custom profile fails closed

- GIVEN a custom profile with a missing `inherits` key, an inheritance cycle, or an unknown setting key
- WHEN Mantle resolves the profile table
- THEN planning MUST fail with a deterministic invalid-profile blocker before unit execution

### Requirement: Profile override precedence

r[rust_package_planning.profile_package_overrides] Mantle MUST support named-package, wildcard, and build-override profile overrides with Cargo's first-match precedence and forbidden-setting rules.

#### Scenario: Precedence ladder resolves one value

- GIVEN a unit whose package matches a named override, the `"*"` wildcard, and the active profile
- WHEN Mantle resolves one setting
- THEN the named package override MUST win over the wildcard, the wildcard MUST win over build-override and profile settings, and build-override MUST win over profile settings for host units

#### Scenario: Wildcard excludes workspace members

- GIVEN a workspace member package and a `"*"` override
- WHEN Mantle resolves the member's profile
- THEN the `"*"` override MUST NOT apply to the workspace member

#### Scenario: Forbidden override settings are rejected

- GIVEN an override table that sets `panic`, `lto`, or `rpath`
- WHEN Mantle parses the root profile table
- THEN parsing MUST fail with a deterministic forbidden-override blocker

### Requirement: Profile selection rules

r[rust_package_planning.profile_selection] Mantle MUST select profiles with Cargo's documented command defaults and flag equivalences.

#### Scenario: Command defaults match Cargo

- GIVEN build-like, test, and bench planning requests with no explicit profile flag
- WHEN Mantle selects the profile
- THEN build-like requests MUST use `dev`, test requests MUST use `test`, and bench requests MUST use `bench`

#### Scenario: Release flag and explicit profile agree

- GIVEN `--release` and `--profile release`
- WHEN Mantle selects the profile for the same command
- THEN both inputs MUST select the same resolved profile
- AND a profile name that does not resolve MUST fail closed before execution
