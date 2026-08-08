# Rust Package Planning Delta

## ADDED Requirements

### Requirement: Built-in build-override defaults for host units

r[rust_package_planning.profile_build_override_defaults] Mantle MUST derive build-script and proc-macro profile environment from Cargo's built-in build-override defaults rather than from the active target profile.

#### Scenario: Release build scripts are not optimized

- GIVEN a native custom-build or proc-macro host unit under the `release` or `bench` profile
- WHEN Mantle derives the child build-script environment
- THEN Mantle MUST set `OPT_LEVEL=0` and `DEBUG=false`
- AND the derivation MUST come from the pure resolver, not from ambient process environment

#### Scenario: Dev build scripts omit debug info by default

- GIVEN a native custom-build host unit under the `dev` or `test` profile
- WHEN Mantle derives the child build-script environment
- THEN Mantle MUST set `OPT_LEVEL=0` and `DEBUG=false`
- AND `NUM_JOBS` MUST remain the deterministic value `1`

### Requirement: Build-override scope and dual-use handling

r[rust_package_planning.profile_build_override_scope] Mantle MUST apply build-override defaults to build scripts, proc macros, and their host-compiled dependencies, and MUST handle dual-use packages explicitly.

#### Scenario: Build-script dependencies use build-override settings

- GIVEN a dependency compiled only as a host unit for build scripts or proc macros
- WHEN Mantle plans that host-dependency unit
- THEN its profile environment MUST use the build-override defaults

#### Scenario: Dual-use package is recorded, not silently split

- GIVEN a package that is both a normal target dependency and a build dependency built once
- WHEN Mantle derives its profile environment
- THEN the `DEBUG` value MUST follow the target profile's debug setting
- AND the receipt MUST record the dual-use decision instead of claiming Cargo's two-build behavior

#### Scenario: Old divergent values are rejected

- GIVEN a derivation that sets `OPT_LEVEL=3` for a release build script or `DEBUG=true` for a dev build script without a dual-use record
- WHEN build-override validation runs
- THEN validation MUST fail with a deterministic parity blocker
