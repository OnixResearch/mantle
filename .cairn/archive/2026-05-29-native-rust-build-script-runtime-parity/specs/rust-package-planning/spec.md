## ADDED Requirements

### Requirement: Native build-script runtime parity

r[rust_package_planning.native_build_script_runtime] Mantle MUST execute supported Rust build scripts with an explicit Cargo-compatible environment and typed metadata receipt model.

#### Scenario: build-script environment is declared

GIVEN a selected custom-build host unit runs under Mantle
WHEN Mantle launches the build script
THEN the runtime environment MUST include required package, target cfg, profile, manifest links, and `OUT_DIR` values derived from native planning facts
AND it MUST NOT inherit undeclared ambient host variables.

#### Scenario: build-script metadata is typed

GIVEN a build script writes Cargo metadata lines
WHEN Mantle captures build-script output
THEN it MUST parse supported rustc cfg, rustc env, link lib, link search, rerun, and links metadata into typed receipt fields.

#### Scenario: malformed or unsupported metadata fails closed

GIVEN a build script emits malformed or unsupported metadata
WHEN Mantle parses build-script output
THEN it MUST emit a deterministic build-script-metadata blocker before the dependent target unit runs.
