## ADDED Requirements

### Requirement: Native build-script package metadata environment

r[rust_package_planning.native_build_script_package_metadata_env] Mantle MUST provide bounded Cargo package metadata environment variables when compiling and running native Rust units.

#### Scenario: Build-script compile-time package version is available

GIVEN a native package manifest resolves to package version `0.39.1`
WHEN Mantle compiles that package's custom-build host unit with direct rustc
THEN Mantle MUST set `CARGO_PKG_VERSION=0.39.1` in the rustc environment.
AND Mantle MUST set deterministic version component env vars for major, minor, patch, and pre-release.

#### Scenario: Optional package metadata defaults are deterministic

GIVEN a native package manifest omits optional package metadata such as description, homepage, license, license-file, repository, readme, rust-version, or authors
WHEN Mantle derives package metadata environment variables
THEN Mantle MUST set those bounded `CARGO_PKG_*` variables to empty strings rather than reading ambient process state.

#### Scenario: Build-script runtime package metadata matches compile-time data

GIVEN Mantle executes a compiled custom-build host unit
WHEN Mantle derives the child build-script environment
THEN Mantle MUST pass through the bounded `CARGO_PKG_*` package metadata variables from the derivation environment.
AND Mantle MUST continue to preserve deterministic fallback behavior only for legacy units that lack package-derived name data.

#### Scenario: Package metadata frontier moves

GIVEN topology execution currently blocks while compiling `aws-lc-sys` with missing `CARGO_PKG_VERSION`
WHEN package metadata env binding is applied
THEN self-probe verification MUST show that this missing `CARGO_PKG_VERSION` blocker no longer stops `aws-lc-sys` build-script compilation.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
