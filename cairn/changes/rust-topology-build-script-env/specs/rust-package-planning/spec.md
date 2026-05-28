# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Native build-script execution environment

r[rust_package_planning.native_build_script_execution_env] Mantle MUST execute native Rust build scripts with a bounded deterministic Cargo-like environment derived from the selected unit and invocation.

#### Scenario: Build script receives deterministic tool and target environment

GIVEN a native custom-build unit is executed for metadata
WHEN Mantle launches the compiled build-script executable
THEN Mantle MUST set `RUSTC` to the selected rustc path.
AND Mantle MUST set `HOST`, `TARGET`, and `PROFILE` deterministically from the active Rust topology invocation.
AND Mantle MUST preserve the cleared-env boundary except for the bounded PATH and explicit build-script variables.

#### Scenario: Build script runs from package root

GIVEN a native custom-build unit source path has a parent package directory
WHEN Mantle launches the compiled build-script executable
THEN Mantle MUST set the child current directory to that package directory.
AND Mantle MUST set `CARGO_MANIFEST_DIR` to the same package directory.

#### Scenario: Missing RUSTC topology blocker moves

GIVEN topology execution currently fails because a build script reports `Environment variable $RUSTC is not set during execution of build script`
WHEN build-script env binding is applied
THEN self-probe verification MUST show that this missing `$RUSTC` failure no longer blocks the first topology units.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
