## ADDED Requirements

### Requirement: Native build-script profile environment

r[rust_package_planning.native_build_script_profile_env] Mantle MUST provide bounded Cargo-compatible profile environment variables when running native build scripts.

#### Scenario: cc-rs profile opt level reaches build-script runtime

GIVEN Mantle runs a native custom-build host unit with the default dev profile
WHEN Mantle derives the child build-script environment
THEN Mantle MUST set `OPT_LEVEL=0`.
AND Mantle MUST derive the value from Mantle's selected profile rather than ambient process environment.

#### Scenario: Debug and job profile fields are deterministic

GIVEN Mantle runs a native custom-build host unit with a supported profile
WHEN Mantle derives profile environment variables
THEN Mantle MUST set deterministic `DEBUG` and `NUM_JOBS` values.
AND Mantle MUST use bounded profile defaults rather than reading ambient Cargo state.

#### Scenario: Profile env frontier moves

GIVEN topology execution currently runs `aws-lc-sys` and blocks inside cc-rs on missing `OPT_LEVEL`
WHEN profile env binding is applied
THEN self-probe verification MUST show that this missing `OPT_LEVEL` blocker no longer stops `aws-lc-sys` build-script execution.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
