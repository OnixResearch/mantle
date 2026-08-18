## ADDED Requirements

### Requirement: Native build-script target cfg environment

r[rust_package_planning.native_build_script_target_cfg_env] Mantle MUST provide bounded Cargo-compatible target cfg environment variables when running native build scripts.

#### Scenario: Target architecture reaches build-script runtime

GIVEN Mantle runs a native custom-build host unit for active target `x86_64-unknown-linux-gnu`
WHEN Mantle derives the child build-script environment
THEN Mantle MUST set `CARGO_CFG_TARGET_ARCH=x86_64`.
AND Mantle MUST derive the value from the selected target triple rather than ambient process environment.

#### Scenario: Common target cfg fields are deterministic

GIVEN Mantle runs a native custom-build host unit for a supported target triple
WHEN Mantle derives target cfg environment variables
THEN Mantle MUST set deterministic values for target vendor, os, env, family, endian, and pointer width.
AND Mantle MUST use empty strings for known-empty Cargo fields such as target env on triples without an env component.

#### Scenario: Build-script target cfg frontier moves

GIVEN topology execution currently runs `aws-lc-sys` and blocks on missing `CARGO_CFG_TARGET_ARCH`
WHEN target cfg env binding is applied
THEN self-probe verification MUST show that this missing `CARGO_CFG_TARGET_ARCH` blocker no longer stops `aws-lc-sys` build-script execution.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
