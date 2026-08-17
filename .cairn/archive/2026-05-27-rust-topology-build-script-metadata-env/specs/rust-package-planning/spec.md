## ADDED Requirements

### Requirement: Native linked build-script metadata environment

r[rust_package_planning.native_linked_build_script_metadata_env] Mantle MUST propagate bounded linked dependency build-script metadata into dependent build-script environments using Cargo-compatible `DEP_*` variables.

#### Scenario: Package-level build path is planned

GIVEN a native package manifest declares `[package] build = "builder/main.rs"`
WHEN Mantle plans native package targets
THEN Mantle MUST include a custom-build target for that package-level build script.
AND Mantle MUST use the deterministic Cargo-compatible custom-build target name `build-script-build`.

#### Scenario: Build-script metadata captures safe custom keys

GIVEN a build script emits safe custom metadata lines such as `cargo:include=/path/include`
WHEN Mantle parses build-script stdout
THEN Mantle MUST record the metadata key and value deterministically.
AND Mantle MUST continue to reject malformed rustc metadata before execution proceeds.

#### Scenario: Linked dependency metadata reaches dependent build script

GIVEN a package has a normal dependency whose manifest declares `links = "aws_lc_0_39_1"`
AND that linked dependency build script emits `cargo:include=/path/include`
WHEN Mantle runs the dependent package's build script
THEN Mantle MUST set `DEP_AWS_LC_0_39_1_INCLUDE=/path/include` in the child environment.
AND Mantle MUST keep the environment bounded to derived `DEP_*` variables plus the existing build-script variables.

#### Scenario: Linked dependency build script runs before dependent build script

GIVEN a dependent build script needs linked dependency metadata
WHEN Mantle orders the combined native topology
THEN Mantle MUST schedule the linked dependency's custom-build host unit before the dependent build script.
AND Mantle MUST report a deterministic blocker if no linked dependency metadata producer exists.

#### Scenario: Metadata frontier moves

GIVEN topology execution currently blocks in `aws-lc-rs` with `missing DEP_AWS_LC_ include`
WHEN linked build-script metadata propagation is applied
THEN self-probe verification MUST show that this missing `DEP_AWS_LC_` blocker no longer stops `aws-lc-rs` build-script execution.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
