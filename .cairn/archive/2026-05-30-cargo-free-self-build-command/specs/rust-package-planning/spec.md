## ADDED Requirements

### Requirement: Cargo-free self-build command

r[rust_package_planning.cargo_free_self_build_command] Mantle MUST provide a first-class bounded command that builds a Mantle binary through native Cargo-free Rust topology execution without invoking Cargo as planner or build orchestrator.

#### Scenario: self-build command uses the native Cargo-free topology rail

GIVEN an operator launches `mantle self-build --cargo-free --out <dir>` from a supported Mantle source root
WHEN Mantle performs the build
THEN it MUST execute `rust-plan --no-cargo-oracle --execute-topology` with an explicit execution output root
AND it MUST select exactly one successful `mantle` binary unit from the topology receipt.

#### Scenario: self-build command forbids Cargo

GIVEN the Cargo-free self-build command runs
WHEN planning, topology execution, rustc, or build-script execution attempts to invoke `cargo`
THEN a failing Cargo guard MUST record the attempt
AND the command MUST fail without claiming a Cargo-free build.

#### Scenario: self-build command emits audit evidence

GIVEN the Cargo-free self-build command completes or blocks
WHEN the output directory is inspected
THEN it MUST contain the copied Mantle binary when successful, the full topology receipt, command stderr, status code, smoke output, Cargo guard status, source digest, binary BLAKE3 digest, and a machine-readable summary.

#### Scenario: self-build command remains bounded

GIVEN the Cargo-free self-build command reports success
WHEN the summary is inspected
THEN it MUST state that the result is not Crunch bootstrap, release reproducibility, source-built compiler/toolchain closure, or full Cargo compatibility evidence.
