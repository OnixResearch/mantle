# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Native build-script package name environment

r[rust_package_planning.native_build_script_package_name_env] Mantle MUST set native build-script `CARGO_PKG_NAME` from the manifest package name rather than the build-script target name.

#### Scenario: Build script receives manifest package name

GIVEN a native package manifest name contains a hyphen
AND Mantle executes that package's custom-build unit
WHEN Mantle derives the build-script child environment
THEN Mantle MUST set `CARGO_PKG_NAME` to the manifest package name with hyphen spelling preserved.
AND Mantle MUST NOT derive `CARGO_PKG_NAME` from `build-script-build` or other target names.

#### Scenario: Legacy unit fallback remains deterministic

GIVEN a build-script unit lacks explicit package-name env data
WHEN Mantle derives the build-script child environment
THEN Mantle MAY fall back to the deterministic crate-style target name.
AND Mantle MUST keep that fallback bounded to units without package-derived name data.

#### Scenario: Package-name fix preserves current topology frontier

GIVEN topology execution currently advances through successful build-script metadata runs after build-script env binding
WHEN package-name env binding is applied
THEN self-probe verification MUST show that the topology frontier does not regress to the old missing `$RUSTC` blocker.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
